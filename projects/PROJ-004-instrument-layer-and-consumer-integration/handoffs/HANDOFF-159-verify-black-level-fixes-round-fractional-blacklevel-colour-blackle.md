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
  id: HANDOFF-159
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-075

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
  tokens_total: 6781670            # REAL combined count — what cost-audit reads
  estimated_usd: 5.30              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 35
  branch: claude/spec-075-verify-base
  pr: null                         # no PR opened (brief); reviews PR 262
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "FLOOR, summed before the handback commit from transcript 2a423e07-a9d4-43ac-9eb2-a77a6f0f00ff.jsonl (found by the scratchpad UUID): 43 unique message.ids, all claude-opus-5-5, no subagents; input 86 + cache-creation 142,865 (all 1-hour) + cache-read 6,616,298 + output 22,421 (max per message); opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Re-sum at ship. PUNCH LIST is tests-only: the source at 365b55e is correct, and the four test gaps are fixed in 3e5dd05 on this branch."
  synced_at: 2026-11-19
---

# HANDOFF-159: VERIFY: Black-level fixes and MaskedAreas (SPEC-075)

## Delegation Summary

You are the **verifier** for SPEC-075. Review it cold: you did not build it.

- **Under review:** PR #262 (`claude/spec-075-build`, head 365b55e), built under
  HANDOFF-157.
- **Your branch:** `claude/spec-075-verify-base`. It is that head plus one commit that
  advances the spec to verify and adds this handoff.
- **Before you start:** CI on #262 was green on all 22 checks.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-075-black-level-fixes-round-fractional-blacklevel-colour-blackle.md`
  (AC1–AC9).
- **Build handback:** `HANDOFF-157`'s `## Handback`, which gives the evidence for each
  AC, the red-proofs and four deviations. Check each claim yourself rather than
  trusting it.
- **Decisions:**
  - `DEC-068`, all of it. This is the contract.
  - `DEC-061` §3: the pattern rule and `InvalidBlackLevelPattern`.
  - `DEC-012`: an interpretation tag costs only itself.
  - `DEC-040`: the `ActiveArea` origin.
  - `DEC-058` §5: the version rule.
  - `DEC-009`: red-proof, control first.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them.
  - `just test` already passes `--all-features`, so don't pass the flag again.
  - Give each mutant copy its own `CARGO_TARGET_DIR`. The builder got one false green
    from a shared target dir.

## What to check hard

1. **Rounding (DEC-068 §1):**
   - Check `(2·num + den) / (2·den)` in `u64`, with an exact half rounding up.
   - A zero denominator must still be malformed, and must cost only 50714.
   - `black_levels_exact` must be filled only for RATIONAL.
   - The builder added a fallback for values above `i32::MAX` that the spec did not
     list. Judge whether it is sound and consistent with DEC-012.
2. **The colour pattern (DEC-068 §2):**
   - The index must be exactly `B[((y−top) mod R)×C + ((x−left) mod C)]`. Check it for an
     odd origin on the slot path and on the general path.
   - The count rule must be refused on all four entry points before `dst` is written.
   - White must be per sample.
   - The general (`R` or `C` > 2) path must be correct when threaded and in f32.
   - Look for a hot-loop regression on the slot path.
3. **The fixture deviation:** 17 legacy colour tests went red and were fixed by adding
   `black_level_repeat_dim = Some([2, 2])` to four fixture helpers. The orchestrator
   approved this.
   - Confirm that no expected value or assertion changed.
   - Confirm that `[2, 2]` makes those fixtures mean what they meant before. Under the
     old `level_at`, 4 values were read as a 2×2 slot table.
4. **MaskedAreas (DEC-068 §3):**
   - All six malformed classes, plus the 16 cap.
   - A rectangle at exactly the plane edge is accepted.
   - Malformed costs only 50830, and the rest of `Sensor` is unchanged.
   - The read allocates nothing unbounded before the cap check.
5. **Byte identity (AC8):** run `just test` with the corpus. Every pinned hash and oracle
   must be unedited, apart from the four `DEVELOP_VERSION` pins, which go from 4 to 5.
   Read `git diff origin/main -- tests/` and confirm it.
6. **Red-proofs:** re-run at least one red-proof yourself for each of AC3, AC4 and AC5.
   Add a mutant of your own where you think the tests are weak.
7. **Gates (AC9):**
   - `just test`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`;
   - `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check`;
   - `semver-check`, `decisions-index --check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence for each AC, any mutants you ran, and the gate
  results.
- **Commit this handoff's filled handback** to `claude/spec-075-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-075-build`.
- **A punch-list fix** is fine on your branch if it is small and tests-only. Anything
  that touches `src/` goes in the punch list instead.

## Out of Scope

- A fractional black in develop.
- `BlackLevelDeltaH`/`V`.
- A colour `BlackLevels` option.
- Using `MaskedAreas` in develop.
- GainMap.
- SPEC-076's hardenings.
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

- **Branch / PR:** `claude/spec-075-verify-base`. No PR opened, per the brief. Under review: https://github.com/jysf/irradiance/pull/262 (head 365b55e).
- **Commits on this branch:**
  - `3e5dd05`: tests only. Four test gaps the verify mutants found.
  - The handback commit (this file and the timeline's verify line).
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes for the source. AC1 to AC9 hold at 365b55e. Four checks the handoff asked for were not pinned by any test; `3e5dd05` adds those tests, tests-only.
- **Verdict:** ⚠ PUNCH LIST (tests-only, already fixed on this branch).
  - **Nothing in `src/` needs to change.** 365b55e's source is correct as built.
  - **The punch:** carry `3e5dd05` into #262, for example by cherry-picking it onto `claude/spec-075-build`, or merge this branch instead. Then #262 is approvable as is.

**The punch list (all four fixed in 3e5dd05):**

Each mutant was applied in an archived copy with its own `CARGO_TARGET_DIR`. Each was green on the build's suite (lib, `sensor_levels`, `colour_black_pattern`, `color_oracle`, `threaded`) and red with the new test.

| # | Mutant (survived the build's suite) | Handoff item | New test, red against the mutant |
|---|---|---|---|
| P1 | The `develop_color_f32_into` general-pattern arm drops the `ActiveArea` origin (`top 0, left 0`). | 2: general path correct in f32 | `four_by_four_pattern_takes_the_general_path_in_f32`, through both f32 entries. Red: single and threaded f32 disagree. |
| P2 | White is taken from `white_levels.last()` instead of `.first()`. | 2: white per sample | `white_is_per_sample_not_per_position`: `WhiteLevel [60000, 30000, 30000, 30000]` must develop as `[60000]`. Red: the outputs differ. |
| P3 | The MaskedAreas read drops `bottom > height`. | 4: DEC-068 §3 classes | `"bottom > height"` case in `malformed_masked_areas_cost_only_the_tag`. Red. |
| P4 | The MaskedAreas read drops `left >= right`. | 4: DEC-068 §3 classes | `"left == right"` case in the same table. Red. |

No existing assertion was changed. Both test files are fmt-clean, and `lint-ci`, `lint-no-allow` and `name-length-check` are green with them.

**The seven areas:**

1. **Rounding (DEC-068 §1):** confirmed.
   - `round_rational` computes `(2·num + den) / (2·den)` in `u64` with checked arithmetic, so an exact half rounds up: `2049/2` gives 1025.
   - A zero denominator gives `None` from `round_rational`, then `MalformedRationalValue`, which `cost_the_field` costs once as 50714. `black_level`, `black_levels` and `black_levels_exact` all fall to their absent values.
   - `black_levels_exact` is filled only on the RATIONAL arm. SHORT and LONG return an empty exact array.
   - `black_level` is `rounded.first()` of the same read, so the scalar and the array cannot disagree. This replaces the old pair of reads (`scalar` plus `values`). It is behaviour-identical for SHORT, LONG, count 0 and malformed.
   - **The `i32::MAX` fallback is sound and consistent with DEC-012.** On `RationalValueOutOfRange` it takes the old `uints` read, so such a file reads exactly as on main, and a failure still costs only the tag. One minor asymmetry, not a defect: a non-integral value above `i32::MAX` stays malformed instead of rounding. No real file is near that range.
   - **My mutant M5** (half rounds down, `+ den − 1`) is red in `fractional_black_level_rounds_to_nearest` and in the colour fractional test.
2. **The colour pattern (DEC-068 §2):** confirmed.
   - **The slot path** fills slot `(ro, co)` with `B[(ro mod R)·C + (co mod C)]`. The `slot_normalizer` closure body is unchanged apart from taking `levels` as an argument. Parity is still taken from the `ActiveArea` origin with `wrapping_sub`, so it is right for an odd origin. Each enum arm builds its own closure and is monomorphised, so there is no per-pixel `match`: no hot-loop regression.
   - **The general path** uses `pattern_index`, Euclidean remainders in `i64` from `(top, left)`: exactly `B[((y−top) mod R)·C + ((x−left) mod C)]`. The AC5 hand table (`[3, 2]`, origin top 1 left 3) and the `[4, 4]` odd-origin equivalence pin it in u16, single and threaded. With P1, also in f32.
   - **The count rule** goes through the shared `black_pattern_dims`, the same check the mono path now calls. It is resolved in `develop_color_into`, `develop_color_f32_into` and `develop_color_bands` (the threaded path, from the calling thread) before `dst` or any scratch buffer is touched. The threaded entries fall back to the single-thread ones, which check it too. AC6 is sentinel-proven on all four entries.
   - **One behaviour note, not a defect:** on a colour entry, `InvalidBlackLevelPattern` now comes before `NotAColorFilterArray`, `UnsupportedCfaPattern` and `MissingColorCalibration`. The rustdoc lists it in that order.
   - **White** is the first `white_levels` value, else `white_level`, else `2^bits − 1`, the same for every photosite. P2 pins it.
3. **The fixture deviation:** confirmed benign.
   - `git diff origin/main -- tests/color_oracle.rs tests/threaded.rs` is exactly four added lines, each `black_level_repeat_dim = Some([2, 2])`. No expected value or assertion changed.
   - **`[2, 2]` keeps their meaning.** The old `level_at` read slot `pos = ro·2 + co` from a 4-value array. The new slot fill is `B[(ro mod 2)·2 + (co mod 2)]`, the same index. White was one value in each fixture, and the old `last()` equals the new `first()` for one value.
   - The only other 4-value fixture in `tests/` (`colour_metadata_oracle.rs:694`) is metadata-only and never developed.
4. **MaskedAreas (DEC-068 §3):** confirmed.
   - The type, count 0, count not a multiple of 4, and the 16 cap are all checked before `uints` is called, so the read allocates at most 64 values.
   - An empty rectangle (`top ≥ bottom` or `left ≥ right`) and a rectangle past `bottom > height` or `right > width` are malformed. An edge-exact rectangle reads.
   - Malformed pushes only 50830, and `rest_of` shows every other field and the plane unchanged.
   - Width and height are now read before `MaskedAreas`. They are still read before `BitsPerSample`, so the order of the structural errors is unchanged.
   - I did not re-run exiftool. I accepted the build's pasted output, which agrees with the read tests.
5. **Byte identity (AC8):** confirmed.
   - `IRRADIANCE_CORPUS_DIR=…/images just test` on 365b55e+a71d5e6: **729 passed, 0 failed, 1 ignored**, 42 binaries, 12/12 corpus files present, 0 SKIP lines, EXIT 0.
   - `rational_black_levels_read_rounded_and_exact` passes. The Pixel 7 Pro gives `Some(1024)`, and the Canon gives 4 exact values.
   - **`git diff origin/main...HEAD -- tests/`** changes:
     - the four `DEVELOP_VERSION` pins, `"4"` to `"5"` (`black_levels.rs:247`, `develop_options.rs:308`, `orientation_tiles.rs:290`, `warp_alpha.rs:340`), each with a DEC-068 message;
     - the four fixture lines above;
     - one new tier-B test in `ifd_reader.rs`;
     - the two new test files.

     No pinned hash and no oracle was edited. `grep 'DEVELOP_VERSION, "'` finds exactly those four pins.
6. **Red-proofs**, each in an archived copy with its own `CARGO_TARGET_DIR`:
   - **AC3 (M1, `black_level_values` forced through `uints`):** 4 red in `sensor_levels` (`fractional_black_level_rounds_to_nearest`, `pixel_7_pro_black_levels_read_exact_and_rounded`, `integral_rational_black_level_keeps_exact_values`, `fractional_black_develops_as_its_rounded_value`), plus `fractional_black_develops_as_its_rounded_value_in_colour`. This matches the build's claim.
   - **AC4 (M2, the old `level_at` slot read: `B[ro·2+co]`, else the last value):** `one_by_two…` and `two_by_one…` are red, and `four_by_four` stays green, as expected for a slot-only mutant.
   - **AC5 (M3, `pattern_index` ignores the origin):** `general_black_pattern_index_matches_hand_table` and `four_by_four_pattern_takes_the_general_path` are red.
   - **My own:** M5 (half rounds down) is red. M4, M6, M7 and M8 survived and became P1–P4 above.
7. **Gates (AC9),** all green on this branch:
   - **Lint and format:** `fmt-check` (after my tests too), `lint-ci` (clippy 0.1.98 pinned, `-D warnings`), `lint-no-allow`, `lint-fuzz`.
   - **Repo checks:** `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check`.
   - **`semver-check`:** 202 pass, 58 skip, no semver update required.
   - **`decisions-index --check`:** up to date (68).
   - **CI on #262:** 22/22 green, per the orchestrator. I did not re-query it.

### Cost self-report

- **Tokens (total):** 6,781,670 (a floor, summed before this commit)
- **Estimated USD:** 5.30
- **Duration (minutes):** about 35
- **Source of the number:** the transcript `2a423e07-a9d4-43ac-9eb2-a77a6f0f00ff.jsonl`, found by the scratchpad UUID.
  - 43 unique `message.id`s, taking the max output for each; no subagents; all `claude-opus-5-5`.
  - Input 86, 1-hour cache-creation 142,865, cache-read 6,616,298, output 22,421.
  - Priced at opus-5-5 rates.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the source. The four fixture lines (approved) and the fourth version pin (`warp_alpha.rs`) are as the build reported.
- **Follow-up work identified:**
  - [XS] A RATIONAL `BlackLevel` value above `i32::MAX` still takes the old integral read, so a non-integral one stays malformed instead of rounding. Leave it unless a real file shows up; if one does, round from the raw `u32` pair in `u64`.
  - [process] The spec's failing-test list had no test for per-sample white or the f32 general path, though the handoff named both. A design checklist line, "every DEC rule clause maps to a test, including each malformed class", would have caught P1–P4 before build.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The handoff's seven items were precise. The only judgement call was whether test-only gaps make a punch list or an approval with notes. I chose punch list, because the handoff named those exact properties.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The spec's AC7 named six malformed cases, but DEC-068 §3 implies eight checks (`bottom > height` and `left ≥ right` too). The AC should list every clause of the DEC it pins.

3. **If you did this task again, what would you do differently?**
   — I would write my own mutants first, one per DEC clause, and batch them in one background script from the start. That approach found all four gaps cheaply. Giving each copy its own target dir from the outset avoided the false green the builder hit.
