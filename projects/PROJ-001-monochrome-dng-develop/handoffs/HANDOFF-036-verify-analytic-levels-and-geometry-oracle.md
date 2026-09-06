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
  id: HANDOFF-036
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify, not a measurement.
                                    # The verify hint has been RIGHT before (HANDOFF-033);
                                    # the build hint is 0-for-11. CORRECT THIS to whatever
                                    # your system prompt reports as `message.model`.
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-036: Verify SPEC-015 — the analytic oracle, at `7439f49`

## Delegation Summary

Verify `SPEC-015` at **`7439f49`** on
`feat/spec-015-analytic-levels-and-geometry-oracle` (pushed, not merged; `main`
at `23087dc`). It closes `STAGE-002`.

**This is a strong build.** Every number it reports reproduces the design
probe's independently, `src/` is untouched, and the red-proofs run where CI can
see them — which is more than `SPEC-013`'s managed. Verify it on that basis: the
risk is not sloppiness, it is a **well-made oracle with a blind spot the
orchestrator has already measured** (below). Your job is to find the next one.

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| two commits, CI **9/9 on both** | ✅ `2532dc2` run `34000895054`; `7439f49` run `34001284845` |
| **`src/` 0 lines changed vs `main`** (`AC7`) | ✅ `git diff --stat main...HEAD -- src/` is empty |
| 150 tests (was 143), 0 failed | ✅ summed across 9 targets, corpus present |
| "zero skipped" | ✅ the one `SKIP` line is `corpus_absent_file_is_missing_not_an_error`'s deliberate temp-dir probe — pre-existing, not this spec's |
| every measured number reproduces the design probe | ✅ **exactly** — max dev `0.499968 / 0.499968 / 0.499969`, truncation `50.1 % / 49.1 % / 45.0 %`, distinct levels `15872 / 16164` |
| **both red-proofs red with `IRRADIANCE_CORPUS_DIR` unset** (`AC6`) | ✅ ran myself: levels `0.499968 → 264.658371`, 15,841/17,408 px; orientation `[10,0,11,1,12,2] → [0,1,10,11,0,0]`, 6/6 |
| `AC8` — tier-B under 60 s | ✅ **14.68 s** parallel; **35.96 s** at `--test-threads=1`. Both pass; the serial figure is closer to the bound than the handback's "~15 s" implies |
| **`AC3` — no orientation table in the oracle** | ✅ **zero** `5=>`…`8=>` arms in either new file; all 7 mentions of "orientation" in `tests/support/oracle.rs` are doc comments saying it reimplements nothing |
| ten gates, not eleven | ✅ honest — the eleventh has always been fuzz, and the spec's Non-Goals excluded it |
| gates run by me | ✅ all green, `lint-ci` at **clippy 0.1.98** asserted |
| `DEC-020`, `DEC-021`, provenance row, two signals | ✅ present; `decisions-audit` 0 structural errors |

⚠ **Credit where due.** The orientation red-proof carries all three clauses
properly — an `assert_ne!` on the mutant output with an explicit "semantic
no-op, this red-proof has caught NOTHING" message, a green-on-honest control
inside the same test, **and** a separate `the_orientation_fixture_oracle_control_is_green`
for the apparatus. And `FU-2` is a self-caught bug: the build's first
optimisation (pairing distinct values instead of full rank/count) was wrong, its
own honest-tree assertion caught it, and it was filed as a signal rather than
buried. That is the behaviour the handback contract exists to produce.

## ⚠ The blind spot the orchestrator measured — confirm, then decide what it means

**A rank/frequency oracle cannot distinguish one valid permutation from
another.** This follows directly from `DEC-020` and it is not a bug — it is the
price of refusing to reimplement the orientation table. But **nothing says so in
writing**, and a reader will otherwise assume the oracle covers orientation on
real data. It does not.

Measured. Mutation: apply **`Orientation 8`'s mapping where the file says 6** —
a *valid, same-size, same-multiset* permutation (verify the bijection yourself:
for this geometry both 6 and 8 are genuine bijections of the crop window, so no
out-of-bounds zeros appear to give it away):

```
6 => (out_y, flip_y(out_x))            // honest
6 => (flip_x(out_y), out_x),           // MUTANT — Orientation 8's mapping
```

Result on 46,726,912 real pixels — file changed, compiled, output changed:

```
AC1  every_pixel_is_within_half_an_lsb_of_the_exact_affine_map .... PASSED  (blind)
AC3  the_developed_histogram_is_the_normalized_crop_windows ....... PASSED  (blind)
AC4  distinct_output_levels_equal_distinct_input_levels ........... PASSED  (blind)
```

**All three of SPEC-015's tier-B oracle tests pass on a wrong permutation
applied to a real 47-megapixel frame.** What caught it: `SPEC-014`'s
`crop_source_coords_matches_the_worked_example_for_all_eight_orientations`,
`SPEC-014/FU-3`'s `develop_into_applies_orientation_to_pixels_not_only_dimensions`,
and this spec's own red-proof **honest-tree guard** — all three positional, all
three on hand-built fixtures of 6 pixels or fewer.

**So: levels and crop-window contents are checked on 111.5 M real pixels;
*which* permutation was applied is checked on 6.** That may be entirely
acceptable — the positional coverage exists and `DEC-020` bought real
independence for it — but **judge it and say so with the reason.** If it is
acceptable, the orchestrator's view is that it belongs in `DEC-020`'s
`## Consequences` in as many words, because it is exactly the kind of limit a
future reader will assume away. If you disagree, say why.

## Your own checks — where the orchestrator did not go

1. **Does the mutate-and-rebuild actually rebuild?** `DEC-021`'s orientation
   half copies, mutates and rebuilds a probe crate (`SPEC-013`'s `DEC-017`
   mechanism). `SPEC-013`'s verify found three ways to fool that apparatus.
   **Break the rebuild deliberately** — make the injection non-compiling, and
   separately make it a semantic no-op — and confirm the test fails loudly
   rather than silently comparing something against itself. The `assert_ne!`
   guard suggests the no-op case is handled; prove it.
2. **Is `FU-2`'s blind spot actually closed?** The build says it replaced
   distinct-value pairing with full rank/count. Construct a fault that changes
   **only multiplicity** — same set of distinct values, different counts — and
   confirm `bound_check` and `multiset_equal` both catch it. That is the exact
   class `distinct-value-dedup-drops-multiplicity` names, and the fix should be
   provable rather than asserted.
3. **What is `AC1`'s sensitivity floor?** The levels red-proof uses
   `BlackLevel + 64`, which produces a max deviation of **264.66** — four orders
   of magnitude over the 0.5 bound. `DEC-004` measured `BlackLevel + 1` as
   SSIMULACRA2 **100.00**, i.e. completely invisible to the develop oracle.
   **Does this oracle catch `+1`?** Nobody has measured it. If it does, that is
   a strong result worth recording; if it does not, the oracle's floor is worth
   knowing before `STAGE-002` closes on it.
4. **Is `the_orientation_fixture_oracle_control_is_green` load-bearing?**
   Mutate it and see what dies. A control that cannot fail is not a control
   (`DEC-009`).
5. **`AC8` under CI's conditions.** 14.68 s parallel / 35.96 s serial locally.
   CI is a different machine and `cargo test` there may serialise differently.
   Confirm the margin is real rather than local.
6. **`AC2`'s floor.** It asserts `> 40 %` against a measured 45.0–50.1 %. Is
   5 points of margin enough for a file the corpus does not yet hold? Judge
   whether the floor is defensible or merely passing.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-015-analytic-levels-and-geometry-oracle.md`
  — read `## The design decision this spec rests on` first; it is the spec.
- **Build handoff:** `HANDOFF-035` and its `## Handback`.
- **Stage:** `STAGE-002` — this spec closes it.
- **Toolchain brief:** `guidance/toolchain-brief.md` (DEC-004 rule 5).
- **Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.

### Decisions that apply

- `DEC-004` — analytic, never by comparison. Its rule 1 is your job description.
- `DEC-005` — why SSIMULACRA2 cannot do this (and check 3 above).
- `DEC-020` — **new.** Rank/frequency, never positional. Conf. 0.85. This is the
  decision the blind spot above falls out of.
- `DEC-021` — **new.** The two red-proofs use deliberately different mechanisms.
  Conf. 0.85. Check 1 above is aimed at its riskier half.
- `DEC-018` / `DEC-019` — the rounding rule and the crop-origin convention the
  oracle must *not* read as its source of truth.
- `DEC-009`, `DEC-017` — control discipline and the mutate-rebuild mechanism.

### Constraints that apply

- `oracle-must-be-shown-red` — this time the constraint **has** a subject. `AC6`
  is the strongest form this repo has shipped: red **with the corpus absent**.
  Confirm that is genuinely what CI runs.
- `library-not-application` — the oracle lives in `tests/`, not the library.
- `provenance-recorded-per-algorithm` — one new row, separate from
  `src/develop.rs`'s, because the ledger tracks implementations.
- `test-before-implementation`, `no-panics-on-untrusted-input`.

## Out of Scope

- **Fixing anything.** Report; do not repair. A punch list is a verdict.
- **Editing `src/`.** `AC7` binds you too.
- Opening the PR, merging, running `handback-sync`, or closing `STAGE-002`.

## Return Criteria

1. **Ten gates + `just lint-ci`** (there is no fuzz gate — the spec's Non-Goals
   excluded it; confirm that reasoning rather than inheriting it), run by you,
   pasted, summed across all targets, with the clippy version asserted.
   **Observe CI green on the SHA you approve.**
2. **Watch both red-proofs fail yourself, with the corpus absent** (§15 check 9,
   `DEC-004` rule 1), and paste the numbers.
3. **Confirm or kill the permutation blind spot above**, and say what it means.
4. Checks 1–6 under *Your own checks*, each answered with a measurement.
5. Every mutation: file changed **and** compiled **and** *output changed*.
   ⚠ **Stage your work before mutate-and-revert**; md5-verify every revert.
6. **Provenance** (§15 check 11) — one new row, class 1, honest?
7. Handback with a real `tokens_total` **deduped by `message.id`**, priced
   **per-component**, **rounded up ~20 %** — measured here at 9.9 %, 15.4 % and
   19.2 % low across three sessions.
   ⚠ **Do not hand-write `cost.sessions`** — fill the `handback:` block only.
   ⚠ The project transcript directory also holds the **orchestrator's** live
   session, on a different model, text-matching this delegation because it wrote
   this handoff. It is **not** a prior attempt — identify yours by the uuid in
   **your own scratchpad path** (`SPEC-014/FU-8`).
8. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
9. Findings `SB-N`/`FU-N` with §15 dispositions — numbering **continues this
   spec's sequence, `FU-1`…`FU-3` are taken**, so your first is `FU-4`.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

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
