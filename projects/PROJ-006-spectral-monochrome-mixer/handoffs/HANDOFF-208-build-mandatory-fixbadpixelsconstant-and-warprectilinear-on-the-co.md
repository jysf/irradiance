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
  id: HANDOFF-208
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-098

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
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 34834175           # REAL combined count — what cost-audit reads
  estimated_usd: 23.69             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 95
  branch: feat/spec-098-colour-fix-bad-pixels
  pr: 367
  completed_at: 2026-11-27         # YYYY-MM-DD
  notes: "Floor, measured live from this session's transcript (db3b4676), 119 deduplicated messages: input 249, cache creation 354,425, cache read 34,360,863, output 118,638; no subagents. USD at 5/25 per M with 1-hour cache writes."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-27
---

# HANDOFF-208: Mandatory FixBadPixelsConstant on the colour entries and WarpRectilinear passed through at sensor geometry, the Q2, Q3 and SL3 files

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-098` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. The colour develop, the sensor-geometry
planes and the developed mix apply `OpcodeList1`'s first `FixBadPixelsConstant` on the raw
CFA, filling each bad photosite from its four nearest same-colour photosites (colour from
`CFAPattern`; `BayerPhase` unused). A new `DevelopPath::SensorGeometry` lets the planes
pass a mandatory `WarpRectilinear` through to the consumer. The developed colour still
refuses it, and the developed mix refuses a mandatory one with `N ≠ 1`; both wait for
SPEC-099. The Q2, Q3 and SL3 are promoted to `[[file]]` entries. There is no new `Error`
variant, no version bump and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md` (Design Notes: onyx's two rules)
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-098-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md` (Design §1 to §9, Implementation Context, AC1 to AC11, Failing Tests, the rule-clause map, mutants (a) to (r), the moved assertions and the count sites). The spec is the contract.
- **Decision:** `./decisions/DEC-086-the-colour-entries-fix-bad-pixels-from-the-four-nearest-same-colour-photosites-and-sensor-geometry-passes-the-warp-through.md` (proposed)
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-086`: this spec's decision: who applies the fix, where, the kernel, `BayerPhase`, the new path, the warp rules, the error order.
- `DEC-084` item 3, as amended (dated note): the developed mix's opcodes.
- `DEC-040`: the CFA colour is indexed from the `ActiveArea` origin with `wrapping_sub`.
- `DEC-053`, `DEC-049`: `check_required_processing` is every entry's first question; the paths say what each composition applies.
- `DEC-078` §2: `SensorGeometry` refuses the colour path's three malformed-tag sets.
- `DEC-069` §6: the SDK is read and run locally only; never vendored, built in CI or linked; none of its output is committed.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.

### Constraints that apply

- `no-panics-on-untrusted-input`: the kernel uses checked arithmetic only; the `opcode` fuzz target gains the four-path chain (AC10).
- `oracle-must-be-shown-red`: AC1's literals were computed by the Adobe SDK; mutants (a) to (r) must each turn a named test red.
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row for the kernel (Design §9).

### Prior related work

- SPEC-017 (the mono applier), SPEC-054/SPEC-056 (the required-processing check), SPEC-091, SPEC-092, SPEC-093 (the entries), SPEC-097 (the last manifest promotion; its verify found two missed count sites, all listed this time).

### Facts the design probe measured (2026-11-27, release build, this machine)

- **The full table is in the spec's `## Implementation Context`:** opcode facts, bad-photosite counts, the SDK comparisons, the synthetic literals, the digests per entry, and what each entry returns.
- **AC7's digests are pins from the prototype, not an oracle.** Record them as given, and report a mismatch rather than editing it.
- **The prototype passed the whole corpus suite** except the 13 moved assertions the spec lists; `ac6_the_mix_never_parses_opcode_list_1` also moves once the mix parses list 1 (the prototype did not).
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-098-design-probe.rs.txt` (every entry on the three files), `spec-098-design-sdk.rs.txt` (stage 1/2 comparisons), `spec-098-design-synth.rs.txt` (the prototype against SDK-fixed mosaics), `spec-098-design-synth-dng.py.txt` (the synthetic DNG writer), `spec-098-design-prototype.diff.txt` (the uncommitted src diff), and `spec-098-design-probe-output.txt`.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file anywhere in the repo.**

## Expected Deliverables

- Code and doc changes implementing SPEC-098's Acceptance Criteria (Design §1 to §9).
- All Failing Tests written first, seen failing against `main`, then passing; every moved assertion moved as listed.
- The mutant table (a) to (r), with the test each mutant failed.
- The `opcode` fuzz smoke with the new seed.
- A PR against `main` from branch `feat/spec-098-colour-fix-bad-pixels`, referencing HANDOFF-208, SPEC-098, STAGE-034, PROJ-006, DEC-086, DEC-084, DEC-040 and DEC-053, with the provenance row and the oracle (the SDK-computed literals) stated.

## Out of Scope

- The warp on `develop_color_*` and the developed mix, the per-plane parser and accessor: SPEC-099.
- A public CFA bad-pixel applier; the demosaic and colour-transform primitives keep refusing list 1.
- Any change to the mono path, `src/warp.rs`, `DEVELOP_VERSION`, `MONO_MIX_VERSION`, the `Error` enum, or any existing output, hash or Tier-B pin.
- Committing, copying or redistributing any `-PB` corpus file.

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

- **Branch / PR:** `feat/spec-098-colour-fix-bad-pixels`, PR linked in the PR's own description (opened after this commit).
- **Completed at:** 2026-11-27
- **All acceptance criteria met?** yes.
  - AC1: `tests/colour_bad_pixels.rs` `ac1_the_kernel_fills_from_the_four_nearest_same_colour_photosites`, `ac1_colour_is_indexed_from_the_active_area_origin` (and `ac1_the_fix_runs_under_a_black_pattern_larger_than_two_by_two`, added). Every SDK literal matched as registered, on all six entries.
  - AC2: `ac2_bayer_phase_is_not_used`. AC3: `ac3_constant_flags_and_instances`. AC4: `ac4_the_fix_reads_raw_samples`. AC5: `ac5_bands_threads_and_strips_equal_the_whole`.
  - AC6: `tests/required_processing.rs` `ac6_every_path_applies_its_table` (13 list shapes by 6 paths, every entry, plus the mix's N = 1 applied / optional N = 3 skipped by output); `src/opcode.rs` `colour_applies_the_first_fix_bad_pixels_and_every_gain_map`, `sensor_geometry_passes_the_first_warp_through`, `ac6_mono_mix_path_applies_gain_maps_and_the_first_warp`.
  - AC7: `tests/leica_colour.rs` `ac7_the_opcode_files_reach_the_sensor_geometry_entries`, ran on 3/3 files. All nine digests matched as registered (and the three mono `develop_into` digests).
  - AC8: `ac8_the_sl3_bad_columns_are_fixed`: 27,651 zeros in columns 9528 to 9535; 10,615 fixable and 17,036 not; 22,761 changed plane pixels in columns 9525 to 9535; crop ends at 9523. All as registered.
  - AC9: the whole suite is green with the corpus; `DEVELOP_VERSION` and `MONO_MIX_VERSION` unchanged; `just semver-check` reports no semver update required (additive).
  - AC10: the four-path chain in `fuzz/fuzz_targets/opcode.rs`, seed `fuzz/seeds/opcode/warp-rectilinear-n3.bin`; `opcode` smoke 2,348,150 runs in 61 s, no crash.
  - AC11: the three `[[file]]` entries; `plane_md5_matches_the_pinned_raw_checksum` green on all three; the Compression 7 red-proof turned red on the Q3 (mutant 0cd2a978…) and SL3 (mutant 120c3f01…); 37/37 preview pins matched.
- **Failing tests first:** `tests/colour_bad_pixels.rs` was written and run against main before any src change: 6 of 6 AC tests red (every entry refused list 1). `ac6_every_path_applies_its_table` and the opcode unit tests name `DevelopPath::SensorGeometry`, which does not exist on main (red by compile). The AC7/AC8 tests refuse on main with the corpus (list 1).
- **Gates:** `just test` with the corpus 1008 passed, 0 failed, 1 ignored; `just lint-ci`, `lint-fuzz`, `fmt-check`, `doc-check`, `wasm-check`, `validate`, `semver-check`, `cost-audit` all pass. The changed tier-A/tier-B tests also pass with no corpus.

### Mutants (each applied by hand by a scripted string replacement, then reverted)

- (a) green from the axial four: red, the five kernel ACs (ac1 both, ac2, ac3, ac4).
- (b) red and blue from the diagonals: red, the same five.
- (c) colour from the plane origin: red, `ac1_colour_is_indexed_from_the_active_area_origin`, `ac2_bayer_phase_is_not_used`.
- (d) fixed (in-place, raster-order) reads: red, the five kernel ACs.
- (e) truncating mean: red, ac1 (origin), ac2, ac3, ac4.
- (f) n = 0 writes 0: red, `ac3_constant_flags_and_instances`.
- (g) candidates anywhere in the plane: red, ac1 (0,1), ac2.
- (h) neighbours inside the ActiveArea only: red, ac1 (0,1), ac2.
- (i) Constant compared as u16: red, ac3.
- (j) an optional first instance not applied: red, ac3.
- (k) every FBPC in list 1 applied: red, 14 tests (ac3, ac6 table, the opcode unit tests, the mix and mono_mix refusals, the mono duplicate tests).
- (l) the candidate test on the normalised value: red, ac4, `ac6_the_mix_applies_the_first_fix_bad_pixels`.
- (m) BayerPhase decides green (the SDK's parity): red, the five kernel ACs.
- (n) sensor geometry refuses list 3: red, `ac6_every_path_applies_its_table`, `ac6_the_mix_applies_a_mandatory_warp`, `colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry`, `sensor_geometry_passes_the_first_warp_through`.
- (o) colour passes list 3: red, ac6 table, the list-3 colour test, `colour_applies_the_first_fix_bad_pixels_and_every_gain_map`.
- (p) the mix applies an optional N = 3 warp: red, `ac6_every_path_applies_its_table`.
- (q) the fix after normalize_range: red, ac4, `ac6_the_mix_applies_the_first_fix_bad_pixels`.
- (r) Applied::First for MonoMix list 3: red, ac6 table, the list-3 colour test, the mix unit test.
- Extras: (s1) scratch_len sizes the skipped N = 3 warp: red, ac6 table. (s2) the mix resolves no fix: red, ac1 to ac5. (s3) a malformed FBPC swallowed: red, ac3 and the mix malformed test. (s4) the pattern normaliser applies no fix: **survived at first**; closed by the new `ac1_the_fix_runs_under_a_black_pattern_larger_than_two_by_two`, then red. (s5) a short warp block counted as not N = 1: red, the mix unit test.

### Cost self-report

- **Tokens (total):** 34,834,175 (floor, live)
- **Estimated USD:** 23.69
- **Duration (minutes):** about 95
- **Source of the number:** this session's transcript, deduplicated by message id per docs/cost-tracking.md

### Drift and new artifacts

- **New decisions emitted:** none. DEC-086 gained a dated note recording the maintainer's two rulings (Q3/SL3 as tier-B entries; the SDK departure accepted).
- **Deviations from spec:**
  - `src/develop.rs`: `scratch_len` under the mix now also skips an optional first warp whose `N` is not 1 (one shared `develop_warp` helper), so the scratch form agrees with the develop. Without it `develop_with_scratch_into` would refuse a sized-by-`scratch_len` buffer on such a file. Unchanged for every held file and every fixture without an optional multi-plane warp. Caught by mutant (s1).
  - `tests/developed_mix.rs`: `ac6_the_mix_never_parses_opcode_list_1` renamed to `ac6_the_mix_refuses_a_malformed_fix_bad_pixels_and_scratch_len_ignores_it`; its last two mix-oracle assertions were removed (the mix now refuses that file). `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels` renamed to `ac6_the_mix_applies_the_first_fix_bad_pixels` as the spec says.
  - `tests/support/developed_mix.rs`: a new `two_mandatory_fix_bad_pixels` builder for fault F2 (the old builder kept for the applied case).
  - Tests added beyond the Failing Tests list: `sensor_geometry_passes_the_first_warp_through` (src/opcode.rs unit) and `ac1_the_fix_runs_under_a_black_pattern_larger_than_two_by_two` (after mutant s4 survived). `colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry` gained an N = 3 case.
  - AC7 also asserts the f32 rows form, `mono_mix_plane_rows_into`, and the three plain-mono `develop_into` digests from the Implementation Context.
  - The N = 3 seed is generated by `examples/fuzz-seeds.rs` (synthetic coefficients), as SPEC-097 did.
  - `DevelopPath::Demosaic` / `ColourTransform` doc wording changed (they no longer share Colour's opcodes); no behaviour change.
  - The Q2, Q3 and SL3 `make` is `LEICA CAMERA AG`, each file's own Make tag (the M10/M11 entries say `Leica Camera AG` because their files do). Nothing reads `make` for a comparison.
- **Count and partition sites updated (audited by grep, not only from the spec):** `tests/corpus_manifest.rs` (`MANIFEST_ENTRIES` 14 to 17 and its section comment; the preview total 28 to 37 and its doc), `tests/previews.rs` (`PINNED_PREVIEWS` 28 to 37), `tests/plane_oracle.rs` (`DECODABLE` 6 to 7 with `LEICA_Q2_COLOUR`; new `LEICA_OPCODE`; `DECODABLE_COMPRESSED` 7 to 9), `tests/ifd_reader.rs` (`CFA_CHECKED_ELSEWHERE` 7 to 10), `tests/metadata_oracle.rs` (`CFA_METADATA_NOT_YET_ORACLED` 5 to 8), and the manifest header (28 to 37). Every other manifest walk (`plane_validate.rs`, `examples/corpus-status.rs`, `metadata_oracle.rs` loops) has no count. `ifd_reader.rs` `EXPECTED_FILES` (7, mono only) does not move.
- **Follow-up work identified:**
  - SPEC-099 (already an outline): the warp on the developed outputs; onyx's answer on the N = 3 warp is still open.
  - [XS] a public CFA bad-pixel applier, so the demosaic and colour-transform primitives can accept list 1.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — "scratch_len is unchanged" read as absolute, but the optional N ≠ 1 skip needs `scratch_len` to skip it too, or the scratch form refuses. Settled by sharing one helper.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Mutant coverage of the second normaliser: the design's mutants all hit the 2×2-slot path, and no tier-A fixture reached the black-pattern path with the fix. Worth a line in the template: "every read site needs its own fixture".

3. **If you did this task again, what would you do differently?**
   — Run the full suite only after the manifest and count sites are written. My first run read the manifest mid-edit and reported partition failures that were not real.
