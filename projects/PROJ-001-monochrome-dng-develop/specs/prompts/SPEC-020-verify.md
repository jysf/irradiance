# SPEC-020 — verify dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo
root). The orchestrator has already reconciled the build (HEAD `61e5eb4`,
3 CI runs green, 20 new tests passing, `src/` untouched, `cost.sessions
[build]` synced). This session reproduces the claims and returns a
verdict. Do not paste the reconciliation as fact — HANDOFF-045 carries
it, and every row is yours to re-run.

---

```
Cycle: verify. You are the reviewer for SPEC-020. You did not build it — review it cold.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (checks 1-12) and §16's four codified lessons
     (including this week's `a-fix-inherits-the-precondition-of-the-thing-it-fixes`).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-045-verify-develop-oracle-vs-dnglab-srgb.md
     — your contract. It carries the reconciliation the orchestrator already made
     (reproduce it, do not inherit it), the four repo-specific bars 9-12, the three
     build-proposed `closed:` dispositions to judge, and your return criteria.
  3. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-020-develop-oracle-vs-dnglab-srgb.md
     — read `## Acceptance Criteria` and `## Failing Tests` in full. `## Implementation Context`
     names the pre-registered numeric thresholds every AC bites through.
  4. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-043-build-develop-oracle-vs-dnglab-srgb.md
     — the build's own handback. `## Cost self-report`, `## Drift and new artifacts` and `##
     Reflection` carry the three FUs the build raised. Read the build's own honest self-report
     before forming your own opinion of any finding.
  5. decisions/DEC-023-ssimulacra2-devdep-sanction.md — the new dev-dep sanction; contains
     the corrected BSD-2-Clause licence and the 38-crate `cargo tree` count.
  6. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-020-develop-oracle-vs-dnglab-srgb-timeline.md

Branch: feat/spec-020-develop-oracle-vs-dnglab-srgb
Code + docs SHA under review: 5bb827b (branch tip).
Build-code SHA: 61e5eb4 (the last commit touching tests/). Everything after
61e5eb4 is orchestrator bookkeeping (handback-sync stamp, HANDOFF-045, timeline).
Approve whichever SHA you actually measured, and observe CI green on that SHA.

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
The default corpus root does not exist. All 20 new tests in tests/perceptual_oracle.rs
are TIER A by design (AC7): they run with the corpus unset. Say in your handback
whether the corpus was present and whether any test SKIPped.

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. PATCH-004 made both recipes print their own clippy version;
     quote what they say. And read CI: constraints.yaml requires the gate
     OBSERVED green on the shipping SHA, not inferred from a local run.
  2. Every mutation (including a verify's own red-proof reproduction) must
     change the file AND compile AND change the output. That third clause
     has caught FIVE false red-proofs across earlier specs.
  3. Tier-B tests pass whether or not the corpus is present — a silent
     skip is not a pass. This spec's twist: the whole red-proof (AC7)
     runs tier A, so bar 3 mostly does not fire here; but the optional
     tier-B smoke does, and FU-1's proposed close depends on judging
     whether the substitute test really exercises the load-bearing claim.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-045's `handback:` block
     only, so `handback-sync` runs once cleanly. `notes:` must be ONE
     PHYSICAL LINE — handback-sync truncates multi-line YAML scalars and
     leaves the spec unparseable while every gate reports green
     (`handback-sync-truncates-multi-line-scalars`).

The four things that make THIS spec different:

  a. `src/` is untouched by design (AC10). The whole delivery is
     test-support + one dev-dep + one DEC + the perceptual oracle test file.
     Confirm with `git diff --name-only 303a8f6..HEAD -- src/` — should print
     nothing. Then read the four new files:
       tests/support/pnm.rs         — the P6-over-P5 PNM reader (AC1/AC2)
       tests/support/ssimulacra2.rs — the metric wrapper (AC3)
       tests/support/perturb.rs     — shift/gamma/warp fault generators
       tests/perceptual_oracle.rs   — the tier-A red-proof + optional tier-B smoke

  b. The FOUR MEASURED FAULT SCORES are load-bearing. The build reports:
       identity 100.000  |  1-px shift 61.823  |  missing warp −82.338  |  gamma 1.05 90.038
     Reproduce each with `cargo test --all-features --test perceptual_oracle
     <test_name> -- --exact --nocapture`. A score off by ~0.5 is noise; off by
     10+ is a finding. Bar 9 (§15) — did the oracle go red? — is the missing-warp
     one specifically: DEC-005's pre-registered falsifier is "a missing warp
     must land far below 85" and the measured −82.338 satisfies it. See it
     with your own eyes.

  c. Bar 10 (fuzz target) is N/A on this spec — the PNM reader consumes bytes
     it produced itself from dnglab, not from an attacker. Confirm no fuzz
     target was added anyway, then answer bar 10 `n/a` with that reason.

  d. `DEC-023` is a permissive dev-dep sanction (bar 12). Run
       cargo deny check licenses
       cargo deny --manifest-path fuzz/Cargo.toml check licenses
     yourself — both graphs, per SPEC-003's shipping-week lesson. One line
     quoted for `ssimulacra2`'s actual licence (BSD-2-Clause per DEC-023).
     `[dev-dependencies]` placement matters: a runtime dep would violate
     `library-not-application` even at a permissive licence.

The three build-proposed `closed:` dispositions (FU-1, FU-2, FU-3) each rest
on an ASSUMPTION about a mechanical trigger that will catch future drift.
Apply this week's `a-fix-inherits-the-precondition-of-the-thing-it-fixes`
lesson: judge the assumption, not the reasoning around it. HANDOFF-045
carries the three assumptions verbatim.

Do not fix anything you find. Report; do not repair. Do not open the PR
(PR #15 is already open by the orchestrator), do not merge, do not run
handback-sync.

Return: findings labelled SB-N / FU-N — numbering RESTARTS at 1 for
SPEC-020 (SPEC-007-onward per-spec convention, AGENTS.md §15). Then
exactly one of:
  ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED
and a filled `handback:` block in HANDOFF-045 with a REAL tokens_total
deduped by message.id (identify your OWN session by its scratchpad UUID,
not by text-matching spec strings — this repo's own memory
`identify-own-transcript-for-cost-handback` records that trap), priced
per-component, rounded up to cover the turns that write the handback
(measured: self-reports here run 9.9%–15.4% low). Also fill the new
`verdict:` field in the handback (`approved` | `punch-list` | `rejected`)
so it copies into spec.task.verify_verdict at ship.
```
