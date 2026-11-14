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
  status: pending                  # pending | accepted | completed | rejected

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
