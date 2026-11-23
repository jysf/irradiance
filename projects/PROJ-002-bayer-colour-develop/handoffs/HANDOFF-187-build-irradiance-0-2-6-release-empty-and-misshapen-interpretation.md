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
  id: HANDOFF-187
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-23
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-090

project:
  id: PROJ-002
  stage: STAGE-030
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

# HANDOFF-187: irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-090` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle: Phase 1 of the 0.2.6 release, reversible prep
only. You never tag or publish.

## Context the Receiving Agent Needs

Read these, in this order:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`.
  The Release Scope's changed-input list (with its two **confirm** items) and the nine
  build steps are the contract.
- **The model:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-088-irradiance-0-2-5-release-calibration-converged-weight-unhono.md`,
  the 0.2.5 release spec. Use its release notes, internal record and filled pre-flight
  as the shape.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-030-the-0-2-6-release.md`.
- **What is being released:** SPEC-089 in `specs/done/` (and its build and verify
  handbacks, HANDOFF-185 and HANDOFF-186), DEC-078 §7, and the `[Unreleased]` CHANGELOG
  entry. `git diff v0.2.5 origin/main -- src/` touches only `src/ifd.rs` and
  `src/color.rs`.
- **`AGENTS.md`**: §15 "Cutting a release?" and the gates.

### Decisions that apply

- `DEC-067` — `just semver-check` against the crates.io baseline decides compatibility.
- `DEC-006` — the release pre-flight categories.
- `DEC-069` §6 — the SDK stays local: never committed, packaged or run in CI.
- `DEC-078` §7 (and the DEC-076 §3 amendment note) — what changed, for the notes.

### Constraints that apply

- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

### Prior related work

- `HANDOFF-184` — the 0.2.5 prep, the same nine-step shape.
- `PR #319` — SPEC-089's build and verify.

## Expected Deliverables

- A PR against `main` from branch `release/0.2.6-prep`, carrying build steps 1 to 9:
  - the changed-input reconciliation, with both **confirm** items answered;
  - the version bump and the promoted `## [0.2.6]` section, with any CHANGELOG wording
    the reconciliation contradicts corrected;
  - the release notes and the internal record;
  - the filled pre-flight.
- Use `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  The SDK objects and `dng_validate` are under
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk`.
- Run `scripts/sdk-colour.sh` and `scripts/sdk-stage2.sh` into your scratchpad, never
  into the repository. Build the `v0.2.5` `irr` for the `irr ifd` diff in a scratch
  worktree of the tag.
- **Stop and report, do not fix,** if:
  - `semver-check` reports a breaking change;
  - a held pin moves, or `irr ifd` differs from `v0.2.5` on a held file;
  - the SDK colour maxima differ from 6 / 6 / 1 / 2 / 5;
  - the stage-2 table differs from 0.2.5's;
  - the changed-input diff finds a behaviour change that SPEC-089 does not describe.

## Out of Scope

- Any library behaviour change. The stage is release-only.
- Tagging, `cargo publish`, and the GitHub release: the user's Phase 2.
- The cold verify: a separate handoff after your PR is up.

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
