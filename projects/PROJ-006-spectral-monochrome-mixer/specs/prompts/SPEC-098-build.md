# SPEC-098 — build cycle prompt

You are the **implementer** for SPEC-098 (PROJ-006, STAGE-034), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (not `rtk git`, which has served
stale branch state in this repo).

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-208-build-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md`
(your context and the handback you must fill),
`projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-098-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md`
(Design, Implementation Context, Acceptance Criteria, Failing Tests, the rule-clause map,
the mutants, the moved assertions and the count sites are the contract),
`decisions/DEC-086-the-colour-entries-fix-bad-pixels-from-the-four-nearest-same-colour-photosites-and-sensor-geometry-passes-the-warp-through.md`,
`decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md`
item 3 and its amendment note, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

- `src/opcode.rs`: add `DevelopPath::SensorGeometry`. In `DevelopPath::applied`, list 1 is
  the first `FixBadPixelsConstant` for `Mono`, `Colour`, `SensorGeometry` and `MonoMix`;
  list 3 is the first `WarpRectilinear` for `Mono` and `SensorGeometry`, and for `MonoMix`
  the first one only when its `N` (first parameter word) is 1. `SensorGeometry`'s
  malformed sets are the colour three. Update the docs of `DevelopPath`,
  `check_required_processing` and `check_list`.
- `src/color.rs`: resolve the fix once per call (after the `GainMap`s, before the CFA:
  levels, GainMaps, then `parse_fix_bad_pixels_constant`, errors propagated) into a small
  `Copy` value, and apply it at the raw read of `slot_normalizer` and
  `pattern_normalizer` (Design §2–§3). The plane and mixed-plane entries ask with
  `SensorGeometry`; document on each that the plane is not warped and the consumer applies
  `OpcodeList3`. `resolve_mix_stage_two` resolves the fix too.
- `src/develop.rs`: under the mix, skip an optional first warp whose `planes != 1`;
  `scratch_len` is unchanged. Docs for `DevelopOptions::mono_mix` and `develop_core`.
- `src/lib.rs` and `fuzz/fuzz_targets/opcode.rs`: the four-path chain; the N = 3 seed.
- Tests: `tests/colour_bad_pixels.rs` (new), `tests/leica_colour.rs` (extended),
  `tests/required_processing.rs` (`ac6_every_path_applies_its_table`), and every moved
  assertion the spec lists.
- `tests/corpus/manifest.toml`: promote the three `[[available]]` entries to `[[file]]`
  with Design §7's values and preview pins, keep the never-redistribute note on the Q3 and
  SL3, and update every count site the spec lists (`MANIFEST_ENTRIES` 14 to 17, the
  preview total and `PINNED_PREVIEWS` 28 to 37, `DECODABLE` 6 to 7, `DECODABLE_COMPRESSED`
  7 to 9, `CFA_CHECKED_ELSEWHERE` 7 to 10, `CFA_METADATA_NOT_YET_ORACLED` 5 to 8, the
  manifest header).
- `docs/provenance-ledger.md`, `docs/conformance-matrix.md`, `CHANGELOG.md` `[Unreleased]`.

Do NOT:
- add or rename an `Error` variant, a dependency or `unsafe`; add any public item but
  `DevelopPath::SensorGeometry`;
- bump `DEVELOP_VERSION` or `MONO_MIX_VERSION`;
- change any existing output, hash or Tier-B pin, the mono path or `src/warp.rs`;
- apply the warp anywhere (SPEC-099).

If an AC7 digest, an AC1 literal or a preview pin fails, report it; do not edit it.
**Never copy `LEICA-Q3-PB/leica_q3_01.dng`, `LEICA-SL3-PB/leica_sl3_01.dng` (or any `-PB`
file) into the repo, a fixture, a seed or CI.**

## Order of work

1. Write the Failing Tests first and see each fail (or, with the corpus, refuse) against
   `main`. Record that in the handback.
2. Implement; make them pass, and move each listed assertion as the spec says.
3. Apply each mutant (a) to (r) by hand, record which test fails, and revert.
   HANDOFF-202's handback describes a scripted string-replacement harness that worked.
4. Add the seed and run the `opcode` smoke fuzz:
   `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run opcode fuzz/corpus/opcode fuzz/seeds/opcode -- -max_total_time=60`
   (`mkdir -p fuzz/corpus/opcode` first in a fresh worktree).
5. Run the gates:
   - `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
     (NEVER `just test --all-features`);
   - `just lint-ci`, `just lint-fuzz`, `just fmt-check`, `just doc-check`, `just wasm-check`;
   - `just validate`, `just semver-check` (minor, additive), `just cost-audit`.
   Confirm that AC7, AC8 and AC11 ran on all three Leica files (not skipped), and that the
   Compression 7 red-proof turned red on the Q3 and SL3.

## Hand back

Fill HANDOFF-208's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section. Then open a PR against `main`
(SPEC-098 only) from `feat/spec-098-colour-fix-bad-pixels`, referencing HANDOFF-208,
SPEC-098, STAGE-034, PROJ-006, DEC-086, DEC-084, DEC-040 and DEC-053.

The commit-message BODY must be plain prose: a push with code tokens like `#[...]`, `->`
or `--flag` is rejected; reword it, do not retry. End it with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back:
- the PR number;
- whether AC1's literals, AC7's digests and the preview pins matched as registered;
- the SL3 counts (AC8);
- which test each mutant failed;
- the `opcode` fuzz run count;
- the true token cost.
