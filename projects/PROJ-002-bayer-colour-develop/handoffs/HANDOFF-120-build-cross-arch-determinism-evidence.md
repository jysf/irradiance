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
  status: completed

task:
  spec_id: SPEC-041

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  tokens_total: 7213996               # REAL combined count — what cost-audit reads
  estimated_usd: 5.16
  duration_minutes: 12
  branch: claude/spec-041-build-29eaff
  pr: https://github.com/jysf/irradiance/pull/181
  completed_at: 2026-11-10
  notes: "claude-opus-5-5, transcript-summed (bd02789d). Committed hashes HELD on ubuntu-latest x86_64 and macos-latest arm64, dev and release profiles (CI run 42947290870). No kernel change, no new dep."
  synced_at: 2026-11-10
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
- **Branch / PR:** `claude/spec-041-build-29eaff` → https://github.com/jysf/irradiance/pull/181 (commit `f6d36d0`, plus this handback commit).  **Completed at:** 2026-11-10  **All criteria met?** yes
- **Deliverables:** `tests/determinism.rs` (4 tier-A synthetic Bayer fixtures — RGGB 16-bit LE, BGGR 14-bit BE, GRBG 12-bit LE, GBRG 8-bit BE, 24×16 so demosaic runs both interior fast path and reflecting border; in-test 64-bit FNV-1a; 12 committed constants; plus FNV reference vectors, unpack round-trip vs. generated samples, fixtures-are-distinct, `determinism_red_proof`), CI `determinism` matrix job, `docs/determinism.md`, CHANGELOG `[Unreleased] → Added`, provenance-ledger row.
- **Cross-arch:** **the committed hashes HELD on both arches.** CI run 42947290870: `rust / determinism (ubuntu-latest, x86_64)` — `uname -m: x86_64`, host `x86_64-unknown-linux-gnu`, dev 7/7 + release 7/7; `rust / determinism (macos-latest, arm64)` — `uname -m: arm64`, host `aarch64-apple-darwin`, dev 7/7 + release 7/7. Constants were recorded locally on macOS arm64. No divergence, so no new spec raised and no kernel touched.
- **Oracle shown red (DEC-009, control first):** (1) placeholder constants → the three hash tests + red_proof red, actuals printed; then committed → green. (2) `determinism_red_proof` runs the control (unperturbed fixture == committed) then flips one interior sample's LSB → all three hashes move, per fixture. (3) Local kernel mutants, each reverted with `git checkout` (src diff empty after; post-revert 7/7): `tone.rs` divisor 65535→65534 → `to_linear_f32_into_hash` + red_proof red, unpack/demosaic green; 16-bit LE unpack `^ 1` → unpack/demosaic/linear/round-trip/red_proof red; `demosaic.rs` `rnd4` half-up→half-down → demosaic + linear + red_proof red, unpack green. Each mutant reddened exactly the tests downstream of it.
- **Gates:** `just test` 514 passed / 0 failed (507 + 7 new); `just lint` green; `cargo test --release --test determinism` green locally.
- **For verify:** check the two `rust / determinism (…)` legs on #181 both ran (each pins `uname -m`), the red-proof bites, `git diff main -- src` is empty.

### Cost self-report
- **Tokens (total):** 7213996  **Estimated USD:** 5.16  **Duration (min):** ~12  **Source:** transcript (`bd02789d-dc19-4b64-be54-cce38b7c4c9b.jsonl`, deduped by message id; input + cache-write + cache-read + output; Opus 5.5 rates $5/$25 per M, cache write $6.25, read $0.50). Measured just before the handback commit; the final commit/report turns are not included (well under 1M).

### Drift and new artifacts
- **New decisions:** none  **Deviations:** (a) added a release-profile run to each matrix leg and an explicit `uname -m` arch assertion per leg — same two OS/arches, just stronger evidence; (b) `docs/determinism.md` also names `warp.rs` (`f64` bilinear `WarpRectilinear`) as unmeasured, which the spec did not list but is a real float path; (c) two extra tests beyond the four named (FNV reference vectors, unpack round-trip) so the hashes pin a *correct* plane, not merely a stable one.  **Follow-ups:** a future spec could hash the `f64` colour path (`develop_color_into`/`develop_color_f32_into`) and a Compression-7 fixture on the same matrix — both currently named unmeasured.

### Reflection
1. What was unclear? — Nothing material; the spec's "8/12/14/16-bit where the output differs" read as "cover all four"; I did, one per phase.
2. A constraint/decision that should have been listed? — DEC-008 (the two unpack paths) decides which fixtures exercise which code; worth citing alongside DEC-018. DEC-044 (threaded output byte-identical) is relevant to naming `_threaded` variants.
3. What would you do differently? — Nothing significant; recording constants from a placeholder-red run first made the DEC-009 sequence fall out naturally.
