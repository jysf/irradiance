# SPEC-068 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-068-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-13 (DEC-063: a unit is a full-width row band, so callers split dst with split_at_mut, no unsafe; probe: Q2M row bands 44.7 ms single -> 6.1 ms at 8 threads, byte-identical; crate kernel 238 ms vs probe 44.7 ms = SPEC-069 evidence; HANDOFF-143 build ready)
- [ ] **build** — HANDOFF-143
- [ ] **verify**
- [ ] **ship**
