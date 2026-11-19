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
  id: HANDOFF-163
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-059

project:
  id: PROJ-004
  stage: STAGE-016
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-163: VERIFY: GainMap parse and apply on the mono and colour paths (SPEC-059)

## Delegation Summary

You are the **verifier** for SPEC-059. Review it cold: you did not build it.

- **Under review:** PR #267 (`claude/spec-059-build`, head 56f0b6b), built under
  HANDOFF-161. CI was green on all 22 checks.
- **Your branch:** `claude/spec-059-verify-base`. It is that head plus one commit that
  advances the spec to verify and adds this handoff.

SPEC-059 implements DEC-069: `OpcodeList2` `GainMap` (OpcodeID 9) is parsed and applied
at stage 2 on both develop paths, and a new public `apply_gain_maps` exposes the step.
The Pixel 3a, 4a and 7 Pro, refused until now, develop. The real-file cross-check against
Adobe's `dng_validate` is SPEC-060's, not yours.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-059-gainmap-parse-apply-on-mono-and-colour-paths-with-the-analyt.md`
  (Design §1–8, AC1–AC9).
- **Decision:** `decisions/DEC-069-opcodelist2-gainmap-applies-at-stage-2-on-both-develop-paths.md`.
  §2 is the rule, and every clause of it should map to a test that can reach it (the
  `every-rule-clause-maps-to-a-reachable-test` signal in `guidance/signals.yaml`).
- **Also relevant:** DEC-049 and DEC-053 (required processing and stage primitives),
  DEC-051 (the error-kind table), DEC-058 §5 (`DEVELOP_VERSION`), DEC-060 (zero-allocation
  scratch develop), DEC-061 and DEC-068 (per-position and colour-pattern black).
- **Build handback:** HANDOFF-161's `## Handback`, which has the mutant table, the gates
  and the deviations. Check each claim yourself.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them.
  - `just test` already passes `--all-features`.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.
    Red must come from the target assertion, not from an unrelated error.

## What to check hard

1. **The rule (DEC-069 §2), clause by clause.** Read `GainMaps` and the parser against
   the decision. The clauses are:
   - pixel-centre sampling, `(r + 0.5) / H`, over the `ActiveArea`-relative stage-2
     bounds;
   - index clamping at both map edges, with fraction 0;
   - bilinear interpolation, rows then columns, in `f64`;
   - the clip at white, and rounding to `u16` after each map, in list order;
   - an empty area meaning the whole image, and intersection with the image;
   - the pitch phase anchored at the pre-intersection `Top` and `Left`;
   - `Plane = 0` only, using map plane `min(Plane, MapPlanes − 1)`;
   - optional maps applied.

   For each clause, name the test that would go red without it.
2. **The oracle is independent.** `tests/gain_map.rs` claims its oracle never calls the
   crate. Confirm it. Recompute at least two of the AC3 hand values yourself.
3. **The "pre-gained twin" (AC4, AC5).** The mono and colour tests compare against a file
   with black 0 and white 65535 whose raw window holds the oracle's gained stage 2. Check
   this proves the gain is applied **at stage 2 in `ActiveArea` coordinates**, and is not
   satisfied vacuously. For example, would a gain indexed in raw (not active-area)
   coordinates, or applied after the crop, still pass? The fixture needs a non-zero
   `ActiveArea` origin and a crop for that.
4. **Re-run the build's mutants.** The build has seven red and one it calls equivalent:
   "no clip at 1", which it says is equivalent because the saturating `f64` to `u16`
   cast clips. Decide whether that is truly equivalent, or whether a value between
   65535.5 and 65536 (or the order of clip and round) distinguishes them.
5. **Try to beat the tests with mutants of your own.** For example:
   - sampling at corners, dropping the `+ 0.5`;
   - normalising by the crop size instead of the `ActiveArea` size;
   - clamping only the low edge;
   - interpolating columns first in `f32`;
   - anchoring the pitch phase after the intersection;
   - applying only the first `GainMap` in list 2;
   - skipping optional maps;
   - one rounding for all maps instead of one per map;
   - accepting a 17th map;
   - an off-by-one in the parameter-size check.

   If a reasonable mutant survives and a small test closes it, add that test on your
   branch.
6. **The two refusal-test rewrites.**
   - `tests/plane_tiles.rs::pixel_files_develop_with_gainmap_applied` was approved by the
     orchestrator.
   - `tests/develop_options.rs::develop_into_output_is_unchanged_on_held_files`'s
     `PIXEL_REFUSED` loop was **not separately approved**.

   For each, confirm the old assertion's intent can only have changed because of
   DEC-069, that nothing else in either test was weakened, and that every other held
   file's assertion is untouched.
7. **Public API and errors.**
   - `apply_gain_maps`, `Error::TooManyGainMaps` and the new `Opcode::GainMap` variant are
     what DEC-069 expects.
   - `Error::GainMapImageWrongLength` is **not named in DEC-069**. Is it the right shape
     (compare the other stage primitives' wrong-length errors)? Is it recorded in DEC-051's
     later additions, the error-kind table and the CHANGELOG? Does it need a line in
     DEC-069?
   - New variants are appended last. Check semver-check is clean.
8. **No collateral change.**
   - Files with no `GainMap` must keep their output and speed: every existing pin holds,
     and only the four `DEVELOP_VERSION` pins move.
   - The mono scratch develop still allocates nothing, with a `GainMap` present as well.
   - Threaded colour output equals single-thread output with a `GainMap`.
9. **Provenance.** DEC-069 calls reading the Adobe SDK class 4. The ledger row must
   record the Adobe licence agreement's terms, and no SDK code may be copied. Spot-check
   for that.
10. **Gates:**
    - `just test` with the corpus;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`;
    - a short `opcode` fuzz run.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants.
- **Commit this handoff's filled handback** to `claude/spec-059-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-059-build`.
- **A punch-list fix** is fine on your branch if it is tests or docs only. Anything that
  would change non-test `src/` behaviour goes in the punch list.

## Out of Scope

- SPEC-060: the Pixel output pins, the `dng_validate` cross-check and `scripts/sdk-stage2.sh`.
- The max-black normalisation follow-up (DEC-069 §Context).
- The optional `WarpRectilinear` on the Pixels.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your
    scratchpad UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

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
