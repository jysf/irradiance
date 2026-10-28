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
  id: HANDOFF-063
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual verifier model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-023

project:
  id: PROJ-001
  stage: STAGE-005
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
  status: completed                # completed | blocked | rejected
  tokens_total: 3812629            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 13
  branch: claude/gallant-murdock-f59084
  pr: null                         # verify opens no PR — it reviews the build's PR #31 cold
  completed_at: 2026-10-28         # YYYY-MM-DD
  notes: "APPROVED at 69d9a1b2. Independent cold review: re-ran the red-proof and every gate and wrote its OWN injections for all three offenders (unreadable .md -> validate RED by name; superseded handoff skipped vs flipped-to-pending rc 1; DEC grep-guard aborts vs reaches). AC1-AC5 all hold; src/ diff EMPTY; just lint-ci + 13 CI jobs green on 69d9a1b2; decisions-audit 0 structural errors. tokens_total is a transcript SUM deduped by message.id (scratchpad-UUID anchored, cache-inclusive, all claude-opus-4-8) -- a floor, not a budget delta."
  synced_at: 2026-10-28
---

# HANDOFF-063: audit every gate script for the mute-death shape

## Delegation Summary

The orchestrator (architect) handed `SPEC-023` to the verifier for the
**verify** cycle: an independent cold review of the build's PR #31. Filled
retrospectively by the orchestrator from the verify session's report (session
`local_1bfbd0f7`, branch `claude/gallant-murdock-f59084`); the operative verify
contract was that session's brief, not the scaffold below.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-<slug>/brief.md`
- **Stage:** `./projects/PROJ-001-<slug>/stages/STAGE-005-<slug>.md`
- **Spec:** `./projects/PROJ-001-<slug>/specs/SPEC-023-<slug>.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `HANDOFF-YYY` — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-023's Acceptance Criteria.
- All failing tests in SPEC-023 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-XXX-<slug>`.
- PR description referencing: this handoff ID, the spec ID, the stage
  ID, the project ID, all referenced `DEC-*`, and any new `DEC-*`
  created during implementation.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- ...

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/gallant-murdock-f59084` — verify opened no PR; it
  reviewed the build's PR #31 (`claude/spec-023-build`) cold.
- **Completed at:** 2026-10-28
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** at PR #31 head `69d9a1b2fcc5997ff29f3bff513d41604500eea6`.
  - **AC2 — red-proof run independently** (own runs, not the CI badge):
    `gate-audit-red-proof.sh` exit 0, read line-by-line — temp-copy only, a clean
    negative control, three mute-death shapes caught by name + slug, comment
    exclusion, and isolation (remove the injection → green); working tree clean
    before and after. It injects into a copied gate and *requires* the find, so a
    scan-nothing gate fails it — not vacuous.
  - **AC1** — `just gate-audit` exit 0, 8 gates scanned; the scan-count defense is
    layered (glob > 0 + four named anchors; `EXAMINED == handed` + engine-rc
    guard). An independent injection into `decisions-index.sh` (untouched by the
    red-proof) was flagged `[unguarded-grep]` — it genuinely bites.
  - **AC3** — each offender proven by hand: (a) an unreadable `.md` turns `just
    validate` **RED by name** (`Permission denied @ rb_sysopen`) where the
    reconstructed old oracle swallowed the same `Errno::EACCES` into green; (b) a
    superseded handoff is skipped (rc 0) vs flipped-to-pending (rc 1), and
    `HANDOFF-038` is de-kludged (`synced_at`/notes → null); (c) an unguarded grep
    aborts (rc 1) vs `|| true` reaches (rc 0). `just decisions-audit` → 0
    structural errors, 9 pre-existing scope warnings.
  - **AC4** — `test.sh` 19/19 grep guards (no logic change); `just cost-audit` +
    its red-proof pass; `cost-audit.sh` correctly NOT flagged.
  - **AC5** — both commands in AGENTS.md §6 + `app.just`; `git diff
    origin/main -- src/` EMPTY; `just lint-ci` exit 0; **CI green on `69d9a1b2`**
    — 13 named jobs (incl. `cost-capture audit` running both new steps).

### Cost self-report

- **Tokens (total):** **3,812,629** — transcript sum over the verify session's
  scratchpad-UUID JSONL, DEDUPED BY `message.id` (all `claude-opus-4-8`,
  cache-inclusive). A FLOOR, a transcript sum — not a budget-delta.
- **Estimated USD:** null — Max subscription, extra-usage disabled; no per-token
  meter.
- **Duration (minutes):** ~13.
- **Source of the number:** harness report — the verify session's own transcript
  `usage` objects, located by its scratchpad UUID and summed.

### Drift and new artifacts

- **New decisions emitted:** none by verify.
- **Deviations from spec:** none. The `scanned`-vs-`parsed` wording in
  `validate.sh` is strictly stronger than the spec's literal Notes assert and is
  documented.
- **Follow-up work identified:** none blocking. `next_id` scanning test fixtures
  stays a `watch` signal (`next-id-scans-test-fixtures`); the two new CI steps
  live inside the existing `cost-capture audit` job (a documented pattern, not a
  finding).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking; the ACs named the exact offenders and the proof each
   needed, so the review was reconciliation, not discovery.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` (widened to gates by SPEC-003/FU-14) was the
   operative rule and it was cited.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The one surprise was PR #31 being merged to `main` before
   verify ran; the cold review stands as the retrospective check.
