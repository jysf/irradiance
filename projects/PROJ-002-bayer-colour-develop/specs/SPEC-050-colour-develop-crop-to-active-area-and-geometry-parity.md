---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-050
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-105 build (cold session; records its real model at handback)
  created_at: 2026-11-07

references:
  decisions: [DEC-043, DEC-040, DEC-019, DEC-038] # DEC-043 = crop the colour path to full geometry parity (NEW, this spec's design — sink-side scatter); DEC-040 = the ActiveArea-origin offset that DEFERRED this crop; DEC-019 = DefaultCropOrigin relative to ActiveArea; DEC-038 = the colour develop path.
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation, no-new-top-level-deps-without-decision, one-spec-per-pr, provenance-recorded-per-algorithm]
  related_specs: [SPEC-047, SPEC-049] # the colour-develop path (offset + fuse) this crops

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-049]             # builds on the fused colour develop path (SPEC-047 offset + SPEC-049 fuse, both shipped)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-012's <capability>". Optional; null is acceptable.
value_link: "Closes STAGE-012's colour-correctness goal: the colour path now crops and orients to the same geometry as the mono path and LibRaw, instead of emitting the masked border."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 15000000        # ~M-complexity, in line with SPEC-047/048/049 build+verify (sink-only change, but a contract move + golden regen)
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null           # orchestrator main-loop session spans many specs' design/ship cycles; a per-cycle split is not observable (DEC-013: a null here is honest, a guess is not)
      estimated_usd: null
      duration_minutes: 40
      notes: "Design of SPEC-050: DEC-043 (sink-side crop/orient scatter, memory shape preserved), HANDOFF-105 build, the build prompt, failing-test set, cycle advanced to build."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-050: colour develop crop to active area and geometry parity

## Context

The colour develop path still emits the **wrong geometry**. SPEC-047 (P0, DEC-040)
fixed the *colours* on an odd `ActiveArea` origin with a wrapping offset, but
deliberately deferred the **crop**: `develop_color_into` / `develop_color_f32_into`
output the whole `width×height` plane, **including the masked border outside
`ActiveArea`**, and apply neither `DefaultCrop` nor `Orientation`. The mono path
`develop_into` already resolves the full geometry — `ActiveArea` → `DefaultCrop`
→ `Orientation` (`resolve_geometry`, `output_dimensions`) — and LibRaw crops too,
so a colour develop and a mono develop of the same file come out at different
sizes and orientations, and the colour image carries optical-black border. This
is the external 0.2.0 staff review's **§3B** and the pyirradiance report's
"outputs the masked border, which LibRaw crops." Last correctness piece of
`STAGE-012` (after P0 SPEC-047, P1 SPEC-048, P2 SPEC-049).

## Goal

Make the colour path crop and orient to the **same geometry as `develop_into`**:
resolve `ActiveArea` → `DefaultCrop` → `Orientation` and produce output at
`output_dimensions(sensor)`, dropping the masked border. This changes
`develop_color_into`'s output **dimensions** (a deliberate contract change DEC-040
deferred), so bump `DEVELOP_VERSION` and regenerate the colour goldens; the
`develop_color_f32_into` equality and the SPEC-047 colour correctness must hold.

## Inputs

- **Files to read:** `src/color.rs` (`develop_color_into` ~265 /
  `develop_color_f32_into` ~332 — the sink closure that writes `dst_triples[pixel]`
  is what this spec changes; `pixel_count`), `src/demosaic.rs`
  (`develop_color_fused` ~458 — the whole-plane traversal that calls
  `emit(pixel, r, g, b)`; **do not change it**, the crop happens in the sink),
  `src/develop.rs` (`resolve_geometry` ~135, `output_dimensions` ~230, and
  `crop_source_coords` ~249 — the forward output→crop map whose **inverse** the
  sink needs; and `crop_and_orient_from_active_into` ~613 for how the mono gather
  reads it), `src/ifd.rs` (`ActiveArea`, `Sensor`, `active_area_origin`).
- **Reference only:** `pyirradiance/notes/active_area_origin_repro.py` + the
  report; LibRaw is the geometry yardstick (never linked).
- **Prior decisions:** `DEC-040` (the offset that deferred this), `DEC-019`
  (`DefaultCropOrigin` relative to `ActiveArea`), `DEC-038` (the colour path).

## Outputs

- **Files modified:** `src/color.rs` — the sink scatters each developed pixel to
  its `DefaultCrop` + `Orientation` output slot (dropping the masked border and the
  non-crop margin), so `dst` is `output_dimensions(sensor)·3`; the fused kernel in
  `src/demosaic.rs` is unchanged. `src/develop.rs` — the new
  `output_coords_from_crop` inverse helper. `src/lib.rs` — `DEVELOP_VERSION` bump.
- **Output contract change:** `develop_color_into` / `develop_color_f32_into`
  expect `dst` sized to `output_dimensions(sensor)` (× 3), not `width×height×3`.
  State the new contract in the doc comments; callers size `dst` from the public
  `output_dimensions`. (DEC-040 named this the contract-changing geometry.)
- **New decision:** `DEC-043` — crop the colour path to full `DefaultCrop` /
  `Orientation` parity with `develop_into`. Written at this spec's **design**.
- **Goldens:** regenerate the colour goldens (output now cropped/oriented), with
  the reason in the commit.

## Acceptance Criteria

- [ ] `develop_color_into` output **dimensions equal `output_dimensions(sensor)`**
  (the resolved `DefaultCrop`, swapped for a 90° `Orientation`), for files with
  non-trivial `ActiveArea` / `DefaultCrop` / `Orientation`.
- [ ] The masked border outside `ActiveArea` is **not** in the output.
- [ ] On the active-area pixels, colour is unchanged from SPEC-047 (grey stays
  grey; the ActiveArea-origin phase still holds) — the crop moves the frame, not
  the colours.
- [ ] For a fixture with a `DefaultCrop` + `Orientation`, the colour output's
  geometry **matches the mono `develop_into`** path (same crop rectangle, same
  orientation).
- [ ] `develop_color_f32_into` still equals `to_linear_f32_into(develop_color_into(..))`
  exactly, at the new dimensions.
- [ ] `DEVELOP_VERSION` bumped; no panic for any geometry (reuse
  `resolve_geometry`'s typed rejections); determinism; no new dependency.

## Failing Tests

*Formalised at design (`DEC-043`); the build writes them first, in
`tests/color_oracle.rs` and `src/develop.rs`'s unit tests. Each must fail against
today's whole-plane colour output before the change.*

- `"colour_output_has_output_dimensions_geometry"` — a `fixture_sensor` with a
  non-trivial `ActiveArea` + `DefaultCrop` + `Orientation`; `develop_color_into`
  called with `dst` sized to `output_dimensions(sensor)·3` succeeds, and the
  masked border is gone (a known border pixel no longer appears at [0,0]).
- `"colour_geometry_matches_the_mono_path"` — for the same fixture the colour
  output's dimensions equal `output_dimensions(sensor)`, i.e. the same crop
  rectangle and orientation `develop_into` resolves.
- `"colour_crop_pixels_equal_the_whole_plane_develop_at_their_source"` — the
  *values* the crop keeps are byte-identical to today's whole-plane
  `develop_color_into` at the corresponding `(DefaultCrop + inverse-orient)` source
  pixel: the crop moves the frame, not the colours (so `SPEC-047`'s
  grey-stays-grey / odd-origin colour still holds on every kept pixel).
- `"output_coords_from_crop_is_the_inverse_of_crop_source_coords"` — for all eight
  orientations over a small grid, `crop_source_coords(orientation,
  output_coords_from_crop(orientation, cx, cy, cw, ch), cw, ch) == (cx, cy)` (the
  round-trip that pins the new inverse helper).
- `"develop_color_f32_still_equals_to_linear_at_new_dims"` — at the new output
  dimensions, `develop_color_f32_into == to_linear_f32_into(develop_color_into)`.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The colour maths (`DEC-038`), the demosaic kernel, and the SPEC-048/049 fast
  path / fuse — unchanged; this spec moves geometry, not pixels' colour.
- The mono `develop_into` — already correct; reuse its helpers, don't change it.
- `WarpRectilinear` / `OpcodeList` application on the colour path — the mono path
  has it; folding it into colour is its own later spec if wanted.
- P3 threading — separate STAGE-012 spec.
- `BlackLevelRepeatDim` beyond the 2×2 the level slot already handles — note it if
  relevant, don't expand here.

## Notes for the Implementer

- **Crop at the sink, don't re-scope the kernel (`DEC-043`).** `develop_color_fused`
  already develops every plane pixel with `SPEC-047`'s correct colours; leave it
  byte-for-byte. The change is in `src/color.rs`'s sink closure: map each plane
  pixel `(x, y)` to its output slot — if it's inside the `ActiveArea` *and* the
  `DefaultCrop` rectangle, scatter its matrixed triple to `dst` at `DefaultCrop`
  offset + the **inverse of `crop_source_coords`**; otherwise drop it. This keeps
  the developed *values* identical to `SPEC-047` (the criterion) and keeps
  `SPEC-049`'s memory shape — **do not introduce an active-area or full-plane RGB
  buffer** (that re-adds the ~140 MB SPEC-049 removed); scatter straight into `dst`.
- **Add the inverse helper in `src/develop.rs`** (`output_coords_from_crop`, the
  source→output inverse of `crop_source_coords`) and pin it with the round-trip
  test. Orientations 1–4 keep `crop_w×crop_h`; 5–8 transpose.
- **The contract change is the point** — `dst` is now `output_dimensions`-sized.
  Fix the length checks + `Error::RgbBufferWrongLength` expectations, document the
  new contract in the doc comments, and keep `develop_color_f32_into` in lockstep
  (scatter the quantised `u16`, then `/65535.0` — not the raw `f64`).
- Keep zero-dependency, no-panic (reuse `resolve_geometry`'s typed errors; scatter
  through `.get_mut`), determinism. LibRaw/rawpy are the pyirradiance yardstick —
  reference only.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
