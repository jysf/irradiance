---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-006
  status: proposed
  activity: requirements
  priority: medium
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-23
shipped_at: null

closed_reason: null

value:
  thesis: >
    A Leica shooter with a colour body (Q2/Q3, M10/M11, SL2/SL3) wants the same
    true-linear monochrome the Monochrom bodies give. A deterministic, bit-exact,
    clip-aware mixer from a colour Bayer DNG to irradiance's monochrome u16 plane lets
    onyx's Monochrom app open colour files in its mono darkroom with no second RAW
    library and no per-image tuning loop.
  beneficiaries:
    - "onyx (the Monochrom app) — colour Leica files become developable in its mono darkroom, not just inspectable"
    - "The maintainer — mono renders of his colour Leica files with named filter looks"
    - "crates.io users of irradiance — a permissive, spec-derived Bayer-to-mono path no other Rust RAW crate offers"
  success_signals:
    - "onyx develops a colour Leica DNG to mono through the public mixer API"
    - "The mixer output is bit-exact against an analytic oracle and byte-identical across platforms (DEC-002)"
    - "A clipped channel never darkens or tints a highlight: the renormalisation oracle shows a monotone ramp through clipping"
    - "Five filter presets ship, each ledgered as an approximation in docs/provenance-ledger.md"
    - "Every existing develop output is unchanged (no DEVELOP_VERSION bump for existing paths)"
  risks_to_thesis:
    - "The mix-space choice (white-balanced camera RGB vs a linear working space) decides whether a preset means the same thing on every camera; the wrong call makes presets per-camera guesses"
    - "onyx may want free weights (a slider) more than presets, or may not need colour-to-mono before its Darkroom v1 ships"
    - "Without spectral sensitivity in a DNG, filter presets are approximations; a user who expects a true Wratten 25 response will be disappointed"
    - "The tier-B corpus has no colour Leica DNG (only Canon, Pixel and Nikon colour files); the bodies the thesis names are untested until one is added"
    - "Clip renormalisation can make a visible step where one channel clips and its neighbour does not; a smooth version may be needed and must stay deterministic"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap: []
---

# PROJ-006: Spectral monochrome mixer (Bayer → mono)

> **PROPOSED. Framed 2026-11-23.** The ID was reserved for this plan on 2026-11-21 when the
> DNG writer took PROJ-005 (`docs/future-plans.md`). Status stays `proposed` until jysf
> activates it.

## What This Project Is

A scene-linear monochrome formation for **colour** Bayer DNGs. irradiance already
develops a colour DNG to linear sRGB (demosaic, white balance, the colour matrix;
STAGE-007/008) and a Monochrom DNG to a monochrome `u16` plane (PROJ-001). This project
joins the two: a public primitive that takes a colour Bayer DNG to the same monochrome
plane shape the mono develop produces, by a weighted sum of the channels after white
balance, with fixed-point rational weights, deterministic clip-aware renormalisation,
and five named filter presets.

## Why Now

- **Unblocked.** The weights only mean something on white-balanced RGB, so the mixer
  waited on SPEC-033 (white balance + matrix) and SPEC-034 (illuminant interpolation).
  Both shipped. STAGE-027..029 then made the colour path refuse what it cannot honour,
  so a mixer built on it now inherits a hardened input contract.
- **A named consumer.** onyx's Monochrom app is Monochrom-first because the mono path was
  the finished one (its app design doc). Colour files open in its Inspect mode but cannot
  be developed. The mixer is the bridge that lets its darkroom take a colour Leica file.
- **The queue is clear.** irradiance 0.2.6 is published and STAGE-030 closed. PROJ-002 has
  only the ColorChecker oracle (STAGE-008, SPEC-035) open, which waits on a photograph.
  The writer (PROJ-005) runs in its own session and touches different files.

## Success Criteria

See `value.success_signals`. In short: a public mixer onyx calls, an analytic bit-exact
oracle, monotone highlights through clipping, five ledgered presets, and no change to any
existing output.

## Scope

### In scope
- **The mix-space decision, first.** Weights in white-balanced camera RGB (as
  `docs/future-plans.md` says: cheapest, white balance folds into the weights, but a
  weight set is per camera) **or** in a linear working space after the colour matrix
  (one weight set means the same on every camera, at the cost of the matrix step). Decided
  at STAGE-031 design as a DEC, with a probe on the corpus's colour files (Canon 5D Mark III,
  Pixel 3a/4a/7 Pro, Nikon D300S; no colour Leica yet, see Dependencies).
- **The mixer primitive.** Colour Bayer DNG to a monochrome `u16` plane with the mono
  develop's output dimensions (crop and orientation as `develop_color_into`), fixed-point
  rational weights (DEC-018), validate-before-allocate, panic-free, fuzzed, and a threaded
  variant byte-identical to the serial one (DEC-002).
- **Clip-aware renormalisation.** A channel at or above the white level is dropped and the
  remaining weights are renormalised to sum to one. Deterministic, per pixel, no spatial
  reconstruction.
- **Filter presets.** Yellow, Green, Orange, Red and Ortho (blue), each recorded in
  `docs/provenance-ledger.md` as an **approximation** with its derivation, because a DNG
  carries no spectral sensitivity.
- **An oracle that can go red.** Analytic fixtures (known RGB in, exact mono out) plus, if
  the linear-space option wins, an identity: luminance weights must reproduce the Y of
  `to_srgb_linear_into`'s output to within rounding.

### Explicitly out of scope
- Spatial highlight reconstruction (an unbounded tuning loop; PROJ-002 already fences it).
- Per-image interactive editing and tone (`crustyimg-lab`, or the caller's tone layer).
- Spectral sensitivity measurement or per-camera spectral profiles.
- X-Trans or other non-Bayer mosaics (refused as the colour path refuses them today).
- A release. The mixer is new public API; a release spec is a separate call, made when
  STAGE-031 ships and onyx says whether it needs crates.io or a path dependency.

### Undecided, considered at STAGE-031 design
- **An `irr` hook** that renders a mixed mono for eyeballing (the dev/oracle binary only,
  never a product surface). Worth it if it is the cheapest way to look at preset output on
  real files; not decided now.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-031 (not yet framed) — The mixer core: mix-space DEC, the public primitive (serial + threaded), fixed-point weights, clip-aware renormalisation, analytic oracle, fuzz target
- [ ] STAGE-032 (not yet framed) — Filter presets: five presets as ledgered approximations, an oracle per preset, and the per-camera question the STAGE-031 DEC leaves open

**Count:** 0 shipped / 0 active / 2 pending

## Questions for onyx

Asked before STAGE-031 is framed, because the answers shape the API:
1. Which colour bodies must the mixer cover first (Q2/Q3, M10/M11, SL2/SL3)?
2. Presets only, free weights (a slider), or both?
3. Should the output feed onyx's existing mono pipeline unchanged (same `u16` plane, same
   dimensions as `develop::output_dimensions`), or does it want the mixed plane before
   crop and orientation?
4. When does onyx need it, and does it need it on crates.io or is a path dependency fine?

## Dependencies

### Depends on
- PROJ-002 STAGE-007 (demosaic) and STAGE-008's SPEC-033/034 (white balance, matrix,
  illuminant interpolation): shipped.
- PROJ-002 STAGE-027..029: the colour path's refusals, which the mixer inherits.
- **A colour Leica DNG in the tier-B corpus** (Q2/Q3, M10/M11 or SL2/SL3). The corpus has
  none today. STAGE-031 can build and prove the mixer on the Canon/Pixel/Nikon files, but
  STAGE-032's presets should not be judged until a Leica colour file is in.
- External input (not a dependency): onyx's answers to the questions above.

### Enables
- onyx's Monochrom app developing colour Leica files in its mono darkroom.
- crustyimg black-and-white conversion from colour RAW.

## Project-Level Reflection

*Filled in at close.*

- **Did the thesis hold?** <yes | partly | no | too-early>
- **What changed between brief and reality?** <...>
- **What carries to the next project?** <...>
