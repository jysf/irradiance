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
  id: HANDOFF-160
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 5498418          # REAL combined count — what cost-audit reads
  estimated_usd: 4.41            # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 50
  branch: claude/spec-076-verify-base
  pr: null                         # no PR opened (brief); reviews PR 264
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "FLOOR, summed before the handback commit from transcript e907e25f-5337-49b4-a02a-34c292e831b9.jsonl (found by the scratchpad UUID): 41 unique message.ids; input 81 + cache-creation 98,348 (all 1-hour) + cache-read 5,370,199 + output 29,790 (max per message); all claude-opus-5-5, no subagents; opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Re-sum at ship. PUNCH LIST is tests-only: the source at c9d9c7f is correct, and the two surviving mutants (H2 applier-parse allocation, H5 trailing-comment call) are pinned in d79dc50 on this branch."
  synced_at: 2026-11-19
---

# HANDOFF-160: VERIFY: STAGE-019 verify test hardenings (SPEC-076)

## Delegation Summary

You are the **verifier** for SPEC-076. Review it cold: you did not build it.

- **Under review:** PR #264 (`claude/spec-076-build`, head c9d9c7f), built under
  HANDOFF-158. CI was green on all 22 checks.
- **Your branch:** `claude/spec-076-verify-base`. It is that head plus one commit that
  advances the spec to verify and adds this handoff.

SPEC-076 is tests only: five hardenings, H1–H5, each meant to catch a named mutant. The
deliverable being judged is the proof that each strengthened test catches its regression,
and that nothing else changed.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-076-stage-019-verify-test-hardenings.md`
  (H1–H5, AC1–AC6).
- **Build handback:** `HANDOFF-158`'s `## Handback`, which has the mutant table, the
  gates and three deviations. Check each claim yourself.
- **Where the items come from:** the §Follow-up work section and mutant table in each
  of HANDOFF-138, HANDOFF-140 and HANDOFF-142.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them.
  - `just test` already passes `--all-features`.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.
    Red must come from the target assertion, not from an unrelated error.

## What to check hard

1. **Tests only (AC6):**
   - `git diff origin/main -- src/` must touch only the `#[cfg(test)]` module of
     `src/develop.rs`.
   - `DEVELOP_VERSION` must be untouched.
   - No existing assertion may be weakened or deleted. Read every removed line in
     `tests/`, especially the `scratch_develop_allocates_nothing` refactor from bool to
     `Result`, and the H3 comment replacement.
2. **Re-run each mutant** from the build's table yourself: H1, H2, H3 under both
   framings, H4, H5a and H5b. Confirm each one goes red on the intended assertion.
3. **Try to beat each test with a mutant of your own.** For example:
   - H1: a median that sorts only part of the neighbours, or picks the wrong middle for
     an even count.
   - H2: an allocation on a different malformed-opcode path.
   - H3: a warp that runs but is a no-op.
   - H4: a scratch write after the check but before the error returns.
   - H5: the helper call split across lines, or `geometry.orientation` passed in a
     different argument position.

   If a reasonable mutant survives and a small test closes it, add that test on your
   branch.
4. **The deviations:**
   - H3 guards only the AC3 pattern (k = 0), because the narrow-spread case does not
     move under this warp. Is that sound, or is a fixture change needed for the spec's
     "both cases" intent?
   - H2 asserts `Error::Truncated`.
   - H1's fixture location differs from the spec.
5. **H1's hand-written medians:** recompute them yourself from the fixture values.
6. **Gates:**
   - `just test` with the corpus;
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
   - `cost-audit`, `gate-audit`, `name-length-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants.
- **Commit this handoff's filled handback** to `claude/spec-076-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-076-build`.
- **A punch-list fix** is fine on your branch if it is tests only. Anything that would
  touch non-test `src/` goes in the punch list.

## Out of Scope

- The O7/O8 tile reorder.
- The slow 24-pin test.
- The DEC-051 prose sweep.
- Any non-test code change.
- SPEC-075 rework.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your
    scratchpad UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

## Handback

*Filled in by the receiving agent.*

### Execution notes

- **Branch / PR:** `claude/spec-076-verify-base` (no PR opened, per the brief). Reviews https://github.com/jysf/irradiance/pull/264 at c9d9c7f.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** yes, once d79dc50 on this branch is carried into PR 264. AC1 to AC6 hold at c9d9c7f. Two of my own mutants survived the build's whole suite, one against H2 and one against H5. d79dc50 pins both, tests only.
- **Verdict:** ⚠ PUNCH LIST, tests only and already fixed in d79dc50 on this branch. The source at c9d9c7f is correct and nothing in non-test `src/` needs to change. To close it, cherry-pick d79dc50 onto `claude/spec-076-build`, or fast-forward the PR to this branch, as was done for SPEC-075.

#### 1. Tests only (AC6)

- `git diff origin/main...c9d9c7f -- src/` is one hunk at `src/develop.rs` line 2022, inside `mod tests` (`both_gathers_walk_through_the_traversal_helper`). `DEVELOP_VERSION` is not in the diff.
- Every removed line in `tests/` was read. None weakens an assertion:
  - **`black_levels.rs` H3:** the `FirstValue` develop moved out of the `k > 0` branch, and `assert_ne!(first_out, got)` is kept under `k > 0` unchanged. Only the misleading comment was rewritten.
  - **`develop_err`:** scratch is filled with `SENTINEL` instead of 0, and an assertion was added. This is strictly stronger.
  - **`scratch_develop_allocates_nothing`:** the bool became the call's `Result`. Every row still asserts `is_ok() == expect_ok` and zero allocations, so the checks are equivalent. The `Error` is now dropped outside the counted closure, but allocations are counted when they happen, so this hides nothing.

#### 2. The build's mutants, re-checked

Each mutant was applied in its own scratch copy with its own `CARGO_TARGET_DIR`, and I read each panic message. H1 and H3 overlap my own V1 and V5 below, with the same files and targets.

- **H1:** the sort skipped → "interior (1, 1)" 50 vs 40. Reproduced by V2 and V3, which give the same red at the same line.
- **H3:** the warp skipped → "the warp did not run", under StandardCrop and then FullActiveArea, with the framing loop flipped in the copy only. Reproduced by V5.
- **H4:** a scratch write before the error → "scratch written". Reproduced by V6.
- **H2, H5a, H5b:** I did not re-run the build's exact source edits. The fixed tests are supersets of the build's assertions: the H2 row and the first-argument check are untouched, and the line cut removes everything the whole-line filter removed. V4 and V7 below show the new paths red.

#### 3. My mutants

| # | Item | File | Change | Result (message) |
|---|---|---|---|---|
| V1 | H1 | `src/develop.rs` applier | sort only when all eight neighbours are present (the edge pixel is left unsorted) | RED: `fix_bad_pixels_median_of_unsorted_neighbours` "edge (3, 2)" 30 vs 35. The old median test is green |
| V2 | H1 | `src/develop.rs` `lower_median` | upper middle for an even count | RED: "interior (1, 1)" 50 vs 40 |
| V3 | H1 | `src/develop.rs` applier | sort only the first n−1 gathered values | RED: "interior (1, 1)" 50 vs 40 |
| V4 | H2 | `src/develop.rs` `develop_core` | a discarded `format!` on the develop's own `parse_fix_bad_pixels_constant` error (the `MalformedOpcodeParams` path) | **SURVIVED** all 9 alloc tests. The H2 row is refused earlier, by `check_required_processing`, with `Truncated`, so it never reaches this parse. Fixed in d79dc50 with a four-byte-params row: RED "four-byte FixBadPixelsConstant params in OpcodeList1: allocated" 3 vs 0 |
| V5 | H3 | `src/warp.rs` `apply_warp_into` | the warp runs its length checks, then copies its source (a no-op warp) | RED: "blacks [100, 200, 300, 400] StandardCrop α 1: the warp did not run". With the framing loop flipped in the copy: the same message under FullActiveArea |
| V6 | H4 | `src/develop.rs` `develop_with_scratch_into` | scratch[0] written after `develop_core` returns an error, before the error is returned | RED: `pattern_error_order` via `develop_err`: "InvalidWarpAlpha { warp_alpha: 1.5 }: scratch written" |
| V7 | H5 | `src/develop.rs` `crop_orient_normalize_into` | the helper call replaced by a wrapper (defined outside the scanned body) that walks with orientation 1, with `for_each_out_pixel(geometry.orientation,` left in a TRAILING comment on the same line | **SURVIVED** the whole suite: `cargo test --all-features --no-fail-fast` gave 719 passed, 0 failed with the corpus. The scan skipped only whole comment lines. Fixed in d79dc50 by cutting each line at its first `//`: RED "fn crop_orient_normalize_into(: must call the traversal helper once, outside a comment" 0 vs 1 |

The H5 split-across-lines and wrong-position variants from the brief need no mutant. The real calls are already split across lines, and `trim()` removes the newline. `geometry.orientation` in any position other than first fails the first-argument check, as H5a showed.

#### 4. The deviations

- **H3, the k = 0 pattern only: sound, and no fixture change is needed.** The guard proves that the warp pass executes for this sensor, these options, each framing and each α. The pass does not depend on pixel values, so the k = 10 case runs the same code. A mutant that skipped the warp only for narrow-spread input is not a reasonable one. V5 is red under both framings. I did not re-measure the build's "unchanged at k = 10" claim. The build's optional follow-up, a larger fixture so the guard can also run at k = 10, is fine but not needed.
- **H2 asserts `Error::Truncated`: correct.** `check_list` reads the raw framing and refuses the short parameter block as `Truncated`, before any applier. This is also why V4 slipped past, and the second row now covers the applier's `MalformedOpcodeParams`.
- **H1's location: fine.** The fixture sits in `tests/develop.rs` beside the other `apply_fix_bad_pixels_constant` tests, which is where the spec's Design section put it. The ramp fixture the handoff named lives in `tests/develop_scratch.rs`.

#### 5. H1's medians, recomputed

The gather follows `NEIGHBOR_OFFSETS` order, and `lower_median` takes index (n − 1) / 2.

- **Interior (1, 1):**
  - gathered 90, 10, 70, 50, 20, 40, 60, 30;
  - sorted 10, 20, 30, 40, 50, 60, 70, 90;
  - index 3 is **40**, and index 3 of the unsorted gather is 50.
- **Edge (3, 2):**
  - gathered 20, 80, 30, 35, 45;
  - sorted 20, 30, 35, 45, 80;
  - index 2 is **35**, and index 2 of the unsorted gather is 30.

Both match the test. The two zeros are the only bad pixels, and neither is the other's neighbour. V1 and V2 confirmed the unsorted values.

#### 6. Gates (`IRRADIANCE_CORPUS_DIR` set, 12/12 corpus files present)

| Gate | c9d9c7f (build head) | d79dc50 (with the fixes) |
|---|---|---|
| `just test` | 732 passed, 0 failed, 1 ignored | 732 passed, 0 failed, 1 ignored (rows were added, not test functions) |
| `lint-ci` | green | green |
| `lint-no-allow` | green | green |
| `fmt-check` | green | green (after `cargo fmt`, which only re-wrapped lines) |
| `validate` | green | green, at the handback commit |
| `doc-check` | green | green, at the handback commit |
| `cost-audit` | green | green, at the handback commit |
| `gate-audit` | green | green, at the handback commit |
| `name-length-check` | green | green, at the handback commit |

### Cost self-report

- **Tokens (total):** 5,498,418 (a floor, summed before this handback commit; re-sum at ship)
- **Estimated USD:** 4.41
- **Duration (minutes):** 50
- **Source of the number:** transcript `e907e25f-5337-49b4-a02a-34c292e831b9.jsonl`, found by the scratchpad UUID. Deduped by message.id, taking the largest output_tokens for each; no subagents; all claude-opus-5-5. 41 unique message.ids; input 81 + cache-creation 98,348 (all 1-hour) + cache-read 5,370,199 + output 29,790 (max per message). Priced at opus-5-5 rates: input 5 and output 25 per M, cache read 0.1x, 1-hour cache write 2x.

### Drift and new artifacts

- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - none in the verify. The spec asked for "the opcode error" in H2. The build asserts `Truncated` (correct for its row), and d79dc50 adds the applier's `MalformedOpcodeParams` row beside it.
- **Follow-up work identified:**
  - **Required to ship:** carry d79dc50 into PR 264.
  - **Optional:** the build's larger warp fixture, so H3's guard can also run at k = 10.
  - **Optional:** the H5 scan still ignores `/* */` block comments. The codebase does not use them in function bodies, and this is not worth a parser.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing much. The handoff's mutant hints mapped directly onto the gaps, and two of them ("a different malformed-opcode path" and "split across lines/comment") pointed at the two survivors.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The develop has two malformed-`OpcodeList1` refusal points: the framing check in `check_required_processing`, and the applier's own parameter parse. A spec that says "the opcode error" should name which one, because a row for one never reaches the other.

3. **If you did this task again, what would you do differently?**
   — I would start the full-suite run for a source-scan mutant first, since it is the long pole at about 9 minutes, and design the other mutants while it runs.
