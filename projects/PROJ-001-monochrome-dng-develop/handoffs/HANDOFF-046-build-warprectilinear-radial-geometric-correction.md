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
  to_agent: claude-sonnet-5         # CORRECTED from tier_map.build's
                                    # claude-opus-5 prediction — standing
                                    # record now 0 FOR 14 on the build hint.
                                    # `message.model` this session actually
                                    # reports: claude-sonnet-5.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: completed                # pending | accepted | completed | rejected

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
# ⚠ ROUND 3 (punch-list). This block carried round 2's numbers until
# 2026-09-07; they are preserved verbatim in `## Completion — round 3`
# below (14,298,209 tokens, $34.26, 29 min, on claude-opus-5) AND already
# transcribed into the spec's `cost.sessions` (the SECOND `cycle: build`
# entry), so the overwrite loses nothing — exactly as round 2's own
# overwrite of round 1 did not. `synced_at` is reset to null so
# `just handback-sync SPEC-018` appends round 3 as a THIRD build session
# rather than skipping this file for idempotence.
#
# ⚠ `notes:` is ONE PHYSICAL LINE **and starts with a double quote**. That
# quote is load-bearing, not decoration: `scripts/_lib.sh`'s
# get_handback_field strips a trailing YAML comment from any value that
# does NOT begin with a quote, so an unquoted note is truncated at its
# first bare hash. That is exactly what happened to round 1's note, which
# reached the spec as `notes: 12/14 ACs green; AC8/AC9` — raised as FU-5.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 5579812            # round 3 only; deduped by message.id from my OWN transcript, identified by the scratchpad UUID 7e7e11d9-61a4-4474-9b50-f1b26ee9afc0 (NOT text-matched — a stale pre-/clear file can text-match and be a different model). All 51 metered messages report message.model claude-opus-5. Measured while writing this block, so the commit that lands it is not counted.
  estimated_usd: 15.02             # per-component (in 102, out 29270, cache-write 115825, cache-read 5434615) at published Opus-tier rates ($15/$75/$18.75/$1.50 per Mtok) = $12.52, + 20% handback uplift
  duration_minutes: 14             # first to last transcript timestamp (00:21:43Z to 00:35:51Z), wall clock, snapshot at write time
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: 16                           # round 3 did NOT open or modify it; PR 16 was already open on arrival — recorded as observed
  completed_at: 2026-09-07
  notes: "Round 3 (punch-list) on HANDOFF-048's one ship blocker, SB-3 CLOSED at b92b30c. One file, one rustdoc paragraph - src/warp.rs 53-55 replaced by 53-62, 10 insertions and 3 deletions, all inside doc comments, so decoded pixel output cannot move and the suite stays at 206 passed / 0 failed / 2 ignored. SB-3 was three defects in one sentence and all three are fixed. (A) The invented alternatives are gone: DEC-024 records neither zero-fill nor error, and records no alternative at all for the out-of-extent rule - its Alternatives Considered, lines 138-168, is Options A, B and C, every one about the kernel or the DEC-005 threshold. Re-measured rather than copied: grep -rn zero-fill over the tree returns 9 hits, 6 in process documents that discuss this very finding and 3 in source (tests/ifd_reader.rs 333, tests/support/perturb.rs 14, and the claim itself); grep -c on DEC-024 returns 0. (B) AC11 replaced by AC7: AC11 governs the KERNEL choice with measured per-frame scores (SPEC-018 428-433), while AC7 is the criterion that pre-registered the out-of-extent rule (SPEC-018 387-395) and is the AC DEC-024 itself names in the same clause that records the choice. (C) The 'records no per-frame oracle scores' clause is replaced by the measurement DEC-024 does hold - AC8 at -60.169 on L1021223.DNG (DEC-024 97-98) - reproduced by running the reader rather than quoted from the handback, since just test prints that same figure back in the ignore reason for warp_scores_at_least_eightyfive_via_spec_020_oracle. Phrased non-exclusively because DEC-024 also records AC9's -60.193 / -55.075 pair; a cleft would have been a fresh 16-rule-1 over-generalisation inside the sentence fixing an imprecision. WARNING, the one judgement call: the dispatch's suggested wording, clamps to edge per DNG 6.4.1, would have inherited SB-3's own precondition. DEC-024 195-196 records that 6.4.1 is SILENT on the out-of-extent rule, confirmed by full-text search, and src/warp.rs 36-43 - the reverify-approved heading five lines above the rewrite - says the same, so citing 6.4.1 as the SOURCE of a clamp rule would have been the same species of false citation one paragraph later. The rewrite takes the intent (the honest citation is the DNG spec and AC7, not a DEC that never weighed it) and states it as 6.4.1's SILENCE plus AC7, which is what both sources support. DEC-024 is byte-unchanged: git diff 08ad42e..b92b30c -- decisions/ is empty. Gates on b92b30c: lint-ci on the PINNED clippy 0.1.98 (not local 0.1.97), cargo fmt --check clean - note just fmt is NOT a recipe in this repo, fmt runs inside just lint - just test at 206 passed / 0 failed / 2 ignored run TWICE, once with IRRADIANCE_CORPUS_DIR pointed at the real corpus (4 frames present) and once with it unset (12 SKIP lines, default root absent), identical counts both ways; just deny licenses ok; just msrv 1.90.0; just fuzz-warp 23,438,651 runs in 61 s, zero crashes, cov 172 ft 457. CI GREEN on b92b30c - push run 34173566347 and pull_request run 34173569505, 10 jobs each, all success, fuzz smoke warp_opcode included, both watched to completion with gh run watch --exit-status rather than inferred. Nothing else touched: FU-1 through FU-11 all still open and owed a ship disposition, DEC-024, SPEC-018's body and ACs, src/lib.rs's SB-2 rewrite, src/develop.rs, every test including SB-1's, and PR 16, which was open on arrival and received a git push only. WARNING for the orchestrator: FU-11 will bite this entry too. HANDOFF-046's to_agent is still claude-sonnet-5 from round 1 and handback-sync line 97 reads it, but all 51 metered messages this session report claude-opus-5, so the round-3 cost.sessions entry will carry the WRONG agent exactly as round 2's did. Left unedited because the dispatch forbids the fix; FU-11 restated with one more instance."
  synced_at: 2026-09-07
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

## Completion

### Summary

`src/opcode.rs` and `src/warp.rs` created (SPEC-017 not yet built, so this
build scaffolded the opcode module first — see "Which of SPEC-017/018
landed first" below). Parser round-trips all three decodable frames'
real `OpcodeList3` bytes byte-for-byte (`tests/opcode.rs`). Bilinear
kernel shipped per the pre-registered rule. `src/develop.rs` gained the
warp stage, restructured mid-build after a design-time correction (see
`SB-1`/Finding 1 below). 12 of 14 acceptance criteria are met and tested
green; 2 (`AC8`, `AC9`) are measured but cannot pass with any correct
implementation, for reasons external to this build — see `SB-1`.

### Findings

- **`FU-1`** — `SPEC-018`'s own `## Context`/`## Implementation Context`
  named `OpcodeList3`'s IFD tag as `0xC740`. That is actually
  `OpcodeList1`'s tag (DNG 1.7.0.0 p.56-57 confirms `OpcodeList3 = 51022`
  / `0xC74E`). No code was ever wrong: `TAG_OPCODE_LIST_3` in
  `src/ifd.rs` (landed by an earlier spec) already carried the correct
  value. `unrun-docs-carry-errors` instance — the citation is corrected
  in `src/opcode.rs`'s module doc and this narrative.
- **`SB-1` (Finding 1, fixed in this build)** — The design-time
  assumption that `WarpRectilinear` runs *after* `DefaultCrop`/
  `Orientation` (carried from `SPIKE-001` through this spec's own
  `## Context`, `## Implementation Context`, and `docs/measured-q2m-
  dng.md`'s framing) is backwards per DNG 1.7's own `DefaultCropOrigin`/
  `DefaultCropSize` text: they describe the "**final** image area", cut
  from an already-fully-processed `ActiveArea`-sized image. Fixed:
  `src/develop.rs` now normalizes+warps over `ActiveArea`
  (8392x5632 for Q2M) and extracts `DefaultCrop` (8368x5584) afterward.
  For Q2M the two sizes differ under 1%, so this changes nothing
  numerically for THIS camera, but a future camera with a larger
  `ActiveArea`/`DefaultCrop` gap would render wrong under the original
  assumption. `DEC-024` records the fix and cites the exact clauses.
- **`SB-1` (Finding 2, NOT fixable by this build — needs a ship-cycle
  decision)** — `SPEC-020`'s oracle (SSIMULACRA2 vs `dnglab analyze
  --srgb`) cannot validate `WarpRectilinear` in either direction.
  Verified by inspecting `dnglab`/`rawler`'s own source
  (`github.com/dnglab/dnglab`): `OpcodeList1`/`2`/`3` appear ONLY as tag
  constants and `IFD::copy_tag` pass-through calls
  (`rawler/src/decoders/dng.rs`) — `WarpRectilinear` and
  `FixBadPixelsConstant` appear nowhere in the repository. `dnglab
  --srgb` never applies any DNG opcode. Measured on `L1021223.DNG`: the
  CORRECT warp scores **-60.169** (`AC8`), while a synthetic no-warp
  baseline scores **83.145** (with an approximate gamma stand-in for
  the linear-vs-sRGB comparison) — applying the right correction makes
  the match to `dnglab` WORSE, not better, because `dnglab`'s reference
  is itself uncorrected. `AC9`'s red-proof is correspondingly vacuous:
  honest score -60.193, `kr1=0` mutated score -55.075 — technically
  `< 85` but only because the honest score never reaches 85 either.
  **`AC8` and `AC9` cannot pass with ANY correct implementation of this
  spec** — this is not a kernel or code defect (per `## The design
  decision this spec rests on`: "if AC8 fails... change the kernel", but
  no kernel closes this gap, and trying more of them would be pure
  waste). Both tests are marked `#[ignore]` with the full reason inline
  (`tests/warp.rs`), still runnable via `cargo test -- --ignored`.
  `AC4`/`AC10` (this spec's oracle-free, analytic correctness proof —
  see below) are unaffected and green. **Disposition needed at
  ship/verify**: either formally narrow `DEC-005`/`SPEC-020`'s scope to
  exclude `WarpRectilinear`-bearing pixels (matching `SPEC-015`'s own
  precedent — an analytic check substituting for a comparison oracle
  that structurally cannot cover a pipeline stage), or find a second
  DNG-opcode-capable reference decoder. Full record in `DEC-024`.

### AC8/AC9 measured scores (informational — tests `#[ignore]`d, see `SB-1`)

| | score |
|---|---|
| `L1021223.DNG` (AC8, real warp) | -60.169 |
| `L1021223.DNG` no-warp baseline (diagnostic only, approximate gamma applied) | 83.145 |
| `L1021223.DNG` AC9 honest (real warp, no gamma) | -60.193 |
| `L1021223.DNG` AC9 `kr1=0` mutated | -55.075 |

`AC8` was not re-measured on `L1026016.DNG`/`L1026192.DNG` once the root
cause (the oracle sees no opcode processing at all, on any frame) was
confirmed on the first — re-running would not change the conclusion,
and the corpus/`dnglab` round-trip costs ~25s per frame.

### Which of SPEC-017/018 landed `src/opcode.rs` first

**SPEC-018 (this build).** `src/opcode.rs` did not exist on `main` at
the start of this cycle. `Opcode::Unknown { id: u32, flags: u32, params:
Vec<u8> }` is the escape hatch SPEC-017's own coordination text
pre-registers; `parse_opcode_list(bytes: &[u8]) -> Result<Vec<Opcode>,
Error>` matches its stated signature exactly. SPEC-017 extends this
module with its own `FixBadPixelsConstant` variant and match arm; it
does not need to rewrite anything here.

### Gates run (stating the list — `the-gate-count-is-not-defined-anywhere`)

`cargo fmt --check`, `cargo clippy --all-targets --all-features -D
warnings` (local 0.1.97), `just lint-ci` (pinned 0.1.98 — caught two
CI-only lints local clippy missed: `manual_saturating_arithmetic`,
`manual_is_multiple_of`, both fixed), `cargo check --all-targets
--all-features` (typecheck), `~/.cargo/bin/cargo +1.90.0 check
--all-targets --all-features` (MSRV), `cargo deny check licenses` +
`cargo deny --manifest-path fuzz/Cargo.toml check licenses` (both
green, no new dependency added to either graph — root `Cargo.toml` is
`0` lines changed), `./scripts/lint-red-proof.sh`,
`./scripts/cost-audit-red-proof.sh`, `just lint-no-allow`, `cargo test
--all-features` (full suite, corpus present, 205 passed / 2 `#[ignore]`d
/ 0 failed — see `SB-1`), and `PATH="$HOME/.cargo/bin:$PATH"
~/.cargo/bin/cargo +nightly fuzz run warp_opcode fuzz/corpus/warp_opcode
fuzz/seeds/warp_opcode -- -max_total_time=60` (20,077,163 executions,
zero crashes). Test count before → after: 172 passed → 205 passed + 2
`#[ignore]`d (33 new passing tests, 35 new test functions total).
`fuzz-warp` CI job added to `.github/workflows/ci.yml` in this PR
(no prior fuzz target — `ifd`/`plane`/`develop` — has a CI job yet;
`AGENTS.md` §5 flags that as a standing gap this build does not
retroactively close, only its own target).

### Reflection

1. **What would I do differently next time?** Run the design-time
   probe against the REAL corpus (a decode-and-score round-trip, not
   just byte-level parsing) before writing `## The design decision this
   spec rests on`'s kernel-selection loop — the broken-oracle finding
   would have surfaced at design instead of consuming a build cycle's
   worth of debugging that briefly (and wrongly) suspected the warp
   math itself.
2. **Does any template/constraint/decision need updating?** Yes —
   `docs/oracle-contract.md`'s "this oracle is single-sourced" warning
   should be extended with the concrete case measured here (a feature
   the reference doesn't implement at all, not just a rendering
   difference). Recorded as evidence for `guidance/signals.yaml` at
   ship, per that file's own ritual.
3. **Follow-up spec to write now?** Yes, in spirit: a spec (or a `DEC-*`
   revision) to formally narrow `SPEC-020`'s warp-oracle claim — `SB-1`
   above is the concrete ask.
4. **Where was the worst defect caught?** `build` — both the pipeline-
   order error and the broken-oracle finding were caught by this
   cycle's own design-time probes, before either reached verify or ship.
5. **What can a user do now that they couldn't before?** Before: a
   developed Q2M image was visibly wrong by ~504 px (6% of width) at
   the corners, matching no reference render. After: `develop_into`
   applies the real per-frame `WarpRectilinear` correction, verified
   geometrically correct against the DNG specification and
   `SPIKE-001`'s independent measurement (confirmed: 503.7 px /
   437.7 px / 407.0 px corner displacement, one measurement per frame,
   `tests/warp.rs::AC4`) — though `SPEC-020`'s oracle cannot itself
   confirm the visual improvement (`SB-1`).

---

## Completion — round 2 (punch-list)

Dispatched 2026-09-07 via `specs/prompts/SPEC-018-rebuild.md` after
`HANDOFF-047` returned `⚠ PUNCH LIST` at `40f5d45`. Scope: `SB-1` and
`SB-2` only. No new build handoff was minted (PATCH-003's round-2
pattern); this section continues `HANDOFF-046`, and the `handback:`
block above now carries round 2's numbers — **round 1's, replaced
there, were `tokens_total: 116480125`, `estimated_usd: 49.61`,
`duration_minutes: 82`, `completed_at: 2026-09-07`, on
`claude-sonnet-5`**, and are already transcribed into the spec's
`cost.sessions` (the `cycle: build` entry), so nothing is lost by the
overwrite.

Ship SHA: **`08ad42e`**. One commit, on top of `76793b9`.

### Summary

Documentation and test only. `src/develop.rs`, `src/lib.rs` and
`src/warp.rs` change **comments only** — no logic changed, and decoded
pixel output is byte-identical across the round (`irr develop
L1021223.DNG` reports `samples[0..8] [2028, 1855, 1818, 1868, 1805,
1889, 1943, 2013]`, `max 51764` both before and after). Test count
**205 → 206 passed / 0 failed / 2 ignored**, the +1 being `SB-1`'s new
tier-A test. The two `#[ignore]`-marked tier-B tests are untouched;
`FU-10` (whether `SPEC-020`'s oracle scope should be formally narrowed)
is a ship-cycle question and was not reopened.

### `SB-1` — the develop-side warp branch now has a live test

**Test name:**
`develop_into_crops_from_the_warped_active_area_not_the_warped_crop`
(tier A, `tests/warp.rs`, +278 lines including its fixture helpers).

It is the only test in the tree that develops a `Sensor` with a
non-`None` `opcode_list_3` all the way to pixels. The fixture is a
96x72 raw plane, `ActiveArea` 80x64 at `(6, 4)`, `DefaultCrop` 60x48 at
origin `(7, 5)` — a strict, off-centre sub-rectangle, so the two
candidate pipeline orders place the optical centre in different spots
and normalise `r` against different half-diagonals. The
`WarpRectilinear` is the committed **real** `L1021223.DNG` `OpcodeList3`
bytes (`tests/oracle-fixtures/opcodelist3-L1021223.hex`), and the plane
is an affine ramp (`256*x + 4*y`) chosen because bilinear interpolation
reproduces an affine function exactly — every developed value decodes
back to the source coordinate the pipeline actually sampled, so a wrong
order shows up as a wrong coordinate rather than as noise.
`BlackLevel = 0` / `WhiteLevel = 65535` makes normalization the
identity.

Three assertions:

1. **The fixture discriminates.** Both orders are computed in-test from
   the same normalized `ActiveArea` window, and must disagree — 2837 of
   2880 developed pixels differ, asserted, plus a per-pixel assertion at
   each probe. Without this the other two could hold vacuously.
2. **`develop_into` crops from the WARPED `ActiveArea`.** Whole-buffer
   equality against `crop(warp(active))`, whole-buffer inequality
   against `warp(crop(active))`, and two named probe pixels at opposite
   crop corners. This is the assertion that goes red on a revert.
3. **Identity warp is bit-identical to no warp.** `kr0 = 1.0`,
   `kr1 = kr2 = kr3 = kt0 = kt1 = 0.0` through the full develop pipeline
   equals the `opcode_list_3: None` path exactly — and must differ from
   the real-warp output, asserted, so assertion 3 cannot pass vacuously
   either. ⚠ **Stated honestly:** `develop_into` routes an identity warp
   through `warp.filter(|w| !is_identity(w))` into the warp-FREE branch,
   so what this assertion pins today is that short-circuit, not the
   resampler's own no-op behaviour (`AC5` pins the resampler directly in
   `src/warp.rs`'s unit tests). It would still catch a `Some(warp)`
   branch that silently diverged from the warp-free path.

**Red-proof — mutated, run, watched fail; reverted, run, watched pass.**
The mutation replaces `develop_into`'s `Some(warp)` arm with
`crop_orient_normalize_into` followed by `apply_warp_into` at
`(out_width, out_height)` — i.e. the pre-Finding-1 crop-then-warp order.

| `src/develop.rs` | md5 |
|---|---|
| honest | `318977b683d168d4572efdd5f672cc98` |
| mutated (crop-then-warp) | `cf714e8c973c5b4c60c647cca181ca77` |
| reverted | `318977b683d168d4572efdd5f672cc98` (byte-identical to honest) |

The mutation **compiles** (three `dead_code` warnings only, for the two
now-unused helpers and two `Geometry` fields) and **changes real decoded
output** — `irr develop L1021223.DNG` goes
`samples[0..8] [2028, 1855, 1818, 1868, 1805, 1889, 1943, 2013]`,
`max 51764` → `[1801, 2001, 2036, 1972, 1819, 1689, 2142, 2185]`,
`max 60918`.

**Suite delta under the mutation: 205 passed / 1 FAILED / 2 ignored.**
The single failure is the new test, and it names the finding:

```
crop top-left (0, 0): develop_into = 4130, warp-then-crop = 3948, crop-then-warp = 4130
assertion `left == right` failed: crop top-left: develop_into must sample the
WARPED ActiveArea, not the warped crop (DEC-024 Finding 1)
  left: 4130
 right: 3948
```

`develop_into` returned exactly the crop-then-warp value. Note the
count: round 1's 205 tests **all stayed green** under this mutation —
which is `SB-1` restated as a measurement, and the reason the new test
had to exist. After revert: 206 passed / 0 failed / 2 ignored.

### `SB-2` — the two false rustdoc claims, before and after

**1. `src/warp.rs`, `# Kernel` section (was lines 59-62).**

Before:

> Tried first per the spec's pre-registered rule; shipped because all
> three decodable Q2M frames scored >= 85 through SPEC-020's oracle with
> it (see the kernel-choice `DEC-*` for the measured per-frame numbers).
> Bicubic and Lanczos-3 are the recorded alternatives, not implemented.

After:

> Bilinear is `SPEC-018`'s pre-registered FIRST choice, and it shipped
> because nothing measured argued for escalating past it — **not**
> because it scored well through `SPEC-020`'s oracle. That oracle
> produced no score to ship on: `dnglab`/`rawler` do not implement DNG
> `OpcodeList` processing, so it cannot see this stage in either
> direction, and `AC8`/`AC9` are `#[ignore]`-marked with that reason
> inline (`DEC-024` Finding 2). The pre-registered escalation trigger was
> an oracle score below the bar; no such score exists, and a sharper
> kernel could not have moved a number the reference never computes.
>
> What the kernel choice actually rests on is `tests/warp.rs`'s analytic
> pair, which needs no corpus and no reference decoder: `AC4` reproduces
> each frame's own `f(1)`-derived corner displacement to within 1 px
> through this kernel, and `AC10` gives that check teeth — zeroing `kr1`
> moves the detected peak 339.5 px on `L1021223.DNG`'s coefficients,
> against a 20 px bar (measured by running the test, not carried forward
> from a document). Bicubic and Lanczos-3 remain the recorded,
> unimplemented alternatives; `DEC-024` records why chasing them would
> have been waste.

The 339.5 px figure was **re-measured for this commit** by running
`warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more` with
`--nocapture` (`honest peak (8367, 5583)`, `kr1=0 mutated peak
(8085, 5394)`), not copied from `DEC-024` — `unrun-docs-carry-errors`
applies to a correction exactly as it applies to the original claim.

**2. `src/lib.rs`, crate-root doc (was lines 56-57).**

Before:

> ...and [`warp`] (the `WarpRectilinear` resampler), wired into
> `develop_into` between `Orientation` and the (still absent) tone curve.
> Still absent, by design: `ASCII` and the signed field types...

After:

> ...and [`warp`] (the `WarpRectilinear` resampler), applied inside
> `develop_into` after levels normalization, over the full `ActiveArea`
> window, and BEFORE `DefaultCrop` extracts the final image area and
> `Orientation` reorients it (`DEC-024` Finding 1 — the design-time
> assumption that the warp ran after cropping and orientation was
> backwards; this sentence stated that pre-correction order in the very
> commit, `40f5d45`, that corrected the code). Still absent, by design:
> the tone curve (`SPEC-019`), `ASCII` and the signed field types...

The "very commit `40f5d45`" claim was verified by running the reader:
`git show 40f5d45 -- src/lib.rs` shows the wrong sentence being **added**
(`+//! wired into develop_into between Orientation and...`) in the same
commit whose `--stat` carries the `src/develop.rs` restructure.

### Incidental fixes, in the two SBs' immediate surroundings

Reported per return criterion 7; none chased beyond the doc comments
`SB-2` names.

- **Three dangling `` `DEC-*` `` placeholders resolved to `DEC-024`** —
  `src/warp.rs` x1 (the out-of-extent paragraph, one line above the
  rewritten Kernel section) and `src/develop.rs` x2 (inside
  `develop_into`'s own oracle and allocation docs). `SB-2` asks each
  rewrite to cite `DEC-024` by id; leaving a bare `DEC-*` in the
  adjacent sentence would have been incoherent. `grep -rn 'DEC-\*'
  src/ tests/` now returns 0 matches, down from 3.
- **One of those resolutions exposed a false claim of `SB-2`'s own
  species, and it was corrected rather than made specific.**
  `src/develop.rs`'s allocation doc said to see the kernel-choice DEC
  "for the alternative considered (a third caller-supplied buffer)".
  `DEC-024` records no such alternative — `grep -n 'caller-supplied\|
  third.*buffer\|scratch buffer'` on it returns nothing for the first
  two patterns and only its Consequences line for the third. The
  sentence now cites what `DEC-024` actually holds (that Consequences
  section's record of this allocation), and the neighbouring claim about
  `docs/provenance-ledger.md`'s `src/warp.rs` row carrying the measured
  peak RSS was checked the same way and **is** true (465,010,688 bytes,
  in that row).
- `src/warp.rs`'s out-of-extent paragraph also claimed the DEC carries
  "the measured per-frame scores". It does not, for `SB-2`'s own reason;
  that phrase is gone.

### Gates run (stating the list — `the-gate-count-is-not-defined-anywhere`)

All on `08ad42e`, corpus present
(`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`):

`cargo fmt --check` (clean), **`just lint-ci`** — clippy **0.1.98**
(`88d9e12ae1 2026-08-18`), the pinned/CI-floating version, not local
0.1.97, and the recipe printed which answered (PATCH-004) — `just
typecheck`, `just test` (**206 passed / 0 failed / 2 ignored**), `just
deny` + `just deny-fuzz` (both `licenses ok`; the only output is the
pre-existing unmatched-`Zlib`-allowance warning), `just msrv` (1.90.0),
`just lint-no-allow`, `./scripts/lint-red-proof.sh` (control clean →
injection rejected → all five lints fired), `./scripts/cost-audit-red-
proof.sh`, and **`just fuzz-warp`** — **22,937,786 runs in 61 s, zero
crashes**, `cov: 172 ft: 453 corp: 77/18Kb`. No new dependency in
either cargo graph; `Cargo.toml` is 0 lines changed.

**CI observed green on the ship SHA `08ad42e`** — run
[`34164609782`](https://github.com/jysf/irradiance/actions/runs/34164609782),
`conclusion: success`, **10 jobs, all success**, including `rust / fuzz
smoke — warp_opcode (60s)`, `rust / test`, `rust / clippy -D warnings`,
`rust / MSRV (1.90.0)`, both `cargo-deny` jobs, and `rust / lint policy
red-proof (must fail red)`. The paired `pull_request` run
`34164612764` is also green.

### Not touched, per the dispatch

`FU-1`..`FU-10` (dispositioned at ship, not here — including `FU-10`'s
oracle-narrowing question, which `DEC-024` explicitly leaves open),
`DEC-024` itself, `SPEC-018`'s acceptance criteria, and all PR
operations. PR **#16** was already open on arrival and is untouched —
this round pushed to the branch with `git push` only, and opened,
closed and modified nothing on GitHub.

---

## Completion — round 3 (punch-list)

Dispatched 2026-09-07 via `specs/prompts/SPEC-018-rebuild-2.md` after
`HANDOFF-048` returned `⚠ PUNCH LIST (round 3)` at `08ad42e`. Scope:
`SB-3` only — one rustdoc sentence. No new build handoff was minted
(`PATCH-003`'s round-2 pattern, continued); this section continues
`HANDOFF-046`, and the `handback:` block above now carries round 3's
numbers. **Round 2's, replaced there, were `tokens_total: 14298209`,
`estimated_usd: 34.26`, `duration_minutes: 29`, `completed_at:
2026-09-07`, on `claude-opus-5`**, and are already transcribed into the
spec's `cost.sessions` (the second `cycle: build` entry), so the
overwrite loses nothing. `synced_at` is reset to `null` so `just
handback-sync SPEC-018` appends round 3 as a THIRD build session rather
than skipping this file for idempotence.

Ship SHA: **`b92b30c`**. One commit, on top of `32599d2`.

### Summary

**One file, one paragraph, ten lines for three.** `src/warp.rs:53-55`
is the only change in the round: a doc comment in the module header.
No code, no test, no `DEC-*`, no spec text — `git diff --stat` for the
round is `src/warp.rs | 13 +++++++---`, and the decoded pixel output
cannot move because nothing outside a `//!` comment was touched. Test
count unchanged at **206 passed / 0 failed / 2 ignored**.

### `SB-3` — the false `DEC-024` citation is gone

**Before** (`src/warp.rs:53-55` at `08ad42e`, quoted verbatim):

```rust
//! The alternatives (zero-fill, error) are recorded in the kernel-choice
//! decision, `DEC-024` (`AC11`) — which records no per-frame oracle scores,
//! for the reason the Kernel section below states.
```

**After** (`src/warp.rs:53-62` at `b92b30c`, quoted verbatim):

```rust
//! The rule rests on §6.4.1's silence, searched above, plus `AC7` — the
//! acceptance criterion that pre-registered it — and not on any weighing of
//! candidates. `DEC-024` records the choice in a single Consequences
//! "Neutral" line: clamp-to-edge, on simplicity and determinism grounds,
//! pinned by `AC7`'s test. It records no alternative against it; its
//! "Alternatives Considered" (Options A, B, C) is entirely kernel and oracle
//! threshold. `DEC-024` does record measured oracle scores — `AC8` is
//! **-60.169** on `L1021223.DNG` — and those measurements are what
//! established that the oracle cannot validate this stage at all, for the
//! reason the Kernel section below states.
```

All three of the reverify's problems, in its own order:

**(A) The invented alternatives are gone.** `DEC-024` records neither
"zero-fill" nor "error" as an alternative to clamp-to-edge — it records
**no alternative at all** for the out-of-extent rule, and the rewrite now
says exactly that. Re-measured rather than carried from the handback
(§16 rule 4): `grep -rn zero-fill` over the working tree returns **9**
hits, six of them in process documents that discuss this very finding
(`SPEC-018.md:135`, its timeline `:87`, `prompts/SPEC-018-rebuild-2.md`
`:57,:62,:63`, `HANDOFF-048:44`) and three in source —
`tests/ifd_reader.rs:333` ("`600..616` stay zero-filled by `resize`") and
`tests/support/perturb.rs:14`, both unrelated, plus the claim itself at
`src/warp.rs:53`. `grep -c 'zero-fill\|zero fill'` on `DEC-024` returns
**0**. The reverify's "exactly 3 hits" and this "9" are the same
measurement over different scopes: its three are the source hits, and the
extra six are the documents the finding itself has since been written
into — including the dispatch that sent me here.

**(B) `AC11` → `AC7`.** `AC11` reads "the WarpRectilinear **kernel
choice** is a `DEC-*`, with measured scores... alternatives considered
(bilinear, bicubic, Lanczos), the chosen kernel, the measured score on
each of the three decodable frames" (`SPEC-018.md:428-433`) — nothing to
do with out-of-extent pixels. `AC7` is the criterion that pre-registered
this rule: "pixels whose source lands OUTSIDE the input extent are
handled per the DNG spec... If it says undefined, pick one and record in
the kernel-choice DEC" (`:387-395`), with
`outside_pixels_follow_the_dng_spec_rule` as its test. `AC7` is also the
AC `DEC-024` itself names in the same sentence that records the choice,
so the two now agree.

**(C) "records no per-frame oracle scores" → the measurement.**
`DEC-024:97-98` records `AC8` at **-60.169** on `L1021223.DNG`, and
`:109-110` `AC9`'s honest/mutated pair **-60.193 / -55.075**. The
rewrite states the `AC8` figure and says what it established. Phrased
non-exclusively ("does record measured oracle scores — `AC8` is...")
because those are not the only measurements in the record; a cleft
("what it records **is** the `AC8` score") would have been a fresh §16
rule 1 over-generalisation in the sentence fixing an imprecision. The
`AC8` figure was independently reproduced by running the reader, not
copied: `just test` prints it back in the `#[ignore]` reason for
`warp_scores_at_least_eightyfive_via_spec_020_oracle` — "Measured
-60.169 on L1021223.DNG".

### ⚠ Where this fix did NOT inherit its predecessor's precondition

The dispatch's own suggested wording was "the out-of-extent handling
clamps to edge **per DNG § 6.4.1**", and taking it literally would have
made round 3 the third instance of the same species — a fix inheriting
the precondition of the thing it fixes, which is precisely what
`HANDOFF-048` flagged.

**DNG 1.7.0.0 §6.4.1 does not hold a clamp-to-edge rule. It is silent.**
That is not my reading of the spec (I did not open it); it is what the
two documents this rustdoc must agree with already record:

- `DEC-024:195-196` — "The out-of-extent pixel rule (**DNG 1.7.0.0
  §6.4.1 is silent** — confirmed by full-text search of the
  specification) is clamp-to-edge".
- `src/warp.rs:36-43` — the section heading five lines above the
  rewrite, **approved by this same reverify**: "# Out-of-extent pixels —
  DNG 1.7.0.0 **is silent**; this build clamps", followed by the
  full-text search that established it.

So "clamps to edge per §6.4.1" would have contradicted the approved
paragraph it sits inside, one screen below its own heading. The rewrite
takes the dispatch's *intent* — the honest citation for this rule is the
DNG spec and this spec's own `AC7`, not a decision record that never
weighed it — and states it as **§6.4.1's silence plus `AC7`**, which is
what both sources actually support.

### What `DEC-024` was verified against, clause by clause

Read during the rewrite, not from memory (§16 rule 4). Every claim in
the ten new lines maps to one clause:

| New text | `DEC-024` clause |
|---|---|
| "records the choice in a single Consequences \"Neutral\" line: clamp-to-edge" | `## Consequences`, 5th bullet, **lines 195-196** — "**Neutral.** The out-of-extent pixel rule... is clamp-to-edge" |
| "on simplicity and determinism grounds" | same bullet, **line 197** — "chosen for simplicity and determinism" |
| "pinned by `AC7`'s test" | same bullet, **lines 197-199** — "`AC7` (`outside_pixels_follow_the_dng_spec_rule`, `src/warp.rs`) pins this choice" |
| "It records no alternative against it" | absence, asserted over the whole record: `## Alternatives Considered`, **lines 138-168**, holds Options A, B and C and nothing else |
| "its \"Alternatives Considered\" (Options A, B, C) is entirely kernel and oracle threshold" | **line 140** Option A "bicubic or Lanczos-3, chasing the `>= 85` score"; **line 148** Option B "relax `DEC-005`'s `>= 85` threshold"; **line 158** Option C "(chosen): keep bilinear... rely on the analytic checks" |
| "`AC8` is **-60.169** on `L1021223.DNG`" | `### Finding 2`, **lines 97-98** — "on `L1021223.DNG` scored **-60.169** (`AC8`)" |
| "those measurements... established that the oracle cannot validate this stage at all" | `### Finding 2` heading, **line 95** — "`SPEC-020`'s oracle cannot see `WarpRectilinear` at all"; mechanism at **lines 116-130** |

`DEC-024` is **byte-unchanged** this round — `git diff 08ad42e..b92b30c
-- decisions/` is empty.

### Gates run (stating the list — `the-gate-count-is-not-defined-anywhere`)

All on `b92b30c`:

- **`just lint-ci`** — clippy **0.1.98** (`88d9e12ae1 2026-08-18`), the
  pinned/CI-floating version, printed by the recipe. Clean under
  `-D warnings`.
- **`cargo fmt --check`** — clean. ⚠ **`just fmt` is not a recipe in
  this repo** (the dispatch's gate list names it); `cargo fmt --check`
  is what `just lint` runs, and is what I ran.
- **`just test`** — **206 passed / 0 failed / 2 ignored**, unchanged, and
  run **twice**: with `IRRADIANCE_CORPUS_DIR` pointed at
  `~/PSeven/experiments/crustimg_redo_plus/images` (`corpus: present` on
  all four frames) and with the variable **unset** (`corpus: SKIP` ×
  12, default root absent). Same 206/0/2 both ways — the tier-B
  discipline holds.
- **`just deny`** — `licenses ok`, exit 0. Only output is the
  pre-existing unmatched-`Zlib`-allowance warning.
- **`just msrv`** — `cargo +1.90.0 check --all-targets --all-features`,
  exit 0.
- **`just fuzz-warp`** (60 s) — **23,438,651 runs in 61 s, zero
  crashes**, `cov: 172 ft: 457 corp: 77/19Kb`. `git status` clean after.

No new dependency in either cargo graph; `Cargo.toml` is 0 lines
changed. `just typecheck` was not run separately — `just lint-ci` and
`just msrv` both compile `--all-targets --all-features`.

**CI observed green on the ship SHA `b92b30c`** — `push` run
[`34173566347`](https://github.com/jysf/irradiance/actions/runs/34173566347),
`conclusion: success`, **10 jobs, all success**: `rust / fuzz smoke —
warp_opcode (60s)`, `rust / test`, `rust / clippy -D warnings`, `rust /
fmt --check`, `rust / MSRV (1.90.0)`, both `cargo-deny` jobs, `rust /
lint policy red-proof (must fail red)`, `rust / panic-free policy — no
#[allow] escape (--lib)`, and `cost-capture audit`. The paired
`pull_request` run
[`34173569505`](https://github.com/jysf/irradiance/actions/runs/34173569505)
is also `success`, 10/10. Watched to completion with `gh run watch
--exit-status`, both exit 0 — observed, not inferred from a green
predecessor.

### Files touched

**One:** `src/warp.rs` (10 insertions, 3 deletions, all inside `//!`).
Plus this handback file, in a separate commit. `SPEC-018`'s timeline
already carried `build (round 3)` at `[~]` from the orchestrator's
dispatch and was left alone. Not touched, per the dispatch: `FU-1`
through `FU-11` (all dispositioned at ship — **including `FU-11`**, so
`cost.sessions` and `scripts/handback-sync.sh` are unedited), `DEC-024`,
`SPEC-018`'s body and ACs, `src/lib.rs`'s `SB-2` rewrite,
`src/develop.rs`, every test including `SB-1`'s, and all PR operations —
PR **#16** was open on arrival, and this round pushed to the branch with
`git push` only.

### ⚠ Carried forward for the orchestrator: `FU-11` will bite this entry too

`HANDOFF-046`'s `to_agent` is still `claude-sonnet-5` (round 1's value),
and per `HANDOFF-048` that is the field `scripts/handback-sync.sh:97`
reads. This session is **`claude-opus-5`** on every metered message.
Left unedited because the dispatch is explicit — *"Do NOT fix
HANDOFF-046's `to_agent` field"* — but that means the round-3
`cost.sessions` entry `handback-sync` writes will carry the **wrong
agent**, exactly as round 2's did. Not a new finding; `FU-11` restated
with one more instance for its ship-cycle fix.
