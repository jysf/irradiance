---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-098
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
  decisions: [DEC-084, DEC-053]    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-017, SPEC-091, SPEC-092, SPEC-093, SPEC-097]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: the Leica Q2, Q3 and SL3 colour DNGs (mandatory FixBadPixelsConstant and WarpRectilinear) reach the colour path and the mixer instead of refusing"

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

# SPEC-098: Mandatory FixBadPixelsConstant and WarpRectilinear on the colour entries, the Q2, Q3 and SL3 files

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-098]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Second and last spec of STAGE-034 (PROJ-006). After SPEC-097 (shipped 2026-11-27) the
M10 and M11 reach every colour and mixer entry, so the only refusals left on held Leica
colour files are these three bodies:

- **Q2** (`LEICA-Q2/L1000750.DNG`, raw.pixls.us, CC0),
  **Q3** (`LEICA-Q3-PB/leica_q3_01.dng`) and **SL3** (`LEICA-SL3-PB/leica_sl3_01.dng`)
  (Photography Blog, no stated licence, local corpus only, never committed).
- All three unpack. Every colour entry then refuses with
  `UnsupportedRequiredOpcode { list: 1, id: 4 }`, because each carries a mandatory
  OpcodeList1 `FixBadPixelsConstant` and a mandatory OpcodeList3 `WarpRectilinear`.
- The colour entries (`develop_color_into`, `develop_color_f32_into`,
  `srgb_linear_plane_*`, `mono_mix_row`, `mono_mix_plane_*`, and `develop_into` with
  `mono_mix`) apply only OpcodeList2 `GainMap`s. Every other mandatory opcode is refused
  through `check_required_processing` (`SPEC-054`, `SPEC-056`, `DEC-053`).
- The mono develop already applies both opcodes on the held Q2 Monochrom:
  `apply_fix_bad_pixels_constant` (`SPEC-017`, a 3×3 median of valid neighbours, with no
  CFA colour) and the rectilinear warp with `warp_alpha` (`SPEC-064`). The developed mix
  (`SPEC-093`, `DEC-084`) mixes before geometry, so it reaches the same warp.

onyx's rules, from its SPEC-093 reply (2026-11-26), recorded in STAGE-034's Design Notes:
1. **Warp at sensor geometry is the consumer's.** SPEC-091's planes and
   `mono_mix_plane_*` are onyx's viewport input, and the viewport applies the warp on the
   GPU. These entries must not refuse a mandatory `WarpRectilinear`, and they document that
   the consumer applies it. Only the developed outputs (`develop_color_into` and the
   developed mix) apply the warp, or refuse when they cannot.
2. **`FixBadPixelsConstant` runs on the CFA before demosaic, on every entry,** filling each
   bad photosite from neighbours of the **same CFA colour**, so export matches the viewport.

## Goal

The held Q2, Q3 and SL3 colour files produce a linear plane, a mixed plane, a colour
develop and a developed mix with no refusal. `FixBadPixelsConstant` is applied on the CFA
before demosaic on every colour entry. The developed outputs apply `WarpRectilinear`, and
the sensor-geometry entries pass it through to the consumer. No output changes for any
file that develops today.

## Inputs

- **Files to read:**
  - `src/opcode.rs`: `check_required_processing`, `DevelopPath` and the
    `FixBadPixelsConstant` parse, including `BayerPhase`.
  - `src/develop.rs`: `apply_fix_bad_pixels_constant`, the warp, and `develop_core` under
    `mono_mix`.
  - `src/color.rs`: every colour entry's refusal list and the demosaic call.
  - The shipped SPEC-017, SPEC-064, SPEC-091, SPEC-092 and SPEC-093, plus DEC-053 and
    DEC-084.
- **External:** DNG 1.7.0.0 on `FixBadPixelsConstant` (p.95, `BayerPhase`) and
  `WarpRectilinear` (p.90). The Adobe DNG SDK's bad-pixel fix on a CFA. `dng_validate`
  stage 2 and 3 output on each file, as the oracle.
- **Corpus:** the three files above, all held since 2026-11-26. Record the Q3 and SL3 in
  the manifest at design, metadata only, under the same rule as the M11.

## Questions for design

- **Which colour entries apply which opcode, and in what order** relative to GainMap,
  black/white scaling, white balance and demosaic? Must the opcode check refuse a
  mandatory warp in OpcodeList3 on the sensor-geometry entries but not on the developed
  ones?
- **The same-colour fill:** which kernel applies (median of same-colour neighbours, at
  which offsets)? How is `BayerPhase` used, and how is it checked against `CFAPattern`?
  Does the SDK's result match within a stated bound?
- **How many photosites** on each held file actually equal the constant? If there are
  none, how do the tests prove the fix runs?
- **The developed colour warp:** is it the same warp code as the mono develop, applied to
  three channels? Does `develop_color_into` gain a scratch need, and does that affect
  `scratch_len`?
- **How the sensor-geometry entries pass the warp through:** documentation only, or an
  accessor so onyx can read the warp parameters?
- **Versions:** confirm that no file that develops today changes, so `DEVELOP_VERSION`
  and `MONO_MIX_VERSION` stay. Is a new path value in `DevelopPath` needed?
- **Does any held file then hit a further refusal?** Report what it hits.
- **Spec size:** one spec or two (bad pixels, then warp)? Split at design if the warp half
  is large.

## Non-Goals

- Highlight recovery above `WhiteLevel` (onyx's open question in the PROJ-006 brief).
- Other mandatory opcodes no held file carries; each gets its own spec once a file shows
  it.
- Applying the warp at sensor geometry inside irradiance; per onyx's rule 1, that's the
  consumer's.
- Filter presets (a later PROJ-006 stage).
