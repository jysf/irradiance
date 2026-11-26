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
  id: HANDOFF-203
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-26
  status: completed                  # pending | accepted | completed | rejected

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
  status: completed
  agent: claude-opus-5-5
                                   #   to_agent above is a prediction from tier_map
  verdict: approved
  basis: transcript
  tokens_total: 12293376
  estimated_usd: 8.75
  duration_minutes: 80
  branch: claude/spec-092-verify
  pr: https://github.com/jysf/irradiance/pull/354
  completed_at: 2026-11-26
  notes: "FLOOR taken before the handback commit, from transcript 29c227de-0fb7-46f6-8420-8dd83ce30639.jsonl (69 unique message.ids, all claude-opus-5-5, no subagents): input 145 + cache-creation 171,754 (1-hour) + cache-read 12,081,934 + output 39,543, priced at 5 and 25 USD per million."
  synced_at: 2026-11-26
---

# HANDOFF-203: VERIFY: the mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip, the plane's black and white levels (SPEC-092)

## Delegation Summary

You are the **verifier** for SPEC-092. Review it cold: you did not build it.

- **Under review:** PR 354 (`feat/spec-092-mono-mixer`). The code is commit `b822fa2`; CI
  run 45855649150 on it was green, including both `determinism` legs. The later build
  commit `0efa11a` touches the handoff, spec and timeline only. The build ran under
  HANDOFF-202 in a separate session.
- **Your branch:** `claude/spec-092-verify`, cut from that head, plus this handoff and the
  verify prompt. The build already advanced the spec to `verify`.

SPEC-092 adds, in `src/color.rs`, `mono_mix_row`, `mono_mix_plane_into` and
`mono_mix_plane_rows_into`, six public constants (`MONO_MIX_ROW_FRACTION_BITS`,
`MONO_MIX_FADE_START`, `MONO_MIX_FADE_WIDTH_BITS`, `MONO_MIX_BLACK`, `MONO_MIX_WHITE`,
`MONO_MIX_VERSION`) and three `Error` variants. It generalises SPEC-091's identity sink so
the mono plane reuses the one fused traversal; no existing output may change.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md`.
  Judge the ACs against the spec **as designed**: `git show origin/main:<spec path>` (after
  PR 353). That is Design §1 to §12, AC1 to AC15, mutants (a) to (u), and the Failing Tests.
- **Decisions:** DEC-083 (accepted: row format, rounding, threshold, kernel, overflow),
  DEC-079 (the fold and the fade), DEC-033 (f64 basic operations only), DEC-018, DEC-016,
  DEC-040, DEC-044, DEC-051, DEC-053, DEC-078, DEC-069, DEC-070, DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `provenance-recorded-per-algorithm`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-202's `## Handback` (mutant table, deviations, digests).
- **Prior verify:** HANDOFF-198 (SPEC-091) and its FU-10 lesson: a shared code path makes
  "equals the old path" oracles move with the mutant.
- **Toolchain:** run `just test`, never `just test --all-features`; set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; use
  `/usr/bin/git` for branch and HEAD; touch a file after restoring it between mutants.

## What to check hard

1. **Every AC (1 to 15) against the spec as designed.** AC5's 15 golden rows must equal the
   spec's table as written. `MONO_MIX_VERSION == "1"` asserted beside AC5 and AC12, and
   documented like `DEVELOP_VERSION` (a mix is fixed by both versions, Design §12). The
   levels documented as part of the contract.
2. **The shared-sink generalisation.** It touches SPEC-091's shipped path. Confirm no
   existing output or determinism hash moved, and that the "equals the old path" oracles do
   not move with a mutant.
3. **The build's four deviations.** Rule on each: tests code-first (red by compile failure
   plus mutants); the AC8 point for mutant (i); the second order fixture
   `[2^-53, 2^-53, 1]`; the AC6 grid at about 135 s in the debug CI job.
4. **Mutants.** Re-run at least (e), (h), (i), (j), (k), one (l), one (m) and (r). Add:
   the fade decided on `v_i` sign versus `q_i` sign; the shift replaced by a truncating
   division; `u` computed from a white-balanced channel; band `dst` length off by one row;
   `MixRowOutOfRange` checked before the file checks. Check the argued equivalents (t), (u).
5. **The integer kernel** against DEC-083's overflow argument and AC14's extremes.
6. **Determinism.** Read the CI logs: both legs run the mixer hashes, and the fade engages
   on the fixture (8 of 384 pixels).
7. **Allocations and panics.** List the band path's allocations; no panics; the fuzz target
   reaches all three functions.
8. **Docs and API.** The threshold rustdoc (Design §5), CHANGELOG under Unreleased,
   `just semver-check` additions only.
9. **Gates.** `just test` with the corpus, `just validate`, `just cost-audit`, fmt, clippy,
   doc-check.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- Evidence for each item above, including the mutant table and the deviation rulings.
- Fixes on the verify branch are tests or docs only; `src/` changes go in a punch list.
- If approved: `just advance-cycle SPEC-092 ship --verdict approved`, `just handback-sync
  SPEC-092`, push, and fast-forward `feat/spec-092-mono-mixer` only if it is a
  fast-forward. Do not merge PR 354; do not start the ship.

## Out of Scope

- The developed mix (SPEC-093); presets; negative weights; a clip mask; a fade parameter.
- A reusable band plan (FU-9 closed as measured, Design §7); threaded variants.
- Any change to `DEVELOP_VERSION` or an existing hash. If one seems necessary, that is a
  rejection.

## Return Criteria — how to hand back

Fill the `handback:` front matter (with a real `tokens_total`) and the `## Handback`
section, including the reflection; set `handoff.status` and `handback.status` to
`completed`; return the verdict.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-092-verify`, carried onto `feat/spec-092-mono-mixer`, https://github.com/jysf/irradiance/pull/354 (not merged).
- **Completed at:** 2026-11-26
- **All acceptance criteria met?** yes. The code under review, `b822fa2`, is unchanged. The verify adds three tests in commit `993cda4`; each closes a mutant that survived the build's oracles.
- **For `verify`:** ✅ APPROVED at `b822fa2` (code). The verify tip carries test and docs commits only.

**ACs against the spec as designed** (`origin/main` after PR 353):

- **AC5.** The 15 golden rows in `tests/mono_mix.rs` were compared by script with the spec's table, and all 15 equal it. `MONO_MIX_VERSION == "1"` is asserted beside them, and in `mono_mix_hashes_match_committed` beside AC12's hashes, each with the "bump in the same change" comment.
  - Its doc comment matches Design §12: a history list, what bumps it, what does not, and that a mix is fixed by the pair (`DEVELOP_VERSION`, `MONO_MIX_VERSION`), with callers recording both.
  - `MONO_MIX_BLACK` and `MONO_MIX_WHITE` are documented as part of the contract, and changing them is an API change that also bumps the version. A doc test pins them.
- **AC1 to AC4, AC6 to AC8, AC10, AC11, AC14.** Each named Failing Test exists and passes in my run. Each is red under at least one of the re-run or new mutants below.
  - The kernel (`mono_mix_pixel`) decides sign on `q_i`, takes `m` over all three channels, caps `u` at 8192, and clamps before the cast.
  - Refusal order matches Design §6: weights first, then required processing, buffers, levels, GainMaps, CFA, calibration, and the row last.
- **AC9.** The fuzz target reaches all three functions with valid and invalid weights, and `dst` is one short or one long on half the inputs. My run: 5,415,719 runs in 61 s, no crash. `just lint-fuzz` is green.
- **AC12.**
  - CI run 45857000180 on the head `0efa11a` (code `b822fa2`): both determinism jobs green, x86_64 and arm64 (`uname -m` asserted in each).
  - Both legs ran `cargo test --test determinism` and `--release`, and each printed `mono_mix_hashes_match_committed ... ok` and `colour_determinism_red_proof ... ok`.
  - Locally, the fixture's fade changes **8 of 384 pixels**, the same as the build reported.
- **AC13.**
  - `just test` with the corpus: **960 passed, 0 failed** across 57 test binaries. The only skips were the three SDK or dng_validate cross-checks, which have no environment set.
  - No line of an existing determinism constant was removed or changed (diff checked), and `DEVELOP_VERSION` is still `"8"`.
  - `tests/error_kind.rs` gains exactly three rows.
  - `just semver-check`: no semver update required. The public diff is the three functions, six constants and three variants.

**The shared-sink generalisation.**
- The RGB entries now pass `PlaneSamples::Rgb` and the matrix as their `build`. The store bodies (`store_u16`, `store_f32_*`) are untouched, and SPEC-091's guards (`the_u16_sink_keeps_todays_operation_order`, `the_f32_sink_fuses_no_term_of_any_channel`) still test the real sinks.
- The committed SPEC-091 hashes and Tier-B digests are constants, so they cannot move with a mutant. V9 (RGB samples per pixel shrunk to 1) and the band mutants turned SPEC-091's `linear_plane` tests red together with the mono ones.
- The mono "equals the old path" oracles are independent of the shared path: AC1 and AC3 use `demosaic_into` and an `i128` oracle in the test, and AC2's band vs whole plane runs through two different traversals.

**The integer kernel.** It matches DEC-083's overflow argument: `|P·2^13|` and `|N·u|` are below `2^61.6` and have opposite signs. The saturating forms never engage, and AC14's three extreme rows equal the `i128` oracle.

**Allocations on the band path:**
- one `Sensor` clone, opcode bytes included (FU-9, closed as measured);
- one green plane of `width × (band + halo)` rows, in `develop_rows_banded`.

The mixer adds none: the row is `[i32; 3]` and the weights `[f64; 3]`. No `width·height` buffer is on the band path. The whole-plane entry keeps its one `width·height` green plane.

**Mutants.** My own harness (not the build's): each mutant applied by string replacement. Each run covers the lib `color::tests`, plus `mono_mix`, `determinism`, `linear_plane` and the four Entry tables, with the corpus; the file is restored after each.

| Mutant | Applied as | Result, failing tests |
|---|---|---|
| (e) fade removed | `N·8192` for `N·u` | RED: `ac6_…`, `ac7_…`, `ac8_…`, `ac1_…`, random-row oracle, `ac5_…`, both AC12 tests |
| (h) fade scales positives | `P·u` | RED: `ac3_…` (unit and plane), `ac1_…`, `ac4_…`, `ac5_…`, `ac6_…`, `ac7_…`, `ac14_…`, … |
| (i) `u` uncapped | `.min(8192)` removed | RED: `ac8_…`, `ac1_…`, `ac4_…`, `ac5_…`, random-row oracle, both AC12 tests |
| (j) threshold off by one | `65536 − m` | RED: `ac8_…`, `ac6_…`, `ac1_…`, `ac5_…`, random-row oracle, both AC12 tests |
| (k) upper clamp dropped | `.max(0) as u16` | RED: `ac14_…`, `ac3_…`, `ac1_…`, `ac4_…`, `ac5_…`, `ac6_…`, `ac7_…`, … |
| (l) channel 1, site 2 fused | `w2.mul_add(c1, …)` | RED: `ac11_the_row_fuses_no_term_of_any_channel` only |
| (m) channel 2, `(t1 + t2) + t0` | reordered | RED: `ac11_the_row_keeps_its_summation_order` only |
| (r) halo 0 / odd top / `dst` from row 0 | three runs | RED, all three: `ac2_every_pixel_written_and_strips_tile` and SPEC-091's `ac11_strips_tile_to_the_whole_plane`; Tier B on halo and row 0 |
| V1 (u): sign test `q > 0` (what a `v_i` test reduces to) | `qi >= 0` → `qi > 0` | SURVIVED: equivalent, as argued. A zero `q_i` adds nothing to either sum |
| V2 (t): shift → truncating division | `sum / 2^37` | SURVIVED: equivalent, as argued. They differ only for a negative numerator, which clamps to 0 either way |
| V3: `m` from white-balanced channels | R and B doubled before the max | RED: `ac8_…`, `ac1_…`, `ac4_…`, `ac5_…`, random-row oracle, both AC12 tests |
| V4: band `dst` one row long | `band_rows + 1` | RED: `ac10_…`, `ac2_…`, the four Entry tables, SPEC-091's band tests |
| **V5: `MixRowOutOfRange` before the file checks (whole plane)** | row built before `srgb_linear_plane` | **SURVIVED the build's suite.** RED on the new `the_row_range_refusal_comes_after_every_file_check` |
| **V5b: the same in the band form** | the same before `srgb_linear_plane_rows` | **SURVIVED.** RED on the new test |
| **V6: `MONO_MIX_FADE_START = 57000`** | the public constant only | **SURVIVED.** RED on the new `the_public_mixer_constants_are_the_kernels` |
| **V7: `MONO_MIX_ROW_FRACTION_BITS = 23`** | the public constant only | **SURVIVED.** RED on the new test |
| **V8: `MONO_MIX_FADE_WIDTH_BITS = 12`** | the public constant only | **SURVIVED.** RED on the new test |
| V9: RGB samples per pixel 1 | `PlaneSamples::Rgb => 1` | RED: the SPEC-091 plane tests and digests, the Entry tables |
| **V10: the kernel rounds half down** | `MIX_HALF − 1` | **RED only on `ac5_…`'s local corpus digests**, so CI could not see it. Now also RED on the new `the_kernel_rounds_an_exact_tie_up` |

**Verify additions** (tests only, `993cda4`):
- `color::tests::the_public_mixer_constants_are_the_kernels`. The three public constants were documentation only: the kernel reads private `MIX_*` constants and no test referenced the public ones. This test pins each public constant to its designed value and to the private one, and checks the boundary `MONO_MIX_FADE_START` names.
- `color::tests::the_kernel_rounds_an_exact_tie_up`. Exact ties, plain and faded, put the kernel's round half up in the CI suite.
- `mono_mix::the_row_range_refusal_comes_after_every_file_check`. The tiny-neutral file with a short `dst`, a short `src`, an unsupported opcode, no CFA, and a reversed band must refuse with the earlier error, leaving `dst` untouched.

None is ship-blocking: the code was right in each case, and the gap was in the oracles.

**Rulings on the build's four deviations:**
1. **Tests written code-first. Accepted as recorded.** It departs from `test-before-implementation`. The red evidence is real: the tests fail to compile against main, and the pre-registered mutants and my sample re-run all go red. It is not a ship blocker.
2. **The AC8 point for (i), `[0, 1000, 500]` giving 32000. Accepted.** It is within AC8's intent, and it fixes a design-time blind spot. Re-run: (i) is red on `ac8_…`.
3. **The second order fixture `[2^-53, 2^-53, 1]`. Accepted, and required.** Design §8 says the terms are chosen so each wrong pairing differs, and its worked example covers only one pairing. The fixture's own `assert_ne!` proves it distinguishes, and (m) on channel 2 is red.
4. **AC6 at about 135 s in the debug CI job. Accepted.** The grid is as specified, and CI's `rust / test` job finished green in 7 m 40 s. The build's [XS] idea to start each ray at the last `k` below the threshold stays an optional follow-up.

**Docs.**
- `MONO_MIX_FADE_START`'s rustdoc has Design §5's three parts word for word in substance: the compared value, the threshold, and how it relates to the raw photosite and to GainMaps.
- The module doc has a "Mono mix" section.
- CHANGELOG `[Unreleased]` lists the functions, constants and variants.
- The provenance-ledger row is class 1, our own decisions. `docs/determinism.md` adds a mixer section and changes no other row.

**Gates (local):** all green.
- `just test` with the corpus
- `just fmt-check`
- `just lint-ci`, clippy 0.1.98, after one reversed-range literal in my own new test was reworded
- `just doc-check`
- `just semver-check`
- `just lint-fuzz`
- `just validate`
- `just cost-audit`
- `just decisions-audit --changed`, advisory: DEC-079 and DEC-083 re-read, and the code is consistent with both

### Cost self-report

- **Tokens (total):** 12,293,376 (floor, measured before the handback commit)
- **Estimated USD:** 8.75
- **Duration (minutes):** about 80
- **Source of the number:** this session's transcript (`29c227de-0fb7-46f6-8420-8dd83ce30639.jsonl`, found by the scratchpad UUID), deduplicated by `message.id` with the largest `output_tokens` per message, per docs/cost-tracking.md. Priced at claude-opus-5-5's 5 and 25 USD per million, cache reads at 0.1x and 1-hour cache writes at 2x. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the code. The verify added the three tests above, which are not in the Failing Tests list.
- **Follow-up work identified:**
  - [XS] The `demosaic` fuzz target still has no CI smoke job or `just` recipe (SPEC-091's follow-up, still open). It now reaches six public entries.
  - [XS, optional] Shorten `ac6_…` in the debug profile (the build's idea).
  - Design lesson for SPEC-093: a public constant that describes a private computation needs a test that ties the two, or it is documentation only. Pre-register a constants-equal-kernel test with every constant that is part of a contract.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. Design §6 states the order "last `MixRowOutOfRange`", but no AC fixture combines it with another fault, so that clause had no oracle until this verify.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The rule-clause map's "§2 levels are constants: doc test" covered only two of the five public constants. The other three had no row in the map at all.

3. **If you did this task again, what would you do differently?**
   — Mutate the public constants and the order of every "last" refusal first. Those were the three survivors. The kernel and row mutants the design pre-registered were all well covered.
