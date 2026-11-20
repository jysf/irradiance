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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 16261949          # REAL combined count — what cost-audit reads
  estimated_usd: 11.32             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 24
  branch: claude/spec-059-verify-base
  pr: 267
  completed_at: 2026-11-19          # YYYY-MM-DD
  notes: "PUNCH LIST, tests and docs only, fixed on the verify branch at 2e1a9c8: a new colour test kills the no-intersection mutant V24, plus stale colour-path docs and a DEC-069 consequence line. Source at 56f0b6b correct. 27 verifier mutants: 22 red, V24 closed, 4 equivalent. Cost is a floor before the handback commit: transcript 4ad4774a-533b-4a3a-a07b-1d8053bca58a.jsonl, 83 unique message ids, all claude-opus-5-5, no subagents; input 165, cache-creation 202,095 (1-hour), cache-read 16,007,785, output 51,904."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-19
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

- **Branch / PR:** `claude/spec-059-verify-base` (pushed; no PR opened, nothing pushed to `claude/spec-059-build`). Reviewed PR #267 at head 56f0b6b.
  - `2e1a9c8`: one new colour test that closes a surviving mutant, and three stale doc paragraphs plus a DEC-069 consequence line. Tests and docs only; no non-test behaviour change.
  - Then this handback and the timeline's verify line.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes, AC1 to AC9 hold at 56f0b6b. One test gap (V24 below) is closed on this branch.
- **For `verify`:** ⚠ **PUNCH LIST, tests and docs only, already fixed** on this branch at `2e1a9c8`. The source at 56f0b6b is correct. To close it, carry `2e1a9c8` into #267 (cherry-pick onto `claude/spec-059-build`), or fast-forward the build branch to this one.

**The punch:**

- **P1 (tests).** Verifier mutant V24 drops the area's intersection with the image, so `row_end` and `col_end` become the raw `Bottom` and `Right`. It survived the whole suite.
  - On `apply_gain_maps` and the mono path it is equivalent, because every visited sample has `r < H` and `c < W`.
  - On the colour path it is observable. The fused demosaic reflects at the **plane** border, not the `ActiveArea` border, so it reads the masked column just right of the `ActiveArea`. `gained_normalizer` keeps that read ungained only because of the intersection.
  - `ac5`'s fixture cannot see this: its maps end exactly at `W` and `H`, and its crop sits 2 columns inside the `ActiveArea`.
  - Fix: `ac5_area_past_the_image_edge_leaves_reads_outside_the_active_area_ungained`. It uses the ac5 plane and phase maps, with `Bottom` and `Right` at 100 and the crop set to the whole `ActiveArea`. It is green on the real tree, and red on V24 at the target assertion ("a read outside the ActiveArea must not be gained").
- **P2 (docs).** These still said the colour path applies no opcode, now fixed:
  - `develop_color_into`'s `# Errors` ("this path applies none");
  - `to_srgb_linear_into`'s `# Errors` ("a mandatory opcode in any list"). It now also says a composer applies the maps first;
  - `demosaic_into`'s `# Errors` ("this path applies no opcode").
- **P3 (docs).** DEC-069's Consequences now names `Error::GainMapImageWrongLength` next to `apply_gain_maps` and `TooManyGainMaps` (item 7).

**Evidence for the ten "What to check hard" items:**

1. **The rule, clause by clause.** I read `parse_gain_map_params`, `prepare`, `GainMaps::apply`, `gain_at` and `grid_axis` against DEC-069 §2. Every clause is implemented as written. For each clause, the test that goes red without it (mutant IDs refer to the table below):
   - **Pixel centres, `(r + 0.5) / H` over the `ActiveArea` bounds.**
     - V01 (corner sampling) is red on ac2, ac3 (`48640` became `45056`), ac4 and ac5.
     - V02m and V02c (normalising by the crop size) are red on ac4 and ac5 respectively.
   - **Index clamping at both edges, fraction 0.**
     - V03 (no high clamp) is red on the `grid_axis` unit test and ac2.
     - V20 (no clamp at all) is red on ac3 (the corner `50000`), ac2 and ac4.
   - **Bilinear, rows then columns, in `f64`.**
     - V04 (columns first in `f32`) and V04c (rows first in `f32`) are red on ac4 and ac5.
     - The build's swapped-entry-index mutant was red.
     - V04b (columns first in `f64`) survives. It is the same bilinear in exact arithmetic and differs only in the last `f64` ulp, which can move a `u16` only on an exact .5 tie. I count it as equivalent in practice and add no test.
   - **The clip at white, and a round to `u16` after each map, in list order.**
     - V08 (one rounding for the list) is red on ac3 (`[2,2,2,2]` against `[1,1,1,1]`) and ac2.
     - V06 (only the first map) is red on ac2 ("four phase maps in one list"), ac4 and ac5.
     - The clip itself is equivalent (item 4).
   - **An empty area is the whole image, and the area is intersected with the image.**
     - V19 (empty area as nothing) is red on ac3 (`600`), ac2 and ac4.
     - V24 (no intersection) survived, and is closed by P1.
   - **The pitch phase anchored at the pre-intersection `Top` and `Left`.**
     - The build's "anchored at 0" mutant and my V23 (row pitch ignored) are red on ac3 (the pitch-skipped `1000`), ac2, ac4 and ac5.
     - V18 (an empty area anchored at `Top`/`Left` instead of 0) is red on ac3 and ac2.
     - V05 (anchor after the intersection) is **equivalent by construction**. `Top` and `Left` are `u32 ≥ 0`, and intersecting with `[0, H) × [0, W)` can only lower `Bottom`/`Right`. So the post-intersection `Top` equals the pre-intersection one whenever any sample is touched. The clause cannot be observed in these coordinates.
   - **`Plane = 0` only, using map plane `min(Plane, MapPlanes − 1)`.** V15 (plane ignored) is red on ac2 ("Plane 1 applies to nothing"). V16 (last map plane) is red on ac2 ("Plane 0 of a two-plane map").
   - **Optional maps apply.** V07 (skip optional maps) is red on `ac4_optional_gain_maps_apply_too`, on ac1 (an optional malformed map is no longer validated), and on the AC6 malformed row.
2. **The oracle is independent.** `oracle`, `oracle_axis`, `encode` and `normalize` in `tests/gain_map.rs` use only the test's own `Map` struct and `std`. The only `irradiance::` calls in the file are the system under test and the fixture I/O (`write_dng`, `Container::parse`, `unpack_into`). `oracle` walks the area by pitch steps; the crate tests membership. It intersects with `min`, interpolates by its own formula, and shares no helper with `src/`.

   I recomputed the AC3 hand values myself:
   - **Interior `(1, 2)`:** `fr = (1.5/4 − 0.25)/0.5 = 0.25` and `fc = (2.5/4 − 0.25)/0.5 = 0.75`. Column 0 is `1.25·0.75 + 0.5·0.25 = 1.0625` and column 1 is `1.5·0.75 + 2·0.25 = 1.625`. So `g = 1.0625·0.25 + 1.625·0.75 = 1.484375`, and `32768 · 1.484375 = 48640` exactly. ✓
   - **Corner `(0, 0)`:** `fr = fc = −0.25`, which clamps to index 0, so `g = 1.25` and `40000 · 1.25 = 50000`. ✓
   - **Rounding per map:** `1 · 1.4` gives `⌊1.9⌋ = 1`, and then 1 again. One rounding of `1.96` would give 2. ✓
3. **The pre-gained twin proves stage-2 placement in `ActiveArea` coordinates. It is not vacuous.**
   - Both fixtures have a non-zero, odd `ActiveArea` origin (left 3, top 1) and a `DefaultCrop` at (1, 1) that is smaller than the `ActiveArea` on both axes.
   - The twin's levels are the identity (black 0, white 65535). Its `ActiveArea` window holds the oracle's gain over the test-side normalised stage 2.
   - Each placement error is red on the twin comparison alone:
     - V11, the mono gain indexed in raw coordinates, is red on ac4 (`develop_into`).
     - V12, the same on colour, is red on ac5.
     - V13, the mono gain in crop coordinates (indexed after the crop), is red on ac4.
     - V02m and V02c, normalising by the crop size, are red on ac4 and ac5.
   - Each test also asserts that the gain changed the output against the un-gained file.
   - ac4's `warp = true` rows hold the before-warp position. I did not mutate the warp order separately.
4. **The build's "no clip" equivalence holds.** V14a (`clamp(0, 1)` changed to `max(0)`) and V14b (no clamp at all) are both green.
   - **Proof.** With the clip, `x ≤ 1` gives `x·65535 + 0.5 ≤ 65535.5`, whose floor is 65535. Without it, a value in `[65535.5, 65536.5)` floors to 65535 or 65536, and Rust's saturating `as u16` takes both to 65535. Larger values saturate too.
   - **Below 0.** A negative gain is finite, so the parser admits it. Its floor is ≤ 0, and the cast takes it to 0, the same as the low clamp.
   - **No NaN.** Every input is finite, so no NaN can reach the cast.
   - **Order.** Clip-then-round equals round-then-clip, because floor is monotone and the saturation is a clip. So no value between 65535.5 and 65536, and no ordering of clip and round, can tell them apart.
   - The build's wrapping-cast form was red, as reported.
5. **My own mutants:** the table below. 27 mutants: 22 red, 1 survivor closed by P1, and 4 equivalent (V04b, V05, V14a, V14b).
6. **The two refusal-test rewrites.**
   - **`plane_tiles::pixel_files_develop_with_gainmap_applied`.** It asserted `UnsupportedMandatoryOpcode { id: 9 }` with kind `Unsupported`, which is exactly the refusal DEC-069 §1–2 removes. It now asserts `Ok` on both paths, keeps the corpus SKIP, and adds an `opcode_list_2.is_some()` guard. Nothing else in the file changed except the module doc.
   - **`develop_options::develop_into_output_is_unchanged_on_held_files`** (not separately approved).
     - The `PINNED` loop, which covers every other held file's MD5 and dimensions, is byte-for-byte unchanged. The diff touches only the `DEVELOP_VERSION` pin and the Pixel loop.
     - The old Pixel assertion was "both entries give the same refusal, and it is the opcode-9 refusal". Its only DEC-069-sensitive part is the refusal. The new assertion keeps the cross-entry half in a stronger form: both entries develop, with equal MD5s.
     - Its intent changed only because of DEC-069, and nothing was weakened. **I confirm the rewrite.**
7. **Public API and errors.**
   - `apply_gain_maps(stage2, width, height, opcode_list_2) -> Result<usize, Error>`, `Error::TooManyGainMaps { count }` and `Opcode::GainMap { .. }` (the parameter block's fields, then `gains`) are what DEC-069 §3 and §4 describe.
   - `Error::GainMapImageWrongLength { expected: u64, actual: usize }` has exactly the shape and Display pattern of `FixBadPixelsPlaneWrongLength`, `WarpSourceWrongLength` and `DemosaicSourceWrongLength`, and is a `CallerError` like them. It is recorded in DEC-051's later additions, `tests/error_kind.rs` (68 variants) and the CHANGELOG. DEC-069 did not name it; P3 adds the line.
   - New variants come last in both enums. `just semver-check`: 202 pass, 58 skip, "no semver update required".
8. **No collateral change.**
   - `just test` (the recipe's three commands, with `--no-fail-fast`, the corpus at 12/12) gives **755 passed, 0 failed, 1 ignored**, the same as the build. Every pinned hash and oracle holds. Only the four `DEVELOP_VERSION` pins moved.
   - **Speed.** With no `GainMap`, both paths take a separate match arm whose closure is the old one: mono `Uniform`/`Pattern` with `gains.is_empty()`, and colour `with_normalizer!`'s `true` arms. That is a structural argument; I did not benchmark.
   - **Zero allocation.** `scratch_develop_allocates_nothing` has 8 four-map rows (fix-bad-pixels and warp, Orientations 1 and 6, both framings) plus the 17-map refusal, and `apply_gain_maps_allocates_nothing` is new. All are green, and V09 and V17 turned the alloc rows red, so they are live.
   - **Threads.** `colour_outputs` asserts that threaded equals single-thread for u16 and f32 on every ac5 fixture, now including P1's.
9. **Provenance.**
   - The ledger row is class 1 plus class 4, and quotes the per-file notice. I checked the local SDK tree (`tools/dng_sdk/src/dng_sdk_1_7_1`): there is no Adobe licence-agreement file, `DNG_ReadMe.txt` names none, and `dng_gain_map.cpp`'s header is exactly the notice quoted. The row's "terms, as found" is accurate.
   - **No code copied.** The SDK computes the gain with a stateful row interpolator that steps `fValueBase + fValueStep·index` in `f32`. `gain_at` is a stateless per-sample function in `f64` with its own `grid_axis`. No SDK identifiers appear in `src/`.
   - Getting the agreement's text from Adobe's download page remains the build's follow-up.
10. **Gates**, all green on 2e1a9c8 (`just test` on 1d008bb plus the new test):
    - `just test` with the corpus: 755 / 0 / 1;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`, `semver-check`;
    - `fuzz-opcode` for 45 s: 2,361,037 runs, 0 crashes.

**Verifier mutants.** The control run came first and was green: lib 193, gain_map 12, required_processing 49, develop_scratch_alloc 10. Each mutant ran in its own tree copy with its own `CARGO_TARGET_DIR`, running `cargo test --all-features --no-fail-fast --lib --test gain_map --test required_processing --test develop_scratch_alloc`, and I read each failure's panic message.

| ID | Mutant | Result | Red tests (first message read) |
|---|---|---|---|
| V01 | corner sampling, no `+ 0.5` | red | ac2, ac3, ac4, ac5 (ac3 `45056` vs `48640`) |
| V02m | mono maps normalised by crop size | red | ac4 (`develop_into`) |
| V02c | colour maps normalised by crop size | red | ac5 |
| V03 | clamp only the low edge | red | `grid_axis` unit, ac2 |
| V04 | columns first, in `f32` | red | ac4, ac5 |
| V04b | columns first, in `f64` | **green, equivalent in practice** | differs in the last ulp only (item 1) |
| V04c | rows first, in `f32` | red | ac4 |
| V05 | anchor after the intersection | **green, equivalent** | unobservable for `u32` `Top`/`Left` (item 1) |
| V06 | only the first map applied | red | ac2 ("four phase maps"), ac4, ac5 |
| V07 | optional maps skipped | red | ac4 optional, ac1, AC6 malformed |
| V08 | one rounding for the list | red | ac2, ac3 (`[2,2,2,2]`) |
| V09 | a 17th map accepted | red | unit cap test, ac1, AC6 seventeen, both alloc tests, the fuzz-seed test |
| V10 | size check accepts one byte long | red | ac1 ("one byte long") |
| V11 | mono gain in raw coordinates | red | ac4 |
| V12 | colour gain in raw coordinates | red | ac5 |
| V13 | mono gain in crop coordinates | red | ac4 |
| V14a | no clip at 1 (`max(0)`) | **green, equivalent** | saturating cast (item 4) |
| V14b | no clamp at all | **green, equivalent** | saturating cast, both ends (item 4) |
| V15 | `Plane` ignored | red | ac2 ("Plane 1 applies to nothing") |
| V16 | the last map plane used | red | ac2 ("Plane 0 of a two-plane map") |
| V17 | list 2 counts only the first `GainMap` as applied | red | AC6 four-maps and seventeen, ac4, ac5, scratch alloc |
| V18 | empty area anchored at `Top`/`Left` | red | ac2, ac3 (`[300; 4]`) |
| V19 | empty area treated as nothing | red | ac2, ac3, ac4 |
| V20 | indices not clamped at all | red | `grid_axis` unit, ac2, ac3, ac4 |
| V21 | `GainMap` accepted in lists 1 and 3 on colour | red | AC6 list 1 or 3 |
| V22 | non-finite gains accepted | red | ac1 ("gain NaN") |
| V23 | row pitch ignored | red | ac2, ac3 (pitch-skipped `1000`), ac4, ac5 |
| V24 | area not intersected with the image | **green, then red after P1** | new ac5 past-the-edge test |

### Cost self-report

- **Tokens (total):** 16,261,949, a **floor** taken just before the handback commit.
- **Estimated USD:** 11.32, at opus-5-5 list rates: in 5 and out 25 per MTok, cache reads ×0.10, 1-hour writes ×2.0.
- **Duration (minutes):** 24, the transcript span.
- **Source of the number:** the transcript `4ad4774a-533b-4a3a-a07b-1d8053bca58a.jsonl`, found by the scratchpad UUID. Each unique `message.id` is counted once, at its largest output. All are `claude-opus-5-5`, with no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-069's Consequences gained one line naming `GainMapImageWrongLength` (P3).
- **Deviations from spec:** none in the build beyond those it reported. I confirm both refusal rewrites (item 6) and the "no clip" equivalence (item 4).
- **Follow-up work identified:**
  - (carried from the build) The text of the Adobe DNG SDK licence agreement from Adobe's download page, for the ledger. The zip ships none.
  - (carried from the build) A composed-stages test: normalise, then `apply_gain_maps`, then `demosaic_into`, then `to_srgb_linear_into`, which should equal `develop_color_into` on a `GainMap` file. Since SPEC-059, the colour stage primitives accept `GainMap` files and rely on the composer to apply the maps.
  - The colour fused demosaic reflects at the plane border and reads masked photosites outside the `ActiveArea` (pre-existing, DEC-040). It is out of scope here, but SPEC-060's `dng_validate` comparison may show it at the crop edges, because the SDK demosaics the `ActiveArea` only.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. Item 1 lists "intersection with the image" as one clause with the empty area, but it only becomes observable on the colour path, through the demosaic's reads outside the `ActiveArea`. Saying which path makes a clause observable would point the verifier at the right fixture sooner.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That the colour fused kernel reads photosites outside the `ActiveArea` (it reflects at the plane edge). Every stage-2 rule that is bounded by the image depends on it.

3. **If you did this task again, what would you do differently?**
   — Write the mutant replacements against unique multi-line anchors from the start. One single-line anchor matched another opcode's size check and had to be rerun.
