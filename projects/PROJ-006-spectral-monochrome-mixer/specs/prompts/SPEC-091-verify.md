# SPEC-091 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify. You did not build this;
review it cold.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-198-verify-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-189-build-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md — the build's handback (deviations and follow-ups).
4. /projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md — the stage.
5. Every decision listed in the handoff, DEC-079 first.
6. /guidance/constraints.yaml — constraints for the paths under review.
7. /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md — the spec. Judge against the version at `3ecdad74` (as designed), not the build's edits.

Work on `claude/spec-091-verify-base` (PR 343's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every acceptance criterion, with evidence. Re-run the gates yourself; do not trust the
  build's numbers.
- Re-run mutants (a) to (k) and the handoff's extra mutants; show each fails a named test.
- Read `gh pr checks 343` and CI run 45666936351's determinism logs first. Then rule on the
  refactor of the existing colour entries, the four build deviations, and the band path's
  memory, as the handoff asks.
- Fixes on your branch are tests or docs only. `src/` changes go in a punch list.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-091 ship --verdict approved` and
   `just handback-sync SPEC-091` (CI's cost audit needs both in the same push), push
   `claude/spec-091-verify-base`, and fast-forward `feat/spec-091-srgb-linear-plane` to it so
   PR 343 carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
