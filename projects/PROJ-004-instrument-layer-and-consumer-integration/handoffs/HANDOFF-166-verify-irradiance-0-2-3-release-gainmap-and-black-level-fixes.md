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
  id: HANDOFF-166
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-077

project:
  id: PROJ-004
  stage: STAGE-024
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
  tokens_total: 9021275               # REAL combined count — what cost-audit reads
  estimated_usd: 6.14              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 12
  branch: claude/spec-077-verify-base
  pr: null
  completed_at: 2026-11-19               # YYYY-MM-DD
  notes: "APPROVED at c69c813 (PR 272's head plus the verify doc fixes; src byte-identical to cf8b252). FLOOR, measured live from transcript 3fa65943-883c-4b14-9b7f-c2621b7e976e.jsonl (63 unique message.ids, all claude-opus-5-5, no subagents): input 125 + cache-creation 113,049 (1-hour) + cache-read 8,885,417 + output 22,684 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The handback commit and push turns are not in it; re-sum at ship."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-166: VERIFY: irradiance 0.2.3 release prep, Phase 1 (SPEC-077)

## Delegation Summary

You are the **verifier** for SPEC-077, the 0.2.3 release prep. Review it cold: you did
not build it.

- **Under review:** PR #272 (`claude/spec-077-build`, head 228a76e; release candidate
  `cf8b252`), built under HANDOFF-165. CI was green on all 22 checks.
- **Your branch:** `claude/spec-077-verify-base`. It is that head plus one commit that
  advances the spec to verify and adds this handoff.

This is the last check before the user tags `v0.2.3` and runs `cargo publish`, which
can't be undone (only yanked). Your job is to find anything that would make the
published crate or its notes wrong.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-077-irradiance-0-2-3-release-gainmap-and-black-level-fixes.md`
  (Release Scope, Build work 1–8, Release Notes, Release Pre-Flight, Phase 2 runbook).
- **Build handback:** HANDOFF-165's `## Handback`. Check each claim yourself.
- **Precedent:** SPEC-073 (0.2.2) in `specs/done/`, and its verify handoff.
- **Decisions:** DEC-006, DEC-052, DEC-067, DEC-068, DEC-069.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  - `cargo-semver-checks` is installed. `just semver-check` uses the crates.io baseline.

## What to check hard

1. **Semver.** Re-run `just semver-check` yourself, default and with `--all-features`.
   Then independently read `git diff v0.2.2 origin/claude/spec-077-build -- src/` for
   public items. Confirm three things:
   - no `pub` item was removed or had its signature changed;
   - every new public field or variant is on a `#[non_exhaustive]` type;
   - no new dependency or feature was added, and the MSRV is unchanged.

   Name anything the tool cannot see, such as a changed `Display` text, a newly refused
   input, or changed output. Judge whether each belongs in "Behaviour changes" rather
   than requiring 0.3.0.
2. **The src diff is doc-only.** `git diff origin/main -- src/` must touch comments
   only. Confirm it, and that `DEVELOP_VERSION` is `"6"` on the candidate.
3. **The CHANGELOG is complete and accurate.**
   - Every library-visible change since `v0.2.2` has an entry under `[0.2.3]`. Cross-check
     against `git log v0.2.2..origin/main` and the shipped specs: SPEC-074, 075, 076,
     059 and 060.
   - Nothing was dropped in the promotion, and the merged `DEVELOP_VERSION` entry is
     correct.
   - An empty `[Unreleased]` sits on top.
4. **The release notes are true.** Check every claim against the code or evidence: the
   headline, the `dng_validate` figure, the behaviour changes, and "every other file's
   output is byte-identical". Nothing should be overstated, such as a colour-accuracy
   claim.
5. **README and crate docs.**
   - No remaining text says a Pixel `GainMap` is refused or unsupported. Grep for
     `GainMap`, `Pixel`, `opcode 9` and `UnsupportedMandatoryOpcode`.
   - Version strings say 0.2.3 where they should.
   - The new README sentence wraps like the surrounding text.
   - `cargo test --doc --all-features` and `just doc-check` pass.
6. **The package.**
   - Run `cargo publish --dry-run` yourself, and `cargo package --list`.
   - The file list contains no SDK, corpus or `scripts/` bytes and no tier-B fixtures.
   - The packaged `Cargo.toml` has no self dev-dependency.
   - `.cargo_vcs_info.json` names the candidate.
   - Compare the file list with 0.2.2's (19 files) and explain any difference.
7. **The evidence reproduces.** Run `just test` with the corpus and confirm 12/12
   present, with counts that match the handback's. Also run `just msrv`, `just deny` and
   `just package-targets-check`.
8. **The pre-flight and the Phase 2 runbook.**
   - Every `[now]` item is marked with real evidence.
   - Every `[cut]` item is something the user can actually run as written.
   - The commands are copy-paste correct: the tag name, the `cargo add irradiance@=0.2.3`
     form, the `.cargo_vcs_info.json` curl URL, and the yank command.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above.**
- **Commit this handoff's filled handback** to `claude/spec-077-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-077-build`.
- **A punch-list fix** is fine on your branch if it is docs, CHANGELOG or spec text only.
  Any `src/` code change, or a version change, goes in the punch list for the user.

## Out of Scope

- The tag, `cargo publish` and a GitHub release (the user's Phase 2).
- STAGE-016's carried follow-ups.
- Any library behaviour change.

## Hard rules

- **Never** tag, publish, yank, force-push, merge, or enable auto-merge.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-077-verify-base`, no PR (as instructed). Fix commit
  `c69c813`; this handback is the commit after it.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes. All eight checks below were run, and three
  doc defects were fixed on this branch.
- **Verdict: ✅ APPROVED at `c69c813`.** That is PR #272's head (`228a76e`) plus
  `d6cdbfb` (verify setup) and `c69c813` (verify doc fixes). Its `src/` and
  `Cargo.toml` are byte-identical to the candidate `cf8b252`.
  - The candidate `cf8b252` is **not** approved on its own. Its release notes miss one
    newly refused input, and its headline overstates the SDK match.
  - So the merge that the tag goes on must include `c69c813`. Either merge
    `claude/spec-077-verify-base` instead of `claude/spec-077-build`, or cherry-pick
    `c69c813` onto #272 first.
  - `c69c813` changes the packaged README by wording and wrapping only.

#### Evidence per "What to check hard" item

1. **Semver. ✔**
   - `just semver-check` (`cargo-semver-checks` 0.51.0, crates.io 0.2.2 baseline)
     gives "202 checks: 202 pass, 58 skip", "no semver update required".
     `check-release --baseline-version 0.2.2 --all-features` gives the same result.
   - I read `git diff v0.2.2 HEAD -- src/` by hand. No `pub` item was removed or
     re-signed. The only removed `pub` line is `DEVELOP_VERSION = "4"`, now `"6"`.
   - The new public items are all additive:
     - `develop::apply_gain_maps`;
     - `ifd::TAG_MASKED_AREAS` and `ifd::MAX_MASKED_AREAS`;
     - `Sensor::masked_areas` and `Sensor::black_levels_exact` (`Sensor` is
       `#[non_exhaustive]`);
     - `Error::TooManyGainMaps` and `Error::GainMapImageWrongLength`, appended last on
       the `#[non_exhaustive]` `Error`;
     - `Opcode::GainMap`, appended last on the `#[non_exhaustive]` `Opcode`.
   - `ErrorKind` gained no variant.
   - `InvalidBlackLevelPattern` already existed in 0.2.2. The design paragraph that
     listed it as new is corrected.
   - `Cargo.toml` changes only the version line, so there is no new dependency or
     feature, and `rust-version` stays 1.90.
   - **What the tool cannot see:**
     - **Display text:** two new arms only. No existing text changed.
     - **Changed output:** `DEVELOP_VERSION` 4 → 6, a fractional black, a non-2×2
       colour black pattern, and optional `GainMap`s now applied. All are already in
       the notes.
     - **Colour BlackLevel count:** a colour count that disagrees with
       `BlackLevelRepeatDim` is now refused. This is already in the notes.
     - **GainMap refusals (missed by the notes; fixed):** 0.2.2 skipped optional
       `GainMap`s unread (`check_list` at `v0.2.2`). 0.2.3 parses every `GainMap` in
       `OpcodeList2` (`GainMaps::from_list`). A file with a malformed optional
       `GainMap`, or more than 16 of them, developed before and is now refused with
       `MalformedOpcodeParams { id: 9 }` or `TooManyGainMaps`. A malformed mandatory
       one changes variant, from `UnsupportedMandatoryOpcode` to
       `MalformedOpcodeParams`. DEC-069 §3 intends this.
     - **`malformed_tags`:** a malformed `MaskedAreas` (50830) now appears in it,
       and a fractional `BlackLevel` no longer does. The CHANGELOG states both.
   - My judgment: every item is a refusal of malformed input or a correction of
     wrong output, so it fits "Behaviour changes" in a patch, as the 0.2.2
     precedent does. None needs 0.3.0.
2. **The src diff is doc-only. ✔**
   - `git diff origin/main HEAD -- src/` touches only `//!` and `///` lines: the
     `develop` module pipeline, the `develop_into` tone sentence and the `lib.rs`
     history.
   - `DEVELOP_VERSION` is `"6"`.
   - The new intra-doc link `[crate::tone]` resolves (doc-check is clean).
3. **The CHANGELOG. ✔ (one entry added)**
   - `git log v0.2.2..origin/main` lists five library commits:
     - SPEC-074: the one Compression 7 header parse, under Changed;
     - SPEC-075: rounding, colour RepeatDim and MaskedAreas, under Added, Fixed and
       Changed;
     - SPEC-059: GainMap, under Added and Changed;
     - SPEC-060: tests and evidence only, with zero `src/` lines;
     - SPEC-076: test hardenings only, 31 test lines in `src/develop.rs`.
   - Everything else is CI or projects-only.
   - Nothing was dropped in the promotion. The merged `DEVELOP_VERSION` entry (4 → 6,
     citing DEC-068 §4 and DEC-069 §5) is correct.
   - An empty `[Unreleased]` is on top.
   - Added under Changed: the GainMap refusal from item 1.
4. **The release notes. ✔ after fixes**
   - The `dng_validate` figures reproduce exactly. I ran `scripts/sdk-stage2.sh` with
     the local 1.7.1 build into my scratchpad, then
     `--test gain_map_corpus --release`, and all 8 passed. The gain step's max |d|
     is 1 on each file, and the exact share matches the build:
     - Pixel 4a: 99.9794%;
     - Pixel 3a: 99.9672%;
     - Pixel 7 Pro: 99.9664%.
   - **The headline overstated this.** It said the develop "matches Adobe's DNG SDK
     within one code". The full stage 2 differs from the SDK by up to 7 (4a), 6 (3a)
     and 4 (7 Pro), and those differences exist without the gain too. The headline
     now limits the claim to the gain step on the three tested files. It also said
     "Android phones write [it] as mandatory", but DEC-069 notes the Samsung's maps
     are optional, so that now reads "Pixel phones".
   - "Every other file's output is byte-identical" holds. No 0.2.2 MD5 or checksum
     pin was removed or changed in `tests/`; only Pixel pins were added, and every
     pin held in `just test`.
   - No colour-accuracy claim is made: Known limitations says it is uncalibrated.
   - The behaviour-change bullet from item 1 was added.
5. **README and crate docs. ✔ (one rewrap)**
   - I grepped for `GainMap`, `Pixel`, `opcode 9`, `id: 9` and
     `UnsupportedMandatoryOpcode`. Nothing says a Pixel or a `GainMap` is refused.
     The remaining hits describe current behaviour: lists 1 and 3, and other opcodes.
   - The status line says 0.2.3.
   - The new status sentence ran onto one 120-character line. It is rewrapped to the
     block's width.
   - `cargo test --doc --all-features`: 4 passed.
   - `just doc-check`: 2 surfaces, 0 warnings.
6. **The package. ✔**
   - `cargo publish --dry-run` packaged v0.2.3, 19 files, 1.7 MiB (371.7 KiB
     compressed), verified, and aborted the upload, on `d6cdbfb` and again on
     `c69c813`.
   - The file list is identical to the 19 entries in the crates.io 0.2.2 `.crate`,
     checked with a `diff` against the cached file. There are no `scripts/`,
     `tests/`, SDK or corpus bytes.
   - The packaged dev-dependencies are only `ssimulacra2` and `toml`, with no self
     dev-dependency.
   - `.cargo_vcs_info.json` names the HEAD it was packed from (`c69c813` on the last
     run). At the cut it names the merged commit.
7. **The evidence reproduces. ✔**
   - `just test` with the corpus: "corpus: 12/12 present"; 44 result lines, 764
     passed, 0 failed, 1 ignored, exit 0, 3:45 wall. This matches the handback's 764
     and 224 s.
   - `just msrv` (`+1.90.0 check --all-targets --all-features`), `just deny`,
     `just deny-fuzz` and `just package-targets-check` (red case caught) all pass.
   - `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `cost-audit`,
     `gate-audit` and `name-length-check` all pass.
8. **The pre-flight and the Phase 2 runbook. ✔ (one command added)**
   - The `[now]` items match what I reproduced.
   - The tag command `git tag -a v0.2.3 -m "irradiance 0.2.3"` is correct.
   - `cargo add irradiance@=0.2.3` is valid `cargo add` syntax.
   - `cargo yank --version 0.2.3` and `--undo` match `cargo yank --help`.
   - **The `.cargo_vcs_info.json` curl command was missing** from item 1 `[cut]`,
     which the 0.2.2 pre-flight had. It is added. Its 0.2.2 form printed `46b6fc2…`,
     which equals `v0.2.2^{commit}`.
   - Runbook step 1 (`git checkout main && git pull`) assumes the merged branch
     carries `c69c813`. See the verdict.

### Cost self-report

- **Tokens (total):** 9,021,275 (floor; measured before the handback commit)
- **Estimated USD:** 6.14
- **Duration (minutes):** about 12
- **Source of the number:** harness transcript `3fa65943-….jsonl`, deduplicated by
  `message.id`, with the largest output per message and no subagents
  (`docs/cost-tracking.md` "Measuring a cycle").

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none to the library. Doc fixes in `c69c813`:
  - the CHANGELOG Changed bullet;
  - the release-notes headline and the behaviour-change bullet;
  - the pre-flight curl command;
  - the Release Scope variant list;
  - the README rewrap.
- **Follow-up work identified:**
  - Merge `c69c813` with #272 before tagging (the verdict's condition).
  - The release-pre-flight template could carry the crates.io vcs-info curl line
    by default, since 0.2.3's build dropped it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing much. "Name anything the tool cannot see, such as a newly refused
   input" pointed straight at the gap. It took reading `check_list` at `v0.2.2` to
   prove the gap.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-069 §3 already states the refusal. The notes just didn't carry it
   over.

3. **If you did this task again, what would you do differently?**
   — Run the SDK cross-check first. Its end-to-end lines are what show the headline
   overstated the match, and the script takes about 4 s.
