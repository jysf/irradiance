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
