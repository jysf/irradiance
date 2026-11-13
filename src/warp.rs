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
//! The rule rests on §6.4.1's silence, searched above, plus `AC7` — the
//! acceptance criterion that pre-registered it — and not on any weighing of
//! candidates. `DEC-024` records the choice in a single Consequences
//! "Neutral" line: clamp-to-edge, on simplicity and determinism grounds,
//! pinned by `AC7`'s test. It records no alternative against it; its
//! "Alternatives Considered" (Options A, B, C) is entirely kernel and oracle
//! threshold. `DEC-024` does record measured oracle scores — `AC8` is
//! **-60.169** on `L1021223.DNG` — and those measurements are what
//! established that the oracle cannot validate this stage at all, for the
//! reason the Kernel section below states.
//!
//! **Under `Framing::FullActiveArea` no visible sample is clamped**
//! (`SPEC-064`, `DEC-059` §3). There the whole active area is the output, so
//! [`crate::develop::develop_with_into`] first checks the (lerped) warp with
//! this module's private `scale_bounds` and `full_active_area_admits`: with
//! every coefficient finite, the optical centre inside the image and
//! `0 <= f <= 1` on `r^2 in [0, 1]`, each source `c + f*(x - c)` is a convex
//! combination of two points inside the image, so the clamp above never
//! decides a pixel. A
//! warp that fails the check is refused before any write. Under the standard
//! crop the clamp behaves as described above, unchanged (`DEC-059` §4).
//!
//! # Kernel — bilinear (pre-registered rule, `## The design decision this
//! spec rests on`)
//!
//! Bilinear is `SPEC-018`'s pre-registered FIRST choice, and it shipped
//! because nothing measured argued for escalating past it — **not** because
//! it scored well through `SPEC-020`'s oracle. That oracle produced no score
//! to ship on: `dnglab`/`rawler` do not implement DNG `OpcodeList`
//! processing, so it cannot see this stage in either direction. `SPEC-021`
//! formally narrowed `SPEC-020`'s oracle scope to exclude warp-bearing pixels
//! (`DEC-026`, amending `DEC-005`) and retired `SPEC-018`'s `#[ignore]`d
//! `AC8`/`AC9` into a passing characterization of that blindness
//! (`tests/warp.rs`'s `warp_pixels_are_below_the_srgb_oracle_baseline`, which
//! measures the warp-bearing score below the no-warp baseline — `DEC-024`
//! Finding 2). The pre-registered escalation trigger was an oracle score
//! below the bar; no such score exists, and a sharper kernel could not have
//! moved a number the reference never computes.
//!
//! What the kernel choice actually rests on is `tests/warp.rs`'s analytic
//! pair, which needs no corpus and no reference decoder: `AC4` reproduces
//! each frame's own `f(1)`-derived corner displacement to within 1 px
//! through this kernel, and `AC10` gives that check teeth — zeroing `kr1`
//! moves the detected peak 339.5 px on `L1021223.DNG`'s coefficients,
//! against a 20 px bar (measured by running the test, not carried forward
//! from a document). Bicubic and Lanczos-3 remain the recorded, unimplemented
//! alternatives; `DEC-024` records why chasing them would have been waste.
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

/// The exact minimum and maximum of `warp`'s backward radial scale
/// `f(s) = kr0 + kr1·s + kr2·s² + kr3·s³` over `s = r²` in `[0, 1]`
/// (`SPEC-064`, `DEC-059` §3) — the quantity [`source_coord`] multiplies
/// `(x − c)` by.
///
/// Closed form, no sampling and no tolerance: the extrema of a cubic on a
/// closed interval lie at `s = 0`, `s = 1`, or a real root of
/// `f'(s) = kr1 + 2·kr2·s + 3·kr3·s²` strictly inside `(0, 1)`. A linear
/// derivative (`kr3 = 0`) and a constant one (`kr2 = kr3 = 0`) are handled
/// without dividing by zero.
///
/// A NaN coefficient yields a NaN bound (the comparisons below propagate it
/// instead of letting `f64::min` drop it), which
/// [`full_active_area_admits`] refuses.
pub(crate) fn scale_bounds(warp: &WarpRect) -> (f64, f64) {
    let [kr0, kr1, kr2, kr3] = warp.kr;
    let f = |s: f64| kr0 + s * (kr1 + s * (kr2 + s * kr3));

    // f'(s) = a + b·s + c·s², divided through by the largest of |kr1|,
    // |kr2|, |kr3| (which leaves its roots unchanged). Without that, finite
    // but huge coefficients overflow `b·b − 4·a·c` to infinity or NaN, the
    // roots are lost, and an interior peak far above 1 passes (SPEC-064
    // verify). Scaled, |a|, |b|, |c| <= 3, so the discriminant is at most 45.
    // A non-finite scale (an infinite or all-NaN tail) finds no roots; the
    // finiteness clause of `full_active_area_admits` refuses that warp.
    let scale = kr1.abs().max(kr2.abs()).max(kr3.abs());
    let (a, b, c) = if scale > 0.0 && scale.is_finite() {
        (kr1 / scale, 2.0 * (kr2 / scale), 3.0 * (kr3 / scale))
    } else {
        (0.0, 0.0, 0.0)
    };
    let mut roots = [f64::NAN; 2];
    if c != 0.0 {
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            // The cancellation-free form: q = -(b + sign(b)·√disc) / 2, then
            // the roots are q / c and a / q.
            let sq = disc.sqrt();
            let q = -0.5 * (b + sq.copysign(b));
            roots[0] = q / c;
            roots[1] = if q != 0.0 { a / q } else { q / c };
        }
    } else if b != 0.0 {
        roots[0] = -a / b;
    }

    let mut min = f(0.0);
    let mut max = min;
    let mut include = |v: f64| {
        if v.is_nan() || min.is_nan() {
            min = f64::NAN;
            max = f64::NAN;
        } else {
            min = min.min(v);
            max = max.max(v);
        }
    };
    include(f(1.0));
    for s in roots {
        // A NaN root (none found) fails both comparisons and is skipped; a
        // NaN coefficient has already made f(1) NaN above.
        if s > 0.0 && s < 1.0 {
            include(f(s));
        }
    }
    (min, max)
}

/// `DEC-059` §3: whether `warp` (already lerped) can be applied over the
/// whole active area with no sample clamped. True exactly when every
/// coefficient and the optical centre are finite, `0 ≤ cx, cy ≤ 1`, and
/// `bounds` (from [`scale_bounds`]) lie within `[0, 1]`.
///
/// Sufficient, not necessary: `r` is normalised to 1 at the pixel farthest
/// from the centre, and with `0 ≤ f ≤ 1` the source `c + f·(x − c)` is a
/// convex combination of `x` and `c`, both inside the image.
///
/// Every comparison is written so a NaN refuses.
pub(crate) fn full_active_area_admits(warp: &WarpRect, bounds: (f64, f64)) -> bool {
    let finite = warp.kr.iter().chain(warp.kt.iter()).all(|v| v.is_finite())
        && warp.cx.is_finite()
        && warp.cy.is_finite();
    let centre_inside = (0.0..=1.0).contains(&warp.cx) && (0.0..=1.0).contains(&warp.cy);
    let (min, max) = bounds;
    finite && centre_inside && min >= 0.0 && max <= 1.0
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
/// A no-op (`is_identity`) copies `src` into `dst` unchanged, bit-for-bit
/// (`AC5`) — no coordinate arithmetic, no rounding, so there is no floating-
/// point path that could disagree with an exact copy.
///
/// # Required processing — the caller validates
///
/// This is the **applier** of `OpcodeList3`'s `WarpRectilinear`, not a develop
/// entry: it takes no [`crate::ifd::Sensor`], so it cannot see the file's other
/// opcodes or whether a list was unreadable (`SPEC-056`, `DEC-053`). A caller
/// composing its own pipeline from this function owns that question and should
/// ask it first with [`crate::opcode::check_required_processing`]
/// (`DevelopPath::Mono` if it also applies `OpcodeList1`'s
/// `FixBadPixelsConstant`, as [`crate::develop::develop_into`] does).
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

    // ── SPEC-064 / DEC-059 §3: the FullActiveArea bounds rule (AC9) ─────────

    fn warp_with(kr: [f64; 4], cx: f64, cy: f64) -> WarpRect {
        WarpRect {
            kr,
            kt: [0.0, 0.0],
            cx,
            cy,
            planes: 1,
        }
    }

    /// `f(s)` at an endpoint, written out here (not via `scale_bounds`).
    fn f_at(kr: [f64; 4], s: f64) -> f64 {
        kr[0] + kr[1] * s + kr[2] * s * s + kr[3] * s * s * s
    }

    #[test]
    fn scale_bounds_match_the_probe() {
        // (label, kr, cx, cy, probe min, probe max) — SPEC-064 §1, α = 1.
        // Every held set has its extrema at the endpoints, so `f(0)` and
        // `f(1)` give the exact expectation; the probe's 9-digit values are
        // checked to their printed precision.
        type ProbeCase = (&'static str, [f64; 4], f64, f64, f64, f64);
        let cases: [ProbeCase; 5] = [
            (
                "L1021223",
                [
                    0.999_251_106,
                    -0.061_376_512_877_263_58,
                    -0.093_915_541_393_360_16,
                    0.055_882_009_215_291_75,
                ],
                0.5,
                0.5,
                0.899_841_061,
                0.999_251_106,
            ),
            (
                "L1026016",
                [
                    0.999_251_106,
                    -0.041_845_169_276_595_75,
                    -0.102_311_478_808_510_63,
                    0.057_876_719_170_212_77,
                ],
                0.5,
                0.5,
                0.912_971_177,
                0.999_251_106,
            ),
            (
                "L1026192",
                [
                    0.999_251_106,
                    -0.032_331_453_292_576_424,
                    -0.104_204_110_174_672_49,
                    0.056_364_293_537_117_91,
                ],
                0.5,
                0.5,
                0.919_079_836,
                0.999_251_106,
            ),
            (
                "Pixel 3a",
                [
                    0.980_158_686_637_878_4,
                    0.091_988_243_162_631_99,
                    -0.177_878_528_833_389_28,
                    0.107_260_853_052_139_28,
                ],
                0.501_746_677_217_029_4,
                0.502_773_708_767_361_2,
                0.980_158_687,
                1.001_529_254,
            ),
            (
                "Pixel 7 Pro",
                [
                    0.989_696_025_848_388_7,
                    0.009_945_581_667_125_225,
                    -0.003_183_106_426_149_606_7,
                    0.003_037_228_714_674_711_2,
                ],
                0.499_835_510_096_680_2,
                0.491_674_211_869_976_9,
                0.989_696_026,
                0.999_495_730,
            ),
        ];
        for (label, kr, cx, cy, probe_min, probe_max) in cases {
            let warp = warp_with(kr, cx, cy);
            let (min, max) = scale_bounds(&warp);
            let ends = (f_at(kr, 0.0), f_at(kr, 1.0));
            let (want_min, want_max) = (ends.0.min(ends.1), ends.0.max(ends.1));
            assert!(
                (min - want_min).abs() < 1e-12,
                "{label}: min {min} vs {want_min}"
            );
            assert!(
                (max - want_max).abs() < 1e-12,
                "{label}: max {max} vs {want_max}"
            );
            assert!(
                (min - probe_min).abs() < 1e-9,
                "{label}: min {min} vs probe {probe_min}"
            );
            assert!(
                (max - probe_max).abs() < 1e-9,
                "{label}: max {max} vs probe {probe_max}"
            );
            let admitted = full_active_area_admits(&warp, (min, max));
            assert_eq!(
                admitted,
                label != "Pixel 3a",
                "{label}: admitted {admitted}"
            );
        }

        // An interior peak: f(0) = f(1) = 0.9, f(0.5) = 1.05 (spec Notes).
        let peak = warp_with([0.9, 0.6, -0.6, 0.0], 0.5, 0.5);
        let (min, max) = scale_bounds(&peak);
        assert!((max - 1.05).abs() < 1e-12, "interior max {max}");
        assert!((min - 0.9).abs() < 1e-12, "interior min {min}");
        assert!(!full_active_area_admits(&peak, (min, max)));

        // A true cubic with both an interior maximum and an interior minimum:
        // f(s) = 0.5 + s(s - 0.3)(s - 1) = 0.5 + 0.3 s - 1.3 s^2 + s^3, so
        // f(0) = f(1) = 0.5 with a bump above between 0 and 0.3 and a dip
        // below between 0.3 and 1. f'(s) = 0.3 - 2.6 s + 3 s^2, roots
        // s = (2.6 -/+ sqrt(3.16)) / 6.
        let kr = [0.5, 0.3, -1.3, 1.0];
        let disc: f64 = 2.6 * 2.6 - 4.0 * 3.0 * 0.3;
        let roots = [(2.6 - disc.sqrt()) / 6.0, (2.6 + disc.sqrt()) / 6.0];
        let values = [
            f_at(kr, 0.0),
            f_at(kr, 1.0),
            f_at(kr, roots[0]),
            f_at(kr, roots[1]),
        ];
        let want_min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let want_max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (min, max) = scale_bounds(&warp_with(kr, 0.5, 0.5));
        assert!(
            (min - want_min).abs() < 1e-12,
            "two-root min {min} vs {want_min}"
        );
        assert!(
            (max - want_max).abs() < 1e-12,
            "two-root max {max} vs {want_max}"
        );
        assert!(
            want_max > f_at(kr, 0.0).max(f_at(kr, 1.0)),
            "the max is interior"
        );
        assert!(
            want_min < f_at(kr, 0.0).min(f_at(kr, 1.0)),
            "the min is interior"
        );
    }

    #[test]
    fn scale_bounds_degenerate_cubics() {
        // kr3 = 0: a linear derivative (the interior-peak fixture above is
        // also this shape).
        let (min, max) = scale_bounds(&warp_with([1.0, -0.2, 0.1, 0.0], 0.5, 0.5));
        assert!(
            (min - 0.9).abs() < 1e-12 && (max - 1.0).abs() < 1e-12,
            "{min} {max}"
        );
        // kr2 = kr3 = 0: a constant derivative.
        let (min, max) = scale_bounds(&warp_with([1.0, -0.1, 0.0, 0.0], 0.5, 0.5));
        assert!(
            (min - 0.9).abs() < 1e-12 && (max - 1.0).abs() < 1e-12,
            "{min} {max}"
        );
        // kr1 = kr2 = kr3 = 0: a zero derivative, f constant.
        let (min, max) = scale_bounds(&warp_with([0.75, 0.0, 0.0, 0.0], 0.5, 0.5));
        assert_eq!((min, max), (0.75, 0.75));
        // A double root (zero discriminant) at s = 0.5: f' = 3(s - 0.5)^2.
        let (min, max) = scale_bounds(&warp_with([0.5, 0.75, -1.5, 1.0], 0.5, 0.5));
        assert!(
            (min - 0.5).abs() < 1e-12 && (max - 0.75).abs() < 1e-12,
            "{min} {max}"
        );
        // Negative discriminant: no real root, endpoints only.
        let (min, max) = scale_bounds(&warp_with([0.5, 0.1, 0.0, 0.1], 0.5, 0.5));
        assert!(
            (min - 0.5).abs() < 1e-12 && (max - 0.7).abs() < 1e-12,
            "{min} {max}"
        );
        // The identity and lerp(0) of any warp are admitted.
        let identity = warp_with([1.0, 0.0, 0.0, 0.0], 0.5, 0.5);
        assert!(full_active_area_admits(&identity, scale_bounds(&identity)));
        // f < 0 somewhere is refused; so is a centre outside the image.
        let negative = warp_with([-0.5, 0.0, 0.0, 0.0], 0.5, 0.5);
        assert!(!full_active_area_admits(&negative, scale_bounds(&negative)));
        let off = warp_with([1.0, -0.1, 0.0, 0.0], 1.5, 0.5);
        assert!(!full_active_area_admits(&off, scale_bounds(&off)));
        let off = warp_with([1.0, -0.1, 0.0, 0.0], 0.5, -0.1);
        assert!(!full_active_area_admits(&off, scale_bounds(&off)));
        // A centre on the image edge is inside.
        let edge = warp_with([1.0, -0.1, 0.0, 0.0], 0.0, 1.0);
        assert!(full_active_area_admits(&edge, scale_bounds(&edge)));
        // The scale's own bounds are inclusive too: f falls from 1 at the
        // centre to exactly 0 at the corner (every corner pixel samples the
        // centre), so DEC-059 §3's 0 <= f <= 1 admits it (SPEC-064 verify).
        let to_zero = warp_with([1.0, -1.0, 0.0, 0.0], 0.5, 0.5);
        assert_eq!(scale_bounds(&to_zero), (0.0, 1.0));
        assert!(full_active_area_admits(&to_zero, scale_bounds(&to_zero)));
    }

    #[test]
    fn scale_bounds_huge_coefficients_keep_the_interior_peak() {
        // SPEC-064 verify: f(0) = f(1) = 0.5 (the s^2 and s^3 terms cancel at
        // s = 1), but f(2/3) = 0.5 + k·4/27, far above 1. Unscaled, the
        // derivative's discriminant overflows (b·b for k = 1e154 and up; b
        // itself at k = 1e308), the interior root is lost, and the bounds
        // came back as [0.5, 0.5], so the predicate admitted the warp.
        for k in [1e153, 1e154, 1e200, 1e308] {
            let warp = warp_with([0.5, 0.0, k, -k], 0.5, 0.5);
            let (min, max) = scale_bounds(&warp);
            assert_eq!(min, 0.5, "k = {k:e}: min");
            let want = 0.5 + k * (4.0 / 27.0);
            assert!(
                ((max - want) / want).abs() < 1e-12,
                "k = {k:e}: max {max:e}, want {want:e}"
            );
            assert!(!full_active_area_admits(&warp, (min, max)), "k = {k:e}");
        }
    }

    #[test]
    fn scale_bounds_nan_refuses() {
        for i in 0..4 {
            let mut kr = [1.0, -0.1, 0.0, 0.0];
            kr[i] = f64::NAN;
            let warp = warp_with(kr, 0.5, 0.5);
            let bounds = scale_bounds(&warp);
            assert!(
                bounds.0.is_nan() || bounds.1.is_nan(),
                "kr[{i}] NaN: bounds {bounds:?} must carry the NaN"
            );
            assert!(!full_active_area_admits(&warp, bounds), "kr[{i}] NaN");
            // Even if a caller hands in clean bounds, the warp's own NaN
            // refuses.
            assert!(!full_active_area_admits(&warp, (0.5, 0.9)), "kr[{i}] NaN");
        }
        for (cx, cy) in [(f64::NAN, 0.5), (0.5, f64::NAN), (f64::INFINITY, 0.5)] {
            let warp = warp_with([1.0, -0.1, 0.0, 0.0], cx, cy);
            assert!(
                !full_active_area_admits(&warp, scale_bounds(&warp)),
                "{cx} {cy}"
            );
        }
        let mut kt = warp_with([1.0, -0.1, 0.0, 0.0], 0.5, 0.5);
        kt.kt = [f64::NAN, 0.0];
        assert!(!full_active_area_admits(&kt, scale_bounds(&kt)), "kt NaN");
        // NaN bounds on a clean warp refuse too.
        let clean = warp_with([1.0, -0.1, 0.0, 0.0], 0.5, 0.5);
        assert!(!full_active_area_admits(&clean, (f64::NAN, 0.9)));
        assert!(!full_active_area_admits(&clean, (0.5, f64::NAN)));
    }
}
