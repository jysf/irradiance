# SPEC-100 — build cycle prompt

You are the **implementer** for SPEC-100 (PROJ-006, STAGE-034), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (not `rtk git`, which has served
stale branch state in this repo).

Read, in order: `projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-212-build-the-per-plane-warp-on-the-developed-colour-output-and-the-de.md`
(your context, the watch-outs and the handback you must fill),
`projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-100-the-per-plane-warp-on-the-developed-colour-output-and-the-de.md`
(Design, Implementation Context, Acceptance Criteria, Failing Tests and the rule-clause map
are the contract),
`decisions/DEC-088-the-developed-outputs-apply-the-ca-step-then-green-warp-before-the-matrix-for-colour-over-the-mixed-plane-for-the-mix.md`,
DEC-087 items 2 to 6, DEC-084 items 1, 3, 5, 6 and 7 with their amendment lines, DEC-086
item 6, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change

- `src/opcode.rs`: `DevelopPath::Colour` and `DevelopPath::MonoMix` apply the first
  `WarpRectilinear` in list 3 (`Applied::First`; drop `FirstSinglePlane` if it is then
  unused). Update both variants' docs.
- `src/warp.rs`: `ChromaticShift`'s tables and nodes in fixed-size arrays (no heap, same
  arithmetic); a crate-private gather of three interleaved channels at one
  `source_coord`, each through `bilinear_sample_with` (Design §2).
- `src/color.rs`:
  - the colour entries, with a **mandatory** first warp only: the per-plane checks after
    the CFA and before the calibration (Design §5), then the warped body (Design §2: green
    plane, `ActiveArea` camera buffer through `CaDelay` and `ActiveWindow`, the gather into
    the crop and orientation, the entry's own store). With an optional warp or none, the
    existing code path, untouched.
  - the threaded forms call the single-thread entry when the warp applies.
  - `CaDelay` borrows its ring; `developed_mix_strips` takes the resolved CA step and a
    ring slice and runs each strip with its `H`-row halo (Design §3). Without the CA step,
    the existing strips, untouched.
- `src/develop.rs`: `develop_warp` gives `W` lerped and non-identity for the mix (no
  `N == 1` filter); `develop_core` resolves the CA step for the mix and checks it in rank;
  `scratch_len` and `mix_needed_scratch` add `T_H` and `R`; `run_mix_passes` lays out the
  ring (Design §4). Docs on `DevelopOptions::mono_mix`, `develop_with_into`,
  `scratch_len`.
- `src/demosaic.rs`: the strip-work length for `S + 2H` rows.
- Tests: `tests/colour_warp.rs` (new), `tests/developed_mix.rs`,
  `tests/develop_scratch_alloc.rs`, `tests/leica_colour.rs`, `tests/colour_sdk_stage3.rs`,
  `tests/threaded.rs`, `tests/required_processing.rs` (and `src/opcode.rs` units), as the
  spec's Failing Tests list.
- `docs/provenance-ledger.md` (one row, class 1), `CHANGELOG.md` `[Unreleased]`.

Do NOT:
- add any public item, or options to the colour develop;
- apply an optional warp on the colour develop;
- change the CA step's arithmetic, `apply_warp_into`, `parse_warp_rectilinear`, the mono
  develop of a colour file, or any sensor-geometry output;
- bump `DEVELOP_VERSION` or `MONO_MIX_VERSION`;
- change any other file's output, hash or tier-B pin;
- add a dependency or `unsafe`.

If a pre-registered digest or the AC9 bound fails, report it; do not edit it.
**Never copy `LEICA-Q3-PB/leica_q3_01.dng`, `LEICA-SL3-PB/leica_sl3_01.dng` (or any `-PB`
file), any SDK output or any derived pixel into the repo, a fixture, a seed or CI.**

## Order of work

1. Write the Failing Tests first and see each fail (or, with the corpus, refuse) against
   `main`. Record that in the handback.
2. Move `ChromaticShift`'s tables off the heap and make `CaDelay` borrow its ring first,
   and run SPEC-099's tests: its pins must hold before anything else changes.
3. Implement the colour develop, then the mix; make the tests pass.
4. Apply each mutant in the rule-clause map by hand (the order mutants first), record
   which test fails, and revert. HANDOFF-202's handback describes a scripted
   string-replacement harness that worked.
5. Run AC9 with `IRRADIANCE_SDK_STAGE3_DIR` set to the folder HANDOFF-212 names (or one you
   regenerate with `scripts/sdk-stage3.sh`, outside the repo); record the per-patch numbers.
6. Run the gates:
   - `just test` with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
     (NEVER `just test --all-features`);
   - `just lint-ci`, `just fmt-check`, `just doc-check`, `just wasm-check`;
   - `just validate`, `just semver-check` (no public change), `just cost-audit`.
   Confirm that AC7 ran on all three Leica files (not skipped).
7. Time `develop_color_into` and the default developed mix on the Q3 (release build) and
   record them.

## Hand back

Fill HANDOFF-212's `handback:` (a REAL `tokens_total`, measured from your transcript per
`docs/cost-tracking.md`) and its `## Handback` section. Then open a PR against `main`
(SPEC-100 only) from `feat/spec-100-developed-warp`, referencing HANDOFF-212, SPEC-100,
STAGE-034, PROJ-006, DEC-088, DEC-087, DEC-084, DEC-086 and DEC-059.

The commit-message BODY must be plain prose: a push with code tokens like `#[...]`, `->`
or `--flag` is rejected; reword it, do not retry. End it with:
Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back:
- the PR number;
- whether the six colour and fifteen mix digests matched as registered;
- AC9's worst `|ours − baseline|` and smallest no-CA value per file and plane, and the
  mix's worst;
- which test each mutant failed;
- the Q3 timings;
- the true token cost.
