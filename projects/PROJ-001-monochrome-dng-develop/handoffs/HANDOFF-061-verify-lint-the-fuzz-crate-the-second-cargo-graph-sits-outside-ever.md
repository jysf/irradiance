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
  id: HANDOFF-061
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual verifier model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-011

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
  tokens_total: 1806524            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 15
  branch: claude/cool-booth-9bd68d
  pr: null                         # verify opens no PR — it reviews the build's PR #28 cold
  completed_at: 2026-10-28         # YYYY-MM-DD
  notes: "APPROVED at 06ae10e. Ran the red-proof itself (not the CI badge): control clean -> injected unnecessary_cast into opcode.rs caught at exit 101 with the clippy help-URL match -> the same copy WITHOUT -D warnings exits 0, proving -D warnings is the only teeth (the fuzz crate has no #![deny]). src/+fuzz_targets diff EMPTY; CI 26/26; decisions-audit clean. One cosmetic non-issue (AGENTS.md section 6 lint-fuzz block drops a trailing 2>&1 - still runnable, not a finding). tokens_total is a FLOOR deduped by message.id (16 ids) - a transcript sum, not a budget-delta; no per-token USD meter."
  synced_at: 2026-10-28
---

# HANDOFF-061: lint the fuzz crate — the second cargo graph sits outside every -D warnings gate

## Delegation Summary

The orchestrator (architect) handed `SPEC-011` to the verifier for the
**verify** cycle: an independent cold review of the build's PR #28. Filled
retrospectively by the orchestrator from the verify session's report (session
`local_21cdd314`, branch `claude/cool-booth-9bd68d`); the operative verify
contract was that session's brief, not the scaffold below.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-<slug>/brief.md`
- **Stage:** `./projects/PROJ-001-<slug>/stages/STAGE-005-<slug>.md`
- **Spec:** `./projects/PROJ-001-<slug>/specs/SPEC-011-<slug>.md`
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

- Code changes implementing SPEC-011's Acceptance Criteria.
- All failing tests in SPEC-011 now passing.
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

- **Branch / PR:** `claude/cool-booth-9bd68d` — verify opened no PR; it reviewed
  the build's PR #28 (`claude/nostalgic-taussig-d61c60`) cold.
- **Completed at:** 2026-10-28
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** at PR #28 head `06ae10e08f3ee03aabc21971f1b34b7d98f001ae`.
  - **AC1** — `just lint-fuzz` exit 0 (pinned clippy 0.1.98); the root `clippy` CI
    job has no `--manifest-path`, the new `clippy-fuzz` job adds it — two
    invocations, both required.
  - **AC2 — red-proof run independently** (own runs, not the CI badge): control
    (unmutated fuzz copy) clean; an injected `unnecessary_cast` into
    `fuzz_targets/opcode.rs` is caught at **exit 101** with the clippy
    `index.html#unnecessary_cast` help-URL match and the span on the injected
    line; the **same copy without `-D warnings` exits 0**, proving `-D warnings`
    is the only teeth (the fuzz crate has no `#![deny]`). Not vacuous; working
    tree untouched.
  - **AC3** — DEC-027 records the scope (default `-D warnings`, not the five),
    extends and cites DEC-011; DEC-011 has the back-pointer; INDEX 26→27;
    `decisions-audit` 0 structural errors.
  - **AC4** — both recipes with commands in AGENTS.md §6's block.
  - **AC5** — `git diff origin/main -- src/ fuzz/fuzz_targets/` EMPTY; whole diff
    within the allowed set; **CI 26/26 pass** on head `06ae10e`.

### Cost self-report

- **Tokens (total):** **1,806,524** — transcript sum over the verify session's
  scratchpad-UUID JSONL, DEDUPED BY `message.id` (16 unique ids, all
  `claude-opus-4-8`, cache-inclusive). A FLOOR, a transcript sum — not a
  budget-delta.
- **Estimated USD:** null — Max subscription, extra-usage disabled; no per-token
  meter.
- **Duration (minutes):** ~15.
- **Source of the number:** harness report — the verify session's own transcript
  `usage` objects, located by its scratchpad UUID and summed.

### Drift and new artifacts

- **New decisions emitted:** none by verify (the build emitted DEC-027).
- **Deviations from spec:** none.
- **Follow-up work identified:** none blocking. One cosmetic non-issue (not a
  finding): AGENTS.md §6's `lint-fuzz` block drops the recipe's trailing `2>&1` —
  still runnable.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec named the exact hunt (a vacuous red-proof, a hidden
   src/fuzz-target change); both came up clean.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` (widened to gates) correctly framed the
   red-proof requirement.

3. **If you did this task again, what would you do differently?**
   — Nothing material; running the red-proof on the real host confirmed the
   gate rather than reading it.
