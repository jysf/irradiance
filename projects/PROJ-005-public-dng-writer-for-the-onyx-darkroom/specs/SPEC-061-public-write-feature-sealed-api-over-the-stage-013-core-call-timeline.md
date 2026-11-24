# SPEC-061 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-061-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-11-20 (PR 284; DEC-072 amended in PR 291)
- [x] **design** — completed 2026-11-21
- [x] **build** — completed 2026-11-24 (PR 329; HANDOFF-175; 21,952,535 tokens, about $16.56, floor)
- [x] **verify** — 2026-11-24, ⚠ PUNCH LIST round 1 (HANDOFF-190; 10,145,002 tokens, about $7.58, floor). P1: `scripts/lint-fuzz-red-proof.sh` must copy `tests/support/dng_write_fuzz.rs`; CI red-proof job is red on PR 329. ✅ APPROVED round 2 at 15ed4fc (orchestrator re-verify)
- [ ] **ship** — prompt: pending (verify approved at round 2: P1 fixed in 15ed4fc, CI 22/22, re-verified by the orchestrator 2026-11-24)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).
