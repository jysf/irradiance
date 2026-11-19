# SPEC-059 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-059-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-19 (STAGE-016 design PR 266; DEC-069; HANDOFF-161 build ready)
- [x] **build** — HANDOFF-161, completed 2026-11-19 (branch claude/spec-059-build; GainMap parsed and applied at stage 2 on both paths, apply_gain_maps public, DEVELOP_VERSION 6; oracle bit-exact on 15 shapes; 8 mutants, 7 red, the no-clip mutant equivalent by the saturating cast and its wrapping form red; the Pixel refusal test rewritten to assert both paths develop, flagged for the orchestrator)
- [ ] **verify**
- [ ] **ship**
