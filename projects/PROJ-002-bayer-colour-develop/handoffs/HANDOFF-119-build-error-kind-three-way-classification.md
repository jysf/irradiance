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
  status: completed

task:
  spec_id: SPEC-040

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  tokens_total: 5756260            # REAL combined count — what cost-audit reads
  estimated_usd: 4.20
  duration_minutes: 25
  branch: claude/spec-040-build-ed86ac
  pr: 182
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input 82 + cache write 122,174 + cache read 5,608,811 + output 25,193, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns. synced_at left null for handback-sync."
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
- **Branch / PR:** `claude/spec-040-build-ed86ac` → https://github.com/jysf/irradiance/pull/182  **Completed at:** 2026-11-10  **All criteria met?** yes
- **Table re-derived against the live enum:** the `Error` variant set parsed from `src/lib.rs` at f30c1ef is **identical** to DEC-051's table — 55 variants, 12 `CallerError` / 18 `Unsupported` / 25 `Corrupt`. No drift, no new variant, nothing classified outside the table.
- **Compile-time exhaustiveness confirmed?** Yes. Adding a throwaway `DummyUnclassified` variant to `Error` produced `error[E0004]: non-exhaustive patterns: &Error::DummyUnclassified not covered` in `kind()` (and in `Display`); the dummy was removed and the tree restored before commit.
- **The test bites (own mutation checks, all reverted):** moving `PlaneTooLarge` into the `Corrupt` arm turned `every_variant_classified` and `spec_039_additions_classified` red ("PlaneTooLarge: got Corrupt, DEC-051 says CallerError"); a dummy variant classified in `kind()` but absent from the test table failed with `Error variants missing from the DEC-051 table: ["DummyUnclassified"]`.
- **How the test catches a missing variant:** an integration test cannot match `Error` exhaustively (it is `#[non_exhaustive]` downstream), so `every_variant_classified` parses the variant names out of `src/lib.rs` (`include_str!`) and requires the representative table to cover exactly that set, plus DEC-051's 55 / 12 / 18 / 25 counts.
- **Gates:** `just test` — 23 binaries, **512 passed, 0 failed** (507 before + 5 new), corpus present. `just lint` and `just lint-no-allow` green. `cargo fmt --check` clean.
- **Untouched:** no `Error` variant, `Display` text or error-production site changed; `kind()` is total and infallible (no panic path).

### Cost self-report
- **Tokens (total):** 5,756,260  **Estimated USD:** 4.20  **Duration (min):** ~25  **Source:** transcript (this session's usage records, deduped by message id; claude-opus-5-5)

### Drift and new artifacts
- **New decisions:** none  **Deviations:** none from DEC-051. Two additive tests beyond the three named in the spec: `dec_051_judgment_calls` (pins `NotTiff`/`NoSensorIfd`/`MissingColorCalibration` → Unsupported vs. `NoSensorIfdCandidatesMalformed`/`MissingTag` → Corrupt) and `error_kind_is_copy_and_comparable`.
- **Follow-ups:** none required. Note for maintainers: adding an `Error` variant now needs three edits in step — the `kind()` arm (compiler-enforced), the `tests/error_kind.rs` table (test-enforced, by name), and DEC-051's table / counts.

### Reflection
1. What was unclear? — Nothing material; DEC-051's table was complete and exact against the live enum.
2. A constraint/decision that should have been listed? — No. The spec's "a variant missing from the test is a gap the test itself surfaces" needed a mechanism an integration test can't get from exhaustiveness; parsing the enum's variant names from source was the simplest one.
3. What would you do differently? — Nothing; most wall time was the full `just test` red-proof rebuilds.
