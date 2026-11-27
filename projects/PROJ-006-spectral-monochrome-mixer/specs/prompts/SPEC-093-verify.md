# SPEC-093 — verify prompt

You're the verifier for a spec-driven workflow. Cycle: verify. You did not build this;
review it cold.

Read files in order, before running anything:

1. /AGENTS.md — conventions you must follow.
2. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-205-verify-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md — your handoff. Follow its "Context the Receiving Agent Needs" and "What to check hard" sections exactly.
3. /projects/PROJ-006-spectral-monochrome-mixer/handoffs/HANDOFF-204-build-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md — the build's handback (mutant table, deviations and follow-ups).
4. /projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md — the stage.
5. Every decision listed in the handoff, DEC-084, DEC-083 and DEC-079 first.
6. /guidance/constraints.yaml — constraints for the paths under review.
7. /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md — the spec. Judge against the version on `origin/main` after PR 356 (as designed, including "onyx's reply (2026-11-26)"), not the build's edits.

Work on `claude/spec-093-verify` (PR 358's head plus this handoff). Set
`IRRADIANCE_CORPUS_DIR` as the handoff says.

Before checking, mark the verify cycle `[~]` in /projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d-timeline.md. If you hit something that needs
architect judgment or an external unblock, change it to `[?]` with a one-line reason
and stop.

Verify:
- Every acceptance criterion, with evidence. Compare AC4's digests and `scratch_len`
  values with the spec's registered table by script. Re-run the gates yourself; do not
  trust the build's numbers.
- The shared code the change touches (`develop_core`, the demosaic traversal refactor,
  `RING` at module level): no existing hash or Tier-B pin moved, and a mutant in the
  path shared by the oracle and the implementation is caught by an independent pin
  (the SPEC-091 FU-10 lesson).
- FU-5 and FU-6: every public constant and default tied by a test; every ranked
  refusal pair covered on all four entries, and each skipped pair truly impossible.
- Re-run a meaningful sample of the build's mutants (a)–(x) and your own (mix after
  geometry, strip size, `black_levels` read, version bumps, an `OpcodeList1` refusal
  skipped, the scratch formula off by one strip, the optional warp skipped); show each
  fails a named test. Check the argued equivalents (u), (y) and (z).
- Allocations: zero in `develop_with_scratch_into`, one in `develop_with_into`.
- Read CI run 45898355595's determinism logs: both legs ran the four developed-mix hashes.
- Rule on the build's deviations.
- Fixes on your branch are tests or docs only. `src/` changes go in a punch list.
- Out of scope: STAGE-034's lifting of the colour paths' refusal of mandatory
  `FixBadPixelsConstant` and `WarpRectilinear` on Leica files.

When done:
1. Fill the handoff's `handback:` front matter (verdict, and a REAL `tokens_total`
   measured from your transcript per docs/cost-tracking.md) and its `## Handback`
   section, including the three reflection questions.
2. Set `handoff.status` and `handback.status` to `completed`.
3. If approved: run `just advance-cycle SPEC-093 ship --verdict approved` and
   `just handback-sync SPEC-093` (CI's cost audit needs both in the same push), push
   `claude/spec-093-verify`, and fast-forward `feat/spec-093-developed-mix` to it so PR
   358 carries the verify commits. Do not merge. Commit messages are plain prose only.
4. If punch list or rejected: push your branch, and report the punch list without advancing.
5. Mark verify in the timeline (`[x]` with verdict, cost and date, or `[?]`).
