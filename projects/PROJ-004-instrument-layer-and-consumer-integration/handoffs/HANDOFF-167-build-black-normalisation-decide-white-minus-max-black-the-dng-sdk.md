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
  id: HANDOFF-167
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-19
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-078

project:
  id: PROJ-004
  stage: STAGE-025
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
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 22208358           # REAL combined count — what cost-audit reads
  estimated_usd: 15.56             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 150
  branch: claude/spec-078-build
  pr: 276
  completed_at: 2026-11-20         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit and PR turns, from transcript 82149180-6978-4ac5-a3f8-666c6afba95f.jsonl (104 unique message.ids, all claude-opus-5-5, no subagents): input 215 + cache-creation 248,955 (1-hour) + cache-read 21,873,698 + output 85,490 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Re-sum at ship. One stop-and-report: 13 tier-A oracles outside AC5 moved; orchestrator approved." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-167: BUILD: black normalisation by white minus the maximum black (SPEC-078)

## Delegation Summary

You are the **builder** for SPEC-078. **Start only after the SPEC-078 design PR has
merged.** Cut `claude/spec-078-build` from that `origin/main`.

**The task:** implement DEC-070. The colour path (all four entries, `u16` and `f32`,
threaded) and the mono path under `BlackLevels::PerPosition` scale by `W − B`, where `B` is
the maximum black the path applies. The mono default (`FirstValue`) and every
uniform-black file stay byte-identical.

Also:
- set `DEVELOP_VERSION` to 7;
- add the Canon to the SDK recipe;
- tighten SPEC-060's SDK cross-check to the pre-registered bounds in AC4;
- re-pin only what DEC-070 changes, listing each pin.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-078-black-normalisation-decide-white-minus-max-black-the-dng-sdk.md`
  (Design 1–6, AC1–AC7, and the Failing Tests clause map).
- **Decisions:**
  - DEC-070 (the rule, the probe tables and the expected numbers);
  - DEC-018 (rounding and clamping);
  - DEC-061 (`PerPosition`);
  - DEC-068 (colour pattern, fractional rounding);
  - DEC-069 (where `GainMap` sits after normalisation);
  - DEC-060 (zero-allocation scratch develop).
- **Code:**
  - `src/develop.rs`: `normalize`, `resolve_levels`, the `Blacks` resolution and the
    gather and active-area loops;
  - `src/color.rs`: `normalize_sample`, `slot_normalizer`, `pattern_normalizer`, and
    SPEC-059's `with_normalizer!` sites;
  - `tests/gain_map_corpus.rs` and `scripts/sdk-stage2.sh` (SPEC-060);
  - `tests/black_levels.rs`, `tests/develop_options.rs`, `tests/develop_scratch.rs`.
- **The SDK:**
  - `dng_validate` is at `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate the stage-2 inputs with the recipe into your scratchpad, and set
    `IRRADIANCE_SDK_STAGE2_DIR`.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`.

## Expected Deliverables

1. **The implementation and tests** for AC1–AC6, in a PR against `main` from
   `claude/spec-078-build`, with CI green.
2. **The handback:**
   - the re-pin list (old → new hash, file, path, and why);
   - the AC4 numbers for every file and comparison (max |Δ|, the fraction exact, the
     histogram);
   - the AC6 mutant table.
3. **Gates**, all green: AC7.

## Out of Scope

- `BlackLevelDeltaH` / `BlackLevelDeltaV`.
- DEC-068's fractional rounding.
- SPEC-079's script hardening.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes.
- **An existing assertion may be strengthened, never weakened or deleted.**
  - The one pre-approved exception is
    `tests/black_levels.rs::canon_per_position_differs_only_at_position_3` (SPEC-078
    AC5).
  - Any other pin or assertion that DEC-070 moves, beyond the AC5 list: **stop and
    report** before changing it.
- **If a measured number lands outside a pre-registered bound,** stop and report it.
  Don't widen the bound.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill
  the `## Handback` section.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-078-build` — https://github.com/jysf/irradiance/pull/276
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** yes. One deviation needed approval: 13 tier-A oracles outside the AC5
  list moved, and the orchestrator approved them on 2026-11-20 (see Drift). CI status is in the PR.

**AC1.** `tests/black_normalisation.rs` (new) asserts every row of the table by hand, with the
arithmetic in comments:
- mono `PerPosition`, through `develop_with_into` and `develop_with_scratch_into`;
- colour normalisation, through `src/color.rs` unit tests on both the slot path and the general path;
- colour end to end, through an equivalence on all four entries (u16, f32, threaded). A
  `[2047, 2047, 2047, 2048]` file must develop identically to a black-2048 file whose positions 0–2
  are shifted up by one code.

The uniform fixture (black 2047 everywhere) gives today's column under both `PerPosition` and
`FirstValue`.

**AC2.** Only the AC5 pins and the four `DEVELOP_VERSION` pins moved. Every uniform-black file's
pins hold, and so does every mono default (`FirstValue`) pin, including the Canon's, the Pixels' and
the 7 Pro's.

**AC3.**
- Mono `PerPosition` and all four colour entries apply DEC-070.
- Threaded output equals single-thread: asserted in `tests/black_normalisation.rs` and
  `tests/gain_map_corpus.rs`.
- `tests/develop_scratch_alloc.rs` is green.
- New refusal test: `per_position_refuses_a_maximum_black_at_white`, with blacks
  `[2047, 2047, 2047, 15000]` and `W = 15000`. It returns `InvalidLevels { black_level: 15000,
  white_level: 15000 }` and writes nothing, and `FirstValue` still develops that file.

**AC4.** Run with `IRRADIANCE_SDK_STAGE2_DIR` set to outputs freshly generated by the committed
recipe, which now includes the Canon. Δ is the SDK minus ours.

| Comparison | File | max \|Δ\| | Exact | Histogram | Bound |
|---|---|---|---|---|---|
| No gain | Canon 5D Mark III (`-stage2`) | 1 (min 0, max 1) | 22,333,249 / 22,430,520 = 99.5663% | {0: 22333249, +1: 97271} | ≤ 1, {0,+1}, ≥ 99.5% ✅ |
| No gain | Pixel 4a | 1 (min 0, max 1) | 12,167,292 / 12,192,768 = 99.7911% | {0: 12167292, +1: 25476} | ≤ 1, {0,+1}, ≥ 99.5% ✅ |
| No gain | Pixel 3a | 1 (min 0, max 1) | 12,174,916 / 12,192,768 = 99.8536% | {0: 12174916, +1: 17852} | ≤ 1, {0,+1}, ≥ 99.5% ✅ |
| No gain | Pixel 7 Pro | 2 (min −2, max 1) | 3,217,671 / 11,350,968 = 28.3471% | {−2: 1228430, −1: 6904199, 0: 3217671, +1: 668} | ≤ 2 ✅ |
| End to end | Pixel 4a | 4 (min −1, max 4) | 12,165,076 / 12,192,768 = 99.7729% | {−1: 350, 0: 12165076, 1: 9031, 2: 14416, 3: 3879, 4: 16} | ≤ 4, ≥ 99.7% ✅ |
| End to end | Pixel 3a | 4 (min −1, max 4) | 12,170,943 / 12,192,768 = 99.8210% | {−1: 567, 0: 12170943, 1: 9848, 2: 9930, 3: 1471, 4: 9} | ≤ 4, ≥ 99.7% ✅ |
| End to end | Pixel 7 Pro | 4 (min −4, max 2) | 3,216,792 / 11,350,968 = 28.3394% | {−4: 25, −3: 413204, −2: 1910314, −1: 5809105, 0: 3216792, 1: 1458, 2: 70} | ≤ 4 ✅ |
| Gain step | Pixel 4a | 1 | 99.9794% | {−1: 350, 0: 12190254, 1: 2164} | ≤ 1, ≥ 99.9% ✅ |
| Gain step | Pixel 3a | 1 | 99.9672% | {−1: 567, 0: 12188768, 1: 3433} | ≤ 1, ≥ 99.9% ✅ |
| Gain step | Pixel 7 Pro | 1 | 99.9664% | {−1: 565, 0: 11347156, 1: 3247} | ≤ 1, ≥ 99.9% ✅ |

Every figure equals the DEC-070 probe's. With the variable unset, the tests print a visible SKIP, as
before.

**AC5 re-pins** (`tests/gain_map_corpus.rs` `PINNED`). The reason is the same for all eight: the 3a's
and 4a's blacks are 1023, 1023, 1022, 1022, so their `PerPosition` and colour outputs now scale by
`W − B`.

| File | Path | Old | New |
|---|---|---|---|
| Pixel 4a | `develop_with_into(StandardCrop, PerPosition)` | b1feae5d5184e196cd4ca0d1c4c10256 | 73d85bb84102586d767e2036c70df563 |
| Pixel 4a | `develop_with_into(FullActiveArea, PerPosition, warp 0)` | 435929609d2960438f0f69d61200eabd | 8cdd4bc68e117a259b67cec86588e511 |
| Pixel 4a | `develop_color_into` | 00c8361c0f9741f4f3714b15fb6f5bae | 092b1ca02b474f3ba73d55e7893af058 |
| Pixel 4a | `develop_color_f32_into` | c5899c08b544ebf3d2780f9891ef2290 | 3a5d78cdca45b02f443cd4e30c6682dc |
| Pixel 3a | `develop_with_into(StandardCrop, PerPosition)` | 25391555e20bc3749114019db10fcdb3 | 0787d0980ce83ab107439bd7b515663e |
| Pixel 3a | `develop_with_into(FullActiveArea, PerPosition, warp 0)` | ff0973a390ee1300407955ae1c951fa6 | 8dd219504381e8e91ac8f6cbcacababb |
| Pixel 3a | `develop_color_into` | 6ab25b632f65f4651bc2ae122e33294f | 6b4a538d8e0e81fc9da9adc318f9041e |
| Pixel 3a | `develop_color_f32_into` | f8d21871ca1949c199ac7347ec779a01 | 135015408536104e2247e9af16dbcf04 |

The two mono default pins (4a `a2de8cfb…`, 3a `98cafcff…`) and all five Pixel 7 Pro pins did not
change.

The pre-approved Canon test is now
`canon_per_position_scales_every_position_by_white_minus_the_maximum_black`. At every one of the
5796 × 3870 samples it asserts by hand that:
- `FirstValue` = `scale(v, 2047, 12953)`;
- `PerPosition` = `scale(v, b, 12952)`, where `b` is 2047 at positions 0–2 and 2048 at position 3.

At positions 0–2 the two modes subtract the same black and differ only through the scale. Wherever
they differ there, `PerPosition` is strictly above. Measured counts:
- 6,780,144 samples differ at positions 0–2 (new under DEC-070);
- 5,602,332 differ at position 3, unchanged;
- the largest difference is 6, unchanged.

**AC6 — mutants.** Each was applied to the source, run against the tier-A suites plus the corpus and
SDK suites, and then restored. The unmutated control ran first and was green.

| Mutant | Change | Red on (target first) |
|---|---|---|
| (a) per-position denominator `W − b` | mono pattern closures back to `normalize(s, b, white)`; colour slots `(b, W − b)` | AC1: `per_position_scales_by_white_minus_the_maximum_black`, `colour_scales_by_white_minus_the_maximum_black`, `color::tests::colour_slots_scale_by_white_minus_the_maximum_black`. AC4: `canon_no_gain_is_within_its_bound`, `end_to_end_and_no_gain_are_within_their_bounds`. Also the AC5 pins, the Canon test and the 13 updated oracles (30 failing tests in all) |
| (b) `B` as the minimum black | `max()` → `min()` at the three resolution sites (mono, colour slots, colour pattern) | AC1: `per_position_scales_by_white_minus_the_maximum_black`, `colour_scales…`, `colour_slots…`, `colour_general_pattern…`. Also the refusal test and the other oracles, 29 failing tests in all |
| (c) no clip at `W − B` | `.min(range)` removed in `normalize_range` | **Equivalent; no test went red.** Proof: for `s = v − b > W − B = r`, the numerator is at least `(r + 1)·65535 + ⌊r/2⌋`, so the quotient is at least `65535 + 65535/r`, which is more than 65535. `u16::try_from` then fails and the function returns `u16::MAX`. The clipped value is `(r·65535 + ⌊r/2⌋)/r = 65535`, because `⌊r/2⌋ < r`. So the two agree on every input, and no `u64` overflow is possible (`s ≤ 65535`). This is the same shape as the equivalence argued in SPEC-059. AC1's 14999 row lands exactly at `r`, and the 15000 row exercises the saturation. |
| (d) DEC-070 applied to `FirstValue` | the `FirstValue` branch returns a one-black pattern scaled by `W − max(pattern)` | AC2: `develop_into_output_is_unchanged_on_held_files` (develop_options), `scratch_output_matches_the_pins` (develop_scratch), `default_is_first_value_and_pins_hold`, `uniform_black_gives_todays_values`, and the Canon test |
| (e) validation against the first black | `if max_black >= white` → `if values[0] >= white` | AC3: `per_position_refuses_a_maximum_black_at_white`, plus `bad_patterns_are_refused_only_when_used` |

**The orchestrator's condition 2: indexing is still tested after the oracle update.** Each
wrong-position mutant went red:
- **Mono, transposed `Blacks::at` index** (positions 1 and 2 swapped on `[2, 2]`): red on
  `per_position_matches_hand_computed_block`, `non_square_and_3x3_patterns`,
  `uniform_cfa_plane_develops_uniform_under_per_position`, `black_is_applied_before_the_warp`
  (black_levels), `per_position_blacks_survive_the_tiled_walk` (orientation_tiles), the SDK
  end-to-end test and the Pixel pins.
- **Colour, transposed `pattern_index` cell** (general path): red on
  `color::tests::general_black_pattern_index_matches_hand_table`,
  `four_by_four_pattern_takes_the_general_path` and its f32 twin.
- **Colour, slots 1 and 2 swapped** in `slot_normalizer`: red on 8 `color_oracle` tests, including
  `grey_stays_grey_across_origin_parities_and_bayer_phases`, `fused_develop_color_is_byte_identical_to_the_reference`
  and `develop_color_composes_normalize_demosaic_matrix`. Also red on the colour_black_pattern 4×4
  tests and the Pixel pins.

**AC7 gates**, all green locally:
- `just test` with the corpus, then again with `IRRADIANCE_SDK_STAGE2_DIR`: each 774 passed, 0 failed, 1 ignored, all 12 corpus files present, and the SDK lane ran on the second;
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
- `cost-audit`, `gate-audit`, `name-length-check`;
- `semver-check`: 202 pass, 58 skip, "no semver update required";
- `fuzz-opcode` for 60 s: 2,913,628 runs, no crash.

CI: see the PR.

### Cost self-report

- **Tokens (total):** 22,208,358 (transcript `82149180-6978-4ac5-a3f8-666c6afba95f.jsonl`, deduped by
  `message.id`, largest output per message, no subagents)
- **Estimated USD:** 15.56 (opus-5-5: $5/Mtok in, $25 out, cache read ×0.10, 1-hour cache write ×2.0)
- **Duration (minutes):** about 150
- **Source of the number:** transcript. This is a floor taken before the last commit, push and PR
  turns; re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **Thirteen tier-A oracles outside the AC5 list encoded the old denominator.** The orchestrator
    approved updating them on 2026-11-20, on four conditions. Only the denominator changes; each
    oracle takes `B` as the max of its own fixture's blacks, without calling any crate helper, and
    still subtracts the per-position `b`. The tests:
    1. `src/color.rs` `general_black_pattern_index_matches_hand_table`: `65535 − blacks[i]` →
       `65535 − 600`, the fixture's maximum. The oracle now subtracts the hand table's black from the
       sample explicitly.
    2. `tests/black_levels.rs` `per_position_matches_hand_computed_block`: `4095 − b` → `4095 − 400`.
       The hand-computed 4×4 literals were re-derived outside the crate, and so were the alternative
       candidates in its comment.
    3. `tests/black_levels.rs` `non_square_and_3x3_patterns`: `4095 − b` → `4095 − 300` for `[1, 2]`
       and `[2, 1]`, and `4095 − 360` for `[3, 3]`. Its three literal blocks were re-derived.
    4. `tests/black_levels.rs` `uniform_cfa_plane_develops_uniform_under_per_position`:
       `16368 − b` → `16368 − 1023`. The `k ≠ 15345` exception is removed, and see below.
    5. `tests/orientation_tiles.rs` `per_position_blacks_survive_the_tiled_walk`: `WHITE − b` →
       `WHITE − 400`.
    6. `tests/color_oracle.rs`, eight tests through the shared `reference_normalize`, whose per-slot
       `white − b` becomes `white − max(slot blacks)`, and `grey_mosaic`, whose
       `raw = b + k(W − b)·asn` becomes `b + k(W − B)·asn`, with the grey tolerance still 8:
       `develop_color_composes_normalize_demosaic_matrix`,
       `colour_crop_pixels_equal_the_whole_plane_develop_at_their_source`,
       `colour_output_has_output_dimensions_geometry`, `colour_geometry_matches_the_mono_path`,
       `fused_develop_color_is_byte_identical_to_the_reference`,
       `grey_stays_grey_across_origin_parities_and_bayer_phases`,
       `plane_relative_phase_tints_an_odd_origin_grey`,
       `trivial_geometry_is_byte_identical_to_the_whole_plane`.

    No other assertion moved.
  - **Why the `k ≠ 15345` exception existed, and why it went:** under the old rule the 1022
    positions divided by 15346 and the 1023 positions by 15345. At `k = 15345` the plane rounded to
    65531 against 65535, so it was not uniform and the uniformity check had to skip that `k`. Under
    DEC-070 every position scales by the same 15345, so the plane is uniform, and equal to the
    LinearRaw develop, at every `k`. The check now runs for all three `k`. The `FirstValue`
    differs-from-`PerPosition` check was also only made for `k = 200`; it is now made for every
    `k > 0`, because at `k = 15345` the modes differ (65531 against 65535).
  - **The colour path gets no levels refusal.** It had none before, and Design §3's "same variant as
    today" exists only on mono. On colour, `W ≤ B` gives range 0, so every sample maps to 0 with no
    panic; `colour_with_no_range_maps_to_zero_without_panicking` covers it. The AC3 refusal test is
    on mono `PerPosition`. That path already refused any value at or above `W`, which is the same set
    as `B ≥ W`, and it now reports `B`.
  - **AC1 on colour** is asserted on the normaliser by unit tests, because `slot_normalizer` and
    `pattern_normalizer` are private, and on the public entries by the shifted-uniform equivalence.
    No AC1 value is read from the colour output directly.
- **Follow-up work identified:** none new. DEC-070 §5 already carries `BlackLevelDeltaH/V`, and
  SPEC-079 carries the script hardening.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — AC5's list named only the corpus pins and the Canon test. Thirteen tier-A oracles also encode the
   per-position denominator, so the build stopped once for approval. A grep for each oracle's
   `normalize(…, black, white)` at design time would have found them.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The colour path's lack of a levels refusal. Design §3 reads as if the colour path had one.

3. **If you did this task again, what would you do differently?**
   — I would run the full tier-A suite against a throwaway implementation before writing anything
   else, so the full list of moved assertions arrives in one report.
