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
  id: HANDOFF-106
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: 4696185               # REAL combined count — what cost-audit reads
  estimated_usd: 3.60              # tokens_total × your rate, or your harness's number
  duration_minutes: 45
  branch: claude/spec-050-verify
  pr: 153
  completed_at: 2026-11-09               # YYYY-MM-DD
  notes: "verify APPROVED at 1576d2c; tokens summed from this session's own transcript usage records (claude-opus-5-5), /cost not callable as a tool in a delegated session"                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-106: colour develop crop to active area and geometry parity (verify)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-050` to `claude-opus-5`
(acting as verifier) for the **verify** cycle: an independent cold review of
build PR #152 (branch `claude/spec-050-colour-crop-geometry`, head `1576d2c`),
which crops and orients the colour develop to `develop_into`'s geometry at the
sink (`DEC-043`). The verifier did not write the code.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-050-colour-develop-crop-to-active-area-and-geometry-parity.md` — Acceptance + Failing Tests are the contract.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Decision:** `./decisions/DEC-043-crop-the-colour-path-to-full-geometry-parity-with-develop-into.md` — the sink-side crop/orient scatter.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-105-build-colour-develop-crop-to-active-area-and-geometry-parity.md` (on the PR branch).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-043` — crop the colour path to full geometry parity with `develop_into`, at the sink; kernel untouched; no new RGB buffer.
- `DEC-040` — CFA phase / level slot indexed from the ActiveArea origin (must still hold on kept pixels).
- `DEC-019` — `DefaultCropOrigin` is relative to the ActiveArea.
- `DEC-041` — STAGE-012 discipline: bit-identity proof + measured time/RSS ledger line + a DEC.
- `DEC-042` — the fused kernel + f32 entry point (memory shape to preserve).
- `DEC-038` — the colour maths; must not move.

### Constraints that apply

- `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`, `test-before-implementation`,
  `no-new-top-level-deps-without-decision`, `one-spec-per-pr`, `provenance-recorded-per-algorithm`.

### Prior related work

- `HANDOFF-105` / PR #152 — the build under review.
- `HANDOFF-104` / PR #149 — SPEC-049 verify (the fused path this crops).

## Expected Deliverables

### The verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection honest?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Behavioural surface exercised, not just shape.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Merging PR #152 or advancing SPEC-050's cycle (orchestrator, at ship).
- Any change to the colour maths, the fused kernel, or the mono path.

## Return Criteria — how to hand back

Done: `handback:` filled (real `tokens_total`), `## Handback` filled, statuses
set to `completed`, verify PR opened against `main`.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-050-verify` → PR #153 (this handoff file only).
- **Completed at:** 2026-11-09
- **All acceptance criteria met?** yes.
- **Verdict: ✅ APPROVED** at PR #152 head `1576d2c2a1bd9d8d10e2c7cbd8b34854e3f07a91`.

**Gates, run by me on the PR branch:** `just test` exit 0 (lib 164 passed incl.
`output_coords_from_crop_is_the_inverse_of_crop_source_coords`; `color_oracle`
28 passed incl. all 5 SPEC-050 failing tests plus the trivial-geometry pin and the
hostile-geometry test; every other suite green), `just lint-ci` exit 0,
`just fmt-check` exit 0, `just validate` exit 0, `just cost-audit` exit 0. PR
CI: all 13 checks SUCCESS. `just decisions-audit --changed origin/main`:
advisory only, flags DEC-036/040/041/042/043, which are all consistent with the change (below).

**Per criterion:**

1. *Output dims equal `output_dimensions(sensor)`.* Met. `check_color_buffers`
   sizes `dst` from `resolve_geometry` + `oriented_dimensions` (the mono path's own
   functions, made `pub(crate)`, not re-derived). Tests cover orientation 6
   (13×9 crop → 9×13) and all 8 + absent. A whole-plane `dst` is a typed
   `RgbBufferWrongLength`. **Behavioural:** release `irr color` on the real Canon 5D3
   `5G4A9394.DNG` writes a 5760×3840 P6 (plane 5920×3950), equal to `irr develop`'s
   `out_dimensions`. On `main` the same command writes 5920×3950.
2. *Masked border gone.* Met. Output [0,0] ≠ plane (0,0) in the test, and on the
   real file the PR image begins at plane (140, 96) = ActiveArea (122, 80) +
   DefaultCropOrigin (18, 16).
3. *Kept-pixel colour unchanged.* Met, and independently re-proved, not taken
   from the build's word. The committed test
   `colour_crop_pixels_equal_the_whole_plane_develop_at_their_source` is **not a
   tautology**: its expected side is `reference_develop_color` (the test's own
   origin-aware `reference_normalize`, then the real `demosaic_into`, then
   `reference_to_srgb`, which never touches the fused traversal or the sink),
   gathered through a test-local transcription of the TIFF orientation table. It
   covers 4 Bayer phases × 9 orientation cases on an odd-top ActiveArea, plus
   grey-stays-grey on a cropped orientation-7 grey across 4 origin parities × 4
   phases. **My own differential against the OLD code:** I dumped `main`'s
   whole-plane `develop_color_into` bytes (a separate `main` worktree) for 5
   geometries × 4 phases on a 21×17 plane. The geometries include odd-left and
   odd-top plane crop origins, a crop flush against the plane's far edge
   (reflect zone), the trivial full plane, and a 3×1 crop. Then on the PR I
   compared the new output, mapped through `develop_into` on an index plane, for
   all 9 orientation cases: **30,060 kept pixels byte-identical, 0 mismatches**,
   and f32 bit-equal to `u16/65535` at every one. **Real file:** all 3,840 rows of
   the PR's 5D3 render are byte-identical to `main`'s whole-plane render at the
   crop offset.
4. *Geometry matches mono.* Met. `colour_geometry_matches_the_mono_path` uses
   `develop_into` on an index plane (black 0, white 65535 → identity) as the oracle
   for WHERE each slot comes from, over all 8 orientations + absent, with dims
   asserted equal to `output_dimensions`. This is a real external oracle, not a
   restatement of the sink.
5. *Inverse helper.* Met. The round-trip test covers all 8 orientations, including
   the two 90° rotations 6/8, over 6 crop shapes (square, wide, tall, 1-wide,
   1-tall, 1×1). It also asserts in-bounds, no slot reused and no slot missed, so
   the helper is a bijection onto the transposed rectangle.
6. *f32 parity at new dims.* Met. `develop_color_f32_still_equals_to_linear_at_new_dims`
   (4 phases × 9) compares by `assert_eq!` on `f32`s (exact), and also checks the
   wrong-length error.
7. *Memory.* Met. `develop_color_fused` diff vs `main` is empty (`src/demosaic.rs`
   untouched). `CropScatter` holds only scalars (no `Vec`), and the only scratch
   is still the `width·height` green plane + SPEC-049's ring. No active-area or
   full-plane RGB buffer. **Measured:** release `irr color` max RSS on the 5D3 file,
   3 runs each: `main` 565.0 MB → PR 542.3 MB (−22.8 MB, the smaller `dst` and
   the CLI's own matching buffers). No regression.
8. *Contract change.* Met. `DEVELOP_VERSION` "3" → "4". Length checks moved to
   `check_color_buffers` (src → geometry → dst, `develop_into`'s order). The
   `RgbBufferWrongLength` doc and Display text are generalised. `irr color` sizes
   from `output_dimensions`, and the PPM header uses out dims. README, CHANGELOG,
   lib docs and the provenance ledger are updated. `irr color` is the only in-repo
   caller.
9. *No panic / determinism / deps / unsafe.* Met. Hostile `DefaultCrop` /
   `Orientation` / out-of-range ActiveArea give typed errors on both entry points.
   All writes go through `.get_mut`, and sink arithmetic is checked, wrapping or
   saturating. A zero plane width does not divide by zero (`checked_div`). No new
   dependency (`Cargo.toml`/`Cargo.lock` unchanged), no `unsafe`, nothing
   nondeterministic introduced.

**Oracle teeth (`oracle-must-be-shown-red`), mutated by me, each restored:**
M1 swap orientation 6/8 arms in `output_coords_from_crop` → round-trip + 3 colour
tests red; M2 `crop_left` +1 → 6 red; M3 drop the ActiveArea top from `crop_top`
→ 3 red; M4 negate `step_y` → 7 red; M5 f32 sink skips the crop → f32-parity test
red.

### Cost self-report

- **Tokens (total):** 4696185
- **Estimated USD:** ~3.60 (Opus 5.5 list rates; ~97% of tokens are cache reads)
- **Duration (minutes):** ~45
- **Source of the number:** harness report: the sum of this session's own
  transcript usage records (input + output + cache creation + cache read, deduped
  by message id; model claude-opus-5-5), taken just before the commit. `/cost` is
  not callable as a tool in a delegated session.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-043 covers the design; the build's affine
  stride sink is an implementation detail of it, pinned by the same tests.
- **Deviations from spec:** none material. Two noted, both documented in the code
  and HANDOFF-105:
  - Error precedence changed. A geometry error now comes before
    `NotAColorFilterArray` / `MissingColorCalibration`, mirroring `develop_into`,
    so a non-CFA plane with a bad ActiveArea reports `InvalidActiveArea`.
  - The out-of-range-ActiveArea test changed meaning from "runs" to "typed
    rejection". That is correct now that the colour path resolves geometry.
- **Follow-up work identified (non-blocking, none holds the spec):**
  - `SPEC-050/FU-1`: DEC-043's `affected_scope` omits `src/bin/irr.rs`,
    `README.md` and `CHANGELOG.md`, which the contract change touched. This is a
    traceability nit only.
  - `SPEC-050/FU-2`: the committed geometry fixture's plane crop origin is
    (4, 4), so its even/even parity is exercised only through the odd ActiveArea
    top. My differential covered odd plane-crop origins and a far-edge crop. Adding one
    such geometry to `geometry_fixture`'s sweep would make that permanent.
  - The build already names the ≈5% sink time cost: a row-level emit from the
    kernel could remove it if the kernel is ever reopened (out of scope under
    DEC-043). It is recorded in the ledger per DEC-041.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The brief named the build branch as `claude/confident-blackburn-4250b6`, but
   PR #152's head is `claude/spec-050-colour-crop-geometry`. `gh pr view`
   resolved it at once. "Regenerate the colour goldens" also had no stored golden
   file to regenerate, as the build noted.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-043 + DEC-040 + DEC-041 were sufficient.

3. **If you did this task again, what would you do differently?**
   — Lead with the old-vs-new differential (dump `main`'s whole-plane bytes, compare
   through `develop_into`'s source map), as I did here. It is the one check that
   proves "byte-identical to the OLD develop" directly instead of through a
   reference that was itself edited in the same PR (`reference_normalize` gained
   ActiveArea-origin awareness here).
