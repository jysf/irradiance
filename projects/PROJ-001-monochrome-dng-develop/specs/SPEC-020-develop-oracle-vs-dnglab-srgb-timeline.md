# SPEC-020 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-020-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06, main-loop, not separately metered.
  Cycle transitioned frame → design. Spec body drafted from the
  frame stub: eleven acceptance criteria, eight named failing tests
  (all tier A, all runnable without corpus), one required DEC for
  the `ssimulacra2 = "0.5.1"` dev-dep sanction, one provenance-ledger
  row, and a `## Implementation Context` block splitting design-time
  probes into "confirmed", "required before build handoff", and
  "required in build (pre-registered)". `depends_on: []`; the spec
  lands **before** SPEC-018 by construction — the tier-A red-proof
  perturbs a synthetic reference through the metric wiring, so it
  needs no develop pipeline. Precedent: SPEC-015's L1/L2/L3 shape
  and its FU-10 lesson (tier-A red-proof runs with corpus unset).
- [~] **build** — `HANDOFF-043` dispatched 2026-09-06 on branch
  `feat/spec-020-develop-oracle-vs-dnglab-srgb`. The design pass
  authorises the `ssimulacra2` dev-dep + its DEC to land in the same
  build pass (AGENTS.md §13 point 4), with the licence re-confirmed
  on the pinned tree before the DEC is written. Fixture choice is
  pre-registered in the spec (AC4/AC5/AC6 thresholds are read from
  DEC-005; a fixture that cannot satisfy them is a wrong fixture,
  not a wrong threshold). Return Criteria: eleven gates + `just
  lint-ci` observed green on the ship SHA, `src/` 0 lines changed,
  all four tier-A fault scores measured with corpus unset, `notes:`
  one physical line, do not open the PR or run `handback-sync`.
- [ ] **verify** — a separate agent runs the tier-A red-proof with
  `IRRADIANCE_CORPUS_DIR` unset (per AC7) and verifies each
  deliberate fault turns the score below 85, exactly the discipline
  DEC-004 rule 1 and constraint `oracle-must-be-shown-red` mandate.
- [ ] **ship** — CI observed green on the shipping SHA (AC11);
  provenance row landed (AC8); dev-dep sanction DEC referenced
  (AC9); Follow-ups table populated per AGENTS.md §15.
