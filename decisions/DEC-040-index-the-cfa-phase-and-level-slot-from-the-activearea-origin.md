---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-040
  type: decision
  confidence: 0.85
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

created_at: 2026-11-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/color.rs
  - src/demosaic.rs
  - src/lib.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-047-colour-path-honours-the-activearea-origin.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md

tags:
  - colour
  - demosaic
  - bayer
  - active-area
  - black-level
  - develop-version
  - stage-012
  - spec-047
---

# DEC-040: Index the CFA phase and level slot from the ActiveArea origin

## Decision

In the colour develop path, compute each pixel's **CFA colour** (`BayerPhase`,
`src/demosaic.rs`) and its **BlackLevel/WhiteLevel 2×2 slot** (`normalize_mosaic`,
`src/color.rs`) from coordinates taken **relative to the ActiveArea origin**, not
the plane's top-left. Realize it with the **offset** form

```
row_odd = (y.wrapping_sub(active_top) & 1) == 1
col_odd = (x.wrapping_sub(active_left) & 1) == 1
```

in both functions, where `(active_top, active_left)` is the `ActiveArea` origin
(`(0, 0)` when the tag is absent, matching `resolve_geometry`). **Do not crop**
the colour path for this fix, and **do not** change `develop_color_into`'s
input/output lengths. Bump `DEVELOP_VERSION` **2 → 3**.

The shared origin lookup is a small helper — `active_area_origin(sensor) -> (u32,
u32)` — used by both `normalize_mosaic` and `demosaic_into` (and available to
`develop.rs`), so the two call sites cannot drift.

## Why the offset, not a crop

The pyirradiance rawpy/LibRaw benchmark and the external 0.2.0 staff review (§3B)
both offered two fixes: index relative to the origin (offset), or crop to the
ActiveArea before demosaic. For a **ship-first, standalone P0** the offset is the
smaller, safer change:

- **No public-contract change.** `develop_color_into` keeps `src.len() ==
  width·height` and `dst.len() == width·height·3`. Cropping would change the
  output **dimensions** to the active-area size, a semver-visible contract change
  every caller (pyirradiance included) would have to track, and would need a new
  active-area dimensions accessor distinct from the existing public
  `output_dimensions` (which is DefaultCrop+Orientation, a different rectangle).
- **Even-origin output is byte-identical.** `y.wrapping_sub(even) & 1 == y & 1`,
  so the Canon 5D3 `(80, 122)` and Nikon D300S `(0, 0)` corpus goldens do not
  move. Only odd-origin files change — and they change from wrong to right.
- **`wrapping_sub` is panic-free and parity-correct everywhere,** including the
  masked-border rows above/left of the origin (`y < active_top`): the low bit of
  a wrapping subtraction is still the true parity.

The crop form is the right *eventual* shape (it also removes the masked border
and would give the colour path full geometry parity with the mono `develop_into`),
but that is a larger, separate change with its own output-contract and golden
churn. It is deferred to its own STAGE-012 spec (the colour-geometry crop/orient
follow-up), out of P0.

## Scope and non-goals

- **In:** ActiveArea-origin-relative phase and level-slot indexing in the colour
  path; the `DEVELOP_VERSION` bump; the shared `active_area_origin` helper.
- **Out (own follow-up spec):** cropping the colour output to the active area,
  removing the masked border, and full `DefaultCrop`/`Orientation` parity with
  `develop_into`. Also out: the exact-LibRaw demosaic behaviour at the
  active-area boundary (whether edge pixels reflect within the active area or
  read the masked border) — a crop-time concern.
- The colour math itself (`DEC-038`, the ColorMatrix/Bradford path) is untouched;
  this decision only moves *which* 2×2 slot and *which* CFA colour each pixel is
  assigned.

## Consequences

- `DEVELOP_VERSION` 3 signals the output change for odd-origin (and any
  per-slot-black-level) files; affected goldens, if any, are regenerated with the
  reason in the commit. Even-origin fixtures are unaffected.
- A new typed error is **not** needed: the offset never fails (no crop, no
  validation beyond the existing `ActiveArea` parse). A malformed ActiveArea is
  still only rejected on the paths that crop (the mono `resolve_geometry`), which
  this decision does not touch.
- Acceptance is the grey-stays-grey red-proof in SPEC-047: a flat-grey subject
  stays grey (per-channel spread < 1e-3) for all four origin parities on all four
  Bayer phases, and the pre-fix plane-relative computation is shown red (magenta)
  for the odd parities.
