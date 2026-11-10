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
- [x] **build** — delegated via HANDOFF-111. Branch `claude/spec-055-build-<suffix>`
  from freshly-merged `main`. Fix the dev-graph evaluation + write the mutation
  red-proof (the load-bearing deliverable), feature-gate `irr` + target-list check,
  reconcile the policy doc, run the gates, open a PR, fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ⚠ PUNCH LIST at 0e975ae; licence gate + irr gating correct, but the new CI red-proof job was red from two script bugs (CI colour codes broke the grep; grep -q under pipefail killed printf mid-write). Both fixed (--color never; here-string reads), independent mutation proof green on cargo-deny 0.19.9 + 0.20.2, irr stays out of the package. Shipped via the rebased #174. — cold review: red-proof bites on root + fuzz, default build has no
  `irr`, oracle test intact under `--all-features`.
- [x] **ship** (2026-11-10) — group D merged via PR #174 (rebased verify superset of #168/#172). Cost 17,200,000 tok / $12.06 (build 9.80M + verify 7.40M, handback-sync). Brag logged; STAGE-010 backlog updated.
