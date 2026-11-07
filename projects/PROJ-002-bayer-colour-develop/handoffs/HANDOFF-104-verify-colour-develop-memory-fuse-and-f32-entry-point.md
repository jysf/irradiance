---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-104
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-07
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-049

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 206109             # REAL combined count — what cost-audit reads
  estimated_usd: 3.5               # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/spec-049-verify
  pr: null                         # stamped after the PR opens
  completed_at: 2026-11-07         # YYYY-MM-DD
  notes: "Verdict APPROVED at main 212eb985cb5a3cfc7335f6db78733fde8d350be9 (the rolling-cache state, #147 then #148 merged). Cold verify (this spec skipped verify before #147/#148 merged directly). tokens_total is this verify session's get_usage context-window token figure — an interactive /cost cumulative is not callable as a tool in a delegated session, same constraint the build handback (HANDOFF-103) noted; reported, not invented. If the orchestrator has the true /cost cumulative at ship, prefer it."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-104: colour-develop memory fuse and f32 entry point (verify)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-049` to `claude-opus-5`
(acting as verifier) for the **verify** cycle: an independent cold review of the
colour-develop fuse + f32 entry point as it stands on `main` (the rolling-cache
version — PR #147 did the fuse, PR #148 revised it to a rolling normalised-row
cache; both merged). The verifier did not write the code.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-049-colour-develop-memory-fuse-and-f32-entry-point.md` — Acceptance + Failing Tests are the contract.
- **Decision:** `./decisions/DEC-042-fuse-the-colour-develop-passes-and-add-an-f32-entry-point.md` — as amended for the rolling-row cache.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-103-build-colour-develop-memory-fuse-and-f32-entry-point.md`.
- **Code (on `main`):** `src/color.rs` (fused `develop_color_into`, `make_normalizer`, `develop_color_f32_into`), `src/demosaic.rs` (`develop_color_fused` + rolling cache), `src/lib.rs`, `src/tone.rs::to_linear_f32_into` (f32 reference), `tests/color_oracle.rs`.

### Decisions that apply

- `DEC-042` — the fuse + f32 entry point (this spec's decision; amended for the rolling cache).
- `DEC-041` — STAGE-012 perf discipline: `dst_after == dst_before` proof + measured peak-RSS/time + the DEC.
- `DEC-040` — index the CFA phase and level slot from the ActiveArea origin (preserved).
- `DEC-038` — the colour maths (ColorMatrix/Bradford); must not move.
- `DEC-016` / `DEC-018` — caller-owned buffers; clamp/round (`quantize`) discipline.

### Constraints that apply

- `no-panics-on-untrusted-input` — panic-free.
- `no-new-top-level-deps-without-decision` — zero new deps; no SIMD/unsafe.
- `one-spec-per-pr` — this PR is SPEC-049 only.

### Prior related work

- `HANDOFF-103` (build) — the fuse + rolling-cache revision.
- `PR #147` / `PR #148` — the build and its rolling-cache revision (both merged to `main`).

## Expected Deliverables

### The verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection honest?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Re-measuring the full 23.4 MP release benchmark on this host (noisy/host-dependent; the ledger flags re-measure elsewhere). The memory win was instead confirmed structurally + against the arithmetic.
- Any change to the `u16` output, colour maths (`DEC-038`), demosaic kernel, or the SPEC-047/048 work.

## Return Criteria — how to hand back

Done: `handback:` filled (real `tokens_total`), `## Handback` filled, statuses
set to `completed`, verify PR opened against `main`.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-049-verify` → verify PR against `main`.
- **Completed at:** 2026-11-07
- **All acceptance criteria met?** yes.
- **For `verify`:** the verdict — **✅ APPROVED (at commit `212eb985cb5a3cfc7335f6db78733fde8d350be9`)** — the current `main`, the rolling-cache state (#147 then #148 merged).

**What was checked, and the evidence.**

- **AC1 — `u16` output byte-identical.** `just test` fully green (163 unit +
  22 `color_oracle` + the integration suites; tier-B corpus absent and skips
  **loudly**, AGENTS.md bar 4). `fused_develop_color_is_byte_identical_to_the_reference`
  passes across 3 sizes (9×7, 20×24 taller than the 8-row ring, 3×5) × all four
  Bayer phases. The reference is **genuinely independent and non-tautological**:
  `reference_normalize` (a separate transcription of the per-slot level map) →
  the **real public `demosaic_into`** (not the fused traversal under test) →
  `reference_to_srgb` (an independent matrix). The fused `develop_color_into`
  drives a different traversal (`develop_color_fused` + rolling cache + green
  plane + matrix sink), so a cache bug diverges from the reference. Confirmed by
  **adversarial mutation, observed red personally**: (a) perturbing the cache
  index in `cache_cell` (`col` → `col+1`) turned the byte-identity test red
  (proving the interior cache path is exercised and checked); (b) perturbing the
  matrix-into-`dst` sink (`+1.0` on out_r) turned the byte-identity + compose
  tests red. `demosaic_into` and the shared Hamilton-Adams kernel are
  **byte-for-byte unchanged** — the `src/demosaic.rs` diff is a single pure-add
  hunk (589 lines added, 0 removed) below `u32_from`; `src/color.rs` removed only
  `normalize_mosaic` + doc comments (the DEC-038 maths, `quantize`,
  `normalize_sample`, `apply_matrix_into` untouched). The SPEC-047
  grey-across-origins goldens (odd/even ActiveArea origins) and the SPEC-034
  dual-illuminant set exercise the fused path with non-zero origin and stay green.

- **AC2 — scratch buffers gone + peak-RSS row.** Confirmed **structurally**: the
  fused `develop_color_into`/`develop_color_f32_into` allocate only the `w·h`
  `green` plane; `develop_color_fused` allocates only the `RING·w` (8-row, ~91 KB)
  ring — no `camera_rgb`, no `normalized`, no per-row allocation in the hot loop.
  The ledger row (`docs/provenance-ledger.md`) records 376,143,872 B → 235,864,064 B
  = **−140,279,808 B (−133.8 MiB)**, which matches `camera_rgb`'s exact size
  (`5712·4096·3·2 = 140,378,112 B`) minus the ~91 KB ring. Self-consistent.

- **AC3 — `develop_color_f32_into` equals `to_linear_f32_into(develop_color_into(..))`.**
  `develop_color_f32_equals_to_linear_of_u16` passes (all sizes × phases). By
  construction the f32 sink computes `f32::from(quantize(..)) / 65535.0` — the
  identical `quantize` the integer path uses, then the identical `/65535.0`
  `to_linear_f32_into` uses (`src/tone.rs`: `f32::from(sample) / 65535.0`), not a
  divide of the raw `f64` matrix result. The mutation above (perturbing only the
  **u16** sink) also turned this test red, confirming it genuinely couples the two
  paths. `develop_color_f32_into` is the only public-surface addition (additive,
  semver-minor); `develop_color_into`'s signature is unchanged.

- **AC4 — panic-free, deterministic, no new dep.** `just lint-ci` green
  (clippy 0.1.98, `--all-targets --all-features -D warnings`); no `unsafe`, no new
  dependency, committed-constant `f64` ops + integer normalise, no threading/SIMD.
  Every cached/border read goes through `.get`/reflect; all arithmetic is
  checked/saturating/wrapping. The out-of-range-ActiveArea panic test is green.

- **Rolling cache correctness (reviewed by hand).** Each photosite is normalised
  exactly once (`fill_norm_row`, once per row as it enters the ring). The live
  span any output row `y` reads is `[y−1, y+3]` (5 rows: green row `y+1` reads
  `±2`, red/blue reads `±1`); `need_norm = min(y+4, h)` fills up to `y+3` and
  `need_green = min(y+2, h)` finalises green up to `y+1` (exactly what interior
  red/blue's `down = y+1` needs) — no off-by-one. `RING = 8 > 5`, and within any
  5-row window no two rows share a `row % 8` slot, so a still-live row is never
  overwritten; the slot invariant (slot `r%8` holds the newest row `≤ norm_filled`
  with that residue) holds. Interior fast path and border reflect both feed the
  same kernel helpers; `active_area_origin` is the one shared `crate::ifd` helper
  for both the normalizer slot and the demosaic phase, so they cannot disagree.

**Gates:** `just test` ✅ · `just lint-ci` ✅ · `just fmt-check` ✅ ·
`just validate` ✅ · `just cost-audit` ✅ · `just decisions-audit --changed`
advisory-only (exit 0) — the change is consistent with DEC-040 (origin preserved),
DEC-041 (perf discipline), DEC-042. No ship-blockers.

### Cost self-report

- **Tokens (total):** 206109
- **Estimated USD:** ~3.5 (order of magnitude; Opus list rate, no cache discount)
- **Duration (minutes):** ~40
- **Source of the number:** harness report — `get_usage` (this session's
  context-window token figure). An interactive `/cost` cumulative is not callable
  as a tool in this delegated session (same constraint HANDOFF-103 noted); this is
  the harness's own figure, reported rather than invented. Prefer the true `/cost`
  cumulative at ship if the orchestrator has it.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-042 already carries the rolling-cache
  amendment the build made.
- **Deviations from spec:** the fused traversal lives in `src/demosaic.rs`
  (`develop_color_fused` + cache helpers), not `src/color.rs` as the spec's prose
  said — the build flagged and justified this (reuses the private Hamilton-Adams
  kernel verbatim, keeping bit-identity structural). No behavioural deviation.
- **Follow-up work identified:**
  - `SPEC-049/FU-1` (follow-up, non-blocking) — `DEC-042`'s `affected_scope` omits
    `src/demosaic.rs`, where the fuse added 589 lines (`develop_color_fused` + the
    cache helpers). Those additions are consistent with and governed by DEC-040 /
    DEC-041 (both of which list `src/demosaic.rs`), and the placement was justified
    in the build handback, so this is a traceability/documentation nit only. Add
    `src/demosaic.rs` to DEC-042's `affected_scope` at a convenient pass. Does not
    hold the spec.
  - Noted (not a finding): the spec's `cost.sessions` is still `[]` — the build
    cost (494844 tok) lives in HANDOFF-103's handback and syncs at ship via
    `just handback-sync SPEC-049`; this verify session appends here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The two-PR history (#147 normalise-on-read, #148
   rolling-cache revision) needed care to make sure I reviewed the *current* `main`
   state (the rolling cache), not the superseded first cut; the build note and
   `git log` made that unambiguous.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-042/041/040/038/018/016 were all named and sufficient. The only gap
   is cosmetic: DEC-042's `affected_scope` should list `src/demosaic.rs` (FU-1).

3. **If you did this task again, what would you do differently?**
   — Nothing material. Confirming the memory win structurally (the fused path
   allocates only the green plane + ring, `camera_rgb`/`normalized` are absent)
   plus the self-consistent arithmetic was a stronger and cheaper check than
   re-running a noisy 23.4 MP RSS benchmark — I'd lead with that again.
