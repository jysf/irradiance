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
  id: HANDOFF-161
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 38230872           # REAL combined count — what cost-audit reads
  estimated_usd: 25.38             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 27
  branch: claude/spec-059-build
  pr: https://github.com/jysf/irradiance/pull/267
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "LIVE FLOOR, re-sum at ship. Transcript a33f1412-dd84-4832-989c-e2110ebe2cd1.jsonl (found by the scratchpad UUID), 127 unique message.ids, all claude-opus-5-5, no subagents: input 262 + cache-creation 351,741 (all 1-hour) + cache-read 37,759,582 + output 119,287 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour write x2.0). Duration is the transcript span at the re-sum, before the handback commit and CI." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-19
---

# HANDOFF-161: BUILD: GainMap parse and apply on both develop paths (SPEC-059)

## Delegation Summary

You are the **builder** for SPEC-059. Cut `claude/spec-059-build` from `origin/main`
after the STAGE-016 design PR merges.

**The task:** implement DEC-069.
- `GainMap` (OpcodeID 9) from `OpcodeList2` is applied on the mono and colour develop
  paths at stage 2, by the exact rule in DEC-069 §2.
- The required-processing check accepts every `OpcodeList2` `GainMap` on both paths.
- A new public `apply_gain_maps` stage primitive is added.
- `DEVELOP_VERSION` goes to `"6"`.
- The rule is proven bit-exact against a hand-written oracle on writer-built fixtures,
  with a red-proof for each part of it.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-059-gainmap-parse-apply-on-mono-and-colour-paths-with-the-analyt.md`.
  Read all of it: Design §1–§8, AC1–AC9 and the Notes.
- **Decisions:**
  - `DEC-069`, all of it. This is the contract: the rule, the malformed classes, the
    16-map cap, one implementation with two callers, and the version.
  - `DEC-049` and `DEC-053`: required processing and stage primitives.
  - `DEC-058` §5: version.
  - `DEC-060`: the mono scratch develop allocates nothing.
  - `DEC-061` and `DEC-068`: black levels.
  - `DEC-040`: the `ActiveArea` origin.
  - `DEC-012`.
  - `DEC-009`: red-proof, control first.
- **Code:**
  - `src/opcode.rs`: the framing reader, `check_list`, `DevelopPath::applied_id`, the
    `Opcode` enum, `parse_opcode_list` and `__fuzz_required_processing`'s caller in
    `src/lib.rs`.
  - `src/develop.rs`: `crop_orient_normalize_into`, `normalize_active_area_into`,
    `resolve_levels_with`, `develop_core`, `develop_with_scratch_into` and
    `apply_fix_bad_pixels_constant`.
  - `src/color.rs`: the four entry points, `develop_color_bands`, `slot_normalizer` and
    `pattern_normalizer`.
- **Fixtures:** `tests/support/dng_writer.rs` `extra_tags` writes `OpcodeList2` (tag
  51009, UNDEFINED) with no writer change.
- **Facts from the design probe**, so you don't re-derive them:
  - The Pixel 3a, 4a and 7 Pro carry four mandatory 30×40 maps each. Their params are in
    DEC-069 §Context and STAGE-016's Design Notes.
  - `GainMap` is their only blocker on both paths.
  - The rule was checked against Adobe `dng_validate`. You do **not** need the SDK for
    this spec; SPEC-060 uses it.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - `just test` already passes `--all-features`.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`, and read each panic message.
    Red must come from the target assertion.

## Expected Deliverables

1. **Commits:**
   1. the parser and the `GainMaps` type, with AC1;
   2. the oracle and `apply_gain_maps`, with AC2 and AC3;
   3. mono and colour wiring, with AC4 and AC5;
   4. required processing, with AC6;
   5. the version, CHANGELOG, provenance row and fuzz.

   Combine commits where that keeps each one green.
2. **Evidence for each AC** in the handback:
   - every AC8 mutant, with the failing test and its message;
   - the AC7 corpus `just test` summary;
   - the fuzz run counts;
   - any public-shape choices (`Opcode::GainMap`, the wrong-length error).
3. **Gates (AC9),** all green with the corpus env set:
   - `just test`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
   - `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`;
   - `name-length-check`, `semver-check`, `decisions-index --check`.
4. **PR** against `main` from `claude/spec-059-build`, referencing SPEC-059, HANDOFF-161
   and DEC-069, with CI green on its head. The macOS determinism leg is non-blocking.

## Out of Scope

- The Pixel tier-B tests, pins and the SDK cross-check (SPEC-060).
- The max-black normalisation difference.
- Other `OpcodeList2` opcodes, and `GainMap` in lists 1 and 3.
- Colour `WarpRectilinear`.
- A release.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003), or any Adobe SDK file or output.
- **If an existing test or oracle goes red, the change is wrong.** The only permitted
  edits to existing tests are:
  - the `DEVELOP_VERSION` pins (`"5"` to `"6"`; grep for them);
  - new rows in `tests/required_processing.rs` and `tests/develop_scratch_alloc.rs`.

  If a fixture is now refused by design, stop and ask.
- **Fill the `handback:` front matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section below before reporting done.

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

- **Branch / PR:** `claude/spec-059-build`, https://github.com/jysf/irradiance/pull/267, cut from a30b86d.
  - `d9b9ed0`: the parser, `GainMaps`, both wirings, required processing, `apply_gain_maps`, version 6, and the tests.
  - `1aa4106`: the variant order (for semver), CHANGELOG, provenance, the fuzz target and seeds, and the timeline.
  - `2b9bbeb`: the second Pixel refusal pin and the DEC-051 error-kind rows.
  - Then this handback.
  - The handoff's five commits were combined into these, so that each one stays green.
- **Completed at:** 2026-11-19.
- **All acceptance criteria met?** Yes, AC1 to AC9. One caveat: the AC8 "no clip at 1" mutant is equivalent, as explained below.

**Evidence, AC by AC:**

- **AC1:**
  - `tests/gain_map.rs::ac1_gain_map_parses_to_the_fields_written` covers every field and every gain.
  - The unit test `opcode::tests::gain_map_entry_reads_every_written_gain_in_place` reads each gain through the in-place accessor.
  - `ac1_every_malformed_class_is_a_typed_error` runs 17 cases (all 12 named classes, plus `MapPointsH` 0, infinite spacing, NaN origin, an infinite gain and a huge grid). Each gives `MalformedOpcodeParams { id: 9, declared_size }` from both `parse_opcode_list` and `apply_gain_maps`, with nothing written.
- **AC2:** `ac2_apply_gain_maps_equals_the_oracle_bit_for_bit` covers 15 shapes:
  - a 2×3 grid with clamping and interpolation;
  - the four pitch-2 phases, separately and together;
  - an edge-intersected area, an empty area, and an area wholly outside the image;
  - `Plane` 1, and plane 0 of a two-plane map;
  - a 1×1 map, a clip at white, a gain below 1, and two maps on the same samples.

  `ac2_cases_are_sensitive` proves each case changes the image (or leaves it, where it must), and that one rounding differs from two.
- **AC3:** `ac3_hand_values` checks literal values, each with its arithmetic:
  - a corner clamp, 50000;
  - an interior bilinear, 48640;
  - a clip at white, 65535;
  - pitch-skipped 1000, against touched 2000;
  - rounding per map, 1 not 2;
  - an empty area as the whole image, 600.
- **AC4:** `ac4_mono_develop_equals_the_oracle_gain_then_the_existing_pipeline`.
  - The fixture is a 13×10 LinearRaw plane with `ActiveArea` origin (3, 1), black 512 and white 60000, at Orientations 1 and 6, with and without a mandatory non-identity warp.
  - The entries are `develop_into`, `develop_with_into` (both framings, oriented and not) and `develop_with_scratch_into`.
  - Each equals the same develop of a **pre-gained twin**: black 0 and white 65535, so levels are the identity, with the raw window set to the oracle applied to the test-side normalised stage 2.
  - `ac4_optional_gain_maps_apply_too` shows optional maps apply the same as mandatory ones.
  - Zero allocation: new rows in `scratch_develop_allocates_nothing`, covering 4 maps with and without fix-bad-pixels and warp, Orientations 1 and 6, both framings, and the 17-map refusal. Also a new `apply_gain_maps_allocates_nothing`.
- **AC5:** `ac5_colour_develop_equals_the_pre_gained_twin_on_every_entry`.
  - The fixture is a 14×11 RGGB plane with origin (3, 1) and four pitch-2 phase maps, at Orientations 1 and 3.
  - All four entries are checked (u16 and f32 compared bit for bit), and each threaded entry equals its single-thread one.
  - **Equivalent used:** the pre-gained twin above is exact for colour too. Reads outside the `ActiveArea` (the masked border, reflected reads) are normalised identically in both files and not gained, because they are not stage 2.
- **AC6:** new rows in `tests/required_processing.rs`:
  - four mandatory maps develop on mono (alone, and beside the Q2-Mono's two opcodes) and on all four colour entries, and the public check agrees;
  - a mandatory `GainMap` in list 1 or 3 is `UnsupportedRequiredOpcode { list, id: 9 }` on both paths, and an optional one is skipped;
  - an unknown mandatory opcode beside `GainMap`s keeps `UnsupportedMandatoryOpcode`, and a mandatory fix-bad-pixels opcode in list 2 keeps `UnsupportedRequiredOpcode`;
  - 17 maps give `TooManyGainMaps { count: 17 }` and a malformed map (zero pitch, or one byte short) gives `MalformedOpcodeParams`. Both come from all 9 develop doors (`develop_into`, `develop_with_into` and `develop_with_scratch_into` under both framings, and the four colour entries), with `dst` and scratch sentinel-unchanged. Sixteen maps develop.
- **AC7:** `just test` with `IRRADIANCE_CORPUS_DIR` set gives **755 passed, 0 failed, 1 ignored**, with 12/12 corpus files present, in 173 s.
  - Every pinned hash and oracle passes. The only edits to existing assertions are the four `DEVELOP_VERSION` pins and the deviations below.
  - The three Pixels now develop in colour and mono (4016x3008, 4016x3008 and 3866x2908).
- **AC8, the red-proof.** The control ran first. Each mutant ran in its own copy of the tree with its own `CARGO_TARGET_DIR`, running `cargo test --all-features --test gain_map`. Every failure was read.

  | Mutant | Result | Failing tests | Message read |
  |---|---|---|---|
  | control (unmutated) | green, 12/12 | none | |
  | corner sampling (no `+ 0.5`) | **red** | ac2, ac3, ac4, ac5 | ac3: `left: 45056, right: 48640` (the interior bilinear) |
  | indices not clamped | **red** | ac2, ac3, ac4 | ac3: `left: 63125, right: 50000` (the corner clamp) |
  | rows and columns swapped in the entry index | **red** | ac2, ac3, ac4, ac5 | ac3: `left: 32256, right: 48640` |
  | pitch phase anchored at 0 | **red** | ac2, ac3, ac4, ac5 | ac3: `left: 1000, right: 2000` (the pitch case) |
  | no clip at 1 (`clamp(0, 1)` changed to `max(0)`) | **green, an equivalent mutant** | none | see below |
  | no clip at 1, with a wrapping cast (`as u32 as u16`) | **red** | ac2, ac3, ac4, ac5 | ac3: `left: 14464, right: 65535` (the clip at white) |
  | one rounding for the whole list | **red** | ac2, ac3 | ac3: `left: [2, 2, 2, 2], right: [1, 1, 1, 1]` |
  | empty area treated as nothing | **red** | ac2, ac3, ac4 | ac3: `left: [300; 4], right: [600; 4]` |

  **Why "no clip at 1" survives.** The value is rounded back to `u16` after every map with Rust's `as u16`, which saturates at 65535. So the explicit clip and the cast's saturation are the same function on every input, and no test can tell them apart. Its observable form (the clip removed and the conversion made wrapping) is red on all four tests.
- **AC9:** every gate is green on the final tree, with the corpus env set:
  - `just test` (above), `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
  - `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`;
  - `name-length-check`, `decisions-index --check`;
  - `semver-check`: 202 pass, "no semver update required". It first failed on discriminant changes, so the new variants moved to the end of each enum.

  CI is on PR 267.
- **Fuzz:** `just fuzz-opcode` gave **3,948,136 runs in 76 s, 0 crashes**.
  - The target now also drives `apply_gain_maps` and asserts its count equals the parse's `GainMap` count. It runs the same bytes through the required-processing check in all three list slots, so `GainMap` IDs reach lists 1, 2 and 3.
  - There are 13 new `fuzz/seeds/opcode/gainmap-*` seeds. `just fuzz-seeds` regenerates every existing seed byte-identically.
- **Public-shape choices:**
  - `Opcode::GainMap { top, left, bottom, right, plane, planes, row_pitch, col_pitch, points_v, points_h, spacing_v, spacing_h, origin_v, origin_h, map_planes, gains: Vec<f32> }`, a struct variant like its siblings, placed last.
  - The wrong-length error is a new `Error::GainMapImageWrongLength { expected: u64, actual: usize }`, a CallerError.
  - `Error::TooManyGainMaps { count: u32 }` is Corrupt, like `TooManyIfds`.
  - `apply_gain_maps` returns the number of `GainMap`s in the list, including ones that touch no sample.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 38,230,872. This is a **live floor**, taken before the handback commit and CI, so re-sum it at ship.
- **Estimated USD:** 25.38 (opus-5-5 list rates, 1-hour cache writes).
- **Duration (minutes):** 27, the transcript span at the re-sum.
- **Source of the number:** the transcript `a33f1412-dd84-4832-989c-e2110ebe2cd1.jsonl`, found by the scratchpad UUID. It has 127 unique `message.id`s, all `claude-opus-5-5`, and no subagents. Each message was counted once, at its largest output.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-051's "Later additions" paragraph records the two new variants' kinds, as `tests/error_kind.rs` asks.
- **Deviations from spec:**
  - **The Pixel refusal pin in `tests/plane_tiles.rs`.** The old test, `pixel_files_develop_refuses_mandatory_gainmap`, asserted that `develop_color_into` and `develop_into` refuse the three Pixels with `UnsupportedMandatoryOpcode { id: 9 }`. That is exactly the behaviour DEC-069 removes, so it could not stay. It is now `pixel_files_develop_with_gainmap_applied`:
    - it asserts `Ok` on both entries, into buffers sized from `output_dimensions`;
    - it keeps the same corpus SKIP;
    - it has no pins, which are SPEC-060's.

    The orchestrator approved this.
  - **The same pin in `tests/develop_options.rs`.** `develop_into_output_is_unchanged_on_held_files`'s `PIXEL_REFUSED` loop asserted the "opcode 9 is unrecognized" refusal. The full corpus run found it after the approval above. It got the same treatment, with the constant renamed `PIXELS`: `develop_into` and `develop_with_into(default)` develop each Pixel `Ok` and identically. **This one was not separately approved, so verify should confirm it.**
  - **`tests/error_kind.rs`.** Two DEC-051 table rows, the variant count from 66 to 68, and the bucket counts to 16/21/31. That test's own comment requires this whenever a variant is added.
  - **A doc line in `tests/required_processing.rs`.** The module doc's "OpcodeList2: nothing, on either path" line was left as it is (now stale), and the new section says what changed.
  - **The empty area's pitch phase** is anchored at (0, 0), the whole image's top-left. DEC-069 §2 says the area becomes the whole image but does not name the anchor. That matches the SDK reading (`Overlap` returns the image bounds), and SPEC-060's real-file check covers it only if a Pixel has an empty area (none do).
  - **Colour reads outside the `ActiveArea`** (the masked border and reflected reads) are not gained, because they are not stage 2.
- **Follow-up work identified:**
  - **The SDK licence terms.** The zip ships no licence-agreement file, so the provenance row records only the per-file notice. Whoever ships or verifies may want the agreement's text from Adobe's download page on file.
  - **The colour stage primitives** (`demosaic_into`, `demosaic_into_threaded` and `to_srgb_linear_into`) now accept files with `GainMap`s, because the Colour check passes them, but they take a plane that is already past stage 2. Their docs say a composer applies `apply_gain_maps` first. A test that composes the stages to equal `develop_color_into` on a `GainMap` file would pin that.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The handoff listed the files a build may edit, but not the two tier-B tests that pinned the Pixel refusal (`plane_tiles`, `develop_options`), or `error_kind`'s variant table. A grep for `id: 9` and `opcode 9` at design time would have found all three.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two:
   - semver-check's `enum_no_repr_variant_discriminant_changed`: new variants on a public enum go last.
   - The DEC-051 table and count in `tests/error_kind.rs`.

3. **If you did this task again, what would you do differently?**
   — Run the corpus suite with `--no-fail-fast` before the first commit. `just test` stops at the first failing binary, which hid the second Pixel pin behind the first. Also run every shell command whose redirect target is a long path through a script file: the `rtk` hook dropped the redirect target and lost one test run.
