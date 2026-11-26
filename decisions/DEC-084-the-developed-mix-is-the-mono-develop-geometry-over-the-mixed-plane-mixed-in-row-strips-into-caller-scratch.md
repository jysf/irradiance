---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-084
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-006
repo:
  id: irradiance

created_at: 2026-11-26
supersedes: null
superseded_by: null
status: accepted                   # onyx's reply 2026-11-26, accepted by jysf
deciders: [claude, jysf]

affected_scope:
  - src/develop.rs
  - src/color.rs
  - src/demosaic.rs
  - src/opcode.rs
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md

tags:
  - mixer
  - develop-options
  - geometry
  - scratch
  - stage-031
---

# DEC-084: the developed mix is the mono develop's geometry over the mixed plane, mixed in row strips into caller scratch

## Decision

`DevelopOptions` gains one field, `mono_mix: Option<[Rational; 3]>`, default `None`.
With `Some(weights)`, `develop_with_into`, `develop_with_scratch_into`,
`output_dimensions_with` and `scratch_len` develop a colour Bayer DNG to a mixed
monochrome image at the mono develop's geometry.

1. **The rule (parity).** The developed mix is, bit for bit, what the mono develop
   writes for a plane that is already normalised: SPEC-092's mixed plane
   (`mono_mix_plane_into(sensor, src, weights)`) read at black `MONO_MIX_BLACK` and
   white `MONO_MIX_WHITE`, with no further level, `GainMap` or `FixBadPixelsConstant`
   step, then `OpcodeList3`'s first `WarpRectilinear` lerped to `warp_alpha` over the
   `ActiveArea`, then the framing and the orientation, all under the same options. So
   the mix replaces stage 2 (levels and `GainMap`s) and nothing after it changes. A
   consumer that draws the mixed plane at sensor geometry and warps, crops and rotates
   it as it does a mono sensor plane (onyx's GPU viewport, its DEC-062 parity check)
   sees the same pixels the develop writes.
2. **The weights** are `DEC-083` §1's (non-negative rationals, positive denominators,
   summing to exactly one) and the arithmetic is `DEC-083`'s row and kernel, unchanged.
3. **Opcodes: a new `DevelopPath::MonoMix`.** The developed mix applies every `GainMap`
   in `OpcodeList2` (inside the mix, as the colour path does) and the first
   `WarpRectilinear` in `OpcodeList3` whatever its flags (as the mono develop does).
   It applies nothing from `OpcodeList1`: a mandatory `FixBadPixelsConstant` is
   `UnsupportedRequiredOpcode { list: 1, id: 4 }`, and an optional one is skipped
   unparsed. Its malformed-tag sets are the colour path's (levels and geometry, CFA,
   colour).
4. **The other options.** `framing`, `apply_orientation` and `warp_alpha` mean what they
   mean for the mono develop, including `DEC-059` §3's bounds rule under
   `FullActiveArea`. `black_levels` is not read: the mix's levels are the colour path's
   (`DEC-068` §2, `DEC-070`), which already index `BlackLevelRepeatDim`.
5. **Where the work happens.** The mixer runs through the banded fused traversal
   (`develop_rows_banded`, `DEC-044`) in strips of 512 rows, each strip's green plane
   and ring cache in caller scratch, and each pixel goes straight to its place:
   - **No non-identity warp:** strips cover the framed rows, and each framed pixel is
     scattered to its output slot by the colour develop's crop and orientation map
     (`CropScatter`, `DEC-043`), built from the options-applied geometry. There is no
     full-frame intermediate.
   - **A non-identity warp:** strips cover the `ActiveArea` rows, and each active pixel
     is written into an `ActiveArea`-sized buffer `A1`. Then the mono develop's own
     passes run unchanged: `apply_warp_into` from `A1` into `A2`, and
     `crop_and_orient_from_active_into` from `A2` into `dst`.
6. **Scratch.** With `w` the plane width, `h` its height, `A` the active area's sample
   count and `S = 512`, the strip working memory is `T = w·(min(h, S + 7) + 8)`
   samples (the band's green rows with its halo and even alignment, plus the
   eight-row ring). `scratch_len` returns `T` without a warp and `A + max(A, T)` with
   one (`T` lives in `A2`'s place while the strips run). `develop_with_scratch_into`
   allocates nothing under the mix, as `DEC-060` §2 requires of it, and
   `develop_with_into` makes its one allocation of exactly `scratch_len` samples. The
   layout stays private (`DEC-060` §3).
7. **Refusal order** (nothing written to `dst` or scratch before the passes):
   `InvalidWarpAlpha`, `InvalidMixWeights`, required processing on
   `DevelopPath::MonoMix`, `SourcePlaneWrongLength`, the geometry, the colour levels
   then the `GainMap`s, `DevelopBufferWrongLength`, under `FullActiveArea` the
   `OpcodeList3` parse and bounds, under `StandardCrop` the `OpcodeList3` parse, the
   CFA, `MissingColorCalibration`, `MixRowOutOfRange`, then (scratch form)
   `DevelopScratchWrongLength`. `output_dimensions_with` checks `InvalidWarpAlpha`,
   `InvalidMixWeights`, then the geometry; `scratch_len` adds the `OpcodeList3` parse.
   Neither reads the CFA, the levels or the calibration.
8. **Versions.** Neither `DEVELOP_VERSION` nor `MONO_MIX_VERSION` changes: no existing
   output moves. The developed mix is fixed by (`DEVELOP_VERSION`, `MONO_MIX_VERSION`,
   options, weights included), so `DEC-058` §5's cache key gains `MONO_MIX_VERSION`
   when `mono_mix` is set. A change to the mixer's arithmetic bumps
   `MONO_MIX_VERSION`; a change to where the mix sits in the develop, or to anything
   else this rule fixes, bumps `DEVELOP_VERSION`.

## Context

STAGE-031's third output (onyx's answers, 2026-11-23): the developed mono for export,
as an option on `develop_with_into`, so onyx's export goes through the same geometry
and resampling as its viewport and its GPU-to-develop warp parity check carries over.
SPEC-092 shipped the mixed plane at sensor geometry with constant levels 0 and 65535
for exactly that viewport.

A design-time probe on 2026-11-26 (a throwaway example, not committed; its source is in
the PROJ-006 session notes) ran the five held colour files on the release build:

| File | Whole plane | Strips of 256 | Strips of 512 | Non-identity warp |
|---|---|---|---|---|
| Canon 5D Mark III | 398 ms | 1.057× | 1.035× | none |
| Pixel 3a | 413 ms | 1.080× | 1.063× | optional, radial |
| Pixel 4a | 419 ms | 1.117× | 1.056× | optional, identity (skipped) |
| Pixel 7 Pro | 398 ms | 1.091× | 1.047× | optional, radial; Orientation 3 |
| Nikon D300S | 231 ms | 0.988× | 0.986× | none |

The strip timings went through the public band entry, which also clones the `Sensor`,
re-resolves the levels and allocates per call, so they bound the design's overhead from
above. Every strip partition equalled the whole plane. The rule-1 oracle (the mixed plane
developed as a mono sensor at levels 0 and 65535) ran on every file under the default
options, `warp_alpha = 0`, `FullActiveArea` and `apply_orientation = false`; the Pixel 3a's
warp fails `DEC-059` §3 under `FullActiveArea` (scale up to 1.0015), as it would for a
mono develop.

The probe also read the held Q2 Monochrom's opcode lists: its `FixBadPixelsConstant`
(`OpcodeList1`) and `WarpRectilinear` (`OpcodeList3`) both have flags 0, mandatory.
No colour Leica file is held, so whether a Q3 or M11 colour DNG carries the same
mandatory `FixBadPixelsConstant` is not known. If it does, every colour entry, the
developed mix included, refuses it (item 3).

## Alternatives Considered

- **Mix after the warp** (warp the RGB, then mix). It would resample three channels
  and need an RGB intermediate, and the result would not equal the viewport's (which
  warps the mixed plane), so onyx's parity check would break. Rejected.
- **A whole-plane mix into scratch** (`mono_mix_plane_into`, then the mono passes). The
  simplest composition, but it needs the `w·h` mixed plane plus the traversal's `w·h`
  green plane: about 240 MB of `u16` at 60 MP before the warp's two active buffers.
  Strips cost 1–6% more time on the held files and keep the strip memory near
  `512·w` samples (about 10 MB at 9,600 columns).
- **No-warp through an `A`-sized buffer and the mono gather** (`crop_and_orient_from_active_into`).
  The mono path's own code for the last step, but `A` more scratch (about 120 MB at
  60 MP) where the scatter needs none. The scatter is the colour develop's
  `DEC-043` map, its inverse relation to the mono gather is pinned by
  `output_coords_from_crop_is_the_inverse_of_crop_source_coords`, and SPEC-093's
  parity tests compare the two on all eight orientations and both framings.
- **A separate entry point** (`develop_mono_mix_into`). onyx asked for an option on
  `develop_with_into` so the export call stays one call with one options value;
  `DevelopOptions` is `#[non_exhaustive]` for exactly this.
- **A struct for the option** (`MonoMix { weights }`, non-exhaustive). Nothing else is
  planned for it: `DEC-083` fixes the fade with no parameter and presets are named
  weight sets. `Option<[Rational; 3]>` matches `mono_mix_*`'s argument.
- **Applying `FixBadPixelsConstant` on the CFA.** The mono applier's 3×3 median mixes
  colours on a Bayer plane; a CFA-aware kernel is a new algorithm and should be measured
  on a real colour Leica file first. Refused for now, typed.
- **Refusing `black_levels: PerPosition` with the mix.** It would make the default
  (`FirstValue`) the only accepted value of a field the mix never reads; documenting
  that the field is not read is simpler and keeps every options value usable.
- **Checking the CFA in `output_dimensions_with`.** The mono sizing entries check only
  what sizing needs (`DEC-058` §3, `SPEC-065`); a mix on a non-CFA file is refused by
  the develop, before any write.

## Consequences

- **Positive:** the export matches the viewport by construction; SPEC-093 tests the
  rule against the mono develop of the mixed plane, on all five held files and on
  tier-A fixtures with a warp, every orientation and both framings.
- **Positive:** no full-frame intermediate without a warp; with one, the scratch is the
  mono develop's own `2A` on both held files that carry a non-identity warp (Pixel 3a
  and 7 Pro).
- **Positive:** additive. `DevelopOptions::default()` is unchanged, so every existing
  output is too.
- **Negative:** the developed mix and `develop_color_into` disagree on an optional
  `WarpRectilinear`: the mix applies it (the mono rule), the colour develop skips it.
  On the Pixel 3a and 7 Pro that is a visible difference between the colour and the
  mono export of the same file, as it already is between `develop_color_into` and a
  CFA-as-mono develop.
- **Negative:** a colour file with a mandatory `FixBadPixelsConstant` cannot be
  developed to a mix (item 3). If colour Leica files carry one, as the Q2 Monochrom
  does, that needs a CFA-aware applier before STAGE-032's presets can be judged on them.
- **Neutral:** strips redo up to seven halo rows each, 1–6% of the mix time on the
  held files.

## onyx's reply (2026-11-26)

onyx accepted the decision: mixing before geometry makes export equal the viewport by
construction, and the strip scratch suits iPad. `black_levels` is ignored under the mix,
documented, with no new error variant. Optional warps are applied, so the mono and mixed
exports share geometry, and `mono_mix` composes with `warp_alpha` and `Framing` as for the
mono develop. The mandatory-opcode refusal on colour Leica files is real (Q2, Q3, SL3) and
is lifted on every entry together in STAGE-034, not here.

## Validation

SPEC-093's acceptance criteria: equality with the mono develop of `mono_mix_plane_into`'s
output on tier-A fixtures (all eight orientations, both framings, `apply_orientation`,
`warp_alpha` 0, ½ and 1, a non-identity warp) and on the five held files with
pre-registered digests; byte identity of every existing output with the option absent;
zero allocations in `develop_with_scratch_into` with the mix; the ranked refusal
fixtures on all four entries; and developed-mix hashes on both CI determinism legs.

## References

- Related specs: SPEC-093 (this decision's spec), SPEC-092, SPEC-091, SPEC-065, SPEC-063
- Related decisions: DEC-083 (row and kernel), DEC-079 (linear sRGB, the fold),
  DEC-058 (options), DEC-059 (`warp_alpha`, `FullActiveArea`), DEC-060 (scratch),
  DEC-043 (the colour crop scatter), DEC-044 (bands), DEC-049 (required processing),
  DEC-068 and DEC-070 (colour levels), DEC-016 (caller-owned buffers)
- Discussions: onyx's answers (2026-11-23) and replies (2026-11-25, 2026-11-26) in the
  PROJ-006 brief
