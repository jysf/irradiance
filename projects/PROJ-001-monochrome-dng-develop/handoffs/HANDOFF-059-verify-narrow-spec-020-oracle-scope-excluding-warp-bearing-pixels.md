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
  id: HANDOFF-059
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual verifier model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-021

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
  tokens_total: 3715683            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 15
  branch: claude/admiring-ramanujan-d14395
  pr: null                         # verify opens no PR — it reviews the build's PR #26 cold
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "APPROVED at b37a70e. Exercised both reframed tests against the REAL corpus (non-vacuous): warp_pixels_are_below_the_srgb_oracle_baseline measured L1021223 -60.169 / L1026192 -40.281 < 83.145 and skips the Orientation-6 frame loudly; rotated_frame_is_out_of_the_srgb_oracle_scope asserts real dims 8368x5584 != 5584x8368. src diff comment-only (0 logic), #[ignore] 3->1, CI 11/11 green. Flagged the build handback tokens_total 360000 as a budget-delta (non-comparable). This figure is a FLOOR deduped by message.id (28 ids); no per-token USD meter."
  synced_at: 2026-10-27
---

# HANDOFF-059: narrow spec 020 oracle scope excluding warp-bearing pixels

## Delegation Summary

The orchestrator (architect) handed `SPEC-021` to the verifier for the
**verify** cycle: an independent cold review of the build's PR #26. Filled
retrospectively by the orchestrator from the verify session's report (session
`local_a3e14680`, branch `claude/admiring-ramanujan-d14395`); the operative
verify contract was that session's brief, not the scaffold below.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-<slug>/brief.md`
- **Stage:** `./projects/PROJ-001-<slug>/stages/STAGE-005-<slug>.md`
- **Spec:** `./projects/PROJ-001-<slug>/specs/SPEC-021-<slug>.md`
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

- Code changes implementing SPEC-021's Acceptance Criteria.
- All failing tests in SPEC-021 now passing.
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

- **Branch / PR:** `claude/admiring-ramanujan-d14395` — verify opened no PR; it
  reviewed the build's PR #26 (`claude/spec-021-build`) cold.
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** at PR #26 head `b37a70eb68b2b1beb0707b231e237caa7f7e2f0a`.
  - **AC1** — DEC-026 (DEC-022 shape): the ≥85 clause AND the Validation premise
    quoted verbatim (diffed exact against DEC-005), "why amend not revert" note,
    warp + Orientation out-of-scope with measured evidence + DEC-024/FU-6/FU-2
    cites; DEC-005 points back. `decisions-index --check` and `decisions-audit`
    exit 0.
  - **AC2** — `oracle-contract.md` layer-3 row annotated + a new warp/Orientation
    carve-out paragraph citing DEC-024 F2 + DEC-026.
  - **AC3 — reframe is non-vacuous, exercised on the REAL corpus (both dnglab +
    corpus present on the verify host):** `warp_pixels_are_below_the_srgb_oracle_baseline`
    ran its `< 83.145` assertion on measured scores L1021223 `-60.169` /
    L1026192 `-40.281`, and skips L1026016 (Orientation 6) loudly;
    `rotated_frame_is_out_of_the_srgb_oracle_scope` runs `assert_ne!` on real
    dims (dnglab `8368×5584` ≠ ours `5584×8368`) behind an `orientation !=
    Some(1)` guard, replacing the old latent `assert_eq!` panic with a loud
    `return None` skip. `tests/warp.rs` has 0 `#[ignore]`; repo-wide **3 → 1**.
  - **AC4** — `src/` diff is comment-only (`src/warp.rs`, 6 `//!` lines, 0 logic);
    AC4 (503.7/437.7/407.0 px) + AC10 (339.5 px) untouched, pass corpus-free; no
    duplicate red-proof. **AC5** — cross-refs repointed; `src/lib.rs` legitimately
    untouched (grep-confirmed no stale ref).
  - **CI:** all 11 gates green on `b37a70e` (×2 = 22/22). Local CI-config suite
    245 / 0 / 1 ignored.

### Cost self-report

- **Tokens (total):** **3,715,683** — transcript sum DEDUPED BY `message.id`
  (28 ids / 92 raw records, 3.29×; 96.2% cache-read; all `claude-opus-4-8`). A
  FLOOR — note-writing and the final report are undercounted.
- **Estimated USD:** null — Max subscription, extra-usage disabled; no per-token
  meter.
- **Duration (minutes):** ~15.
- **Source of the number:** harness report — the verify session's own transcript
  `usage` objects, located by its scratchpad UUID and summed.

### Drift and new artifacts

- **New decisions emitted:** none by verify (the build emitted DEC-026).
- **Deviations from spec:** two extra comment-only cross-ref fixes beyond the
  three named in AC5 (documented, all comment-only).
- **Follow-up work identified:** none blocking. ⚠ The **build** handback's
  `tokens_total: 360000` is a remaining-budget delta, not a transcript sum —
  flagged non-comparable (the `token-counts-not-comparable` class, cf.
  SPEC-017/FU-9). Recorded as-is with this caveat rather than invented.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec named the exact inversion to check and the two hunts
   (vacuous pass, hidden src logic); both came up clean.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` being pre-discharged by the shipped AC10 was
   correctly stated, so no duplicate red-proof was expected or added.

3. **If you did this task again, what would you do differently?**
   — Nothing material; having dnglab + corpus on the host let the reframed
   characterization be exercised for real rather than read.
