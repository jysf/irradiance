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
  id: HANDOFF-179
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-22
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-085

project:
  id: PROJ-002
  stage: STAGE-027
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

# HANDOFF-179: VERIFY: apply CameraCalibration and AnalogBalance on the colour develop (SPEC-085)

## Delegation Summary

You are the **verifier** for SPEC-085. Review it cold: you did not build it.

- **Under review:** PR 306 (`feat/spec-085-camera-calibration`, head `2267a34`), built
  under HANDOFF-178. CI is green on all 22 checks.
- **Your branch:** `claude/spec-085-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-085 implements DEC-076:
- five new IFD0 `Sensor` fields (the two calibrations, `AnalogBalance`, the two
  signatures);
- the calibration on all four colour entries under the signature rule, with the SDK's
  lerp of the folded `AB·CC·CM` products on the ColorMatrix path and `K(g)` with the
  reference neutral on the ForwardMatrix path;
- a `NeutralToXY` loop that starts at D50 and runs to convergence (at most 30 passes);
- `DEVELOP_VERSION` 8, and a new env-gated SDK colour lane with a pre-registered
  bound of 12 codes.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-085-apply-cameracalibration-and-analogbalance-on-the-colour-deve.md`.
  Judge the bounds and ACs against the spec **as designed in PR 305**
  (`git show origin/main:<spec path>`), not as edited on the build branch.
- **Decisions:** DEC-076 (the rule and its probe table), DEC-034 (the weight loop it
  amends), DEC-033 (`f64` basic ops, no FMA), DEC-012 (interpretation tags cost their
  field), DEC-049 (`Sensor` is `#[non_exhaustive]`), DEC-069 §6 (the SDK stays local).
- **Build handback:** HANDOFF-178's `## Handback`, with the AC4 table and 18 deltas per
  file, the six re-pins and their causes, the AC6 mutant table and two approved
  deviations. Check each claim yourself.
- **The SDK:** objects at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/obj`,
  headers under `.../tools/dng_sdk/src/dng_sdk_1_7_1/dng_sdk/source`. Its
  `dng_color_spec.cpp` is the class-4 reference for DEC-076 §2, §4 and §5.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The rule against the SDK source.** Read `dng_color_spec.cpp` yourself (the
   constructor, `FindXYZtoCamera_SingleOrDual`, `SetWhiteXY`, `NeutralToXY`) and compare it
   with `src/color.rs`:
   - the signature equality, with absent as empty;
   - `CameraCalibration2` used only on the dual path;
   - the fold `AB·CC_i·CM_i` before the lerp;
   - the ForwardMatrix path's `K(g)`, the reference neutral and `diag(1/R)`;
   - the loop's start, tolerance, 30-pass cap and midpoint.

   The SDK's `g` weights illuminant 1 and ours weights illuminant 2. Confirm the
   inversion is handled, including the SDK's swap when `T1 > T2` (the Pixel 3a's
   illuminants are 20 then 17).
2. **Bit identity under the identity calibration.** Read `fold` and the ForwardMatrix
   path. When `AB·CC` is exactly the identity, is the output bit-identical to the old
   code at the same `g`, including the old `D` from the rationals (`den/num`)? Is
   "exactly the identity" tested on rationals that are exactly 1 and 0, and what happens
   with `2/2` or `-0`? Do AC3's identity rows actually reach the FM path?
3. **AC4 reproduces from scratch.** Run `scripts/sdk-colour.sh` cold into a new
   scratchpad directory, outside the repository, then `tests/colour_sdk.rs` with
   `IRRADIANCE_SDK_COLOUR_DIR` set. Your numbers should equal the handback's 18 deltas
   per file, within the ≤ 12 bound.
   - Is the expected side independent of `src/color.rs`? It should use its own
     XYZ(D50)→sRGB constants, rounding and clamp.
   - Does set-means-must-run hold (SPEC-079): a missing matrix file or a missing corpus
     file fails, and unset prints a visible SKIP?
   - Does the script refuse an output directory inside the repo?
4. **The independent reference in `tests/color_oracle.rs`.** It must not read
   `src/color.rs`.
   - Check its new transform and loop against DEC-076 line by line.
   - The three dual fixtures were re-derived. Confirm that only the reference moved, and
     that none of their assertions were weakened.
   - AC2 (f) asserts both the 3-pass and the converged weight. Recompute one weight
     independently.
5. **The FM-path trap.** AC2 (b) and (e) must use a non-diagonal calibration, so that `K`
   is observable. The handback says AC2 (c)'s FM row uses the dual fixture, because a
   diagonal `K = AB` is unobservable on a single-illuminant FM file. Is `AnalogBalance`
   really observable wherever the test claims it is?
6. **The two approved deviations.**
   - The re-pinned weight: check it follows from the convergent loop and the 3-pass
     contrast is still asserted.
   - The singular-`ColorMatrix1` case that now develops:
     - confirm no panic, no NaN and no division by zero is reachable;
     - try a singular `ColorMatrix2` with a valid `ColorMatrix1`, and singular
       interpolated matrices at an intermediate `g`. What happens, and is it typed?
     - Rule whether the CHANGELOG states the behaviour change accurately.
7. **Shapes and refusals (AC3).**
   - Every `MissingColorCalibration` row reaches all four colour entries (`u16`, `f32`,
     both threaded) and leaves `dst` untouched.
   - An ill-shaped calibration that the rule does not apply develops `Ok`.
   - A non-positive reference neutral is refused, not divided by.
8. **Parse (AC1).** The ASCII/BYTE reader is bounds-checked (`MAX_TAG_VALUES`, the
   payload read) and cuts at the first NUL. A wrong-type tag costs only its field. The
   exiftool comparisons on held files have perturbed-value red-proofs. Also check what
   the `ifd` fuzz target exercises.
9. **Mutants.** Re-run the build's (a) to (g). Then try at least:
   - the signature comparison without cutting at NUL;
   - the loop starting at g = 0 but converging;
   - the convergence tolerance off by 10×;
   - the midpoint step removed;
   - the f32 entry or the threaded entry left on the old transform;
   - `R` computed from `asn` without `inv(K)`.

   Record what goes red, or argue equivalence.
10. **The files outside the plan.** `src/develop.rs`, `src/plane.rs`,
    `examples/corpus-status.rs`, and `tests/black_levels.rs`, `develop_options.rs`,
    `orientation_tiles.rs` and `warp_alpha.rs` changed by a few lines each. Confirm each
    change is mechanical (new fields, the version pin, the corpus-status lane) and that
    no assertion was weakened (`git diff origin/main -- tests/ src/`).
11. **Pins (AC5).** Exactly the six Pixel colour pins moved, plus the `DEVELOP_VERSION`
    pins. Every mono pin holds. Threaded equals single-thread. Spot-check the cause
    attribution for one file.
12. **Release surface and provenance.**
    - `DEVELOP_VERSION` is `"8"` and documented.
    - The CHANGELOG `[Unreleased]` entry names the files and both causes.
    - `semver-check` is additive.
    - The provenance-ledger row cites DNG 1.7.1.0 Chapter 6 as class 1 and the SDK
      `dng_color_spec.cpp` as class 4, with no code copied.
    - `scripts/sdk-camera-to-pcs.cpp` is this repo's own harness and is never built in
      CI.
    - No SDK bytes, outputs, corpus bytes or harness binary are committed.
13. **The build's cost figure.** The handback reports 33.19M tokens over a 25-minute
    span, and calls it a floor. Note whether that looks plausible; the orchestrator will
    check it at ship.
14. **Gates:**
    - `just test` with the corpus, without and with `IRRADIANCE_SDK_COLOUR_DIR`;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
      `semver-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including your own AC4 numbers and a table of
  your own mutants.
- **A ruling on item 6's singular-matrix behaviour.**
- **Commit this handoff's filled handback** to `claude/spec-085-verify-base` and push it.
  Do not open a PR, and do not push to `feat/spec-085-camera-calibration`.
- **Fixes:** a punch-list fix is fine on your branch if it is tests, scripts or docs
  only. Any `src/` change goes in the punch list. A measured number outside a
  pre-registered bound is reported, never fixed by widening the bound.

## Out of Scope

- SPEC-086's and SPEC-087's refusals, including whether a singular unused
  `ColorMatrix_i` should refuse.
- Triple-illuminant profiles, Robertson CCT, `NormalizeForwardMatrix`.
- Running the SDK in CI, vendoring it or committing its outputs.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript (found by your scratchpad
    UUID, deduped by `message.id`, `basis: transcript`);
  - `to_agent` and `handback.agent` corrected to the model that actually ran.

  Then fill the `## Handback` section.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

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
