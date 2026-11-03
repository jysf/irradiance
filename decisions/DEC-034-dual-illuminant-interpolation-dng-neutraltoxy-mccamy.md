---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-034
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-02
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# Governs SPEC-034's extension of the SPEC-033 colour transform. Does not touch
# the per-pixel apply, the committed C, the demosaic, or DEC-030's metadata oracle.
affected_scope:
  - src/color.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-034-dual-illuminant-interpolation-chromatic-adaptation.md

tags:
  - colour
  - colour-transform
  - dual-illuminant
  - chromatic-adaptation
  - cct
  - mccamy
  - dng
  - determinism
  - spec-034
  - stage-008
---

# DEC-034: Dual-illuminant interpolation uses the DNG reciprocal-temperature NeutralToXY method with a McCamy CCT

## Decision

STAGE-008's scene-adaptive colour matrix (SPEC-034, extending SPEC-033) is built by
**interpolating the two DNG calibrations** at the scene's colour temperature, following
the Adobe DNG **`NeutralToXY` reciprocal-temperature** model:

- The scene interpolation weight `g ∈ [0, 1]` is found from `AsShotNeutral` by a **fixed
  3-iteration** `NeutralToXY` loop (dng_sdk's iteration count): interpolate
  `CM(g) = (1−g)·ColorMatrix1 + g·ColorMatrix2`, map the camera neutral through
  `inv(CM(g))` to an XYZ white, take its `xy`, get its CCT, and re-derive `g` by
  **reciprocal temperature (mired)** between the two `CalibrationIlluminant` CCTs
  (`g = clamp((1/T1 − 1/T)/(1/T1 − 1/T2), 0, 1)`).
- **CCT from chromaticity is McCamy's cubic** (`n = (x−0.3320)/(0.1858−y)`;
  `T = 449·n³ + 3525·n² + 6823.3·n + 5520.33`) — a **polynomial**, so the whole weight
  computation is `f64` with `+ − × ÷` only (no transcendental), hence byte-identical
  across platforms per DEC-033. CCTs for the `CalibrationIlluminant` EXIF-LightSource codes
  come from a **committed table**.
- `ColorMatrix` (and `ForwardMatrix` when present at both illuminants) are **linearly
  interpolated** by `g`; `cam2xyz` is then built with SPEC-033's per-path structure
  (`FM(g)·D`, else `inv(CM(g))·D`).
- A **single calibration** (no `ColorMatrix2`, a missing/unknown `CalibrationIlluminant`,
  or `T1 == T2`) **degenerates byte-identically to SPEC-033** (illuminant 1 only). The
  computation runs **once per image**, off the per-pixel path.

Two things are **deferred**, not adopted here: the ColorMatrix-path scene-white→D50
chromatic-adaptation CAT (SPEC-033's implicit-D50 treatment is kept; SPEC-035's ΔE judges
whether the CAT is worth its own spec), and `CameraCalibration` / `AnalogBalance` (identity,
DEC-033).

## Context

SPEC-033 (DEC-033) built the camera→XYZ matrix from the **first** illuminant only. A DNG
is calibrated at **two** illuminants (`CalibrationIlluminant1/2`, with `ColorMatrix1/2` and
usually `ForwardMatrix1/2`); the colour-correct matrix for a shot is interpolated between
them at the scene temperature. The scene temperature is not stored — it is derived from
`AsShotNeutral`, and that derivation is circular (the neutral's chromaticity needs a matrix,
the matrix needs the temperature, the temperature needs the chromaticity), which the DNG
spec resolves by a short fixed-point iteration (`NeutralToXY`). Getting a *defensible*
interpolation now matters because the bit-exact oracle can only prove the implementation
matches the documented method; whether the colour is *right* is SPEC-035's ΔE — so the
method must be the recognised DNG one, not an ad-hoc guess.

McCamy's 1992 cubic is the standard closed-form CCT-from-chromaticity used across the RAW
ecosystem; crucially for this project it is a **polynomial**, so it keeps the cross-platform
determinism DEC-033 established (the OETF table, the f64 basic-op matrix path) without a
transcendental or a Planckian-locus table.

## Alternatives Considered

- **A: Full dng_sdk fidelity now — the ColorMatrix-path scene-white→D50 Bradford CAT and
  `CameraCalibration`/`AnalogBalance` too.**
  - Rejected for SPEC-034. The general CAT (arbitrary white → D50) needs the Bradford
    cone-response matrix, its inverse, and a per-white diagonal — a second block of
    committed constants and a CAT builder — and `CameraCalibration` means parsing more
    tags. That is an [L] blow-up for a spec named "interpolation," and the ForwardMatrix
    path (Canon) is already D50-correct, so the CAT only moves the `ColorMatrix`-only
    (Nikon) render. Deferred to be driven by SPEC-035's measured ΔE, not guessed at.
- **B (chosen): Interpolation-only, DNG `NeutralToXY` + McCamy, keeping SPEC-033's per-path
  structure, with a byte-identical single-illuminant fallback.**
  - Faithful to the DNG interpolation model, bounded to [M] by reusing SPEC-033's matrix
    infra, deterministic, and non-regressing (single-calibration files unchanged). The one
    remaining approximation (the CM-path implicit D50) is explicit and measurable.
- **C: Planckian-locus or Robertson CCT instead of McCamy.**
  - Rejected: both pull in a transcendental or a committed locus/isotherm table and more
    code, for accuracy that McCamy already gives well within the tolerance SPEC-035 will
    set. McCamy keeps the path polynomial and the determinism argument trivial.
- **D: No interpolation — keep SPEC-033's single-illuminant matrix.**
  - Rejected: it is wrong for any scene far from `CalibrationIlluminant1` (a daylight shot
    through a tungsten calibration), which is most of them. This is the whole point of the
    spec.

## Consequences

- **Positive.** The matrix is scene-adaptive by the recognised DNG method; dual-calibration
  DNGs render materially more accurately across illuminants. Deterministic (f64 basic ops +
  a polynomial CCT, once per image) and cross-platform per DEC-033. Reuses SPEC-033's matrix
  infra and oracle harness, so it stays [M]. The single-illuminant fallback is
  byte-identical to SPEC-033 — a built-in regression guard.
- **Negative — the honest cost.** McCamy is an approximation (good to a few K near the
  Planckian locus, looser far from it) — acceptable because the interpolation weight is a
  smooth blend and SPEC-035's ΔE is the real accuracy gate. The deferred ColorMatrix-path
  CAT means the Nikon (`ColorMatrix`-only) render stays approximate until a follow-on; the
  bit-exact oracle cannot catch that (by design — the ΔE will). The `ILLUMINANT_CCT` table
  covers the common EXIF LightSource codes; an unknown code falls back to single-illuminant
  rather than guessing a temperature.

## Validation

Right if: SPEC-034 ships interpolation whose weight matches a hand-computed DNG
`NeutralToXY` value to the bit, is `f64`-deterministic, degenerates byte-identically to
SPEC-033 on single-calibration input, and has a red-proof that goes red on a perturbed
`ColorMatrix2`. Revisit if: SPEC-035's ColorChecker ΔE shows the deferred ColorMatrix-path
white-adaptation CAT (or a more accurate CCT than McCamy, or `CameraCalibration`) is needed
to hit tolerance — each becomes its own spec citing this DEC; or if a corpus file carries a
`CalibrationIlluminant` the table does not cover and the single-illuminant fallback proves
too coarse (then extend the table).

## References

- Extends `SPEC-033` / `DEC-033` (the f64 colour-transform pipeline this interpolates
  within) and `DEC-030` (the rationals the matrices are read from). Feeds `SPEC-035` (ΔE,
  which judges the deferred CAT). Reuses SPEC-033's `mat_mul` / `inv` / `wb_diagonal` /
  committed `C`.
- Standards: Adobe DNG Specification (the dual-illuminant `NeutralToXY` interpolation and
  reciprocal-temperature weighting — provenance class 1); McCamy, "Correlated color
  temperature as an explicit function of chromaticity coordinates," *Color Research &
  Application* 17(2), 1992 — the CCT cubic (class 2, a published paper). No implementation
  code read.
- Constraints: `no-panics-on-untrusted-input` (every divide in the weight path guarded →
  typed error / fallback, never a NaN in `dst`), `oracle-must-be-shown-red`.
