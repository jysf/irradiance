# SPEC-013 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-013-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-04, main loop. The plane is already bit-exact (4/4,
  verified twice), so this spec is about making the assertion permanent and
  **provably able to fail**. The probe's most useful result was a mistake it
  made: an injected fault that changed the file, compiled, and was a semantic
  **no-op** — so `AC4` now requires asserting the **output** changed, not just
  the file. ⚠ A genuine faulty digest was **not** obtained (two runs killed by
  timeouts on a 95 MB plane) and is deliberately not quoted. Six ACs, six
  failing tests. `HANDOFF-030` ready.
- [ ] **build** — `HANDOFF-030`. Dispatch to a separate CLI session.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
