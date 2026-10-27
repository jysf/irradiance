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
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: pending                  # pending | accepted | completed | rejected

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
