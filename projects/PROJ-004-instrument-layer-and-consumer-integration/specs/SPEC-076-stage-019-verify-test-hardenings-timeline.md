# SPEC-076 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-076-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-19 (five [XS] hardenings from HANDOFF-138, 140 and 142, each with a named mutant; tests only; built after SPEC-075 merges because both edit tests/black_levels.rs; HANDOFF-158 build ready)
- [x] **build** — HANDOFF-158, completed 2026-11-19 (PR 264; 732 passed, 0 failed with 12/12 corpus files; six mutants shown red, each in its own target dir; H3 guard on the AC3 pattern only, since the narrow-spread case is not moved by the warp)
- [ ] **verify**
- [ ] **ship**
