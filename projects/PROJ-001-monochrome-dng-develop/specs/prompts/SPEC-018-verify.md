# SPEC-018 — verify dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo
root, on the SPEC-018 branch). The orchestrator has already reconciled
the build (HEAD `40f5d45`, CI green, `handback-sync` run,
`cost.sessions[build]` filled at 116,480,125 tokens). This session
reproduces the claims and returns a verdict.

⚠ **This spec's two findings are structurally load-bearing.** They
question SPEC-020's develop oracle at a design level, not just
SPEC-018. Read HANDOFF-046's `## Completion` and DEC-024 before
forming any opinion of your own.

---

```
Cycle: verify. You are the reviewer for SPEC-018 — WarpRectilinear. You did not build it — review it cold.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (checks 1-12) and §16's four codified lessons
     (including this week's `a-fix-inherits-the-precondition-of-the-thing-it-fixes`).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-047-verify-warprectilinear-radial-geometric-correction.md
     — your contract. It carries the two findings the build raised (with the
     checks you must run for each), the four repo-specific bars 9-12, and the
     return criteria.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md
     — the build handoff and its filled handback. `## Completion` details the
     two findings. Read the build's own honest self-report before forming your
     own opinion.
  4. decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md
     — the new decision record. Read in FULL. The oracle-narrowing question
     is at its heart.
  5. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md
     — the spec. `## Acceptance Criteria` (14 numbered ACs), `## Failing Tests`,
     `## Implementation Context`. AC1's per-frame hex fixtures live under
     tests/oracle-fixtures/.
  6. projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-020-develop-oracle-vs-dnglab-srgb.md
     — the develop oracle SPEC-018 depends_on. Its assumptions are what
     finding 2 questions. `docs/oracle-contract.md § "This oracle is
     single-sourced"` was the earlier warning — finding 2 goes deeper.
  7. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md

Branch: feat/spec-018-warprectilinear-radial-geometric-correction
Code SHA under review: 40f5d45 (the one commit touching src/). Nothing on top
except orchestrator bookkeeping (handback-sync stamp, HANDOFF-047, timeline).
Approve whichever SHA you actually measured, and observe CI green on that SHA.

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
The default corpus root does not exist. AC8/AC9 are #[ignore]d (finding 2);
run them WITH `cargo test -- --ignored` to reproduce the finding's numbers,
not to gate on them. AC10 (tier-A red-proof) runs without the corpus.

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. Both recipes print their own clippy version (PATCH-004); quote
     what they say. And read CI: constraints.yaml requires the gate OBSERVED
     green on the shipping SHA, not inferred from a local run.
  2. Every mutation must change the file AND compile AND change the output.
     That third clause has caught FIVE false red-proofs.
  3. Tier-B tests pass whether or not the corpus is present. This spec's
     twist: the tier-B AC8/AC9 are #[ignore]d, so the corpus half is
     structurally quiet either way. Reproduce with --ignored explicitly.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-047's `handback:` block
     only. `notes:` must be ONE PHYSICAL LINE.

The two findings you must judge (this is the crux — HANDOFF-047 has the
detailed checks):

  Finding 1 — the pipeline-order fix. Build discovered SPEC-018's design was
  wrong: warp runs BEFORE DefaultCrop, not after (per DNG 1.7 § 6.4.1). Read
  the spec clause yourself. Read src/develop.rs. Judge whether the fix
  landed and whether it has teeth (a red-proof of `crop-then-warp` should
  fail the tests it should).

  Finding 2 — dnglab does not implement DNG opcode processing at all. Build
  read dnglab's source and confirmed no application code for WarpRectilinear
  or FixBadPixelsConstant. Consequence: a correct warp scores −60.169 against
  dnglab; doing nothing scores +83.145. SPEC-020's oracle is structurally
  broken for warp-bearing pixels. Build's disposition: AC8/AC9 #[ignore]d
  with DEC-024's reasoning, AC4/AC10 (analytic geometry check) as the actual
  correctness gate.

  ⚠ FOR FINDING 2 SPECIFICALLY: verify the "dnglab does not implement
  opcodes" claim IN CODE, not from the build's assertion. §15 rule 8
  behavioral pre-flight and §16 rule 4 unrun-docs-carry-errors both apply.
  Options: (a) clone/pull dnglab source and grep for the missing application
  code, or (b) run `dnglab analyze --srgb` on L1021223.DNG and inspect
  whether the output shows radial distortion correction (a straight line at
  the corner will bend outward if the warp applied, stay straight if not).

The four repo-specific bars are answered in detail in HANDOFF-047:
  - Bar 9 (oracle red): shifts from SPEC-020's perceptual to SPEC-018's AC10
    analytic red-proof, which MUST turn red on kr1=0 mutation
  - Bar 10 (fuzz target): warp_opcode ran 60s in CI on 40f5d45; confirm
  - Bar 11 (provenance): TWO new rows expected — opcode parser + warp+kernel
  - Bar 12 (no new deps): Cargo.toml [dependencies] MUST stay empty;
    `cargo tree -e normal` should print `irradiance v0.1.0` alone

Do not fix anything you find. Report; do not repair. Do not open the PR
(orchestrator opens it after your verdict), do not run handback-sync.

Return: findings labelled SB-N / FU-N — numbering RESTARTS at 1 for SPEC-018
(per-spec convention). Judge finding 2 explicitly as SB-N (block ship until
DEC-024's oracle narrowing is accepted or replaced) or FU-N (ship with
#[ignore] and address at a follow-up spec). Then exactly one of:
  ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED
and a filled `handback:` block in HANDOFF-047 with:
  - status: completed | blocked | rejected
  - a REAL tokens_total deduped by message.id (identify your OWN session by
    scratchpad UUID, not text-matching — identify-own-transcript-for-cost-
    handback in this project's memory)
  - estimated_usd priced per-component at your model's rates, +20% uplift
  - notes: ONE PHYSICAL LINE
  - verdict: approved | punch-list | rejected (mirrors the banner)
```
