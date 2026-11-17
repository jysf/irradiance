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
  id: HANDOFF-153
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-16
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-073

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

# HANDOFF-153: BUILD: the irradiance 0.2.2 release prep (SPEC-073, Phase 1)

## Delegation Summary

You are the **builder** for SPEC-073, the 0.2.2 release. Cut `claude/spec-073-build`
from `origin/main` after the design PR merges. **This is Phase 1 only.** That means the
reversible prep:
- a semver check, then the version bump;
- the CHANGELOG promotion and the release notes;
- the evidence and the pre-flight.

**You never tag, push a tag, or publish.** The user cuts 0.2.2 after your PR merges.

**The user decided** the version (0.2.2, a compatible patch release) and **approved
installing `cargo-semver-checks`** from crates.io for a mechanical semver check.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-073-the-next-irradiance-release-semver-check-changelog-promotion.md`.
  Read all of it:
  - Release Scope (the design's API audit);
  - Build work items 1–8;
  - Release Notes (the drafted headline);
  - Release Pre-Flight;
  - the Notes (the Phase 2 runbook and the gotchas).
- **Precedent:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md`,
  the 0.2.0 release. Mirror its filled pre-flight and its notes style.
- **Decisions:** DEC-052 (semver and the sealed types), DEC-028 (publishing), DEC-006
  (release specs and the CHANGELOG), DEC-066 (`just test` now takes about 3 min).
- **Files:** `Cargo.toml`, `CHANGELOG.md`, `README.md`, the crate docs in `src/lib.rs`,
  and `.repo-context.yaml` (scheme semver, already set).
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.

## Expected Deliverables

1. **The semver check first.**
   - `cargo install cargo-semver-checks --locked`, then
     `cargo semver-checks check-release --baseline-rev v0.2.1` (record the tool version
     and its whole output).
   - **If it reports any breaking change, stop.** Write it up in the handback and set
     the spec `blocked: true` with a question. Do not bump to 0.3.0 or "fix" the API
     yourself.
2. **The prep** (spec Build work items 2–5): the version, the CHANGELOG, the release
   notes (from `just release-notes PROJ-004`, edited into the spec), and README or
   crate-doc accuracy fixes, wording only.
3. **The evidence** (item 6): `just test` with the corpus, and the three timing
   examples.
4. **The package guards** (item 7):
   - `just package-targets-check`;
   - `just deny` and `just deny-fuzz`;
   - `cargo publish --dry-run`, whose packaged manifest must have no self
     dev-dependency.
5. **The pre-flight** (item 8): fill every item `[now]`, `[cut]` or `N/A`, with the
   evidence.
6. **Gates, all green:**
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
7. **PR.** Open it against `main` from `claude/spec-073-build`, referencing SPEC-073
   and HANDOFF-153. Its body carries the release notes and the semver-checks result.
   Confirm CI is green on its head.

## Out of Scope

- Tagging, pushing a tag, `cargo publish`, a GitHub release, and `cargo yank`. All of
  these are Phase 2, for the user.
- Any API change, and any version other than 0.2.2.
- Library code changes beyond doc wording.
- New features.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, **tag or publish**.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
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
