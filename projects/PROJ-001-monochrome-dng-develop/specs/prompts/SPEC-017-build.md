# SPEC-017 — build dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root,
on the SPEC-017 branch). HANDOFF-044 is the contract. SPEC-018 shipped
first, so `src/opcode.rs` exists on `main` — SPEC-017 EXTENDS that module
with the `FixBadPixelsConstant` variant rather than scaffolding it.

The design's byte-level probe already ran (SPEC-017 `## Context` carries
the 28-byte OpcodeList1 payload for all three decodable Q2M frames). AC4
and AC5 are HIT-counts, not output-equality — the discipline STAGE-003
called out (a no-op opcode and an unexecuted opcode are indistinguishable
from output alone).

---

```
Cycle: build. You are the implementer for SPEC-017 — FixBadPixelsConstant.

Read first, in this order:
  1. AGENTS.md — §5 (the FOUR +toolchain traps), §11 (unread-field rule), §12
     (four testing bars including oracle-must-be-shown-red and fuzz-target-
     arrives-with-parser), §15 (cycle contract), §16 (four codified lessons +
     this week's a-fix-inherits-the-precondition, N=6 for unrun-docs-carry-
     errors).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-044-build-fixbadpixelsconstant-opcode.md
     — your contract. The four traps, the coordination note with SPEC-018 on
     src/opcode.rs, and the return criteria all live there.
  3. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md
     — the spec. `## Context` carries the probed 28-byte OpcodeList1 payload
     (identical byte-for-byte across all three decodable Q2M frames).
     `## Where the opcode module lives` is the SPEC-017 side of the shared
     src/opcode.rs contract with SPEC-018. `## Acceptance Criteria` (11 ACs).
     `## Failing Tests` (10 tests). `## Implementation Context` (design-time
     probes).
  4. projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-018-warprectilinear-radial-geometric-correction.md
     `## Where the opcode module lives` — SPEC-018's side of the coordination
     contract. Read to confirm the enum shape you extend and match SPEC-018's
     Unknown variant convention.
  5. src/opcode.rs — read it in full. SPEC-018 built the module first, so its
     Opcode enum is on main. You extend it with FixBadPixelsConstant.
  6. tests/warp.rs and tests/support/opcode.rs — SPEC-018's tests. Confirm the
     module shape by reading the tests that exercise it.
  7. src/develop.rs — the develop pipeline. `develop_into` is your integration
     point; the FixBadPixelsConstant applier runs BEFORE levels normalise,
     which is BEFORE `apply_warp_into` (per SPEC-018's Finding 1 pipeline
     order). Look at how SPEC-018 wired warp in and mirror the shape.
  8. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark build `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode-timeline.md

Branch: feat/spec-017-fixbadpixels-opcode (cut from main at 5908764 — the
SPEC-018 archive commit, so main is post-SPEC-018-ship).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
The default corpus root does not exist. AC5 (per-frame HIT count > 0) and
AC7 (SPEC-020 oracle score not worse) need the corpus; AC1-4, AC6, AC8-11
run without it.

Six things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. Both recipes print their own clippy version (PATCH-004).
  2. Every mutation must change the file AND compile AND change the output.
     Five false red-proofs across earlier specs.
  3. Tier-B tests pass whether or not the corpus is present — a silent skip
     is not a pass.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-044's `handback:` block
     only.
     ⚠ `notes:` MUST BE DOUBLE-QUOTED (signal `handback-sync-treats-hash-
     as-comment-in-unquoted-notes` filed this ship: unquoted values with a
     bare `#` silently truncate at the hash). Do NOT contain `#` in your
     notes.
     ⚠ Also update HANDOFF-044's top-level `to_agent` field to your actual
     model (signal `handback-sync-inherits-stale-to-agent-across-punch-list-
     rounds`: handback-sync reads that field for cost.sessions.agent, so if
     opus-5 runs but to_agent still says opus-5 as a prediction, correct it
     to what your system prompt reports).
  5. The opcode stream is BIG-ENDIAN; the TIFF payload is little-endian.
     SPEC-018 already handles this in src/opcode.rs — you inherit that
     convention. Write AC1's round-trip test FIRST on the 28-byte Q2M
     fixture and treat any parser change that alters its output as a defect.
  6. cargo fuzz shells to a bare `cargo build` — the +toolchain trap. Match
     `just fuzz-plane` (or `just fuzz-warp` from SPEC-018) line-for-line
     in `app.just` for the new `fuzz-opcode`.

What makes THIS spec different, and specifically:

  a. THE HIT COUNT IS THE ACCEPTANCE CRITERION, NOT AN OUTPUT DIFF.
     STAGE-003 explicitly required this: a no-op opcode and an unexecuted
     opcode are indistinguishable from the output alone. AC4 asserts
     `apply_fix_bad_pixels_constant(...) -> Result<usize, Error>` returns
     the replaced-pixel count. AC5 asserts count > 0 on each of the three
     real Q2M frames. AC8's tier-A red-proof mutates the applier to skip
     the replacement and asserts count returns 0. If your test asserts
     "output looks unchanged", you have missed the point.

  b. SRC/OPCODE.RS EXISTS ON MAIN — extend, don't rewrite. SPEC-018
     shipped the Opcode enum with FixBadPixelsConstant already listed in
     the pre-registered shape from SPEC-017's design (see
     src/opcode.rs). Confirm the exact enum shape matches SPEC-017's
     `## Where the opcode module lives`, add the parser branch for
     OpcodeID == 4, and add the applier as a new function. Do NOT
     rewrite the module.

  c. THE PIPELINE ORDER PLACES YOU BEFORE LEVELS NORMALIZE. SPEC-014
     built normalize; SPEC-018 built warp (over ActiveArea, before
     DefaultCrop per DEC-024 Finding 1). SPEC-017 runs BEFORE
     normalize — on the raw uncropped, un-normalised plane. Look at
     `develop_into` in src/develop.rs and insert the OpcodeList1
     application step at the correct position.

  d. Q2M CONSTANT = 0 IS DEEP BLACK. All three decodable Q2M frames
     carry OpcodeList1 with `Constant = 0`, `BayerPhase = 2`. Real bad
     pixels are RARE on this sensor; expect AC5's counts to be
     small-but-positive. Report them precisely in the handback — those
     are the first real data points on Q2M dead-pixel density this repo
     will have.

  e. THE PROVENANCE ROW REPLACES SPEC-018's PLACEHOLDER. SPEC-018's row
     in docs/provenance-ledger.md covers the module; SPEC-017's applier
     is a separate algorithm (per-pixel local median). Add a new row.

  f. `library-not-application` STILL BITES. No new `[dependencies]`;
     the 3×3 median and u32 parsing are hand-written. If Cargo.toml
     gains an entry, the whole spec is in question.

Do not open the PR — the orchestrator opens it after your handback lands.
Do not run handback-sync.

Return: findings labelled FU-N or SB-N — numbering RESTARTS at 1 for
SPEC-017 (per-spec convention). Then a filled `handback:` block in
HANDOFF-044 with:
  - status: completed | blocked | rejected
  - a REAL tokens_total deduped by message.id (identify your OWN session by
    scratchpad UUID, not text-matching — identify-own-transcript-for-cost-
    handback in this project's memory)
  - estimated_usd priced per-component at your model's rates, +20% uplift
  - notes: ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
  - the three per-frame HIT counts for AC5 (L1021223 / L1026016 / L1026192)
  - the three SPEC-020 oracle score deltas for AC7 (before → after
    FixBadPixelsConstant, per frame)
  - which of SPEC-017/018 landed src/opcode.rs first (answer: SPEC-018 —
    you extended)
  - CORRECT the handoff's top-level to_agent to your actual message.model
    (per FU-11 signal)
```
