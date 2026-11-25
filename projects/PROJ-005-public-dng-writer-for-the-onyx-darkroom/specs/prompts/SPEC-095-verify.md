# SPEC-095 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify (the cold review of a
release's Phase 1 prep). You did not build this; review it cold. Never tag or publish.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow, including "During ship" for releases.
2. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-199-verify-irradiance-0-2-7-release-the-public-dng-writer.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-197-build-irradiance-0-2-7-release-the-public-dng-writer.md — the build's handback (deviations).
4. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-032-the-0-2-7-release-the-public-dng-writer.md — the stage.
5. Every decision listed in the handoff.
6. /guidance/constraints.yaml.
7. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md — the spec. Judge against the version at `3f9df0b` (as designed); review the build's Release Notes, internal record and Pre-Flight on the branch.
8. /projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md and its verify handoff HANDOFF-188 — the model.

Work on `claude/spec-095-verify-base` (PR 345's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer-timeline.md. If you hit something that
needs architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every item in "What to check hard", with evidence. Re-run the gates yourself; do not
  trust the build's numbers.
- Your own packaged-crate consumer, on a different held monochrome DNG than the build's.
- Read `gh pr checks 345` first, then rule on the build's three deviations.
- Fixes on your branch are wording and records only. `src/`, tests, fuzz, `.github/` and
  `Cargo.toml` changes go in a punch list.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-095 ship --verdict approved` and
   `just handback-sync SPEC-095` (CI's cost audit needs both in the same push), push
   `claude/spec-095-verify-base`, and fast-forward `release/0.2.7-prep` to it so PR 345
   carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
