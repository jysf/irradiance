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
- [ ] **verify** — ready to start; SB-1/FU-1/FU-2 await disposition.
- [ ] **ship** — awaits verify.
