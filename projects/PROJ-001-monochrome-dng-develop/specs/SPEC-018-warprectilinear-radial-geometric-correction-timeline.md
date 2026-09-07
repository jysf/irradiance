# SPEC-018 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-018-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06, main-loop, not separately metered.
  Cycle transitioned frame → design. 14 acceptance criteria, 12 named
  failing tests, `depends_on: [SPEC-020]` (SPEC-018 is scored through
  SPEC-020's oracle — AC8 — so shipping without SPEC-020 in place is
  the trap SPEC-015 was created to break, applied to STAGE-003). Two
  new modules (`src/opcode.rs` — the OpcodeList3 parser, and
  `src/warp.rs` — the resampler), a mandatory fuzz target
  (`warp_opcode`, §12 bar 2 fires), a DEC-* for the resampling
  kernel choice (bilinear first, upgrade only if AC8 requires,
  measurements go in the DEC's alternatives-considered), and two
  provenance rows (parser + application). SPIKE-001's warp
  coefficients are recorded as measurements; DNG 1.7.0.0 § 6.4.1 is
  the required-before-build probe (radius normalisation and
  out-of-extent rule).
- [x] **build** — `HANDOFF-046` dispatched 2026-09-06 on branch
  `feat/spec-018-warprectilinear-radial-geometric-correction` (cut
  from `main` at `7fe53fb`), completed 2026-09-07 with findings.
  `src/opcode.rs` created first (SPEC-017 not yet built); parser
  round-trips all three frames' real `OpcodeList3` bytes byte-for-byte.
  Bilinear kernel shipped per the pre-registered rule. Design-time
  re-reading of DNG 1.7's own `DefaultCropOrigin`/`Size` text found
  the pipeline-order assumption above wrong: `WarpRectilinear` runs
  over `ActiveArea`, before `DefaultCrop`, not after — corrected in
  `src/develop.rs` (`DEC-024` Finding 1). 12/14 ACs met and tested
  green (AC1-7, AC10-14; AC4/AC10 are this spec's real, oracle-free
  correctness proof). AC8/AC9 measured but marked `#[ignore]`:
  `dnglab`/`rawler` do not implement DNG `OpcodeList` processing at
  all (confirmed against the `dnglab/dnglab` source — no
  `WarpRectilinear`/`FixBadPixelsConstant` application code anywhere
  in the repository), so SPEC-020's oracle cannot validate this
  feature in either direction (`DEC-024` Finding 2) — a correct warp
  scores WORSE against dnglab's uncorrected reference, not better.
  See `HANDOFF-046`'s `handback:` block and `DEC-024` for the full
  record.
- [ ] **verify** — a separate agent runs `warp_scores_at_least_
  eightyfive_via_spec_020_oracle` on the corpus (AC8), the tier-A
  red-proof `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_
  more` with corpus unset (AC10), and — the DEC-004-rule-1 verify
  discipline — the tier-B mutation red-proof by hand (AC9) rather
  than trusting the harness. Confirms the kernel-choice DEC records
  measured scores per candidate kernel, not just the winner.
- [ ] **ship** — CI observed green on the shipping SHA (AC14), the
  new `fuzz-warp` recipe wired into CI as a smoke run in the same PR
  (§12 bar 2: fuzz targets arrive with the parser, not retrofitted),
  provenance rows landed (AC13), Follow-ups table populated per
  AGENTS.md §15. If AC8's measured full-resolution score lands
  materially different from DEC-005's ¼-res calibration, ship
  cycle emits a follow-up recalibrating the threshold (DEC-005
  revisit condition).
