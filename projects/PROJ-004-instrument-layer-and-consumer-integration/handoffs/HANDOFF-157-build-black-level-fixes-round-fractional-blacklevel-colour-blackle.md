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
  id: HANDOFF-157
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-18
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-075

project:
  id: PROJ-004
  stage: STAGE-023
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
  tokens_total: 22453279           # REAL combined count — what cost-audit reads
  estimated_usd: 15.75             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-075-build
  pr: https://github.com/jysf/irradiance/pull/262
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "Transcript 91ac1b3f-39ab-454e-80bc-0ff9cf1f716a.jsonl, 95 message.ids deduped (max output each), no subagents, all claude-opus-5-5: input 198, output 80,376, cache read 22,104,382, cache write 268,323 (all 1-hour). Opus 5.5 at $5 in / $25 out, reads x0.10, 1-hour writes x2.0. Summed before the handback commit, so a floor; re-sum at ship."
  synced_at: 2026-11-19
---

# HANDOFF-157: BUILD: Black-level fixes and MaskedAreas (SPEC-075)

## Delegation Summary

You are the **builder** for SPEC-075. Cut `claude/spec-075-build` from `origin/main`
**after** the STAGE-023 design PR merges.

**The task**, in three parts, all in DEC-068:
- **Round a non-integral RATIONAL `BlackLevel`** to nearest, and keep the exact values in
  a new `Sensor::black_levels_exact`.
- **Make the colour normalizer index `BlackLevelRepeatDim`** with DEC-061 §3's rule, and
  refuse a count mismatch with `InvalidBlackLevelPattern` on all four colour entry
  points.
- **Pass `MaskedAreas` through** as `Sensor::masked_areas`.

`DEVELOP_VERSION` goes to `"5"`. Every held file's develop output must stay
byte-identical.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-075-black-level-fixes-round-fractional-blacklevel-colour-blackle.md`.
  Read all of it: the probe table, Design §2–§5, AC1–AC9 and the Notes.
- **Decisions:**
  - `DEC-068` (all of it);
  - `DEC-061` §3 (the pattern rule and the error);
  - `DEC-012` (interpretation tags cost themselves only);
  - `DEC-040` (the `ActiveArea` origin);
  - `DEC-058` §5 (the version rule);
  - `DEC-009` (red-proof, control first).
- **Code:**
  - `src/ifd.rs`: the sensor interpretation around `TAG_BLACK_LEVEL` (about lines
    2030–2230), `Container::rationals`, `array::<4>` for `ActiveArea`, `cost_the_field`,
    and the `Sensor` struct (about line 690).
  - `src/color.rs`: `make_normalizer` (about line 1240), `level_at`, `normalize_sample`,
    and the four entry points.
  - `src/develop.rs`: `resolve_levels_with` and `Blacks` (about lines 560–660).
  - `src/lib.rs`: `DEVELOP_VERSION`.
- **Tests and fixtures:**
  - `tests/support/dng_writer.rs` and its `extra_tags` (`RawTag`), which writes every
    fixture with no writer change;
  - `tests/black_levels.rs`, SPEC-066's oracle style (expected values are never computed
    by calling the crate).
- **Toolchain:**
  - AGENTS.md §5–6. `just test` runs the `corpus` profile plus the debug `*_alloc`
    binaries in about 3 minutes.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Do red-proofs on a local commit you never push, or an archived copy with its own
    target dir.
- **Facts from the design probe**, so you don't re-derive them:
  - The Pixel 7 Pro's `BlackLevel` is RATIONAL `262172/256 262224/256 262223/256
    262178/256`. All four round to 1024.
  - The Canon's is RATIONAL and integral.
  - No held file has `MaskedAreas`, a `[1, 2]`/`[2, 1]` pattern, or more than one
    `WhiteLevel` value.

## Expected Deliverables

1. **Commits:**
   1. the `BlackLevel` read and `black_levels_exact`;
   2. the colour pattern;
   3. `MaskedAreas`;
   4. `DEVELOP_VERSION` and the CHANGELOG;
   5. the feedback item-8 row.

   Combine commits where that keeps each one green.
2. **Evidence for each AC** in the handback:
   - **AC3, AC4 and AC5:** each red-proof, giving the mutant, the failing test and the
     revert. The `[1, 2]` and `[2, 1]` equivalences must be shown red against today's
     `level_at`.
   - **AC7:** the `exiftool -MaskedAreas` output for one fixture.
   - **AC8:** the corpus `just test` summary.
3. **Gates (AC9),** all green with the corpus env set:
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `lint-fuzz`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cost-audit`
   - `gate-audit`
   - `name-length-check`
   - `semver-check`
4. **PR** against `main` from `claude/spec-075-build`, referencing SPEC-075 and
   HANDOFF-157, with CI green on its head. The macOS determinism leg is non-blocking.

## Out of Scope

- A fractional black in develop.
- `BlackLevelDeltaH`/`V`.
- A colour `BlackLevels` option, or a colour black ≥ white refusal.
- Any change to mono `FirstValue`/`PerPosition` output.
- Using `MaskedAreas` in develop.
- GainMap, or making the Pixel files develop.
- SPEC-076's test hardenings.
- A release.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **If an existing test or oracle goes red, the change is wrong.** Fix the change, never
  the test. The only permitted edits to existing tests are the three `DEVELOP_VERSION`
  pins (`"4"` to `"5"`) and new tier-B assertions.
- **Fill the `handback:` front matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section below before reporting done.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-075-build`, https://github.com/jysf/irradiance/pull/262. It is cut from `origin/main` 1776d02, the merged design PR 261.
- **Commits:**
  - `d762fa0`: the `BlackLevel` read, `black_levels_exact`, the colour pattern, `MaskedAreas`, the new tests and the approved fixture edit.
  - `b6b24b8`: `DEVELOP_VERSION` 5, the four pins, the CHANGELOG and the feedback item-8 row.
  - `aba1369`: resolve the colour levels as data, for clippy 0.1.98's `type_complexity`. Behaviour is unchanged.
  - the handback commit.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes. There are two deviations, listed below; the orchestrator approved the first.

**Evidence per AC:**

- **AC1** (`tests/sensor_levels.rs`):
  - `2049/2` gives 1025, and `20491/20` gives 1025. `20489/20` gives 1024, as a control for rounding down.
  - The Pixel 7 Pro's four rationals, little-endian, give `black_levels == [1024; 4]`, `black_levels_exact` equal to the four `Rational`s as written, and no 50714 in `malformed_tags`.
  - `5/0` gives `black_level == None`, empty arrays, and 50714 in `malformed_tags` once, with `white_level` intact.
- **AC2:**
  - SHORT `[512]` gives an empty `black_levels_exact`.
  - The writer's LONG `[256, 257, 258, 259]` reads unchanged.
  - `524032/256` gives 2047, with `black_levels_exact == [524032/256]`.
- **AC3:** the mono `develop_into` and the colour `develop_color_into` (also run threaded) are byte-identical between a fractional black and SHORT `round(b)`. Both assert that the output differs from a black-0 file the test builds itself.
  - Red-proof (mutant 2): `black_level_values` forced through `uints` (the pre-SPEC-075 read), in an archived copy with its own target dir. 4 tests in `sensor_levels` go red: the three read tests and `fractional_black_develops_as_its_rounded_value`, with `assertion left != right failed: a fractional black must not develop as black 0`. In `colour_black_pattern`, `fractional_black_develops_as_its_rounded_value_in_colour` goes red the same way. Reverted, all are green.
- **AC4** (`tests/colour_black_pattern.rs`): odd `ActiveArea` origin (left 3, top 1), blacks 1000/1700/2300/2900. Each equivalence holds on the single-thread and the threaded entry: `[1, 2]` `[a, b]` against `[2, 2]` `[a, b, a, b]`, `[2, 1]` `[a, b]` against `[a, a, b, b]`, and `[4, 4]` tiled against `[2, 2]`.
  - Red-proof (mutant 1): today's `src/` from 1776d02 with `level_at`, under the new test file, in its own target dir. `one_by_two…` and `two_by_one…` go red with `left == right failed` (today's slots are `[a, b, b, b]` against `[a, b, a, b]`, and `[a, b, b, b]` against `[a, a, b, b]`: one slot each). `four_by_four…` goes red too, since today it takes `[a, b, a, b]` from the 4×4's first row. The empty-black control stays green.
  - Note: the first run of this red-proof shared a target dir with the control copy, so it reused a stale binary and showed all green. A separate `CARGO_TARGET_DIR` gave the real red.
- **AC5** (`src/color.rs`, `general_black_pattern_index_matches_hand_table`): a 6×4 hand-written index table for `[3, 2]` from origin top 1, left 3. It is checked through `pattern_index` and through the full general-path normalizer.
  - Red-proof (mutant 3): R and C swapped in `pattern_index` gives `left: Some(3) right: Some(5)` at (x 0, y 0). Reverted, it is green.
  - `small_patterns_take_the_slot_path` pins that `[1, 2]` and `[2, 1]` take the slot table.
- **AC6:**
  - `[2, 2]` with 3 values and `[1, 2]` with 4 return `InvalidBlackLevelPattern` with the right fields from `develop_color_into`, `develop_color_into_threaded`, `develop_color_f32_into` and `develop_color_f32_into_threaded`. Each `dst` stays sentinel-filled.
  - An absent `BlackLevel` develops identically to `[0]`.
  - The orchestrator asked for `four_values_with_no_repeat_dim_are_refused`, which pins `{ 1, 1, 4 }` for four values with no repeat dim. All of these are red against 1776d02's `src/`.
- **AC7:**
  - One and four LONG rectangles read verbatim and in order (unsorted and overlapping). A rectangle at exactly the plane edge reads. SHORT and LONG both read. Absent gives an empty field. 16 rectangles read.
  - Each of the six malformed cases gives an empty field and `malformed_tags == [50830]`, with the plane and every other `Sensor` field unchanged: count 5, count 0, 17 rectangles, `top == bottom`, `right > width`, and RATIONAL.
  - exiftool 13.55 on the four-rectangle fixture: `Masked Areas : 10 0 12 16 0 0 12 2 0 14 12 16 0 1 2 16` (the same with `-n`). The crate read gives `10 0 12 16 / 0 0 12 2 / 0 14 12 16 / 0 1 2 16`, which matches.
- **AC8:**
  - `just test` with `IRRADIANCE_CORPUS_DIR` set, on aba1369: 729 passed, 0 failed, 1 ignored, 42 binaries, 12/12 corpus files present, 2 min 45 s. Every pinned hash and oracle is unedited apart from the version pins. The Canon colour pin is green.
  - The new tier-B test `tests/ifd_reader.rs::rational_black_levels_read_rounded_and_exact` passes with no SKIP. The Pixel 7 Pro gives `Some(1024)`, `[1024; 4]`, the four rationals and no 50714. The Canon gives `[2047, 2047, 2047, 2048]` and 4 `black_levels_exact`.
- **AC9:**
  - Green on aba1369: `fmt-check`, `lint-ci` (clippy 0.1.98 pinned), `lint-no-allow`, `lint-fuzz`, `doc-check`, `msrv` (1.90.0), `deny`, `validate`, `cost-audit`, `gate-audit`, `name-length-check` and `decisions-index --check`.
  - `semver-check`: 202 pass, 58 skip, no semver update required.
  - CI: see the PR checks.

### Cost self-report

- **Tokens (total):** 22,453,279 (a floor, summed before this commit)
- **Estimated USD:** 15.75
- **Duration (minutes):** about 25 of agent wall time, including background test runs
- **Source of the number:** the transcript `91ac1b3f-39ab-454e-80bc-0ff9cf1f716a.jsonl`, found by the scratchpad UUID:
  - 95 `message.id`s, deduped, taking the max output for each;
  - no subagents;
  - all `claude-opus-5-5`, at opus-5-5 rates, with 1-hour cache writes.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **17 legacy colour tests went red under DEC-068's count rule**, and the orchestrator approved a fixture-only fix (option A). The four fixture helpers built 4 blacks with no `BlackLevelRepeatDim`, which defaults to `[1, 1]`. Each gained `black_level_repeat_dim = Some([2, 2])`, and no expected value or assertion changed. Those helpers are the inline sensor in `develop_color_composes_normalize_demosaic_matrix`, `fused_fixture` and `grey_origin_sensor` in `tests/color_oracle.rs`, and `sensor()` in `tests/threaded.rs`. The red tests:
    - `tests/color_oracle.rs` (11): `colour_crop_pixels_equal_the_whole_plane_develop_at_their_source`, `colour_output_has_output_dimensions_geometry`, `colour_geometry_matches_the_mono_path`, `develop_color_composes_normalize_demosaic_matrix`, `develop_color_f32_equals_to_linear_of_u16`, `even_origin_is_byte_identical_to_plane_relative`, `develop_color_f32_still_equals_to_linear_at_new_dims`, `fused_develop_color_is_byte_identical_to_the_reference`, `grey_stays_grey_across_origin_parities_and_bayer_phases`, `plane_relative_phase_tints_an_odd_origin_grey`, `trivial_geometry_is_byte_identical_to_the_whole_plane`.
    - `tests/threaded.rs` (6): `threaded_f32_equals_single_threaded`, `threaded_colour_develop_transposing_falls_back_and_still_matches`, `threaded_colour_develop_non_transposing_equals_single_threaded`, `threads_one_equals_the_single_thread_entry`, `output_is_independent_of_thread_count`, `band_boundary_reflects_at_the_true_edge`.
    - Confirmed: all 17 are green after the fixture edit, with no assertion changed (`color_oracle` 28/28, `threaded` 8/8).
  - **A fourth `DEVELOP_VERSION` pin**, `tests/warp_alpha.rs:340`, was not in the spec's list of three. It moved to `"5"` the same way.
  - **A RATIONAL `BlackLevel` with a numerator or denominator above `i32::MAX`** cannot be held as a `Rational`. It falls back to the pre-DEC-068 integral `uints` read, with `black_levels_exact` empty, so such a file reads exactly as before rather than becoming newly malformed. This is documented on `Container::black_level_values`. No held file is near that range.
  - **The colour normalizer's shape is a data enum (`ColourLevels`) resolved once.** Each entry point matches it and builds its closure, rather than one function returning two closure types, which clippy 0.1.98 rejects as `type_complexity`.
- **Follow-up work identified:**
  - The spec's pin inventory missed one of four. A design probe that greps for the pin (`DEVELOP_VERSION, "`) instead of listing files would catch this.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The design did not say what the colour count rule does to existing tier-A fixtures. Seventeen tests built a shape that DEC-068 now refuses, and it took an orchestrator round trip to settle. A design probe over `tests/` for sensors with `black_levels` set and no repeat dim would have surfaced it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The "no existing test edits" rule needed an exception path for fixtures that a DEC deliberately makes invalid. The spec listed only the version pins.

3. **If you did this task again, what would you do differently?**
   — I would run the whole colour suite against a stub of the new rule before writing code, and give every red-proof copy its own `CARGO_TARGET_DIR` from the start. One red-proof first came back falsely green from a shared target dir.
