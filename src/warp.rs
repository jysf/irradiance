//! `WarpRectilinear` resampler — `SPEC-018`.
//!
//! Applies the radial (and, if ever non-zero, tangential) geometric
//! correction DNG 1.7.0.0 §6.4.1 defines, over a same-size `u16` plane in
//! [`crate::develop::develop_into`]'s representation (`DEC-018`).
//!
//! # The transform (DNG 1.7.0.0 §6.4.1, p.103-104)
//!
//! For destination pixel `(x, y)`, the polynomial gives the SOURCE pixel to
//! sample — this is already an inverse (destination→source) map, exactly
//! what a resampler needs:
//!
//! ```text
//! cx = x0 + cx_hat*(x1-x0)              cy = y0 + cy_hat*(y1-y0)
//! mx = max(|x0-cx|, |x1-cx|)             my = max(|y0-cy|, |y1-cy|)
//! m  = sqrt(mx^2 + my^2)
//! dx = (x-cx)/m                          dy = (y-cy)/m
//! r  = sqrt(dx^2 + dy^2)                 (normalized to 1.0 at the farthest
//!                                         pixel from the optical center —
//!                                         the image corner, when cx_hat =
//!                                         cy_hat = 0.5; SPIKE-001's
//!                                         "r normalizes to 1.0 at the
//!                                         corner" assumption, confirmed
//!                                         against the spec text itself)
//! f  = kr0 + kr1*r^2 + kr2*r^4 + kr3*r^6
//! x' = cx + f*(x-cx) + m*Δxt             y' = cy + f*(y-cy) + m*Δyt
//! ```
//!
//! where `(x0,y0)`/`(x1,y1)` are the top-left/bottom-right pixel coordinates
//! of the warped (destination) image — here, `(0,0)`/`(width-1,height-1)`.
//! The radial term needs only `r^2` (never `r` itself), so this
//! implementation never calls `sqrt`. With `kt0 = kt1 = 0.0` (every measured
//! Q2M frame — `## Non-Goals`), the tangential term `Δxt = Δyt = 0` and the
//! map collapses to `x' = cx + f*(x-cx)`, `y' = cy + f*(y-cy)` exactly.
//!
//! # Out-of-extent pixels — DNG 1.7.0.0 is silent; this build clamps
//!
//! §6.4.1 states the transform but never addresses a source coordinate
//! landing outside `[0, width) x [0, height)` — confirmed by a full-text
//! search of the specification for "clamp"/"outside"/"extrapolat" near the
//! `WarpRectilinear` section (the only other clamp language in the spec
//! governs `WarpRectilinear2`'s domain-restricted radius, a different,
//! DNG-1.6+ opcode this build does not implement). Every REAL Q2M
//! coefficient set measures `f(r) <= kr0 < 1` for all `r` in `[0,1]`
//! (monotonically decreasing from `kr0` at `r=0`), so `|x'-cx| = f*|x-cx| <=
//! |x-cx|`: the source coordinate for every real frame's destination pixel
//! stays inside the source extent by construction. Out-of-extent sampling
//! can only be exercised by a hand-built `WarpRect` with `f(r) > 1`
//! somewhere (`AC7`'s test). Chosen and recorded here per the spec's own
//! invitation to choose when it is silent: **clamp-to-edge** — the sampled
//! coordinate is clamped into `[0, width-1] x [0, height-1]` before
//! interpolation, so an out-of-extent pixel repeats its nearest edge pixel.
//! Alternatives (zero-fill, error) and the measured per-frame scores are in
//! the kernel-choice `DEC-*` (`AC11`).
//!
//! # Kernel — bilinear (pre-registered rule, `## The design decision this
//! spec rests on`)
//!
//! Tried first per the spec's pre-registered rule; shipped because all three
//! decodable Q2M frames scored >= 85 through SPEC-020's oracle with it (see
//! the kernel-choice `DEC-*` for the measured per-frame numbers). Bicubic
//! and Lanczos-3 are the recorded alternatives, not implemented.
//!
//! # Determinism (`DEC-002`)
//!
//! Single-threaded, no `rayon`, no runtime SIMD dispatch, `f64` arithmetic
//! throughout with no data-dependent branching that could vary output
//! byte-for-byte between runs on the same input (`AC12`).
//!
//! # Provenance
//!
//! The transform: DNG 1.7.0.0 §6.4.1, provenance class 1 (published
//! specification). The bilinear kernel: a standard, publicly-documented
//! image-resampling technique (not read from any RAW-decoder
//! implementation), provenance class 1. See `docs/provenance-ledger.md`.

use crate::opcode::WarpRect;
use crate::Error;

/// Whether `warp`'s coefficients produce the identity transform: `kr0 = 1.0`,
/// `kr1 = kr2 = kr3 = kt0 = kt1 = 0.0` exactly (`AC5`). Exact float equality
/// is deliberate — this is a literal-coefficient short-circuit for a camera
/// that writes a no-op warp opcode (or a hand-built test fixture), not a
/// tolerance check; real Q2M coefficients never land here.
pub(crate) fn is_identity(warp: &WarpRect) -> bool {
    warp.kr == [1.0, 0.0, 0.0, 0.0] && warp.kt == [0.0, 0.0]
}

/// Bilinear-sample `src` (a `width x height` plane) at floating-point
/// coordinate `(x, y)`, clamping out-of-extent coordinates to the nearest
/// edge pixel (module docs, "Out-of-extent pixels").
///
/// Rounds to nearest at the `u16` boundary (`DEC-018`'s convention,
/// `(x + 0.5).floor()`), clamped to `[0, 65535]` — bilinear interpolation of
/// in-range `u16` samples cannot overshoot that range, but the clamp keeps
/// this function total rather than trusting the arithmetic.
fn bilinear_sample(src: &[u16], width: u32, height: u32, x: f64, y: f64) -> u16 {
    let max_x = f64::from(width.saturating_sub(1));
    let max_y = f64::from(height.saturating_sub(1));
    let x = x.clamp(0.0, max_x);
    let y = y.clamp(0.0, max_y);

    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = (x0 + 1.0).min(max_x);
    let y1 = (y0 + 1.0).min(max_y);
    let tx = x - x0;
    let ty = y - y0;

    let sample = |xi: f64, yi: f64| -> f64 {
        let xi = xi as u32; // clamped into [0, width-1] above; cast cannot overflow
        let yi = yi as u32; // clamped into [0, height-1] above; cast cannot overflow
        let index = u64::from(yi)
            .checked_mul(u64::from(width))
            .and_then(|v| v.checked_add(u64::from(xi)))
            .and_then(|v| usize::try_from(v).ok());
        f64::from(index.and_then(|i| src.get(i).copied()).unwrap_or(0))
    };

    let v00 = sample(x0, y0);
    let v10 = sample(x1, y0);
    let v01 = sample(x0, y1);
    let v11 = sample(x1, y1);
    let v0 = v00 * (1.0 - tx) + v10 * tx;
    let v1 = v01 * (1.0 - tx) + v11 * tx;
    let v = v0 * (1.0 - ty) + v1 * ty;

    let rounded = (v + 0.5).floor().clamp(0.0, f64::from(u16::MAX));
    // `rounded` is finite and inside `[0, 65535]` by the clamp above.
    rounded as u16
}

/// Inverse-map destination pixel `(x, y)` to its source coordinate, per DNG
/// 1.7.0.0 §6.4.1 (module docs). Never calls `sqrt` — the radial polynomial
/// is expressed in `r^2` throughout.
///
/// # Errors
///
/// [`Error::UnsupportedWarpTangentialTerms`] if `warp.kt != [0.0, 0.0]`
/// (`## Non-Goals`: the pure-radial path is what this build implements).
fn source_coord(
    warp: &WarpRect,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> Result<(f64, f64), Error> {
    if warp.kt != [0.0, 0.0] {
        return Err(Error::UnsupportedWarpTangentialTerms {
            kt0: warp.kt[0],
            kt1: warp.kt[1],
        });
    }

    let x0 = 0.0;
    let y0 = 0.0;
    let x1 = f64::from(width.saturating_sub(1));
    let y1 = f64::from(height.saturating_sub(1));
    let cx = x0 + warp.cx * (x1 - x0);
    let cy = y0 + warp.cy * (y1 - y0);
    let mx = (x0 - cx).abs().max((x1 - cx).abs());
    let my = (y0 - cy).abs().max((y1 - cy).abs());
    let m = mx.hypot(my);

    let fx = f64::from(x);
    let fy = f64::from(y);
    if m == 0.0 {
        // Degenerate 1x1 (or otherwise zero-extent) plane: every pixel IS
        // the optical center, so the identity source coordinate is the only
        // sensible answer and avoids a 0/0 division.
        return Ok((fx, fy));
    }
    let dx = (fx - cx) / m;
    let dy = (fy - cy) / m;
    let r2 = dx * dx + dy * dy;
    let [kr0, kr1, kr2, kr3] = warp.kr;
    let f = kr0 + kr1 * r2 + kr2 * r2 * r2 + kr3 * r2 * r2 * r2;

    // kt0 = kt1 = 0.0 (checked above), so the tangential term is exactly
    // zero and x' = cx + f*(x-cx), y' = cy + f*(y-cy).
    let xp = cx + f * (fx - cx);
    let yp = cy + f * (fy - cy);
    Ok((xp, yp))
}

/// Apply `warp` to `src` (`width x height`), writing the resampled image
/// into `dst` — same shape as `src` (DNG 1.7.0.0 §6.4.1: the warp maps the
/// input extent to itself, `AC6`).
///
/// A no-op ([`is_identity`]) copies `src` into `dst` unchanged, bit-for-bit
/// (`AC5`) — no coordinate arithmetic, no rounding, so there is no floating-
/// point path that could disagree with an exact copy.
///
/// # Errors
///
/// - [`Error::WarpSourceWrongLength`] / [`Error::WarpDestWrongLength`] if
///   `src`/`dst` do not hold exactly `width * height` samples.
/// - [`Error::UnsupportedWarpTangentialTerms`] if `warp.kt != [0.0, 0.0]`.
pub fn apply_warp_into(
    warp: &WarpRect,
    width: u32,
    height: u32,
    src: &[u16],
    dst: &mut [u16],
) -> Result<(), Error> {
    let expected =
        u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(Error::WarpSourceWrongLength {
                expected: u64::MAX,
                actual: src.len(),
            })?;
    let expected_len = usize::try_from(expected).map_err(|_| Error::WarpSourceWrongLength {
        expected,
        actual: src.len(),
    })?;
    if src.len() != expected_len {
        return Err(Error::WarpSourceWrongLength {
            expected,
            actual: src.len(),
        });
    }
    if dst.len() != expected_len {
        return Err(Error::WarpDestWrongLength {
            expected,
            actual: dst.len(),
        });
    }

    if is_identity(warp) {
        dst.copy_from_slice(src);
        return Ok(());
    }

    for y in 0..height {
        for x in 0..width {
            let (sx, sy) = source_coord(warp, width, height, x, y)?;
            let value = bilinear_sample(src, width, height, sx, sy);
            let index = u64::from(y)
                .checked_mul(u64::from(width))
                .and_then(|v| v.checked_add(u64::from(x)))
                .and_then(|v| usize::try_from(v).ok());
            if let Some(slot) = index.and_then(|i| dst.get_mut(i)) {
                *slot = value;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    fn identity_warp() -> WarpRect {
        WarpRect {
            kr: [1.0, 0.0, 0.0, 0.0],
            kt: [0.0, 0.0],
            cx: 0.5,
            cy: 0.5,
            planes: 1,
        }
    }

    #[test]
    fn identity_warp_is_the_identity_transform() {
        let src: Vec<u16> = (0..12u16).collect();
        let mut dst = vec![0u16; 12];
        apply_warp_into(&identity_warp(), 4, 3, &src, &mut dst).expect("fits");
        assert_eq!(dst, src);
    }

    #[test]
    fn wrong_length_buffers_are_rejected_not_panicked() {
        let warp = identity_warp();
        let src = vec![0u16; 11]; // one short for 4x3=12
        let mut dst = vec![0u16; 12];
        assert!(matches!(
            apply_warp_into(&warp, 4, 3, &src, &mut dst),
            Err(Error::WarpSourceWrongLength { .. })
        ));

        let src = vec![0u16; 12];
        let mut dst = vec![0u16; 11];
        assert!(matches!(
            apply_warp_into(&warp, 4, 3, &src, &mut dst),
            Err(Error::WarpDestWrongLength { .. })
        ));
    }

    #[test]
    fn nonzero_tangential_terms_are_a_stop_and_report_finding() {
        let warp = WarpRect {
            kr: [1.0, 0.0, 0.0, 0.0],
            kt: [0.01, 0.0],
            cx: 0.5,
            cy: 0.5,
            planes: 1,
        };
        let src = vec![0u16; 12];
        let mut dst = vec![0u16; 12];
        let err = apply_warp_into(&warp, 4, 3, &src, &mut dst)
            .expect_err("non-zero tangential must be reported, not applied");
        assert!(
            matches!(err, Error::UnsupportedWarpTangentialTerms { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn outside_pixels_follow_the_dng_spec_rule() {
        // kr0 = 3.0 pushes pixel (3,3)'s source to (6.0, 6.0) — the source
        // COORDINATE clamps to (3,3) before any index is computed; a buggy
        // implementation that instead computed a raw out-of-bounds index
        // (18, for a 16-element buffer) and fell back to 0 on a failed
        // lookup would produce 0, not the edge pixel's value, so this
        // discriminates clamp-to-edge from "zero" or an unclamped cast
        // (`6.0 as u32 = 6`, which would also miss the edge value).
        let warp = WarpRect {
            kr: [3.0, 0.0, 0.0, 0.0],
            kt: [0.0, 0.0],
            cx: 0.5,
            cy: 0.5,
            planes: 1,
        };
        let width = 4u32;
        let height = 4u32;
        let mut src = vec![100u16; 16];
        // Bottom-right pixel (3,3) is the edge (6.0, 6.0) clamps to — set it
        // to a distinct value so clamping is observable.
        src[3 * 4 + 3] = 4242;
        let mut dst = vec![0u16; 16];
        apply_warp_into(&warp, width, height, &src, &mut dst).expect("fits");
        assert_eq!(
            dst[3 * 4 + 3],
            4242,
            "clamp-to-edge must sample the edge pixel, not 0 or a wrapped index"
        );
    }

    #[test]
    fn warp_output_is_bit_identical_across_two_runs() {
        let warp = WarpRect {
            kr: [0.999251106, -0.0613765129, -0.0939155414, 0.0558820092],
            kt: [0.0, 0.0],
            cx: 0.5,
            cy: 0.5,
            planes: 1,
        };
        let width = 64u32;
        let height = 48u32;
        let src: Vec<u16> = (0..width * height)
            .map(|i| u16::try_from(i % 4096).unwrap())
            .collect();
        let mut dst1 = vec![0u16; (width * height) as usize];
        let mut dst2 = vec![0u16; (width * height) as usize];
        apply_warp_into(&warp, width, height, &src, &mut dst1).expect("fits");
        apply_warp_into(&warp, width, height, &src, &mut dst2).expect("fits");
        assert_eq!(dst1, dst2, "same input must produce bit-identical output");
    }
}
