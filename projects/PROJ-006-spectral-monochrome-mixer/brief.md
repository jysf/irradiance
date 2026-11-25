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
    - "Five filter presets ship as named weight sets, each ledgered as an approximation in docs/provenance-ledger.md"
    - "A crates.io release onyx can pin exactly, with semver-check reporting additions only"
    - "Every existing develop output is unchanged (no DEVELOP_VERSION bump for existing paths)"
  risks_to_thesis:
    - "Clip detection must happen on raw camera channels, but a linear working space mixes after the matrix, where a clipped channel has spread into all three; the clip rule may be harder than renormalisation"
    - "onyx puts PROJ-006 on its later horizon, after its Darkroom v1, so real use may come well after this ships"
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

> **PROPOSED. Framed 2026-11-23 (PR 324); re-framed the same day from onyx's answers,
> with STAGE-031 framed.** The ID was reserved for this plan on 2026-11-21 when the DNG
> writer took PROJ-005 (`docs/future-plans.md`). Status stays `proposed` until jysf
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
- **The working space, first.** Expected: a **linear working space** after the colour
  matrix, so one weight set means the same on every body (onyx prefers it; jysf leans the
  same way, 2026-11-23). Confirmed at SPEC-091 design as a DEC, with a test run on the
  held colour files (Canon 5D Mark III, Pixel 3a/4a/7 Pro, Nikon D300S; no colour Leica
  yet, see Dependencies). The alternative, white-balanced camera RGB, stays on the table
  only if the test run says so.
- **A public white-balanced linear RGB step** in front of the mixer, at sensor geometry
  (onyx's colour reference view, its own candidate mixes, its quality study).
- **The mixer, in two output shapes.** At sensor geometry (before crop, orientation and
  warp) for onyx's GPU viewport, carrying its own black and white levels; and developed,
  as an opt-in mix option on `develop_with_into` matching `output_dimensions_with`, so the
  mix goes through the existing geometry and onyx's warp parity check carries over.
  Weights-only API with rational fixed-point weights (DEC-018); validate-before-allocate,
  panic-free, fuzzed, threaded variants byte-identical to serial (DEC-002).
- **Clipping detected on the raw camera channels**, before white balance and any matrix
  (onyx's point), with a deterministic per-pixel rule that keeps highlights monotone. No
  spatial reconstruction.
- **Filter presets as named weight sets.** Yellow, Green, Orange, Red and Ortho (blue),
  each recorded in `docs/provenance-ledger.md` as an **approximation**, because a DNG
  carries no spectral sensitivity. onyx stores resolved weights, never preset names, so a
  later table change cannot alter an existing print.
- **A crates.io release**, additions only: no change to the output bits of any existing
  `develop_into`, `develop_with_into`, `develop_color_into`, `demosaic_into` or
  `unpack_into` call (onyx takes only exact-pinned crates.io releases, its DEC-031/072).
- **An oracle that can go red.** Analytic fixtures (known RGB in, exact mono out) plus, if
  the linear-space option wins, an identity: luminance weights must reproduce the Y of
  `to_srgb_linear_into`'s output to within rounding.

### Explicitly out of scope
- Spatial highlight reconstruction (an unbounded tuning loop; PROJ-002 already fences it).
- Per-image interactive editing and tone (`crustyimg-lab`, or the caller's tone layer).
- Spectral sensitivity measurement or per-camera spectral profiles.
- X-Trans or other non-Bayer mosaics (refused as the colour path refuses them today).
- Sony, Fuji and Hasselblad bodies (onyx: later).

### Decided at SPEC-092 design (2026-11-25)
- **No `irr` hook** for now: SPEC-093's developed mix is the natural thing to look at, and
  SPEC-092 pins the plane with golden digests. The fixed-point row, `τ` and the weights
  rule are `DEC-083`.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-031 (proposed) — Colour to mono: SPEC-091 the white-balanced linear RGB step + working-space DEC, SPEC-092 the mixer at sensor geometry + clip rule, SPEC-093 the developed mix on `develop_with_into` (outlines, chained)
- [ ] (not yet framed) — Filter presets: five named weight sets ledgered as approximations, Q3 and M11 colour fixtures, judged on a colour Leica file
- [ ] (not yet framed) — The release: publish the mixer to crates.io, additions only, semver clean

**Count:** 0 shipped / 0 active / 3 pending

## onyx's answers (2026-11-23)

The four framing questions went to onyx after PR 324. Its answers set this brief:
1. **Bodies:** Q3 and M11 first, then Q2 and M10, then SL2/SL3; Sony, Fuji and Hasselblad
   later. Confirmed by jysf 2026-11-23. Neither side holds a colour Leica frame yet.
2. **Controls:** both, but the API takes weights only; presets are named weight sets.
   onyx also asked for the white-balanced linear RGB plane as its own public step.
3. **Output shape:** two outputs, sensor geometry (with its own levels) for the viewport
   and developed (a mix option on `develop_with_into`) for export. Detect clipping on raw
   camera channels.
4. **Timing and delivery:** no rush (onyx's later horizon, after its Darkroom v1); must be
   an exact-pinnable crates.io release, additions only, no change to existing output bits.

onyx also prefers a linear working space and asked that the matrix step stay
fixed-point. Today's matrix is f64, rounded per operation and CI-proven byte-identical
across platforms; SPEC-091's design decides the new path's arithmetic, and the existing
colour output may not change. onyx's own request 7 (`docs/upstream/irradiance-requests.md`
in its repo) still describes a camera-RGB path and needs updating on its side.

## onyx's reply to the design (2026-11-25)

onyx accepted DEC-079 (linear sRGB, weights folded into the matrix), which it cites as
`irradiance/DEC-079`. It will use the unclamped f32 plane for candidate mixes and the
quality study; the u16 plane only feeds its colour reference view. Fixed-point in the
mixer only is enough, provided the row is built bit-identically everywhere, pinned per
test file and exposed. It accepted the fade of negative weights near clip in place of
drop-and-renormalise, on three conditions: no parameter, bit-identical to no fade when no
weight is negative, and the clip threshold documented exactly. It does not need a clip
mask; its overlays come from the raw CFA plane. New asks: the f32 plane bit-identical on
aarch64 and x86_64 with a golden digest, and a row-range (band) form for iPad memory
(about 720 MB for a 60 MP f32 plane). SPEC-091 took the plane's asks on 2026-11-25; the
mixer's are recorded in SPEC-092's outline. onyx will look for Q3 and M11 colour DNGs
before STAGE-032. Who supplies them is still open.

## Dependencies

### Depends on
- PROJ-002 STAGE-007 (demosaic) and STAGE-008's SPEC-033/034 (white balance, matrix,
  illuminant interpolation): shipped.
- PROJ-002 STAGE-027..029: the colour path's refusals, which the mixer inherits.
- **A colour Leica DNG in the tier-B corpus** (Q2/Q3, M10/M11 or SL2/SL3). The corpus has
  none today. STAGE-031 can build and prove the mixer on the Canon/Pixel/Nikon files, but
  STAGE-032's presets should not be judged until a Leica colour file is in.
- External input (not a dependency): onyx's answers, above.

### Enables
- onyx's Monochrom app developing colour Leica files in its mono darkroom.
- crustyimg black-and-white conversion from colour RAW.

## Project-Level Reflection

*Filled in at close.*

- **Did the thesis hold?** <yes | partly | no | too-early>
- **What changed between brief and reality?** <...>
- **What carries to the next project?** <...>
