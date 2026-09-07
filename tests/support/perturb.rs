//! Synthetic-fault generators for the develop oracle's tier-A red-proof
//! (`SPEC-020`).
//!
//! The oracle lands before the develop pipeline exists (`## The design
//! decision this spec rests on`), so its red-proof cannot score a real
//! render — it perturbs a synthetic reference and scores the perturbed copy
//! against it, exercising the same metric wiring
//! (`tests/support/ssimulacra2.rs`) a real render will later go through.
//! Each function here is one `DEC-005`-calibrated fault class: a positional
//! shift, a radial lens-distortion warp, and a tone-curve gamma exponent.

/// Shift `samples` (`width x height`) right by `dx` columns, clamping at the
/// edge (the shifted-in column repeats the edge value rather than wrapping
/// or zero-filling — the perturbation is purely positional, not also a
/// black-fill artifact at one edge). `AC4`.
pub fn shift_horizontal(width: u32, height: u32, samples: &[u16], dx: i32) -> Vec<u16> {
    let (w, h) = (width as usize, height as usize);
    let mut out = vec![0u16; w * h];
    for y in 0..h {
        for x in 0..w {
            let sx = (x as i64 - i64::from(dx)).clamp(0, w as i64 - 1) as usize;
            out[y * w + x] = samples[y * w + sx];
        }
    }
    out
}

/// IEC 61966-2-1 sRGB EOTF (decode: sRGB-encoded -> linear light).
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// IEC 61966-2-1 sRGB OETF (encode: linear light -> sRGB-encoded).
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Apply a per-pixel gamma exponent to `samples`, **in the linear domain,
/// before sRGB** — `DEC-005`'s calibration methodology exactly (`AC6`):
/// sRGB decode -> `linear.powf(gamma)` -> sRGB encode -> requantize to
/// 16-bit. Models a legitimate tone-curve implementation difference, not a
/// geometry fault — the oracle must ADMIT this one.
pub fn apply_linear_gamma(samples: &[u16], gamma: f32) -> Vec<u16> {
    samples
        .iter()
        .map(|&s| {
            let srgb = f32::from(s) / 65_535.0;
            let linear = srgb_to_linear(srgb).max(0.0).powf(gamma);
            let reencoded = linear_to_srgb(linear).clamp(0.0, 1.0);
            (reencoded * 65_535.0).round() as u16
        })
        .collect()
}

fn bilinear_sample(width: usize, height: usize, samples: &[u16], fx: f32, fy: f32) -> u16 {
    let fx = fx.clamp(0.0, (width - 1) as f32);
    let fy = fy.clamp(0.0, (height - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let at = |x: usize, y: usize| f32::from(samples[y * width + x]);
    let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * tx;
    let bottom = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * tx;
    (top + (bottom - top) * ty).round().clamp(0.0, 65_535.0) as u16
}

/// `SPIKE-001`'s `OpcodeList3` coefficients for the Leica Q2 Monochrom's
/// `WarpRectilinear`: `f(r) = kr0 + kr1 r^2 + kr2 r^4 + kr3 r^6`, radial-only
/// (no tangential term), optical centre `(0.5, 0.5)`. **A measurement, not
/// this file's authority** — re-verify against a fresh `OpcodeList3` parse
/// if the exact numeric coefficients matter (`SPEC-020/## Inputs`).
const WARP_KR0: f32 = 0.999_251_1;
const WARP_KR1: f32 = -0.061_376_51;
const WARP_KR2: f32 = -0.093_915_54;
const WARP_KR3: f32 = 0.055_882_01;

fn warp_scale(r: f32) -> f32 {
    let r2 = r * r;
    WARP_KR0 + WARP_KR1 * r2 + WARP_KR2 * r2 * r2 + WARP_KR3 * r2 * r2 * r2
}

/// Apply `SPIKE-001`'s radial warp to `samples` (`width x height`),
/// simulating a develop pipeline that OMITS `WarpRectilinear`'s correction
/// (`AC5`): radius normalised to `1.0` at the corner (half-diagonal),
/// optical centre at the image centre, sampled bilinearly.
pub fn apply_missing_radial_warp(width: u32, height: u32, samples: &[u16]) -> Vec<u16> {
    let (w, h) = (width as usize, height as usize);
    let cx = (w as f32 - 1.0) / 2.0;
    let cy = (h as f32 - 1.0) / 2.0;
    let half_diagonal = (cx * cx + cy * cy).sqrt();
    let mut out = vec![0u16; w * h];
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let r = if half_diagonal > 0.0 {
                (dx * dx + dy * dy).sqrt() / half_diagonal
            } else {
                0.0
            };
            let scale = warp_scale(r);
            let sx = cx + dx * scale;
            let sy = cy + dy * scale;
            out[y * w + x] = bilinear_sample(w, h, samples, sx, sy);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_by_zero_is_the_identity() {
        let samples = vec![1u16, 2, 3, 4, 5, 6];
        assert_eq!(shift_horizontal(3, 2, &samples, 0), samples);
    }

    #[test]
    fn shift_clamps_at_the_edge_rather_than_wrapping() {
        // 3x1: shifting right by 1 repeats column 0's value at the new
        // column 0 (clamped source index), never wraps column 2's value in.
        let samples = vec![10u16, 20, 30];
        assert_eq!(shift_horizontal(3, 1, &samples, 1), vec![10, 10, 20]);
    }

    #[test]
    fn gamma_one_is_the_identity_up_to_rounding() {
        let samples = vec![0u16, 1000, 32_768, 65_535];
        let out = apply_linear_gamma(&samples, 1.0);
        for (a, b) in samples.iter().zip(out.iter()) {
            let diff = i32::from(*a) - i32::from(*b);
            assert!(
                diff.abs() <= 1,
                "gamma 1.0 should be near-identity: {a} -> {b}"
            );
        }
    }

    #[test]
    fn gamma_above_one_darkens_midtones() {
        // Applying a >1 exponent in the linear domain lowers a midtone
        // value; a pure white or pure black sample is a fixed point.
        let samples = vec![0u16, 32_768, 65_535];
        let out = apply_linear_gamma(&samples, 1.05);
        assert_eq!(out[0], 0);
        assert_eq!(out[2], 65_535);
        assert!(
            out[1] < samples[1],
            "midtone should darken under gamma > 1: {} -> {}",
            samples[1],
            out[1]
        );
    }

    #[test]
    fn warp_leaves_the_optical_centre_unchanged() {
        // r = 0 at the centre pixel -> warp_scale(0) == kr0 != 1.0 in
        // general, but bilinear sampling AT the centre with any nonzero
        // scale still resolves to the centre pixel itself, since dx=dy=0.
        let width = 5u32;
        let height = 5u32;
        let mut samples = vec![100u16; 25];
        samples[12] = 999; // centre of a 5x5 plane, index 2*5+2
        let warped = apply_missing_radial_warp(width, height, &samples);
        assert_eq!(warped[12], 999);
    }

    #[test]
    fn warp_is_the_identity_at_r_equals_kr0_fixed_scale_only_at_centre() {
        // Sanity: warping a perfectly uniform plane changes nothing,
        // regardless of the radial scale function (every sample is equal).
        let samples = vec![42u16; 16 * 16];
        let warped = apply_missing_radial_warp(16, 16, &samples);
        assert_eq!(warped, samples);
    }
}
