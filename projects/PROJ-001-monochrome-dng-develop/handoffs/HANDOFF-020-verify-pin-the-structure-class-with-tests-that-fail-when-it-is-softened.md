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
  id: HANDOFF-020
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: null    # dispatch hint only (SPEC-007/FU-6); set to what ACTUALLY ran
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-21
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-008

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

# HANDOFF-020: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-008` for the **verify** cycle. Independent
session.

This spec exists because SPEC-007's verify proved a boundary was guarded at **one
point in five**. Its whole value is that the guard is now real — so the scrutiny
that matters is whether the new tests fail for the **right reasons**, not whether
they fail.

## Context the Receiving Agent Needs

### Already reconciled — and this time across the whole class, not one point

Ten gates green, 66 tests, `main` untouched, one commit ahead, tree clean.

**All four structural mutants mutation-tested by the orchestrator**, each asserted
to compile *and* apply before concluding:

| structural tag → tolerant | before SPEC-008 | now |
|---|---|---|
| `Compression` | 0 failures | **1 — killed** |
| `StripOffsets` | 0 failures | **1 — killed** |
| `StripByteCounts` | 0 failures | **1 — killed** |
| `BitsPerSample` | 0 failures | **1 — killed** |

`RowsPerStrip` was already covered. The gap SPEC-007's verify found is closed.

Also verified in code: `is_structural_tag()` (`src/ifd.rs:188`) is a real per-tag
list gating `TYPE_RATIONAL` at `:841`, and `sensor()`'s `Orientation` now records
at most once and **only when no valid value was found anywhere** — which is FU-1
and FU-2 together.

### What deserves scrutiny — the tests, not the mutants

1. **Do the new tests fail for the RIGHT reason?** Each kills its mutant, but a
   test can be red for an unrelated cause. Check the assertion actually names the
   tag and error it claims to (`UnexpectedFieldType { tag, field_type }`), the way
   the `RowsPerStrip` original does.
2. **Is `is_structural_tag()`'s list right?** Compare it tag-for-tag against
   `DEC-012`'s amended Structure row. A tag missing from the list silently regains
   the global RATIONAL looseness; a tag wrongly added rejects a legal encoding.
   ⚠ Confirm `SubIFDs` (330) is in it — FU-4's whole point.
3. **The `Orientation` logic has four combinations** (`ifd0` ok/err ×
   `sensor` ok/err). The fix reads correctly for the two the fixtures cover.
   Are the other two right — in particular a *good* `ifd0` value with an
   *erroring* sensor read?
4. **FU-5 — is the division actually pinned?** The complaint was that every
   well-formed RATIONAL fixture used denominator `1`, so a mutant pushing the
   numerator and ignoring the quotient survived. Verify a denominator ≠ 1 fixture
   now exists **and** kills that mutant.
5. **Two "bonus" mutants** were claimed beyond the required four. What were they,
   and do they cover anything the four do not?
6. **`docs/provenance-ledger.md` extended in place, "no new DEC needed."** Agree?

### One process note worth recording

`HANDOFF-019` was the first handoff written with `to_agent: null` per
`SPEC-007/FU-6`, and the build filled it with what **actually ran**
(`claude-sonnet-5`). The fix worked on first use — no prediction, no mismatch.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

You do **not** need to re-run the four mutants — the orchestrator did, and pasted
the numbers. Spend the cycle on whether the tests are *honest*: right reason, right
assertion, complete class.

⚠ Traps that have each produced wrong answers here: zero-match `cargo test <name>`
**exits 0**; **sum across targets**; and **assert a mutation compiled and applied**
before concluding (five failures on this project).

Label findings `SB-N` / `FU-N` for **this** spec. If APPROVED, set
`task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Redrawing `DEC-012`'s Structure/Interpretation line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates and whatever you re-ran.
2. Fill `## Completion` and `handback:`; set `to_agent` to what **actually ran**;
   `tokens_total` deduplicated by `message.id`, captured **before** the session
   closes.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship`).
4. Commit on `feat/spec-008-pin-structure-class`; do not merge. Do not run
   `handback-sync`.

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
