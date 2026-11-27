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
  id: HANDOFF-206
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-097

project:
  id: PROJ-006
  stage: STAGE-034
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
  tokens_total: 17442907           # REAL combined count — what cost-audit reads
  estimated_usd: 11.98             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 24
  branch: feat/spec-097-ljpeg-precision-below-bps
  pr: 362
  completed_at: 2026-11-27         # YYYY-MM-DD
  notes: "Measured live from this session transcript, deduplicated by message.id: a floor."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-27
---

# HANDOFF-206: Accept a lossless-JPEG precision below BitsPerSample, the Leica M10 file

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-097` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. `check_precision` accepts a
Compression 7 frame when `P ≤ BitsPerSample ≤ 16`, not only when `P = BitsPerSample`. The
samples are stored as decoded, and white still defaults from `BitsPerSample`. The Leica
M10 and M11 colour files are promoted to `[[file]]` manifest entries and pinned. There is
no new `Error` variant, no version bump and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-097-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md` (Design §1 to §7, Implementation Context, AC1 to AC8, Failing Tests, mutants (a) to (i)). The spec is the contract.
- **Decision:** `./decisions/DEC-085-a-lossless-jpeg-precision-below-bitspersample-decodes-unchanged-and-white-still-defaults-from-bitspersample.md` (proposed; the rule, the samples, the white default, what does not change)
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-085`: this spec's decision. The rule is `P ≤ BitsPerSample ≤ 16`; samples are unshifted; white comes from `BitsPerSample`.
- `DEC-077` §3, as amended by DEC-085: where the check sits, its error and its paths, all unchanged.
- `DEC-029`: samples above `WhiteLevel` are legal and clip at white. The M10 has 5.2% of its samples there.
- `DEC-047`: the frame's shape binding, which still runs before the precision check.
- `DEC-061` / `DEC-070`: the mono and colour level maps, unchanged.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.
- `DEC-069` §6: the SDK is read and run locally only. It is never vendored, built in CI or linked, and none of its output is committed.

### Constraints that apply

- `no-panics-on-untrusted-input`: the check is two comparisons; the `plane` fuzz smoke covers the newly reachable decode (AC8).
- `oracle-must-be-shown-red`: the dnglab checksum oracle's existing red-proof covers the two new files; mutants (a) to (i).
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: no new algorithm. Add one sentence to the ledger's lossless-JPEG row for DEC-085's class-4 SDK reading (Design §7).

### Prior related work

- SPEC-086 / DEC-077: the check being relaxed, and `tests/ljpeg_precision_binding.rs`.
- SPEC-044: the dnglab `--raw-checksum` plane oracle and its Compression 7 red-proof.
- SPEC-062: the manifest's preview pins and their exiftool procedure.

### Facts the design probe measured (2026-11-27, release build, this machine)

- **The full table is in the spec's `## Implementation Context`:** SOF-3 headers, sample
  facts, dnglab checksums, stage-1 and stage-2 agreement, and the FNV-1a digests per entry.
- **Both files reach every entry with `Ok`** once the check is relaxed. Neither has
  opcode lists.
- **AC7's digests are pins from the prototype, not an oracle.** Record them as given, and
  report a mismatch rather than editing it.
- **The manifest values in Design §5** were measured on disk: sha256, bytes, dnglab
  checksums, and exiftool's preview pins.
- **Probe source** (not in the repo): `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b/spec-097-design-probe.rs.txt`.
  - It ran as a throwaway `examples/` file with `cargo build --release --example` against
    a `check_precision` patched to `>`.
  - It reads `dng_validate` stage-1 and stage-2 TIFFs from a scratch directory. The build
    does not need them.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  The M11 lives at `LEICA-M11-PB/leica_m11_01.dng`. **Never copy it anywhere in the
  repo.**

## Expected Deliverables

- Code and doc changes implementing SPEC-097's Acceptance Criteria (Design §1 to §7).
- All Failing Tests written first, seen failing against `main`, then passing.
- The mutant table (a) to (i), with the test each mutant failed.
- The `plane` fuzz smoke run with the two new seeds.
- A PR against `main` from branch `feat/spec-097-ljpeg-precision-below-bps`.
- The PR description references HANDOFF-206, SPEC-097, STAGE-034, PROJ-006, DEC-085,
  DEC-077, DEC-029 and DEC-047. It also gives:
  - provenance: no new algorithm, plus the ledger sentence;
  - the oracle: dnglab's checksum on both files, and the red-proof that covers them.

## Out of Scope

- Mandatory `FixBadPixelsConstant` / `WarpRectilinear` on the colour entries (the Q2, Q3,
  SL3): STAGE-034's next spec.
- `P > BitsPerSample` or `BitsPerSample > 16`: still refused.
- Any change to the white default, the decoder, `resolve_white` or
  `resolve_colour_levels`.
- A new `Error` variant, a version bump, or any change to an existing output, hash or
  Tier-B pin.
- Committing, copying or redistributing any `-PB` corpus file.

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

- **Branch / PR:** `feat/spec-097-ljpeg-precision-below-bps`, https://github.com/jysf/irradiance/pull/362. Not merged.
- **Completed at:** 2026-11-27
- **All acceptance criteria met?** Yes. AC1 to AC8 hold; nothing was edited to match.
- **CI:** CI run 46021168302 on `4a02aa0` is green, all 25 checks. The handback commit after it is docs only.

### Results

- **Red first.** Against `main` (d3af59b), before the change:
  - `ac1_precision_below_bits_per_sample_decodes_unchanged` and `ac4_white_defaults_from_bits_per_sample_not_precision` failed with `LjpegPrecisionMismatch { precision: 2, bits_per_sample: 16 }` and `{ 12, 16 }`.
  - `ac2_…`, `ac3_…`, `ac5_matching_…` and the Display test were green, as the spec predicted.
  - `tests/leica_colour.rs` (both tests) and `plane_md5_matches_the_pinned_raw_checksum`, `compressed_files_are_skipped_by_name`, `compression7_plane_is_byte_identical_across_two_decodes` and `ljpeg_decode_fault_turns_compression7_oracle_red` failed with the corpus present: the M10 refused at unpack.
  - The preview pins (`corpus_previews_match_exiftool_pins`) and the count bumps were already green on `main`: preview walks don't decode the plane.
- **AC1:** the lowest P the grid reached is **2** (P2 in BPS16). `tiff::lj92_strip` encodes it; no encoder change.
  - Grid: (2,16), (8,16), (12,16), (14,16), (15,16), (8,12), (12,14), both orders, strips and tiles, all three paths.
  - Every cell decodes to exactly the encoded samples.
- **AC2 / AC3:** every listed pair refuses with `LjpegPrecisionMismatch { precision, bits_per_sample }`, `ErrorKind::Corrupt`, on every cell, and a sentinel-filled `dst` is untouched. Containers with `BitsPerSample` 17, 20 and 32 parse and reach the check; nothing refuses them earlier.
- **AC4:** P12 in BPS16 with no `WhiteLevel` develops to exactly the decoded samples. With `WhiteLevel` 4095 it gives the hand-computed `round(min(v, 4095) · 65535 / 4095)`.
- **AC5:** `plane_md5_matches_the_pinned_raw_checksum` passes on both files: the M10 at `a1c6050a2cbfaf7d1b9a827ee3fd5bf6` and the M11 at `9ae8c096b5ed99b86a88e52d8025168c`, both equal to dnglab.
  - `ljpeg_decode_fault_turns_compression7_oracle_red` goes red on each under the fault: M10 mutant `90a70468a8070a869b565740adb55879`, M11 mutant `8d262e716b4824d3c72f4a75c41f32b1`.
- **AC6:** every existing plane pin, determinism hash and Tier-B develop pin is unchanged and green. `DEVELOP_VERSION` is still `"8"` and `MONO_MIX_VERSION` still `"1"`. `just semver-check`: no semver update required (202 checks pass).
- **AC7:** all ten digests matched the Implementation Context table as registered, and both developed sizes matched (5976×3984, 9528×6328). The M10's facts hold: max 16383, 1,243,041 above `WhiteLevel` 15000, none at or above 2^14.
- **Preview pins:** all 7 new pins matched exiftool's values as registered (`corpus_previews_match_exiftool_pins`, 28 of 28).
- **AC8:** the `plane` smoke ran **1,644,814 runs in 61 s** with no crash. It used the two new seeds, `compression7-p12-in-bps16.tiff` (one strip, little-endian) and `tiled-lj92-p14-in-bps16.tiff` (one tile, big-endian), written by `examples/fuzz-seeds.rs`. Regenerating the seeds left every existing seed byte-identical.
- **Gates:** all green. `just test` with the corpus, 14 of 14 files present: 993 passed, 0 failed, 1 ignored. A `--no-fail-fast` corpus run also gave 974 passed, 0 failed. `lint-ci`, `lint-fuzz`, `fmt-check`, `doc-check`, `validate` and `cost-audit` pass.

### The mutants

Each was applied by a string-replacement harness, then `ljpeg_precision_binding` (and `plane_oracle` for (i)) ran with the corpus present. The file was restored after each run. All are red.

- (a) the old equality: `ac1_…`, `ac4_…`.
- (b) `<` for the P bound: `ac5_matching_precision_still_decodes`.
- (c) the `≤ 16` bound dropped: `ac3_…`.
- (d) the P bound dropped: `ac2_…`.
- (e) the bound as `< 16`: `ac1_…`, `ac4_…`, `ac5_matching_…`.
- (f) the tile arm's call removed: `ac2_…`, `ac3_…` (the tiled cells).
- (g) the strip arm's call removed: `ac2_…`, `ac3_…` (the strip cells).
- (h) white default `2^(BitsPerSample − 4) − 1`: `ac4_…`.
- (i) a shift `v << (BitsPerSample − P)` after the strip decode: `ac1_…`, `ac4_…`, and with the corpus present `plane_md5_matches_the_pinned_raw_checksum`, `compression7_plane_is_byte_identical_across_two_decodes`, `the_ljpeg_red_proof_control_is_green` and `ljpeg_decode_fault_turns_compression7_oracle_red`.

### Cost self-report

- **Tokens (total):** 17,442,907 (input 183, cache creation 208,729, cache read 17,182,044, output 51,951), a floor
- **Estimated about 11.98, at Opus 5.5 list rates of 5 and 25 USD per million input and output, with 1-hour cache writes at 2.0×:** about 11.98, at Opus 5.5 list rates of 5 and 25 USD per million input and output, with 1-hour cache writes at 2.0×
- **Duration (minutes):** 24, by the transcript timestamps
- **Source of the number:** this session's transcript, deduplicated by `message.id` per `docs/cost-tracking.md`, measured live (a floor).

### Drift and new artifacts

- **New decisions emitted:** none. DEC-085 is the design's.
- **Deviations from spec:**
  - `tests/corpus_manifest.rs`'s `manifest_preview_pins_parse` also hard-codes the preview total. It went from 21 to 28 along with `PINNED_PREVIEWS`; the spec named only the latter.
  - The 16 is a private constant, `MAX_LJPEG_PRECISION` in `src/plane.rs`. The `ljpeg` module has no named bound (its parser uses a literal `2..=16`), and the decoder was out of bounds to touch.
  - The seeds are generated by `examples/fuzz-seeds.rs` (the repo's seed source), not written by the test. They use the same writer and the same 16×16 LinearRaw shape.
- **Follow-up work identified:** none new. The design's two open items stand: onyx on highlight headroom above `WhiteLevel`, and the M11's licence class, which the maintainer ruled `personal-evaluation-only` (2026-11-27).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The second hard-coded preview total (`manifest_preview_pins_parse`) wasn't in the list of counts to bump. Also, `fuzz/corpus/plane` doesn't exist in a fresh worktree, so the smoke command fails until it's created.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — Grep for every literal count of manifest entries and previews before the first run, not only the named constants.
