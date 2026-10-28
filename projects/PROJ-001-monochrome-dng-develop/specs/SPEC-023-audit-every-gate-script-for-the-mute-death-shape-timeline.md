# SPEC-023 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-023-<cycle>.md`.

## Cycles

- [x] **design** — 2026-10-28, main loop (orchestrator). STAGE-005's capstone,
  generalising SPEC-016's one-gate fix. A read-only Explore subagent surveyed all
  50 gate scripts and re-measured the mute-death shape: 3238dcb's "8 of 13" is
  down to **3 reachable offenders** (`validate.sh`, `handback-sync.sh`,
  `decisions-audit.sh`) + 3 lower-risk structural variants; recommended a bespoke
  `gate-audit` over shellcheck (which misses this class). AC1–AC5 written. Scaffolded
  as SPEC-901 (next_id scanned the SPEC-900 test fixture) and hand-renamed to 023;
  that next_id bug recorded as a signal (out of scope).
- [~] **build** — `HANDOFF-062`. `scripts/gate-audit.sh` (comment-excluding,
  scan-count-asserting) + `scripts/gate-audit-red-proof.sh` (mirror
  `validate-red-proof.sh`); `app.just` + AGENTS.md §6; two paired CI jobs. Fix the
  3 reachable offenders (incl. SPEC-016's two follow-ups; de-kludge HANDOFF-038)
  + any variant the audit flags. 0 `src/` change. Delegated to a cold session.
- [ ] **verify** — cold, independent: confirm the audit excludes comments and
  asserts its scan count; run the red-proof (control clean → injected shape caught
  by name → isolation); confirm the three fixes behaviorally (validate goes red on
  a non-SyntaxError fault; a superseded handoff is skipped; HANDOFF-038 syncs clean
  with handback null); 0 `src/` change. Separate session.
- [ ] **ship** — CHANGELOG, cost totals + `just cost-audit`, reflection, archive;
  close the `handback-sync-trips-on-superseded-handoffs` signal.
