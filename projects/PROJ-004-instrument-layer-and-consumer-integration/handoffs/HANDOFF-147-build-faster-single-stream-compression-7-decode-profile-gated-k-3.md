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
  id: HANDOFF-147
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 13803286           # REAL combined count — what cost-audit reads
  estimated_usd: 10.10             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 105
  branch: claude/spec-070-build
  pr: https://github.com/jysf/irradiance/pull/241
  completed_at: 2026-11-14         # YYYY-MM-DD
  notes: "FLOOR, measured while the session was live from transcript a1b7751f-6cb2-4577-8a7f-b88e4aff297d.jsonl (72 unique message.ids, all claude-opus-5-5, no subagents): input 150 + cache-creation 182,020 (1-hour) + cache-read 13,561,352 + output 59,764 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Re-sum at ship."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-147: BUILD: faster Compression 7 decode, a buffered scan reader and an 11-bit Huffman lookahead (SPEC-070)

## Delegation Summary

You are the **builder** for SPEC-070. Cut `claude/spec-070-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-065
and this handoff.

**The task.** In `src/ljpeg.rs`:
- `ScanReader` buffers whole destuffed data bytes in a `u64`, and its fill **stops
  silently** at a marker or EOF;
- `HuffTable` gains two 2,048-entry tables:
  - `lookup`: code length and symbol;
  - `fast`: total bits and the final difference;
- `decode_difference` answers from those tables when every bit it needs is buffered, and
  otherwise falls back to today's unchanged bit-at-a-time path, having consumed nothing;
- `restart` gives back the whole buffered bytes the old reader had not fetched.

- **Hard criteria:** samples and every `Result` (error fields included) are byte-
  identical.
- **Targets:** K-3 III ≤ 150 ms (from 704–708) and Typ 246 ≤ 140 ms (from 601–606). The
  probe measured 108 and 95 ms.
- **Also deliver:**
  - a committed timing example, `just ljpeg-timing`;
  - two new tier-A unit tests against a test-local copy of today's reader.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-070-faster-single-stream-compression-7-decode-profile-gated-k-3.md`.
  Read all of it:
  - §1, the probe, profile and differential;
  - §2, the reader, and §3, the tables and the decode order;
  - §4, the constraints, and §5, timing;
  - AC1–AC7, the red-proof list, and the Notes on the source-scanning tests.
- **Decision:** `./decisions/DEC-065-the-lossless-jpeg-scan-reader-buffers-whole-bytes-and-decodes-through-an-11-bit-lookahead-table.md`.
- **Also:**
  - DEC-056 (the tile window);
  - DEC-037 (two-component scans);
  - DEC-050 (EOI optional);
  - DEC-018 (modular arithmetic);
  - DEC-016 (no allocation);
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/ljpeg.rs`: all of `HuffTable`, `ScanReader`, `extend`, `pow2`, both reconstruct
    loops, the decode entry points, and `mod tests` (`BitWriter`, `encode_scan`,
    `encode_lj92`, the segment builders);
  - `tests/plane_oracle.rs`: `DECODABLE_COMPRESSED` and the Compression 7 red-proof;
  - `tests/plane_tiles.rs`: the `Window`…`open_scan` allocation scan;
  - `tests/plane_units_alloc.rs`;
  - `fuzz/fuzz_targets/ljpeg.rs`, whose header has the run command;
  - `examples/unpack-timing.rs` and `app.just`.
- **Probe reference:** the design PR's description has the probe's diff. It is
  **reference only**; rewrite it lint-clean.
- **Toolchain:**
  - AGENTS.md §5–6 and §16;
  - `guidance/toolchain-brief.md`;
  - `just test` already passes `--all-features`; add no flag;
  - for tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo;
  - the full suite takes about 25 min; run it in the background. **Run the ljpeg fuzz
    target for 60 s before it.**

## Expected Deliverables

1. **The baseline first.**
   - Write `examples/ljpeg-timing.rs` and the `app.just` recipe (spec §5).
   - Run it on `origin/main` with only the example added, and keep the output.
2. **Tests** (AC2, AC3).
   - Write the `ReferenceReader`, a verbatim copy of today's reader in `mod tests`.
   - Write the random-table encoder and both unit tests.
   - They pin behaviour, so they pass on today's code. That is expected; record it, and
     record the outcome counts.
3. **The change** (spec §2–§3):
   - lint-clean;
   - no `unsafe`, no allocation, no new dependency;
   - the slow path unchanged and reached having consumed nothing.
4. **Red-proofs.** Apply each mutation in the spec's list by hand, show the named test
   red, and revert. Report the two benign mutants green, and name the timing example as
   their only guard. For the restart give-back mutant, say how the generator put a
   stuffed byte before a marker.
5. **Timing.**
   - Run `just ljpeg-timing` after the change, back to back with item 1's baseline.
   - Record both tables and the machine in the handback, and the CHANGELOG line.
6. **Docs.**
   - CHANGELOG `[Unreleased]` Changed.
   - Every `src/ljpeg.rs` doc that describes the one-byte, bit-at-a-time reader.
7. **Gates, all green with the corpus env set:**
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

   Also:
   - `just decisions-audit --changed`;
   - the ljpeg fuzz target, 60 s;
   - `just fuzz-plane 60`.
8. **PR.** Open it against `main` from `claude/spec-070-build`, referencing SPEC-070,
   DEC-065 and HANDOFF-147. Confirm CI is green on its head.

## Out of Scope

- SIMD intrinsics, `unsafe`, and new dependencies.
- Threads inside irradiance.
- Prediction or reconstruction speed-ups.
- Restart-interval units.
- Any output or error change.
- A CI timing gate.
- A `just fuzz-ljpeg` recipe.
- Speeding up the slow tier-B tests (signal `slow-tier-b-tests-in-debug`).

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). MD5 strings and timings
  are fine.
- **If any existing plane, ljpeg or develop test goes red, the change altered behaviour.**
  Fix the change, never the test. There is no permitted test edit in this spec.
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

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

- **Branch / PR:** `claude/spec-070-build` (worktree branch `claude/spec-070-build-c45abc`), https://github.com/jysf/irradiance/pull/241, cut from a97345f.
  - 8da45d7 adds the timing example, the recipe and the two tests, on the unchanged reader.
  - 916d89d is the change.
  - 301cfb8 is the CHANGELOG line.
  - This handback commit follows.
- **Completed at:** 2026-11-14
- **All acceptance criteria met?** Yes.
  - **AC1:** `just test` with `IRRADIANCE_CORPUS_DIR` set ran 37 binaries: 697 passed, 0 failed, 1 ignored, with no SKIP lines. The dnglab `raw_checksum` pins for all five `DECODABLE_COMPRESSED` files, the units tests and the develop pins are green and unedited. `q2m_units_allocate_nothing` did not flake.
  - **AC2:** `buffered_reader_matches_the_bit_at_a_time_reader` ran 42,031 cases; the whole `ljpeg` unit set, both new tests included, takes about 2.8 s in debug. Outcomes:
    - Ok 2,775;
    - Truncated 22,103;
    - LjpegUnexpectedMarker 16,086;
    - LjpegInvalidHuffmanTable 1,067.

    The test asserts that every class is reached. It covers 300 random canonical tables (a quarter with 16-bit codes, another quarter topping out at 12–15 bits, and symbols 0–16 plus about 1 in 20 above 16), with `Ri` 0, 1, 2, 3 and 7.
  - **AC3:** `decode_into_is_unchanged_on_lj92_streams_with_restarts` ran 17,567 cases:
    - Ok 4,155;
    - Truncated 8,585;
    - LjpegUnexpectedMarker 4,130;
    - LjpegInvalidHuffmanTable 697.

    **Choice:** a test-local reconstruction, `reference_reader_decode_into`. It is a copy of `decode_into` → `reconstruct` → the dense `reconstruct_window` for Nf=1. It calls the production `check_dense_length`, `check_window`, `initial_prediction`, `neighbour`, `predict` and `apply_point_transform`; only the reader differs. No test-only seam was added to production code.
  - Both tests passed on the unchanged reader (commit 8da45d7), as the spec expects.
  - **AC4:** all 40 existing `ljpeg` unit tests are green and unedited. `ljpeg_shape_binding`, `plane_tiles` and the Compression 7 red-proof and its control are green too. `neighbour(out, slot, 2)` still occurs twice, because `predict_component` is untouched.
  - **AC5:** no allocation, no `unsafe`, no new dependency. `plane_tiles`' scan and `plane_units_alloc` are green and unedited.
  - **AC6:** K-3 III 103.2 ms (target ≤ 150) and Typ 246 90.7 ms (target ≤ 140); the table is below.
  - **AC7:** every gate is green with the corpus set: `just test`, lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, msrv, cost-audit and gate-audit.
    - The ljpeg fuzz target (the header command) ran 2,041,085 runs in 61 s, clean. It needed `mkdir fuzz/corpus/ljpeg`, which is gitignored.
    - `just fuzz-plane 60` ran 1,504,242 runs, clean.
    - `just decisions-audit --changed` is advisory. It named DEC-050 and DEC-052 through CHANGELOG.md. Both still hold: nothing allocates beyond today, and no public item changed.

**Timing.** `just ljpeg-timing` on a Mac16,8 (Apple M4 Pro), release, one warm-up then the median / min of 7, in ms. The before run is 8da45d7 (a97345f plus only the example, the tests and the recipe), built in a temporary worktree. The two runs were back to back.

| file | before | after |
|---|---|---|
| PENTAX-K3III-MONO/K3III.DNG (6304 × 4224) | 692.1 / 688.3 | **103.2 / 100.9** |
| LEICA-M-MONOCHROM-TYP246/M2462362.DNG (5984 × 4000) | 590.8 / 590.4 | **90.7 / 88.4** |
| GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng (4032 × 3024) | 323.7 / 320.6 | 60.8 / 60.5 |

- An earlier baseline run on the same commit gave 697.3, 597.7 and 328.7.
- With `#[inline]` rather than `#[inline(always)]` on `decode_difference` and `decode_buffered`, the after figures were 118.7, 103.3 and 70.1. So `#[inline(always)]` stays, as in the probe.
- The CHANGELOG line uses the back-to-back pair.

**Red-proofs.** Each mutant was applied by hand with perl to the final source and then reverted, and `src/ljpeg.rs` was compared byte for byte afterwards. The script and log are local.

| mutant | red |
|---|---|
| `fast` consumes `total − 1` bits | AC2, AC3 and 10 more existing `ljpeg` tests. **AC1:** `plane_md5_matches_the_pinned_raw_checksum` fails, Typ 246 giving `LjpegInvalidHuffmanTable` |
| `fill` treats `0xFF xx` as a data byte | AC2, AC3, `malformed_inputs_are_typed_errors_never_panics` |
| `restart` gives back 1 source byte for a stuffed byte | AC2 (restart result, an `LjpegUnexpectedMarker` naming a different byte), AC3 |
| `total <= nbits` guard dropped | AC2, AC3, `hostile_bitstreams…`, `malformed_inputs…`, `two_component_truncated_scan_is_typed_error` |
| `SSSS` 16 stored in `fast` as −32768 | AC2, AC3, `clipped_window_stops_after_the_last_visible_row` |
| `SSSS` 16 stored in `fast` as 0 | AC2, AC3, `clipped_window_stops_after_the_last_visible_row` |
| `l + s <= nbits` check dropped in the `lookup` step | AC2, AC3 |
| `read_bit` returns `Truncated { at: byte_pos }` instead of `fetch_byte`'s error | AC2, AC3, `malformed_inputs…` |
| **benign:** `LUT_BITS` 11 → 10 | green everywhere; only `just ljpeg-timing` guards it |
| **benign:** fast path disabled (every symbol takes the slow path) | green everywhere; only `just ljpeg-timing` guards it |

- **How the restart give-back mutant was reached.** `encode_random_stream`'s `junk` option puts whole data bytes after an interval's last symbol and before its `RSTm`, in about a third of the intervals of half the streams. Those bytes are `[x, FF, 00]`, `[FF, 00]` or `[FF, 00, x]`, with x ≠ 0xFF. The decoder has not read them at the restart, so they are the whole buffered bytes `restart` must give back. The marker check then fails with `LjpegUnexpectedMarker`, whose `marker` names the byte `restart` landed on: x correctly, and 0x00 under the mutant.
- The mutant is also red with `junk` forced off, because corrupted streams (bit flips, an injected `0xFF`) leave stuffed bytes unread before a marker naturally.

### Cost self-report

- **Tokens (total):** 13,803,286 (a floor, measured while the session was live)
- **Estimated USD:** 10.10
- **Duration (minutes):** about 105
- **Source of the number:** transcript `a1b7751f-6cb2-4577-8a7f-b88e4aff297d.jsonl` (found by the scratchpad UUID), deduplicated by message.id with the max output per message, at opus-5-5 rates. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **The `fast` encoding** is the total bits in the high half and `diff + 32767` in the low 16 bits (`FAST_DIFF_BIAS`). This differs from the probe's `diff as u16`: it needs no `as` cast, maps −32767..=+32768 onto 0..=65535, and leaves 0 free for "no entry", since every total is at least 1.
  - **The `lookup` and `fast` tables are built** by a private free function, `build_lookahead`, called at the end of `HuffTable::build`. `LUT_SIZE` and `LUT_MASK` are derived from `LUT_BITS` with const `checked_shl`.
  - **The optional 8-byte refill is included.** It is exact: it stops where the byte loop would, with the stuffed flags clear.
  - **`decode_difference` and `decode_buffered` are `#[inline(always)]`.** The spec said `#[inline]`, but `#[inline]` measured about 15% slower (see Timing).
  - **The `pow2` doc** no longer says the bit reader uses it. `read_bit` uses `take(1)`; `pow2` itself is unchanged.
  - **The probe's 28,880-case corpus differential was not re-run.** The tier-A tests, the five dnglab pins, both fuzz targets and the red-proofs stand in for it.
- **Follow-up work identified:**
  - [XS] **Add a `just fuzz-ljpeg` recipe** that creates `fuzz/corpus/ljpeg` first. The header command fails on a fresh checkout because that directory does not exist. This is already a spec non-goal.
  - [XS] **AGENTS.md §6 lists no timing examples.** It now omits three: orient, unpack and ljpeg. This repeats SPEC-069's follow-up.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The probe diff, the red-proof list and the field-name notes made it transcription. The only gap was the `fast` encoding without `as`, which the spec explicitly left to the builder.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Possibly the inlining attribute. The spec specified `#[inline]`, but the probe's 108 ms depended on `#[inline(always)]`. With plain `#[inline]` the target was still met, but about 15% slower.

3. **If you did this task again, what would you do differently?**
   — I would write the mutant script before the change, with the needles chosen from the spec's list. Applying all ten mutants and the AC1 run took about 10 minutes, and ordering them before the 25-minute suite (whose tier-B red-proofs read `src/ljpeg.rs`) avoided a race.
