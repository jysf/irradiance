---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-100
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
  decisions: [DEC-087, DEC-086, DEC-084, DEC-059]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-064, SPEC-093, SPEC-098, SPEC-099]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-099]             # e.g. [SPEC-002]

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

# SPEC-100: the per-plane warp on the developed colour output and the developed mix

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-100]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Fourth spec of STAGE-034 (PROJ-006), split out of SPEC-099 at its design (2026-11-28,
DEC-087). SPEC-099 corrects lateral chromatic aberration at sensor geometry: on a
per-plane `WarpRectilinear` (`N = 3`, sets that differ) the sensor-geometry planes
(`srgb_linear_plane_*`, `mono_mix_plane_*` and their rows forms) resample red and blue to
green's geometry, and `opcode::sensor_geometry_warp` names the one warp left for the
consumer (green's set). The developed outputs are unchanged by SPEC-099 and still refuse
the held Q2, Q3 and SL3 files:

- `develop_color_into`, `develop_color_f32_into` and the threaded forms refuse with
  `UnsupportedRequiredOpcode { list: 3, id: 1 }`: `DevelopPath::Colour` applies no warp.
- The developed mix (`develop_with_into` with `mono_mix`) refuses a mandatory first
  `WarpRectilinear` whose `N` is not 1 and skips an optional one (DEC-086 item 6).

onyx's rule (its DEC-114, 2026-11-28, in SPEC-099's Context): the developed mix is the
CA-corrected mixed plane with green's warp applied at `warp_alpha` and `Framing`, as onyx's
viewport applies it, so export equals the viewport by construction (DEC-084 rule 1).

**What SPEC-099's design measured that this spec can use** (its Implementation Context, the
probe in the PROJ-006 session notes, `spec-099-design-*`):
- Against `dng_validate -3` (stage 3, camera RGB after the per-plane warp) on the three
  files, the CA step followed by green's warp registers red and blue to the SDK as well as
  green's warp alone registers to an SDK run with every set replaced by green's (the
  residual shift agrees within 0.042 px at 48 patches). One resample with each plane's own
  set (the DNG stage-3 order) registers equally well (within 0.02 px of the baseline). The
  two-resample form is slightly smoother (lower mean difference to the SDK), the
  one-resample form is sharper.
- The CA step, single-threaded on a full plane, costs 0.5–0.64 s on these files.

## Goal

The held Q2, Q3 and SL3 files give a colour develop and a developed mix with no refusal:
the colour develop applies the per-plane warp, and the developed mix is SPEC-099's
CA-corrected mixed plane with green's warp at `warp_alpha` and `Framing`, bit-identical to
the mono develop of that plane (DEC-084 rule 1). Every file with no per-plane difference
(`N = 1`, or identical sets) keeps its output bit for bit.

## Inputs

- **Files to read:** SPEC-099 and DEC-087 (the CA rule and the accessor); `src/develop.rs`
  (`develop_core`, `run_mix_passes`, `mix_needed_scratch`, `develop_warp`); `src/color.rs`
  (`developed_mix_strips`, `develop_color_with`, `develop_color_bands`, `CropScatter`);
  `src/warp.rs`; DEC-059, DEC-084, DEC-086.
- **External:** `dng_validate -3` on the three files (SPEC-099's script).
- **Corpus:** the three files (tier B, local only for the Q3 and SL3).

## Questions for design

- **The colour develop:** the CA step then green's warp (two resamples, matching the
  developed mix and the viewport), or each plane's own set in one resample (the DNG order)?
  Where it runs relative to the colour matrix (the warp is defined on camera RGB, before
  the matrix), and what intermediate it costs (`3·A` `u16` is about 363 MB at 60 MP).
- **The developed mix:** the strip pass needs SPEC-099's CA ring inside
  `developed_mix_strips` (strips plus the ring's halo), then green's warp over `A1`. Which
  DEC-084 and DEC-086 items does it amend, and how does `scratch_len` change?
- **`warp_alpha` and `Framing` on the colour develop:** gained, or the defaults only?
- **Optional warps:** the developed mix skips an optional `N ≠ 1` warp today and the colour
  develop skips every optional warp; SPEC-099's planes run the CA step whatever the flags.
  Does the developed mix start applying an optional `N = 3` warp, so it matches
  `mono_mix_plane_into` again (DEC-084 rule 1)? Does `DEVELOP_VERSION` move?
- **Threaded forms:** the same output, bit-identical across thread counts.
- **The oracle:** SPEC-099's `dng_validate -3` cross-check, now on the developed colour.

## Non-Goals

- The CA step itself and the accessor (SPEC-099).
- Tangential warp terms, a per-plane `N` other than 1 or 3.
- Highlight recovery above `WhiteLevel` and filter presets.

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
