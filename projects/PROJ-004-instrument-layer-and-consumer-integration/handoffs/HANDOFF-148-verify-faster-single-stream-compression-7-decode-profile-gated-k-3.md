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
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
