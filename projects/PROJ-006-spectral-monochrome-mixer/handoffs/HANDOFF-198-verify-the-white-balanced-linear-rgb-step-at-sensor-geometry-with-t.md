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
  id: HANDOFF-198
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-091

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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 19052713               # REAL combined count — what cost-audit reads
  estimated_usd: 12.62              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-091-verify-base
  pr: https://github.com/jysf/irradiance/pull/343
  completed_at: 2026-11-25               # YYYY-MM-DD
  notes: "Floor, measured before the final commit. Approved at code ea65f2d; verify added three test guards, a fuzz buffer-length variation and two doc sentences, no src change outside the test module."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-25
---

# HANDOFF-198: VERIFY: the white-balanced linear RGB step at sensor geometry (SPEC-091)

## Delegation Summary

You are the **verifier** for SPEC-091. Review it cold: you did not build it.

- **Under review:** PR 343 (`feat/spec-091-srgb-linear-plane`). The code is commit
  `ea65f2dc`; CI run 45666936351 on it was green, including both `determinism` legs. The
  two later build commits (`3ba0966`, `f4210b8`) touch the handoff, spec and timeline only.
  The build ran under HANDOFF-189 in a separate session.
- **Your branch:** `claude/spec-091-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already advanced the spec to `verify`.

SPEC-091 adds, in `src/color.rs`:
- `srgb_linear_plane_into` and `srgb_linear_plane_f32_into`, which write the white-balanced
  linear-sRGB plane at sensor geometry (every pixel of `width × height`).
- The row-range forms `srgb_linear_plane_rows_into` and `srgb_linear_plane_rows_f32_into`,
  built on `develop_rows_banded`.
- One new `Error::InvalidRowRange`, classified `CallerError`.

The `u16` form must equal today's colour pixels; the `f32` form is the unclamped `f64` sum
divided by 65535, cast once. It also adds a Tier-A colour fixture to `tests/determinism.rs`
with committed plane hashes, and golden digests of the five held files. No existing output
may change. The design took two asks from onyx (the downstream app): the `f32` plane must
be bit-identical on aarch64 and x86_64, and a band form must exist for iPad memory.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md`.
  Judge the ACs against the spec **as designed**: `git show 3ecdad74:<spec path>` (PR 340's
  merge). That is Design §1 to §8, AC1 to AC13, mutants (a) to (k), and the Failing Tests.
- **Decisions:** DEC-079 (accepted: working space, fold, arithmetic, clip-rule evidence);
  DEC-033 (f64 basic operations only); DEC-018 (`quantize`); DEC-016; DEC-040 (CFA phase from
  the ActiveArea origin); DEC-042; DEC-044 (row bands); DEC-051 (`Error::kind`, no wildcard);
  DEC-053, DEC-078 (required processing, malformed tags first); DEC-069 (GainMaps); DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-189's `## Handback`, in the same handoffs directory. It has the
  AC1 byte counts, the AC3 counts, the AC11 partitions, the committed hashes, a mutant table,
  and the deviations listed below.
- **Background:** `docs/determinism.md` and `.github/workflows/ci.yml`'s `determinism` job
  (SPEC-041).
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`.
  - Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  - Use `/usr/bin/git` for branch and HEAD.
  - Give each mutant run its own source state: touch the file after restoring it, or cargo
    may reuse a mutant build (the build hit this). Plain cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC against the spec as designed.** Each Failing Tests entry must exist and fail
   when its clause is wrong. Re-run the gates yourself; do not trust the build's numbers.
2. **The refactor of existing entries.** The build moved `develop_color_into`,
   `develop_color_f32_into` and both threaded colour entries onto a shared generic sink
   (`develop_color_with`, `PixelSlot`, `store_u16`, `store_f32_quantised`). This is the
   highest-risk change because it touches shipped outputs.
   - Confirm every existing colour output is unchanged: the existing Tier-B pins and the
     colour tests pass, serial equals threaded, and the four expressions keep today's
     operation order.
   - Diff the old and new sink bodies by eye.
3. **The four build deviations.** Rule on each:
   - (a) **AC4's slack** was widened from `0.5 + 1e-3` to `0.5 + 65535/2^25 + 1e-9`. Is the
     build's argument right (one f32 ulp in `[0.5, 1)`)? Is the new bound still tight
     enough to catch mutant (c)?
   - (b) **AC2 gained a check (iv).** It compares against the independent `demosaic_into`
     then `to_srgb_linear_into` composition, so mutant (f) is catchable. Is that composition
     really independent of `develop_color_fused`'s phase handling? Are the fixture's levels
     really the identity, so the comparison is exact?
   - (c) **Mutant (k)** (`mul_add` in the f32 sink) survives every hash. A unit test with a
     cancelling matrix was added for it. Does that test exercise the real sink, or a copy?
     Would it catch `mul_add` in any of the three channel sums?
   - (d) **`docs/determinism.md`** splits the colour row in two. Does each row claim no more
     than AC12 measures?
4. **Mutants.** Re-run (a) to (k) yourself, plus at least these extras:
   - The band forms skip the `InvalidRowRange` check, so `end > height` slips through.
   - `InvalidRowRange` is checked after `RgbBufferWrongLength`, not before.
   - The `f32` whole-plane form clamps only the negatives.
   - The u16 sink's operation order is swapped (`m02·b` first): AC1 must catch it if f64
     rounding differs.
   - `Error::kind` classifies `InvalidRowRange` as `Corrupt`.
   Give a table: mutant, how applied, which named test failed.
5. **Determinism evidence.**
   - Read CI run 45666936351's two `determinism` job logs. Confirm the arm64 leg really ran
     on arm64 and both legs asserted the new colour hashes.
   - Confirm the fixture's f32 plane holds a negative and an above-1.0 value, and that
     dual-illuminant interpolation really runs: a fixture that falls back to a single
     illuminant would not exercise it.
   - Is the red-proof real?
6. **Band memory.** The point of the band form is that internal scratch scales with the band,
   not the plane. Read `develop_rows_banded` and the band body. List every allocation and
   its size. Flag any `width·height`-sized buffer in the band path.
7. **No panics.** Check the new range arithmetic (`rows.end − rows.start`, multiplication by
   `width·3`, `u32` to `usize`, ranges near `u32::MAX`). Does the fuzz target reach
   `InvalidRowRange`, `RgbBufferWrongLength` and `Ok` for the band forms? Run 60 s of the
   `demosaic` fuzz target.
8. **Docs and API.**
   - The public doc comments must state the geometry, the unclamped f32 rule (and that it is
     not `develop_color_f32_into`'s), the refusal order and the band equality.
   - Check the CHANGELOG and DEC-051's added sentence.
   - `cargo semver-checks` must report no breakage. Confirm the public diff is exactly five
     items.
9. **Gates.** Read `gh pr checks 343` first. Then run:
   - `just test` with the corpus
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`
   - `deny`, `lint-fuzz`, `msrv`
   - `validate`, `gate-audit`, `cost-audit`, `just decisions-audit --changed`
   - `cargo semver-checks`

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and rulings on items 2, 3
  and 6.
- **Commit** this handoff's filled handback, and any tests or docs fixes, to
  `claude/spec-091-verify-base`, and push it. If you approve:
  1. Run `just advance-cycle SPEC-091 ship --verdict approved`.
  2. Run `just handback-sync SPEC-091` in the same push, so CI's cost audit passes.
  3. Fast-forward `feat/spec-091-srgb-linear-plane` to the verify tip, so PR 343 carries
     the verify commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** other sessions also mint IDs. Before minting a DEC or HANDOFF, compute the next
  free one from `origin/main` plus the open PRs.

## Out of Scope

- The mixer, the fold, the fixed-point row and the clip fade (SPEC-092). The developed mix
  (SPEC-093).
- Threaded band forms; a band form of the cropped `develop_color_into`.
- A reusable plan so many small strips don't re-resolve levels, GainMaps and the matrix.
  The build noted this for SPEC-092; note it, don't fix it.
- Any change to `DEVELOP_VERSION` or an existing hash. If one seems necessary, that is a
  rejection.

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

- **Branch / PR:** `claude/spec-091-verify-base`, fast-forwarded onto `feat/spec-091-srgb-linear-plane`, https://github.com/jysf/irradiance/pull/343
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** Yes, AC1 to AC13, judged against the spec as designed at `3ecdad74`, with the four build deviations ruled acceptable (below). Verify found three oracle gaps, all in the tests and none in the code; each is closed by a test-only addition on this branch.
- **For `verify`:** ✅ **APPROVED.** The code is commit `ea65f2d`, which is unchanged: no `src/` line outside `color.rs`'s test module moved. The verify tip is `8800550`, the last commit carrying code or tests; the commits after it touch the handoff, spec and timeline only.

**Gates, re-run here (macOS arm64):**
- `just test` with `IRRADIANCE_CORPUS_DIR` set, on the build's head: exit 0, 936 passed, 0 failed, 1 ignored. The corpus was present.
- After the verify additions:
  - lib tests: 216 passed.
  - `cargo test --test determinism`, debug and release, no features (as CI runs it): 10 passed each.
  - The focused mutant control (11 suites): 379 passed.
- `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`, `deny`, `lint-fuzz`, `msrv`, `validate`, `gate-audit` and `cost-audit` all pass.
- `semver-check`: 202 checks pass, "no semver update required".
- `decisions-audit --changed` is advisory and names DEC-011 for the fuzz edit. The edit is consistent with it: the fuzz crate stays separate.
- The public diff is exactly five items: the four `pub fn`s and `Error::InvalidRowRange`. `check_supported_bayer` is `pub(crate)`.
- `DEVELOP_VERSION` and every existing determinism constant are unchanged.

**CI (`gh pr checks 343`; run 45666936351 on `ea65f2dc`).** All checks are green. I read both `determinism` job logs:
- **arm64 leg:**
  - The job's own guard printed `uname -m: arm64`, `host: aarch64-apple-darwin` and image `macos-26-arm64`.
  - `cargo test --test determinism` and `--release` each ran `colour_plane_hashes_match_committed` and `colour_determinism_red_proof`, both ok, 9 passed.
- **x86_64 leg:** `uname -m: x86_64`, `host: x86_64-unknown-linux-gnu`, image `ubuntu-26.04`. The same two tests passed in debug and release.

So the arm64-recorded u16 and f32 hashes reproduce on x86_64.

**Determinism fixture.**
- The f32 plane holds a negative and an above-1.0 value; this is asserted in the test.
- The red-proof is real: one LSB moves both hashes, and the control must first reproduce the committed constants.
- The build's check that "removing the second illuminant moves the hash" does not prove that interpolation runs. A weight clamped to D65 would pass it too. I added `colour_fixture_interpolates_between_its_illuminants`. The plane differs from both controls whose two calibrations carry the same matrices, so the weight lies strictly inside (0, 1). It passes.

**Item 2: the refactor of the shipped colour entries.** Ruling: **sound; no shipped output moved.**
- I diffed the bodies by eye.
  - `store_u16` is the old u16 expression verbatim, in the same order: `(m_i0·r + m_i1·g) + m_i2·b`, then `quantize`.
  - `store_f32_quantised` is `f32::from(quantize(..)) / 65535.0`, as before. Only the `map` is new.
  - Both threaded entries now pass those same functions to `develop_color_bands`.
  - `develop_color_with` keeps the crop-before-matrix order and the in-`setup` matrix build, so the error order is unchanged.
- Evidence:
  - Every existing Tier-B pin and colour test passes.
  - Serial equals threaded: the `threaded` suite passes.
  - Every existing hash is unchanged.
- One consequence the handoff did not anticipate: because the plane and `develop_color_into` now share `store_u16`, AC1 can no longer see a reordered u16 sum. Both sides move together.
  - The extra mutant (x4) passed the whole 379-test focused set, the u16 hash and the AC13 digests. On real data `quantize` absorbs the one-ulp difference.
  - I added `color::tests::the_u16_sink_keeps_todays_operation_order`. The row `[1.1, 0.3, -0.1]·[476, 15845, 26416]` sums to just under 2635.5 in today's order and just over it in every non-equivalent order (swapping the first two terms is exact). It catches both x4 and a partial reorder (x4b).

**Item 3: the four build deviations.**
- **(a) AC4's slack:** accepted.
  - The argument is right. A u16 below 65535 means f < 1, and in [0.5, 1) half an f32 ulp is 2^-25, which is 65535/2^25 ≈ 0.00195 of a code. So the designed bound `0.5 + 1e-3` is below the cast's own rounding.
  - The new bound `0.5 + 65535/2^25 + 1e-9` is still tight enough for mutant (c): dividing by 65536 shifts f·65535 by u/65536 codes, which exceeds the slack for every u above about 32,900. AC4 went red under (c) here.
- **(b) AC2's check (iv):** accepted.
  - `demosaic_into` (`src/demosaic.rs:279`) builds its own `BayerPhase` and does not call `develop_color_fused`.
  - The fixture's levels are exactly the identity. At 16 bits with no BlackLevel and no WhiteLevel, `normalize_sample(v, 0, 65535)` is `(v·65535 + 32767) / 65535 = v`.
  - `to_srgb_linear_into` applies the same matrix and `quantize`, so the comparison is exact.
  - (f) fails (iv). It also fails six older colour-phase and threaded tests outside `linear_plane.rs` (for example `plane_relative_phase_tints_an_odd_origin_grey` and `threaded_demosaic_equals_single_threaded`), so (f) was only invisible to AC2's checks (i) to (iii), not to the suite.
- **(c) Mutant (k):** the test exercises the real sink, but it was too narrow. I added a broader guard.
  - `the_f32_sink_never_fuses_a_multiply_and_an_add` calls the real `store_f32_unclamped`, not a copy. Its cancellation sits only in channel 0's first term.
  - A `mul_add` in channel 1 (k2), channel 2 (k3), channel 0's last term (k4) or channel 0's second term (k5) passed it, and every hash.
  - I added `the_f32_sink_fuses_no_term_of_any_channel`. It places `0.1·3` at every term j of every channel i, with `-0.3` at each other term k: 18 cases. It catches k1 to k5 and the all-three mutant.
- **(d) `docs/determinism.md`:** accepted. The "measured" row names the whole-plane forms only, which is what AC12 hashes. The second row says plainly that the crop sink and `to_srgb_linear_into` are not hashed separately. I added two sentences naming the verify-added guards.

**Item 6: band memory.** Ruling: **no `width·height` buffer in the band path.** These are the allocations in one band call (`srgb_linear_plane_rows` into `develop_rows_banded` into `develop_color_fused`):

1. `sensor.clone()` in `develop_rows_banded`. It copies the `Sensor`'s vectors: the strip and tile tables (one entry per strip or tile), the three opcode-list byte strings, the matrices and `black_levels`. None of these scales with `width·height`, but the clone is paid on every band call.
2. `green`: `width × (band rows + halo)` u16. The halo is at most 4 rows above (3, rounded down to even) and 3 below, so at most `(rows + 7)·width·2` bytes.
3. The ring `cache` in `develop_color_fused`: `8·width` u16.

There is nothing else:
- `GainMaps` is a fixed array of borrowed views, documented as allocating nothing.
- `check_required_processing` allocates nothing.
- The levels and the matrix live on the stack.

The whole-plane forms keep their one `width·height` green plane, as Design §2 says.

**Item 7: no panics.**
- The range check `start >= end || end > height` runs before any arithmetic on the range. After it, `end − start` is saturating, `width·rows·3` is checked in u64, `u32 → usize` uses `try_from` with a saturating fallback, and `first` is a `saturating_mul`. A range near `u32::MAX` is refused as `InvalidRowRange` because `end > height`.
- The fuzz target as built reached `InvalidRowRange` and `Ok` for the band forms, but never `RgbBufferWrongLength`: every `dst` it passed was exactly sized.
- I changed it (`fuzz/fuzz_targets/demosaic.rs`) so a third input byte makes the band `dst` one short or one long on half the inputs. `lint-fuzz` passes.
- `cargo +nightly fuzz run demosaic`, 60 s: 2,288,052 runs, no crash.

**Item 8: docs and API.** The four doc comments state:
- the geometry, including the masked border;
- the unclamped f32 rule, and that it is not `develop_color_f32_into`'s;
- the refusal order, with `InvalidRowRange` after the geometry and before `RgbBufferWrongLength`;
- the bit-for-bit band equality.

The CHANGELOG `[Unreleased]` entry and DEC-051's sentence (`CallerError` 19, enum 79) are correct. `check_plane_buffers` refuses in `check_color_buffers`' order, because `CropScatter::new` adds only `resolve_geometry`.

**AC10: mutants, re-run here.**
- Method: each mutant was applied by an exact-string patch, run with `cargo test --all-features --profile corpus --no-fail-fast` over 11 suites (`--lib`, `linear_plane`, `determinism`, `error_kind`, the four `Entry` tables, `threaded`, `color_oracle`, `colour_sdk`, `develop_oracle`) with the corpus set, then restored with `git checkout` and `touch`.
- The control was green, 379 passed.
- The table lists the failing tests by name. The named catcher is in bold.

| Mutant | Applied as | Failed |
|---|---|---|
| (a) sink still crops | `CropScatter::new(sensor)?` for `WholePlane` in the whole-plane body | **`ac2_every_pixel_of_the_plane_is_written`**, `ac1_…`, `ac4_…`, both `ac11_…`, `ac13_…` |
| (b) f32 quantises first | `store_f32_quantised` at both f32 entry points | **`ac3_the_f32_plane_keeps_negatives_and_overs`**, **`ac5_the_f32_plane_is_the_unquantised_sum`**, `colour_plane_hashes_match_committed`, `colour_determinism_red_proof`, `ac13_…` |
| (c) divide by 65536 | three divisors in `store_f32_unclamped` | **`ac5_…`**, `ac4_…`, both colour determinism tests, `ac13_…`, both FMA guards |
| (d) GainMaps skipped | `GainMaps::none` in both plane bodies | **`ac7_gain_maps_apply_to_the_plane`**, **`ac1_…`**, `ac13_…`, two gain-map refusal rows (`malformed_gain_map_…`, `seventeen_gain_maps_…`) |
| (e) levels skipped | whole-plane body normalises with the raw mosaic | **`ac1_the_plane_cropped_equals_the_developed_image`**, `ac3_…`, `ac7_…`, `ac11_strips_tile_on_held_files`, `ac13_…` |
| (f) CFA phase from plane origin | `(0, 0)` for `active_area_origin` in `develop_color_fused` | **`ac2_…`** (check iv), plus 6 existing colour-phase and threaded tests (e.g. `plane_relative_phase_tints_an_odd_origin_grey`, `grey_stays_grey_across_origin_parities_and_bayer_phases`) |
| (g) dst written before a late check | band body develops with an identity fallback, then builds the matrix with `?` (correct output on success) | **`ac6_buffer_and_input_refusals_leave_dst_untouched`**, `ac6_the_k3iii_exception_and_the_already_refused_tags_hold` |
| (h) band drops its halo | `BAND_HALO` 3 to 0 | **both `ac11_…`**, plus the threaded band tests |
| (i) band top not even | `& !1` removed | **`ac11_strips_tile_to_the_whole_plane`**, plus the threaded band tests |
| (j) dst indexed from row 0 | `first = 0` | **both `ac11_…`** |
| (k) all three sums fused | `mul_add` on the first term of each channel | **`the_f32_sink_never_fuses_a_multiply_and_an_add`**, `the_f32_sink_fuses_no_term_of_any_channel` |
| k2 / k3: FMA in channel 1 / channel 2 only | one channel's first term | **`the_f32_sink_fuses_no_term_of_any_channel`** only; the build's guard and all hashes passed |
| k4 / k5: FMA on channel 0's last / second term | `m02.mul_add(bf, ..)` / `m01.mul_add(gf, m00·rf)` | **`the_f32_sink_fuses_no_term_of_any_channel`** only |
| x1: band skips `end > height` | the check keeps only `start >= end` | **`ac6_…`** (no panic) |
| x2: `InvalidRowRange` after `RgbBufferWrongLength` | range check moved after the dst check | **`ac6_…`** |
| x3: f32 whole plane clamps negatives | `max(0.0)` after `store_f32_unclamped`, whole plane only | **`ac3_…`**, **`ac5_…`**, both `ac11_…` (band ≠ whole), both colour determinism tests, `ac13_…` |
| x4: u16 order swapped (`m_i2·b` first) | all three channels of `store_u16` | **none** before verify (379 passed: AC1 is blind, since both sides share `store_u16`); after verify **`the_u16_sink_keeps_todays_operation_order`** |
| x4b: u16 last two terms swapped | channel 1 only | **`the_u16_sink_keeps_todays_operation_order`** |
| x5: `InvalidRowRange` classed `Corrupt` | moved to the `Corrupt` arm | **`every_variant_classified`** |

### Cost self-report

- **Tokens (total):** 19,052,713 (floor, measured before the final commit)
- **Estimated 12.62:** 12.62
- **Duration (minutes):** about 25, by the transcript's timestamps (17:47 UTC on)
- **Source of the number:** this session's transcript (`f4520a5d-7d1a-4e73-a98f-760835aa1a00.jsonl`). It is deduplicated by `message.id`, taking the largest `output_tokens` per message, per docs/cost-tracking.md. It is priced at Opus 5.5's $5 / $25 per million, with cache reads at 0.1x and 1-hour cache writes at 2x. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by verify. The build's four deviations are ruled acceptable above. Verify's additions are test-only:
  - `src/color.rs` test module: `the_f32_sink_fuses_no_term_of_any_channel` and `the_u16_sink_keeps_todays_operation_order`;
  - `tests/determinism.rs`: `colour_fixture_interpolates_between_its_illuminants`;
  - `fuzz/fuzz_targets/demosaic.rs`: band `dst` one short or one long;
  - `docs/determinism.md`: two sentences.
- **Punch list (`src/`):** none.
- **Follow-up work identified:**
  - The `demosaic` fuzz target has no CI smoke job and no `just` recipe; CI smokes `dng_write`, `ifd`, `ljpeg`, `opcode`, `plane` and `warp_opcode`. The four new entries are fuzzed only when someone runs it by hand. This is an [XS] chore: a `just fuzz-demosaic` recipe plus a CI smoke job.
  - For SPEC-092, along with the build's note about re-resolving levels, GainMaps and the matrix per strip: each band call also clones the `Sensor`, opcode-list bytes included. A reusable plan would remove both costs.
  - For SPEC-092's design: the fixed-point mixer row will have the same two blind spots verify hit here. A shared sink makes the "equals the old path" oracle move with the mutant, and a reordering or FMA is absorbed by the output's resolution. Pre-register an injected cancellation fixture per channel and per term.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The handoff's x4 expected AC1 to catch a reordered u16 sum. After the shared-sink refactor (which the spec's own Notes asked for), AC1 compares two outputs of the same function, so it cannot catch it. Spotting that took reading the refactor first.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-011 (the separate fuzz crate), because the spec edits a fuzz target. It was advisory and easy.

3. **If you did this task again, what would you do differently?**
   — Write each mutant as an exact-string patch with an asserted match count from the start; a regex over the whole file would also have mutated the test module's copies of the same expressions. And for every "no X" rule, enumerate where X could appear (every channel, every term) before trusting a single-site guard.
