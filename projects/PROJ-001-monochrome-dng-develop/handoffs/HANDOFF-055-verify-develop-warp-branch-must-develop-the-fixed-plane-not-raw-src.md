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
  id: HANDOFF-055
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual verifier model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-022

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
  tokens_total: 3663255            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 15
  branch: claude/brave-wilson-93437b
  pr: null                         # verify opens no PR — it reviews the build's PR #21 cold
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "✅ APPROVED at f412ac3. Independent red-proof reproduced the build's md5 triple (ab3d77→3f0689→ab3d77) and the 243/0/3→241/2/3 suite delta exactly; AC9 stayed GREEN under the sever. tokens_total is a transcript sum DEDUPED BY message.id (32 ids from 82 raw usage objects; 97.1% cache-read) — a FLOOR, written before session close. No per-token USD meter (Max, extra-usage disabled)."
  synced_at: 2026-10-27
---

# HANDOFF-055: develop warp branch must develop the fixed plane not raw src

## Delegation Summary

The orchestrator (architect) handed `SPEC-022` to the verifier for the
**verify** cycle: an independent cold review of the build's PR #21. This file is
filled retrospectively by the orchestrator from the verify session's report
(session `local_49f354db`, branch `claude/brave-wilson-93437b`); the operative
verify contract was that session's brief, not the scaffold below.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-<slug>/brief.md`
- **Stage:** `./projects/PROJ-001-<slug>/stages/STAGE-005-<slug>.md`
- **Spec:** `./projects/PROJ-001-<slug>/specs/SPEC-022-<slug>.md`
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

- Code changes implementing SPEC-022's Acceptance Criteria.
- All failing tests in SPEC-022 now passing.
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

- **Branch / PR:** `claude/brave-wilson-93437b` — verify opened no PR; it
  reviewed the build's PR #21 (`claude/optimistic-leakey-0367f3`) cold.
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** at PR #21 head `f412ac3afd0fc5197ec20e9f9f6bac49c39715e8`.
  - **AC1–AC3** — honest tree `243 / 0 / 3`; both new tier-A tests
    (`develop_warp_branch_develops_the_fixed_plane`,
    `develop_warp_branch_actually_warps`) green and genuinely exercise the
    optical-centre fixed point, the non-vacuous `HIT == 1`, and warp ≠ no-warp
    off-centre with the centre `1200` under both branches.
  - **AC4 red-proof — reproduced independently** (own md5s, scratchpad `rsync`
    copy, working tree untouched): severing the warp arm's `effective_src → src`
    at `src/develop.rs:850` gave md5 `ab3d771288e36b5353c84dcb0e16ee6f` (honest)
    → `3f0689bcbb4ebbf9eb0ea7843454d90f` (severed) → `ab3d77…` (reverted,
    byte-identical). The severed copy compiled and ran;
    `develop_warp_branch_develops_the_fixed_plane` → RED `left 0, right 1000`,
    `develop_warp_branch_actually_warps` → RED `left 0, right 1200`, while
    SPEC-017's AC9 (`develop_output_is_bit_identical_across_two_runs`, the
    no-warp branch) stayed **GREEN** — the control proving the new tests reach a
    mutation AC9 cannot. Suite `243/0/3` → `241/2/3` under the sever; exactly the
    two new tests flip.
  - **Diff hygiene:** zero `src/` change (`tests/develop.rs` the only code file);
    `just decisions-audit --changed` clean (DEC-004 advisory only, consistent);
    CI 22/22 green on the head, observed.
  - **Nothing failed to reconcile** against HANDOFF-054 — all md5s, RED values
    and the `241/2/3` delta matched independently.

### Cost self-report

- **Tokens (total):** **3,663,255** — transcript sum DEDUPED BY `message.id`
  (32 distinct ids from 82 raw usage objects; 97.1% cache-read). A FLOOR,
  written before session close.
- **Estimated USD:** null — Max subscription, extra-usage disabled; no per-token
  meter. The token count is the real figure.
- **Duration (minutes):** ~15.
- **Source of the number:** harness report — the verify session's own transcript
  `usage` objects, located by its scratchpad UUID (`bfb45fe4…`) and summed.
  `/cost` is unavailable in this desktop Code session.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none.
- **Follow-up work identified:** none blocking. (Minor, already transparent: the
  spec/timeline/prompt show as modified-in-PR because the build worktree was cut
  from `main` pre-design-commit and cherry-picked `4bb57a0`.)

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's Inputs recipe and AC4's red-proof protocol were exact;
   the sever site and the expected RED values were spelled out.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-024 (the oracle is blind to this stage) correctly framed why the
   analytic red-proof is load-bearing.

3. **If you did this task again, what would you do differently?**
   — Nothing material; the cold review reproduced every claim on the first pass.
