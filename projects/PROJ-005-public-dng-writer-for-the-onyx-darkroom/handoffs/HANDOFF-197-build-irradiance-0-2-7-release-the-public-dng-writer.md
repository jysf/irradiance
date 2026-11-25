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
  id: HANDOFF-197
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-095

project:
  id: PROJ-005
  stage: STAGE-032
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

# HANDOFF-197: irradiance 0.2.7 release: the public DNG writer

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-095` to a fresh `claude-opus-5-5`
session (acting as implementer) for the **build** cycle: Phase 1 of the 0.2.7 release, the
reversible prep PR. You never tag or publish.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-032-the-0-2-7-release-the-public-dng-writer.md`
- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md`
  (its "Build work" list is your task list, steps 1 to 11, in order).
- **The model to copy:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`,
  the 0.2.6 release: its Release Notes shape, internal record and filled Release
  Pre-Flight.
- **The previous stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
  (what shipped, and its reflection's limits).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-006`: the two-phase release and the pre-flight categories.
- `DEC-028`: irradiance publishes to crates.io, honestly scoped.
- `DEC-067`: `just semver-check` against the crates.io baseline decides the version.
- `DEC-055`, `DEC-071`: the writer lives in the crate behind the `write` feature; v1 scope.
- `DEC-080`: the two `Error` variants and their payload types exist in every build.
- `DEC-082`: the preview SubIFD (for the release notes' wording).

### Constraints that apply

- `library-not-application`: no binary ships; `irr` stays behind its dev feature.
- `no-new-top-level-deps-without-decision`: `[dependencies]` stays empty; the docs.rs
  table adds none.

### Prior related work

- `HANDOFF-187` / PR 321 / SPEC-090: the 0.2.6 prep; the same Phase 1, with the SDK lanes
  this release does not need.
- PRs 329, 332, 336, 339: the four writer builds; PR 342: the STAGE-017 close.

### Environment

- `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
- `IRRADIANCE_DNG_VALIDATE` set to the local `dng_validate` under `tools/dng_sdk` (never
  commit it or its outputs; DEC-069 §6).
- Run `just test`, not `just test --all-features` (the recipe already passes it).
- Use `/usr/bin/git` to establish branch and HEAD.

## Expected Deliverables

- Branch `release/0.2.7-prep` cut from an up-to-date `origin/main`, after step 1's check
  that no `src/` or `Cargo.toml` change has landed since `153f517`. **If one has (PR 343,
  SPEC-091, is open), stop with `[?]` and report; do not re-scope.**
- One PR against `main` carrying only: `Cargo.toml` (version and the
  `[package.metadata.docs.rs]` table), `CHANGELOG.md`, README wording, and this project's
  records (the spec's Release Notes, internal record and Pre-Flight; the timeline; this
  handoff's handback).
- The spec's `## Release Notes` and internal record written, the Pre-Flight filled
  `[now]` / `[cut]` / `N/A`, and every gate in step 11 green.
- The packaged-crate writer check (step 9) recorded with its output MD5s and
  `dng_validate` results.
- PR description referencing HANDOFF-197, SPEC-095, STAGE-032, PROJ-005 and the decisions
  above.

## Out of Scope

- Any change to `src/`, tests or fuzz targets. A defect found in prep goes in the
  handback; the stage is release-only.
- Tagging, `cargo publish`, a GitHub release: Phase 2, the user's.
- PROJ-006's SPEC-091 and the STAGE-017 close's two process-debt items.

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
