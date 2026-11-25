# SPEC-095 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-095-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-11-25 (STAGE-032 framed after the STAGE-017 close, PR 342)
- [x] **design** — completed 2026-11-25 (0.2.7 patch, semver clean on 153f517 with and without all features; docs.rs metadata; writer-only, PR 343 held)
- [ ] **build** — prompt: `prompts/SPEC-095-build.md` (HANDOFF-197)
- [ ] **verify** — prompt: pending (cold verify, after the build)
- [ ] **ship** — prompt: pending (after the user's tag and publish)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).
