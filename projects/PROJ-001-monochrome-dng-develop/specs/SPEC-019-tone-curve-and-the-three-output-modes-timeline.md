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
- [x] **build** — `HANDOFF-052`, `claude-opus-5` (implementer). PR #18
  (branch `claude/spec-019-tone-curve-13488a`, head `46b5f78`), all 10 ACs
  reported met; CI observed green on head SHA (fmt, clippy -D, test, MSRV,
  both cargo-deny graphs, lint red-proof, no-#[allow], fuzz smokes).
  Orchestrator reconciled against the remote + `gh pr checks` 2026-10-26.
- [x] **verify** — `HANDOFF-053`, separate session `spec-019-verify` (branch
  `claude/spec-019-verify-4d9fa5`). ✅ **APPROVED** at `46b5f78`, no findings /
  no punch list. All AC1–AC10 verified cold (each AC → a test confirmed to run);
  gates exit 0 locally; CI 22/0 on the SHA; oracle red-proof forced
  independently then reverted byte-identical; `develop_into`/`plane`/`ifd`
  0 lines changed (DEC-018 intact); DEC-025 present/honest. Verify cost
  16,131,777 tokens (~$41.43), transcribed to HANDOFF-053's handback by the
  orchestrator. Reconciled against PR #18 (OPEN, MERGEABLE/CLEAN, head `46b5f78`).
- [ ] **ship** — NOT started (awaiting go-ahead; merge is irreversible). Merge
  PR #18 → main, CHANGELOG `[Unreleased] → Added`, `cost.totals` +
  `just cost-audit` (needs HANDOFF-052's build handback, which lands with the
  merge), spec Reflection, `just archive-spec SPEC-019`. This is the last spec in
  STAGE-003 → write the stage-level reflection and mark STAGE-003 shipped.
