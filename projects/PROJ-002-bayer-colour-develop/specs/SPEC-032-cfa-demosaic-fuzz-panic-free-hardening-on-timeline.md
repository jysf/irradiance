# SPEC-032 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-032-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-01 on `claude/spec-032-fuzz` (main session).
      The last STAGE-007 spec: a `demosaic` libFuzzer target (mirrors
      `fuzz/fuzz_targets/develop.rs`) + seeds, GRBG/GBRG bit-exact oracle fixtures
      (closes SPEC-030/FU-1 — all four Bayer phases now covered), DC-preservation on
      all four phases, and a tier-A panic-free-on-hostile-CFA test. No `src` change
      (HA is already phase-driven + panic-free). Failing tests enumerated; `cycle: design`.
- [ ] **build** — dispatch to a cold session once the design PR merges.
- [ ] **verify** — cold independent review after build.
- [ ] **ship** — main session (reflection, cost totals, `complexity_actual`, archive,
      brag). Likely TEST-ONLY → no CHANGELOG (confirm). STAGE-007 then fully shipped →
      run the STAGE-007 stage-ship (Prompt 1d).
