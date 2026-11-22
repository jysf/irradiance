---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-087
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
  id: PROJ-002
  stage: STAGE-027
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-012, DEC-053, DEC-051]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-086]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-086]  # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-027's <capability>". Optional; null is acceptable.
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

# SPEC-087: Malformed interpretation tags refuse at develop

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-087]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

`STAGE-027` (PROJ-002), third of three. The audit in PR 303 (§1, §4 rank 2,
"Spec B") found that a malformed interpretation-class tag (`DEC-012`) is pushed
onto `Sensor::malformed_tags` and then treated as absent. The only consumer of
`malformed_tags` is the opcode-list check (`DEC-053`). For `BlackLevel`,
`WhiteLevel`, `ActiveArea`, `DefaultCropOrigin/Size`, `Orientation`, the
second-illuminant colour tags, `CFALayout` and `CFAPlaneColor`, a malformed
value develops exactly like an absent one. That re-opens PR 301's two CFA
refusals through a bad field type.

DEC-012's rationale for tolerance ("not consumed yet") has aged: these tags are
consumed now. No held file trips this today (the audit found `malformed_tags`
empty on the held files), so it is ranked for breadth, not for a live hit.

`depends_on: [SPEC-086]` is about ordering (the shared `Error` tail), not content.

**Carried in from SPEC-085 (shipped 2026-11-22).** Decide at design whether this spec
also covers a dual-illuminant file with a singular `ColorMatrix1/2`, or a singular
applied `CameraCalibration1/2`, that the converged weight never uses. Since DEC-076 §5
such a file develops (the loop starts at D50 and inverts only the product at the
weight it lands on), and is refused only when the weight needs the singular product.
So whether it develops depends on the scene. Refuse it outright, or keep the SDK's
behaviour; see SPEC-085's build ruling and the CHANGELOG entry.

**Carried in from SPEC-086 (shipped 2026-11-22).** A malformed `DNGVersion` (50706) with
no `DNGBackwardVersion` is recorded in `malformed_tags` and then skipped, so DEC-077's
version guard does not run. The SPEC-086 verify ruled that correct under DEC-077 as
written, and asked that this spec refuse it as `UnsupportedTag { tag: 50706 }`, matching
a malformed 50707, because the file's reader version is then unknown. Give 50706 its own
`Display` wording, as 50707 has.

## Goal

A malformed interpretation tag that a develop path consumes refuses at that
develop with a typed `Error`, generalising DEC-053 and amending DEC-012. The
tolerant parse is kept, so metadata and previews still read, and the existing
nuance survives: a malformed `BlackLevelRepeatDim` is fatal only when
`BlackLevel` has more than one value.

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

**Pre-register every numeric threshold.** A tolerance, bound or score an AC
checks is written here at design, with its falsifier, before the build measures
anything. A measured number outside it is reported, never fixed by widening the
bound (signal `pre-register-the-tolerance`).

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)
- [ ] Criterion 3 (testable)

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

**Rule-clause map.** When this spec implements a `DEC-*` rule, map every clause
of it (each malformed class, each entry point, each numeric path) to a named
test above, and say which code path that test must reach. Name the path where the
clause changes the output, not just one that runs it: a clause tested only where it
is unobservable leaves a mutant the build's suite cannot see (signal
`every-rule-clause-maps-to-a-reachable-test`).

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
