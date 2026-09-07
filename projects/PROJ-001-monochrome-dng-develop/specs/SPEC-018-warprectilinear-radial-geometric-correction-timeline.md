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
- [x] **build (round 2, punch-list)** — completed 2026-09-07 at ship
  SHA `08ad42e` (bookkeeping tip `957612e`). HANDOFF-046 round-2
  handback: 14,298,209 tokens on claude-opus-5 ($34.26, 29 min),
  notes correctly double-quoted (avoiding FU-5's bare-`#` truncation).
  SB-1 closed via new tier-A test
  `develop_into_crops_from_the_warped_active_area_not_the_warped_crop`
  — the only test in the tree developing a non-None opcode_list_3 to
  pixels, hand-built Sensor with ActiveArea 80×64 and DefaultCrop
  60×48 off-centre so the two orders separate 2837/2880 pixels;
  red-proof observed both directions (src/develop.rs md5
  318977…→cf714e…→318977…). SB-2 closed by rewriting src/warp.rs
  52-77 and src/lib.rs 56-58 to match reality, citing DEC-024;
  incidentally resolved three dangling DEC-* placeholders and
  corrected one further false claim of the same species. 206/0/2
  tests, CI green on 08ad42e (run 34164609782, 10 jobs including
  fuzz-warp smoke), no logic changes (irr develop L1021223.DNG max
  51764 unchanged). FUs 1-10 UNTOUCHED and still owed a ship
  disposition.
- [x] **verify (round 2, reverify)** — completed 2026-09-07, `⚠ PUNCH
  LIST (round 3)` at ship SHA `08ad42e`. HANDOFF-048 handback:
  6,790,688 tokens on claude-opus-5 ($17.38, 16 min), notes properly
  double-quoted per FU-5. SB-1 CLOSED (fresh mutation red-proof
  reproduced on the shipped file — md5 11118242… → f620d615… →
  11118242…, fixture separates 2837/2880 pixels, identity warp
  bit-identical). SB-2 CLOSED for both named claims. **SB-3 NEW,
  ship-blocking:** SB-2's own rewrite in src/warp.rs:53-55
  introduced a false DEC-024 citation — claims DEC-024 (AC11) records
  "zero-fill, error" out-of-extent alternatives, but DEC-024 records
  neither and AC11 governs kernel choice not out-of-extent rule; same
  sentence's "records no per-frame oracle scores" is imprecise
  (DEC-024:98 records -60.169). Same species as SB-2, landed INSIDE
  the SB-2 correction pass — `a-fix-inherits-the-precondition` biting
  §16 rule 4 (unrun-docs-carry-errors). **FU-11 NEW:** cost.sessions
  round-2 build entry says agent: claude-sonnet-5 but the session was
  opus-5 (97 metered messages, priced at Opus rates); handback-sync
  read HANDOFF-046's to_agent left at round-1's stale value.
  Non-gating, ship-cycle fix. FU-1..10 all still live.
- [~] **build (round 3, punch-list)** — dispatched 2026-09-07 via
  `prompts/SPEC-018-rebuild-2.md`. VERY tight scope: fix SB-3 only —
  rewrite src/warp.rs:53-55 to name what DEC-024 actually holds
  (clamp-to-edge per DNG § 6.4.1, no alternatives weighed), and
  correct the AC8 measured-score imprecision to cite -60.169. FUs
  1-11 stay open. Handback continues on HANDOFF-046 as round 3.
- [ ] **verify (round 3)** — awaits round-3 build. Fresh handoff
  confirms SB-3 closed cleanly and no round-4 SB introduced.
- [ ] **ship** — CI observed green on the shipping SHA (AC14), the
  new `fuzz-warp` recipe wired into CI as a smoke run in the same PR
  (§12 bar 2: fuzz targets arrive with the parser, not retrofitted),
  provenance rows landed (AC13), Follow-ups table populated per
  AGENTS.md §15. If AC8's measured full-resolution score lands
  materially different from DEC-005's ¼-res calibration, ship
  cycle emits a follow-up recalibrating the threshold (DEC-005
  revisit condition).
