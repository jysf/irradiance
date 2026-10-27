# SPEC-022 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-022-<cycle>.md`.

## Cycles

- [x] **design** — spec authored (orchestrator, 2026-10-27). Fixture pinned via
  the optical-centre fixed-point trick; AC1–AC4 (incl. the warp-arm red-proof);
  two failing tests specified. Recipe gathered by a read-only Explore subagent.
- [ ] **build** — `HANDOFF-054`. Write `develop_warp_branch_develops_the_fixed_plane`
  and `develop_warp_branch_actually_warps` in `tests/develop.rs`; discharge the
  red-proof (sever the warp arm's `effective_src → src` → new test RED, AC9
  GREEN → revert byte-identical). No `src/` change. Delegated to a separate session.
- [ ] **verify** — cold, independent: re-run the red-proof on the shipped file,
  confirm the new test goes red under the sever while AC9 stays green. Separate
  session.
- [ ] **ship** — CHANGELOG, cost totals + `just cost-audit`, reflection, archive.
