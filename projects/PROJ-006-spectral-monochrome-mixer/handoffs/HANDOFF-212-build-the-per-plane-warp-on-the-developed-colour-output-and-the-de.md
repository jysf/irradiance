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
  id: HANDOFF-212
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-100

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
  tokens_total: 44470294           # REAL combined count — what cost-audit reads
  estimated_usd: 30.36             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 47
  branch: feat/spec-100-developed-warp
  pr: 377
  completed_at: 2026-11-28         # YYYY-MM-DD
  notes: "Build in a side session started by the PROJ-006 orchestrator. A FLOOR measured just before the final commit over transcript bf11e18e-c2f6-4e4b-9ee0-d47bc9ea2619.jsonl (131 unique message.ids, all claude-opus-5-5, no subagents): input 269 + cache-creation 440,409 (all 1-hour) + cache-read 43,868,672 + output 160,944. The commit, push and CI wait come after; re-measure at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-28
---

# HANDOFF-212: The per-plane warp on the developed colour output and the developed mix

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-100` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. The held Leica Q2, Q3 and SL3 (a
mandatory three-set `WarpRectilinear`) stop refusing on the two developed outputs:
- the colour develop (`develop_color_into`, `develop_color_f32_into`, both threaded forms)
  applies a **mandatory** first warp as SPEC-099's CA step, then green's set `W`
  (`sensor_geometry_warp(..).consumer()`) on each camera plane **before** the matrix, then
  the crop and orientation; optional warps stay skipped;
- the developed mix applies the first warp **whatever its flags** as the mono develop of
  SPEC-099's CA-corrected mixed plane with `W` at `warp_alpha` and `Framing`, the CA ring
  inside each strip and in the caller's scratch.
No new public item, no version bump, no output change for any file that develops today.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's answers)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-100-the-per-plane-warp-on-the-developed-colour-output-and-the-de.md` (Design §1 to §7, Implementation Context, AC1 to AC10, Failing Tests, the rule-clause map). The spec is the contract.
- **Decision:** `./decisions/DEC-088-the-developed-outputs-apply-the-ca-step-then-green-warp-before-the-matrix-for-colour-over-the-mixed-plane-for-the-mix.md` (proposed). Items 2 to 5 are the contract for the code.
- **Toolchain brief:** `./guidance/toolchain-brief.md` and `AGENTS.md` §5 and §6.

### Decisions that apply

- `DEC-088`: this spec's decision: one rule, the colour develop (DNG order, mandatory only, no options, threaded fallback), the developed mix (`W`, whatever the flags, CA ring per strip, no allocation), scratch, refusals and order, versions.
- `DEC-087`: the CA step (items 2 to 5), reused unchanged; item 6 has a dated amendment line.
- `DEC-084`: the developed mix; items 1, 3, 6 and 7 have dated amendment lines. Rule 1 is the oracle.
- `DEC-086` item 6: replaced (dated line).
- `DEC-059`: `warp_alpha`'s lerp and `FullActiveArea`'s bounds rule, now read on `W`.
- `DEC-060` §2: `develop_with_scratch_into` allocates nothing; `develop_with_into` allocates once.
- `DEC-044` (bands, threads, the 5-to-8 fallback), `DEC-043` (the colour crop scatter), `DEC-024` (bilinear), `DEC-058` §5 (versions).
- `DEC-069` §6 and `DEC-003`: the SDK and tier-B files stay local; tests skip by name.

### Constraints that apply

- `no-panics-on-untrusted-input`: checked arithmetic and `.get` reads in the gather, the strips and the scratch split; sizes from validated geometry.
- `oracle-must-be-shown-red`: AC9's red-proof (colour without the CA step at least 0.30 px off); AC3's composition is built from public primitives and the tests' own CA reference, never crate code.
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row (class 1: DNG's stage order and `WarpRectilinear`).

### Prior related work

- SPEC-099 (HANDOFF-210/211, PRs 373/374): the CA step, `CaDelay`, `ChromaticShift`, `tests/support/ca_reference.rs`, `tests/colour_sdk_stage3.rs`, `scripts/sdk-stage3.sh`. Its reflection: write the order mutants first; check that float mutants differ on Linux too.
- SPEC-093 (the developed mix, `tests/developed_mix.rs`'s rule-1 oracle and ranked faults), SPEC-050/051 (the colour scatter and threads), SPEC-064 (`warp_alpha`).

### Facts the design probe measured (2026-11-28, release build, this machine)

- **All numbers are in the spec's Implementation Context:** geometry (the Q3 and SL3 are Orientation 8, output 6336×9520), the exactness route, six colour and fifteen mix pre-registered digests, the rejected post-matrix order, the AC9 table.
- **Pre-registered digests** come from a probe built on public entries only: `srgb_linear_plane_into` and `mono_mix_plane_into` (SPEC-099's pins, reproduced), `ca_reference.rs`, `apply_warp_into`, `to_srgb_linear_into`, and the TIFF orientation table. If a digest misses, compare the gather with `apply_warp_into` term by term (Notes for the Implementer) and report; never edit a pin.
- **Exactness:** `srgb_linear_plane_into == to_srgb_linear_into(stripped, reference(demosaic_into(stripped, n)))` at every sample on all three files (the SL3 with DEC-086's fix applied by hand: 10,615 sites). So AC3 and AC7 can check the colour develop bit for bit.
- **The SDK references** already exist from SPEC-099's build, outside the repo: `/private/tmp/claude-501/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance--claude-worktrees-spec-099-build-ddb2b5/111b6c75-4198-4766-9cc9-425a4fe387af/scratchpad/sdk3` (`q2-full.tif`, `q2-green.tif`, `q2-green.dng`, and the same for `q3` and `sl3`). If that folder is gone, regenerate with `scripts/sdk-stage3.sh ~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate ~/PSeven/experiments/crustimg_redo_plus/images <dir outside the repo>`.
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-100-design-probe.rs.txt` (the composition, the mix digests, the SDK oracle, the matrix measured from `to_srgb_linear_into`), `spec-100-design-probe-postmatrix.rs.txt` (the rejected order) and `spec-100-design-probe-output.txt`.
- **Corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file, SDK output or derived pixel into the repo, a fixture, a seed or CI.**
- **Cost baseline:** the probe's composition took 2.4 s (CA plane) plus 2.7 to 2.9 s (three warps, matrix, crop) on the Q3, single-threaded. Time `develop_color_into` and the default developed mix on the Q3 and record them (not a gate).

### Watch-outs found at design

- `ChromaticShift` holds its tables in `Vec`s and `CaDelay` owns a `Vec` ring. Under the mix's scratch form that breaks the zero-allocation tests. Move the tables to fixed arrays (`4097` `f64` each, the nodes likewise) and let `CaDelay` borrow its ring; the sensor-geometry pins must not move.
- `tests/developed_mix.rs::ac6_the_mix_sizes_an_optional_multi_plane_warp_as_it_develops` encodes DEC-086 item 6 (optional `N = 3` skipped, mandatory refused). It must be rewritten, not deleted (AC6).
- `tests/leica_colour.rs` asserts the developed outputs refuse `{ list: 3, id: 1 }`; those asserts go, replaced by AC7.
- With an optional warp, or no warp, the colour develop must stay on today's code path, untouched; the same for the mix without a CA step.

## Expected Deliverables

- Code and doc changes implementing SPEC-100's Acceptance Criteria (Design §1 to §7); the colour entries', threaded forms', `DevelopOptions::mono_mix`, `scratch_len` and both `DevelopPath` variants' docs.
- All Failing Tests written first, seen failing against `main`, then passing.
- The rule-clause map's mutants, each with the test that turned red (order mutants first).
- AC9 run locally with `IRRADIANCE_SDK_STAGE3_DIR` set; its per-file, per-plane numbers in the handback.
- CHANGELOG `[Unreleased]`: the two developed outputs develop the Q2, Q3 and SL3; the colour develop applies a mandatory warp (DNG order) and still skips optional ones; the developed mix applies the first warp whatever its flags with `N` 1 or 3 (an optional `N ≠ 1` warp, skipped before, is now applied or refused as malformed).
- `docs/provenance-ledger.md`: one row.
- A PR against `main` from branch `feat/spec-100-developed-warp`, referencing HANDOFF-212, SPEC-100, STAGE-034, PROJ-006, DEC-088, DEC-087, DEC-084, DEC-086 and DEC-059.

## Out of Scope

- `warp_alpha`, `Framing` or `apply_orientation` on the colour develop; any new public item.
- Applying an optional warp on the colour develop.
- A ring in place of the colour develop's `3A` camera buffer; threading the warped colour develop (follow-ups).
- The CA step's arithmetic, the accessor, the rim rule (SPEC-099).
- The mono develop of a colour file (`mono_mix: None`), `parse_warp_rectilinear`, `apply_warp_into`'s behaviour.
- `DEVELOP_VERSION`, `MONO_MIX_VERSION`.
- Committing, copying or redistributing any `-PB` corpus file or SDK output.

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

- **Branch / PR:** `feat/spec-100-developed-warp`, PR 377 (https://github.com/jysf/irradiance/pull/377).
- **Completed at:** 2026-11-28
- **All acceptance criteria met?** yes.
  - AC1: `src/opcode.rs` units (`colour_applies_the_first_fix_bad_pixels_and_every_gain_map`, `ac6_mono_mix_path_applies_gain_maps_and_the_first_warp`): both paths accept a mandatory first warp with `N` 1, 2 and 3 and an optional `N = 3`, and refuse a second mandatory one (also behind an optional one) and a warp in lists 1 and 2; the stage primitives still apply nothing from list 3. `Applied::FirstSinglePlane` is gone. `tests/required_processing.rs`: the every-path table (Colour and MonoMix columns now `Ok` for mandatory `N = 1` and `N = 3`), the every-entry list-3 test, and the mix's applied/sized warp by output.
  - AC2: `tests/colour_warp.rs` `ac2_unchanged_cases_are_bit_identical`: no list 3, optional `N = 1`, optional differing `N = 3` and optional malformed `N = 2` give the no-list-3 bytes on all four colour entries at 1 and 4 threads (plain and `ActiveArea` scenes, `DefaultCrop`, Orientation 1); mandatory identical `N = 3` equals mandatory `N = 1` on the colour develop and the developed mix; a mandatory identity warp copies. SPEC-093's `tests/developed_mix.rs` oracle stays green.
  - AC3: `ac3_the_colour_develop_is_the_dng_order_composition` and `ac3_every_orientation_and_the_active_area`: the three variants (plain, `ActiveArea (2, 2, 38, 46)` with a crop inside, `cx 0.3 cy 0.6`) under orientations 1 to 8 equal the composition at every sample; the `f32` entry is `u16 / 65535` bit for bit; the output differs from the no-CA composition, the post-matrix order and red's set as the warp; red-only leaves blue unresampled; mandatory `N = 1` is the composition without the CA step. The composition uses only `demosaic_into`, `to_srgb_linear_into`, `apply_warp_into`, `parse_warp_rectilinear`, `ca_reference.rs` and the file's own TIFF table.
  - AC4: `tests/developed_mix.rs` `ac4_the_mix_is_the_mono_develop_of_the_ca_plane`: three scenes (no `ActiveArea`, fixture (i), fixture (i) off-centre) times mandatory and optional differing `N = 3`, both weight sets, SPEC-093's whole grid (orientations 1 to 8, both framings, `apply_orientation` both ways, `warp_alpha` 0, ½, 1): 480 cases equal the mono develop of `mono_mix_plane_into`'s plane with `W` as one `N = 1` set, on both develop entries; the CA step changes the mixed plane; red's set as the warp differs. `ac4_ca_strips_tile_across_a_strip_edge`: an 8 × 1100 plane (`H` = 63 rows by the tests' own halo), four option sets.
  - AC5: `ac5_scratch_len_is_the_formula` (no CA: SPEC-093's numbers; CA without a warp `T_H + R`; CA with a warp `A + max(A, T_H + R)` on fixture (i), where `T_H + R > A`, and on the 8 × 1100 plane, where it is `2A`; `± 1` refused, nothing written). `tests/develop_scratch_alloc.rs` `ac5_the_ca_mix_allocates_nothing_with_scratch` (seven fixtures, both framings, `warp_alpha` 0, ½, 1, orientations 1 and 8: 0 allocations) and `ac5_the_ca_mix_allocates_once_without` (exactly one allocation of `scratch_len` samples), in debug and the corpus profile.
  - AC6: `tests/colour_warp.rs` `ac6_colour_refusals_and_their_order` (`N = 2` differing and identical, blue's `kt`, a folding green with `slope_min = −5`, identical sets and `N = 1` with `kt`; each loses to an unsupported CFA and beats `MissingColorCalibration`; sentinel buffers untouched on all four entries, threaded at 3; the same lists flagged optional develop as with no list 3; a second mandatory warp is still `{ list: 3, id: 1 }`). `tests/developed_mix.rs` `ac6_the_mix_per_plane_refusals` (`N = 2` mandatory and optional malformed on both entries and `scratch_len`; blue's `kt` after the CFA and before the calibration, weights first; the fold on the develop and `scratch_len`; `CFAPlaneColor [0, 1, 1]` on `scratch_len`; no CFA sizes with `H = 0`; under `FullActiveArea` a failing `W` refuses while a failing red develops). The ranked fault table gained F10 (differing sets with a tangential term, between the CFA and the calibration; old F10 to F13 are F11 to F14): 102 and 88 pairs; (F8, F10) is skipped as impossible like (F8, F14). The optional-`N = 3` test is rewritten (applied as `W`, sized as a warp).
  - AC7: `tests/leica_colour.rs` `ac7_the_opcode_files_develop`, 3/3 files. **All 21 pre-registered digests matched on the first run**: colour `u16` / `f32` Q2 `0x15fde0a3aad3b45e` / `0xbd4bee088e773f2c`, Q3 `0x3a822d1812f19464` / `0x6af7d2cd38f95e42`, SL3 `0x1d8d6442e403cd46` / `0xd5f79bb4f80cb5d0`; the developed mix's five per file as registered. The threaded forms at 2, 3 and 8 threads equal; both mix entries agree; `scratch_len` is `2A` at `warp_alpha > 0`. On the Q2 and Q3 `develop_color_into` equals the composition at every sample (0 differ). The developed-output refusal asserts in `ac7_the_opcode_files_are_ca_corrected` are gone; the primitives still refuse `{ list: 1, id: 4 }`; SPEC-099's sensor-geometry and mono pins hold.
  - AC8: `ac8_threads_give_the_same_bytes` (three variants × three set shapes × eight orientations, 1, 2, 3, 8 threads) and `tests/threaded.rs` `threaded_colour_develop_with_a_mandatory_warp_equals_single_threaded` (four Bayer phases × eight orientations on the odd-top geometry).
  - AC9: `tests/colour_sdk_stage3.rs` `ac9_the_developed_outputs_register_to_dng_validate_stage_3`, with SPEC-099's `scripts/sdk-stage3.sh` outputs (outside the repo), 3/3 files. Worst `|ours − baseline|` / smallest no-CA: Q2 red 0.042 / 0.592, blue 0.022 / 0.480, green 0.000; Q3 red 0.018 / 0.690, blue 0.016 / 0.442, green 0.000; SL3 red 0.032 / 0.620, blue 0.036 / 0.406, green 0.000. The developed mix: Q2 0.056, Q3 0.012, SL3 0.010 px. Identical to the design probe's table.
  - AC10: no other pin moved (the whole suite with the corpus); `DEVELOP_VERSION` "8", `MONO_MIX_VERSION` "1"; `just semver-check`: "no semver update required" (202 checks pass).
- **Failing tests first:** the new and rewritten tier-A tests were run against `origin/main` (a throwaway worktree, only the test files copied): `colour_warp` 5 of 5 red (every mandatory case refuses `{ list: 3, id: 1 }`), `developed_mix` 8 red (the AC4, AC5, AC6 and ranked-table tests: the mandatory `N = 3` refuses, the optional one is skipped), `develop_scratch_alloc` 3 red, `threaded` 1 red, `required_processing` 2 red, `plane_warp` 1 red. Tier B on main: the developed outputs refuse, so AC7 cannot reach its digests.
- **Order of work:** `ChromaticShift`'s tables moved to `[f64; 4097]` arrays and `CaDelay` borrows its ring first; SPEC-099's `plane_warp`, `leica_colour` and `colour_sdk_stage3` (AC8) were green before anything else changed.
- **Gates:** `just test` with the corpus (every manifest file present, the Q2, Q3 and SL3 included; AC7 ran on 3/3): 63 binaries, 1,054 passed, 0 failed, 1 ignored. `just lint-ci`, `lint-fuzz`, `fmt-check`, `doc-check`, `wasm-check`, `validate` and `semver-check` pass; `cost-audit` after the sync. AC9 ran locally with `IRRADIANCE_SDK_STAGE3_DIR` set (3/3 files).

### Mutants (scripted string replacement, each applied, tested, reverted)

Harness: `mutants.py` in this session's scratchpad: 28 mutants, each applied by exact string replacement, the tier-A binaries (`colour_warp`, `developed_mix`, `develop_scratch_alloc`, `threaded`, `required_processing`, `plane_warp`; the version mutants `developed_mix`, `leica_colour`, `determinism`) run with `--no-fail-fast`, then the files restored from git. Order mutants first. **25 red, 3 output-equivalent:**
- 2 post-matrix order (the matrix into the camera buffer, the identity at the store): both ac3.
- 5 colour per-plane checks before the CFA: `ac6_colour_refusals_and_their_order`. After the calibration: the same. `W`'s `kt` not checked: the same (the calibration then wins).
- 5 mix per-plane checks after `MissingColorCalibration`: `ac6_the_mix_per_plane_refusals`, `ac8_every_ranked_pair_on_both_develop_entries`.
- 2 `W` = `K1` on the colour develop: both ac3. Skip the CA step: both ac3. Apply an optional warp: ac2, ac6.
- 2 gather nearest neighbour, truncated source coordinate, one shared sample: both ac3. `output_coords_from_crop` for `crop_source_coords`: `ac3_every_orientation_and_the_active_area`. Crop origin left out: ac2, both ac3.
- 2 threaded forms band the warped develop: colour ac6, ac8, `threaded_colour_develop_with_a_mandatory_warp_equals_single_threaded`.
- 3 `parse_warp_rectilinear` (`K1`) for the mix: both ac4, `ac5_the_ca_mix_allocates_nothing_with_scratch`, `ac6_the_mix_per_plane_refusals`. Keep DEC-086 item 6's skip: ac2, both ac4, `ac5_scratch_len_is_the_formula`, `ac6_every_path_applies_its_table`, the mix refusals and more. Skip the CA step when `W` is the identity: both ac4. Band `s..e` without the halo: both ac4.
- 4 `T` for `T_H`: `ac4_ca_strips_tile_across_a_strip_edge`, `ac5_scratch_len_is_the_formula`, both AC5 allocation tests. `R` left out: both ac4, the formula, `ac8_the_sizing_entries_check_only_what_they_need`. `2H + 1` ring rows: the same four (not equivalent here: the delay indexes `2H + 2` rows, so a short ring drops writes).
- 5 `scratch_len` skips the fold: the mix refusals. Skips the plane colours: the mix refusals, `ac8_one_warp_block_both_malformed_and_tangential_is_f8`, the sizing test.
- 6 `DEVELOP_VERSION` / `MONO_MIX_VERSION` bump: `ac4_held_files_match_the_oracle_and_the_digests`, `developed_mix_contract_constants_are_the_computation`, the committed-hash tests.
- **Equivalent (no test can see them):**
  - 2 identity `W` resampled instead of copied: `source_coord` of an identity set is the pixel itself (or within an ulp of it), so the bilinear sample is the pixel's value.
  - 3 strip halo `H − 1`: a bilinear read with non-zero weight lies within `⌈B⌉ = H − 1` rows of its pixel (row `y + ⌈B⌉` carries weight 0 when `B` is an integer), and across a strip edge the ring still holds the previous strip's rows; `H = ⌈B⌉ + 1` is the delay's arrival condition (DEC-087 item 4), which the mutant leaves alone.
  - 3 one delay across all strips: the overlapping halo rows are pushed again with identical values, so the ring's contents and every finished row are the same.

### Timing and memory (release, Q3, single thread, best of 3)

- `develop_color_into`: 4.21 s, peak RSS 1.06 GB for the process (the 121 MB mosaic, the 363 MB output, the 363 MB `ActiveArea` camera buffer and the 121 MB green plane). The design probe's composition took 2.4 s plus 2.7 to 2.9 s.
- The default developed mix (`develop_with_into`, Rec. 709 luma): 3.37 s, peak RSS 0.58 GB (mosaic, output and `2A` scratch).

### Cost self-report

- **Tokens (total):** 44,470,294 (a floor measured just before the final commit; input 269, cache creation 440,409, all 1-hour, cache read 43,868,672, output 160,944; 131 messages, all claude-opus-5-5, no subagents)
- **Estimated USD:** 30.36 (5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x)
- **Duration (minutes):** 47 (the transcript's first-to-last timestamp at measurement; the CI wait comes after)
- **Source of the number:** transcript `bf11e18e-c2f6-4e4b-9ee0-d47bc9ea2619.jsonl` (this session's scratchpad id), per `docs/cost-tracking.md` "Measuring a cycle": unique `message.id`s, the largest `output_tokens` per message, no subagents. A floor taken before the final commit.

### Drift and new artifacts

- **New decisions emitted:** none (DEC-088 built as written).
- **Deviations from spec:**
  - `Stage::Mix` carries the resolved `ChromaticShift` inline (two 4097-entry tables, about 64 KB on the stack), so `clippy::large_enum_variant` is allowed on `Stage` with the reason: boxing would allocate, which the scratch form must not.
  - The ranked table's new fault is numbered F10 and the later ones renumbered (F11 to F14), rather than appended, so the table stays in rank order; the pair counts moved from 89 / 76 to 102 / 88.
  - `scratch_len` under the mix reads no tangential term (a differing `kt` sizes as without it); the develop refuses it. DEC-088 item 5 lists only the parse, the plane colours and the fold for `scratch_len`, so this is as written, recorded because AC6 tests it.
  - `tests/plane_warp.rs`'s SPEC-099 AC6 tail (which asserted the developed outputs refuse) now asserts they develop and that the mix applies an `N = 3` warp, mandatory or optional.
- **Follow-up work identified:**
  - A delay ring in place of the colour develop's `3A` camera buffer (about 300 MB saved on the Q3), and threading the warped colour gather over output rows (DEC-088 alternatives; not this spec).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The ranked fault table's "gains the per-plane faults at their rank" left the numbering open; I inserted at rank and renumbered.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `tests/required_processing.rs`'s every-path table and every-entry list-3 test, and `tests/plane_warp.rs`'s SPEC-099 AC6 tail, encode the old refusals; they were in the Failing Tests list only as "AC1", so finding them took a run.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Pre-registered digests from a public-primitive probe matched first time again; writing the composition check into AC7 made the SL3's digest-only case the only one not tied sample by sample.
