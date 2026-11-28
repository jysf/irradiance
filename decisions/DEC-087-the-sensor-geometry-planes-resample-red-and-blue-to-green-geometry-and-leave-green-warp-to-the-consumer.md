---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-087
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
supersedes: null                   # amends DEC-086 items 5 and 8; DEC-086 stays active
superseded_by: null
status: proposed                   # accepted at the SPEC-099 ship; jysf accepts by merging it
deciders: [claude, jysf]

affected_scope:
  - src/opcode.rs
  - src/warp.rs
  - src/color.rs
  - src/lib.rs
  - decisions/DEC-086-the-colour-entries-fix-bad-pixels-from-the-four-nearest-same-colour-photosites-and-sensor-geometry-passes-the-warp-through.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-099-the-per-plane-warprectilinear-on-the-developed-colour-output.md

tags:
  - opcode
  - warp
  - chromatic-aberration
  - colour
  - mixer
  - stage-034
---

# DEC-087: the sensor-geometry planes resample red and blue to green's geometry, and leave green's warp to the consumer (amending DEC-086 items 5 and 8)

## Decision

1. **The accessor.** `opcode::sensor_geometry_warp(&Sensor) ->
   Result<Option<SensorGeometryWarp>, Error>` reads the first `WarpRectilinear` in
   `OpcodeList3`, whatever its flags, and returns:
   - `consumer()`: the one warp the consumer applies at sensor geometry. It is green's set
     (the set whose `CFAPlaneColor` is 1) for `N = 3` on a CFA sensor, the one set for
     `N = 1`, and `K1` for `N = 3` on a sensor with no CFA, which is what `develop_into`
     applies;
   - `sets()`: every set in file order, each a `PlaneWarp { plane, color, warp }`, where
     `color` is `CFAPlaneColor[plane]` for `N = 3` on a CFA sensor and `None` otherwise
     (one set for every plane, or no CFA);
   - `is_per_plane()`: whether the CA step of item 2 runs.
   `Ok(None)` without list 3 or a `WarpRectilinear` in it. `N` other than 1 or 3 is
   `MalformedOpcodeParams { id: 1, .. }`. A `CFAPlaneColor` that is not one each of 0, 1
   and 2, with `N = 3`, is `UnsupportedCfaPlaneColor`. It describes the file and refuses
   neither tangential terms nor a fold. It allocates nothing. `parse_warp_rectilinear` is
   unchanged: it still returns `K1`, which is red's set on a colour file, and monochrome
   consumers keep using it.
2. **When the CA step runs.** On `DevelopPath::SensorGeometry` (`srgb_linear_plane_into`,
   `srgb_linear_plane_f32_into`, `mono_mix_plane_into` and their `_rows_` forms), for the
   first `WarpRectilinear` whatever its flags, when `N = 3` and red's or blue's set differs
   from green's (all six coefficients compared with `==`). Only a plane whose set differs
   is resampled. With no warp, with `N = 1`, or with both sets equal to green's, the
   traversal is the existing one and the output is bit-identical. The step does not
   depend on `warp_alpha`. Green's set stays the consumer's (`DEC-086` item 5, now for
   green only).
3. **The rule.** It runs on the demosaiced camera triple, before the colour matrix, over
   the `ActiveArea` (`aw × ah`) in `ActiveArea`-relative coordinates:
   - **Geometry** as `warp.rs`'s `source_coord`: `cx = cx̂·(aw − 1)`, `cy = cŷ·(ah − 1)`,
     `m = mx.hypot(my)` with `mx = max(|cx|, |aw − 1 − cx|)` and `my` likewise. If
     `m = 0`, there is no step.
   - **Fold check.** Green's radial map `ρ = r·f_g(r²)` must be strictly increasing on
     `r ∈ [0, 1]`. That holds exactly when `q(s) = kr0 + 3kr1·s + 5kr2·s² + 7kr3·s³ > 0`
     on `s ∈ [0, 1]`. Its minimum is `scale_bounds` of `[kr0, 3kr1, 5kr2, 7kr3]`
     (closed form, `DEC-059` §3). Otherwise the entry refuses with the new
     `Error::UnsupportedWarpFold { slope_min }` (`ErrorKind::Unsupported`).
   - **Table.** `T = 4096`. For `j = 0..=T`, `σ_j = j/T`, with `f(k, s) = k0 + k1·s +
     k2·s·s + k3·s·s·s` and `h(s) = s·f_g(s)·f_g(s)`:
     - If `σ_j ≥ f_g(1)·f_g(1)`, then `s_j = 1`.
     - Otherwise bisect on `[0, 1]`: `mid = 0.5·(lo + hi)`, stop when `mid` equals `lo`
       or `hi`, set `lo = mid` if `h(mid) < σ_j` and `hi = mid` otherwise. Then
       `s_j = 0.5·(lo + hi)`.
     - Each differing plane `c` gets `t_c[j] = f_c(s_j)/f_g(s_j)`.
   - **Source.** Pixel `(x, y)`: `dx = (x − cx)/m`, `dy = (y − cy)/m`,
     `σ = dx·dx + dy·dy`, `u = min(σ·T, T)`, `j = min(⌊u⌋, T − 1)`,
     `k = t_c[j] + (t_c[j+1] − t_c[j])·(u − j)`. The source is
     `(cx + k·(x − cx), cy + k·(y − cy))`.
   - **Value.** `bilinear_sample` (`DEC-024`) of plane `c`'s camera samples over the
     `ActiveArea` at that source: clamped to the `ActiveArea`, rounded half up. Green
     passes through, and so does every pixel outside the `ActiveArea`.
   - **Beyond green's image** (`ρ > f_g(1)`), the ratio is held at its rim value `t_c[T]`.
   - **Halo.** `B = m·max_j max(|t_c[j] − 1|, |t_c[j+1] − 1|)·√((j+1)/T)` over the
     resampled planes, and `H = ⌈B⌉ + 1` rows, capped at `ah`.
4. **Where.** A delay ring of `2H + 2` camera-RGB rows sits between the fused traversal
   and the entry's `store`. It is flushed after the traversal. The rows forms develop
   their band plus `H` rows on each side through `develop_rows_banded`, run the same ring,
   and store only their own rows. So every value equals the whole plane bit for bit, and
   any tiling still gives the plane. All three stores (`u16`, unclamped `f32`, mono) are
   unchanged and read the corrected triple.
5. **Refusals and order.** On these entries, after the CFA check and before the colour
   calibration:
   1. `MalformedOpcodeParams { id: 1, .. }` for `N` other than 1 or 3;
   2. when the sets differ, `UnsupportedWarpTangentialTerms` for the first set (in plane
      order) with a non-zero `kt`;
   3. `UnsupportedWarpFold`.

   With identical sets, non-zero tangential terms are not refused, as before. Nothing is
   written before every check passes.
6. **What does not change.**
   - `DevelopPath::Colour` still refuses a mandatory `WarpRectilinear`.
   - `DevelopPath::MonoMix` keeps `DEC-086` item 6. Both developed outputs are
     `SPEC-100`'s.
   - Unchanged: the mono develop, `parse_warp_rectilinear`, `apply_warp_into`,
     `demosaic_into` and `to_srgb_linear_into`.
   - No `Error` variant changes. One is added: `UnsupportedWarpFold`.
   - `DEVELOP_VERSION` (`"8"`) and `MONO_MIX_VERSION` (`"1"`) do not move. The planes
     change only for files with an `N = 3` warp whose sets differ. The held ones are all
     mandatory, and their `SPEC-098` outputs are unreleased.
7. **Amends `DEC-086`.**
   - Item 5: the sensor-geometry path now applies the CA step of item 2, and leaves only
     green's set (or the one set) to the consumer.
   - Item 8: the per-plane accessor exists (item 1).

## Context

These decisions come from STAGE-034 and `SPEC-099`. The Leica Q2, Q3 and SL3 each carry a
mandatory `WarpRectilinear` with `N = 3`. They have one centre (`0.5, 0.5`), no tangential
terms, and sets that differ by up to 1.45, 1.66 and 1.38 px at sensor geometry (lateral
chromatic aberration). onyx answered on 2026-11-28 (its DEC-114, in SPEC-099's Context):

- The sensor-geometry planes resample red and blue to green's geometry, always.
- One warp (green's) then goes over the mixed plane, both in its viewport and in the
  developed mix.
- `N = 1` and identical sets stay bit-identical, and tangential terms stay refused.
- An accessor names the consumer's warp and every set with its colour.

**Driven, not assumed (2026-11-28; probe in the PROJ-006 session notes):**
- **Monotone.** `q`'s minimum is 0.75, 0.74 and 0.94 on the three greens, so no held file
  refuses. The table differs from an exact per-pixel inverse by at most 4.8e-4 px. `H`
  is 3 on all three files.
- **Against `dng_validate -3`.** This compares camera RGB after the SDK's per-plane warp
  with the CA step followed by green's warp. The registration error is the radial
  Lucas–Kanade shift at eight 160×160 patches per plane.
  - Red and blue register within 0.042 px of the baseline. The baseline is the
    green-only warp against an SDK run with red's and blue's sets overwritten by
    green's, so only the demosaic and the kernel differ.
  - Without the CA step, red and blue are 0.41–2.2 px off.
  - One resample with each plane's own set registers equally well.
- **Exactness.** With the step removed, the fused plane equals
  `to_srgb_linear_into(demosaic_into(normalised))` at every sample on all three files.
  So tier-B and tier-A tests can check the rule through public stage primitives.
- **Pre-registered digests.** Applying this rule in the probe gives
  `srgb_linear_plane_into` digests `0xd352bb75bf29d270` (Q2), `0xc6f67381a8b94593` (Q3)
  and `0xbb9908b919ce2b08` (SL3). The probe's no-CA pipeline reproduces SPEC-098's pins
  first.

## Alternatives Considered

- **Each plane warped with its own set at sensor geometry** (the full DNG warp there).
  That contradicts onyx's rule 1: the viewport warps the plane itself. Rejected.
- **One set over everything (no CA step).** It leaves 0.4–2.2 px of colour fringing at
  the corners, and onyx asked for the correction. Rejected.
- **Newton per pixel instead of the table.** It is exact to the last bit but costs
  several polynomial evaluations per pixel. The table's error (at most 4.8e-4 px) is
  about 100 times below the oracle's resolution. Rejected for cost.
- **Extrapolating the polynomial beyond green's image.** It is smooth on the held files,
  but the DNG polynomial is only defined for `r ≤ 1`. An extrapolated cubic can turn
  over, and it would need a second fold check with an unbounded domain. Holding the rim
  ratio is continuous and bounded. Chosen; recorded as an open question to onyx.
- **A full camera-RGB intermediate** (`3·A` `u16`, about 363 MB at 60 MP). It is
  unnecessary because the displacement is a few pixels. The ring costs about 0.5 MB.
  Rejected.
- **Refusing `N = 3` when only tangential terms are present and the sets are equal.**
  That would refuse a file that develops today, while nothing at sensor geometry applies
  those terms. Rejected.
- **Checking the CA bound in linear sRGB** (what the entries return). The `u16` clamp and
  the matrix mix the planes, and the shift estimate was off by up to 1.5 px. The oracle
  stays in camera RGB and is tied to the entries by the exactness tests. Rejected.

## Consequences

- **Positive.** The three files' viewport planes have no lateral CA. onyx applies one warp
  per file, named by one accessor, so the viewport and the coming developed mix
  (`SPEC-100`) use the same set. `N = 1` files are untouched.
- **Negative.**
  - The rows forms read `H` more rows on each side.
  - The CA step adds about 0.5–0.65 s to a 60 MP plane, single-threaded.
  - Until `SPEC-100`, a file with an optional `N = 3` warp whose sets differ (none held)
    breaks `DEC-084` rule 1's parity: the mixed plane is CA-corrected, and the developed
    mix skips the warp.
  - The red and blue planes go through two interpolations once green's warp is applied.
    This is measured as slightly smoother than the SDK's single interpolation.
- **Neutral.**
  - `N = 2` and `N ≥ 4` on a colour file are now refused as malformed. They were passed
    through before; no file is known to have them.
  - Beyond the rim the correction is a choice, not the specification.

## Validation

- SPEC-099 AC2 checks the identity cases bit-identical.
- AC3 and AC4 check the rule exactly against a reference written from item 3.
- AC5 checks the bands, and AC6 the refusals and their order.
- AC7 checks the pre-registered digests and the accessor on the held files.
- AC8 is the `dng_validate -3` bound, with its red-proof: within 0.10 px of the baseline
  with the CA step, and at least 0.30 px off without it.
- AC9 checks that every other output is unchanged.

## References

- SPEC-099, SPEC-100 (developed outputs), SPEC-098, SPEC-064, SPEC-093.
- DEC-024 (bilinear, clamp to edge), DEC-059 (`scale_bounds`, `warp_alpha`), DEC-084,
  DEC-086.
- DNG 1.7.1.0, `WarpRectilinear` (provenance class 1). The ratio composition and the
  table are derived here from the specification's own transform (class 1). No decoder
  source was read.
