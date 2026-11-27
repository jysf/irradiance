---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-099
  type: story                      # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
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
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-006
  stage: STAGE-034
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-086, DEC-084, DEC-059]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-064, SPEC-091, SPEC-093, SPEC-098]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-098]             # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: the Leica Q2, Q3 and SL3 colour DNGs (a mandatory three-plane WarpRectilinear) give a colour develop and a developed mix instead of refusing"

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

# SPEC-099: The per-plane WarpRectilinear on the developed colour outputs, the Q2, Q3 and SL3 files

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-099]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Third spec of STAGE-034 (PROJ-006), split out of SPEC-098 at its design (2026-11-27,
DEC-086). After SPEC-098 the Q2 (`LEICA-Q2/L1000750.DNG`, CC0), Q3 and SL3
(`LEICA-Q3-PB/leica_q3_01.dng`, `LEICA-SL3-PB/leica_sl3_01.dng`, Photography Blog, local
only, never committed) give a linear plane and a mixed plane at sensor geometry, with
`FixBadPixelsConstant` applied on the CFA. Their developed outputs still refuse:

- `develop_color_into`, `develop_color_f32_into` and the threaded forms refuse with
  `UnsupportedRequiredOpcode { list: 3, id: 1 }`: `DevelopPath::Colour` applies no warp.
- The developed mix (`develop_with_into` with `mono_mix`) refuses the same way: DEC-086
  item 6 refuses a mandatory `WarpRectilinear` whose `N` is not 1 on `DevelopPath::MonoMix`.

Each file's `WarpRectilinear` has `N = 3`, one coefficient set per colour plane, with
`cx = cy = 0.5` and no tangential terms (SPEC-098's Implementation Context). The full
correction moves a corner by about 460 px on the Q2, 524 px on the Q3 and 43 px on the SL3,
and the three planes' sets differ by up to 1.1, 1.2 and 1.3 px at the corner (lateral
chromatic aberration). The mono warp (`SPEC-064`, `src/warp.rs`) applies one set to one
plane, and the parser keeps only `K1` (`src/opcode.rs`, `parse_warp_rectilinear_params`).

onyx's rule 1 (STAGE-034 Design Notes): only the developed outputs apply the warp, or
refuse when they cannot. DEC-084 makes the developed mix equal onyx's viewport, which warps
the mixed plane, so the developed mix's three-plane rule depends on what the viewport does.

## Goal

The held Q2, Q3 and SL3 files give a colour develop and a developed mix with no refusal,
each plane warped by its own coefficient set as DNG 1.7.1.0 specifies, with no output
change for any file that develops today.

## Inputs

- **Files to read:** `src/warp.rs` (`apply_warp_into`, `source_coord`, the bilinear
  sampler); `src/opcode.rs` (`parse_warp_rectilinear_params`, `WarpRect`, `DevelopPath`);
  `src/color.rs` (the fused traversal and every colour entry); `src/develop.rs`
  (`develop_core`, `run_mix_passes`, `mix_needed_scratch`); SPEC-064, SPEC-093, SPEC-098;
  DEC-059, DEC-084, DEC-086.
- **External:** DNG 1.7.1.0 `WarpRectilinear` ("N must be 1 or the total number of image
  planes"). `dng_validate -3` on each file: stage 3 is the camera-RGB `ActiveArea` after
  the per-plane warp (3 planes, 8392×5632 on the Q2, 9536×6344 on the Q3 and SL3; already
  generated at SPEC-098's design), the oracle candidate.
- **Corpus:** the three files, promoted to `[[file]]` entries by SPEC-098's build.

## Questions for design

- **Where the colour warp runs:** on camera RGB before the colour matrix (the DNG stage-3
  position, needed because each plane has its own warp), and what intermediate that costs
  (`3·A` `u16` is about 363 MB at 60 MP). Can it gather straight into the cropped,
  oriented `dst`?
- **The developed mix with `N = 3`:** warp each plane and then mix (the DNG order), or mix
  and warp with one set (DEC-084's order)? Which set does onyx's viewport use on a
  three-plane file? This is onyx's question, and it decides whether DEC-084 needs an
  amendment.
- **An accessor** for the per-plane sets, so onyx's viewport can read them (SPEC-098 only
  documents the pass-through, and `parse_warp_rectilinear` returns `K1`).
- **`develop_color_f32_into` and the threaded forms:** the same warp, bit-identical
  across thread counts?
- **The oracle:** `dng_validate` stage 3 against the warped camera RGB, within a
  pre-registered bound (the SDK's resampling kernel may differ from irradiance's
  bilinear one).
- **`warp_alpha` and `Framing`:** does the colour develop gain them, or only the defaults?
- **Optional warps:** `develop_color_into` skips an optional `WarpRectilinear` today (the
  Pixels carry one with `N = 1`). Applying optional ones would change their output, so
  `DEVELOP_VERSION` must move if it does. The default is to keep skipping them.
- **Spec size:** M is expected. Split again if the developed mix's answer needs its own
  decision.

## Non-Goals

- Applying the warp at sensor geometry (onyx's rule 1).
- `FixBadPixelsConstant` (SPEC-098).
- Tangential warp terms (no held file has them; `UnsupportedWarpTangentialTerms` stays).
- Highlight recovery and filter presets.

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
