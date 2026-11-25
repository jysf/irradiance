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
  id: HANDOFF-196
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-094

project:
  id: PROJ-005
  stage: STAGE-017
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

# HANDOFF-196: VERIFY: STAGE-017's three follow-ups, bundled (SPEC-094)

## Delegation Summary

You are the **verifier** for SPEC-094. Review it cold: you did not build it.

- **Under review:** PR 339 (`claude/spec-094-build`, head `e833553`), built under
  HANDOFF-195 by a separate session. CI was 23 of 23 green at that head.
- **Your branch:** `claude/spec-094-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-094 verify`.

SPEC-094 closes three items, one commit each:
1. `da50aed`: `WriteInputError::EmptyOpcodeList { list }` refuses `Some(&[])` for any opcode
   list. It runs under every framing, first in the opcode step, before the `OpcodeList1`
   walk, in `plan`.
2. `5f411c2`: a `rust / fuzz smoke — dng_write (60s)` CI job.
3. `e26aeb6`: three writer doc links written as plain code, so a no-feature
   `--document-private-items` build no longer fails on them.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md`.
  Judge it against the spec **as designed** (`git show 6eab1b8:<spec path>`): AC1 to AC7,
  Design §1 to §3, the rule-clause map, and the distinguishing fixtures.
- **Decisions:** DEC-081 §2 and its Follow-up; DEC-075 §5, §7 and §9; DEC-080; DEC-051 and
  DEC-052.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `library-not-application`, `no-copyleft-dependencies`.
- **Build handback:** HANDOFF-195's `## Handback`, in the same handoffs directory. It has the
  red proof, the before and after doc-command table, the one moved assertion, and three
  deviations.
- **Prior verifies:** HANDOFF-190, 192 and 194. Their mutant tables are the model.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`.
  - Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
    `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Use `/usr/bin/git` for branch and HEAD.
  - Give each mutant its own source state. Plain cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC is met against the spec as designed,** and every rule-clause-map row reaches a
   test that fails when its clause is wrong.
2. **Mutants.** Each must fail a named test; give a table. At least these:
   - check only list 1;
   - check only under the cutting framings, so FullSensor accepts an empty list;
   - check only in `serialize`, which breaks parity;
   - move the check after the `OpcodeList1` walk;
   - check the lists in the order 3, 2, 1;
   - refuse `len < 4` instead of `len == 0`;
   - refuse a 4-byte count-0 list;
   - report the wrong `list` number;
   - drop the check entirely.
3. **The build's three deviations.** Rule on each:
   - (a) One existing assertion moved: `opcode_list_1_allow_list_under_cutting_framings` went
     from `MalformedOpcodeList` to `EmptyOpcodeList`. The design said no such assertion
     existed. Is this the only one? Grep for every way a zero-length opcode list can be built,
     including through variables and helpers, not only the literal `Some(&[])`.
   - (b) There was no CI-job list to update, so the AGENTS.md recipe block now names the job.
     Is anything else stale, such as a list of fuzz targets, of CI jobs, or of gates in a
     handoff template?
   - (c) `every_refusal_is_typed` covers only StandardCrop. Is the per-framing coverage in the
     new test enough?
4. **Red proof.** The build added the variant first, without the check, and saw 5 failures.
   Re-run that: revert only the check from `da50aed` and show which tests fail.
5. **The CI job.** Compare it line by line with `fuzz-ifd`, `fuzz-plane` and `fuzz-ljpeg`:
   toolchain, `cargo-fuzz --locked`, corpus dir, seeds dir and `-max_total_time=60`. Does it
   build the target with the features the target needs, as the local recipe does? Read its
   log on the PR and confirm it ran for about 60 s on the real seeds, and was not a no-op.
6. **Doc links.** Re-run AC6's command yourself, without features and with
   `--all-features`. Confirm that no `crate::dng` link is left, and that the 13 `ljpeg`
   errors are the only ones. Rule on the plain-code choice for `src/lib.rs`: is it right that
   the comment never reaches public docs?
7. **No panics, no allocation.** `dng_write_alloc` still passes. The fuzz target reaches the
   new refusal: the `refuse-empty-opcode-list` seed, and empty lists from the fuzzer's own
   length byte.
8. **Docs.** Check the module docs' "Checks and their order", the reason's doc comment and
   `Display`, the CHANGELOG, and DEC-081's Follow-up bullet ("resolved by SPEC-094").
9. **Gates.** Read `gh pr checks 339` first. Then run:
   - `just test` with the corpus and `dng_validate` set;
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`;
   - `deny`, `deny-fuzz`, `lint-fuzz`, **`lint-fuzz-red-proof`**, `msrv`;
   - `validate`, `gate-audit`, `cost-audit`, `semver-check`;
   - `just decisions-audit --changed`;
   - 60 s of `just fuzz-dng-write`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and your rulings on items 3,
  5 and 6.
- **Commit** this handoff's filled handback, and any tests or docs fixes, to
  `claude/spec-094-verify-base`, and push it. If you approve:
  1. Run `just advance-cycle SPEC-094 ship --verdict approved`.
  2. Run `just handback-sync SPEC-094` in the same push, so CI's cost audit passes.
  3. Fast-forward `claude/spec-094-build` to the verify tip, so PR 339 carries the verify
     commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` or `.github/` change goes in the
  punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** before minting a DEC or HANDOFF, compute the next free one from `origin/main`
  plus the open PRs. Right now those are DEC-083 and HANDOFF-197.

## Out of Scope

- The 13 `ljpeg` private doc links, and any CI gate on `--document-private-items`.
- README's fuzz recipe list. The build named it as a follow-up for the release spec.
- Walking `OpcodeList2` or `OpcodeList3`, or changing the `OpcodeList1` allow-list.
- Any reader change.
- The STAGE-017 close. The orchestrator does it.

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
