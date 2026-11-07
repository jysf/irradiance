//! Colour transform — camera-space linear RGB to display-referred sRGB.
//!
//! STAGE-007's [`crate::demosaic::demosaic_into`] produces **camera-space**
//! linear RGB: the channels are the sensor's native primaries, scaled by the
//! camera's own white, with no display encoding — so a demosaiced image reads
//! greenish and flat. This module (STAGE-008, `SPEC-033`) turns it into a
//! **display-referred sRGB** image, the first recognisable colour picture.
//!
//! # The transform — one 3×3 matrix built once (`DEC-033`)
//!
//! Three linear stages fold into a single 3×3 matrix `M`, built once per image
//! and applied per pixel:
//!
//! 1. **White balance** from `AsShotNeutral` (`asn`): a neutral subject reads
//!    `asn` in camera space, so `D = diag(1/asn_R, 1/asn_G, 1/asn_B)` maps
//!    neutral to `(1, 1, 1)`.
//! 2. **camera → XYZ(D50)**: `ForwardMatrix1` (`FM1`) already maps white-balanced
//!    camera values to XYZ(D50), so `cam2xyz = FM1 · D`. When no `ForwardMatrix`
//!    is present (the Nikon D300S), `ColorMatrix1` (`CM1`) maps XYZ→camera, so
//!    `cam2xyz = CA(W→D50) · inv(CM1)` with scene white `W = inv(CM1)·asn`,
//!    Bradford-adapted to D50 (DNG spec ch. 6). **No `D`** on this path: `inv(CM)`
//!    expects raw camera values, and `D` already counted white balance (the
//!    original `inv(CM1)·D` tinted grey magenta; fixed in `DEVELOP_VERSION` 2).
//! 3. **XYZ(D50) → linear sRGB**: the committed constant `C = S · B`, where
//!    [`BRADFORD_D50_TO_D65`] (`B`) is the Bradford chromatic adaptation and
//!    [`SRGB_FROM_XYZ_D65`] (`S`) is IEC 61966-2-1's linear-sRGB-from-XYZ(D65).
//!
//! `M = C · cam2xyz` (`= C · FM1 · D`, or `C · CA · inv(CM1)`).
//!
//! # Scene-adaptive interpolation (`SPEC-034`, `DEC-034`)
//!
//! A DNG is calibrated at **two** illuminants — `ColorMatrix1/2` (and often
//! `ForwardMatrix1/2`) at `CalibrationIlluminant1/2` — and the colour-correct
//! matrix for a shot is the one **interpolated** between them at the scene's own
//! colour temperature. When both calibrations are present (and both illuminant
//! codes are known), this module finds the scene interpolation weight
//! `g ∈ [0, 1]` from `AsShotNeutral` by the DNG reciprocal-temperature
//! `NeutralToXY` method — a fixed 3-iteration loop that interpolates
//! `CM(g) = (1−g)·CM1 + g·CM2`, maps the camera neutral through `inv(CM(g))` to
//! an XYZ white, takes its chromaticity, derives its CCT by **McCamy's cubic**
//! (a polynomial, so the whole weight stays `+ − × ÷` only), and re-derives `g`
//! by mired interpolation between the two `CalibrationIlluminant` CCTs. The
//! matrices are then lerped at the converged `g` (`FM(g)` when both forward
//! matrices are present, else `inv(CM(g))`) and `cam2xyz` built from them exactly
//! as the single-illuminant path does. The weight is computed **once per image**,
//! off the per-pixel path. See [`interpolation_weight`].
//!
//! When the interpolation preconditions are not met — no `ColorMatrix2`, a
//! missing or unknown `CalibrationIlluminant`, `T1 == T2`, or a degenerate step
//! in the iteration — the build falls back to the **single-illuminant** path
//! (`ForwardMatrix1`, else `inv(ColorMatrix1)`), which is **byte-identical** to
//! `SPEC-033`. So a single-calibration sensor is unchanged (the regression
//! guard), and only dual-calibration files shift — toward the scene illuminant.
//! Public signatures are unchanged; this is an internal accuracy refinement.
//!
//! # Arithmetic — committed-constant `f64`, basic ops only (`DEC-033`)
//!
//! The build and per-pixel apply are `f64` using only `+ − × ÷`. These are
//! IEEE-754 correctly-rounded and byte-identical on macOS and Linux (no FMA, no
//! transcendental) — the same basis [`crate::tone::to_linear_f32_into`] already
//! relies on, and [`crate::warp`]'s radial polynomial. This is **not** the
//! exact-rational discipline of `DEC-030` (the colour-**metadata** oracle, which
//! stays exact): the sRGB primaries and the Bradford CAT are irrational, so —
//! exactly as `DEC-025` runs the OETF's transcendental once into a committed
//! table — the irrational colour constant `C` is built from the **committed**
//! `S`/`B` arrays, and determinism comes from committed data + IEEE basic ops.
//!
//! The sRGB **OETF** is not applied here: this module produces **linear**-sRGB,
//! and [`crate::tone::encode_srgb16_into`] (the committed `DEC-025` table) does
//! the display encode. `color` outputs linear; `tone` encodes.
//!
//! # Scale-free apply (`DEC-018`)
//!
//! Inputs and outputs are interleaved `u16` where full-scale `65535` means
//! `1.0` (`DEC-018`), so the `/65535` on input and `×65535` on output cancel and
//! the apply is scale-free: `out_i = Σ_j M[i][j]·cam_j` directly, with `cam_j`
//! the `u16` value. Each result is clamped to `[0, 65535]` **before** the
//! round-half-up quantize, so a saturated highlight or a negative off-gamut mix
//! is pinned to the window and the rounding can never leave `u16` range (the
//! clamp is load-bearing, `DEC-018`).
//!
//! # Caller-owned buffers (`DEC-016`)
//!
//! Like [`crate::plane::unpack_into`], [`crate::develop::develop_into`] and
//! [`crate::demosaic::demosaic_into`], the public functions here write a
//! caller-owned `dst`. [`develop_color_into`] and [`develop_color_f32_into`]
//! allocate one internal `width·height` green plane (`SPEC-049`'s fuse,
//! `DEC-042`): the mosaic is level-normalised once into a small rolling row
//! cache (not a full scratch plane) and the colour matrix is applied straight
//! into `dst`, so neither a normalized mosaic nor a camera-RGB intermediate is
//! allocated. The public `src`/`dst` stay caller-owned.
//!
//! # Float entry point (`SPEC-049`, `DEC-042`)
//!
//! [`develop_color_f32_into`] shares that fused pipeline but writes scene-linear
//! `f32` directly, equal to `to_linear_f32_into(develop_color_into(..))` to the
//! bit (quantise to `u16`, then `/65535.0`), so a float consumer never holds the
//! `u16` result and its `f32` copy at once.
//!
//! # Panic policy (`no-panics-on-untrusted-input`)
//!
//! A zero-denominator required rational, an absent or ill-shaped
//! `AsShotNeutral`/`ForwardMatrix`/`ColorMatrix`, a singular `ColorMatrix`, and
//! an off-gamut or saturated mix are typed errors ([`Error::MissingColorCalibration`])
//! or clamped values — never a panic and never a NaN written to `dst`. Wrong
//! buffer lengths are [`Error::ColorSourceWrongLength`] / [`Error::RgbBufferWrongLength`].
//!
//! # Provenance
//!
//! The DNG colour model (`ForwardMatrix`/`ColorMatrix`/`AsShotNeutral`,
//! camera→XYZ) is the Adobe DNG Specification; the sRGB primaries are
//! IEC 61966-2-1; `B` is the Bradford chromatic-adaptation transform
//! (`D50 → D65`). All three are public standards — provenance **class 1**, no
//! implementation read. See `docs/provenance-ledger.md`.

use crate::demosaic::develop_color_fused;
use crate::ifd::{active_area_origin, Rational, Sensor};
use crate::Error;

/// Linear sRGB ← CIE XYZ (D65), the IEC 61966-2-1 primaries (sRGB colour space
/// definition). The inverse of the published XYZ-from-sRGB matrix; its rows map
/// the D65 white point to `(1, 1, 1)`. A committed `f64` constant (`DEC-033`):
/// the sRGB primaries are irrational, so this is transcribed from the standard
/// once and reproduced exactly, as `DEC-025` does for the OETF table.
const SRGB_FROM_XYZ_D65: [[f64; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

/// Bradford chromatic adaptation, CIE XYZ (D50) → CIE XYZ (D65). Maps the D50
/// white point `[0.96422, 1.0, 0.82521]` to the D65 white point
/// `[0.95047, 1.0, 1.08883]`. A committed `f64` constant (`DEC-033`): the
/// Bradford cone-response matrix is irrational, transcribed from the standard
/// once. Folded with [`SRGB_FROM_XYZ_D65`] into the one XYZ(D50)→linear-sRGB
/// constant `C = S · B` built in [`build_camera_to_srgb_linear`].
const BRADFORD_D50_TO_D65: [[f64; 3]; 3] = [
    [0.9555766, -0.0230393, 0.0631636],
    [-0.0282895, 1.0099416, 0.0210077],
    [0.0122982, -0.0204830, 1.3299098],
];

/// Correlated colour temperature (kelvin) of each EXIF `LightSource` /
/// DNG `CalibrationIlluminant` code `SPEC-034` recognises (`DEC-034`). A
/// `CalibrationIlluminant` code not in this table is treated as **unknown** and
/// takes the single-illuminant fallback, rather than guessing a temperature.
/// Only the standard illuminants with a crisp, published CCT are listed; the
/// fuzzy daylight/fluorescent codes are deliberately omitted (their nominal CCTs
/// are ranges, not points).
///
/// Sources: EXIF 2.3 `LightSource` tag code assignments (also Adobe DNG
/// Specification §Chapter 5, `CalibrationIlluminant`); the CCTs are the CIE
/// standard-illuminant values — Illuminant A = 2856 K, B = 4874 K, C = 6774 K,
/// and the CIE D-series `Dxx` names the correlated temperature directly
/// (D50 = 5003 K, D55 = 5503 K, D65 = 6504 K, D75 = 7504 K, each the
/// International-Temperature-Scale-corrected value `≈ xx00 × 1.4388/1.4380`).
/// Code 1 (Daylight) takes the nominal 5500 K; codes 3 (Tungsten) and 17
/// (Standard light A) are incandescent ≈ Illuminant A = 2856 K; code 24
/// (ISO studio tungsten) is the ISO 3200 K studio standard.
const ILLUMINANT_CCT: &[(u32, f64)] = &[
    (1, 5500.0),  // Daylight (EXIF nominal)
    (3, 2856.0),  // Tungsten (incandescent) ≈ CIE Illuminant A
    (17, 2856.0), // Standard light A (CIE Illuminant A)
    (18, 4874.0), // Standard light B (CIE Illuminant B)
    (19, 6774.0), // Standard light C (CIE Illuminant C)
    (20, 5503.0), // D55 (CIE D55)
    (21, 6504.0), // D65 (CIE D65)
    (22, 7504.0), // D75 (CIE D75)
    (23, 5003.0), // D50 (CIE D50)
    (24, 3200.0), // ISO studio tungsten
];

/// McCamy's 1992 cubic CCT-from-chromaticity: `n = (x − x0) / (y0 − y)`;
/// `T = c3·n³ + c2·n² + c1·n + c0`. A **polynomial** (`n³ = n·n·n`, no `powf`),
/// so [`mccamy_cct`] stays `+ − × ÷` only and cross-platform (`DEC-033`/`DEC-034`).
/// The epicentre `(0.3320, 0.1858)` and the four coefficients are the published
/// constants of McCamy, *Color Research & Application* 17(2), 1992 (a class-2
/// paper; see `docs/provenance-ledger.md`).
const MCCAMY_EPICENTRE_X: f64 = 0.3320;
const MCCAMY_EPICENTRE_Y: f64 = 0.1858;
const MCCAMY_C3: f64 = 449.0;
const MCCAMY_C2: f64 = 3525.0;
const MCCAMY_C1: f64 = 6823.3;
const MCCAMY_C0: f64 = 5520.33;

/// Transform interleaved **camera-space linear RGB** (`width·height·3` `u16`
/// samples) into interleaved **linear-sRGB** `u16`, by white balance + colour
/// matrix (`SPEC-033`). The 3×3 matrix `M` is built once from `sensor`'s colour
/// metadata (`build_camera_to_srgb_linear`) and applied per pixel; the sRGB
/// OETF is **not** applied (this is linear-sRGB, which [`crate::tone`] encodes).
///
/// Both buffers are caller-owned and this function allocates nothing
/// (`DEC-016`). The apply is scale-free (`65535 ≡ 1.0`, `DEC-018`): each output
/// channel is `Σ_j M[i][j]·cam_j`, clamped to `[0, 65535]` before the
/// round-half-up quantize.
///
/// # Errors
///
/// - [`Error::ColorSourceWrongLength`] if `camera_rgb.len() != width·height·3`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width·height·3`.
/// - [`Error::MissingColorCalibration`] if `sensor` carries no usable
///   `ForwardMatrix1`/`ColorMatrix1`, no `AsShotNeutral`, a zero-denominator
///   required rational, or a non-invertible `ColorMatrix`.
///
/// Never panics: lengths are checked before any write, every matrix entry is a
/// guarded division, and the quantize clamps before rounding.
pub fn to_srgb_linear_into(
    sensor: &Sensor,
    camera_rgb: &[u16],
    dst: &mut [u16],
) -> Result<(), Error> {
    let rgb_len = interleaved_rgb_len(sensor)?;
    let expected = usize::try_from(rgb_len).unwrap_or(usize::MAX);
    if camera_rgb.len() != expected {
        return Err(Error::ColorSourceWrongLength {
            expected: rgb_len,
            actual: camera_rgb.len(),
        });
    }
    if dst.len() != expected {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst.len(),
        });
    }

    let matrix = build_camera_to_srgb_linear(sensor)?;
    apply_matrix_into(&matrix, camera_rgb, dst);
    Ok(())
}

/// Develop a raw **Bayer mosaic** (`width·height` `u16` samples) all the way to
/// interleaved **linear-sRGB** `u16` (`width·height·3`): per-channel
/// level-normalize → demosaic → colour matrix, fused into two memory-lean passes
/// (`SPEC-033`; the `SPEC-049` fuse, `DEC-042`).
///
/// The per-channel level-normalize happens on the **mosaic, before demosaic**:
/// black-subtract is per CFA colour and does not commute with interpolation, so
/// each photosite is normalized by the `black_levels`/`white_levels` entry for
/// its 2×2 position (scalar `black_level`/`white_level` fallback, then
/// `0`/`2^bits-1`), clamped per `DEC-018`. Rather than materialise a normalized
/// mosaic and a `camera_rgb` intermediate, the fused path (`DEC-042`) sweeps one
/// output row at a time: it level-normalises each sample **once** into a small
/// rolling cache of recent rows, writes the finalised green into a single
/// `width·height` green plane, then interpolates red/blue, assembles the
/// camera-RGB triple in registers, and applies the colour matrix
/// ([`build_camera_to_srgb_linear`]) straight into `dst`. The `u16` output is
/// **byte-identical** to the pre-fuse `normalize → demosaic_into → matrix`
/// composition (the colour goldens and `tests/color_oracle.rs`'s byte-equivalence
/// test are the gate).
///
/// The public `src`/`dst` are caller-owned (`DEC-016`); the only `width·height`
/// scratch is the green plane (`~47 MB` at 23 MP, down from the pre-fuse
/// `~187 MB`); the rolling normalised-row cache is a few rows, negligible.
///
/// # Errors
///
/// - [`Error::ColorSourceWrongLength`] if `src.len() != width·height`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width·height·3`.
/// - [`Error::NotAColorFilterArray`] / [`Error::UnsupportedCfaPattern`] for a
///   non-CFA plane or an unsupported mosaic, or [`Error::MissingColorCalibration`]
///   if the colour matrix cannot be built — in that order (CFA before matrix).
///
/// Never panics.
pub fn develop_color_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    let pixels = pixel_count(sensor)?;
    let rgb_len = pixels.checked_mul(3).ok_or(Error::RgbBufferWrongLength {
        expected: u64::MAX,
        actual: dst.len(),
    })?;
    let expected_src = usize::try_from(pixels).unwrap_or(usize::MAX);
    let expected_dst = usize::try_from(rgb_len).unwrap_or(usize::MAX);
    if src.len() != expected_src {
        return Err(Error::ColorSourceWrongLength {
            expected: pixels,
            actual: src.len(),
        });
    }
    if dst.len() != expected_dst {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst.len(),
        });
    }

    // The one remaining scratch buffer: the `width·height` green plane.
    let mut green = vec![0u16; expected_src];
    let normalize = make_normalizer(sensor, src);
    let (dst_triples, _remainder) = dst.as_chunks_mut::<3>();

    develop_color_fused(sensor, &mut green, &normalize, || {
        let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] =
            build_camera_to_srgb_linear(sensor)?;
        Ok(move |pixel: usize, r: u16, g: u16, b: u16| {
            let rf = f64::from(r);
            let gf = f64::from(g);
            let bf = f64::from(b);
            if let Some([out_r, out_g, out_b]) = dst_triples.get_mut(pixel) {
                *out_r = quantize(m00 * rf + m01 * gf + m02 * bf);
                *out_g = quantize(m10 * rf + m11 * gf + m12 * bf);
                *out_b = quantize(m20 * rf + m21 * gf + m22 * bf);
            }
        })
    })
}

/// Develop a raw **Bayer mosaic** (`width·height` `u16` samples) to interleaved
/// **scene-linear sRGB** `f32` (`width·height·3`), sharing [`develop_color_into`]'s
/// fused pipeline but writing `f32` directly so a float consumer (the Python
/// bindings, ML dataloaders) never holds the `u16` result and its `f32` copy at
/// once (`SPEC-049`, `DEC-042`).
///
/// The value written is **exactly** `to_linear_f32_into(develop_color_into(..))`:
/// each channel is quantised to `u16` first — the identical clamp-and-round
/// [`quantize`] (`DEC-018`) the integer path uses — and then divided by
/// `65535.0`, matching [`crate::tone::to_linear_f32_into`] to the bit. The raw
/// `f64` matrix result is **not** divided directly; that would skip the
/// clamp-and-round and differ. As in [`crate::tone::to_linear_f32_into`] this is
/// scene-linear, not display-encoded (the sRGB OETF is [`crate::tone`]'s).
///
/// The only internal scratch is the `width·height` green plane; `src`/`dst` are
/// caller-owned (`DEC-016`).
///
/// # Errors
///
/// - [`Error::ColorSourceWrongLength`] if `src.len() != width·height`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width·height·3`.
/// - [`Error::NotAColorFilterArray`] / [`Error::UnsupportedCfaPattern`] /
///   [`Error::MissingColorCalibration`], in that order (CFA before matrix).
///
/// Never panics.
pub fn develop_color_f32_into(sensor: &Sensor, src: &[u16], dst: &mut [f32]) -> Result<(), Error> {
    let pixels = pixel_count(sensor)?;
    let rgb_len = pixels.checked_mul(3).ok_or(Error::RgbBufferWrongLength {
        expected: u64::MAX,
        actual: dst.len(),
    })?;
    let expected_src = usize::try_from(pixels).unwrap_or(usize::MAX);
    let expected_dst = usize::try_from(rgb_len).unwrap_or(usize::MAX);
    if src.len() != expected_src {
        return Err(Error::ColorSourceWrongLength {
            expected: pixels,
            actual: src.len(),
        });
    }
    if dst.len() != expected_dst {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst.len(),
        });
    }

    let mut green = vec![0u16; expected_src];
    let normalize = make_normalizer(sensor, src);
    let (dst_triples, _remainder) = dst.as_chunks_mut::<3>();

    develop_color_fused(sensor, &mut green, &normalize, || {
        let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] =
            build_camera_to_srgb_linear(sensor)?;
        Ok(move |pixel: usize, r: u16, g: u16, b: u16| {
            let rf = f64::from(r);
            let gf = f64::from(g);
            let bf = f64::from(b);
            if let Some([out_r, out_g, out_b]) = dst_triples.get_mut(pixel) {
                // Quantise to the identical integer path, THEN divide by 65535.0
                // — the bit-exact definition of `to_linear_f32_into(quantize(..))`.
                *out_r = f32::from(quantize(m00 * rf + m01 * gf + m02 * bf)) / 65535.0;
                *out_g = f32::from(quantize(m10 * rf + m11 * gf + m12 * bf)) / 65535.0;
                *out_b = f32::from(quantize(m20 * rf + m21 * gf + m22 * bf)) / 65535.0;
            }
        })
    })
}

/// `width·height·3`, the interleaved-RGB sample count, as `u64` — errors (rather
/// than wrapping) if the product overflows, mirroring `demosaic_into`.
fn interleaved_rgb_len(sensor: &Sensor) -> Result<u64, Error> {
    pixel_count(sensor)?
        .checked_mul(3)
        .ok_or(Error::RgbBufferWrongLength {
            expected: u64::MAX,
            actual: 0,
        })
}

/// `width·height`, the mosaic / pixel count, as `u64`.
fn pixel_count(sensor: &Sensor) -> Result<u64, Error> {
    u64::from(sensor.width)
        .checked_mul(u64::from(sensor.height))
        .ok_or(Error::ColorSourceWrongLength {
            expected: u64::MAX,
            actual: 0,
        })
}

/// Build the single camera-linear-RGB → linear-sRGB matrix `M = C · cam2xyz`
/// from `sensor`'s colour metadata (`DEC-033`, `DEC-034`): `C = S · B`
/// (committed), `D = diag(1/asn)`, and `cam2xyz` from the illuminant-interpolated
/// calibration — `FM(g) · D` (both forward matrices present) else
/// `inv(CM(g)) · D` — at the scene weight `g` from [`interpolation_weight`]. When
/// interpolation does not apply (`g` is `None`), `cam2xyz` is built from
/// illuminant 1 alone (`FM1 · D`, else `inv(CM1) · D`), **byte-identical to
/// `SPEC-033`**.
///
/// # Errors
///
/// [`Error::MissingColorCalibration`] if `AsShotNeutral` is absent / ill-shaped
/// / non-positive, neither `ForwardMatrix1` nor `ColorMatrix1` is present and
/// well-shaped, a required rational has a zero denominator, or the
/// (interpolated) `ColorMatrix` is singular.
fn build_camera_to_srgb_linear(sensor: &Sensor) -> Result<[[f64; 3]; 3], Error> {
    let asn = sensor
        .as_shot_neutral
        .as_ref()
        .ok_or(Error::MissingColorCalibration)?;
    let wb = wb_diagonal(asn)?;

    // `SPEC-034`: the scene interpolation weight (DNG `NeutralToXY`, `DEC-034`),
    // found once per image. `None` ⇒ the single-illuminant fallback, which is
    // byte-identical to `SPEC-033` (the regression guard).
    let cam2xyz = match interpolation_weight(sensor) {
        Some(g) => cam_to_xyz_interpolated(sensor, &wb, asn, g)?,
        None => cam_to_xyz_single_illuminant(sensor, &wb, asn)?,
    };

    let c = mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
    Ok(mat_mul(c, cam2xyz))
}

/// The single-illuminant `cam2xyz` — illuminant 1 only, exactly as `SPEC-033`:
/// `FM1 · D` when `ForwardMatrix1` is present and well-shaped, else
/// `inv(CM1) · D`, else [`Error::MissingColorCalibration`]. This is the fallback
/// [`build_camera_to_srgb_linear`] takes when interpolation does not apply, and
/// it is byte-identical to the shipped `SPEC-033` result (a single-calibration
/// sensor is unchanged — the regression guard).
fn cam_to_xyz_single_illuminant(
    sensor: &Sensor,
    wb: &[[f64; 3]; 3],
    asn: &[Rational],
) -> Result<[[f64; 3]; 3], Error> {
    if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
        let fm = matrix_from_rationals(fm1)?;
        Ok(mat_mul(fm, *wb))
    } else if let Some(cm1) = sensor.color_matrix_1.as_ref() {
        let cm = matrix_from_rationals(cm1)?;
        let inv_cm = inv(cm).ok_or(Error::MissingColorCalibration)?;
        cm_path_cam_to_xyz(&inv_cm, asn)
    } else {
        Err(Error::MissingColorCalibration)
    }
}

/// The interpolated `cam2xyz` at a known scene weight `g` (`SPEC-034`). Uses the
/// **ForwardMatrix path** — `FM(g) · D`, `FM(g) = lerp(FM1, FM2, g)` — only when
/// **both** `ForwardMatrix1` and `ForwardMatrix2` are present and well-shaped (an
/// FM pair maps white-balanced camera values to XYZ(D50) directly, `SPEC-033`);
/// an FM at only one illuminant (or an ill-shaped one) is no usable pair, so it
/// falls to the **ColorMatrix path** — `inv(CM(g)) · D`, `CM(g) = lerp(CM1, CM2, g)`.
/// `CM1`/`CM2` are present and well-shaped here ([`interpolation_weight`] checked
/// them to obtain `g`); a singular interpolated `CM(g)` is
/// [`Error::MissingColorCalibration`] — `SPEC-033`'s structure at the
/// interpolated matrix, never a divide by zero.
fn cam_to_xyz_interpolated(
    sensor: &Sensor,
    wb: &[[f64; 3]; 3],
    asn: &[Rational],
    g: f64,
) -> Result<[[f64; 3]; 3], Error> {
    if let (Some(fm1), Some(fm2)) = (
        sensor.forward_matrix_1.as_ref(),
        sensor.forward_matrix_2.as_ref(),
    ) {
        if let (Ok(f1), Ok(f2)) = (matrix_from_rationals(fm1), matrix_from_rationals(fm2)) {
            let fm = lerp_matrix(f1, f2, g);
            return Ok(mat_mul(fm, *wb));
        }
    }

    let cm1 = matrix_from_rationals(
        sensor
            .color_matrix_1
            .as_ref()
            .ok_or(Error::MissingColorCalibration)?,
    )?;
    let cm2 = matrix_from_rationals(
        sensor
            .color_matrix_2
            .as_ref()
            .ok_or(Error::MissingColorCalibration)?,
    )?;
    let cm = lerp_matrix(cm1, cm2, g);
    let inv_cm = inv(cm).ok_or(Error::MissingColorCalibration)?;
    cm_path_cam_to_xyz(&inv_cm, asn)
}

/// Bradford cone-response matrix (XYZ → sharpened RGB), the `M_A` of the
/// Bradford chromatic-adaptation transform. A committed `f64` constant
/// (`DEC-033`), transcribed once from the standard (Lindbloom / CIE).
const BRADFORD_CONE: [[f64; 3]; 3] = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

/// CIE XYZ of the D50 white (the DNG profile connection space white), `Y = 1`.
const D50_WHITE: [f64; 3] = [0.96422, 1.0, 0.82521];

/// `cam2xyz` for the **ColorMatrix** path: `CA(W → D50) · inv(CM)`, where the
/// scene white is `W = inv(CM) · AsShotNeutral` (DNG spec ch. 6, "Mapping Camera
/// Color Space to CIE XYZ Space"). `ColorMatrix` maps XYZ **under the
/// calibration illuminant** to camera values, so `inv(CM)` already yields XYZ
/// of the scene as lit; no white-balance diagonal `D` may be applied (it would
/// count white balance twice and tint neutral magenta). Instead the scene white
/// is Bradford-adapted to D50, so a raw neutral `c = AsShotNeutral` lands on D50
/// white (`Y = 1`), which the committed `C = S · B` maps to linear sRGB
/// `(1, 1, 1)`. Basic `f64` ops only (`DEC-033`): the cone matrix, its inverse
/// and a diagonal of ratios.
///
/// [`Error::MissingColorCalibration`] if the neutral is malformed, or the white
/// is degenerate (non-positive cone responses) or the cone matrix fails to invert.
fn cm_path_cam_to_xyz(inv_cm: &[[f64; 3]; 3], asn: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    let neutral = neutral_vector(asn).ok_or(Error::MissingColorCalibration)?;
    let white = mul_mat_vec(inv_cm, neutral);
    let rho_src = mul_mat_vec(&BRADFORD_CONE, white);
    let rho_dst = mul_mat_vec(&BRADFORD_CONE, D50_WHITE);
    if !(rho_src[0] > 0.0 && rho_src[1] > 0.0 && rho_src[2] > 0.0) {
        return Err(Error::MissingColorCalibration);
    }
    let scale = [
        [rho_dst[0] / rho_src[0], 0.0, 0.0],
        [0.0, rho_dst[1] / rho_src[1], 0.0],
        [0.0, 0.0, rho_dst[2] / rho_src[2]],
    ];
    let cone_inv = inv(BRADFORD_CONE).ok_or(Error::MissingColorCalibration)?;
    let ca = mat_mul(cone_inv, mat_mul(scale, BRADFORD_CONE));
    Ok(mat_mul(ca, *inv_cm))
}

/// Find the scene interpolation weight `g ∈ [0, 1]` by the DNG
/// reciprocal-temperature `NeutralToXY` method (`DEC-034`), or `None` to signal
/// the single-illuminant fallback.
///
/// Preconditions for interpolation: `ColorMatrix1` **and** `ColorMatrix2` present
/// and well-shaped, **and** both `CalibrationIlluminant` codes present and known
/// (in [`ILLUMINANT_CCT`]), with distinct reciprocal temperatures. Then a fixed
/// **3-iteration** loop (dng_sdk's count), starting `g = 0`:
/// 1. `CM(g) = (1−g)·CM1 + g·CM2`;
/// 2. `white = inv(CM(g)) · AsShotNeutral` (the camera neutral's XYZ);
/// 3. `(x, y)` = its chromaticity;
/// 4. `T` = [`mccamy_cct`] of `(x, y)`;
/// 5. `g ← clamp((1/T1 − 1/T) / (1/T1 − 1/T2), 0, 1)`.
///
/// `None` whenever a precondition fails or a step is degenerate (a singular
/// `CM(g)`, an `xy` with sum ≤ 0, a McCamy zero denominator, `T1 == T2`) — the
/// caller then takes the byte-identical single-illuminant path, never a panic or
/// a NaN. The whole computation is `f64` `+ − × ÷` only (McCamy is a polynomial)
/// and runs once per image, off the per-pixel path.
fn interpolation_weight(sensor: &Sensor) -> Option<f64> {
    let cm1 = matrix_from_rationals(sensor.color_matrix_1.as_ref()?).ok()?;
    let cm2 = matrix_from_rationals(sensor.color_matrix_2.as_ref()?).ok()?;
    let t1 = cct_of_illuminant(sensor.calibration_illuminant_1?)?;
    let t2 = cct_of_illuminant(sensor.calibration_illuminant_2?)?;
    let inv_t1 = 1.0 / t1;
    let denom = inv_t1 - 1.0 / t2;
    if denom == 0.0 {
        // `T1 == T2` (or coincident reciprocals): the mired blend is undefined.
        return None;
    }
    let neutral = neutral_vector(sensor.as_shot_neutral.as_ref()?)?;

    let mut g = 0.0;
    for _ in 0..3 {
        let cm = lerp_matrix(cm1, cm2, g);
        let inv_cm = inv(cm)?; // singular interpolated matrix → fallback
        let white = mul_mat_vec(&inv_cm, neutral);
        let (x, y) = xyz_to_xy(white)?; // sum ≤ 0 → fallback
        let t = mccamy_cct(x, y)?; // zero denominator → fallback
        g = ((inv_t1 - 1.0 / t) / denom).clamp(0.0, 1.0);
    }
    Some(g)
}

/// The CCT (kelvin) of a `CalibrationIlluminant` / EXIF `LightSource` code from
/// the committed [`ILLUMINANT_CCT`] table, or `None` for an unrecognised code
/// (which takes the single-illuminant fallback rather than a guessed temperature).
fn cct_of_illuminant(code: u32) -> Option<f64> {
    ILLUMINANT_CCT
        .iter()
        .find(|&&(c, _)| c == code)
        .map(|&(_, t)| t)
}

/// McCamy's cubic CCT from a chromaticity `(x, y)` (`DEC-034`):
/// `n = (x − 0.3320) / (0.1858 − y)`, `T = 449·n³ + 3525·n² + 6823.3·n + 5520.33`.
/// `None` when the denominator `0.1858 − y` is exactly zero (never a divide to
/// `inf`/`NaN`). A polynomial — `n³ = n·n·n`, no `powf` — so it stays `+ − × ÷`
/// only and byte-identical across targets.
fn mccamy_cct(x: f64, y: f64) -> Option<f64> {
    let denom = MCCAMY_EPICENTRE_Y - y;
    if denom == 0.0 {
        return None;
    }
    let n = (x - MCCAMY_EPICENTRE_X) / denom;
    Some(MCCAMY_C3 * n * n * n + MCCAMY_C2 * n * n + MCCAMY_C1 * n + MCCAMY_C0)
}

/// The chromaticity `(x, y) = (X/(X+Y+Z), Y/(X+Y+Z))` of an XYZ triple, or `None`
/// when the sum `X+Y+Z` is non-positive (never a 0-divide).
fn xyz_to_xy(xyz: [f64; 3]) -> Option<(f64, f64)> {
    let [x, y, z] = xyz;
    let sum = x + y + z;
    if sum <= 0.0 {
        return None;
    }
    Some((x / sum, y / sum))
}

/// The element-wise linear interpolation `(1−g)·a + g·b` of two 3×3 matrices,
/// fully unrolled (no indexing). At `g = 0` this is exactly `a` and at `g = 1`
/// exactly `b` (`1.0·x == x`, `0.0·x + x == x` in IEEE), so the endpoints
/// reproduce illuminant 1 / illuminant 2 to the bit.
fn lerp_matrix(a: [[f64; 3]; 3], b: [[f64; 3]; 3], g: f64) -> [[f64; 3]; 3] {
    let [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]] = a;
    let [[b00, b01, b02], [b10, b11, b12], [b20, b21, b22]] = b;
    let w = 1.0 - g;
    [
        [w * a00 + g * b00, w * a01 + g * b01, w * a02 + g * b02],
        [w * a10 + g * b10, w * a11 + g * b11, w * a12 + g * b12],
        [w * a20 + g * b20, w * a21 + g * b21, w * a22 + g * b22],
    ]
}

/// The matrix-vector product `m · v` of a row-major 3×3 and a 3-vector, each dot
/// summed left to right. Only `f64` `+`/`×` (no FMA), byte-identical across
/// targets (`DEC-033`). Used to map the camera neutral through `inv(CM(g))`.
fn mul_mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let [v0, v1, v2] = v;
    [
        m00 * v0 + m01 * v1 + m02 * v2,
        m10 * v0 + m11 * v1 + m12 * v2,
        m20 * v0 + m21 * v1 + m22 * v2,
    ]
}

/// The three `AsShotNeutral` rationals as an `f64` vector `(R, G, B)` — the
/// camera neutral the `NeutralToXY` iteration maps. A length other than 3, or a
/// zero-denominator entry, is `None` (→ the single-illuminant fallback). Unlike
/// [`wb_diagonal`] this does **not** require a positive neutral: the weight
/// iteration only reads the chromaticity, and a non-positive component surfaces
/// later as a degenerate `xy` (sum ≤ 0) → fallback, not here.
fn neutral_vector(asn: &[Rational]) -> Option<[f64; 3]> {
    match asn {
        [r, g, b] => Some([
            rational_to_f64(r).ok()?,
            rational_to_f64(g).ok()?,
            rational_to_f64(b).ok()?,
        ]),
        _ => None,
    }
}

/// The white-balance diagonal `D = diag(1/asn_R, 1/asn_G, 1/asn_B)` from the
/// three positive `AsShotNeutral` rationals. Each diagonal entry is `den/num`
/// (`1 / (num/den)`); a zero numerator or denominator, or a non-positive
/// neutral, is [`Error::MissingColorCalibration`] — never a divide that yields
/// `inf`/`NaN`.
fn wb_diagonal(asn: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    match asn {
        [r, g, b] => Ok([
            [reciprocal_rational(r)?, 0.0, 0.0],
            [0.0, reciprocal_rational(g)?, 0.0],
            [0.0, 0.0, reciprocal_rational(b)?],
        ]),
        _ => Err(Error::MissingColorCalibration),
    }
}

/// `1 / (num/den) = den/num` for a positive neutral. Requires `num > 0` and
/// `den > 0` (`AsShotNeutral` is a positive rational per DNG); anything else is
/// [`Error::MissingColorCalibration`].
fn reciprocal_rational(r: &Rational) -> Result<f64, Error> {
    if r.num <= 0 || r.den <= 0 {
        return Err(Error::MissingColorCalibration);
    }
    Ok(f64::from(r.den) / f64::from(r.num))
}

/// A row-major 3×3 `f64` matrix from exactly 9 rationals. A length other than 9,
/// or a zero-denominator entry, is [`Error::MissingColorCalibration`] (a matrix
/// that cannot be read is unusable calibration, not a panic).
fn matrix_from_rationals(entries: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    match entries {
        [a, b, c, d, e, f, g, h, i] => Ok([
            [
                rational_to_f64(a)?,
                rational_to_f64(b)?,
                rational_to_f64(c)?,
            ],
            [
                rational_to_f64(d)?,
                rational_to_f64(e)?,
                rational_to_f64(f)?,
            ],
            [
                rational_to_f64(g)?,
                rational_to_f64(h)?,
                rational_to_f64(i)?,
            ],
        ]),
        _ => Err(Error::MissingColorCalibration),
    }
}

/// `num/den` as `f64`. A zero denominator is [`Error::MissingColorCalibration`]
/// rather than a divide to `inf`/`NaN` (`DEC-030`'s rationals keep `den` verbatim,
/// zero included). `f64::from` is lossless for `i32`.
fn rational_to_f64(r: &Rational) -> Result<f64, Error> {
    if r.den == 0 {
        return Err(Error::MissingColorCalibration);
    }
    Ok(f64::from(r.num) / f64::from(r.den))
}

/// The matrix product `A · B` of two row-major 3×3 `f64` matrices, fully
/// unrolled: `(A·B)[i][j] = Σ_k A[i][k]·B[k][j]`, each dot summed left to right.
/// Only `f64` `+`/`×` (no FMA), so it is byte-identical across targets (`DEC-033`).
fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]] = a;
    let [[b00, b01, b02], [b10, b11, b12], [b20, b21, b22]] = b;
    [
        [
            a00 * b00 + a01 * b10 + a02 * b20,
            a00 * b01 + a01 * b11 + a02 * b21,
            a00 * b02 + a01 * b12 + a02 * b22,
        ],
        [
            a10 * b00 + a11 * b10 + a12 * b20,
            a10 * b01 + a11 * b11 + a12 * b21,
            a10 * b02 + a11 * b12 + a12 * b22,
        ],
        [
            a20 * b00 + a21 * b10 + a22 * b20,
            a20 * b01 + a21 * b11 + a22 * b21,
            a20 * b02 + a21 * b12 + a22 * b22,
        ],
    ]
}

/// The exact 3×3 cofactor inverse in `f64`, or `None` if the matrix is singular
/// (`det == 0.0`). Used for the `ColorMatrix1` path (XYZ→camera inverted to
/// camera→XYZ); a singular `ColorMatrix` surfaces as
/// [`Error::MissingColorCalibration`] in [`build_camera_to_srgb_linear`], never
/// a divide by zero.
fn inv(m: [[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let [[a, b, c], [d, e, f], [g, h, i]] = m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det == 0.0 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (e * i - f * h) * inv_det,
            (c * h - b * i) * inv_det,
            (b * f - c * e) * inv_det,
        ],
        [
            (f * g - d * i) * inv_det,
            (a * i - c * g) * inv_det,
            (c * d - a * f) * inv_det,
        ],
        [
            (d * h - e * g) * inv_det,
            (b * g - a * h) * inv_det,
            (a * e - b * d) * inv_det,
        ],
    ])
}

/// Apply a built matrix `m` per pixel over interleaved camera RGB into `dst`
/// (`out_i = Σ_j m[i][j]·cam_j`, then [`quantize`]). The buffers are consumed in
/// lock-step 3-sample chunks, so a length mismatch simply stops at the shorter
/// (the callers validate lengths first); every element written is a clamped,
/// rounded `u16`, never a panic or a NaN.
fn apply_matrix_into(m: &[[f64; 3]; 3], camera_rgb: &[u16], dst: &mut [u16]) {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let (src_triples, _) = camera_rgb.as_chunks::<3>();
    let (dst_triples, _) = dst.as_chunks_mut::<3>();
    for (&[r, g, b], [out_r, out_g, out_b]) in src_triples.iter().zip(dst_triples.iter_mut()) {
        let rf = f64::from(r);
        let gf = f64::from(g);
        let bf = f64::from(b);
        *out_r = quantize(m00 * rf + m01 * gf + m02 * bf);
        *out_g = quantize(m10 * rf + m11 * gf + m12 * bf);
        *out_b = quantize(m20 * rf + m21 * gf + m22 * bf);
    }
}

/// Clamp a transform result to `[0, 65535]` **before** rounding half up, so the
/// rounding is well-defined and can never leave `u16` range (`DEC-018`): a
/// negative off-gamut mix pins to `0`, an overshoot to `65535`. `(v + 0.5).floor()`
/// on a value already in `[0, 65535]` is in `[0, 65535]`, so the cast cannot
/// truncate — the same `as u16` pattern [`crate::warp`]'s resampler uses.
fn quantize(value: f64) -> u16 {
    let clamped = value.clamp(0.0, 65535.0);
    (clamped + 0.5).floor() as u16
}

/// Build the per-cell level-normalize map the fused develop path
/// ([`develop_color_fused`]) fills its rolling row cache from (`SPEC-049`,
/// `DEC-042`). Given a mosaic cell `(col, row)` — already reflected into range by
/// the demosaic logic — it returns the per-channel level-normalized value for
/// that cell: `[black, white] → [0, 65535]` by the `black_levels` /
/// `white_levels` entry for the cell's 2×2 position (scalar `black_level` /
/// `white_level`, then `0` / `2^bits-1`, as the fallback), clamped per `DEC-018`.
///
/// This is the per-photosite map the pre-fuse `normalize_mosaic` applied once
/// over a whole scratch plane. The fuse calls it **once per row** as that row
/// enters the rolling cache (not a full scratch plane, and not re-run per
/// neighbour read), so each sample is normalised exactly once. The four slot
/// `(black, white)` pairs are resolved **once** here; the returned closure then
/// does no per-read division to find its slot. For a cell at `(col, row)` the
/// value equals the pre-fuse `normalize_mosaic`'s output at plane index
/// `row·width + col` — same slot, same [`normalize_sample`] — so the fused render
/// stays byte-identical.
///
/// The 2×2 slot is taken **relative to the `ActiveArea` origin** (`DEC-040`): in
/// DNG the per-slot level pattern starts at the active area, not the plane's
/// top-left, so a file with an odd `ActiveArea.top`/`.left` would otherwise
/// subtract the wrong black at every photosite. The offset form
/// `(row − active_top) & 1` is computed with `wrapping_sub`, panic-free and
/// parity-correct even above/left of the origin (the low bit of a wrapping
/// subtraction is still the true parity); for an even origin — the whole existing
/// corpus — it reduces to `row & 1`, so the goldens are unchanged. The demosaic
/// uses the same origin for the CFA phase.
fn make_normalizer<'a>(sensor: &'a Sensor, src: &'a [u16]) -> impl Fn(u32, u32) -> u16 + 'a {
    let width = usize::try_from(sensor.width).unwrap_or(0);
    let (active_top, active_left) = active_area_origin(sensor);
    let active_top = usize::try_from(active_top).unwrap_or(0);
    let active_left = usize::try_from(active_left).unwrap_or(0);
    let white_default = max_value_for_bits(sensor.bits_per_sample).unwrap_or(u32::from(u16::MAX));

    // Resolve the four 2×2-slot `(black, white)` pairs once; `level_at` depends
    // only on the slot, not the pixel, so this is the same value the pre-fuse
    // per-pixel `level_at` would have returned at each slot.
    let mut levels = [(0u32, u32::from(u16::MAX)); 4];
    for (pos, slot) in levels.iter_mut().enumerate() {
        let black = level_at(&sensor.black_levels, pos, sensor.black_level, 0);
        let white = level_at(&sensor.white_levels, pos, sensor.white_level, white_default);
        *slot = (black, white);
    }

    move |x: u32, y: u32| -> u16 {
        let col = usize::try_from(x).unwrap_or(0);
        let row = usize::try_from(y).unwrap_or(0);
        let row_odd = (row.wrapping_sub(active_top) & 1) == 1;
        let col_odd = (col.wrapping_sub(active_left) & 1) == 1;
        let pos = match (row_odd, col_odd) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        let (black, white) = levels.get(pos).copied().unwrap_or((0, u32::from(u16::MAX)));
        let index = row.checked_mul(width).and_then(|rb| rb.checked_add(col));
        let sample = index.and_then(|i| src.get(i)).copied().unwrap_or(0);
        normalize_sample(sample, black, white)
    }
}

/// The level for a 2×2 position: the per-channel array entry at `pos` (falling
/// back to the array's last entry when `pos` is past a shorter array — e.g. a
/// single-value `WhiteLevel`), else the `scalar` fallback, else `default`.
fn level_at(levels: &[u32], pos: usize, scalar: Option<u32>, default: u32) -> u32 {
    match levels.get(pos).or_else(|| levels.last()) {
        Some(&v) => v,
        None => scalar.unwrap_or(default),
    }
}

/// Map one raw sample through `[black, white] → [0, u16::MAX]`, round to
/// nearest, clamped (`DEC-018`) — the per-channel analogue of
/// [`crate::develop`]'s scalar `normalize`, made total: if `black > white`
/// (malformed) or `black == white` the result is `0` rather than a panic.
fn normalize_sample(sample: u16, black: u32, white: u32) -> u16 {
    let value = u32::from(sample);
    // Clamp without panicking even if `black > white` (`u32::clamp` requires
    // `min <= max`): the normal path has `black <= white` and this is identical.
    let clamped = value.clamp(black.min(white), black.max(white));
    let numerator = u64::from(clamped.saturating_sub(black)).saturating_mul(u64::from(u16::MAX));
    let denominator = u64::from(white.saturating_sub(black));
    let half = denominator.checked_div(2).unwrap_or(0);
    let scaled = numerator
        .checked_add(half)
        .and_then(|v| v.checked_div(denominator))
        .unwrap_or(0);
    u16::try_from(scaled).unwrap_or(u16::MAX)
}

/// `2^bits - 1`, without the panicking `pow`/shift the crate-root lint forbids;
/// `None` if `2^bits` does not fit `u32` (mirrors `develop::max_value_for_bits`).
fn max_value_for_bits(bits: u32) -> Option<u32> {
    2u32.checked_pow(bits)?.checked_sub(1)
}

#[cfg(test)]
#[allow(
    // Unit tests are not a decode path exercised on untrusted input
    // (AGENTS.md §11); the same five lints are allowed here as in the other
    // modules' test blocks and in src/bin/irr.rs.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    /// A 3×3 identity, for round-trip / neutral assertions.
    const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    /// Apply a matrix to a single camera triple, returning the quantized RGB —
    /// a one-pixel wrapper over `apply_matrix_into`.
    fn apply_one(m: &[[f64; 3]; 3], rgb: [u16; 3]) -> [u16; 3] {
        let mut out = [0u16; 3];
        apply_matrix_into(m, &rgb, &mut out);
        out
    }

    #[test]
    fn apply_matrix_is_bit_exact_for_injected_matrices() {
        // Identity passes the camera triple through unchanged.
        assert_eq!(apply_one(&IDENTITY, [100, 200, 300]), [100, 200, 300]);

        // A diagonal of 0.5 halves each channel, with an exact round-half-up
        // tie: 2001·0.5 = 1000.5 → 1001, 2003·0.5 = 1001.5 → 1002, 5·0.5 = 2.5 → 3.
        let half = [[0.5, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 0.5]];
        assert_eq!(apply_one(&half, [2001, 2003, 5]), [1001, 1002, 3]);

        // A negative entry can drive a channel below zero → clamps to 0.
        // Row0 = R - G = 100 - 150 = -50 → 0; rows 1/2 are the identity.
        let negative = [[1.0, -1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert_eq!(apply_one(&negative, [100, 150, 7]), [0, 150, 7]);

        // An overshoot clamps to 65535: Row0 = 2·R = 2·40000 = 80000 → 65535.
        let overshoot = [[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert_eq!(apply_one(&overshoot, [40000, 10, 20]), [65535, 10, 20]);
    }

    #[test]
    fn quantize_clamps_before_rounding() {
        assert_eq!(quantize(-1.0), 0); // negative pins to 0
        assert_eq!(quantize(-0.4), 0);
        assert_eq!(quantize(0.5), 1); // half rounds up
        assert_eq!(quantize(0.4999), 0);
        assert_eq!(quantize(2.5), 3);
        assert_eq!(quantize(2.4999), 2);
        assert_eq!(quantize(65535.0), 65535);
        assert_eq!(quantize(65535.5), 65535); // clamp before +0.5, never 65536
        assert_eq!(quantize(70000.0), 65535); // overshoot pins to 65535
    }

    #[test]
    fn inv_3x3_round_trips_and_flags_singular() {
        // An integer upper-triangular matrix with det = 1: its inverse is the
        // integer matrix [[1,-2,5],[0,1,-4],[0,0,1]], exactly representable, so
        // inv(CM)·CM == I to the bit.
        let cm = [[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [0.0, 0.0, 1.0]];
        let inverse = inv(cm).expect("non-singular");
        assert_eq!(
            inverse,
            [[1.0, -2.0, 5.0], [0.0, 1.0, -4.0], [0.0, 0.0, 1.0]]
        );
        assert_eq!(mat_mul(inverse, cm), IDENTITY);

        // A singular matrix (row 1 = 2·row 0) has det 0 → None.
        let singular = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [0.0, 0.0, 1.0]];
        assert!(inv(singular).is_none());
    }

    #[test]
    fn build_matrix_matches_independent_rederivation() {
        // The builder must assemble exactly M = C · cam2xyz, C = S · B, with
        // cam2xyz = FM1·D (forward path) or inv(CM1)·D (matrix path). Re-derive
        // each independently and compare to the bit.

        // Forward path: a synthetic FM1 and asn = (1/2, 1/1, 1/2).
        let fm1_entries = matrix_rationals([
            [60000, 30000, 6422],
            [20000, 70000, 10000],
            [10000, 10000, 62521],
        ]);
        let asn = vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ];
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(fm1_entries.clone()),
            color_matrix_1: None,
            as_shot_neutral: Some(asn.clone()),
        });
        let fm = reference_matrix(&fm1_entries);
        let wb = reference_wb(&asn);
        let c = reference_mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
        let expected = reference_mat_mul(c, reference_mat_mul(fm, wb));
        assert_eq!(build_camera_to_srgb_linear(&sensor).unwrap(), expected);

        // Matrix path: an invertible CM1 and the same asn, no ForwardMatrix.
        let cm1_entries = matrix_rationals([[10000, 2000, 3000], [0, 10000, 4000], [0, 0, 10000]]);
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: Some(cm1_entries.clone()),
            as_shot_neutral: Some(asn.clone()),
        });
        // cam2xyz = CA(W→D50) · inv(CM1) with W = inv(CM1)·asn — no `D` (it would
        // double-count white balance). Re-derive CA by explicit Bradford steps.
        let inv_cm = inv(reference_matrix(&cm1_entries)).expect("invertible");
        let asn_v = [0.5, 1.0, 0.5];
        let w = ref_mul_vec(inv_cm, asn_v);
        let (rs, rd) = (
            ref_mul_vec(BRADFORD_CONE, w),
            ref_mul_vec(BRADFORD_CONE, D50_WHITE),
        );
        let scale = [
            [rd[0] / rs[0], 0.0, 0.0],
            [0.0, rd[1] / rs[1], 0.0],
            [0.0, 0.0, rd[2] / rs[2]],
        ];
        let ca = reference_mat_mul(
            inv(BRADFORD_CONE).expect("cone invertible"),
            reference_mat_mul(scale, BRADFORD_CONE),
        );
        let expected = reference_mat_mul(c, reference_mat_mul(ca, inv_cm));
        let built = build_camera_to_srgb_linear(&sensor).unwrap();
        assert_eq!(built, expected);
        // Defining property: a raw neutral develops to linear-sRGB (1, 1, 1).
        for v in ref_mul_vec(built, asn_v) {
            assert!((v - 1.0).abs() < 1e-6, "grey must map to white, got {v}");
        }
    }

    #[test]
    fn missing_or_unusable_calibration_is_a_typed_error() {
        let asn = vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ];

        // No ForwardMatrix and no ColorMatrix.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral: Some(asn.clone()),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // No AsShotNeutral.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(matrix_rationals([[1, 1, 1]; 3])),
            color_matrix_1: None,
            as_shot_neutral: None,
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // A zero-denominator neutral.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(matrix_rationals([[1, 1, 1]; 3])),
            color_matrix_1: None,
            as_shot_neutral: Some(vec![
                Rational { num: 1, den: 0 },
                Rational { num: 1, den: 1 },
                Rational { num: 1, den: 1 },
            ]),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // A singular ColorMatrix.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: Some(matrix_rationals([
                [10000, 20000, 30000],
                [20000, 40000, 60000],
                [0, 0, 10000],
            ])),
            as_shot_neutral: Some(asn),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));
    }

    // ── test-only reference helpers (independent of the shipped code path) ──

    /// The colour fields a `Sensor` fixture needs for the matrix builder.
    struct ColorMeta {
        forward_matrix_1: Option<Vec<Rational>>,
        color_matrix_1: Option<Vec<Rational>>,
        as_shot_neutral: Option<Vec<Rational>>,
    }

    /// A `Sensor` carrying only colour metadata (everything else absent/empty).
    fn color_sensor(meta: ColorMeta) -> Sensor {
        use crate::ifd::Compression;
        Sensor {
            ifd_index: 0,
            width: 2,
            height: 2,
            bits_per_sample: 16,
            samples_per_pixel: 1,
            photometric: 32803,
            compression: Compression::Uncompressed,
            rows_per_strip: None,
            strip_offsets: vec![],
            strip_byte_counts: vec![],
            black_level: None,
            white_level: None,
            black_level_repeat_dim: None,
            active_area: None,
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            opcode_lists: [false, false, false],
            opcode_list_1: None,
            opcode_list_3: None,
            color_filter: None,
            color_matrix_1: meta.color_matrix_1,
            color_matrix_2: None,
            forward_matrix_1: meta.forward_matrix_1,
            forward_matrix_2: None,
            as_shot_neutral: meta.as_shot_neutral,
            calibration_illuminant_1: None,
            calibration_illuminant_2: None,
            black_levels: vec![],
            white_levels: vec![],
            malformed_tags: vec![],
        }
    }

    /// Nine `num/10000` rationals (row-major) from integer numerators.
    fn matrix_rationals(rows: [[i32; 3]; 3]) -> Vec<Rational> {
        let mut out = Vec::with_capacity(9);
        for row in rows {
            for num in row {
                out.push(Rational { num, den: 10000 });
            }
        }
        out
    }

    /// A reference 3×3 `f64` matrix from 9 rationals — a triple-loop, written
    /// separately from `matrix_from_rationals`.
    fn reference_matrix(entries: &[Rational]) -> [[f64; 3]; 3] {
        let mut m = [[0.0f64; 3]; 3];
        for (idx, r) in entries.iter().enumerate() {
            m[idx / 3][idx % 3] = f64::from(r.num) / f64::from(r.den);
        }
        m
    }

    /// A reference WB diagonal from the three neutral rationals.
    fn reference_wb(asn: &[Rational]) -> [[f64; 3]; 3] {
        let mut d = [[0.0f64; 3]; 3];
        for (idx, r) in asn.iter().enumerate().take(3) {
            d[idx][idx] = f64::from(r.den) / f64::from(r.num);
        }
        d
    }

    /// A reference matrix product via the standard i/j/k triple loop with a
    /// zero accumulator. `0.0 + x == x` in IEEE, so this matches
    /// `mat_mul`'s left-to-right unrolled sum to the bit.
    fn reference_mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut out = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0f64;
                for k in 0..3 {
                    sum += a[i][k] * b[k][j];
                }
                out[i][j] = sum;
            }
        }
        out
    }

    #[test]
    fn reference_mat_mul_agrees_with_mat_mul() {
        // The independent reference must match the shipped `mat_mul` to the bit,
        // so the builder test's expected values are a real cross-check.
        let a = SRGB_FROM_XYZ_D65;
        let b = BRADFORD_D50_TO_D65;
        assert_eq!(reference_mat_mul(a, b), mat_mul(a, b));
    }

    #[test]
    fn normalize_sample_matches_the_scalar_affine_map() {
        // black 100, white 4100 → span 4000; 2100 is the midpoint → 32768.
        assert_eq!(normalize_sample(2100, 100, 4100), 32768);
        assert_eq!(normalize_sample(100, 100, 4100), 0); // at black
        assert_eq!(normalize_sample(4100, 100, 4100), 65535); // at white
        assert_eq!(normalize_sample(50, 100, 4100), 0); // below black clamps
        assert_eq!(normalize_sample(9000, 100, 4100), 65535); // above white clamps
        assert_eq!(normalize_sample(500, 100, 100), 0); // degenerate span, no panic
    }

    // ── SPEC-034: dual-illuminant interpolation (DNG NeutralToXY + McCamy) ──

    /// EXIF/DNG `CalibrationIlluminant` codes used in the fixtures.
    const ILLUMINANT_A: u32 = 17; // Standard light A → 2856 K
    const ILLUMINANT_D65: u32 = 21; // D65 → 6504 K

    /// A `Sensor` with the full dual-calibration colour metadata (reuses
    /// [`color_sensor`] for the inert fields).
    #[allow(clippy::too_many_arguments)]
    fn dual_sensor(
        color_matrix_1: Option<Vec<Rational>>,
        color_matrix_2: Option<Vec<Rational>>,
        forward_matrix_1: Option<Vec<Rational>>,
        forward_matrix_2: Option<Vec<Rational>>,
        as_shot_neutral: Option<Vec<Rational>>,
        calibration_illuminant_1: Option<u32>,
        calibration_illuminant_2: Option<u32>,
    ) -> Sensor {
        let mut s = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral,
        });
        s.color_matrix_1 = color_matrix_1;
        s.color_matrix_2 = color_matrix_2;
        s.forward_matrix_1 = forward_matrix_1;
        s.forward_matrix_2 = forward_matrix_2;
        s.calibration_illuminant_1 = calibration_illuminant_1;
        s.calibration_illuminant_2 = calibration_illuminant_2;
        s
    }

    /// The fixture neutral `(1/2, 1/1, 1/2)`.
    fn asn_half_one_half() -> Vec<Rational> {
        vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ]
    }

    /// The fixture `ColorMatrix1` (warm calibration) — `num/10000`. Built so the
    /// neutral maps toward a warm white; invertible.
    fn fixture_cm1() -> Vec<Rational> {
        matrix_rationals([
            [39216, -17647, -39216],
            [-24510, 73529, -100490],
            [-9804, -70588, 259804],
        ])
    }

    /// The fixture `ColorMatrix2` (cool calibration) — `num/10000`; invertible.
    fn fixture_cm2() -> Vec<Rational> {
        matrix_rationals([
            [35893, -13393, -9821],
            [-6071, 41071, -23214],
            [-32143, -17857, 53571],
        ])
    }

    /// A 3×3 identity as `num/10000` rationals (`inv(identity) == identity`).
    fn identity_rationals() -> Vec<Rational> {
        matrix_rationals([[10000, 0, 0], [0, 10000, 0], [0, 0, 10000]])
    }

    // Independent reference transcription of the NeutralToXY weight iteration —
    // triple-loop lerp / mat-vec and a plainly-written McCamy, separate from the
    // shipped unrolled helpers (the genuine cross-check is `tests/color_oracle.rs`
    // + the perturbed-ColorMatrix2 red-proof; these mirror the file's existing
    // in-module reference convention, which reuses the private `inv`).

    fn ref_lerp(a: [[f64; 3]; 3], b: [[f64; 3]; 3], g: f64) -> [[f64; 3]; 3] {
        let mut o = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                o[i][j] = (1.0 - g) * a[i][j] + g * b[i][j];
            }
        }
        o
    }

    fn ref_mul_vec(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
        let mut o = [0.0f64; 3];
        for i in 0..3 {
            o[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2];
        }
        o
    }

    fn ref_mccamy(x: f64, y: f64) -> f64 {
        let n = (x - 0.3320) / (0.1858 - y);
        449.0 * n * n * n + 3525.0 * n * n + 6823.3 * n + 5520.33
    }

    /// A reference NeutralToXY weight (N=3) from the two ColorMatrices, the
    /// neutral, and the two illuminant CCTs.
    fn ref_weight(cm1: &[Rational], cm2: &[Rational], asn: &[Rational], t1: f64, t2: f64) -> f64 {
        let m1 = reference_matrix(cm1);
        let m2 = reference_matrix(cm2);
        let neutral = [
            f64::from(asn[0].num) / f64::from(asn[0].den),
            f64::from(asn[1].num) / f64::from(asn[1].den),
            f64::from(asn[2].num) / f64::from(asn[2].den),
        ];
        let inv_t1 = 1.0 / t1;
        let denom = inv_t1 - 1.0 / t2;
        let mut g = 0.0f64;
        for _ in 0..3 {
            let cmg = ref_lerp(m1, m2, g);
            let ic = inv(cmg).expect("reference CM(g) invertible");
            let w = ref_mul_vec(ic, neutral);
            let sum = w[0] + w[1] + w[2];
            let x = w[0] / sum;
            let y = w[1] / sum;
            let t = ref_mccamy(x, y);
            g = ((inv_t1 - 1.0 / t) / denom).clamp(0.0, 1.0);
        }
        g
    }

    #[test]
    fn interpolation_weight_matches_dng_neutral_to_xy() {
        let cm1 = fixture_cm1();
        let cm2 = fixture_cm2();
        let asn = asn_half_one_half();
        let sensor = dual_sensor(
            Some(cm1.clone()),
            Some(cm2.clone()),
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );

        let g = interpolation_weight(&sensor).expect("dual calibration interpolates");
        let g_ref = ref_weight(&cm1, &cm2, &asn, 2856.0, 6504.0);

        // Bit-exact against the independent NeutralToXY re-derivation.
        assert_eq!(
            g, g_ref,
            "the weight must match the independent NeutralToXY reference to the bit"
        );
        // A genuine interior blend — not a trivially-clamped endpoint — and the
        // hand-computed value.
        assert!(g > 0.0 && g < 1.0, "expected an interior weight, got {g}");
        assert!(
            (g - 0.5202685520213226).abs() < 1e-9,
            "weight drifted from the hand-computed value: {g}"
        );
    }

    #[test]
    fn weight_clamps_outside_the_calibration_range() {
        // Identity ColorMatrices → the neutral maps to itself (as XYZ), so the
        // McCamy CCT is read straight off the neutral's chromaticity and the
        // weight is a single fixed clamp.
        let identity = identity_rationals();

        // A warm neutral (McCamy ≈ 2714 K, cooler than T1 = 2856 K) → g clamps to 0.
        let warm = vec![
            Rational { num: 6, den: 5 }, // 1.2
            Rational { num: 1, den: 1 }, // 1.0
            Rational { num: 3, den: 5 }, // 0.6
        ];
        let sensor = dual_sensor(
            Some(identity.clone()),
            Some(identity.clone()),
            None,
            None,
            Some(warm),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(
            interpolation_weight(&sensor),
            Some(0.0),
            "a neutral cooler than T1 must clamp to g = 0 (illuminant 1)"
        );

        // A cool neutral (McCamy ≈ 9930 K, hotter than T2 = 6504 K) → g clamps to 1.
        let cool = vec![
            Rational { num: 9, den: 10 }, // 0.9
            Rational { num: 1, den: 1 },  // 1.0
            Rational { num: 7, den: 5 },  // 1.4
        ];
        let sensor = dual_sensor(
            Some(identity.clone()),
            Some(identity),
            None,
            None,
            Some(cool),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(
            interpolation_weight(&sensor),
            Some(1.0),
            "a neutral hotter than T2 must clamp to g = 1 (illuminant 2)"
        );
    }

    #[test]
    fn mccamy_cct_matches_known_chromaticities() {
        // D65 xy → ≈ 6504 K; D50 xy → ≈ 5003 K (McCamy's own exactness).
        let d65 = mccamy_cct(0.31271, 0.32902).expect("D65 is well-defined");
        assert!((d65 - 6504.0).abs() < 5.0, "D65 McCamy CCT off: {d65}");
        let d50 = mccamy_cct(0.34567, 0.35850).expect("D50 is well-defined");
        assert!((d50 - 5002.0).abs() < 5.0, "D50 McCamy CCT off: {d50}");

        // A zero denominator `0.1858 − y` is guarded (no divide to inf/NaN).
        assert_eq!(mccamy_cct(0.4, 0.1858), None);
    }

    #[test]
    fn interpolation_preconditions_that_fall_back() {
        let cm1 = fixture_cm1();
        let cm2 = fixture_cm2();
        let asn = asn_half_one_half();

        // No ColorMatrix2 → cannot interpolate.
        let no_cm2 = dual_sensor(
            Some(cm1.clone()),
            None,
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&no_cm2), None);

        // An unknown CalibrationIlluminant code → cannot interpolate.
        let unknown_illum = dual_sensor(
            Some(cm1.clone()),
            Some(cm2.clone()),
            None,
            None,
            Some(asn.clone()),
            Some(99),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&unknown_illum), None);

        // T1 == T2 (both Standard A) → the mired blend is undefined.
        let same_temp = dual_sensor(
            Some(cm1),
            Some(cm2),
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_A),
        );
        assert_eq!(interpolation_weight(&same_temp), None);
    }

    #[test]
    fn single_illuminant_build_is_byte_identical_to_spec_033() {
        // A ForwardMatrix1 + ColorMatrix1 sensor with NO ColorMatrix2 falls back
        // to the SPEC-033 illuminant-1 path — byte-identical. Build the SPEC-033
        // result independently (C · FM1 · D) and compare to the bit.
        let fm1 = matrix_rationals([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]]);
        let asn = asn_half_one_half();
        let sensor = dual_sensor(
            Some(fixture_cm1()),
            None, // no CM2 ⇒ fallback
            Some(fm1.clone()),
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&sensor), None, "no CM2 ⇒ fallback");

        let c = reference_mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
        let expected = reference_mat_mul(
            c,
            reference_mat_mul(reference_matrix(&fm1), reference_wb(&asn)),
        );
        assert_eq!(
            build_camera_to_srgb_linear(&sensor).unwrap(),
            expected,
            "the single-illuminant fallback must equal SPEC-033's C · FM1 · D to the bit"
        );
    }

    #[test]
    fn degenerate_interpolation_inputs_do_not_panic() {
        let asn = asn_half_one_half();

        // (a) A ColorMatrix2 with a zero-denominator entry ⇒ unusable ⇒ fallback;
        //     the build still succeeds via ForwardMatrix1.
        let mut bad_cm2 = fixture_cm2();
        bad_cm2[0] = Rational { num: 1, den: 0 };
        let fm1 = matrix_rationals([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]]);
        let sensor = dual_sensor(
            Some(fixture_cm1()),
            Some(bad_cm2),
            Some(fm1),
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&sensor), None);
        assert!(build_camera_to_srgb_linear(&sensor).is_ok());

        // (b) A singular ColorMatrix1 (row 1 = 2·row 0), no ForwardMatrix, with a
        //     present ColorMatrix2 and illuminants: the weight iteration's first
        //     inv(CM(0)) = inv(CM1) is singular ⇒ fallback ⇒ the single-illuminant
        //     path also hits the singular CM1 ⇒ typed error, never a panic.
        let singular_cm1 =
            matrix_rationals([[10000, 20000, 30000], [20000, 40000, 60000], [0, 0, 10000]]);
        let sensor = dual_sensor(
            Some(singular_cm1),
            Some(fixture_cm2()),
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // (c) The chromaticity guard: a non-positive XYZ sum is None, not a divide.
        assert_eq!(xyz_to_xy([-1.0, 0.0, 0.0]), None);
        assert_eq!(xyz_to_xy([0.0, 0.0, 0.0]), None);
        assert!(xyz_to_xy([0.3, 0.4, 0.3]).is_some());
    }
}
