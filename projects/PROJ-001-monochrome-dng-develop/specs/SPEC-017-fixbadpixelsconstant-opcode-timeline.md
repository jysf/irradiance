# SPEC-017 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-017-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06, main-loop (unmetered). Rescoped from
  the framed stub (`(not yet written)`) into the full design at
  `SPEC-017-fixbadpixelsconstant-opcode.md`, with `## Failing Tests`
  drafted and `## Implementation Context` filled from a direct
  `exiftool -b -OpcodeList1` probe of all three decodable Q2M
  frames. Design does not commit any code (per §12 test-before-
  implementation); it commits the fixtures the build will assert
  against.
- [x] **build** — completed 2026-09-07 via `prompts/SPEC-017-build.md`
  on branch `feat/spec-017-fixbadpixels-opcode` (cut from `main` at
  `5908764`, post-SPEC-018 ship). `HANDOFF-044`'s `## Completion`
  section has the full report. SPEC-018 landed `src/opcode.rs` first;
  extended it with `Opcode::FixBadPixelsConstant` (ID 4) per the
  pre-registered shape — no rewrite. `apply_fix_bad_pixels_constant`
  wired into `develop_into` BEFORE levels normalize. 10/10 named tests
  exist and are real; 9 pass, 1 (`AC5`,
  `q2m_frames_have_at_least_one_replaced_pixel`) is `#[ignore]`d with
  a measured reason — **SB-1: all three decodable Q2M frames measure
  HIT = 0**, not "small-but-positive" as designed. AC1/AC4/AC8
  independently prove the parser and applier are both correct and
  reached, so this is a real finding about the corpus, not an
  applier defect — needs a verify/ship judgment call (relax AC5's
  threshold, or treat as a real gap). AC7 (SPEC-020 oracle,
  before-vs-after) passes: two comparable frames score identically
  (delta +0.000, consistent with HIT=0), one frame (`L1026016.DNG`)
  skips loudly — `FU-2`: `dnglab --srgb` never applies EXIF
  `Orientation`, a pre-existing SPEC-020-oracle gap this build
  surfaced, not a SPEC-017 defect (the identically-shaped, already-
  `#[ignore]`d `SPEC-018` oracle test would hit the same mismatch).
  `FU-1`: AC3's own pre-registered example used `Flags: 0` but
  SPEC-018's shipped parser already dispatches mandatory-vs-optional
  at parse time, making that exact case unreachable as literally
  written — adapted to `Flags: 1`, documented inline; `AC6`'s
  mandatory-unknown case is covered separately through
  `develop_into`. Eleven gates + `fuzz-opcode` (13,696,114 runs, 0
  crashes) + `cost-audit-red-proof` all green. Chapter citation
  corrected: `FixBadPixelsConstant` is DNG 1.7.0.0 **Chapter 7**
  "Opcode List Processing" (p.95), not Chapter 6 as this handoff and
  the spec both said — verified against the published DNG 1.6.0.0
  PDF directly during build. Pushed as `cd82ca8`; CI run `34191738033`
  completed/success, 11/11 jobs green (including the new `fuzz smoke
  — opcode (60s)` job).
- [x] **verify (round 1)** — completed 2026-09-08, `⚠ PUNCH LIST` at
  ship SHA `cd82ca8`. HANDOFF-050 handback: 27,235,932 tokens on
  claude-opus-5 ($65.83, 30 min), to_agent CORRECTLY verified before
  handback-sync (second spec where FU-11 discipline holds). SB-1
  DOWNGRADED to FU-3 with mechanical evidence: verifier wrote a C
  14-bit unpacker from the DNG packing rule and reproduced `irr unpack`
  independently — ZERO zeros inside ActiveArea on all three frames,
  whole-plane minima 2/30/2. Applier is correct; AC5's "small but
  positive" assumption was wrong. **SB-2 NEW, ship-blocking:** nothing
  asserts `develop_into` USES the fixed plane — mirrors SPEC-018/SB-1
  exactly. Severing `effective_src → src` (md5 `b3b922d0` → `548e240f`)
  compiles, leaves 231/0/3 green, and the mutant AC7 passes with delta
  +0.000 on both comparable frames because HIT=0 makes before/after
  bit-identical. Direct hit on STAGE-003's "assert branch HIT, not
  merely image unchanged". One-line fix pre-registered by verifier.
  All four repo-specific bars pass; several FUs (FU-4 false rustdoc,
  FU-5 provenance row placement, FU-6 peak-RSS ledger drift, FU-7
  in-place neighbour semantics, FU-8 stale AC text, FU-9 tokens_total
  method inconsistency) dispositioned at ship.
- [x] **build (round 2, punch-list)** — completed 2026-09-08 at ship
  SHA `6e4376b` (bookkeeping tip `759f200`). HANDOFF-044 round-2
  handback: 9,026,059 tokens on claude-opus-5 ($ per-component + 20%
  uplift), to_agent correctly verified before handback-sync (FU-11
  discipline holding for the second round). SB-2 closed by extending
  `develop_output_is_bit_identical_across_two_runs` with a
  centre-pixel assertion at `tests/develop.rs:594-599`
  (`dst1[2 * 5 + 2] == 1000`). Red-proof: sever `effective_src → src`
  in src/develop.rs (md5 b3b922d0 → 1c4f4e34, compiles), assertion
  turns red at :594 with "left: 0 / right: 1000"; reverted, tree
  byte-identical, green. Suite 231/0/3 → 230/1/3 under mutation.
  Note: round-1 verify's md5 (548e240f) not reproducible from the
  edit text recorded — semantic mutation is same, exact edit differs.
  Round-2 build also flagged (courtesy) that the applier is in
  src/develop.rs not src/opcode.rs — FU-5 from round-1 already
  covers this split. CI 11/11 green on both SHAs. FUs 1-9 remain
  open.
- [~] **verify (round 2)** — `HANDOFF-051` dispatched 2026-09-08 via
  `prompts/SPEC-017-reverify.md`. VERY tight scope: reproduce the
  SB-2 red-proof semantically (mutation md5 need not match), check
  no new SB introduced by the diff, confirm src/ untouched. Round-1
  findings and FU-1..9 out of scope.
