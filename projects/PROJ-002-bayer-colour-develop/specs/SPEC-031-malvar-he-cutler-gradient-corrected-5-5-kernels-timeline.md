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
- [ ] **build** — dispatch to a cold session once the design PR merges. ⚠ L-risk:
      ship green first (R/B bilinear), then R/B colour-difference, if full HA > [M].
- [ ] **verify** — cold independent review after build.
- [ ] **ship** — main session (reflection, cost totals, `complexity_actual`,
      archive, brag).
