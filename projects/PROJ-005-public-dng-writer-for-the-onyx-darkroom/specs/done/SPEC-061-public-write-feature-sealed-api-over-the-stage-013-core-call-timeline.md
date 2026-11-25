# SPEC-061 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-061-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-11-20 (PR 284; DEC-072 amended in PR 291)
- [x] **design** — completed 2026-11-21
- [x] **build** — completed 2026-11-24 (PR 329; HANDOFF-175; 21,952,535 tokens, about $16.56, floor)
- [x] **verify** — 2026-11-24, ⚠ PUNCH LIST round 1 (HANDOFF-190; 10,145,002 tokens, about $7.58, floor). P1: `scripts/lint-fuzz-red-proof.sh` must copy `tests/support/dng_write_fuzz.rs`; CI red-proof job is red on PR 329. ✅ APPROVED round 2 at 15ed4fc (orchestrator re-verify)
- [x] **ship** — 2026-11-24 (PR 329 merged as d8a8bbb; DEC-080 accepted, DEC-075 §5 edge limit, oracle 8 corrected; 35,681,088 tokens / about $26.09 across design, build x2, verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).
