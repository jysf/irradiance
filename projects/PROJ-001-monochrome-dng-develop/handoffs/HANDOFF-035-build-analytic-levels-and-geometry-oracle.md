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
  id: HANDOFF-035
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.build, not a measurement.
                                    # Standing record: 0 FOR 10 on the build hint. CORRECT THIS
                                    # to whatever your own system prompt reports as
                                    # `message.model` before handing back
                                    # (signal `tier-map-predicts-what-it-should-record`).
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-015

project:
  id: PROJ-001
  stage: STAGE-002
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

# HANDOFF-035: Build SPEC-015 — the analytic levels and geometry oracle

## Delegation Summary

Build `SPEC-015`. It closes `STAGE-002`. `SPEC-014` shipped the develop path and
asserted **its own arithmetic**; this spec checks that arithmetic against
expectations derived independently of how `develop_into` computes them, and
proves the check can go red.

Branch from `main` at `23087dc` (SPEC-014 merged, CI green, 143 tests).
`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` —
the default root does not exist.

## ⚠ The one idea this whole spec turns on

**An oracle that reimplements the transform is a mirror.** Written by the same
project from the same reading of the same spec, a second copy of the eight-case
orientation table fails and succeeds for exactly the same reasons as the first.
`DEC-004` already names the limit — it verifies *"the arithmetic we chose"*.

The design probe found the way out, and **measured that it works**. Read
`## The design decision this spec rests on` in the spec before writing anything.
Two rules follow from it, and they are what the acceptance criteria enforce:

1. **Never write the eight-case orientation table** (`AC3`). Assert the
   *permutation property* instead: `develop_into` rearranges the normalized crop
   window, so the output histogram must equal the histogram of that window taken
   in raster order with **no orientation applied**. Measured to hold exactly on
   the `Orientation 6` frame — with no knowledge of what 6 means anywhere in the
   check.
2. **Never derive expected values from `DEC-018`'s rounding rule** (`AC1`).
   Assert `< 0.5 LSB` from the **exact real-valued** affine map. That bound is
   satisfied by any correct rounding and violated by every wrong map, so it is a
   statement about the transform rather than about our choice. `FU-4`'s existing
   test already pins the choice — do not duplicate it.

Separate **what is forced** (endpoints — exact) from **what is chosen**
(interior rounding — bounded). That distinction is the spec.

## What is already measured — reproduce, do not re-derive

The spec's `## Implementation Context` carries all of it: the tolerance table
across three frames (max **0.499968**, **zero** pixels at or above 0.5, over
111,529,040), the truncation trap (**45.0–50.1 %** of pixels), the histogram
property holding exactly with distinct-level counts landing on the full in-range
domain (**15,872** and **16,164**), and both fault injections with their pixel
counts. The probe ran in **2.6 s** for all three frames in release.

⚠ **The tolerance is pre-registered** (`pre-register-the-tolerance`). `< 0.5`,
falsifier a single pixel at `≥ 0.5`. If you measure a max at or above 0.5, that
is a **finding, and you stop** — it is not a threshold to relax.

## Two things that make this different from a normal build

**1. You are writing a check, not a feature. `src/` is off-limits** (`AC7`).
`src/develop.rs`, `src/plane.rs` and `src/ifd.rs` must be **0 lines changed**
against `main`. If the oracle finds a real defect, that is the **most valuable
outcome this spec can have** — stop, report it, and do not adjust either side to
make it pass. An oracle edited until it agrees is worse than no oracle.

**2. The red-proof must run where CI can see it** (`AC6`). `SPEC-013/FU-1` is the
precedent *and* the warning: its red-proof genuinely works and CI has **never
once executed it**, because it needs the corpus. `SPEC-014/FU-7` measured the
same shape from the other side — four of six `tests/develop.rs` tests execute
zero assertions with the corpus absent. So `AC5`'s two faults must go red with
`IRRADIANCE_CORPUS_DIR` **unset**, over a hand-built fixture. `SPEC-013`'s
reviewer measured that shape at **1.47 s** for two cold builds, and
`SPEC-014/FU-3` used it successfully; you are not inventing a mechanism.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. `just lint-ci`, **not** `just lint` — local clippy is 0.1.97 and CI
   floats at 0.1.98; assert the version you actually linted under. **Push and
   read CI** — `constraints.yaml` requires the gate *observed* green on your SHA.
2. ⚠ **`src/` is 0 lines changed vs `main`.** Show it (`git diff --stat main...HEAD -- src/`).
3. **Watch both red-proofs fail yourself, with the corpus absent**, and paste the
   pixel counts. Every mutation: file changed **and** compiled **and** *output
   changed*. That third clause has caught four false red-proofs in three specs.
4. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --` and shipped a reconstruction. md5-verify
   every revert.
5. **`SPEC-013`'s and `SPEC-014`'s tests keep passing untouched** — 143 before,
   say what after.
6. **No fuzz target.** This adds no parser and no new input surface (spec
   Non-Goals). Say so explicitly rather than adding one; §12 bar 2 does not fire.
7. **Provenance row** — separate row, class 1, DNG 1.7. The ledger tracks
   implementations, not features, and this is a second implementation.
8. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports, **rounded up ~20 %** to cover the turns that write
   the handback — measured here at **9.9 %**, **15.4 %** and **19.2 %** low
   across three sessions, and the 20 % uplift landed the last one 3.1 % low.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block only, so
   `handback-sync` runs once cleanly. Hand-writing has caused four
   duplicate-entry cleanups.
   ⚠ **The project transcript directory also holds the ORCHESTRATOR's live
   session**, on a different model, text-matching this delegation because it
   wrote this handoff. It is **not** a prior attempt. Identify your own
   transcript by the uuid in **your own scratchpad path**, not by content match
   (signal `orchestrator-transcript-looks-like-a-prior-attempt`,
   `SPEC-014/FU-8`).
9. **Correct `handoff.to_agent`** to what your system prompt reports. Standing
   record: the build hint is **0 for 10**.
10. **Do not run `handback-sync`; do not open the PR.**
11. Findings `SB-N`/`FU-N` with proposed §15 dispositions, numbering from `FU-1`
    (this spec's own sequence). A `spec:` disposition must **name an AC that
    would fail** without it.
12. Answer §15's reflection questions in the handback.

## Out of Scope

- Anything in `src/`. See `AC7` and Return Criterion 2.
- A second copy of the orientation table (`AC3`) — the failure mode this spec
  is designed around.
- Re-asserting what `SPEC-014` already asserts. Read `tests/develop.rs` and
  `src/develop.rs`'s unit tests first; endpoints on real tags, crop dimensions,
  `Orientation` 1 and 6 dimensions, the `ActiveArea`-relative origin and the
  rounding pin are all already there.
- SSIMULACRA2 / `dnglab --srgb` / any perceptual comparison — `DEC-004` and
  `DEC-005` closed that with measurements.
- A fuzz target (Return Criterion 6).
- Opening the PR, running `handback-sync`, or touching `STAGE-002`'s close.

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
