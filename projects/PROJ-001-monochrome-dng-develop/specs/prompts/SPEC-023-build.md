# SPEC-023 — build dispatch

Build cycle for **SPEC-023: audit every gate script for the mute-death shape**
(STAGE-005, PROJ-001). Delegated via **HANDOFF-062**.

Read first, in order:
1. This prompt.
2. `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-062-build-audit-every-gate-script-for-the-mute-death-shape.md`.
3. The spec (SPEC-023) — Inputs (the offender map), AC1–AC5, Notes.
4. `guidance/toolchain-brief.md`.

Add a check *over* the gate scripts for the mute-death shape (a gate that reports
green, or dies, without its check running) + fix the offenders. Four traps:

1. **The audit is itself source-matching tooling** (AGENTS §16 rules 2–3): it
   MUST exclude comments (`validate.sh:296–299` is PROSE describing the shape — a
   naive grep fires on it) and assert its own scan count (a glob that matched
   nothing is a loud failure). `gate-audit` that silently matches nothing is the
   defect it hunts.
2. **No code fixes without a proof.** The red-proof (`gate-audit-red-proof.sh`,
   mirror `validate-red-proof.sh`) must catch an injected mute shape BY NAME for
   the right reason, with a clean control + isolation. Pair it with the gate as
   adjacent CI jobs (`ci.yml:24–33`).
3. **Three reachable offenders to fix** (then the audit goes green): `validate.sh`
   (rc-capture + `parsed == ${#parse_targets[@]}` assert + broaden the ruby
   `rescue` — a non-SyntaxError fault goes RED, and `validate-red-proof.sh` must
   still pass); `handback-sync.sh` (skip `handoff.status == superseded`, then
   **de-kludge HANDOFF-038**: restore its `handback` to null + drop the manual
   `synced_at` and its note); `decisions-audit.sh:~217` (guard the `grep`). Fix
   any lower-risk variant (`cost-audit.sh`, `_lib.sh`, `test.sh`) the audit flags.
4. **0 `src/` change.** shellcheck is NOT the engine (SC2143/2181/2015 miss this
   class); a complementary shellcheck job is optional — defer if it churns.

Fill the handback with a REAL transcript-sum `tokens_total` (deduped by
message.id via your scratchpad UUID — NOT a budget-delta). Open a PR against
`main`; confirm CI green by observation. zsh: `$pipestatus`.
