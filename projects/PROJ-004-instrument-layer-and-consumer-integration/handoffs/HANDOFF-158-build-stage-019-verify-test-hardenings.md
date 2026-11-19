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
  id: HANDOFF-158
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-18
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-076

project:
  id: PROJ-004
  stage: STAGE-023
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 6316433               # REAL combined count — what cost-audit reads
  estimated_usd: 4.78              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-076-build
  pr: https://github.com/jysf/irradiance/pull/264
  completed_at: 2026-11-19               # YYYY-MM-DD
  notes: "FLOOR, summed before the handback commit from transcript 2b3b79ac-80fe-432a-94f8-bc7e26a3f647.jsonl (found by the scratchpad UUID): 46 unique message.ids, all claude-opus-5-5, no subagents; input 95, output 24,302 (max per message), cache read 6,183,894, cache write 108,142 (all 1-hour). Opus 5.5 at 5 in / 25 out per M, reads x0.10, 1-hour writes x2.0. Re-sum at ship."
  synced_at: 2026-11-19
---

# HANDOFF-158: BUILD: STAGE-019 verify test hardenings (SPEC-076)

## Delegation Summary

You are the **builder** for SPEC-076. **Start only after SPEC-075's build PR has
merged.** Cut `claude/spec-076-build` from that `origin/main`, because both specs edit
`tests/black_levels.rs`.

**The task:** five tests-only hardenings, H1–H5, from the STAGE-019 verify follow-ups.
Each must catch a named mutant, which you apply by hand, show red, revert and record. No
non-test code changes.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-076-stage-019-verify-test-hardenings.md`.
  H1–H5 give each change and its mutant.
- **Where the items come from:** the §Follow-up work section and mutant table in each
  of HANDOFF-138 (SPEC-065: H1, H2), HANDOFF-140 (SPEC-066: H3, H4) and HANDOFF-142
  (SPEC-067: H5).
- **Code:**
  - `tests/develop.rs`: the `apply_fix_bad_pixels_constant` tests and
    `fix_bad_pixels_median_with_fewer_than_eight_neighbours`.
  - `tests/develop_scratch_alloc.rs`: the error-path list in
    `scratch_develop_allocates_nothing`.
  - `tests/black_levels.rs`: AC5 at about line 493, and `develop_err` at about line 682.
  - `src/develop.rs`: `both_gathers_walk_through_the_traversal_helper` in its test
    module, and `for_each_out_pixel` with its two callers.
- **Toolchain:**
  - AGENTS.md §5–6, with `IRRADIANCE_CORPUS_DIR` as in HANDOFF-157. Use the corpus files
    in place.
  - Do mutants on a local commit you never push, or in the working tree, then revert.

## Expected Deliverables

1. **One commit per item, or one commit for all five,** with tests only. `git diff
   origin/main -- src/` shows only the `#[cfg(test)]` module.
2. **A mutant table in the handback:** item, file:line, the change, the test that went
   red, and confirmation it was reverted.
3. **Gates,** all green with the corpus env set:
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `cost-audit`
   - `gate-audit`
   - `name-length-check`
4. **PR** against `main` from `claude/spec-076-build`, referencing SPEC-076 and
   HANDOFF-158, with CI green.

## Out of Scope

- The O7/O8 tile reorder.
- The slow 24-pin test.
- The DEC-051 prose sweep.
- Any non-test code change.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths.
- Never commit corpus bytes.
- **An existing assertion may be strengthened, never weakened or deleted.** The one
  exception is the misleading AC5 comment that H3 replaces.
- **Fill the `handback:` front matter** with a real `tokens_total` re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates), and
  fill the `## Handback` section below.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-076-build`, https://github.com/jysf/irradiance/pull/264 (cut from `origin/main` at 1e68028, after SPEC-075 merged).
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** yes. AC1 to AC5 each have a mutant shown red (table below). AC6: `git diff origin/main -- src/` touches only the `#[cfg(test)]` module, `DEVELOP_VERSION` is untouched, and every gate is green.

#### Mutant table

Each mutant was applied by hand to its own copy of the tree (scratchpad, never in the worktree, never committed), built with its own `CARGO_TARGET_DIR`, shown red and discarded. The worktree's `src/` was never mutated. Line numbers are on the build commit.

| Item | File:line | Change | Test that went red (message) | Reverted |
|---|---|---|---|---|
| H1 | `src/develop.rs:898` | `valid.sort_unstable()` commented out | `tests/develop.rs` `fix_bad_pixels_median_of_unsorted_neighbours`: "interior (1, 1)" 50 vs 40. The old ramp fixture `fix_bad_pixels_median_with_fewer_than_eight_neighbours` stayed green (`--no-fail-fast`), which is the gap | yes (copy discarded) |
| H2 | `src/develop.rs:1469` | the `check_required_processing` error mapped through a discarded `format!` | `tests/develop_scratch_alloc.rs` `scratch_develop_allocates_nothing`: "truncated FixBadPixelsConstant in OpcodeList1: allocated" 3 vs 0. Parser-level `opcode_lookups_allocate_nothing` stayed green | yes |
| H3 | `src/develop.rs:1693`, `:1714` | both `apply_warp_into` calls replaced by a copy of the normalized area into the warp buffer | `tests/black_levels.rs` `black_is_applied_before_the_warp`: "blacks [100, 200, 300, 400] StandardCrop α 1: the warp did not run". With the framing loop flipped in the mutant copy only: the same message under FullActiveArea | yes |
| H4 | `src/develop.rs:1494` | first scratch sample set to 0 just before `resolve_levels_with` (the pattern check) | `tests/black_levels.rs` `pattern_error_order` through `develop_err`: "InvalidBlackLevelPattern { repeat_rows: 2, repeat_cols: 2, count: 3 }: scratch written" | yes |
| H5a | `src/develop.rs:1005` | `crop_orient_normalize_into` passes `1` instead of `geometry.orientation` | `src/develop.rs` `both_gathers_walk_through_the_traversal_helper`: first argument "1" vs "geometry.orientation". The old count checks ran first and passed | yes |
| H5b | `src/develop.rs:1129` | `crop_and_orient_from_active_into`'s call replaced by a plain row walk defined outside the scanned body, with the call text left in a `//` comment | the same test: "must call the traversal helper once, outside a comment" 0 vs 1. The old count check ran first and passed | yes |

A first H3 mutant, which set the plan's warp to `None`, went red for the wrong reason (`DevelopScratchWrongLength`, because the scratch layout still paid for a warp). It was replaced by the pass-level skip above, which goes red on the H3 assertion itself.

#### Gates (`IRRADIANCE_CORPUS_DIR` set, 12/12 corpus files present)

| Gate | Result |
|---|---|
| `just test` | 732 passed, 0 failed, 1 ignored (221 s) |
| `lint-ci` | green |
| `lint-no-allow` | green |
| `fmt-check` | green |
| `validate` | green |
| `doc-check` | green |
| `cost-audit` | green |
| `gate-audit` | green |
| `name-length-check` | green |

### Cost self-report

- **Tokens (total):** 6,316,433 (a floor, summed before this handback commit; re-sum at ship)
- **Estimated USD:** 4.78
- **Duration (minutes):** 25
- **Source of the number:** transcript `2b3b79ac-80fe-432a-94f8-bc7e26a3f647.jsonl`: 46 message.ids deduped (max output each), no subagents, all claude-opus-5-5. Input 95, output 24,302, cache read 6,183,894, cache write 108,142 (all 1-hour), at opus-5-5 rates.

### Drift and new artifacts

- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - H3's warp-ran guard runs on the AC3 pattern only (k = 0). The narrow-spread case leaves FirstValue values of 10 to 13, which this small warp on the 12x11 plane does not move: identical with and without it, as measured. The AC3 case alone covers both framings and both α values. The existing `assert_ne!(first_out, got)` is kept, still under k > 0. Only the misleading comment was replaced.
  - H2's truncated list surfaces as `Error::Truncated` from the required-processing check, not a dedicated opcode variant. The row asserts that variant. To do so, `scratch_develop_allocates_nothing` now keeps each call's `Result` rather than a bool. Every existing row is asserted exactly as before.
  - H1's fixture lives in `tests/develop.rs` beside the other applier tests. The old ramp fixture is in `tests/develop_scratch.rs`, not `tests/develop.rs` as the handoff said.
- **Follow-up work identified:**
  - none required. Optional: a narrow-spread warp fixture large enough for the warp to move pixels would let H3's guard run on both AC5 cases.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The handoff placed the ramp fixture in `tests/develop.rs` when it is in `tests/develop_scratch.rs`. The H2 "opcode error" variant wasn't named, but one run settled it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The H3 guard needs a fixture the warp can visibly move. The narrow-spread AC5 case can't, so "both cases" wasn't achievable as written. A mutant also has to keep the scratch layout consistent, or it goes red on the length check instead of the target assertion.

3. **If you did this task again, what would you do differently?**
   — I would read each mutant's panic message before calling it red. The first H3 mutant failed for the wrong reason, and only the message showed it.
