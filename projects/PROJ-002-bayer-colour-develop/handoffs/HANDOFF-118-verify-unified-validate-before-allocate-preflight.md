---
# Maps to ContextCore handoff.* semantic conventions.
#
# VERIFY half of SPEC-039 (group E); the build half is HANDOFF-117.
# Authored at ship from the cold reviewer's report (the verify ran self-contained
# from SPEC-039 + DEC-050 + HANDOFF-117 on-branch; this file records the cycle and
# its cost so `just handback-sync` can transcribe it).
# Rationale + full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-118
  cycle: verify
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: verifier
  created_at: 2026-11-10
  status: completed

task:
  spec_id: SPEC-039

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handback:
  status: completed
  agent: claude-opus-5-5
  tokens_total: 4344310
  estimated_usd: 3.20
  duration_minutes: null
  branch: claude/spec-039-verify-fa8a16
  pr: null
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 verify. Verdict APPROVED at 2f256f0 (PR #178), no code change needed. 507 tests pass + corpus 9/9, lint, CI all green incl the new ifd/plane/ljpeg fuzz smoke jobs, plane fuzz 9,940,571 runs clean; an independent mutation confirmed validate and unpack_into share one check (no drift). tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  synced_at: 2026-11-10
---

# HANDOFF-118: VERIFY the unified validate-before-allocate preflight (SPEC-039, group E)

## Delegation Summary

The cold verifier reviewed the SPEC-039 build (PR #178, commit `2f256f0`) against
SPEC-039 and DEC-050 — the `plane::validate` preflight over both the uncompressed and
the Compression-7 path with the caller-selected pixel budget checked first, the
`unpack_into` refactor onto shared helpers, the EOI-optional ratification, the new
`ifd`/`plane`/`ljpeg` fuzz smoke jobs and the docs reconcile. The verify ran
self-contained from the spec, DEC-050 and HANDOFF-117 on the build branch; this file is
the orchestrator's record of the cycle and its cost.

## Verdict

**✅ APPROVED at `2f256f0`** (PR #178). No punch list; no code change.

What the reviewer confirmed cold:
- The caller budget (`Error::PlaneTooLarge`) is checked first on **both** paths; a
  forged huge dimension over a tiny file is rejected with no allocation sized by the claim.
- `validate` and `unpack_into` genuinely share one check — an independent mutation proved
  they move together (no drift); error precedence preserved (`PlaneBufferWrongLength` wins).
- The uncompressed bound holds under aliased strips (`StripBytesExceedFile`); Compression-7
  validates structurally while the decoded-frame shape binding still fires at decode.
- EOI-optional holds (a valid scan minus the trailing EOI decodes `Ok`; a truncated scan is
  `Truncated`).
- `just test` 507 pass + corpus 9/9, `just lint`, `decisions-audit`, the docs check, the
  `plane` fuzz (9,940,571 runs clean) and all of PR #178's CI (incl. the three new smoke
  jobs) green. The three red-proofs were shown red first.

## Handback

### Execution notes

- **Branch:** `claude/spec-039-verify-fa8a16` (no PR — build was clean)
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes
- **Verdict:** ✅ APPROVED at `2f256f0`

### Cost self-report

- **Tokens (total):** 4,344,310
- **Estimated USD:** $3.20
- **Duration (minutes):** not reported
- **Source of the number:** session transcript (deduped by message id; `/cost` not callable in a delegated session)

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none.
- **Follow-up work identified:** none beyond DEC-050's already-deferred group-C items.
