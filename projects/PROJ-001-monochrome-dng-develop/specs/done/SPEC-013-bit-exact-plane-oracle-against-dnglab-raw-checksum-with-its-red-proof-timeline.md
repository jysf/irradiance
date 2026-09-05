# SPEC-013 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-013-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-04, main loop. The plane is already bit-exact (4/4,
  verified twice), so this spec is about making the assertion permanent and
  **provably able to fail**. The probe's most useful result was a mistake it
  made: an injected fault that changed the file, compiled, and was a semantic
  **no-op** — so `AC4` now requires asserting the **output** changed, not just
  the file. ⚠ A genuine faulty digest was **not** obtained (two runs killed by
  timeouts on a 95 MB plane) and is deliberately not quoted. Six ACs, six
  failing tests. `HANDOFF-030` ready.
- [x] **build** — 2026-09-04, `HANDOFF-030`, CLI session. All six ACs met; MD5
  from RFC 1321 (all 7 vectors pass); oracle green on all 4 decodable files;
  red-proof measured red (`59b032fe4320a27989ce61f3e3da7ff2` vs. honest
  `cb653b5bec24d166eef2fd258ee61ac4`) with a working negative control;
  `DEC-017` records the red-proof mechanism and the rejected first attempt.
  Eleven gates + `lint-ci` green locally; branch pushed, CI pending/observed
  (see Handback).
- [ ] **verify** — handoff written after build hands back.
- [ ] **ship**
