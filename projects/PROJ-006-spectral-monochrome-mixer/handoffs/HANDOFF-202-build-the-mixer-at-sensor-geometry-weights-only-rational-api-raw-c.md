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
  id: HANDOFF-202
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-092

project:
  id: PROJ-006
  stage: STAGE-031
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
  tokens_total: 27463707           # REAL combined count — what cost-audit reads
  estimated_usd: 19.45             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 38
  branch: feat/spec-092-mono-mixer
  pr: 354
  completed_at: 2026-11-26         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 0a0649c3-731f-4348-ac40-9bf00f022115.jsonl (106 unique message.ids, all claude-opus-5-5, no subagents): input 214 + cache-creation 319,410 (1-hour) + cache-read 27,034,526 + output 109,557, priced at 5 and 25 USD per million. Re-sum at ship. CI run 45855649150 on b822fa2 green on both determinism legs."
  synced_at: 2026-11-26
---

# HANDOFF-202: The mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip, the plane's black and white levels

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-092` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle: three public functions in
`src/color.rs` that mix a colour Bayer DNG to a mono `u16` plane at sensor geometry (whole
plane and row band) and expose the fixed-point row, six public constants (one the mixer version), three `Error`
variants, the fade of negative weights near clip in exact integer arithmetic, a
cross-architecture determinism hash, and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (read "onyx's reply to the design (2026-11-25)")
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md` (Design §1 to §11, AC1 to AC15, Failing Tests, mutants (a) to (u))
- **Decisions:** `./decisions/DEC-083-the-mono-mixer-row-is-i32-with-24-fraction-bits-and-its-fade-starts-at-57343.md` (the numbers) and `./decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md` (the approach and the fade)
- **The shipped sibling:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/done/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` and its handoffs HANDOFF-189 / HANDOFF-198 (`## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-092-mono-mixer`, https://github.com/jysf/irradiance/pull/354
- **Completed at:** 2026-11-26
- **All acceptance criteria met?** yes. AC1 to AC15 are green locally; AC12's two CI determinism legs: CI run 45855649150 on `b822fa2`, all 25 checks green, including `rust / determinism (ubuntu-latest, x86_64)` and `rust / determinism (macos-latest, arm64)` (each runs the determinism file in dev and release), so the four mixer hashes recorded on arm64 reproduce on x86_64. One test was strengthened after the mutant run (AC8, below); no AC was edited and no golden value was changed.
- **The change.** `src/color.rs` generalises SPEC-091's sink: `develop_color_with`, `srgb_linear_plane`, `srgb_linear_plane_rows` and `srgb_linear_plane_rows_with` take a `const C` (samples per pixel: 3 for RGB, 1 for mono) and a `build` step that makes what `store` reads (the matrix, or the mixer's row). `build` runs inside the fused traversal's `setup` (whole plane) or after `check_supported_bayer` (band), so `MixRowOutOfRange` follows the CFA and calibration errors and precedes any write. `check_plane_buffers` takes a `PlaneSamples` (Rgb or Mono) that sizes `dst` and names its refusal (`RgbBufferWrongLength` or `MonoBufferWrongLength`). The mono entries check the weights first, then call the same two bodies the RGB plane uses; the traversal is not copied. The kernel `mono_mix_pixel` is one private function used by both entries; the row is `mix_weights` (i128 check, one f64 division each), `fold_mix_row` (Design §8's private function) and `quantize_row_entry`. Saturating `i64` operations satisfy the crate's lints and never engage (AC14); the final clamp precedes an `as u16` so that dropping it wraps (mutant k is observable).
- **Red first.** The tests were written in the same session as the implementation rather than strictly before it (see Deviations). Against `origin/main`'s `src/color.rs` and `src/lib.rs`, `tests/mono_mix.rs`, `tests/determinism.rs` and `tests/error_kind.rs` fail to compile (the three functions and `MONO_MIX_VERSION` unresolved; the three variants missing). The 34 hand-applied mutants below are the red-proof of every oracle.
- **AC5, the fifteen golden rows matched as registered** on the first run (luminance, red, ortho on Canon, Pixel 3a, Pixel 4a, Pixel 7 Pro, Nikon), with `MONO_MIX_VERSION == "1"` asserted beside them. The fifteen mono-plane digests (FNV-1a, little-endian, a local record) were recorded at build on macOS arm64: Canon `0xf5b2_d791_3c4b_841c` / `0x8209_545e_a5a3_0323` / `0xc48b_0756_2cd2_228c`; Pixel 3a `0xa95d_e8a9_4cc9_276d` / `0x97ca_2799_20a6_9c85` / `0x033a_2108_715c_b32e`; Pixel 4a `0x46fa_545d_cdd1_fdcc` / `0x6dcb_a961_5f58_852a` / `0x4c7e_dfd2_6880_15c9`; Pixel 7 Pro `0x4443_fa3b_6795_3787` / `0xb52a_9e5a_4549_db21` / `0xfe77_f164_558c_3e5d`; Nikon `0x0287_b989_6a6a_f991` / `0xd060_9abb_fb0b_2766` / `0x39b6_7f36_f3a6_4295`.
- **AC4, the luminance identity.** Rows with a negative entry are exactly the Canon's and Nikon's (asserted). Kept pixels: Canon 23,380,901 of 23,384,000 (99.9867%), Nikon 12,480,541 of 12,481,536 (99.9920%), every pixel on the three Pixels. Largest deviation from `clamp(65535·Σ w_j·f_j)`: Canon 0.501281, Pixel 3a 0.501509, Pixel 4a 0.502518, Pixel 7 Pro 0.503160 (the largest), Nikon 0.501847 codes, all inside the bound. Pixels more than one code from the clamped `u16` plane's mix: Canon 574,961 (2.46%), Pixel 4a 616,584 (5.06%), both above zero as required.
- **AC6.** Zero decreasing rays on each of the five rows over the 90,000-ray grid. The control, the plain fold on the Canon luminance row, decreases on 7,199 rays, the probe's count.
- **AC1, AC3, AC7 on Tier A.** The odd-ActiveArea fixture's luminance row `[6274658, 11754804, 2827626]` has no negative entry (AC3; 149 pixels have `m ≥ 57344`); its first one-hot set with a negative entry is red `[1, 0, 0]`, row `[50596018, -4038464, -6723495]` (AC1, where the fade changes at least one pixel, asserted). AC7's ramp is 48 flat 12×12 patches (camera ratio 1000 : 600 : 550, exposure 0.80 to 2.21 of white), with a ForwardMatrix whose Y row has a negative blue term (luminance row's blue entry negative, asserted): the mono value at each centre never decreases; 35 centres with a clipped channel differ from the plain fold; the plain fold itself drops from 65535 to 65474 when blue clips (asserted as a control).
- **AC2.** Tier A: no sentinel left, two calls agree, and the four SPEC-091 partitions (one strip, 1-row, 7-row, a split at row 1 inside the masked border) equal the whole plane for luminance and the negative one-hot set. Tier B: Pixel 7 Pro and Canon, a 1-row lead then 256-row strips with an odd remainder, equal the whole plane.
- **AC12.** On SPEC-091's colour fixture (bytes unchanged), luminance and the **red one-hot set** (index 0, the first with a negative entry, asserted): rows `[8472749, 11496691, 2367239]` and `[59354934, -5925351, -6460732]`; committed hashes rows `0xe837_0a06_b8c6_e1cb`, `0x55b0_e759_ac2e_6883`, planes `0x7b07_6b08_b1b5_1de1`, `0x0f50_77a8_76c2_fbd3`; the fade changes 8 of the 384 pixels. The extended `colour_determinism_red_proof`: the one-bit sample flip moves both plane hashes and leaves both row hashes; AsShotNeutral red `9/20 → 10/20` moves both row hashes. CI: CI run 45855649150 on `b822fa2`, all 25 checks green, including `rust / determinism (ubuntu-latest, x86_64)` and `rust / determinism (macos-latest, arm64)` (each runs the determinism file in dev and release), so the four mixer hashes recorded on arm64 reproduce on x86_64.
- **AC10 and the tables.** The two entries joined `tests/malformed_tags.rs`, `tests/misshapen_tags.rs` (and its `COLOUR_READERS`), `tests/unhonoured_tags.rs` and `tests/required_processing.rs` (its `ColourEntry` table and the staged-crate probe's colour list), the band over `0..H`. Counts grew by exactly the new rows: `malformed_tags` AC1 from `7*11 + 2*10 + 11*9` to `7*13 + 2*12 + 11*11`, `required_processing`'s every-entry sentinel harness from 13 to 15, `error_kind`'s enum from 79 to 82 (`CallerError` 19 to 21, `Unsupported` 27 to 28). `tests/error_kind.rs` gained exactly three rows, and DEC-051's later-additions paragraph records them.
- **AC9.** `fuzz/fuzz_targets/demosaic.rs` calls the three entries: half the inputs with luminance, the rest with three signed-byte rationals (zero and negative denominators, negative numerators and wrong sums reachable), and `dst` one short or one long on half the inputs for both planes. Smoke run: 2,008,218 runs in 61 s, no crash.
- **AC15, mutants** (each applied by hand by a scripted string replacement, the lib `color::tests`, `tests/mono_mix.rs` and `tests/determinism.rs` run with the corpus, then the file restored from git; failing tests listed, the spec-named test in bold):

  | Mutant | Applied as | Failed |
  |---|---|---|
  | (a) row transposed | `fold_mix_row` reads `M[i][j]` | **`ac5_golden_rows_and_digests_of_the_held_files`**, **`ac4_luminance_matches_the_unclamped_plane`**, `ac11_the_row_fuses_no_term_of_any_channel`, `ac11_the_row_keeps_its_summation_order`, `mono_mix_hashes_match_committed`, `colour_determinism_red_proof` |
  | (b) half to even | `t.round_ties_even()` | **`ac11_the_row_rounds_half_up`**, `ac14_no_overflow_at_the_extremes` |
  | (c) half away from zero | `t.round()` | **`ac11_the_row_rounds_half_up`** |
  | (d) scale 2^23 in the row only | `MIX_ROW_SCALE = 2^23` | **`ac5_…`**, **`ac4_…`**, `ac11_the_row_rounds_half_up`, both AC12 tests |
  | (e) fade removed | `N·8192` for `N·u` | **`ac6_the_fade_is_monotone_on_the_saturating_ray_grid`**, **`ac7_a_ramp_through_clipping_stays_monotone`**, `ac8_…`, `ac1_…`, the random-row oracle test, `ac5_…`, both AC12 tests |
  | (f) `m` from green only | `m = x[1]` | **`ac8_the_fade_starts_after_57343_and_reads_the_largest_channel`**, `ac1_…`, the random-row oracle test, `ac5_…`, both AC12 tests |
  | (g) `m` the mean | `m = (r + g + b)/3` | **`ac8_…`**, `ac6_…`, `ac1_…`, the random-row oracle test, `ac5_…`, both AC12 tests |
  | (h) the fade scales positive terms too | `P·u` for `P·8192` | **`ac3_no_negative_row_is_the_plain_fold`** (unit), **`ac1_…`**, `ac3_no_negative_row_plane_is_the_plain_fold`, `ac6_…`, `ac7_…`, `ac8_…`, `ac14_…`, `ac4_…`, `ac5_…`, both AC12 tests |
  | (i) `u` not capped | `.min(8192)` removed | **`ac8_…`** (after the strengthening below), **`ac1_…`**, the random-row oracle test, `ac4_…`, `ac5_…`, both AC12 tests |
  | (j) threshold off by one | `65536 − m` | **`ac8_…`**, `ac6_…`, `ac1_…`, the random-row oracle test, `ac5_…`, both AC12 tests |
  | (k) upper clamp dropped | `.max(0) as u16` | **`ac14_no_overflow_at_the_extremes`**, and every kernel test that reaches white |
  | (l) `mul_add` at each of the 9 fold sites | one run per channel and site | **`ac11_the_row_fuses_no_term_of_any_channel`** on all nine, and only it |
  | (m) fold pairs reordered, two per channel | `(t0 + t2) + t1` and `(t1 + t2) + t0`, six runs | **`ac11_the_row_keeps_its_summation_order`** on all six, and only it |
  | (n) weight sum checked in f64 | the i128 check replaced by `w0 + w1 + w2 != 1.0` | **`ac10_refusals_in_order_leave_dst_untouched`** (`[2/10, 7/10, 1/10]` refused) |
  | (o) `num·(1/den)` | `weight_to_f64` | **`ac11_a_weight_converts_by_one_division`** |
  | (p) weights after the file checks | the required-processing check moved before `mix_weights` in `mono_mix_plane_into` | **`ac10_…`** (the invalid weight on the opcode file) |
  | (q) `dst` written before `MixRowOutOfRange` | an out-of-range row mixed as zeros, the error returned after the write | **`ac10_…`** (the tiny-neutral sentinel) |
  | (r) band: halo dropped / local top not even / `dst` from row 0 | `BAND_HALO = 0`; `& !1` removed; `first = 0` | **`ac2_every_pixel_written_and_strips_tile`** on all three; `ac2_strips_tile_on_held_files` on the halo and row-0 mutants (Tier B starts are odd, so `start − 3` is even and cannot see the odd-top mutant, as in SPEC-091) |
  | (s) CFA phase from the plane origin | `(0, 0)` in `develop_color_fused` | **`ac1_the_plane_equals_the_i128_oracle_on_demosaic_triples`**, `ac3_no_negative_row_plane_is_the_plain_fold` |

  **Equivalent, argued, not run.** (t) A division truncating toward zero instead of the floor shift: the two differ only when `P·8192 + N·u + 2^36` is negative and not a multiple of `2^37`, where the shift gives a value at most −1 and truncation a value at most 0; the lower clamp sends both to 0. (u) The sign test made on `v_i` instead of `q_i`: they disagree only where `q_i = 0` (a `v_i` within `2^-25` of zero), and a zero coefficient adds nothing to either `P` or `N`.
- **Gates run locally:** `just test` with `IRRADIANCE_CORPUS_DIR` set (960 passed, 0 failed), `just lint-ci` (clippy stable, which caught one reversed range literal and two `chunks_exact` uses in the new tests), `just fmt-check`, `just doc-check` (both feature surfaces, 0 warnings), `just validate`, `just semver-check` (202 checks pass, no semver update required; it does not list additions: the diff adds exactly the three functions, six constants and three variants), `just cost-audit`.

### Cost self-report

- **Tokens (total):** 27,463,707 (floor, measured before the handback commit)
- **Estimated USD:** 19.45
- **Duration (minutes):** about 38 by the transcript's own timestamps; the wall-clock includes the local suite, the mutant run and the CI wait
- **Source of the number:** this session's transcript (`0a0649c3-731f-4348-ac40-9bf00f022115.jsonl`, found by the scratchpad UUID), deduplicated by `message.id` with the largest `output_tokens` per message, per docs/cost-tracking.md; priced at claude-opus-5-5's 5 and 25 USD per million, cache reads at 0.1x and 1-hour cache writes at 2x. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-051's later-additions paragraph gained one sentence for the three variants.
- **Deviations from spec:**
  - **Tests were not written strictly before the implementation.** The library code and the tests were written in the same session, the code first. The red state is shown two ways instead: the new tests do not compile against `origin/main`'s `src/`, and every pre-registered mutant fails a named test.
  - **AC8 gained one point.** As written, AC8's triples could not see mutant (i) (`u` uncapped): its only below-threshold triple has `m = 57343`, where `65535 − m = 8192` exactly. `ac1_…` and the random-row oracle caught (i), as the spec also lists, but `ac8_…` did not. The test now also asserts `x = [0, 1000, 500]` with `q = [0, 2^30, −2^30]` gives 32000 (the plain fold), which the mutant drives to 0; re-run, (i) fails `ac8_…`. Committed separately.
  - **Extra unit test.** `mono_mix_kernel_matches_the_oracle_on_random_rows` (20,000 random rows and triples against the `i128` oracle, biased to the fade band) is not in the Failing Tests list; it catches most kernel mutants independently of the corpus.
  - **AC7 control.** The ramp also asserts that the plain fold decreases somewhere along it, so the monotone check is shown able to fail on that fixture.
  - **The RGB plane entries were moved onto the generalised bodies** (`PlaneSamples::Rgb` plus the matrix as their `build`). Their output and hashes are unchanged.
  - **`ac6_…` is slow unoptimised:** 135 s in the plain `test` profile (CI's debug test job), under a second in `just test`'s `corpus` profile. The grid is as specified.
- **Follow-up work identified:**
  - FU-9 stays closed as "measured, not worth it" (Design §7); nothing new.
  - [XS] If CI time matters, `ac6_…` could start each ray at the last `k` below the threshold (the kernel is exactly the linear plain fold there, which a separate test pins), cutting its unoptimised time several-fold. Not done: the spec's grid says `k` from 0.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. Design §8's "keep the order" example terms `t = [1, 2^-52, 2^-53]` distinguish only the `(t0 + t2) + t1` pairing; `(t1 + t2) + t0` rounds to the same value there, so that pairing uses `t = [2^-53, 2^-53, 1]`. AC6's "until every channel is at 65535" never ends for a zero channel; the probe's `kmax = 65535/min nonzero c + 1` was used.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The crate's `arithmetic_side_effects` lint was anticipated by the spec (saturating forms that never engage).

3. **If you did this task again, what would you do differently?**
   — Check each pre-registered fixture against its mutant by computing the mutant's value for every point the fixture lists, at design. That finds AC8's blind spot for (i) before build, the same way the FU-10 fixtures already assert their mutant's value differs.
