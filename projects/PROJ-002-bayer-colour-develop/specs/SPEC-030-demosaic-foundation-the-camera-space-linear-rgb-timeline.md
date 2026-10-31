# SPEC-030 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-030-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-10-31 on `claude/stage-007-specs` (main session).
      Bilinear baseline + the camera-space linear RGB buffer convention (caller-owned
      interleaved `&mut [u16]`, DEC-016) + the bit-exact analytic-fixture oracle
      (+ perturbed-kernel red-proof, DEC-017) + `irr demosaic` P6 PPM. Border policy
      = phase-preserving reflect; round-half-up integer averages. Failing tests
      enumerated; `cycle: design`, build-ready.
- [ ] **build** — dispatch to a cold session (HANDOFF) once the design PR merges;
      MHC is SPEC-031 against this same harness.
- [ ] **verify** — cold independent review after build.
- [ ] **ship** — main session (reflection, cost totals, `complexity_actual`,
      archive, brag).
