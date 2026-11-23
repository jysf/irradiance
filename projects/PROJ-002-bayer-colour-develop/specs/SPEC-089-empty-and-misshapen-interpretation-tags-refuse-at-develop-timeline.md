# SPEC-089 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-089-<cycle>.md`.

## Instructions

_(Timeline will be populated as the architect writes each cycle's prompt.)_

- [x] design — 2026-11-23, claude-opus-5-5 (orchestrator): both review findings
      reproduced through the parser at c78ec4c, neighbouring readers audited, corpus
      re-measured; DEC-078 §7 amendment; build handed to HANDOFF-185.
- [ ] build — HANDOFF-185, branch `feat/spec-089-misshapen-tags`, separate session.
- [ ] verify — a cold verify handoff after the build PR opens.
- [ ] ship
