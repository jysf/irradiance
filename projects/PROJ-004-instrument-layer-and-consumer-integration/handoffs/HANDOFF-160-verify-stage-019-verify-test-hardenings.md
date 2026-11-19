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
  status: pending                  # pending | accepted | completed | rejected

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
