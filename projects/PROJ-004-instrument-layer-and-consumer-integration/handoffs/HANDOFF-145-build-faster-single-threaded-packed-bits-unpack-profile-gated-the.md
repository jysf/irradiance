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
  id: HANDOFF-145
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10989685            # REAL combined count — what cost-audit reads
  estimated_usd: 8.21              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-069-build
  pr: https://github.com/jysf/irradiance/pull/238
  completed_at: 2026-11-14
  notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (65 unique message.ids, all claude-opus-5-5, no subagents): input 133 + cache-creation 166,643 (1-hour) + cache-read 10,776,918 + output 45,991 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The handback floor was 9,728,429 and $7.35."
  synced_at: 2026-11-14
---

# HANDOFF-145: BUILD: faster packed 12/14-bit unpack, whole byte groups (SPEC-069)

## Delegation Summary

You are the **builder** for SPEC-069. Cut `claude/spec-069-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-064
and this handoff.

**The task.** `src/plane.rs`'s `unpack_bitstream` (12- and 14-bit, MSB-first) decodes
whole byte groups first:
- 14-bit: 7 bytes → 4 samples;
- 12-bit: 3 bytes → 2 samples.

Today's unchanged `BitReader` then reads the remaining tail, **positioned inside the same
slice**.

- **Hard criterion:** output and errors are byte-identical.
- **Target:** the Q2M single-thread `unpack_into` median is ≤ 30 ms, from 238–243 ms.
  The probe measured 9.7 ms.
- **Also deliver:**
  - a committed timing example, `just unpack-timing`;
  - a new tier-A round-trip test file;
  - one permitted retarget of the `plane_oracle` red-proof injector.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-069-faster-single-threaded-packed-bits-unpack-profile-gated-the.md`.
  Read all of it:
  - the §1 probe and profile;
  - §2, the kernel, and §3, the red-proof retarget (**the only permitted test edit**);
  - §4, timing;
  - AC1–AC7, the red-proof list, and the Notes on the source-scanning tests.
- **Decision:** `./decisions/DEC-064-the-packed-12-and-14-bit-unpack-reads-whole-byte-groups-and-bitreader-stays-for-the-tail.md`.
- **Also:**
  - DEC-063 (units, and where `unpack_packed` is called);
  - DEC-008 (the two unpack paths);
  - DEC-017 (the red-proof mechanism);
  - DEC-016 (no allocation);
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/plane.rs`: `BitReader`, `unpack_bitstream`, `unpack_packed`, `decode_unit`,
    `decode_tile_row`, and the existing unit tests;
  - `tests/plane_oracle.rs`: the module doc, `inject_chunk_extraction_fault`, the two
    red-proofs and their controls, and the byte-order needle test;
  - `tests/plane_unpack.rs` (`Fixture`, `MultiStrip`, `pack_msb`);
  - `tests/plane_units.rs` (the tiled builders);
  - `examples/orient-timing.rs` and its `app.just` recipe.
- **Toolchain:**
  - AGENTS.md §5–6 and §16. Rule 5 applies: AC2's oracle is the test's own packer, never
    the crate.
  - `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - The full suite takes about 25 min; run it in the background. **Run
    `just fuzz-plane 60` before it.**

## Expected Deliverables

1. **The baseline first.**
   - Write `examples/unpack-timing.rs` and the `app.just` recipe (spec §4).
   - Run it on `origin/main` with only the example added, and keep the output.
2. **Tests.**
   - Write `tests/plane_groups.rs` (AC2) and the `src/plane.rs` unit test (AC3).
   - These pin behaviour, so they pass on today's code. That is expected; record it.
   - The test-first evidence is the red-proof list (item 5).
3. **The kernel** (spec §2):
   - groups through `as_chunks` / `as_chunks_mut`;
   - lint-clean shifts and narrowing;
   - the tail `BitReader` positioned in the same slice;
   - no `unsafe`, no allocation, no new dependency.
4. **The retarget** (spec §3). In `inject_chunk_extraction_fault`, change only its
   `needle`, its replacement and its doc comment. The new fault swaps the 14-bit group's
   shifts 42 and 28. Show both red-proofs passing (`mutant != honest`) and both controls
   green.
5. **Red-proofs.** Apply each mutation in the spec's list by hand, show the named
   **tier-A** test red, and revert. Report the benign `full − 1` mutant as green, and say
   that the timing example is its only guard.
6. **Timing.**
   - Run `just unpack-timing` after the change, back to back with item 1's baseline.
   - Record both tables and the machine in the handback, and the CHANGELOG line.
7. **Docs.**
   - CHANGELOG `[Unreleased]` Changed.
   - Any `src/plane.rs` module or function doc that describes the per-sample loop.
8. **Gates, all green with the corpus env set:**
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

   Also run `just decisions-audit --changed` and `just fuzz-plane 60`.
9. **PR.** Open it against `main` from `claude/spec-069-build`, referencing SPEC-069,
   DEC-064 and HANDOFF-145. Confirm CI is green on its head.

## Out of Scope

- SIMD intrinsics, `unsafe`, and new dependencies.
- Threads inside irradiance.
- Compression 7 (SPEC-070).
- The 8- and 16-bit path.
- Editing `BitReader`.
- Any output or error change.
- A CI timing gate.
- Speeding up the slow tier-B tests (signal `slow-tier-b-tests-in-debug`).

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). MD5 strings and timings
  are fine.
- **If any existing plane test or develop pin goes red, the change altered output.** Fix
  the change, never the test. The single exception is spec §3's injector.
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

- **Branch / PR:** `claude/spec-069-build`, https://github.com/jysf/irradiance/pull/238. The change is 7b15289, cut from 2e48dd7. CI was 44/44 green on 7b15289; this handback commit follows it.
- **Completed at:** 2026-11-14
- **All acceptance criteria met?** Yes.
  - **AC1:** `just test` with `IRRADIANCE_CORPUS_DIR` set ran 695 passed, 0 failed, 1 ignored, with no SKIP lines. Every plane and develop pin is unedited and green.
  - **AC2:** `tests/plane_groups.rs` (new) has strips, bands and clipped tiles; it passed on the old code too, as the spec expects.
  - **AC3:** `groups_keep_bitreader_errors_and_partial_writes` checks every strip length from 0 to the exact length, for 0–17 samples at 12 and 14 bits.
  - **AC4:** the retargeted injector gives fixture honest `d1d83299c631541fac68da1051b19a23` against mutant `ca91201dc89cb9b90cc168de7e4b3e92`, and L1021223 honest `cb653b5bec24d166eef2fd258ee61ac4` against mutant `8773222ba81cdd4289fb79c595ea1b26`. Both controls are green.
  - **AC5:** no allocation, no `unsafe`, no new dependency. `plane_tiles` and `plane_units_alloc` are green and unedited.
  - **AC6:** 9.7 ms, against a 30 ms target (table below).
  - **AC7:** every gate is green: `just test`, lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, msrv, cost-audit and gate-audit. `just decisions-audit --changed` is advisory; it names DEC-063 and DEC-064, and I re-read both. `just fuzz-plane 60` was clean twice: 3,183,714 runs, then 2,741,344 on the final code.

**Timing.** `just unpack-timing` on a Mac16,8 (Apple M4 Pro), release, one warm-up then the median and min of 7, in ms. The before run is 2e48dd7 plus only the example; the two runs were back to back.

| case | before median / min | after median / min |
|---|---|---|
| `unpack_into`, 1 thread | 239.0 / 238.1 | **9.7 / 9.6** |
| 8 units of 704 rows, 8 threads | 31.2 / 30.8 | 1.4 / 1.3 |

An earlier pair, run about 20 min apart, gave 244.7 → 10.0 and 32.3 → 1.5. The CHANGELOG line uses the back-to-back pair.

**Red-proofs.** Each was applied by hand and then reverted; `src/plane.rs` was compared byte for byte afterwards. All are tier A.

| mutant | red |
|---|---|
| 14-bit shifts 42 and 28 swapped (the injection) | AC2 strips, bands and tiles; `hand_built_fixtures_plane_matches_its_known_md5`; AC3 |
| 14-bit mask `0x3FFF` changed to `0x1FFF` | AC2 ×3; AC3 |
| 12-bit first shift 12 changed to 11 | AC2 ×3; AC3 |
| 12-bit dispatched to the 14-bit group | AC2 ×3; AC3 |
| tail reader one byte after the last group | AC2 ×3; AC3 |
| tail handed a sub-slice instead of a position | AC3 only, on the `Truncated` `at` assertion (AC2 stays green, as expected) |
| `full` reduced by one | **green everywhere**: the benign mutant. The tail re-decodes one group correctly, so only `unpack-timing` would show it. |

### Cost self-report

- **Tokens (total):** 9,728,429 (a floor, measured while the session was live)
- **Estimated USD:** 7.35
- **Duration (minutes):** about 60
- **Source of the number:** transcript `3b9fff38-fdd9-4f81-817f-80c9a2febeb8.jsonl`, deduplicated by message.id with the max output per message, at opus-5-5 rates. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The tail reader is `BitReader::new(strip)` followed by `reader.byte_pos = tail_byte`, one of spec §2's two options. I first added a private `BitReader::at` constructor, but that left `new` without a non-test caller, and `-D dead_code` failed lint-ci. Setting `byte_pos` leaves `BitReader` byte-for-byte unedited and needs no `allow`.
  - The injector needle is `(v.checked_shr(42), v.checked_shr(28))`, a tuple binding. rustfmt splits a four-element array literal across lines, which would have made a one-line needle fragile.
  - The injector's `assert_eq!` failure message still says "BitReader::read's chunk extraction". Spec §3 forbids editing assertions, so the message is left stale; it only prints if the needle moves.
  - I did not add `unpack-timing` to the AGENTS.md §6 command block. `orient-timing` is not listed there either.
- **Follow-up work identified:**
  - [XS] Reword the stale `inject_chunk_extraction_fault` assert message in `tests/plane_oracle.rs` to name the 14-bit group shifts.
  - [XS] AGENTS.md §6 lists neither `orient-timing` nor `unpack-timing`. Decide whether timing examples belong in that block.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little; the probe numbers and red-proof list made it transcription. The handoff's out-of-scope line "Editing `BitReader`" read against spec §2's "Add a private constructor", until dead_code settled it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — rustfmt's array-width heuristic. Any needle-based red-proof must survive `cargo fmt`, so the spec's suggested needle shape should be checked against formatted code.

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt` and lint-ci right after the kernel edit, before the first red-proof pass. Both changes forced a second full red-proof pass.
