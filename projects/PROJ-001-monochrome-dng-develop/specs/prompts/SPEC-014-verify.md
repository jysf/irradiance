# SPEC-014 — verify dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root).
The orchestrator has already reconciled the build; this session reproduces it
and returns a verdict. Do not paste the reconciliation as fact — HANDOFF-033
carries it, and every row in it is yours to re-run.

---

```
Cycle: verify. You are the reviewer for SPEC-014. You did not build it — review it cold.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (checks 1-12) and §16's three codified lessons.
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-033-verify-level-normalization-activearea-to-defaultcrop-and-orientation.md
     — your contract. It carries the orchestrator's reconciliation table
     (reproduce it, do not inherit it), seven checks it did NOT make, and your
     return criteria.
  3. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-014-level-normalization-activearea-to-defaultcrop-and-orientation.md
     — read `## Implementation Context` in full. The blind-spot section is the spec.
  4. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-032-build-... — the build's own handback.
  5. guidance/constraints.yaml, guidance/toolchain-brief.md.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-014-...-timeline.md

Branch: feat/spec-014-level-normalization-geometry-orientation
Code SHA under review: 1404aac (the only commit touching src/). Branch head is
docs-only on top of it — approve whichever SHA you actually measured, and
observe CI green on that SHA.

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images
The default corpus root does not exist. A tier-B test passes whether or not the
corpus is present; only `just test` names what is missing. Say in your handback
whether the corpus was present and whether any test SKIPped.

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. And read CI: constraints.yaml requires the gate OBSERVED green on
     the shipping SHA, not inferred from a local run.
  2. Every mutation must change the file AND compile AND change the output.
     That third clause has caught four false red-proofs in three specs.
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-033's `handback:` block only,
     so `handback-sync` runs once cleanly — hand-writing has caused four
     duplicate-entry cleanups. ⚠ FIRST_SESSION_PROMPTS.md's generic Prompt 4
     still says to append the cost session yourself; it is stale on this point
     (open signal `cost-field-has-two-owners`). HANDOFF-033 wins.

The one thing that makes this spec different: IT HAS NO ORACLE, by design.
SPEC-013's --raw-checksum attaches to the uncropped, un-normalised plane by
contract, and DEC-004 settled that no comparison oracle ever will cover this
surface. SPEC-015 is the analytic oracle and is still in `frame`. Until it
lands, this spec's own tests are the only check that exists — review them as the
sole line of defence, and answer HANDOFF-033's check 7 explicitly: does
`oracle-must-be-shown-red` bite here at all, and if not, say so with the reason.

Do not fix anything you find. Report; do not repair. Do not open the PR, do not
merge, do not run handback-sync.

Return: findings labelled SB-N / FU-N with §15 dispositions — numbering
CONTINUES SPEC-014's sequence, FU-1 is already taken, so your first is FU-2 —
then exactly one of:
  ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED
and a filled `handback:` block with a REAL tokens_total deduped by message.id,
priced per-component, rounded up to cover the turns that write the handback
(measured: self-reports here run 9.9% and 15.4% low).
```
