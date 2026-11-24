---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-079
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

created_at: 2026-11-24
supersedes: null
superseded_by: null
status: proposed
deciders: [jysf, claude]

affected_scope:
  - src/color.rs
  - src/demosaic.rs
  - src/develop.rs
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md

tags:
  - colour
  - mixer
  - working-space
  - arithmetic
  - stage-031
---

# DEC-079: mixer weights are defined in linear sRGB and applied folded into camera space

## Decision

PROJ-006's mono weights **mean** weights on linear sRGB (BT.709 primaries, D65, the space
`to_srgb_linear_into` already produces), but the mixer **applies** them as one row folded
into the camera-to-linear-sRGB matrix, `v = wᵀ · M`, on the white-balanced camera values,
so no clamped RGB intermediate ever feeds a mix. The per-pixel mixer arithmetic is
integer: `v` is built once per image in `f64` (DEC-033's basis) and quantised once to a
fixed-point row.

## Context

STAGE-031 (PROJ-006) needed the working space decided before the RGB step, the mixer
and the developed mix are designed. onyx (its PROJ-006 answers, 2026-11-23) prefers a
linear working space so one preset table means the same on every body, asked for the
white-balanced linear RGB plane as its own public step, wants clipping detected on raw
camera channels, and asked for fixed-point arithmetic. `docs/future-plans.md` had
assumed weights on white-balanced camera RGB.

A design-time probe (2026-11-24, a throwaway unit test, not committed) ran the existing
colour path over the five held colour files and measured, per pixel, the unclamped
`M · cam` before `quantize`:

| File | Luminance row expressed in WB camera RGB | linear sRGB < 0 | > white | Rec.2020 < 0 | Y from clamped plane off by > 1 code (max) |
|---|---|---|---|---|---|
| Canon 5D Mark III | [0.225, 0.886, −0.110] | 3.23% | 0.02% | 2.63% | 2.70% (30439) |
| Pixel 3a | [0.246, 0.694, 0.060] | 0.00% | 0.92% | 0.00% | 0.77% (4035) |
| Pixel 4a | [0.184, 0.760, 0.057] | 0.00% | 5.06% | 0.00% | 5.06% (9089) |
| Pixel 7 Pro | [0.217, 0.711, 0.072] | 0.00% | 0.01% | 0.00% | 0.01% (1391) |
| Nikon D300S | [0.369, 0.750, −0.119] | 0.19% | 0.01% | 0.00% | 0.19% (11761) |

Three facts follow. **(1)** The same luminance needs different camera-space weights on
every body, two of them with a negative blue weight, so camera-RGB presets would be
per-camera guesses. **(2)** The existing `u16` linear-sRGB plane clamps: up to 3.2% of
pixels have a negative channel (Canon) and up to 5.1% exceed white after white balance
(Pixel 4a), and a mix computed from that clamped plane is off by more than one code on
up to 5.1% of pixels, by as much as 30439 codes. **(3)** A wider space (Rec.2020) barely
helps the Canon's negatives (3.23% to 2.63%), so changing the space does not fix the
clamp; not clamping does.

## Alternatives Considered

- **Option A: weights on white-balanced camera RGB** (`docs/future-plans.md`)
  - What it is: the mix is `Σ wᵢ · camᵢ / asnᵢ`, cheapest, no matrix.
  - Why rejected: fact (1). A "Red filter" would look different on a Q3 and an M11, and
    every preset would need a table per body. onyx rejects it for the same reason.

- **Option B: weights on a materialised, clamped linear RGB plane**
  - What it is: the mixer reads the RGB step's `u16` output and mixes it.
  - Why rejected: fact (2). Saturated colours and white-balance overs are lost before
    the mix; a red filter on a deep-blue sky would read the clamped zero.

- **Option C: a wider working space (linear Rec.2020 or ProPhoto) for the weights**
  - What it is: define weights on wider primaries so fewer colours go negative.
  - Why rejected: fact (3); and with the fold (below) the space only names what the
    weights mean, it does not change what is clamped. Linear sRGB is the space irradiance
    already builds, cross-checks against Adobe's SDK, and documents; its luminance row is
    BT.709's `[0.2126, 0.7152, 0.0722]`. A later DEC can add a named space if a preset
    needs it, by folding a constant 3×3 in front of `M`.

- **Option D (chosen): linear-sRGB weights, applied folded into camera space**
  - What it is: `mono = v · cam` with `v = wᵀ · M`, `M` the existing
    camera-to-linear-sRGB matrix (white balance, calibration, illuminant interpolation
    and the D50-to-D65 adaptation all inside it). Mathematically the same as mixing the
    unclamped linear-sRGB plane; nothing is clamped until the single mono value is.
  - Why selected: presets mean the same on every body (onyx's ask), no gamut or
    white-balance clamp corrupts the mix (fact 2), and the clip mask stays meaningful
    because the mix is still a per-camera-channel sum, so a clipped raw channel's term
    can be found and treated (SPEC-092 designs the rule).

## Consequences

- **Positive:** one preset table for every body; the mix is exact where the clamped
  plane is not; raw-channel clip detection stays well defined; the per-pixel cost is
  three multiply-adds, cheaper than materialising RGB.
- **Positive:** the RGB step SPEC-091 exposes is the same space the weights are
  defined in, so onyx's colour reference view and its own candidate mixes share the
  mixer's meaning. Because its `u16` form clamps (fact 2), SPEC-091 also offers an
  unclamped `f32` form for candidate mixes and the quality study.
- **Negative:** the folded row `v` can have negative entries (the Canon and Nikon
  rows above), so "drop the clipped channel and renormalise" is not automatically
  monotone. SPEC-092 must pre-register a monotone clip rule against that.
- **Negative:** the existing colour path's `f64` matrix apply is not fixed-point.
  The existing outputs may not change (onyx's additions-only rule), so the RGB step
  keeps `f64` (DEC-033) and only the mixer's per-pixel step is integer.
- **Neutral:** a camera whose matrix cannot be built refuses the mix exactly as it
  refuses the colour develop (`MissingColorCalibration`, DEC-076 §3).

### Arithmetic (§3)

1. The matrix `M` is built exactly as today (`build_camera_to_srgb_linear`, DEC-033,
   DEC-034, DEC-076); the RGB step applies it per pixel in `f64` as today, so its `u16`
   form equals the existing composition bit for bit.
2. The mixer builds `v = wᵀ · M` once per image in `f64`, from the caller's rational
   weights, then quantises each entry once to a signed fixed-point integer (scale and
   width chosen at SPEC-092 design; the scale must keep a one-code error bound on a
   16-bit input). Every per-pixel step is integer multiply, add, round half up, clamp.
3. Determinism comes from the same two facts as DEC-033 (committed constants, IEEE basic
   operations only, no FMA) for the once-per-image build, plus exact integer arithmetic
   per pixel. The per-pixel oracle is therefore exact.

## Validation

- SPEC-091's equivalence oracle: the RGB step cropped and oriented equals
  `develop_color_into`'s output byte for byte on every held colour file.
- SPEC-092's oracle: with `w` = BT.709 luminance, the mono equals the luminance of the
  **unclamped** linear-sRGB plane to within the fixed-point bound, and differs from the
  clamped plane's luminance on the pixels the probe above found.
- Revisit if the Q3 or M11 (not yet in the corpus) shows a materially different clamp
  profile, or if a preset in STAGE-032 cannot be expressed on linear-sRGB primaries.

## References

- Related specs: SPEC-091, SPEC-092, SPEC-093 (STAGE-031)
- Related decisions: DEC-018 (u16 full scale), DEC-033 (f64 basic ops, committed
  constants), DEC-034 (illuminant interpolation), DEC-076 (calibration), DEC-002
  (determinism)
- Discussions: onyx's PROJ-006 answers (recorded in the PROJ-006 brief), PR 324, PR 325
