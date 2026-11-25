# SPEC-081 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify. You did not build this;
review it cold.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-192-verify-exif-block-and-xmp-placement-at-a-fixed-base-offset.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-191-build-exif-block-and-xmp-placement-at-a-fixed-base-offset.md — the build's handback (deviations and follow-ups).
4. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md — the stage.
5. Every decision listed in the handoff, including DEC-081 (proposed).
6. /guidance/constraints.yaml — constraints for the paths under review.
7. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-081-exif-block-and-xmp-placement-at-a-fixed-base-offset.md — the spec. Judge against the version at `f5c1b9c` (as designed), not the build's edits.

Work on `claude/spec-081-verify-base` (PR 332's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-081-exif-block-and-xmp-placement-at-a-fixed-base-offset-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every acceptance criterion, with evidence. Re-run the gates yourself; do not trust the
  build's numbers.
- Mutate the writer and show each mutant fails a named test.
- Read `gh pr checks 332` first, then rule on the three build deviations and on DEC-081, as
  the handoff asks.
- Fixes on your branch are tests or docs only. `src/` changes go in a punch list.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-081 ship --verdict approved` and
   `just handback-sync SPEC-081` (CI's cost audit needs both in the same push), push
   `claude/spec-081-verify-base`, and fast-forward `claude/spec-081-build` to it so PR 332
   carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
