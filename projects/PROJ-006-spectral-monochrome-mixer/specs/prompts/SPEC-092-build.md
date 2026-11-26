# SPEC-092 — build cycle prompt

You are the **implementer** for SPEC-092 (PROJ-006, STAGE-031), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh`.

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-202-build-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md` (your context and the handback you must fill), `projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md` (Design,
Acceptance Criteria, Failing Tests and the mutant list are the contract),
`decisions/DEC-083-the-mono-mixer-row-is-i32-with-24-fraction-bits-and-its-fade-starts-at-57343.md`,
`decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md`,
then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

Add to `src/color.rs`:
`mono_mix_row(sensor, weights: &[Rational; 3]) -> Result<[i32; 3], Error>`,
`mono_mix_plane_into(sensor, src, weights, dst: &mut [u16])` and its row-range form
`mono_mix_plane_rows_into(sensor, src, weights, rows: core::ops::Range<u32>, dst)`, plus
the constants `MONO_MIX_ROW_FRACTION_BITS = 24`, `MONO_MIX_FADE_START = 57343`,
`MONO_MIX_FADE_WIDTH_BITS = 13`, `MONO_MIX_BLACK = 0`, `MONO_MIX_WHITE = 65535`,
`MONO_MIX_VERSION = "1"` (Design §12: documented like `DEVELOP_VERSION`, asserted beside
the golden rows and the hashes).

- Weights: non-negative rationals with positive denominators summing to exactly one
  (checked in `i128`), converted by one `f64` division each.
- Row: `v_i = (w_0·M[0][i] + w_1·M[1][i]) + w_2·M[2][i]` in `f64`, no `mul_add`, then
  `q_i = ⌊v_i·2^24 + 0.5⌋` as `i32` (refuse out of range as `MixRowOutOfRange`).
- Per pixel, in `i64` on the fused traversal's camera triple `x`:
  `P = Σ_{q≥0} q·x`, `N = Σ_{q<0} q·x`, `u = min(65535 − max(x), 8192)`,
  `F = clamp((P·8192 + N·u + 2^36) >> 37, 0, 65535)`.
- Every pixel of the `width × height` plane is written through SPEC-091's identity sink,
  generalised to one sample per pixel; the band form reuses `develop_rows_banded` and is
  bit-identical to the whole plane.
- Refusals: `InvalidMixWeights` first, then SPEC-091's plane order with
  `MonoBufferWrongLength` in `RgbBufferWrongLength`'s place, then `MixRowOutOfRange` last;
  `dst` untouched on every refusal.
- Three `Error` variants appended at the enum's end, in this order:
  `InvalidMixWeights { channel: Option<u8> }` (`CallerError`),
  `MonoBufferWrongLength { expected: u64, actual: usize }` (`CallerError`),
  `MixRowOutOfRange { channel: u8 }` (`Unsupported`).

Do NOT add a clip mask, a fade parameter, a plan type, threaded variants, an `irr` hook,
any other `Error` variant, a dependency, `unsafe`, or a `DEVELOP_VERSION` bump; do NOT
change any existing function's output or existing determinism hash; do NOT copy the
traversal. If a golden row in AC5 or the AC4 bound fails, report it; do not edit it.

## Order of work

1. Write the Failing Tests first (`tests/mono_mix.rs`, the `src/color.rs` unit tests, the
   determinism test, the new `Entry` rows, the `error_kind` rows) and see each fail.
2. Implement; make them pass with the existing suite green.
3. AC15: apply each pre-registered mutant (a) to (s) by hand, record which test fails,
   revert. Argue (t) and (u) as equivalent in the handback.
4. Extend `fuzz/fuzz_targets/demosaic.rs`; run the smoke fuzz
   (`~/.cargo/bin/cargo +nightly fuzz run demosaic`, the toolchain brief's length).
5. `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
   (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`, `just doc-check`,
   `just validate`, `just semver-check` (additions only), `just cost-audit`. Push and
   confirm the CI `determinism` job is green on both legs (AC12).

## Hand back

Fill HANDOFF-202's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section, then open a PR against `main`
(SPEC-092 only) from `feat/spec-092-mono-mixer`, referencing HANDOFF-202, SPEC-092,
STAGE-031, PROJ-006, DEC-083 and DEC-079. Commit-message BODY plain prose (a push with
code tokens like `#[...]`, `->`, `--flag` is rejected; reword, do not retry), ending with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back: the PR number, whether AC5's fifteen golden rows matched as registered, the
AC4 kept-pixel fractions on the Canon and Nikon and the largest deviation seen, the AC6
violation count (expected zero) and the control's count, which one-hot weight set the
determinism fixture uses, both CI determinism legs' result, which test each mutant
failed, and the true token cost.
