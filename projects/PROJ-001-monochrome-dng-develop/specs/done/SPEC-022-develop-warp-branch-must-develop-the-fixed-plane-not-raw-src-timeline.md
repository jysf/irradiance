# SPEC-022 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-022-<cycle>.md`.

## Cycles

- [x] **design** — spec authored (orchestrator, 2026-10-27). Fixture pinned via
  the optical-centre fixed-point trick; AC1–AC4 (incl. the warp-arm red-proof);
  two failing tests specified. Recipe gathered by a read-only Explore subagent.
- [x] **build** — `HANDOFF-054` (build session, 2026-10-27, PR #21). Added
  `develop_warp_branch_develops_the_fixed_plane` and
  `develop_warp_branch_actually_warps` to `tests/develop.rs`, both green on the
  honest tree. Red-proof discharged in a scratchpad copy: severing the warp arm's
  `effective_src → src` turned the new test RED (`developed[12]: left 0,
  right 1000`) while AC9 stayed GREEN; revert byte-identical (md5
  `ab3d7712…` → `3f0689bc…` → `ab3d7712…`); `--no-fail-fast` suite 243/0/3 →
  241/2/3. No `src/` change. All eight gates green locally.
- [ ] **verify** — cold, independent: re-run the red-proof on the shipped file,
  confirm the new test goes red under the sever while AC9 stays green. Separate
  session.
- [ ] **ship** — CHANGELOG, cost totals + `just cost-audit`, reflection, archive.
