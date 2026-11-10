---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-043
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-07
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/color.rs
  - src/develop.rs
  - src/lib.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-050-colour-develop-crop-to-active-area-and-geometry-parity.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md
  - docs/provenance-ledger.md

tags:
  - colour
  - geometry
  - active-area
  - default-crop
  - orientation
  - develop-version
  - stage-012
  - spec-050
---

# DEC-043: Crop the colour path to full geometry parity with develop_into

## Decision

Make `develop_color_into` / `develop_color_f32_into` resolve and apply the full
DNG geometry — `ActiveArea` → `DefaultCrop` → `Orientation` — producing output at
`output_dimensions(sensor)`, the same as the mono `develop_into` and LibRaw, and
dropping the masked border. This is the crop `DEC-040` deferred; `SPEC-047`'s
ActiveArea-origin offset stays and handles the colours.

**Reuse the mono path's geometry, don't reinvent it:** `resolve_geometry`
(the `ActiveArea` window, the `DefaultCrop` rectangle relative to it per `DEC-019`,
the validated `Orientation`) and `output_dimensions` (the final size, dimensions
swapped for a 90° orientation) already exist in `src/develop.rs`.

## The key move: crop at the SINK, don't re-scope the kernel

`develop_color_fused` (`src/demosaic.rs`, `SPEC-049`) walks the **whole plane** in
raster order and calls `emit(pixel = y·width + x, r, g, b)` with the camera-RGB
triple; today the sink in `src/color.rs` writes `dst_triples[pixel]`, so `dst` is
`width·height·3` and carries the masked border. The colours are already correct
(`SPEC-047`'s ActiveArea-origin phase/level offset is inside the kernel).

**Crop by changing the sink, not the kernel.** Leave `develop_color_fused`
byte-for-byte: it still develops every plane pixel with the same neighbours in the
same order, so **every developed value is identical to `SPEC-047`'s** — this spec
*moves the frame, not the colours* (the acceptance criterion). The new sink maps
each plane pixel `(x, y)` to its place in the output: if `(x, y)` lies inside the
`ActiveArea` **and** inside the `DefaultCrop` rectangle, write its matrixed triple
to `dst` at the output position given by `DefaultCrop` offset + the **inverse of
`crop_source_coords`** (the orientation map); otherwise drop it. `dst` is sized to
`output_dimensions`. The masked border and the non-crop active pixels are still
developed (a thin, cheap margin — their green feeds neighbours) but never written.

**Why this over "develop the active window and reflect at its edge":** the active
pixels' values would then change within 2 px of the active-area edge (an
active-edge reflect instead of the plane-edge reflect `SPEC-047` emits), which the
"colour unchanged from `SPEC-047`" criterion forbids — and the `DefaultCrop`
rectangle insets from the active area, so for real files the dropped margin's
neighbours never reach an output pixel anyway. Sink-cropping keeps the fused kernel
and its bit-identity guarantee untouched and is the smaller, safer change.

## Memory shape is preserved (SPEC-049's win stays)

Scratch stays exactly `SPEC-049`'s: the `width·height` green plane + the
row-sized rolling normalise ring. **No active-area or full-plane RGB intermediate
is introduced** (that would re-add the ~140 MB `camera_rgb` SPEC-049 removed). The
scatter writes straight into the caller's `dst`, which now *shrinks* to the output
size. Peak RSS must not regress from SPEC-049.

## The inverse-orientation helper

`crop_source_coords(orientation, out_x, out_y, crop_w, crop_h) → (crop_x, crop_y)`
is the forward (output→source) map the mono gather uses. The sink scatters
source→output, so add its inverse in `src/develop.rs` — e.g.
`output_coords_from_crop(orientation, crop_x, crop_y, crop_w, crop_h) → (out_x,
out_y)` — and pin it with a **round-trip test**: for all eight orientations over a
small grid, `crop_source_coords(orientation, output_coords_from_crop(..)) ==
(crop_x, crop_y)`. (Orientations 1–4 keep `crop_w×crop_h`; 5–8 transpose to
`crop_h×crop_w`; the flips are involutions, the two 90° rotations swap.)

## The contract change

`develop_color_into` / `develop_color_f32_into` now expect `dst` sized to
`output_dimensions(sensor)` (× 3), not `width·height·3` — the deliberate,
semver-visible change `DEC-040` deferred. Document it in the doc comments and fix
the length checks/errors; callers size `dst` from the public `output_dimensions`,
exactly as `develop_into`'s callers already do. Bump `DEVELOP_VERSION` (3 → 4) and
regenerate the colour goldens (the output geometry moves for any file with a
non-trivial `DefaultCrop`/`Orientation`; a trivial-geometry fixture — full-plane
`ActiveArea`, no crop, orientation 1 — stays byte-identical, a useful pin).

## f32 parity

`develop_color_f32_into` stays equal to `to_linear_f32_into(develop_color_into(..))`
at the new dimensions. Crop/orient is a pixel rearrangement with no arithmetic, so
it commutes with the `u16`-then-`/65535.0` of the float path: scatter the quantised
`u16`, then divide — do not reorder so the division lands before the quantise.

## Scope and non-goals

- **In:** the ActiveArea crop, `DefaultCrop`, and `Orientation` on the colour path,
  at output dimensions, via a sink-side scatter; the inverse-orientation helper;
  the `DEVELOP_VERSION` bump.
- **Out:** `WarpRectilinear` / `OpcodeList` on the colour path (the mono path has
  it; its own later spec); the colour maths (`DEC-038`); the demosaic kernel and
  the SPEC-048/049 fast path/fuse (reused byte-for-byte, not changed); the mono
  `develop_into` (reused, not changed); P3 threading; `unsafe`, SIMD, new deps.
- Panic-free for any geometry — reuse `resolve_geometry`'s typed rejections
  (`InvalidActiveArea`, `InvalidDefaultCrop`, `UnsupportedOrientation`); the sink's
  scatter writes through `.get_mut`.
