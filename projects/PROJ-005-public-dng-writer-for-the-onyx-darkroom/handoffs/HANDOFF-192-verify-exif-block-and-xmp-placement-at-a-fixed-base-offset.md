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
  id: HANDOFF-192
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-24
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-081

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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 6732483            # REAL combined count — what cost-audit reads
  estimated_usd: 5.25              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 50
  branch: claude/spec-081-verify-base
  pr: 332
  completed_at: 2026-11-24         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript e1ed8aaa-0485-4bae-b1aa-4275adc0fa82.jsonl (41 unique message.ids, all claude-opus-5-5, no subagents): input 83 + cache-creation 150,236 (1-hour) + cache-read 6,563,461 + output 18,703, priced at 5 and 25 USD per million, 1-hour cache writes at 2x, as HANDOFF-191 did. Approved at 5d89f90 (the code under review; the verify commit adds only docs). 17 of 17 mutants killed."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-24
---

# HANDOFF-192: VERIFY: EXIF block, XMP, descriptive tags and rendering hints (SPEC-081)

## Delegation Summary

You are the **verifier** for SPEC-081. Review it cold: you did not build it.

- **Under review:** PR 332 (`claude/spec-081-build`, head `1ed1865`), built under
  HANDOFF-191 by a separate session. CI was 22 of 22 green at that head.
- **Your branch:** `claude/spec-081-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-081 verify`.

SPEC-081 extends the shipped writer (SPEC-061) with: a caller EXIF block copied verbatim at
file offset 8 with IFD0 after it, `ExifIFD`/`GPSInfo` as `8 + offset` (DEC-072);
`EXIF_BLOCK_BASE` and `exif_block_base`; `XMLPacket`; eight typed ASCII descriptive tags
(DEC-071 §3a); four rendering hints as `Rational` (DEC-074); every new check in `plan`, after
`UniqueCameraModel` and before the `OpcodeList1` walk (DEC-081 §4); five new
`WriteInputError` reasons; no new `Error` variant.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-081-exif-block-and-xmp-placement-at-a-fixed-base-offset.md`.
  Judge the ACs against the spec **as designed** (`git show f5c1b9c:<spec path>`): AC1 to
  AC10, Design §1 to §7, and the rule-clause map.
- **Decisions:** DEC-081 (proposed, the design's choices), DEC-072 with its amendment,
  DEC-071 §2 and §3a, DEC-074, DEC-075 §1 and §7, DEC-080; DEC-051, DEC-052, DEC-058,
  DEC-016, DEC-069 §6.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`,
  `library-not-application`.
- **Build handback:** HANDOFF-191's `## Handback` (same handoffs dir): the exiftool and
  `dng_validate` results and three deviations.
- **Prior verify:** HANDOFF-190 (SPEC-061). Its mutant table is a model; its punch list was a
  CI job missing from the handoff's gate list.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`. Run `just test`, never
  `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Use `/usr/bin/git` for branch and HEAD. Give each mutant run its own source state; plain
  cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC met against the spec as designed,** and every rule-clause-map row reaching a
   test that fails when the clause is wrong.
2. **Mutants.** At least: drop the pad byte (odd block), write the pointer as the in-block
   offset (no `+ 8`), write `GPSInfo` when only an Exif offset is given, accept an odd offset,
   accept `offset + 2 == len + 1`, skip `DateTime`'s form check, let a RATIONAL hint take a
   negative, write `BaselineExposure` as RATIONAL, treat `Some("")` as absent, accept an empty
   XMP, move one new check after the `OpcodeList1` walk, run one new check only in
   `serialize` (parity), and break `exif_block_base` to return 10. Each must fail a named
   test. Give a table.
3. **The three build deviations.** (a) `InvalidAscii` before `DateTimeInvalid` for a
   non-ASCII `DateTime`: is that DEC-081 §4's order, and is it documented in the module docs?
   (b) The exiftool oracles on a synthetic plane: does oracle 5 alone still prove the block
   on real files? (c) All 22 fuzz seeds regenerated: do they still reach their named
   outcomes, with the same cases as before?
4. **Red-proofs.** Oracle 4 (a base-10 block is caught, after the base-8 negative control,
   and the two blocks differ) and the hints half of oracle 5 (the K-3 render differs without
   `BaselineExposure`). Run the new tests against `f5c1b9c`'s `src/` too: they must fail.
5. **The test block builder** (`tests/support/exif_block.rs`): are its offsets really
   absolute for the given base, including the Interop pointer? A builder bug that writes
   block-relative offsets would make oracle 3 pass for the wrong reason only if exiftool
   tolerates it. Check one block by hand (hex) against the TIFF and Exif rules.
6. **No panics, no allocation.** The new arithmetic (block length, pad, pointer sums, XMP
   length) on caller input; `written_len` and `write_into` allocate nothing with every field
   set. Does the fuzz target reach every new refusal, or only `Ok`?
7. **Docs.** The module docs ("What is written", "Checks and their order") list every new tag
   and check in the order the code runs them; CHANGELOG, provenance row, AGENTS.md block.
8. **Gates:** `just test` with the corpus and `dng_validate` set, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`, **`lint-fuzz-red-proof`**,
   `msrv`, `validate`, `gate-audit`, `cost-audit`, `just decisions-audit --changed`,
   `cargo semver-checks`, 60 s of `just fuzz-dng-write`, and **`gh pr checks 332` first**.
9. **DEC-081.** Is each of its seven items sound and followed by the code? Should it be
   accepted at ship?

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and rulings on items 3 and 9.
- **Commit** this handoff's filled handback (and any tests or docs fixes) to
  `claude/spec-081-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-081 ship --verdict approved`, then `just handback-sync SPEC-081`
  (so CI's cost audit passes at ship), and fast-forward `claude/spec-081-build` to the
  verify tip so PR 332 carries the verify commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** a PROJ-006 session also mints IDs. Before minting a DEC or HANDOFF, compute the next
  free one from `origin/main` plus open PRs.

## Out of Scope

- Previews (SPEC-082); UTF-8 `LocalizedCameraModel`; `Copyright`'s editor part.
- The empty-opcode-list follow-up in DEC-081 (SPEC-061 behaviour).
- Any reader change. If a round trip exposes a reader bug, report it.
- Accepting DEC-081 (the orchestrator does it at ship).

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

*Filled in by the receiving agent (verifier, claude-opus-5-5), reviewed cold.*

### Execution notes

- **Branch / PR:** `claude/spec-081-verify-base`, verifying https://github.com/jysf/irradiance/pull/332
- **Completed at:** 2026-11-24
- **All acceptance criteria met?** yes. AC1 to AC10, judged against the spec as designed
  (`f5c1b9c`).
- **For `verify`:** ✅ **APPROVED at `5d89f90`** (PR 332's code at `1ed1865`, plus the verify
  handoff; this verify commit changes only docs).

**1. Acceptance criteria.** Every AC has its named test, each passing locally, and each fails
under at least one mutant below (so no AC passes vacuously).

| AC | Evidence |
|---|---|
| AC1 options and base | `exif_block_base_is_pinned_to_eight`: the constant is 8, the function returns 8 for the default options and for every field set, every new field and `ExifBlock::default()` start as `None`/empty. Killed M13, M17. |
| AC2 tags and layout | `every_new_tag_matches_the_spec_table` (all 15 rows: exact type, count, bytes; odd and even block and XMP; ascending; every out-of-line value at an even offset), `exif_block_sits_at_the_base_and_ifd0_follows_it` (6 shapes, header offset `8 + align2(len)`, block bytes verbatim, pad byte 0, pointers present only for the offsets given), `absent_options_write_no_tag_and_empty_strings_write_count_one`. Killed M01, M02, M03, M09, M14, M17. |
| AC3 round trip | `round_trip_holds_with_every_new_option_set`: three framings, odd block, odd XMP, hints read back through `Container::rationals`. |
| AC4 validation | `every_refusal_is_typed` (34 new cases through `written_len`, `write_into`, `write_to_vec`, with `kind()`; boundary `offset + 2 == len` passes), `checks_run_in_the_documented_order` (9 new steps, `write_into` agrees at each), `date_time_form_is_checked_not_the_calendar`, `rational_hints_refuse_zero_and_unsigned_negatives`. |
| AC5 exiftool parity | `exiftool_reads_the_block_and_the_new_tags`, exiftool 13.55: passes on all 6 block shapes. |
| AC6 red-proofs | Oracle 4 re-run: the base-8 negative control passes, the base-10 block differs from it, and exiftool gives 5 mismatches (`ExposureTime` 8.928571429, `FNumber` 0.0008105041336, `DateTimeOriginal` "26:11:24 09:30:15", `InteropIndex` absent, `GPSLatitude` 0 1 43.078925). Hints half: K-3 III without `BaselineExposure` differs in 76,681,557 of 76,681,728 samples (DEC-074's figure exactly). |
| AC7 dng_validate | 1.7.1: K-3 III (source tags 271, 272, 305, 306, 315, 33432 + four hints) and Q2M (+ 50709, 50735) with the block and XMP: no warnings; K-3 III `-16 -tif` render equals the source's (6192 x 4128, 0 differ). |
| AC8 determinism, allocation | `write_is_deterministic_whatever_the_buffer_held` (odd block, odd XMP, every option), `tests/dng_write_alloc.rs` (every metadata option set, odd 13-byte block): pass. |
| AC9 fuzz | `fuzz_seeds_reach_their_named_outcome` (28 seeds, each new refusal seed names its reason); `just fuzz-dng-write` 3,945,083 runs in 61 s, no crash; `lint-fuzz`, `lint-fuzz-red-proof`, `deny-fuzz` green. |
| AC10 docs and gates | See items 7 and 8. |

**Red-proof against the design's `src/`.** With `f5c1b9c`'s `src/` and this branch's tests, the
three writer test binaries do not compile (unresolved `ExifBlock`, `EXIF_BLOCK_BASE`,
`exif_block_base`; no variants `DateTimeInvalid`, `RationalInvalid`, `ExifBlockWithoutIfd`,
`ExifIfdOffsetInvalid`, `EmptyXmp`; no field `artist` and so on). The tests cannot pass
without the feature; behavioural teeth come from the mutants.

**2. Mutants.** Each in its own worktree copy of `src/dng/write.rs` (reset between runs), running
`cargo test --all-features --test dng_write --test dng_write_corpus --test dng_write_alloc
--no-fail-fast` with exiftool on PATH (corpus and `dng_validate` unset for speed). 17 of 17
killed.

| # | Mutant | Fails |
|---|---|---|
| M01 | drop the pad byte (IFD0 at `8 + len`) | `exif_block_sits_at_the_base_and_ifd0_follows_it` |
| M02 | pointer as the in-block offset (no `+ 8`) | `every_new_tag_matches_the_spec_table`, `exif_block_sits_…`, `exiftool_reads_the_block_and_the_new_tags`, `red_proof_a_block_built_for_another_base_is_caught` |
| M03 | write `GPSInfo` from the Exif offset when only that is given | `exif_block_sits_…` |
| M04 | accept an odd offset | `every_refusal_is_typed`, `checks_run_in_the_documented_order` |
| M05 | accept `offset + 2 == len + 1` | `every_refusal_is_typed` |
| M06 | skip `DateTime`'s form check | `date_time_form_is_checked_not_the_calendar`, `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_reach_their_named_outcome` |
| M07 | a RATIONAL hint takes a negative numerator (`as u32`) | `rational_hints_refuse_zero_and_unsigned_negatives`, `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…` |
| M08 | `BaselineExposure` written as RATIONAL | `every_new_tag_matches_…`, `round_trip_holds_with_every_new_option_set`, `exiftool_reads_…`, `red_proof_…` (its control) |
| M09 | `Some("")` treated as absent | `absent_options_write_no_tag_and_empty_strings_write_count_one` |
| M10 | accept an empty XMP | `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…` |
| M11 | the XMP check moved after the `OpcodeList1` walk | `checks_run_in_the_documented_order` |
| M12 | the XMP check only in `serialize` (parity broken) | `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…` |
| M13 | `exif_block_base` returns 10 | `exif_block_base_is_pinned_to_eight` |
| M14 | the block not copied | `exif_block_sits_…`, `exiftool_reads_…`, `red_proof_…` |
| M15 | `BaselineExposure` accepts a zero denominator | `rational_hints_…`, `every_refusal_is_typed`, `checks_run_…` |
| M16 | the EXIF block check moved before the hints | `checks_run_in_the_documented_order` |
| M17 | `EXIF_BLOCK_BASE` = 10 | `exif_block_base_is_pinned_to_eight`, `every_new_tag_matches_…`, `exif_block_sits_…`, `red_proof_…`, `tags_match_the_spec_table` |

**3. Rulings on the three build deviations.**

- (a) **`InvalidAscii` before `DateTimeInvalid`: accepted.** DEC-081 §4 says "the descriptive
  strings in tag order (`DateTime`'s form with its own)", and the spec says the form is checked
  "also". Checking the character class first, then the form, per field, is the natural reading.
  It is documented in the module docs ("printable ASCII of at most 1024 bytes, then `DateTime`'s
  form") and pinned (`every_refusal_is_typed` expects `InvalidAscii { field: "DateTime" }` for
  `caf\u{e9}` and a tab). At ship, add a clause to DEC-081 §4 so the order is stated there too.
- (b) **Oracles 3 and 4 on a synthetic plane: accepted.** Where the block goes and what the
  pointers hold do not depend on the plane: only the strip moves. Tier A pins the block bytes,
  the pad and the pointers on all 6 shapes. Oracle 5 places the same oracle-3 block and XMP in
  the K-3 III and Q2M, and `dng_validate` accepts both. On real files oracle 5 proves the files
  are valid, not exiftool parity; the parity is plane-independent, so this is enough. Running
  without the corpus is a gain (they run wherever exiftool is installed).
- (c) **All 22 seeds regenerated: accepted.** I checked byte for byte: each of the 22 seeds at
  `f5c1b9c` equals its new version with exactly one `00 00` (the empty metadata-flag word)
  inserted, and nothing else changed. Same cases. `fuzz_seeds_reach_their_named_outcome` still
  sees each one reach its outcome (28 seen).

**4. Red-proofs:** see AC6 above and the design-`src/` run.

**5. The test block builder, checked by hand.** I dumped `build_odd(8, Both)` written by the
writer (199-byte block, an XMP packet) and parsed it with an independent Python TIFF walker (not
the repo's reader). Header IFD0 = 208 = 8 + 200, pad byte at 207 = 0. `ExifIFD` = 8 (in-block
0); Exif IFD has 6 entries, 2 + 72 + 4 = 78 bytes, so its values start at 86: `ExposureTime` at
86 (1/250), `FNumber` at 94 (28/10), `DateTimeOriginal` at 102 (20 bytes), all absolute.
`InteropOffset` = 122 = 8 + 114, and 114 = 78 + 8 + 8 + 20 is the Exif IFD's length. Interop IFD
at 122 (18 bytes, `R98`, inline) ends at 140 = `GPSInfo` = 8 + 132. GPS IFD: `GPSLatitude` at
182 = 140 + 2 + 36 + 4 (47/1, 36/1, 1234/100). Every IFD and out-of-line value is at an even
offset, inside the block and before IFD0, and every next-IFD link is 0. The offsets are absolute
for the base, Interop included, so oracle 3 passes for the right reason.

**6. No panics, no allocation.** The new arithmetic in `plan` is checked: `offset + 2` via
`usize::try_from(…).and_then(checked_add)`, `align2(block_len)?` and `checked_add` for the IFD0
offset, `EXIF_BLOCK_BASE.checked_add(o)` for the pointers, `u32::try_from` for the IFD0 offset,
`Value::count`/`byte_len` checked for XMP and RATIONALs; `u32::try_from` (not `as`) for the
unsigned hints. `just lint-no-allow` is green. The allocation test sets every field. The fuzz
input decodes every new field (strings via `from_utf8_lossy`, offsets as bytes, hints as `i8`
pairs), so random inputs reach every new refusal, not only `Ok`. Each refusal has its own seed,
asserted to reach its named reason.

**7. Docs.** The module docs' "What is written" lists the eight strings, `XMLPacket`, the four
hints with their types, and the block, base, pad and layout diagram. "Checks and their order"
lists UniqueCameraModel, then the strings (ASCII then form), hints, block offsets, XMP, opcode
lists, size. That matches the code's order, which M11 and M16 pin. The CHANGELOG
`[Unreleased]` Added entry, the provenance-ledger row (TIFF 6.0, Exif 2.32 §4.6.3, XMP Part 3
§1.1.4, DNG 1.7.1, DEC-072/074/081, black-box tools) and the AGENTS.md fuzz-dng-write block are
all present.

**8. Gates** (re-run by me, at `5d89f90`). `gh pr checks 332` was read first: 22 of 22 pass. With
the corpus and `IRRADIANCE_DNG_VALIDATE` set, `just test` gives 912 passed, 0 failed, 1 ignored
across 55 binaries. Only the SPEC-060 and SPEC-085 SDK cross-checks skip (their dirs are unset).
These are green: `just lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`, `deny`, `deny-fuzz`,
`lint-fuzz`, `lint-fuzz-red-proof`, `msrv`, `validate`, `gate-audit` and `cost-audit`.
`decisions-audit --changed f5c1b9c` is advisory and names DEC-075, DEC-080 and DEC-081; the
change is consistent with all three. `cargo semver-checks`: 202 pass, "no semver update
required". As the build noted, the `write` surface is not in the 0.2.6 baseline, so this covers
only the default surface. `just fuzz-dng-write`: 3,945,083 runs in 61 s, no crash.

**9. DEC-081: sound, followed by the code; accept at ship.**
- §1 shape: `ExifBlock` is `#[non_exhaustive]` with `Default`, the constant and the function exist,
  and every field is an `Option`.
- §2 empty values: an empty string gives count 1 (M09), an empty XMP is refused (M10, M12).
- §3 hint type: `Rational`. A zero denominator is refused on all four. The three RATIONAL tags
  refuse a negative part (M07); `BaselineExposure` takes either sign.
- §4 order: in `plan`, after UCM, before the walk (M11, M16, parity M12).
- §5 inside: even and `offset + 2 <= len` (M04, M05).
- §6 the form, not the calendar (M06; month 13 passes).
- §7 five reasons, all `CallerError`, no new `Error` variant.

Two wording fixes at ship, neither blocking: §4 should say the ASCII check comes before
`DateTime`'s form (deviation a). §2's "an empty descriptive string writes an empty tag" should
except `DateTime`, whose form check refuses `""` (`date_time_form_is_checked_not_the_calendar`
pins that).

### Cost self-report

- **Tokens (total):** 6,732,483 (floor, taken before the handback commit)
- **Estimated USD:** 5.25 (per component: 5 and 25 USD per million, 1-hour cache writes at 2x,
  cache reads at 0.1x)
- **Duration (minutes):** about 50
- **Source of the number:** this session's transcript
  (`e1ed8aaa-0485-4bae-b1aa-4275adc0fa82.jsonl`, 41 unique `message.id`s, all claude-opus-5-5,
  no subagents), deduplicated by `message.id` per docs/cost-tracking.md

### Drift and new artifacts

- **New decisions emitted:** none. No IDs minted.
- **Deviations from spec:** none by the verifier. The build's three are ruled above (all
  accepted).
- **Follow-up work identified:**
  - At ship (docs only): the two DEC-081 wording clauses above (§4 ASCII before form; §2's
    `DateTime` exception).
  - Nit: `absent_options_write_no_tag_and_empty_strings_write_count_one` sets `o.copyright`
    twice and never sets `o.date_time` (correct, because `""` is refused there, but a comment
    would help).
  - Still open from DEC-081: the empty-opcode-list follow-up (SPEC-061 behaviour).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. Two small things: `just decisions-audit --changed` needs a base ref here (the
   handoff's bare form errors under `just`), and the branch was already checked out in the
   orchestrator's worktree, so I had to reset my worktree's branch onto it instead.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. It would help to say that "do not read any writer implementation" means outside
   writers (exiftool, SDK, dnglab), not irradiance's own `src/`, which a verifier has to read
   to mutate.

3. **If you did this task again, what would you do differently?**
   — Start the mutant run in a separate worktree first, in parallel with `just test`, as I did
   here. Script the mutants as exact unique-match replacements, so a mutant that fails to apply
   stops the run instead of passing silently.
