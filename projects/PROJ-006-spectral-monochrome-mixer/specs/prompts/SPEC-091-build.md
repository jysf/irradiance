# SPEC-091 — build cycle prompt

You are the **implementer** for SPEC-091 (PROJ-006, STAGE-031), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh`.

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-189-build-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (your context and the handback you must fill), `projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (Design,
Acceptance Criteria, Failing Tests and the mutant list are the contract),
`decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md`,
then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

Add two public functions to `src/color.rs`:
`srgb_linear_plane_into(sensor, src, dst: &mut [u16])` and
`srgb_linear_plane_f32_into(sensor, src, dst: &mut [f32])`. They run
`develop_color_into`'s pipeline (levels, `OpcodeList2` `GainMap`s, Hamilton-Adams,
the camera-to-linear-sRGB matrix) but write **every** pixel of the `width × height`
plane through an identity sink, not the crop sink. The `u16` form is today's
`quantize` expression bit for bit; the `f32` form is the same `f64` sum divided by
65535 and cast once, **unclamped**. Refusals are exactly `develop_color_into`'s, in its
order, with `dst` untouched.

Do NOT add an `Error` variant, a dependency, `unsafe`, threaded variants, or a
`DEVELOP_VERSION` bump, and do NOT change any existing function's output. Factor the
sink; do not copy the traversal.

## Order of work

1. Write the Failing Tests first (`tests/linear_plane.rs`, the `src/color.rs` unit
   test, the new rows in the four `Entry` tables) and see each fail.
2. Implement; make them pass with the existing suite green.
3. AC10: apply each pre-registered mutant (a) to (g) by hand, record which test fails,
   revert.
4. Extend `fuzz/fuzz_targets/demosaic.rs`, run the smoke fuzz.
5. `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
   (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`,
   `just validate`, `just semver-check` (two additions only), `just cost-audit`.

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
AC3 negative and over counts, which test each mutant failed, and the true token cost.
