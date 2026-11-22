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
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 15986361           # REAL combined count — what cost-audit reads
  estimated_usd: 11.52             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 22
  branch: claude/spec-085-verify-base
  pr: null                         # verify returns a verdict; PR 306 is under review
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "APPROVED at 2267a34 (PR 306 head). FLOOR taken before the handback commit, from transcript 1e544d8e-4a30-4c65-b85b-b51b11836719.jsonl (78 unique message.ids, all claude-opus-5-5, no subagents): input 160 + cache-creation 220,944 (1-hour) + cache-read 15,707,163 + output 58,094. Re-sum at ship." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-22
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

*Filled in by the receiving agent (claude-opus-5-5, cold verify, 2026-11-22). I did not
build SPEC-085. Every number below is my own measurement on this machine.*

### Execution notes

- **Branch / PR:** verdict committed to `claude/spec-085-verify-base` (no PR, by
  instruction). Under review: https://github.com/jysf/irradiance/pull/306, head `2267a34`.
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** Yes. AC1 to AC7 hold against the spec as designed in
  PR 305 (`origin/main`), and no pre-registered bound was touched.
- **For `verify`:** ✅ **APPROVED at `2267a34`** (PR 306's head; `979820b` on this branch
  adds only the handoff and the cycle move). No ship-blockers. Four follow-ups, FU-1 to FU-4,
  none of which holds the spec.

### Evidence, check by check

1. **The rule against the SDK source.** I read `dng_color_spec.cpp` (the constructor,
   `FindXYZtoCamera_SingleOrDual`, `SetWhiteXY`, `NeutralToXY`) and `dng_string::operator==`.
   - **Signature:** the SDK compares two `dng_string`s, and `dng_string::Set` stores an empty
     string as null. So absent and empty are equal, and `resolve_calibration`'s
     `unwrap_or_default` slice equality matches that. Both are case-sensitive.
   - **`CameraCalibration2`:** used only when the dual model runs. On the single path the SDK
     sets `fCameraCalibration2 = fCameraCalibration1`, and ours checks `CC_2` only under
     `applies && dual`.
   - **The fold:** the SDK computes `fColorMatrix_i = fAnalogBalance * fCameraCalibration_i *
     fColorMatrix_i` before the lerp. Ours computes `fold(k_i, cm_i)` with
     `k_i = AB · CC_i`, so it is the same product.
   - **The ForwardMatrix path:** the SDK computes
     `FM · inv(diag(inv(AB·CC(g)) · cameraWhite)) · inv(AB·CC(g))`, with `CC(g)` lerped
     separately from the matrix. `fm_path_cam_to_xyz` has the same structure. The SDK's
     `cameraWhite` comes from `colorMatrix · XYZ(white)` (normalised and pinned to
     [0.001, 1]), where ours uses `AsShotNeutral` directly. That difference is inside the
     residual AC4 measures and is not a defect.
   - **The loop:** both start at the SDK's `D50_xy_coord()` = (0.3457, 0.3585). Both use a
     tolerance of `0.0000001` on `|Δx| + |Δy|`, take `next` on convergence, and take the
     midpoint on the last of 30 passes. Ours runs passes 1..=30 and takes the midpoint at
     pass 30; the SDK runs 0..30 and takes it at 29. These are the same pass.
   - **The g inversion:** the SDK's `g` weights illuminant 1 after swapping so that
     `T1 < T2`. Ours is `clamp((1/T1 − 1/T) / (1/T1 − 1/T2), 0, 1)`. That expression equals
     `1 − g_sdk` when `T1 < T2`. It is also correct without any swap when `T1 > T2`: it gives
     0 at `T = T1`, 1 at `T = T2`, and the clamp sends `T` beyond either end to that end's
     illuminant.
   - **Empirical check of the swap:** the Pixel 3a, whose illuminants are 20 then 17, comes
     out at 6 codes in AC4, and my probe of a swapped-order fixture behaves as reasoned.
2. **Bit identity under the identity calibration.**
   - `fold` returns `CM` itself when `K == IDENTITY`. The ForwardMatrix path passes
     `k = None` and returns `FM · wb`, with `wb` being the old `D` built from the rationals.
   - The identity test compares `f64`s, so `2/2` gives exactly 1.0 and counts as identity,
     and `0/-1` gives -0.0, which equals 0.0 and also counts as identity. Both give the
     correct result.
   - AC3's identity rows (single CM, single FM, dual CM, dual FM) do reach the ForwardMatrix
     path where they claim to.
   - **The gap, FU-2:** the fixture's `AsShotNeutral` is (1/2, 1, 1/2). For those values
     `den/num` and `1/(num/den)` are equal, so the shortcut cannot be observed on this
     fixture. My mutant (n), which removes it, survives the whole suite (see 9).
3. **AC4 reproduces from scratch.**
   - I ran `scripts/sdk-colour.sh` cold into my scratchpad. It built the harness from 69 SDK
     objects and wrote five matrix files. The 7 Pro's white is xy (0.319745, 0.333331), as
     DEC-076 says.
   - **My numbers (max |Δ| in codes): 7 Pro 6, 3a 6, 4a 1, Canon 2, Nikon 5.** All 18 deltas
     on every file are identical to the build's table. The bound of 12 holds.
   - **Independence of the expected side:** `tests/colour_sdk.rs` uses its own transcribed
     S and B constants, its own `mm`, its own round-half-up and its own clamp. It never calls
     `src/color.rs`.
   - **Set means must run:** with one matrix file removed the lane fails ("every matrix file
     must exist"). With the corpus pointed at a missing directory it fails with "MISSING …
     must run". Unset, it prints a visible `SKIP`, and `corpus-status` prints
     `sdk-colour: SKIP`.
   - **Output directory:** the script refuses `./tmp-out-inside` and the main checkout's
     `irradiance/x` ("output directory is inside the repository") and creates nothing.
4. **The independent reference in `tests/color_oracle.rs`.**
   - It imports only the public API and never reads `src/color.rs`.
   - I checked its `ref_fold`, `ref_dual`, `ref_weight_and_passes` and `fm_cam2xyz` line by
     line against DEC-076 §4–5, and they match.
   - The diff against `origin/main` removes only the old reference helpers (`ref_weight` and
     the old `reference_matrix` body). No assertion line was removed or weakened in the
     three dual fixtures.
   - **AC2 (f) weights, recomputed independently in Python** (plain floats, my own cofactor
     inverse): 3-pass 0.5202685520213217 against the pinned …226, and loop
     0.6708871184066119 against the pinned …116. Both agree to 1e-15.
   - **FU-1:** this fixture never meets the 1e-7 tolerance. Its per-pass step shrinks by
     about 0.76 a pass, is still 3.5e-6 at pass 30, and the result is the pass-30 midpoint.
     So `ac2f_the_loop_converges…` actually pins the midpoint branch, not convergence.
5. **The ForwardMatrix-path trap.**
   - AC2 (b) and (e) use non-diagonal `cc_skew_1` and `cc_skew_2`. AC2 (b) also asserts that
     a diagonal `K` on a single-illuminant FM file moves the output by at most 1 code, which
     shows the trap directly.
   - AC2 (c) asserts `assert_ne!(got, no_ab)` on both rows, so `AnalogBalance` is observably
     applied wherever the test claims it is. On the FM row it acts through `g` (dual
     fixture), as the handback says. Mutant (e) goes red on exactly this test.
6. **The two approved deviations.**
   - **The re-pinned weight:** 0.6708871184066116 follows from the DEC-076 §5 loop (my
     independent recompute above), and the 3-pass value 0.5202685520213226 is still asserted
     as the contrast. Accepted.
   - **Singular `ColorMatrix1`. Ruling: accepted, with the CHANGELOG wording corrected at
     ship (FU-3).** I probed it with a throwaway test through all five public entries
     (`to_srgb`, u16, u16 threaded, f32, f32 threaded), wrapped in `catch_unwind`. Results:
     - **Never a panic, the entries always agree, threaded equals single, every f32 output
       is finite.** Every `inv` returns an `Option` and is handled, and `quantize` maps NaN
       to 0, so no NaN can reach `dst`.
     - Singular `CM1` with valid `CM2` (A then D65): `Ok`.
     - Singular `CM2` with valid `CM1`: `Ok`.
     - Singular `CM2` with `T1 > T2`: `Ok`.
     - **Singular `CM1` with `T1 > T2`: `MissingColorCalibration`.**
     - `CM2 = −CM1`, so `X(g)` is singular at g = ½: `Ok`, because the degenerate step falls
       back to the single-illuminant path.
     - A singular applied `CC_1` or `CC_2` on a dual file: `Ok`.
     - A singular `K` on the FM path: `MissingColorCalibration`.
     - A sweep of 990 perturbed dual fixtures (CM path and applied-`CC_2` path): 74
       `MissingColorCalibration`, 916 `Ok`, no panics.

     The behaviour matches the SDK, which inverts only the matrix at the `g` it lands on,
     and every refusal is typed. But it is data-dependent. Whether a singular matrix refuses
     depends on which end the scene weight settles at, so a singular `ColorMatrix1` still
     refuses when it belongs to the warmer, higher-numbered illuminant. A singular **applied
     calibration** develops too when its product is unused, which is looser than a literal
     reading of DEC-076 §3's "a singular product".

     The CHANGELOG body does carry the condition ("when it settles fully on illuminant 2").
     Its bold headline, "A dual-illuminant file whose `ColorMatrix1` is singular now
     develops", over-generalises, and it says nothing about singular calibrations. That is a
     documentation precision issue (FU-3); whether to refuse belongs to SPEC-087.
7. **Shapes and refusals (AC3).**
   - `assert_refused_on_every_entry` covers five entries (`to_srgb`, u16, u16 threaded, f32,
     f32 threaded) with sentinel `dst`s, on all five rows (`CC_1` with 8 entries, a zero
     denominator, `AB` with 2 entries, an `AB` zero numerator, a non-positive reference
     neutral).
   - The non-positive neutral row runs on the FM path, where `R` exists.
   - The two ill-shaped-but-unused rows develop `Ok`.
   - Refusal happens in `resolve_calibration` / `fm_path_cam_to_xyz`, both inside
     `build_camera_to_srgb_linear`, which every entry calls before any write.
8. **Parse (AC1).**
   - `signature_bytes` checks the type (ASCII/BYTE), `MAX_TAG_VALUES` and the existing
     bounds-checked `payload`, then cuts at the first NUL.
   - Writer fixtures cover: both byte orders; ASCII and BYTE; an inline value and an
     external one; no NUL; wrong types (LONG and SHORT on the rationals, and a wrong type on
     each signature) costing only their field; absent tags being `None` and not malformed.
   - The exiftool comparison has a per-field perturbation red-proof on the held files and on
     replayed lines. Mutant (h) turns it red.
   - The `ifd` fuzz target calls `container.sensor()`, so it drives the new reader. I ran it
     for 61 s (4.26M runs) and `develop` for 61 s (1.93M runs); both were clean.
9. **Mutants.** Each ran in its own copy of the tree with its own `CARGO_TARGET_DIR`. The
   control ran first and is green. The suite is `--lib`, `color_oracle`, `colour_sdk` (SDK
   lane set) and `colour_metadata_oracle`. Mutants that survived that set also ran through
   the whole corpus suite.

   | Mutant | Goes red on | AC4 (max \|Δ\|) |
   |---|---|---|
   | control | nothing | 6 / 6 / 1 / 2 / 5 |
   | (a) calibration ignored | `ac2a` to `ac2e`, three AC3 rows, threaded, AC4 | 7 Pro **1161**, 4a 23, Canon 3 |
   | (b) 3-pass loop | `ac2f`, `ac2c`, `ac2d`, `ac2e`, the weight unit test, the degenerate unit test, AC4 | 7 Pro **148**, 3a **39** |
   | (c) lerp of the factors | `ac2d`, `ac2e`, AC4 | 7 Pro **23** |
   | (d) signature rule always applies | `ac3_mismatched…`, `ac3_an_unused…`, `ac2c` | unchanged |
   | (e) `AnalogBalance` dropped | `ac2c`, threaded | unchanged |
   | (f) FM path as `FM(g) · D` | `ac2b`, `ac2e`, the AC3 non-positive-neutral row, threaded | unchanged |
   | (g) `CC_2` checked on the single path | `ac3_an_unused_ill_shaped…` | unchanged |
   | (h) signature not cut at NUL | both `ifd` NUL tests, the exiftool held-file test | unchanged |
   | (i) the loop starts at g = 0, then converges | `ac2f`, the three old dual fixtures, the weight and degenerate unit tests | unchanged |
   | (j) tolerance 10× looser | Tier A survives; **Tier B red**: Pixel 4a and 3a colour pins | unchanged |
   | (j2) tolerance 10× tighter | Tier A survives; **Tier B red**: Pixel 4a and 3a colour pins | unchanged |
   | (k) midpoint removed | `interpolation_weight_matches_dng_neutral_to_xy` (unit test) | unchanged |
   | (l1) f32 entry on the uncalibrated transform | the AC3 non-positive-neutral row | unchanged |
   | (l2) threaded entries on the uncalibrated transform | threaded = single, the AC3 refusal row | unchanged |
   | (m) `R` from `asn` without `inv(K)` | `ac2b`, `ac2c`, `ac2e`, the AC3 refusal row, threaded, AC4 | 4a **597**, Canon **225** |
   | (n) FM identity shortcut removed | **survives the whole corpus suite** | unchanged |

   AC4 columns are 7 Pro / 3a / 4a / Canon / Nikon; "unchanged" means equal to the
   control.

   - (a) to (g) reproduce the build's table and the probe's numbers (1161, 148/39, 23).
   - (j) and (j2) are caught only by the held-file pins (FU-1).
   - (l1) is caught at Tier A only through a refusal row: no Tier-A test compares calibrated
     **f32 values** with the reference. The Tier-B f32 pins cover the held Pixels (FU-4).
   - (n) is near-equivalent. It differs only where `den/num ≠ 1/(num/den)` in the last bit,
     and no fixture or held file has an identity calibration on the FM path with such a
     neutral (FU-2).
10. **Files outside the plan.** `src/develop.rs` and `src/plane.rs` (five `None` fields
    each), `examples/corpus-status.rs` (the lane's SKIP/present line), and
    `tests/black_levels.rs`, `develop_options.rs`, `orientation_tiles.rs`, `warp_alpha.rs`
    (the `DEVELOP_VERSION` pin, 7 → 8, message extended). All are mechanical and nothing was
    weakened.
11. **Pins (AC5).**
    - Exactly six pins moved, all in `gain_map_corpus.rs` `PINNED`: the 4a, 3a and 7 Pro,
      each for `develop_color_into` and `develop_color_f32_into`. The four
      `DEVELOP_VERSION` pins moved too.
    - All mono pins hold, since both `just test` runs are green.
    - The Canon and Nikon have only mono or SDK-bounded comparisons, no colour pins.
    - Threaded equals single in `a_calibrated_threaded_develop_equals_single_thread`, and in
      my probe across all cases.
    - **Cause attribution, spot-checked through AC4 under my own mutants:**
      - **Pixel 3a:** ignoring the calibration leaves it at 6, with the same deltas, while
        the 3-pass loop returns it to 39. So the loop alone moved it, as claimed.
      - **7 Pro:** it moves under both mutants (1161 and 148), so both causes moved it.
12. **Release surface and provenance.**
    - `DEVELOP_VERSION` is `"8"`, with a doc paragraph.
    - The CHANGELOG `[Unreleased]` entry names all five files and both causes, apart from
      FU-3's headline.
    - `semver-check`: "no semver update required" (additive).
    - The ledger row cites DNG 1.7.1.0 Chapter 6 as class 1 and SDK `dng_color_spec.cpp` as
      class 4, read, with **no code copied**. The code is not a transliteration: the
      structure, names and `Option` fallbacks are irradiance's own.
    - `scripts/sdk-camera-to-pcs.cpp` is irradiance's own harness of about 35 lines, built
      only by `sdk-colour.sh`. No `.github/` change, so it is never built in CI.
    - The 22-file diff contains no SDK bytes, outputs, corpus bytes or binaries.
13. **The build's cost figure.** 33.19M tokens and $22.58 over 25 minutes is plausible.
    - That USD at opus-5-5 rates implies mostly cache reads, as expected on an L build.
    - This verify, with two corpus suites, 16 mutant builds and the SDK harness, spans
      22 minutes in its own transcript, so a 25-minute build is credible.
    - It is labelled a floor taken before the handback commit, so the orchestrator should
      re-sum it at ship.
14. **Gates, all green on this branch's head (`979820b`, whose code is `2267a34`):**
    - `just test` with the corpus, without `IRRADIANCE_SDK_COLOUR_DIR` (EXIT 0) and with it
      (EXIT 0);
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit` (14 scripts, no mute-death), `name-length-check`,
      `decisions-index --check`, `semver-check`;
    - fuzz on `ifd` and `develop`, 61 s each, clean.

### Findings

No ship-blockers.

- **FU-1:** AC2 (f)'s fixture ends at the 30-pass midpoint and never meets the 1e-7
  tolerance, so the name `ac2f_the_loop_converges…` overstates what it pins. A tolerance
  10× off survives Tier A and is caught only by the Tier-B Pixel pins.
  Suggest: a Tier-A dual fixture that converges by tolerance in a known number of
  passes, with the count asserted.
- **FU-2:** DEC-076 §4's bit-identity clause on the ForwardMatrix path is untested.
  Removing the identity shortcut survives the whole corpus suite, because the AC3 identity
  fixture's neutral (1/2, 1, 1/2) makes `den/num` equal `1/(num/den)`. Suggest: add an
  identity row whose `AsShotNeutral` rationals differ in the last bit between the two forms.
- **FU-3:** the CHANGELOG's bold headline "A dual-illuminant file whose `ColorMatrix1` is
  singular now develops" over-generalises.
  - It refuses when `ColorMatrix1` is the higher-temperature illuminant.
  - A singular applied `CameraCalibration_i` whose product is unused also develops, which is
    unstated.
  - Fix the wording at ship (docs only). Whether a singular unused matrix or calibration
    should refuse goes to SPEC-087.
- **FU-4:** no Tier-A test compares a calibrated develop through `develop_color_f32_into`
  with the reference. The f32 entry on the uncalibrated transform is caught only by a
  refusal row at Tier A, and by the Pixel f32 pins at Tier B.

### Cost self-report

- **Tokens (total):** 15,986,361, a floor taken before the handback commit (input 160 + cache creation 220,944, all 1-hour + cache read 15,707,163 + output 58,094)
- **Estimated USD:** 11.52 (opus-5-5: input $5 and output $25 per MTok; cache read ×0.10; 1-hour cache write ×2.0)
- **Duration (minutes):** 22, the transcript's timestamp span
- **Source of the number:** this session's own transcript, `1e544d8e-4a30-4c65-b85b-b51b11836719.jsonl`, found by the scratchpad UUID and measured per docs/cost-tracking.md "Measuring a cycle": 78 unique message.ids, all claude-opus-5-5, no subagents, deduplicated by message.id with each message's largest output_tokens (`basis: transcript`). Re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the verify. Branch note: the local branch name
  `claude/spec-085-verify-base` was already checked out in another worktree
  (`opcode-demosaic-review-fixes-f9348f`), so I worked on a local branch tracking
  `origin/claude/spec-085-verify-base` and pushed to that remote branch.
- **Follow-up work identified:** FU-1 to FU-4 above. FU-3's refusal question goes to
  SPEC-087, which already owns malformed and singular interpretation inputs.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. "AC2 (f) asserts the converged weight" assumed the fixture converges;
   checking that independently showed it ends at the cap. A handoff could ask the verifier to
   record the pass count of every Tier-A loop fixture.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-076 §3's "a singular product is `MissingColorCalibration`" should say "at the weight
   actually used". The convergent loop makes every singular-matrix refusal depend on the
   scene weight, as in the SDK, and the decision text does not say so.

3. **If you did this task again, what would you do differently?**
   — Run every surviving mutant through the full corpus suite from the start rather than a
   Tier-A subset first. Two of my three Tier-A survivors were caught only at Tier B.
