---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-033
  type: decision
  confidence: 0.85
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

# Governs the STAGE-008 colour transform (SPEC-033 and the specs that build on
# it). Does not touch the mono develop path, the demosaic, or DEC-030's
# colour-metadata oracle (which stays exact-rational).
affected_scope:
  - src/color.rs
  - src/lib.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-033-colour-transform-core-first-colour-develop-wb.md

tags:
  - colour
  - colour-transform
  - determinism
  - floating-point
  - srgb
  - bradford
  - dng
  - spec-033
  - stage-008
---

# DEC-033: The colour transform is a committed-constant f64 matrix pipeline — not exact rational

## Decision

STAGE-008's colour transform (SPEC-033: white balance → camera→XYZ → XYZ→linear-sRGB)
is computed in **`f64` using only the basic operations `+ − × ÷`**, with the one
irrational stage — XYZ(D50)→linear-sRGB, which folds the **Bradford D50→D65** chromatic
adaptation into the **IEC 61966-2-1 linear-sRGB-from-XYZ** matrix — held as a
**committed `f64` constant** `C` in `src/color.rs`. The configurable stages come from
the sensor's own exact rationals (`AsShotNeutral`, `ForwardMatrix1`, else the cofactor
inverse of `ColorMatrix1`), folded with `C` into **one 3×3 matrix `M` built once per
image** and applied per pixel: `out_i = Σ_j M[i][j]·cam_j`, then clamped to
`[0, 65535]` and rounded half-up to `u16`. Because full-scale `u16` means `1.0`
(`DEC-018`), the apply is scale-free. The sRGB **OETF** is not computed here at all — it
stays the committed lookup table of `DEC-025` (`tone`), which this transform feeds with a
**linear**-sRGB image.

Two simplifications are adopted and documented: (1) **single illuminant** — SPEC-033 uses
`ColorMatrix1`/`ForwardMatrix1` only, with no scene-white interpolation and no
illuminant-white adaptation on the `ColorMatrix` path (SPEC-034 adds the scene-adaptive
selection); (2) **`AnalogBalance = CameraCalibration = Identity`** — these DNG tags are
not parsed, so the reference-neutral reduces to `AsShotNeutral`.

## Context

The project is precision-obsessed for good reasons: the mono develop path and the
demosaic are bit-exact integer pipelines, the OETF is a committed table so no
transcendental runs at runtime (`DEC-025`: "determinism follows from the committed
integer table, not from matching a runtime float"), and the colour-**metadata** oracle
(`DEC-030`) compares parsed rationals to `exiftool` via `i128` cross-multiply with **no
float tolerance**. A new colour **transform** has to declare its arithmetic, because
"bit-exact on analytic fixtures" and "byte-identical on macOS and Linux" are stage
requirements.

The transform is unavoidably irrational: the sRGB primaries (IEC 61966-2-1) and the
Bradford cone-response matrix are irrational real numbers. There is no exact rational
representation of `C` to compare against — unlike `DEC-030`, where both sides were the
same rational and equality *had* to be exact. So the question is not "rational vs float
for an exact comparison"; it is "how do we make an inherently-real computation
reproducible and verifiable." IEEE-754 `f64` `+ − × ÷` are correctly-rounded and
therefore byte-identical across the two targets **provided** no fused-multiply-add and no
transcendental enters the path (Rust emits neither unless asked). `tone::to_linear_f32_into`
already ships a runtime `f32` division on exactly this basis.

## Alternatives Considered

- **A: Full exact-rational / fixed-point pipeline (the `DEC-030` style), committing `C`
  as scaled integers.**
  - Rejected for SPEC-033. `C` is irrational, so a fixed-point commit is itself an
    approximation — it buys *by-construction* cross-platform identity that IEEE `f64`
    basic ops already give, at the cost of an `i64`/`i128` matrix pipeline, a 3×3 rational
    inverse, and a generator for the committed constant. That is the [L] blow-up the
    frame warned about, for no colorimetric gain (the ΔE oracle, not the arithmetic,
    judges correctness). Revisit only if a measured cross-platform divergence ever
    appears (none is expected from basic ops).
- **B (chosen): `f64` basic-op pipeline; commit the irrational constant; OETF stays the
  table.**
  - The committed `C` is the direct analog of `DEC-025`'s committed OETF table: the
    irrational runs once, offline, into committed bytes an implementation reproduces
    exactly. Determinism comes from committed data + IEEE basic ops. Keeps SPEC-033 at
    [M], reuses `tone`/`demosaic`/`irr`.
- **C: `f32` instead of `f64`.**
  - Rejected: `f32` through a 9-term matrix accumulates visible quantization before the
    `u16` quantize, and costs nothing to avoid. `f64` throughout, quantize once at the
    end.
- **D: Apply the OETF in `color` too (skip the `tone` table).**
  - Rejected: re-introduces a runtime transcendental the project deliberately removed
    (`DEC-025`). `color` produces linear-sRGB; `tone` encodes. Clean layering.

## Consequences

- **Positive.** SPEC-033 stays [M] and reuses the STAGE-007 harness. The colour math is
  bit-exact on analytic fixtures (an injected clean `M` is hand-computable; the builder is
  checked by independent re-derivation to the bit) with a red-proof, and reproducible
  across platforms by the same IEEE basis `tone` already relies on. The committed `S`/`B`
  constants are auditable against published standards (provenance **class 1**).
- **Positive.** The OETF stays the one committed table; there is exactly one place a
  transcendental ever ran (the table generator), unchanged.
- **Negative — the honest cost.** "Bit-exact" here means *the implementation matches a
  committed constant and an independent re-derivation bit-for-bit*, not *exact rational*
  as in `DEC-030`. Whether the colour is colorimetrically **right** is a separate
  question answered only by `SPEC-035`'s ColorChecker ΔE. The single-illuminant and
  identity-`AnalogBalance`/`CameraCalibration` simplifications make the `ColorMatrix`
  (Nikon) render approximate until `SPEC-034`; the bit-exact oracle does not catch that —
  by design, the ΔE does.

## Validation

Right if: SPEC-033 ships `color::to_srgb_linear_into` / `develop_color_into` producing
linear-sRGB `u16`, with the committed `S`/`B` constants, verified bit-exact on analytic
fixtures + a red-proof, byte-identical on macOS and Linux, feeding `tone`'s OETF.
Revisit if: a measured cross-platform divergence appears on the transform path (then pin
down the offending op or move to fixed-point per Alternative A); or if `SPEC-034`/`SPEC-035`
show the single-illuminant / identity-calibration simplifications cost more ΔE than
acceptable (then parse `AnalogBalance`/`CameraCalibration` and/or add the illuminant
adaptation — each its own change citing this DEC).

## References

- Governs `SPEC-033` (this spec) and the STAGE-008 specs that build on the transform
  (`SPEC-034` dual-illuminant interpolation, `SPEC-035` ΔE). Builds on `DEC-018`
  (full-scale `u16`, the clamp), `DEC-025` (the OETF is a committed table; `color`
  produces linear), `DEC-016` (caller-owned buffers), and contrasts with `DEC-030`
  (exact-rational colour-metadata oracle — unchanged).
- Standards (provenance class 1): Adobe DNG Specification (the camera→XYZ colour model,
  `ForwardMatrix`/`ColorMatrix`/`AsShotNeutral`); IEC 61966-2-1 (sRGB primaries + OETF);
  the Bradford chromatic-adaptation transform. No implementation code read.
- Constraints: `no-panics-on-untrusted-input` (den-0 rationals, singular `ColorMatrix`,
  off-gamut mixes are typed errors / clamped, never panics), `oracle-must-be-shown-red`.
