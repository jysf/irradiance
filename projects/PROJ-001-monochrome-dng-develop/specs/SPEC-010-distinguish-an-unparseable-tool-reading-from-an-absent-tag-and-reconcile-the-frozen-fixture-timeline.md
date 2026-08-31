# SPEC-010 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-010-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-22, main loop. Probed the defect rather than
  describing it: all four multi-valued tags collapse absent and garbled to a
  byte-identical `ToolReading`, and `BlackLevel [512,999]` reads `Some(512)`.
  Found the information is **discarded, not missing** — `Field.values` is
  already `Option<Vec<u32>>` and the distinction dies in `reading_from_fields`.
  The fix was already built and measured by `SPEC-005/FU-8`, so the spec tells
  build to **reproduce, not re-derive**. Eight failing tests named; the
  red-proof is the same code with one comparison removed. `HANDOFF-024` ready.
- [ ] **build** — `HANDOFF-024`. Dispatch to a separate CLI session.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
