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
