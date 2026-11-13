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
  id: HANDOFF-137
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-12
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-065

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
  tokens_total: 24495029           # REAL combined count — what cost-audit reads
  estimated_usd: 16.99             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 65
  branch: claude/spec-065-build
  pr: 224
  completed_at: 2026-11-12         # YYYY-MM-DD
  notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript 2a702dbf-8f01-4323-84fe-26eccdf7e734.jsonl (106 unique message.ids, all claude-opus-5-5, no subagents): input 216 + cache-creation 246,913 (1-hour) + cache-read 24,150,133 + output 97,767 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 20,612,106 / 14.65 before the handback commit, PR and CI turns."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-13
---

# HANDOFF-137: BUILD: caller-owned develop scratch, allocation-free (SPEC-065)

## Delegation Summary

You are the **builder** for SPEC-065. Cut `claude/spec-065-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-060
and this handoff.

The task:
- Add `develop::scratch_len(sensor, &options)` and
  `develop::develop_with_scratch_into(sensor, src, dst, scratch, &options)`, plus
  `Error::DevelopScratchWrongLength`.
- Refactor `develop_with_into` into one private core that both functions call. The scratch
  form allocates nothing. `develop_with_into` allocates one `Vec` of exactly `scratch_len`.
  The private layout reuses the plane copy for the warp output, so the scratch is P + A,
  not P + 2A.
- Make `parse_fix_bad_pixels_constant` / `parse_warp_rectilinear` allocation-free walkers
  with identical results.
- Make `apply_fix_bad_pixels_constant` allocation-free.

Output and error order must be unchanged everywhere. The SPEC-063 MD5 pins must stay green
unedited.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-065-caller-owned-develop-scratch-scratch-len-plus-a-scratch-vari.md`.
  Read all of it:
  - the §1 probe (6 allocations / 283,943,504 bytes on a Q2M frame; expected
    `scratch_len` values);
  - Design §2–§7, especially §4's validation order and passes by case, and §5's walker
    equivalence rules;
  - AC1–AC13, the red-proof list, and the Notes.
- **Decision:** `./decisions/DEC-060-develop-scratch-is-one-exact-length-caller-slice-with-a-private-layout-and-the-scratch-path-allocates-nothing.md`.
  It covers the shape, the exact-length rule, the contract, the layout table, and the
  shared core.
- **Also:**
  - DEC-058 (options; validate then override);
  - DEC-059 (`warp_alpha`, and the `FullActiveArea` bounds check whose position is
    unchanged);
  - DEC-018/024 (levels, warp);
  - DEC-051 (wildcard-free `Error::kind`);
  - DEC-052;
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/develop.rs`: `develop_with_into` and its FBP copy and warp `vec!`s, plus
    `apply_fix_bad_pixels_constant`;
  - `src/opcode.rs`: `read_raw_opcode`, `parse_opcode_list`, the two lookups, and
    `check_list` as the walker pattern;
  - `src/lib.rs`.
- **Tests to mirror:**
  - `tests/previews_alloc.rs` for the counting allocator: its own binary, a thread-local
    switch, the negative control;
  - `tests/develop_options.rs` and `tests/warp_alpha.rs` for the tier-B gating, pins and
    sentinels;
  - `tests/opcode.rs`;
  - `tests/error_kind.rs`.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place and never copy them into the repo. Long tier-B binaries run in
  the background.

## Expected Deliverables

1. **Tests first.** Write `tests/develop_scratch.rs`, `tests/develop_scratch_alloc.rs`,
   the `tests/opcode.rs` equivalence test, and the `tests/error_kind.rs` row, as the
   spec's Failing Tests list says. Show them red; a compile failure counts for the new
   API. The AC8 equivalence test should **pass** on main, because it pins today's
   behaviour before the rewrite. Record that order in the handback.
2. **Implement it**, following Design §2–§5:
   - the walkers first, kept green by AC8;
   - then the allocation-free neighbours;
   - then the core, with the validation order exactly as §4 lists it, and the four
     cases with the layout from DEC-060 §3 (`split_at_mut`, no `unsafe`);
   - then the two public functions and the delegation.
3. **Fuzz (§6).** Add both differentials, and run `develop` and `opcode` for at least 60 s
   each. Report the run counts, and confirm that the existing seeds cover all four cases.
4. **Docs (§7).** CHANGELOG `[Unreleased]` (Added and Changed, with the Q2M figures),
   plus the `src/develop.rs` module and `develop_into` doc updates. No provenance row.
   Do **not** bump `DEVELOP_VERSION`.
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
6. **Red-proofs.** For each mutation in the spec's red-proof list, apply it by hand, show
   the named test red, and revert. List them in the handback.
7. **PR.** Open a PR against `main` from `claude/spec-065-build` that references SPEC-065,
   DEC-060 and HANDOFF-137. Confirm CI is green on its head.

## Out of Scope

- Warping straight into `dst`.
- A colour-path or unpack scratch variant.
- Making `parse_opcode_list` allocation-free.
- CFA handling (SPEC-066) and the blocked transpose (SPEC-067).
- `irr` CLI flags, a release, or a `DEVELOP_VERSION` bump.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). MD5 strings and lengths in
  source are fine.
- If a SPEC-063 MD5 pin or an existing precedence test goes red, the change altered
  behaviour. Fix the change, never the test.
- Fill the `handback:` front-matter, including a real `tokens_total` re-summed from your
  own transcript (`basis: transcript`, dedupe by `message.id`, max output per message,
  opus-5-5 rates). Fill the `## Handback` section too.

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

- **Branch / PR:** `claude/spec-065-build`, https://github.com/jysf/irradiance/pull/224
  - 3446123: failing tests first;
  - 06d486c: the implementation;
  - 2f28be1: core-independent oracles found by the red-proofs;
  - 71e50df: the scratch fuzz seeds;
  - then this handback.
- **Completed at:** 2026-11-12
- **All acceptance criteria met?** Yes, AC1 to AC13. Three were strengthened beyond the spec; see Deviations.
- **Tests-first order.** The new tests were committed before any `src/` change. `develop_scratch` and `develop_scratch_alloc` failed to compile on main: the API is missing (`E0432`) and so is `Error::DevelopScratchWrongLength` (`E0599`). `error_kind` failed to compile for the same reason. `walkers_equal_parse_opcode_list_then_find_map` **passed on main** before the rewrite, pinning today's lookups, and stayed green after it.
- **Implementation order** was as Design §2–§5 lists it: the walkers (`first_recognised`), the neighbours array, then `develop_core` with validation steps 0–10, the `Scratch::{Caller, Allocate}` source and `run_passes` with the DEC-060 §3 layout via `split_at_mut_checked`. The public functions come last.
- **Gates.** All runs set `IRRADIANCE_CORPUS_DIR` to the held images in place; corpus 12/12 present.
  - `just test`: 655 passed, 0 failed, 1 ignored (the existing SPEC-017 SB-1 ignore).
  - Green: `lint-ci` (clippy 0.1.98 pinned), `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`.
  - `decisions-audit --changed`: advisory only. The change is consistent with the listed DECs. DEC-051's prose table was not updated, matching what DEC-058 and DEC-059 did; `tests/error_kind.rs` carries the count (64; 14 CallerError).
- **Q2M figures confirmed (tier B, L1021223):**
  - `develop_with_into`: 1 allocation of 189,415,424 bytes;
  - `develop_with_scratch_into`: 0 allocations;
  - `scratch_len`: 94,707,712 at default options and 47,443,968 at α = 0 on all three Q2M frames, 0 on the five warp-free files;
  - all 8 SPEC-063 pins hold through the scratch path.
- **Fuzz (AC12):**
  - `develop`: 4,860,234 runs in 71 s clean; after the new seeds, 3,050,060 runs in 71 s clean.
  - `opcode`: 4,229,874 runs in 71 s clean.
  - Seed coverage was checked by classifying every develop seed with `scratch_len`. Before: only 0 (neither) and 60 (2A). After the five new seeds: 0, 48 (P), 60 (2A) and 78 (P + A).
- **Red-proofs.** Each mutation was applied by hand, run against the named test in tier A, and reverted; the tree was clean after each. All 15 went red:
  1. A `vec!` left in the caller-scratch path → `scratch_develop_allocates_nothing`.
  2. The warp lookup calls `parse_opcode_list` again → `opcode_lookups_allocate_nothing`.
  3. The walker returns at the first match → `walkers_equal_parse_opcode_list_then_find_map`.
  4. The walker skips the unknown-mandatory refusal → same test.
  5. No reuse (P + 2A):
     - (a) → `scratch_len_per_case_and_alpha`;
     - (b) → `develop_with_into_allocates_once`.
  6. The warp runs before normalize in the "both" case → `each_case_equals_its_composition_from_public_parts`.
  7. The warp reads the wrong half:
     - (a) warp-only, unwritten buffer → `scratch_output_equals_develop_with_into`;
     - (c) the same mutation → `output_does_not_depend_on_scratch_contents`;
     - (b) "both" case, reading the plane copy → the composition test.
  8. A scratch write before the length check → `wrong_length_scratch_is_refused_before_any_write`.
  9. The length check before the `OpcodeList1` parse → `scratch_variant_reports_the_same_errors_in_the_same_order`.
  10. "At least n" accepted → `wrong_length_scratch_is_refused_before_any_write`.
  11. The neighbours array:
      - (a) not sorted → the existing `tests/develop.rs` suite;
      - (b) `lower_median` over the whole array → `fix_bad_pixels_median_with_fewer_than_eight_neighbours`.
- **Provenance ledger:** no row, because no new algorithm was added (spec §7). `DEVELOP_VERSION` is still `"4"`. No new dependency.

### Cost self-report

- **Tokens (total):** 20,612,106. This is a **floor**, taken live before the handback commit, the PR and the CI turns.
- **Estimated USD:** 14.65 (opus-5-5, per component)
- **Duration (minutes):** about 65
- **Source of the number:** this session's transcript `2a702dbf-8f01-4323-84fe-26eccdf7e734.jsonl` (the scratchpad UUID). Method: dedupe by `message.id`, take the max `output_tokens` per message. There are no subagents. Re-sum it at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **§6 "no new seeds are required" was wrong.** No develop seed carried a `FixBadPixelsConstant`, so the differential never reached the P or P + A layouts. I added five seeds to `examples/fuzz-seeds.rs` (`scratch-fix-*`); `just fuzz-seeds` regenerated only those five files.
  - **The byte-identity oracle (AC2/AC6) is blind to a bug in the shared core.** Both entries run the same passes, so a pass-order or wrong-buffer bug moves both sides. Red-proofs 6 and 7b were green in tier A against AC2, and only the tier-B MD5 pins caught them (CI has no corpus). I added `each_case_equals_its_composition_from_public_parts`: the "both" case must equal fixing with the public `apply_fix_bad_pixels_constant` and then running the warp-only develop, and the fix-only case the opcode-free develop. AC6's tier-A loop now covers all four cases, not only "both".
  - **The spec's last red-proof (`lower_median` over the whole array) was invisible to every existing test**, because each fixture's bad pixel had all eight neighbours. I added `fix_bad_pixels_median_with_fewer_than_eight_neighbours` (corner and edge pixels, values by hand).
  - **AC7's table** rebuilds one row per distinct refusal shape from `tests/develop.rs`, `develop_options.rs` and `warp_alpha.rs` on the "both" fixture (27 rows). Test binaries cannot share fixtures.
  - **AC10:** `scratch_len` overflow is reachable on 64-bit (a hostile `u32::MAX × u32::MAX` header makes P + A or 2A exceed `u64`), so it is pinned by `scratch_len_overflow_is_typed` rather than documented as unreachable.
  - **`develop_with_into`'s allocation sits after both opcode lists are parsed**, at step 9. Before, the plane copy came between the `OpcodeList1` parse and the `StandardCrop` `OpcodeList3` parse. It cannot fail, so no precedence changed, and every existing precedence test is unedited and green.
- **Follow-up work identified:**
  - Candidate lesson signal: **"identity against a shared-core twin is not an oracle."** When two public entries are refactored onto one core, a relation between them cannot see a bug inside the core. Pair it with an independent composition or an analytic oracle in tier A.
  - DEC-051's prose table has not been updated since DEC-058. It may be worth a sweep, or a note that `tests/error_kind.rs` is now the source of truth.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two claims in the spec were wrong, and the red-proofs found both. The seeds did not cover all four cases, and AC2's byte-identity relation could not catch the pass-order red-proofs the spec assigned to it. Neither slowed the build much, but both would have shipped silently without the hand-run mutations.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — A rule that a red-proof list should name its oracle's independence. Several mutations were assigned to an oracle that structurally shares the mutated code.

3. **If you did this task again, what would you do differently?**
   — I would write the composition oracle with the AC2 tests from the start, and classify the fuzz seeds before trusting a "seeds already cover it" claim.

