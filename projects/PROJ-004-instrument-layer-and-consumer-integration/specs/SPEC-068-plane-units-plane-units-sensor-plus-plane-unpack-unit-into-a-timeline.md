# SPEC-068 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-068-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-13 (DEC-063: a unit is a full-width row band, so callers split dst with split_at_mut, no unsafe; probe: Q2M row bands 44.7 ms single -> 6.1 ms at 8 threads, byte-identical; crate kernel 238 ms vs probe 44.7 ms = SPEC-069 evidence; HANDOFF-143 build ready)
- [ ] **build** — HANDOFF-143
- [x] **verify** — completed 2026-11-13 (HANDOFF-144, APPROVED at d555dea, no code changes; just test 690/0/1 with 12/12 corpus, plane oracles ran with no SKIP, every gate green, fuzz-plane 3.17M runs clean, CI 44/44; 12/12 tier-A mutants red incl. the build's 8; kernels are the old bodies moved; old vs new unpack_into identical on the zero-width tiled input and in Compression 7 allocation counts, so AC6 holds as no new allocation; XS follow-up: pin the absolute Compression 7 allocation count)
- [ ] **ship**
