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
  id: HANDOFF-057
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual verifier model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-016

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
  status: completed                # completed | blocked | rejected
  tokens_total: 3788150            # REAL combined count — what cost-audit reads
  estimated_usd: null              # Max subscription, extra-usage disabled — no per-token meter; tokens_total is the real figure
  duration_minutes: 18
  branch: claude/mystifying-tharp-ee7171
  pr: null                         # verify opens no PR — it reviews the build's PR #23 cold
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "APPROVED at d1cad62. Reproduced the red-proof independently (OLD validate.sh passes the SPEC-900 fixture, NEW fails by name with the Psych message), broke a real handoff live to prove the broadened scope catches it, confirmed missing-ruby dies loudly and HANDOFF-039 is fixed (121 parsed / 0 fail). One non-blocking follow-up: a narrow residual mute-gate if ruby is present but crashes for a non-SyntaxError reason. tokens_total is a FLOOR, deduped by message.id (30 billed msgs); no per-token USD meter."
  synced_at: 2026-10-27
---

# HANDOFF-057: the harness stops claiming what it has not checked

## Delegation Summary

The orchestrator (architect) handed `SPEC-016` to the verifier for the
**verify** cycle: an independent cold review of the build's PR #23. Filled
retrospectively by the orchestrator from the verify session's report (session
`local_c810d40d`, branch `claude/mystifying-tharp-ee7171`); the operative verify
contract was that session's brief, not the scaffold below.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-<slug>/brief.md`
- **Stage:** `./projects/PROJ-001-<slug>/stages/STAGE-005-<slug>.md`
- **Spec:** `./projects/PROJ-001-<slug>/specs/SPEC-016-<slug>.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `HANDOFF-YYY` — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-016's Acceptance Criteria.
- All failing tests in SPEC-016 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-XXX-<slug>`.
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

- ...

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

- **Branch / PR:** `claude/mystifying-tharp-ee7171` — verify opened no PR; it
  reviewed the build's PR #23 (`claude/spec-016-build-9736e8`) cold.
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** at PR #23 head `d1cad62d2e9569713dcda6b4157e2d37ec42aaff`.
  - **AC1** — `_lib.sh:front_matter_parse_report` uses ruby `YAML.parse`
    (syntax-only; rescues only `Psych::SyntaxError`), prints
    `FAIL<TAB>file<TAB>message`. Scope = 121 artifacts (`projects/**` incl.
    `handoffs/`+`patches/`, plus `decisions/`), a strict superset of the old 26.
    Required-keys check retained. A missing parser `die`s loudly (ran with ruby
    off PATH → exit 1) — the mute-gate class closed.
  - **AC2** — red-proof reproduced independently (not via the committed script):
    ran the OLD (main) and NEW (PR) scripts on the SPEC-900 fixture — OLD exit 0
    (passes, does not flag), NEW exit 1 (fails, flags by name) with
    `did not find expected key while parsing a block mapping at line 15 column 1`.
    Per-file Psych messages carry different line/col (non-uniform), vs the
    `safe_load(permitted_classes:)` masquerade that fails uniformly.
  - **AC3** — broadened scope caught live: broke a real handoff (HANDOFF-050),
    NEW gate flagged it by name, reverted. HANDOFF-039 genuinely fixed. `just
    validate` green at **121 parsed / 0 failures**; `just validate-red-proof`
    PASS. CI green on `d1cad62` across 11 jobs (observed at step level).
  - **No `src/` change**; `just decisions-audit --changed` clean.

### Cost self-report

- **Tokens (total):** **3,788,150** — transcript sum DEDUPED BY `message.id`
  (30 billed messages; in 60 / out 40,530 / cache-create 107,362 / cache-read
  3,640,198). A FLOOR — excludes the report-writing turns.
- **Estimated USD:** null — Max subscription, extra-usage disabled; no per-token
  meter.
- **Duration (minutes):** ~18.
- **Source of the number:** harness report — the verify session's own transcript
  `usage` objects, located by its scratchpad UUID and summed. `/cost` is
  unavailable in this desktop Code session.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none.
- **Follow-up work identified:** one, non-blocking — a narrow residual of the
  mute-gate class: `validate.sh` reads `front_matter_parse_report` through a
  process substitution whose non-zero exit is invisible to `set -e`, so if ruby
  is PRESENT but crashes for a non-`Psych::SyntaxError` reason (e.g. a future
  invalid-UTF-8 artifact raising in `l.rstrip`), no FAIL/PARSED lines emit,
  `parsed` stays 0, and the gate could report green. Latent today (121/121 clean
  UTF-8). Cheap guard: assert a PARSED line was seen, or check the subshell rc.
  Folds naturally into STAGE-005's existing gate-script-audit backlog item.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's Design refresh named the uniform-failure trap and the
   live falsifier (HANDOFF-039), and AC2 spelled out the old-vs-new red-proof.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` framed the whole review.

3. **If you did this task again, what would you do differently?**
   — Nothing material; every AC reproduced on the first pass.
