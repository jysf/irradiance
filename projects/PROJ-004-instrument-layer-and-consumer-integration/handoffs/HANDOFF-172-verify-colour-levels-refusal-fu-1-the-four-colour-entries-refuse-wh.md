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
  id: HANDOFF-172
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-080

project:
  id: PROJ-004
  stage: STAGE-026
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

# HANDOFF-172: VERIFY: colour levels refusal and two tooling fixes (SPEC-080)

## Delegation Summary

You are the **verifier** for SPEC-080. Review it cold: you did not build it.

- **Under review:** PR #283 (`claude/spec-080-build`, head 71e5758), built under
  HANDOFF-171. CI is green on all 22 checks.
- **Your branch:** `claude/spec-080-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-080 makes the four colour develop entries refuse `W ≤ B` with
`Error::InvalidLevels { black_level: B, white_level: W }` (DEC-070 §3, second amendment),
with no output change; `scripts/sdk-stage2.sh` refuses an output directory inside any git
worktree of the repository; and `just validate` parses `feedback/` front matter and checks
its `status`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-080-colour-levels-refusal-fu-1-the-four-colour-entries-refuse-wh.md`
  (Context 1–3, Design 1–3, AC1–AC8, the rule-clause map).
- **Decisions:** DEC-070 (§1, §3 and both amendments), DEC-068 §2, DEC-051.
- **Build handback:** HANDOFF-171's `## Handback`. Check each claim yourself.
- **The SDK** for the script check: `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Write every SDK output into your scratchpad, **never** into any checkout. The builder
  once wrote outputs into the main checkout to demonstrate the old gap; don't repeat it.
  Compare resolved roots instead.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.
  - Run long suites in the foreground, or keep working until they finish; don't end your
    turn while a run is still going.

## What to check hard

1. **The refusal is complete and in the right place.** Read `colour_range` and every
   branch of `resolve_colour_levels`.
   - `B` is the maximum applied black on every branch; `W` is the colour white
     (`white_levels.first()`, then `white_level`, then the bit-depth default).
   - Nothing is allocated or written before it on any of the four entries; the threaded
     entries refuse on the calling thread. Is there any colour entry or helper (a public
     function that takes a `Sensor` and normalises) that bypasses `resolve_colour_levels`?
     Search for every caller of the normalisers.
   - `normalize_range` is unchanged and still total.
2. **No output moved.** `git diff origin/main -- tests/ src/`: only the pre-approved
   unit test was rewritten; everything else is added. Run `just test` with the corpus, with
   and without the SDK variable: every pin holds; `DEVELOP_VERSION` is `"7"`.
3. **The tests can fail.** Re-run at least three of the builder's five mutants with an
   unmutated control first, then add your own, for example:
   - the error reporting the first black (or the white) in the wrong field;
   - the check placed after `resolve_colour_gain_maps` (does AC3 catch it?);
   - `W` taken from `white_level` instead of `white_levels.first()`;
   - the refusal applied in the f32 entry but not the u16 one.
4. **AC3's fixtures.** Confirm the "malformed `GainMap`" is genuinely refused by the
   `GainMap` check when levels are fine (the builder's control), so the ordering test is
   not vacuous.
5. **The script (AC6).** Read the `git worktree list` logic: paths with spaces, a symlinked
   output path that resolves into a worktree, a relative output path, and the git-absent
   fallback. Reproduce the outside-every-worktree run and the byte-identity of the 7 TIFFs
   against SPEC-079's sha256s. Reproduce at least two refusals by comparing resolved roots,
   without writing into any checkout.
6. **Validate (AC7).** Read the change. Reproduce the broken-YAML and `status: closed`
   failures on a temporary copy; confirm `_template.md` and the front-matter-less file are
   skipped and that the success line's count includes the 9 feedback files. Is the
   "starts with `---`" rule robust (a BOM, CRLF line endings)?
7. **Docs and CHANGELOG.** The four `# Errors` sections and `Error::InvalidLevels` are
   accurate; the CHANGELOG `[Unreleased]` entry names the newly refused input precisely.
   `semver-check` is clean.
8. **Nothing forbidden is committed:** no SDK bytes or outputs, no corpus bytes, no
   `Cargo.lock`.
9. **Gates:** `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`,
   `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`; CI.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including your own mutant table.
- **Commit this handoff's filled handback** to `claude/spec-080-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-080-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. Any
  `src/` change goes in the punch list.

## Out of Scope

- Any output change, the release, `BlackLevelDeltaH/V`, the public DNG writer (PROJ-005).

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

## Handback` section below.** An unfilled handback is an
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
