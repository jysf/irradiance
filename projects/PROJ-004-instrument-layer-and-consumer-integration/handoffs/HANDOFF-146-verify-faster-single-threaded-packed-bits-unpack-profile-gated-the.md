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
  id: HANDOFF-146
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-14
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-069

project:
  id: PROJ-004
  stage: STAGE-020
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 6884347               # REAL combined count — what cost-audit reads
  estimated_usd: 5.29              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 55
  branch: claude/spec-069-verify-a21a37
  pr: 238
  completed_at: 2026-11-14               # YYYY-MM-DD
  notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (49 unique message.ids, all claude-opus-5-5, no subagents): input 104 + cache-creation 113,425 (1-hour) + cache-read 6,739,266 + output 31,552, same rates. APPROVED at 90c4bc4 (code 7b15289), no code changes. The handback floor was 4,912,796 and $3.81."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-14
---

# HANDOFF-146: VERIFY: faster packed 12/14-bit unpack, whole byte groups (SPEC-069)

## Delegation Summary

You are the **cold verifier** for SPEC-069. You did **not** implement it. Review it
against SPEC-069 and DEC-064, and return exactly one verdict.

- **The build:** **PR #238**, branch `claude/spec-069-build`, cut from `origin/main` at
  `2e48dd7`. Its head is `90c4bc4`: the code commit `7b15289`, then the handback.
- **Your branch:** your worktree is cut from `claude/spec-069-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers:
- In `src/plane.rs`, `unpack_bitstream` decodes whole byte groups (14-bit: 7 bytes → 4
  samples; 12-bit: 3 → 2). The unchanged `BitReader` reads the tail, via
  `BitReader::new(strip)` and then `byte_pos` set to the group boundary.
- `tests/plane_groups.rs`: the tier-A round trips over strips, bands and clipped tiles.
- The `src/plane.rs` unit test `groups_keep_bitreader_errors_and_partial_writes`.
- The one permitted retarget of `tests/plane_oracle.rs`'s `inject_chunk_extraction_fault`.
- `examples/unpack-timing.rs`, and an `unpack-timing` recipe in `app.just`.
- CHANGELOG.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-069-faster-single-threaded-packed-bits-unpack-profile-gated-the.md`.
  Read all of it: the §1 probe, §2 the kernel, §3 the retarget, §4 timing, AC1–AC7, the
  red-proof list and the Notes.
- **Decision:** `./decisions/DEC-064-the-packed-12-and-14-bit-unpack-reads-whole-byte-groups-and-bitreader-stays-for-the-tail.md`.
  Also DEC-063 (units and the call sites), DEC-008, DEC-017 (the red-proof mechanism),
  DEC-016 and DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-145-build-faster-single-threaded-packed-bits-unpack-profile-gated-the.md`.
  Treat its claims as claims. The points to judge:
  - **The retargeted injector (AC4).**
    - Diff `tests/plane_oracle.rs` against `2e48dd7`. Only `inject_chunk_extraction_fault`'s
      needle, replacement and doc comment may change.
    - The needle `(v.checked_shr(42), v.checked_shr(28))` must occur exactly once in
      `src/plane.rs`, inside the **live** 14-bit group path, not in dead code or a
      comment.
    - The fault must change values but not the number of bits consumed.
    - Confirm both red-proofs ran and that their honest and mutant digests differ (the
      handback lists them), and that both controls are green.
    - The build left the assertion's failure message stale ("BitReader::read's chunk
      extraction"), because the spec forbids assertion edits. Judge whether that is
      right, or should have been fixed as part of the permitted edit.
  - **The tail position.** The build set `byte_pos` after `BitReader::new`, instead of
    adding a constructor (a dead-code lint). Confirm `BitReader`'s own code is
    byte-for-byte unchanged (`git diff 2e48dd7 -- src/plane.rs`). Confirm the tail
    reports the same `Truncated { at }` as the old per-sample loop.
  - **The "sub-slice" mutant is caught only by AC3.** That is expected: the values are
    the same and only the error offset differs. Confirm AC3 would catch it on every
    residue, not just one.
  - **The benign mutant (`full − 1`) stays green everywhere.** Agree or disagree that
    the timing example is an acceptable sole guard.
  - **AC2 passes on the old code by design.** Confirm that AC2's packer is the test's
    own MSB-first code and never calls the crate (AGENTS.md §16 rule 5). Confirm it
    covers every tail residue at row ends and strip ends, 12- and 14-bit, strips, bands
    (`max_band_rows` 1 and 2) and clipped tiles.
  - **Timing (AC6).**
    - Reproduce `just unpack-timing` on `2e48dd7` plus only the example, then on the
      build head, back to back.
    - The build reports single-thread 239.0 → 9.7 ms, and 8 threads 31.2 → 1.4 ms.
    - The target is ≤ 30 ms absolute.
- **Toolchain:**
  - AGENTS.md §5–6 and §16, and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - The full suite takes about 25 min; run it in the background. Run mutants on an
    archived copy of the tree with its own target dir, so the source-copying red-proof
    tests never stage a mutated `src/plane.rs`.

## The verify contract

### Acceptance criteria and gates
- Check that every AC1–AC7 is met by a test that would fail if the property broke, and
  that the build reflection is honest.
- Run the gate suite yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`
- Run `just decisions-audit --changed` and `just fuzz-plane 60`.
- Observe CI green on the PR head with `gh pr checks 238`.

### The behavioural surface (this is where defects escape)

**Byte identity is the hard criterion (AC1).**
- `git diff 2e48dd7 -- tests/` must show only `tests/plane_groups.rs` (new) and the one
  injector function in `tests/plane_oracle.rs`.
- Every dnglab pin, the SPEC-068 any-order test, and the develop pins must have **run**
  (no SKIP) and passed.
- The other source-scan needles must still match exactly once, in live code:
  - `plane_oracle`'s `u16::from_le_bytes(*chunk)` and `u16::from_be_bytes(*chunk)`;
  - `plane_validate`'s guards;
  - `plane_tiles`' allocation tokens.

**Mutate it yourself**, outside any test knob, and run **tier A only**:
- the build's 7;
- the 14-bit group stride as 6 bytes (`as_chunks::<6>`) with matching arithmetic;
- the 12-bit mask `0x0FFF` changed to `0x07FF`;
- `full` computed from `strip.len()` alone, ignoring `dst.len()`. Does a tile row whose
  visible width is clipped still decode, or does it write or read past its window?
- the group path also taken for 16-bit (it should never be reached; say whether anything
  notices).

A mutant that stays green, or that only tier B catches, is a finding unless you judge it
benign and say why.

**No allocation, no panics, scope.**
- No `unsafe` and no allocation.
- No indexing; checked or saturating arithmetic only.
- `as_chunks` and `as_chunks_mut` are within MSRV 1.90.
- `BitReader`, `unpack_byte_aligned`, `decode_unit` and `decode_tile_row` are unchanged.
- No public API change.
- The CHANGELOG states the before and after numbers honestly.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #238.
  - Commit the filled HANDOFF-146 handback (and the timeline's verify line) on a branch
    named `claude/spec-069-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #238 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #238.
  - Reference HANDOFF-146, SPEC-069, DEC-064 and PR #238.
- **Rejected:** comment on #238 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- SIMD, `unsafe`, threads inside irradiance, and Compression 7 (SPEC-070).
- The 8- and 16-bit path.
- Speeding up the slow tier-B tests (signal `slow-tier-b-tests-in-debug`).
- The build's two [XS] follow-ups: the stale assert message, and timing examples in
  AGENTS.md §6. Note them if you agree; fix the message only if you judge it part of the
  permitted edit.

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

- **Branch / PR:** `claude/spec-069-verify-a21a37` (handback only). The verdict is a comment on https://github.com/jysf/irradiance/pull/238. No PR was opened, because there are no code changes.
- **Completed at:** 2026-11-14
- **All acceptance criteria met?** Yes.
  - **AC1:** `just test` with the corpus env ran 695 passed, 0 failed, 1 ignored, with 12/12 corpus files present and no SKIP. These all ran and passed:
    - the dnglab pins: `plane_md5_matches_the_pinned_raw_checksum`, `dnglab_checksum_matches_strips` and `dnglab_checksum_matches_tiles_with_edge_tiles`;
    - `units_assemble_in_any_order_on_held_files`;
    - the develop and orientation pins.

    `git diff 2e48dd7 -- tests/` shows only `tests/plane_groups.rs` (new) and `inject_chunk_extraction_fault`; only its needle, replacement and doc comment changed. `u16::from_le_bytes(*chunk)` and `u16::from_be_bytes(*chunk)` each occur once, in `unpack_byte_aligned`. The `plane_validate` guards and the `plane_tiles` scans pass unedited.
  - **AC2:** `tests/plane_groups.rs` packs with its own MSB-first packer and never calls the crate to pack. It covers 12- and 14-bit, widths 1–17 and heights 1–5:
    - one strip, and strips of 1 and 2 rows;
    - bands at `max_band_rows` 1 and 2 through `unpack_unit_into`;
    - clipped tiles 4 and 8 wide and 1–3 long.

    So every tail residue appears at a row end and at a strip end.
  - **AC3:** `groups_keep_bitreader_errors_and_partial_writes` compares the `Result` and the whole `dst`, poison included, for every strip length up to the exact one. It catches the sub-slice mutant on every residue. Once at least one group has been decoded (any count of 4 or more at 14-bit), the tail's error `at` is the group boundary plus the local offset, and the sub-slice reader drops the boundary. That holds for residues 0–3 alike.
  - **AC4:** the needle `(v.checked_shr(42), v.checked_shr(28))` occurs once in `src/plane.rs`, at line 307, in the live body of `unpack_groups14` and not in a comment. The swap trades samples 0 and 1 of each group, so it changes values but not the bit count.
    - Both red-proofs passed in my `just test`, and so did both controls: `the_fixture_oracle_control_is_green_without_a_corpus` and `the_honest_control_is_green`.
    - The build's handback lists honest and mutant digests that differ (fixture d1d8… vs ca91…, L1021223 cb65… vs 8773…). I did not re-print them; the `assert_ne!` passing is the same evidence.
  - **AC5:** the diff adds no `unsafe` and no allocation; `plane_tiles` and `plane_units_alloc` are green unedited.
    - `BitReader`, `pow2`, `unpack_byte_aligned`, `unpack_packed`, `decode_unit` and `decode_tile_row` have no diff hunks.
    - No item is `pub`. Nothing indexes; the arithmetic is checked and the shifts use `checked_shr`.
    - `as_chunks` and `as_chunks_mut` pass `just msrv` (1.90); `as_chunks` was already in use.
  - **AC6:** reproduced back to back (table below): 9.9 ms against the 30 ms target.
  - **AC7:** green: lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, msrv, cost-audit and gate-audit. `decisions-audit --changed` is advisory and exited 0. `fuzz-plane 60` ran 3,241,476 runs clean. `gh pr checks 238` shows 44/44 pass on 90c4bc4.
- **For `verify`:** ✅ **APPROVED at 90c4bc4** (the PR #238 head; the code is 7b15289).

**Timing.** `just unpack-timing` on a Mac16,8 (Apple M4 Pro), release, one warm-up then the median and min of 7, in ms. The before tree is 2e48dd7 plus only `examples/unpack-timing.rs`, in its own target dir; the two runs were back to back.

| case | before median / min | after median / min |
|---|---|---|
| `unpack_into`, 1 thread | 233.4 / 232.9 | **9.9 / 9.5** |
| 8 units of 704 rows, 8 threads | 30.8 / 30.7 | 1.3 / 1.3 |

This agrees with the build's 239.0 → 9.7 and 31.2 → 1.4, and the CHANGELOG figures are honest.

**Mutants.** Each mutant was applied by script to a `git archive` copy of HEAD with its own target dir. Each ran tier A only, with `IRRADIANCE_CORPUS_DIR` unset: `--lib`, plus `plane_groups`, `plane_unpack`, `plane_units`, `plane_tiles`, `plane_validate` and `plane_oracle`. The unmutated control was green.

| mutant | result (tier A) |
|---|---|
| 14-bit shifts 42 and 28 swapped | red: AC2 ×3, AC3, the fixture MD5, the SPEC-068 tier-A any-order test and the tile tests |
| 14-bit mask `0x1FFF` | red: AC2 ×3, AC3 and the tile and unit tests |
| 12-bit first shift 11 | red: AC2 ×3, AC3 and the tile and unit tests |
| 12-bit sent to the 14-bit group | red: AC2 ×3, AC3 and `validate_and_unpack_agree_on_mutated_layouts` |
| tail one byte after the groups | red: AC2 ×3, AC3 and `a_two_strip_14bit_plane_respects_per_strip_byte_padding` |
| tail handed a sub-slice | red: AC3 only (expected: the values match, `at` differs) |
| `full` minus one | **green**: benign (see below) |
| 14-bit stride of 6 bytes, with matching arithmetic | red: AC2 ×3, AC3 and the fixture MD5 |
| 12-bit mask `0x07FF` | red: AC2 ×3, AC3 and the tile and unit tests |
| `full` from `strip.len()` alone | red: clipped tiles only (AC2 tiles, plus the `plane_tiles` and SPEC-068 tiled tests) |
| group path also taken for 16-bit | **green**: benign, because it is unreachable |

- **`full` from the strip alone.** A clipped tile row still never writes or reads past its window: the group loop zips against `dst.as_chunks_mut`, so writes stay inside `dst`. The tail is what breaks. `split_at_mut_checked` sees a boundary past `dst`, returns `Ok(())`, and leaves the tail samples unwritten (poison). AC2's clipped tiles turn red.

  Strips cannot show this mutant. Byte padding never adds a whole group, so in a strip `strip.len() / group_bytes` never exceeds `dst.len() / per_group`. That is why only tiles catch it, and they do on tier A.
- **The group path for 16-bit.** `unpack_packed` sends every multiple of 8 to `unpack_byte_aligned`, so `unpack_bitstream` never sees 16. Nothing can notice, and nothing needs to.
- **`full − 1`.** It is benign, but `unpack-timing` does not guard it. It moves one group (4 samples) per call onto `BitReader`, which is invisible in a 47 MP timing. The example would only catch a gross regression, such as `full` forced to 0, which puts the whole plane back at about 233 ms. The output is identical, so the one-group case needs no guard.

### Cost self-report

- **Tokens (total):** 4,912,796 (a floor: measured before the commit, push and PR-comment turns)
- **Estimated USD:** 3.81 (opus-5-5 at $5/$25 per MTok; cache reads ×0.10; cache writes at the 1-hour rate, ×2.0)
- **Duration (minutes):** about 55
- **Source of the number:** transcript `6b90664d-1770-409c-a9e3-f2f21059effa.jsonl` (the scratchpad UUID), deduplicated by message.id with the max output per message. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the code under review. The build's `byte_pos` assignment is one of §2's two permitted options, and `BitReader` is byte-for-byte unchanged.
- **Follow-up work identified:**
  - [XS] The stale assert message in `inject_chunk_extraction_fault` ("BitReader::read's chunk extraction"). Leaving it was right: spec §3 says every assertion stays unedited, and the message prints only if the needle moves. Reword it in a later chore.
  - [XS] AGENTS.md §6 lists neither `orient-timing` nor `unpack-timing` (the build's note). I agree.
  - The spec's red-proof list calls the timing example the guard for `full − 1`. It is not one, and nothing needs to be, since that mutant gives identical output. Future specs should call such mutants "benign, unguarded".

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The handoff's "`full` from `strip.len()` alone" left the variant open; I mutated the group functions' return value, the only place `full` combines the two lengths.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That strip byte padding can never add a whole group, so only clipped tiles exercise the `dst`-bounded side of `full`. The tests already cover it; the spec could say why tiles are load-bearing there.

3. **If you did this task again, what would you do differently?**
   — The same: start the 25-minute suite and the mutant sweep (on an archived copy) at once, and hold the timing pair until the CPU is quiet.
