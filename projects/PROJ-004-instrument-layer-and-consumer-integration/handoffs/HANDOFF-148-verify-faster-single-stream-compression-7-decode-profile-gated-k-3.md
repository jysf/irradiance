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
  id: HANDOFF-148
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-14
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-070

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 14633694               # REAL combined count — what cost-audit reads
  estimated_usd: 10.22               # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-070-verify-93c388
  pr: https://github.com/jysf/irradiance/pull/241
  completed_at: 2026-11-14               # YYYY-MM-DD
  notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (81 unique message.ids, all claude-opus-5-5, no subagents): input 168 + cache-creation 171,141 (1-hour) + cache-read 14,410,448 + output 51,937, same rates. APPROVED at ff9137e, no code changes. The handback floor was 13,278,178 and $9.39."
  synced_at: 2026-11-14
---

# HANDOFF-148: VERIFY: faster Compression 7 decode, a buffered scan reader and an 11-bit Huffman lookahead (SPEC-070)

## Delegation Summary

You are the **cold verifier** for SPEC-070. You did **not** implement it. Review it
against SPEC-070 and DEC-065, and return exactly one verdict.

- **The build:** **PR #241**, branch `claude/spec-070-build`, cut from `origin/main` at
  `a97345f`. Its head is `ff9137e`. The commits are:
  - `8da45d7`: the timing example, the recipe, and the two new tests on the **unchanged**
    reader;
  - `916d89d`: the change;
  - `301cfb8`: the CHANGELOG line;
  - `ff9137e`: the handback.
- **Your branch:** your worktree is cut from `claude/spec-070-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers, in `src/ljpeg.rs`:
- **A buffered reader.** `ScanReader` buffers whole destuffed bytes in a `u64`, with
  per-byte stuffed flags. Its fill stops silently at a marker or EOF, and it has an
  optional 8-byte refill.
- **Two lookahead tables.** `HuffTable` gains `lookup` and `fast` (11 bits each), built
  by `build_lookahead`.
- **A two-path decode.** `decode_difference` and `decode_buffered` are
  `#[inline(always)]`, and today's bit-at-a-time path is the cold fallback.
- **Restart give-back.** `restart` gives back the unread whole buffered bytes.
- **Tests:**
  - a verbatim `ReferenceReader` in `mod tests`;
  - a random canonical-table stream encoder;
  - `buffered_reader_matches_the_bit_at_a_time_reader` (AC2);
  - `decode_into_is_unchanged_on_lj92_streams_with_restarts` (AC3).

It also adds `examples/ljpeg-timing.rs`, the `ljpeg-timing` recipe and a CHANGELOG line.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-070-faster-single-stream-compression-7-decode-profile-gated-k-3.md`.
  Read all of it: the §1 probe, §2 the reader, §3 the tables and the decode order, §4 the
  constraints, §5 timing, AC1–AC7, the red-proof list and the Notes.
- **Decision:** `./decisions/DEC-065-the-lossless-jpeg-scan-reader-buffers-whole-bytes-and-decodes-through-an-11-bit-lookahead-table.md`.
  Also DEC-056, DEC-037, DEC-050, DEC-018, DEC-016 and DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-147-build-faster-single-stream-compression-7-decode-profile-gated-k-3.md`.
  Treat its claims as claims. The points to judge:
  - **The "consume nothing before the slow path" rule.** It is what makes errors
    identical. Read `decode_difference` / `decode_buffered` line by line, and confirm
    that no path mutates `acc`, `nbits`, `byte_pos` or the flags and then falls through to
    the slow path.
  - **The 8-byte refill was optional, and the probe never proved it.** It is not part of
    the 28,880-case corpus differential. Confirm that it:
    - takes only bytes the byte loop would take;
    - stops at the same `byte_pos`;
    - records clear stuffed flags;
    - never takes a byte when fewer than 8 bits are free.

    Then mutate it yourself (below).
  - **The `fast` encoding deviates from the probe:** `FAST_DIFF_BIAS`, with
    `diff + 32767` in the low 16 bits. Confirm that −32767 and +32768 both round-trip,
    and that no reachable entry encodes as 0 ("no entry").
  - **`#[inline(always)]` deviates from the spec's `#[inline]`.** The build measured
    `#[inline]` about 15% slower. Judge whether the deviation is justified. Check that the
    attributes compile under MSRV and pass `lint-ci`.
  - **AC3's test-local reconstruction** (`reference_reader_decode_into`) copies the
    dense single-component path and calls production helpers. Judge whether it is an
    independent old-behaviour reference. It must differ only in the reader. It covers
    `Nf=1` only; judge whether the two-component path needs the same treatment, given AC2
    and the dnglab pins.
  - **The `junk` generator** puts whole data bytes, stuffed pairs among them, between an
    interval's last symbol and its `RSTm`. That is what makes the restart give-back
    mutant red. Confirm it really reaches a restart with unread stuffed bytes buffered.
    Confirm too that the build's outcome counts (AC2: 42,031 cases; AC3: 17,567) are what
    the tests print.
  - **The probe's corpus differential was not re-run.** You may run your own local copy:
    link today's `src/ljpeg.rs` (from `a97345f`) as a second module, and compare
    `decode_into` / `decode_window_into` Results and buffers on mutated real tiles and
    strips. **Never commit it, and never commit corpus bytes (DEC-003).** It is optional
    but valued; say whether you ran it and what it found.
  - **Timing (AC6).**
    - Reproduce `just ljpeg-timing` on `8da45d7`, which is the before case: the example
      on today's reader.
    - Then reproduce it on the build head, back to back.
    - The build reports K-3 III 692.1 → 103.2 ms, Typ 246 590.8 → 90.7 and Pixel 4a
      323.7 → 60.8. The targets are 150 and 140 ms.
- **Toolchain:**
  - AGENTS.md §5–6 and §16, and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - The full suite takes about 25 min; run it in the background.
  - Run mutants on an **archived copy** of the tree with its own target dir, so the
    source-copying red-proof tests never stage a mutated `src/ljpeg.rs`.
  - The ljpeg fuzz target needs `mkdir -p fuzz/corpus/ljpeg` first (gitignored). Then use
    the command in `fuzz/fuzz_targets/ljpeg.rs`'s header.
  - `tests/plane_units_alloc.rs`'s `q2m_units_allocate_nothing` has a known
    parallel-test race. If it flakes, rerun it alone and record that.

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
- Run `just decisions-audit --changed`, the ljpeg fuzz target for 60 s, and
  `just fuzz-plane 60`.
- Observe CI green on the PR head with `gh pr checks 241`.

### The behavioural surface (this is where defects escape)

**Byte and error identity are the hard criteria (AC1–AC4).**
- **No existing test edited.** `git diff a97345f -- tests/` must be empty. In
  `src/ljpeg.rs`'s `mod tests`, the existing 40 tests must be unchanged: diff that
  module, and confirm that only additions appear.
- **Every pin ran.** The five dnglab pins (`DECODABLE_COMPRESSED`), the units tests and
  the develop pins must have **run** (no SKIP) and passed.
- **Source-scan needles still hold:**
  - `neighbour(out, slot, 2)` occurs exactly twice, in live code;
  - `plane_tiles`' `Window`…`open_scan` allocation scan stays clean.
- **Scope.** `fetch_byte`, `decode_symbol`, `receive`, `receive_and_extend`, `extend`,
  `parse_headers` (apart from the tables) and every reconstruction function are
  unchanged. Use `git diff a97345f -- src/ljpeg.rs`.

**Mutate it yourself**, outside any test knob, and run **tier A only**:
- the build's 10;
- the 8-byte refill checking only 7 of its 8 bytes for `0xFF`;
- the 8-byte refill leaving the previous stuffed flags in place instead of clearing the
  new ones;
- `fill`'s threshold changed from `nbits <= 56` to `nbits <= 64` (does anything overflow
  or drop bits?);
- `FAST_DIFF_BIAS` off by one;
- `build_lookahead` replicating a code over one index too few (span − 1);
- `restart` not clearing the stuffed flags;
- `decode_difference`'s top-up threshold changed from 32 to 0, so the buffer is filled
  only when empty. Is that only slower, or can a valid long code now take the slow path
  at the wrong point?

A mutant that stays green, or that only tier B catches, is a finding unless you judge it
benign and say why.

**No allocation, no panics, scope.**
- No `unsafe` and no allocation; the tables are arrays inside `HuffTable`.
- No indexing; checked or wrapping arithmetic only.
- No public API change.
- The CHANGELOG states the before and after numbers honestly.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #241.
  - Commit the filled HANDOFF-148 handback (and the timeline's verify line) on a branch
    named `claude/spec-070-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #241 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #241.
  - Reference HANDOFF-148, SPEC-070, DEC-065 and PR #241.
- **Rejected:** comment on #241 with the reasons, and push the handback branch.

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

- SIMD, `unsafe`, and threads inside irradiance.
- Prediction speed-ups and restart-interval units.
- Speeding up the slow tier-B tests (signal `slow-tier-b-tests-in-debug`).
- The build's two [XS] follow-ups: a `just fuzz-ljpeg` recipe that creates its corpus
  dir, and timing examples in AGENTS.md §6. Note them if you agree.
- The `plane_units_alloc` race (already an XS in the stage backlog).

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

- **Branch / PR:** the verify branch is `claude/spec-070-verify-93c388`, cut from `claude/spec-070-verify-base` (ff9137e plus the brief). It holds this handback and the SPEC-070 timeline's verify line, and no code. The PR under review is https://github.com/jysf/irradiance/pull/241. No superset PR was opened.
- **Completed at:** 2026-11-14
- **All acceptance criteria met?** Yes. AC7's `just test` met it with the known `plane_units_alloc` race recorded, not hidden; see the finding below.
- **For `verify`:** ✅ **APPROVED at ff9137e** (the PR #241 head; the code is 916d89d), with no code changes.

**Gates** (corpus set, 12/12 present, no SKIP line):

- **`just test`:** 697 passed, 0 failed and 1 ignored over 37 binaries plus the doctests. That matches the build's count.
  - The suite stopped at `plane_units_alloc`, where `q2m_units_allocate_nothing` failed: 399 allocations, a concurrent test's counted allocations landing in the Q2M window.
  - I reran that binary alone and serially. The test passed alone, and 3 of 3 serial runs passed.
  - I ran the nine binaries after it, and the doctests, with `--no-fail-fast`. All of them passed.
- **The other gates:**
  - lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, msrv, cost-audit and gate-audit are all exit 0.
  - `decisions-audit --changed` is advisory, exit 0.
- **Fuzz:**
  - ljpeg (the header command, after `mkdir -p fuzz/corpus/ljpeg`): 2,440,507 runs in 61 s, clean.
  - `just fuzz-plane 60`: 1,401,371 runs, clean.
- **CI:** `gh pr checks 241` shows 44/44 pass on ff9137e.

**Finding (recorded; not a SPEC-070 code defect): the `plane_units_alloc` race is now reproducible.**

- **The counts.** `cargo test --test plane_units_alloc` with the corpus failed 3 of 3 at the build head (194, 237 and 200 allocations), and once more inside `just test`. It passed 3 of 3 at 8da45d7 (the same tests on the old reader).
- **Why it is the race and not an allocation:**
  - the binary passes serially, 3 of 3;
  - the Q2M test passes alone;
  - Q2M is uncompressed and never reaches `ljpeg`.

  `ALLOCATIONS` is one global counter, and `units_and_unit_decode_allocate_nothing` counts its Compression 7 fixtures' expected Huffman-table allocations on another thread. My guess is that the debug-build cost of `build_lookahead` (two 2,048-entry tables per `HuffTable`) lengthens those counted windows enough to overlap Q2M's window every time.
- **What it means.** Locally, `just test` with the corpus is now reliably red until the backlog XS lands. CI has no corpus, so it stays green.
- **Recommendation:** promote that XS (count per thread, or serialise the binary) and do it before the next spec gated on `just test`. Fixing it was out of scope here.

**The behavioural surface:**

- **No existing test edited.** `git diff a97345f -- tests/` is empty, and 8da45d7's diff to `src/ljpeg.rs` has zero removed lines. The only test-module line that 916d89d changes is in the new `outcome` helper, added by 8da45d7.
- **Every pin ran and passed.** `plane_md5_matches_the_pinned_raw_checksum` (all five `DECODABLE_COMPRESSED`), `ljpeg_decode_fault_turns_compression7_oracle_red` and its control, `units_assemble_in_any_order_on_held_files`, and the develop suites all ran with 12/12 corpus present and no SKIP.
- **The needles hold.** `neighbour(out, slot, 2)` occurs exactly twice, and `the_tile_path_allocates_nothing` is green.
- **Scope.** `fetch_byte`, `decode_symbol`, `receive`, `receive_and_extend`, `extend`, `pow2` (only its doc changed), `parse_headers` and every reconstruction function have no hunks. `decode_difference`'s old body moved verbatim into `#[cold] decode_bit_at_a_time`.
- **The AC2 and AC3 counts reproduce exactly.**
  - AC2: 42,031 cases (Ok 2,775; Truncated 22,103; LjpegUnexpectedMarker 16,086; LjpegInvalidHuffmanTable 1,067).
  - AC3: 17,567 cases (Ok 4,155; Truncated 8,585; LjpegUnexpectedMarker 4,130; LjpegInvalidHuffmanTable 697).

**The points to judge:**

- **Consume nothing before the slow path: holds.**
  - In `decode_buffered`, the only mutation before a `None` is `fill()`. That buffers only bytes `fetch_byte` would return, and stops exactly where it would fail.
  - Every consuming branch (`fast`, and `lookup` with `SSSS` 0, 16, or 1–15 with `ssss <= after`) returns `Some`.
  - The `1..=15` guard is in the match arm, so a short tail or an `SSSS` above 16 falls to `_ => None` untouched.
  - `read_bit` errors only at `nbits == 0`, after the bit-at-a-time reader would have consumed the same bits, through `fetch_byte` at the same `byte_pos`.
- **The 8-byte refill: correct.**
  - It takes `whole = (64 − nbits) / 8` bytes, which is exactly what the byte loop takes from bytes with no `0xFF`. It takes none when fewer than 8 bits are free.
  - It requires all 8 bytes to be present and `0xFF`-free (an exact has-zero-byte test on `!word`), so it never takes a byte the loop would refuse.
  - It shifts the stuffed flags by `whole`, which clears the new flags.
- **The `fast` encoding: sound.**
  - `fast_entry` puts `total` (at least 1) in the high half, so no reachable entry is 0.
  - `FAST_DIFF_BIAS` maps −32767..=+32768 onto 0..=65535 losslessly.
  - In practice the only reachable `fast` differences are ±1,023 (`SSSS` ≤ 10, since `l ≥ 1`) and +32,768 (`SSSS` 16). That is why the downward off-by-one mutant is benign (below).
- **`#[inline(always)]`: justified.** I re-measured it: plain `#[inline]` gives K-3 III 117.9, Typ 246 103.5 and Pixel 4a 72.5 ms, against 105.5 / 91.6 / 65.3 with `always`, about 12% slower. It has no semantic effect, and lint-ci and msrv (1.90) are green.
- **AC3's `reference_reader_decode_into` is an independent old-behaviour reference.**
  - It is a faithful copy of `decode_into` → `reconstruct` → dense `reconstruct_window` for Nf=1: the same check order, the same `count == 0` initial prediction, the same neighbour and predict calls, and `& 0xFFFF`. Only the reader differs.
  - The two-component path needs no equivalent. Its reader use is the same per-call `decode_difference` and `restart`, with a table switch. The reader state does not depend on the table, so AC2 covers it, and the Pixel dnglab pins, `two_component_*` and my corpus differential exercise it too.
- **The `junk` generator reaches it.** Junk bytes follow `flush_to_byte` and precede `RSTm`, so they are whole bytes still buffered at the restart. The give-back mutant (b03) and my stuffed-flag mutant (v12) are red on AC2 and AC3.
- **The probe's corpus differential: I ran a local copy (not committed, and no corpus bytes copied into any tree).**
  - It was a scratch archive with a97345f's `src/ljpeg.rs` linked as `ljpeg_old`, and a test-only hook in `plane.rs` that captured every strip or tile slice and its window during `unpack_into` on the five `DECODABLE_COMPRESSED` files.
  - It compared the clean `decode_into` / `decode_window_into` (578 units), then 3,552 mutated copies (48 per strip and 6 per tile). The mutations were truncation, bit flips, an injected `0xFF`, an injected `RSTm` or other marker, a trailing lone `0xFF`, random-byte runs, deleted bytes and tail truncation, with 1 in 16 also hitting the headers.
  - It compared the `Debug` of the whole `Result` and the whole destination buffer.
  - **0 differences.** Outcomes: Ok 1,455; LjpegUnexpectedMarker 1,316; Truncated 764; LjpegInvalidHuffmanTable 16; LjpegMissingMarker 1.

**Mutants** (tier A only: `cargo test --all-features --lib ljpeg::` on an archived copy with its own target dir and no corpus env, applied with perl, original restored and compared):

| mutant | result |
|---|---|
| b01 `fast` consumes `total − 1` | red: AC2, AC3 + 19 more |
| b02 `fill` takes `0xFF xx` as data | red: AC2, AC3, `malformed_inputs…` |
| b03 `restart` gives back 1 for a stuffed byte | red: AC2, AC3 |
| b04 `total <= nbits` dropped | red: AC2, AC3, `hostile_bitstreams…`, `malformed_inputs…`, `two_component_truncated…` |
| b05 `SSSS` 16 as −32768 | red: AC2, AC3, `clipped_window_stops…` |
| b06 `SSSS` 16 as 0 | red: AC2, AC3, `clipped_window_stops…` |
| b07 `ssss <= after` dropped | red: AC2, AC3 |
| b08 `read_bit` returns its own `Truncated` | red: AC2, AC3, `malformed_inputs…` |
| b09 `LUT_BITS` 10 | green (benign, speed only) |
| b10 fast path disabled | green (benign, speed only) |
| v11 refill checks 7 of 8 bytes for `0xFF` | red: AC2, AC3, `clipped_window_matches…`, `full_window…`, `two_component_restart_is_pinned` |
| v12 refill keeps the old stuffed flags (no shift) | red: AC2, AC3 |
| v13 `fill` threshold `nbits <= 64` | red: AC2, AC3 + 10 more |
| v14a `FAST_DIFF_BIAS` 32766 | **green, benign** |
| v14b `FAST_DIFF_BIAS` 32768 | red: AC2 only |
| v15 `build_lookahead` span − 1 | **green, benign** |
| v16 `restart` does not clear the stuffed flags | **green, benign** |
| v17 top-up threshold 32 → 0 | **green, benign (slower only)** |

Why each green mutant is benign:

- **v14a.** The bias is applied consistently, so any bias in 1,023..=32,767 round-trips every reachable `fast` difference (±1,023 and +32,768). 32,766 differs only for −32,767, which no `fast` entry can hold. v14b breaks +32,768 and is red.
- **v15.** A missing table entry is "no entry", which takes the slow path having consumed nothing. Fewer entries cost speed only.
- **v16.** This is an equivalent mutant. `acc` and `nbits` are cleared at the restart, so give-back reads only flags `i < nbits/8`, all for bytes buffered since the restart. The stale flags sit above them and are never read.
- **v17.** The buffer then fills only in `read_bit` at `nbits == 0`. With 0 to 31 bits buffered, a code whose bits are not all there still takes the slow path, having consumed nothing. That path reads bit by bit, refills at 0 and decodes the same value, so no valid long code is misread. It is only slower.

**Timing** (`ljpeg-timing` release binaries, Mac16,8 / Apple M4 Pro, median / min of 7 in ms, 8da45d7 against ff9137e, back to back, two rounds):

| file | before (8da45d7) | after (ff9137e) |
|---|---|---|
| K-3 III (6304 × 4224) | 710.0 / 704.6; 714.5 / 711.2 | **105.0 / 104.0; 106.1 / 105.1** (target ≤ 150) |
| Typ 246 (5984 × 4000) | 604.8 / 598.8; 615.0 / 607.0 | **90.3 / 90.0; 92.3 / 90.6** (target ≤ 140) |
| Pixel 4a (4032 × 3024) | 335.8 / 331.6; 336.0 / 331.3 | 64.3 / 63.5; 64.5 / 63.9 |

This reproduces the build's 692.1 → 103.2, 590.8 → 90.7 and 323.7 → 60.8 within about 3–6%. My before figures are a little higher, so the CHANGELOG's ~6.5× is honest, if anything conservative.

**No allocation, no panics, scope.**
- **No `unsafe`.** The tables are fixed arrays inside `HuffTable`: 12 KiB per table, the same allocation count.
- **Lint-clean.** It uses `get`/`get_mut`, checked shifts and `try_from`, with no `as` or indexing (lint-ci green).
- **No public item changed.**
- **The CHANGELOG states the back-to-back before and after numbers.**

### Cost self-report

- **Tokens (total):** 13,278,178 (a floor, measured while the session was live)
- **Estimated USD:** 9.39
- **Duration (minutes):** about 60
- **Source of the number:** transcript `88aa37a9-5bcd-47ff-971d-609c1aede5e1.jsonl` (found by the scratchpad UUID). It is deduplicated by message.id with the max output per message, at opus-5-5 rates with 1-hour cache writes. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the verify. I agree with the build's four deviations: the `fast` encoding, `build_lookahead` as a free function, the 8-byte refill included, and `#[inline(always)]`.
- **Follow-up work identified:**
  - **[XS, promote] fix the `plane_units_alloc` race** (count per thread, or serialise the binary). It now fails every parallel local run with the corpus at this head (4 of 4, against 0 of 3 before), so a local `just test` is red until it lands.
  - **[XS] a `just fuzz-ljpeg` recipe** that creates `fuzz/corpus/ljpeg`. I agree: the header command needs the `mkdir`.
  - **[XS] timing examples in AGENTS.md §6.** I agree.
  - **[XS, optional] fix the `FAST_DIFF_BIAS` doc.** "−32767 round-trips" is true but vacuous, since no `fast` entry can hold it. It could say that the reachable `fast` differences are ±1,023 and +32,768.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The brief called the alloc race a flake to rerun alone. At this head it is no longer a flake, so I had to show it was not an allocation: the serial runs, the run alone, and the pre-change baseline.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The handoff's suggested mutants include several that are equivalent by construction: span − 1, not clearing the flags on a restart, a top-up threshold of 0, and a downward bias. It would help to say up front that the lookahead tables are accelerators whose missing entries are always safe, so mutants that only remove entries or fills are expected to be benign.

3. **If you did this task again, what would you do differently?**
   — I would run `just test` with `--no-fail-fast` from the start, or run the alloc binary first. The race stopped the suite at binary 27 of 37, and I had to rerun the rest separately.
