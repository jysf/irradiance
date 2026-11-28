---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-088
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

created_at: 2026-11-28
supersedes: null                   # amends DEC-084 items 1, 3, 6 and 7, DEC-086 item 6 and DEC-087 item 6; all three stay active
superseded_by: null
status: proposed                   # at SPEC-100's design, 2026-11-28; accepted at its ship
deciders: [claude, jysf]

affected_scope:
  - src/color.rs
  - src/develop.rs
  - src/opcode.rs
  - src/warp.rs
  - src/demosaic.rs
  - decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md
  - decisions/DEC-086-the-colour-entries-fix-bad-pixels-from-the-four-nearest-same-colour-photosites-and-sensor-geometry-passes-the-warp-through.md
  - decisions/DEC-087-the-sensor-geometry-planes-resample-red-and-blue-to-green-geometry-and-leave-green-warp-to-the-consumer.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-100-the-per-plane-warp-on-the-developed-colour-output-and-the-de.md

tags:
  - warp
  - chromatic-aberration
  - colour
  - mixer
  - develop
  - stage-034
---

# DEC-088: the developed outputs apply the CA step then green's warp: on the camera triple before the matrix for the colour develop, over the mixed plane for the developed mix

## Decision

Let `W` be `opcode::sensor_geometry_warp(sensor).consumer()`: green's set for `N = 3` on a
colour file, the one set for `N = 1` (`DEC-087` item 1). "The CA step" is `DEC-087` items
2 and 3, unchanged: red and blue resampled to green's geometry on the demosaiced camera
triple, when `N = 3` and their sets differ from green's, whatever the flags and
independent of `warp_alpha`.

1. **One rule for every developed output.** A developed output is the CA step at sensor
   geometry, then one warp, `W`, over the `ActiveArea` exactly as `warp::apply_warp_into`
   applies it (`DEC-024` bilinear, clamped to the `ActiveArea`, rounded half up to `u16`),
   then the framing and the orientation. `warp_alpha` lerps `W` only (`DEC-059` §1), so it
   dials back lens distortion and never chromatic aberration. This is onyx's rule
   (its DEC-114): the viewport draws the sensor-geometry plane and applies `W` at the
   user's alpha, so export and viewport use one geometry.

2. **The colour develop** (`develop_color_into`, `develop_color_f32_into` and the threaded
   forms). When the first `WarpRectilinear` in `OpcodeList3` is **mandatory**:
   - each pixel is the matrix of the warped camera triple, which places the warp where
     DNG places it (stage 3, before the colour matrix). For a pixel of the output at
     `ActiveArea` position `(x, y)` (from the crop and orientation map, `DEC-043`), each
     camera plane `c` is `apply_warp_into(W, aw, ah, plane_c)` at `(x, y)`, where
     `plane_c` is the CA-corrected camera plane over the `ActiveArea`; the triple then goes
     through the entry's own store (`store_u16`, or `store_f32_quantised`, so the `f32`
     entry stays exactly `to_linear_f32_into` of the `u16` one).
   - Exactly: `develop_color_into` equals the crop and orientation of
     `to_srgb_linear_into(warp_planes(ca(demosaic_into(n))))`, where `warp_planes` applies
     `apply_warp_into(W, ..)` to each camera plane over the `ActiveArea` and leaves the
     masked border as it is.
   - `N` must be 1 or 3 (`DEC-087` item 5). With `N = 1`, or identical sets, there is no
     CA step and `W` is applied to all three planes.
   - **No options.** `warp_alpha` is 1, the framing is the `DefaultCrop` and the
     orientation is applied, as the colour develop has always done. `DevelopOptions` is not
     taken. An options-taking colour entry is additive later and changes no output here.
   - **An optional first warp is still skipped**, CA step included, so every file that
     develops today keeps its output bit for bit (the Pixel 3a and 7 Pro carry optional
     `N = 1` warps). A file with an optional `N = 3` warp whose sets differ (none held)
     therefore gets an uncorrected colour develop while its sensor-geometry plane is
     corrected; this is recorded, not fixed.
   - **The threaded forms** produce the same bytes: with an applied warp they run the
     single-thread body, as orientations 5 to 8 already do (`DEC-044`).

3. **The developed mix** (`DevelopOptions::mono_mix`). `DEC-084` rule 1 stays, with its
   two inputs named: the mixed plane is `mono_mix_plane_into`'s, which is CA-corrected
   since `DEC-087`, and the warp is `W` lerped to `warp_alpha`. So the developed mix equals,
   bit for bit, the mono develop of `mono_mix_plane_into`'s plane at levels
   `MONO_MIX_BLACK` and `MONO_MIX_WHITE` with `W` as its `WarpRectilinear` (an `N = 1`
   list holding `W`), under the same options.
   - It applies the first `WarpRectilinear` **whatever its flags**, as the mono develop and
     `mono_mix_plane_into` do (`DEC-084` item 3, onyx's 2026-11-26 reply: optional warps
     are applied). `N` must be 1 or 3. This replaces `DEC-086` item 6: a mandatory
     `N = 3` warp is applied, not refused, and an optional one is applied, not skipped.
   - The CA step does not depend on `warp_alpha`. At `warp_alpha = 0`, `W` is the identity
     and is skipped, and the CA-corrected mix goes straight to the framing (the strips'
     crop scatter).
   - **Where.** Each strip of the strip pass (`DEC-084` item 5) develops its rows plus `H`
     rows on each side (clamped to the plane) through the band traversal, runs the CA delay
     ring of `DEC-087` item 4 over them, and mixes only its own rows from the corrected
     triple. That is exactly `mono_mix_plane_rows_into`'s value for every pixel, so strips
     still tile the plane.
   - **No allocation.** The ring lives in the caller's scratch, and the CA step's tables
     live in fixed-size arrays (no heap), so `develop_with_scratch_into` still allocates
     nothing and `develop_with_into` makes exactly one allocation (`DEC-060` §2). The
     sensor-geometry entries' output is unchanged by moving the tables.

4. **Scratch for the developed mix.** With `S = 512`, `w` the plane width, `h` its height,
   `A` the `ActiveArea`'s sample count and `H` the CA halo (`DEC-087` item 3; 0 when the
   CA step does not run): the strip work is `T_H = w·(min(h, S + 2H + 7) + 8)`, and the
   ring is `R = 3·(2H + 2)·w`. `scratch_len` is `T_H + R` without an applied warp and
   `A + max(A, T_H + R)` with one. With `H = 0` (`R` is then not reserved) both are
   `DEC-084` item 6's numbers, so every file without a per-plane difference sizes as today.
   On the held files `A ≥ T_H + R`, so the scratch stays `2A`.

5. **Refusals and order.**
   - **Colour develop**, only with a mandatory first warp: after the CFA check and before
     the colour calibration, `DEC-087` item 5's three
     (`MalformedOpcodeParams { id: 1, .. }` for `N` other than 1 or 3 or a non-finite later
     set; with differing sets, `UnsupportedWarpTangentialTerms` for the first set with a
     non-zero `kt`; `UnsupportedWarpFold`), then `UnsupportedWarpTangentialTerms` for `W`
     itself when its `kt` is not zero (the applied warp is radial only, `SPEC-018`). Every
     check runs before any write. With an optional first warp nothing new is read.
   - **Developed mix**, `DEC-084` item 7 with one insertion: after the CFA, `DEC-087` item
     5's three, then `MissingColorCalibration` and `MixRowOutOfRange` as before. The
     `OpcodeList3` parse and, under `FullActiveArea`, the bounds rule (`DEC-059` §3) now
     read `W` lerped (previously `K1`, which is red's set on a colour file). `W`'s own
     tangential terms are refused by the applier, as on the mono develop.
   - **`scratch_len` under the mix** reads what it needs to size the ring: the per-plane
     parse (`MalformedOpcodeParams { id: 1, .. }`), and, when `N = 3` on a CFA sensor,
     `CFAPlaneColor` (`UnsupportedCfaPlaneColor`) and the fold (`UnsupportedWarpFold`),
     because `H` needs green's set. It still reads no levels or calibration.
     `output_dimensions_with` is unchanged.

6. **Versions.** Neither `DEVELOP_VERSION` (`"8"`) nor `MONO_MIX_VERSION` (`"1"`) moves.
   - The colour develop changes only for files with a mandatory `WarpRectilinear`, which
     it refused (`UnsupportedRequiredOpcode { list: 3, id: 1 }`); relaxing a refusal does
     not bump (`DEC-058` §5). Optional warps are skipped as before.
   - The developed mix changes for a mandatory `N = 3` warp (refused before) and for an
     optional `N ≠ 1` warp (skipped before; none held). The developed mix itself is
     unreleased (`SPEC-093` is in `[Unreleased]`; 0.2.7 has no `mono_mix`), so no released
     output moves.
   - The mixer's arithmetic is unchanged.

7. **Amends.**
   - `DEC-084` item 1 (the warp is `W`; the plane is CA-corrected), item 3 (the first warp
     whatever its flags, `N` 1 or 3), item 6 (the CA ring and halo) and item 7 (the
     per-plane checks; `scratch_len` reads `CFAPlaneColor` and the fold for `N = 3`).
   - `DEC-086` item 6: replaced by items 2 and 3 here.
   - `DEC-087` item 6: the developed outputs now apply the per-plane warp.
   - `opcode::DevelopPath::Colour` applies the first `WarpRectilinear` in `OpcodeList3`
     (when mandatory), and `DevelopPath::MonoMix` applies it whatever its `N` (the
     single-plane condition goes).

## Context

STAGE-034's last spec (`SPEC-100`). After `SPEC-099` the Leica Q2, Q3 and SL3 give
CA-corrected planes at sensor geometry, but `develop_color_into` and the developed mix still
refuse their mandatory `N = 3` warp. onyx bound the developed mix on 2026-11-28 (its
DEC-114, relayed by the orchestrator): the CA step at sensor geometry, then the mix, then
`W` at `warp_alpha` and `Framing`, as its viewport does; `warp_alpha` dials back lens
distortion only; files with `N = 1` or identical sets, and every file that develops today,
keep their output. The colour develop was left to this design.

**Driven, not assumed** (design probe, 2026-11-28, public API only, against
`dng_validate` 1.7.1 stage 3; sources and output in the PROJ-006 session notes,
`spec-100-design-*`):
- **Warp before the matrix or after.** Warping the `u16` linear-sRGB plane (what
  `srgb_linear_plane_into` returns, already clamped) instead of the camera triple differs
  from the DNG order in about half the samples, by more than 16 codes in 139,489 (Q2),
  586,577 (Q3) and 458,164 (SL3) samples, and by up to 12,275, 8,437 and 35,260 codes.
  Interpolating after the clamp mixes pinned out-of-gamut values. Before the matrix the
  clamp comes once, last, as in the SDK.
- **Registration against the SDK.** In camera RGB over the crop (the values the matrix
  receives, tied to the entry by the exactness route), the CA step then `W` registers red
  and blue within 0.042 px (Q2), 0.018 px (Q3) and 0.036 px (SL3) of the green-only
  baseline, and green equals it; without the CA step they are at least 0.406 px off. The
  developed mix, against the SDK's camera RGB mixed by the same fixed-point row, is within
  0.056 px of its baseline. Measured through the entry's `u16` output and an inverse
  matrix instead, one Q2 blue patch reads 0.217 px, from clipped samples; so the bound is
  stated in camera RGB, as `SPEC-099`'s was.
- **Exactness.** `srgb_linear_plane_into` equals `to_srgb_linear_into(ca(demosaic_into(n)))`
  at every sample on all three files, the SL3 with the bad-pixel fix applied by the
  probe. So the colour develop's rule can be checked bit for bit from public stage
  primitives and the tests' own CA reference.

## Alternatives Considered

- **One resample per plane with its own set** (DNG's stage-3 order, no CA step). It
  registers as well (`SPEC-099`: within 0.02 px) and is a little sharper. But at
  `warp_alpha < 1` "each plane's own set" has no meaning that keeps CA corrected, so a
  future options-taking colour entry would need the composition anyway. It would also give
  the colour develop a different geometry from the viewport and the developed mix of the
  same file (by the table error and one interpolation), and need a second implementation
  of the CA geometry inside the warp. Rejected.
- **Warping after the matrix** (the colour develop as the mono develop's geometry over
  `srgb_linear_plane_into`, channel by channel). It would make the colour develop a
  composition of public entries, but it interpolates clamped values (Context: up to
  12,275 codes off the DNG order) and is not what DNG specifies. Rejected.
- **`warp_alpha` and `Framing` on the colour develop.** No consumer has asked: onyx's
  colour view is the sensor-geometry plane, and its export is the mix. Adding them means
  four new public entries. Deferred; additive.
- **Applying optional warps on the colour develop.** It would change the Pixel 3a's and
  7 Pro's colour output and bump `DEVELOP_VERSION`. Rejected.
- **Keeping the developed mix's optional-`N ≠ 1` skip.** It keeps `DEC-084` rule 1 broken
  for such a file (its mixed plane is CA-corrected, the developed mix is not warped).
  The skip was only a placeholder until the per-plane rule was decided, and it was never
  released. Rejected.
- **A delay ring instead of the `ActiveArea` camera buffer for the colour develop.**
  `W`'s vertical reach is up to about 524 rows on the Q3, so a ring of about 60 MB could
  replace the 363 MB buffer. The output is the same. Deferred as an optimisation (SPEC-100
  follow-up); the buffer is the simplest correct form.
- **Threading the warped colour develop.** The gather could be split over output rows.
  Deferred; the fallback is already the rule for orientations 5 to 8.

## Consequences

- **Positive.** The Q2, Q3 and SL3 develop to colour and to a mix with no refusal. The
  developed mix equals the viewport's geometry by construction again (`DEC-084` rule 1),
  so onyx's warp parity check carries over. The colour develop follows the DNG order and
  can be checked against public primitives bit for bit.
- **Negative.**
  - The warped colour develop holds an `ActiveArea` camera buffer (`3·A` `u16`, about
    363 MB at 60 MP) besides the green plane, and it is single-threaded.
  - Red and blue are interpolated twice (CA step, then `W`), slightly smoother than the
    SDK.
  - The colour develop and the developed mix still disagree on optional warps (the mix
    applies them, the colour develop skips them), as `DEC-084` already records.
- **Neutral.** `scratch_len` under the mix reads `CFAPlaneColor` for an `N = 3` warp.
  `N = 2` and `N ≥ 4` become malformed on the developed mix too, as on the
  sensor-geometry entries.

## Validation

SPEC-100's acceptance criteria: the colour develop against the exact composition of public
primitives on tier-A fixtures and on the Q2 and Q3; the developed mix against the mono
develop of `mono_mix_plane_into`'s plane with `W` on tier-A fixtures (every orientation,
both framings, `warp_alpha` 0, ½ and 1) and on the three files; pre-registered digests; the
`dng_validate -3` bound with its red-proof; identity cases bit-identical; the scratch
length and zero allocation; the ranked refusals; threaded output equal to single-thread;
every other output unchanged.

## References

- SPEC-100, SPEC-099, SPEC-098, SPEC-093, SPEC-064, SPEC-050, SPEC-051.
- DEC-087 (the CA step, `W`), DEC-086, DEC-084 (the developed mix), DEC-059
  (`warp_alpha`), DEC-058 §5 (versions), DEC-060 (scratch), DEC-044 (bands and threads),
  DEC-043 (the colour crop scatter), DEC-024 (bilinear).
- DNG 1.7.1.0, `WarpRectilinear` and the stage order (provenance class 1). No decoder
  source was read.
