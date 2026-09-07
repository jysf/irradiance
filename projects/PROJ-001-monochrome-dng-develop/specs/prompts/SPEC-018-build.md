# SPEC-018 — build dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo
root). Branch `feat/spec-018-warprectilinear-radial-geometric-correction`
is already created from `main` at `7fe53fb`. HANDOFF-046 carries the
full contract; the coefficient table in SPEC-018's `## Context` has
been PATCHED (SPEC-020 verify FU-1) to name the coefficients as
per-frame — reproduce the correction, do not inherit it from memory.

This is the single item in PROJ-001 that most directly decides whether
the project's thesis holds.

---

```
Cycle: build. You are the implementer for SPEC-018 — WarpRectilinear.

Read first, in this order:
  1. AGENTS.md — §5 (the FOUR +toolchain traps), §11 (unread-field rule), §12
     (four testing bars including oracle-must-be-shown-red and fuzz-target-
     arrives-with-parser), §15 (cycle contract, deferred follow-ups), §16 (four
     codified lessons + this week's a-fix-inherits-the-precondition).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md
     — your contract. The two things most likely to go wrong, coordination
     with SPEC-017 on src/opcode.rs, per-frame coefficient reality, and
     return criteria all live there.
  3. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md
     — the spec. Read `## Context` in full (⚠ per-frame coefficient table
     is authoritative), `## Acceptance Criteria` (14 numbered ACs), `## Failing
     Tests`, `## Implementation Context` (design-time probes required before
     the parser is written).
  4. projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-020-develop-oracle-vs-dnglab-srgb.md
     — the develop oracle that scores your output. `tests/perceptual_oracle.rs`
     and `tests/support/{pnm,ssimulacra2,perturb}.rs` are the wiring your AC8
     and AC9 call.
  5. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md
     `## Where the opcode module lives` — the coordination contract for
     src/opcode.rs from SPEC-017's side. Match the enum shape.
  6. spikes/done/SPIKE-001-*.md — the original coefficient measurement
     and the r-normalisation caveat.
  7. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark build `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md

Branch: feat/spec-018-warprectilinear-radial-geometric-correction (cut from main at 7fe53fb).
The handoff commit is on the branch already; work continues on top of it.

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
The default corpus root does not exist. AC8 (≥85 per frame) and AC9 (tier-B
kr1-zeroed) need the corpus; AC10 (tier-A kr1-zeroed) runs without it.

Six things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. Both recipes print their own clippy version (PATCH-004); quote
     what they say.
  2. Every mutation must change the file AND compile AND change the output.
     That third clause has caught FIVE false red-proofs.
  3. Tier-B tests pass whether or not the corpus is present — a silent skip
     is not a pass.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-046's `handback:` block
     only, `handback-sync` runs once cleanly. `notes:` must be ONE PHYSICAL LINE.
  5. The opcode stream is BIG-ENDIAN; the TIFF payload is little-endian.
     Write AC1's round-trip test FIRST on L1021223.DNG's exact bytes; treat
     any parser change that alters its output as a defect.
  6. cargo fuzz shells to a bare `cargo build` — the +toolchain trap. Match
     `just fuzz-plane` line-for-line in `app.just` for the new `fuzz-warp`.

What makes THIS spec different, and specifically:

  a. THE COEFFICIENTS ARE PER-FRAME. Only kr0 = 0.9992511060 is constant across
     L1021223 / L1026016 / L1026192; kr1 varies ~1.9x. SPIKE-001 measured ONE
     frame; SPEC-020 verify (FU-1) measured all three. AC1 commits one hex
     fixture per frame; AC4's corner-displacement expected value is per-frame;
     AC8's ≥85 must hold on ALL THREE frames. Do NOT hardcode ~504px or 0.899841
     as a Q2M constant — compute f(r=1) from each frame's own coefficients.

  b. `library-not-application` bites hard here. Bilinear ~20 lines, bicubic
     ~40, Lanczos-3 ~60 — all hand-written. `Cargo.toml` gains NO new
     `[dependencies]` entry. If it does, the whole spec is wrong.

  c. The kernel choice is PRE-REGISTERED: try bilinear first, measure on all
     three frames, if all ≥85 ship bilinear; if not, upgrade one step and
     record alternatives in a DEC. Do NOT tune parameters to hit 85; a
     kernel that needs tuning is a wrong kernel.

  d. SPEC-018 shares src/opcode.rs with SPEC-017. Whichever spec builds
     SECOND extends the module; do NOT rewrite. SPEC-017's initial enum
     shape is in its `## Where the opcode module lives` — match it. State
     in the handback which of SPEC-017/018 landed src/opcode.rs first.

  e. The DNG-spec probes (r normalisation, out-of-extent rule, opcode ID
     and version bytes) come from DNG 1.7.0.0 § 6.4.1, NOT from memory.
     Cite the exact clauses in the kernel-choice DEC. §16 rule 4
     (unrun-docs-carry-errors, N=6) is why.

  f. The `a-fix-inherits-the-precondition` lesson: DEC-005's ≥85 was
     calibrated at ¼ res on one file (-68); SPEC-020 lifted it to full
     metric on synthetic input (-82.338); SPEC-018 is a THIRD scale
     change — full res on REAL Q2M frames. Assume ≥85 until measured;
     material divergence is a finding, not a threshold to relax.

Do not open the PR — the orchestrator opens it after your handback lands.
Do not run handback-sync.

Return: findings labelled FU-N or SB-N — numbering RESTARTS at 1 for
SPEC-018 (per-spec convention). Then a filled `handback:` block in
HANDOFF-046 with:
  - status: completed | blocked | rejected
  - a REAL tokens_total deduped by message.id (identify your OWN session by
    scratchpad UUID, not text-matching — this project's memory records that
    trap as identify-own-transcript-for-cost-handback)
  - estimated_usd priced per-component at your model's rates, +20% uplift
    for the turns writing the handback (measured under-count: 9.9%-15.4%)
  - notes: ONE PHYSICAL LINE
  - per-frame scores for AC8 (L1021223 / L1026016 / L1026192)
  - the answer to "which of SPEC-017/018 landed src/opcode.rs first"
```
