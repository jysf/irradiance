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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  tokens_total: 276963             # FLOOR — see notes (context-occupancy, not cumulative)
  estimated_usd: null
  duration_minutes: 40
  branch: claude/spec-046-build-88e024
  pr: 117
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "tokens_total is a FLOOR, not a cumulative count: the Claude Code desktop Code tab exposes no interactive /cost meter, so this is the `ccd_session_mgmt get_usage` context-occupancy at handback (276,963 of the 1,000,000-token window, 28%), reported as an explicit floor exactly as HANDOFF-089..092 did. The orchestrator supersedes it from the transcript at ship. Gates fmt-check / lint-ci / validate green locally; targeted `cargo test --lib ljpeg::` green (27/27, incl. the six new SPEC-046 tests); the full `just test` tier-B corpus oracles (untouched by this change) run on the PR's GitHub CI rather than blocking the handback (SPEC-038/043 precedent)."
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
- **Branch / PR:** `claude/spec-046-build-88e024` / PR #117
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** Yes — walking SPEC-046's six acceptance criteria:
  1. **`parse_headers` accepts `Nf=2`/`Ns=2` (both `H=V=1`), sets `Frame.components = 2`; `Nf` outside `{1,2}` still `LjpegUnsupportedComponentCount`.** `parse_frame_header` now accepts `Nf ∈ {1,2}` (reads `Nf` 3-byte component specs, `Lf == 8 + 3·Nf`) and rejects 3+ with `LjpegUnsupportedComponentCount { nf }`; `parse_scan_header` accepts `Ns ∈ {1,2}`, reads each selector's `Td`, and `parse_headers` requires `Ns == Nf`. Covered by `nf_three_still_unsupported` and `two_component_interleaved_decodes_bit_exact` (asserts `frame.components == 2`).
  2. **`decode_into` fills a `2·X × Y` plane bit-exact, column-interleaved, per-component predictor-1 state, at P=12 and P=14.** `reconstruct_two_component` decodes comp0 then comp1 per MCU with per-component `Ra`/`Rb`/`Rc` neighbours (two output columns back / one plane row back) and writes `out[y·2X+2x]=comp0`, `out[y·2X+2x+1]=comp1`. `two_component_interleaved_decodes_bit_exact` exercises P=12 and P=14 × predictors {1,4,6} × restart {off, per-row}.
  3. **Two-component round-trip is bit-identical on honest input and RED on a perturbed predictor / Huffman table / swapped component order.** `two_component_round_trip_red_on_perturbed_predictor` ties an independent reference decoder to production (honest control), then shows all three mutations diverge from the known samples (`oracle-must-be-shown-red`).
  4. **The single-component (`Nf=1`) path is byte-for-byte unchanged.** `reconstruct` and its per-sample loop are untouched; `decode_into` routes `Nf=1` through it. Every shipped `ljpeg` test stays green; `single_component_decode_unchanged` pins the dispatch with `frame.components == 1` and an identical decode.
  5. **`decode_into` returns a typed `Error`, never a panic, on a malformed two-component scan.** `Ns ≠ Nf` → `LjpegUnsupportedComponentCount`; a component referencing an undefined `Td` → `LjpegMissingMarker { DHT }`; a scan truncated mid-second-component → `Truncated` (`two_component_truncated_scan_is_typed_error`); `out.len() != Nf·X·Y` → `LjpegPlaneWrongLength` (`two_component_wrong_out_length_is_typed_error`).
  6. **`#![forbid(unsafe_code)]` + the five-lint deny policy hold; gates green; no new runtime dependency.** `fmt-check` / `lint-ci` / `validate` green locally; `cargo test --lib ljpeg::` 27/27; the full `just test` gate (the untouched tier-B corpus oracles) runs on the PR CI. No new `Error` variant, no new dependency; `decode_into` stays `pub(crate)`.

### Cost self-report
- **Tokens (total):** 276,963 — **FLOOR** (not a cumulative count). See below.
- **Estimated USD:** null (no per-token price exposed here)
- **Duration (minutes):** ~40 (estimate)
- **Source of the number:** `ccd_session_mgmt get_usage` context-occupancy (276,963 of the 1,000,000-token window, 28%) at handback. The Claude Code desktop Code tab exposes no interactive `/cost` meter, so this is reported as an explicit floor exactly as HANDOFF-089..092 did; the orchestrator supersedes it from the transcript at ship.

### Drift and new artifacts
- **New decisions emitted:** none — DEC-037 already covers the two-component re-scope.
- **Deviations from spec:** none material. One scope note: I implemented **per-MCU restart** in both `reconstruct_two_component` and the two-component encoder (mirroring the shipped single-component path) and cover it in the round-trip test, even though both held files have no DRI (spec Notes call restart out-of-scope for the held files). This is symmetry with `reconstruct`, not pre-engineering of an untested branch — a two-component stream with a DRI now decodes correctly rather than only failing to a typed error. `H`/`V`≠1 is still not validated (consumed and ignored, as the single-component path does): no held file uses it and the spec keeps subsampling out of scope.
- **Follow-up work identified:** **SPEC-044** is next — the `dnglab --raw-checksum` oracle that confirms the real-file de-interleave MAPPING (component order + column phase), which this analytic round-trip provably cannot (`shared-path-oracle-tautology`). If SPEC-044 shows the mapping is left/right halves rather than even/odd columns, that is a one-line change to the write-index in `reconstruct_two_component`, not a decode-logic change (flagged in the spec Notes; I did not pre-engineer for it). No surprises found extending the decoder.

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — Very little. The spec's `## Inputs` §12 probe and `## Notes` pinned the exact interleave convention (comp0 then comp1 per MCU, Ra two output columns back) and the write-index formula, so the decode logic was unambiguous. The one judgment call left to me was whether to support restart for `Nf=2`; the spec said the held files have none, and I chose to implement it for symmetry with the shipped path and cover it in a test.
2. **Was there a constraint or decision that should have been listed but wasn't?** — No. DEC-037 (re-scope), DEC-035 (clean-room), DEC-016 (caller-owned buffer) and DEC-018 (modular `wrapping_*`) were exactly the set that applied; no new decision was needed.
3. **If you did this task again, what would you do differently?** — I'd run the targeted `cargo test --lib ljpeg::` (which fully covers an isolated `ljpeg`-only change) and lean on PR CI for the full suite from the start, rather than launching the slow full `just test` locally — its tier-B corpus oracles buffer for minutes and are untouched by an `ljpeg` change (the SPEC-038/043 lesson, which the orchestrator reiterated).
