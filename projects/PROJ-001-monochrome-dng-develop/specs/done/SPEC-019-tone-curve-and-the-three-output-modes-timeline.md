# SPEC-019 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-019-<cycle>.md`.

## Instructions

- [x] **design** — 2026-10-26, main loop (not separately metered). Scope
  grounded in a real-file probe (`exiftool` on `L1026192.DNG`): the Q2M carries
  no profile tone curve, so the curve is the sRGB OETF. Chose an additive
  encode layer over `develop_into` (`DEC-018` not reopened). Spec + 10 ACs +
  Failing Tests written; `HANDOFF-052` (build) minted; `prompts/SPEC-019-build.md`
  written. New `DEC-025` (tone/output model) pre-specified for build to emit.
- [x] **build** — 2026-10-26, `HANDOFF-052`. PR
  [#18](https://github.com/jysf/irradiance/pull/18) against `main` from
  `claude/spec-019-tone-curve-13488a`. All 10 ACs met: `src/tone.rs` (three
  encoders) + committed generated `src/tone_table.rs` + `examples/gen-tone-table.rs`
  + `just gen-tone-table` + `tests/tone_oracle.rs` (analytic ±1 oracle, red-proof
  observed red with green control) + `irr develop --format` reader + provenance
  row + `DEC-025`. `just lint-ci`/`lint-no-allow`/`test`/`deny`/`deny-fuzz`/`msrv`
  green locally; CI observed on the shipping SHA. Build cost ~41.3M tokens
  (handback). 10 tone tests confirmed run (5 oracle + 5 unit).
- [x] **verify** — 2026-10-27, `HANDOFF-053`, separate session `spec-019-verify`
  (branch `claude/spec-019-verify-4d9fa5`). ✅ **APPROVED** at `46b5f78`, no
  findings / no punch list. All AC1–AC10 verified cold (each AC → a test
  confirmed to run); gates exit 0 locally; CI 22/0 on the SHA; oracle red-proof
  forced independently then reverted byte-identical; `develop_into`/`plane`/`ifd`
  0 lines changed (DEC-018 intact); DEC-025 present/honest. Verify cost
  16,131,777 tokens (~$41.43), transcribed to HANDOFF-053's handback by the
  orchestrator. Reconciled against PR #18.
- [x] **ship** — 2026-10-27. PR #18 squash-merged to `main` (`5000da0`).
  CHANGELOG `[Unreleased] → Added`; `cost.totals` recomputed (design null +
  build 41.3M + verify 16.1M); `just cost-audit` + `just validate` green; spec
  Reflection written; archived to `specs/done/`. Last spec in STAGE-003 → stage
  reflection written and STAGE-003 marked shipped.
