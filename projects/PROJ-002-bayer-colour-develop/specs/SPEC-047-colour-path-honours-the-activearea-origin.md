---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-047
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
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
  stage: STAGE-012
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-038]             # DEC-038 = the colour develop path + DEVELOP_VERSION; DEC-040 (ActiveArea-relative phase) is NEW, formalized at this spec's design
  constraints: []                  # design cycle fills exact ids: determinism, no-panic, oracle-must-be-shown-red
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-012's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-012's first capability: correct colour for DNGs with an odd ActiveArea origin, cropped to the active area."

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
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-047: colour path honours the ActiveArea origin

## Context

The Python bindings' rawpy/LibRaw benchmark (`docs/reviews/`, and
`pyirradiance/notes/upstream-review-2026-11-06.md`) found that the colour
develop path picks each pixel's CFA colour and its BlackLevel/WhiteLevel 2×2
slot from the **plane's** top-left corner, but in DNG both patterns start at the
**ActiveArea's** top-left. When `ActiveArea.top` or `.left` is odd, every pixel
is assigned the wrong colour: a flat grey subject develops to magenta (mean RGB
`0.589 0.000 0.669`) where LibRaw gives `0.200 0.199 0.202`, and **no error is
raised**. The same path also emits the whole uncropped plane, including the
masked border outside ActiveArea.

Confirmed in code: `normalize_mosaic` (`src/color.rs:729`) computes the slot from
`(row & 1, col & 1)` with `row`/`col` plane-relative, and `demosaic_into`
(`src/demosaic.rs`) computes the phase from `(y & 1, x & 1)` the same way; the
colour path never crops to the active area. The mono/tone path already resolves
geometry (`resolve_geometry`, `src/develop.rs:130`).

The real corpus files have **even** origins (Canon 5D3 `(80, 122)`, Nikon D300S
`(0, 0)`), so the current goldens are unaffected — but odd-origin cameras exist,
LibRaw handles them, and the failure is silent wrong colour. This is the external
0.2.0 staff review's **§3B** item, and a **pre-0.2.0 correctness blocker** — the
first spec of `STAGE-012`.

## Goal

Make the colour develop path take the CFA phase and the BlackLevel/WhiteLevel
slot from the **ActiveArea origin**, and crop its output to the active area, so a
DNG with an odd active-area origin develops to correct colour and the masked
border is dropped. Bump `DEVELOP_VERSION`.

## Inputs

- **Files to read:** `src/color.rs` (`develop_color_into`, `normalize_mosaic`),
  `src/demosaic.rs` (`demosaic_into`, `BayerPhase`), `src/develop.rs`
  (`resolve_geometry` — the ActiveArea → DefaultCrop → Orientation resolution to
  reuse), `src/ifd.rs` (`ActiveArea`, `Sensor`).
- **Reference only (not linked):** `pyirradiance/notes/active_area_origin_repro.py`
  and the report, which use LibRaw as the ground truth.
- **Prior decision:** `DEC-038` (the colour develop path + `DEVELOP_VERSION`).

## Outputs

- **Files modified:** `src/color.rs` and `src/demosaic.rs` (index the phase and
  level slot from the ActiveArea origin, crop to the active area), `src/lib.rs`
  (`DEVELOP_VERSION` 2 → 3).
- **New decision:** `DEC-040` — the CFA phase and level-pattern slot are indexed
  from the ActiveArea origin; records the chosen implementation (crop vs offset)
  and why. Written during this spec's **design** cycle.
- **Output contract:** cropping changes `develop_color_into`'s output
  **dimensions** (the `dst` length is now the resolved active-area size, not the
  full plane) — the design cycle states the new contract, matching `develop_into`.
- **No new public function or dependency** (P1/P2/P3 add those, in later specs).
- **Goldens:** regenerate any colour fixtures whose output changes, with the
  reason in the commit.

## Acceptance Criteria

- [ ] On a synthetic RGGB DNG with a flat grey subject, `develop_color_into`
  produces grey — per-channel spread `< 1e-3` — for **all four ActiveArea origin
  parities** (even/even, even/odd, odd/even, odd/odd) on **all four Bayer phases**
  (RGGB, BGGR, GRBG, GBRG).
- [ ] **Red-proof:** the same test on today's plane-relative code fails (magenta)
  for the odd parities — the fix is shown to be load-bearing.
- [ ] The colour output is cropped to the active area (no pixels from the masked
  border outside ActiveArea appear in the output).
- [ ] Even-origin behaviour is preserved on the active-area pixels (the Canon and
  Nikon colour values are unchanged there; any golden diff is the crop alone).
- [ ] `DEVELOP_VERSION` is `3`.
- [ ] A malformed ActiveArea (origin or size outside the plane) does not panic —
  typed error or safe clamp, consistent with `resolve_geometry`.
- [ ] Determinism held; no new dependency.

## Failing Tests

*Sketched at frame; the **design** cycle writes them in full (and formalizes
`DEC-040`) before handoff.*

- **`tests/color_oracle.rs`** (or a new colour-geometry test)
  - `"grey stays grey across the four origin parities and four Bayer phases"` —
    asserts channel spread `< 1e-3` for the 16 combinations.
  - `"the plane-relative phase tints an odd-origin grey magenta"` — the red-proof
    control, asserting the pre-fix computation fails.
  - `"colour output is cropped to the active area"` — asserts output dimensions
    equal the resolved active-area size.

## Non-Goals

- The colour math — the ColorMatrix / Bradford-adaptation path (`DEC-038`) and
  the ColorChecker ΔE work (`SPEC-035`) are untouched.
- Demosaic performance (P1), colour-develop memory + the f32 entry point (P2),
  and threading (P3) — separate specs in `STAGE-012`'s backlog.
- Full `DefaultCrop` / `Orientation` parity with the mono path beyond the
  ActiveArea crop — if wanted, its own spec; this one fixes the colour and the
  active-area crop only.
- The mono/tone develop path (`develop_into`) — already correct.
- Subsampled (`Hi`/`Vi != 1`) or tiled layouts — out (see the
  `unexercised-sof3-paths-garble-silently` risk; a different concern).

## Notes for the Implementer

- **The decision (DEC-040, at design):** recommended implementation is to **crop
  the colour path to the active area before normalize/demosaic**, reusing
  `resolve_geometry`, so the origin becomes `(0, 0)` — this makes both the phase
  and the level-slot computation correct at once and removes the border work. The
  alternative is to offset the index (`((y - top) & 1, (x - left) & 1)`) in
  `normalize_mosaic` and `demosaic_into` without cropping; the design chooses and
  records why. Either way `DEVELOP_VERSION` bumps 2 → 3.
- Cropping changes the output dimensions — do not silently keep full-plane
  dimensions; match the mono path's resolved size and state it in the doc comment.
- Keep the zero-dependency, no-panic, determinism rules. LibRaw/rawpy are the
  reference oracle in pyirradiance — read-only, never linked.

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
