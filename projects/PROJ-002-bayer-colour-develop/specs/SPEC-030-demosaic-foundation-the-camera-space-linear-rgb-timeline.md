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
- [x] **build** — complete 2026-10-31 on `claude/spec-030-demosaic-568a93` (cold session,
      HANDOFF-074, model claude-opus-4-8). `src/demosaic.rs` (bilinear, phase-driven, reflect
      border), `tests/demosaic_oracle.rs` (bit-exact RGGB/BGGR fixtures + perturbed-kernel
      red-proof + no-panic/typed-error + `irr demosaic` P6 smoke), `irr demosaic` → P6 PPM.
      All four gates green (`just test`, `just lint-ci`, `cargo fmt --check`, `just validate`).
      MHC is SPEC-031 against this same harness.
- [ ] **verify** — cold independent review after build.
- [ ] **ship** — main session (reflection, cost totals, `complexity_actual`,
      archive, brag).
