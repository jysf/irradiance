# SPEC-014 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-014-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-05, main loop. Measured geometry and levels on all
  four decodable files. **The finding that shapes the spec:** no decodable file
  has a non-zero `ActiveArea` origin, so an implementation that ignores it
  passes every corpus test — `SPIKE-001`'s shape, with `SPIKE-002` as the
  precedent for the cost. `AC4`'s hand-built fixture is the only thing that can
  see it. Also measured that both files carry samples **below** `BlackLevel` and
  reach `WhiteLevel` **exactly**, so `AC2` is live on the first file. Eight ACs,
  seven failing tests, one `DEC` required. `HANDOFF-032` ready.
- [x] **build** — 2026-09-05, `HANDOFF-032`, dispatched to this CLI session
  (Sonnet 5 — the map predicted opus, corrected on hand-back). `src/develop.rs`
  added: levels normalization (clamped, `DEC-018`) and the `ActiveArea` →
  `DefaultCrop` (`DEC-019`) → `Orientation` geometry. Seven failing tests now
  pass; fuzz target `develop` ran 14,562,321 executions (61s), zero crashes;
  `SPEC-013`'s plane oracle re-run untouched. Branch
  `feat/spec-014-level-normalization-geometry-orientation`.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
