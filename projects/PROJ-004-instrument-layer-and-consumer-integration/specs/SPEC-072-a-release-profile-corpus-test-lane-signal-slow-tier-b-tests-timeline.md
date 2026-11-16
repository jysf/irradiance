# SPEC-072 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-072-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-16 (DEC-066: a [profile.corpus] inheriting test with opt-level 3, so assertions and overflow checks stay on; just test runs every test under it, just test-debug keeps the old run, CI unchanged; probe: full corpus suite 157 s against about 37 min in debug, same 697/0/1, every pin held; target <= 5 min; cut after SPEC-071 merges; HANDOFF-150 build ready)
- [ ] **build** — HANDOFF-150
- [ ] **verify**
- [ ] **ship**
