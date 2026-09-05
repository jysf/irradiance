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
- [ ] **verify** — `HANDOFF-033`, at `80913a3` (branch head now `b4c56f6`, the
  reconciliation commit). Orchestrator reconciled the build against git and disk
  first (`DEC-004` rule 1): CI observed green on the shipping SHA itself
  (`80913a3`, run `33954821798`, 9/9), 141 tests / 0 failed summed with zero
  skips, `lint-ci`/`validate`/`cost-audit`/`decisions-audit` clean, all seven
  named failing tests matched against the live test list. **`AC4` measured, not
  inherited:** mutating `develop_into` to ignore the `ActiveArea` origin left
  140 of 141 tests green — the only failure was `AC4`'s hand-built fixture, to
  the exact wrong value the raw-plane reading gives. Seven checks handed on that
  the orchestrator did not make, including the nightly fuzz run and a new error
  introduced by `FU-1`'s own fix.
- [ ] **ship**
