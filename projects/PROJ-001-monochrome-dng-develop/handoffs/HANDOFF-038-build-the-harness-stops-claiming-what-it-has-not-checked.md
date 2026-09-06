---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-038
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.build, not a measurement.
                                    # Standing record: 0 FOR 11 on the build hint. CORRECT THIS
                                    # to what your own system prompt reports as `message.model`.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-016

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-038: Build SPEC-016 — the harness stops claiming what it has not checked

## Delegation Summary

Build `SPEC-016`. Five surfaces each report a result they did not establish;
each gets a fix **and a falsifier**, so the class closes by a failing test rather
than by anyone remembering.

Branch from `main`. `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist. 152 tests pass today.

## ⚠ Read this before the spec

**All five findings are already measured**, in `## Implementation Context`.
Reproduce them; do not re-derive. Two changed when the design session measured
them rather than trusting the record, so the record you inherit is the corrected
one:

- `SPEC-012/FU-1` was *"two depths have no tests"* — confirmed **by mutation**:
  deleting `8` and `12` from `SUPPORTED_BITS` leaves **152/152 green**.
- `SPEC-005/FU-2` was *"`req()` truncates"* — sharpened. It truncates **and its
  doc comment justifies the truncation** as correct for this corpus. The defect
  is the undefended assumption, not the `.first()`.

## The two things most likely to go wrong

**1. Every test here must be TIER A.** Each finding is about a surface that lies
*when something is absent* — the corpus, a tool, a test, a parser — and CI is
precisely where things are absent. A tier-B test of any of them reproduces
`ci-cannot-prove-bit-exactness`, which is the signal this spec exists to stop
compounding. `AC1` in particular can only be proven in an environment **without**
`exiftool`, so build that environment rather than reasoning about it.

**2. `AC3` must assert over the constant, not over a list you type.** A test
whose body says `[8, 12, 14, 16]` passes forever after someone adds a fifth
depth — which is the same defect one level up, and would be an ironic way to fail
this spec. Enumerate `SUPPORTED_BITS` itself.

## Two facts about this machine, measured — do not assume either

- **`python3` here has no `pyyaml`.** `import yaml` → `ModuleNotFoundError`.
- **`ruby -ryaml` works**, and Ruby ships on macOS and on `ubuntu-latest`.

`AC5` needs a real parser. State which you chose and **confirm it exists in the
CI image**, rather than discovering it in a red run.

## The design question is settled — do not reopen it

`corpus-status` **checks the tools**; it does not merely stop claiming. The
reasoning and the rejected option are in the spec, and `AC1` expects a `DEC-*`
recording both. One decision for the spec, not one per criterion.

## Return Criteria

1. **Ten gates + `just lint-ci`**, run by you, pasted, summed across all targets,
   clippy version asserted (local 0.1.97; CI floats at 0.1.98). ⚠ **Say which
   gate list you ran** — the count is genuinely ambiguous here
   (`the-gate-count-is-not-defined-anywhere`, `bar: 3`); report the ambiguity,
   do not resolve it.
2. **Push and read CI.** Observed green on your SHA, run id and job count.
3. **All five red-proofs watched and pasted** (`AC6`). Each: file changed **and**
   compiled **and** *output changed*. ⚠ That third clause caught a false
   red-proof in `PATCH-002` two days ago, where the obvious injection removed the
   very text the detector was written to survive — so check that your injection
   exercises the path you think it does.
4. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --`, and the orchestrator did the same thing to
   `PATCH-002` this week. Mutate in an isolated copy, or commit first; md5-verify
   every revert.
5. **Test count**: 152 before, say what after, and confirm `SPEC-013`'s and
   `SPEC-015`'s oracles still pass untouched.
6. **No fuzz target added** — `AC4` fixes an existing target's seeds. Say so.
7. Handback with a real `tokens_total` deduped by `message.id`, priced
   per-component, **rounded up ~20 %** (measured 9.9 %, 15.4 %, 19.2 % low across
   three sessions here). ⚠⚠ **`notes:` MUST BE ONE PHYSICAL LINE** —
   `handback-sync` transcribes only the first line of a multi-line YAML scalar,
   leaving an unterminated quote that makes the spec's front matter unparseable
   while every gate still reports green. Measured twice; one shipped undetected
   for two days. That is `AC5`'s own subject, so breaking it here would be
   unfortunate.
   ⚠ The project transcript directory also holds the **orchestrator's** live
   session on a different model — not a prior attempt. Identify yours by the uuid
   in **your own scratchpad path**.
8. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
9. Findings `SB-N`/`FU-N` from `FU-1`, with proposed §15 dispositions.
10. Answer §15's reflection questions.

## Out of Scope

- **The gate-script audit.** Sized in the spec (8 of 13 `pipefail` scripts, 28
  unguarded greps in `test.sh` alone) and left to `STAGE-005`'s own bullet.
  Folding it in makes this XL.
- **Defining "the gates"** — a repo decision, filed as a signal.
- **Fixing `handback-sync`'s truncation.** `AC5` makes it detectable; the writer
  is a separate, filed problem.
- Anything in `src/` beyond `SUPPORTED_BITS`'s coverage. No decoder behaviour
  changes.
- Opening the PR, running `handback-sync`, or touching `STAGE-005`'s close.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
