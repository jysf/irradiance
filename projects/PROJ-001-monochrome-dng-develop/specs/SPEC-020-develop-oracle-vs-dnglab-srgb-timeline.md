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
- [x] **build** — `HANDOFF-043` complete, 2026-09-06, on branch
  `feat/spec-020-develop-oracle-vs-dnglab-srgb` @ `61e5eb4`
  (three commits: `2e0d2c4`, `db18c77`, `61e5eb4`). CI green on
  every SHA. 36,917,935 tokens on claude-sonnet-5 (corrected from
  the tier_map's opus-5 prediction), $15.23 estimated, ~40 min.
  Handback synced into `cost.sessions[build]` this session. Four
  measured fault scores match DEC-005's calibration shape:
  identical 100.000, 1-px shift 61.823, missing warp −82.338,
  gamma 1.05 90.038. Three FUs raised (FU-1..3), all proposed
  `closed:` in the handback; verify judges those. `src/` untouched
  as pre-registered (AC10). Dev-dep `ssimulacra2 = "0.5.1"` landed
  under DEC-023 with the licence corrected from the design's guess
  (BSD-3) to the measured value (BSD-2-Clause).
- [~] **verify** — `HANDOFF-045` dispatched 2026-09-06 on the same
  branch. Verifier runs the 8 generic + 4 repo-specific bars from
  §15, judges the three build-proposed `closed:` dispositions,
  confirms the tier-A red-proof turns red below 85 with
  `IRRADIANCE_CORPUS_DIR` unset (AC7), and returns
  `✅ APPROVED` / `⚠ PUNCH LIST` / `❌ REJECTED` with per-finding
  `SB-N` / `FU-N` labels. Does not open the PR (orchestrator's step
  after verdict).
- [ ] **ship** — CI observed green on the shipping SHA (AC11);
  provenance row landed (AC8); dev-dep sanction DEC referenced
  (AC9); Follow-ups table populated per AGENTS.md §15.
