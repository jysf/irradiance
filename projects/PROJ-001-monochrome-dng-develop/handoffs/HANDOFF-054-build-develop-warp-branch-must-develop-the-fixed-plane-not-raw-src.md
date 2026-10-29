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
  id: HANDOFF-054
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual implementer model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  tokens_total: 7430645            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 20
  branch: claude/optimistic-leakey-0367f3
  pr: https://github.com/jysf/irradiance/pull/21
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "Test-only, zero src/ change. tokens_total is a transcript sum DEDUPED BY message.id (43 distinct ids from 126 raw usage objects = 2.93x; 96.8% cache-read) — a FLOOR, written before the session closed. Worktree was cut from main (1cf848d), not from the design commit 4bb57a0 as the handoff assumed; cherry-picked 4bb57a0 (clean, byte-identical parents) to bring SPEC-022 + this handoff into the branch."
  synced_at: 2026-10-27
---

# HANDOFF-054: develop warp branch must develop the fixed plane not raw src

## Delegation Summary

The orchestrator (architect) hands `SPEC-022` to the implementer for the
**build** cycle: add two tier-A tests to `tests/develop.rs` proving
`develop_into`'s warp branch develops the bad-pixel-**fixed** plane, and
discharge the red-proof. **No `src/` change** — this is test-completeness, the
exact warp-branch mirror of SPEC-017's SB-2 (no-warp) assertion.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-022-develop-warp-branch-must-develop-the-fixed-plane-not-raw-src.md`
  — read the whole thing; its **Inputs** section is the fixture recipe (exact
  helpers, coeffs, the optical-centre fixed-point trick) and its **AC1–AC4**
  are the contract. The **Notes for the Implementer** has the red-proof hygiene.
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-005-harden-the-harness-the-gates-the-oracle-s-parser-and-the-second-cargo-graph.md`
- **Project brief:** `./projects/PROJ-001-monochrome-dng-develop/brief.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint
  quirks (pinned vs local clippy), the `+toolchain`/PATH trap, zsh `$pipestatus`
  vs bash `PIPESTATUS`. Read it (DEC-004 rule 5).
- **The code under test:** `src/develop.rs` `develop_into` (the `Some(warp)`
  arm and the `effective_src` resolution), `apply_fix_bad_pixels_constant`;
  `src/warp.rs` `is_identity` / `apply_warp_into`.
- **The mirror to copy:** SPEC-017's SB-2 assertion in `tests/develop.rs`,
  `develop_output_is_bit_identical_across_two_runs` (the `dst1[2*5+2] == 1000`
  line and its fixture). Your test is this, through the warp arm.
- **Helpers to reuse:** `tests/support/opcode.rs`
  (`single_fix_bad_pixels_constant_list`, `single_warp_rectilinear_list`),
  `minimal_sensor` in `tests/develop.rs`.

### Decisions that apply

- `DEC-024` — dnglab/rawler don't implement DNG opcodes, so the SSIMULACRA2
  oracle can't see this stage end-to-end; analytic tier-A assertions like this
  one are the only coverage, which is why the red-proof is load-bearing.
- `DEC-013` — the delegated-cost handback contract: fill the `handback:` block
  with a REAL `tokens_total` before reporting done.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching only `tests/`:

- `oracle-must-be-shown-red` — the new test MUST be proven red by a real
  mutation (sever the warp arm's `effective_src → src`): file changed AND
  compiles AND the asserted output moved. All three legs, both directions
  (revert byte-identical). This is the spec's whole point.
- `test-before-implementation` — the test is the deliverable; there is no
  implementation to precede here, but the red-proof stands in for "shown to
  fail for the right reason."
- `no-panics-on-untrusted-input` — unchanged (no `src/` change); don't weaken
  any existing assertion.

### Prior related work

- **SPEC-017** (shipped, PR #17 / commit `2de4836`) — this spec IS its FU-10.
  SB-2 round 2 added the no-warp centre assertion; HANDOFF-050 documents that
  red-proof's shape (scratchpad-copy mutation, md5 triple, `--no-fail-fast`).
- **SPEC-018** (shipped, PR #16) — WarpRectilinear parser + resampler; its
  `tests/warp.rs` `develop_into_crops_from_the_warped_active_area_not_the_warped_crop`
  is the only existing test that drives `develop_into` through the warp arm.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Two tier-A tests in `tests/develop.rs` (`develop_warp_branch_develops_the_fixed_plane`,
  `develop_warp_branch_actually_warps`) satisfying AC1–AC3, green on the honest
  tree.
- AC4 red-proof discharged in a scratchpad copy and **recorded in the handback**:
  the honest→severed→reverted md5 triple for `src/develop.rs`, proof the new
  test went RED (`developed[12]: left 0, right 1000`) while AC9 stayed GREEN,
  and the `--no-fail-fast` suite delta.
- **No `src/` change** in the committed diff; `git status` clean apart from
  `tests/develop.rs` (and this handoff + the timeline).
- All gates green (fmt, lint-ci on the PINNED clippy, test, deny, deny-fuzz,
  msrv, the standing red-proofs) on the PR head; CI observed green, not inferred.
- A PR against `main` from the build session's branch. PR description references
  this handoff (HANDOFF-054), SPEC-022, STAGE-005, PROJ-001, and DEC-024.

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

- **Any `src/develop.rs` (or other `src/`) logic change.** The warp branch
  already develops `effective_src`; this proves it. The only src edit is the
  throwaway red-proof mutation, fully reverted.
- The oracle-scope narrowing for warp-bearing pixels — that is SPEC-021.
- Sourcing or fabricating a *real* Q2M frame with a bad pixel — the fixture is
  synthetic (a hand-built `Sensor` with both opcode lists set).
- Any new decoder, camera, or format; any lossless-JPEG support.

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

- **Branch / PR:** `claude/optimistic-leakey-0367f3` → https://github.com/jysf/irradiance/pull/21
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** yes.
  - **AC1** — `develop_warp_branch_develops_the_fixed_plane`: `develop_into` on
    the 5×5 whole-plane fixture (bad pixel `0` at the optical centre `(2,2)`,
    non-identity `WarpRectilinear` `kr1 = -0.1`) routes through the `Some(warp)`
    arm and yields `developed[12] == 1000` — the neighbours' lower-median, not
    the raw marker. Green.
  - **AC2** — same test: a direct `apply_fix_bad_pixels_constant` on a copy of
    the raw plane returns HIT count `1` (non-vacuous). Green.
  - **AC3** — `develop_warp_branch_actually_warps`: with a `1000 + 100*x` ramp
    background, `opcode_list_3 = Some(..)` differs from `= None` (the warp
    resampled off-centre) while the optical-centre fixed point develops the fixed
    lower-median `1200` under both. Green.
  - **AC4** — red-proof discharged in a scratchpad copy (`rsync` minus
    `target/`/`.git/`; working tree never touched). Severing the warp arm's
    `effective_src → src` at the `normalize_active_area_into` call:
    ① **file changed** — md5 honest `ab3d771288e36b5353c84dcb0e16ee6f` → severed
    `3f0689bcbb4ebbf9eb0ea7843454d90f`; ② **compiles** — the severed copy built
    and ran; ③ **output moved on the asserted path only** —
    `develop_warp_branch_develops_the_fixed_plane` went **RED** at
    `developed[12]: left 0, right 1000` (its companion red too, at
    `left 0, right 1200`) while SPEC-017's AC9
    (`develop_output_is_bit_identical_across_two_runs`, the no-warp branch)
    stayed **GREEN**. Revert is **byte-identical** (md5 back to
    `ab3d771288e36b5353c84dcb0e16ee6f`). The sever touched only the warp
    branch's use site (`normalize_active_area_into`, ~`src/develop.rs:850`); the
    no-warp site (`crop_orient_normalize_into`, ~`:834`, AC9's path) was left
    intact — which is what makes AC9 the green control.
  - **`--no-fail-fast` suite delta** (both measured without the corpus, the CI
    configuration): honest **243 passed / 0 failed / 3 ignored** → severed
    **241 / 2 / 3**. Exactly the two new tests flip; no other binary moves.
    (Plain `cargo test` fail-stops at the develop binary and would have hidden
    ~190 downstream tests; `$pipestatus` read directly — this shell is zsh.)
- **Committed diff:** `tests/develop.rs` only for the test deliverable (+159
  lines, one commit); the design docs ride in via the cherry-picked `4bb57a0`,
  and this handoff + the timeline carry the build bookkeeping. `src/develop.rs`
  is byte-identical to the honest tree — the only src edit was the throwaway
  red-proof mutation, which lived and died in the scratchpad copy.
- **All eight gates green locally:** `fmt --check`, `lint-ci` (pinned clippy
  0.1.98, `-D warnings`), `test` (243/0/3), `deny`, `deny-fuzz`, `msrv` (1.90.0),
  `lint-red-proof`, `cost-audit-red-proof`.

### Cost self-report

- **Tokens (total):** **7,430,645** — a transcript sum **DEDUPED BY `message.id`**
  (43 distinct ids from 126 raw usage objects = 2.93× inflation; 96.8%
  cache-read). A FLOOR: written before the session closed, so PR follow-up and
  CI observation are not counted.
- **Estimated USD:** null — Max subscription with extra-usage disabled, so there
  is no per-token meter. The token count above is the real figure.
- **Duration (minutes):** ~20 (first transcript line 23:02Z; still open at
  handback).
- **Source of the number:** harness report — the session transcript's own
  per-turn `usage` objects, located by this session's scratchpad UUID
  (`f851eb08…`) and summed (deduped by `message.id`). `/cost` is unavailable as
  a slash command in this desktop Code session; this is its equivalent.

### Drift and new artifacts

- **New decisions emitted:**
  - none.
- **Deviations from spec:**
  - None in substance. One setup correction: the worktree was cut from `main`
    (`1cf848d`), not from the design commit `4bb57a0` the handoff assumed, so
    SPEC-022's spec/handoff/prompt were absent. Cherry-picked `4bb57a0` onto the
    branch (clean — its two modified files were byte-identical between my HEAD
    and its parent), bringing the intended state. No change to the spec's own
    content.
  - AC3's assertion is stated as "the two whole outputs differ while the centre
    agrees under both" (equivalently, they differ only at off-centre pixels) —
    a robust restatement of the spec's "differs at off-centre pixels," not a
    narrowing.
- **Follow-up work identified:**
  - none. The spec's own Non-Goals (SPEC-021 oracle-scope narrowing; a real Q2M
    frame with a bad pixel) are already tracked.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec itself; its Inputs/AC1–AC4 were precise enough to write
   the tests and predict every value (centre 1000/1200, HIT 1, lower-median of 8)
   before running anything. The one real friction was the worktree base: the
   handoff said "your worktree already has HANDOFF-054 and the SPEC-022 spec,"
   but it was cut from `main`, not from the design commit — a one-time
   cherry-pick, noted above, not a spec defect.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red`, `test-before-implementation`, and
   `no-panics-on-untrusted-input` were the right three, and DEC-024 is exactly
   why the analytic red-proof carries the weight here.

3. **If you did this task again, what would you do differently?**
   — Verify the worktree base against the design commit before reading for files
   that "should already be here," rather than discovering the divergence from a
   failed read. The rest — reuse the committed helpers, keep the bad pixel at the
   optical centre, mutate a scratchpad copy — the spec already routed correctly.
