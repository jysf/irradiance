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
//!    `cam2xyz = inv(CM1) · D` — a single-illuminant simplification that treats
//!    the `ColorMatrix` output as XYZ(D50) directly (`DEC-033`; refined by
//!    `SPEC-034`, judged by `SPEC-035`'s ΔE).
//! 3. **XYZ(D50) → linear sRGB**: the committed constant `C = S · B`, where
//!    [`BRADFORD_D50_TO_D65`] (`B`) is the Bradford chromatic adaptation and
//!    [`SRGB_FROM_XYZ_D65`] (`S`) is IEC 61966-2-1's linear-sRGB-from-XYZ(D65).
//!
//! `M = C · cam2xyz` (`= C · FM1 · D`, or `C · inv(CM1) · D`).
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
//! caller-owned `dst`. [`develop_color_into`] allocates internal scratch (the
//! normalized mosaic and the camera-RGB intermediate), exactly as
//! `develop_into`'s warp path allocates; the public `src`/`dst` stay
//! caller-owned.
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

use crate::demosaic::demosaic_into;
use crate::ifd::{Rational, Sensor};
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
/// interleaved **linear-sRGB** `u16` (`width·height·3`): the compose of
/// per-channel level-normalize → [`crate::demosaic::demosaic_into`] →
/// [`to_srgb_linear_into`] (`SPEC-033`).
///
/// The per-channel level-normalize runs on the **mosaic, before demosaic**:
/// black-subtract is per CFA colour and does not commute with interpolation, so
/// each photosite is normalized by the `black_levels`/`white_levels` entry for
/// its 2×2 position (scalar `black_level`/`white_level` fallback, then
/// `0`/`2^bits-1`), clamped per `DEC-018`. `demosaic_into` then runs on the
/// normalized values unchanged, and `to_srgb_linear_into` applies the colour
/// matrix.
///
/// The public `src`/`dst` are caller-owned (`DEC-016`); the normalized mosaic
/// and the camera-RGB intermediate are internal scratch, as `develop_into`'s
/// warp path allocates.
///
/// # Errors
///
/// - [`Error::ColorSourceWrongLength`] if `src.len() != width·height`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width·height·3`.
/// - whatever [`crate::demosaic::demosaic_into`] returns (a non-CFA plane, an
///   unsupported mosaic) or [`to_srgb_linear_into`] returns
///   ([`Error::MissingColorCalibration`]).
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

    // (1) per-channel level-normalize the mosaic into a width·height scratch.
    let mut normalized = vec![0u16; expected_src];
    normalize_mosaic(sensor, src, &mut normalized);

    // (2) demosaic the normalized mosaic into a camera-RGB scratch.
    let mut camera_rgb = vec![0u16; expected_dst];
    demosaic_into(sensor, &normalized, &mut camera_rgb)?;

    // (3) apply the colour matrix: camera-RGB → linear-sRGB `dst`.
    to_srgb_linear_into(sensor, &camera_rgb, dst)
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
/// from `sensor`'s colour metadata (`DEC-033`):
/// `C = S · B` (committed), `cam2xyz = FM1 · D` when `ForwardMatrix1` is present
/// else `inv(CM1) · D`, `D = diag(1/asn)`.
///
/// # Errors
///
/// [`Error::MissingColorCalibration`] if `AsShotNeutral` is absent / ill-shaped
/// / non-positive, neither `ForwardMatrix1` nor `ColorMatrix1` is present and
/// well-shaped, a required rational has a zero denominator, or the
/// `ColorMatrix` is singular.
fn build_camera_to_srgb_linear(sensor: &Sensor) -> Result<[[f64; 3]; 3], Error> {
    let asn = sensor
        .as_shot_neutral
        .as_ref()
        .ok_or(Error::MissingColorCalibration)?;
    let wb = wb_diagonal(asn)?;

    let cam2xyz = if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
        let fm = matrix_from_rationals(fm1)?;
        mat_mul(fm, wb)
    } else if let Some(cm1) = sensor.color_matrix_1.as_ref() {
        let cm = matrix_from_rationals(cm1)?;
        let inv_cm = inv(cm).ok_or(Error::MissingColorCalibration)?;
        mat_mul(inv_cm, wb)
    } else {
        return Err(Error::MissingColorCalibration);
    };

    let c = mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
    Ok(mat_mul(c, cam2xyz))
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

/// Per-channel level-normalize a Bayer mosaic into `dst` (same length): each
/// photosite is mapped `[black, white] → [0, 65535]` by the `black_levels` /
/// `white_levels` entry for its 2×2 position, with the scalar `black_level` /
/// `white_level` (then `0` / `2^bits-1`) as the fallback. Runs on the mosaic
/// **before** demosaic because per-colour black-subtract does not commute with
/// interpolation.
fn normalize_mosaic(sensor: &Sensor, src: &[u16], dst: &mut [u16]) {
    let width = usize::try_from(sensor.width).unwrap_or(0);
    let white_default = max_value_for_bits(sensor.bits_per_sample).unwrap_or(u32::from(u16::MAX));
    for (pixel, (out, &sample)) in dst.iter_mut().zip(src.iter()).enumerate() {
        let row = pixel.checked_div(width).unwrap_or(0);
        let col = pixel.checked_rem(width).unwrap_or(0);
        let pos = match (row & 1, col & 1) {
            (0, 0) => 0,
            (0, _) => 1,
            (_, 0) => 2,
            (_, _) => 3,
        };
        let black = level_at(&sensor.black_levels, pos, sensor.black_level, 0);
        let white = level_at(&sensor.white_levels, pos, sensor.white_level, white_default);
        *out = normalize_sample(sample, black, white);
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
        let inv_cm = inv(reference_matrix(&cm1_entries)).expect("invertible");
        let expected = reference_mat_mul(c, reference_mat_mul(inv_cm, wb));
        assert_eq!(build_camera_to_srgb_linear(&sensor).unwrap(), expected);
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
}
