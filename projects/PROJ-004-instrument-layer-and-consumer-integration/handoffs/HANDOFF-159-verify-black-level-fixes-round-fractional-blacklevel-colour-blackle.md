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
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **Verdict:** ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** <transcript path, method>

### Drift and new artifacts

- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
