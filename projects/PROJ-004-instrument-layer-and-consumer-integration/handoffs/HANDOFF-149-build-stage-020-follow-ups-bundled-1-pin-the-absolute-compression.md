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
  id: HANDOFF-149
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-16
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-071

project:
  id: PROJ-004
  stage: STAGE-021
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
  status: completed  # completed | blocked | rejected
  agent: claude-opus-5-5  # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript  # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 5077732  # REAL combined count — what cost-audit reads
  estimated_usd: 4.05  # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: claude/spec-071-build
  pr: https://github.com/jysf/irradiance/pull/247
  completed_at: 2026-11-16  # YYYY-MM-DD
  notes: "FLOOR from transcript 630367db-95dd-47bc-92d2-dad2f9d75ae2.jsonl (37 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with max output per message: input 76 + cache-creation 112,631 (1-hour) + cache-read 4,946,934 + output 18,091; opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so re-sum at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-149: BUILD: STAGE-020 follow-ups, bundled (SPEC-071)

## Delegation Summary

You are the **builder** for SPEC-071. Cut `claude/spec-071-build` from `origin/main`
**after** the design PR merges. **SPEC-072 is cut after your PR merges**, because both
edit AGENTS.md §6.

**The task**, four small items, with no `src/` change:
1. **An absolute Compression 7 allocation pin** in `tests/plane_units_alloc.rs`. It is
   10 per strip or tile, measured at design.
2. **One assert message** reworded in `tests/plane_oracle.rs`'s
   `inject_chunk_extraction_fault`.
3. **A `just fuzz-ljpeg` recipe** that creates `fuzz/corpus/ljpeg` first.
4. **AGENTS.md §6** lists the recipes it is missing: `orient-timing`, `unpack-timing`,
   `ljpeg-timing` and `fuzz-ljpeg`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-071-stage-020-follow-ups-bundled-1-pin-the-absolute-compression.md`.
  Read all of it: the §1 measured table, §2–§4, AC1–AC5 and the Notes.
- **Decisions:** DEC-063 §2, DEC-064 §2 and DEC-016.
- **Code:**
  - `tests/plane_units_alloc.rs`. Its tally is per thread since PR 243; keep it so.
  - `src/ljpeg.rs` (`parse_headers`, `parse_dht`), to explain the 10.
  - `src/plane.rs` (`decode_unit`), for AC1's red-proof.
  - `tests/plane_oracle.rs`.
  - `app.just`.
  - `fuzz/fuzz_targets/ljpeg.rs`.
  - AGENTS.md §6.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - The full `just test` still takes about 37 min in debug (SPEC-072 fixes that), so run
    it in the background.
  - Do AC1's red-proof on an **archived copy** with its own target dir.

## Expected Deliverables

1. **The four items**, as the spec's §1–§4 describe.
2. **AC1's red-proof:** the added allocation turns the new absolute assertion red, while
   the old-equals-new assertion alone stays green. Show both.
3. **AC3:** delete `fuzz/corpus/ljpeg`, run `just fuzz-ljpeg 60`, and show that it is
   created and the run is clean.
4. **AC4:** list the `app.just` recipe names against the §6 block, with none missing.
5. **Gates, all green with the corpus env set:**
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

   Also confirm `git diff origin/main -- src/` is empty.
6. **PR.** Open it against `main` from `claude/spec-071-build`, referencing SPEC-071 and
   HANDOFF-149. Confirm CI is green on its head.

## Out of Scope

- Any `src/` change.
- Reducing the allocations.
- Pinning held files.
- The test lane (SPEC-072).
- CHANGELOG (this is test and tooling only).

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **If an existing test goes red, the change is wrong.** Fix the change, never the test.
  The only permitted test edits are §1's new assertion and §2's message.
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback

### Execution notes

- **Branch / PR:** `claude/spec-071-build`, https://github.com/jysf/irradiance/pull/247. The build commit is 1dcf5c6; this handback follows it.
- **Completed at:** 2026-11-16
- **All acceptance criteria met?** Yes. The evidence is below and in the PR body.
  - **AC1.** `C7_ALLOCATIONS_PER_CHUNK = 10` is asserted as `decode_allocs == 10 × chunk_count`, where the chunk count comes from the sensor's tile or strip offsets, for both compressed fixtures at every band height. The old-equals-new check stays beside it, and the tally stays per thread.
    - **The 10 are** two `parse_headers` calls per chunk (`plane`'s DEC-047 shape binding, then `ljpeg::open_scan`) × 5 allocations each (the `dc_tables` growth, the `huffval` Vec, the `tds` Vec, `component_tables`, and the `HuffTable` clone).
    - **Red-proof**, on an archived copy with its own target dir:
      - a `black_box(Vec::with_capacity(1))` after the C7 strip arm's `parse_headers` gives `c7 multi-strip band 1 … left: 44 right: 40`, while the old assertion just above it passed (44 = 44);
      - the tile-arm mutation alone gives `c7 tiles band 1 … left: 132 right: 120`;
      - `main`'s test file against the same mutated tree gives 3 passed.
  - **AC2.** Only the message literal changed. It names `unpack_groups14`, the function that holds the needle (`unpack_bitstream` calls it).
  - **AC3.** With `fuzz/corpus/ljpeg` deleted, `just fuzz-ljpeg 60` created it and finished `Done 2659833 runs in 61 second(s)`, exit 0, with no crash.
  - **AC4.** All 35 `app.just` recipe names (29 plain and 6 with `seconds`) match a §6 block comment header, and none is missing.
  - **AC5.** With the corpus env set:
    - `just test` ran 37 binaries: 697 passed, 0 failed, 1 ignored, with 0 SKIP lines.
    - These are green: `fmt-check`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `validate`, `gate-audit`, `deny`, `deny-fuzz`, `doc-check`, `msrv` and `cost-audit`.
    - `git diff origin/main -- src/` is empty.

### Cost self-report

- **Tokens (total):** 5,077,732. This is a floor, measured before the handback commit, so re-sum it at ship.
- **Estimated USD:** 4.05
- **Duration (minutes):** about 45
- **Source of the number:** the transcript (630367db-95dd-47bc-92d2-dad2f9d75ae2.jsonl), deduped by message.id, priced at opus-5-5 rates.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - AC4 asks for every recipe, and the §6 block was also missing `doc-check` and `doc-check-red-proof` (SPEC-056) and `validate-red-proof` (SPEC-016), so they were added. The `licences` header became `deny / deny-fuzz (licences)` so both recipe names appear. The prose-history sentence names all seven.
  - The assert message names `unpack_groups14` rather than the spec's example wording "in unpack_bitstream", because the needle lives in `unpack_groups14`.
  - The branch was renamed from the worktree's `claude/spec-071-build-061422` to `claude/spec-071-build`, as the handoff names it.
- **Follow-up work identified:** none required. One optional item: the §6 `test` entry shows only `cargo test --all-features`, while the recipe also runs the `corpus-status` example first. SPEC-072 rewrites that line anyway.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The spec said four recipes were missing, but AC4's "every recipe" check found three more, plus the `licences` header naming neither `deny` recipe. The spec's allocation breakdown also did not say the header is parsed twice per chunk, which is the reason the count is 10 rather than 5.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — I would run the AC4 name-against-header comparison at design time, so the spec states the true missing set.
