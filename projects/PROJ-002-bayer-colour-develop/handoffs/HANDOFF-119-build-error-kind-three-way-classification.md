---
# Maps to ContextCore handoff.* semantic conventions.
# BUILD half of SPEC-040. Rationale + handback contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-119
  cycle: build
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-040

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: null
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null
  notes: null
  synced_at: null
---

# HANDOFF-119: Build `Error::kind()` / `ErrorKind` (SPEC-040)

## Delegation Summary

Add a stable three-way `Error::kind(&self) -> ErrorKind { Unsupported, Corrupt,
CallerError }` (`#[non_exhaustive]`) so a consumer can split "unsupported format, skip
it" from "corrupt/hostile file" from "my own API misuse". The classification rubric and
the **full per-variant table** are **DEC-051** — read it first; this is pure build
against it. No new `Error` variant, no `Display` change, no change to how any error is
produced.

## What to read first

- **Decision:** `./decisions/DEC-051-error-kind-three-way-classification-and-the-wildcard-free-match.md` — the rubric, the full table, the wildcard-free-match decision, the four flagged judgment calls, the rejected alternatives (do NOT add `is_unsupported`, a 4th kind, or a `_ =>` arm).
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-040-error-kind-errorkind-needs-a-dec-first.md` — Acceptance Criteria, Failing Tests, Outputs.
- **Code:** `src/lib.rs` — the `Error` enum (`#[non_exhaustive]`, ~55 variants at 2026-11-10) and its `Display`; add `ErrorKind` + `impl Error { pub fn kind }` here.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

## Expected Deliverables

- `#[non_exhaustive] pub enum ErrorKind { Unsupported, Corrupt, CallerError }` (derive `Debug, Clone, Copy, PartialEq, Eq`).
- `impl Error { pub fn kind(&self) -> ErrorKind { match self { … } } }` with **no `_ =>` arm** — the compiler-exhaustive match IS the "a new variant cannot land unclassified" guarantee.
- `tests/error_kind.rs` — a representative of **every** variant asserted to DEC-051's kind; plus the three-buckets-distinct and SPEC-039-additions checks.
- Re-derive the table against the live enum (it may have drifted); classify any new variant by the rubric and flag it in the handback. Confirm locally that adding a throwaway dummy variant makes `kind()` fail to compile (then remove the dummy); report that in the handback.
- CHANGELOG `[Unreleased] → Added`; a `docs/provenance-ledger.md` row.
- `just test` (`--all-features`) + `just lint` green. A PR against `main` from `claude/spec-040-build-<suffix>` referencing HANDOFF-119, SPEC-040, STAGE-010, PROJ-002, DEC-051.

### If `cycle: verify` — the verify contract

Review cold. Confirm: `kind()` has no wildcard arm (adding a dummy variant fails to
compile — the §12 behavioural check here), every variant matches DEC-051's table, the
four judgment calls (`PlaneTooLarge`→CallerError, `NotTiff`/`NoSensorIfd`→Unsupported,
`MissingColorCalibration`→Unsupported, the Invalid*→Corrupt) are as decided, `Display`
and error production are untouched. `just decisions-audit --changed`. Output exactly one
of ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Any new `Error` variant, `Display` change, or change to how an error is produced.
- `is_unsupported`, a 4th kind, a `_ =>` arm (all DEC-051-rejected).
- Determinism (SPEC-041); the release (SPEC-042).

## Return Criteria

Fill the `handback:` front-matter (a real `tokens_total`) and the `## Handback` section;
set `handoff.status`/`handback.status` → `completed`; open the PR. Report token usage
even if blocked.

## Handback

### Execution notes
- **Branch / PR:** [link]  **Completed at:** YYYY-MM-DD  **All criteria met?** yes/no
- **Compile-time exhaustiveness confirmed?** (throwaway dummy variant — did `kind()` fail to compile?)
- **For verify:** the verdict + commit SHA.

### Cost self-report
- **Tokens (total):** <real number>  **Estimated USD:** <n>  **Duration (min):** <n>  **Source:** `/cost` | transcript | harness

### Drift and new artifacts
- **New decisions:** <DEC-NNN or none>  **Deviations:** <list>  **Follow-ups:** <list>

### Reflection
1. What was unclear? — <answer>
2. A constraint/decision that should have been listed? — <answer>
3. What would you do differently? — <answer>
