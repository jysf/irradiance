# SPEC-095 — build prompt

You're the implementer for a spec-driven workflow. Cycle: build (Phase 1 of a release:
the reversible prep PR). You never tag or publish.

Read files in order, before changing anything:

1. /AGENTS.md — conventions you must follow, including "During ship" for releases.
2. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-197-build-irradiance-0-2-7-release-the-public-dng-writer.md — your handoff. Follow its "Context the Receiving Agent Needs" section exactly.
3. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md — the project context.
4. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-032-the-0-2-7-release-the-public-dng-writer.md — the stage.
5. Every decision listed in the handoff (DEC-006, DEC-028, DEC-067, DEC-055, DEC-071, DEC-080, DEC-082).
6. /guidance/constraints.yaml.
7. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md — the spec. Its "Build work" steps 1 to 11 are your task list.
8. /projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md — the 0.2.6 release, your model for the notes, the internal record and the pre-flight.

Cut `release/0.2.7-prep` from an up-to-date `origin/main`, after step 1's check. Set
`IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` as the handoff says. Use
`/usr/bin/git` to establish branch and HEAD.

Before starting, mark the build cycle `[~]` in /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer-timeline.md. If you hit something that
needs architect judgment or an external unblock (a `src/` change since `153f517`, a
semver break, a writer defect in the packaged-crate check), change it to `[?]` with a
one-line reason and stop.

Do:
- The 11 build steps in order. No `src/`, test or fuzz change: a defect is reported,
  not fixed.
- Scratch crates and unpacked packages go in your session's scratchpad, never the repo.
- If the spec is ambiguous, STOP and ask. Don't guess.

When done:
1. Fill the handoff's `handback:` front matter (a REAL `tokens_total`, measured from
   your transcript per docs/cost-tracking.md) and its `## Handback` section, including
   the three reflection questions and anything in the spec the evidence contradicted.
2. Set `handoff.status` and `handback.status` to `completed`.
3. Run: just advance-cycle SPEC-095 verify
4. Run every gate in step 11 (including `just lint-fuzz-red-proof`) and, once the PR is
   open, confirm `gh pr checks` is all green, 23 of 23, before reporting.
5. Open the PR from `release/0.2.7-prep` against `main`. Its description references
   PROJ-005, STAGE-032, SPEC-095, HANDOFF-197 and the decisions used. Commit messages are
   plain prose only (the repo ruleset rejects code tokens).
6. Mark build `[x]` in the timeline with the PR number, cost and date.
