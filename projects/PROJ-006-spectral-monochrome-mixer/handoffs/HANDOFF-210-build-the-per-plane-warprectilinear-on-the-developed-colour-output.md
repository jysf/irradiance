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
  id: HANDOFF-210
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-099

project:
  id: PROJ-006
  stage: STAGE-034
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
  tokens_total: 45828995           # REAL combined count — what cost-audit reads
  estimated_usd: 30.76             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: feat/spec-099-sensor-geometry-ca
  pr: null
  completed_at: 2026-11-28         # YYYY-MM-DD
  notes: "Build in a side session started by the PROJ-006 orchestrator. A FLOOR measured live just before the commit over transcript 111b6c75-4198-4766-9cc9-425a4fe387af.jsonl (134 unique message.ids, all claude-opus-5-5, no subagents): input 271 + cache-creation 417,566 (all 1-hour) + cache-read 45,253,091 + output 158,067. The commit, PR and CI wait come after; re-measure at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-28
---

# HANDOFF-210: The per-plane WarpRectilinear at sensor geometry: red and blue resampled to green's geometry, and the consumer's warp named

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-099` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. On a first `WarpRectilinear` with
`N = 3` whose red or blue set differs from green's, the six sensor-geometry entries
(`srgb_linear_plane_into`, `srgb_linear_plane_f32_into`, `mono_mix_plane_into` and their
`_rows_` forms) resample red and blue to green's geometry on the demosaiced camera triple,
before the matrix, through a delay ring between the fused traversal and `store`. A new
public accessor `opcode::sensor_geometry_warp` names the one warp the consumer applies
(green's set) and every set with its `CFAPlaneColor` colour. One new `Error` variant
(`UnsupportedWarpFold`), one new local-only SDK cross-check (`scripts/sdk-stage3.sh`), no
version bump. The developed colour output and the developed mix are untouched (SPEC-100).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's answers of 2026-11-28)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md` (Design Notes: onyx's rule 1 as amended)
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-099-the-per-plane-warprectilinear-on-the-developed-colour-output.md` (Design §1 to §7, Implementation Context, AC1 to AC10, Failing Tests, the rule-clause map). The spec is the contract.
- **Decision:** `./decisions/DEC-087-the-sensor-geometry-planes-resample-red-and-blue-to-green-geometry-and-leave-green-warp-to-the-consumer.md` (proposed). Item 3 is the exact arithmetic; implement it term by term.
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-087`: this spec's decision: the accessor, when the CA step runs, the rule, where, the refusals and their order, what does not change.
- `DEC-086` items 5 and 8 (dated amendment lines): the sensor-geometry path and the accessor it deferred.
- `DEC-024`: the bilinear kernel and its clamp-to-edge rule, reused for the CA sample (clamped to the `ActiveArea`).
- `DEC-059` §3: `scale_bounds`, reused for the fold check on `[kr0, 3kr1, 5kr2, 7kr3]`.
- `DEC-044`: band invariance, which the rows forms' extra `H` rows rely on.
- `DEC-079`, `DEC-083`: the planes' and the mixer's arithmetic, unchanged; the CA step only replaces the camera triple they read.
- `DEC-069` §6: the SDK is run locally only; never vendored, built in CI or linked; none of its output is committed.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.

### Constraints that apply

- `no-panics-on-untrusted-input`: checked arithmetic, `.get` reads; the table and ring are sized from validated geometry; the `warp_opcode` fuzz target gains the per-plane parser (AC10).
- `oracle-must-be-shown-red`: AC8 asserts its red-proof (green-only at least 0.30 px off); AC3's reference must be the test's own code from DEC-087's text.
- `test-before-implementation`: write the Failing Tests first and see each fail (or mismatch) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row for the CA rule (class 1, derived from DNG 1.7.1.0's own transform).

### Prior related work

- SPEC-018 and SPEC-064 (`src/warp.rs`), SPEC-091, SPEC-092 (the entries), SPEC-093 (the developed mix, untouched here), SPEC-098 (`DevelopPath::SensorGeometry`, the pins this spec moves).
- HANDOFF-208/209 and PR 367/368: SPEC-098's build and verify. Its verify found five oracle gaps by mutation; the rule-clause map here is written to close that class up front.

### Facts the design probe measured (2026-11-28, release build, this machine)

- **The full tables are in the spec's Implementation Context:** the nine coefficient sets, the fold minima, the CA displacements, the table error, `H = 3`, the oracle per file and plane, and the three pre-registered `u16` digests.
- **The pre-registered `srgb_linear_plane_into` digests** (Q2 `0xd352bb75bf29d270`, Q3 `0xc6f67381a8b94593`, SL3 `0xbb9908b919ce2b08`) come from a probe that reproduced SPEC-098's pins with the step off. If yours differ, check DEC-087 item 3 term by term before anything else, and report; never edit a pin to match. The `f32` and mono pins are re-pinned (no independent prediction); say in the handback that the `u16` pins matched first.
- **Exactness route:** with the step off, the fused plane equals `to_srgb_linear_into(demosaic_into(n))` at every sample on all three files, `n` the test's own normalisation given in the spec. Use it for AC3 and AC7.
- **The oracle (AC8)** needs `dng_validate -3` on each file and on a scratch copy whose red and blue sets are overwritten by green's. The probe's patcher is `spec-099-design-patch-warp.py.txt`; turn it into `scripts/sdk-stage3.sh` (shell plus a short Python or Perl patch, writing only into its output directory). The SDK binary is at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`. Each SDK run takes a few seconds; the stage-3 TIFFs are 280–360 MB each.
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-099-design-probe-coeffs.rs.txt` and `-coeffs-module.rs.txt` (the sets and the fold check), `spec-099-design-probe-oracle-camera.rs.txt` (the camera-RGB oracle with the Lucas–Kanade shift), `spec-099-design-probe-srgb-digests.rs.txt` (the digests, the exactness check and the sRGB-space attempt), `spec-099-design-patch-warp.py.txt`, and `spec-099-design-probe-output.txt`.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file, SDK output or derivative pixel anywhere in the repo.**
- **Cost baseline:** the probe's naive CA (two planes, 60 MP, single thread) took 0.49–0.64 s; `srgb_linear_plane_into` is about 1.1 s today on the Q3 and SL3. Measure before and after on the Q3 and record it (not a gate).

## Expected Deliverables

- Code and doc changes implementing SPEC-099's Acceptance Criteria (Design §1 to §7), the six entries' docs saying the plane is CA-corrected and the consumer applies `sensor_geometry_warp(..).consumer()`.
- All Failing Tests written first, seen failing against `main`, then passing; SPEC-098's sensor-geometry pins in `tests/leica_colour.rs` moved as AC7 says.
- The rule-clause map's mutants, each with the test that turned red.
- The `warp_opcode` fuzz smoke with the new seed.
- `scripts/sdk-stage3.sh`, the `$IRRADIANCE_SDK_STAGE3_DIR` SKIP line in `corpus-status`, and AC8 run locally with its numbers in the handback.
- CHANGELOG `[Unreleased]` lines: the accessor, the CA step and the new error; `N = 2` and `N ≥ 4` now refused on the sensor-geometry entries.
- A PR against `main` from branch `feat/spec-099-sensor-geometry-ca`, referencing HANDOFF-210, SPEC-099, STAGE-034, PROJ-006, DEC-087, DEC-086, DEC-059 and DEC-024, with the provenance row and the oracle stated.

## Out of Scope

- `develop_color_*`, the developed mix, `warp_alpha` and `Framing` on them, their thread forms and optional-warp rules: SPEC-100.
- Any change to the mono develop, `parse_warp_rectilinear`, `apply_warp_into`, `demosaic_into`, `to_srgb_linear_into`, `DEVELOP_VERSION` or `MONO_MIX_VERSION`.
- Tangential warp terms when the CA step runs, and `N` other than 1 or 3.
- Committing, copying or redistributing any `-PB` corpus file or any SDK output.

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

- **Branch / PR:** `feat/spec-099-sensor-geometry-ca`, PR linked in the PR's own description (opened after this commit).
- **Completed at:** 2026-11-28
- **All acceptance criteria met?** yes.
  - AC1: `tests/plane_warp.rs` `ac1_sensor_geometry_warp_names_the_consumer_warp_and_every_set`: no list 3 and a warp-free list give `Ok(None)`; three distinct sets come back in file order coloured `[0, 1, 2]`, each equal to its set with `planes == 3`, `consumer()` green's, per plane; mandatory and optional agree; `CFAPlaneColor` `[2, 1, 0]`, `[1, 0, 2]`, `[0, 2, 1]` colour by `CFAPlaneColor` and name the set coloured 1; identical sets are not per plane; `N = 1` gives one uncoloured set; monochrome `N = 1` and `N = 3` give `K1`; `N = 2` and `N = 4` are malformed; `[0, 1, 1]` is `UnsupportedCfaPlaneColor`; a second warp is ignored; `kt` is reported; `parse_warp_rectilinear` still returns `K1` (red).
  - AC2: `ac2_no_per_plane_difference_is_bit_identical`, six entries (three whole, three rows forms over two strips), plain and `ActiveArea` scenes, four list shapes.
  - AC3: `ac3_red_and_blue_are_resampled_to_green_geometry` (including the optional flag and red-only) and `ac3_the_active_area_and_the_optical_centre` (`ActiveArea (2, 2, 38, 46)`, `cx 0.3 cy 0.6`, and both). The reference is `tests/support/ca_reference.rs`, written from DEC-087 item 3 and shared with AC7 and AC8. Every sample equal; the masked border equals the no-CA output; the corner reaches the held rim (`f_g(1)^2 = 0.81`).
  - AC4: `src/color.rs` `ac4_every_store_sees_the_corrected_triple` (the triple captured through the same body with a store that writes the camera triple; u16, unclamped f32 and mono each equal their store on it; green passes through, red and blue move).
  - AC5: `ac5_bands_equal_the_whole_plane`: tilings `0..3, 3..11, 11..40`, every single row, `0..40` and `0..1, 1..39, 39..40`, on the plain and `ActiveArea` scenes, three rows entries.
  - AC6: `ac6_refusals_and_their_order`: blue kt and red-before-blue kt; the fold (`slope_min == -5`, kind `Unsupported`, `Display` names it); `q` minimum exactly 0 refuses; a fold with identical sets is not refused; `N = 2` differing and identical; CFA pattern and `CFAPlaneColor` beat the warp; the warp beats `MissingColorCalibration`; `MissingColorCalibration` still comes after a clean warp; `InvalidMixWeights` beats it on both mix entries; `InvalidRowRange` stays before it; sentinel `dst` untouched on every refusal; a 1x1 `ActiveArea` with a folding green equals the no-list-3 output; `develop_color_into`/`_f32_` still `{ list: 3, id: 1 }`, the developed mix refuses a mandatory N = 3 and skips an optional one (equal to the no-list-3 develop). `tests/error_kind.rs` classifies the variant (`Unsupported`, 29).
  - AC7: `tests/leica_colour.rs` `ac7_the_opcode_files_are_ca_corrected`, ran on 3/3 files. The accessor gives the table's three sets, coloured R, G, B, `consumer()` the G row, per plane. **The three pre-registered `u16` digests matched first**: Q2 `0xd352bb75bf29d270`, Q3 `0xc6f67381a8b94593`, SL3 `0xbb9908b919ce2b08`. Re-pinned after that: `srgb_linear_plane_f32_into` Q2 `0x46f44d43f6882eb4`, Q3 `0xaf64a2b4e74aef07`, SL3 `0x1112c7705d7631e7`; `mono_mix_plane_into` Q2 `0xe6bb2752f00f7913`, Q3 `0x97329ab81317c5ec`, SL3 `0x00d47ed9b11d1be2`; each differs from its SPEC-098 pin. Rows 100..200, 0..5 and the last 5 equal the plane on all three entries. The plain mono `develop_into` pins are unchanged; the developed outputs still refuse. On the Q2 and Q3, `srgb_linear_plane_into` equals `to_srgb_linear_into(reference(demosaic_into(n)))` at every sample (0 differ).
  - AC8: `tests/colour_sdk_stage3.rs` `ac8_red_and_blue_register_to_dng_validate_stage_3`, with `scripts/sdk-stage3.sh` outputs outside the repo, 3/3 files. Worst `|CA+green - baseline|` / smallest `|green-only - baseline|` (bound 0.10 / red-proof 0.30): Q2 red 0.042 / 0.592, Q2 blue 0.022 / 0.480, Q3 red 0.018 / 0.690, Q3 blue 0.016 / 0.442, SL3 red 0.032 / 0.620, SL3 blue 0.036 / 0.406. Per patch in the test's output; close to the design probe's table.
  - AC9: the whole suite is green with the corpus (counts below); `DEVELOP_VERSION` "8", `MONO_MIX_VERSION` "1"; `just semver-check` reports "no semver update required" (additions only).
  - AC10: `fuzz/fuzz_targets/warp_opcode.rs` runs `__fuzz_warp_planes` for `None`, `[0, 1, 2]` and `[2, 1, 0]` with the invariants asserted; seed `fuzz/seeds/warp_opcode/warp-rectilinear-n3-distinct.bin`; smoke 3,243,469 runs in 61 s, no crash.
- **Failing tests first:** `tests/plane_warp.rs`'s AC2, AC3 and AC5 were run against main (with the accessor import removed): AC3 (both) and AC5 red (4,700 and 3,889 samples differ from the reference; the CA step did not run), AC2 green (the guard). AC1, AC6 and the unit tests name `sensor_geometry_warp` / `UnsupportedWarpFold`, which do not exist on main (red by compile). AC7's u16 digest on main is SPEC-098's pin, not the pre-registered one.
- **Gates:** `just test` with the corpus (every manifest file present, including the Q2, Q3 and SL3): 62 binaries, 1,032 passed, 0 failed, 1 ignored. The first run failed only `tests/error_kind.rs` `every_variant_classified` (its pinned variant count, 82 to 83; DEC-051's tally sentence gained the new variant) and stopped there, so the suite was run again in full. `just lint-ci`, `lint-fuzz`, `fmt-check`, `doc-check`, `wasm-check`, `validate` and `semver-check` pass; `cost-audit` passes after the sync. AC8 ran locally with `IRRADIANCE_SDK_STAGE3_DIR` set (3/3 files); unset, `corpus-status` prints its SKIP line.

### Mutants (scripted string replacement, each applied, tested, reverted)

Harness: `mutants.py` in this session's scratchpad (each mutant applied by exact string replacement, `tests/plane_warp.rs` plus the CA unit tests run, then reverted). All 21 red in the end:
- 1 consumer = `K1` (red's set): red, ac1, both ac3, ac6, `ac4_every_store_sees_the_corrected_triple`, `ca_equal_sets_build_no_step` (AC7's consumer assertion would also fail; not run under the mutant, being tier B).
- 1 colours by index: red, ac1 (the `[2, 1, 0]`, `[1, 0, 2]`, `[0, 2, 1]` cases).
- 2 resample an equal-set plane anyway (output-equivalent): red, `ca_equal_sets_build_no_step`.
- 2 identical sets still per plane: red, ac1, ac2.
- 3.1 plane coordinates instead of `ActiveArea`-relative: red, `ac3_the_active_area_and_the_optical_centre`, `ca_value_samples_inside_the_active_area_only`.
- 3.1 `sqrt(mx² + my²)` instead of `hypot`: **survived at first**. On every centred geometry (all held files and the first fixtures) `mx`, `my` are half-integers, the sum is exact and both round alike, so no plane can tell. Closed by `ca_m_is_hypot` (a 6 × 40 `ActiveArea`, centre `(0.07, 0.6)`, where the two differ in the last bit under macOS's libm; the mutant turns it red there). Linux's glibc `hypot` rounds that case like the sqrt form, so CI's run showed the difference is platform-dependent; the test asserts only `m == mx.hypot(my)`.
- 3.2 refuse at `q < 0` only: red, ac6, `ca_fold_boundary_refuses`. 3.2 never refuse: red, the same two.
- 3.3 extrapolate past `s = 1` (bisect on `[0, 2]`, no rim): red, both ac3, `ca_bisection_inverts_a_known_map`.
- 3.4 nearest node: red, both ac3, `ca_ratio_interpolates_and_clamps_at_the_rim`. 3.4 no `min(u, T)` clamp: red, `ca_ratio_interpolates_and_clamps_at_the_rim` (on a plane it is output-equivalent: σ never exceeds 1 by more than an ulp).
- 3.5 clamp 2 px beyond the `ActiveArea` (into the masked border): red, both ac3, ac5. 3.5 truncate instead of half up: red, both ac3.
- 3.6 `H = 0`: red, both ac3, both ac5, `ca_table_and_bound_on_a_hand_checkable_set`. 3.6 `H = ⌈B⌉ − 1`: **red only in the unit test at first**: on the 48 × 40 scenes `B` (a radial bound) is well above the vertical reach, and red's outward sources clamp at the edge. Closed by `ac5_the_halo_covers_a_tall_narrow_active_area` (an 8 × 40 `ActiveArea`, blue shrinking to 0.7 so its sources move inward about 5.9 rows, whole plane against the reference and single-row bands).
- 4 resample the masked border: red, `ac3_the_active_area_and_the_optical_centre`, `ca_value_samples_inside_the_active_area_only`. 4 the ring finishes a row before row `y + H` arrives: red, both ac3, ac5.
- 5 warp before the CFA: red, ac6. 5 warp after the calibration: red, ac6. 5 tangential terms not refused: red, ac6. 5 `N = 2` accepted: red, ac1, ac6.

### Timing (`srgb_linear_plane_into`, release, Q3, best of 5)

- Before (origin/main, SPEC-098, the warp passed through): min 0.964 s, median 0.985 s.
- After: min 2.297 s, median 2.301 s. So the step adds about 1.33 s on a 60 MP plane, single-threaded, against the design probe's 0.49 to 0.64 s for a bare two-plane loop. The extra cost is the ring's per-sample lookups (a row modulo per bilinear read) and the per-plane σ; a follow-up could precompute the ring row bases and share σ between red and blue. Not a gate; recorded, not optimised here.

### Cost self-report

- **Tokens (total):** 45,828,995 (a floor measured live before the commit; input 271, cache creation 417,566, all 1-hour, cache read 45,253,091, output 158,067; 134 messages, all claude-opus-5-5, no subagents)
- **Estimated USD:** 30.76 (5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x)
- **Duration (minutes):** 45
- **Source of the number:** transcript `111b6c75-4198-4766-9cc9-425a4fe387af.jsonl` (this session's scratchpad id), per `docs/cost-tracking.md` "Measuring a cycle": unique `message.id`s, the largest `output_tokens` per message, no subagents. A floor taken before the commit.

### Drift and new artifacts

- **New decisions emitted:** none (DEC-087 built as written).
- **Deviations from spec:**
  - `tests/leica_colour.rs` `ac8_the_sl3_bad_columns_are_fixed` (SPEC-098's, not in this spec's list) now drops list 3 from both sensors before counting the pixels the fix changes. With list 3 kept, the CA resample spreads each fixed photosite into its neighbours' red and blue, and the count moved from 22,761 to 28,012. Without the warp the comparison isolates the fix as SPEC-098 measured it, and every registered number still holds.
  - The per-plane parse also refuses a non-finite coefficient in set 2 or 3 as `MalformedOpcodeParams { id: 1, .. }` (`parse_warp_rectilinear_params` checks only `K1` and the centre). DEC-087 does not say; a NaN set cannot be resampled. No held file has one.
  - `is_per_plane()` is `false` on a sensor with no CFA (no sensor-geometry entry runs there), and the one-pixel `ActiveArea` case (`m = 0`) checks no fold: DEC-087 item 3 lists "if m = 0 there is no step" before the fold check, so the geometry decides first. The tangential refusal comes before both (item 5's order).
  - `bilinear_sample`'s arithmetic moved into `bilinear_sample_with` (a sampler closure), which both `apply_warp_into` and the CA step call; `apply_warp_into`'s output is unchanged (every warp pin and test green).
  - AC8's `n` applies no bad-pixel fix on the SL3: the patches never reach its bad columns (9528 to 9535, outside the crop). Stated in the test's header.
  - Extra tests beyond the spec's list: the rows forms' `InvalidRowRange` stays before the warp refusal; a one-row-then-38-then-one tiling; `ac5_the_halo_covers_a_tall_narrow_active_area` and `ca_m_is_hypot` (both added to close mutants, above); `ca_value_samples_inside_the_active_area_only`.
- **Follow-up work identified:**
  - SPEC-100 (framed) as planned: the developed colour output and the developed mix.
  - Optional, small: the CA step's cost (2.3 s against 1.0 s for the Q3 plane). Precomputing the ring's row bases and sharing σ between the two planes should bring it near the probe's figure; output-invariant.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The one undefined detail was whether a one-pixel `ActiveArea` with a folding green refuses (fold first) or runs nothing (geometry first); AC6 settles the 1x1 case only with a well-behaved green. I took DEC-087 item 3's order (geometry, then fold) and tested it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — SPEC-098's `ac8_the_sl3_bad_columns_are_fixed` depends on the sensor-geometry plane's warp behaviour, so it moves with this spec; it was not in the spec's list of tests to change.

3. **If you did this task again, what would you do differently?**
   — Nothing structural: the shared test-side reference (`tests/support/ca_reference.rs`) let AC3, AC7 and AC8 check one rule, and the pre-registered digests matched on the first run.
