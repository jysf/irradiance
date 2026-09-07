---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.

handoff:
  id: HANDOFF-046
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session
                                    # (correction from tier_map.design's
                                    # claude-opus-5 prediction, per DEC-004
                                    # rule 3 — silent cost-surprise trap).
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.build, not a
                                    # measurement. Standing record: 0 FOR 13
                                    # on the build hint (HANDOFF-043 came back
                                    # as sonnet-5, not opus-5). CORRECT THIS
                                    # in the handback to what your own system
                                    # prompt reports as `message.model`.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-018

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` MUST be a real number from your interface (/cost in Claude
# Code; usage in the API). `notes:` MUST be ONE PHYSICAL LINE — handback-sync
# truncates multi-line YAML scalars and leaves the spec unparseable while
# every gate reports green (`handback-sync-truncates-multi-line-scalars`).
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: null                         # build does not open the PR; orchestrator does
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one PHYSICAL line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-046: Build SPEC-018 — WarpRectilinear radial geometric correction

## Delegation Summary

Build `SPEC-018`. **This is the single item in PROJ-001 that most directly
decides whether the project's thesis holds.** `SPIKE-001` measured ~504 px
of inward displacement at the corner (6% of image width); skipping the
warp produces a visibly wrong image, and a missing warp scores −82.338
against `dnglab analyze --srgb` through SPEC-020's oracle (measured, not
inferred — the previous ¼-res calibration was −68.05, SPEC-020 lifted it
to −82.338 at full metric). Land it in a state where all three decodable
Q2M frames score **≥ 85** through SPEC-020's oracle.

Branch is created for you: `feat/spec-018-warprectilinear-radial-geometric-correction`
from `main` at `7fe53fb` (PRs #12 and #15 merged, SPEC-020 shipped, SPEC-017
designed). Rebase to the tip of `main` before you start if it moved.

```
export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
```

The default corpus root does not exist. AC8 (SSIMULACRA2 ≥ 85 per frame)
and AC9 (tier-B kr1-zeroed red-proof) both need it; AC10 (tier-A
kr1-zeroed red-proof) runs without it.

## ⚠ The one idea this whole spec turns on

**Bit-identity with dnglab is not the goal, geometric correctness is.**
`docs/oracle-contract.md` § "This oracle is single-sourced" is why: `--srgb`
output comes from rawler, so bit-agreement proves we match rawler, not
that we are right. SPEC-020's ≥ 85 SSIMULACRA2 bar is a *perceptual*
check — geometry and gross tone — and it is enough. If your resampling
kernel produces output that scores ≥ 85 on all three decodable frames,
the kernel is correct enough regardless of whether it matches dnglab
pixel-for-pixel.

**Pre-registered rule for kernel selection (from the spec's `## The design
decision this spec rests on`):** try bilinear first; measure the score on
all three Q2M frames; if all ≥ 85, ship bilinear. If not, upgrade one step
(bicubic, then Lanczos-3) and record the alternatives in a DEC-*. Do NOT
tune parameters to hit 85; a kernel that needs tuning is a wrong kernel.

## ⚠ Read this before the spec: the FU-1 fix landed at SPEC-020's ship

**The WarpRectilinear coefficients are per-frame, not a camera constant.**
SPEC-020 verify (FU-1, 2026-09-06) parsed OpcodeList3 out of all three
decodable Q2M frames and measured:

```
                kr0             kr1              kr2              kr3
L1021223  0.9992511060  −0.0613765129  −0.0939155414  0.0558820092
L1026016  0.9992511060  −0.0418451693  −0.1023114788  0.0578767192
L1026192  0.9992511060  −0.0323314533  −0.1042041102  0.0563642935
```

Only `kr0` is constant. `kr1` varies **~1.9×** across frames. `SPIKE-001`'s
coefficient set is `L1021223.DNG`'s specifically. SPEC-018's `## Context`
carries the corrected table on `main` at `7fe53fb`. The three consequences:

1. **AC1's round-trip fixture** — commit ONE hex fixture per decodable
   frame (`tests/oracle-fixtures/opcodelist3-L1021223.hex`, ...L1026016.hex,
   ...L1026192.hex), and assert each parses to its own coefficient set.
2. **AC4's corner-displacement expected values are per-frame.** L1021223's
   `f(1) = 0.899841` gives ~504 px; the other two frames' `kr1..kr3` sums
   yield different corner scales — compute each frame's f(r=1) directly
   from its coefficients, do not hardcode 0.899841 as a Q2M constant.
3. **AC8's ≥ 85 must hold on ALL THREE frames** — one frame passing
   is one measurement, not a boundary. `measurement-over-generalised`
   (AGENTS.md §16 rule 1) is why. Report per-frame scores in the handback.

`tests/support/perturb.rs:78–82` (SPEC-020's synthetic-fault generator)
carries L1021223.DNG's coefficients exactly to f32 — that stays; it is
the synthetic-fixture reference, not a real-frame reference. Do NOT
change perturb.rs.

## Coordination with SPEC-017 on `src/opcode.rs`

SPEC-017 (`FixBadPixelsConstant`) is designed and its build handoff
HANDOFF-044 is staged; whichever spec builds SECOND extends the module
the first created. SPEC-017's design pre-registers the `Opcode` enum's
initial shape:

```rust
pub enum Opcode {
    FixBadPixelsConstant { constant: u32, bayer_phase: u32 },
    Unknown { id: u32, flags: u32, params: Vec<u8> },
}
```

If SPEC-017 built first, the module exists on `main`. Add a
`WarpRectilinear { kr: [f64; 4], kt: [f64; 2], cx: f64, cy: f64,
planes: u32 }` variant and the parser branch for `OpcodeID == 1` —
do NOT rewrite the module.

If SPEC-018 (this handoff) builds first, `Opcode::Unknown` carries
raw parameter bytes so SPEC-017 can parse them into
`FixBadPixelsConstant`. SPEC-018's `## Where the opcode module lives`
records the same coordination contract from SPEC-017's side.

State which happened in the handback.

## The DNG-spec probes required before you write the parser

`SPEC-018`'s `## Implementation Context → Required before build handoff`
carries these — reproduce here so nothing is missed:

1. **Read DNG 1.7.0.0 § 6.4.1 "WarpRectilinear" in full.** The
   spec's *parameter order*, *radius normalisation convention*
   (SPIKE-001's caveat: "assumes the DNG convention that r normalizes
   to 1.0 at the corner. The pixel figures should be confirmed
   against the spec before being quoted as exact"), and the
   *out-of-extent pixel rule* all come from the spec, not from
   this handoff. Cite the exact clause for each in the resampling-
   kernel DEC.

2. **Resolve `r`'s normalisation.** If DNG says half-width, the
   corner displacement figure is different from ~504 px and every
   AC4 expected value derives from a different `r_max`. This is
   design-time work the build must complete before writing the
   applier.

3. **Resolve the out-of-extent pixel rule.** Clamp / zero /
   undefined / spec-silent. Cite the clause; if spec-silent,
   choose and record in the kernel DEC.

4. **Confirm the WarpRectilinear opcode ID and version at the byte
   level from a fresh IFD parse.** SPIKE-001 said `OpcodeID == 1,
   version 1.4.0.0`; re-parse to be sure, and record the exact
   bytes in each of the three tier-A hex fixtures.

## The four codified lessons apply here

1. **`measurement-over-generalised`** — every per-frame score,
   per-frame coefficient, per-frame corner displacement is a
   claim. State each with its exact command and its scope. "The
   Q2M's kr1 is −0.061" is a wrong sentence; "L1021223.DNG's kr1
   is −0.0613765129" is right.

2. **`attribute-text-inside-doc-comments`** — the parser's
   panic-free lint scanner already trips this. Any assertion that
   greps source text asserts a match count and excludes doc comments.

3. **`a-gate-that-fails-mutely-is-a-gate-that-never-ran`** — every
   new gate (`just fuzz-warp`, kernel-choice DEC) dies through its
   own error message, not through `set -o pipefail`.

4. **`unrun-docs-carry-errors` — N=6 as of today.** Instance 6 was
   SPEC-018's own Context table, mis-cited as a camera constant.
   Do NOT re-introduce a from-memory citation of another camera's
   coefficients in this build. Read each frame's own bytes.

## The `a-fix-inherits-the-precondition-of-the-thing-it-fixes` lesson

When the DEC-005 tolerance was originally set, it was calibrated on
one file at ¼ res (−68 for missing warp). SPEC-020 verified at full
metric on synthetic input (−82.338 for missing warp). SPEC-018's AC8
lands on REAL Q2M frames at full resolution — a THIRD scale change.
Assume the ≥ 85 threshold holds until you measure. If AC8 lands
materially different from the previous two calibrations, that is
a **finding** worth reporting, not a threshold to relax. DEC-005
already marks this as a revisit condition.

## The two things most likely to go wrong

**1. `library-not-application` violation.** The temptation is to pull
`resize` / `imageproc` / `image` for the resampler — do NOT.
Bilinear is ~20 lines, bicubic ~40, Lanczos-3 ~60, all
hand-implementable within the panic-free discipline. `Cargo.toml`
gains no `[dependencies]` entry; if it does, the whole spec is
wrong.

**2. The opcode stream is BIG-ENDIAN.** Same trap SPEC-017's
HANDOFF-044 flags. Writing the parser with the wrong endianness
produces plausible-looking values that are wrong in a subtle way
(e.g. `0.0` reads the same both ways, and floats near 1.0 differ by
only a few bits between endiannesses). Write AC1's round-trip test
FIRST, on `L1021223.DNG`'s exact bytes, before any other parser
work.

## The `+toolchain` trap applies to `fuzz-warp` too

`cargo fuzz` shells out to a bare `cargo build` which resolves to
Homebrew's stable. Match `just fuzz-plane` line-for-line in
`app.just`:

```
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run warp_opcode \
    fuzz/corpus/warp_opcode fuzz/seeds/warp_opcode -- -max_total_time=60
```

Add the same shape to `app.just`, add the AGENTS.md §6 code-block
line, and add the CI smoke step in the same PR (§12 bar 2 — fuzz
targets arrive with the parser, not retrofitted).

## Return Criteria

1. **All 14 acceptance criteria met and their named tests pass**
   (`## Failing Tests` in the spec lists 12 named tests — some ACs
   share tests). Run each with `cargo test --all-features <name> --
   --exact --nocapture` and confirm it *ran* (the
   `named-tests-can-pass-vacuously` trap fires when a partial name
   matches zero tests). Report count of tests before → count after.

2. **Ten gates + `just lint-ci` + `just fuzz-warp`**, run by you,
   pasted, summed across all targets, clippy version asserted
   (local 0.1.97; CI floats at 0.1.98). Say which gate list you
   ran — the count is ambiguous (`the-gate-count-is-not-defined-
   anywhere` open signal).

3. **Push and read CI.** Observed green on your SHA, run id and
   job count. The `fuzz-warp` smoke run must land in the same PR
   (§12 bar 2).

4. **Both red-proofs watched and pasted** (AC9 tier-B and AC10
   tier-A). Each: file changed AND compiled AND *output changed*.
   ⚠ That third clause caught a false red-proof in `PATCH-002`;
   check that your injection exercises the path you think it does.

5. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s
   build lost its entire change to `git checkout --`, and the
   orchestrator did the same to `PATCH-002`. Mutate in an isolated
   copy, or commit first; md5-verify every revert.

6. **AC8's three PER-FRAME scores recorded in the handback.**
   SSIMULACRA2 score for each of `L1021223.DNG`, `L1026016.DNG`,
   `L1026192.DNG` after full develop with the chosen kernel.
   These are DATA — the first real full-resolution scores this
   repo will have.

7. **AC11's kernel-choice DEC.** Alternatives considered (bilinear,
   bicubic, Lanczos-3), chosen kernel, per-frame scores per
   kernel measured, spec clauses for radius normalisation and
   out-of-extent rule cited. `just deny check licenses` and
   `just deny --manifest-path fuzz/Cargo.toml check licenses`
   both quoted.

8. **Provenance rows for both new algorithms** (AC13). Two rows:
   opcode parser (class 1, DNG spec) and warp+kernel (class 1
   for transform; class 1 if kernel from published paper, class 2
   if from a permissive crate reading — honest per §16 rule 3).

9. **`## Follow-ups` table intentionally deferred** — dispositions
   happen at ship. Raise findings as `FU-N` in the handback
   narrative; the reviewer or ship cycle disposition them.

10. **State which of SPEC-017/018 landed `src/opcode.rs` first.**
    In one line in the handback. If SPEC-017 landed first, you
    extended; if SPEC-018 (this handoff), you scaffolded for
    SPEC-017 to extend.

11. **PR NOT opened by build** — orchestrator opens it after the
    handback lands.

## Cost

Fill the `handback:` block above with real numbers from your
interface. `just handback-sync SPEC-018` (from an orchestrator's
session) transcribes them. `notes:` is ONE PHYSICAL LINE. Identify
your own transcript by scratchpad UUID, not by text-matching spec
strings (`identify-own-transcript-for-cost-handback` in this
project's memory). Price per-component at your model's published
rates; add +20% for the turns writing the handback (measured
under-count: 9.9%–15.4%).

## References

- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md`
  — the spec (`## Acceptance Criteria`, `## Failing Tests`, `##
  Implementation Context`, `## Notes for the Implementer`, `##
  The design decision this spec rests on`). ⚠ `## Context` was
  patched at SPEC-020's ship — the per-frame coefficient table
  is authoritative.
- `projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-020-develop-oracle-vs-dnglab-srgb.md`
  — SPEC-020's shipped spec with the four measured fault scores
  (identity 100.000, 1-px shift 61.823, missing warp −82.338,
  gamma 1.05 90.038) and the FU-1 disposition that produced the
  Context correction.
- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md`
  — the sibling spec that shares `src/opcode.rs`. Its `## Where
  the opcode module lives` is the coordination contract from the
  other side.
- `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-044-build-fixbadpixelsconstant-opcode.md`
  — SPEC-017's build handoff. Read to know what the sibling will
  add to the module.
- `spikes/done/SPIKE-001-*.md` — the original coefficient measurement
  and the `r` normalisation caveat.
- `tests/support/perturb.rs`, `tests/support/pnm.rs`,
  `tests/support/ssimulacra2.rs`, `tests/perceptual_oracle.rs`
  — SPEC-020's ship. The metric wiring is done; SPEC-018's AC8/AC9
  call it.
- `docs/oracle-contract.md` — the develop-layer contract; § "This
  oracle is single-sourced" for why bit-identity with dnglab is
  wrong.
- `docs/measured-q2m-dng.md` — the pipeline order (OpcodeList3 runs
  after normalise + geometry + orientation).
- `docs/provenance-ledger.md` — where AC13's two rows land.
- `AGENTS.md` §5 (measured toolchain, four `+toolchain` traps), §6
  (commands — every recipe's commands appear in the block), §11
  (unread-field discipline), §12 (four testing bars — oracles
  ship red, fuzz targets ARRIVE with the parser), §15 (cycle
  contract), §16 (four codified lessons).
- `guidance/constraints.yaml` — `no-panics-on-untrusted-input`,
  `oracle-must-be-shown-red`, `provenance-recorded-per-algorithm`,
  `library-not-application`, `test-before-implementation` all
  apply.
- `guidance/toolchain-brief.md` — filled 2026-08-16.
- **DNG 1.7.0.0 specification § 6.4.1 "WarpRectilinear"** — the
  authoritative source. Read it in build; cite the exact clauses
  in the kernel DEC.

## What this handoff does NOT ask for

- **A general-purpose image resampler.** The kernel exists to serve
  WarpRectilinear on one plane at Q2M resolution. Do NOT pre-generalise.
- **Tangential terms (`kt0`, `kt1`).** All three Q2M frames measure
  `kt0 = kt1 = 0.0`; the applier can implement the pure-radial path.
  The parser reads them (AC1); the applier treats a non-zero value
  as a stop-and-report finding, not a codepath.
- **`GainMap` (colour lens-shading).** Not on monochrome; PROJ-002.
- **Bit-identity with dnglab's warp.** `docs/oracle-contract.md` §
  "This oracle is single-sourced". Perceptual ≥ 85, not bit.
- **Recalibrating DEC-005's ≥ 85 threshold at full resolution.**
  DEC-005 marks this as a revisit condition, but it happens
  **when** SPEC-018 lands, not as one of AC8's tests. If AC8 lands
  materially different from SPEC-020's synthetic calibration, that
  is a `FU-N` for ship, not a threshold adjustment now.
