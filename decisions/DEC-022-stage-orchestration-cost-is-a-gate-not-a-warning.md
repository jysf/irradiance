---
# Maps to ContextCore insight.* semantic conventions.
insight:
  id: DEC-022
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - scripts/cost-audit.sh
  - scripts/_lib.sh
  - projects/_templates/stage.md

tags:
  - cost
  - gates
  - process
  - dec-013
---

# DEC-022: A shipped stage's orchestration cost is a gate, not a warning — amending DEC-013 §5

## Decision

`orchestration_cost` on a stage with `status: shipped` is **enforced by
`just cost-audit`**, which exits non-zero and names the stage. This **amends
`DEC-013` §5**, which decided the opposite.

`STAGE-001` is exempt (`STAGE_ORCH_COST_GRANDFATHERED`), by the same mechanism
and reasoning as `COST_AUDIT_GRANDFATHERED`.

## Context

⚠ **This record exists because `PATCH-002` made the change and claimed it
decided nothing.** Its own text said *"No `DEC-*`: this adds a gate for a
decision already recorded in `DEC-013` §5; it decides nothing new."* That is
backwards. `DEC-013` §5 reads, verbatim:

> Warn-only, no gate, no view yet: **capture first.**

And the stage template — plus all five `STAGE-00N` files — still told the author,
at the field itself:

> `# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)`

So `PATCH-002` reversed a recorded decision, wrote that it hadn't, and left five
files instructing authors of the opposite. `STAGE-003`, `STAGE-004` and
`STAGE-005` would each have blocked at close on a field their own front matter
called *never a gate*. Raised as `PATCH-002/SB-1` by the independent verify —
the review that only happened because the patch was merged before it ran.

**Why amend rather than revert.** `DEC-013` §5's *"capture first"* was the right
call **at the time it was made**: the field was new, nothing had ever been
recorded in it, and gating an empty field fails every stage on day one. That
condition has expired. What has since been measured:

- `STAGE-001` shipped 2026-08-22 with `sessions: []`, and **no gate, report or
  status line noticed for fifteen days.**
- `STAGE-002`'s close on 2026-09-06 is the **first time the field was ever
  filled**, and it happened because the orchestrator remembered — the
  `brag-step-skipped-at-ship` shape exactly.
- The amount is not marginal: `STAGE-002` measured **~84.2M** tokens of
  orchestration against **187.0M** of delegated spec cost, ≈**31 %** of the
  stage, and spend no spec's `cost.sessions` would ever record.

"Capture first" has now had one capture in three weeks. The evidence it was
waiting for is in.

## Alternatives Considered

- **Leave it warn-only, per `DEC-013` §5 as written.**
  - Why rejected: measured — fifteen days and one shipped stage produced zero
    captures. A warning nobody reads is the `brag-step-skipped-at-ship` shape,
    which this repo has already paid for once (six ships, zero entries, caught
    by a human).

- **Gate it, and say nothing** (what `PATCH-002` actually did).
  - Why rejected: it leaves `DEC-013` §5 and five stage files asserting the
    opposite of the behaviour. A reader who trusts the comment at the field is
    misled at exactly the moment they need it. This is the same defect class as
    `unrun-docs-carry-errors` — a document describing behaviour nobody re-ran.

- **Gate it only for stages created after this date.**
  - Why rejected: adds a second exemption axis to reason about, when the
    grandfather list already handles the one real case (`STAGE-001`) explicitly
    and by name.

## Consequences

- **Positive.** The field can no longer be skipped silently, and the amount it
  captures is a third of a stage's spend. `STAGE-003`/`004`/`005` will each be
  asked for it at close, which is the intent.
- **Negative.** A stage cannot ship without a number, and there will be closes
  where orchestration genuinely spanned sessions with no observable split — the
  `STAGE-001` situation. The escape hatch is the grandfather list, which is
  **deliberately per-id and by name**: adding to it is a visible act, not a
  default. ⚠ If that list starts growing, the gate is wrong, not the stages.
- **Neutral.** `DEC-013` §5's *"a null here is honest; a guess is not"* survives
  intact and is **not** amended. A stage with no observable split should say so
  and use the grandfather list, not invent a figure.

## Validation

Right if `STAGE-003`'s close records a real orchestration figure without anyone
being reminded. Wrong — and this decision should be revisited rather than the
stage exempted — if the grandfather list grows past `STAGE-001`.

## References

- Amends: `docs/decisions/DEC-013-delegated-cost-handback.md` §5 (the template's
  namespace — see AGENTS.md §10)
- `PATCH-002` (the gate), `PATCH-003` (this record, and `SB-1`'s remediation)
- Signal: `brag-step-skipped-at-ship`
