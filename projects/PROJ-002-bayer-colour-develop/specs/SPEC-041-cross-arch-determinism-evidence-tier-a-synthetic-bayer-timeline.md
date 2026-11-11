# SPEC-041 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-041-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-03) — ID reserved; scope captured.
- [x] **design** (orchestrator, 2026-11-10) — no new DEC (determinism rests on DEC-016/
  018/033, tier-B-out-of-CI on DEC-003). Approach: a `tests/determinism.rs` that builds
  tier-A synthetic Bayer fixtures, hashes the three outputs (`unpack_into`,
  `demosaic_into`, `to_linear_f32_into`) with an in-test FNV-1a (no new dep) against
  committed `const` hashes, run on a `ubuntu-latest` + `macos-latest` (arm64) CI matrix so
  cross-arch agreement is the evidence; float paths named (`to_linear_f32_into` = one IEEE
  division; the `f64` colour path named but unmeasured); oracle shown red (perturbed run
  fails). A found divergence becomes its own spec — no kernel change. Spec moved `frame →
  build`; HANDOFF-120 prepared.
- [x] **build** (claude-opus-5-5 cold, PR #181 → rebased #183 merged) — delegated via HANDOFF-120. Branch `claude/spec-041-build-29eaff` from
  freshly-merged `main`. Committed hashes + the two-OS matrix + the red-proof + the
  float-path doc; gates; PR; fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ✅ APPROVED at f307abe; 514 tests, both CI determinism legs ran on their real arches (ubuntu x86_64 + macos arm64), the red-proof bites, hashes reproduced locally; no PR, no change. Confirmed: the matrix runs on BOTH arches, hashes asserted on both,
  the red-proof bites, float paths named honestly, no kernel changed to force agreement.
- [x] **ship** (2026-11-10) — code merged via PR #183 (rebased superset of #181). Cost 9,525,484 tok / $6.99 (build 7.21M/$5.16 + verify 2.31M/$1.83). brag logged; STAGE-010 → 7 shipped / 1 active.
