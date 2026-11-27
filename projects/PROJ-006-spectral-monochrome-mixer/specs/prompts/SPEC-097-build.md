# SPEC-097 — build cycle prompt

You are the **implementer** for SPEC-097 (PROJ-006, STAGE-034), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (not `rtk git`, which has served
stale branch state in this repo).

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-206-build-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md`
(your context and the handback you must fill),
`projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-097-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md`
(Design, Implementation Context, Acceptance Criteria, Failing Tests and the mutant list are
the contract),
`decisions/DEC-085-a-lossless-jpeg-precision-below-bitspersample-decodes-unchanged-and-white-still-defaults-from-bitspersample.md`,
`decisions/DEC-077-the-develop-refuses-tags-that-change-sample-meaning-and-a-newer-dng-backward-version.md`
§3 and its amendment note, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

- `src/plane.rs`: `check_precision` accepts a Compression 7 frame when
  `P ≤ BitsPerSample ≤ 16` and otherwise returns the existing
  `Error::LjpegPrecisionMismatch { precision, bits_per_sample }`. Name the 16 once. The
  call sites, their order (after `check_strip_shape`, before any sample) and the paths
  (`unpack_into`, `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`, strips and tiles)
  do not move. Update the function's doc to DEC-085.
- `src/lib.rs`: `LjpegPrecisionMismatch`'s doc comment only (Design §4). Its fields, kind
  and `Display` are unchanged.
- `tests/ljpeg_precision_binding.rs`: rewritten to AC1 to AC4. Keep
  `ac5_matching_precision_still_decodes` and `precision_mismatch_display_names_both_values`
  as they are.
- `tests/corpus/manifest.toml`: promote the two `[[available]]` Leica entries to
  `[[file]]` entries with Design §5's values, preview pins included. Remove the
  `[[available]]` pair and keep the never-redistribute note on the M11. Bump
  `MANIFEST_ENTRIES` (12 to 14), `PINNED_PREVIEWS` (21 to 28) and `DECODABLE_COMPRESSED`
  (5 to 7). Add both files to `CFA_METADATA_NOT_YET_ORACLED`, with the reason.
- `tests/leica_colour.rs` (new): AC7 on each present file, skipping by name when absent.
- `fuzz/seeds/plane/`: `compression7-p12-in-bps16.tiff` and `tiled-lj92-p14-in-bps16.tiff`.
- `docs/conformance-matrix.md` (Leica M10 and M11 colour rows), `docs/provenance-ledger.md`
  (one sentence, Design §7), `CHANGELOG.md` `[Unreleased]` Changed.

Do NOT:
- add or rename an `Error` variant, a public item, a dependency or `unsafe`;
- bump `DEVELOP_VERSION` or `MONO_MIX_VERSION`;
- change any existing output, hash or Tier-B pin;
- touch `resolve_white`, `resolve_colour_levels` or the lossless-JPEG decoder.

If an AC7 digest or a preview pin fails, report it; do not edit it. **Never copy
`LEICA-M11-PB/leica_m11_01.dng` (or any `-PB` file) into the repo, a fixture, a seed or
CI.**

## Order of work

1. Write the Failing Tests first and see each fail (or, for AC7 with the corpus present,
   refuse at unpack) against `main`. Record that in the handback.
2. Implement; make them pass with the existing suite green.
3. Apply each mutant (a) to (i) by hand, record which test fails, and revert.
   HANDOFF-202's handback describes a scripted string-replacement harness that worked.
4. Add the two seeds and run the `plane` smoke fuzz:
   `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=60`.
5. Run the gates:
   - `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
     (NEVER `just test --all-features`);
   - `just lint-ci`, `just lint-fuzz`, `just fmt-check`, `just doc-check`;
   - `just validate`, `just semver-check` (no change expected), `just cost-audit`.
   Confirm that `plane_md5_matches_the_pinned_raw_checksum` and
   `ljpeg_decode_fault_turns_compression7_oracle_red` ran on both Leica files: green for
   the first, red under the fault for the second.

## Hand back

Fill HANDOFF-206's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section. Then open a PR against `main`
(SPEC-097 only) from `feat/spec-097-ljpeg-precision-below-bps`, referencing HANDOFF-206,
SPEC-097, STAGE-034, PROJ-006, DEC-085, DEC-077, DEC-029 and DEC-047.

The commit-message BODY must be plain prose: a push with code tokens like `#[...]`, `->`
or `--flag` is rejected; reword it, do not retry. End it with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back:
- the PR number;
- whether AC7's digests and the preview pins matched as registered;
- the lowest P the AC1 grid reached;
- which test each mutant failed;
- the `plane` fuzz run count;
- both Tier-B oracle results per Leica file;
- the true token cost.
