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
  id: HANDOFF-009
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-006

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

# HANDOFF-009: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-006` for the **verify** cycle, at
`618fd6f`. Independent session; that independence is the point.

⚠ **ID note:** this is `HANDOFF-009`, renamed by hand. `just new-handoff`
allocated `008`, which `SPEC-002`'s build handoff already holds on its own branch
— the command counts what is visible in the current worktree, so parallel
branches collide. Do not renumber it back.

Context worth having: SPEC-001's equivalent gate took **three** build rounds and
three verifies, each round found insufficient by a reviewer and believed closed by
its author. This spec exists because of what round 3 found.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat

- Honest tree: **all eight gates exit 0**.
- `#[allow(clippy::panic, clippy::expect_used)]` planted on a `pub fn`: seven
  gates green, **NO-ALLOW 101**, `E0453 … overruled by previous forbid` naming
  the exact attribute. It is the only gate that sees it.
- Inner `#![allow]` spelling: also 101.
- `scripts/lint-red-proof.sh` and `src/lib.rs` confirmed untouched
  (`git diff main..HEAD`).

### What deserves scrutiny

1. **Is `--lib` the right scope, and is the claim honest?** The gate covers the
   library target only, and `constraints.yaml:33` was rewritten to say so. SPEC-001's
   F-4 was raised because the previous wording *overstated*. Does it now overstate,
   or **understate**?
2. **Try to bypass it.** `-F` is a compiler-level forbid, so this should be far
   harder than SPEC-001's shell script — but that is an assumption, not a finding.
   Ideas: an `#[allow]` behind a `cfg`; `#[cfg_attr(..., allow(...))]`; an
   `#[expect(...)]` attribute; a lint alias; something in a test or `src/bin/`
   that leaks into `--lib`.
3. **Three disclosed deviations** — CI inlines the cargo invocation instead of
   calling `just` (because `just` isn't on `ubuntu-latest`, found by *executing*
   the extracted YAML rather than reading it); a branch reset; an `AGENTS.md` §6
   addition. Confirm each is accurate and confined.
4. **A gap in my spec the builder found:** it never said *where* to plant the
   attack, and planting after the `#[cfg(test)]` module trips
   `clippy::items_after_test_module`, turning CLIPPY red for an unrelated reason.
   Should the red-proof pin the placement?

### Settled — do not reopen

`DEC-009`'s red-proof (complementary, deliberately untouched) · the `-F`
mechanism (three properties measured at design) · MSRV 1.90 · SPEC-002's work.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per
`AGENTS.md` §15 "During verify".

Run the planted-`#[allow]` attack yourself — check #9. ⚠ Plant it **before** the
`#[cfg(test)]` module (item 4), and mind the `attribute-text-inside-doc-comments`
signal (**N=5**): `src/lib.rs` carries attribute text in its own module docs.

**Label every finding ship-blocking or follow-up.** A defect letting a panic reach
the library is ship-blocking; a sharp edge that fails loudly is a follow-up — file
it and approve. If the gate is sound, say so plainly.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- `scripts/lint-red-proof.sh`; re-litigating `DEC-009`.
- SPEC-002's branch. ⚠ Both branches touch `app.just`; a conflict is expected and
  is the orchestrator's to resolve. Do not pre-reconcile it.

## Return Criteria — how to hand back

1. Verify cost session with a real `tokens_total`; if `/cost` is unavailable, sum
   transcript usage objects and **say so**, with cache-read share.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with SHA, every finding labelled ship-blocking or follow-up.
4. Commit on `feat/spec-006-allow-attribute-gate`; do not merge.

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
