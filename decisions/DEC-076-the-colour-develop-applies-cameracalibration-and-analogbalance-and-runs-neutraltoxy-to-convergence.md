---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-076
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-22
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/color.rs
  - decisions/DEC-034-dual-illuminant-interpolation-dng-neutraltoxy-mccamy.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-085-apply-cameracalibration-and-analogbalance-on-the-colour-deve.md

tags:
  - colour
  - calibration
  - develop
  - stage-027
---

# DEC-076: the colour develop applies CameraCalibration and AnalogBalance, and runs NeutralToXY to convergence

## Decision

### 1. What is read

From IFD0, under DEC-012's contract (absent is `None`; a malformed tag is `None` and is
recorded in `malformed_tags`):

- `CameraCalibration1` and `CameraCalibration2` (50723, 50724), as rationals;
- `AnalogBalance` (50727), as rationals;
- `CameraCalibrationSignature` and `ProfileCalibrationSignature` (50931, 50932), as
  bytes (`ASCII` or `BYTE`), cut at the first NUL.

They are new public fields on the `#[non_exhaustive]` `Sensor` (DEC-049), so the change is
additive.

### 2. The signature rule

`CameraCalibration1/2` apply only when the camera signature equals the profile signature.
An absent signature counts as the empty string, so two absent signatures match. When they
do not match, both calibrations are the identity.

`AnalogBalance` is not gated. Absent, it is `(1, 1, 1)`.

The DNG specification states the rule: the calibration tags "should only be used" when
`CameraCalibrationSignature` exactly matches the profile's `ProfileCalibrationSignature`.
Adobe DNG SDK 1.7.1 implements it as a string equality in which absent is empty
(`dng_color_spec.cpp`). The STAGE-027 outline's wording, "absent or matching", was looser;
this is the rule.

### 3. What makes an applied calibration unusable

Only a calibration the develop will use is checked:

- `CameraCalibration1` whenever the rule applies.
- `CameraCalibration2` only when the rule applies **and** the dual-illuminant path runs.
  The single-illuminant path never reads it, as in the SDK.
- Either calibration absent is the identity for its illuminant.

A used calibration that is not 9 rationals with non-zero denominators is
`Error::MissingColorCalibration`. So is an `AnalogBalance` that is not 3 rationals with
positive numerators and denominators, a singular product, or a non-positive reference
neutral on the ForwardMatrix path (§4). This is the same error and the same treatment an
unusable `ColorMatrix` gets today, so no variant is added.

> **Amended by DEC-078 §7.2 (2026-11-23, SPEC-089).** "Only a calibration the develop
> will use is checked" is reversed. Every **present** `CameraCalibration1/2` is checked,
> used or not, and one that is not 9 rationals with non-zero denominators is never
> silently the identity. On a parsed file the reader records it in `malformed_tags` and
> the develop refuses it as `Error::MalformedTag` (DEC-078 §7.1). On a hand-built
> `Sensor`, the colour code refuses it as `Error::MissingColorCalibration`. Which
> calibration is *applied* is unchanged: `CameraCalibration1` whenever the rule applies,
> and `CameraCalibration2` only when the dual-illuminant path also runs.

### 4. The transform

`g` keeps DEC-034's meaning, the weight of illuminant 2.

- **Fold per illuminant:** `X_i = AB · CC_i · CM_i`.
- **The ColorMatrix path** interpolates the folded products,
  `X(g) = (1 − g)·X_1 + g·X_2`. It is then today's structure with `inv(X(g))` in place of
  `inv(CM(g))`:
  - `W = inv(X(g)) · AsShotNeutral`;
  - `cam2xyz = CA(W → D50) · inv(X(g))`.
- **The ForwardMatrix path** interpolates the calibration on its own,
  `K(g) = AB · ((1 − g)·CC_1 + g·CC_2)`. It takes the reference neutral
  `R = inv(K(g)) · AsShotNeutral`, then
  `cam2xyz = FM(g) · diag(1/R) · inv(K(g))`.
- **Single illuminant:** `X = AB · CC_1 · CM_1` and `K = AB · CC_1`.
- **Identity is byte-identical.** When `AB · CC` is exactly the identity, the output is
  bit-identical to the transform without calibration at the same `g`. The ForwardMatrix
  path keeps DEC-033's `D` from the rationals there.

This is the DNG specification's camera model, Chapter 6: `XYZtoCamera = AB · CC · CM`,
and `CameraToXYZ_D50 = FM · D · inv(AB · CC)` with `D` mapping the reference neutral to
`(1, 1, 1)`.

The specification describes `CM` and `CC` as each interpolated. The SDK instead
interpolates the folded `AB · CC_i · CM_i` (`fColorMatrix1 = fAnalogBalance *
fCameraCalibration1 * fColorMatrix1`, then the lerp). The two differ only when both the
calibration and the matrix change between illuminants. On the Pixel 7 Pro the difference
is 23 codes, against 6 for the SDK's form. This follows the SDK's form, because the SDK is
this stage's oracle and the specification does not say which product is lerped.

For a diagonal `K`, the ForwardMatrix formula equals DEC-033's `FM(g) · D` exactly in
real arithmetic. So on the held ForwardMatrix files (the Pixel 4a and the Canon) the
calibration acts only through `g`.

### 5. NeutralToXY runs to convergence (amends DEC-034's pass count)

The weight loop:
- starts at the D50 chromaticity `(0.3457, 0.3585)`;
- each pass: McCamy CCT of the current `xy`, then the mired `g`, then
  `next = xy(inv(X(g)) · AsShotNeutral)`;
- stops when `|Δx| + |Δy| < 1e-7`, taking `next`;
- runs at most 30 passes, and if the 30th has not converged, takes the midpoint of the
  last two `xy`.

The final `g` comes from the final `xy`. Every degenerate step still falls back to the
single-illuminant path. McCamy, the committed CCT table and DEC-033's basic-ops-only `f64`
are unchanged.

This is the SDK 1.7.1 loop (`dng_color_spec::NeutralToXY`: `kMaxPasses = 30`, the same
tolerance and the same midpoint). DEC-034 recorded a fixed 3 passes as "dng_sdk's iteration
count". That is not what SDK 1.7.1 does, and 3 passes do not converge on the held Pixels.

### 6. `DEVELOP_VERSION` goes from 7 to 8

Colour output changes on every held dual-illuminant colour file: the Pixel 7 Pro, 3a and
4a, the Canon 5D Mark III and the Nikon D300S. Mono output does not change.

## Context

**The gap.** The STAGE-027 audit (`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md`,
rank 1) found that the calibration tags were never read. Re-measured at SPEC-085's design
(2026-11-22, `exiftool -n`):
- **Pixel 7 Pro:** `CC1 = diag(0.8594, 1, 0.3408)` and `CC2 = diag(0.6162, 1, 0.5703)`,
  with no signatures, so the calibration applies. It has no ForwardMatrix.
- **Pixel 4a** (`com.adobe` both) and **Canon 5D Mark III** (`com.adobe` both): small
  diagonal calibrations, on the ForwardMatrix path.
- **Pixel 3a:** identity calibrations.
- **Nikon D300S:** none.
- Every file's `AnalogBalance` is `(1, 1, 1)` or absent.

**The probe (2026-11-22, on main at `3a3f308`).**
- **The harness.** A local C++ harness, about 35 lines, linked against the
  already-built Adobe DNG SDK 1.7.1 objects. It prints `dng_color_spec::CameraToPCS()`
  after `SetWhiteXY(NeutralToXY(CameraNeutral()))` for the default profile. That is the
  SDK's own camera→XYZ(D50) matrix, scaled so that `AsShotNeutral` maps to `Y = 1`, the
  same normalisation irradiance's `cam2xyz` has. HANDOFF-178 carries the source.
- **The model.** A Python model of today's `cam2xyz` and of this decision's was compared
  with it entry by entry. It was also compared in u16 output codes on 18 probe values:
  six camera colours at a quarter of full scale, through the committed `C = S · B`.

| File | Today | Calibration, 3 passes | Calibration, factors lerped, 30 passes | **This decision** |
|---|---|---|---|---|
| Pixel 7 Pro | 0.1598 / 1160 | 0.0171 / 148 | 0.0024 / 23 | **0.00019 / 6** |
| Pixel 3a | 0.0072 / 39 | 0.0072 / 39 | 0.00033 / 7 | **0.00033 / 7** |
| Pixel 4a | 0.0051 / 23 | 0.0002 / 1 | 0.00017 / 1 | **0.00017 / 1** |
| Canon 5D Mark III | 0.0011 / 3 | 0.0005 / 2 | 0.00047 / 2 | **0.00047 / 2** |
| Nikon D300S | 0.0002 / 5 | (no calibration) | 0.00016 / 5 | **0.00016 / 5** |

Each cell is the max |Δ| of the matrix entries, then the max |Δ| in codes, against the SDK.

**What the table shows:**
- **The 7 Pro needs all three parts:** the calibration, the convergent loop and the SDK's
  interpolation form.
- **The loop alone moves the 3a.** The loop converges in 4 passes on the Canon and the
  Nikon, 5 on the 4a, 16 on the 7 Pro and 19 on the 3a.
- **The calibration moves the 7 Pro's scene white.** Without it, the SDK's white is xy
  (0.328, 0.403), far off the daylight locus. With it, the white is (0.320, 0.333).
- **What remains is not this decision's.** The last 1 to 7 codes come from McCamy against
  the SDK's Robertson CCT, the SDK's `NormalizeForwardMatrix`, and its clipped Bradford.
  This decision leaves all three alone.

## Provenance

- **Class 1:** Adobe DNG Specification 1.7.1.0, Chapter 6, "Mapping Camera Color Space to
  CIE XYZ Space": the `AB · CC · CM` model, the ForwardMatrix form with the reference
  neutral, and the `CameraCalibrationSignature` / `ProfileCalibrationSignature` rule in
  the tag definitions.
- **Class 4:** Adobe DNG SDK 1.7.1, `dng_color_spec.cpp`, read and recorded with no code
  copied. It settled three points: the signature equality with absent as empty, the lerp
  of the folded products, and the convergent `NeutralToXY` loop. The SDK licence position
  is as recorded in the provenance ledger's SPEC-059 row. The probe harness links the SDK
  locally only. It is never vendored, built in CI or linked into the crate (DEC-069 §6).

## Alternatives Considered

- **Apply the calibration only on the ColorMatrix path.** Rejected. It would leave the
  ForwardMatrix path wrong for any non-diagonal calibration, and the specification
  defines both.
- **Interpolate `CC` and `CM` separately** (`AB · lerp(CC) · lerp(CM)`, one reading of
  the specification). Rejected. It is 23 codes off the SDK on the 7 Pro, against 6.
- **Keep 3 passes and add only the calibration.** Rejected. That leaves the 7 Pro at 148
  codes and the 3a at 39.
- **Refuse calibrations instead of applying them.** Rejected. The tags are the camera
  maker's correction and the SDK applies them; refusing would stop the Pixel 7 Pro from
  developing at all.
- **Emulate the SDK's Robertson CCT and Bradford clipping too.** Deferred. They would close
  the last few codes but replace DEC-034's McCamy cubic. That is its own decision if
  SPEC-035's ΔE oracle shows it matters.

## Consequences

- The colour develop honours the calibration it parses. Every held colour file's
  camera→sRGB transform matches the SDK's within 7 codes in the probe; SPEC-085 AC4
  gates it at ≤ 12.
- `DEVELOP_VERSION` is `"8"`. Colour pins move on the five held colour files; mono pins
  do not.
- DEC-034's interpolation stands, except its pass count, which §5 replaces.
- A wrong-type calibration tag still costs its field silently. SPEC-087 makes that a
  refusal.
- SPEC-035's ColorChecker ΔE now measures the develop, not a missing calibration.
