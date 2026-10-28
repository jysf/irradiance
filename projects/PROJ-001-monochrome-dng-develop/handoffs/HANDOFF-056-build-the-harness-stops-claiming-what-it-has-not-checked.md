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
  id: HANDOFF-056
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: pending                  # pending | accepted | completed | rejected

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

# HANDOFF-056: the harness stops claiming what it has not checked

## Delegation Summary

The orchestrator (architect) hands `SPEC-016` to the implementer for the
**build** cycle: rewrite `just validate` to actually PARSE every artifact's
front matter with a real YAML parser and fail loudly (naming the file and the
parser's own message), with a red-proof. The design is complete and freshly
re-measured — read the spec's **Design refresh (2026-10-27)**. This is a
shell-script build; no `src/` change.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-016-the-harness-stops-claiming-what-it-has-not-checked.md`
  — read the whole thing, especially **Design refresh (2026-10-27)** (current
  numbers), **AC1–AC3**, **Failing Tests**, and **Traps**.
- **The gate under rewrite:** `scripts/validate.sh` + `scripts/_lib.sh`. Today
  it reads front matter with line-oriented `awk`/`grep`/`sed` and NEVER parses;
  it also scopes to **specs + patches + spikes only** (26 artifacts) — not
  `handoffs/` or `decisions/`.
- **The live falsifier in the repo:** `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-039-verify-orchestration-cost-is-documented-at-stage-close-and-nothing-checks-it.md`
  — its front matter does not parse (`did not find expected key …` ~line 49).
  It is outside today's `validate` scope, which is exactly why it slipped
  through. **Fix its front matter as part of this spec** so the new gate ships
  green (that fix IS the gate doing its job on its first run).
- **Toolchain brief:** `./guidance/toolchain-brief.md` — pinned vs local clippy
  (`lint-ci` not `lint`), the `+toolchain` PATH trap, zsh `$pipestatus`, the
  `just` gate recipes. Read it (DEC-004 rule 5).
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-005-harden-the-harness-the-gates-the-oracle-s-parser-and-the-second-cargo-graph.md`

### Decisions that apply

- `DEC-013` — the field-shape rules `validate` is entitled to check. The parse
  gate does NOT recheck DEC-013's rules; it only ensures the parser can REACH
  them. Keep the existing "required keys present" check (it catches valid YAML
  with missing keys, which a parse alone would not).
- `DEC-017` — the red-proof mechanism (mutate a copy, show old-vs-new). AC2's
  red-proof follows this shape for a shell gate.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `scripts/` + `tests/` +
one broken handoff:

- `oracle-must-be-shown-red` — the new gate MUST be proven red: show the OLD
  script PASSING on a parse-broken fixture and the NEW script FAILING on the
  same bytes, **with the parser's message** (AC2's third clause — an exit code
  alone does not prove the check ran; a wrong parser API fails uniformly, see
  the spec's Design refresh).
- `test-before-implementation` — the failing test
  (`validate_rejects_front_matter_that_no_parser_can_read`) is the deliverable's
  spine; write it first.
- `cost-captured-per-cycle` — fill the handback with a REAL `tokens_total`
  before reporting done (DEC-013).

### Prior related work

- `HANDOFF-038` (build) — the pre-rescope L-sized attempt at this spec; its
  context is at git `3238dcb`. This spec is the S-sized AC5-only slice.
- `guidance/signals.yaml` — `handback-sync-truncates-multi-line-scalars` (the
  defect class that produces the broken front matter), `named-tests-can-pass-vacuously`
  and `the-gate-count-is-not-defined-anywhere` (AC3 works around the last one).

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`scripts/validate.sh` rewritten** to parse every artifact's front matter
  under `projects/**`, `decisions/`, `handoffs/`, `patches/` with a real YAML
  parser (`ruby -ryaml` recommended — confirm it on the CI runner), failing
  non-zero on a parse error and printing the file + the parser's own message.
  Keep the existing required-keys check. Name the parser you chose in the
  handback and confirm it exists on CI.
- **The failing test** `validate_rejects_front_matter_that_no_parser_can_read`
  (AC1) — a hand-crafted fixture (unterminated quote in a multi-line scalar, the
  `handback-sync` shape). State its form (bats / standalone shell / appended
  case) in the handback.
- **AC2 red-proof**, recorded in the handback: OLD script PASSES the fixture,
  NEW script FAILS it, both on the same bytes, with the parser's message shown.
- **HANDOFF-039's front matter fixed** so the broadened gate ships green; report
  the cause (the `did not find expected key` break) and the repo-wide count
  after the fix (expected 0 of 120).
- **CI green** on the PR head (AC3) — `just lint-ci` (pinned clippy); name the
  gate list you ran in the handback. Observe CI, don't infer.
- **A PR against `main`.** Description references HANDOFF-056, SPEC-016,
  STAGE-005, PROJ-001, DEC-013, DEC-017.

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

- **The four deferred pre-rescope ACs** (corpus-status overclaim, `req()`
  truncation, `SUPPORTED_BITS` coverage, plane fuzz `white_level`) — separate
  STAGE-005 backlog items, measured at `3238dcb`.
- **The full gate-script audit** (the `pipefail` + unguarded-`grep` sweep) — a
  separate STAGE-005 item; folding it in makes this XL.
- **Fixing `handback-sync`'s writer.** This spec makes the defect DETECTABLE at
  validate-time; it does not fix the producer. (Fixing the one existing broken
  artifact, HANDOFF-039, so the gate ships green is in scope; rewriting the
  tool that produced it is not.)
- **Defining "the gates."** `the-gate-count-is-not-defined-anywhere` is open;
  AC3 works around it — just name the list you ran.
- **Anything in `src/`** — no decoder behaviour changes.

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
