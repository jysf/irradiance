# SPEC-077 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-077-<cycle>.md`.

## Instructions

- [x] **build** — HANDOFF-165: completed 2026-11-19, PR #272 (Phase 1 prep, unmerged; cargo-semver-checks 0.51.0 against crates.io 0.2.2 clean, 202 of 202 pass, nothing to accept; 0.2.3, CHANGELOG promoted with the two DEVELOP_VERSION entries merged, release notes refreshed, README and crate-doc wording fixed; candidate cf8b252, 12/12 corpus, 764 passed, 224 s; SDK gain step within 1 code; dry-run 19 files, sha1 cf8b252, no self dev-dependency; all 14 gates green; cost floor in HANDOFF-165)
- [x] **verify** — HANDOFF-166: completed 2026-11-19, APPROVED at c69c813 on claude/spec-077-verify-base (PR #272 head plus the verify doc fixes; src identical to cf8b252). Semver 202 of 202 on both runs; 12/12 corpus, 764 passed; SDK gain step reproduced; package 19 files, identical to 0.2.2's list. Fixed: a missing behaviour change (malformed or more than 16 GainMaps now refused), the headline's SDK claim scoped to the gain step, the vcs-info curl line restored, the README rewrap. The tag must go on a merge that includes c69c813.
