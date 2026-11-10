---
# Maps to ContextCore handoff.* semantic conventions.
#
# VERIFY half of SPEC-053 (group B); the build half is HANDOFF-110.
# Rationale + full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-114
  cycle: verify
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: verifier
  created_at: 2026-11-10
  status: pending

task:
  spec_id: SPEC-053

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  agent: claude-opus-5-5
  tokens_total: 3831612            # REAL combined count — `/cost`
  estimated_usd: 2.79
  duration_minutes: null
  branch: claude/spec-053-verify-f53864
  pr: null
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 verify. Verdict APPROVED at 90f4898, no code change needed. Tests, lint, both red-proofs and the bit-exact corpus check all pass; DEVELOP_VERSION unchanged; fuzz clean (ljpeg 3.35M, plane 3.10M). tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  synced_at: 2026-11-10
---

# HANDOFF-114: VERIFY the Compression-7 decoded-frame shape binding (SPEC-053, group B)

## Delegation Summary

You are the **cold verifier** for the group-B build. You did **not** implement it.
Review it against SPEC-053 and DEC-047 and return one verdict. The build is **open PR
#169**, branch `claude/spec-053-build-570012` at commit `90f4898`.
**Review the code on that branch** — your verify worktree is cut from it.

The build binds the decoded SOF-3 `Frame`'s effective shape to the strip it fills:
`samples_per_line × components == sensor.width` and `lines == rows_in_strip`, checked
in `unpack_jpeg` via a `parse_headers` pre-decode, plus a reject of `Hi/Vi != 0x11` in
`parse_frame_header`. Two new `Ljpeg*` errors; **no `DEVELOP_VERSION` bump** (it only
turns a former silent-`Ok` wrong-shape case into `Err`).

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-053-compression-seven-decoded-frame-shape-binding.md` — all of it.
- **Decision:** `./decisions/DEC-047-bind-the-decoded-sof3-frame-shape-to-the-strip.md` — the shape rule, the no-bump reasoning, the rejected alternatives.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-110-build-compression-seven-decoded-frame-shape-binding.md`.
- **Code:** `src/ljpeg.rs` (`unpack_jpeg`, `parse_headers`, `parse_frame_header`) and `src/lib.rs` (the two new `Ljpeg*` errors + `Display`).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #2).

## The verify contract

Review it cold.

- Acceptance criteria met? Tests pass (`just test`, `just lint`, the fuzz smoke)?
  Build reflection honest?
- Decision drift — `just decisions-audit --changed`.
- Constraints (five no-panic deny lints; checked arithmetic over the shape math);
  non-trivial choices missing a `DEC-*`.
- **§12 behavioural surface:** confirm the rejection **actually fires on the
  equal-area wrong-shape case** that previously returned `Ok(())` (a red-proof shown
  red first), and the `Hi/Vi != 0x11` reject fires — not just that a dimension field
  was read. **And** confirm **both** held Compression-7 corpus files (the Pentax
  K3III two-component SOF-3) still decode **bit-exact** and `DEVELOP_VERSION` is
  unchanged.
- Corpus bit-exact (local tier-B run).

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## How to hand back

Report your verdict and real token usage (`/cost`) in your final message. A punch-list
fix goes on `claude/spec-053-verify-<suffix>` as a PR against `main` referencing
HANDOFF-114, SPEC-053, DEC-047, PR #169; a clean build needs only the verdict + cost.

## Out of scope

- Re-implementing the binding or adding new SOF-3 shape support.
- The other wave groups (A/SPEC-052, C/SPEC-054, D/SPEC-055) and group E (SPEC-039).
