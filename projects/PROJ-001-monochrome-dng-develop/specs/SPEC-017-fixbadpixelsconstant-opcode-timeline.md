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
- [~] **build** — dispatched 2026-09-07 via
  `prompts/SPEC-017-build.md` on branch
  `feat/spec-017-fixbadpixels-opcode` (cut from `main` at `5908764`,
  post-SPEC-018 ship). `HANDOFF-044` is the contract. SPEC-018 shipped
  first so `src/opcode.rs` exists on `main`: SPEC-017 EXTENDS the
  `Opcode` enum with the `FixBadPixelsConstant` variant per the
  pre-registered shape in SPEC-017 `## Where the opcode module
  lives`. Pipeline insertion point is BEFORE `normalize_active_area_
  into` in `develop_into` (OpcodeList1 runs on the raw plane, before
  levels and before SPEC-018's warp). AC5 is the HIT-count assertion
  the STAGE-003 design note guarded against — a no-op opcode and an
  unexecuted opcode are indistinguishable from output alone.
  Handback discipline updated for this ship's new signals:
  `notes:` must be double-quoted and `#`-free (FU-5 sibling), and
  the handoff's `to_agent` must be corrected to the actual model
  before `handback-sync` runs (FU-11's inheritance trap).
- [ ] **verify** — awaits build.
- [ ] **ship** — awaits verify.
