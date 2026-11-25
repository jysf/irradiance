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
  status: pending                  # pending | accepted | completed | rejected

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
