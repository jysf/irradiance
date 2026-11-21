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
  id: HANDOFF-177
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-177: Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-084` to `claude-opus-5-5` (acting
as verifier) for the **`verify`** cycle. You did not build this; review it cold.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-084-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md` (AC1 to AC7, the
  failing tests with hand-computed bytes, the exact outputs).
- **Build handoff and its handback:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-176-build-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md`.
  The builder's claims to check are there: seven mutants red, 793 passed locally,
  22 of 22 CI checks on `2419b34`, three deviations.
- **The build:** PR 298 (`claude/spec-084-build`). Your base branch,
  `claude/spec-084-verify-base`, is that branch plus this handoff.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-014-0-2-x-maintenance-post-release-correctness-and-hygiene.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-003` (tier-B never committed), `DEC-008` (the two unpack paths AC3 covers),
  `DEC-014` (the tri-state shape AC2 extends to `req`), `DEC-029` (why SPEC-016's old
  AC4 was dropped).

### Constraints that apply

- `oracle-must-be-shown-red`: re-run at least the AC1, AC2 and AC3 mutants
  yourself; don't trust the pasted output alone. Try at least one mutant the
  builder did not.
- `library-not-application`: confirm no non-test line under `src/` changed.

## Expected Deliverables (verify contract)

Check, at minimum:
1. **AC1.** The promise sentence is written only by `preflight_summary`, and only
   for a complete corpus with both tools. `corpus-status` uses the same two probes
   the tests use. Run `cargo run --all-features --example corpus-status` with a tool
   hidden from PATH (for example a PATH without `/opt/homebrew/bin`) and confirm the
   real output stops making the promise and names the tool. That is the behavioural
   check this AC exists for.
2. **AC2.** `req` refuses zero or several values with a message naming the tag; the
   live `metadata_oracle` passes with
   `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
   and `exiftool` on PATH. Use the corpus in place; never copy it.
3. **AC3.** The table's bytes match the spec's hand computation; the depth-set
   assertion really enumerates `SUPPORTED_BITS` rather than a copied list.
4. **AC4.** `continue-on-error` is gone from `determinism` and nowhere else changed in
   `ci.yml`; the macOS leg ran and passed on the PR head; the signal is `done`.
5. **AC5.** The exact message; `bash -n` clean.
6. **The builder's deviations and its optional follow-up** (the hard-coded
   `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` lists): judge whether each is acceptable.
7. Decision drift: `just decisions-audit --changed` if available.

Fix small doc or test issues yourself on `claude/spec-084-verify-base` (commit,
push); anything larger is a punch-list item. Output exactly ONE of ✅ APPROVED (at
a commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Repo rules a cold session trips on

- Commit-message bodies are plain prose: no code tokens. A push with them is
  rejected; reword, don't retry.
- Never add `Cargo.lock`; stage explicit paths. Run `just test`, not
  `just test --all-features`. `cargo test --no-fail-fast` for totals.
- Use `/usr/bin/git` for authoritative branch state.
- Don't force-push, merge, tag or publish, and don't push to `claude/spec-084-build`;
  push only to `claude/spec-084-verify-base`.
- Keep working until the verdict is written and the handback is filled and pushed.
  If you start a long background run, wait for it; do not end your turn while it runs.

## Out of Scope

- Any `src` behaviour change, release or CI change beyond AC4.
- Re-designing the spec. Disagreement with the design is a punch-list note, not a rewrite.

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
