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
  id: HANDOFF-062
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-28
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-023

project:
  id: PROJ-001
  stage: STAGE-005
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

# HANDOFF-062: audit every gate script for the mute-death shape

## Delegation Summary

The orchestrator (architect) hands `SPEC-023` to the implementer for the
**build** cycle: add a bespoke `scripts/gate-audit.sh` + its red-proof over the
verdict-bearing gate scripts (a check *over* the gates for the mute-death
shape), and fix the three reachable offenders it flags — `validate.sh`
(process-sub exit / no `parsed` assert), `handback-sync.sh` (no `superseded`
skip), `decisions-audit.sh` (unguarded `grep`) — the first two being SPEC-016's
two follow-ups. **No `src/` change.** The design is fully specified — read the
spec.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-023-audit-every-gate-script-for-the-mute-death-shape.md`
  — read it whole: **Inputs** is the offender map (exact `file:line`), **AC1–AC5**
  the contract, **Notes** the idioms and the `HANDOFF-038` de-kludge.
- **The shape:** `AGENTS.md §16 rule 3` (~`:1424–1437`) + the signal
  `a-gate-that-fails-mutely-is-a-gate-that-never-ran` (`guidance/signals.yaml`).
- **The red-proof template:** `scripts/validate-red-proof.sh` (control / inject /
  assert-landed / rc-capture / assert-message / isolation); CI pairs a gate with
  its red-proof (`.github/workflows/ci.yml:24–33`).
- **The three offenders:** `scripts/validate.sh` (~`:300–311`, `:392`; helper
  `_lib.sh:372–399`), `scripts/handback-sync.sh` (~`:51–90`, `:72`, `:164–175`;
  live proof `HANDOFF-038`), `scripts/decisions-audit.sh` (~`:217`). Plus the
  lower-risk variants `cost-audit.sh`, `_lib.sh:135–137`, `test.sh`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — pinned vs local clippy,
  zsh `$pipestatus`, the `just` gate recipes. Read it.

### Decisions that apply

- `DEC-009` — a red-proof needs a negative control (the shape to mirror).
- `DEC-017` — the red-proof rebuilds a mutated COPY (working tree untouched).
- `DEC-006` — lint-policy red-proof mechanics (lineage). There is **no DEC on
  gate-script conventions** — that lives in AGENTS.md §16; do not write a new one.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `scripts/`, `ci.yml`,
`app.just`, one handoff, AGENTS.md:

- `oracle-must-be-shown-red` (~`:53–56`) — every gate (incl. the new
  `gate-audit`) ships with a deliberate-fault test + a clean negative control.
- `test-before-implementation` — the red-proof + the two behavioral fixes are
  the spine.

### Prior related work

- **SPEC-016** (shipped, PR #23/#24) — fixed ONE gate (`validate` now parses);
  its two follow-ups (validate process-sub exit, handback-sync superseded-skip)
  are two of this spec's offenders. `validate-red-proof.sh` is its red-proof.
- **SPEC-011** (shipped, PR #28/#30) — the most recent shell-gate + red-proof +
  CI-job spec; the shape to copy for `gate-audit` + its CI pairing.
- **SPEC-003** (shipped) — widened `oracle-must-be-shown-red` to cover gates
  (FU-14); the same uncovered-graph argument DEC-011 made for licensing.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`scripts/gate-audit.sh`** (new) — the bespoke scanner (comment-excluding,
  asserts its own scan count) + **`scripts/gate-audit-red-proof.sh`** (mirror
  `validate-red-proof.sh`); `app.just` recipes + AGENTS.md §6; two CI jobs paired
  adjacently (AC1, AC2).
- **The three offender fixes** (AC3): `validate.sh` (rc-capture + `parsed ==
  target` assert + broaden the ruby rescue; a non-SyntaxError fault goes RED),
  `handback-sync.sh` (skip `handoff.status == superseded` + **de-kludge
  HANDOFF-038**: restore its `handback` to null, drop the manual `synced_at`),
  `decisions-audit.sh` (guard the `grep`). Each proven.
- **Any lower-risk variant gate-audit flags** (`cost-audit.sh`, `_lib.sh`,
  `test.sh`) fixed so the audit ships green (AC4); name which in the handback.
- **No `src/` change** — assert `git diff` touches no `src/`.
- All gates green, `just lint-ci` + the existing suite + the two new jobs, CI
  observed green on the PR head; name the gate list.
- The handback filled with a REAL transcript-sum `tokens_total` (deduped by
  `message.id` via your scratchpad UUID — say so; **not a budget-delta**). A PR
  against `main` referencing HANDOFF-062, SPEC-023, STAGE-005, PROJ-001, DEC-009,
  DEC-017.

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

- **The `next_id` / `new-spec` fixture-pollution bug** — recorded as a signal
  (`next-id-scans-test-fixtures`); it is a scaffolder, not a gate. Separate patch.
- **`shellcheck` as the engine** — bespoke `gate-audit` is primary; a
  complementary shellcheck CI job is optional, defer if it churns.
- **The view / scaffold scripts** (`*-view.sh`, `dash`, `status`, `new-*`) —
  out of the audit's subject set.
- **Any `src/` change**; rewriting gate logic beyond the mute-death fixes.
- The four deferred `(not yet written) [XS]` STAGE-005 items.

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
