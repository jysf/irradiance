---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-054
  type: decision
  confidence: 0.7
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

created_at: 2026-11-11
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - tests/colorchecker_oracle.rs
  - tests/support/colorimetry.rs
  - tests/corpus/manifest.toml
  - tests/corpus/colorchecker/**
  - docs/oracle-contract.md
  - docs/provenance-ledger.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-035-colorchecker-e-oracle-gated-on-the-maintainer-s.md

tags:
  - colour
  - oracle
  - colorchecker
  - delta-e
---

# DEC-054: The ColorChecker ΔE2000 oracle contract (reference, metric, normalisation, pre-registered bound)

## Decision

The colour-accuracy oracle for PROJ-002 (SPEC-035) is defined as follows. Each choice
is fixed **before** the capture is seen, so the bound cannot be tuned to the data.

1. **Reference.** X-Rite's published ColorChecker Classic (24-patch) L\*a\*b\* data.
   Both formulations are embedded as test constants:
   - `ColorChecker24_Before_Nov2014`;
   - `ColorChecker24_After_Nov2014`.

   They are transcribed from the CGATS text files, pinned by SHA-256 in SPEC-035. The
   **production date printed on the back of the maintainer's chart** selects which set
   applies. The selection is recorded in the corpus manifest entry for the capture. It
   is not inferred from the brand, and it is not a default.

   The values are treated as **D50 / 2°**. The files themselves do not state an
   illuminant; BabelColor and X-Rite describe the data as L\*a\*b\* D50, which is the
   ICC/graphic-arts convention. This is an assumption with provenance, recorded here.

2. **Metric.** CIEDE2000 with kL = kC = kH = 1, computed in **L\*a\*b\*(D50)**. The
   implementation is test-only (`tests/support/colorimetry.rs`). It must reproduce all
   **34 pairs** of Sharma, Wu & Dalal (2005) to within 1e-4.

3. **Path from pipeline to Lab.**
   - Take each patch's mean from `develop_color_f32_into`. Its output is scene-linear
     sRGB primaries, D65 white, clamped to [0, 1].
   - Convert linear sRGB to XYZ(D65) with the IEC 61966-2-1 matrix.
   - Adapt XYZ(D65) to XYZ(D50) with **Bradford**, the matrix as published by Lindbloom.
   - Convert XYZ(D50) to Lab(D50).

   The test computes these matrices **independently** from the published coefficients.
   It does not import the library's private constants, so a constant error in the
   library cannot cancel out.

4. **Exposure normalisation: one scalar, pre-registered.**
   - Compute k = mean over the neutral patches **D4, C4, B4 and E4** of
     Y_ref / Y_measured. These are the middle neutrals: L\* ≈ 51, 67, 81 and 36. The
     white patch A4 and the black patch F4 are excluded because of clipping and the
     noise floor.
   - Multiply every patch's XYZ by k before converting to Lab.
   - No other fitting is allowed: no per-channel gain, no matrix fit, no white-balance
     correction.

   The pipeline's own white balance (`AsShotNeutral`) and matrices are what is under
   test.

5. **Bound, pre-registered.** Over all 24 patches:
   - **mean ΔE00 ≤ 6.0**;
   - **max ΔE00 ≤ 15.0**.

   The bound is set for a matrix-only pipeline: dual-illuminant ColorMatrix plus
   Bradford, with no HueSat/LookTable profile and no tone curve in the linear output.
   Such pipelines typically land at mean ΔE00 3–6 on this chart. The bound is
   deliberately not tight, because this oracle exists to catch a **wrong** pipeline,
   not to certify a calibrated one.

   **If the honest capture fails the bound, that is a finding, not a reason to move
   the bound.** The spec stops, and the maintainer decides between three options:
   a defect to fix, a documented limitation, or a re-registered bound with a written
   reason. Every per-patch ΔE is recorded in the handback whatever the outcome.

6. **Red-proof.** Two deliberate faults are injected by mutating a parsed `Sensor`'s
   public fields:
   - set `as_shot_neutral` to unity, which removes the white balance;
   - permute the rows of `color_matrix_1`/`color_matrix_2` (R↔B), giving a wrong
     camera→XYZ.

   Each must drive the **mean ΔE00 above 2 × the bound (> 12)** on the same capture.
   The honest run is the negative control and runs first (DEC-009). The tier-A synthetic
   chart proves the same thing without the capture, so CI also carries a red.

7. **Patch localisation is a committed sidecar, not detection.**
   `tests/corpus/colorchecker/<capture-id>.patches.toml` lists 24 patch centres in
   **develop-output coordinates** (after crop and orientation), plus one sampling
   half-width.
   - Patches are sampled as the mean over a square window. The window is at most 50 %
     of the patch side, so edges and the black surround stay out.
   - A patch whose window contains a channel at 1.0 (clipped) fails the test loudly.
   - So does a patch whose per-channel coefficient of variation exceeds 5 %, which
     indicates glare or a misplaced window.

   The orchestrator writes the sidecar from the actual capture, at capture time.
   Automatic chart detection is out of scope.

8. **Tiers.**
   - **Tier A** (committed, runs in CI):
     - the ΔE2000 Sharma vectors;
     - Lab and XYZ round-trips;
     - a synthetic chart: the reference Lab is pushed backwards to linear sRGB, then
       through the same patch-sampling and normalisation code. It must give
       ΔE ≈ 0 (≤ 0.01 after u16 quantisation), and the injected faults must turn it
       red.
   - **Tier B** (the capture, never committed per DEC-003): the real end-to-end
     measurement. When the file is absent it skips loudly and names the file.

## Context

PROJ-002's thesis names "colour checked by ColorChecker ΔE" as its fully independent
colour oracle. Everything else in the colour stack is checked bit-exact against
transcriptions or cross-checked against dnglab, and dnglab is single-sourced on
rawler. The brief records the risk that a loose ΔE could pass while the colour is
visibly wrong, and that tuning against the chart is an unbounded loop. Fixing the
metric, the normalisation and the bound before seeing the data answers both.

The capture camera is the Nikon D750 (`docs/conformance-matrix.md`). It writes only
compressed NEF, and this library does not decode Bayer `Compression 7`. The capture
therefore enters as an **uncompressed DNG made by Adobe DNG Converter**. Adobe's
converter is preferred over `dnglab convert` because it embeds Adobe's colour
matrices, which are independent of rawler; with dnglab, the matrices under test
would come from the same source as the cross-check.

## Alternatives Considered

- **Fit a 3×3 correction or per-channel gains before measuring.**
  - Rejected. That measures the fit, not the pipeline.
- **Set the bound after seeing the capture.**
  - Rejected. That is the theatre the brief warns about.
- **Use the reference's sRGB values or ΔE76.**
  - Rejected. The published sRGB values carry a known reliability caveat. ΔE76 is
    perceptually poor on saturated patches. Lab(D50) is what the chart data actually
    is.
- **Detect the chart automatically.**
  - Rejected. It is a computer-vision project of its own, and a hand-placed sidecar
    is checkable by eye.
- **Compare against dnglab's render of the same capture.**
  - Not the oracle, because dnglab is single-sourced. Reporting it in the handback for
    context is optional.

## Consequences

- **Positive:**
  - PROJ-002 gets the independent colour oracle its thesis names.
  - CI gets a tier-A red for the metric and normalisation code.
  - The bound cannot be gamed after the fact.
- **Negative:**
  - The bound is loose, by design. A pass means "not wrong", not "calibrated".
  - The bound and the D50 assumption are maintainer-visible judgement calls; the
    confidence of this decision is 0.7.
- **Follow-ups:**
  - If the capture fails, the likely suspect is the ColorMatrix-path scene-white→D50
    adaptation that SPEC-034 deferred to this oracle.
  - A tighter, calibrated bound would need HueSat/LookTable support. That is not in
    scope.
