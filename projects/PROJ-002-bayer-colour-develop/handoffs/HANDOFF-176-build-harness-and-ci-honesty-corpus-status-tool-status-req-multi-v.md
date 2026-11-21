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
  id: HANDOFF-176
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-21
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-084

project:
  id: PROJ-002
  stage: STAGE-014
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

# HANDOFF-176: Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-084` to `claude-opus-5-5` (acting
as implementer) for the **`build`** cycle.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-084-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md`. It is the
  contract. Its Inputs section has the line numbers and the measurements taken at
  design; its Failing Tests section has the hand-computed AC3 bytes.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-014-0-2-x-maintenance-post-release-correctness-and-hygiene.md`
  (the "Second batch" note says where each item came from and why AC4 was dropped).
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-003`: tier-B corpus files are never committed; every new test here is tier A.
- `DEC-008`: the two uncompressed unpack paths (byte-aligned 8/16, MSB-first
  bitstream 12/14); AC3 covers both.
- `DEC-014`: the optional exiftool fields' absent / unreadable / value shape; AC2
  gives `req` the same refusal to keep a truncated head.
- `DEC-029`: a sample above WhiteLevel is legal, which is why SPEC-016's old AC4 is
  dropped. Do not resurrect it.

### Constraints that apply

- `oracle-must-be-shown-red`: AC6's mutants, each watched red then restored.
- `test-before-implementation`: write the three tests first and see them fail on
  `main`'s code (AC1 and AC2 fail; AC3 passes on `main` and must go red under its
  mutants).
- `one-spec-per-pr`.
- `library-not-application`: no `src` behaviour change; the only `src/` edit is a
  `#[cfg(test)]` test.

### Prior related work

- SPEC-016 (pre-rescope, git 3238dcb): the original AC1 to AC4 text.
- SPEC-005/FU-2 and SPEC-010/FU-2: the `req` truncation, left open twice because the
  check was written narrower than the finding. Write AC2's test to the finding.
- SPEC-083 verify FU-3 (HANDOFF-174): the `sdk-stage2.sh` wording.
- Signal `macos-arm64-runner-unavailable` in `guidance/signals.yaml`.

## Expected Deliverables

- The changes in the spec's Outputs, meeting AC1 to AC7.
- The three named tests passing, and the AC6 mutants each shown red, with the command
  and the failing assertion pasted into the Handback.
- A PR against `main` from `claude/spec-084-build`, referencing HANDOFF-176, SPEC-084,
  STAGE-014, PROJ-002 and the DECs above.
- Keep working until the PR is open and the handback is filled. If you start a long
  background run, wait for it; do not stop and report while it is still running.

### Repo rules a cold session trips on

- Commit-message bodies are plain prose: no code tokens (no backticks, `#[...]`, `->`,
  `--flag`). A push with them is rejected. PR bodies and files are fine.
- `Cargo.lock` is gitignored; never add it. Stage explicit paths, never `git add -A`.
- Run `just test`, not `just test --all-features` (the recipe already passes it).
- Use `cargo test --no-fail-fast` for totals; a plain run stops at the first failing
  binary.
- Corpus: `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place; never copy them into the repo.
- Don't force-push, merge, tag or publish.

## Out of Scope

- Any `src` behaviour change, `DEVELOP_VERSION` bump or release.
- Requiring or installing `exiftool` or `dnglab` in CI.
- Any CI change beyond the `determinism` job's `continue-on-error` and its comment.
- The dropped SPEC-016 AC4 (fuzz seeds and `SampleExceedsWhiteLevel`).

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
