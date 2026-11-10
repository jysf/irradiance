# SPEC-055 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-055-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — full design against the current tree;
  DEC-048 minted; cargo-deny 0.19.9 probed (dev-deps resolve but are absent from the
  gate's graph; `exclude-dev=false` and an empty `[workspace]` both failed to surface
  them — recorded so build does not repeat them); `irr` disposition chosen
  (`required-features`); HANDOFF-111 (build) prepared. Spec moved `frame → build`.
- [~] **build** — delegated via HANDOFF-111. Branch `claude/spec-055-build-<suffix>`
  from freshly-merged `main`. Fix the dev-graph evaluation + write the mutation
  red-proof (the load-bearing deliverable), feature-gate `irr` + target-list check,
  reconcile the policy doc, run the gates, open a PR, fill the handback.
- [ ] **verify** — cold review: red-proof bites on root + fuzz, default build has no
  `irr`, oracle test intact under `--all-features`.
- [ ] **ship** — merge, recover true build/verify cost, brag, stage bookkeeping.
