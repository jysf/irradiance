---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-034
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
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
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-034, DEC-033, DEC-030]  # interpolation method / colour-transform f64 pipeline / exact-rational colour metadata
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-033, SPEC-035]  # refines SPEC-033; feeds SPEC-035

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-033]             # refines the core transform's matrix selection

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-008's <capability>". Optional; null is acceptable.
value_link: >
  Refines STAGE-008 colour accuracy: selects the colour matrix for the actual scene
  illuminant (interpolating the DNG's two calibrations) and adapts D50→D65.

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
  tokens_estimate: 35000000  # ~24M build + ~11M verify; novel interpolation math (no reuse), but reuses color.rs matrix infra + the oracle harness
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-034: **Dual-illuminant interpolation + chromatic adaptation**:

## Context

[[SPEC-033]] builds the camera→XYZ matrix from the **first** illuminant's calibration
(`ForwardMatrix1`, else `inv(ColorMatrix1)`), ignoring the second. But a DNG carries
**two** calibrations — `ColorMatrix1/2` (and often `ForwardMatrix1/2`) at
`CalibrationIlluminant1/2` (e.g. StandardA ~2856 K and D65 ~6504 K) — and the
colour-correct matrix for a shot is the one **interpolated** between them at the scene's
own colour temperature, which the DNG model derives from `AsShotNeutral`. Using only
illuminant 1 skews colour for any scene far from it (a daylight shot rendered through a
tungsten calibration, say). This spec makes the matrix **scene-adaptive** — the DNG's
own chromatic-adaptation mechanism — sharpening accuracy ahead of [[SPEC-035]]'s ΔE.
(The fixed PCS→display step — Bradford **D50→D65** folded into SPEC-033's committed
constant `C` — is already present and unchanged; this spec adds the *scene-illuminant*
selection, not that fixed step.)

## Goal

Build the camera→XYZ matrix from the illuminant-interpolated calibration: find the scene
interpolation weight `g ∈ [0, 1]` from `AsShotNeutral` (the DNG reciprocal-temperature
`NeutralToXY` method), linearly interpolate `ColorMatrix1/2` (and `ForwardMatrix1/2` when
present) by `g`, and build `cam2xyz` from the interpolated matrices exactly as SPEC-033
does from illuminant 1 — degenerating to SPEC-033's single-illuminant result when only
one calibration is present. Verified **bit-exact** on analytic fixtures (known
matrices + illuminants + neutral → known `g` → known interpolated `cam2xyz`) with a
red-proof.

## Design

One change, localized to SPEC-033's `build_camera_to_srgb_linear` in `src/color.rs`:
where it uses `FM1` / `CM1` directly, it now uses an **interpolated** `FM(g)` / `CM(g)`.
The interpolation weight `g ∈ [0, 1]` is found **once per image**, in `f64` with `+ − × ÷`
only (no transcendental — so it stays byte-identical across platforms, `DEC-033` / the
new `DEC-034`). Public signatures (`to_srgb_linear_into`, `develop_color_into`) are
**unchanged** — this is an internal accuracy refinement; only the pixel values shift, and
only for dual-calibration files.

**Finding the weight — the DNG reciprocal-temperature `NeutralToXY` method.**
Preconditions for interpolation: `ColorMatrix1` **and** `ColorMatrix2` present and
well-shaped, and both `CalibrationIlluminant1/2` present with a known code. Otherwise →
**single-illuminant fallback** (below). Then:

- Illuminant CCTs from a committed EXIF-LightSource→CCT table `ILLUMINANT_CCT`
  (`17` StandardA → 2856, `21` D65 → 6504, `23` D50 → 5003, `20` D55 → 5503, `22` D75 →
  7504, `19` StandardC → 6774, `18` StandardB → 4874, `1` Daylight → 5500, `3`/`248`
  Tungsten → 2856, …; an **unknown** code → fallback). `T1 = cct(CalibrationIlluminant1)`,
  `T2 = cct(CalibrationIlluminant2)`.
- Iterate a fixed **N = 3** times (dng_sdk's count), starting `g = 0`:
  1. `CM(g) = (1−g)·CM1 + g·CM2` (element-wise lerp of the two 3×3).
  2. `white_xyz = inv(CM(g)) · (asn_R, asn_G, asn_B)` — the camera neutral's XYZ
     (`inv` is SPEC-033's cofactor inverse; singular → `MissingColorCalibration`).
  3. `xy = ( X/(X+Y+Z), Y/(X+Y+Z) )` (sum ≤ 0 → `MissingColorCalibration`, never a 0-divide).
  4. **McCamy CCT:** `n = (x − 0.3320) / (0.1858 − y)`;
     `T = 449·n³ + 3525·n² + 6823.3·n + 5520.33` (committed constants; `n³ = n·n·n`;
     a zero denominator `0.1858 − y` → fallback). Polynomial, so no transcendental.
  5. `g ← clamp( (1/T1 − 1/T) / (1/T1 − 1/T2), 0, 1 )` — reciprocal-temperature (mired)
     interpolation: `T = T1 → g = 0` (CM1), `T = T2 → g = 1` (CM2); `T1 == T2` → fallback.
- The converged `g` is the weight.

**Building `cam2xyz` from the interpolated matrices** (SPEC-033's per-path structure,
unchanged except the matrix):
- **ForwardMatrix path** (both `FM1` and `FM2` present): `FM(g) = (1−g)·FM1 + g·FM2`,
  `cam2xyz = FM(g) · D`, `D = diag(1/asn)` (SPEC-033's `wb_diagonal`). FM present at only
  one illuminant is treated as "no usable FM pair" → ColorMatrix path (documented).
- **ColorMatrix path:** `cam2xyz = inv(CM(g)) · D`. (SPEC-033's single-illuminant
  structure, now at the interpolated `CM(g)`; the scene-white→D50 adaptation CAT that
  would replace the implicit-D50 treatment is a **Non-Goal** here — see below.)
- Then `C · cam2xyz` with SPEC-033's committed `C = S · B` (unchanged).

**Single-illuminant fallback.** If the interpolation preconditions fail (no `CM2`, a
missing/unknown `CalibrationIlluminant`, `T1 == T2`, or a degenerate weight step), skip
weight-finding and build `cam2xyz` from illuminant 1 **exactly as SPEC-033 does** — so a
single-calibration sensor is **byte-identical** to SPEC-033 (a regression guard, and what
the shipped SPEC-033 fixtures assert). The fallback is normal, not an error.

**Determinism & cost.** All of the above is `f64` basic ops computed once per image, off
the per-pixel path; `apply_matrix_into` (SPEC-033) is untouched. `DEC-034` records the
method and why McCamy (deterministic polynomial) over Planckian/Robertson (transcendental
/ table lookup).

## Inputs

- **Files to read / modify:** `src/color.rs` — extend `build_camera_to_srgb_linear`;
  reuse `mat_mul`, `inv`, `wb_diagonal`, `matrix_from_rationals`, `rational_to_f64`, the
  committed `SRGB_FROM_XYZ_D65` / `BRADFORD_D50_TO_D65` and the `#[cfg(test)]` oracle
  harness. Add private helpers + one committed table (below).
- **Read:** `src/ifd.rs` — `Sensor.color_matrix_2`, `forward_matrix_2`,
  `calibration_illuminant_1`/`_2` (SHORT LightSource codes), `as_shot_neutral` (the
  neutral the weight is found from); `Rational`.
- **Decisions:** `DEC-034` (this spec), `DEC-033` (the f64 pipeline it extends),
  `DEC-030` (the rationals).
- **Standards (provenance class 1–2):** Adobe DNG Specification (the dual-illuminant /
  `NeutralToXY` interpolation model); McCamy, "Correlated color temperature as an
  explicit function of chromaticity coordinates" (1992) — the CCT polynomial. No code read.

## Outputs

- **`src/color.rs`:** `build_camera_to_srgb_linear` interpolates (`FM(g)`/`CM(g)`); new
  **private** helpers — `interpolation_weight(sensor) -> Option<f64>` (`None` ⇒
  single-illuminant fallback), `mccamy_cct(x, y) -> Option<f64>`,
  `cct_of_illuminant(code) -> Option<f64>`, `lerp_matrix(a, b, g)`, `xyz_to_xy`,
  `mul_mat_vec` — and a committed `const ILLUMINANT_CCT: &[(u32, f64)]` (or match arm)
  with each code's source. **No public API change, no new `Error` variant** (singular /
  ill-shaped interpolated matrix reuses `MissingColorCalibration`; the fallback is not an
  error).
- **`tests/color_oracle.rs` + `src/color.rs` in-module tests:** the interpolation oracle
  (below).
- **`docs/provenance-ledger.md`:** a row for the DNG `NeutralToXY` interpolation + the
  McCamy CCT (DNG spec = class 1; McCamy 1992 = class 2, a published paper).
- **`CHANGELOG.md`:** `[Unreleased] → Changed` — dual-calibration DNGs now interpolate
  the colour matrix to the scene illuminant (pixel values shift; **public API unchanged**;
  single-calibration files byte-identical).
- **`decisions/DEC-034-*.md`:** minted at design (this cycle).

## Acceptance Criteria

- [ ] Given `ColorMatrix1/2` + `CalibrationIlluminant1/2` + `AsShotNeutral`, the weight `g`
  equals the hand-computed DNG reciprocal-temperature `NeutralToXY` value (N=3) **to the
  bit**; `CM(g)`/`FM(g)` are the exact element-wise lerp; `cam2xyz` is built from them.
- [ ] A neutral at `T1` → `g = 0` (uses `CM1`/`FM1`); at `T2` → `g = 1`; a temperature
  outside `[T1, T2]` **clamps** to `0` / `1`.
- [ ] `mccamy_cct` reproduces known chromaticity→CCT values (e.g. the D65 and D50 white
  points) within the formula's own exactness, bit-exact against the committed constants.
- [ ] **Single-illuminant fallback is byte-identical to SPEC-033:** a sensor with no
  `ColorMatrix2` (or a missing/unknown `CalibrationIlluminant`, or `T1 == T2`) produces
  the exact `cam2xyz` SPEC-033 produced — the shipped SPEC-033 analytic fixtures still
  pass unchanged.
- [ ] The oracle goes **red** on a perturbed `ColorMatrix2` coefficient (which moves `g`
  and the output): mutant ≠ honest, honest control green (`oracle-must-be-shown-red`,
  `DEC-017`).
- [ ] Determinism: the weight path is `f64` `+ − × ÷` only (no FMA/transcendental), computed
  once per image; `apply_matrix_into` unchanged.
- [ ] Panic-free / typed errors, never a NaN in `dst`: a singular interpolated matrix, a
  degenerate `xy` (sum ≤ 0), `T1 == T2`, den-0 rationals, an unknown illuminant code — each
  is `MissingColorCalibration` or the graceful fallback, no panic. The mono develop + the
  demosaic + STAGE-006/007 + the SPEC-033 colour oracles stay green.
- [ ] Panic-free lints hold; the oracle is Tier-A (no corpus) so CI runs it.

## Failing Tests

In **`src/color.rs`** (`#[cfg(test)] mod tests`, private helpers) and
**`tests/color_oracle.rs`** (public API, analytic, Tier-A):
- `"interpolation_weight_matches_dng_neutral_to_xy"` — known `CM1/CM2`/illuminants/`asn`
  → hand-computed `g` (N=3), bit-exact.
- `"weight_clamps_outside_the_calibration_range"` — neutral hotter than `T2` → `g=1`;
  cooler than `T1` → `g=0`.
- `"mccamy_cct_matches_known_chromaticities"` — D65/D50 xy → expected CCT.
- `"forward_matrix_path_interpolates_both_matrices"` — `FM1`+`FM2` present → `cam2xyz` from
  `FM(g)`; pinned linear-sRGB output.
- `"color_matrix_path_interpolates"` — `CM1`+`CM2`, no `FM` → `cam2xyz` from `inv(CM(g))`.
- `"single_illuminant_is_byte_identical_to_spec_033"` — `CM2` absent → equals SPEC-033's
  illuminant-1 result; the shipped SPEC-033 fixtures are re-asserted unchanged.
- `"a_perturbed_second_matrix_turns_the_oracle_red"` + `"the_honest_control_is_green"`.
- `"degenerate_interpolation_inputs_do_not_panic"` — singular `CM(g)`, `xy` sum ≤ 0,
  `T1 == T2`, unknown illuminant code (→ fallback), den-0 rationals.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The base colour transform (WB + camera→XYZ→sRGB + OETF + `develop_color_into` + irr)
  — that is [[SPEC-033]]; this spec changes only *which* matrix the transform builds. The
  fixed PCS→display Bradford **D50→D65** is already in SPEC-033's committed `C` and is not
  touched.
- **The ColorMatrix-path scene-white→D50 chromatic-adaptation CAT.** SPEC-033 treats
  `inv(ColorMatrix)` output as XYZ(D50) directly; the fully-correct DNG model adapts the
  *scene* white (from the neutral) to D50 with a general Bradford CAT before `C`. This spec
  interpolates `CM(g)` but keeps SPEC-033's implicit-D50 structure for that path — the CAT
  refinement is **deferred**: [[SPEC-035]]'s ΔE measures whether it is worth its own spec.
  (The ForwardMatrix path already maps to D50 correctly, so Canon is unaffected by this
  deferral; it is the `ColorMatrix`-only Nikon path that stays approximate.)
- ColorChecker ΔE accuracy measurement — [[SPEC-035]].
- **Planckian / Robertson CCT** — McCamy's polynomial is chosen for determinism
  (`DEC-034`); a locus-based CCT (transcendental / table) is not used.
- `CameraCalibration` / `AnalogBalance` (assumed identity, `DEC-033`); per-camera profile
  curves / HueSatMap / LookTable; highlight recon; denoise.
- Illuminant estimation beyond what `AsShotNeutral` gives (no auto-WB / scene analysis);
  any public API / signature change.

## Notes for the Implementer

- **Localized change, heavy reuse.** The edit is `build_camera_to_srgb_linear` + new
  private helpers; reuse `mat_mul` / `inv` / `wb_diagonal` / `matrix_from_rationals` /
  `rational_to_f64` / the committed `C` constants / the oracle harness. No public
  signature changes, no new `Error` variant.
- **The weight uses `ColorMatrix`, the render may use `ForwardMatrix`.** `CM1`+`CM2` are
  required to find `g` (they exist at both illuminants); `FM(g)` is used only for the final
  `cam2xyz` when both `FM1`/`FM2` are present. No `CM2` ⇒ you cannot interpolate ⇒
  single-illuminant fallback, even if `FM2` exists.
- **Single-illuminant fallback is a regression guard, not a branch to gloss.** It must be
  **byte-identical** to SPEC-033 — re-run SPEC-033's shipped analytic fixtures under this
  code and assert they are unchanged. Most synthetic SPEC-033 fixtures have one calibration,
  so they exercise the fallback by construction.
- **Determinism (`DEC-033`/`DEC-034`): `f64`, `+ − × ÷` only.** McCamy is a cubic — write
  `n*n*n`, not `powf`. Commit the McCamy constants and the `ILLUMINANT_CCT` table as
  documented `const`s with their source in a comment. The whole weight computation runs
  once per image, outside the pixel loop.
- **Red-proof mutates a REAL coefficient** — perturb a `ColorMatrix2` entry (which shifts
  `g` and the output) and assert the output changes, honest control green (`plane_oracle.rs`
  lesson, `DEC-017`). A no-op red-proof is a reject.
- **Pre-named build-split escape hatch (if > [M]):** ship the interpolation *machinery*
  first — `lerp_matrix`, `mccamy_cct`, `cct_of_illuminant` + `ILLUMINANT_CCT`, `xyz_to_xy`,
  and the analytic tests with `g` **injected** — then the iterative `NeutralToXY`
  weight-from-neutral + the `build_camera_to_srgb_linear` wiring as a follow-on. The
  machinery is fully testable without the iteration; that is the clean cut (the SPEC-031 /
  SPEC-033 pattern).
- Guard every divide: `X+Y+Z ≤ 0`, `0.1858 − y == 0`, `T1 == T2`, a singular `CM(g)`, a
  den-0 rational → `MissingColorCalibration` or the fallback, **never** a NaN into `dst`.

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
