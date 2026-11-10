---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. This is the VERIFY half of SPEC-052 (group A);
# the build half is HANDOFF-109. `handoff.cycle` distinguishes them.
#
# The `handback:` block is the RETURN path for the verify cost.
# Rationale + full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-113
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.verify — the executing agent
  from_role: architect
  to_role: verifier
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
# Filled in by the EXECUTING AGENT before it reports its verdict. Report the REAL
# token number from your own interface (`/cost` in Claude Code). If your platform
# exposes none, set tokens_total: null AND say why in notes. Do not invent one.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null
  notes: null
  synced_at: null
---

# HANDOFF-113: VERIFY the parser resource guard (SPEC-052, group A)

## Delegation Summary

You are the **cold verifier** for the group-A build. You did **not** implement it.
Review it against SPEC-052 and DEC-046 and return one verdict. The build is **already
merged to `main`** (PR #167); **review the code at `main`**. Your verify worktree is
cut from `main`.

The build adds an aggregate IFD entry-table byte budget: as the IFD walk proceeds,
`read_ifd` keeps a running total of `entry_count × 12` bytes across the whole walk and
rejects — with a new `Error::IfdTableBytesExceedFile` — before any
`Vec::with_capacity`, when that total would exceed the file length, so a ~770 KB file
cannot drive a ~72 MiB parse.

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-052-parser-resource-guard-aggregate-entry-byte-budget.md` — all of it, especially `## Acceptance Criteria` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-046-bound-aggregate-ifd-entry-table-bytes-by-file-length.md` — the budget rule and the rejected alternatives (do NOT re-litigate them).
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-109-build-parser-resource-guard-aggregate-entry-byte-budget.md` — the build contract you are checking against.
- **Code:** `src/ifd.rs` (`read_ifd`, the running byte total, the new error site) and `src/lib.rs` (`Error::IfdTableBytesExceedFile` + its `Display`).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #1).

## The verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass (`just test`, `just lint`, the fuzz
  smoke the spec names)? Build reflection answered honestly (*"nothing was unclear"*
  is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations (the five no-panic deny lints; checked arithmetic on the
  running total — it must not itself overflow); non-trivial choices missing a `DEC-*`.
- **§12 behavioural surface (the defect class that escapes):** confirm the guard
  actually **fires before the allocation** on a forged-dimension / aliased-entry-table
  input — not merely that a size field was validated in isolation — **and** that a
  legitimate corpus file (and the tier-A fixtures) still parse `Ok`. A test that only
  checks the arithmetic without exercising the reject-before-allocate path is the gap.
- Corpus bit-exact where it applies (local tier-B run).

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## How to hand back

Report your verdict and your **real token usage** (`/cost`) in your final message —
the orchestrator transcribes the verify cost at ship from your report. If you open a
punch-list fix branch, open a PR against `main` from `claude/spec-052-verify-<suffix>`
and reference HANDOFF-113, SPEC-052, DEC-046, PR #167. If the build is clean, a verdict
with the cost figure is enough — no PR required.

## Out of scope

- Re-implementing the guard, or widening it beyond the aggregate byte budget.
- The other wave groups (B/SPEC-053, C/SPEC-054, D/SPEC-055) and group E (SPEC-039).
