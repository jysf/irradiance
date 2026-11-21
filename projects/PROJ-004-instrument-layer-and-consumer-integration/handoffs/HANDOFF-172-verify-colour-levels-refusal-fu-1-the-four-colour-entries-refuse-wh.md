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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed  # completed | blocked | rejected
  agent: claude-opus-5-5  # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved  # verify handoffs: approved | punch-list | rejected
  basis: transcript  # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 8706914  # REAL combined count — what cost-audit reads
  estimated_usd: 6.33  # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 95
  branch: claude/spec-080-verify-base
  pr: 283
  completed_at: 2026-11-20  # YYYY-MM-DD
  notes: "APPROVED at 71e5758 (PR 283 head). Floor taken before the commit from transcript 9a5f7349-ad15-4fd7-812c-812b69377011.jsonl, 54 message ids, no subagents; re-sum at ship. Pushed by refspec because worktree monorail-000-backlog-f34c1a holds the branch."
  synced_at: 2026-11-20
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

*Filled in by the receiving agent (cold verifier, claude-opus-5-5).*

### Execution notes

- **Branch / PR:** verdict and handback on `claude/spec-080-verify-base` (pushed by refspec;
  no PR opened). Under review: PR #283, head `71e5758`, CI 22/22 green, mergeable.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** yes, AC1 to AC8.
- **For `verify`:** ✅ **APPROVED at `71e5758`** (the PR #283 head; this branch adds only the
  handoff and this handback on top).

**1. Refusal complete and in the right place.**
- `colour_range(white, max_black)` refuses `W ≤ B` with `InvalidLevels { black_level: B,
  white_level: W }`. All three branches of `resolve_colour_levels` call it:
  - scalar: `B = black_level.unwrap_or(0)`;
  - slots: `B` is the max of the four slot blacks, which covers the 1×1, 1×2, 2×1 and 2×2 shapes;
  - general: `B` is the max of every value.
- `W` is `white_levels.first()`, then `white_level`, then the bit-depth default. It is unchanged.
- Order on all four entries is unchanged: required processing, buffers, levels, GainMaps.
- The threaded entries resolve the levels in `develop_color_bands` on the calling thread,
  before any band. Their fallbacks go through the single-thread entries, so they refuse too.
- Callers of `slot_normalizer`, `pattern_normalizer` and `normalize_range`: only
  `with_normalizer!` in the colour path (fed by `resolve_colour_levels`), the mono path
  (already refusing), and unit tests.
- No other public function takes a `Sensor` and normalises. `to_srgb_linear_into` takes
  camera RGB; `demosaic_into` applies no levels; `apply_gain_maps` takes no `Sensor`.
- `src/develop.rs` is untouched, so `normalize_range` is unchanged and still total.

**2. No output moved.**
- In `git diff origin/main -- tests/ src/`, the only removed code lines are:
  - the pre-approved unit test;
  - the three `saturating_sub` range lines, now in `colour_range`;
  - doc lines.
- `tests/colour_levels.rs` is all additions.
- `just test` with the corpus: 790 passed / 0 failed / 1 ignored, both with
  `IRRADIANCE_SDK_STAGE2_DIR` unset (SKIP line shown) and set to my scratchpad SDK outputs
  ("present — the dng_validate cross-check will run").
- `DEVELOP_VERSION` is `"7"`.

**3. Mutants.** Tier A, `cargo test --all-features --profile corpus --no-fail-fast`. Each copy
came from `git archive` into the scratchpad and had its own `CARGO_TARGET_DIR`. I read every
panic message.

| mutant | result (pass/fail/ign) | red tests |
|---|---|---|
| control, unmutated | 776/0/1, green | none |
| B1 check removed (builder's) | 770/6/1 | `colour_with_no_range_is_refused` + 5 in `colour_levels` ("expected InvalidLevels, got Ok") |
| B2 `<` for `≤` (builder's) | 772/4/1 | the W = B tests (equal, WhiteLevel 0, GainMap order, unit) |
| B3 first black (builder's) | 771/5/1 | the slot/4×4 cases ("W = B = 5000: ()" — Ok where Err was expected) |
| B5 threaded entries skip (builder's) | 771/5/1 | the same 5 integration tests, naming `develop_color_into_threaded` |
| V1 error fields swapped (mine) | 773/3/1 | the W < B cases; W = B is symmetric, as expected |
| V2 levels after GainMaps, all 3 sites (mine) | 775/1/1 | only `the_levels_are_refused_before_a_malformed_gain_map`, so AC3 is what catches the reorder |
| V3 W from `white_level` first (mine) | 774/2/1 | `the_colour_white_is_the_first_whitelevel_value` + the unit test |
| V4 u16 single entry skips, f32 refuses (mine) | 771/5/1 | 5 integration tests, naming `develop_color_into` |
| V5 scalar branch unchecked (mine) | 770/6/1 | the scalar cases + the unit test |

The builder's three counts reproduce exactly (4, 5, 5).

**4. AC3 fixtures.** The control half of `the_levels_are_refused_before_a_malformed_gain_map`
asserts `MalformedOpcodeParams { id: 9, .. }` with valid levels. It passes on the control.
Every mutant that refuses nothing shows `left: "MalformedOpcodeParams { id: 9,
declared_size: 3 }"`. So the GainMap really is refused when the levels are fine, and the
ordering test is not vacuous. V2 proves it bites.

**5. Script (AC6).**
- The real `dng_validate` ran into a scratchpad directory whose path holds a space: exit 0,
  7 TIFFs. All 7 sha256s equal SPEC-079's table (HANDOFF-169) byte for byte.
- Refusal cases. Each used `/usr/bin/true` as the validator, so a check that wrongly let a
  path through would still have written no SDK output. Each refused before `mkdir`, and a
  check afterwards found nothing left behind:
  - a relative path in this worktree;
  - an absolute path in the main checkout;
  - a path in another worktree (`monorail-000-backlog-f34c1a`);
  - a scratchpad symlink whose parent resolves into the main checkout;
  - an output path that is a symlink to an existing worktree directory;
  - `../../../x`, which escapes into the main checkout.
- A dangling symlink into a worktree dies at `mkdir`, creating nothing.
- With git absent from `PATH`, the fallback message is printed. This worktree is still
  refused, and an outside directory is created.
- Outside any checkout, the fallback message is printed and the script-relative root is refused.
- A throwaway repo with spaces in both the main and the worktree paths:
  - each worktree is refused from the other;
  - a `/tmp` spelling resolving to `/private/tmp` is refused;
  - a sibling-prefix directory (`wt with space2`) is correctly accepted;
  - a prunable worktree's own path is refused.

**6. Validate (AC7).**
- `just validate` is green: "374 … parse (ruby), 9 of them feedback with a valid status".
- On a scratchpad `git archive` copy:
  - broken YAML fails, naming the file and the parser message;
  - `status: closed` fails, naming the file;
  - `status: closed` in `feedback/archive/` fails (10 feedback counted);
  - a broken `_template.md` is skipped (green, 9);
  - broken prose appended to the front-matter-less file is skipped (green, 9).
- Robustness (non-blocking, see follow-ups):
  - CRLF: detection works, but the status read keeps the `\r`, so a valid `status: open` is
    falsely rejected with a confusing message (`status(='open', expected one of open …)`);
  - a quoted `status: "open"` is falsely rejected;
  - a BOM-prefixed file is silently not parsed, visible only as a count that does not
    rise.
  - No current feedback file is affected: all are LF, BOM-less and unquoted.

**7. Docs and CHANGELOG.**
- The four `# Errors` sections list `InvalidLevels` in the right position: after the
  pattern check, before the GainMaps.
- The threaded entries say it is refused on the calling thread.
- `Error::InvalidLevels` now describes `B` as the maximum black on both paths.
- The CHANGELOG `[Unreleased]` entry names the four entries, the maximum black, all three
  pattern shapes, the old all-zero behaviour, and `DEVELOP_VERSION` staying 7.
- `semver-check`: 202 pass, 58 skip, "no update required".

**8. Nothing forbidden committed.**
- The diff against `origin/main` touches 9 files: `CHANGELOG.md`, 3 PROJ-004 docs, 2
  scripts, `src/color.rs`, `src/lib.rs` and `tests/colour_levels.rs`.
- No `Cargo.lock`, TIFF, DNG, SDK output or corpus bytes are in the diff or the tree.
- My SDK outputs, mutant copies and validate copies live only in the session scratchpad.

**9. Gates.** All green locally:
- `lint-ci` (0 warnings), `lint-no-allow`, `fmt-check`, `validate`;
- `doc-check` (2 surfaces), `cost-audit`, `gate-audit` (13 scripts);
- `name-length-check` (892 paths), `decisions-index --check` (70);
- `semver-check` as above.

CI on PR #283 is 22/22 SUCCESS.

### Cost self-report

- **Tokens (total):** 8,706,914 (input 125, cache write 1-hour 138,274, cache read 8,541,463,
  output 27,052). This is a floor measured before the commit; re-sum at ship.
- **Estimated USD:** 6.33 at opus-5-5 rates ($5 in, $25 out, reads ×0.10, 1-hour writes ×2).
- **Duration (minutes):** ~95
- **Source of the number:** transcript `9a5f7349-ad15-4fd7-812c-812b69377011.jsonl`, found by
  the scratchpad UUID. It is deduped by `message.id` (54 ids), taking the largest
  `output_tokens`. There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none found.
- **Follow-up work identified** (none blocks the verdict):
  - `scripts/validate.sh` feedback status read: strip a trailing CR and surrounding quotes
    before the vocabulary check, and treat a leading UTF-8 BOM as part of the `---` test (or
    reject it loudly). It is scripts only and small; it fits the release spec or a patch.
  - `scripts/sdk-stage2.sh`: under `set -e`, a worktree directory that exists but cannot be
    entered makes the command substitution fail. The script then exits with no message.
    It is unlikely; add an `|| printf` fallback.
  - DEC-070 §3 (second amendment) says mono and colour "refuse the same malformed levels".
    That is true against mono `PerPosition`. Mono's default `FirstValue` checks the first
    black, not the maximum, so wording for the release notes should not overclaim.
  - The AC3 control half checks `develop_color_into` only. The other three entries are
    covered by the second half, and V2 shows the test bites, so this is optional.
  - Process: line 168 of this handoff is the same broken template heading the builder
    reported on HANDOFF-171 (a half-substituted "## Handback" fragment).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only the branch checkout: `claude/spec-080-verify-base` is held by the orchestrator
   worktree, so `git checkout -B` fails. I used a local tracking branch and pushed by refspec.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The "never write SDK outputs into any checkout" rule plus `/usr/bin/true` as the
   validator made the refusal checks safe without writing anything.

3. **If you did this task again, what would you do differently?**
   — I would start the mutant batch before the first `just test`. They contend for CPU, and
   the mutants were the long pole.
