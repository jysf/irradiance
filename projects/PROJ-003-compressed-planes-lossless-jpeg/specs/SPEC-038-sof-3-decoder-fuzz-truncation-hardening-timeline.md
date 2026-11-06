# SPEC-038 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-038-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (fuzz-target spec + DEC-036 the `fuzzing`-feature exposure + HANDOFF-089 + build prompt)
- [x] **build** — PR #106, 2026-11-05 (HANDOFF-089) — libFuzzer `ljpeg` target over the single-component SOF-3 decoder via the `fuzzing` feature (DEC-036); 14 own-work seeds + two in-crate panic-free replay tests; all six gates green; manual nightly fuzz ran 2.55M executions with no crash. Cost 267,704 tokens (FLOOR — context-occupancy, no interactive /cost).
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
