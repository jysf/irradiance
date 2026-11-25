# SPEC-096 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-096-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-25 (no new DEC; cites DEC-002, copies SPEC-056's gate shape)
- [~] **build** — HANDOFF-200, branch `chore/spec-096-wasm32-build-gate`
- [ ] **verify** — HANDOFF-201, a separate cold agent
- [ ] **ship** — after the user merges

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).
