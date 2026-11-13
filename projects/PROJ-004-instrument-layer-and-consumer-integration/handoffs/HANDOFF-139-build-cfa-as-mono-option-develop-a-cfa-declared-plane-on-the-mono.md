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
  id: HANDOFF-139
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-13
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-066

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
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 16370481               # REAL combined count — what cost-audit reads
  estimated_usd: 11.76              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-066-build
  pr: https://github.com/jysf/irradiance/pull/227
  completed_at: 2026-11-13               # YYYY-MM-DD
  notes: "FLOOR from transcript 5ed79b91-42b4-4c23-9230-1fe9f9e1acdf.jsonl at handback time (82 unique message.ids, all claude-opus-5-5, no subagents, max output per message): input 167 + cache-read 16,097,182 + cache-creation 208,139 (all 1-hour) + output 64,993. USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Excludes the handback commit and CI-confirmation turns; re-sum at ship."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-139: BUILD: per-position black levels on the mono path, CFA-as-mono (SPEC-066)

## Delegation Summary

You are the **builder** for SPEC-066. Cut `claude/spec-066-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-061
and this handoff.

The task:
- Add `develop::BlackLevels { FirstValue, PerPosition }` (`#[non_exhaustive]`, default
  `FirstValue`), the `DevelopOptions::black_levels` field, and
  `Error::InvalidBlackLevelPattern` (`Corrupt`).
- **Under `PerPosition`**, every mono develop entry point applies `BlackLevel` per
  photosite:
  - indexed by `BlackLevelRepeatDim` (default `[1, 1]`);
  - phased from the `ActiveArea` origin;
  - taken on raw samples **before** the warp;
  - with the white level per sample (scalar), as today;
  - with no allocation.
- **`FirstValue`** is today's output byte for byte. `DEVELOP_VERSION` stays `"4"`.

No routing change: a CFA plane already develops on the mono path (DEC-061 §1).

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-066-cfa-as-mono-option-develop-a-cfa-declared-plane-on-the-mono.md`.
  Read all of it:
  - the §1 probe: every held file's level tags, and the Canon numbers. Under
    `FullActiveArea` with no orientation, 5,602,332 of 22,430,520 samples differ, by at
    most 6, all at position 3.
  - Design §2–§6;
  - AC1–AC13, the red-proof list and the Notes.
- **Decision:** `./decisions/DEC-061-the-mono-develop-honours-blacklevelrepeatdim-under-an-opt-in-per-position-black-level-option-and-white-stays-per-sample.md`.
- **Also:**
  - DEC-058 (options; the version rule);
  - DEC-060 (the scratch path allocates nothing; the shared core);
  - DEC-040 (the `ActiveArea`-origin phase, as the colour path uses it);
  - DEC-018 (clamp);
  - DEC-012 (a malformed tag costs the field: the K-3 III's `BlackLevelRepeatDim` is
    `None`);
  - DEC-051, DEC-052;
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/develop.rs`:
    - `DevelopOptions`;
    - `resolve_levels`;
    - `normalize`;
    - `crop_orient_normalize_into`;
    - `normalize_active_area_into`;
    - `develop_core` (levels are resolved right after geometry);
    - `run_passes`.
  - `src/color.rs` `make_normalizer` / `level_at`, for reference only. Do not reuse it:
    it hard-codes a 2×2 grid.
  - `src/ifd.rs`: `Sensor` level fields; `Sensor::test_fixture` and `Cfa::test_fixture`
    under the `test-fixtures` feature.
  - `src/lib.rs`.
- **Tests to mirror:**
  - `tests/develop_options.rs` (tier-B gating, MD5 pins, sentinels);
  - `tests/develop_scratch.rs` and `tests/develop_scratch_alloc.rs` (the counting
    allocator);
  - `tests/error_kind.rs`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Run long tier-B binaries in the background.

## Expected Deliverables

1. **Tests first.** Write `tests/black_levels.rs`, the alloc row and the `error_kind`
   row, as the spec's Failing Tests list says, and show them red. A compile failure counts
   for the new API. Record the order in the handback.
   - **AC2–AC5's expected values come from the normalize formula written in the test**,
     never from calling the crate. A byte-identity relation between the two modes cannot
     see a bug in the pattern arm (`shared-path-oracle-tautology`, now N=3).
2. **Implement it** following Design §2–§4:
   - level resolution at today's call position;
   - a `Uniform` fast path with no per-pixel modulo;
   - the pattern index from raw coordinates relative to the `ActiveArea` origin, at both
     normalize sites;
   - no `unsafe`, no allocation, and no panicking arithmetic.
3. **Fuzz (§5).**
   - Option bit 5, the collapse differential, and three synthetic seeds via
     `examples/fuzz-seeds.rs`.
   - Run `develop` for at least 60 s and report the run count.
   - Classify every develop seed by level mode.
4. **Docs (§6).**
   - CHANGELOG `[Unreleased]`.
   - The develop module paragraph.
   - The field and enum docs.
   - No provenance row, and no `DEVELOP_VERSION` bump.
   - Record a release-build Q2M `develop_into` wall time before and after (single run, no
     gate).
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

   A mutant that only tier B catches is a gap to close. CI has no corpus.
7. **PR.** Open it against `main` from `claude/spec-066-build`, referencing SPEC-066,
   DEC-061 and HANDOFF-139. Confirm CI is green on its head.

## Out of Scope

- Any routing change or CFA refusal.
- Per-position white.
- `BlackLevelDeltaH` and `BlackLevelDeltaV`.
- RATIONAL `BlackLevel` (dropped to 0 on every path today).
- The colour path's `BlackLevelRepeatDim` handling.
- SPEC-067.
- `irr` CLI flags, a release, or a `DEVELOP_VERSION` bump.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- **Never commit corpus bytes or derived corpus pixels (DEC-003).** MD5 strings, counts
  and lengths in source are fine.
- **Fix the change, never the test.** If a SPEC-063 MD5 pin, a SPEC-065 scratch test or an
  existing precedence test goes red, the change altered default behaviour.
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

- **Branch / PR:** `claude/spec-066-build`, https://github.com/jysf/irradiance/pull/227
- **Completed at:** 2026-11-13
- **All acceptance criteria met?** yes. AC1–AC13 hold. Notes:
  - **AC2, k = 15345.** The two per-position ranges (15345 and 15346) do not round
    alike at this k: positions 0/1 give 65535 and positions 2/3 give 65531. Only the
    analytic value is asserted there. At k = 0 and 200 the output is uniform and equals
    the LinearRaw develop.
  - **AC2, FirstValue at k = 15345.** FirstValue happens to equal PerPosition here, so
    the "modes differ" assertion is made at k = 200 only.
  - **AC3, crop origin.** It is (1, 2) inside the ActiveArea, not odd on both axes. With
    an ActiveArea origin of (3, 1), a both-odd crop origin lands on the plane origin's
    parity, and the "phase from the DefaultCrop origin" mutant would be invisible. With
    (1, 2) the three candidate origins give three different blocks, as the Notes require.
  - **AC5.** Uniformity needs the four ranges to round alike. Two cases are used: the
    AC3 pattern at k = 0, and `[1000, 1001, 1002, 1003]` with white 65535 at k = 10.
    The test asserts the rounding premise itself.
- **Tests-first order:**
  - `tests/black_levels.rs`, the `per_position_allocates_nothing` row and the
    `error_kind` row were written first. All three binaries failed to compile on the
    missing API (`BlackLevels`, `DevelopOptions::black_levels`,
    `Error::InvalidBlackLevelPattern`): 2 + 4 + 6 errors.
  - Then the implementation. Tier A went green after one test-side fix: the fixture's
    phase function underflowed in the padding left of the ActiveArea, and
    `rem_euclid` fixed it.
- **Gates (corpus env set):**
  - `just test`: 668 passed, 0 failed, 1 ignored.
  - Also green: lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check,
    deny, msrv, cost-audit, gate-audit.
  - `decisions-audit --changed` is advisory only.
- **First full run red, fixed in the change:** `develop_oracle`'s
  `the_oracle_is_red_on_an_orientation_fault` scans the text of
  `crop_orient_normalize_into` for its loop. My first cut moved that loop into a helper.
  Fix the change, not the test: the loops stay in `crop_orient_normalize_into` and
  `normalize_active_area_into`, and the level dispatch moved to the new
  `crop_orient_normalize_for` and `normalize_active_area_for` wrappers. Commit 19af30c.
- **Tier B:**
  - Canon 5D3, FullActiveArea, unoriented: 5796×3870; positions 0–2 are equal; at
    position 3 PerPosition is ≤ FirstValue on every sample, differs on exactly
    5,602,332 samples, and by at most 6. This matches the design probe.
  - The three Q2M frames, M Monochrom, Typ 246, K-3 III and Nikon under PerPosition
    MD5-equal their SPEC-063 pins.
- **Red-proofs:** 12 mutations, each applied by hand to `src/develop.rs`, run on tier A
  only (no corpus env), and reverted. Each turned the named test red:
  1. Pattern ignored (FirstValue always): AC2, AC3, AC4, AC5, AC6, AC7, AC9.
  2. Entries 0 and 1 swapped: AC3, AC4, AC5.
  3. Phase from the plane origin: AC2, AC3, AC4, AC5.
  4. Phase from the DefaultCrop origin: AC2, AC3, AC4, AC5.
  5. Index transposed: AC2, AC3, AC4, AC5.
  6. Fixed 2×2 slot: AC4.
  7. Warp site, black by active-grid (output) position: AC5.
  8. Warp site, pattern ignored: AC5.
  9. No count check, index clamped: AC6, AC7, AC9.
  10. Pattern validated under FirstValue too: AC6.
  11. Pattern check after the dst length check: AC7.
  12. Per-position white (the last WhiteLevel value): AC3's two-WhiteLevel fixture.

  All 12 were re-run after the 19af30c restructure and stayed red. Mutant 8's re-run
  needed a closure type annotation to compile.
- **Fuzz:**
  - `develop` ran for 60 s: 4,566,463 runs in 61 s, clean.
  - Seed classification, 38 develop seeds:
    - 15 have no BlackLevel (one, `tiled-uncompressed-12-mm`, has bit 5 set by its
      trailing byte; it develops Ok);
    - 20 have a single uniform value with bit 5 clear;
    - 3 are new non-uniform patterns with bit 5 set:
      `black-levels-2x2-odd-active-area` (Ok), `black-levels-1x2` (Ok) and
      `black-levels-count-mismatch` (InvalidBlackLevelPattern 1×1 / 4).
- **Q2M L1021223 `develop_into` wall time** (release build, single runs, alternating
  builds on an idle machine): 0.627 s and 0.631 s before (06d776a); 0.634 s and
  0.644 s after. No meaningful change, as expected, since the Q2M takes Uniform.
- **Docs:** CHANGELOG `[Unreleased]` (two Added entries), the develop module paragraph,
  and the field and enum docs. No provenance row (DNG `BlackLevel` semantics, no new
  algorithm). No `DEVELOP_VERSION` bump.

### Cost self-report

- **Tokens (total):** 16,370,481. This is a floor at handback; the handback commit and
  CI turns are excluded.
- **Estimated USD:** 11.76
- **Duration (minutes):** about 60 by the transcript's timestamps.
- **Source of the number:** transcript `5ed79b91-42b4-4c23-9230-1fe9f9e1acdf.jsonl`,
  deduped by message.id, max output per message, opus-5-5 rates, no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The AC3 crop origin is (1, 2), not odd on both axes (see above).
  - AC5 uses two cases so that "uniform normalize(k)" is well defined.
  - The per-position white red-proof is modelled as reading the last WhiteLevel value
    (one global white), not a true per-pixel white; the two-WhiteLevel fixture catches
    any read past the first value.
- **Follow-up work identified:** none new beyond DEC-061 §5's three (RATIONAL
  BlackLevel, the colour path's BlackLevelRepeatDim, BlackLevelDeltaH/V).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Three things:
   - AC2 and AC5 say "uniform normalize(k)", but distinct per-position blacks give
     distinct ranges, so uniformity holds only at k values that happen to round alike.
   - AC3 asks for an odd DefaultCrop origin, which with an odd ActiveArea origin
     reproduces the plane parity.

   All three needed the fixtures re-derived.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Yes. `tests/develop_oracle.rs` and `tests/develop.rs` fault-inject by scanning
   `src/develop.rs` source text for named function bodies, so refactoring those
   functions breaks a test that looks unrelated. A handoff touching `develop.rs` should
   name them.
3. **If you did this task again, what would you do differently?**
   — Grep `tests/` for `include_str!` and `read_to_string` of `src/` before
   restructuring any function, and run the full suite before the first commit rather
   than only the targeted binaries.
