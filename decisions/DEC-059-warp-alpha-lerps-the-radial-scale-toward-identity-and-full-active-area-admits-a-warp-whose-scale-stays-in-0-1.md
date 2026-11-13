---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-059
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-12
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/develop.rs
  - src/opcode.rs
  - src/warp.rs
  - src/lib.rs
  - fuzz/fuzz_targets/develop.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-064-warp-alpha-over-warprect-lerp-measure-the-backward-radial-sc.md

tags:
  - develop
  - options
  - warp
  - public-api
  - develop-version
  - stage-019
---

# DEC-059: warp_alpha lerps the radial scale toward identity, and FullActiveArea admits a warp whose lerped scale provably stays in [0, 1]

Amends DEC-058 §4. It narrows the blanket refusal of `FullActiveArea` with a warp into
a bounds rule. It does not supersede DEC-058.

## Decision

### 1. `WarpRect::lerp(alpha)`

`WarpRect::lerp(&self, alpha: f64) -> WarpRect` returns the warp whose backward radial
scale is `f_α(r) = (1 − α) + α·f(r)`:

- `kr0' = (1 − α) + α·kr0`;
- `kr1' = α·kr1`, `kr2' = α·kr2`, `kr3' = α·kr3`;
- `kt0' = α·kt0`, `kt1' = α·kt1`;
- `cx`, `cy` and `planes` are unchanged.

Because the displacement of a pixel is `(1 − f(r))·(x − c)`, the lerped displacement is
exactly `α` times the full one. `α = 1` returns the coefficients bit for bit (`1 − 1 = 0`,
`0 + kr0 = kr0`, `1·k = k`). `α = 0` returns `[1, 0, 0, 0]` (some zeros may be `-0.0`,
which compares equal), which `is_identity` already skips.

`lerp` is pure arithmetic and does not validate `alpha`. A value outside `[0, 1]`
extrapolates. The develop path validates instead (§2).

### 2. `DevelopOptions::warp_alpha`

- A new `pub warp_alpha: f64` field. The default is `1.0`, which is today's full warp, so
  under DEC-058 §2 and §5 it does not bump `DEVELOP_VERSION`.
- The develop applies `warp.lerp(options.warp_alpha)` in place of `warp`. An identity
  result (`α = 0`) skips the warp exactly as an identity opcode does today.
- **Domain:** `warp_alpha` must be finite and in `[0, 1]`. `-0.0` is accepted. Anything
  else is `Error::InvalidWarpAlpha { warp_alpha }`, `ErrorKind::CallerError`. It comes
  from `develop_with_into` and `output_dimensions_with`, checked first, under every
  framing, whether or not the file carries a warp. An out-of-domain argument is the
  caller's error whatever the file holds.
- **At `α = 0` the warp is not applied, so its tangential terms are not refused.** A
  file with non-zero `kt` develops warp-free at `α = 0`. The caller asked for no
  correction, and none is applied.

### 3. The bounds rule for `FullActiveArea`

Under `Framing::FullActiveArea`, a lerped warp that is not the identity is accepted
exactly when all of these hold:

- every coefficient of the lerped warp, and `cx` and `cy`, is finite;
- `0 ≤ cx ≤ 1` and `0 ≤ cy ≤ 1`, so the optical centre is inside the image;
- `0 ≤ f_α(s) ≤ 1` for every `s = r²` in `[0, 1]`.

`f_α` is a cubic in `s`. Its extrema on `[0, 1]` are taken at `s = 0`, `s = 1` and the
real roots of `f_α'(s) = kr1' + 2·kr2'·s + 3·kr3'·s²` that lie in `(0, 1)`. That is a
closed-form, deterministic computation with no sampling and no tolerance.

**Why this is sufficient.** `r` is normalised by `m`, the distance from the optical
centre to the farthest corner, so every pixel has `r ≤ 1`. The source of destination
pixel `x` is `c + f·(x − c)`. With `0 ≤ f ≤ 1` that is a convex combination of `x` and
`c`, and both lie in the image, so the source does too. No sample is clamped, and the
DEC-024 clamp never decides a visible pixel.

**It is sufficient, not necessary.** A lens with `f > 1` only near the centre would move
no sample out of the image, yet it is refused. Relaxing that later, for example with a
pixel-exact check, is additive (DEC-058 §4).

Otherwise the result is `Error::UnsupportedWarpedFullActiveArea`, which now also
carries `warp_alpha`, `scale_min` and `scale_max`. The variant is unreleased (it is only
in CHANGELOG `[Unreleased]`), so changing its fields breaks no released API.

The refusal keeps DEC-058 §4's position: after the buffer lengths, before `OpcodeList1`
is parsed, before the `FixBadPixelsConstant` copy and before any write to `dst`.

### 4. `StandardCrop` keeps today's behaviour at every `α`

The bounds rule applies only to `FullActiveArea`. Under `StandardCrop` a lerped warp is
applied as today: out-of-extent samples clamp to the edge (DEC-024), and nothing is
refused. Starting to refuse there would turn a result into a refusal for an existing
(input, options) pair, which DEC-058 §4 and §5 rule out. Whether clamped samples can
land inside a `DefaultCrop` is a separate question with no held file to answer it (§
Context).

## Context

`docs/future-plans.md` § PROJ-004 "Pin at design time" 1 required `f(r) ≤ 1` to be
measured on the Q2M, not assumed. It also required a lens with `f > 1` to get a typed
error, never an out-of-bounds read.

The SPEC-064 design probe (main at 0d8fa19) found the following. Exact cubic extrema
were cross-checked by a dense 10⁶-point sampling, and a pixel-exact count of the
destination pixels whose source falls outside the active area:

| File | α = 1 scale `[min, max]` | corner pull at α = 1 | out-of-extent pixels, α ∈ {.25, .5, .75, 1} |
|---|---|---|---|
| Q2M L1021223 | [0.899841061, 0.999251106] | 506.1 px | 0, 0, 0, 0 |
| Q2M L1026016 | [0.912971177, 0.999251106] | 439.7 px | 0, 0, 0, 0 |
| Q2M L1026192 | [0.919079836, 0.999251106] | 408.9 px | 0, 0, 0, 0 |
| Pixel 7 Pro (off-centre `cx`, `cy`) | [0.989696026, 0.999495730] | 1.2 px | 0, 0, 0, 0 |
| Pixel 3a | [0.980158687, **1.001529254**] | −3.9 px (outward) | **299, 323, 382, 439** (worst 3.09 px) |
| Pixel 4a | identity | 0 | 0 |

- **Every held Q2M frame passes at every α.** Its scale decreases monotonically from
  `kr0 = 0.999251106` at the centre to `f(1)` at the corner. `FullActiveArea` therefore
  becomes available on the requester's camera. The corner pull is now measured: 506.1 px
  on L1021223, against the unrecorded "503.8 px" future-plans warned about.
- **A real over-unity lens is held.** The Pixel 3a's scale exceeds 1 near the corners at
  every `α > 0`, and its pixel-exact count is non-zero at every `α > 0`. The analytic rule
  and the pixel-exact count agree on every held coefficient set. Pixel files do not
  develop today (an unrecognised mandatory opcode 9), so the Pixel 3a's coefficients
  enter as a hand-built fixture: a real-world polynomial, not an invented one.
- **The Pixel 7 Pro has an off-centre optical centre with `f ≤ 1`**, and its count is
  0. That supports the convex-combination argument for a centre that is not `0.5`.

## Alternatives Considered

- **Keep the blanket refusal (DEC-058 §4).** Rejected. The measurement shows it refuses
  every Q2M frame for no visible defect.
- **A pixel-exact check**, inverse-mapping every destination pixel before writing.
  Rejected for now. It costs a full pass over the active area before any output, and
  the analytic rule agrees with it on every held file. It stays the additive relaxation
  if a lens with `f > 1` only near the centre turns up.
- **Check only the corner (`s = 1`).** Rejected. A cubic can peak inside `(0, 1)`, so
  the corner alone is not a bound.
- **Check only `f ≤ 1`** (the future-plans wording). Rejected as incomplete. With
  `f < 0` or an optical centre outside the image, the convex-combination argument fails,
  so `0 ≤ f` and `cx`, `cy` in `[0, 1]` are part of the rule.
- **Check `f` once, at α = 1, and rely on `f ≤ 1 ⇒ f_α ≤ 1`.** Rejected. Checking the
  lerped scale itself is exact for the α actually requested. It never refuses more than
  the α = 1 check would, and `α = 0` needs no check because the identity skips the warp.
- **Refuse over-unity warps under `StandardCrop` too.** Rejected (§4). It would change an
  existing result into a refusal.
- **`warp_alpha` as a fixed-point integer** so options could be `Eq`. Rejected.
  future-plans names `lerp(alpha)` over `f64`, and DEC-058 §1 already withheld `Eq` for
  this field.

## Consequences

- `FullActiveArea` develops every held Q2M frame at every `α`. The requester gets an
  instrument view of all 8392×5632 photosites with any amount of lens correction.
- `DEVELOP_VERSION` stays `"4"`. The default `α = 1` leaves every existing
  (input, options) pair unchanged. The only pairs whose result changes are
  `FullActiveArea` with a warp in `[0, 1]`, which go from a refusal to a result, and
  DEC-058 §5 says relaxing a refusal does not bump.
- One new `Error` variant (`InvalidWarpAlpha`, `CallerError`). Three new fields on the
  unreleased `UnsupportedWarpedFullActiveArea`.
- `WarpRect` gains one public method, `lerp`.
