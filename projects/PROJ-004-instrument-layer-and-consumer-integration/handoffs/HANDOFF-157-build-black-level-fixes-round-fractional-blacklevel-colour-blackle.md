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
  id: HANDOFF-157
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-18
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-075

project:
  id: PROJ-004
  stage: STAGE-023
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

# HANDOFF-157: BUILD: Black-level fixes and MaskedAreas (SPEC-075)

## Delegation Summary

You are the **builder** for SPEC-075. Cut `claude/spec-075-build` from `origin/main`
**after** the STAGE-023 design PR merges.

**The task**, in three parts, all in DEC-068:
- **Round a non-integral RATIONAL `BlackLevel`** to nearest, and keep the exact values in
  a new `Sensor::black_levels_exact`.
- **Make the colour normalizer index `BlackLevelRepeatDim`** with DEC-061 §3's rule, and
  refuse a count mismatch with `InvalidBlackLevelPattern` on all four colour entry
  points.
- **Pass `MaskedAreas` through** as `Sensor::masked_areas`.

`DEVELOP_VERSION` goes to `"5"`. Every held file's develop output must stay
byte-identical.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-075-black-level-fixes-round-fractional-blacklevel-colour-blackle.md`.
  Read all of it: the probe table, Design §2–§5, AC1–AC9 and the Notes.
- **Decisions:**
  - `DEC-068` (all of it);
  - `DEC-061` §3 (the pattern rule and the error);
  - `DEC-012` (interpretation tags cost themselves only);
  - `DEC-040` (the `ActiveArea` origin);
  - `DEC-058` §5 (the version rule);
  - `DEC-009` (red-proof, control first).
- **Code:**
  - `src/ifd.rs`: the sensor interpretation around `TAG_BLACK_LEVEL` (about lines
    2030–2230), `Container::rationals`, `array::<4>` for `ActiveArea`, `cost_the_field`,
    and the `Sensor` struct (about line 690).
  - `src/color.rs`: `make_normalizer` (about line 1240), `level_at`, `normalize_sample`,
    and the four entry points.
  - `src/develop.rs`: `resolve_levels_with` and `Blacks` (about lines 560–660).
  - `src/lib.rs`: `DEVELOP_VERSION`.
- **Tests and fixtures:**
  - `tests/support/dng_writer.rs` and its `extra_tags` (`RawTag`), which writes every
    fixture with no writer change;
  - `tests/black_levels.rs`, SPEC-066's oracle style (expected values are never computed
    by calling the crate).
- **Toolchain:**
  - AGENTS.md §5–6. `just test` runs the `corpus` profile plus the debug `*_alloc`
    binaries in about 3 minutes.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Do red-proofs on a local commit you never push, or an archived copy with its own
    target dir.
- **Facts from the design probe**, so you don't re-derive them:
  - The Pixel 7 Pro's `BlackLevel` is RATIONAL `262172/256 262224/256 262223/256
    262178/256`. All four round to 1024.
  - The Canon's is RATIONAL and integral.
  - No held file has `MaskedAreas`, a `[1, 2]`/`[2, 1]` pattern, or more than one
    `WhiteLevel` value.

## Expected Deliverables

1. **Commits:**
   1. the `BlackLevel` read and `black_levels_exact`;
   2. the colour pattern;
   3. `MaskedAreas`;
   4. `DEVELOP_VERSION` and the CHANGELOG;
   5. the feedback item-8 row.

   Combine commits where that keeps each one green.
2. **Evidence for each AC** in the handback:
   - **AC3, AC4 and AC5:** each red-proof, giving the mutant, the failing test and the
     revert. The `[1, 2]` and `[2, 1]` equivalences must be shown red against today's
     `level_at`.
   - **AC7:** the `exiftool -MaskedAreas` output for one fixture.
   - **AC8:** the corpus `just test` summary.
3. **Gates (AC9),** all green with the corpus env set:
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
   - `name-length-check`
   - `semver-check`
4. **PR** against `main` from `claude/spec-075-build`, referencing SPEC-075 and
   HANDOFF-157, with CI green on its head. The macOS determinism leg is non-blocking.

## Out of Scope

- A fractional black in develop.
- `BlackLevelDeltaH`/`V`.
- A colour `BlackLevels` option, or a colour black ≥ white refusal.
- Any change to mono `FirstValue`/`PerPosition` output.
- Using `MaskedAreas` in develop.
- GainMap, or making the Pixel files develop.
- SPEC-076's test hardenings.
- A release.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **If an existing test or oracle goes red, the change is wrong.** Fix the change, never
  the test. The only permitted edits to existing tests are the three `DEVELOP_VERSION`
  pins (`"4"` to `"5"`) and new tier-B assertions.
- **Fill the `handback:` front matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section below before reporting done.

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
