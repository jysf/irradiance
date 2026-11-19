# SPEC-011 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-011-<cycle>.md`.

## Cycles

- [x] **design** — 2026-10-27, main loop (orchestrator). Framed 2026-08-22
  (PATCH-001/FU-4); designed here. A read-only Explore subagent gathered the
  inputs and ran clippy on `fuzz/` live: the crate is already clippy-clean (0
  warnings on 0.1.98 and 0.1.99, even under the five panic-free lints), so the
  gate ships green and the red-proof fault must be a default-clippy violation.
  Settled the open scope question: default `-D warnings`, NOT the five panic-free
  lints (DEC-027). AC1–AC5 written.
- [~] **build** — `HANDOFF-060`. Add `just lint-fuzz` (`--manifest-path
  fuzz/Cargo.toml --all-targets -- -D warnings`, mirror `deny-fuzz`) + a CI job
  (mirror `licenses-fuzz`); `scripts/lint-fuzz-red-proof.sh` (mirror
  `lint-red-proof.sh`/DEC-009, one default-clippy fault) + its CI job; DEC-027
  (scope decision, extends DEC-011) + INDEX; AGENTS.md §6. No `src/` or
  `fuzz_targets/` change. Delegated to a cold session.
- [ ] **verify** — cold, independent: confirm the gate runs clippy on the fuzz
  graph and is green; the red-proof catches an injected fault with a clean
  control; DEC-027 records the scope with the counter-argument; 0 `src/` change.
  Separate session.
- [ ] **ship** — CHANGELOG, cost totals + `just cost-audit`, reflection, archive.
