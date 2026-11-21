# SPEC-061 — build prompt

You're the implementer for a spec-driven workflow. Cycle: build.

Read files in order, before writing code:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/handoffs/HANDOFF-175-build-public-write-feature-sealed-api-over-the-stage-013-core-call.md — your handoff. Follow its "Context the Receiving Agent Needs" section exactly.
3. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md — the project context.
4. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md — the stage.
5. Every decision listed in the handoff (DEC-071, DEC-072, DEC-073, DEC-074, DEC-075,
   DEC-055, DEC-050, DEC-016, DEC-051, DEC-052, DEC-058, DEC-069).
6. /guidance/constraints.yaml — constraints for the paths you'll touch.
7. /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call.md — the spec.

Cut your branch `claude/spec-061-build` from an up-to-date `main`. Set
`IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` as the handoff says.

Before coding, mark the build cycle `[~]` in /projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Implement:
- Make the failing tests pass. The probe table in the spec's Implementation Context is
  measured; trust it over priors.
- Don't violate constraints. If one needs breaking, STOP and ask.
- For non-trivial decisions not already in DEC-075, create /decisions/DEC-NNN-<slug>.md
  with an honest confidence, and run `just decisions-index`.
- If the spec is ambiguous, STOP and ask. Don't guess.

When done:
1. Fill the handoff's `handback:` front matter (a REAL `tokens_total`, measured from
   your transcript per docs/cost-tracking.md) and its `## Handback` section, including
   the three reflection questions and the dng_validate results.
2. Set `handoff.status` and `handback.status` to `completed`.
3. Run: just advance-cycle SPEC-061 verify
4. Open the PR from `claude/spec-061-build` against `main`. Its description references
   PROJ-005, STAGE-017, SPEC-061, HANDOFF-175, the decisions used, the constraints
   checked and any new DEC. Commit messages are plain prose only.
5. Mark build `[x]` in the timeline with the PR number, cost and date.
