---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-050
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-040, DEC-019, DEC-038] # DEC-040 = the ActiveArea-origin offset that DEFERRED this crop; DEC-019 = DefaultCropOrigin relative to ActiveArea; DEC-038 = the colour develop path. DEC-043 (crop the colour path to full geometry parity) is NEW, formalized at this spec's design.
  constraints: []                  # design fills exact ids: no-panic, determinism, no-new-deps
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
  tokens_estimate: null
  sessions: []
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

- **Files to read:** `src/color.rs` (`develop_color_into`, `develop_color_f32_into`,
  the fused pipeline from SPEC-049, `pixel_count`), `src/develop.rs`
  (`resolve_geometry`, `output_dimensions`, and the crop/orient helpers
  `crop_orient_normalize_into` / `crop_and_orient_from_active_into` — how the mono
  path crops+orients; reuse this, don't reinvent it), `src/ifd.rs` (`ActiveArea`,
  `Sensor`).
- **Reference only:** `pyirradiance/notes/active_area_origin_repro.py` + the
  report; LibRaw is the geometry yardstick (never linked).
- **Prior decisions:** `DEC-040` (the offset that deferred this), `DEC-019`
  (`DefaultCropOrigin` relative to `ActiveArea`), `DEC-038` (the colour path).

## Outputs

- **Files modified:** `src/color.rs` — the colour path crops to the active area,
  demosaics within it (reflecting at the active-area edge, not across the masked
  border), and applies `DefaultCrop` + `Orientation`, producing
  `output_dimensions(sensor)`; `src/lib.rs` — `DEVELOP_VERSION` bump.
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

*Sketched at frame; the **design** cycle writes them in full (and formalises
`DEC-043`), in `tests/color_oracle.rs` / `tests/develop*`.*

- `"colour_output_has_output_dimensions_geometry"` — a `fixture_sensor` with a
  non-trivial `ActiveArea` + `DefaultCrop` + `Orientation`; `develop_color_into`'s
  output dims equal `output_dimensions(sensor)`, and the border is gone.
- `"colour_geometry_matches_the_mono_path"` — the crop rectangle and orientation
  match `develop_into`'s for the same fixture.
- `"active_area_colour_is_unchanged_after_crop"` — the SPEC-047 grey-stays-grey /
  odd-origin colour still holds on the cropped pixels.
- `"develop_color_f32_still_equals_to_linear_at_new_dims"`.

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

- **Reuse `develop.rs`'s geometry, don't reinvent it.** `resolve_geometry` already
  yields the `ActiveArea` window, the `DefaultCrop` rectangle (relative to the
  active area, `DEC-019`) and the validated `Orientation`; `output_dimensions`
  gives the final size; `crop_and_orient_from_active_into` / the mono crop/orient
  helpers already do the extraction. The colour path should normalise + demosaic
  the **active-area window** (so demosaic reflects at the active-area edge, which
  is the whole reason P0 chose offset-then-defer-crop), then `DefaultCrop` +
  `Orientation` into `dst`.
- **The contract change is the point** — `dst` is now `output_dimensions`-sized.
  Do not keep full-plane dims; document it and keep `develop_color_f32_into` in
  lockstep.
- Keep zero-dependency, no-panic (reuse `resolve_geometry`'s typed errors),
  determinism. LibRaw/rawpy are the pyirradiance yardstick — reference only.

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
