# SPEC-038 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-038-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (fuzz-target spec + DEC-036 the `fuzzing`-feature exposure + HANDOFF-089 + build prompt)
- [x] **build** — PR #106, 2026-11-05 (HANDOFF-089) — libFuzzer `ljpeg` target over the single-component SOF-3 decoder via the `fuzzing` feature (DEC-036); 14 own-work seeds + two in-crate panic-free replay tests; all six gates green; manual nightly fuzz ran 2.55M executions with no crash. Cost 267,704 tokens (FLOOR — context-occupancy, no interactive /cost).
- [x] **verify** — APPROVED 2026-11-05 at commit `d6b73ad9aa92f2bfdd13ba043b80c7ff04888cdc` (HANDOFF-090). Cold review of PR #106: six gates green (five via #106 CI on this exact commit — test, clippy, clippy fuzz graph, fmt-check, license policy + license fuzz graph; plus `just validate` locally, 46 artifacts); panic-free replay tests confirmed non-tautological (each hostile input asserts a distinct typed Error variant that exists in lib.rs; corpus non-empty asserted); DEC-036 fuzzing-feature keeps `ljpeg` pub(crate) with the fuzz-graph gate green; 14 own-work seeds; no new runtime dep; scope limited to tests + the one feature entry. Nightly cargo-fuzz spot-run: 3,098,220 runs in 31s, no crash. Cost 185,694 tokens (FLOOR — context-occupancy, no interactive /cost). Prompt: `prompts/SPEC-038-verify.md` (the verify prompt was delivered as the session task).
- [ ] **ship** — prompt: `prompts/SPEC-038-ship.md` — APPROVED; last STAGE-009 spec, so run Prompt 1d (Stage Ship) for STAGE-009 after shipping.

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
