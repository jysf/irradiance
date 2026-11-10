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
  id: HANDOFF-109
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-052

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

# HANDOFF-109: Parser resource guard — aggregate entry/byte budget (SPEC-052, group A)

## Delegation Summary

The architect hands `SPEC-052` to the implementer for the **build** cycle: add an
aggregate IFD entry-table byte budget to the container walk so a small hostile file
cannot amplify into a large metadata parse (review finding #1). The whole decision
is **DEC-046** — a running total of each IFD's `count × 12` bytes, rejected the
moment it exceeds the file length, checked inside `read_ifd` before the per-IFD
allocation. **Read SPEC-052 in full and DEC-046 first; the guard is ~10 lines, the
care is in the threading, the `checked_add`, and the fixture that isolates the new
guard from the cycle guard (distinct offsets).**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-052-parser-resource-guard-aggregate-entry-byte-budget.md` — read all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-046-bound-aggregate-ifd-entry-table-bytes-by-file-length.md` — the full rationale, the exact rule, and the measured corpus baseline.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #1 and its evidence).
- **Code:** `src/ifd.rs` (`Container::parse` l.848, `walk_chain` l.905, `read_ifd` l.955, `MAX_IFDS` l.69, `ENTRY_SIZE` l.81, `Entry` l.339), `src/lib.rs` (`Error` l.217, `TooManyIfds` + its `Display` l.258/l.763), `tests/support/tiff.rs` (`Ifd::new`, `tiff`, `at_offset`), `fuzz/fuzz_targets/ifd.rs`, `examples/fuzz-seeds.rs`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap. Read it (DEC-004 rule 5).

> ⚠ The spec's `src/ifd.rs` line numbers were read on 2026-11-10 against the
> current tree; re-confirm them before editing in case of drift.

### Decisions that apply

- `DEC-046` — the aggregate-table-bytes guard this spec builds (the rule, the error variant, the mechanics).
- `DEC-045` — the re-plan that created this group; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus is never committed; the corpus check is a local run only.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — five deny lints; the guard runs on hostile bytes; use `checked_add`.
- `oracle-must-be-shown-red` — the amplification test needs a red-proof against a mutated guard.
- `library-not-application` — zero I/O; no allocation added; the library sets no cap on `n` beyond the file-length bound.

### Prior related work

- `HANDOFF-084` (SPEC-039) — **superseded**; its validate-before-allocate work is the plane analogue of this, and its `StripBytesExceedFile` idea is the same "declared bytes may not exceed the file" idiom (DEC-046 "Why this shape").
- `SPEC-003` — the IFD reader + its fuzz target being extended here.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-052's Acceptance Criteria (the DEC-046 guard).
- All failing tests in SPEC-052 now passing, including the red-proof shown red first.
- The `fuzz/fuzz_targets/ifd.rs` extension + a new aliased-table seed; run `just fuzz` 60 s clean.
- CHANGELOG `[Unreleased]` entry and a `docs/provenance-ledger.md` row.
- A PR against `main` from branch `claude/spec-052-build-<suffix>`.
- PR description referencing: this handoff ID (HANDOFF-109), the spec (SPEC-052),
  the stage (STAGE-010), the project (PROJ-002), and the decisions (DEC-045, DEC-046).
- The tier-B corpus run result (both DNGs still parse) recorded in the handback.

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

- Any change to `MAX_IFDS`, `MAX_IFD_DEPTH`, `MAX_TAG_VALUES`, the `visited` cycle
  guard, or how value payloads are read.
- The pixel/plane validate (SPEC-039 / group E) and any `Error::kind`
  classification code (SPEC-040).
- An absolute `MAX_TOTAL_IFD_ENTRIES` cap (DEC-046 rejected it).
- The Compression-7 shape binding (SPEC-053) and the other review findings — they
  are separate specs.

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
