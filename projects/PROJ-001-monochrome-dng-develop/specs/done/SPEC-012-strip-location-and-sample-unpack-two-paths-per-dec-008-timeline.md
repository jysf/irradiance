# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-04, main loop. Byte-level probe against both corpus
  shapes: hand-unpacked the 14-bit MSB-first and 16-bit LE strip heads and
  cross-checked both against `dnglab --raw-pixel`'s own plane — **they agree
  exactly**. The spec therefore pins first-sample values as measured fact, so
  the builder has a first-pixel checkpoint before `SPEC-013`'s MD5 oracle
  exists. The wrong paths were measured too (43019, 39186 — both impossible
  against `WhiteLevel 16383`), which is what `AC3` asserts. Nine acceptance
  criteria, seven failing tests, one `DEC` required for the allocation shape.
  `HANDOFF-028` ready.
- [ ] **build** — `HANDOFF-028`. Dispatch to a separate CLI session.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
