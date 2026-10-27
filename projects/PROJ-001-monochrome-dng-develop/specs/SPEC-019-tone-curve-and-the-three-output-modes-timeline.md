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
- [~] **verify** — `HANDOFF-053`, delegated to a separate session on a worktree
  cut from the PR branch. Cold review of PR #18 vs the 10 ACs; the one check the
  orchestrator could not do from outside — actually forcing the oracle red — is
  the verifier's required step. Returns one verdict; no fixes.
- [ ] **ship** — merge, CHANGELOG, `cost.totals`, `just cost-audit`, Reflection,
  follow-up disposition; this closes STAGE-003's spec backlog → stage reflection.
