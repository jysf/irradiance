---
# Maps to ContextCore handoff.* semantic conventions.
#
# VERIFY half of SPEC-054 (group C); the build half is HANDOFF-112.
# Rationale + full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-115
  cycle: verify
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: verifier
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-054

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  agent: claude-opus-5-5
  tokens_total: 7521269            # REAL combined count — `/cost`
  estimated_usd: 5.25
  duration_minutes: null
  branch: claude/spec-054-verify-rebased
  pr: 175
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 verify. Verdict PUNCH LIST — code approved as-is at cd05206; the one punch item was a doc-only HANDOFF-112 restoration (fixed, shipped via the rebased #175). Removing the check from each of the 5 real develop entries in turn broke 6-7 tests each (per-entry coverage confirmed); corpus bit-exact via a hash probe across 8 files; DEVELOP_VERSION unchanged; fuzz clean. The mono f32/threaded entries named in the brief do not exist. tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  synced_at: 2026-11-10
---

# HANDOFF-115: VERIFY required-processing validation + the Sensor API seal (SPEC-054, group C)

## Delegation Summary

You are the **cold verifier** for the group-C build — the most design-heavy of the
four. You did **not** implement it. Review it against SPEC-054 and DEC-049 and return
one verdict. The build is **open PR #170**, branch `claude/spec-054-build-938c45` at
commit `cd05206`. **Review the code on that branch** — your verify worktree is cut
from it.

The build keeps the `OpcodeList2` bytes (new `Sensor.opcode_list_2`), adds a
`pub(crate)` **flags-aware** required-processing scan over the raw opcode framing +
`Error::UnsupportedRequiredOpcode { list, id }`, called at **every** public develop
entry (mono `develop_into` + its `f32` + threaded; colour `develop_color_into` +
`develop_color_f32_into` + threaded), and seals `Sensor` as `#[non_exhaustive]`,
migrating the 7 `tests/` struct-literal fixtures onto a dev-feature-gated in-crate
constructor. **No `DEVELOP_VERSION` bump.**

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-054-required-processing-validation-and-sensor-api-stability.md` — all of it, especially `## Design`, `## Acceptance Criteria`, `## Failing Tests`.
- **Decision:** `./decisions/DEC-049-validate-required-opcode-processing-and-seal-the-sensor-view.md` — the applied-set per path, the raw-scan-not-typed-parse rule, the `Sensor` seal + fixture migration, the corpus-safety probe, the rejected alternatives (do NOT re-litigate: no typed-parse validation, no colour-path opcode application, no deferring the seal).
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-112-build-required-processing-validation-and-sensor-api-stability.md`.
- **Code:** `src/opcode.rs` (the flags-aware scan; note `parse_opcode_list` historically **drops `flags`** for recognized opcodes — the scan must read raw bytes, not that typed parse), `src/develop.rs` + `src/color.rs` (the call at each public entry), `src/ifd.rs` (`Sensor.opcode_list_2`, the `#[non_exhaustive]` seal), `src/lib.rs` (the new error + `Display`), `Cargo.toml` + `tests/` (the dev-feature fixture constructor across `develop_oracle.rs`, `colour_metadata_oracle.rs`, `color_oracle.rs`).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #3, open decision #3) and `./docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` §4.

## The verify contract

Review it cold.

- Acceptance criteria met? Tests pass (`just test` runs `--all-features`; `just lint`;
  the fuzz smoke)? Build reflection honest?
- Decision drift — `just decisions-audit --changed`.
- Constraints (five no-panic deny lints — the scan runs on hostile opcode-list bytes
  with checked arithmetic over the `id/version/flags/size` framing;
  `library-not-application` — the fixture constructor must **not** enter the default /
  published surface); non-trivial choices missing a `DEC-*`.
- **§12 behavioural surface (this is where the defect escapes):**
  - The error must fire from **EACH** public develop entry — mono, mono-`f32`,
    mono-threaded, colour, colour-`f32`, colour-threaded — **not one representative**.
    Confirm a test exercises every door.
  - The scan validates on **raw bytes**, so it catches a mandatory opcode in
    `OpcodeList2`, a mandatory opcode on the colour path, a recognized-but-wrong-list
    mandatory opcode, and a **second** mandatory instance beyond the one the applier
    consumes — with the red-proofs shown **red first**.
  - The dev-feature `Sensor` constructor does **not** leak into the default/published
    surface (check it is feature-gated, like the `fuzzing` precedent).
  - The whole tier-B corpus still develops **bit-exact** and `DEVELOP_VERSION` is
    unchanged (DEC-049's corpus probe says it must).
- Corpus bit-exact (local tier-B run).

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## How to hand back

Report your verdict and real token usage (`/cost`) in your final message. A punch-list
fix goes on `claude/spec-054-verify-<suffix>` as a PR against `main` referencing
HANDOFF-115, SPEC-054, DEC-049, PR #170; a clean build needs only the verdict + cost.

## Out of scope

- Honouring any new opcode, or applying opcodes on the colour path.
- Sealing `Cfa` / `WarpRect` / `Opcode` (that sweep is SPEC-042, group F).
- The other wave groups (A/SPEC-052, B/SPEC-053, D/SPEC-055) and group E (SPEC-039).
