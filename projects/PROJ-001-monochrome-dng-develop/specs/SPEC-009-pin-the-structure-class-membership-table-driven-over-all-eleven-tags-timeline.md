# SPEC-009 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-009-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-03, main loop. Re-measured all four carried
  `SPEC-008` findings on `main` rather than inheriting them; every one still
  holds at 96 tests, thirty more than when they were raised. Found that
  `DEC-014` has changed `AC4`'s stakes — `malformed_tags` is now an *input to
  the oracle*, so recording more tags widens its blind spot. Seven acceptance
  criteria, five failing tests, and the red-proof is an eleven-way mutation.
  `HANDOFF-026` ready.
- [ ] **build** — `HANDOFF-026`. Dispatch to a separate CLI session.
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
