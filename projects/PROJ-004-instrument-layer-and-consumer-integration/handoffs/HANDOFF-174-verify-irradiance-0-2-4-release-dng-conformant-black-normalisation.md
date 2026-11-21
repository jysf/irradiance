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
  id: HANDOFF-174
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-083

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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10601568               # REAL combined count — what cost-audit reads
  estimated_usd: 7.73              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: claude/spec-083-verify-base
  pr: null
  completed_at: 2026-11-20               # YYYY-MM-DD
  notes: "APPROVED at 5a53497, which is PR 287 head 91119b9 plus the verify setup fe2deec and the verify doc fix 5a53497. src and Cargo.toml are byte-identical to 91119b9. The merge the tag goes on must include 5a53497. FLOOR, from transcript 7b54cae4-4d37-4044-b912-30dbc61a9e65.jsonl, 55 unique message ids, all claude-opus-5-5, no subagents: input 109, cache-creation 177,357 (1-hour), cache-read 10,393,891, output 30,211 (max per message). Opus-5-5 rates: in 5, out 25 per M; cache read 0.1x, 1-hour write 2x. The handback commit and push turns are not in it, so re-sum at ship. Duration is a wall-clock estimate."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-174: VERIFY: irradiance 0.2.4 release prep, Phase 1 (SPEC-083)

## Delegation Summary

You are the **verifier** for SPEC-083. Review it cold: you did not build it.

- **Under review:** PR #287 (`claude/spec-083-build`, head 91119b9), built under
  HANDOFF-173. CI is green on all 22 checks.
- **Your branch:** `claude/spec-083-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

Phase 1 of the 0.2.4 release: two small script fixes, version 0.2.4, the CHANGELOG
promotion, the release notes, tier-B and SDK evidence, the package checks and the
pre-flight. Nothing is tagged or published; that is the user's Phase 2. Your verdict
decides whether the user tags this tree.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-083-irradiance-0-2-4-release-dng-conformant-black-normalisation.md`
  (Release Scope's "every input whose result changes" list, Build work 1–9, Release Notes,
  Pre-Flight, the Phase 2 runbook).
- **The model:** SPEC-077 (0.2.3) and its verify, HANDOFF-166. That verify found the release
  notes missed a newly refused input; look for the same class of miss here.
- **What's released:** DEC-070 (both §3 amendments), SPEC-078, SPEC-080, and the scripts from
  SPEC-079/080/083.
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Write outputs into your scratchpad only, never into any checkout.
- **Toolchain:** AGENTS.md §5–6. Run `just test`, never `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Run long suites in the foreground, or keep working until they finish; don't end your turn
  while a run is still going.

## What to check hard

1. **Semver, and what it can't see.** Re-run `just semver-check`, default and
   `--all-features`. Then read `git diff v0.2.3 origin/claude/spec-083-build -- src/`
   yourself, every `Err(` path and every changed return value:
   - no `pub` item removed or changed; no new dependency, feature or MSRV change;
   - list every input whose result changes since `v0.2.3` (refusal, error field, output),
     and check the release notes name each one. The design found one new refusal (colour
     `W ≤ B`), one changed field (`PerPosition` `black_level`), and the output change. Is
     anything else there, for example a changed `Display` text?
   - Judge whether the notes' "Breaking" line about the `black_level` field is right to
     include, and whether anything should push this to 0.3.0.
2. **The src diff since the build branch point is doc-only.** `git diff origin/main --
   src/` on the candidate touches nothing (or comments only). `DEVELOP_VERSION` is `"7"`.
3. **The CHANGELOG.** `[0.2.4]` holds every library-visible change since `v0.2.3`
   (`git log v0.2.3..origin/main`, SPEC-078, 079, 080): nothing dropped in the promotion,
   the lead paragraph accurate, an empty `[Unreleased]` on top. The SPEC-080 entry's
   corrected wording names `BlackLevels::PerPosition`.
4. **The release notes are true.** Check every claim against code or evidence: the headline,
   "within one code before the GainMap" (which files, which comparison), the Pixel 7 Pro
   limitation, "byte-identical" for uniform-black and `FirstValue` files. Nothing overstated;
   the SPEC-080 wording caution respected (colour = `PerPosition`, not `FirstValue`).
5. **README and crate docs.** The README roadmap edit (writer moved to PROJ-005, optical
   black parked) is accurate; version strings say 0.2.4 where they should; nothing still
   says the colour path refuses nothing or scales by `W − b`. `cargo test --doc
   --all-features` and `just doc-check` pass.
6. **The package.** Run `cargo publish --dry-run` and `cargo package --list` yourself (in a
   clean scratch worktree if the tree is dirty). 19 files, the same list as the published
   0.2.3 `.crate`; no SDK, corpus or `scripts/` bytes; no self dev-dependency;
   `.cargo_vcs_info.json` names the candidate.
7. **The evidence reproduces.** `just test` with the corpus (12/12 present, counts match);
   a fresh `scripts/sdk-stage2.sh` run plus the cross-check (numbers equal SPEC-078's);
   `just msrv`, `just deny`, `just package-targets-check`.
8. **The script fixes.** Read both. Reproduce at least the CRLF, quoted-status and BOM cases
   for `validate.sh` and the unenterable-directory case for `sdk-stage2.sh`, on scratch
   copies. Confirm `validate` still fails a genuinely bad status, and that the BOM refusal
   doesn't trip on any real file.
9. **The pre-flight and the Phase 2 runbook.** Every `[now]` item has real evidence; every
   `[cut]` item is runnable as written; the commands are copy-paste correct (tag name,
   `cargo add irradiance@=0.2.4`, the `.crate` curl URL for 0.2.4, the yank command). The
   runbook asks the user to report the `[cut]` checks back.
10. **Nothing forbidden is committed:** no SDK bytes or outputs, no corpus bytes, no
    `Cargo.lock`.
11. **Gates:** every gate in Build work 9; CI.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence.
- **Commit this handoff's filled handback** to `claude/spec-083-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-083-build`.
- **A punch-list fix** is fine on your branch if it is docs, release notes, CHANGELOG or
  scripts only. Any `src/` change goes in the punch list.

## Out of Scope

- Tagging, publishing, or a GitHub release.
- Any library code change.
- The STAGE-026 and PROJ-004 closes.

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

*Filled in by the receiving agent (HANDOFF-174 verify, claude-opus-5-5).*

### Execution notes

- **Branch / PR:** `claude/spec-083-verify-base`, pushed by refspec, no PR (as
  instructed). The local branch is `claude/spec-083-verify-470495`, because the
  orchestrator worktree `monorail-000-backlog-f34c1a` has `claude/spec-083-verify-base`
  checked out. Doc fix commit `5a53497`; this handback is the commit after it.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** Yes. All eleven checks below were run, and one
  release-notes miss was fixed on this branch.
- **Verdict: ✅ APPROVED at `5a53497`.** That is PR #287's head `91119b9`, plus
  `fe2deec` (the verify setup) and `5a53497` (the verify doc fix). Its `src/` and
  `Cargo.toml` are byte-identical to `91119b9` (`git diff --stat 91119b9 5a53497 -- src
  Cargo.toml` is empty).
  - `91119b9` is **not** approved on its own. Its release notes and CHANGELOG miss one
    input whose result changed since 0.2.3 (FU-1).
  - So the merge the tag goes on must include `5a53497`. Either merge this branch into
    PR #287's branch, or cherry-pick `5a53497` onto it before merging. The package
    evidence (item 6) was taken at `91119b9`, so re-check `.cargo_vcs_info.json` against
    the tagged commit at cut, as pre-flight item 1 already says.

### Evidence, item by item

1. **Semver, and what it can't see.**
   - `just semver-check` (`cargo-semver-checks 0.51.0`): "v0.2.3 -> v0.2.4 (minor
     change)", 202 checks: 202 pass, 58 skip, "no semver update required".
     `cargo semver-checks check-release --baseline-version 0.2.3 --all-features`: the
     same.
   - I read `git diff v0.2.3 origin/claude/spec-083-build -- src/` myself. It touches
     `src/color.rs`, `src/develop.rs` and doc comments in `src/lib.rs`. No `pub` item is
     added, removed or changed. `normalize_range` is `pub(crate)`. The `Display` impl is
     not touched, and the `Error::InvalidLevels` change is doc-only. `Cargo.toml` changes
     only `version`, so there is no new dependency or feature and no MSRV change.
   - Every input whose result changes since `v0.2.3`:
     - (a) **Newly refused:** a colour input with `W ≤ B`. Named in the notes.
     - (b) **Mono `PerPosition`:** the same refusal set (`find(v >= W)` and
       `max >= W` refuse the same patterns), with `black_level` now the maximum. Named.
     - (c) **Output change** on a pattern whose blacks differ. Named. Uniform black is
       byte-identical: `normalize_range(s, b, W − b)` equals both `normalize` and
       `normalize_sample` for `b ≤ W`, and mono `FirstValue` still runs `normalize`.
     - (d) `DEVELOP_VERSION` 6 to 7. Named.
     - (e) **MISSED, FU-1: the error-precedence change.** `resolve_colour_levels` runs
       after `check_colour_required_processing` and `check_color_buffers`, but before
       `resolve_colour_gain_maps` and the CFA and matrix checks in
       `develop_color_fused`. That order is the same in 0.2.3 (`git show
       v0.2.3:src/color.rs`, lines 350–356). So a `W ≤ B` colour input that also fails a
       later check changes error variant. I measured it with a scratch probe crate that
       depends on crates.io 0.2.3 and on the candidate by path, on the held Canon with
       `white_levels = [1000]`:

       | Input | 0.2.3 | 0.2.4 |
       |---|---|---|
       | Canon, W ≤ B only | `Ok`, all-zero `dst` | `InvalidLevels { 2048, 1000 }` |
       | Canon, W ≤ B, no colour matrix | `MissingColorCalibration` | `InvalidLevels { 2048, 1000 }` |
       | Canon, W ≤ B, `color_filter = None` | `NotAColorFilterArray` | `InvalidLevels { 2048, 1000 }` |
       | Canon, no colour matrix (control) | `MissingColorCalibration` | `MissingColorCalibration` |

       The `GainMap` errors are covered by reading the code, not by a run. Fixed in
       `5a53497`: a Behaviour changes line, a CHANGELOG sentence, and Release Scope and
       internal-record lines.
   - **The "Breaking / action required" line** is right to include. It is an honest
     heads-up for exact-value matchers, and it correctly says "none for the API".
   - **Nothing pushes this to 0.3.0.** Every change is to output on malformed or
     position-varying-black inputs, or to which error a malformed input reports. That is
     behaviour that `DEVELOP_VERSION` and the documented error order exist to signal. No
     signature, type or variant changed.
2. **The src diff since the branch point is doc-only.** `git diff origin/main
   origin/claude/spec-083-build -- src/` is empty. `Cargo.toml` changes only the
   version line. `src/lib.rs:175` says `DEVELOP_VERSION: &str = "7"`.
3. **The CHANGELOG.** `[0.2.4] - 2026-11-20` holds both `[Unreleased]` entries from
   `main` unchanged, apart from the `PerPosition` wording. `git diff origin/main` shows
   only the lead paragraph and that fix. An empty `[Unreleased]` is on top. The lead
   paragraph is accurate.
   - `v0.2.3..origin/main` is 13 commits.
   - SPEC-079 has no library change, so it correctly has no entry.
   - The SPEC-080 entry names `BlackLevels::PerPosition`.
   - The entry lacked the precedence change (FU-1), now added.
4. **The release notes are true.** I checked every number against DEC-070's probe tables
   and my own SDK run:
   - "6 codes to 1 (69.4% to 99.6%)", "5 to 1" for the 3a and 4a, and with the
     `GainMap` "6 to 4" and "7 to 4" are DEC-070's 6/69.36% to 1/99.57%, 5 to 1, 6 to 4
     and 7 to 4.
   - "Within one code before the `GainMap`" holds for the Canon (plain stage 2) and the
     3a and 4a `-pre.tif`, `max|d| 1`.
   - The Pixel 7 Pro "up to 2 codes" is `max|d| 2`.
   - "Byte-identical" was shown by the code argument in 1(c). Every MD5 pin also holds
     in `just test`.
   - The wording caution holds: colour equals `PerPosition`, and `FirstValue` checks its
     first black.
   - The one claim that overreached was "0.2.3 developed such a file to an all-zero
     image". That is true only when no later check also fails. Fixed by FU-1's line.
5. **README and crate docs.** The status line says 0.2.4, and the roadmap edit is
   accurate:
   - the PROJ-005 brief owns the public writer;
   - the PROJ-004 brief line 121 has "Instrument layer B: optical black" `(parked)`.

   No live text says the colour path refuses nothing or scales by `W − b`. The remaining
   "its own black" hits describe the subtraction, which is correct. `cargo test --doc
   --all-features` passed (within `just test`). `just doc-check`: 2 surfaces, 0 rustdoc
   warnings.
6. **The package.** Checked in a detached scratch worktree of `91119b9` with a clean
   tree, since removed:
   - `cargo package --list` gives 19 files, identical to the published 0.2.3 `.crate`'s
     `tar -tz` list (diffed after stripping the prefix);
   - `cargo publish --dry-run` with no `--allow-dirty`: "Packaged 19 files, 1.8MiB
     (374.7KiB compressed)", "Uploading irradiance v0.2.4", "aborting upload due to dry
     run". The only other warnings are the expected "ignoring example/test" lines;
   - the packaged `.cargo_vcs_info.json` sha1 is `91119b9a3e8f…`;
   - its dev-dependencies are only `ssimulacra2` and `toml`, with no self dev-dependency;
   - there are no SDK, corpus, `scripts/` or image bytes.
7. **The evidence reproduces.**
   - `just test` with `IRRADIANCE_CORPUS_DIR` set (SDK unset):
     - "corpus: 12/12 present — no tier-B test will skip";
     - 46 results, 790 passed, 0 failed, 1 ignored, exit 0, 3:27 wall-clock.
   - A fresh `scripts/sdk-stage2.sh` into a scratchpad path with a space wrote 7 TIFFs.
     All 7 sha256s equal HANDOFF-169's table (SPEC-079).
   - `--test gain_map_corpus -- --nocapture` with `IRRADIANCE_SDK_STAGE2_DIR` set: 11
     passed. Every number equals the notes' table (and so SPEC-078's) digit for digit:
     - no gain: Canon 1 / 99.5663%, 4a 1 / 99.7911%, 3a 1 / 99.8536%, 7 Pro 2 /
       28.3471%;
     - end to end: 4a 4 / 99.7729%, 3a 4 / 99.8210%, 7 Pro 4 / 28.3394%;
     - gain step: 1 / 99.9794%, 99.9672% and 99.9664%.
   - `just msrv` (1.90.0), `just deny`, `just deny-fuzz` ("licenses ok" twice) and
     `just package-targets-check` all passed.
   - I did not re-run the three timing examples. They are measurements, not gates.
8. **The script fixes.** I read both diffs.
   - `validate.sh`, on a `git archive` copy in the scratchpad. The control is green
     with 9 feedback files.
     - Accepted, exit 0: CRLF `open`, `"open"`, `'deferred'`, and CRLF plus quoted.
     - Rejected, exit 1, by name: `bogus`, CRLF `bogus` and `"bogus"`. Unbalanced and
       mismatched quotes are rejected twice, by status and by the YAML parse.
     - A BOM before `---` is refused by name: "starts with a UTF-8 BOM before its
       '---'".
     - A BOM file with no front matter is skipped, as any front-matter-less file is.
     - Isolation is green.
     - No tracked file in `feedback/`, `projects/` or `decisions/` has a BOM, so the
       refusal trips on no real file.
   - `sdk-stage2.sh`, on a scratch git repo plus a worktree, both chmod 000. The
     validator is `/usr/bin/true`.
     - The old script dies with only the shell's own "cd: Permission denied".
     - The new one warns "cannot enter worktree … refusing its recorded path as it is"
       and carries on to the corpus check.
     - An unenterable out dir dies with "cannot enter the output directory".
     - An unenterable parent dies with "cannot enter the output directory's parent".
     - An out path under the locked worktree dies with "parent does not exist". It
       still refuses and writes nothing.
     - Nothing was written into any checkout.
9. **The pre-flight and the Phase 2 runbook.** Every `[now]` item matches what I
   re-ran. I checked the commands:
   - `git tag -a v0.2.4 -m "irradiance 0.2.4"` and `cargo add irradiance@=0.2.4` are
     valid syntax;
   - the `.crate` curl-into-`tar -xzO` pipeline works on macOS bsdtar. Run against
     0.2.3, it printed sha1 `61a2671…`, which equals `git rev-parse v0.2.3^{commit}`;
   - `cargo yank --version` and `--undo` exist in this cargo;
   - step 5 asks the user to report the `[cut]` checks back.

   One fix in `5a53497`: the `[cut]` smoke said "a hand-built colour sensor". `Sensor`
   is `#[non_exhaustive]`, so a probe crate cannot build one. It now says to set a
   parsed Canon `Sensor`'s `white_levels` to `vec![1000]` and expect `InvalidLevels {
   2048, 1000 }`. That is exactly what my probe crate ran.
10. **Nothing forbidden is committed.**
    - `git ls-files` has no `Cargo.lock`.
    - The only image-extension files tracked are the pre-existing hand-built
      `fuzz/seeds/**/*.tiff`, none changed since `v0.2.3`.
    - No SDK bytes or outputs are tracked.
    - PR #287 changes 8 files: CHANGELOG, Cargo.toml, README, the spec, timeline and
      handoff, and the two scripts.
11. **Gates.** All passed locally on this branch:
    - `just test` (above), `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
    - `validate` (re-run after `5a53497`), `doc-check`, `deny`, `deny-fuzz`, `msrv`;
    - `cost-audit`, `gate-audit` (13 scripts), `name-length-check` (896 paths);
    - `decisions-index --check` (70 decisions), `semver-check` (202 of 202),
      `package-targets-check`.

    CI on PR #287 at `91119b9`: 22 of 22 SUCCESS, mergeable.

### Cost self-report

- **Tokens (total):** 10,601,568. This is a floor; the handback commit and push turns
  are not in it.
- **Estimated USD:** 7.73, at opus-5-5 rates: in 5 and out 25 per M; cache read 0.1x,
  1-hour write 2x.
- **Duration (minutes):** about 45, a wall-clock estimate.
- **Source of the number:** the transcript
  `7b54cae4-4d37-4044-b912-30dbc61a9e65.jsonl`, found by the scratchpad UUID. It is
  deduped by `message.id` with output at its maximum per message. It has 55 ids, all
  claude-opus-5-5, and no subagents. The split: input 109, cache creation 177,357
  (1-hour), cache read 10,393,891, output 30,211.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the build. The verify added the one docs fix
  described above. No `src/` change.
- **Follow-up work identified:**
  - `FU-1` (fixed, `5a53497`): the release notes and CHANGELOG missed the colour
    error-precedence change. A `W ≤ B` colour input that 0.2.3 refused with a later
    `GainMap`, CFA or matrix error now gets `InvalidLevels`. This is the same class as
    SPEC-077's verify miss, a refused input the notes didn't name. It is now the second
    instance, so it is worth recording in `guidance/signals.yaml` at ship. The design's
    `Err(` diff found the new error but not the errors it now pre-empts.
  - `FU-2` (fixed, `5a53497`): pre-flight item 5's `[cut]` probe asked for "a
    hand-built colour sensor", which `#[non_exhaustive]` forbids outside the crate.
  - `FU-3` (follow-up, nit): `sdk-stage2.sh`'s new warning says "refusing its recorded
    path as it is". It means the recorded path is still treated as the repository.
    Clearer: "keeping its recorded path as a refused location". Scripts only; doesn't
    hold the release.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. One snag: `claude/spec-083-verify-base` was checked out in another
   worktree, so `git checkout -B` with that name failed. I used a local branch name and
   pushed by refspec, as SPEC-080's verify did.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The design's "every `Err(` path" diff should also ask a second question: which
   existing errors does a new check now come before? A new early refusal changes the
   reported error for every input that also fails a later check. The release-spec
   template could name that.

3. **If you did this task again, what would you do differently?**
   — I would write the two-version probe crate first (crates.io 0.2.3 and the candidate
   by path, same input). It answers "did this input's result change" by running, not by
   reading, and it also proves the `[cut]` smoke probe is runnable as written.
