---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-008                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-02
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The colour half of PROJ-002's thesis — turning STAGE-007's camera-space RGB into
    a correct, display-referred sRGB image via the DNG colour pipeline (white balance
    + illuminant-interpolated colour matrix + chromatic adaptation), and reaching
    ColorChecker ΔE, the one fully-independent colour oracle PROJ-001 structurally
    could not touch. When this ships a Bayer DNG becomes a recognisable, colour-accurate
    photo and PROJ-002 is complete.
  delivers:
    - "A Bayer DNG develops end to end to display-referred sRGB: demosaic (STAGE-007) → white balance (AsShotNeutral) → camera→XYZ (ForwardMatrix/ColorMatrix) → D50→D65 adaptation → linear-sRGB → sRGB OETF (tone), via `develop_color_into`"
    - "`irr` emits the full-colour sRGB render — the first recognisable colour picture"
    - "Colour accuracy checked by per-patch ColorChecker ΔE vs a measured chart (the fully-independent oracle) + a scoped dnglab-render cross-check; the colour math verified bit-exact on analytic fixtures"
  explicitly_does_not:
    - "Highlight reconstruction, denoise, or advanced rendering intents — an unbounded tuning loop (brief out-of-scope)"
    - "Profile / HSV tone maps beyond the sRGB OETF (DEC-025) — camera-profile curves are a later project"
    - "X-Trans / non-Bayer colour; a CLI (irr stays the dev/oracle binary)"
    - "Applying LinearizationTable / GainMap — the plane is STAGE-006's; colour consumes the normalised linear RGB"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-008: Colour transform: white balance, colour matrix, display encode, ColorChecker dE

## What This Stage Is

The final PROJ-002 stage: the colour transform that turns STAGE-007's demosaiced
**camera-space** RGB into a correct **display-referred sRGB** image. White balance
(`AsShotNeutral`) neutralises the camera white; the DNG colour matrices
(illuminant-interpolated `ColorMatrix` / `ForwardMatrix`) map camera → XYZ;
chromatic adaptation takes D50 → D65; XYZ → linear-sRGB; then `tone`'s sRGB OETF
(DEC-025) encodes display sRGB. The colour MATH is verified **bit-exact on analytic
fixtures** (a known camera-RGB pixel × a known matrix/WB = a known sRGB — the
STAGE-006/007 "verify where the math is exact" discipline), and — the capstone —
colour **accuracy** is verified by **ColorChecker ΔE** against a measured chart (the
one fully-independent colour oracle PROJ-001 could never reach), with a scoped
dnglab-render cross-check. When this ships, a Bayer DNG becomes a recognisable,
colour-accurate photo — the first actual picture — and **PROJ-002 is complete**.

## Why Now

STAGE-007 delivered the trusted camera-space RGB (demosaic, bit-exact-verified); the
colour transform is the only step between it and a real photo, and it is the **last
PROJ-002 stage** — it completes the thesis (a permissive pure-Rust crate develops a
Bayer DNG to *correct colour*, ΔE-verified). The colour math and the first two specs
need **no corpus** (analytic fixtures); only the ColorChecker ΔE capstone needs the
maintainer's chart shot — so the stage starts now and the independent accuracy oracle
lands when the shot arrives (maintainer decision, this frame: **build now,
ΔE-gated-later**).

## Success Criteria

- A Bayer DNG develops to a **display-referred sRGB** image via WB +
  illuminant-interpolated colour matrix + D50→D65 adaptation + the sRGB OETF; `irr`
  emits it — the first recognisable colour picture
- The colour math is verified **bit-exact** on analytic fixtures (known camera-RGB +
  known matrix/WB → known sRGB), with pinned, cross-platform-reproducible arithmetic
- Colour accuracy is within a **stated per-patch ΔE2000** tolerance of a measured
  **ColorChecker** (when the chart shot is available), and the ΔE oracle goes **red**
  on a deliberately wrong colour matrix or white balance
- A scoped **dnglab-render** cross-check (the DEC-005/DEC-026 develop oracle, now on
  the full colour render) holds on the corpus
- The colour-matrix math carries a **provenance-ledger entry**; the monochrome and
  demosaic paths stay green (colour is additive)

## Scope

### In scope
- **The colour transform:** white balance (`AsShotNeutral`); camera → XYZ
  (**`ForwardMatrix` when present** — Canon — **else invert `ColorMatrix`** — Nikon);
  **dual-illuminant interpolation** of the matrices between `CalibrationIlluminant1/2`
  by the scene white; **Bradford** adaptation D50 → D65; XYZ → linear-sRGB; reuse
  `tone` (DEC-025) for the sRGB OETF.
- **A 3-channel colour develop path** — new `develop_color_into` (or a widened
  signature) composing demosaic → WB → matrix → encode; the mono path stays intact.
- **`irr` colour output** — the full-colour sRGB render (first recognisable picture).
- **Oracles:** analytic bit-exact matrix/WB fixtures; the **ColorChecker ΔE** oracle
  (gated on the chart shot) + its red-proof; a scoped **dnglab-render** cross-check.
- **Provenance-ledger** entry for the colour-matrix math.

### Explicitly out of scope
- Highlight reconstruction, denoise, advanced rendering intents (brief out-of-scope).
- Profile / HSV tone maps beyond the sRGB OETF (DEC-025); camera-profile curves.
- **X-Trans / non-Bayer colour**; a CLI (irr stays the dev/oracle binary).
- Applying **LinearizationTable / GainMap** — the plane is STAGE-006's; if a corpus
  file carries one and it changes the plane, that is surfaced there, not here.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-008` promotes the `(not yet written)` lines below into real
specs at `cycle: frame`; the `[S]`/`[M]`/`[L]` tag carries onto `task.complexity`.

- [x] SPEC-033 (shipped 2026-11-02) — [M] **Colour-transform core + first colour develop**:
      WB (AsShotNeutral) + camera→XYZ (ForwardMatrix1 else inv(ColorMatrix1)) +
      committed Bradford-D50→D65·linear-sRGB constant (DEC-033); `color::to_srgb_linear_into`
      + `color::develop_color_into`; `irr color` → P6 PPM (first colour picture). Built
      **whole** (L-risk did not materialise), bit-exact analytic oracle + FM1 red-proof;
      verified ✅ on Canon (FM1) + Nikon inv(CM1). Build+verify 32.5M tok / $72.62.
- [x] SPEC-034 (shipped 2026-11-03) — [M] **Dual-illuminant interpolation**: `build_camera_to_srgb_linear`
      interpolates `ColorMatrix1/2` + `ForwardMatrix1/2` at the scene colour temperature (DNG
      `NeutralToXY` + McCamy CCT, DEC-034), once per image in f64; single-calibration byte-identical
      to SPEC-033. No public API change. Bit-exact analytic oracle + perturbed-CM2 red-proof;
      verified ✅ on real Canon (FM(g), g≈0.426) + Nikon (inv(CM(g)), g≈0.763). Build+verify 30.9M tok / $67.28.
      (The ColorMatrix-path scene-white→D50 CAT is deferred — SPEC-035's ΔE judges.)
- [ ] SPEC-035 (design — capture addendum pending; DEC-054) — [M] **ColorChecker ΔE oracle (GATED on the maintainer's
      ColorChecker shot)**: render the chart, measure per-patch **ΔE2000** vs the known
      ColorChecker reference (Lab/sRGB), with a red-proof (a wrong matrix/WB spikes ΔE).
      The fully-independent colour-accuracy capstone — the oracle PROJ-001 could not reach.

**Count:** 2 shipped / 1 active / 0 pending

> **Framing notes for review (Prompt 1c steps 3–4):**
> - **3 specs, one gated.** Core transform → illuminant interpolation → ColorChecker
>   ΔE. The first two need no corpus (analytic fixtures); the third is **blocked on
>   the maintainer's ColorChecker shot** (same camera, even light, no clipped patches).
>   Per this frame's decision, build the first two now; the ΔE capstone lands when the
>   chart arrives.
> - **The L-risk is the core spec.** The split (matrix math / `develop_color` + `irr`)
>   is pre-named so the build can take it without a re-frame.
> - **The ΔE oracle is the only FULLY-independent colour check** (brief risk #1): the
>   dnglab-render cross-check is single-sourced on rawler, so it is a sanity
>   cross-check, not the accuracy oracle.

## Design Notes

- **Verify where the math is exact; ΔE where it isn't.** Applying a known matrix/WB
  to a known camera-RGB pixel is exactly computable (pin f32/f64 + rounding), so the
  analytic fixtures are **bit-exact** on the MATH — the STAGE-006/007 discipline.
  Whether the colour is *right* end to end is a tolerance question only **ColorChecker
  ΔE** answers independently; the dnglab-render SSIMULACRA2 check (DEC-005/DEC-026, now
  on the colour render) is single-sourced on rawler (brief risk #1), so it is a
  cross-check, not the accuracy oracle.
- **DNG colour direction:** `ForwardMatrix` (camera→XYZ, D50) directly when present
  (Canon 5D-III has it); else invert `ColorMatrix` (XYZ→camera → camera→XYZ) (Nikon
  D300S has no ForwardMatrix — SPEC-026). `AsShotNeutral` is the camera neutral → WB
  scales camera-RGB per channel so neutral maps to grey.
- **Illuminant interpolation (spec 2):** `CalibrationIlluminant1/2` (LightSource
  codes) → CCT; interpolate the two matrices by the inverse-CCT of the scene white
  (from AsShotNeutral); Bradford-adapt D50→D65 for sRGB. Spec 1 uses a single
  illuminant to get a first image; spec 2 adds the interpolation + adaptation.
- **Representation:** `develop_color_into` writes linear-sRGB `u16` (DEC-018), then
  `tone::encode_srgb16_into` / `encode_srgb8_into` (DEC-025) apply the OETF — the same
  3-output-mode shape the mono develop uses; interleaved RGB per the demosaic buffer
  (DEC-016 / DEC-032).
- **Cross-platform:** the brief wants a byte-identical **plane** on macOS + Linux but
  colour only **within ΔE** on both — pin the colour arithmetic (fixed f32/f64 +
  rounding) so the render is reproducible, and treat any residual cross-platform ΔE as
  a measured, stated bound.
- **ColorChecker shot (maintainer):** same camera as the corpus (Nikon / Canon), even
  light, no clipped patches; the oracle needs the known reference (a Calibrite / X-Rite
  ColorChecker Classic 24-patch Lab/sRGB table). Spec 3 is gated on it.
- **Provenance-ledger** entry for the colour-matrix math (the DNG colour model is from
  the published Adobe DNG spec — not copyleft code), beside the demosaic entry.

## Dependencies

### Depends on
- **STAGE-007** (shipped) — camera-space RGB from `demosaic_into` (the colour input).
- **STAGE-006** (shipped) — the parsed colour metadata (`color_matrix_1/2`,
  `forward_matrix_1/2`, `as_shot_neutral`, `calibration_illuminant_1/2`, per-channel
  levels), all verified truthful.
- **PROJ-001** (shipped) — `tone` (DEC-025, the sRGB OETF + 3 output modes), the
  develop oracle (DEC-005 / DEC-026, reused as the dnglab cross-check), the `irr` harness.
- External: the Adobe DNG spec (colour-matrix model), `dnglab analyze --srgb` (render
  cross-check), a ColorChecker Lab/sRGB reference. **Corpus: the maintainer's
  ColorChecker shot — gates the ΔE spec (spec 3) only; the first two need none.**

### Enables
- **PROJ-002 complete** — a permissive pure-Rust crate develops a Bayer DNG to correct
  colour, ΔE-verified → the project ship (Prompt 1e).
- crustyimg's colour-camera RAW-develop path; the permissive **colour** RAW primitive
  the Rust ecosystem lacks (the adoption thesis).

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1d (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m — only if this stage was
  framed with `just frame-stage`. Counts outlines whose scope held from frame to
  ship. A low number means framing ran ahead of what was knowable; a high one
  means it's paying for itself.>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
