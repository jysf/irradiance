---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. `handback:` is the RETURN path and is not
# optional: `just handback-sync SPEC-NNN` reads it and appends the cost session.
# Contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-093
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-046

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` is the one field the cost gate reads — report the REAL number
# from your interface (Claude Code: `/cost`). If your platform exposes NO token
# count, set tokens_total: null AND say why in `notes`. Do not invent a number.
handback:
  status: pending                  # completed | blocked | rejected
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-093: Two-component (Nf=2) SOF-3 decode + column de-interleave (build)

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-046` to `claude-opus-5` (implementer) for
the **`build`** cycle: extend `ljpeg` to decode a **two-component (`Nf=2`)
column-interleave** SOF-3 frame — two per-component predictor streams in one
interleaved scan — and de-interleave the two column components into one monochrome
`u16` plane, so both held Compression-7 DNGs decode. The `Nf=1` path stays
byte-for-byte unchanged. This is the re-plan spec (DEC-037); it unblocks SPEC-044's
bit-exact oracle.

## Context the Receiving Agent Needs

### Primary
- **Spec:** `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-046-two-component-nf2-sof-3-decode-and-column-de-interleave.md` — read all of it. `## Inputs` carries the §12 probe of both real strips (one interleaved scan, Ns=2, both H=V=1, one Td=0 DC table, predictor 1, Pt=0, no DRI); `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
- **Project brief** + the stage `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
- **Toolchain brief:** `guidance/toolchain-brief.md`.

### Decisions that apply
- `DEC-037` — the re-scope: decode the held files' monochrome two-component column-interleave; Bayer/CFA colour (3+ components) stays out. Read it first.
- `DEC-035` — clean-room / patent-clean (T.81 Annex H); two-component interleave is the same standard, no new algorithm.
- `DEC-016` (caller-owned no-alloc buffer), `DEC-018` (u16 modular `wrapping_*` arithmetic so `arithmetic_side_effects` is satisfied by intent).

### Constraints
- `no-panics-on-untrusted-input` — every malformed two-component scan returns a typed `Error`, never a panic.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work
- `SPEC-037` (shipped) — the single-component `reconstruct` / `decode_into` / `Frame` / `predict` / `HuffTable` you extend; `encode_lj92` / `encode_scan` in `mod tests` is the encoder to extend for a two-component round-trip.
- `SPEC-043` (shipped) — the Compression-7 route in `plane::unpack_into` that already sizes `out` as `rows × sensor.width` (= `Nf·X`); do not change it.

### The build contract
- Implement SPEC-046's `## Outputs`; make every `## Failing Test` pass (tier-A, hand-built two-component SOF-3 via the extended encoder).
- Keep the `Nf=1` path byte-for-byte unchanged (characterization test). `decode_into` stays `pub(crate)`; no new `Error` variant, no new dependency.
- `just test`, `just lint-ci`, `just fmt-check`, `just validate` all green; `#![forbid(unsafe_code)]` + the five-lint deny policy hold.
- Open a PR against `main` from `claude/spec-046-build-<suffix>` referencing `SPEC-046`, `STAGE-011`, `PROJ-003`, `HANDOFF-093`, `DEC-037`.

## Out of Scope
- The bit-exact `dnglab --raw-checksum` corpus oracle on the real files, and confirming the exact de-interleave mapping — **SPEC-044** (the analytic round-trip here CANNOT confirm the mapping; it shares the convention — `shared-path-oracle-tautology`).
- Provenance / determinism / binding hardening — **SPEC-045**.
- The Compression-7 routing / container — **SPEC-043** (shipped).
- `Nf>2`, subsampled components (`H`/`V`≠1), Bayer/CFA colour — out (DEC-037).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`, or an explicit FLOOR with a note if your interface has none, as HANDOFF-089..092 did), fill `## Handback` (reflection included), set both statuses `completed`, run `just advance-cycle SPEC-046 verify`, open the PR, and mark build `[x]` in the timeline. If blocked: fill `## Handback`, set `handoff.status: rejected` / `handback.status: blocked`, still report tokens, set `task.blocked: true`, and add a question to `guidance/questions.yaml`.

Keep commit-message bodies PLAIN PROSE — this repo's push ruleset rejects code tokens (arrows, `#[...]`, `--flags`, dense backticks) in commit messages; put rich detail in the PR body and files.

---

## Handback

*Filled in by the receiving agent.*

### Execution notes
- **Branch / PR:** <fill>
- **Completed at:** <YYYY-MM-DD>
- **All acceptance criteria met?** <yes/no — walk each of SPEC-046's six>

### Cost self-report
- **Tokens (total):** <real `/cost`, or a FLOOR with a note>
- **Estimated USD:** <or null>
- **Duration (minutes):** <n>
- **Source of the number:** <interface>

### Drift and new artifacts
- **New decisions emitted:** <none expected — DEC-037 covers the re-scope>
- **Deviations from spec:** <...>
- **Follow-up work identified:** <SPEC-044 is next — the dnglab oracle that confirms the de-interleave mapping; flag anything surprising about the real scan if you probe it>

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
