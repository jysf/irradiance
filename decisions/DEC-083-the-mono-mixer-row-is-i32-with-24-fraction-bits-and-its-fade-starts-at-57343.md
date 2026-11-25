---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-083
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-006
repo:
  id: irradiance

created_at: 2026-11-25
supersedes: null
superseded_by: null
status: proposed
deciders: [claude]

affected_scope:
  - src/color.rs
  - src/lib.rs
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md

tags:
  - mixer
  - fixed-point
  - clip-rule
  - arithmetic
  - stage-031
---

# DEC-083: the mono mixer's row is `i32` with 24 fraction bits, rounded half up, and its fade of negative weights starts at 57343

## Decision

`DEC-079` §3 left three numbers to SPEC-092's design: the fixed-point format of the
folded row `v = wᵀ·M`, the fade threshold `τ`, and what the caller's weights may be.
They are:

1. **Weights.** Three `Rational`s `[w_R, w_G, w_B]` on linear sRGB. Each has a
   positive denominator and a non-negative numerator, and the three sum to exactly
   one, checked in exact integer arithmetic (`i128` cross-multiplication). Each is
   converted once to `f64` as `f64::from(num) / f64::from(den)`.
2. **The `f64` row.** `v_i = (w_R·M[0][i] + w_G·M[1][i]) + w_B·M[2][i]` for camera
   channel `i`, left to right, IEEE basic operations only, no `mul_add` (`DEC-033`).
   `M` is `build_camera_to_srgb_linear` unchanged.
3. **The fixed-point row.** `q_i = ⌊v_i·2^24 + 0.5⌋` as `i32` (24 fraction bits,
   round half up, the `DEC-018` rule). `v_i·2^24` is exact (a power-of-two scale),
   so the only rounding is the one `⌊· + 0.5⌋`. A `v_i` that is not finite, or whose
   `v_i·2^24 + 0.5` lies outside `[−2^31, 2^31)` (about `|v_i| ≥ 128`), is
   `Error::MixRowOutOfRange { channel }`.
4. **The fade threshold.** `T = 57343 = 65535 − 2^13`, so `τ = T/65535 ≈ 0.87500`
   and the fade width is `D = 2^13 = 8192` codes.
5. **The per-pixel step**, on the camera triple `x` (after levels and `GainMap`s,
   before white balance, `u16` with white at `65535`):
   `P = Σ_{q_i ≥ 0} q_i·x_i`, `N = Σ_{q_i < 0} q_i·x_i`, `m = max_i x_i`,
   `u = min(65535 − m, 8192)`, `F = clamp(⌊(P·8192 + N·u + 2^36) / 2^37⌋, 0, 65535)`,
   all in `i64`. `u = (1 − s)·D` exactly, where
   `s = clamp((m/65535 − τ)/(1 − τ), 0, 1)` is `DEC-079`'s fade, so the fade needs no
   division and no rounding before the last step.

## Context

`DEC-079` (accepted) fixed the approach: weights on linear sRGB folded into the camera
matrix, an integer per-pixel step, and the fade of negative weights near clip, which
onyx accepted on the conditions that it has no parameter, is bit-identical to no fade
when no weight is negative, and has its threshold documented exactly. It asked for a
scale that keeps a one-code error bound on a 16-bit input, and for `τ ≥ 0.5` (the
monotonicity proof's condition).

A design-time probe on 2026-11-25 (a throwaway unit test, not committed; its source and
output are in the PROJ-006 session notes) ran the five held colour files through the
real fused traversal and the real `build_camera_to_srgb_linear`, with luminance
`[2126, 7152, 722]/10000`, an illustrative red `[4/5, 1/5, 0]`, ortho `[0, 1/2, 1/2]`
and the three one-hot weights, at three candidate thresholds:

| `T` (`τ`) | Neutral grey reaches white at (rows with a negative entry) | Luminance, Canon / Nikon | Peak slope in the fade band | Pixels the fade changes |
|---|---|---|---|---|
| 61439 (0.9375) | 94.1% to 99.5% of raw white | 96.1% / 96.0% | up to 20.8 output codes per camera code | ≤ 4.41% |
| **57343 (0.875)** | **88.9% to 99.5%** | **94.3% / 94.1%** | **up to 10.9** | **≤ 4.66%** |
| 49151 (0.75) | 80.4% to 99.5% | 92.7% / 92.4% | up to 5.9 | ≤ 5.16% |

On a 300 × 300 grid of colour directions, each ramped in exposure past clipping with
each camera channel saturating at `65535`, the fade was monotone on every ray at every
threshold, for every row with a negative entry. The plain fold (no fade) was not: for
example 7,199 of 90,000 rays on the Canon's luminance row and 3,050 on the Nikon's
darken as they brighten. The largest `|v_i|` on any held file was 3.72 (the Canon).

## Alternatives Considered

- **Scale.** 2^20 fraction bits would bound the row's error at 0.094 codes; 2^24 bounds
  it at `3·65535/2^25 ≈ 0.0059` codes and still fits `i32` up to `|v| < 128`, 34 times
  the largest held value. More bits would need `i64` coefficients for no visible gain.
- **Rounding.** Round half to even or half away from zero would be just as
  deterministic; half up is `DEC-018`'s rule everywhere else in the crate, so it is the
  one a reader expects.
- **`τ = 0.9375`.** Grey reaches white later (better), but the fade band is 4096 codes,
  so its slope reaches 20.8 output codes per camera code on the Canon's green row: on a
  14-bit file that is a step of about 80 output codes per raw code in the top 6%, the
  posterisation onyx's review of "white once any channel clips" objected to.
- **`τ = 0.75`.** The gentlest slope, but grey reaches white as early as 80% of raw
  white and the fade changes highlights from three quarters of white up.
- **A non-power-of-two threshold** (DEC-079's probe used 0.9). It would need an integer
  division per pixel; with `D = 2^13` the whole step is multiplies, adds and one shift.
- **Negative weights.** Allowed by the arithmetic (the fade's monotonicity depends on
  `v`, not `w`), but a weight set with negative entries is a channel mixer, not a
  filter, and it can push `|v|` past the row's range. Refusing them now and allowing
  them later is an additive change; the reverse is not.
- **Weights checked in `f64`.** Refuses valid weights: `2/10 + 7/10 + 1/10` sums to
  `0.9999999999999999` in `f64`.

## Consequences

- **Positive:** every per-pixel operation is exact integer arithmetic, so the mixed
  plane is the same on every platform once the row is; the row is three `i32`s onyx can
  read, store and compare.
- **Positive:** with no negative `q_i`, `N = 0` and
  `⌊(P·2^13 + 2^36)/2^37⌋ = ⌊(P + 2^23)/2^24⌋`, which is the plain fold rounded half up,
  bit for bit, as onyx asked.
- **Positive:** the error against the exact real-valued fade on the `f64` row is at most
  `0.5 + 3·65535/2^25 ≈ 0.506` codes, within `DEC-079` §3's one-code bound.
- **Negative:** on the Canon and Nikon a neutral grey reaches white at about 94% of raw
  white under luminance weights, and at 88.9% under the Canon's one-hot green row.
  `DEC-079` recorded 92–95% at `τ = 0.9` and onyx accepted that cost.
- **Negative:** a negative weight is refused (`Error::InvalidMixWeights`) until a later
  decision allows it.
- **Neutral:** a file whose matrix would put a row entry past `|v| ≈ 128` is refused
  (`Error::MixRowOutOfRange`); no held file is within a factor of 34 of it.

## Overflow (why `i64` is enough)

`|q_i| ≤ 2^31`, `x_i ≤ 65535 < 2^16`, so `|P|, |N| < 3·2^47 < 2^48.6`. With `u ≤ 2^13`,
`|P·2^13|` and `|N·u|` are below `2^61.6`, have opposite signs, and adding `2^36` keeps
the total below `2^62`. No intermediate can overflow `i64`.

## Validation

SPEC-092's acceptance criteria: the per-file golden rows; the luminance identity against
SPEC-091's unclamped `f32` plane within `0.5 + 3·65535/2^25` plus that plane's own cast
error; bit-identity with the plain fold on every row with no negative entry; a zero
count of decreasing steps on the saturating-ray grid; the extreme-value overflow case
against an `i128` oracle; and the committed row and plane hashes on both CI
determinism legs. Revisit `τ` if a colour Leica file (STAGE-032) shows a visible kink
in the fade band.

## References

- Related specs: SPEC-092 (this decision's spec), SPEC-091, SPEC-093
- Related decisions: DEC-079 (the approach and the fade), DEC-018 (u16 full scale,
  round half up), DEC-033 (f64 basic operations), DEC-002 (determinism)
- Discussions: onyx's design reply (2026-11-25, recorded in the PROJ-006 brief)
