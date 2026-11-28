# SPEC-099 — build cycle prompt

You are the **implementer** for SPEC-099 (PROJ-006, STAGE-034), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (not `rtk git`, which has served
stale branch state in this repo).

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-210-build-the-per-plane-warprectilinear-on-the-developed-colour-output.md`
(your context and the handback you must fill),
`projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-099-the-per-plane-warprectilinear-on-the-developed-colour-output.md`
(Design, Implementation Context, Acceptance Criteria, Failing Tests and the rule-clause map
are the contract),
`decisions/DEC-087-the-sensor-geometry-planes-resample-red-and-blue-to-green-geometry-and-leave-green-warp-to-the-consumer.md`
(item 3 is the exact arithmetic), DEC-086 items 5 and 8 with their amendment lines, then
`AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

- `src/opcode.rs`: a crate-private per-plane parse of the first `WarpRectilinear` (all `N`
  sets, `cx`, `cy`, `N` checked to be 1 or 3), and on it the public accessor
  `sensor_geometry_warp(&Sensor)` with `SensorGeometryWarp` (`consumer`, `sets`,
  `is_per_plane`) and `PlaneWarp` (Design §1). `parse_warp_rectilinear` and its params
  parser stay byte-for-byte as they are. Update the `DevelopPath::SensorGeometry` doc.
- `src/warp.rs`: the CA rule (DEC-087 item 3): the fold check through `scale_bounds`, the
  4096-interval table by bisection, the halo `H`, and the per-pixel source with
  `bilinear_sample` over the `ActiveArea`. Unit tests as the spec lists.
- `src/color.rs`: resolve the per-plane warp once per call, after the CFA check and before
  the calibration (Design §5), into a small value; wire the delay ring between the fused
  traversal and `store` in `srgb_linear_plane` (flushed after the traversal) and in
  `srgb_linear_plane_rows` (band plus `H` rows each side). When the CA step does not run,
  the code path must be exactly today's. Update the six entries' docs.
- `src/lib.rs`: `Error::UnsupportedWarpFold { slope_min: f64 }` (kind `Unsupported`,
  `Display`), and the `__fuzz_warp_planes` hook; `fuzz/fuzz_targets/warp_opcode.rs` and
  a seed with `N = 3` distinct sets.
- `scripts/sdk-stage3.sh` (new; writes only into its output directory), the
  `$IRRADIANCE_SDK_STAGE3_DIR` SKIP line in `corpus-status`, and a line in `AGENTS.md` §6
  beside `sdk-stage2`.
- Tests: `tests/plane_warp.rs` (new), `tests/leica_colour.rs` (AC7: the sensor-geometry
  pins move), `tests/colour_sdk_stage3.rs` (new, AC8), `tests/error_kind.rs`.
- `docs/provenance-ledger.md` (one row, class 1), `CHANGELOG.md` `[Unreleased]`.

Do NOT:
- touch `develop_color_*`, the developed mix (`develop_core`, `developed_mix_strips`,
  `scratch_len`), the mono develop, `apply_warp_into` or `parse_warp_rectilinear`;
- bump `DEVELOP_VERSION` or `MONO_MIX_VERSION`;
- change any other file's output, hash or tier-B pin;
- add a dependency or `unsafe`, or any public item beyond the accessor, its two types and
  the one `Error` variant.

If an AC7 `u16` digest or the AC8 bound fails, report it; do not edit it.
**Never copy `LEICA-Q3-PB/leica_q3_01.dng`, `LEICA-SL3-PB/leica_sl3_01.dng` (or any `-PB`
file), any SDK output or any derivative pixel into the repo, a fixture, a seed or CI.**

## Order of work

1. Write the Failing Tests first and see each fail (or, with the corpus, mismatch) against
   `main`. Record that in the handback.
2. Implement; make them pass.
3. Apply each mutant in the rule-clause map by hand, record which test fails, and revert.
   HANDOFF-202's handback describes a scripted string-replacement harness that worked.
4. Generate the SDK references with `scripts/sdk-stage3.sh` into a directory outside the
   repo and run AC8 with `IRRADIANCE_SDK_STAGE3_DIR` set; record the per-patch numbers.
5. Add the seed and run the `warp_opcode` smoke fuzz:
   `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run warp_opcode fuzz/corpus/warp_opcode fuzz/seeds/warp_opcode -- -max_total_time=60`
   (`mkdir -p fuzz/corpus/warp_opcode` first in a fresh worktree).
6. Run the gates:
   - `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
     (NEVER `just test --all-features`);
   - `just lint-ci`, `just lint-fuzz`, `just fmt-check`, `just doc-check`, `just wasm-check`;
   - `just validate`, `just semver-check` (minor, additive), `just cost-audit`.
   Confirm that AC7 ran on all three Leica files (not skipped).
7. Time `srgb_linear_plane_into` on the Q3 before and after (release build) and record it.

## Hand back

Fill HANDOFF-210's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section. Then open a PR against `main`
(SPEC-099 only) from `feat/spec-099-sensor-geometry-ca`, referencing HANDOFF-210,
SPEC-099, STAGE-034, PROJ-006, DEC-087, DEC-086, DEC-059 and DEC-024.

The commit-message BODY must be plain prose: a push with code tokens like `#[...]`, `->`
or `--flag` is rejected; reword it, do not retry. End it with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back:
- the PR number;
- whether AC7's three `u16` digests matched as registered, and the re-pinned `f32` and
  mono digests;
- AC8's worst `|CA − baseline|` and smallest `|green-only − baseline|` per file and plane;
- which test each mutant failed;
- the `warp_opcode` fuzz run count;
- the Q3 timing before and after;
- the true token cost.
