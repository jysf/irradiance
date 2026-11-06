# SPEC-043 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-043-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe of both held files — both single-strip, 12/14-bit, MM/II — + the `unpack_into` routing seam + the `Frame`/`decode_into` contract; acceptance + failing tests + HANDOFF-091 + build prompt)
- [ ] **build** — prompt: `prompts/SPEC-043-build.md` (HANDOFF-091)
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
