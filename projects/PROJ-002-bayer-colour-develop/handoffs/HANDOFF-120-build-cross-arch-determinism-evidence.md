---
# Maps to ContextCore handoff.* semantic conventions.
# BUILD half of SPEC-041. Rationale + handback contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-120
  cycle: build
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-041

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

# HANDOFF-120: Build the cross-arch determinism evidence (SPEC-041)

## Delegation Summary

Commit golden hashes of `unpack_into`, `demosaic_into` and `to_linear_f32_into` over
tier-A synthetic Bayer fixtures, assert them on a **linux x86_64 + macOS arm64** CI
matrix (the cross-arch agreement is the evidence), name the float paths that could
diverge, and show the oracle red. No new DEC (the determinism rests on DEC-016 / DEC-018
/ DEC-033); no kernel change — if an output diverges between arches, STOP and raise a new
spec, do not force agreement.

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-041-cross-arch-determinism-evidence-tier-a-synthetic-bayer.md` — Acceptance Criteria, Failing Tests, Outputs.
- **Decisions:** `DEC-018` (intent casts, no silent truncation), `DEC-033` (committed-constant `f64`, basic ops only — no FMA/transcendental), `DEC-003` (tier-B never in CI), `DEC-009` (control before red-proof).
- **Code:** `src/plane.rs` (`unpack_into`), `src/demosaic.rs` (`demosaic_into`), the module holding `to_linear_f32_into` (tone); `tests/support/tiff.rs` (the SPEC-027 tier-A CFA builders — build fixtures, don't use held files); `.github/workflows/ci.yml` (mirror an existing job for the matrix).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

## Expected Deliverables

- `tests/determinism.rs` — builds tier-A synthetic Bayer fixtures (≥1 RGGB + ≥1 other phase; bit depths where output differs), hashes each of the three outputs with a small in-test FNV-1a over the raw bytes (`u16`/`f32` as LE bytes) and asserts a committed `const` hash. **No new dependency** — the hash is a few lines.
- `.github/workflows/ci.yml` — a `determinism` job with `strategy.matrix.os: [ubuntu-latest, macos-latest]` running `cargo test --test determinism`.
- The **oracle shown red** (DEC-009 control first): a perturbed sample or a locally-mutated kernel yields a different hash and fails the assert, shown red before green.
- The **float paths named** in a doc (`docs/determinism.md` or a section): `to_linear_f32_into` = one IEEE `f32` division (correctly-rounded, identical both arches); the colour path (`src/color.rs`, `f64`, STAGE-008) named as an **unmeasured** path (DEC-033 basic-ops, out of scope); `unpack_into`/`demosaic_into` are integer — no float.
- CHANGELOG `[Unreleased] → Added`; a `docs/provenance-ledger.md` row.
- `just test` (`--all-features`) + `just lint` green. A PR against `main` from `claude/spec-041-build-<suffix>` referencing HANDOFF-120, SPEC-041, STAGE-010, PROJ-002.

### If `cycle: verify` — the verify contract

Review cold. Confirm the matrix job really runs on **both** `ubuntu-latest` and
`macos-latest` (not one with the other skipped), the committed hashes are asserted on
both, the red-proof bites (a perturbed run fails — the §12 behavioural check), the float
paths are named honestly, and no kernel was changed to force agreement. `just
decisions-audit --changed`. Output exactly one of ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- No tier-B corpus file in CI (DEC-003) — corpus hashes stay a local record.
- No kernel change; a found divergence becomes its own spec.
- No extra OS/arch beyond the two; no colour-path (`f64`) measurement — only name it.
- `Error::kind` (SPEC-040); the release (SPEC-042).

## Return Criteria

Fill the `handback:` front-matter (a real `tokens_total`) and the `## Handback` section;
set `handoff.status`/`handback.status` → `completed`; open the PR. Report token usage
even if blocked.

## Handback

### Execution notes
- **Branch / PR:** [link]  **Completed at:** YYYY-MM-DD  **All criteria met?** yes/no
- **Cross-arch:** did the committed hashes hold on BOTH ubuntu-latest and macos-latest? (any divergence, and what you did)
- **For verify:** the verdict + commit SHA.

### Cost self-report
- **Tokens (total):** <real number>  **Estimated USD:** <n>  **Duration (min):** <n>  **Source:** `/cost` | transcript | harness

### Drift and new artifacts
- **New decisions:** <DEC-NNN or none>  **Deviations:** <list>  **Follow-ups:** <list>

### Reflection
1. What was unclear? — <answer>
2. A constraint/decision that should have been listed? — <answer>
3. What would you do differently? — <answer>
