---
# Maps to ContextCore handoff.* semantic conventions.
# VERIFY half of SPEC-041 (build half: HANDOFF-120). Authored at ship from the cold
# reviewer's report (the verify ran self-contained from SPEC-041 + the handoff on-branch).
# Rationale + handback contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-122
  cycle: verify
  from_agent: claude-opus-4-8
  to_agent: claude-opus-5
  from_role: architect
  to_role: verifier
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
  agent: claude-opus-5-5
  tokens_total: 2311488
  estimated_usd: 1.83
  duration_minutes: null
  branch: claude/spec-041-verify-2e0a31
  pr: null
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 verify. Verdict APPROVED at f307abe (PR #181, superseded by the rebased #183), no code change. Tests 514/0, lint green, both CI determinism legs ran on their real arches (ubuntu x86_64 + macos arm64), the red-proof bites (a one-bit sample flip moves all three hashes), and the hashes reproduced locally. tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  synced_at: 2026-11-10
---

# HANDOFF-122: VERIFY the cross-arch determinism evidence (SPEC-041)

## Verdict

**✅ APPROVED at `f307abe`** (PR #181, shipped via the rebased #183). No punch list; no
code change.

Confirmed cold: the CI `determinism` job ran and passed on **both** `ubuntu-latest`
(x86_64) and `macos-latest` (arm64), dev + release; the committed FNV-1a hashes of
`unpack_into` / `demosaic_into` / `to_linear_f32_into` are asserted on both arches over
tier-A synthetic Bayer fixtures; the oracle is shown red (a one-bit sample flip moves all
three hashes, control first per DEC-009); the float paths are named honestly in
`docs/determinism.md`; no kernel was changed. `just test` 514 pass, `just lint` green,
no new dependency, `decisions-audit` advisory-only.

## Handback

### Cost self-report
- **Tokens (total):** 2,311,488  **Estimated USD:** $1.83  **Source:** session transcript (deduped by message id; `/cost` not callable in a delegated session)

### Drift and new artifacts
- **New decisions:** none.  **Deviations:** none.  **Follow-ups:** none.
