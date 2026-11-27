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
  id: HANDOFF-205
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-26
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-093

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
  status: completed  # completed | blocked | rejected
  agent: claude-opus-5-5  # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved  # verify handoffs: approved | punch-list | rejected
  basis: transcript  # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 21924077  # REAL combined count — what cost-audit reads
  estimated_usd: 17.05  # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 240
  branch: claude/spec-093-verify
  pr: https://github.com/jysf/irradiance/pull/358
  completed_at: 2026-11-27  # YYYY-MM-DD
  notes: "Floor, measured live from transcript ef47f062-3d10-4ec0-8ce2-86a58d193f1b.jsonl, deduplicated by message.id; re-sum at ship. Wall time includes an app restart that killed the mutant run part way (resumed). USD at Opus 5.5 rates, 1-hour cache writes."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-27
---

# HANDOFF-205: VERIFY: the developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling (SPEC-093)

## Delegation Summary

You are the **verifier** for SPEC-093. Review it cold: you did not build it.

- **Under review:** PR 358 (`feat/spec-093-developed-mix`). The code is commit `6b29c48`;
  CI run 45898355595 on it was green (25 checks), including both `determinism` legs. The
  later build commit `db31cce` touches the handoff, spec and timeline only. The build ran
  under HANDOFF-204 in a separate session.
- **Your branch:** `claude/spec-093-verify`, cut from that head, plus this handoff and the
  verify prompt. The build already advanced the spec to `verify`.

SPEC-093 adds `DevelopOptions::mono_mix: Option<[Rational; 3]>` and
`opcode::DevelopPath::MonoMix`. The developed mix is the mono develop's geometry (warp
lerped to `warp_alpha`, framing, orientation) run over SPEC-092's mixed plane, mixed
before geometry in 512-row strips into caller scratch. `black_levels` is not read under
the mix. No version constant and no `Error` variant changes; no existing output may move.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md`.
  Judge the ACs against the spec **as designed**: `git show origin/main:<spec path>` (after
  PR 356), Design §1 to §11, AC1 to AC14, mutants (a) to (z), and "onyx's reply
  (2026-11-26)".
- **Decisions:** DEC-084 (accepted: the rule, the opcode set, the strips, the scratch, the
  order, the versions), DEC-083 (the row and the kernel), DEC-079, DEC-058, DEC-059,
  DEC-060, DEC-043, DEC-044, DEC-049, DEC-068, DEC-070, DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `provenance-recorded-per-algorithm`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-204's `## Handback` (mutant table, deviations, AC numbers).
- **Prior verifies:** HANDOFF-198 (SPEC-091, FU-10: a shared path makes "equals the old
  path" oracles move with the mutant) and HANDOFF-203 (SPEC-092, FU-5 and FU-6: public
  constants not tied to the kernel, an untested refusal order).
- **Toolchain:** run `just test`, never `just test --all-features`; set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; use
  `/usr/bin/git` for branch and HEAD.

## What to check hard

1. **Every AC.** AC4's digests and `scratch_len` values equal the registered table
   (compare by script). AC5 (`black_levels` ignored, documented). `mono_mix` composes with
   `framing`, `apply_orientation` and `warp_alpha` at 0, ½ and 1 (onyx's parity case).
   With the option absent, every existing output is byte-identical.
2. **Shared code.** `develop_core`, the demosaic traversal refactor (buffers from
   scratch), `RING` at module level. No existing hash or Tier-B pin moved; the oracle
   cannot move together with a mutant in the shared path.
3. **The build's deviations.** F10 removes both colour matrices; AC11's red-proof uses
   `2^-26` on `kr0`, not one ulp; fuzz option bit 6; `RING` module-level; only
   `tests/developed_mix.rs` seen failing first.
4. **FU-5 and FU-6.** Every public constant and default tied by a test; every ranked
   refusal pair covered on all four entries; the skipped pairs (F7, F8) and (F8, F13)
   truly impossible.
5. **Mutants.** A sample of (a)–(x), plus: mix after geometry, strip size, `black_levels`
   read, version bumps, an `OpcodeList1` refusal skipped, the scratch formula off by one
   strip, the optional warp skipped. Check the argued equivalents (u), (y), (z).
6. **Allocations.** Zero in `develop_with_scratch_into`, exactly one in
   `develop_with_into`.
7. **Determinism.** Both CI legs ran the four developed-mix hashes.
8. **No panics, fuzz.** The `demosaic` and `develop` targets.
9. **Docs and API.** Rustdoc on `mono_mix`, the caching note, CHANGELOG under Unreleased,
   `just semver-check` additions only.
10. **Gates.** `just test` with the corpus, validate, cost-audit, fmt, clippy, doc-check.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- Evidence for each item above, including the mutant table and the deviation rulings.
- Fixes on the verify branch are tests or docs only; `src/` changes go in a punch list.
- If approved: `just advance-cycle SPEC-093 ship --verdict approved`, `just handback-sync
  SPEC-093`, push, and fast-forward `feat/spec-093-developed-mix` only if it is a
  fast-forward. Do not merge PR 358; do not start the ship.

## Out of Scope

- STAGE-034's lifting of the colour paths' refusal of mandatory `FixBadPixelsConstant`
  and `WarpRectilinear` on Leica files.
- Threaded variants, a public strip size, an `irr` hook, any new `Error` variant.
- Any change to `DEVELOP_VERSION`, `MONO_MIX_VERSION` or an existing hash. If one seems
  necessary, that is a rejection.

## Return Criteria — how to hand back

Fill the `handback:` front matter (with a real `tokens_total`) and the `## Handback`
section, including the reflection; set `handoff.status` and `handback.status` to
`completed`; return the verdict.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-093-verify`, fast-forwarded onto `feat/spec-093-developed-mix` (https://github.com/jysf/irradiance/pull/358). Not merged.
- **Completed at:** 2026-11-27
- **All acceptance criteria met?** Yes. AC1 to AC14 hold against the spec as designed (`origin/main` after PR 356, onyx's reply included).
- **For `verify`:** ✅ **APPROVED** at code commit `6b29c48` (no `src/` change since). The verify adds two tests and the verify docs only.

### Results

- **AC4, compared by script** (`ac4cmp.py` parsed the registered table out of `origin/main`'s spec and the test's printed digests and sizes):
  - 19 digests and 19 output sizes compared, 0 mismatches;
  - the Pixel 3a under `FullActiveArea` is refused with `scale_max = 1.0015292540192604`;
  - every `scratch_len` equals §7's numbers, and the 3a and 7 Pro equal the mono develop's `scratch_len` for `mono_of`.
  - `DEVELOP_VERSION == "8"` and `MONO_MIX_VERSION == "1"` are asserted beside the table.
- **AC5:** `black_levels` is ignored under the mix, and the fixture can tell (as mono, `FirstValue` and `PerPosition` differ). It is documented on the field, in the module docs and in CHANGELOG. V3, a mutant that reads it, is red.
- **onyx's parity case.** AC1 crossed α 0 and ½ with `apply_orientation` true only. I added `ac1_parity_grid_composes_every_option`:
  - fixtures (ii) and (iv) × Orientation 1, 6 and 7 × both framings × `apply_orientation` true and false × α 0, ½ and 1;
  - every case equals the oracle, and α ½ differs from both ends.
- **Option absent:** AC2 holds, and every existing determinism hash and Tier-B pin passes unchanged (`just test` with the corpus).
- **Gates.**
  - `just test` with the corpus: 988 passed, 0 failed, 1 ignored. That is the build's 986 plus my two.
  - `fmt-check`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `doc-check`, `validate` and `cost-audit` are green.
  - `semver-check`: no semver update required. `DevelopOptions` and `DevelopPath` are both `#[non_exhaustive]`, so the field and the variant are additive.
  - `decisions-audit --changed` flags DEC-079, DEC-083 and DEC-084. I re-read DEC-084 items 1 to 8 against the code, and it is consistent.
- **Allocations (AC10):** `develop_with_scratch_into` makes 0 allocations under the mix (12 success cases, F0 to F13, the Pixel 7 Pro). `develop_with_into` makes exactly one, of `2·scratch_len` bytes. Both tests are live in the run above.
- **Determinism (AC11).** CI run 45898355595 on `6b29c48`: `rust / determinism` on ubuntu-latest x86_64 and on macos-latest arm64 each ran `developed_mix_hashes_match_committed` and `colour_determinism_red_proof`, in debug and in release. The four hashes are one array assert.
- **Fuzz (AC12):** 61 s each, no crash:
  - `demosaic`: 5,045,743 runs;
  - `develop`: 4,844,843 runs.
  - Use `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run`, per the toolchain brief. A bare `cargo +nightly` exits 101.

### Shared code (the SPEC-091 FU-10 check)

The oracle shares three pieces with the implementation:
- `develop_color_fused_in`: the whole plane for the oracle, bands for the implementation;
- `apply_warp_into`;
- `crop_and_orient_from_active_into`.

I mutated each (V8, V9, V10). Each time the AC1 equality tests stayed green, because the oracle moved with the mutant, exactly as FU-10 predicts. A pin computed independently caught every one:
- **V8, the ring cell value in the shared traversal:** 23 tests, including `colour_plane_hashes_match_committed`, `mono_mix_hashes_match_committed` and `developed_mix_hashes_match_committed`, which run on both CI legs, and the Tier-B goldens.
- **V9, the warp sampler's rounding:** `developed_mix_hashes_match_committed` (CI; docs/determinism.md notes it is the only cross-arch hash of the warp) and `colour_determinism_red_proof`, plus 9 Tier-B warp pins.
- **V10, the crop origin dropped from the shared gather:** the Tier-A tests `develop_into_crops_from_the_warped_active_area_not_the_warped_crop` and `passing_warps_develop_under_full_active_area`, and the Tier-B pins.

The demosaic refactor reads the band's local `height` where the old code cloned the `Sensor`. `RING` keeps its value (8). Mutant (x), the full height passed to a band, is red. V12 fills the strip scratch with garbage before every strip, and it survives: the traversal never reads stale scratch.

### FU-5 and FU-6

- **FU-5.** Each constant this spec adds or relies on goes red when changed alone:
  - `mono_mix` default: (w), and AC9(a).
  - `MONO_MIX_WHITE`: (v) is red on `developed_mix_contract_constants_are_the_computation` itself, not only on SPEC-092's tests.
  - `MONO_MIX_VERSION`, `DEVELOP_VERSION`: V4a, V4b.
  - The strip height 512: V2a.
  - `T`'s halo and ring: (q1), V6a, V6b.
  - `RING`: named in AC9(d).
  - `DevelopPath::MonoMix`'s table: V5, (i), (j).
- **FU-6.** Every ranked pair is checked on both develop entries: 76 on `develop_with_into`, 89 on the scratch form. The sizing entries' refusals and their pairs are covered too. My extra order mutant V13 (the CFA checked after the calibration and the row) is red on `ac8_every_ranked_pair_on_both_develop_entries`.
  - **(F7, F8) is truly impossible.** F8 is the `StandardCrop`-only parse (rank 8). Under `FullActiveArea` the same malformed block is rank 7's own parse error, so the two need opposite framings.
  - **(F8, F13) was not impossible.** The build's reason, "a malformed block has no tangential term", is false. `parse_warp_rectilinear_params` refuses a block with a NaN optical centre, or with one byte past its exact `4 + 48·N + 16` length, and that same block can carry `kt0 = 0.01`. I added `ac8_one_warp_block_both_malformed_and_tangential_is_f8`: both forms, both develop entries and `scratch_len` return F8's error, with `dst` and scratch untouched. It passes on the build's code, so nothing in `src/` was wrong; only the oracle had the gap.

### Mutants

All runs used the harness `vmutants.py` (scratchpad) with the corpus set. It applies each mutant, runs the lib tests and the `developed_mix`, `determinism`, `develop_scratch_alloc`, `required_processing`, `mono_mix` and `develop` suites, and runs the whole suite for V8 to V10. It then restores `src/`.

The build's (a)–(x): I re-ran 15, and all 15 are red.

| Mutant | Verdict | Failing tests (count; examples) |
|---|---|---|
| (a) | RED | 3: `ac6_the_mix_applies_a_mandatory_warp`, `ac1_parity_grid_composes_every_option`, `ac1_the_developed_mix_…` |
| (c) | RED | 3: `ac1_the_developed_mix_…`, `ac1_parity_grid_…`, `ac4_held_files_…` |
| (e1) | RED | 13 |
| (f2) | RED | 10 |
| (g) | RED | 12 |
| (h) | RED | 3: `ac6_gain_maps_apply_once`, … |
| (i) | RED | 6: `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels`, `ac8_*`, … |
| (k2) | RED | 2: `ac6_the_mix_never_parses_opcode_list_1`, `ac8_the_sizing_entries_…` |
| (l) | RED | 8 |
| (o) | RED | 1: `ac8_every_ranked_pair_on_both_develop_entries` |
| (p1) | RED | 2: `ac8_each_fault_alone`, `ac8_every_ranked_pair_…` |
| (q1) | RED | 7: `develop::tests::ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac3_…`, … |
| (s) | RED | 3: `ac9_scratch_len_…`, `ac8_the_sizing_entries_…`, `ac4_…` |
| (v) | RED | 22, incl. `developed_mix_contract_constants_are_the_computation` |
| (x) | RED | 10 |

My own mutants:

| Mutant | Verdict | Failing tests |
|---|---|---|
| V1, mix after geometry (R, G, B warped one by one, then mixed) | RED | 8: `ac6_the_mix_applies_a_mandatory_warp`, `ac1_parity_grid_…`, `ac1_the_developed_mix_…`, `ac4_…`, `developed_mix_hashes_match_committed`, … |
| V2a, strip height 256 in the core and `T` together (= (z)) | RED | 3, literals only: `ac9_scratch_len_…`, `ac3_…`, `ac4_…` (scratch numbers) |
| V2b, loop step 600 with `T` for 512 | RED | 6 |
| V2c, loop step 256 with `T` for 512 | SURVIVED, equivalent | smaller strips fit in `T`, and strips are output-invariant |
| V3, `black_levels` read under the mix | RED | 12, incl. `ac5_black_levels_is_not_read_under_the_mix` |
| V4a, `MONO_MIX_VERSION` "2" | RED | 5: `ac4_…`, `developed_mix_contract_constants_…`, `developed_mix_hashes_match_committed`, … |
| V4b, `DEVELOP_VERSION` "9" | RED | 3: `ac4_…`, `developed_mix_contract_constants_…`, `developed_mix_hashes_match_committed` |
| V5, the `OpcodeList1` refusal skipped under the mix | RED | 6: `opcode::tests::ac6_mono_mix_path_…`, `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels`, `ac8_*`, `ac10_*` |
| V6a, `T` one halo row short | RED | 7 |
| V6b, `T` sized for two strips | RED | 3: `ac9_scratch_len_…`, `ac3_…`, `ac4_…` |
| V7, optional warp skipped (develop and `scratch_len` consistent) | RED | 9: `ac1_parity_grid_…`, `ac1_the_developed_mix_…`, `ac9_…`, `ac8_*`, … |
| V8, V9, V10, shared path | RED | see "Shared code" above |
| V11, the scatter's orientations 5 and 7 swapped | RED | 3: `output_coords_from_crop_is_the_inverse_of_crop_source_coords`, `ac1_the_developed_mix_…`, `ac1_parity_grid_…` |
| V12, garbage in the strip scratch before each strip | SURVIVED, equivalent (the desired result) | — |
| V13, CFA checked after the calibration and the row | RED | 1: `ac8_every_ranked_pair_on_both_develop_entries` |

V13's first run survived because of a bug in the mutant, not the code. It bound the row's `Result` without `?`, which keeps the original order. Re-run as written above, it is red.

**The argued equivalents hold.**
- **(u):** both resolutions are pure functions of the sensor and the weights, resolved once in `develop_core`.
- **(y):** a scatter-dropped row is harmless by the halo argument. V2c and V12 support the strip-invariance and stale-scratch halves.
- **(z):** V2a is red only on the scratch literals, AC4's scratch numbers included. Every pixel assertion passes.

### Rulings on the build's deviations

1. **F10 removes both colour matrices: accepted.** Measured: with `ForwardMatrix1` present, removing `ColorMatrix1` alone develops `Ok`, so "no colour matrix" has to remove both. With F11 added it gives `MixRowOutOfRange`, as the build said.
2. **AC11's red-proof uses `2^-26` on `kr0`: accepted, and the measurement holds.** One ulp up or down on each of `kr0` to `kr3` moves neither warped hash. `kr0 + 2^-26` moves both, and `2^-27` to `2^-30` move neither. So `2^-26` is the smallest power-of-two step that moves them.
3. **Fuzz option bit 6: accepted.** On `origin/main`, bit 5 has been `black_levels = PerPosition` since SPEC-066.
4. **`RING` at module level: accepted.** It is `pub(crate)`, the value 8 is unchanged, and AC9(d) names it beside the strip height.
5. **One combined allocation in `develop_rows_banded`: accepted.** It is allocation only, and the existing band and threaded hashes and tests pass.
6. **Only `tests/developed_mix.rs` seen failing first: accepted as a follow-up lesson, not ship-blocking.** Every later test is shown red by a mutant here or in the build: AC9(d) by (q1), (s), V2a, V6a and V6b; the opcode unit test by V5; the AC10 tests by V5 and (i); the determinism hashes by V1, V4a, V4b, V8 and V9.
7. **Path-aware `required_processing`, 236 → 276 and 15 → 17: accepted.** They follow from DEC-084 item 3.
8. **`allow(clippy::too_many_arguments)` with a reason: accepted.** `lint-no-allow` is green.

### Cost self-report

- **Tokens (total):** 21,924,077, a floor taken live. Re-sum at ship.
- **Estimated USD:** about 17.05, per component at Opus 5.5 rates with 1-hour cache writes.
- **Duration (minutes):** about 240 of wall time, including an app restart that killed the mutant run part way. It was resumed from where the log stopped, and the in-flight mutant was re-run.
- **Source of the number:** this session's transcript, `ef47f062-3d10-4ec0-8ce2-86a58d193f1b.jsonl` (no subagents), deduplicated by `message.id` per docs/cost-tracking.md.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verify. Two tests were added to `tests/developed_mix.rs`: `ac1_parity_grid_composes_every_option` and `ac8_one_warp_block_both_malformed_and_tangential_is_f8`.
- **Findings** (none ship-blocking):
  - **V-1 (follow-up, closed here).** The (F8, F13) skip was wrong: the pair can coexist in one block. It is closed by the new test, and the code was already right.
  - **V-2 (follow-up, closed here).** onyx's parity grid was not fully crossed. It is closed by the new test.
  - **V-3 (follow-up, [XS]).** `ac4_held_files_match_the_oracle_and_the_digests` silently `continue`s past any held file missing from the corpus. With `IRRADIANCE_CORPUS_DIR` pointing at a partial corpus it passes vacuously. Assert how many files ran when the variable is set, as some Tier-B suites do. This run checked all five.
  - **V-4 (follow-up, carried).** The `demosaic` fuzz target still has no CI smoke job (signal `fuzz-targets-without-a-ci-smoke-job`), and it now reaches the developed mix.
  - **V-5 (note).** `develop_into`'s warp has no cross-arch hash of its own; the developed-mix warped pair is the only one. V9 shows it is effective. A mono warp hash in `tests/determinism.rs` would remove the single point of failure. This is optional.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   - The spec pre-registered (F7, F8) as impossible and left the rest of the impossible pairs to the build. A pair should only be skipped with a counter-example search; for (F8, F13) that search took one look at the parser.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   - No. DEC-084 covered everything the code does. The toolchain brief's nightly `PATH` trap is the one gotcha a verifier hits; it is in the brief, not the handoff.
3. **If you did this task again, what would you do differently?**
   - Write each mutant as a full `Result`-propagating edit and probe it before trusting a survivor (V13).
   - Commit test work before a long background run, so a restart cannot leave `src/` mutated beside uncommitted tests.
