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
  id: HANDOFF-199
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-095

project:
  id: PROJ-005
  stage: STAGE-032
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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7895841            # REAL combined count — what cost-audit reads
  estimated_usd: 6.08              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-095-verify-base
  pr: 345
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 680b6fef-017f-400e-9db9-3a5b42170bde.jsonl (the scratchpad UUID; 51 unique message.ids, all claude-opus-5-5, no subagents): input 101 + cache-creation 148,391 (1-hour) + cache-read 7,718,002 + output 29,347, deduplicated by message.id with each message's largest output_tokens, priced at 5 and 25 USD per million, cache read 0.1x, 1-hour cache writes 2x. Approved at candidate 252360b; wording fixes only (CHANGELOG names SPEC-061's fourteen reasons; two release-note lines). Own consumer on the Leica M Monochrom Typ 246 with EXIF block, XMP, descriptive tags, hints and its own JPEG preview: deterministic, sample-equal, dng_validate 1.7.1 clean."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-199: VERIFY: irradiance 0.2.7 release, the public DNG writer (SPEC-095)

## Delegation Summary

You are the **cold verifier** for SPEC-095's Phase 1, the reversible prep of the 0.2.7
release. You did not build it. Nothing is tagged or published, and you must not tag or
publish either.

- **Under review:** PR 345 (`release/0.2.7-prep`, head `cdf42ed`), built under HANDOFF-197.
  The candidate is `252360b` (the version bump); the commits after it are records only.
  CI was 23 of 23 green on `f561181`; the run on `cdf42ed` (handback only) was in progress
  at dispatch.
- **Your branch:** `claude/spec-095-verify-base`, which is that head plus one commit adding
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-095
  verify`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md`.
  Judge it against the spec **as designed** (`git show 3f9df0b:<spec path>`): Release
  Scope, Build work steps 1 to 11, and the Phase 2 runbook. The build filled the Release
  Notes, the internal record and the Pre-Flight; those are what you review.
- **The model:** SPEC-090 (the 0.2.6 release, `projects/PROJ-002-bayer-colour-develop/specs/done/`)
  and its cold verify HANDOFF-188, which found missed changed inputs.
- **Decisions:** DEC-006, DEC-028, DEC-067, DEC-055, DEC-071, DEC-080, DEC-082, DEC-050
  (the pixel budget behind `PlaneTooLarge`).
- **Constraints:** `library-not-application`, `no-new-top-level-deps-without-decision`.
- **Build handback:** HANDOFF-197's `## Handback`, with three deviations.
- **Toolchain:**
  - `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`;
  - `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
    (outside the irradiance checkout; never commit it or its outputs);
  - run `just test`, never `just test --all-features`; use `/usr/bin/git` for branch and
    HEAD; scratch crates and unpacked packages go in your scratchpad.

## What to check hard

1. **The cut.** PR 343 (SPEC-091) is not in: `git diff --stat 153f517 252360b -- src
   Cargo.toml` shows only the version and the docs.rs table. Confirm PR 343 is still
   unmerged and say so.
2. **The changed-input list, re-derived yourself** from `git diff v0.2.6 252360b -- src/`:
   is any reader, develop, colour or `irr` line changed? Does anything outside the writer
   return the two new `Error` variants? Rule on the build's `PlaneTooLarge` addition: is
   the writer's budget check really the existing variant with its existing kind, and are
   the notes right about it? Is any other writer refusal missing from the CHANGELOG or the
   notes (count them against `WriteInputError` and every `Err(` in `src/dng/write.rs`)?
3. **Semver,** re-run: `just semver-check` and `cargo semver-checks --all-features`, after
   the bump. Both must say no update required.
4. **The packaged crate, independently.** Run `cargo package` yourself on a clean tree of
   the candidate. Then:
   - the packaged `Cargo.toml` keeps `[package.metadata.docs.rs] features = ["write"]`
     after cargo's normalisation, and has no self dev-dependency;
   - in the unpacked package, build and doc as docs.rs would (`RUSTDOCFLAGS='-D warnings'
     cargo doc --no-deps --features write`) and confirm `dng::write` is in the output;
   - your own consumer crate, written cold, writes a **different** held monochrome DNG
     than the build used (it used the Q2 Monochrom `L1021223.DNG`) at all three framings
     with the EXIF block, XMP, descriptive tags and a JPEG preview set where the file
     gives you them. Re-read each output, compare samples, and run `dng_validate` on it.
     Determinism: two runs give identical bytes;
   - without `write` the consumer cannot name `irradiance::dng`.
   - The package list is 0.2.6's 19 files plus exactly the three writer files.
5. **Release Notes, line by line.** Check every claim against the code and the tests:
   the headline ("same input, same bytes", "no allocation in `write_into`", "no new
   dependency", "dng_validate accepts"), each Added item, "Breaking / action required:
   none", and every Known limitation, writer and reader. Flag anything overstated or
   missing, as HANDOFF-188 did for 0.2.6.
6. **CHANGELOG.** `[Unreleased]` promoted to `## [0.2.7] - <date>` with a lead paragraph
   and a fresh empty `[Unreleased]`; the stale reason count corrected; every entry true of
   the shipped code.
7. **README.** The writer section matches the shipped API: options from SPEC-081 and
   SPEC-082, SPEC-094's refusal. If its example is not compiled anywhere, compile it
   yourself in the consumer crate.
8. **The Pre-Flight and the Phase 2 runbook.** Every `[now]` has evidence you could
   reproduce; every `[cut]` is something the user can actually run. The runbook uses an
   annotated tag and the `ls-remote` `^{}` check, says to confirm PR 343 is unmerged, and
   its `[cut]` smoke on `irradiance = "=0.2.7"` with `write` would tell 0.2.7 from 0.2.6.
9. **The build's three deviations.** Rule on each: `PlaneTooLarge` (item 2); `write` added
   to the self dev-dependency's features (stripped by `cargo package`? confirm in the
   packaged manifest); the wrong `dng_validate` path in the brief (records only).
10. **Gates.** Read `gh pr checks 345` first. Then run `just test` with the corpus and
    `dng_validate` set, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `lint-fuzz-red-proof`,
    `fmt-check`, `validate`, `doc-check`, `deny`, `deny-fuzz`, `msrv`, `cost-audit`,
    `gate-audit`, `name-length-check`, `decisions-index --check`,
    `package-targets-check`, and `cargo publish --dry-run` on a clean tree.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED, with
  evidence for every item above.
- **Fixes on your branch:** wording in the CHANGELOG, README, Release Notes, Pre-Flight
  and records only. A change to `src/`, tests, fuzz, `.github/` or any `Cargo.toml` line
  goes in a punch list.
- Commit this handoff's filled handback and any fixes to `claude/spec-095-verify-base`,
  and push it. If you approve:
  1. Run `just advance-cycle SPEC-095 ship --verdict approved`.
  2. Run `just handback-sync SPEC-095` in the same push, so CI's cost audit passes.
  3. Fast-forward `release/0.2.7-prep` to the verify tip, so PR 345 carries the verify
     commits. Do not merge.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** compute any next-free DEC or HANDOFF from `origin/main` plus the open PRs.

## Out of Scope

- Tagging, `cargo publish`, a GitHub release: Phase 2 is the user's.
- PROJ-006's SPEC-091 (PR 343), and the STAGE-017 close's two process-debt items.
- Any library behaviour change.

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

- **Branch / PR:** `claude/spec-095-verify-base` (PR 345's head `cdf42ed` plus the verify
  handoff, then this verify's commits), fast-forwarded onto `release/0.2.7-prep` so
  https://github.com/jysf/irradiance/pull/345 carries them. Not merged, tagged or published.
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes. All ten "What to check hard" items pass. Fixes
  are wording only.
- **For `verify`:** ✅ **APPROVED at `252360b`** (`252360bcea981dc8dfb46179a9a2be91ee0b6933`,
  the candidate that gets tagged once it merges). The records and wording on top of it
  change no `src/`, test, fuzz, `.github/` or `Cargo.toml` line.

**Evidence, item by item** (re-run by me; the build's numbers not reused):

1. **The cut.** `git diff --stat 153f517 252360b -- src Cargo.toml` is `Cargo.toml` only,
   +8/−1: the version line and the commented `[package.metadata.docs.rs] features =
   ["write"]` table. `git diff --stat 153f517 origin/main -- src Cargo.toml` is empty
   (`origin/main` = `3f9df0b`). `gh pr view 343`: `state OPEN`, `mergedAt null`, so PR 343
   is still unmerged and not in the release.
2. **Changed inputs, re-derived.** `git diff --stat v0.2.6 252360b -- src/ Cargo.toml`: the
   three new writer files, `src/lib.rs` (+47/−3) and `Cargo.toml`. I read the whole
   `src/lib.rs` diff: the gated `pub mod dng`, the private `mod write_types`, the two
   variants, two `Display` arms and two `kind()` arms (the −3 are the arms' line ends). No
   reader, develop, colour, `irr` or `ErrorKind` line. A grep for both new variants outside
   `src/lib.rs` finds constructions only in `src/dng/write.rs` (line 633's `invalid()` and
   line 860). **`PlaneTooLarge`:** `plan()` step 1 returns the existing
   `Error::PlaneTooLarge { pixels, limit }` when `width × height` exceeds `max_pixels`,
   before any other check; it keeps its existing kind (no `kind()` line changed). The notes,
   CHANGELOG and README describe it correctly. **Refusal count:** `WriteInputError` has 22
   variants, and every one is constructed in `src/dng/write.rs` (`InvalidAscii`,
   `RationalInvalid`, `OutputTooLarge` and `OutputBufferWrongLength` at two sites each).
   Every `Err(` in the file is one of those, `PlaneTooLarge` or the one
   `UnsupportedOpcodeForFraming`; the `too_large()` sites inside the strip copy are
   `OutputTooLarge`. No refusal is missing from the notes, which name all 22 in prose.
   One gap was in the CHANGELOG, which named only the eight later reasons and counted
   SPEC-061's fourteen; fixed (wording).
3. **Semver** (`cargo-semver-checks 0.51.0`, clean detached worktree of `252360b`): `just
   semver-check` and `cargo semver-checks --all-features` both print "v0.2.6 -> v0.2.7
   (minor change) … 202 checks: 202 pass, 58 skip … no semver update required".
   `size_of::<irradiance::Error>()`: 40 (align 8) on `v0.2.6` and on the unpacked
   package, with and without `write` (my own scratch crates).
4. **Packaged crate, independently.** `cargo package` on that clean worktree: `Packaged 22
   files, 1.9MiB (416.6KiB compressed)`, verified; `.cargo_vcs_info.json` `sha1`
   `252360bcea981dc8dfb46179a9a2be91ee0b6933`. The normalised manifest keeps
   `[package.metadata.docs.rs] features = ["write"]`, `version = "0.2.7"`, `rust-version =
   "1.90"` and an empty `[dependencies]`. Its only dev-dependencies are `ssimulacra2` and
   `toml`, with no self dev-dependency. `irr` is `required-features = ["irr"]`. In the
   unpacked package: `cargo build --features write` and `--no-default-features` exit 0;
   `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --features write` exit 0 and generates
   `irradiance/dng/write/` (`write_into`, `write_to_vec`, `written_len`, `exif_block_base`,
   `EXIF_BLOCK_BASE`, `Framing`, `JpegPreviewColor`, `WriteInputError`, `ExifBlock`,
   `JpegPreview`, `WriteOptions`). **My consumer** (written cold, release build, path
   dependency on the unpacked package with `write`) reads the **Leica M Monochrom (Typ 246)
   `LEICA-M-MONOCHROM-TYP246/M2462362.DNG`**: 5984×4000, 12-bit, no `ActiveArea`, crop
   4,4 + 5976×3992, black 0, white 3750, no opcode lists. It writes the file at all three
   framings with:
   - its eight descriptive tags read from IFD0 (`Make`, `Model`, `Software`, an empty
     `Artist` and `Copyright`, `CameraSerialNumber`; `DateTime` and
     `LocalizedCameraModel` absent);
   - its four hints;
   - an XMP packet;
   - a hand-built Exif IFD whose `ExposureTime` value is out of line at an absolute offset
     built for `exif_block_base()`;
   - the file's own 160×120 YCbCr JPEG preview (5,929 bytes).
   For each output, `written_len` equals the length, and `write_into` into a 0xAA-filled
   buffer equals `write_to_vec`. The re-read samples equal the source, 23,936,000 of
   23,936,000, and `previews()` returns the caller's JPEG byte for byte. exiftool reads
   back `ExposureTime 1/125`, ISO 320, `ExifVersion 0232`, `Rating 3`, the strings and
   the crop. The output is 47,878,962 bytes at every framing. MD5s:
   - `FullSensor` and `FullActiveArea`: `53baf35064fae33489da78b63877b22f`, identical
     as expected, since the file has no `ActiveArea` and so no margin;
   - `StandardCrop`: `47ea338228c300a733ed75781aea261c`.
   A second run gave the same three MD5s. Adobe `dng_validate` 1.7.1 on each: exit 0,
   "Validation complete", no `Warning` or `Error` line. The README's example, compiled
   verbatim in the same crate, runs and writes 47,872,272 bytes. A second consumer
   without `write` fails with `error[E0433]` ("the item is gated behind the `write`
   feature"). `cargo package --list` on `v0.2.6` and `252360b`, diffed: 19 → 22, the
   additions exactly `src/dng/mod.rs`, `src/dng/write.rs`, `src/write_types.rs`.
   (The M Monochrom CCD, `L1000622.DNG`, was tried first: it has no JPEG preview, so the
   Typ 246 is the file.)
5. **Release Notes, line by line.** True of the code and tests:
   - the headline: determinism is pinned by `write_is_deterministic_whatever_the_buffer_held`;
     the no-allocation claim by `tests/dng_write_alloc.rs`; `[dependencies]` is empty;
     `dng_validate` covers all four held mono files at three framings in the suite, plus
     my Typ 246 run;
   - every Added item;
   - "Breaking: none";
   - every Known limitation.
   Two lines fixed (wording):
   - "Refusals, each checked by `written_len` too" overstated it, since the
     buffer-length reason is `write_into`'s alone;
   - "Without a preview the output is byte-identical to a file written without the field"
     was circular. It now says IFD0 gets no `SubIFDs` entry, as the module docs say.
   Nothing missing.
6. **CHANGELOG.** `## [0.2.7] - 2026-11-25` with a lead paragraph and a fresh empty
   `## [Unreleased]`. The stale count is corrected to 22, and every entry is true of the
   code (including SPEC-094's "a 4-byte list declaring 0 opcodes is still written",
   which `walk_list_1` accepts). Fix: SPEC-061's fourteen reasons are now named, so the
   CHANGELOG names all 22, as step 5 asked.
7. **README.** The writer section lists every SPEC-081 and SPEC-082 option, SPEC-094's
   empty-list refusal, `PlaneTooLarge` and `UnsupportedOpcodeForFraming`, and `write_to_vec`.
   Its example is not compiled anywhere in the repository; I compiled and ran it (item 4).
8. **Pre-Flight and runbook.** Each `[now]` reproduced: dry run 22 files, clean tree,
   no `v0.2.7` tag locally or on origin, `v0.2.6` annotated, the vcs sha1, the manifest
   fields, the package list, `deny`, `package-targets-check` and the consumer checks.
   Each `[cut]` is runnable. The runbook confirms PR 343 is unmerged (step 1), uses
   `git tag -a`, and checks `cat-file -t` and `ls-remote … '^{}'`. The `[cut]` smoke on
   `=0.2.7` with `write` would tell 0.2.7 from 0.2.6, because 0.2.6 has no `write`
   feature and the MD5 pin is the package's.
9. **The build's deviations, ruled:**
   - **`PlaneTooLarge`: upheld.** It is the existing variant with its existing kind,
     checked first, and the notes are right about it. The design missed it; this was not
     a defect.
   - **`write` on the self dev-dependency: upheld.** `cargo package` strips it; the
     packaged manifest has no self dev-dependency. Records only.
   - **The `dng_validate` path: upheld, records only.** The binary is at
     `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`. The
     spec's "Notes for the Implementer" still says "under `tools/dng_sdk`", which is
     correct relative to the experiments folder; left as is.
10. **Gates.** `gh pr checks 345` read first: 23 of 23 pass on `cdf42ed`. `just test` on
    `f8721cf` (`src/` identical to `252360b`) with the corpus and `dng_validate` set:
    55 results, 923 passed, 0 failed, 1 ignored, exit 0. It printed "corpus: 12/12
    present", exiftool and dnglab on PATH, and "dng-validate: present". The three writer
    `dng_validate` tests ran and passed (`..._renders_like_the_source`,
    `..._the_k3_renders_like_its_source`, `..._a_written_preview`). The SDK colour and
    stage-2 lanes SKIPped by design. On the clean `252360b` worktree these exited 0:
    `lint-ci`, `lint-no-allow`, `lint-fuzz`, `lint-fuzz-red-proof`, `fmt-check`,
    `doc-check` (2 surfaces, 0 warnings), `msrv` (`cargo +1.90.0`), `gate-audit`,
    `name-length-check`, `deny`, `deny-fuzz`, `package-targets-check` and `cargo publish
    --dry-run` ("aborting upload due to dry run"). The records gates (`validate`,
    `cost-audit`, `decisions-index --check`) ran on the verify tip after
    `advance-cycle` and `handback-sync`.

### Cost self-report

- **Tokens (total):** 7,895,841 (a floor, measured before the handback commit)
- **Estimated USD:** 6.08
- **Duration (minutes):** about 60
- **Source of the number:** the session transcript `680b6fef-017f-400e-9db9-3a5b42170bde.jsonl`,
  deduplicated by `message.id` per docs/cost-tracking.md (basis: transcript). 51 unique
  messages, all `claude-opus-5-5`, no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none of my own. My wording fixes:
  - `CHANGELOG.md`: SPEC-061's fourteen reasons named;
  - the spec's Release Notes: the refusal lead-in and the preview line;
  - the internal record: a note that the CHANGELOG sentence was made true by the verify.
- **Follow-up work identified:**
  - none blocking;
  - observation for Phase 2: the CHANGELOG heading is dated 2026-11-25. If the tag lands
    on a later day, the date will not match the publish; 0.2.6 has the same exposure.
  - The README's writer example is compiled nowhere in CI. A doctest or an example
    target would pin it; that is a `src/` or tests change, for a later spec.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The handoff's "different held monochrome DNG" left me to find which
   one has a JPEG preview. The M Monochrom CCD has none; the Typ 246 has a 160×120 one.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-050 was listed for `PlaneTooLarge`, and that was enough.

3. **If you did this task again, what would you do differently?**
   — Pick a held file with an `ActiveArea` margin and a preview for the consumer, if one
   exists outside the Q2M. The Typ 246 has no margin, so its `FullSensor` and
   `FullActiveArea` outputs are identical. The suite covers the margin on all four files,
   but my independent run did not exercise it.
