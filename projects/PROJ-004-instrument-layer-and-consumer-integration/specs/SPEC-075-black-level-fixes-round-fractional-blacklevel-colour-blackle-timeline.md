# SPEC-075 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-075-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-19 (probe on 4c5dd2c: only the Pixel 7 Pro has a non-integral black, 262172/256 and so on, all rounding to 1024; no held file has a [1, 2] or [2, 1] pattern, more than one WhiteLevel, or MaskedAreas; writer extra_tags builds every fixture; the user chose rounding; DEC-068; HANDOFF-157 build ready)
- [x] **build** — HANDOFF-157, completed 2026-11-19 (PR 262; 729 passed, 0 failed with 12/12 corpus files; 17 legacy colour fixtures gained a [2, 2] repeat dim with orchestrator approval; a fourth version pin in warp_alpha)
- [x] **verify** — HANDOFF-159, completed 2026-11-19 (PUNCH LIST, tests-only: source at 365b55e correct; 729 passed, 0 failed with 12/12 corpus files; red-proofs for AC3, AC4 and AC5 reproduced; four surviving mutants (f32 general path origin, per-sample white, two MaskedAreas classes) pinned by tests in 3e5dd05 on claude/spec-075-verify-base)
- [ ] **ship**
