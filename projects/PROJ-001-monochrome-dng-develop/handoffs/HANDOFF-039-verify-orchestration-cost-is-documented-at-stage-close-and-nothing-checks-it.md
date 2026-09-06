---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-039
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # PREDICTION from tier_map.verify, not a measurement.
                                    # Correct it to what your system prompt reports.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: PATCH-002

project:
  id: PROJ-001
  stage: STAGE-XXX
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-039: Verify PATCH-002 — the stage orchestration-cost gate, at `705c784`

## Delegation Summary

Verify `PATCH-002` at **`705c784`** on `fix/patch-002-orchestration-cost-has-no-gate`
(pushed, not merged; `main` at `781930f`). CI 9/9 on that SHA, run `34023570708`.

⚠ **The orchestrator wrote this patch.** Build normally goes to a separate
session here; it did not. **Review it as work by someone who was also grading
it** — that is the whole reason this verify is worth its cost.

## What the patch does

`cost-audit` now fails when a stage with `status: shipped` has an empty
`orchestration_cost`. The template has said *"THE ORCHESTRATOR FILLS THIS"* since
2026-08-15 and nothing checked it: `STAGE-001` shipped 2026-08-22 with
`sessions: []` and no gate, report or status line noticed for fifteen days.
`STAGE-002`'s close on 2026-09-06 is the first time the field was ever filled.

Not a rounding error: `STAGE-002` measured **~84.2M** tokens of orchestration
against **187.0M** of delegated spec cost — roughly **31 %** of the stage, and
spend no spec's `cost.sessions` would ever record.

## ⚠ Attack the red-proof first — I already shipped one false version of it

Its first draft was wrong, and how it was wrong is the most useful thing here.
The injection wrote a bare `sessions: []`, which **also deleted the template's
commented example** (`# - tokens_total: N`). With that text gone, even a naive
`grep -q tokens_total` implementation *passed* the proof — nothing left to
false-match. That is AGENTS.md §16's *"the obvious test exercises the wrong
path"*, verbatim. The injection now reproduces the real shipped shape, comment
included, and asserts the comment survived.

**Do not take my word that it is fixed.** Three mutations are claimed caught:

| mutation | claimed |
|---|---|
| `stage_has_orchestration_cost` always returns "filled" | red-proof FAILS |
| the naive `grep -q tokens_total` implementation | red-proof FAILS |
| the reason string emptied | red-proof FAILS |

Reproduce all three, then **find a fourth I did not think of.** Candidates: an
`orchestration_cost` block absent entirely rather than empty; an entry whose
`tokens_total` is `null`; a real YAML entry sitting *outside* the block;
`status: shipped` written with odd spacing or quoting.

## Your own checks

1. **Is the grandfathering honest, or hiding a live failure?** `STAGE-001` is
   exempt via `STAGE_ORCH_COST_GRANDFATHERED`. Confirm it is load-bearing
   (remove it — STAGE-001 must fail) *and* justified: is reconstructing
   STAGE-001's orchestration genuinely impossible, or merely inconvenient? §4
   says *"a null here is honest; a guess is not"*, and I claimed reconstruction
   would be a guess. **Test that claim rather than accepting it.**
2. **Does the gate fire on the states that matter?** It keys on
   `get_stage_status = "shipped"`. What about `cancelled`? A stage file outside
   `projects/*/stages/`? Judge the scope, not just the code's match to it.
3. **Does `find_all_stages` find every stage?** `-maxdepth 1` under
   `projects/*/stages` — confirm that matches where `just new-stage` puts them.
4. **The JSON surface.** `--json` emits `missing_cost: ["orchestration"]`.
   Confirm it is well-formed and that the human line and JSON cannot drift — I
   collapsed them to one source *because* a mutation showed they could.
5. **Does the new CI step actually execute?** I verified by reading the job log
   (step `cost-audit goes red on an unfilled stage orchestration_cost`, printing
   its own success line). Confirm independently — a step that exists in YAML and
   never runs is this patch's own subject.
6. **`shellcheck`.** Clean on the new script, unchanged in count on the two
   edited ones. Re-run it; I am not confident I checked every warning class.

## Context

- **Patch:** `projects/PROJ-001-monochrome-dng-develop/patches/PATCH-002-*.md` —
  its Problem section carries the measurements and the near-miss that shaped the
  detector.
- Changed: `scripts/_lib.sh` (three helpers), `scripts/cost-audit.sh` (a third
  loop), `scripts/cost-audit-red-proof.sh` (new), `.github/workflows/` (one
  step), `app.just` + `AGENTS.md` §6.
- `DEC-013` §5 — the rationale the gate enforces. **No new `DEC-*`**: this
  decides nothing, it enforces something already decided.
- Constraints: `cost-captured-per-cycle`; and `oracle-must-be-shown-red` by
  analogy — this repo's rule is that a gate ships proven red.

## Out of Scope

- **Backfilling `STAGE-001`.** If you judge the grandfathering unjustified, raise
  it as a finding; do not fill it.
- The gate-count ambiguity and `handback-sync`'s truncation — both filed signals.
- Opening or merging the PR; running `handback-sync`.

## Return Criteria

1. **Gates, run by you**, pasted, clippy version asserted, and **say which list
   you ran** — the count is ambiguous and that is a filed signal, not yours to
   resolve.
2. **Observe CI green on the SHA you approve.**
3. **All three claimed mutations reproduced**, plus your attempts at a fourth.
   Each: file changed **and** ran **and** *output changed*.
4. ⚠ **Mutate in an isolated copy.** I lost this very patch to `git checkout --`
   mid-mutation and recovered it from a scratch backup — SPEC-010's exact
   failure. There is a `wip` commit in the history for that reason.
5. Handback with a real `tokens_total`, deduped by `message.id`, per-component,
   **rounded up ~20 %**. ⚠ **`notes:` on ONE PHYSICAL LINE** — `handback-sync`
   truncates multi-line scalars and leaves front matter unparseable while every
   gate reports green.
6. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
7. Findings `SB-N`/`FU-N` from `FU-1` with proposed §15 dispositions.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
