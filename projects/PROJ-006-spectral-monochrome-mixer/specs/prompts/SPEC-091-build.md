# SPEC-091 — build cycle prompt

You are the **implementer** for SPEC-091 (PROJ-006, STAGE-031), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh`.

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-189-build-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (your context and the handback you must fill), `projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (Design,
Acceptance Criteria, Failing Tests and the mutant list are the contract),
`decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md`,
then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

Add four public functions to `src/color.rs`:
`srgb_linear_plane_into(sensor, src, dst: &mut [u16])`,
`srgb_linear_plane_f32_into(sensor, src, dst: &mut [f32])`, and their row-range
forms `srgb_linear_plane_rows_into` / `srgb_linear_plane_rows_f32_into`
(`rows: core::ops::Range<u32>` in whole-plane rows, built on `develop_rows_banded`,
every value bit-identical to the whole-plane form). The whole-plane forms run
`develop_color_into`'s pipeline (levels, `OpcodeList2` `GainMap`s, Hamilton-Adams,
the camera-to-linear-sRGB matrix) but write **every** pixel of the `width × height`
plane through an identity sink, not the crop sink. The `u16` form is today's
`quantize` expression bit for bit; the `f32` form is the same `f64` sum divided by
65535 and cast once, **unclamped**. Refusals are exactly `develop_color_into`'s, in its
order, with `dst` untouched; the band forms add `Error::InvalidRowRange` (the one new
variant, appended at the enum's end, `CallerError`). Add a Tier-A colour fixture to
`tests/determinism.rs` with committed `u16` and `f32` plane hashes (onyx needs the f32
plane bit-identical on aarch64 and x86_64), and golden digests of both planes for the
five held files in `tests/linear_plane.rs`.

Do NOT add any other `Error` variant, a dependency, `unsafe`, threaded variants, or a
`DEVELOP_VERSION` bump, and do NOT change any existing function's output or existing
determinism hash. Factor the sink; do not copy the traversal.

## Order of work

1. Write the Failing Tests first (`tests/linear_plane.rs`, the `src/color.rs` unit
   test, the new rows in the four `Entry` tables) and see each fail.
2. Implement; make them pass with the existing suite green.
3. AC10: apply each pre-registered mutant (a) to (k) by hand, record which test fails,
   revert.
4. Extend `fuzz/fuzz_targets/demosaic.rs`, run the smoke fuzz.
5. `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
   (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`,
   `just validate`, `just semver-check` (five additions only), `just cost-audit`. Push
and confirm the CI `determinism` job is green on both legs (AC12).

## Hand back

Fill HANDOFF-189's `handback:` (a REAL `tokens_total`, measured from your transcript
per `docs/cost-tracking.md`) and its `## Handback` section, then open a PR against
`main` (SPEC-091 only) from `feat/spec-091-srgb-linear-plane`, referencing HANDOFF-189,
SPEC-091, STAGE-031, PROJ-006 and DEC-079. Commit-message BODY plain prose (a push with
code tokens like `#[...]`, `->`, `--flag` is rejected), ending with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back: the PR number, AC1's byte equality on all five held colour files, the
AC3 negative and over counts, the AC11 strip partitions tested, both CI determinism
legs' result, which test each mutant failed, and the true token cost.
