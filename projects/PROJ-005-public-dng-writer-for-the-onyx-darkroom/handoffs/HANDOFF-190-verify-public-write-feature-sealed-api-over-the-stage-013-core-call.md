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
  id: HANDOFF-190
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-24
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-061

project:
  id: PROJ-005
  stage: STAGE-017
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
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10145002           # REAL combined count — what cost-audit reads
  estimated_usd: 7.58              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-061-verify-base
  pr: https://github.com/jysf/irradiance/pull/329
  completed_at: 2026-11-24         # YYYY-MM-DD
  notes: "floor, measured before the handback commit from transcript 185cfade-2523-4d39-9826-ef1001c04347.jsonl (the scratchpad UUID): 57 unique message.ids, all claude-opus-5-5, no subagents, deduplicated by message.id with each message's largest output_tokens. USD at $5/M input, $25/M output, cache read 0.1x, 1-hour cache writes 2x. Verdict punch-list: one required item, P1."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-190: VERIFY: public `write` feature, sealed API over the STAGE-013 core (SPEC-061)

## Delegation Summary

You are the **verifier** for SPEC-061. Review it cold: you did not build it.

- **Under review:** PR 329 (`claude/spec-061-build`, head `fa1e539`), built under
  HANDOFF-175 by a separate session.
- **Your branch:** `claude/spec-061-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-061 verify`.

SPEC-061 adds `irradiance::dng::write` behind the opt-in `write` feature: a mono u16 plane
plus caller metadata in, one little-endian uncompressed single-strip DNG out, through
`written_len`, `write_into` and `write_to_vec`. Framing (FullSensor, FullActiveArea,
StandardCrop) is expressed as tags and the samples are only ever cut at `ActiveArea`
(DEC-073). Two `Error` variants (`InvalidWriteInput`, `UnsupportedOpcodeForFraming`) exist
in every build. No reader change, no new dependency, no I/O in `src/dng`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call.md`.
  Judge the ACs against the spec **as designed** (`git show origin/main:<spec path>`):
  AC1 to AC12, Design §1 to §9, the oracle list in §7 and the probe table in
  `## Implementation Context`.
- **Stage and brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`,
  `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`.
- **Decisions:** DEC-071 (v1 scope), DEC-072 (EXIF offset slot only; the block is
  SPEC-081), DEC-073 (framing as tags, cut only at ActiveArea), DEC-075 (layout,
  validation, the `OpcodeList1` allow-list, two `Error` variants, no allocation),
  DEC-080 (new, proposed, from the build: payload types compiled in every build, named
  under `dng::write`), DEC-055, DEC-050, DEC-016, DEC-051, DEC-052, DEC-058, DEC-069 §6,
  and DEC-077 (the reader's DNG-version guard the written files must pass).
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`,
  `library-not-application`.
- **Build handback:** HANDOFF-175's `## Handback` (in the same handoffs dir): the
  staleness check, the gates, the dng_validate table, five deviations and two follow-ups.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`. Run `just test`, never
  `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Use `/usr/bin/git` to establish branch and HEAD. Give each mutant run its own source
  state and read the panic message.

## What to check hard

1. **Every AC met against the spec as designed**, and every §4 tag row and §5 refusal
   reaching a test that fails when it is wrong.
2. **Oracle 8 deviation (FullSensor render).** The build compared the FullSensor render to
   the FullActiveArea output instead of to the source, because §3 writes FullSensor's
   DefaultCrop as the whole active area. Decide: is the build's reading of §3 right, so
   that the spec's oracle 8 text is the error? Or should FullSensor carry the caller's
   crop, which is a §3/DEC-073 change? Rule on it in the verdict. Do not change `src/`.
3. **DEC-080.** Does it resolve a real contradiction in the spec ("variants in every
   build" against payload types under a feature-gated module)? Is the public path
   (`irradiance::dng::write::WriteInputError`, `Framing`) the same with and without
   `write`? Check `cargo semver-checks` and `cargo doc` in both builds. Does DEC-080 need
   accepting before ship?
4. **FixBadPixelsConstant at the active-area edge** (build follow-up 1). Confirm or refute:
   after a cut, a bad pixel on the active-area edge loses its margin neighbours, so the
   cut file can develop differently there. If it is real, is DEC-075 §5's claim that
   opcode id 4 is position-independent wrong as written? Build a tier-A fixture if you
   can. Put the ruling (document it in DEC-075, refuse, or flag) in the punch list or the
   follow-ups.
5. **Red-proofs (AC7).** Oracles 3 and 5 after their negative controls. Run the new test
   files against `origin/main`'s `src/` as well. Mutate the writer at least: drop the
   `ActiveArea` rebase on a cut, write `DefaultCropOrigin` unrebased, skip the
   `OpcodeList1` allow-list, swap the strip byte order, off-by-one in `written_len`.
   Each must fail a named test. Give a table.
6. **No panics and no allocation (AC10).** Offset and size arithmetic on caller input,
   `max_pixels` checked before any multiplication, `write_into` allocation-free
   (`tests/dng_write_alloc.rs`). Read the fuzz target. Is it reaching the refusal paths,
   or only `Ok`?
7. **The other deviations:** the tuple-variant change to `tests/error_kind.rs` (does it
   still catch a missing `kind()` arm?), the self dev-dependency enabling `write`, the two
   unlisted support files, and the 4 GiB lazily mapped `OpcodeList2` for `OutputTooLarge`
   (does it stay cheap on the CI runners, Linux and macOS arm64?).
8. **`white_level` on 12/14-bit sources** (build follow-up 2). Is it documented in the
   rustdoc? If not, a docs fix on your branch is in scope.
9. **Gates:** `just test` with corpus and dng_validate set, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`, `msrv`, `validate`,
   `gate-audit`, `cost-audit`, `just decisions-audit --changed`, `cargo semver-checks`,
   60 s of `just fuzz-dng-write`, and `gh pr checks 329`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and a ruling on items 2,
  3 and 4.
- **Commit** this handoff's filled handback (and any tests or docs fixes) to
  `claude/spec-061-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-061 ship --verdict approved` and fast-forward
  `claude/spec-061-build` to the verify tip so PR 329 carries the verify commits. Do not
  merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose. The repo ruleset rejects code tokens.
- **IDs:** a separate session is working PROJ-006 and also mints IDs. Before minting a DEC
  or HANDOFF, compute the next free one from `origin/main` plus open PRs.

## Out of Scope

- SPEC-081 (EXIF block, XMP, descriptive tags, DEC-074 rendering hints) and SPEC-082
  (preview).
- Compression, float, CFA, tiles, big-endian, several strips.
- Any reader change (`src/ifd.rs`, `src/plane.rs`, `src/develop.rs`). If a round trip
  exposes a reader bug, report it.
- Migrating `tests/support/dng_writer.rs` onto the public writer.
- Activating PROJ-005 (brief status is the maintainer's call).

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

### Execution notes

- **Branch / PR:** `claude/spec-061-verify-base` (PR 329's head `fa1e539` + this handoff + the verify commit). PR: https://github.com/jysf/irradiance/pull/329
- **Completed at:** 2026-11-24
- **All acceptance criteria met?** AC1 to AC11 yes, with oracle 8 adapted as ruled below. **AC12 no:** CI's `rust / fuzz lint red-proof (must fail red)` job is red on PR 329, and `just lint-fuzz-red-proof` reproduces it locally (punch list P1).
- **For `verify`:** ⚠ **PUNCH LIST** (one required item, P1). The writer itself is sound; nothing in `src/` needs to change.

#### Gates, re-run at verify (env: `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` set)

| Gate | Result |
|---|---|
| `just test` (build's tree, before verify edits) | 900 passed, 0 failed, 55 binaries; SKIP lines only for the unset SDK stage-2 and colour dirs |
| `tests/dng_write.rs` after the verify additions | 18 passed (was 16 test fns + 2 added; the allow-list test gained a case) |
| `fmt-check`, `lint-ci`, `lint-no-allow`, `doc-check` (2 surfaces, 0 warnings), `deny`, `deny-fuzz`, `lint-fuzz`, `msrv`, `validate`, `gate-audit`, `cost-audit` | all exit 0 |
| `just decisions-audit --changed` | exit 0 (advisory list only) |
| `cargo semver-checks --all-features` and default features | 202 pass, no semver update required |
| `just fuzz-dng-write 60` | 4,433,044 runs in 61 s, no crash |
| `just lint-fuzz-red-proof` | **FAILS**: control run cannot find `fuzz_targets/../../tests/support/dng_write_fuzz.rs` in the temp copy |
| `gh pr checks 329` | 21 pass, **1 fail**: `rust / fuzz lint red-proof (must fail red)`, same cause |

#### What to check hard, item by item

1. **ACs against the spec as designed** (`origin/main`). AC1: `cargo doc`/check with no features, default, `--all-features` all build, `write = []` adds no dependency. AC2 to AC5 and AC10: `round_trip_every_framing_and_geometry` (12 cases), `tags_match_the_spec_table` (exact tag, type, count and bytes for all 25 tags, even offsets, left-justified inline SHORT), `backward_version_follows_opcode_lists`, `orientation_is_always_written`, `every_refusal_is_typed` (29 cases, three entry points, `kind()`), `size_query_matches_write`, determinism, `dng_write_alloc`. Every §4 row and §5 refusal reaches a test that a mutant turned red (table below). AC6/AC8: tier B exact on all four held files.
2. **Oracle 8 ruling: the build is right; the spec's oracle-8 text is the error.** §3's table, DEC-073's table ("Full sensor … the whole active area") and oracle 4 (FullSensor develops like the source's FullActiveArea develop) all agree that FullSensor's DefaultCrop is the whole active area. Only oracle 8's prose (and the design probe, which wrote the source crop at FullSensor) says otherwise. No §3/DEC-073 change. To close the gap the build's adaptation left (its FullSensor reference was only another output), verify added a source anchor: the FullActiveArea render's default-crop window must equal the source's render. It does, sample for sample, so FullSensor ≡ FullActiveArea ≡ source-in-window. The spec's oracle-8 sentence should be corrected at ship.
3. **DEC-080 ruling: sound; accept it at ship.** The contradiction is real: the Outputs line puts both variants in every build while §2 names their payload types under a module AC1 compiles only with `write`. A variant's field type must exist whenever the variant does. With `write` the public paths are exactly `irradiance::dng::write::{Framing, WriteInputError}`; without it they have no public path (unnameable, never produced). `doc-check` (both surfaces) and semver-checks (default and all features) pass; `Error` is `#[non_exhaustive]`, so the variants are additive. One nit: `--document-private-items` in a default build reports 3 broken intra-doc links (`src/lib.rs` on `mod write_types`, and `src/write_types.rs`'s module doc), because they point at `crate::dng`. No gate runs that mode (doc-check deliberately doesn't), so it is a follow-up, not a blocker.
4. **FixBadPixelsConstant at the active-area edge: confirmed real.** `apply_fix_bad_pixels_constant` takes neighbours from the whole plane extent (`src/develop.rs:865`, SPEC-017 rule 1). Verify's tier-A fixture `fix_bad_pixels_constant_on_the_active_area_edge_develops_differently_after_a_cut` puts a bad pixel on the active area's corner with a bright margin: the FullSensor and FullActiveArea files develop identically except at that one pixel (source median takes the margin, the cut file's cannot). A bad pixel one step inside matches exactly (negative control). This is intrinsic to the format, not irradiance: OpcodeList1 runs on the stored plane in any reader, so removing the margin removes neighbours. DEC-075 §5 is right that id 4 names no coordinates, but "a cut cannot move them" overstates: its effect at the edge can change. **Ruling: document it in DEC-075 §5 as a known limit (follow-up F1), do not refuse.** A refusal would have to scan sample values for the constant on the edge, which the writer's header-only walk is designed not to do, and the held files are unaffected (oracle 4 exact). onyx should hear about it with the DEC note.
5. **Red-proofs (AC7).** Oracle 3 (negative control, then a flipped sample byte and a moved `DefaultCropOrigin`, each reported) and oracle 5 (the crop-cut, warp-verbatim Q2M file develops differently; dng_validate: 132,753,649 of 140,180,736 render samples differ) both pass. Against `origin/main`'s `src/`, all four new or changed test files fail to compile (`cannot find dng in irradiance`, missing variants): red. Mutant table below.
6. **No panics, no allocation (AC10).** `plan` checks `max_pixels` first in `u64` with a saturating product, before any other arithmetic; every later size, offset and count is `checked_*` or `try_from` into `OutputTooLarge`; `lint-no-allow` passes with the five deny lints. `written_len_and_write_into_allocate_nothing` passes in debug (it is in `just test`'s debug line). The fuzz target reaches both sides: its `Err` branch compares `written_len` with `write_into` by `Debug`, and `fuzz_seeds_reach_their_named_outcome` pins 14 `refuse-*` seeds that are refused and 8 that succeed. `OutputTooLarge` is not reachable from fuzz input (lists are at most 255 bytes); tier A covers it.
7. **Other deviations.** (a) `tests/error_kind.rs` tuple-variant parsing: correct; `kind()` is wildcard-free, so a missing arm is a compile error, and the parser change makes `InvalidWriteInput` count in the 78. (b) Self dev-dependency enabling `write`: fine, path-only and stripped at package time. (c) The two unlisted support files: fine, but `tests/support/dng_write_fuzz.rs` is exactly what breaks P1. (d) 4 GiB lazily mapped `OpcodeList2`: zero-filled allocation is lazily mapped on macOS and Linux; CI `rust / test` and both determinism jobs pass, and locally the whole file runs in under 0.01 s.
8. **`white_level` on 12/14-bit sources: not documented. Fixed on this branch** (rustdoc on `WriteOptions::white_level`: the written file is 16-bit, so pass `2^bits − 1` when the source has no `WhiteLevel`).
9. **Gates:** table above.

#### Mutant table (each applied alone to `src/dng/write.rs`, then restored; `cargo test --all-features --no-fail-fast --test dng_write --test dng_write_alloc`)

| # | Mutant | Fails (named tests) |
|---|---|---|
| M1 | `ActiveArea` tag kept unrebased on a cut | `cutting_framings_omit_active_and_masked_areas`, `round_trip_every_framing_and_geometry`, `develop_equivalence_synthetic`, `red_proof_round_trip_…`, `fix_bad_pixels_…_edge_…` |
| M2 | `DefaultCropOrigin` written in stored-plane coordinates | `round_trip_every_framing_and_geometry`, `develop_equivalence_synthetic`, `red_proof_round_trip_…` |
| M3 | `OpcodeList1` allow-list skipped | `opcode_list_1_allow_list_under_cutting_framings`, `every_refusal_is_typed`, `checks_run_in_the_documented_order`, `fuzz_seeds_reach_their_named_outcome` |
| M4 | strip byte order swapped | `round_trip_every_framing_and_geometry`, `round_trip_black_level_pattern`, `develop_equivalence_synthetic`, `write_is_deterministic_…`, +3 |
| M5 | `written_len` off by one | `size_query_matches_write`, `every_refusal_is_typed`, `write_is_deterministic_…`, `written_len_and_write_into_allocate_nothing` |
| M6 | cutting framings keep the whole plane | `round_trip_every_framing_and_geometry`, `develop_equivalence_synthetic`, `write_is_deterministic_…`, +2 |
| M7 | `DNGBackwardVersion` ignores opcode lists | `backward_version_follows_opcode_lists`, `tags_match_the_spec_table` |
| M8 | masked-area overlap check dropped | `every_refusal_is_typed`, `fuzz_seeds_reach_their_named_outcome` |
| M9 | out-of-line values not word-aligned | `tags_match_the_spec_table` |
| M10 | budget refuses at `== max_pixels` | **survived the build's tests**; now `a_plane_of_exactly_max_pixels_is_within_the_budget` (verify-added) |
| M11 | list-1 walk accepts trailing bytes | **survived the build's tests**; now `opcode_list_1_allow_list_under_cutting_framings` (verify-added trailing-byte case) |
| M12 | StandardCrop drops the caller's crop | `round_trip_every_framing_and_geometry`, `develop_equivalence_synthetic`, `red_proof_round_trip_…` |
| M13 | blank `UniqueCameraModel` accepted | `every_refusal_is_typed` |
| M14 | `write_into` accepts a long `dst` | `size_query_matches_write` |

### Punch list

- **P1 (required, blocks ship):** `scripts/lint-fuzz-red-proof.sh` copies `src/`, `Cargo.toml` and `fuzz/fuzz_targets/` into its temp dir but not `tests/support/dng_write_fuzz.rs`, which `fuzz/fuzz_targets/dng_write.rs` includes by `#[path]`. Its control run then fails to compile, and CI's `rust / fuzz lint red-proof (must fail red)` is red on PR 329. Fix, verified locally at verify (the proof then goes green end to end): after the `fuzz_targets` copy, add `mkdir -p "${WORK_DIR}/tests/support"` and `cp "${REPO_ROOT}/tests/support/dng_write_fuzz.rs" "${WORK_DIR}/tests/support/"` with a one-line comment. Not done here: the branch rule is tests and docs only. Re-verify needs only `just lint-fuzz-red-proof` and `gh pr checks 329`.

### Cost self-report

- **Tokens (total):** 10,145,002 (a floor, taken before the handback commit)
- **Estimated USD:** 7.58 (claude-opus-5-5: $5/M input, $25/M output, cache read 0.1×, 1-hour cache writes 2×)
- **Duration (minutes):** about 60
- **Source of the number:** this session's transcript `185cfade-2523-4d39-9826-ef1001c04347.jsonl` (the scratchpad UUID), measured per docs/cost-tracking.md "Measuring a cycle": 57 unique message.ids, all claude-opus-5-5, no subagents (`basis: transcript`).

### Drift and new artifacts

- **New decisions emitted:** none. DEC-080 reviewed: accept at ship.
- **Deviations from spec:** none new. Verify added tests and one rustdoc paragraph only:
  - `tests/dng_write.rs`: `fix_bad_pixels_constant_on_the_active_area_edge_develops_differently_after_a_cut` (pins item 4), `a_plane_of_exactly_max_pixels_is_within_the_budget` (kills M10), a trailing-bytes case in `opcode_list_1_allow_list_under_cutting_framings` (kills M11).
  - `tests/dng_write_corpus.rs`: oracle 8 now also checks that the FullActiveArea render's default-crop window equals the source's render (upright files only).
  - `src/dng/write.rs`: rustdoc on `WriteOptions::white_level` (item 8). No code change.
- **Follow-up work identified:**
  - **F1:** amend DEC-075 §5 with the FixBadPixelsConstant edge limit (item 4) and tell onyx; if a later spec refuses or flags such input, rewrite the verify test that pins it.
  - **F2:** correct SPEC-061 §7 oracle 8's sentence at ship: FullSensor's render equals the FullActiveArea output's, and that render's default-crop window equals the source's.
  - **F3:** CI has no `fuzz smoke — dng_write (60s)` job, though every other fuzz target has one. AC11 asked only for the local run; add the job (with the P1 fix or after).
  - **F4 (nit):** the three intra-doc links to `crate::dng` from always-compiled private items break under `--document-private-items` without `write`. Plain text or a `cfg_attr` would fix it; no gate runs that mode today.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the handoff. Its gate list omitted `lint-fuzz-red-proof`, and that was the one gate that was red. Reading `gh pr checks` early rather than last would have found it sooner.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-011/DEC-036's fuzz-graph red-proof: a spec that makes a fuzz target `#[path]`-include a file outside `fuzz/` must also extend the red-proof's copy step. Worth a line in the fuzz section of AGENTS.md.

3. **If you did this task again, what would you do differently?**
   — Run CI status and the full red-proof set before the slow corpus suite, and run the mutants before reading the tests closely; the two survivors pointed straight at the missing boundary cases.
