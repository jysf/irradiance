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
  id: HANDOFF-141
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-13
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-067

project:
  id: PROJ-004
  stage: STAGE-019
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
  tokens_total: 13493978           # REAL combined count — what cost-audit reads
  estimated_usd: 9.81              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 64
  branch: claude/spec-067-build
  pr: 230
  completed_at: 2026-11-13         # YYYY-MM-DD
  notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript 6e4cf74a-657d-4ea9-b784-1ff5cb117112.jsonl (74 unique message.ids, all claude-opus-5-5, no subagents): input 155 + cache-creation 183,351 (1-hour) + cache-read 13,256,450 + output 54,022 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 11,900,467 / 8.77 before the handback commit and CI turns."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-13
---

# HANDOFF-141: BUILD: tiled Orientation 5–8 gather on both mono develop paths (SPEC-067)

## Delegation Summary

You are the **builder** for SPEC-067. Cut `claude/spec-067-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-062
and this handoff.

The task:
- In **both** `crop_orient_normalize_into` (warp-free) and
  `crop_and_orient_from_active_into` (post-warp), visit the output in 64×64 tiles for
  Orientations 5–8. Keep today's row walk for 1–4.
- The per-pixel body is moved, not rewritten. The output must be **byte-identical**: 24
  pre-change MD5 pins are in the spec.
- Add `examples/orient-timing.rs` plus a `just orient-timing` recipe, and record
  before/after timings.

There is no public API change, and `DEVELOP_VERSION` stays `"4"`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-067-blocked-orientation-5-8-transpose-in-crop-orient-normalize-i.md`.
  Read all of it:
  - the §1 timing table and the 24 MD5 pins (main at eaa2ae3);
  - the source-scan test;
  - Design §2–§4;
  - AC1–AC7, the red-proof list and the Notes.
- **Decision:** `./decisions/DEC-062-the-mono-orientation-gather-walks-the-output-in-64x64-tiles-for-orientations-5-8-and-stays-byte-identical.md`.
- **Also:**
  - DEC-058 (the version rule);
  - DEC-060 (the scratch path allocates nothing);
  - DEC-061 (the per-position black inside the per-pixel body);
  - DEC-043 (the colour path is untouched);
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/develop.rs`:
    - `crop_source_coords`;
    - `crop_orient_normalize_for` / `crop_orient_normalize_into`;
    - `crop_and_orient_from_active_into`;
    - `run_passes`.
- **Watch out:** `tests/develop_oracle.rs` (`inject_orientation_identity_fault`) and
  `tests/develop.rs` fault-inject by **scanning `src/develop.rs` source text**.
  - The orientation injector needs **exactly one** `crop_source_coords(` call inside
    `crop_orient_normalize_into`, written with its exact indentation.
  - The only permitted test edit is that injector's `needle` literal.
  - The SPEC-066 build lost time here. Grep `tests/` for `read_to_string` of `src/`
    before you restructure anything.
- **Tests to mirror:**
  - `tests/develop_options.rs` (tier-B gating, MD5 pins);
  - `tests/black_levels.rs` (tier-A fixtures with odd origins);
  - `tests/develop_scratch_alloc.rs`;
  - `tests/support/md5.rs` and `tests/support/corpus.rs`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Run long tier-B binaries in the background. 24 Q2M develops in debug are slow;
    consider `--release` for that one binary when iterating, but the gate is `just test`.

## Expected Deliverables

1. **Tests first.**
   - Write `tests/orientation_tiles.rs` (AC1, AC2) and the AC3 unit test.
   - AC1 and AC2 **pass on main before the change**, because they pin today. Show that.
   - The AC3 unit test is the one that is red first.
   - Record the order in the handback.
2. **Implement it**, following Design §2:
   - one traversal rule, in both functions;
   - one per-pixel body each, with exactly one `crop_source_coords(` call in
     `crop_orient_normalize_into`;
   - no allocation, no `unsafe`, and no panicking arithmetic.
   - Then update only the injector `needle` and show the fault test still goes red on its
     injected copy (AC4).
3. **Timing (§3, AC6).**
   - `examples/orient-timing.rs` and `just orient-timing`.
   - Record a before table (the example on main at the design merge plus only the example
     file) and an after table, naming the machine.
   - Report against the target (O5–O8 within 1.15× of O1 at each α) and the floor (O6
     warp-free at least 25% faster).
4. **Docs (§4).** The CHANGELOG `[Unreleased]` Changed line with the medians, and the doc
   sentence. No provenance row, and no `DEVELOP_VERSION` bump.
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

   Also run `just decisions-audit --changed`.
6. **Red-proofs.** For each mutation in the spec's list:
   - apply it by hand;
   - show the named **tier-A** test go red;
   - revert it;
   - list it in the handback.

   A mutant that only tier B catches is a gap; close it in tier A.
7. **PR.** Open it against `main` from `claude/spec-067-build`, referencing SPEC-067,
   DEC-062 and HANDOFF-141. Confirm CI is green on its head.

## Out of Scope

- The colour path's scatter.
- Threads.
- SIMD.
- A per-machine tile size.
- A criterion dependency.
- A CI timing gate.
- Any output change.
- Unit-level unpack.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- **Never commit corpus bytes or derived corpus pixels (DEC-003).** MD5 strings and
  timings in source are fine.
- **If a pin goes red, the change altered output.** That applies to any §1 pin, any
  SPEC-063 pin, or any existing test. Fix the change, never the test (except AC4's
  needle).
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback

### Execution notes

- **Branch / PR:** `claude/spec-067-build`, https://github.com/jysf/irradiance/pull/230. The code commit is 714400f; the handback commit follows it.
- **Completed at:** 2026-11-13
- **All acceptance criteria met?** Yes, except AC6's 1.15× target on the warp-free path, which is a target, not a gate (see below). The floor is met.
- **Order (tests first):**
  1. `tests/orientation_tiles.rs` was written and run on unchanged main (56976a1). All of it passed, including the 24 tier-B pins (611 s in debug), so the pins are pre-change.
  2. The helper and the AC3 unit test were added. AC3 was red with the 5–8 walk untiled and green when tiled.
- **AC4: no test edit was needed.** rustfmt lays out the closure so that the per-pixel body keeps exactly its old indentation (12 / 16 / 12 spaces). The injector's `needle` still matches once inside `crop_orient_normalize_into`, and `the_oracle_is_red_on_an_orientation_fault` passes. `tests/develop_oracle.rs` has no diff at all.
- **AC5:** the `scratch_develop_allocates_nothing` loop gains Orientation 8; 6 was already there.
- **The recipe** lives in `app.just` (where every app recipe lives; `justfile` imports it), not in `justfile` itself.
- **Gates, with corpus 12/12:**
  - `just test`: 675 passed, 0 failed, 1 ignored;
  - green: `fmt-check`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `validate`, `doc-check`, `deny`, `msrv`, `gate-audit`;
  - `fuzz-develop`: 4,721,795 runs in 61 s, clean.
- **Red-proofs,** each applied by hand, red on tier A, then reverted:
  1. inclusive tile clip: AC2 (63×65 O5) and AC3;
  2. tile end not clipped: AC2 and AC3;
  3. `step_by(tile + 1)`: AC2 (column 64 missing) and AC3;
  4. tile axes swapped (extents): AC2 and AC3;
  5. transposed write `visit(out_y, out_x)`: AC2 and AC3;
  6. 5–8 untiled: AC3 only, because byte identity is order-blind;
  7. 1–4 tiled: AC3;
  8. `crop_and_orient_from_active_into` back on its own row loop: AC3's `both_gathers_walk_through_the_traversal_helper`.

  Mutant 8 is why there are two AC3 tests: the order test covers the shared helper, and a source scan proves both functions use it.

**Timing.** Apple M4 Pro (aarch64), `just orient-timing`, release, 1 warm-up and median of 7, in ms. "Before" is 56976a1 plus only the example file.

| α | | O1 | O2 | O3 | O4 | O5 | O6 | O7 | O8 | O6 unoriented |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | before | 190 | 191 | 186 | 186 | 332 | 324 | 371 | 371 | 190 |
| 0 | after | 78 | 93 | 100 | 101 | 95 | 110 | 146 | 146 | 79 |
| 0 | after (run 2) | 84 | 97 | 101 | 101 | 106 | 129 | 157 | 153 | 80 |
| 1 | before | 574 | 572 | 573 | 570 | 705 | 698 | 721 | 724 | 572 |
| 1 | after | 567 | 573 | 565 | 565 | 581 | 576 | 609 | 621 | 595 |
| 1 | after (run 2) | 578 | 585 | 579 | 588 | 609 | 606 | 625 | 633 | 588 |

- **Floor (warp-free O6 at least 25% faster): met.**
  - 324 → 110 ms, 66% faster; 70% against §1's 374.
  - Run 2 gives 129 ms, still 60% faster.
- **Target (O5–O8 within 1.15× of O1 at each α):**
  - **Post-warp: met**, at 1.02–1.10×. This is the requester's path; every Q2M frame carries a warp.
  - **Warp-free: missed**, at 1.21–1.87×.
    - The cause is that warp-free O1 itself dropped from 190 to 78–84 ms. O2–O4 dropped too, to about 95–100. The closure form evidently optimises the row walk better than the old nested loops did; output is byte-identical by the pins.
    - Against the old O1 of 190, every O5–O8 is now faster than O1 used to be.
    - O7 and O8 remain the slowest (about 146–157 ms). They read source rows in descending order, as O3/O4 do along a row.
    - The verify decides whether this is acceptable. Byte identity holds.

### Cost self-report

- **Tokens (total):** 11,900,467. This is a floor at handback; the handback commit and CI turns are excluded.
- **Estimated USD:** 8.77
- **Duration (minutes):** about 64 by the transcript's timestamps.
- **Source of the number:** transcript `6e4cf74a-657d-4ea9-b784-1ff5cb117112.jsonl`, deduped by message.id with the max output per message, at opus-5-5 rates. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - AC4's permitted needle edit was not needed; the test file is unchanged.
  - The recipe is in `app.just`, not `justfile`.
  - AC3 adds a second unit test, `both_gathers_walk_through_the_traversal_helper`, a source scan, so the "one function left untiled" mutant is caught.
  - AC6's warp-free target is missed for the reason above.
- **Follow-up work identified:**
  - (XS, optional) For O7/O8 (and O3/O4), walk the rows inside a tile bottom-up, or the tiles right-to-left, so source reads ascend. The output stays byte-identical, but AC3's pinned order and DEC-062 §1's order rule would change. Measure before deciding.
  - (XS) `tiled_output_matches_the_pre_change_pins` takes about 10 min in debug; 48 Q2M develops dominate `just test` with the corpus. Consider running it only at α 0 through the scratch entry, or under a release test profile.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The one thing was that AC3's red-proof list includes "one function left untiled", which a helper-only order test cannot see. A source scan closed it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The target is a ratio to O1 measured in the same run, so it assumes O1 does not move. Here O1 got 2.4× faster warp-free. A target stated in absolute ms (or against the pre-change O1) would have been unambiguous.

3. **If you did this task again, what would you do differently?**
   — Run the mutant loop with `--no-fail-fast` from the start: the first pass stopped at the failing lib test and never reached AC2.
