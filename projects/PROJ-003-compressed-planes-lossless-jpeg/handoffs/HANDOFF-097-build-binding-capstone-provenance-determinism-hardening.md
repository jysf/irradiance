---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. `handback:` is the RETURN path and is not
# optional: `just handback-sync SPEC-NNN` reads it and appends the cost session.
# Contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-097
  cycle: build                     # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-045

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
  status: null                     # completed | blocked | rejected
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-097: Binding capstone — provenance, determinism, hardening (build)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-045` to the implementer for the **`build`**
cycle: the STAGE-011 capstone — a DNG Compression-7 **binding** provenance-ledger row, a
plane-level macOS/Linux determinism pin, the lossless-JPEG header-hardening the external
0.2.0 review flagged (`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces), and continuous
fuzzing of the **public** Compression-7 route — all closing STAGE-011 and PROJ-003. The
design §12 probe already established that determinism (develop level, SPEC-044), the
two-component restart reset (SPEC-046), and tiled rejection are groundwork-covered, so the
new code is small and localized.

## Context the Receiving Agent Needs

### Primary
- **Spec:** `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-045-binding-capstone-a-provenance-ledger-entry-for-the.md`
  — read all of it. `## Inputs` carries the §12 probe (exact gaps + line refs, what is
  already covered); `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` / `## Notes`
  are the contract.
- **Project brief** + the stage `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
- **The review triage:** `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` — the source
  of the §2.1 / §2.4 items folded here.
- **Toolchain brief:** `guidance/toolchain-brief.md` (`just lint-ci` is the authoritative
  local clippy; the tier-B lesson below).

### The four gaps to close (from the §12 probe — transcribe, don't re-derive)
- `src/ljpeg.rs` `parse_scan_header` validates the predictor `ss` but NOT `al` (Pt); add
  `Pt < P` (pass `precision` in, or check in `parse_headers` once both are known).
- `src/ljpeg.rs` `parse_dht` builds only `tc == 0` tables and ignores other classes / any
  `th`; reject `Tc != 0` and `Th > 3` with a typed `Error`.
- `src/ljpeg.rs` `find_table` returns the FIRST `(th, _)` match; make a later same-`Th` DHT
  win (return the last match, or overwrite in `parse_dht`).
- `fuzz/fuzz_targets/{plane,develop,demosaic}.rs` skip Compression-7 via
  `require_uncompressed().is_err()`; drop that early-return and add a synthetic Compression-7
  SOF-3 seed to `fuzz/seeds/plane/` so the PUBLIC route is fuzzed.

Already covered — do NOT rebuild, just pin/cite: determinism at develop level
(`tests/develop_oracle.rs`, SPEC-044); the two-component restart reset
(`reconstruct_two_component`, SPEC-046, the `ri = x` case in
`two_component_interleaved_decodes_bit_exact`); tiled Compression-7 as a clean
`UnsupportedStripLayout` (`src/plane.rs`).

### Decisions that apply
- `DEC-036` — the dev-only `fuzzing` feature + `__fuzz_ljpeg` hook. **Validation decision for
  this spec: KEEP the hook and ADD public-path coverage** (the direct hook fuzzes the decoder
  deeper than the container-gated route; it is doc-hidden, feature-gated, zero public surface).
  Record this reasoning in the handback; it dispositions DEC-036's open Validation clause — no
  new DEC.
- `DEC-035` — clean-room T.81 (the header-hardening is Annex B/C conformance, no new algorithm).
- `DEC-003` — corpus never committed; a synthetic own-work Compression-7 seed IS fine to commit.
- `DEC-018` — u16 modular `wrapping_*` arithmetic.

### Constraints
- `no-panics-on-untrusted-input` — every new guard returns a typed `Error`, never a panic.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work / building blocks
- `tests/support/tiff.rs` — the DNG builder (`tiff`, `Ifd`, `Entry`, `long`, `cfa_dng_16bit`).
  Add `compression7_dng(order, …)` wrapping a SOF-3 strip from the `cfg(test)`-reexported
  `encode_lj92`; use it for the hardening tests and the fuzz seed.
- `tests/plane_unpack.rs` — the AC7 hostile-input home + `compression7_planes_route_to_the_decoder`.
- `docs/provenance-ledger.md` — the `src/plane.rs` row (stale "never decoded" note) and the two
  `src/ljpeg.rs` decode rows this binding row complements.
- `SPEC-043` / `SPEC-044` / `SPEC-046` (shipped) — the routing, the oracle, the two-component decode.

### The build contract
- Implement SPEC-045's `## Outputs`; make every `## Failing Test` pass.
- Header guards are parse-level, typed, no algorithm change; prefer reusing an existing
  `Ljpeg*` variant (add at most one additive variant). `ljpeg` stays `pub(crate)`.
- Keep the `__fuzz_ljpeg` hook + `fuzzing` feature + `fuzz/fuzz_targets/ljpeg.rs` (DEC-036
  Validation); drop only the `require_uncompressed` early-return and add the Compression-7 seed.
- Open a PR against `main` from `claude/spec-045-build-<suffix>` referencing PROJ-003,
  STAGE-011, SPEC-045, HANDOFF-097, DEC-036, DEC-035.

### Running the gates (the tier-B lesson)
- The plane-level determinism test is tier-B, corpus-gated. Set
  `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and run the
  TARGETED suites — `cargo test --lib ljpeg::`, `cargo test --test plane_unpack`,
  `cargo test --test plane_oracle` / `cargo test --test develop_oracle` — rather than a blind
  full `just test` (its unrelated tier-B oracles buffer for minutes). Confirm `just fmt-check`,
  `just lint-ci`, `just validate` green locally; lean on the PR's CI for the broad sweep.
- For the fuzz change, a short `cargo +nightly fuzz run plane -- -runs=…` on the new
  Compression-7 seed is enough to show the public route is panic-free; the full campaigns run
  on their own cadence.

## Out of Scope
- `#[non_exhaustive]` (PROJ-002 0.2.0 / SPEC-042), the opcode `MAX_TAG_VALUES` bound (PROJ-001),
  `LinearizationTable` rejection (cross-cutting, no held file), `tag_name` in `Error` Display
  (cosmetic) — all explicit SPEC-045 Non-Goals.
- Retiring the `__fuzz_ljpeg` hook (kept — DEC-036 Validation).
- The reconstruction algorithm, routing, bit-exact oracle (SPEC-046/043/044, shipped);
  Bayer/CFA colour multi-component (DEC-037); the review's §3B colour geometry and §5 perf.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`, or an explicit FLOOR with
a note if your interface has none, as HANDOFF-089..096 did), fill `## Handback` (reflection
included, and the DEC-036-keep reasoning), set both statuses `completed`, run
`just advance-cycle SPEC-045 verify`, open the PR, and mark build `[x]` in the timeline. If
blocked: fill `## Handback`, set `handoff.status: rejected` / `handback.status: blocked`, still
report tokens, set `task.blocked: true`, and add a question to `guidance/questions.yaml`.

Keep commit-message bodies PLAIN PROSE — this repo's push ruleset rejects code tokens (arrows,
attribute brackets, dash-dash flags, dense backticks) in commit messages; put rich detail in
the PR body and files.

---

## Handback

*Filled in by the receiving agent.*

### Execution notes
- **Branch / PR:** <branch> / PR #<n>
- **Completed at:** <YYYY-MM-DD>
- **All acceptance criteria met?** <yes/no — walk each SPEC-045 acceptance criterion>

### Cost self-report
- **Tokens (total):** <real /cost number, or an explicit FLOOR with its source>
- **Estimated USD:** <or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** <interface>

### Drift and new artifacts
- **New decisions emitted:** <none expected — DEC-036 Validation dispositioned in prose, not a new DEC>
- **Deviations from spec:** <none, or what and why>
- **Follow-up work identified:** <e.g. the deferred §4 items, if worth a spec>

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
