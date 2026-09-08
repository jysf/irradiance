# SPEC-017 — verify dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root,
on the SPEC-017 branch). Build reported two substantive findings: SB-1
(AC5's HIT count is 0 on all three real Q2M frames) and FU-2 (`dnglab
--srgb` ignores Orientation — same species as SPEC-018's FU-6). The build
made a defensible call on SB-1 (`#[ignore]` AC5 with the finding recorded);
your job is to judge it independently.

⚠ **SB-1 requires an INDEPENDENT MEASUREMENT** — read the raw plane bytes
yourself and count how many samples equal `Constant=0`. If 0, the build's
finding is real (AC5's assumption was wrong). If >0 but the applier still
returns 0, there IS a code bug.

---

```
Cycle: verify. You are the reviewer for SPEC-017 — FixBadPixelsConstant. You did
not build it — review it cold.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (checks 1-12) and §16's four codified lessons
     (unrun-docs-carry-errors is at N=6 as of SPEC-018 ship).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-050-verify-fixbadpixelsconstant-opcode.md
     — your contract. Two findings to judge: SB-1 (zero HIT count) and FU-2
     (dnglab ignores Orientation). Four bars (9-12) and the eight generic
     checks.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-044-build-fixbadpixelsconstant-opcode.md
     — build handoff and `## Completion` section. The build's own honest
     self-report on SB-1 and FU-2.
  4. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md
     — the spec. `## Acceptance Criteria` (11 ACs), `## Failing Tests`,
     `## Implementation Context`.
  5. projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-018-warprectilinear-radial-geometric-correction.md
     — SPEC-018's `## Follow-ups` table has FU-6 (dnglab-ignores-Orientation on
     L1026016), which is FU-2's sibling. Judge whether SPEC-017/FU-2 is the
     same class or a new species.
  6. src/opcode.rs — read the FixBadPixelsConstant applier. SPEC-018 shipped
     the module scaffold; SPEC-017 extended it.
  7. src/develop.rs — the develop pipeline. Confirm the applier is called
     BEFORE normalize (before SPEC-014's levels + before SPEC-018's warp).
  8. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode-timeline.md

Branch: feat/spec-017-fixbadpixels-opcode
Code SHA under review: cd82ca8 (feat commit). Tip 5c1ba36 is bookkeeping-only.
CI: run 34191738033 green on 11 jobs including `fuzz smoke — opcode` (60s,
13.7M executions, zero crashes).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation must change the file AND compile AND change the output.
  3. Tier-B tests pass whether or not the corpus is present. AC5 is #[ignore]d
     (tier-B) per SB-1; run WITH --ignored to reproduce.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-050's handback: only.
     ⚠ notes: MUST BE DOUBLE-QUOTED, NO bare `#` (signal
     handback-sync-treats-hash-as-comment-in-unquoted-notes).
     ⚠ CORRECT this handoff's top-level to_agent to your actual message.model
     BEFORE handback-sync runs (signal
     handback-sync-inherits-stale-to-agent-across-punch-list-rounds).
     Build set to_agent to claude-sonnet-5 correctly — first spec where the
     FU-11 discipline held. Yours will too if you correct it.

The two findings you must judge (this is the crux — HANDOFF-050 has the
detailed checks):

  SB-1 — AC5 HIT count is 0 on all three Q2M frames. Build's own AC1/AC4/AC8
  prove the applier is reachable and correct on synthetic input. Build labels
  this SB-1 and #[ignore]s AC5 with the finding recorded, arguing the raw
  sensor data never contains samples at Constant=0.

  ⚠ INDEPENDENT VERIFICATION REQUIRED: read the raw u16 plane bytes for at
  least one Q2M frame (via SPEC-012's plane::unpack or an irr scratch probe)
  and count samples == 0 across the ActiveArea. If the count is strictly 0,
  the build's finding is REAL and SB-1 downgrades to FU-N (AC5's assumption
  was wrong; disposition = closed with reason or spec amendment). If the
  count is > 0 but the applier still returns 0, there IS a code bug and SB-1
  stands as ship-blocker.

  FU-2 — dnglab --srgb never applies EXIF Orientation. Same species as
  SPEC-018/FU-6 (which SPEC-018's ship dispositioned as evidence for
  SPEC-021's oracle-scope narrowing). Judge whether SPEC-017/FU-2 is:
  (a) a re-instance of FU-6 (dispose "signal: <SPEC-021's target> — evidence"
      or "closed: instance of SPEC-018/FU-6, folded into SPEC-021 Context")
  (b) a materially different finding (dispose as a new signal or new spec)

The four repo-specific bars are answered in detail in HANDOFF-050:
  - Bar 9 (oracle red): AC8 tier-A red-proof — mutate applier, watch AC4 fail
  - Bar 10 (fuzz target): fuzz smoke — opcode green on CI (13.7M runs, 0
    crashes)
  - Bar 11 (provenance): expected ONE new row for FixBadPixelsConstant applier
    citing DNG § Chapter 7 (build corrected Chapter 6 → Chapter 7 mis-citation)
  - Bar 12 (no new deps): Cargo.toml [dependencies] MUST stay empty

Do not fix anything you find. Report; do not repair. Do not open the PR
(orchestrator), do not run handback-sync.

Return: findings labelled SB-N / FU-N — numbering RESTARTS at 1 for SPEC-017
(per-spec convention). Judge SB-1 explicitly: does it stand as ship-blocker,
or downgrade to FU-N based on independent verification? Then exactly one of:
  ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED
and a filled `handback:` block in HANDOFF-050 with:
  - status: completed | blocked | rejected
  - REAL tokens_total deduped by message.id (identify by scratchpad UUID, not
    text-matching — identify-own-transcript-for-cost-handback memory)
  - estimated_usd priced per-component, +20% uplift
  - notes: ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
  - verdict: approved | punch-list | rejected
  - CORRECT top-level to_agent to your actual message.model
```
