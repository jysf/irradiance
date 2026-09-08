---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-022
  type: story                      # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
  blocked: false
  priority: low                    # critical | high | medium | low
                                   #   Low: no current defect — HIT=0 on every
                                   #   decodable Q2M frame makes fixed_plane
                                   #   byte-identical to src, so the unasserted
                                   #   warp branch cannot yet mis-develop. This
                                   #   is harness completeness, not a live bug.
  complexity: XS                   # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-024]             # [DEC-NNN, DEC-MMM]
  constraints: [test-before-implementation, oracle-must-be-shown-red]
  related_specs: [SPEC-017, SPEC-018, SPEC-016]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: "STAGE-005: the suite stops asserting `develop_into` uses the fixed plane on only one of its two branches — the warp branch every real Q2M frame takes gets its own assertion"

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

# SPEC-022: develop warp branch must develop the fixed plane not raw src

## Context

Raised as `SPEC-017/FU-10` at SPEC-017's verify round 2 (2026-09-08) and
dispositioned here at SPEC-017 ship. This is one of the "separate items"
`SPEC-016`'s L→S rescope pushed onto STAGE-005's backlog — a surface
reporting a result it has not fully established, the stage's chartered
class.

`develop_into` (`src/develop.rs`) resolves `effective_src` — the
`FixBadPixelsConstant`-repaired plane when `OpcodeList1` carries the fix,
else the raw `src` (`src/develop.rs:816`) — then consumes it at **two**
call sites, one per branch:

- **no-warp** (`src/develop.rs:826`): `crop_orient_normalize_into(…, effective_src, dst)`
- **warp** (`src/develop.rs:842`): `normalize_active_area_into(…, effective_src, …)`, then `apply_warp_into`, then crop/orient into `dst`

SPEC-017's `AC9` (`develop_output_is_bit_identical_across_two_runs`)
asserts the fixed plane reaches `dst`, but its fixture carries no
`OpcodeList3`, so it exercises **only** the no-warp branch (`:826`).
Verify measured (SPEC-017 round 2) that severing **only** the warp-branch
use site (`:842`, `effective_src → src`; md5 `b3b922d0 → 607c546a`,
compiles) leaves the entire tier-A suite green at 231/0/3: **no tier-A
test sets `opcode_list_3`**, so nothing drives `develop_into` with a fix
opcode and a non-identity warp together, and tier-B is blind because
HIT=0 makes `fixed_plane` byte-identical to `src` on all three Q2M
frames. The sting: every real Q2M frame carries a non-identity
`WarpRectilinear`, so **the branch real files actually take is the
unasserted one**.

Not a live defect today (hence `priority: low`): with HIT=0 the two
branches develop identical bytes whether they read the fixed plane or
raw `src`. It becomes a real correctness hole the moment a frame has a
bad pixel (HIT>0) *and* a warp — then the fixed plane MUST flow through
`:842`, and nothing checks it.

## Goal

Add a synthetic tier-A fixture that drives `develop_into` with BOTH a
`FixBadPixelsConstant` `OpcodeList1` (HIT>0) and a non-identity
`WarpRectilinear` `OpcodeList3`, and asserts the developed output
reflects the FIXED plane through the warp branch (`:842`) — with a
red-proof (§16 rule 2, three clauses): severing `:842`'s
`effective_src → src` turns the new test red while AC9's no-warp
assertion stays green, then reverts byte-identical.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

Testable outcomes. Each must map to at least one test. Cover happy
path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)
- [ ] Criterion 3 (testable)

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- Changing `src/develop.rs` behaviour — the wiring is correct; this spec
  only adds the assertion that proves it for the warp branch.
- The oracle-scope narrowing for warp-bearing pixels — that is `SPEC-021`.
- Sourcing or fabricating a *real* Q2M frame with a bad pixel; the fixture
  is synthetic (a hand-built `Sensor` with both opcode lists set).

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities. Keep short — the full
context graph lives in the handoff file.

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
