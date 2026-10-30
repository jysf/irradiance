# SPEC-024 — BUILD prompt

> Cycle: **build**. Delegated via `HANDOFF-064`. Based on
> `FIRST_SESSION_PROMPTS.md` Prompt 3.

You're the implementer for a spec-driven workflow. Cycle: build.

Read, in order, before writing code:

1. `/AGENTS.md` — conventions you must follow (esp. §11 ship-the-reader, §12
   testing bars, §15 During build).
2. `/projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-064-build-*.md` —
   your handoff. Follow its "Context the Receiving Agent Needs" exactly.
3. `/projects/PROJ-002-bayer-colour-develop/brief.md` — the project.
4. `/projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-*.md` — the stage.
5. `DEC-012` (the one decision the handoff references).
6. `/guidance/constraints.yaml` — constraints for `src/ifd.rs`, `src/lib.rs`.
7. The spec (`specs/SPEC-024-*.md`) — Acceptance Criteria + the 7 Failing Tests
   are the contract.

Implement: make the 7 Failing Tests pass without redesigning; keep the Q2M mono
tests/oracles green (the regression guard). Don't violate constraints.

When done: fill `HANDOFF-064`'s `handback:` block (real `tokens_total`) and its
`## Handback` section, `just advance-cycle SPEC-024 verify`, `just handback-sync
SPEC-024`, open a PR to `main`, and mark build `[x]` in the timeline with the PR
number, cost and date.
