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
  id: HANDOFF-173
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-083

project:
  id: PROJ-004
  stage: STAGE-026
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

# HANDOFF-173: BUILD: irradiance 0.2.4 release prep, Phase 1 (SPEC-083)

## Delegation Summary

You are the **builder** for SPEC-083. **Start only after the SPEC-083 design PR has
merged.** Cut `claude/spec-083-build` from that `origin/main`.

**The task:** Phase 1 of the 0.2.4 release, reversible prep in one CI-gated PR:
- the two small script fixes from SPEC-080's verify (`validate.sh` feedback status
  CR/quotes/BOM; `sdk-stage2.sh` silent exit on an unenterable worktree);
- `just semver-check` against crates.io 0.2.3;
- `Cargo.toml` to `0.2.4`, the CHANGELOG promoted, the release notes written;
- tier-B evidence (corpus, SDK cross-check, timings), package guards, the pre-flight and
  every gate.

You never tag or publish. Phase 2 is the user's.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-083-irradiance-0-2-4-release-dng-conformant-black-normalisation.md`
  (Release Scope, Build work 1–9, Release Notes, Pre-Flight, Notes).
- **The model:** SPEC-077, the 0.2.3 release
  (`specs/done/SPEC-077-irradiance-0-2-3-release-gainmap-and-black-level-fixes.md`). Follow
  its Pre-Flight and internal-record shape item for item.
- **What's being released:** the CHANGELOG `[Unreleased]` section (SPEC-078, SPEC-080),
  DEC-070 with both §3 amendments, and SPEC-080's verify follow-ups (HANDOFF-172's handback).
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Write its outputs into your scratchpad only, never into any checkout.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.

## Expected Deliverables

1. **A PR against `main` from `claude/spec-083-build`,** CI green, with the script fixes,
   the version bump, the CHANGELOG promotion and the filled spec (Release Notes, internal
   record, Pre-Flight `[now]` halves).
2. **The handback:** the semver summary verbatim; the release notes checked line by line
   against the spec's "every input whose result changes" list; the SDK numbers; the
   timings; the package file list; each script-fix scenario with its message.
3. **Gates,** all green, as Build work 9 lists.

## Out of Scope

- Any library code change beyond documentation wording.
- Tagging, publishing, or a GitHub release.
- STAGE-026's close and PROJ-004's close (the orchestrator does them at ship).

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- **A semver-check failure stops the build** for the user's decision.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus bytes, SDK bytes or SDK outputs.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your own
  transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill the
  `## Handback` section.
- Keep working until the PR and the handback are pushed; don't end a turn while a long
  run is still going.

---

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
