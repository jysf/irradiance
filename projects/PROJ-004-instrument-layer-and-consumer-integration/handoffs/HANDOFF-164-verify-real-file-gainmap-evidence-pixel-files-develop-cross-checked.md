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
  id: HANDOFF-164
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-060

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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: punch-list                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 11834498               # REAL combined count — what cost-audit reads
  estimated_usd: 8.62              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 15
  branch: claude/spec-060-verify-base
  pr: null
  completed_at: 2026-11-19               # YYYY-MM-DD
  notes: "Floor, measured live before the handback commit (opus-5-5, 1-hour writes); re-sum at ship. Punch P1-P3 are test-support fixes already in 89b310b."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-19
---

# HANDOFF-164: VERIFY: Real-file GainMap evidence and the dng_validate cross-check (SPEC-060)

## Delegation Summary

You are the **verifier** for SPEC-060. Review it cold: you did not build it.

- **Under review:** PR #269 (`claude/spec-060-build`, head e4e4537), built under
  HANDOFF-162. CI was green on all 22 checks.
- **Your branch:** `claude/spec-060-verify-base`. It is that head plus one commit that adds
  this handoff.

SPEC-060 changes no `src/`. It adds:
- `scripts/sdk-stage2.sh`, which makes the Adobe `dng_validate -2` stage-2 outputs
  before and after `OpcodeList2`;
- a test-support stage-2 TIFF reader;
- `tests/gain_map_corpus.rs`, holding the Pixel develop tests, 15 MD5 pins, the amended
  AC2, and the env-gated SDK cross-check with pre-registered bounds;
- a `corpus-status` SKIP line, an AGENTS.md §6 entry, and the provenance evidence.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-060-real-file-gainmap-evidence-pixel-files-develop-cross-checked.md`
  (Design §1–5, AC1–AC7, and AC2 as amended on the build branch).
- **Decisions:** DEC-069 (§2 the rule, §6 the SDK reference, §Context's probe table),
  DEC-068 (why the 7 Pro's no-gain differences are negative), DEC-059 (warped
  `FullActiveArea` refusal), DEC-003 (corpus).
- **Build handback:** HANDOFF-162's `## Handback`, with its numbers, red-proof and
  deviations. Check each claim yourself.
- **The SDK:**
  - The binary is at
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate your **own** stage-2 outputs with the committed script into your scratchpad,
    and point `IRRADIANCE_SDK_STAGE2_DIR` at them. Don't reuse the builder's directory.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The recipe reproduces from scratch.** Run `scripts/sdk-stage2.sh` cold into a new
   directory, then run the cross-check against it.
   - Do your numbers match the build's table?
   - Does the script write nothing inside the repo?
   - Does it refuse when the tag pattern is missing or appears twice? Test both, on a
     scratch copy.
   - Is the 51009 → 51010 rewrite safe? It must not match a value or offset that merely
     contains the bytes. Check how it locates the IFD entry.
2. **The bounds are pre-registered, not fitted.** Compare the bounds in the test with
   SPEC-060 Design §4 as designed in PR #266:
   - gain step ≤ 1 and ≥ 99.9% exact;
   - end to end ≤ 8;
   - no gain ≤ 5, with every difference ≥ 0 on the 4a and 3a.

   Any change must be a deviation the orchestrator approved. Confirm the comparisons use
   the right inputs, as Design §4 lists them:
   - `pre.tif` + `apply_gain_maps` against `post.tif`;
   - mono `FullActiveArea`, unoriented, warp 0, `PerPosition`, against `post`;
   - no-gain against `pre`.
3. **The cross-check can fail.** Re-run the build's corner-sampling red-proof. Then try at
   least:
   - per-map rounding removed;
   - one Bayer phase's map skipped;
   - the gain applied in raw coordinates rather than `ActiveArea` coordinates.

   Each should turn the gate red on some file. Also mutate the **test** to show the
   comparison isn't vacuous. Would a reader that returns all zeros, a wrong
   width/height pairing, or a byte-order swap be caught?
4. **The TIFF reader.** It must refuse anything that isn't uncompressed, single-sample,
   16-bit and stripped. It must handle both byte orders and multi-strip files, and
   bounds-check its offsets. Read it for panics on hostile input. It's test support, but
   a silent misread would make AC4 meaningless.
5. **AC2 as amended.** Confirm three things:
   - the corner and centre windows and the phase are named;
   - "corner gain > centre gain" is computed from the map's own parsed gains, not from
     the output;
   - "toward 1" is asserted only for the 3a and 7 Pro.

   Recompute one file's two ratios yourself.
6. **The AC1 warp deviation.**
   - At `warp_alpha` 1.0, `FullActiveArea` is refused on the 3a but develops on the 4a and
     7 Pro. Is that consistent with DEC-059's rule, given each file's warp?
   - Does the test assert each file's outcome exactly?
7. **The pins (AC3).**
   - The 15 MD5 pins are new; no existing pin changed (`git diff origin/main -- tests/`).
   - Pins are stable: run them twice, and threaded against single-thread.
   - They don't duplicate SPEC-059's `plane_tiles` and `develop_options` Pixel
     assertions in a way that drifts.
8. **CI visibility.** With `IRRADIANCE_SDK_STAGE2_DIR` unset, the cross-check prints a
   visible SKIP and passes. `corpus-status` shows the SKIP line. Nothing in CI tries to
   run the SDK.
9. **Nothing forbidden is committed:** no SDK bytes, no SDK outputs, no corpus bytes. The
   provenance-ledger text is accurate.
10. **Gates:**
    - `just test` with the corpus, both with and without the SDK variable;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants and your own
  cross-check numbers.
- **Commit this handoff's filled handback** to `claude/spec-060-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-060-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. Any
  `src/` change goes in the punch list. A measured number outside a pre-registered bound
  is reported, never fixed by widening the bound.

## Out of Scope

- The max-black normalisation follow-up (DEC-069 §Context).
- Running the SDK in CI, vendoring it, or committing its outputs.
- The Samsung SM-G960F.
- A colour comparison with the SDK's stage 3.
- SPEC-059 rework.
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

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-060-verify-base` (no PR, by instruction). The fixes are
  commit `89b310b`; the handback is the commit after it. Nothing was pushed to
  `claude/spec-060-build`.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes. AC1 to AC7 hold at e4e4537; every pre-registered
  bound holds on my own SDK outputs without widening. The three punch items are in the
  test-support TIFF reader, not in any AC's evidence path for these files.
- **For `verify`:** ⚠ **PUNCH LIST**, tests only, already fixed on this branch at
  `89b310b`. To close it, cherry-pick `89b310b` onto `claude/spec-060-build`, or
  fast-forward the build branch to this one. No `src/` change was needed or made.

#### Punch (all fixed in `89b310b`; none ship-blocking, none reaches a consumer)

- **P1 (tests).** The reader's multi-strip join was never exercised. `dng_validate`
  writes one strip (StripOffsets 146, RowsPerStrip = height, little-endian, all three
  files), and the tier-A tests also used one strip. My mutant T4 (drop the last strip)
  survived the whole lane. Fix: `stage2_reader_joins_strips_in_offset_order`, two
  one-row strips stored in reverse file order, both byte orders. Red on T4 and on a
  "concatenate strips in file order" mutant.
- **P2 (tests).** `Vec::with_capacity(expected)` trusted the header's
  `ImageWidth × ImageLength`. A hostile 2^31 × 2^31 header panicked with
  `capacity overflow` (smaller ones would abort in the allocator) before the
  strip-length check could refuse it. Fix: the reservation is capped at the file length.
  New test `stage2_reader_refuses_sample_format_and_hostile_size`; it is red with the cap
  reverted (observed: `capacity overflow`).
- **P3 (tests).** `SampleFormat` (339) was not checked, so a signed or float 16-bit
  stage 2 would be read silently as unsigned. Fix: anything but 1 (or absent) is refused;
  same new test, red when the check is disabled.

#### Follow-ups (not holding the spec)

- **FU-1.** `scripts/sdk-stage2.sh` finds the `OpcodeList2` entry by the uniqueness of a
  4-byte tag+type pattern over the whole file, not by walking the IFDs. On all three
  files I walked the IFDs (IFD0 and SubIFDs) independently: the single match is the
  51009 entry (bytes 121934 / 1488 / 11224), and `cmp` shows exactly one byte changed.
  A file with no `OpcodeList2` and one chance match in pixel data would be silently
  corrupted, but the test backstops it (it panics on "no OpcodeList2" and asserts
  pre ≠ post). Walking the IFD would remove the residual risk.
- **FU-2.** The script runs `mkdir -p` before its in-repo refusal, so a refused in-repo
  `<out_dir>` is left behind empty. Cosmetic; git ignores empty directories.
- **FU-3.** With `IRRADIANCE_SDK_STAGE2_DIR` set but the corpus absent, the SDK tests pass
  after per-file corpus SKIP lines. Visible, not silent; noted for a future
  "set means must run" rule.

#### The ten items

1. **Recipe from scratch.** `scripts/sdk-stage2.sh` ran cold into my scratchpad in 4.3 s.
   All six TIFFs are byte-identical (MD5) to the builder's. `git status --ignored` was
   clean before and after. Refusals seen on scratch copies: pattern found 0 times
   (the script's own nogain copy as input) and 2 times (4 bytes appended), both exit 1
   with the count named; an in-repo `<out_dir>` is refused. The MM pattern bytes are
   correct (`C7 41 00 07`). Rewrite safety: see FU-1.
2. **Bounds pre-registered.** `git diff a30b86d HEAD` (PR #266's merge) on the spec shows
   Design §4 unchanged; only AC2, a Design §3 build note and the cycle changed. The test's
   constants are 1, 0.999, 8, 5 and non-negative on 4a/3a. Inputs match Design §4:
   `apply_gain_maps(pre)` vs post; `stage2_like()` (FullActiveArea, unoriented, warp 0,
   PerPosition) vs post; no-gain vs pre.
3. **The cross-check can fail.** See the mutant table. The gate is red on corner sampling,
   truncation, a skipped Bayer phase and crop-origin coordinates, and on a zeroed,
   size-swapped or byte-swapped reader.
4. **The reader.** Refuses compression ≠ 1, spp ≠ 1, bits ≠ 16, tiles, a bad byte-order
   mark and a short strip (existing tests). Offsets are bounds-checked through `slice`
   with `checked_add`; IFD arithmetic cannot overflow `usize` on 64-bit. Defects: P1–P3.
5. **AC2 as amended.** Phase (0, 0), 128-px windows, corners vs centre named in the test.
   The map's corner/centre gain is parsed by hand from `OpcodeList2`, not from the output.
   "Toward 1" is asserted only where the ratio starts below 1 (3a, 7 Pro). My own Python
   recompute (gains from `exiftool -b -OpcodeList2`, ratios from the SDK's own pre/post
   TIFFs, so independent of the crate): corner/centre gain 3.2939/1.0177,
   2.7222/1.0401, 1.4457/1.0192; ratios 4a 3.1962 → 9.3125, 3a 0.2190 → 0.5454,
   7 Pro 0.2074 → 0.2842 (ours 0.2075 → 0.2843).
6. **AC1 warp deviation.** Parsed each file's `WarpRectilinear`: 4a is the identity
   (f ≡ 1); the 7 Pro's radial scale spans [0.9897, 0.9995]; the 3a's reaches 1.0015,
   outside DEC-059's [0, 1]. So refused on the 3a only is exactly DEC-059's rule. The test
   asserts the refusal's variant on the 3a and `Ok` on the other two.
7. **Pins.** `git diff origin/main...HEAD -- tests/` is two added files, no removed line;
   no existing pin changed. The 15 pins passed in at least 10 runs (single-thread
   harness, parallel harness, both `just test` runs, every test-only mutant copy).
   Threaded colour equals single-thread inside the test. No drift with SPEC-059:
   `develop_options` and `plane_tiles` assert only `Ok` and cross-entry equality for the
   Pixels and point at SPEC-060 for the pins.
8. **CI visibility.** Unset, `corpus-status` prints `sdk-stage2: SKIP …` (seen in the
   no-SDK `just test` log) and the two SDK tests return early after an `eprintln!` SKIP.
   Nothing under `.github/` mentions the SDK, `dng_validate` or the variable.
9. **Nothing forbidden.** The diff adds four text files (the handoff, the script, the
   test and the reader); `git diff --numstat` shows no binary. The ledger text matches my
   numbers, including the red-proof's 32 / 20 / 13 and 63.8 / 37.4 / 70.5%.
10. **Gates.** At e4e4537: `just test` with the SDK variable 762 passed, 0 failed,
    1 ignored; without it the same, with 12/12 corpus files and the SKIP line.
    `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`,
    `gate-audit`, `name-length-check` all exit 0. At `89b310b`: the eight
    `gain_map_corpus` tests pass with the SDK, and `lint-ci`, `lint-no-allow`, `fmt-check`
    exit 0. `decisions-audit --changed` names DEC-004 and DEC-010; the change is
    consistent with both (no dependency; the image comparison is DEC-069 §6's SDK lane).

#### My cross-check numbers (my own SDK outputs; signed SDK − ours)

| File | Gain step: max / exact | End to end: max (min, max) / exact | No gain: max (min, max) / exact |
|---|---|---|---|
| 4a | 1 / 99.9794% {-1: 350, 0: 12,190,254, 1: 2,164} | 7 (-1, 7) / 94.3870% | 5 (0, 5) / 93.1660% |
| 3a | 1 / 99.9672% {-1: 567, 0: 12,188,768, 1: 3,433} | 6 (-1, 6) / 89.4246% | 5 (0, 5) / 89.3783% |
| 7 Pro | 1 / 99.9664% {-1: 565, 0: 11,347,156, 1: 3,247} | 4 (-4, 2) / 28.3394% | 2 (-2, 1) / 28.3471% |

Every histogram equals the build's to the count.

#### My mutants (each in its own copy with its own `CARGO_TARGET_DIR`, control first)

| Id | Mutation | Result |
|---|---|---|
| control | none | 6/6 pass |
| M1 | `gain_at` drops both `+ 0.5` (the build's red-proof) | red: 4a 32 / 63.8347%, 3a 20 / 37.3958%, 7 Pro 13 / 70.5469% (assert disabled to measure all three); end to end and pins red too |
| M2a | per-map rounding removed (round once after all maps) | equivalent on the Pixels (each sample meets exactly one map, pitch 2); red in the full suite on SPEC-059's `ac2_apply_gain_maps_equals_the_oracle_bit_for_bit` and `ac3_hand_values` |
| M2b | truncate instead of round half up | red: gain step 4a max 1 but 51.72% exact |
| M3 | skip the phase-(1, 1) map | red: gain step max 44160, 75.33% exact; end to end and pins red |
| M4a | gain in raw coordinates instead of `ActiveArea` (mono) | equivalent on the Pixels: all three have `ActiveArea` 0 0 h w, so raw = stage 2; red in the full suite on SPEC-059's `ac4_mono_develop_equals_the_oracle_gain_then_the_existing_pipeline` |
| M4c | gain in crop-origin coordinates (offset 8, 8) | red: gain step max 6446, 11.86%; end to end 6441; pins red |
| T1 | reader returns all zeros | red: "the SDK's pre and post are identical"; end to end 65535 |
| T2 | reader swaps width and height | red: "SDK stage 2 is ActiveArea-sized" |
| T3 | reader swaps byte order | red: gain step max 65534, 2.52% |
| T4 | reader drops the last strip | **survived** at e4e4537 (single-strip files) → P1; red on the new test |
| R1 | reader capacity uncapped | red on the new test (`capacity overflow` panic) → P2 |
| R3 | strips joined in file order, not offset order | red on the new test |
| R4 | `SampleFormat` check disabled | red on the new test → P3 |

### Cost self-report

- **Tokens (total):** 11834498 (a floor: measured live just before the final commit)
- **Estimated USD:** 8.62
- **Duration (minutes):** about 15 wall-clock, from the transcript's first and last timestamps
- **Source of the number:** this session's transcript `656602ed-7f9c-4c4a-9d79-296447e4071e.jsonl`,
  found by the scratchpad UUID, no subagents; deduplicated by `message.id` with the
  largest `output_tokens`; priced at opus-5-5 list rates (in 5, out 25 per MTok, cache
  reads ×0.10, 1-hour writes ×2.0).

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verifier. The build's two approved deviations
  (AC2 amended; AC1 `FullActiveArea` at `warp_alpha` 0) were checked and hold (items 5
  and 6).
- **Follow-up work identified:** FU-1 to FU-3 above, all small; none needs its own spec.
  Worth knowing: the raw-coordinate class of bug cannot be seen by any real-file test in
  the held corpus, because every Pixel's `ActiveArea` starts at (0, 0). SPEC-059's twin
  fixtures are the only guard for it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — "Each should turn the gate red on some file" for the raw-coordinate mutant cannot
   hold on this corpus (`ActiveArea` origin 0, 0), and per-map rounding cannot be seen
   when each sample meets one map. Both needed a full-suite run to place the guard in
   SPEC-059's tests.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That `dng_validate` writes a single little-endian strip. It means the reader's
   multi-strip and big-endian paths are reached only by tier-A tests, which is why P1
   existed.

3. **If you did this task again, what would you do differently?**
   — Check the SDK output's TIFF layout first; it predicts which reader paths the real
   files exercise and points straight at the untested ones.
