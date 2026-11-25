# SPEC-094 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify. You did not build this;
review it cold.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-196-verify-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-195-build-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md — the build's handback (deviations and follow-ups).
4. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md — the stage.
5. Every decision listed in the handoff, including DEC-081 (accepted; its Follow-up is what item 1 resolves).
6. /guidance/constraints.yaml — constraints for the paths under review.
7. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md — the spec. Judge against the version at `6eab1b8` (as designed), not the build's edits.

Work on `claude/spec-094-verify-base` (PR 339's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every acceptance criterion, with evidence. Re-run the gates yourself; do not trust the
  build's numbers.
- Mutate the writer and show each mutant fails a named test.
- Read `gh pr checks 339` first, then rule on the three build deviations and on the doc-link form, as
  the handoff asks.
- Fixes on your branch are tests or docs only. `src/` and `.github/` changes go in a punch list.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-094 ship --verdict approved` and
   `just handback-sync SPEC-094` (CI's cost audit needs both in the same push), push
   `claude/spec-094-verify-base`, and fast-forward `claude/spec-094-build` to it so PR 339
   carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
