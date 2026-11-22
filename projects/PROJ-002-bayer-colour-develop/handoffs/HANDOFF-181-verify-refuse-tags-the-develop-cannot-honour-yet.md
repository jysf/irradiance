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
  id: HANDOFF-181
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-086

project:
  id: PROJ-002
  stage: STAGE-027
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
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 9951163               # REAL combined count — what cost-audit reads
  estimated_usd: 7.64              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 18
  branch: claude/spec-086-verify-base
  pr: null
  completed_at: 2026-11-22               # YYYY-MM-DD
  notes: "Round 1 PUNCH LIST (P1 interleave-factor read, P2 SampleFormat [1, 3] row); round 2 APPROVED at f71f703 after dbc2017 closed both. Tokens cover both rounds, a floor taken before the round-2 commit."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-22
---

# HANDOFF-181: VERIFY: refuse tags the develop cannot honour yet (SPEC-086)

## Delegation Summary

You are the **verifier** for SPEC-086. Review it cold: you did not build it.

- **Under review:** PR 309 (`feat/spec-086-unhonoured-tags`, head `258eee9`), built under
  HANDOFF-180.
- **Your branch:** `claude/spec-086-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-086 implements DEC-077:
- nine new `Sensor` fields (`DNGVersion`, `DNGBackwardVersion`, `SampleFormat`,
  `LinearizationTable`, `BlackLevelDeltaH/V`, `SubTileBlockSize`, the two interleave
  factors);
- a develop-time refusal inside `opcode::check_required_processing`: the backward-version
  guard (above 1.7.1.0, defaulting from `DNGVersion`) then the seven tags in ascending
  order, with exact no-op values honoured and a malformed tag refused;
- a decode-time refusal of a Compression 7 frame whose precision is not `BitsPerSample`;
- three `Error` variants (72 to 75), `DEVELOP_VERSION` unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-086-refuse-tags-the-develop-cannot-honour-yet.md`.
  Judge the ACs against the spec **as designed in PR 308**
  (`git show origin/main:<spec path>`), not as edited on the build branch.
- **Decisions:** DEC-077 (the rule), DEC-051 (variant order, `kind()` with no wildcard),
  DEC-053 (refuse before writing; refusals leave `DEVELOP_VERSION`), DEC-012 (malformed
  costs its field), DEC-049 (`Sensor` is `#[non_exhaustive]`).
- **Build handback:** HANDOFF-180's `## Handback`: the AC9 mutant table and five
  deviations. Check each claim yourself.
- **Specification:** DNG 1.7.1.0 for `DNGBackwardVersion` (and its default), and the
  seven tags. If you have no copy, say so; do not guess at wording.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **Every develop entry refuses before writing.** Confirm each of the nine AC1 entries
   reaches the check before any write to `dst`, including the threaded colour plans and
   `develop_with_scratch_into`. Is AC1's sentinel assertion on every entry, and does
   mutant (h) really go red? Are there public develop or demosaic entries the spec missed?
2. **The version guard against the specification.**
   - The byte-wise comparison with `[1, 7, 1, 0]`, and the boundary (1.7.1.0 develops,
     1.7.1.1 refuses).
   - The default: absent backward takes `DNGVersion` with its last two bytes zeroed.
   - **Rule on the build's deviation:** a malformed `DNGVersion` with no backward version
     is recorded and then skipped, so no guard runs. Is that acceptable under DEC-077,
     or should it refuse `UnsupportedTag { tag: 50706 }`? Give a ruling and, if it
     should refuse, put it on the punch list (it is a `src/` change).
3. **The tag table.** For each of the seven tags: the refused value, the honoured no-op,
   and a malformed value. Are the honoured values really byte-identical (AC2) on every
   entry? Is the ascending order right (rows j and k)?
4. **The read strictness deviations.** `SampleFormat` and `LinearizationTable` must be
   SHORT and non-empty; the versions BYTE × 4; the deltas accept RATIONAL as well as
   SRATIONAL; `SampleFormat`'s count is not tied to `SamplesPerPixel`. Rule on each. An
   empty `LinearizationTable` (count 0): malformed and refused, or not? A `SampleFormat`
   as LONG (TIFF 6.0 says SHORT)?
5. **No hostile-input panics.** The new reads go through bounded readers
   (`MAX_TAG_VALUES`). A huge `LinearizationTable` or delta count is bounded. A zero
   denominator in a delta is malformed, not a division. Check what the `ifd` fuzz target
   now exercises.
6. **Decode-time precision (AC5).** On the strip path, `unpack_unit_into`,
   `PlaneDecoder::unpack_unit_into` and the tile path. Order after the shape check. Is
   `validate` or `units` affected, and should it be?
7. **Metadata and plane reads still succeed (AC4)**, with the new fields holding the
   written values, and `irr ifd` working on a refused file.
8. **Nothing held changes (AC7).** Run `just test` with the corpus. Confirm no Tier-B pin
   moved (`git diff origin/main -- tests/`) and `DEVELOP_VERSION` is still `"8"`.
   Independently confirm, on the held files, that no SOF-3 precision differs from
   `BitsPerSample` and no tag of the set has a refused value.
9. **Errors (AC8).** The three variants are appended at the end in DEC-077 §4's order,
   with the right kinds; `error_kind.rs` is 75 / 26 / 32 / 17; DEC-051 is updated.
   `UnsupportedTag`'s `Display` for each tag, and its fallback.
10. **Mutants.** Re-run the build's (a) to (h). Then try at least:
    - the version default zeroing the wrong bytes;
    - the version guard placed after the tag set;
    - the tag set in descending order;
    - `SampleFormat` honoured when any (rather than every) value is 1;
    - a delta refusal reading only the first entry;
    - the precision check on the strip path only.

    Record what goes red, or argue equivalence.
11. **The files outside the plan.** `src/color.rs` and `src/develop.rs` gain nine `None`
    fields each. Confirm that is all, and that no existing assertion was weakened
    (`git diff origin/main -- tests/ src/`).
12. **Release surface.** The CHANGELOG `[Unreleased]` entry names the refusals and the
    new fields accurately. `semver-check` is additive.
13. **The build's cost figure.** The handback reports 12.95M tokens over 12 minutes.
    Note whether that looks plausible; the orchestrator re-sums it at ship.
14. **Gates:**
    - `just test` with the corpus;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
      `semver-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants.
- **A ruling on item 2's malformed-`DNGVersion` deviation and on item 4's strictness
  choices.**
- **Commit this handoff's filled handback** to `claude/spec-086-verify-base` and push it.
  Do not open a PR, and do not push to `feat/spec-086-unhonoured-tags`.
- **Fixes:** a punch-list fix is fine on your branch if it is tests, scripts or docs
  only. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

## Out of Scope

- SPEC-087's general rule for malformed interpretation tags.
- Applying `LinearizationTable` or `BlackLevelDelta*`.
- Audit ranks 8, 9 and 11.

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

*Filled in by the receiving agent (cold verify, HANDOFF-181).*

### Execution notes

- **Branch / PR:** `claude/spec-086-verify-base` (no PR, as instructed). Reviewed PR 309's
  head `258eee9`; the spec judged as designed on `origin/main` (PR 308).
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** No, narrowly. AC1 to AC9 pass as written, but DEC-077 §1's
  read contract is not met for the two interleave factors (punch list P1), and one DEC-077
  §2 clause ("every value 1") had no test (P2, fixed here, tests only).
- **For `verify`:** ⚠ **PUNCH LIST** (one `src/` item, one test item already on this branch).

### Punch list

- **P1 (`src/ifd.rs`, for the build branch).** `RowInterleaveFactor` and
  `ColumnInterleaveFactor` are read through `scalar()`, which keeps only the first value and
  accepts BYTE, UNDEFINED and RATIONAL. So a factor written with count 0 reads as absent,
  and one written `[1, 2]` reads as 1. Neither is recorded in `malformed_tags`, and both
  develop with `Ok`. Probed on a scratch copy: count 0 gives `row_interleave_factor: None`,
  `malformed_tags: []`, `develop_into` `Ok(())`; `[1, 2]` gives `Some(1)`, `[]`, `Ok(())`.
  DEC-077 §1 says SHORT or LONG, one value, with a malformed value recorded and then
  refused by §2. Fix: read both factors with a single-value reader (SHORT or LONG,
  count exactly 1, otherwise an error that `cost_the_field` records). Add count-0 and
  count-2 cases for both tags to `ac6_a_malformed_unhonoured_tag_is_refused`. That is
  red today and green after the fix. `SubTileBlockSize` goes through `array::<2>`, which already
  records a wrong count; its loose type acceptance is harmless because only exactly `[1, 1]`
  is honoured. Hostile or hand-made files only: no held file carries either tag.
- **P2 (tests, done on this branch).** Mutant v5 below ("any value is 1" instead of
  "every value is 1") survived every test, because each `SampleFormat` fixture had one
  value. Added row "b2: `SampleFormat` = [1, 3]" to `refusal_rows()` in
  `tests/unhonoured_tags.rs`. It is green on the real code and red on v5 ("b2 … on
  DevelopInto: expected Tag(339), got Ok(())"). The row also feeds AC4 (the field holds
  `[1, 3]`, the plane is unchanged). The orchestrator should carry this commit onto
  `feat/spec-086-unhonoured-tags` with P1.

### Rulings

- **Malformed `DNGVersion` with no `DNGBackwardVersion` (item 2): acceptable for SPEC-086,
  not on the punch list.** DEC-077 §2 names only a malformed *backward* version as a refusal,
  and says "when both are absent there is no check". Treating an unreadable `DNGVersion`
  like an absent one follows DEC-012's interpretation default, and no held file is
  affected. Refusing it would change the decision, not fix the build. Follow-up: SPEC-087's
  general malformed rule should refuse it as `UnsupportedTag { tag: 50706 }`, matching
  50707, because the file's version is then unknown. It should also add 50706 to
  `unhonoured_tag_name`.
- **Read strictness (item 4): all accepted.**
  - `SampleFormat` SHORT only: TIFF 6.0 says SHORT. A LONG one is recorded malformed and
    refused, which I probed (`malformed_tags: [339]`, `UnsupportedTag { tag: 339 }`). Being strict
    here only turns a strange file into a typed refusal, never into wrong pixels.
  - `SampleFormat` count not tied to `SamplesPerPixel`: accepted, because "every value 1"
    makes a count mismatch harmless. P2 now pins "every".
  - `LinearizationTable` SHORT and non-empty: count 0 is recorded malformed and refused
    (probed). Either reading refuses, because presence alone refuses. Malformed is the
    more honest field state.
  - Versions BYTE × 4: as DEC-077 §1 and the DNG tag definition say.
  - Deltas accepting RATIONAL as well as SRATIONAL: accepted. Only an exact all-zero vector
    is honoured. A RATIONAL above `i32::MAX` errors and is refused, and a zero
    denominator is malformed before any division. Leniency cannot reach a wrong-pixel `Ok`.

### Evidence by item

1. **Every entry refuses before writing.** Each of the nine AC1 entries reaches
   `check_required_processing` as its first statement: `develop_core` (mono, both mono
   entries and `develop_with_into`); `check_colour_required_processing`, the first line of the
   four colour entries (the threaded ones before `colour_band_plan`) and of
   `to_srgb_linear_into`; and the first line of both demosaic entries. `develop_with_into` is
   not in AC1's list but shares `develop_core`. The opcode primitives (`apply_gain_maps`,
   `apply_fix_bad_pixels_constant`, `apply_warp_into`) are not develop entries, and nothing
   changed for them. The sentinel is asserted on every entry and every row. Mutant (h) went red
   on the sentinel ("a: SampleFormat = 3 on DevelopColorIntoThreaded: dst (and scratch)
   must be untouched"). Nit: the scratch half of the `develop_with_scratch_into` sentinel
   is vacuous, because `scratch_len` is 0 with no opcode lists.
2. **Version guard.** `[u8; 4]` compares lexicographically, byte by byte from the first. AC2
   develops 1.7.1.0, row i refuses 1.7.1.1, and mutant (c) (`>=`) is red. The default is
   `[a, b, 0, 0]` from `DNGVersion`, and mutants v1 and v2 (the wrong bytes zeroed) are red on
   AC3. I have **no local copy of the DNG 1.7.1.0 specification**. My recollection agrees
   that the default is `DNGVersion` with the last two bytes zeroed, but I did not verify the
   wording.
3. **Tag table.** Each refused value, honoured no-op and malformed (ASCII) case is
   exercised on every entry or on AC6's two entries, and the honoured outputs compare
   byte-identical, including f32 as bits. Ascending order: row j (written in descending
   order in the file) and row k. Mutants v3 and v4 are red.
4. **Strictness.** See Rulings, and P1.
5. **No hostile-input panics.** The new reads go through `uints`, `rationals` and `payload`,
   which bound `count` by `MAX_TAG_VALUES` before allocating. Probed: a `LinearizationTable`
   with count `u32::MAX` is recorded malformed (`TagTooLarge`) and refused, with no panic.
   A zero delta denominator is checked before the division. `lint-no-allow` (forbidding
   unwrap, expect, indexing, panic and arithmetic side effects in the lib) is green. The
   `ifd` fuzz target calls `container.sensor()`, so it now exercises all nine reads, and
   the `develop` and `plane` targets reach the new checks.
6. **Decode-time precision.** `check_precision` sits right after `check_strip_shape` on the
   strip path (`decode_unit`, shared by `unpack_into`, `unpack_unit_into` and
   `PlaneDecoder::unpack_unit_into`) and on the tile path (`decode_tile_row`). AC5 covers
   the three paths × strip and tile × both byte orders × both directions. Mutants (g) and v7
   (the strip only) are red on the tile case. `validate` and `units` never parse a frame
   header, and neither does the shape check, so leaving them alone is consistent with
   DEC-077 §3.
7. **Metadata and plane reads.** AC4 checks `sensor()`, the field values and the `unpack_into`
   samples for every row on both fixtures. On a copy of the M Monochrom Typ 246 file with
   `DNGBackwardVersion` rewritten to 1.8.0.0 (exiftool), `irr ifd` and `irr unpack` exit 0,
   and `irr develop` exits 1 with "DNG backward version 1.8.0.0 is newer than 1.7.1.0, the
   newest this library reads".
8. **Nothing held changes.** `just test` with the corpus exits 0: 836 passed, 0 failed,
   12/12 corpus files present, and only the two SDK cross-checks skip. That matches the
   build's count. `git diff origin/main -- tests/` touches only the two new files and
   `error_kind.rs`. `DEVELOP_VERSION` is `"8"`. Independent re-scan (`exiftool -a -G1 -n`,
   every sub-IFD): no held file has `LinearizationTable`, `BlackLevelDelta*`,
   `SubTileBlockSize` or either interleave factor. `SampleFormat` appears only on the D300S
   (sensor SubIFD `1`). The largest `DNGBackwardVersion` is 1.4.0.0. The M Monochrom has
   `DNGVersion` 1.0.0.0 and no backward version. A SOF-3 header scan of every DNG:
   the Pixels 3a, 4a and 7 Pro have P = 16 (192 frames each) with BitsPerSample 16, the
   Typ 246 has 12/12, and the K-3 III has 14/14.
9. **Errors.** The three variants are appended after `UnsupportedCfaPlaneColor`, in DEC-077 §4's
   order, with explicit `kind()` arms (no wildcard). `error_kind.rs`: 75 / 17 / 26 / 32,
   green. DEC-051 has the paragraph. `Display`: named for all eight tags, with "tag" as the
   fallback. Nit: for a malformed `DNGBackwardVersion` the text "changes what the samples
   mean and is not applied yet" is not accurate. It could be reworded alongside P1.
10. **Mutants.** See the table below. 16 mutants after a green control: 15 red, and v5
    survived until P2.
11. **Files outside the plan.** `src/color.rs` and `src/develop.rs` gain exactly the nine `None`
    fields, in their test-only `Sensor` literals. Across `tests/` and `src/`, the only removed
    lines are the old counts (72, 24, 31), an import line and a match arm, each re-added in
    extended form. No assertion was weakened.
12. **Release surface.** The CHANGELOG `[Unreleased]` entries (Added: the fields, constants
    and variants; Changed: the develop refusals and the decode-time precision) are accurate.
    `semver-check`: 202 pass, "no semver update required".
13. **Build cost.** I re-summed the build's transcript
    (`ad9e0a16-….jsonl`, deduped by `message.id`). It comes to 14,241,449 tokens over a
    12.5-minute span (21:59 to 22:11 UTC). That is consistent with the reported
    12,951,904 floor, which left out the handback commit's messages. The duration is
    plausible: this verify's own full-corpus `just test` plus 16 mutant builds took about
    the same time on this machine.
14. **Gates.** All exit 0: `just test` (corpus), `lint-ci`, `lint-no-allow`, `fmt-check`,
    `validate`, `doc-check`, `cost-audit`, `gate-audit`, `name-length-check`,
    `decisions-index --check` and `semver-check`. `fmt-check` and `lint-ci` were re-run after
    the P2 test edit.

**Mutant table** (control first; each mutant in its own `git archive` copy with its own
`CARGO_TARGET_DIR`, running `unhonoured_tags` and `ljpeg_precision_binding`):

| Mutant | Red on | First failure message |
|---|---|---|
| control | — | green (7 + 3) |
| (a) check not called | ac1 ×2, ac3, ac6 | DNGVersion 1.8.0.0 on DevelopInto: got `Ok(())` |
| (b) `SampleFormat` 3 honoured | ac1 ×2 | row a / row j wrong tag |
| (c) `>=` | ac2 | 1.7.1.0 on DevelopInto refused |
| (d) default ignores `DNGVersion` | ac3 | DNGVersion 1.8.0.0: got `Ok(())` |
| (e) malformed refusal dropped | ac6 | 339 as ASCII: got `Ok(())` |
| (f) zero-delta exemption dropped | ac2 | all-zero BlackLevelDeltaH refused |
| (g) precision check removed | ac5 | P16/BPS12 strip `UnpackInto`: got `Ok` |
| (h) write before the check, threaded colour | ac1 ×2, ac3 | sentinel on DevelopColorIntoThreaded |
| v1 default keeps byte 3 (`[a, b, c, 0]`) | ac3 | 1.7.9.9 refused as `[1, 7, 9, 0]` |
| v2 default zeroes byte 2 (`[a, 0, 0, 0]`) | ac3 | 1.8.0.0: got `Ok(())` |
| v3 version guard after the tag set | ac1 order | row k: got `UnsupportedTag { 50712 }` |
| v4 tag set descending | ac1 order | row j: got `UnsupportedTag { 51157 }` |
| v5 `SampleFormat` honoured when **any** value is 1 | **survived**, red after P2 | b2: got `Ok(())` |
| v6 delta refusal reads only the first entry | ac1 | row d: got `Ok(())` |
| v7 precision on the strip path only | ac5 | tiled P16/BPS12: got `Ok` |
| v8 malformed `DNGBackwardVersion` not refused | ac6 | count 3: got `Ok(())` |

### Cost self-report

- **Tokens (total):** 7,653,140 (a floor: measured just before the handback commit,
  so the commit and report turns are not in it).
- **Estimated USD:** 6.05 (input 95, cache creation 136,714, cache read 7,478,414, output 37,917; Opus 5.5 at $5 / $25 per MTok,
  1-hour cache writes at 2×).
- **Duration (minutes):** 8 (transcript timestamp span).
- **Source of the number:** this session's transcript
  (`16fda036-1e4a-4623-a3af-c3b20b11da7e.jsonl`, matched by the scratchpad UUID), deduped by
  `message.id` keeping the largest `output_tokens`, per `docs/cost-tracking.md`. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verifier. The build's five deviations are ruled on
  above: all accepted, with P1 as a gap the build did not list.
- **Follow-up work identified:**
  - P1 (punch list, `src/`).
  - SPEC-087: refuse a malformed `DNGVersion` when no backward version is present
    (`UnsupportedTag { tag: 50706 }`), and name 50706 in `Display`.
  - Nits, not blocking: the vacuous scratch sentinel, and the 50707 `Display` wording.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing slowed me much. DEC-077 §1 lists the types but not the counts for the scalar
   tags, which is how P1 slipped past design and build.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-012's "present, shaped wrong" applies to a scalar's count. The shared `scalar()`
   reader does not record it, so every new scalar tag that a refusal depends on needs a strict reader.

3. **If you did this task again, what would you do differently?**
   — Probe the readers with odd counts (0, 2, `u32::MAX`) before running mutants. It was the
   cheapest check and it found the only `src/` gap.

---

### Round 2 (2026-11-22): ✅ APPROVED at `f71f703`

The build branch fixed the punch list in `dbc2017` and recorded it in `f71f703`.
`claude/spec-086-verify-base` now carries `origin/feat/spec-086-unhonoured-tags` through a
clean merge. Its `src/` and `tests/` are byte-identical to `f71f703`.

- **What moved (`git diff 258eee9 f71f703`).** Only `src/ifd.rs` (the new
  `single_short_or_long` reader used for both interleave factors, plus their doc comments),
  `src/lib.rs` (a separate `Display` arm for `UnsupportedTag { tag: 50707 }`, and 50707
  removed from the name lookup), `tests/unhonoured_tags.rs` (the P2 row, the AC6 cases and a
  `Display` assertion) and HANDOFF-180's handback. Nothing else changed.
- **P1 closed.**
  - The new AC6 cases (count 0 and count 2 for both factors) fail when `tests/` is run
    against 258eee9's `src/ifd.rs` ("50975 with count 0: malformed_tags [] must name
    50975"), and they pass on the fix.
  - An extra probe on the fix, for both tags and on `develop_into` and `develop_color_into`:
    BYTE 1, RATIONAL 1/1, LONG `[1, 2]`, SHORT count 0, SHORT `[1, 2]` and SHORT with count
    `u32::MAX` are each recorded in `malformed_tags`, give a `None` field, and refuse
    `UnsupportedTag` for the right tag with `dst` untouched.
  - A single LONG 1 develops `Ok`, and a single LONG 2 is read as 2 and refused.
  - The count is checked before `uints`, so a huge count never reaches an allocation.
  - The stricter type set (SHORT or LONG only) matches DEC-077 §1.
- **P2 present and unchanged.** Row b2 is byte-identical to the one in `8c90c4d`.
  Mutant v5 is still red on it ("b2: SampleFormat = [1, 3] on DevelopInto: expected
  Tag(339), got Ok(())").
- **`Display` accurate.** A malformed `DNGBackwardVersion` now reads "DNGBackwardVersion
  (50707) is malformed, so the file's reader version is unknown". It no longer claims the
  tag changes what the samples mean, and the test asserts that. The specific arm comes
  before the general one, so the other tags are unchanged. With 50707 out of the lookup,
  the lookup is only reached for the seven sample-meaning tags and the "tag" fallback.
- **Gates.** All exit 0: `just test` with `IRRADIANCE_CORPUS_DIR` (836 passed, 0 failed,
  12/12 present; same count as round 1, because the new cases sit inside existing tests),
  `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check` and `semver-check` ("no
  semver update required").
- **Method note.** Copies made with `git archive` carry commit-time mtimes. One round-2
  probe that reused another copy's `CARGO_TARGET_DIR` ran a stale binary (7 tests instead
  of 8). Every result above comes from a re-run with a fresh target dir per copy and
  touched sources. The round-1 mutants each had their own target dir from the first build,
  so they were not affected.
- **Remaining follow-ups (unchanged, none blocking).** SPEC-087 should refuse a malformed
  `DNGVersion` when no backward version is present. The scratch half of AC1's sentinel
  check is vacuous on these fixtures.
- **Cost.** `tokens_total` is now both rounds re-summed from this session's transcript:
  9,951,163 (input 119, cache creation 159,021, cache read 9,744,888, output 47,135), about
  $7.64 at Opus 5.5 rates with 1-hour cache writes. That is round 2 adding about 2.3M over
  round 1's 7,653,140 floor. It is a floor, taken before this commit. The duration of 18
  minutes is the whole transcript span, including the idle gap between rounds.
