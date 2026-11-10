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
  id: HANDOFF-084
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-03
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-039

project:
  id: PROJ-002
  stage: STAGE-010
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

# HANDOFF-084: `plane::validate` — validate before allocate

> **⚠ SUPERSEDED — do not dispatch (DEC-045, 2026-11-10).** SPEC-039 was
> re-scoped to group E and now depends on SPEC-052/053/054; its uncompressed-only
> design (which this handoff delegates) is incomplete per the pre-0.2.0 code
> review (finding #5). A fresh build handoff is written when SPEC-039 reaches its
> re-design cycle. This file is kept for history only.

## Delegation Summary

The architect hands `SPEC-039` to the implementer for the **build** cycle: add a public,
allocation-free `plane::validate` and refactor `unpack_into` onto shared check helpers,
including one new rule (strips may not alias past the file) that makes the returned `n`
bounded by the file length. **Read SPEC-039 in full first; `## Design` §2 (error precedence)
and §3 (aliasing) are the non-obvious parts, `## Failing Tests` is the contract.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-039-plane-validate-validate-before-allocate-extract.md` — read all of it.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md`
- **Requirement:** `./feedback/2026-11-02-pyirradiance-requests.md` (item 1)
- **Code:** `src/plane.rs` (`unpack_into`), `src/ifd.rs` (`Sensor`, `MAX_TAG_VALUES`), `tests/support/tiff.rs`, `tests/plane_oracle.rs`, `fuzz/fuzz_targets/plane.rs`, `examples/fuzz-seeds.rs`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-016` — caller-owned buffers; `validate` allocates nothing.
- `DEC-008` / `DEC-029` — the packing rules and the "sample above WhiteLevel is legal" rule are untouched.
- `DEC-003` — tier-B corpus is never committed; the §3 corpus check is a local run only.
- `DEC-009` — control run before any red-proof.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — five deny lints; `validate` runs on hostile bytes.
- `oracle-must-be-shown-red` — the validate tests need a red-proof.
- `library-not-application` — zero I/O, caller-allocated; no cap on `n` in the library.

### Prior related work

- `HANDOFF-074`…`HANDOFF-084`'s siblings: SPEC-028 (multi-strip) introduced the strip loop being refactored; SPEC-027's CFA builders give the tier-A fixtures.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-039's Acceptance Criteria.
- All failing tests in SPEC-039 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `claude/spec-039-build-<suffix>`.
- PR description referencing: this handoff ID, the spec ID, the stage
  ID, the project ID, all referenced `DEC-*`, and any new `DEC-*`
  created during implementation.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- `Error::kind()` (SPEC-040); the determinism CI job (SPEC-041); the release (SPEC-042).
- Any cap on `n`, and any validate helper for `demosaic_into` / `to_linear_f32_into`.
- Changing any existing error variant's meaning or `Display` text.
- Also: **CHANGELOG** (`[Unreleased]` Added + a Changed note for the aliasing rule) and a **provenance-ledger** row are part of this build, as for SPEC-033.

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
