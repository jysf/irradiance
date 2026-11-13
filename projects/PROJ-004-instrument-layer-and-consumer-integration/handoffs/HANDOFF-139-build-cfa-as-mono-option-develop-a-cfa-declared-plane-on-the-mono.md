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
  status: pending                  # pending | accepted | completed | rejected

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
