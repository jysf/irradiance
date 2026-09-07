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
- [ ] **build** — dispatched via
  `HANDOFF-044-build-fixbadpixelsconstant-opcode.md`. Return
  criteria and coordination with SPEC-018 on `src/opcode.rs`
  documented in the handoff.
- [ ] **verify** — awaits build.
- [ ] **ship** — awaits verify.
