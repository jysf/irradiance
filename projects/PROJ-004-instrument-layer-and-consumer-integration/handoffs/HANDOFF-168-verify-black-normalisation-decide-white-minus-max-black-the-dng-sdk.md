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
  id: HANDOFF-168
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 13094325           # REAL combined count — what cost-audit reads
  estimated_usd: 9.52              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 35
  branch: claude/spec-078-verify-base
  pr: 276
  completed_at: 2026-11-20         # YYYY-MM-DD
  notes: "APPROVED at 23ece24 (PR 276 head). Item 3 ruled an acceptable reading of DEC-070 section 3, to be recorded as an amendment at ship; FU-1 colour levels refusal, FU-2 tier-A colour fixture with the maximum black not last. Tokens are a floor from transcript ad04463f-192f-4249-9efc-b8e3e628fb47.jsonl taken before the commit and push turns; 66 message ids, no subagents."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-168: VERIFY: black normalisation scales by white minus the maximum black (SPEC-078)

## Delegation Summary

You are the **verifier** for SPEC-078. Review it cold: you did not build it.

- **Under review:** PR #276 (`claude/spec-078-build`, head 23ece24), built under
  HANDOFF-167. CI is green on all 22 checks.
- **Your branch:** `claude/spec-078-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-078 implements DEC-070. Colour develop (all four entries) and mono
`BlackLevels::PerPosition` now scale by `W − B`, where `B` is the maximum black the path
applies, and subtract each position's own black. `FirstValue` and uniform-black files are
meant to be byte-identical. `DEVELOP_VERSION` is 7. `scripts/sdk-stage2.sh` also writes a
Canon stage-2 reference, with a new Canon cross-check.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-078-black-normalisation-decide-white-minus-max-black-the-dng-sdk.md`
  (Design §1–5, AC1–AC7, and AC4's pre-registered bounds).
- **Decisions:** DEC-070 (the rule, §3 validation, §Context's probe table), DEC-069 (the
  GainMap step and the SDK reference, §6), DEC-068 (the 7 Pro's fractional black, why its
  differences are negative), DEC-061 (per-position black), DEC-018 (rounding).
- **Build handback:** HANDOFF-167's `## Handback`, with its AC4 table, the 8 re-pins, the
  13 oracle updates, the mutants and the deviations. Check each claim yourself.
- **The SDK:**
  - The binary is at
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate your **own** stage-2 outputs with the committed script into your scratchpad,
    and point `IRRADIANCE_SDK_STAGE2_DIR` at them. Don't reuse the builder's directory.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them into the repo.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The rule, by hand.** Recompute AC1's three rows yourself (W = 15000, blacks
   [2047, 2047, 2047, 2048], B = 2048: 8523 → 32768, 4000 → 9882, 14999 → 65535; today's
   column 32765, 9881, 65530). Confirm the tests assert them on mono `PerPosition` (both
   `develop_with_into` and the scratch entry) and on colour. On colour the values are
   checked on the private normaliser plus a shifted-uniform equivalence on the public
   entries. Is that equivalence actually strong enough to pin the colour rule, or could a
   wrong colour denominator still pass it?
2. **`B` is computed once, over the right blacks.** Read every resolution site: mono
   `PerPosition`, colour slots, colour general pattern, `u16`, `f32` and banded threaded.
   - `B` is the max over the blacks the path *applies*, not over the whole tag.
   - It is never recomputed per sample (a speed check, not only correctness).
   - `FirstValue` and uniform-black paths keep their old code path; their pins held.
3. **The ruling the build left open: the colour path refuses nothing when `W ≤ B`.**
   Spec Design §3 and DEC-070 §3 say `W ≤ B` is `Error::InvalidLevels`. The builder applied
   that only on mono `PerPosition`, arguing the colour path never had a levels refusal, so
   on colour `W ≤ B` gives range 0 and every sample maps to 0
   (`colour_with_no_range_maps_to_zero_without_panicking`).
   - Check whether that's true of the colour path before this PR (`git show origin/main`).
     Is there really no colour levels check, or did the build miss one?
   - Rule on it: an acceptable reading of DEC-070 §3 (record it as a DEC-070 amendment in
     the punch list), or a `src/` fix (punch list). Either way, say which and why.
   - Confirm no division by zero or panic is reachable on colour for `W ≤ B`, `W = B`, or
     `W < every black`.
4. **AC4 reproduces from scratch.** Run `scripts/sdk-stage2.sh` cold into a new scratchpad
   directory, then the cross-checks. Do your numbers equal the build's table and the
   DEC-070 probe? Compare the bounds in the tests with the spec as designed in PR #275
   (`git show origin/main:<spec path>`), not as edited on the build branch:
   - no gain: Canon, 3a and 4a ≤ 1, Δ ∈ {0, +1}, at least 99.5% exact; 7 Pro ≤ 2;
   - end to end: 3a and 4a ≤ 4, at least 99.7% exact; 7 Pro ≤ 4;
   - the gain-step gate unchanged (≤ 1, ≥ 99.9%).

   Any change must be an approved deviation. The Canon stage-2 file is new: check the
   script writes it outside the repo and the Canon test reads the right comparison
   (no-gain stage 2 against our mono `PerPosition` `FullActiveArea`, unoriented).
5. **The 13 oracle updates (approved, on four conditions).** For each, confirm:
   - only the denominator changed, and the per-position `b` is still subtracted;
   - each oracle computes `B` from its own fixture's literals, with no crate helper;
   - the re-derived literal blocks in `tests/black_levels.rs` are right — recompute at
     least two by hand or with an independent script;
   - the `k ≠ 15345` carve-out removal is justified as the handback says.

   Run `git diff origin/main -- tests/ src/color.rs` and confirm no assertion moved that
   isn't in the handback's list.
6. **Indexing is still tested.** Re-run the builder's wrong-position mutants (mono
   transposed `Blacks::at`, colour transposed `pattern_index`, colour slots 1 and 2
   swapped). Each must go red. With the denominator now shared by every position, a
   position bug shows only through the subtracted `b`: check that enough tests have
   blacks that differ by more than 1 for that to be visible.
7. **Mutants.** Re-run the build's (a), (b), (d) and (e), and check (c)'s equivalence
   proof (no clip at `W − B`) yourself. Then try at least:
   - `B` computed over the whole `BlackLevel` tag rather than the applied pattern;
   - `B` from the first slot only on the colour slot path;
   - the f32 colour entry left on the old rule while `u16` moves;
   - the threaded colour path left on the old rule.

   Each should go red somewhere. Record what goes red.
8. **The pins (AC2, AC5).**
   - Exactly the 8 Pixel 3a/4a pins listed moved, plus the four `DEVELOP_VERSION` pins
     and the pre-approved Canon test; every other pin holds (`git diff origin/main --
     tests/`).
   - The new pins are stable: run twice, threaded against single-thread.
   - The Canon test's counts (6,780,144 newly differing at positions 0–2; 5,602,332 at
     position 3; max 6) — spot-check them.
9. **The release surface.** `DEVELOP_VERSION` is `"7"` and documented; the CHANGELOG
   `[Unreleased]` entry names the output change and which files it affects; `semver-check`
   is clean against crates.io 0.2.3. The provenance-ledger row is accurate and cites the
   DNG specification's Rescaling text and the SDK as class 4, with no SDK code copied.
10. **Nothing forbidden is committed:** no SDK bytes, no SDK outputs, no corpus bytes.
11. **Gates:**
    - `just test` with the corpus, both with and without the SDK variable;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants and your own
  cross-check numbers.
- **An explicit ruling on item 3.**
- **Commit this handoff's filled handback** to `claude/spec-078-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-078-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. Any
  `src/` change goes in the punch list. A measured number outside a pre-registered bound
  is reported, never fixed by widening the bound.

## Out of Scope

- SPEC-079's work: the `sdk-stage2.sh` IFD walk and mkdir order, the Adobe licence line,
  the monorail triage refresh.
- `BlackLevelDeltaH/V` (DEC-070 §5 carries it).
- Running the SDK in CI, vendoring it, or committing its outputs.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

## Handback` section below.** An unfilled handback is an
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

- **Branch / PR:** reviewed PR #276 (`claude/spec-078-build`, head `23ece24`, CI 22/22 pass,
  mergeable). Handback committed to `claude/spec-078-verify-base`. No PR opened.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** yes. AC1 to AC7 are met and each was re-checked cold. Item 3
  is ruled below as an acceptable reading of DEC-070 §3, to be recorded as an amendment.
- **For `verify`:** ✅ **APPROVED at `23ece24`**, the PR head. No ship-blockers. There are two
  follow-ups (FU-1, FU-2) and one DEC-070 amendment to record at ship.

The verify ran in its own worktree on a local branch tracking
`origin/claude/spec-078-verify-base`. The orchestrator's worktree had that branch checked out, so
the push goes to it by refspec.

#### 1. The rule, by hand

- **AC1 recomputed independently** in Python, outside the crate, with
  `⌊clamp(v − b, 0, W − B) · 65535 / (W − B) + ½⌋` at W = 15000 and B = 2048:

  | Sample | Black | Today | DEC-070 |
  |---|---|---|---|
  | 8523 | 2047 | 32765 | 32768 |
  | 8524 | 2048 | 32768 | 32768 |
  | 4000 | 2047 | 9881 | 9882 |
  | 14999 | 2047 | 65530 | 65535 |
  | 15000 | 2047 | 65535 | 65535 |
  | 2000 | 2047 | 0 | 0 |

  These equal the spec's table and `tests/black_normalisation.rs` `AC1_ROWS`.
- **Where each row is asserted:**
  - Mono `PerPosition`: on both `develop_with_into` and `develop_with_scratch_into`. `develop_mono`
    asserts the two agree, then checks the table plus every filler photosite (14942 and 14937, both
    recomputed).
  - Colour slot and general paths: the private normalisers in `src/color.rs`, which are
    `colour_slots_…` and `colour_general_pattern_…`.
  - The four public colour entries: the shifted-uniform equivalence, with threaded equal to
    single-thread in u16 and f32 bits.
- **Is the equivalence strong enough? Mostly.** It pins the colour denominator to exactly 12952 for
  this fixture, and it pins that positions 0 to 2 subtract their own black. Every reachable wrong
  rule went red on it: the old `W − b`, the minimum, the first slot only, f32 left behind, and
  threaded left behind (mutants a, b, f, h and i).
- **The gap it leaves:** a wrong rule that happens to give 12952 on this fixture passes, because
  the fixture's maximum black is in the last slot. "`B` = the last slot's black" (mutant g) stays
  green on every tier-A colour test. Only the tier-B Pixel colour pins catch it, since the Pixel
  maximum is in slot 0, and CI has no corpus. See FU-2. On mono, the tier-A
  `uniform_cfa_plane_develops_uniform_under_per_position` (Pixel blacks) catches it.

#### 2. `B` is computed once, over the right blacks

- **Mono `PerPosition`:** `resolve_levels_with` takes `max_black` once over `sensor.black_levels`
  and stores `range = W − B` in `Blacks::Pattern`.
- **Colour slots:** `B` is the maximum over the four slot blacks.
- **Colour general pattern:** `B` is the maximum over the pattern values.
- **u16, f32 and threaded:** all go through `resolve_colour_levels`, once per develop at all three
  call sites (`develop_color_into`, `develop_color_f32_into` and `develop_color_bands`), before the
  `with_normalizer!` match. The per-sample closures only call `normalize_range(sample, b, range)`.
  That is the same integer affine map with one `min`, so nothing is computed per sample.
- **"Applied" against "whole tag":** `black_pattern_dims` enforces `len == R × C`, and a slot
  pattern with R, C ≤ 2 fills its slots from every value. So the maximum over the applied blacks
  and the maximum over the tag are the same on every path. Mutant m (colour slot `B` over the whole
  tag) is green: equivalent by construction.
- **`FirstValue` and uniform blacks** keep `Blacks::Uniform` and `normalize(s, b, W)`, the old
  path. Every `FirstValue` and uniform-file pin held, in two full runs.

#### 3. Ruling: the colour path refuses nothing when `W ≤ B`

- **The fact is confirmed.** On `origin/main`, `src/color.rs` has no `InvalidLevels` anywhere.
  `resolve_colour_levels` refused only a bad pattern shape. `normalize_sample` was made total on
  purpose ("if `black > white` (malformed) or `black == white` the result is 0 rather than a
  panic"). The colour entries' `# Errors` docs never listed `InvalidLevels`. The build did not miss
  a check: there was none.
- **No panic is reachable.**
  - `W ≤ B` gives `range = white.saturating_sub(max_black) = 0`.
  - `normalize_range` then clamps the shifted value to 0, and `checked_div(0)` gives `None`, so the
    result is 0. The same holds for `W = B` and for `W` below every black.
  - The gain path multiplies 0. `colour_with_no_range_maps_to_zero_without_panicking` asserts it.
- **Ruling: an acceptable reading of DEC-070 §3. Record it as a DEC-070 amendment, not a `src/`
  fix.**
  - §3's own words are "This is the **existing** `Error::InvalidLevels` check, now made against the
    maximum black". Only mono has an existing check. Adding a new refusal to four public colour
    entries is a behaviour change beyond this spec's scope.
  - Amendment text, for ship: "§3 applies where a levels check exists, mono `PerPosition`. The
    colour path has never refused levels; there `W ≤ B` gives range 0 and every sample develops to
    0, never a panic."
- **One behavioural edge to record:** a malformed colour file with some blacks below `W` and the
  largest at or above `W` used to develop its below-`W` positions. It now develops entirely to 0.
  No panic and no out-of-range value result. Whether colour should refuse levels like mono does
  is FU-1.

#### 4. AC4 reproduced from scratch

`scripts/sdk-stage2.sh` was run cold into a new scratchpad directory, outside the repository,
with the committed script and the local `dng_validate`. It wrote 4 files in 4.4 s, including
`CANON-EOS-5D-MARK-III-stage2.tif`. The cross-checks were then run with
`IRRADIANCE_SDK_STAGE2_DIR` pointing there. Δ is the SDK minus ours.

| Comparison | File | max \|Δ\| | Exact | Histogram | Bound (spec on main) |
|---|---|---|---|---|---|
| No gain | Canon (`-stage2`) | 1 | 22,333,249 / 22,430,520 = 99.5663% | {0: 22333249, +1: 97271} | ≤ 1, {0,+1}, ≥ 99.5% ✅ |
| No gain | Pixel 4a | 1 | 12,167,292 / 12,192,768 = 99.7911% | {0: 12167292, +1: 25476} | ✅ |
| No gain | Pixel 3a | 1 | 12,174,916 / 12,192,768 = 99.8536% | {0: 12174916, +1: 17852} | ✅ |
| No gain | Pixel 7 Pro | 2 | 28.3471% | {−2: 1228430, −1: 6904199, 0: 3217671, +1: 668} | ≤ 2 ✅ |
| End to end | Pixel 4a | 4 | 99.7729% | {−1: 350, 0: 12165076, 1: 9031, 2: 14416, 3: 3879, 4: 16} | ≤ 4, ≥ 99.7% ✅ |
| End to end | Pixel 3a | 4 | 99.8210% | {−1: 567, 0: 12170943, 1: 9848, 2: 9930, 3: 1471, 4: 9} | ≤ 4, ≥ 99.7% ✅ |
| End to end | Pixel 7 Pro | 4 | 28.3394% | {−4: 25, −3: 413204, −2: 1910314, −1: 5809105, 0: 3216792, 1: 1458, 2: 70} | ≤ 4 ✅ |
| Gain step | Pixel 4a / 3a / 7 Pro | 1 / 1 / 1 | 99.9794% / 99.9672% / 99.9664% | as the build | ≤ 1, ≥ 99.9% ✅ |

- Every figure equals the build's table and the DEC-070 probe, digit for digit.
- The bounds in `tests/gain_map_corpus.rs` (`no_gain_bound`, `end_to_end_bound`, the unchanged
  `GAIN_STEP_*`) equal AC4 as designed in PR #275. That was read from
  `git show origin/main:<spec>`, not the build branch's copy. No bound was widened or changed.
- The script refuses an output directory inside the repository, using `pwd -P` and a prefix case.
- `canon_no_gain_is_within_its_bound` compares the right thing: mono `stage2_like()`
  (`PerPosition`, `FullActiveArea`, unoriented, warp 0) against `-stage2.tif`. It also asserts the
  Canon has no `OpcodeList2`.
- With the variable unset, `corpus-status` prints a visible SKIP line.

#### 5. The 13 oracle updates

- **Only the denominator changed.**
  - `tests/black_levels.rs` and `tests/orientation_tiles.rs` add `normalize_max`, which still does
    `sample.saturating_sub(black)` with the per-position `b`, scaled by `white − max_black`.
  - `color_oracle` `reference_normalize` still subtracts each slot's black, and `grey_mosaic` still
    adds `black_slot`.
  - `src/color.rs`'s hand table now subtracts `blacks[i]` explicitly.
- **`B` comes from each fixture's own literals:** `max_black(&PIXEL_BLACKS / &AC3_BLACKS /
  &blacks)`, `BLACKS.iter().max()`, `SLOT_BLACK.iter().max()` and the hand table's
  `blacks.iter().max()`. `color_oracle`'s `reference_normalize` takes the maximum of the four
  `level_at` values from the test-built sensor. That is a test helper, not a crate call.
- **Every re-derived literal block was recomputed independently** (Python, outside the crate, from
  `raw = 500 + 37x + 11y` and the crop at raw (4, 3)). All four match the committed literals
  exactly:
  - `per_position_matches_hand_computed_block`: W − B = 3695;
  - the `[1, 2]` block: 3795;
  - the `[2, 1]` block: 3795;
  - the 3×3 block: 3735.

  The comment's alternative candidate "plane origin 6757" was spot-checked: 381 · 65535 / 3695.
- **The `k ≠ 15345` carve-out removal is justified.** Under one range of 15345, every photosite
  maps `k` the same way, so the plane is uniform for every `k`. The check is now asserted at
  k = 15345 too, which strengthens it. `assert_ne!(first_out, got)` now holds at every k > 0
  rather than only at 200.
- **No assertion moved that is not in the handback's list.** I checked
  `git diff origin/main...HEAD -- tests/ src/color.rs`. The changes are the 13 oracles, the AC5
  `PINNED` hashes, the four `DEVELOP_VERSION` pins (`black_levels`, `develop_options`,
  `orientation_tiles`, `warp_alpha`), the pre-approved Canon rewrite, and the new tests.

#### 6. Indexing is still tested

The builder's three wrong-position mutants were re-run, each in its own copy and target directory:

- **Mono transposed `Blacks::at`** (mutant j): 7 red, including `per_position_matches_hand_computed_block`,
  `non_square_and_3x3_patterns`, `uniform_cfa_plane_…`, `black_is_applied_before_the_warp`,
  `per_position_blacks_survive_the_tiled_walk`, the SDK end-to-end test and the Pixel pins.
- **Colour transposed `pattern_index`** (mutant k): 3 red, `general_black_pattern_index_matches_hand_table`
  and the 4×4 general-path tests in u16 and f32.
- **Colour slots 1 and 2 swapped** (mutant l): 10 red, including `grey_stays_grey_…`,
  `fused_develop_color_is_byte_identical_to_the_reference`, `plane_relative_phase_…` and the Pixel
  pins.

The tier-A fixtures that indexing relies on have blacks that differ by far more than 1:
`AC3_BLACKS` and the orientation `BLACKS` are [100, 200, 300, 400], `SLOT_BLACK` is
[200, 400, 600, 800], and the colour hand table is [100 … 600]. A position bug is visible through
the subtracted `b` by tens of codes.

#### 7. Mutants

Every mutant ran in its own copy, with its own `CARGO_TARGET_DIR`. Each ran the full suite under
the `corpus` profile with `--no-fail-fast`, the corpus, and my SDK directory. The control is the
unmutated tree: two full `just test` runs, 774 passed, 0 failed, 1 ignored. I read the panic
message of each target assertion.

| # | Mutant | Result | Red on (target first) |
|---|---|---|---|
| a | old per-position denominator `W − b` (mono closures and colour slots) | 23 red | AC1 `per_position_scales_…`, `colour_scales_…`, `color::tests::colour_slots_…` ("every slot scales by W − B": 12953 against 12952); AC4 `canon_no_gain_…`, `end_to_end_and_no_gain_…`; plus the pins, the Canon test and the oracles |
| b | `B` = the minimum black (mono and colour) | 24 red | AC1 mono, colour public and both colour unit tests; the AC3 refusal; AC4; the pins |
| c | no clip at `W − B` | **green: equivalent** | Proof re-checked. For `s ≥ r + 1` with `1 ≤ r ≤ 65535`, the quotient is at least `65535 + ⌊(65535 + ⌊r/2⌋)/r⌋ ≥ 65536`, so `u16::try_from` saturates to 65535, which is the clipped value (`⌊r/2⌋ < r`). With `s ≤ 65535` the `u64` cannot overflow. The proof omits the `r = 0` case, which is also equivalent: both forms give `checked_div(0)`, so 0. |
| d | DEC-070 applied to `FirstValue` (`B` over the whole pattern) | 7 red | AC2 `develop_into_output_is_unchanged_on_held_files`, `scratch_output_matches_the_pins`, `default_is_first_value_and_pins_hold`, `default_warp_alpha_is_one_and_pins_hold`, `uniform_black_gives_todays_values`, the Canon test, `colour_geometry_matches_the_mono_path` |
| e | the levels check against the first black | 3 red | AC3 `per_position_refuses_a_maximum_black_at_white` ("W ≤ B has no range to scale by"), `bad_patterns_are_refused_only_when_used`, `per_position_allocates_nothing` |
| f | colour slot `B` from the first slot only | 11 red | AC1 `colour_scales_…`, `colour_slots_…`; the `color_oracle` grey and fused tests; 4×4 |
| g | `B` = the last value (mono and colour; my own) | 5 red | **Tier A on mono only** (`uniform_cfa_plane_…`, `bad_patterns_…`, `per_position_allocates_nothing`); colour only on the tier-B Pixel pins and the SDK end-to-end test. See FU-2 |
| h | the f32 colour entry left on the old rule | 7 red | `colour_scales_…` ("the threaded f32 entry disagrees"), `develop_color_f32_equals_to_linear_of_u16`, `…_at_new_dims`, `threaded_f32_equals_single_threaded`, `output_is_independent_of_thread_count`, the Pixel pins |
| i | the threaded colour path left on the old rule | 12 red | `colour_scales_…` ("the threaded u16 entry disagrees"), `threaded_colour_develop_non_transposing_equals_single_threaded`, `one_by_two_…` / `two_by_one_…`, `output_is_independent_of_thread_count`, the Pixel pins |
| j | mono `Blacks::at` transposed | 7 red | see item 6 |
| k | colour `pattern_index` transposed | 3 red | see item 6 |
| l | colour slots 1 and 2 swapped | 10 red | see item 6 |
| m | colour slot `B` over the whole tag | **green: equivalent** | `len == R × C` is enforced, and every value fills a slot (item 2) |

#### 8. The pins (AC2, AC5)

- **Exactly the 8 Pixel 3a and 4a hashes moved** in `tests/gain_map_corpus.rs` `PINNED`. They match
  the build's old and new table. The two Pixel mono-default hashes and all five 7 Pro hashes are
  unchanged.
- **The other moved assertions** are the four `DEVELOP_VERSION` pins and the pre-approved Canon
  test. Every other pin is untouched in the diff and held in both runs.
- **The new pins are stable.** They held in two full `just test` runs and in the per-file rerun.
  `pixel_files_develop_on_every_entry_and_are_pinned` asserts threaded u16 and f32 equal to
  single-thread on every Pixel.
- **Canon counts**, spot-checked independently: a pure-Python pass over the raw strip of
  `5G4A9394.DNG`, read in place (uncompressed, 16-bit, ActiveArea 80/122/3950/5918). It gives
  scale-only 6,780,144, position 3 5,602,332 and largest 6, exactly the test's constants. The
  test's own `--nocapture` line agrees.
- **AC3:** `develop_scratch_alloc` (`per_position_allocates_nothing`) is green in debug in both
  `just test` runs.

#### 9. The release surface

- `DEVELOP_VERSION` is `"7"`, with a doc paragraph in `src/lib.rs`.
- The CHANGELOG `[Unreleased] → Changed` entry names the change, the four colour entries plus
  `BlackLevels::PerPosition`, and the Canon, 3a and 4a. It states `FirstValue` and uniform files
  are byte-identical, the new `InvalidLevels` reporting, and the version bump.
- `just semver-check` against crates.io 0.2.3: 202 checks, 202 pass, 58 skip, "no semver update
  required".
- **The provenance-ledger row is accurate.** It is class 1 for the DNG 1.7.1.0 Chapter 5 Black
  Subtraction and Rescaling text, and class 4 for `dng_linearization_info.cpp`, read for
  confirmation only with "no code copied". Its figures equal my run. The new code is this crate's
  own `normalize_range`, the same shape as `normalize`, with nothing SDK-shaped.

#### 10. Nothing forbidden is committed

`git diff --name-only origin/main...HEAD` lists only `.rs`, `.md` and `.sh` files. There are no
TIFF, DNG or binary files. My SDK outputs and mutant copies live only in my scratchpad.

#### 11. Gates

All run locally on `f73e020`, which is `23ece24` plus the handoff file only.

| Gate | Result |
|---|---|
| `just test` with corpus and SDK variable | 774 passed, 0 failed, 1 ignored (the existing SPEC-017/SB-1 Q2M ignore); SDK lane ran |
| `just test` with corpus, SDK variable unset | 774 passed, 0 failed, 1 ignored; `sdk-stage2: SKIP` printed |
| `lint-ci` (clippy 0.1.98 pinned) | green |
| `lint-no-allow` | green |
| `fmt-check` | green (exit 0) |
| `validate` | 83 artifacts valid, 358 parse |
| `doc-check` | 2 surfaces, 0 rustdoc warnings |
| `cost-audit` | green |
| `gate-audit` | 13 scripts, no mute-death shape |
| `name-length-check` | 881 paths, none over 150 bytes |
| `decisions-index --check` | up to date (70 decisions) |
| `semver-check` | 202 pass, no update required |
| CI on PR #276 | 22 / 22 pass |

I did not re-run `fuzz-opcode`. It is outside this brief's item list. The build ran it for 60 s
(2,913,628 runs) and the change touches no parser.

### Cost self-report

- **Tokens (total):** see the `handback:` front matter. Re-summed from this session's transcript
  `ad04463f-192f-4249-9efc-b8e3e628fb47.jsonl`, found by the scratchpad UUID, deduplicated by
  `message.id` with the largest `output_tokens`. There were no subagents. It is a floor taken
  just before the commit.
- **Estimated USD:** see front matter. Opus-5-5 rates: $5/Mtok input, $25 output, cache read
  ×0.10, cache write ×2.0. No discount beyond those.
- **Duration (minutes):** about 35 of wall time; the mutant batch dominated.
- **Source of the number:** transcript.

### Drift and new artifacts

- **New decisions emitted:** none. **One DEC-070 amendment to record at ship** (item 3): §3's
  refusal applies where a levels check exists, which is mono `PerPosition`. The colour path stays
  total, and `W ≤ B` develops to 0.
- **Deviations from spec:** none beyond the build's approved 13 oracle updates and its item-3
  reading, which this verify rules acceptable.
- **Follow-up work identified:**
  - **FU-1 (follow-up): colour levels refusal.** Should the four colour entries refuse
    `W ≤ B` with `InvalidLevels`, like mono? Today a malformed colour file silently develops to 0
    on every photosite. Since this PR, that includes a file whose first black is below white. The
    change is behavioural and needs its own spec and a CHANGELOG entry. `Error` already has the
    variant.
  - **FU-2 (follow-up, tests only): a tier-A colour fixture whose maximum black is not last.**
    Every tier-A colour fixture puts its largest black in the last slot: AC1's
    [2047, 2047, 2047, 2048] and `SLOT_BLACK` [200, 400, 600, 800]. So a "`B` = last slot" colour
    bug is caught only by the tier-B Pixel pins, which CI does not run. A one-test fix: repeat the
    shifted-uniform equivalence with the Pixel order [1023, 1023, 1022, 1022], where the maximum
    is first.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The brief's "check out `claude/spec-078-verify-base`" collided with the
   orchestrator's worktree, which had that branch checked out. I used a local tracking branch and
   push by refspec. Also, "B over the whole tag rather than the applied pattern" turned out to be
   equivalent by construction, because `len == R × C` is enforced. A sentence saying so would have
   saved a mutant.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The colour path's deliberate totality: `normalize_sample`'s "0 rather than a panic", and no
   `InvalidLevels` in the colour `# Errors`. It is the fact that settles item 3, and neither
   DEC-070 nor the spec states it.

3. **If you did this task again, what would you do differently?**
   — Start the 13 mutant copies before reading the diff, since they took most of the wall time,
   and run them at most three at a time against a warm shared dependency build.
