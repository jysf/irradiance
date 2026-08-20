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
  id: HANDOFF-006
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-001

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-006: <Task Title — same as the spec's title>

## Delegation Summary

Third verify cycle on `SPEC-001`, at `00f098b`. Round 2's two P1s are addressed
per **`DEC-009`** (supersedes `DEC-007`, which superseded `DEC-006`).

**Read the last paragraph of "Expected Deliverables" before you start.** This gate
has had three build rounds, and knowing when to approve is part of this job.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat it

- Seven gates re-run: green. Spec front matter parses; `cost.totals` 41,017,417
  across 5 sessions.
- **Round 2's exact PL-1 bypass re-run**: `//` comment before `#![forbid]`,
  `#![deny(`→`#![allow(` at column 0, two panicking public functions. Previously
  seven green with the proof printing ✓. Now **`REDPROOF 1`**, and the message
  correctly attributes cause: *"the control run above was clean, so this is the
  policy's fault and nothing else."*
- The `_lib.sh` YAML fix is correct — strip trailing comments only from unquoted
  scalars. Reproduced the old truncation.

### What deserves scrutiny

1. **Is the negative control actually load-bearing?** It is the claim that closes
   the class. Try to construct a state where the **control passes** but the
   mutation run is still meaningless.
2. **Three new supporting mechanisms** arrived with it: lint matching on clippy's
   `index.html#<lint>` help line, diagnostics asserted to fall inside the injected
   line range, and a third run without `-D warnings` pinning the policy at `deny`.
   Each is new surface. Are they sound, and is the third run doing something the
   other two don't?
3. **A fifth bypass was disclosed and deliberately not fixed** — the proof pins the
   policy at the **crate root only**, so a module with its own `#![allow(...)]` is
   uncovered. The orchestrator agreed this is a crate-shape decision, not a script
   one, and has **attached the obligation to `SPEC-003`** (which creates the first
   module). Confirm that disposition is right, and that nothing else silently
   depends on the uncovered case today.
4. **The `_lib.sh` fix is partial by choice.** `get_handoff_field()` (:283) and
   `get_spike_field()` (:327) carry the same bug, unfixed because no live caller
   passes them free text. Is "no current caller" an adequate reason, or is that
   the next silent corruption?
5. **Disclosed out-of-scope edits:** `app.just` and `AGENTS.md`. Confirm accurate
   and confined.
6. **`constraints.yaml`'s `enforcement:`** now names the red-proof as the only
   mechanical enforcement and calls it load-bearing. Is that claim now *true* —
   given item 3?

### Settled — do not reopen

MSRV 1.90 · fuzz deferral to SPEC-003 · `[lints]` in `Cargo.toml` · `core::` ·
`AGENTS.md` §7 · the `signals.yaml` merge divergence · cost figures (5 sessions,
41,017,417; the methodology mismatch is tracked as its own signal).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, working
`AGENTS.md` §15 "During verify". Run the policy-removal attack yourself (check #9)
— a red-proof you did not personally observe failing is a self-report.

⚠ **Mind the `attribute-text-inside-doc-comments` lesson signal (N=3, at bar).**
`src/lib.rs` contains two occurrences of `#![deny(`; the second is prose. Anchor
at column 0. Both the orchestrator and round 2's reviewer walked into this.

### On knowing when to approve

This is the **third** build round on one gate in a scaffold spec, and each prior
round was correctly found insufficient. That history argues for care — but it
also means the marginal round is getting expensive, and *"I found something"* is
not automatically *"this must not ship."*

So: separate **ship-blocking** from **follow-up**. A defect that lets a panic
reach the library is ship-blocking. A sharp edge in a dev script that fails loudly
is a follow-up — file it as a signal or a spec and approve. If the gate is sound
and the remaining items are follow-ups, **approve and say so plainly**; the
project needs this crate to exist so SPEC-002 onward can start.

If **APPROVED**, set `task.cycle: ship`; the orchestrator runs ship.

## Out of Scope

- Fixing anything. Punch-list with file:line.
- Re-litigating settled items above.
- The `signals.yaml` divergence (three signals on `main`, two here; resolution is
  "keep all five").
- Any decoding work.

## Return Criteria — how to hand back

1. Append a **verify** cost session with a real `tokens_total`; if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with the SHA reviewed, and for each finding an explicit
   **ship-blocking / follow-up** label.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

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
