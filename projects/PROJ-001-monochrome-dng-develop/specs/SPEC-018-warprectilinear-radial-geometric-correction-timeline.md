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
- [x] **verify (round 1)** — `HANDOFF-047` dispatched and completed
  2026-09-07 on the same branch, `⚠ PUNCH LIST` at `40f5d45`,
  22,631,969 tokens on claude-opus-5 ($51.98, 22 min). Verifier
  verified BOTH findings against primary evidence: read DNG 1.7.0.0
  directly for Finding 1 (pipeline-order fix confirmed correct);
  used behavioural NCC-tile analysis (invariant to affine tone
  changes) rather than reading dnglab source for Finding 2
  (confirmed dnglab does not apply WarpRectilinear). SBs raised:
  SB-1 (develop_into's warp branch has no live test — mutation
  proved 205/0/2 unaffected by crop-then-warp) and SB-2 (two false
  claims in shipped rustdoc: src/warp.rs:59-61 fabricated scores;
  src/lib.rs:56-57 pre-Finding-1 pipeline order). 10 FUs including
  FU-5 (handback-sync silent-truncate at bare `#`, sibling of
  handback-sync-truncates-multi-line-scalars) and FU-10 (Finding 2
  disposition: ship with #[ignore]s + follow-up spec narrowing
  SPEC-020's oracle scope).
- [~] **build (round 2, punch-list)** — dispatched 2026-09-07 via
  `projects/PROJ-001-monochrome-dng-develop/specs/prompts/SPEC-018-rebuild.md`.
  Scope: SB-1 (add tier-A test exercising the develop_into warp
  branch that turns red on crop-then-warp mutation) and SB-2 (fix
  two rustdoc claims to match DEC-024). FUs deferred to ship.
  Handback continues on HANDOFF-046 (no new build handoff minted;
  matches PATCH-003 round-2 pattern).
- [ ] **verify (round 2)** — awaits round-2 build. New verify
  handoff (HANDOFF-048 or similar) confirms the SBs are closed and
  proceeds to ship.
- [ ] **ship** — CI observed green on the shipping SHA (AC14), the
  new `fuzz-warp` recipe wired into CI as a smoke run in the same PR
  (§12 bar 2: fuzz targets arrive with the parser, not retrofitted),
  provenance rows landed (AC13), Follow-ups table populated per
  AGENTS.md §15. If AC8's measured full-resolution score lands
  materially different from DEC-005's ¼-res calibration, ship
  cycle emits a follow-up recalibrating the threshold (DEC-005
  revisit condition).
