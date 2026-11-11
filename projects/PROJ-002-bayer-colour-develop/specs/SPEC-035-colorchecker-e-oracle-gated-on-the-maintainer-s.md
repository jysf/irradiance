---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-035
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: true                    # GATED on an external input: the maintainer's ColorChecker
                                   #   shot (same camera, even light, no clipped patches). Clear
                                   #   when the shot lands. depends_on also holds it behind 033/034.
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-008
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-054, DEC-003, DEC-009, DEC-030]
  constraints: [oracle-must-be-shown-red, provenance-recorded-per-algorithm, library-not-application, no-copyleft-dependencies]
  related_specs: [SPEC-033, SPEC-034]  # measures the full transform both deliver

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-033, SPEC-034]   # measures the full colour render; also externally GATED (see blocked)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-008's <capability>". Optional; null is acceptable.
value_link: >
  The fully-independent colour-accuracy oracle — per-patch ΔE2000 vs a measured
  ColorChecker — that PROJ-001 structurally could not reach; closes PROJ-002's thesis.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 22000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-035: ColorChecker ΔE2000 oracle (gated on the maintainer's capture)

> **Design status (2026-11-11).** Everything that does **not** depend on the capture is
> designed here, and DEC-054 pins it: the reference data, the metric, the
> normalisation, the pre-registered bound and the red-proof.
>
> Three items are **filled in a short capture addendum** (a design-cycle commit by the
> orchestrator) once the shot exists. They are marked ⏳ below:
> - the capture's manifest entry;
> - the patch sidecar;
> - which reference set applies.
>
> `blocked: true` stays until that addendum lands. The build is dispatched only after
> it. The tier-A half could be built earlier, but splitting the build costs more than
> waiting a day.

## Context

STAGE-008's capstone, and the colour-truth oracle in PROJ-002's thesis. SPEC-033 and
SPEC-034 verify the colour **math** bit-exact on analytic fixtures. The only
real-image cross-check is dnglab, which is single-sourced on rawler. A measured
ColorChecker is the one fully independent answer to whether the colour is right.

SPEC-034 explicitly deferred one question to this oracle: whether the ColorMatrix
path's scene-white→D50 adaptation is needed. The 0.2.0 CHANGELOG states "no
colour-accuracy claim" until this ships.

**Capture camera:** Nikon D750 (`docs/conformance-matrix.md`). It writes only
compressed NEF, and Bayer `Compression 7` is not decoded here, so the shot enters as
an **uncompressed DNG from Adobe DNG Converter** (DEC-054 explains why Adobe and not
dnglab).

**Design-time probes (2026-11-11):**
- **Reference files.** Downloaded and confirmed:
  - `https://babelcolor.com/index_htm_files/ColorChecker24_After_Nov2014.txt`
    - SHA-256 `2d110e3e8c33cb46fad920817efcf3efa504c2a9eb9cf60c9079b9c111557d29`
    - 24 sets, comma decimal separators, header dated 4/28/2015
  - `https://babelcolor.com/index_htm_files/ColorChecker24_Before_Nov2014.txt`
    - SHA-256 `0594f920f656331ca878e5a207173e22c80217dcaa00ae5d45013ccf7f86275f`
    - decimal points
  - X-Rite's own copies are the ZIPs linked from
    `https://www.xrite.com/service-support/new_color_specifications_for_colorchecker_sg_and_classic_charts`.
  - Samples are named by grid position **A1…F4**. The **letter is the chart column**
    (A…F, left to right with the neutral row at the bottom) and the **digit is the
    row** (1…4). So A1 = dark skin, A4 = white, F4 = black, and row 4 is the neutral
    ramp: A4 white, B4, C4, D4, E4, F4 black.
  - Spot values confirmed from the raw files:
    - After, A4 = 95.19 / −1.03 / 2.93
    - Before, A4 = 96.539 / −0.425 / 1.186
- **CIEDE2000 test data.**
  `https://hajim.rochester.edu/ece/sites/gsharma/ciede2000/dataNprograms/ciede2000testdata.txt`
  - SHA-256 `44aebb39107128328add54fbef5ac8ee89909e50508f448a1580adea2058a4b8`
  - 34 rows of `L1 a1 b1 L2 a2 b2 dE00`, with no header
  - Row 1 is `50.0000 2.6772 -79.7751 50.0000 0.0000 -82.7485 2.0425`.
  - The paper is Sharma, Wu & Dalal, *Color Res. Appl.* 30(1):21–30, 2005.
- **Output under test.** `color::develop_color_f32_into` (`src/color.rs:363`) returns
  scene-linear sRGB-primary values with a D65 white, clamped and quantised to u16
  steps, then divided by 65535. No OETF and no tone curve are applied. The crop and
  orientation are already applied, so sidecar coordinates are in this output's
  frame.
- `Sensor`'s colour fields are public and mutable on a parsed value, even though the
  type is `#[non_exhaustive]`, so the red-proof can mutate them:
  - `as_shot_neutral` (`src/ifd.rs:780`)
  - `color_matrix_1/2` (`:766/:768`)
  - `forward_matrix_1/2` (`:772/:775`)

## Goal

Render the maintainer's ColorChecker DNG through `develop_color_f32_into`. Report
per-patch **ΔE00 in Lab(D50)** against the chart's published reference. Assert the
**pre-registered bound** of DEC-054: mean ≤ 6.0 and max ≤ 15.0 over 24 patches. Prove
the oracle bites: removing the white balance or swapping the matrix rows pushes the
mean above 12. A committed tier-A synthetic chart carries the same red into CI.

## Design

**Follow DEC-054.** Don't re-litigate it: no fitting, no post-hoc bound, no automatic
chart detection, no sRGB reference values.

### Test-only code. Nothing is added to the library

- `tests/support/colorimetry.rs`:
  - `const` tables for both reference sets (24 × Lab, in A1…F4 order), each with a
    doc comment giving the source URL and SHA-256.
  - `ciede2000(lab1, lab2) -> f64` (kL = kC = kH = 1), following Sharma et al.'s
    implementation notes, including the hue-angle edge cases.
  - `xyz_d65_from_linear_srgb`: the IEC 61966-2-1 matrix.
  - `bradford_d65_to_d50`: built from the published Bradford cone matrix and the
    D65/D50 white points, **computed in the test, not imported from `src/`**.
  - `lab_d50_from_xyz`: the CIE formulas with ε = 216/24389 and κ = 24389/27.
  - The patch sampler: mean over a square window, plus the clip check and the
    coefficient-of-variation check.
  - The normaliser: scalar k over B4, C4, D4 and E4.
- All of it is f64 and test-only. It can use `unwrap`/`expect`/indexing, because the
  panic-free lints apply to `--lib` only.

### Tier A: committed, runs in CI, always on

`tests/colorchecker_oracle.rs`:

- **Sharma vectors.** Embed the 34 pairs. All must match to within 1e-4.
- **Round-trips.**
  - Lab → XYZ → Lab is identity within 1e-9 for the 24 references.
  - linear sRGB → XYZ(D65) → XYZ(D50) maps the D65 white to the D50 white within 1e-6.
- **Synthetic chart, control.**
  1. Push each reference Lab(D50) backwards: Lab → XYZ(D50) → inverse Bradford →
     linear sRGB.
  2. Paint 24 flat patches into a synthetic f32 RGB image, quantised to u16 steps
     exactly as the library does.
  3. Run the **same** sampler, normaliser and ΔE code.
  4. Assert mean ΔE ≤ 0.01. Use this exact value: any larger slack hides real
     defects.

  Some reference colours fall outside sRGB, and clamping them to [0, 1] adds error.
  The design accepts this: the measured synthetic mean is recorded in the handback,
  and the bound for the synthetic control is "≤ 0.01 for the in-gamut patches, all
  24 reported". The build must list which patches clamp. **Do not widen the bound to
  hide them.**
- **Synthetic red-proof.** Two faults on the synthetic image:
  - an R↔B channel swap;
  - an R gain of 0.5 (the white-balance analogue).

  Each must give mean ΔE > 12. The control runs first (DEC-009).

### Tier B: the capture, which skips loudly when absent

- **The corpus manifest entry** (`tests/corpus/manifest.toml`):
  - file name, camera, and SHA-256;
  - `colorchecker_reference = "before_nov2014" | "after_nov2014"`;
  - the path of the sidecar.

  ⏳ The orchestrator writes this in the capture addendum.
- **The sidecar** (`tests/corpus/colorchecker/<capture-id>.patches.toml`): 24 entries
  `{ id = "A1", x = …, y = … }` in develop-output pixel coordinates, plus
  `half_window = N`. ⏳ The orchestrator writes it in the capture addendum from a
  rendered preview (`irr color` → PPM → PNG), and checks it by overlay.
- **The test.**
  1. Parse the DNG.
  2. Run `develop_color_f32_into`.
  3. Sample, normalise, then compute ΔE for all 24 patches. **Print the full
     per-patch table in every run**, whether it passes or fails.
  4. Assert the DEC-054 bound.
- **The tier-B red-proof.**
  1. Run the honest pass first (the control).
  2. Clone the parsed `Sensor`, set `as_shot_neutral` to `[1/1, 1/1, 1/1]`, develop and
     measure. Assert mean > 12.
  3. Clone again, swap rows 0 and 2 of `color_matrix_1` **and** `color_matrix_2`. If
     the file carries `forward_matrix_*`, swap their columns 0 and 2 as well, so the
     fault is not bypassed by whichever path the library uses. Develop, measure, and
     assert mean > 12.
- **Skip.** If the file is absent, skip loudly, naming the file and
  `IRRADIANCE_CORPUS_DIR` (AGENTS.md §12 bar 4).

### Docs

- `docs/oracle-contract.md` gets a colour layer section summarising DEC-054.
- `docs/provenance-ledger.md` gets rows for:
  - CIEDE2000 (Sharma et al. 2005, a published formula, class 1);
  - the X-Rite reference data (published colorimetric data, class 1);
  - the Bradford and sRGB matrices (published, class 1).

  All of these are test-only; say so in each row.
- The `CHANGELOG.md` **"No colour-accuracy claim"** limitation is **replaced** by the
  measured result: mean and max ΔE00 on the named camera and chart. If the bound
  fails, the bullet is rewritten honestly; it is not deleted.
- `docs/conformance-matrix.md`: the D750 row gets the result.

## Capture guide (maintainer, for the shot)

- **Chart:** fills about 1/3–1/2 of the frame, square to the lens, flat. The Classic
  Mini is small, so get close. Each patch should be well over 100 px wide.
- **Light:** even and diffuse daylight or a single high-CRI source at about 45°, with
  no glare and no mixed light. Avoid direct sun hot-spots and coloured reflections
  from nearby surfaces.
- **White balance:** measure a PRE (preset manual) white balance off the chart's
  middle grey (D4), or use Direct sunlight in daylight. Avoid Auto. `AsShotNeutral` is
  the white balance under test.
- **Exposure:** base ISO, f/5.6–f/8, with the white patch A4 **not clipped**. Bracket
  0, −1/3 and −2/3 EV and keep all three. Shoot 14-bit lossless NEF.
- **Convert:** use Adobe DNG Converter, Preferences → Compatibility → Custom.
  **Uncheck "Use lossless compression"**, and leave "Linear (demosaiced)" off.
  Send the DNG and the date printed on the **back** of the chart.

## Inputs

- DEC-054, the stage file, SPEC-033 and SPEC-034 (the pipeline under test).
- `src/color.rs` (`develop_color_f32_into` `:363`); `src/ifd.rs` (colour fields
  `:766-785`).
- `tests/support/corpus.rs` and `tests/corpus/manifest.toml` (the tier-B
  skip-when-absent harness); the existing colour oracles as a pattern
  (`tests/color_oracle.rs`).
- The three source files above. The build re-downloads them and checks their
  SHA-256 before transcribing anything.

## Outputs

- **Created:**
  - `tests/support/colorimetry.rs`
  - `tests/colorchecker_oracle.rs`
  - `tests/corpus/colorchecker/<capture-id>.patches.toml` (⏳ addendum)
- **Modified:**
  - `tests/corpus/manifest.toml` (⏳ addendum entry)
  - `docs/oracle-contract.md`
  - `docs/provenance-ledger.md`
  - `docs/conformance-matrix.md`
  - `CHANGELOG.md`
- **No** change to `src/`, no new dependency, no `DEVELOP_VERSION` change. If the
  capture fails the bound, **no `src/` fix lands in this spec**; the fix becomes its
  own spec (DEC-054 §5).
- The DNG itself is never committed (DEC-003).

## Acceptance Criteria

- [ ] **Reference transcribed.** Both sets are embedded, the sources are re-downloaded
      and match the pinned SHA-256s, and every one of the 144 values matches its
      source. A test reparses the source text if it is present locally; otherwise the
      check is recorded in the handback.
- [ ] **ΔE2000 correct.** All 34 Sharma pairs are within 1e-4.
- [ ] **Conversions correct.** The Lab↔XYZ round-trip and the white-point mapping
      tests pass.
- [ ] **Synthetic control.** The in-gamut patches reach mean ΔE ≤ 0.01. The patches
      that clamp out of gamut are listed. All 24 values are printed.
- [ ] **Synthetic red.** The R↔B swap and the 0.5 R gain each give mean ΔE > 12,
      shown after the control.
- [ ] **Tier B honest run.** The 24-row per-patch table is printed, together with
      mean, max and the normalisation k. The DEC-054 bound is asserted. **If it
      fails, the build stops and reports instead of tuning anything.**
- [ ] **Tier B red-proof.** The unity-WB fault and the matrix-row-swap fault each give
      mean > 12, after the honest control.
- [ ] **Loud skip.** With the corpus dir unset, the tier-B tests print the missing
      file name and pass as skipped, not as silently green.
- [ ] **Docs.** The oracle-contract, provenance-ledger, conformance-matrix and
      CHANGELOG updates are in, and the CHANGELOG states the measured numbers.
- [ ] **Gates.** All green: `just test`, `lint-ci`, `fmt-check`, `validate`,
      `cost-audit`, `gate-audit`, `deny`, `deny-fuzz`. CI is observed green on the
      PR head.

## Failing Tests

Write these first. Each red must be observed for the stated reason.

- `ciede2000_matches_sharma_2005_all_34_pairs`: red until the formula exists.
- `lab_xyz_round_trip_identity`, `d65_white_maps_to_d50_white`
- `synthetic_chart_control_is_near_zero`
- `synthetic_chart_rb_swap_is_red`, `synthetic_chart_r_gain_is_red`
- `colorchecker_capture_within_pre_registered_bound` (tier B)
- `colorchecker_capture_unity_wb_is_red`, `colorchecker_capture_matrix_row_swap_is_red`
  (tier B)
- `colorchecker_skips_loudly_without_corpus`

No fuzz target is needed: this adds no input surface to the library (§12 bar 2
applies to parsers).

## Non-Goals

- Building or changing the colour transform. If the oracle finds a defect, the fix is
  a new spec.
- **Tuning to minimise ΔE**, or any fit (DEC-054 §4).
- Automatic chart detection; X-Trans; perceptual ΔE on non-chart scenes.
- HueSat/LookTable profile support, which a tighter "calibrated" bound would need.

## Notes for the Implementer

- The After file uses **comma** decimal separators and the Before file uses points.
  Transcribe carefully, and keep the order A1…F4.
- Sharma's notes list the hue-angle and mean-hue edge cases (h′ wrap, C′ = 0). The 34
  pairs exist to catch these; implement from the notes, not from memory.
- Compute the matrices in f64 from the published coefficients in the test. **Do not**
  import or copy `src/color.rs` constants: independence is the point.
- If the honest tier-B run fails the bound, record the table and stop. **Do not**
  change the bound, the normalisation or the patch map to make it pass.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
