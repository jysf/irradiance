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
  id: HANDOFF-167
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-19
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-078

project:
  id: PROJ-004
  stage: STAGE-025
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

# HANDOFF-167: BUILD: black normalisation by white minus the maximum black (SPEC-078)

## Delegation Summary

You are the **builder** for SPEC-078. **Start only after the SPEC-078 design PR has
merged.** Cut `claude/spec-078-build` from that `origin/main`.

**The task:** implement DEC-070. The colour path (all four entries, `u16` and `f32`,
threaded) and the mono path under `BlackLevels::PerPosition` scale by `W − B`, where `B` is
the maximum black the path applies. The mono default (`FirstValue`) and every
uniform-black file stay byte-identical.

Also:
- set `DEVELOP_VERSION` to 7;
- add the Canon to the SDK recipe;
- tighten SPEC-060's SDK cross-check to the pre-registered bounds in AC4;
- re-pin only what DEC-070 changes, listing each pin.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-078-black-normalisation-decide-white-minus-max-black-the-dng-sdk.md`
  (Design 1–6, AC1–AC7, and the Failing Tests clause map).
- **Decisions:**
  - DEC-070 (the rule, the probe tables and the expected numbers);
  - DEC-018 (rounding and clamping);
  - DEC-061 (`PerPosition`);
  - DEC-068 (colour pattern, fractional rounding);
  - DEC-069 (where `GainMap` sits after normalisation);
  - DEC-060 (zero-allocation scratch develop).
- **Code:**
  - `src/develop.rs`: `normalize`, `resolve_levels`, the `Blacks` resolution and the
    gather and active-area loops;
  - `src/color.rs`: `normalize_sample`, `slot_normalizer`, `pattern_normalizer`, and
    SPEC-059's `with_normalizer!` sites;
  - `tests/gain_map_corpus.rs` and `scripts/sdk-stage2.sh` (SPEC-060);
  - `tests/black_levels.rs`, `tests/develop_options.rs`, `tests/develop_scratch.rs`.
- **The SDK:**
  - `dng_validate` is at `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate the stage-2 inputs with the recipe into your scratchpad, and set
    `IRRADIANCE_SDK_STAGE2_DIR`.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`.

## Expected Deliverables

1. **The implementation and tests** for AC1–AC6, in a PR against `main` from
   `claude/spec-078-build`, with CI green.
2. **The handback:**
   - the re-pin list (old → new hash, file, path, and why);
   - the AC4 numbers for every file and comparison (max |Δ|, the fraction exact, the
     histogram);
   - the AC6 mutant table.
3. **Gates**, all green: AC7.

## Out of Scope

- `BlackLevelDeltaH` / `BlackLevelDeltaV`.
- DEC-068's fractional rounding.
- SPEC-079's script hardening.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes.
- **An existing assertion may be strengthened, never weakened or deleted.**
  - The one pre-approved exception is
    `tests/black_levels.rs::canon_per_position_differs_only_at_position_3` (SPEC-078
    AC5).
  - Any other pin or assertion that DEC-070 moves, beyond the AC5 list: **stop and
    report** before changing it.
- **If a measured number lands outside a pre-registered bound,** stop and report it.
  Don't widen the bound.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill
  the `## Handback` section.

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
