---
# Maps to ContextCore handoff.* semantic conventions.
#
# VERIFY half of SPEC-055 (group D); the build half is HANDOFF-111.
# Rationale + full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-116
  cycle: verify
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: verifier
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-055

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: null
  tokens_total: null               # REAL combined count — `/cost`
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null
  notes: null
  synced_at: null
---

# HANDOFF-116: VERIFY the licence gate over the dev-graph + the irr binary gate (SPEC-055, group D)

## Delegation Summary

You are the **cold verifier** for the group-D build. You did **not** implement it.
Review it against SPEC-055 and DEC-048 and return one verdict. The build is **open PR
#168**, branch `claude/spec-055-build-37bb23` at commit `0e975ae`. **Review the code on
that branch** — your verify worktree is cut from it.

The build makes `cargo-deny` evaluate the **dev-dependency graph** (root + `fuzz/`), so
a copyleft dev-dependency is caught (DEC-048 records that `exclude-dev=false` and an
empty `[workspace]` both failed in probe — the shipped knob is whatever actually makes
the mutation fail), and gates the `irr` binary behind an empty `irr` cargo feature
(`required-features = ["irr"]`), with a package-target-list check.

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-055-license-gate-dev-graph-and-package-surface.md` — all of it, especially the mutation red-proof and the package-target-list check.
- **Decision:** `./decisions/DEC-048-license-gate-over-the-dev-graph-and-gate-the-irr-binary.md` — the `irr` disposition, the deny-knob finding, the rejected alternatives.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-111-build-license-gate-dev-graph-and-package-surface.md`.
- **Code / config:** `deny.toml` (the dev-graph evaluation knob), `Cargo.toml` (the `irr` feature + `required-features`, the package target list), the CI job that runs `cargo-deny`, and any gate script the spec names. `tests/` (the `CARGO_BIN_EXE_irr` tests that drove the required-features gate).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (findings #4 and #6).

## The verify contract

Review it cold.

- Acceptance criteria met? The gates actually run (`just lint` / the deny gate /
  `just test`)? Build reflection honest?
- Decision drift — `just decisions-audit --changed`.
- Constraints (`no-copyleft-dependencies`; the `irr` binary must stay out of the
  default/published surface); non-trivial choices missing a `DEC-*`.
- **§12 behavioural surface:** confirm the **mutation red-proof is real** — adding a
  GPL dev-dependency makes `cargo-deny` **fail** (shown red first), not merely that a
  config line changed — **and** that the `irr` binary still builds and its tests run
  under `--features irr` (the empty feature must actually gate `CARGO_BIN_EXE_irr`),
  **and** the package-target-list check runs and excludes `irr` from the published
  package. A gate that is present but toothless (passes with the copyleft dep added) is
  the defect.
- Both cargo graphs (root + `fuzz/`, DEC-011) are covered.

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## How to hand back

Report your verdict and real token usage (`/cost`) in your final message. A punch-list
fix goes on `claude/spec-055-verify-<suffix>` as a PR against `main` referencing
HANDOFF-116, SPEC-055, DEC-048, PR #168; a clean build needs only the verdict + cost.

## Out of scope

- Changing the `irr` binary's behaviour or adding real `irr` subcommands.
- The full public-struct non-exhaustive sweep / package work of SPEC-042 (group F).
- The other wave groups (A/SPEC-052, B/SPEC-053, C/SPEC-054) and group E (SPEC-039).
