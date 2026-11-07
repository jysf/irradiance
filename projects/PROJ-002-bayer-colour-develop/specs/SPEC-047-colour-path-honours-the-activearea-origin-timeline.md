# SPEC-047 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-047-<cycle>.md`.

## Instructions

- [x] **design** (2026-11-06) — `DEC-040` written (offset from the ActiveArea origin, no crop); Acceptance + Failing Tests filled; `HANDOFF-099` (build) and `prompts/SPEC-047-build.md` ready.
- [ ] **build** — `HANDOFF-099` → implementer: add `active_area_origin`, apply the offset in `normalize_mosaic` + `demosaic_into`, bump `DEVELOP_VERSION` to 3, make the failing tests pass. Dispatch with `prompts/SPEC-047-build.md`.
- [ ] **verify** — cold review against the acceptance criteria + the red-proof.
- [ ] **ship** — archive, record the `DEVELOP_VERSION` bump.
