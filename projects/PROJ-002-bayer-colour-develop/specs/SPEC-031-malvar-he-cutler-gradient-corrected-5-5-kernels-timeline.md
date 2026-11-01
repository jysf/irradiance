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
      verify; PR to `main`.
- [ ] **verify** — cold independent review after build.
- [ ] **ship** — main session (reflection, cost totals, `complexity_actual`,
      archive, brag).
