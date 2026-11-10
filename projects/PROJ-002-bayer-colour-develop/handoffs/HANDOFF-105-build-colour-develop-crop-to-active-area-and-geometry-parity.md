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
  id: HANDOFF-105
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-09
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-050

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
  tokens_total: 10626299           # REAL combined count — what cost-audit reads
  estimated_usd: 7.58
  duration_minutes: 15
  branch: claude/spec-050-colour-crop-geometry
  pr: null
  completed_at: 2026-11-09
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id; 10,398,650 of it cache reads) at handback time — /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the few final PR-opening turns."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-105: colour develop crop to active area and geometry parity

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-050` to the implementer for the
**build** cycle: make `develop_color_into` / `develop_color_f32_into` crop and
orient to the **same geometry as `develop_into`** — `ActiveArea` → `DefaultCrop`
→ `Orientation`, output at `output_dimensions(sensor)`, masked border dropped —
by scattering each developed pixel to its output slot **in the sink**, leaving the
fused demosaic kernel byte-for-byte. The developed *values* stay identical to
`SPEC-047`; only the frame moves. Contract change (`dst` is now
`output_dimensions`-sized), `DEVELOP_VERSION` 3 → 4, colour goldens regenerated
(`DEC-043`).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-050-colour-develop-crop-to-active-area-and-geometry-parity.md` — the Acceptance Criteria, Failing Tests and Notes for the Implementer are the contract.
- **Decision:** `./decisions/DEC-043-crop-the-colour-path-to-full-geometry-parity-with-develop-into.md` — **this is the design.** Crop at the sink, don't re-scope the kernel; preserve SPEC-049's memory shape (no RGB buffer); the inverse-orientation helper; the contract change.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).
- **Code:**
  - `src/color.rs` — `develop_color_into` (~265) and `develop_color_f32_into`
    (~332). **The change is here:** the sink closure currently writes
    `dst_triples[pixel]` for the whole plane; it must instead map `pixel` → its
    `DefaultCrop`+`Orientation` output slot and drop pixels outside the crop
    rectangle. Fix the `dst`-length checks + `Error::RgbBufferWrongLength` to
    `output_dimensions·3`. `pixel_count` / `interleaved_rgb_len` are the current
    (plane) sizes — the output size comes from `output_dimensions`.
  - `src/demosaic.rs` — `develop_color_fused` (~458) emits `emit(pixel = y·w + x,
    r, g, b)` for every plane pixel, colours already correct (SPEC-047 origin
    offset inside). **Do NOT change it.** The crop is a sink concern.
  - `src/develop.rs` — `resolve_geometry` (~135), `output_dimensions` (~230),
    `crop_source_coords` (~249, the forward output→crop map), and
    `crop_and_orient_from_active_into` (~613, how the mono path gathers). **Add**
    `output_coords_from_crop` (the source→output inverse of `crop_source_coords`)
    here, next to it, and the round-trip unit test.
  - `src/ifd.rs` — `ActiveArea`, `Sensor`, `active_area_origin`.
  - `src/lib.rs` — `DEVELOP_VERSION` (~147).

### Decisions that apply

- `DEC-043` — **this is the spec.** Sink-side scatter; kernel untouched; memory
  shape preserved (no active-area/full-plane RGB buffer); inverse-orientation
  helper + round-trip test; contract change; `DEVELOP_VERSION` bump.
- `DEC-040` — the ActiveArea-origin offset that *deferred* this crop. The phase /
  level offset lives in the kernel; leave it. This spec adds the crop it deferred.
- `DEC-019` — `DefaultCropOrigin` is relative to the `ActiveArea`. `resolve_geometry`
  already applies this; read the crop rectangle from it, don't recompute.
- `DEC-038` — the colour maths. Do NOT change it; the sink applies the same matrix.
- `DEC-041` — the STAGE-012 perf discipline: a `dst`-equivalence proof (here, the
  kept pixels equal the whole-plane develop at their source) + a measured peak-RSS
  line in `docs/provenance-ledger.md` (expect **no regression** from SPEC-049).
- `DEC-016` / `DEC-018` — caller-owned buffers; the `quantize` clamp/round the f32
  path must divide *after*.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply here:

- `oracle-must-be-shown-red` / `test-before-implementation` — write the failing
  tests first; each must fail against today's whole-plane output before the change.
- `no-panics-on-untrusted-input` — panic-free for any geometry; reuse
  `resolve_geometry`'s typed rejections; the scatter writes through `.get_mut`.
- `provenance-recorded-per-algorithm` — the `DEVELOP_VERSION` bump + the ledger row.
- `no-new-top-level-deps-without-decision` — zero new dependencies; no SIMD/unsafe.
- `one-spec-per-pr` — this PR is SPEC-050 only.

### Prior related work

- `SPEC-047` / `DEC-040` (shipped, PR #139) — the ActiveArea-origin phase/level
  offset; its grey-stays-grey red-proof must still hold on the kept pixels.
- `SPEC-049` / `DEC-042` (shipped, PR #150) — the fused kernel + rolling cache +
  `develop_color_f32_into` this builds on (`depends_on`); keep it byte-for-byte.
- The pyirradiance rawpy/LibRaw benchmark (report under `docs/reviews/`) — §3B
  "outputs the masked border, which LibRaw crops." LibRaw is the geometry
  yardstick, never linked.

## Expected Deliverables

- Code changes implementing SPEC-050's Acceptance Criteria.
- All failing tests in SPEC-050 now passing; the existing colour goldens
  regenerated for the new geometry (reason in the commit), with a trivial-geometry
  pin staying byte-identical.
- A PR against `main` from a `claude/spec-050-*` branch.
- PR description referencing: this handoff ID, SPEC-050, STAGE-012, PROJ-002, and
  DEC-043 / DEC-040 / DEC-019 / DEC-038.

### If `cycle: verify` — the verify contract

n/a (build cycle).

## Out of Scope

- **Any change to the developed colour *values* on the kept pixels, the colour
  maths (`DEC-038`), or the fused demosaic kernel** — the kept pixels must equal
  today's whole-plane `develop_color_into` at their source; if a kept value moves,
  stop.
- Re-scoping `develop_color_fused` to iterate the active window / reflect at the
  active edge — explicitly rejected in `DEC-043` (it would change near-edge values).
- Introducing an active-area or full-plane RGB intermediate buffer — that re-adds
  the ~140 MB SPEC-049 removed; scatter straight into `dst`.
- `WarpRectilinear` / `OpcodeList` on the colour path, P3 threading — separate
  specs. `unsafe`, SIMD, new dependencies.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-050-colour-crop-geometry` → PR (against `main`; number in the PR itself / the session report).
- **Completed at:** 2026-11-09
- **All acceptance criteria met?** yes.
  - *Output dims = `output_dimensions(sensor)`:* `colour_output_has_output_dimensions_geometry` (orientation 6 → 9×13 from a 13×9 crop; a whole-plane `dst` is now `RgbBufferWrongLength`), `colour_geometry_matches_the_mono_path` (all 8 orientations + absent). Real file: Canon 5D3 `5G4A9394.DNG` — `irr color` and `irr develop` both 5760×3840 (plane 5920×3950).
  - *Masked border gone:* output [0,0] is no longer the plane's (0,0) border pixel (same test).
  - *Colours unchanged on kept pixels:* `colour_crop_pixels_equal_the_whole_plane_develop_at_their_source` (4 phases × 9 orientation cases vs the independent reference pipeline, now ActiveArea-origin-aware) + the SPEC-047 grey-stays-grey on a cropped, orientation-7 grey across 4 origin parities × 4 phases. Direct evidence too: before touching code I captured today's whole-plane `develop_color_into` bytes for the odd-origin geometry fixture (4 phases); after the change all **3,744 kept pixels (4 phases × 8 orientations) were byte-identical** to those captured bytes at their source (source map taken from the mono `develop_into` on an index plane). Temporary harness, not committed.
  - *Geometry matches mono:* `colour_geometry_matches_the_mono_path` uses `develop_into` on an index plane as the oracle for WHERE each output pixel comes from — every colour output slot matches.
  - *f32 parity at new dims:* `develop_color_f32_still_equals_to_linear_at_new_dims` (4 phases × 9 orientation cases), plus its wrong-length typed error.
  - *DEVELOP_VERSION / no panic / determinism / no dep:* `DEVELOP_VERSION` 3 → 4; hostile geometry is the typed `InvalidActiveArea` / `InvalidDefaultCrop` / `UnsupportedOrientation` (`hostile_geometry_is_a_typed_error_on_the_colour_path`, updated out-of-range-origin test); every write via `.get_mut`, all arithmetic checked/wrapping/saturating under the crate's deny lints; zero dependencies; `develop_color_fused` byte-for-byte untouched.
  - *Red first:* all 9 new/updated colour tests failed against the whole-plane output before the change (the trivial-geometry pin passed both before and after, as intended); the round-trip unit test did not compile before the helper existed.
  - *Gates:* `just test`, `just lint-ci`, `just fmt-check`, `just validate`, `just cost-audit` — all green.
  - *Peak RSS (23.4 MP, Canon-like ActiveArea + DefaultCrop, median of 7):* u16 235,814,912 → **228,261,888 B** (−7.55 MB, exactly the smaller `dst`); f32 376,193,024 → **361,086,976 B** (−15.1 MB). No regression from SPEC-049. Time ≈1.05× (u16 399 → 420 ms) / ≈1.02× (f32 420 → 430 ms) — the per-pixel crop test in the sink. Recorded in `docs/provenance-ledger.md`.

### Cost self-report

- **Tokens (total):** 10,626,299
- **Estimated USD:** ~$7.58 (Opus list rates; 10.4M of the tokens are cache reads)
- **Duration (minutes):** ~15
- **Source of the number:** sum of this session's own transcript usage records (deduped by message id), claude-opus-5-5. `/cost` is not callable as a tool in a delegated session.

### Drift and new artifacts

- **New decisions emitted:** none (DEC-043 covers it).
- **Deviations from spec:** (1) The sink uses an affine `base + crop_x·step_x + crop_y·step_y` read off `output_coords_from_crop` once, plus a row cursor, instead of calling the helper per pixel — a first cut calling it per pixel (with a per-pixel orientation match) cost ≈7.5% develop time; this cut costs ≈5%. Same mapping, pinned by the same tests. (2) `Geometry` / `resolve_geometry` / `oriented_dimensions` became `pub(crate)` so the colour path reuses them. (3) `Error::RgbBufferWrongLength`'s Display text changed from "demosaic RGB buffer … (width x height x 3)" to a neutral "interleaved RGB buffer … (output width x output height x 3)" since it now also covers the cropped colour dst. (4) Error order now mirrors `develop_into`: src length → geometry → dst length. (5) `irr color`, README and CHANGELOG updated for the new contract.
- **Follow-up work identified:** the ≈5% sink overhead could be removed with a row-level emit API from the fused kernel (skip whole rows outside the crop, hand the sink a row slice) — only if the kernel can be touched later; out of scope under DEC-043. `WarpRectilinear`/`OpcodeList` on the colour path remains the known gap (README says so).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — Little. The one gap: the spec says "regenerate the colour goldens" but the colour goldens are the synthetic SPEC-047 tests in `tests/color_oracle.rs` (there is no stored golden file), and one of them — the out-of-range ActiveArea no-panic test — necessarily changes meaning (now a typed rejection, because the colour path resolves geometry).
2. **Was there a constraint or decision that should have been listed but wasn't?** — DEC-041's time discipline: the spec asked only for "RSS must not regress", but a sink-side scatter has a small per-pixel time cost that is worth stating up front as an accepted trade.
3. **If you did this task again, what would you do differently?** — Write the affine-stride sink first; the per-pixel orientation match was predictably the slow version.
