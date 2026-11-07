---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-047
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M              # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: claude-opus-5          # HANDOFF-099 (build); tier_map.build
  created_at: 2026-11-06

references:
  decisions: [DEC-038, DEC-040]    # DEC-038 = the colour develop path + DEVELOP_VERSION; DEC-040 = index the phase/slot from the ActiveArea origin (offset, no crop) — formalized at this design
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
  tokens_estimate: 15000000
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-11-06
      notes: "Main-session design (un-metered; no /cost meter in the Claude Code Code tab). Formalized DEC-040 (offset from the ActiveArea origin, not a crop — chosen so even-origin files stay byte-identical and develop_color_into's contract is unchanged), wrote the four failing tests, HANDOFF-099 and the build prompt."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 14522562
      estimated_usd: 32.21
      duration_minutes: 40
      recorded_at: 2026-11-06
      notes: "Built whole, APPROVED by verify first pass (HANDOFF-099, PR #137, 0e4b4dc). Recovered 14,522,562 tok / $32.21 (claude-opus-4-8, from transcript cdeb761e, 65 deduped assistant messages, 97.8% cache reads); the HANDOFF-099 handback 28,690,878 was a non-deduped /cost over-count, superseded by the deduped transcript sum. The cold build inherited the orchestrator's claude-opus-4-8 (model:opus did not upgrade above the parent)."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 6848824
      estimated_usd: 16.46
      duration_minutes: 20
      recorded_at: 2026-11-06
      notes: "Verdict APPROVED at 0e4b4dc (HANDOFF-100, PR #138). Recovered 6,848,824 tok / $16.46 (claude-opus-4-8, from transcript 3f6f58fc, 40 deduped assistant messages, 97.0% cache reads); the HANDOFF-100 handback 195,000 was a context-window-occupancy floor, superseded. The verify proved the red-proof load-bearing by neutralizing DEC-040 — both grey tests then failed with a 40,716-count magenta while the byte-identity and no-panic tests stayed green."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-11-06
      notes: "Orchestrator ship bookkeeping (un-metered main session). Recovered the build and verify true costs from their transcripts, wrote the reflection and totals, archived the spec, flipped the STAGE-012 backlog, and logged the brag + changelog."
  totals:
    tokens_total: 21371386
    estimated_usd: 48.67
    session_count: 4
shipped_at: 2026-11-06
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
slot from the **ActiveArea origin** (not the plane's top-left), so a DNG with an
odd active-area origin develops to correct colour. Realize it with the **offset**
form `(y.wrapping_sub(active_top) & 1, x.wrapping_sub(active_left) & 1)` — no
crop, no change to `develop_color_into`'s input/output lengths (DEC-040). Bump
`DEVELOP_VERSION` 2 → 3. (Cropping the masked border and full geometry parity are
a deferred follow-up — see Non-Goals.)

## Inputs

- **Files to read:** `src/color.rs` (`develop_color_into`, `normalize_mosaic`),
  `src/demosaic.rs` (`demosaic_into`, `BayerPhase`), `src/develop.rs`
  (`resolve_geometry` — the ActiveArea → DefaultCrop → Orientation resolution to
  reuse), `src/ifd.rs` (`ActiveArea`, `Sensor`).
- **Reference only (not linked):** `pyirradiance/notes/active_area_origin_repro.py`
  and the report, which use LibRaw as the ground truth.
- **Prior decision:** `DEC-038` (the colour develop path + `DEVELOP_VERSION`).

## Outputs

- **Files modified:**
  - `src/color.rs` — `normalize_mosaic` indexes the 2×2 level slot from the
    ActiveArea origin; a small `active_area_origin(sensor) -> (u32, u32)` helper
    (origin, `(0, 0)` when the tag is absent).
  - `src/demosaic.rs` — `demosaic_into` and its green/colour site helpers compute
    `row_odd`/`col_odd` from the ActiveArea origin via the same helper.
  - `src/lib.rs` — `DEVELOP_VERSION` 2 → 3.
- **No contract change:** `develop_color_into` keeps `src.len() == width*height`
  and `dst.len() == width*height*3`; no new public function, no dependency, no
  crop. The border-crop and geometry parity are a deferred follow-up.
- **Decision:** `DEC-040` (already written) — offset indexing from the ActiveArea
  origin, and why not a crop, for P0.
- **Goldens:** even-origin fixtures are byte-identical, so the existing colour
  goldens do not move; regenerate only a deliberately odd-origin fixture, with
  the reason in the commit.

## Acceptance Criteria

- [ ] On a synthetic RGGB sensor with a flat-grey subject and a per-slot
  BlackLevel pattern, `develop_color_into` produces grey — per-channel spread
  `< 1e-3` — for **all four ActiveArea origin parities** (even/even, even/odd,
  odd/even, odd/odd) on **all four Bayer phases** (RGGB, BGGR, GRBG, GBRG).
- [ ] **Red-proof:** the same assertion on the pre-fix plane-relative computation
  shows a large channel spread (magenta) for the odd-origin parities — the fix is
  shown to be load-bearing.
- [ ] **Even-origin is byte-identical:** an even-origin fixture develops to the
  exact same bytes as the plane-relative path (so the existing goldens hold).
- [ ] `DEVELOP_VERSION` is `"3"`.
- [ ] No panic for any ActiveArea origin, including one at or beyond the plane
  edge (the `wrapping_sub` low bit is still a valid parity).
- [ ] Determinism held; no new dependency; `develop_color_into`'s signature and
  buffer lengths unchanged.

## Failing Tests

In `tests/color_oracle.rs` (the Tier-A synthetic colour oracle — `fixture_sensor`,
`rggb()`, the D50 forward-matrix helpers, and `a_neutral_subject_is_grey` are the
pattern to build on). The build cycle makes these pass.

- **`tests/color_oracle.rs`**
  - `"grey_stays_grey_across_origin_parities_and_bayer_phases"` — a helper builds
    a `fixture_sensor` with `active_area` set to each of the four origin parities
    (e.g. `(2,4)`, `(2,5)`, `(3,4)`, `(3,5)`), a per-slot 2×2 BlackLevel pattern,
    a neutral `as_shot_neutral`, and a D50 forward matrix; fills `src` with a
    flat-grey mosaic for that origin + phase; runs `develop_color_into` and
    asserts every pixel's `max(R,G,B) - min(R,G,B)` is within the grey tolerance,
    across all 4 phases × 4 parities (16 cases).
  - `"plane_relative_phase_tints_an_odd_origin_grey"` — the red-proof control: the
    same input developed with the plane-relative index (`y & 1`) asserts a large
    channel spread for an odd origin, proving the offset is what fixes it
    (a test-local plane-relative reference, as the existing red-proofs perturb a
    reference; it must fail without DEC-040).
  - `"even_origin_is_byte_identical_to_plane_relative"` — for an even origin,
    `develop_color_into` output equals the plane-relative reference exactly (the
    no-regression guarantee for the corpus).
  - `"develop_color_does_not_panic_for_an_out_of_range_active_area"` — an
    ActiveArea origin at/beyond the plane edge develops without panic.

## Non-Goals

- The colour math — the ColorMatrix / Bradford-adaptation path (`DEC-038`) and
  the ColorChecker ΔE work (`SPEC-035`) are untouched.
- Demosaic performance (P1), colour-develop memory + the f32 entry point (P2),
  and threading (P3) — separate specs in `STAGE-012`'s backlog.
- **Cropping the colour output to the active area, removing the masked border,
  and full `DefaultCrop`/`Orientation` parity with `develop_into`** — the larger,
  contract-changing shape (DEC-040 explains why it is out of P0). Its own
  STAGE-012 spec, listed in the backlog.
- The exact-LibRaw demosaic behaviour at the active-area boundary (edge reflection
  within the active area vs reading the masked border) — a crop-time concern,
  deferred with the crop.
- The mono/tone develop path (`develop_into`) — already correct.
- Subsampled (`Hi`/`Vi != 1`) or tiled layouts — out (see the
  `unexercised-sof3-paths-garble-silently` risk; a different concern).

## Notes for the Implementer

- **The fix (DEC-040):** compute `row_odd = (y.wrapping_sub(active_top) & 1) == 1`
  and `col_odd = (x.wrapping_sub(active_left) & 1) == 1` in both `normalize_mosaic`
  (`src/color.rs`) and `demosaic_into` and its site helpers (`src/demosaic.rs`),
  with `(active_top, active_left)` from a shared `active_area_origin(sensor)`
  helper (`(0, 0)` when `sensor.active_area` is `None`, matching
  `resolve_geometry`). `wrapping_sub` keeps it panic-free and parity-correct even
  above/left of the origin.
- **Do not crop and do not change buffer lengths** — the offset is the whole
  change; keeping `develop_color_into`'s contract is the point of choosing offset
  over crop for P0.
- One helper, two call sites: put `active_area_origin` where both `color.rs` and
  `demosaic.rs` can use it (and it is reachable from `develop.rs`), so the two
  cannot drift — the single source of truth for the origin.
- Keep the zero-dependency, no-panic, determinism rules. LibRaw/rawpy are the
  reference oracle in pyirradiance — read-only, never linked.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — The offset-vs-crop call was the decision that mattered; splitting the crop /
   masked-border / geometry work out of P0 kept it small enough to ship with zero
   golden churn and no contract change. Nothing I'd change — except to distrust a
   build's self-reported `/cost` sooner: the handback (28.7M) was roughly double
   the deduped-transcript truth (14.5M), the usual non-deduped over-count.

2. **Does any template, constraint, or decision need updating?**
   — No new rule. `DEC-040` records the decision; the crop follow-up and
   `DEVELOP_VERSION` bump are captured. The handback-vs-deduped gap is already
   covered by the standing "recover true costs from transcripts at ship" practice.

3. **Is there a follow-up spec I should write now before I forget?**
   — Already in the `STAGE-012` backlog: the colour-path crop + masked-border
   removal + full `DefaultCrop`/`Orientation` parity (the §3B crop half this spec
   deferred), plus the P1 demosaic fast path, P2 fused-buffer + f32 entry point,
   and P3 opt-in threading.

4. **Where was the worst defect caught?**
   — none
   *(The build was APPROVED on the first verify pass; the verify's own adversarial
   neutralization of DEC-040 confirmed the red-proof rather than finding a defect.)*

5. **What can a user do now that they couldn't before?**
   — Develop a DNG whose `ActiveArea` origin is odd to correct colour — a flat
   grey stays grey (channel spread < 1e-3) where it previously came out magenta
   (`0.589 0.000 0.669`), now matching LibRaw's `~0.200` on every channel.
