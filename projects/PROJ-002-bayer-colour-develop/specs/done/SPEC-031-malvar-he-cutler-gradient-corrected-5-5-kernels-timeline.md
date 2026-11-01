# SPEC-031 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-031-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-10-31 on `claude/spec-031-mhc` (main session).
      Re-scoped MHC → **Hamilton-Adams** (DEC-031 — MHC is patent-encumbered, US
      7,502,505 active to ~2027-01-25; HA US 5,629,734 expired). Directional green
      (gradient-chosen axis + 2nd-difference correction) + colour-difference R/B,
      signed i32, round-half-up, clamp [0,65535], two-pass (green first), offset-2
      reflect; bit-exact analytic oracle + DC-preservation + directionality fixture
      + red-proof, on SPEC-030's harness. Failing tests enumerated; `cycle: design`.
- [x] **build** — completed 2026-10-31 on `claude/spec-031-hamilton-adams-56ecec`
      (cold session, claude-opus-4-8; HANDOFF-076). **Full Hamilton-Adams shipped
      (green + R/B), not the green-only fallback** — it fit as one [M] build. Two-pass
      kernel in `src/demosaic.rs` (green held in `dst`'s own green slots, zero-alloc —
      DEC-032); bit-exact oracle extended in `tests/demosaic_oracle.rs` (6×6 HA table,
      directionality, classifier tie, DC-preservation, BGGR, red-proof mutant≠honest);
      provenance row + CHANGELOG. Gates green: `just test`, `just lint-ci`,
      `cargo fmt --check`, `just validate`. Oracle is Tier-A (no corpus). Advanced to
      verify; PR #67 → `main`.
- [x] **verify** — cold independent review after build, completed 2026-10-31 on
      `claude/spec-031-verify-311cc7` (cold session, claude-opus-4-8; HANDOFF-077).
      **✅ APPROVED — ship at commit `5450c70`.** Reproduced all gates in-worktree
      (`just test` 16/16 binaries, demosaic_oracle 22/22, tier-B SKIPs loud;
      `just lint-ci` pinned clippy 0.1.98 clean; `cargo fmt --check`; `just
      validate`; `decisions-audit --changed` clean). Load-bearing red-proof
      independently re-proven by breaking the REAL kernel three ways (pass-1 green
      predictor, pass-2 diagonal, pass-2 green-site colour-difference) — each
      turned the oracle red. Hand-recomputed pixel (2,2) incl. a nested classifier
      tie → matches `EXPECTED_HA`. DEC-032 green-slot aliasing audited sound. No
      regression (every STAGE-006 oracle green). Verify `tokens_total: null`
      (desktop `/cost` unavailable — RECOVER from the verify transcript at ship,
      UUID `21f3a4a2-4f9e-4e86-9dba-37e22e208e34`). Follow-up: SPEC-030/FU-1
      (GRBG/GBRG fixture) still open, deferred to SPEC-032 — not a blocker.
      Verify PR #68 (build + verify together, because #67 is unmerged) → `main`;
      merging #68 lands the whole spec and #67 closes as included.
- [x] **ship** — completed 2026-10-31 (main session). Build+verify merged via PR #68
      (#67 closed as superseded). Reflection + recovered cost totals (build
      14,325,716/$34.69 + verify 9,247,175/$20.81 = 23,572,891/$55.50, both recovered
      from the cold sessions' transcripts since handbacks were null), `complexity_actual:
      M`, archive, brag. CHANGELOG written at build (confirmed). `verify_verdict: approved`.
