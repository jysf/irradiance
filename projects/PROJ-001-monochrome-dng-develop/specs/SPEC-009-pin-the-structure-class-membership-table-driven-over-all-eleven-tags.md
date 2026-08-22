---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-009
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
  decisions: [DEC-012]             # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-007, SPEC-008]  # [SPEC-NNN]

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
value_link: "infrastructure enabling STAGE-002's unpack — require_uncompressed() and the StripByteCounts assertion both read tags this guard protects"

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

# SPEC-009: Pin the Structure class membership table-driven over all eleven tags

## Context

> **Framed 2026-08-21, not yet designed.** This spec exists as the destination for
> four `SPEC-008` follow-ups, per AGENTS.md §15's *Where an unresolved follow-up
> goes* — `frame` is the bar a disposition has to clear, not `ready`. Everything
> below `## Context` is still scaffold and the design cycle owns it.

**Carried findings:** `SPEC-008/FU-1`, `SPEC-008/FU-2`, `SPEC-008/FU-3`,
`SPEC-008/FU-5`.

**`SPEC-008/FU-1` — the membership list is pinned at one point out of eleven.**
`is_structural_tag()` (`src/ifd.rs:188-203`) gates `uints()`'s `TYPE_RATIONAL`
acceptance per tag (`src/ifd.rs:841`). It names eleven tags. Exactly **one** —
`TAG_SUB_IFDS` — is enforced by any test.

⚠ **Measured by the orchestrator, 2026-08-21, not inherited from the handback.**
`is_structural_tag()` reduced to `matches!(tag, TAG_SUB_IFDS)`; the mutation was
asserted applied by `diff` before anything was concluded from it; the suite was
**summed across all five targets** (`45 + 0 + 9 + 12 + 0`) with
`IRRADIANCE_CORPUS_DIR` set and the corpus present; the tree was restored and
`git status` confirmed clean.

| | tests |
|---|---|
| baseline | **66 passed** |
| ten of eleven memberships deleted | **66 passed** — nothing goes red |

The four structural fixtures `SPEC-008` added cannot catch it: they plant field
type `250`, which the **general** type gate rejects two lines below the per-tag
gate they never reach.

**The hazard is this stage's.** `Compression` encoded `RATIONAL 2/2` reads `1` →
`require_uncompressed()` (`src/ifd.rs:556`) passes → the unpack reads JPEG bytes
as raw samples. A wrong image from a file that parsed cleanly, which is this
project's signature failure shape. `StripByteCounts` as `RATIONAL 28/2` silently
reading `[14]` is the same defect against the plane's extent.

**Why this recursion terminates**, which is the reason it is worth doing at all.
`SPEC-007` fixed the behaviour, `SPEC-008` pinned the tags, and this pins the
membership — three turns of one screw, which is the `SPEC-001` gate-loop shape.
It stops here because of the **shape of the fix**: one table-driven test over all
eleven memberships has no "one point" left to be narrow at. ⚠ Design must not let
that test derive its table from `is_structural_tag()` itself — a test that reads
the list it is checking is a tautology, and deleting a tag would delete its own
coverage. The table is written out independently, and asserts **both** directions:
each of the eleven rejects a `RATIONAL` entry, and a paired interpretation tag
still reads one.

Scope note: the fix closes the *softening* direction. Adding a twelfth tag to
`is_structural_tag()` without a test row would go uncaught — that is the strict
direction and not the hazard.

**`SPEC-008/FU-2`** — "costed at most once" is unguarded on the only path where it
can fail. Splitting the combined `malformed.push` (`src/ifd.rs:1161-1178`) into one
push per erroring read reproduces `SPEC-007/FU-1`'s exact `[274, 274]` defect and
leaves 66/66 green, because
`orientation_costed_once_when_plane_is_ifd0`'s `sensor_read` is `None` by
construction. Needs a fixture with a malformed `Orientation` on **both** `IFD0` and
the SubIFD plane.

**`SPEC-008/FU-3`** — needs a **decision written down before code**. A well-formed
`IFD0` `Orientation` with a malformed sensor-IFD entry gives the right value and an
**empty** `malformed_tags`, while `Sensor::malformed_tags` is documented
(`src/ifd.rs:553-560`) as recording tags "present but shaped wrong". Both answers
are defensible; nothing says which wins, and it is untested either way.

**`SPEC-008/FU-5`** — `wellformed_orientation_is_not_recorded_malformed`
(`src/ifd.rs:2126-2152`) never asserts the precondition it depends on, unlike both
its neighbours. Verified still absent 2026-08-21. One line:
`assert_eq!(c.sensor_candidates(), vec![1]);`

**Why STAGE-002 and not STAGE-001.** The hazard bites at the unpack, so this is the
right stage; and `STAGE-001`'s close is the forcing function for three `lesson`
signals already at or past their bar (`measurement-over-generalised` at N=3 —
the very lesson that produced `FU-1`). Inserting this ahead of `SPEC-005` would
delay that close for a hazard that has no consumer today.

## Goal

Make every one of `is_structural_tag()`'s eleven memberships load-bearing: a
table-driven test that turns red when any single membership is deleted, plus
fixtures for the two unguarded `Orientation` paths and a decision on what a
swallowed malformed sensor-IFD read should record.

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
