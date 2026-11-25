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
  id: HANDOFF-191
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10624920           # REAL combined count — what cost-audit reads
  estimated_usd: 9.02              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: claude/spec-081-build
  pr: 332
  completed_at: 2026-11-24         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 7b805486-c477-491d-8d27-b4e83a3837a9.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 108 + cache-creation 225,560 (1-hour) + cache-read 10,335,279 + output 63,973, priced at 5 and 25 USD per million as HANDOFF-175 did. Re-sum at ship. exiftool parity 6 of 6 block shapes, base-10 red-proof caught; dng_validate no warnings on K-3 III and Q2M; K-3 III render equal to source, 76,681,557 samples differ without BaselineExposure."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-24
---

# HANDOFF-191: Build SPEC-081, the writer's EXIF block, XMP, descriptive tags and rendering hints

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-081` to `claude-opus-5` (implementer) for the
**build** cycle: extend `irradiance::dng::write` (shipped in SPEC-061) so the spec's
failing tests pass, on a branch cut from `main`.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-081-exif-block-and-xmp-placement-at-a-fixed-base-offset.md`.
  Read Design §1 to §7, the probe table in `## Implementation Context` and the rule-clause
  map. The API names in §1 and the check order in §4 are binding.
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
- **Project brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`
- **Prior spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/done/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call.md`
  and its handoffs HANDOFF-175 (build) and HANDOFF-190 (verify): the writer you extend,
  its oracles, and the punch list that missed a gate.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.
- **Starting code:** `src/dng/write.rs`, `src/write_types.rs`, `tests/dng_write.rs`,
  `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `tests/support/dng_write_oracle.rs`, `tests/support/dng_write_fuzz.rs`,
  `fuzz/fuzz_targets/dng_write.rs`, `examples/fuzz-seeds.rs`.
- **Corpus:** set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
- **Adobe `dng_validate`:** set
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
  for AC6 and AC7. Local only; never in CI; never commit its outputs.

### Decisions that apply

- `DEC-081` (proposed, this design): option shapes, empty values, hint type, check order,
  "inside the block", `DateTime` form, five new reasons.
- `DEC-072` (accepted, with its 2026-11-21 amendment): base 8, verbatim block, pointers
  `8 + offset`, at least one offset, even and inside, the base as a pinned constant and a
  function.
- `DEC-071` §2 and §3a (accepted): XMP as tag 700 BYTE; descriptive tags as typed ASCII,
  printable, 1024-byte bound, `DateTime` form, no tag-number field.
- `DEC-074` (accepted): the four hints, verbatim, zero denominator refused.
- `DEC-075` §1 and §7, `DEC-080` (accepted): layout and error placement you extend.
- `DEC-051`, `DEC-052`, `DEC-058`, `DEC-016`, `DEC-069` §6.

### Constraints that apply

- `no-panics-on-untrusted-input`: `just lint-no-allow` must pass.
- `oracle-must-be-shown-red`: oracle 4 and the hints half of oracle 5, negative control
  first.
- `provenance-recorded-per-algorithm`: the ledger row in Design §7.
- `no-copyleft-dependencies`: no new dependency at all.
- `library-not-application`: no I/O in `src/dng`.

### Prior related work

- SPEC-061 (PR 329, ship PR 330): the writer, DEC-075 and DEC-080.
- PR 284, PR 291, PR 292: the STAGE-017 frame, the DEC-072 amendment, DEC-074.

## Expected Deliverables

- Every file in SPEC-081's `## Outputs`, and every test in `## Failing Tests` passing.
- `just test` (with the corpus and `IRRADIANCE_DNG_VALIDATE` set), `just lint-ci`,
  `just lint-no-allow`, `just fmt-check`, `just doc-check`, `just deny`, `just deny-fuzz`,
  `just lint-fuzz`, `just lint-fuzz-red-proof`, `just msrv`, `just validate`,
  `just gate-audit`, `just cost-audit`, and 60 s of `just fuzz-dng-write`, all green.
  `cargo semver-checks` additions only. **Before handing back, read `gh pr checks` for
  your PR: every CI job must pass** (SPEC-061's punch list was a CI job missing from this
  list).
- A PR against `main` from `claude/spec-081-build`, naming HANDOFF-191, SPEC-081,
  STAGE-017, PROJ-005, DEC-071/072/074/075/080/081 and any new DEC. Commit messages plain
  prose (no code tokens).
- In the handback: the exiftool and `dng_validate` results per output (warnings, the K-3
  render equality with and without `BaselineExposure`), and the token count.

## Out of Scope

- Building, filtering or rebasing EXIF in `src/` (the test builder stands in for
  `diamond-exif/SPEC-025`).
- Previews (SPEC-082); UTF-8 `LocalizedCameraModel`; `Copyright`'s editor part.
- Changing SPEC-061's shipped behaviour, including the empty-opcode-list follow-up in
  DEC-081.
- Any change to the reader (`src/ifd.rs`, `src/plane.rs`, `src/develop.rs`), including an
  ASCII accessor. If a round trip exposes a reader bug, stop and report it.

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

*Filled in by the receiving agent.*

### Execution notes

- **Branch / PR:** `claude/spec-081-build`, https://github.com/jysf/irradiance/pull/332
- **Completed at:** 2026-11-24
- **All acceptance criteria met?** yes. AC1 to AC10 each have their named test, and all pass
  locally with the corpus and `$IRRADIANCE_DNG_VALIDATE` set.

**What was built.** `src/dng/write.rs` gains `EXIF_BLOCK_BASE` (8), `exif_block_base`,
`ExifBlock` (`#[non_exhaustive]`, `Default`) and the 14 `WriteOptions` fields of Design §1.
`SLOTS` grows from 25 to 40, still ascending by tag number. `IFD0_OFFSET` is gone: the header's
IFD0 offset is a planned value, `8 + align2(block length)`, and the block is copied verbatim at 8
after the head region is zeroed, so the pad byte is 0. Two new value kinds (`Rational`,
`SRational`, 8 bytes, out of line) and a `Bytes` kind for `XMLPacket`. Checks 6a to 6d sit in
`plan` between `UniqueCameraModel` and the `OpcodeList1` walk, in DEC-081 §4's order, so
`written_len` and `write_into` agree. `src/write_types.rs` gains the five reasons with rustdoc
and `Display` arms.

**Tests.** `tests/dng_write.rs`: `every_new_tag_matches_the_spec_table` (odd and even block and
XMP), `exif_block_sits_at_the_base_and_ifd0_follows_it` (Exif-only, GPS-only, both; odd and
even), `absent_options_write_no_tag_and_empty_strings_write_count_one`,
`round_trip_holds_with_every_new_option_set` (three framings; hints also read back through
`Container::rationals`), `exif_block_base_is_pinned_to_eight`,
`date_time_form_is_checked_not_the_calendar`,
`rational_hints_refuse_zero_and_unsigned_negatives`; `every_refusal_is_typed` gains 34 cases
(the 8 descriptive fields times 3 bad values, plus 10) and the `offset + 2 == len` boundary that
passes; `checks_run_in_the_documented_order` gains 9 steps and checks `write_into` agrees at each;
determinism, allocation and the seed test cover every new option. `tests/dng_write_corpus.rs`:
oracles 3, 4 and 5. `tests/support/exif_block.rs`: the test-only builder.

**exiftool 13.55 (oracle 3 and 4).** For each of the 6 block shapes (both, Exif-only, GPS-only;
odd and even length), exiftool reads `ExposureTime` 1/250, `FNumber` 28/10, `ISO` 400,
`ExifVersion` 0232, `DateTimeOriginal`, `InteropIndex` R98, `GPSVersionID` 2 3 0 0,
`GPSLatitudeRef` N and `GPSLatitude` 47, 36, 12.34 exactly (rationals compared by
cross-multiplying), the absent IFD's tags as absent, all eight descriptive tags, the four hints
and `XMP-dc:Description`. Red-proof: the negative control (base 8) passes; the base-10 block
(asserted to differ from the base-8 one) gives 5 mismatches: `ExposureTime` read 8.928571429,
`FNumber` 0.0008105041336, `DateTimeOriginal` "26:11:24 09:30:15", `InteropIndex` absent,
`GPSLatitude` 0 1 43.078925.

**dng_validate 1.7.1 (oracle 5).** K-3 III at `StandardCrop` with its source tags 271, 272, 305,
306, 315, 33432 and all four hints, the oracle-3 block and an XMP packet: no `Warning` or `Error`
line; its `-16 -tif` render equals the source's (6192 x 4128, 76,681,728 samples, 0 differ).
Without `BaselineExposure`: valid, and 76,681,557 of 76,681,728 samples differ, matching DEC-074's
probe exactly. Q2M with its source tags 271, 272, 305, 306, 315, 33432, 50709, 50735 and all four
hints, plus the block and XMP: no warnings.

**Gates.** `just test` (corpus and dng_validate set): 912 passed, 0 failed, 55 binaries; only the
two optional SDK cross-checks (SPEC-060, SPEC-085) skip. `just lint-ci` (clippy 0.1.98),
`just lint-no-allow`, `just fmt-check`, `just doc-check`, `just deny`, `just deny-fuzz`,
`just lint-fuzz`, `just lint-fuzz-red-proof`, `just msrv`, `just validate`, `just gate-audit`,
`just cost-audit`: green. `just fuzz-dng-write`: 3,906,300 runs in 61 s, no crash.
`cargo semver-checks`: 202 checks pass on the default surface. The `write` feature does not exist in
the 0.2.6 baseline, so everything under it, including SPEC-081's additions, is new relative to
crates.io; there is nothing under it to break.

### Cost self-report

- **Tokens (total):** 10,624,920 (floor, taken before the handback commit)
- **Estimated USD:** 9.02 (per component, 5 and 25 USD per million, 1-hour cache writes at 2x)
- **Duration (minutes):** about 45
- **Source of the number:** this session's transcript (`7b805486-...jsonl`), deduplicated by
  `message.id` per docs/cost-tracking.md

### Drift and new artifacts

- **New decisions emitted:** none. DEC-081 covered every choice; its status is left `proposed`.
- **Deviations from spec:**
  - Oracle 3 and 4 use a synthetic plane, not a held file, so they run wherever exiftool is
    installed, with no corpus. Oracle 5 covers the held files.
  - At `DateTime`'s position the printable-ASCII and length check runs before the form check, so a
    `DateTime` with a non-ASCII byte is `InvalidAscii { field: "DateTime" }`, not
    `DateTimeInvalid`. The spec says the form is checked "also"; this reads that as after.
  - The fuzz input format gained a `u16` metadata-flag word and its fields after the model, so
    all 22 existing seeds were regenerated (same cases, two bytes longer) beside the 6 new ones.
  - `scripts/lint-fuzz-red-proof.sh` needed no change: the fuzz target still includes only
    `tests/support/dng_write_fuzz.rs`, not the new `exif_block.rs`.
- **Follow-up work identified:** none new. DEC-081's empty-opcode-list follow-up still stands.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   Very little. The probe table and §4's order made the build a transcription. The one choice
   left open was whether `DateTime`'s ASCII check or its form check comes first; it is recorded
   above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   No. It would help to say up front that `cargo semver-checks` cannot see the `write` surface
   until a release carries it, so "additions only" holds only for the default surface.

3. **If you did this task again, what would you do differently?**
   Write the exiftool comparison as a function returning mismatches from the start (as here),
   because the red-proof and the positive oracle then share one code path and the red-proof
   cannot pass because of a different check.
