---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-010
  type: story                      # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  stage: STAGE-002
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-005]        # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: null

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

# SPEC-010: Distinguish an unparseable tool reading from an absent tag and reconcile the frozen fixture

## Context

> **Framed 2026-08-22, not designed.** The destination for four `SPEC-005`
> follow-ups, per AGENTS.md §15. Everything below is scaffold.

**Carried findings:** `SPEC-005/FU-1`, `FU-2`, `FU-4`, `FU-9`.

**`FU-1` — the oracle cannot tell "tag absent" from "tag present but
unparseable."** Both collapse to `None` in `reading_from_fields`, so a garbled
tool reading silently *agrees* with a `None` on our side. Measured by
`SPEC-005`'s reviewer: **5/5 garbled readings diff clean**. This sits outside
`AC2`'s wording, so it is a design gap as much as a build one — the spec did not
anticipate it and neither did the architect.

⚠ **The fix is already specified AND already measured.** A tri-state on the tool
side, compared against `Sensor::malformed_tags`. `SPEC-005/FU-8` built it during
verify round 2 and confirmed all 21 oracle tests stay green — *and* that a
tri-state **without** the `malformed_tags` comparison reds. Do not re-derive
this; reproduce it and build it.

⚠ **It has a consequence in `tests/support/tools.rs`'s `diff()` doc comment**,
which currently reasons about exactly this future. `DEC-013` was `rejected`
partly on the argument that fixing `FU-1` would trip an alarm — `FU-8` measured
that it does **not**, because the `malformed_tags` comparison *is* the generic
guard on the side that holds the information. Update that doc comment when you
land this, and consider whether `DEC-013`'s rejected conclusion now deserves a
successor decision that is **true**.

**`FU-2` — `opt()`/`req()` truncate a multi-valued reading to its head.**
`black="512 999"` → `Some(512)`, diffs clean. Latent on today's monochrome
corpus; **live the moment `SamplesPerPixel > 1`**, which is PROJ-002. Same parse
layer as `FU-1`, so same pass.

**`FU-4` — the tier-A fixture is two frozen literals** carrying the three blind
spots `SPEC-005`'s own `## Context` indicts the old `Expected` table for, and
nothing reconciles them even where the corpus and both tools are present. Both
halves were verified accurate on 2026-08-22 — this is rot risk, not a present
defect. A reconcile-when-both-available test closes it.

**`FU-9` — `is_active()` (`scripts/decisions-audit.sh:152`) reads only
`superseded_by`, never `status`,** so `DEC-013` — the repo's first `rejected`
decision — still reports as governing `tests/support/tools.rs`. ⚠ **Fix the
verb, not the filter.** The reviewer's point: that surfacing is currently the
*only* mechanical signpost from the code to the explanation of why its guard is
gone. Filtering rejected decisions out would silently remove it.

## Goal

1–2 sentences. Unambiguous. If you can't write the goal in two
sentences, split the spec.

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

- ...

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
