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
  id: HANDOFF-209
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 24643404           # REAL combined count — what cost-audit reads
  estimated_usd: 16.62             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 150
  branch: claude/spec-098-verify
  pr: null
  completed_at: 2026-11-27         # YYYY-MM-DD
  notes: "Floor, measured live from this session transcript (e804d70e), 114 deduplicated messages: input 242, cache creation 261,088, cache read 24,307,802, output 74,272; no subagents. USD at 5/25 per M with 1-hour cache writes. The commit, PR and CI wait after the measurement are not counted."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-27
---

# HANDOFF-209: Mandatory FixBadPixelsConstant on the colour entries and WarpRectilinear passed through at sensor geometry, the Q2, Q3 and SL3 files

## Delegation Summary

`claude-opus-5` (acting as architect, the PROJ-006 orchestrator) hands `SPEC-098` to a
cold `claude-opus-5-5` session (acting as verifier) for the **`verify`** cycle of the
build in PR 367 (`feat/spec-098-colour-fix-bad-pixels`, build commit `2006ceb`,
HANDOFF-208's handback).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md` (Design Notes: onyx's two rules)
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-098-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md`, judged as designed on `origin/main` (PR 366), not the build's edits.
- **Build handback:** `HANDOFF-208` (mutant table, deviations).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-086`: the kernel, where it runs, `BayerPhase` unused, `DevelopPath::SensorGeometry`, the warp rules, the error order; with the maintainer's 2026-11-27 note.
- `DEC-084` item 3 and its amendment note: the developed mix's opcodes.
- `DEC-053`, `DEC-085`: the required-processing check and the paths.
- `DEC-040`: CFA colour from the `ActiveArea` origin.

### Constraints that apply

- `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`, `test-before-implementation`, `provenance-recorded-per-algorithm`.

### Prior related work

- `HANDOFF-208`, PR 367 (build); PR 366 (design); PR 363 (SPEC-097 verify, the same Leica files).

## Expected Deliverables

### The verify contract

You are **not** the agent that implemented this. Review it cold.

- Every AC with evidence; re-run every gate; compare AC7's digests, AC8's counts and the preview pins with the spec by script.
- Nothing that worked on `origin/main` is refused or changed: a corpus sweep of every entry on main and on the PR, diffed.
- Rule on the `scratch_len` deviation and on the `ac6_the_mix_never_parses_opcode_list_1` change.
- Re-run a sample of the build's mutants, add the verifier's own.
- Licence: no `-PB` bytes or derived seed in the diff; `just test` with an empty corpus.
- Tests and docs only on the verify branch; any `src/` change goes to a punch list.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Any `src/` change (punch list only); the developed-output warp (SPEC-099); onyx's N = 3 question.

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

- **Branch / PR:** `claude/spec-098-verify`, stacked on PR 367 (`feat/spec-098-colour-fix-bad-pixels`); the PR is linked in its own description. Tests and docs only.
- **Completed at:** 2026-11-27
- **All acceptance criteria met?** yes.
- **For `verify`:** ✅ APPROVED at `2006ceb` (the code, unchanged). Four oracle gaps, none in the code, each closed by a test on this branch.

**Evidence by AC** (judged against the spec on `origin/main`, PR 366):
- AC1: the 20 + 21 SDK literals in `tests/colour_bad_pixels.rs` equal the spec's text, by script (0 differences). An independent model of DEC-086 item 3, written outside the crate, reproduces all 41. The test writes the expected mosaic from the literals, never from a kernel.
- AC2 to AC5: green; mutants (c), (m), (i), (f), (j), (l), (q), (s4), V7a/b and V13 red on them.
- AC6: `ac6_every_path_applies_its_table`, plus the mix and opcode unit tests; mutants (n), (o), (p), (r), (s1), (s5), V8, V9a/b and V10 red.
- AC7: the 12 digests in `tests/leica_colour.rs` (three plane, three f32 plane, three mixed plane, three mono `develop_into`) equal the spec's table by script; 3/3 files ran. My own sweep binary, built separately at 2006ceb, gives the same nine plane digests.
- AC8: `ac8_the_sl3_bad_columns_are_fixed` ran green with the SL3 present (27,651 / 10,615 / 17,036 / 22,761 / 9523, as the spec's literals in the test).
- AC9: `DEVELOP_VERSION` "8", `MONO_MIX_VERSION` "1"; `just semver-check`: no semver update required; the corpus sweep below.
- AC10: `just fuzz-opcode 60` from a clean 2006ceb checkout: 2,608,755 runs in 61 s, 18 seeds (the N = 3 seed among them), no crash. The seed is synthetic (three identical made-up coefficient sets).
- AC11: `plane_md5_matches_the_pinned_raw_checksum`, `corpus_previews_match_exiftool_pins` (37) and the Compression 7 red-proof green with the corpus; the preview pins are the spec's.

**Gates (re-run here):** `just test` with the corpus (17/17 present): 1008 passed, 0 failed, 1 ignored, as the build. With an empty corpus folder: 1014 passed, 0 failed (the 6 are this verify's tests); every tier-B file skips by name, the Q2, Q3, SL3 and M11 included. `fmt-check`, `lint-ci`, `lint-fuzz`, `doc-check`, `wasm-check`, `validate`, `cost-audit`, `semver-check`: all pass (fmt and lint re-run after this branch's tests).

**Nothing that worked before is refused or changed.** A sweep binary (not committed) ran every public entry on all 17 corpus files under 70 develop option sets (both framings, orientation on and off, alpha 1, 0.5 and 0, both black modes, no mix and two weight sets), plus `scratch_len` and the scratch form, built once on `origin/main` and once on 2006ceb: 3,377 results each.
- 0 results that were `Ok` on main are refused on the PR, and 0 developed outputs change: no digest moved on any file, and no non-mix entry or `develop_into` changed at all.
- The only `Ok` to `Ok` change: `scratch_len` with the mix on the Q2, Q3 and SL3 (94,527,488 to 4,439,448 and 120,992,768 to 5,025,472), files whose mix is refused on both sides. That is the deviation below.
- Refused before and after, with a new error: the Q2, Q3 and SL3 developed outputs `{1,4}` to `{3,1}` (designed); the three Q2 Monochrom files' colour, plane and mix entries `{1,4}` to `NotAColorFilterArray` (list 1 now passes, so the CFA check is first; a sentence added to the CHANGELOG).
- Refused before, `Ok` now: the sensor-geometry entries on the Q2, Q3 and SL3 (the spec's goal).

**The ac6 fixture.** `malformed_optional_fix_bad_pixels()` is a synthetic list 1 holding one optional (flags 1) `FixBadPixelsConstant` with a 7-byte parameter block, on fixture (i) (22×18, colour). No real file has one. On main the mix skipped it unread; DEC-086 item 7 and the spec's moved-assertion list make the mix refuse it (`MalformedOpcodeParams { id: 4 }`), so the two dropped assertions (mix equals oracle, mix is Ok) cannot hold. The renamed test asserts more: both mix entries refuse before any write, and `scratch_len` still ignores list 1. **Ruling: accepted**, a designed change and not a regression.

### Mutants

- The build's 24, re-run as written (a)-(r), (s1)-(s5): all red.
- My own 17, all red at the end:
  - V1: green fills from the other colour (axial one step) — the 6 kernel tests.
  - V2: five neighbours; V3: eight same-colour neighbours — the 6 kernel tests.
  - V5: reads already-fixed values (reverse raster) — the 6 kernel tests.
  - V6a: the colour column from the plane origin — ac1 (0,1), ac2.
  - V8: sensor geometry asks the colour path (refuses the warp) — 4 tests.
  - V9a: the mix accepts a mandatory N = 3 warp — 4 tests. V9b: the mix applies an N = 3 warp — 2 tests. V10: the developed colour accepts a mandatory warp — 3 tests.
  - V12: blue counted as green — 8 tests. V13: the mix strips get no fix — 10 tests. V14: a malformed FBPC swallowed on the mix only — 2 tests.
  - **Survived at first**, each an oracle gap, each closed by a new test and then red:
    - V4 (round half down): no AC1 site is a tie (a linear ramp makes every same-colour sum divide cleanly). Fixed by `ac1_a_tie_rounds_half_up` (two hand-worked ties, n = 4 and n = 2).
    - V6b (the colour row from the plane origin): every fixture has `ActiveArea` top 0. Fixed by `ac1_colour_rows_are_indexed_from_the_active_area_top` (top 1; the literals come from the verifier's independent model, and 14 of 17 equal the SDK's left-1 values).
    - V11 (the candidate test reaching one past the `ActiveArea` bottom and right): no fixture's `ActiveArea` ends inside the plane. Fixed by `ac1_the_active_area_bottom_and_right_are_exclusive`.
    - V7a/V7b (the fix, or the candidate test, after the black-pattern normaliser's levels): the pattern scene's blacks are 0, where normalising is the identity. Fixed by `ac4_the_fix_reads_raw_samples_under_a_black_pattern`.
- **Shared code (FU-10 shape):** SH1 (`normalize_range` rounds with a third) and SH2 (demosaic `rnd4` rounds half down), over the full suite with no corpus. The kernel tests' relative comparison stays green, as expected. Independent pins catch both: SH1 20 tests (the calibrated colour references, a hand-computed per-position block); SH2 14 tests (the committed plane, mono-mix and developed-mix hashes, the hand-computed Adams-Hamilton literals).
- Added for coverage without a mutant: `ac5_the_mix_strips_see_the_fix_across_a_strip_edge` (the 1100-row fixture; AC5's 20-row scene fits in one 512-row strip) and `ac6_the_mix_sizes_an_optional_multi_plane_warp_as_it_develops`.

### Rulings on the deviations (each checked against the diff's file list)

- **`scratch_len` under the mix skips an optional N ≠ 1 warp:** accepted. `develop_core` drops that warp (DEC-086 item 6), and `develop_with_scratch_into` requires exactly `scratch_len` samples, so an unchanged `scratch_len` would size a warp buffer the develop never uses and the scratch form would refuse the file. Mutant (s1) shows this. DEC-086 item 7 only says `scratch_len` does not parse list 1, which still holds. The spec's "scratch_len is unchanged" was written about the fix, which adds no scratch. Tested on a synthetic optional N = 3 warp: with the mix, sized as no warp (572 against 852 for N = 1) and the scratch form equals the allocating form and the no-warp mix, under both framings; without the mix, sized and applied as N = 1, scratch form equal. Nothing changes for any file that develops (sweep).
- **ac6 rename and the two dropped assertions:** accepted (above).
- **`two_mandatory_fix_bad_pixels` for F2:** accepted; the spec pre-approves F2 becoming two mandatory FBPCs.
- **Extra tests; AC7 asserting more; `colour_mandatory_opcode_in_list_3` gaining N = 3:** accepted.
- **The seed from `examples/fuzz-seeds.rs`:** accepted, synthetic.
- **`Demosaic`/`ColourTransform` doc wording:** accepted, no behaviour change.
- **Make `LEICA CAMERA AG`:** accepted; it is the spec's spelling and the files' tag.
- File list: every file in the diff is either in the spec's Outputs or in a declared deviation (`examples/fuzz-seeds.rs`). The spec's `tests/develop_scratch_alloc.rs` needed no edit, since it follows F2 through the support builder. HANDOFF-208's `pr:` field is null; PR 367 is the build PR.

### Licence

- No DNG bytes in the diff. The only binary file is the synthetic N = 3 seed.
- The Q3 and SL3 appear only as manifest metadata (path, size, sha256, licence) and FNV regression digests of outputs, as the M11 does, per the maintainer's 2026-11-27 ruling.
- No seed or fixture is derived from them. Every tier-B test skips with an empty corpus.

### Findings

- FU-1 to FU-4 (closed on this branch): the oracle gaps V4, V6b, V11 and V7, above.
- FU-5 (closed on this branch): the CHANGELOG now says a no-CFA file with a mandatory FBPC moves from `{1,4}` to `NotAColorFilterArray` on the colour entries.
- FU-6 (for the ship): DEC-086 item 7 could say in one line that `scratch_len` under the mix sizes an optional N ≠ 1 warp as skipped, since the spec's wording reads the other way.
- FU-7 (informational, pre-existing): `scratch_len` gives a size for a file whose develop is refused at required processing, because it does not run that check (on main too).
- No ship-blockers.

### Cost self-report

- **Tokens (total):** 24,643,404 (floor, live)
- **Estimated USD:** 16.62
- **Duration (minutes):** about 150
- **Source of the number:** this session's transcript, deduplicated by message id per docs/cost-tracking.md

### Drift and new artifacts

- **New decisions emitted:** none. `just decisions-audit --changed` flags DEC-083, DEC-084 and DEC-086 for `src/color.rs`; the diff is consistent with each (the mixer row is untouched; DEC-084 item 3 is amended by DEC-086).
- **Deviations from spec:** none on this branch (tests, the CHANGELOG sentence, this handoff, the timeline).
- **Follow-up work identified:** FU-6 above; SPEC-099 as planned.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only "scratch_len is unchanged", which reads as absolute but was meant for the fix. The rule-clause map named the half-up clause but tested it only against truncation, so it looked covered when no tie existed.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. A design note would help: an SDK-derived scene inherits the SDK run's geometry (top 0, the `ActiveArea` ending at the plane edge, blacks 0), so every boundary clause needs a scene of its own, not only one per read site.

3. **If you did this task again, what would you do differently?**
   — Build the main and PR sweep binaries before anything else, since the 60 MP files make the sweep the longest job; and run mutants in the optimised profile from the start.
