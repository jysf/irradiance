# SPEC-074 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-074-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-18 (bundles STAGE-021's last two [XS] follow-ups; probe on 28ecd5d: slugify already caps new names at 60 since template v0.6.49, so 83 legacy files are renamed; semver-checks --baseline-rev crashes on a clone temp name = file name + about 45 bytes, the crates.io baseline runs 202/202 clean; a throwaway one-parse plane path measured 5 allocations per C7 chunk with plane tests green; DEC-067; HANDOFF-155 build ready)
- [x] **build** — HANDOFF-155, completed 2026-11-18 (PR 259: one Compression 7 header parse, 5 allocations per chunk; 83 legacy artifacts renamed to the slug cap; name-length-check in CI; semver-check on the crates.io baseline; all gates green)
- [x] **verify** — HANDOFF-156, completed 2026-11-18 (APPROVED at f56f5eb: rename set re-derived 83/83 with pure reference rewrites; one C7 parse keeps parse, shape, scan order and is pinned by tests in both paths; gate red-proof reproduced; semver 202/202 both ways; just test 710/0/1 with 12/12 corpus; all gates and 22/22 CI green)
- [ ] **ship**
