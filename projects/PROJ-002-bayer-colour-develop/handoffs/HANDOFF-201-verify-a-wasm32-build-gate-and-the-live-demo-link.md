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
  id: HANDOFF-201
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-096

project:
  id: PROJ-002
  stage: STAGE-033
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

# HANDOFF-201: Verify SPEC-096, a wasm32 build gate and the live demo link

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-096` to a separate `claude-opus-5` agent (verifier)
for the **verify** cycle, cold, on branch `chore/spec-096-wasm32-build-gate` after HANDOFF-200.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-096-a-wasm32-build-gate-and-the-live-demo-link.md`.
  Its design-time probe, Outputs, the two mutations and the pre-registered mutants M1-M4
  are binding.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-033-the-browser-target-stays-buildable.md`
- **Shape to copy:** `scripts/doc-check.sh`, `scripts/doc-check-red-proof.sh` (SPEC-056).
- **Toolchain brief:** `./guidance/toolchain-brief.md`. Locally, Homebrew's cargo/rustc
  come first on PATH and have no wasm32 std; prefix `PATH="$HOME/.cargo/bin:$PATH"`.

### Decisions that apply

- `DEC-002` (proposed) — keeps the browser target open; this spec gates it, does not amend it.
- `DEC-009` — the red-proof mechanism: negative control first, then an attributed red.

### Constraints that apply

- `oracle-must-be-shown-red` — a gate ships with its red-proof and a control.
- `library-not-application` — no app, CLI or wasm-bindgen surface in the crate.
- `one-spec-per-pr`, `cost-captured-per-cycle`.
- **Build handback:** `HANDOFF-200` (its `## Handback`).

## The verify contract

You are **not** the agent that implemented this. Review it cold.

- AC1-AC7 met? Run `just wasm-check` and `just wasm-check-red-proof` yourself (prefix
  `PATH="$HOME/.cargo/bin:$PATH"` locally) and watch the reds happen (AGENTS.md §15
  check 9).
- Run the pre-registered mutants M1-M3 against a temp copy of `scripts/wasm-check.sh` (never
  the working tree) and report which the red-proof kills; read the red-proof for M4.
- Check the spec's claim that `Instant::now()` / `std::fs` compile on wasm32 (the reason the
  red-proof does not use them).
- `just gate-audit`, `just validate`, `just decisions-audit --changed`.
- README link text accurate (it runs the published crate compiled to WebAssembly).
- Output exactly ONE of: ✅ APPROVED (at SHA) / ⚠ PUNCH LIST / ❌ REJECTED, each finding
  labelled `SB-N` or `FU-N`.

## Out of Scope

- A wasm runtime test lane (already filed as a follow-up in the spec).

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
