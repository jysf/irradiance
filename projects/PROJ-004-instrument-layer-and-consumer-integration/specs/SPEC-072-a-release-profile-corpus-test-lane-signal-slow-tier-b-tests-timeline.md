# SPEC-072 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-072-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-16 (DEC-066: a [profile.corpus] inheriting test with opt-level 3, so assertions and overflow checks stay on; just test runs every test under it, just test-debug keeps the old run, CI unchanged; probe: full corpus suite 157 s against about 37 min in debug, same 697/0/1, every pin held; target <= 5 min; cut after SPEC-071 merges; HANDOFF-150 build ready)
- [x] **build** — completed 2026-11-16 (HANDOFF-150, PR #249; [profile.corpus] inherits test with opt-level 3; just test 144.7 s after touch src/lib.rs (159.1 s cold) against just test-debug 2290.3 s, both 697/0/1 with 0 SKIP; -- --list diff empty; assertions and overflow checks proved on under corpus; CI comment only; all 10 gates green; floor 2,603,595 tok / $2.35)
- [x] **verify** — completed 2026-11-16 (HANDOFF-152, PUNCH LIST fixed by the verifier in PR #250 superseding #249: [profile.corpus] moved above [dependencies] because deny-red-proof requires [dev-dependencies] last (CI red on #249); *_alloc zero-count tests documented debug-authoritative after an unused vec! was shown elided under corpus; just test 139 s vs just test-debug 2261 s, both 697/0/1, 0 SKIP; --list diff empty; AC2 reproduced; 10 gates green; floor 5,607,703 tok / $4.22)
- [ ] **ship**
