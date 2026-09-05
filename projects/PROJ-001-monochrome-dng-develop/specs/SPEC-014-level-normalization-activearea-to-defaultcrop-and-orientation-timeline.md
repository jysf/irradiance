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
- [x] **verify** — 2026-09-05, `HANDOFF-033`, dispatched to this CLI session
  (Opus 5 — the map predicted opus and was RIGHT this time). **✅ APPROVED at
  `52e6ecf`**, 0 ship-blockers, 6 follow-ups (`FU-2`…`FU-7`). Corpus present
  7/7, **zero SKIP lines**, 141 tests / 0 failed summed across 9 targets; eleven
  gates + `lint-ci` (clippy 0.1.98 asserted) green; CI **observed** 9/9 on the
  approved SHA itself (run `33980344540`). `AC4`'s fixture **watched red** under
  the ActiveArea-origin mutation — 140/141, `left: 44` vs `right: 172` — and the
  premise re-measured on all 7 files rather than inherited. Fuzz 12,167,207 runs
  / 61 s, zero crashes, seeds byte-unchanged. **New this cycle:** the eight
  `Orientation` values corroborated against **ImageMagick**, 48/48 cells
  identical, closing the six that had no independent point; and a second
  mutation showed **`develop_into`'s orientation pixel path is asserted by
  nothing** (141/141 green while the output changed) — `FU-3`. `oracle-must-be-
  shown-red` judged **inapplicable, not evaded** (no oracle, no gate; `DEC-004`
  says why in advance), with its principle met for `AC4` and unmet for `AC5`.
- [x] **ship** — 2026-09-05. `HANDOFF-034`'s punch-list round (Sonnet 5 — the
  map predicted opus; 0-for-10 on the non-verify hint) discharged all six
  follow-ups; the orchestrator reconciled, found three more (`FU-8`, `FU-9`,
  `FU-10`), and dispositioned all ten. **143 tests, 0 failed**, eleven gates +
  `lint-ci` at clippy 0.1.98, CI 9/9 on `0129796`. Library code **byte-identical
  to the approved SHA** — every change was a test, a doc, a seed or a decision.
  Both red-proofs re-watched by the orchestrator **with the corpus absent**, so
  `FU-3`'s new test closes its hole in the condition CI actually runs in.
  `complexity_actual: L`; totals **88,845,024 / ≈$60.47** against a 26,000,000
  estimate — **3.42×**, the largest miss recorded. Worst defect caught at
  `verify`. Original entry, superseded:
  `HANDOFF-034`, the punch-list round, delegated. `SPEC-014` is
  APPROVED and at `cycle: ship`; five of its six follow-ups carry a `fixed`
  disposition, which §15 lets ship discharge. `FU-3` (a tier-A integration test
  through `develop_into` — the mapper is pinned, its USE is not) and `FU-4` (pin
  `normalize`'s rounding and record it in `DEC-018`, since round and truncate
  differ on 50.0 % of in-range samples and the one point the test pins is where
  they agree) are the two with real content; `FU-2`/`FU-5`/`FU-6` are a seed and
  two docs, `FU-7` is a signal entry. ⚠ Non-standard: `new-handoff` refuses a
  `ship` cycle, so `HANDOFF-034` was hand-written, and its cost session is a
  **metered** ship — the exception §4's "not separately metered" does not cover.
  Bookkeeping (Reflection, Follow-ups table, totals, archive) stays with the
  orchestrator.
