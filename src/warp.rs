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

/// Whether every coefficient and the optical centre of `warp` is finite.
fn is_finite(warp: &WarpRect) -> bool {
    warp.kr.iter().chain(warp.kt.iter()).all(|v| v.is_finite())
        && warp.cx.is_finite()
        && warp.cy.is_finite()
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
    let finite = is_finite(warp);
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
    bilinear_sample_with(width, height, x, y, |xi, yi| {
        let index = u64::from(yi)
            .checked_mul(u64::from(width))
            .and_then(|v| v.checked_add(u64::from(xi)))
            .and_then(|v| usize::try_from(v).ok());
        index.and_then(|i| src.get(i).copied()).unwrap_or(0)
    })
}

/// [`bilinear_sample`]'s kernel (`DEC-024`) over a `width x height` plane whose
/// sample at `(xi, yi)` (both already clamped into the plane) is `at(xi, yi)`:
/// the one arithmetic both the warp and the sensor-geometry CA step
/// (`SPEC-099`, `DEC-087` item 3) use, so neither can drift from the other.
fn bilinear_sample_with(
    width: u32,
    height: u32,
    x: f64,
    y: f64,
    at: impl Fn(u32, u32) -> u16,
) -> u16 {
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
        f64::from(at(xi, yi))
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
/// - [`Error::NonFiniteWarpCoefficients`] if any coefficient or the optical
///   centre is NaN or infinite (checked before the identity shortcut).
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

    if !is_finite(warp) {
        return Err(Error::NonFiniteWarpCoefficients);
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

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-099, DEC-087 item 3: red and blue resampled to green's geometry
// ─────────────────────────────────────────────────────────────────────────────

/// `T`, the number of intervals of the CA ratio table (`DEC-087` item 3).
const CA_TABLE_INTERVALS: usize = 4096;

/// `T` as `f64`.
const CA_TABLE_INTERVALS_F64: f64 = 4096.0;

/// `f(k, s) = k0 + k1·s + k2·s·s + k3·s·s·s`, in exactly this operation order
/// (`DEC-087` item 3; not [`scale_bounds`]' Horner form).
fn radial_scale(k: &[f64; 4], s: f64) -> f64 {
    let [k0, k1, k2, k3] = *k;
    k0 + k1 * s + k2 * s * s + k3 * s * s * s
}

/// The fold check's minimum (`DEC-087` item 3): green's radial map
/// `ρ = r·f_g(r²)` is strictly increasing on `r ∈ [0, 1]` exactly when
/// `q(s) = kr0 + 3·kr1·s + 5·kr2·s² + 7·kr3·s³ > 0` on `s ∈ [0, 1]`, and this is
/// `q`'s minimum there: [`scale_bounds`] (closed form, `DEC-059` §3) of the
/// coefficients `[kr0, 3·kr1, 5·kr2, 7·kr3]`. NaN when a coefficient is NaN.
fn fold_slope_min(green: &WarpRect) -> f64 {
    let [k0, k1, k2, k3] = green.kr;
    let q = WarpRect {
        kr: [k0, 3.0 * k1, 5.0 * k2, 7.0 * k3],
        ..*green
    };
    scale_bounds(&q).0
}

/// The table node `s_j` for `σ_j` (`DEC-087` item 3): `1` at and beyond green's
/// rim (`σ ≥ f_g(1)·f_g(1)`), else the bisection of `h(s) = s·f_g(s)·f_g(s)`
/// against `σ` on `[0, 1]`, run until the midpoint equals an end.
fn table_node(green: &[f64; 4], sigma: f64) -> f64 {
    let rim = radial_scale(green, 1.0);
    if sigma >= rim * rim {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    loop {
        let mid = 0.5 * (lo + hi);
        // Halving an interval of doubles reaches an end in at most about 1100
        // steps; a NaN `h` compares false and moves `hi`, so it ends too.
        if mid == lo || mid == hi {
            break;
        }
        let fm = radial_scale(green, mid);
        if mid * fm * fm < sigma {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The ratio `k` at `σ` from a table `t[0..=T]` (`DEC-087` item 3):
/// `u = min(σ·T, T)`, `j = min(⌊u⌋, T − 1)`, `k = t[j] + (t[j+1] − t[j])·(u − j)`.
fn ratio(table: &[f64], sigma: f64) -> f64 {
    let u = (sigma * CA_TABLE_INTERVALS_F64).min(CA_TABLE_INTERVALS_F64);
    // `u` is in [0, T] (σ ≥ 0), so the cast is exact; a NaN casts to 0.
    let j = (u.floor() as usize).min(CA_TABLE_INTERVALS.saturating_sub(1));
    let t0 = table.get(j).copied().unwrap_or(1.0);
    let t1 = table.get(j.saturating_add(1)).copied().unwrap_or(t0);
    t0 + (t1 - t0) * (u - j as f64)
}

/// The CA step of the sensor-geometry entries, resolved once per call
/// (`SPEC-099`, `DEC-087` item 3): the `ActiveArea`, the optical centre and
/// `m`, the ratio table `t_c` of each plane that is resampled (red, blue), and
/// the halo `H` in rows.
pub(crate) struct ChromaticShift {
    /// The `ActiveArea`'s origin in plane coordinates and its size.
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    /// The optical centre and `m`, in `ActiveArea`-relative coordinates.
    cx: f64,
    cy: f64,
    m: f64,
    /// `t_c[0..=T]` for red (plane 0) and blue (plane 2); `None` for a plane
    /// whose set equals green's, which is not resampled.
    tables: [Option<Vec<f64>>; 2],
    /// `H = ⌈B⌉ + 1`, capped at the `ActiveArea`'s height.
    halo: usize,
}

impl ChromaticShift {
    /// Resolve the step for green's set and the red and blue sets that differ
    /// from it (`None`: equal to green's), over the `ActiveArea`
    /// `(left, top, width, height)`.
    ///
    /// `Ok(None)` when there is nothing to do: no plane differs, or `m == 0`
    /// (a one-pixel `ActiveArea`: every pixel is the optical centre).
    ///
    /// # Errors
    ///
    /// [`Error::UnsupportedWarpFold`] when [`fold_slope_min`] is not `> 0`.
    pub(crate) fn new(
        green: &WarpRect,
        red: Option<&WarpRect>,
        blue: Option<&WarpRect>,
        (left, top, width, height): (u32, u32, u32, u32),
    ) -> Result<Option<ChromaticShift>, Error> {
        if red.is_none() && blue.is_none() {
            return Ok(None);
        }
        // Geometry exactly as `source_coord`, over the ActiveArea.
        let x1 = f64::from(width.saturating_sub(1));
        let y1 = f64::from(height.saturating_sub(1));
        let cx = green.cx * x1;
        let cy = green.cy * y1;
        let mx = cx.abs().max((x1 - cx).abs());
        let my = cy.abs().max((y1 - cy).abs());
        let m = mx.hypot(my);
        if m == 0.0 {
            return Ok(None);
        }

        let slope_min = fold_slope_min(green);
        // A NaN refuses too.
        if slope_min.is_nan() || slope_min <= 0.0 {
            return Err(Error::UnsupportedWarpFold { slope_min });
        }

        // The nodes are shared by both planes; each plane divides its own
        // scale by green's at every node.
        let nodes: Vec<f64> = (0..=CA_TABLE_INTERVALS)
            .map(|j| table_node(&green.kr, j as f64 / CA_TABLE_INTERVALS_F64))
            .collect();
        let table = |set: &WarpRect| -> Vec<f64> {
            nodes
                .iter()
                .map(|&s| radial_scale(&set.kr, s) / radial_scale(&green.kr, s))
                .collect()
        };
        let tables = [red.map(table), blue.map(table)];

        // B = m · max_j max(|t[j] − 1|, |t[j+1] − 1|) · √((j+1)/T): on interval
        // j the ratio is linear between its nodes and √σ ≤ √((j+1)/T), so no
        // source row lies farther than B from its pixel's row.
        let mut bound = 0.0f64;
        for t in tables.iter().flatten() {
            for (j, pair) in t.windows(2).enumerate() {
                let [a, b] = pair else { continue };
                let reach = (a - 1.0).abs().max((b - 1.0).abs());
                let radius = ((j as f64 + 1.0) / CA_TABLE_INTERVALS_F64).sqrt();
                // `max` would drop a NaN; keep it, so the halo falls back to
                // the whole ActiveArea below.
                let term = m * reach * radius;
                bound = if term.is_nan() || bound.is_nan() {
                    f64::NAN
                } else {
                    bound.max(term)
                };
            }
        }
        let rows = usize::try_from(height).unwrap_or(usize::MAX);
        let halo = if bound.is_finite() && bound < f64::from(height) {
            // `bound` is finite, non-negative and below `height`, so the cast
            // is exact.
            (bound.ceil() as usize).saturating_add(1).min(rows)
        } else {
            rows
        };

        Ok(Some(ChromaticShift {
            left,
            top,
            width,
            height,
            cx,
            cy,
            m,
            tables,
            halo,
        }))
    }

    /// `H`: the rows on each side of a pixel's row its resampled value reads.
    pub(crate) fn halo(&self) -> usize {
        self.halo
    }

    /// Whether plane pixel `(x, y)` lies inside the `ActiveArea` (the masked
    /// border passes through unresampled).
    pub(crate) fn contains(&self, x: u32, y: u32) -> bool {
        x.checked_sub(self.left).is_some_and(|ax| ax < self.width)
            && y.checked_sub(self.top).is_some_and(|ay| ay < self.height)
    }

    /// The resampled value of camera plane `c` (0 red, 2 blue) at plane pixel
    /// `(x, y)` inside the `ActiveArea`, or `None` when that plane is not
    /// resampled. `at(px, py)` reads plane `c`'s camera sample at **plane**
    /// coordinates; every read lies inside the `ActiveArea` and within
    /// [`ChromaticShift::halo`] rows of `y`.
    ///
    /// `DEC-087` item 3: `σ = dx·dx + dy·dy`, `u = min(σ·T, T)`,
    /// `j = min(⌊u⌋, T − 1)`, `k = t[j] + (t[j+1] − t[j])·(u − j)`, the source
    /// `(cx + k·(x − cx), cy + k·(y − cy))`, and [`bilinear_sample`]'s kernel
    /// clamped to the `ActiveArea`, rounded half up.
    pub(crate) fn value(
        &self,
        c: usize,
        x: u32,
        y: u32,
        at: impl Fn(u32, u32) -> u16,
    ) -> Option<u16> {
        let table = match c {
            0 => self.tables.first(),
            2 => self.tables.get(1),
            _ => None,
        }?
        .as_deref()?;
        let fx = f64::from(x.wrapping_sub(self.left));
        let fy = f64::from(y.wrapping_sub(self.top));
        let dx = (fx - self.cx) / self.m;
        let dy = (fy - self.cy) / self.m;
        let k = ratio(table, dx * dx + dy * dy);
        let sx = self.cx + k * (fx - self.cx);
        let sy = self.cy + k * (fy - self.cy);
        let (left, top) = (self.left, self.top);
        Some(bilinear_sample_with(
            self.width,
            self.height,
            sx,
            sy,
            |xi, yi| at(left.saturating_add(xi), top.saturating_add(yi)),
        ))
    }
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
    fn non_finite_warp_is_refused_not_resampled_to_black() {
        let src = vec![100u16, 200, 300, 400];
        for bad in [f64::NAN, f64::INFINITY] {
            let mut warps = Vec::new();
            for i in 0..4 {
                let mut w = identity_warp();
                w.kr[i] = bad;
                warps.push(w);
            }
            let mut w = identity_warp();
            w.cx = bad;
            warps.push(w);
            let mut w = identity_warp();
            w.cy = bad;
            warps.push(w);
            for warp in warps {
                let mut dst = vec![7u16; 4];
                let err = apply_warp_into(&warp, 2, 2, &src, &mut dst)
                    .expect_err("non-finite warp must be refused");
                assert!(
                    matches!(err, Error::NonFiniteWarpCoefficients),
                    "{warp:?}: {err:?}"
                );
                assert_eq!(dst, vec![7u16; 4], "nothing written on refusal");
            }
        }
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

    // ── SPEC-099 / DEC-087 item 3: the CA rule ──────────────────────────────

    fn shift_of(green: [f64; 4], red: Option<[f64; 4]>, w: u32, h: u32) -> ChromaticShift {
        let warp = |kr| warp_with(kr, 0.5, 0.5);
        let (g, r) = (warp(green), red.map(warp));
        ChromaticShift::new(&g, r.as_ref(), None, (0, 0, w, h))
            .expect("admitted")
            .expect("a step")
    }

    #[test]
    fn ca_table_and_bound_on_a_hand_checkable_set() {
        // Green the identity, red kr0 = 1.02: every node's ratio is exactly
        // 1.02 / 1 = 1.02, and B = 0.02·m (the largest term is at j = T − 1,
        // where √((j+1)/T) = 1).
        let s = shift_of([1.0, 0.0, 0.0, 0.0], Some([1.02, 0.0, 0.0, 0.0]), 400, 300);
        let table = s.tables[0].as_ref().unwrap();
        assert_eq!(table.len(), CA_TABLE_INTERVALS + 1);
        assert!(table.iter().all(|&t| t == 1.02), "every node is 1.02");
        assert!(s.tables[1].is_none(), "blue is not resampled");
        let m = 199.5f64.hypot(149.5);
        assert_eq!(s.m, m);
        let bound = m * (1.02f64 - 1.0).abs();
        assert_eq!(s.halo, bound.ceil() as usize + 1, "H = ⌈B⌉ + 1 ({bound})");
        assert_eq!(s.halo, 6);
        // H is capped at the ActiveArea's height.
        let s = shift_of([1.0, 0.0, 0.0, 0.0], Some([1.5, 0.0, 0.0, 0.0]), 400, 20);
        assert_eq!(s.halo, 20);
        // SPEC-099 verify: the bound's radius is √((j+1)/T), so the last
        // interval reaches √1 = 1. With a constant ratio 1 + ε and m·ε just
        // above 6, B is just above 6 and H = 8; a radius of √(j/T) would give
        // B = m·ε·√(4095/4096), just below 6, and H = 7.
        let eps = 6.0001 / m;
        let s = shift_of(
            [1.0, 0.0, 0.0, 0.0],
            Some([1.0 + eps, 0.0, 0.0, 0.0]),
            400,
            300,
        );
        let reach = s.tables[0].as_ref().unwrap()[CA_TABLE_INTERVALS] - 1.0;
        assert!(m * reach > 6.0 && m * reach * (4095.0f64 / 4096.0).sqrt() < 6.0);
        assert_eq!(s.halo, 8, "H = ⌈B⌉ + 1 with B = m·ε·√(T/T)");
    }

    #[test]
    fn ca_bisection_inverts_a_known_map() {
        // f_g = 0.5: h(s) = s·0.25, so s = 4σ below the rim σ = 0.25, and 1 at
        // and beyond it.
        let g = [0.5, 0.0, 0.0, 0.0];
        for sigma in [0.0, 0.01, 0.1, 0.2, 0.249] {
            let s = table_node(&g, sigma);
            assert!((s - 4.0 * sigma).abs() <= 4e-16, "σ {sigma}: s {s}");
        }
        assert_eq!(table_node(&g, 0.25), 1.0, "at the rim");
        assert_eq!(table_node(&g, 0.3), 1.0, "beyond the rim, held");
        // A real green (Q3's): h(s_j) reproduces σ_j to the last bits.
        let g = [
            0.999_265_877_659_388,
            -0.054_056_477_178_819_784,
            -0.081_045_705_643_041_78,
            0.044_197_416_001_376_6,
        ];
        let rim = radial_scale(&g, 1.0);
        for j in [1usize, 100, 2048, 3000] {
            let sigma = j as f64 / CA_TABLE_INTERVALS_F64;
            assert!(sigma < rim * rim);
            let s = table_node(&g, sigma);
            let f = radial_scale(&g, s);
            assert!((s * f * f - sigma).abs() < 1e-15, "j {j}");
        }
    }

    #[test]
    fn ca_ratio_interpolates_and_clamps_at_the_rim() {
        let table: Vec<f64> = (0..=CA_TABLE_INTERVALS).map(|j| j as f64).collect();
        // Linear between nodes (not the nearest node).
        assert_eq!(ratio(&table, 10.25 / CA_TABLE_INTERVALS_F64), 10.25);
        assert_eq!(ratio(&table, 10.75 / CA_TABLE_INTERVALS_F64), 10.75);
        // σ = 1 reaches the last node exactly; beyond it, u is clamped to T.
        assert_eq!(ratio(&table, 1.0), 4096.0);
        assert_eq!(ratio(&table, 1.5), 4096.0);
        assert_eq!(ratio(&table, 0.0), 0.0);
    }

    #[test]
    fn ca_fold_boundary_refuses() {
        let red = warp_with([1.02, 0.0, 0.0, 0.0], 0.5, 0.5);
        // q = 0.5 − 0.5·s²: its minimum is exactly 0, at s = 1.
        let green = warp_with([0.5, 0.0, -0.1, 0.0], 0.5, 0.5);
        let r = ChromaticShift::new(&green, Some(&red), None, (0, 0, 10, 10));
        assert!(
            matches!(r, Err(Error::UnsupportedWarpFold { slope_min }) if slope_min == 0.0),
            "{:?}",
            r.err()
        );
        // Just inside: q(1) > 0.
        let green = warp_with([0.5, 0.0, -0.099, 0.0], 0.5, 0.5);
        assert!(ChromaticShift::new(&green, Some(&red), None, (0, 0, 10, 10)).is_ok());
        // A NaN slope refuses.
        let green = warp_with([f64::NAN, 0.0, 0.0, 0.0], 0.5, 0.5);
        let r = ChromaticShift::new(&green, Some(&red), None, (0, 0, 10, 10));
        assert!(matches!(r, Err(Error::UnsupportedWarpFold { slope_min }) if slope_min.is_nan()));
        // m = 0 (a one-pixel ActiveArea) is no step, even folding.
        let green = warp_with([1.0, -2.0, 0.0, 0.0], 0.5, 0.5);
        assert!(matches!(
            ChromaticShift::new(&green, Some(&red), None, (3, 4, 1, 1)),
            Ok(None)
        ));
        // Nothing differs: no step.
        assert!(matches!(
            ChromaticShift::new(&green, None, None, (0, 0, 10, 10)),
            Ok(None)
        ));
    }

    #[test]
    fn ca_value_samples_inside_the_active_area_only() {
        // ActiveArea at (left 2, top 1), 6 × 4; red magnified 1.5×: a corner
        // source falls outside and clamps to the ActiveArea's edge.
        let red = warp_with([1.5, 0.0, 0.0, 0.0], 0.5, 0.5);
        let green = warp_with([1.0, 0.0, 0.0, 0.0], 0.5, 0.5);
        let s = ChromaticShift::new(&green, Some(&red), None, (2, 1, 6, 4))
            .unwrap()
            .unwrap();
        assert!(s.contains(2, 1) && s.contains(7, 4));
        assert!(!s.contains(1, 1) && !s.contains(8, 1) && !s.contains(2, 0) && !s.contains(2, 5));
        let reads = std::cell::RefCell::new(Vec::new());
        let at = |x: u32, y: u32| {
            reads.borrow_mut().push((x, y));
            u16::try_from(x * 100 + y).unwrap()
        };
        // Pixel (2, 1) is the ActiveArea's top-left: source (-1.25, -0.75)
        // relative, clamped to (0, 0) relative = plane (2, 1).
        assert_eq!(s.value(0, 2, 1, at), Some(201));
        assert!(reads
            .borrow()
            .iter()
            .all(|&(x, y)| (2..8).contains(&x) && (1..5).contains(&y)));
        assert_eq!(s.value(2, 2, 1, |_, _| 7), None, "blue is not resampled");
        assert_eq!(s.value(1, 2, 1, |_, _| 7), None, "green is never resampled");
    }

    #[test]
    fn ca_m_is_hypot() {
        // The step's m is source_coord's own `hypot` (DEC-087 item 3).
        let red = warp_with([1.02, 0.0, 0.0, 0.0], 0.07, 0.6);
        let green = warp_with([1.0, 0.0, 0.0, 0.0], 0.07, 0.6);
        let s = ChromaticShift::new(&green, Some(&red), None, (0, 0, 6, 40))
            .unwrap()
            .unwrap();
        let (cx, cy) = (0.07 * 5.0, 0.6 * 39.0);
        let mx = f64::abs(cx).max((5.0 - cx).abs());
        let my = f64::abs(cy).max((39.0 - cy).abs());
        assert_eq!(s.m, mx.hypot(my));
        // On macOS's libm the two forms round apart here, so this test turns a
        // sqrt mutant red (SPEC-099 build); glibc's hypot rounds them alike, so
        // the difference is not asserted (it is platform-dependent).
    }

    /// SPEC-099 verify: [`ca_m_is_hypot`] separates `hypot` from the naive
    /// `sqrt(mx² + my²)` only where the platform's libm rounds them apart, and
    /// its one geometry does so on macOS but not under glibc, so the sqrt
    /// mutant survived on Linux. This sweep of off-centre geometries asserts
    /// `m` is `hypot` on each, and that the two forms differ somewhere on this
    /// platform, so the sweep has teeth wherever it runs.
    #[test]
    fn ca_m_is_hypot_across_geometries() {
        let red = [1.02, 0.0, 0.0, 0.0];
        let green = [1.0, 0.0, 0.0, 0.0];
        let mut apart = 0usize;
        let mut cases = 0usize;
        for (w, h) in [(6u32, 40u32), (7, 9), (13, 31), (29, 17), (40, 6), (3, 57)] {
            for cxh in [0.07, 0.13, 0.31, 0.62, 0.77, 0.91] {
                for cyh in [0.6, 0.11, 0.37, 0.83] {
                    let r = warp_with(red, cxh, cyh);
                    let g = warp_with(green, cxh, cyh);
                    let s = ChromaticShift::new(&g, Some(&r), None, (0, 0, w, h))
                        .unwrap()
                        .unwrap();
                    let (x1, y1) = (f64::from(w - 1), f64::from(h - 1));
                    let (cx, cy) = (cxh * x1, cyh * y1);
                    let mx = cx.abs().max((x1 - cx).abs());
                    let my = cy.abs().max((y1 - cy).abs());
                    let m = mx.hypot(my);
                    assert_eq!(s.m, m, "{w}x{h} centre ({cxh}, {cyh})");
                    cases += 1;
                    if m != (mx * mx + my * my).sqrt() {
                        apart += 1;
                    }
                }
            }
        }
        assert!(
            apart > 0,
            "hypot and sqrt(mx² + my²) agree on all {cases} geometries on this platform"
        );
    }
}
