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
  id: HANDOFF-169
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-079

project:
  id: PROJ-004
  stage: STAGE-025
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
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10271921           # REAL combined count — what cost-audit reads
  estimated_usd: 7.65              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 35
  branch: claude/spec-079-build
  pr: 279
  completed_at: 2026-11-20         # YYYY-MM-DD
  notes: "Floor taken before the handback commit: 61 message ids, opus-5-5 only, no subagents, 1-hour cache writes. Re-sum at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-169: BUILD: PROJ-004 close-out hygiene (SPEC-079)

## Delegation Summary

You are the **builder** for SPEC-079. **Start only after the SPEC-079 design PR has
merged.** Cut `claude/spec-079-build` from that `origin/main`.

**The task:** close PROJ-004's carried follow-ups, with no library code change:
- `scripts/sdk-stage2.sh` finds `OpcodeList2` by walking the IFDs, not by a byte
  pattern, and creates its output directory only after the in-repo check;
- with `IRRADIANCE_SDK_STAGE2_DIR` set, a missing corpus file fails the SDK
  cross-check instead of skipping it;
- the provenance ledger and the script header record that the SDK zip ships no licence
  agreement document;
- the monorail triage record is brought up to date and marked `addressed`;
- FU-2 from SPEC-078's verify: tier-A colour and mono fixtures with the maximum black at
  each slot, and a general-path one with it at index 0.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-079-proj-004-close-out-hygiene-sdk-stage2-sh-walks-the-ifds-and.md`
  (Context 1–6 as found at design, Design 1–6, AC1–AC7).
- **Decisions:** DEC-069 §6 (the SDK reference and the visible skip), DEC-070 (the
  rule FU-2 tests).
- **Verify finding behind FU-2:** HANDOFF-168's `## Handback`, item 1 and FU-2.
- **Code and records:**
  - `scripts/sdk-stage2.sh`;
  - `tests/gain_map_corpus.rs` (`sdk_dir`, `sdk_stage2`, `corpus_plane` and the three
    SDK tests);
  - `tests/black_normalisation.rs` and `tests/colour_black_pattern.rs` (fixture shapes);
  - `docs/provenance-ledger.md` (the SPEC-059 `GainMap` row and the SPEC-078
    black-normalisation row);
  - `feedback/2026-11-11-monorail-000-requests.md` (front matter and the Resolution
    table only).
- **The SDK:**
  - `dng_validate` is at `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`;
    the zip is in `../dl/`.
  - Run the **old** script once into one scratchpad directory and keep its outputs, then
    the new one into another, and compare (AC1).
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them into the repo.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## Expected Deliverables

1. **The changes** for AC1–AC6, in a PR against `main` from `claude/spec-079-build`, with
   CI green.
2. **The handback:**
   - AC1: each Pixel's rewrite offset, old and new; the sha256 of every reference TIFF
     from both scripts; the cross-check numbers;
   - AC2 and AC3: each scenario and its outcome, with the message printed;
   - AC4: the unit tests, and the by-hand run with an empty corpus directory;
   - AC5: the mutant table (the control first, then each mutant and the tier-A tests
     that go red).
3. **Gates**, all green: AC7.

## Out of Scope

- FU-1, a colour levels refusal (next project).
- Any `src/` change, pin change, `DEVELOP_VERSION` change or CHANGELOG entry.
- The brief's stage count and the STAGE-025 reflection (the stage close does them).
- `BlackLevelDeltaH/V`.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus bytes, SDK bytes, SDK outputs or scratch copies.
- **An existing assertion may be strengthened, never weakened or deleted.** If any
  existing test or pin has to move, stop and report before changing it.
- **The ledger states facts only** (what the zip contains, what irradiance does with
  the SDK). Give no legal opinion.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill
  the `## Handback` section.

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

- **Branch / PR:** `claude/spec-079-build`, https://github.com/jysf/irradiance/pull/279 (build commit `38a50d1`)
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** Yes. No existing assertion moved; nothing in `src/`.

**AC1: the IFD walk.** The old script ran cold into one scratchpad directory, the new one into another.

| Pixel | old offset | new offset (IFD) |
|---|---|---|
| 4a | 121934 | 121934 (IFD at 121620) |
| 3a | 1488 | 1488 (IFD at 1174) |
| 7 Pro | 11224 | 11224 (IFD at 10910) |

All 7 TIFFs and the 3 nogain DNGs are byte-identical (`cmp`). The sha256s are the same from both scripts:

| file | sha256 |
|---|---|
| CANON-EOS-5D-MARK-III-stage2.tif | `893d7b94d25de722b6971c3e793fada9f47eabbab86ab3c641a2cfc506d74d1f` |
| GOOGLE-PIXEL-3A-post.tif | `b2f784d7d246e7faf09f3a6e7b2dcd27ebf7c5a5364c28d0067f485439666fc9` |
| GOOGLE-PIXEL-3A-pre.tif | `b0dc823bd4f4588f09c35f37d989848cce05b369763770e80e32cdfdaf19f23a` |
| GOOGLE-PIXEL-4A-post.tif | `d33afbb4c200d637daf71d45a9c599fcfcc7307b118cd44e0380f8433be593d1` |
| GOOGLE-PIXEL-4A-pre.tif | `771609b8bd16b70991cb9ccb7baedf6170135380bb7812d992a8b141459fb1de` |
| GOOGLE-PIXEL-7-PRO-post.tif | `ec2e1a55afe27cd7cd0ba4a93cb8824ced8fc822d8c3ed290fb369d3d5199fd7` |
| GOOGLE-PIXEL-7-PRO-pre.tif | `3fc24044b8871db962f42f7bf7f5be5c3c1f70573e31f9dde3084b95909eba94` |

The cross-check against the new outputs equals SPEC-078's numbers digit for digit:
- gain step: 4a 1 / 99.9794%, 3a 1 / 99.9672%, 7 Pro 1 / 99.9664%;
- no gain: Canon 1 / 99.5663% {0: 22333249, +1: 97271}, 4a 1 / 99.7911%, 3a 1 / 99.8536%, 7 Pro 2 / 28.3471%;
- end to end: 4a 4 / 99.7729%, 3a 4 / 99.8210%, 7 Pro 4 / 28.3394%.

**AC2: refusals.** Run on scratch copies of the 4a, through each script's own `disable_opcode_list_2`, extracted verbatim.

| scenario | old | new |
|---|---|---|
| decoy 51009 pattern planted at byte 13582608 (pixel data) | refused: "pattern found 2 times" | **succeeds**; output differs from input at byte 121934 only |
| 51009 entry's tag changed to 51011 | refused (0 times) | refused: "0 OpcodeList2 (51009) entries found, expected exactly 1" |
| IFD0 entry 47 (50941) rewritten to 51009 / type 7 | refused (2 times) | refused: "2 OpcodeList2 (51009) entries found" |
| truncated to 121700 bytes | refused (0 times) | refused: "IFD at 121620 has 29 entries, which run past the end (121700)" |
| header IFD0 offset = length + 100 | **accepted** | refused: "IFD offset 13586804 is past the end (13586704)" |
| IFD0 NextIFD pointing back at IFD0 | **accepted** | refused: "IFD at 8 is reached twice (an IFD cycle)" |

None printed a Perl warning or hung; the walk uses fatal warnings, so any warning would be a refusal. Two extra cases on synthetic TIFFs, since no held file is `MM`:
- a big-endian file with the entry in a SubIFD, plus three decoys: rewritten at the right byte only (`c741` to `c742`);
- a 51009 entry of type 4: refused with "has type 4, expected 7 (UNDEFINED)".

**AC3: directory order.**
- `./spec079-x` (in the repo, does not exist): refused as inside the repository, and the directory does not exist afterwards. The old script, given an in-repo path, refused but left the directory behind.
- `target/spec079-inrepo` (parent missing): refused with "the output directory's parent does not exist: target".
- A path outside the repo whose parent does not exist: refused, and nothing was created.
- A path outside the repo that does not exist: created, and all outputs were written.

**AC4: set means must run.**
- Unit tests: `sdk_set_and_corpus_file_missing_fails_naming_the_file` (the panic names the manifest path, the resolved file, `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_SDK_STAGE2_DIR`) and `sdk_unset_and_corpus_file_missing_skips` (`None`; a present file is `Some` on both arms).
- By hand, with `IRRADIANCE_SDK_STAGE2_DIR` set and `IRRADIANCE_CORPUS_DIR` pointed at an empty directory, all 3 SDK tests fail with "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng: MISSING at …/empty-corpus/… — $IRRADIANCE_SDK_STAGE2_DIR is set, so the dng_validate cross-check must run: point $IRRADIANCE_CORPUS_DIR at the corpus, or unset $IRRADIANCE_SDK_STAGE2_DIR". The Canon fails the same way.
- Unset: the same 3 tests print "SKIP dng_validate cross-check" and pass.

**AC5: mutants.** Each ran in its own copy with its own `CARGO_TARGET_DIR`. Tier A only: the corpus and SDK variables were unset, and the run was `cargo test --all-features --profile corpus --no-fail-fast`.

| run | result | red tests |
|---|---|---|
| control (unmutated) | 764 passed, 0 failed, 1 ignored | none |
| colour slots `B` = last slot | 1 failed | `maximum_black_at_each_slot_scales_every_slot_by_it` only ("u16 colour, maximum black at slot 0"). The FU-2 gap: nothing else on tier A sees it. |
| colour slots `B` = first slot | 12 failed | the new slot test ("slot 1"); `colour_scales_by_white_minus_the_maximum_black`; `color::tests::colour_slots_scale_by_white_minus_the_maximum_black`, `colour_with_no_range_maps_to_zero_without_panicking`; `four_by_four_pattern_takes_the_general_path`; the color_oracle group (`develop_color_composes_…`, `fused_…`, `colour_crop_…`, `colour_geometry_…`, `colour_output_has_…`, `trivial_geometry_…`) |
| colour general `B` = the pattern's first value | 5 failed | `general_pattern_maximum_black_at_each_index_scales_every_position_by_it` ("pattern index 1"); `color::tests::colour_general_pattern_scales_by_white_minus_the_maximum_black`, `general_black_pattern_index_matches_hand_table`; `four_by_four_pattern_takes_the_general_path` (u16 and f32) |
| (extra) mono `B` = last value | 5 failed | both new tests (mono, slot 0 / index 0); `bad_patterns_are_refused_only_when_used`, `per_position_allocates_nothing`, `uniform_cfa_plane_develops_uniform_under_per_position` |

**AC6: records.**
- The ledger: the SPEC-059 and SPEC-078 SDK rows each gain the finding. The 1.7.1 zip (sha256, 4,259 entries) ships no licence agreement document; its 4 readmes and 3 PDFs have 0 hits for "licen" or "agreement" (`pdftotext`, re-checked here). The only Adobe terms are the per-file notice; the licence files it does contain are third-party. irradiance reads the source and runs `dng_validate` locally, copies no code and redistributes nothing (DEC-069 §6). The ledger gives no opinion.
- The script header says the same.
- The feedback file matches Design §5 and its front matter parses (`YAML.parse`). `just validate` is green, but its scope is `projects/` and `decisions/`, so it does not read `feedback/` or `docs/`.

**AC7: gates.**
- `just test` with the corpus: 778 passed, 0 failed, 1 ignored, both without and with `IRRADIANCE_SDK_STAGE2_DIR`.
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`, `gate-audit` (13 scripts), `name-length-check` and `decisions-index --check` are all green.
- CI: see PR 279.

### Cost self-report

- **Tokens (total):** 10,271,921. That is input 122, output 39,043, cache read 10,069,320 and cache write 163,436 (all 1-hour), over 61 message ids, deduped with the largest `output_tokens` taken.
- **Estimated USD:** $7.65 at opus-5-5 rates ($5 in, $25 out, reads ×0.10, 1-hour writes ×2.0).
- **Duration (minutes):** about 35.
- **Source of the number:** the session transcript `608b9538-500d-47b7-a53e-756dd452a018.jsonl`, which matches this session's scratchpad UUID. No subagents. It is a floor taken before the handback commit, so re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The general-path FU-2 test puts the maximum at **every** one of the 16 indices, not only index 0. Design §6 asks for index 0, and AC5's third mutant ("`B` = the pattern's first value") cannot be red on a fixture whose maximum *is* the first value. Index 0 catches a last-value bug; the other 15 catch the first-value bug. This strengthens the design and drops nothing.
  - Mono `PerPosition` also runs on the `[4, 4]` case. A fourth mutant (mono `B` = last value) was run as extra evidence.
  - `std::fs::read(&file_path)` became `read(file_path)` in the refactored loader, for clippy's `needless_borrows_for_generic_args`.
- **Follow-up work identified:**
  - FU-1 (colour levels refusal) stays out of scope, as designed.
  - AC6 says "`just validate` parses both", but `validate` does not scan `feedback/` or `docs/`. Either widen it or reword future specs' AC. This is process debt for the stage close to triage.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two things. AC5's third mutant does not fit Design §6's index-0-only fixture, because a first-value `B` is correct there. And the old script cannot be run from a copy outside the repo: its repo root comes from the script's own location, so a copy under the scratchpad treats the scratchpad as the repo. It has to be run in place before editing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That `just validate` covers only `projects/` and `decisions/`, so AC6's "validate parses both" needs a separate parse of the feedback front matter.

3. **If you did this task again, what would you do differently?**
   — Extract the Perl function into a harness from the start, as I did for AC2. That tests the exact shipped code on crafted files without needing `dng_validate` or the full four-file run.
