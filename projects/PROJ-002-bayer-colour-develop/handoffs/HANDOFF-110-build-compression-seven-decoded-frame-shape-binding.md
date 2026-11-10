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
  id: HANDOFF-110
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-053

project:
  id: PROJ-002
  stage: STAGE-010
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
  tokens_total: 12629153           # REAL combined count — what cost-audit reads
  estimated_usd: 8.72
  duration_minutes: 24
  branch: claude/spec-053-build-570012
  pr: 169
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-110: Compression-7 decoded-frame shape binding (SPEC-053, group B)

## Delegation Summary

The architect hands `SPEC-053` to the implementer for the **build** cycle: bind the
decoded SOF-3 frame's effective shape to the strip's declared geometry (not just its
area) and reject component sampling factors other than `H=V=1`, so a Compression-7
DNG that would otherwise decode to a silently mis-strided image returns a typed
error instead (review finding #2 + the silent-garble signal). The whole decision is
**DEC-047**. **Read SPEC-053 in full and DEC-047 first; `## Design` has the exact
sites (`unpack_jpeg` pre-decode check; `parse_frame_header` H/V check).**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-053-compression-seven-decoded-frame-shape-binding.md` — all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-047-bind-the-decoded-sof3-frame-shape-to-the-strip.md` — the rule, the two variants, the rejected alternative, and why `DEVELOP_VERSION` does not move.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #2).
- **Code:** `src/plane.rs` (`unpack_jpeg` l.487, the per-strip loop + `decode_into` call l.582), `src/ljpeg.rs` (`parse_headers` l.341, `parse_frame_header` l.436 with the `_hv` read at l.470, `Frame` l.200, the area checks l.993/l.1182), `src/lib.rs` (`Error` l.217), `tests/support/tiff.rs` (`lj92_strip`, `compression7_dng_with_strip`, `compression7_entries`), `fuzz/fuzz_targets/ljpeg.rs`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap (DEC-004 rule 5).

> ⚠ The spec's line numbers were read 2026-11-10 against the current tree;
> re-confirm before editing in case of drift.

### Decisions that apply

- `DEC-047` — the shape-binding + sampling-factor rejection this spec builds.
- `DEC-037` — subsampled `H/V != 1` is out of scope (this rejects it, does not implement it); the `Nf=2` column de-interleave is `2 × samples_per_line` wide.
- `DEC-045` — the re-plan; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus is never committed; the bit-exact corpus check is a local run only.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — five deny lints; checked arithmetic for the `samples_per_line × components` product; the guards run on hostile SOF-3 bytes.
- `oracle-must-be-shown-red` — two red-proofs (shape, sampling), shown red before green.
- `library-not-application` — zero I/O; caller-owned buffers; no signature changes.

### Prior related work

- `SPEC-043`/`SPEC-046` (`STAGE-011`) — the Compression-7 routing and the two-component (`Nf=2`) de-interleave decode this binds a shape check onto.
- `SPEC-052` (`HANDOFF-109`, group A) — the sibling parser-resource guard; same "plausible-wrong-`Ok` at the boundary" failure class.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-053's Acceptance Criteria (the DEC-047 guards).
- All failing tests in SPEC-053 passing, including the two red-proofs shown red first.
- The `fuzz/fuzz_targets/ljpeg.rs` extension; run `just fuzz` (target `ljpeg`) 60 s clean.
- CHANGELOG `[Unreleased]` entry and a `docs/provenance-ledger.md` row.
- A PR against `main` from branch `claude/spec-053-build-<suffix>`.
- PR description referencing: HANDOFF-110, SPEC-053, STAGE-010, PROJ-002, and
  DEC-045 / DEC-047.
- The tier-B corpus run result (both held Compression-7 files still decode bit-exact)
  recorded in the handback.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- Supporting subsampled (`H/V != 1`) frames or any new SOF process — this rejects them.
- Any change to a correctly-shaped `H=V=1` decode's output, or a `DEVELOP_VERSION` bump.
- Changing `decode_into` / `parse_headers` / `unpack_into` signatures (the shape check
  is a pre-decode call in `unpack_jpeg`; see DEC-047's rejected alternative).
- The restart-interval (`DRI`) MCU-reset hardening — a separate item.
- The parser resource guard (SPEC-052) and the licence/package work (SPEC-055).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-053-build-570012` — https://github.com/jysf/irradiance/pull/169
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes.
  - Equal-area transpose (4 × 6 frame in a 6 × 4 strip, both byte orders) → `Err(LjpegPlaneShapeMismatch { 6, 4, 4, 6 })`; it returned a silent `Ok` before the guard (red run recorded below).
  - Both component counts: `Nf=1` and `Nf=2` round-trip `Ok` when the shape matches; an `Nf=2` frame with `2·X == width` passes (pins the `× components` factor) and an `Nf=2` transpose is rejected.
  - `Hi/Vi != 0x11` (0x22, 0x21, 0x12, 0x00) rejected on `Nf=1` and `Nf=2`, including a frame where only the second component is subsampled.
  - Panic-free: five deny lints + `lint-ci` clean; the shape product is `saturating_mul` on `u16 × u8` widened to `u32` (cannot overflow; clippy's `manual_saturating_arithmetic` rejects the `checked_mul().unwrap_or(MAX)` form). `ljpeg` fuzz target 60 s clean (`cargo +nightly fuzz run ljpeg`, 4,894,453 runs in 61 s, zero crashes, including the new shape-invariant assertion).
  - Red-proofs, control first (DEC-009): shape — control `err lossless JPEG: strip frame decodes to 4x6 … declares 6x4`, mutant (shape condition forced false in `src/plane.rs`) `ok …`; sampling — control `err … sampling factors 0x22 …`, mutant (condition forced false in `src/ljpeg.rs`) `ok …`.
  - **Tier-B corpus (local, `IRRADIANCE_CORPUS_DIR` set):** both held Compression-7 DNGs decode `Ok` and bit-exact — `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` 5984×4000 plane MD5 `c72e7d9579e2adaef9c6247841b8ac1d`, `PENTAX-K3III-MONO/K3III.DNG` 6304×4224 plane MD5 `98bae1339c25eb6e04fbac6b5f643189`, both equal to the manifest `raw_checksum` (`tests/ljpeg_shape_binding.rs::corpus_comp7_files_still_bit_exact`; the existing `plane_oracle` Compression-7 oracle also stays green).
  - `DEVELOP_VERSION` unchanged (`"4"`).
- **Red run before the fix** (guards absent, error variants already added so the file compiles): 4 of 8 failed — `equal_area_transposed_frame_is_rejected` got `Ok`, `nf2_effective_width_is_twice_samples_per_line` got `Ok`, `subsampled_component_is_rejected` got `Ok`, `line_count_mismatch_is_a_shape_mismatch` got the old `LjpegPlaneWrongLength`.
- **Gates:** `just test` (with `IRRADIANCE_CORPUS_DIR` set; 0 failures), `just lint-ci`, `just fmt-check`, `just validate`, `just cost-audit` — all pass.

### Cost self-report

- **Tokens (total):** 12,629,153 (input 139 + cache write 180,616 + cache read 12,392,675 + output 55,723; 69 deduped API responses, all `claude-opus-5-5`). Excludes the final commit/report turns.
- **Estimated USD:** ≈ $8.72 (Opus list rates)
- **Duration (minutes):** ~24 (17:36–18:00 UTC)
- **Source of the number:** this session's own transcript (`~/.claude/projects/…spec-053-build-570012/9ee95584-1abe-49c0-9c53-cca1feed9c91.jsonl`, anchored on the scratchpad UUID), `usage` summed over unique `message.id` (input + cache-creation + cache-read + output). `/cost` is not available inside a delegated desktop session, so the transcript `usage` is the harness report.

### Drift and new artifacts

- **New decisions emitted:**
  - none (DEC-047 covers the build as written).
- **Deviations from spec:**
  - The shape rule lives in a small `pub(crate) fn plane::check_strip_shape(frame, expected_width, expected_height)` called from `unpack_jpeg` before `decode_into`, rather than inline. Same site and semantics as DEC-047; factored so the `ljpeg` fuzz hook can run the exact same guard on every parsed header (the spec's "the fuzz target runs both guards on every input").
  - The doc-hidden, `fuzzing`-feature-only `__fuzz_ljpeg` now returns `bool` (invariants held) and `fuzz/fuzz_targets/ljpeg.rs` asserts it. The library's lint policy denies `panic!`, so the assertion has to sit in the fuzz crate; `decode_into` / `parse_headers` / `unpack_into` signatures are unchanged as DEC-047 requires.
  - An existing in-crate test (`plane::tests::compression7_frame_dimension_mismatch_is_typed_error`, a 3×2 frame in a 2×2 strip — unequal area) now expects `LjpegPlaneShapeMismatch` instead of `LjpegPlaneWrongLength`: the pre-decode shape check fires first, as the design places it. Noted in the CHANGELOG entry.
  - `tests/support/tiff.rs` gained `lj92_strip_components(samples, plane_cols, rows, precision, nf, hv)` (`Nf=2` column-interleave + arbitrary `Hi/Vi`); `lj92_strip` now delegates to it and is byte-identical (asserted in the test, and every regenerated fuzz seed came out unchanged).
  - New fuzz seeds: `fuzz/seeds/ljpeg/valid-two-component-p12.lj92`, `fuzz/seeds/ljpeg/subsampled-hv22.lj92`, `fuzz/seeds/plane/compression7-transposed-frame.tiff` (via `examples/fuzz-seeds.rs`).
- **Follow-up work identified:**
  - `Error::LjpegUnsupportedComponentCount`'s doc comment and `Display` still say only single-component (`Nf = 1`) scans are supported; `Nf = 2` has been supported since SPEC-046. Message-only fix — fold into SPEC-040 (error kinds) or the 0.2.0 release sweep.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little; the line numbers were accurate. One gap: the spec says "the fuzz target runs both guards on every input", but the shape guard lives in `unpack_jpeg`, which the `ljpeg` target never reaches, and the hook cannot assert (the crate denies `panic!`). Resolving that took the small helper plus the `bool`-returning hook described above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Clippy's `manual_saturating_arithmetic` (under `-D warnings` in `lint-ci`) rejects the spec's suggested `checked_mul … unwrap_or(MAX)` idiom; `saturating_mul` is the lint-clean form. Worth a line in the toolchain brief next to the arithmetic-side-effects guidance.

3. **If you did this task again, what would you do differently?**
   — Write the `Nf=2` / `Hi/Vi` strip encoder first. Integration tests had no `Nf=2` SOF-3 builder (only the in-crate `cfg(test)` encoder had one), and every acceptance criterion needed it.
