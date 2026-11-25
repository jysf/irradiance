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
  id: HANDOFF-194
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-082

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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 8976828               # REAL combined count — what cost-audit reads
  estimated_usd: 6.74              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: claude/spec-082-verify-base
  pr: 336
  completed_at: 2026-11-25               # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript eb87bed7-698f-4bbb-bd59-79a75b50424d.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 105 + cache-creation 162,490 (1-hour) + cache-read 8,785,354 + output 28,879, priced at 5 and 25 USD per million, 1-hour cache writes at 2x, as HANDOFF-192 did. Approved; the code under review is PR 336 head 59c1b64, and the verify commit adds one test assertion and docs only. 22 of 23 mutants killed (M6 only after the verify adds one tag-order assertion), the survivor X10 equivalent."
  synced_at: 2026-11-25
---

# HANDOFF-194: VERIFY: optional caller JPEG preview as a reduced-resolution subfile (SPEC-082)

## Delegation Summary

You are the **verifier** for SPEC-082. Review it cold: you did not build it.

- **Under review:** PR 336 (`claude/spec-082-build`, head `59c1b64`), built under
  HANDOFF-193 by a separate session. CI was 22 of 22 green at that head.
- **Your branch:** `claude/spec-082-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-082 verify`.

SPEC-082 extends the shipped writer (SPEC-061, SPEC-081) with an optional caller JPEG
preview:
- `WriteOptions::preview: Option<JpegPreview>`, holding the bytes, the declared width and
  height, and `JpegPreviewColor` (YCbCr or Gray).
- The preview is stored as a SubIFD of the raw IFD0 (DEC-082). IFD0 gains `SubIFDs` (330).
- The preview IFD carries `NewSubFileType` 1 and `Compression` 7, with `Photometric` 6 or 1,
  and the JPEG is copied verbatim.
- Layout: header, EXIF block, IFD0 and its values, then the preview IFD and its values, then
  the JPEG, and the raw strip last.
- Two new `WriteInputError` reasons, `PreviewNotJpeg` and `PreviewDimensionsInvalid`, are
  checked in `plan` after XMP and before the `OpcodeList1` walk.
- Output without a preview must stay byte-identical to SPEC-081's.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-082-optional-caller-jpeg-preview-as-a-reduced-resolution-subfile.md`.
  Judge the ACs against the spec **as designed** (`git show 165a096:<spec path>`): AC1 to
  AC10, Design §1 to §7, and the Failing Tests list.
- **Decisions:** DEC-082 (proposed, the design's choices); DEC-075 §1, §7 and §9 (layout,
  error variants, no allocation); DEC-072 (EXIF block at offset 8); DEC-081; DEC-080;
  DEC-051, DEC-052, DEC-016.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`,
  `library-not-application`.
- **Build handback:** HANDOFF-193's `## Handback` (in the same handoffs directory). It holds
  the pinned MD5s, the exiftool and `dng_validate` results, and five deviations.
- **Prior verifies:** HANDOFF-190 (SPEC-061) and HANDOFF-192 (SPEC-081). Their mutant tables
  are the model to follow.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`.
  - Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
    `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Use `/usr/bin/git` for branch and HEAD.
  - Give each mutant run its own source state. Plain cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC is met against the spec as designed.** Every Failing Tests entry must exist
   and fail when its clause is wrong.
2. **Mutants.** Each mutant must fail a named test; give a table. At least these:
   - Write `NewSubFileType` 0 in the preview IFD.
   - Write `Photometric` 6 for Gray.
   - Write `BitsPerSample` with three values for Gray.
   - Point `StripOffsets` 2 bytes past the JPEG.
   - Drop the pad byte after an odd-length JPEG.
   - Write the `SubIFDs` entry out of tag order.
   - Write `SubIFDs` when there is no preview.
   - Place the raw strip before the JPEG.
   - Accept a JPEG that starts `FF D9`.
   - Accept width 0.
   - Accept height 65536.
   - Run the preview checks after the `OpcodeList1` walk.
   - Run a preview check only in `serialize`, which breaks parity.
3. **The five build deviations.** Rule on each:
   - (a) The MD5s were pinned on `165a096` rather than `386b4a7`. Confirm that the two
     commits' `src/` trees are identical, then re-derive the hashes yourself from `386b4a7`'s
     `src/`.
   - (b) The layout and writing loops were extracted into `place_ifd` and `write_ifd`. Does
     oracle 2 really cover every branch of the old code, including FullSensor with masked
     areas and all three opcode lists?
   - (c) The fuzz "tall" flag reaches height 65536. Do all 32 seeds still reach their named
     outcomes, with the old 28 covering the same cases as before?
   - (d) The SPEC-081 corpus test was refactored. Does it still assert everything it did at
     `386b4a7`?
   - (e) There is one extra boundary test. Is it sound?
4. **Red-proofs.** Oracle 5 must run after its negative control. Also run the new tests
   against `165a096`'s `src/`: everything that asserts preview behaviour must fail there.
5. **The test's raw IFD walker (`raw_ifd`).** Is it independent of the crate's reader? A walker
   that shares the reader's bugs would make oracles 1 and 4 agree for the wrong reason. Check
   one written file's preview IFD by hand in hex against TIFF 6.0 and DNG 1.7: tag order,
   types, counts, even offsets, and the SubIFDs pointer.
6. **No panics and no allocation.** Check the new length arithmetic (JPEG length, pad, offset
   sums near 4 GiB, `OutputTooLarge`). Confirm `written_len` and `write_into` allocate nothing
   when a preview is set. Does the fuzz target reach both new refusals as well as `Ok`?
7. **Readers beyond the oracles.** Quick Look was not re-checked in the build. Use the
   K-3 III file with its own preview: `qlmanage -t` the written file, and check the
   thumbnail is the preview, not a raw render. A distinct-colour preview proves it. Never
   commit the outputs.
8. **Docs.** The module docs ("What is written", "Checks and their order") must list
   `SubIFDs`, the preview IFD and both checks in the order the code runs them. Also check
   the CHANGELOG, the provenance row and the AGENTS.md block.
9. **Gates.** Read `gh pr checks 336` first. Then run:
   - `just test` with the corpus and `dng_validate` set
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`
   - `deny`, `deny-fuzz`, `lint-fuzz`, **`lint-fuzz-red-proof`**, `msrv`
   - `validate`, `gate-audit`, `cost-audit`, `just decisions-audit --changed`
   - `cargo semver-checks`
   - 60 s of `just fuzz-dng-write`
10. **DEC-082.** Is each of its items sound, and does the code follow it? Should it be
    accepted at ship?

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and rulings on items 3, 7
  and 10.
- **Commit** this handoff's filled handback, and any tests or docs fixes, to
  `claude/spec-082-verify-base`, and push it. If you approve:
  1. Run `just advance-cycle SPEC-082 ship --verdict approved`.
  2. Run `just handback-sync SPEC-082` in the same push, so CI's cost audit passes.
  3. Fast-forward `claude/spec-082-build` to the verify tip, so PR 336 carries the verify
     commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** a PROJ-006 session also mints IDs. Before minting a DEC or HANDOFF, compute the
  next free one from `origin/main` plus the open PRs. Right now that is DEC-083 and
  HANDOFF-195.

## Out of Scope

- Decoding or checking the JPEG's content: the colour, the dimensions and whether it is a
  baseline JPEG. These are the caller's declaration (DEC-082).
- Several previews; a preview in IFD0 with the raw in a SubIFD; thumbnails in the EXIF block.
- The three STAGE-017 [XS] items: the empty-opcode-list refusal, the CI `dng_write` fuzz job
  and private intra-doc links.
- Any reader change. If a round trip exposes a reader bug, report it.
- Accepting DEC-082. The orchestrator does that at ship.

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

### Execution notes

- **Branch / PR:** `claude/spec-082-verify-base`, fast-forwarded into `claude/spec-082-build`, https://github.com/jysf/irradiance/pull/336
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes. AC1 to AC10 hold against the spec as designed (`165a096`).
- **For `verify`:** ✅ APPROVED. The code under review is `59c1b64`, PR 336's head, with no `src/` change. The verify commit adds one test assertion (the M6 gap below) and docs.

**CI first.** `gh pr checks 336` at `59c1b64`: 22 of 22 pass.

**Gates, re-run locally** with `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` set. All exit 0:
- `just test`, with no FAILED line; the alloc binaries are included.
- `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`.
- `deny`, `deny-fuzz`, `lint-fuzz`, `lint-fuzz-red-proof` (the control is clean and the injection is rejected with `-D warnings`).
- `msrv`, `validate`, `gate-audit`, `cost-audit`.
- `semver-check`: 202 pass, and no semver update is required (the additions are non_exhaustive).
- `decisions-audit --changed 165a096`: advisory, naming DEC-075, DEC-081 and DEC-082; the code is consistent with each.
- `just fuzz-dng-write 60`: 2,231,927 runs in 61 s, no crash.
- `just fuzz-seeds` regenerates all 32 committed seeds byte for byte.

**Oracles 6 and 7 really ran** (re-run with `--nocapture`). exiftool 13.55 passed. `dng_validate` gave no warnings, and each render equals the no-preview render, for:
- the K-3 III with its own 6192×4128 YCbCr preview (6,440,502 bytes);
- the K-3 III with the gray fixture;
- the Q2M with its own 8368×5584 preview (2,576,384 bytes).

**Mutants** (`src/dng/write.rs`, each on its own source copy in a scratch worktree, `cargo test --all-features --no-fail-fast --test dng_write --test dng_write_alloc`):

| # | Mutant | Killed by |
|---|---|---|
| M1 | `NewSubFileType` 0 in the preview IFD | `preview_ifd_matches_the_spec_table`, `previews_returns_the_callers_jpeg`, `red_proof_a_moved_preview_offset_is_caught`, `round_trip_holds_with_a_preview`, `fuzz_seeds_reach_their_named_outcome` |
| M2 | `Photometric` 6 for Gray | `preview_ifd_matches_the_spec_table`, `previews_returns_the_callers_jpeg` |
| M3 | three `BitsPerSample` for Gray | `preview_ifd_matches_the_spec_table`, `preview_sits_between_ifd0_values_and_the_raw_strip` |
| M4 | `StripOffsets` 2 past the JPEG | `preview_ifd_matches_the_spec_table`, `preview_sits_…`, `previews_returns_…`, `red_proof_…`, `fuzz_seeds_…` |
| M5 | no pad after an odd JPEG | `preview_sits_between_ifd0_values_and_the_raw_strip` |
| M6 | `SubIFDs` slot moved after `XMLPacket` | **survived at `59c1b64`**; killed by `preview_sits_between_ifd0_values_and_the_raw_strip` after this verify's assertion (below) |
| M7 | `SubIFDs` written without a preview | `no_preview_output_is_byte_identical_to_spec_081`, `tags_match_the_spec_table`, `every_new_tag_matches_the_spec_table`, `cutting_framings_omit_active_and_masked_areas` |
| M8 | raw strip placed before the JPEG | 9 tests, including `preview_sits_…`, `previews_returns_…`, `round_trip_holds_with_a_preview`, `written_len_and_write_into_allocate_nothing` |
| M9 | a JPEG starting `FF D9` accepted | `every_refusal_is_typed`, `fuzz_seeds_reach_their_named_outcome` |
| M10 | width 0 accepted | `every_refusal_is_typed`, `checks_run_in_the_documented_order` |
| M11 | height 65536 accepted | `every_refusal_is_typed` |
| M12 | the preview checks after the `OpcodeList1` walk | `checks_run_in_the_documented_order` |
| M13 | the SOI check only in `serialize` (parity broken) | `every_refusal_is_typed`, `checks_run_in_the_documented_order`, `fuzz_seeds_reach_their_named_outcome` |
| X1 | `RowsPerStrip` = width | `preview_ifd_matches_the_spec_table` |
| X2 | `StripByteCounts` one short | `preview_ifd_matches_…`, `previews_returns_…`, `red_proof_…`, `fuzz_seeds_…` |
| X3 | `PlanarConfiguration` 2 | `preview_ifd_matches_the_spec_table` |
| X4 | `Compression` 6 | `preview_ifd_matches_…`, `previews_returns_…`, `red_proof_…` |
| X5 | Gray `SamplesPerPixel` 3 | `preview_ifd_matches_…`, `previews_returns_…` |
| X6 | the preview IFD not aligned to even | `preview_sits_between_ifd0_values_and_the_raw_strip` |
| X7 | YCbCr `BitsPerSample` 8, 8, 0 | `preview_ifd_matches_the_spec_table` |
| X8 | the height left unchecked | `every_refusal_is_typed`, `fuzz_seeds_…` |
| X9 | the JPEG not copied | `preview_ifd_matches_…`, `previews_returns_…`, `red_proof_…`, `fuzz_seeds_…` |
| X10 | no `align2` before the JPEG | survives, **equivalent**: the preview IFD (2 + 12 × 11 + 4 = 138 bytes) and YCbCr's 6-byte value always end even |

**M6 gap, fixed in tests.** `preview_ifd_matches_the_spec_table` checks IFD0's tag order only on a file without XMP. There, moving `SubIFDs` after the absent `XMLPacket` slot still ascends. `preview_sits_between_ifd0_values_and_the_raw_strip` sets every SPEC-081 field in its block case, and now asserts that IFD0's tags ascend. The code was already correct.

**Item 3, the build's deviations:**
- (a) **Accepted.** `src/` is identical at `165a096` and `386b4a7` (`git diff --quiet`). I re-derived the three MD5s by building `386b4a7`'s tree with the test's `oracle_2_outputs` appended. They are `fb81830e…`, `36160d2e…` and `e5081b7d…`, exactly the pinned ones.
- (b) **Accepted.** `place_ifd` and `write_ifd` are the old loops moved without change, and `emit` gains only new arms. Oracle 2 covers both branches:
  - inline and out-of-line values;
  - the odd-length pad, through the odd XMP, the odd EXIF block and the 27-byte model;
  - FullSensor with masked areas and all three opcode lists.

  The SPEC-061 and SPEC-081 suites also pass unchanged.
- (c) **Accepted.** Each of the 28 old seeds is the old seed plus one inserted `00` byte, checked byte by byte. `fuzz_seeds_reach_their_named_outcome` still maps every name to its outcome, with 32 seeds in all. The tall flag is the only way a `u16` field can reach 65536.
- (d) **Accepted.** `metadata_options` sets the same 12 fields from the same sources. `baseline_exposure` is filtered after the call, and the SPEC-081 test asserts what it asserted before.
- (e) **Accepted.** `preview_dimension_and_marker_boundaries_pass` covers the pass side of §4's bounds: 65535×65535, 1×1 and a bare `FF D8`.

**Item 4, red-proofs.**
- Oracle 5 runs its negative control first.
- I also ran "the writer ignores the preview": the API is kept, and the `SubIFDs` slot, the 6e checks and the preview layout are removed, so this is 165a096's behaviour. All 9 preview tests fail and oracle 2 passes.

**Item 5, the walker.** `raw_ifd` and `header_ifd0` read raw LE words and never call the crate's reader. I checked a 24×16 synthetic file with an 11-byte YCbCr preview by hand against TIFF 6.0 and DNG 1.7:
- IFD0 sits at 8 with 19 tags ascending, and `SubIFDs` (330, LONG, count 1) points to 286.
- The preview IFD at 286 holds 11 tags, ascending, with the specified types and counts. Its next offset is 0, as is IFD0's.
- `BitsPerSample` is out of line at 424 (`08 00 08 00 08 00`).
- The JPEG sits at 430 for 11 bytes, followed by a 0 pad at 441.
- The raw strip starts at 442, and 442 + 768 is the file length.
- Every offset is even, and inline SHORTs are left-justified.

**Item 6, no panics and no allocation.** All new arithmetic is checked: the JPEG length to u32, the offsets to u32, `checked_add`, and the final `pos > u32::MAX` that gives `OutputTooLarge`. `preview_entries` and `place_ifd` return fixed arrays. `dng_write_alloc` with a YCbCr preview passes. The fuzz target reaches `Ok` (the `preview` and `preview-gray` seeds) and both new refusals.

**Item 7, Quick Look.** I wrote the K-3 III three times at StandardCrop, ran `qlmanage -t -s 256` on each, and sampled 3 pixels:
- no preview: (231,231,231), the raw render;
- its own preview: (211,211,211), which differs from the raw render;
- a solid-red 256×171 JPEG made with `sips`: (254,0,0) at all 3 points.

So Quick Look shows the preview. Nothing was committed.

**Item 8, docs.**
- The CHANGELOG `[Unreleased]` entry, the provenance row and the AGENTS.md fuzz block are accurate.
- The module docs list `SubIFDs`, the preview IFD's tags, the layout and the check order as the code runs them.
- One ship-time nit, non-blocking and in `src/`: the preview paragraph sits between "…as the base plus each in-block offset given:" and the layout diagram that colon introduces. Move the paragraph after the diagram.

**Item 10, DEC-082.** All six items are sound, and the code follows each:
- §1: one `SubIFDs` LONG, no chain.
- §2: the layout, proven by hand and by X6/M5/M8.
- §3: the tag table.
- §4: no decode.
- §5: the order between `EmptyXmp` and `OpcodeList1`, checked in `plan`.
- §6: every framing.

Accept it at ship. Its confidence of 0.8 is fair: Lightroom and Windows remain untested.

### Cost self-report

- **Tokens (total):** 8,976,828 (floor, measured before the handback commit)
- **Estimated USD:** 6.74
- **Duration (minutes):** about 75
- **Source of the number:** the transcript, deduplicated by `message.id` per docs/cost-tracking.md; no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. No IDs minted.
- **Deviations from spec:** none in the code. The verify adds one test assertion (M6).
- **Follow-up work identified:**
  - Ship-time nit: move the module-doc preview paragraph after the layout diagram.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The red-proof "run the new tests against 165a096's src" cannot compile, because the API is new. Stubbing it with "the writer ignores the preview" was the meaningful version; naming that in the handoff would save a step.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — I would write the mutant table first as a script, with exact-match edits that refuse a pattern count other than 1. It made 24 runs cheap. Here it also exposed that a tag-order check is only as strong as the set of tags present in its fixture.
