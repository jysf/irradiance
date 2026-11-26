# SPEC-092 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify. You did not build this;
review it cold.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-203-verify-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-202-build-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md — the build's handback (mutant table, deviations and follow-ups).
4. /projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md — the stage.
5. Every decision listed in the handoff, DEC-083 and DEC-079 first.
6. /guidance/constraints.yaml — constraints for the paths under review.
7. /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md — the spec. Judge against the version on `origin/main` after PR 353 (as designed), not the build's edits.

Work on `claude/spec-092-verify` (PR 354's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every acceptance criterion, with evidence. Re-run the gates yourself; do not trust the
  build's numbers.
- Re-run a sample of the build's mutants (at least (e), (h), (i), (j), (k), one (l), one
  (m) and (r)) and the handoff's extra mutants; show each fails a named test. Check the
  argued equivalents (t) and (u).
- Read `gh pr checks 354` and CI run 45855649150's determinism logs first. Then rule on
  the shared-sink generalisation, the four build deviations, and the band path's
  allocations, as the handoff asks.
- Fixes on your branch are tests or docs only. `src/` changes go in a punch list.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-092 ship --verdict approved` and
   `just handback-sync SPEC-092` (CI's cost audit needs both in the same push), push
   `claude/spec-092-verify`, and fast-forward `feat/spec-092-mono-mixer` to it so PR 354
   carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
