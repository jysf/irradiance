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
- [ ] **build** — handoff pending. Blocks on SPEC-020 shipping (AC8
  reads the SPEC-020 oracle). Build must resolve DNG § 6.4.1's `r`
  normalisation and out-of-extent rule against the spec — SPIKE-001
  flagged the first as unconfirmed — before writing the parser or the
  applier. Kernel choice is measurement-driven: bilinear first, and
  only upgrade if the SPEC-020 score does not clear 85 on all three
  decodable frames.
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
