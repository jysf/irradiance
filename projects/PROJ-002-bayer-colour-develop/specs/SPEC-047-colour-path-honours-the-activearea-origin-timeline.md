# SPEC-047 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-047-<cycle>.md`.

## Instructions

- [x] **design** (2026-11-06) — `DEC-040` written (offset from the ActiveArea origin, no crop); Acceptance + Failing Tests filled; `HANDOFF-099` (build) and `prompts/SPEC-047-build.md` ready.
- [x] **build** (2026-11-06) — `HANDOFF-099` → implementer: added `active_area_origin`, applied the offset in `normalize_mosaic` + `demosaic_into`, bumped `DEVELOP_VERSION` to 3, 4 new Tier-A tests pass. PR #137 (merged to main as `0e4b4dc`).
- [x] **verify** (2026-11-06) — `HANDOFF-100`: cold review against the acceptance criteria + the red-proof. **✅ APPROVED** at `0e4b4dc`. Red-proof shown a real red (DEC-040 neutralized → `plane_relative_phase_tints` and `grey_stays_grey` both fail; 40,716-count magenta). All gates green; PR #137 CI all-green. Cost ≈195k tok (harness session meter; floor).
- [ ] **ship** — archive, record the `DEVELOP_VERSION` bump.
