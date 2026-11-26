# SPEC-093 — build cycle prompt

You are the **implementer** for SPEC-093 (PROJ-006, STAGE-031), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (not `rtk git`, which has served
stale branch state in this repo).

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-204-build-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md`
(your context and the handback you must fill),
`projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md`
(Design, Acceptance Criteria, Failing Tests and the mutant list are the contract),
`decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md`,
`decisions/DEC-083-the-mono-mixer-row-is-i32-with-24-fraction-bits-and-its-fade-starts-at-57343.md`,
then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

- `src/develop.rs`: `DevelopOptions` gains `pub mono_mix: Option<[Rational; 3]>`, default
  `None`. With `Some(weights)`, `develop_with_into`, `develop_with_scratch_into`,
  `output_dimensions_with` and `scratch_len` develop a colour Bayer DNG to a mono image
  that equals, bit for bit, the mono develop of `mono_mix_plane_into`'s plane read at
  black `MONO_MIX_BLACK` and white `MONO_MIX_WHITE` (Design §2). `black_levels` is not
  read under the mix.
- Refusals in Design §3's ranks, inside `develop_core`'s one body; nothing written to
  `dst` or scratch before the passes. The sizing entries check ranks 0, 1, 4 (and
  `scratch_len` the `OpcodeList3` parse); neither parses `OpcodeList1` under the mix.
- `src/opcode.rs`: `DevelopPath::MonoMix` (every `GainMap`, the first `WarpRectilinear`,
  nothing from list 1; the colour path's malformed sets).
- `src/color.rs` (crate-private): the mix's check steps, the strip pass over 512-row
  strips through the band traversal, an `ActiveWindow` slot, and `CropScatter` from an
  options-applied `Geometry`.
- `src/demosaic.rs`: `develop_color_fused` takes its local height and its ring cache from
  the caller; a band form takes one caller slice for green rows and ring, with no
  `Sensor` clone. Existing callers keep their own allocations and their output.
- Scratch under the mix: `T = w·(min(h, 519) + 8)` without a non-identity warp,
  `A + max(A, T)` with one (Design §7). `develop_with_scratch_into` allocates nothing.

Do NOT add an `Error` variant, a public function besides the field and the `DevelopPath`
variant, a dependency, `unsafe`, a threaded variant, or a version bump; do NOT change any
existing output, hash or Tier-B pin; do NOT touch the mono passes
(`crop_orient_normalize_for`, `normalize_active_area_for`). If an AC4 digest or
`scratch_len` number fails, report it; do not edit it.

## Order of work

1. Write the Failing Tests first (`tests/developed_mix.rs`, the unit tests in
   `src/develop.rs` and `src/opcode.rs`, the `tests/develop_scratch_alloc.rs` cases, the
   determinism test, the `Entry` rows) and see each fail against `main`. Record that in
   the handback (SPEC-092's build wrote code first; do not repeat that).
2. Implement; make them pass with the existing suite green.
3. AC14: apply each pre-registered mutant (a) to (x) by hand, record which test fails,
   revert. Argue (u), (y) and (z) as equivalent. HANDOFF-202's handback describes a
   scripted string-replacement harness that worked.
4. Extend `fuzz/fuzz_targets/demosaic.rs` and `fuzz/fuzz_targets/develop.rs` (Design §10);
   run each smoke fuzz (`PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run
   <target>`, the toolchain brief's length).
5. `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
   (NEVER `just test --all-features`), `just lint-ci`, `just lint-fuzz`, `just fmt-check`,
   `just doc-check`, `just validate`, `just semver-check` (additions only),
   `just cost-audit`. Push and confirm the CI `determinism` job is green on both legs
   (AC11).

## Hand back

Fill HANDOFF-204's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section, then open a PR against `main`
(SPEC-093 only) from `feat/spec-093-developed-mix`, referencing HANDOFF-204, SPEC-093,
STAGE-031, PROJ-006, DEC-084 and DEC-083. Commit-message BODY plain prose (a push with
code tokens like `#[...]`, `->`, `--flag` is rejected; reword, do not retry), ending with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back: the PR number, whether AC4's digests and `scratch_len` numbers matched as
registered, AC8's pair count per entry and every skipped pair with its reason, AC10's
allocation counts, the four AC11 hashes and both CI determinism legs' result, which test
each mutant failed, and the true token cost.
