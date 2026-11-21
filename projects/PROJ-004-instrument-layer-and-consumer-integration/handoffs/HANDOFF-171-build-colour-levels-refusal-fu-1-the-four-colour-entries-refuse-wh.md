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
  id: HANDOFF-171
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 15197942           # REAL combined count — what cost-audit reads
  estimated_usd: 10.66              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-080-build
  pr: 283
  completed_at: 2026-11-20         # YYYY-MM-DD
  notes: "Floor taken before the handback commit: 85 message ids, opus-5-5 only, no subagents, 1-hour cache writes. Re-sum at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-171: BUILD: colour levels refusal and two tooling fixes (SPEC-080)

## Delegation Summary

You are the **builder** for SPEC-080. **Start only after the SPEC-080 design PR has
merged.** Cut `claude/spec-080-build` from that `origin/main`.

**The task:**
- the four colour develop entries (`develop_color_into`, `develop_color_f32_into`,
  `develop_color_into_threaded`, `develop_color_f32_into_threaded`) refuse `W ≤ B` with
  `Error::InvalidLevels { black_level: B, white_level: W }`, as mono already does (DEC-070
  §3, second amendment), with no output change for any file that develops today;
- `scripts/sdk-stage2.sh` refuses an output directory inside any worktree of the
  repository, found through git;
- `just validate` parses `feedback/` front matter and checks its `status`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-080-colour-levels-refusal-fu-1-the-four-colour-entries-refuse-wh.md`
  (Context 1–3, Design 1–3, AC1–AC8, the rule-clause map).
- **Decisions:** DEC-070 (§1, §3 and both amendments), DEC-068 §2, DEC-051.
- **Background:** SPEC-078's verify ruled the missing colour refusal acceptable for that
  spec and filed it as FU-1 (HANDOFF-168's handback, item 3). SPEC-079's verify noted the
  script's repo-root detection (HANDOFF-170's handback).
- **Code:** `src/color.rs` (`resolve_colour_levels` and its callers, the unit tests),
  `src/develop.rs` (`resolve_levels`, `resolve_levels_with`), `src/lib.rs`
  (`Error::InvalidLevels`), `scripts/sdk-stage2.sh`, `scripts/validate.sh`.
- **The SDK** for AC6's byte-identity re-run: `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  SPEC-079's handback (HANDOFF-169) lists the expected sha256s.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## Expected Deliverables

1. **The change and tests** for AC1–AC7, in a PR against `main` from
   `claude/spec-080-build`, with CI green.
2. **The handback:**
   - the list of assertions that went red against a throwaway refusal before you wrote
     anything (expected: only the pre-approved unit test);
   - the AC5 mutant table, control first;
   - the AC6 and AC7 scenario outcomes, with each message;
   - the `semver-check` verdict.
3. **Gates**, all green: AC8.

## Out of Scope

- Any output, pin or `DEVELOP_VERSION` change, or a new error variant.
- The release (the next spec).
- `BlackLevelDeltaH/V`.
- The public DNG writer (PROJ-005).

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus bytes, SDK bytes or SDK outputs.
- **An existing assertion may be strengthened, never weakened or deleted.** The one
  pre-approved exception is `src/color.rs`'s
  `colour_with_no_range_maps_to_zero_without_panicking`, rewritten as SPEC-080's Failing
  Tests says. Any other assertion or pin that has to move: **stop and report** before
  changing it.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your own
  transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill the
  `## Handback` section.

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

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-080-build` → https://github.com/jysf/irradiance/pull/283. The build commit is `69634d8`; the handback is the next commit.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** Yes. CI is reported in the PR, not here. Its debug test job takes about 37 minutes.

**Assertions that went red against a throwaway refusal, before any test was written.** I ran the full suite with the corpus set, under the corpus profile with no fail-fast. The result was 763 passed, 1 failed and 1 ignored. The only failure was `color::tests::colour_with_no_range_maps_to_zero_without_panicking`, the pre-approved unit test. No other assertion or pin moved.

**The change.**
- `src/color.rs`:
  - A new helper, `colour_range(white, max_black)`, returns `W − B` or `Error::InvalidLevels { black_level: B, white_level: W }` when `W ≤ B`.
  - All three branches of `resolve_colour_levels` call it: the scalar branch, the slots and the general pattern.
  - The call order is unchanged, so the refusal comes after the pattern-shape check and before the `GainMap`s. The threaded entries reach it in `develop_color_bands`, on the calling thread.
  - `normalize_range` is untouched.
- **Docs:**
  - `InvalidLevels` is added to the `# Errors` of all four entries. Each threaded entry also names it explicitly.
  - The resolver doc and the `Error::InvalidLevels` doc in `src/lib.rs` are updated.
  - `CHANGELOG.md` gets an `[Unreleased]` "Changed" entry.
- `DEVELOP_VERSION` is still 7, and the public API's shape is unchanged.

**Tests.**
- **Pre-approved rewrite:**
  - The old unit test is now `colour_with_no_range_is_refused`. It covers each branch: 2×2 with `W = B` and `B` last (the old fixture); 2×2 with `W < B` and `B` second; `[1, 2]`; a general `[3, 3]` pattern with `B` in the middle; the scalar branch; and the first `white_levels` value winning.
  - `normalize_range_stays_total_at_range_zero` checks that range 0 still maps to 0 without dividing.
  - `colour_one_code_of_range_is_accepted_and_clips_at_white` covers AC2 at the normaliser level, against a hand rule.
- **New file, `tests/colour_levels.rs`:**
  - It holds 8 tests, all tier A, run on all four entries with a sentinel-filled `dst`.
  - AC1: `W = B` and `W < B` on each of the 4 branches, where the 2×2 case has `B` at index 1 of 4 and the 4×4 case has `B` at index 6 of 16. Also WhiteLevel 0 with no BlackLevel, straight from a file.
  - AC1, white rule: `white_levels` differs from `white_level`, in both directions.
  - AC2: `W = B + 1` develops. Planes of `W`, `W + 7` and 65535 develop identically on every entry, and a plane of 0 differs.
  - AC3: a wrong `dst` length is reported first, then a wrong BlackLevel count. `W ≤ B` with a malformed `GainMap` (a well-framed `OpcodeList2` whose `GainMap` has 3 parameter bytes) reports `InvalidLevels`. A control first shows the same list alone is `MalformedOpcodeParams { id: 9 }`.
  - Every refusing fixture with several blacks has its first black below `W`. The scalar branch and the white rule are reached by editing the parsed `Sensor`'s public fields, because the test writer cannot separate them.

**AC5 mutant table.** Each copy had its own target dir. Each ran tier A (corpus unset), under the corpus profile with no fail-fast.

| mutant | passed / failed / ignored | red tests |
|---|---|---|
| control (unmutated) | 776 / 0 / 1 | none |
| check removed | 770 / 6 / 1 | unit refusal; W = B; W < B; white rule; WhiteLevel 0; levels before gain map |
| `<` instead of `≤` | 772 / 4 / 1 | unit refusal; W = B; WhiteLevel 0; levels before gain map (all the W = B cases) |
| check against the first black | 771 / 5 / 1 | unit refusal; W = B; W < B; white rule; levels before gain map |
| slot branch only | 770 / 6 / 1 | unit refusal (the `[3, 3]` case); W = B; W < B; white rule; WhiteLevel 0; levels before gain map (scalar and general cases) |
| threaded entry skips it | 771 / 5 / 1 | W = B; W < B; white rule; WhiteLevel 0; levels before gain map, each on `develop_color_into_threaded` (the u16 threaded entry is checked before the f32 one) |

I read each panic message: every red is the intended assertion, either "expected InvalidLevels, got Ok" or a refusal where `Ok` was returned on the named entry. None is a compile or setup failure.

**AC6, `sdk-stage2.sh`.** Every refusal leaves no directory behind.
- **Inside this worktree:** refused, exit 1. The message names the main checkout, because this worktree sits under its `.claude/worktrees/`.
- **Inside another worktree (the main checkout):** refused, exit 1, with "output directory is inside the repository (…/irradiance): …/irradiance/sdk-out-x".
  - Run from this worktree, the old script accepted the same path and wrote SDK outputs into the main checkout. I deleted them at once; nothing there was tracked or committed.
- **Inside a worktree outside the main checkout:** refused. This was a temporary detached worktree in the scratchpad, so the list is what refuses it, not nesting. The worktree was removed and pruned afterwards.
- **git absent from `PATH`** (a `PATH` of symlinks with git left out): stderr says "sdk-stage2: git is unavailable or this is not a git checkout; refusing only the script-relative root (…/spec-080-build-ee8659)". An in-worktree path is then refused against that root.
- **Outside every worktree:** created, and the cold four-file recipe ran (exit 0). All 7 TIFFs are byte-identical to SPEC-079's sha256s (`893d7b94…`, `b2f784d7…`, `b0dc823b…`, `d33afbb4…`, `771609b8…`, `ec2e1a55…`, `3fc24044…`).
- shellcheck finds nothing in the script.

**AC7, `just validate` on `feedback/`.** These ran in a temp copy of scripts, projects, decisions, feedback and the init files; nothing broken was committed.
- **Control:** passes. 373 files parse, 9 of them feedback; the parse count went up by exactly those 9.
- **Broken YAML** (an unterminated quoted scalar in a copy): exit 1, with "front matter did not parse: (<unknown>): found unexpected end of stream while scanning a quoted scalar at line 1 column 9".
- **`status: closed`:** exit 1, with "invalid/missing: feedback status(='closed', expected one of open addressed deferred)". The same happens in `feedback/archive/`.
- **Skipped files:**
  - Broken front matter appended to `_template.md` changes nothing: still 9, still green.
  - `2026-04-20-bragfile-project.md`, which has no front matter, is not a target.
- `validate-red-proof` is still green. shellcheck reports the same 4 pre-existing findings before and after.

**AC8, gates.**
- `just test` with the corpus: 790 passed, 0 failed, 1 ignored. The only SKIP is the SDK line. With `IRRADIANCE_SDK_STAGE2_DIR` set to the fresh SDK outputs: 790 / 0 / 1 with no SKIP.
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`, `gate-audit`, `name-length-check` and `decisions-index --check` are all green.
- `semver-check`: 202 pass, 58 skip, "no semver update required", as expected for a new refusal on an existing variant.
- CI: see the PR.

### Cost self-report

- **Tokens (total):** 15,197,942: input 174, output 54,025, cache read 14,961,024 and cache write 182,719 (all 1-hour), over 85 message ids. This is a floor, taken before the handback commit.
- **Estimated USD:** $10.66 at opus-5-5 rates ($5 in, $25 out, reads ×0.10, 1-hour writes ×2.0).
- **Duration (minutes):** about 25 by wall clock (the transcript runs from 15:13 to about 15:38 local).
- **Source of the number:** my session transcript (`41a5f6b5-5b05-4fd8-8e9f-b081cd0674f5.jsonl`), measured per docs/cost-tracking.md "Measuring a cycle": deduplicated by `message.id`, with the largest `output_tokens` taken. No subagents. The floor was taken just before the handback commit, so re-sum it at ship.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-070's second amendment was already written at design.
- **Deviations from spec:**
  - The public-entry tests live in a new file, `tests/colour_levels.rs`, which the spec allows.
  - AC3's malformed `GainMap` had to be a well-framed list with bad `GainMap` parameters. A truncated list is refused earlier, by the required-processing check (`Truncated`), so it cannot show that levels come before gains.
  - In AC6, "inside this worktree" is refused by way of the main-checkout entry, because worktrees here are nested in the main checkout. I added the outside-worktree case to show the list itself does the refusing.
  - `docs/provenance-ledger.md` is unchanged: no row describes the colour path as total.
- **Follow-up work identified:**
  - (Process, small) This handoff's body has a broken line 136, "## Handback` section below.**", which looks like a template substitution that lost its start. It renders as a stray heading.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — AC3's "malformed GainMap" did not say which malformation. The obvious one, a truncated list, is caught earlier by the required-processing check, so it does not test levels-before-gains. It took one failed run to find this.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Worktrees in this repo are nested inside the main checkout (`.claude/worktrees/`). So the old script's gap was only the main checkout, not sibling worktrees, and an "another worktree" test passes through nesting alone unless the worktree lives outside.

3. **If you did this task again, what would you do differently?**
   — I would not run the OLD script against an in-repo path to demonstrate the gap. It really did write about 60 MB of SDK outputs into the main checkout before I deleted them. Comparing the resolved roots is enough evidence.
