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
  id: HANDOFF-186
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-23
  status: accepted                 # pending | accepted | completed | rejected

task:
  spec_id: SPEC-089

project:
  id: PROJ-002
  stage: STAGE-029
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

# HANDOFF-186: VERIFY: empty and misshapen interpretation tags refuse at develop (SPEC-089)

## Delegation Summary

You are the **verifier** for SPEC-089. Review it cold: you did not build it.

- **Under review:** PR 319 (`feat/spec-089-misshapen-tags`, head `b2cc3b6`), built under
  HANDOFF-185.
- **Your branch:** `claude/spec-089-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-089 implements DEC-078 §7:
- a present interpretation tag is malformed (recorded in `malformed_tags`, field `None`)
  when its count is zero, when a fixed-count tag has another count, or when a colour
  rational has a zero denominator; ColorPlanes is the `CFAPlaneColor` count on a CFA plane
  (3 when absent or malformed) and `SamplesPerPixel` otherwise;
- DEC-078's existing check then refuses it (`MalformedTag`, or `UnsupportedTag` for
  DEC-077's set), with no change to the sets or the order;
- the colour code no longer falls back from a present matrix it cannot use
  (`dual_illuminant`, the ForwardMatrix branch, `resolve_calibration`);
- no new `Error` variant (76), `DEVELOP_VERSION` stays `"8"`, no Tier-B re-pin, and the
  public `Container::scalar` is unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-089-empty-and-misshapen-interpretation-tags-refuse-at-develop.md`.
  Judge the ACs against the spec **as designed in PR 318**
  (`git show origin/main:<spec path>`): ACs 1 to 9, the rule-clause map, the
  distinguishing fixtures.
- **Decisions:** DEC-078 §7 (the amendment this spec implements) and §2 to §4, DEC-077 §3
  (its set refuses as `UnsupportedTag`), DEC-076 §3 (the "only what is used is checked"
  rule that §7.2 reverses), DEC-012 and DEC-015 (tolerant read; absent is not recorded),
  DEC-051 (variant count).
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`.
- **Build handback:** HANDOFF-185's `## Handback`: the red-on-main table, the AC9 mutant
  table, and three deviations (four changed existing tests, `dual_illuminant` validating
  matrices before the illuminant check, a lone misshapen `ForwardMatrix1` now an error).
- **Prior related work:** SPEC-087 (HANDOFF-182 build, HANDOFF-183 verify) built the
  refusal and the `tests/malformed_tags.rs` machinery that `tests/misshapen_tags.rs` copies.
- **Toolchain:** AGENTS.md §5 and §6. Run `just test`, never `just test --all-features`.
  Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use
  `/usr/bin/git` to establish branch and HEAD. Give each mutant run its own source state
  and read the panic message.

## What to check hard

1. **Every AC met, every rule-clause row reaching a test where the output changes.**
2. **AC9 mutants.** Can each fail on its fixture? Re-run those in doubt, at least (d),
   (h), (i) and (j), plus any the spec's distinguishing fixtures make delicate.
3. **Red on main.** Run the new test file against `origin/main`'s `src/`.
4. **The four changed existing tests:** `tests/color_oracle.rs`
   `ac3_an_unused_ill_shaped_calibration_develops` renamed `..._refuses`; the
   zero-denominator `ColorMatrix2` case in `color::tests::degenerate_interpolation_inputs_do_not_panic`;
   and the two `ifd::tests` reader tests whose fixtures moved to a 3-plane CFA sensor.
   Each must follow from DEC-078 §7, not be edited to pass.
5. **No panics on untrusted input** in the new reader code (count arithmetic, ColorPlanes).
6. **`just decisions-audit --changed`.**
7. **DEC-076 amendment note.** §7.2 reverses DEC-076 §3 and nothing recorded it on DEC-076.
   Add a short note pointing at DEC-078 §7.2; run `just decisions-index`.
8. **Gates:** `just test` with the corpus, `lint-ci`, `lint-no-allow`, `fmt-check`,
   `doc-check`, `validate`; SDK lanes SKIPPED if unset; `gh pr checks 319`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including a table of the mutants you re-ran.
- **Commit** this handoff's filled handback and the DEC-076 note to
  `claude/spec-089-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-089 ship --verdict approved` and fast-forward
  `feat/spec-089-misshapen-tags` to the verify tip so PR 319 carries the verify commits.
  Do not merge.
- **Fixes:** tests or docs only on your branch; any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

## Out of Scope

- An unknown `CalibrationIlluminant` code (audit rank 8), several sensor IFDs (rank 9),
  degenerate warp and gains (rank 11), rank 12.
- Extra values beyond the DNG count for `BlackLevel` and `WhiteLevel`.
- Refusing a well-formed singular matrix (DEC-078 §4).
- The 0.2.6 release.

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
