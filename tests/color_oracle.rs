//! The colour-transform oracle — bit-exact on analytic fixtures (`SPEC-033`).
//!
//! The transform is a committed-constant `f64` matrix pipeline (`DEC-033`):
//! white balance (`AsShotNeutral`) → camera→XYZ(D50) (`ForwardMatrix1`, else
//! inverted `ColorMatrix1`) → XYZ(D50)→linear-sRGB (the committed `C = S · B`).
//! On a synthetic Sensor the output is **exactly computable**, so this oracle
//! re-derives it with an **independent reference implementation** (its own
//! transcription of the sRGB / Bradford constants, its own matrix multiply,
//! inverse and quantize — none of `src/color.rs`'s private code is read) and
//! asserts the public API matches it to the bit, with a perturbed-`ForwardMatrix`
//! red-proof (`DEC-017`, `oracle-must-be-shown-red`).
//!
//! This is the `DEC-030`/demosaic "independent clean-room reference" oracle
//! shape, NOT the exact-rational metadata oracle: the sRGB primaries and the
//! Bradford CAT are irrational (`DEC-033`), so the reference transcribes the
//! same committed constants and the equality is `f64 ==` on a pipeline of basic
//! ops — byte-identical across targets, not exact rational.
//!
//! **Tier-A only** — every fixture is synthetic, no corpus and no tools, so CI
//! runs the whole file. A loose `dnglab`-render cross-check would be Tier-B
//! (corpus-gated) and is explicitly NOT an accuracy gate (`SPEC-035`'s ΔE is);
//! it is left to a follow-on, since this file's job is the MATH.

use irradiance::color::{develop_color_f32_into, develop_color_into, to_srgb_linear_into};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{ActiveArea, Cfa, DefaultCropOrigin, DefaultCropSize, Rational, Sensor};
use irradiance::tone::to_linear_f32_into;
use irradiance::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Independent reference pipeline — transcribed separately from src/color.rs.
// ─────────────────────────────────────────────────────────────────────────────

/// Linear sRGB ← CIE XYZ (D65), IEC 61966-2-1 — transcribed independently here
/// so the oracle is a genuine cross-check, not a mirror of the shipped const.
const S_SRGB_FROM_XYZ_D65: [[f64; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

/// Bradford chromatic adaptation, CIE XYZ (D50) → CIE XYZ (D65).
const B_BRADFORD_D50_TO_D65: [[f64; 3]; 3] = [
    [0.9555766, -0.0230393, 0.0631636],
    [-0.0282895, 1.0099416, 0.0210077],
    [0.0122982, -0.0204830, 1.3299098],
];

/// Reference 3×3 product via the standard triple loop. `0.0 + x == x` in IEEE,
/// so this matches `src/color.rs`'s left-to-right unrolled sum to the bit.
fn mm(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
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

/// Reference 3×3 cofactor inverse (same formula the shipped code uses), or
/// `None` if singular.
fn inv(m: [[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let [[a, b, c], [d, e, f], [g, h, i]] = m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det == 0.0 {
        return None;
    }
    let id = 1.0 / det;
    Some([
        [
            (e * i - f * h) * id,
            (c * h - b * i) * id,
            (b * f - c * e) * id,
        ],
        [
            (f * g - d * i) * id,
            (a * i - c * g) * id,
            (c * d - a * f) * id,
        ],
        [
            (d * h - e * g) * id,
            (b * g - a * h) * id,
            (a * e - b * d) * id,
        ],
    ])
}

/// Reference 3×3 from 9 rationals (row-major).
fn matrix_of(entries: &[Rational]) -> [[f64; 3]; 3] {
    let mut m = [[0.0f64; 3]; 3];
    for (idx, r) in entries.iter().enumerate().take(9) {
        m[idx / 3][idx % 3] = f64::from(r.num) / f64::from(r.den);
    }
    m
}

/// Reference WB diagonal `diag(1/asn)` from the three neutral rationals.
fn wb_of(asn: &[Rational]) -> [[f64; 3]; 3] {
    let mut d = [[0.0f64; 3]; 3];
    for (idx, r) in asn.iter().enumerate().take(3) {
        d[idx][idx] = f64::from(r.den) / f64::from(r.num);
    }
    d
}

/// Independent transcription of the SPEC-034 illuminant→CCT table (a `match`
/// against src/color.rs's committed slice — a genuine cross-check of the values).
fn illum_cct(code: u32) -> Option<f64> {
    match code {
        1 => Some(5500.0),
        3 => Some(2856.0),
        17 => Some(2856.0),
        18 => Some(4874.0),
        19 => Some(6774.0),
        20 => Some(5503.0),
        21 => Some(6504.0),
        22 => Some(7504.0),
        23 => Some(5003.0),
        24 => Some(3200.0),
        _ => None,
    }
}

/// Reference element-wise matrix lerp `(1−g)·a + g·b` (triple loop).
fn ref_lerp(a: [[f64; 3]; 3], b: [[f64; 3]; 3], g: f64) -> [[f64; 3]; 3] {
    let mut o = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            o[i][j] = (1.0 - g) * a[i][j] + g * b[i][j];
        }
    }
    o
}

/// Reference matrix-vector product `m · v`.
fn ref_mul_vec(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    let mut o = [0.0f64; 3];
    for i in 0..3 {
        o[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2];
    }
    o
}

/// Reference McCamy cubic, `None` on a zero denominator.
fn ref_mccamy(x: f64, y: f64) -> Option<f64> {
    let denom = 0.1858 - y;
    if denom == 0.0 {
        return None;
    }
    let n = (x - 0.3320) / denom;
    Some(449.0 * n * n * n + 3525.0 * n * n + 6823.3 * n + 5520.33)
}

/// The 3×3 identity.
const I3: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// The D50 chromaticity the SDK's `NeutralToXY` starts from (DEC-076 §5).
const D50_XY: (f64, f64) = (0.3457, 0.3585);

/// Reference `AnalogBalance` as a diagonal; absent is the identity (DEC-076 §2).
/// Only called on fixtures whose `AnalogBalance` is well-shaped.
fn ref_ab(sensor: &Sensor) -> [[f64; 3]; 3] {
    match sensor.analog_balance.as_ref() {
        None => I3,
        Some(ab) => {
            let mut d = [[0.0f64; 3]; 3];
            for (idx, r) in ab.iter().enumerate().take(3) {
                d[idx][idx] = f64::from(r.num) / f64::from(r.den);
            }
            d
        }
    }
}

/// Reference signature rule (DEC-076 §2): the calibrations apply only when the
/// camera and profile signatures are equal, an absent one counting as empty.
fn ref_calibration_applies(sensor: &Sensor) -> bool {
    let camera: &[u8] = sensor
        .camera_calibration_signature
        .as_deref()
        .unwrap_or(&[]);
    let profile: &[u8] = sensor
        .profile_calibration_signature
        .as_deref()
        .unwrap_or(&[]);
    camera == profile
}

/// Reference `CC_i`: the identity when the rule says no or the tag is absent.
fn ref_cc(sensor: &Sensor, illuminant: u8) -> [[f64; 3]; 3] {
    let tag = if illuminant == 1 {
        sensor.camera_calibration_1.as_ref()
    } else {
        sensor.camera_calibration_2.as_ref()
    };
    match tag {
        Some(cc) if ref_calibration_applies(sensor) => matrix_of(cc),
        _ => I3,
    }
}

/// Reference `K_i = AB · CC_i`.
fn ref_k(sensor: &Sensor, illuminant: u8) -> [[f64; 3]; 3] {
    mm(ref_ab(sensor), ref_cc(sensor, illuminant))
}

/// Reference fold `X_i = K_i · CM_i`, left as `CM_i` itself when `K_i` is exactly
/// the identity (DEC-076 §4: the identity is byte-identical).
fn ref_fold(k: [[f64; 3]; 3], cm: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    if k == I3 {
        cm
    } else {
        mm(k, cm)
    }
}

/// The dual-illuminant inputs the reference weight needs, or `None` when the
/// preconditions fail (the single-illuminant fallback).
struct RefDual {
    x1: [[f64; 3]; 3],
    x2: [[f64; 3]; 3],
    inv_t1: f64,
    denom: f64,
    neutral: [f64; 3],
}

fn ref_dual(sensor: &Sensor) -> Option<RefDual> {
    let cm1 = sensor.color_matrix_1.as_ref()?;
    let cm2 = sensor.color_matrix_2.as_ref()?;
    if cm1.len() != 9 || cm2.len() != 9 {
        return None;
    }
    let t1 = illum_cct(sensor.calibration_illuminant_1?)?;
    let t2 = illum_cct(sensor.calibration_illuminant_2?)?;
    let inv_t1 = 1.0 / t1;
    let denom = inv_t1 - 1.0 / t2;
    if denom == 0.0 {
        return None;
    }
    let asn = sensor.as_shot_neutral.as_ref()?;
    if asn.len() != 3 {
        return None;
    }
    let neutral = [
        f64::from(asn[0].num) / f64::from(asn[0].den),
        f64::from(asn[1].num) / f64::from(asn[1].den),
        f64::from(asn[2].num) / f64::from(asn[2].den),
    ];
    Some(RefDual {
        x1: ref_fold(ref_k(sensor, 1), matrix_of(cm1)),
        x2: ref_fold(ref_k(sensor, 2), matrix_of(cm2)),
        inv_t1,
        denom,
        neutral,
    })
}

/// The mired weight of illuminant 2 at a chromaticity (`None` on a McCamy pole).
fn ref_weight_at(d: &RefDual, xy: (f64, f64)) -> Option<f64> {
    let t = ref_mccamy(xy.0, xy.1)?;
    Some(((d.inv_t1 - 1.0 / t) / d.denom).clamp(0.0, 1.0))
}

/// `xy(inv(X(g)) · neutral)`, or `None` when `X(g)` is singular or the XYZ sum
/// is not positive.
fn ref_neutral_xy(d: &RefDual, g: f64) -> Option<(f64, f64)> {
    let w = ref_mul_vec(inv(ref_lerp(d.x1, d.x2, g))?, d.neutral);
    let sum = w[0] + w[1] + w[2];
    if sum <= 0.0 {
        return None;
    }
    Some((w[0] / sum, w[1] / sum))
}

/// DEC-076 §5's stopping tolerance on `|Δx| + |Δy|`.
const REF_TOLERANCE: f64 = 0.0000001;

/// How the reference `NeutralToXY` loop ended.
#[derive(Debug, Clone, Copy)]
struct RefLoop {
    /// The scene weight read at the final `xy`.
    weight: f64,
    /// The number of passes run (1 to 30).
    passes: u32,
    /// The final chromaticity.
    xy: (f64, f64),
    /// `true` when a step fell under the tolerance; `false` when the 30th
    /// pass took the midpoint instead.
    by_tolerance: bool,
}

/// The reference `NeutralToXY` loop at a given stopping tolerance (DEC-076 §5:
/// from D50, at most 30 passes, stop when `|Δx| + |Δy| < tolerance` taking
/// `next`, else the midpoint of the last two on pass 30). The tolerance is a
/// parameter only so FU-1 can show that a 10× looser or tighter one develops
/// differently; everything else passes [`REF_TOLERANCE`]. `None` ⇒ fallback.
fn ref_neutral_to_xy(sensor: &Sensor, tolerance: f64) -> Option<RefLoop> {
    let d = ref_dual(sensor)?;
    let mut last = D50_XY;
    let mut passes = 0u32;
    let mut by_tolerance = false;
    for pass in 0..30u32 {
        passes = pass + 1;
        let g = ref_weight_at(&d, last)?;
        let mut next = ref_neutral_xy(&d, g)?;
        if (next.0 - last.0).abs() + (next.1 - last.1).abs() < tolerance {
            last = next;
            by_tolerance = true;
            break;
        }
        if pass == 29 {
            next = ((last.0 + next.0) * 0.5, (last.1 + next.1) * 0.5);
        }
        last = next;
    }
    Some(RefLoop {
        weight: ref_weight_at(&d, last)?,
        passes,
        xy: last,
        by_tolerance,
    })
}

/// Reference scene weight by the SDK 1.7.1 `NeutralToXY` loop (DEC-076 §5),
/// with the number of passes run. `None` ⇒ fallback.
fn ref_weight_and_passes(sensor: &Sensor) -> Option<(f64, u32)> {
    ref_neutral_to_xy(sensor, REF_TOLERANCE).map(|l| (l.weight, l.passes))
}

/// Reference scene weight (`None` ⇒ fallback).
fn ref_weight(sensor: &Sensor) -> Option<f64> {
    ref_weight_and_passes(sensor).map(|(g, _)| g)
}

/// DEC-034's superseded loop — a fixed 3 passes from `g = 0`, with no
/// calibration — kept only so SPEC-085 AC2 (f) can assert that the converged
/// weight is not it.
fn ref_weight_three_pass(sensor: &Sensor) -> Option<f64> {
    let cm1 = matrix_of(sensor.color_matrix_1.as_ref()?);
    let cm2 = matrix_of(sensor.color_matrix_2.as_ref()?);
    let d = RefDual {
        x1: cm1,
        x2: cm2,
        ..ref_dual(sensor)?
    };
    let mut g = 0.0f64;
    for _ in 0..3 {
        g = ref_weight_at(&d, ref_neutral_xy(&d, g)?)?;
    }
    Some(g)
}

/// Reference ColorMatrix-path `cam2xyz = CA(W→D50) · inv(CM)`, `W = inv(CM)·asn`
/// (DNG spec ch. 6); the Bradford CA is written out separately from the shipped
/// code. No white-balance diagonal on this path.
fn cm_cam2xyz(inv_cm: [[f64; 3]; 3], sensor: &Sensor) -> [[f64; 3]; 3] {
    const CONE: [[f64; 3]; 3] = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
    let asn: Vec<f64> = sensor
        .as_shot_neutral
        .as_ref()
        .expect("asn")
        .iter()
        .map(|r| f64::from(r.num) / f64::from(r.den))
        .collect();
    let mv = |m: [[f64; 3]; 3], v: [f64; 3]| -> [f64; 3] {
        let mut o = [0.0f64; 3];
        for i in 0..3 {
            o[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2];
        }
        o
    };
    let w = mv(inv_cm, [asn[0], asn[1], asn[2]]);
    let (rs, rd) = (mv(CONE, w), mv(CONE, [0.96422, 1.0, 0.82521]));
    let scale = [
        [rd[0] / rs[0], 0.0, 0.0],
        [0.0, rd[1] / rs[1], 0.0],
        [0.0, 0.0, rd[2] / rs[2]],
    ];
    let ca = mm(inv(CONE).expect("cone invertible"), mm(scale, CONE));
    mm(ca, inv_cm)
}

/// Reference ForwardMatrix-path `cam2xyz` (DEC-076 §4): today's `FM · D` when
/// the calibration is exactly the identity, else `FM · diag(1/R) · inv(K)` with
/// the reference neutral `R = inv(K) · asn`.
fn fm_cam2xyz(fm: [[f64; 3]; 3], k: Option<[[f64; 3]; 3]>, sensor: &Sensor) -> [[f64; 3]; 3] {
    let asn = sensor.as_shot_neutral.as_ref().expect("asn");
    match k {
        None => mm(fm, wb_of(asn)),
        Some(k) => {
            let ik = inv(k).expect("K invertible");
            let neutral = [
                f64::from(asn[0].num) / f64::from(asn[0].den),
                f64::from(asn[1].num) / f64::from(asn[1].den),
                f64::from(asn[2].num) / f64::from(asn[2].den),
            ];
            let r = ref_mul_vec(ik, neutral);
            let d = [
                [1.0 / r[0], 0.0, 0.0],
                [0.0, 1.0 / r[1], 0.0],
                [0.0, 0.0, 1.0 / r[2]],
            ];
            mm(fm, mm(d, ik))
        }
    }
}

/// Reference build of `M = C · cam2xyz` at the reference's own weight.
fn reference_matrix(sensor: &Sensor) -> [[f64; 3]; 3] {
    reference_matrix_at(sensor, ref_weight(sensor))
}

/// Reference build of `M = C · cam2xyz` at a given weight (`None` = the
/// single-illuminant path). DEC-076 §4: the ColorMatrix path inverts
/// `X(g) = lerp(AB·CC_1·CM_1, AB·CC_2·CM_2)`; the ForwardMatrix path takes
/// `K(g) = AB · lerp(CC_1, CC_2)`; an exactly-identity calibration is today's
/// transform.
fn reference_matrix_at(sensor: &Sensor, weight: Option<f64>) -> [[f64; 3]; 3] {
    let (k1, k2) = (ref_k(sensor, 1), ref_k(sensor, 2));
    let cam2xyz = match weight {
        Some(g) => {
            let fm_pair = match (
                sensor.forward_matrix_1.as_ref(),
                sensor.forward_matrix_2.as_ref(),
            ) {
                (Some(fm1), Some(fm2)) if fm1.len() == 9 && fm2.len() == 9 => {
                    Some((matrix_of(fm1), matrix_of(fm2)))
                }
                _ => None,
            };
            if let Some((f1, f2)) = fm_pair {
                let k = if k1 == I3 && k2 == I3 {
                    None
                } else {
                    Some(mm(
                        ref_ab(sensor),
                        ref_lerp(ref_cc(sensor, 1), ref_cc(sensor, 2), g),
                    ))
                };
                fm_cam2xyz(ref_lerp(f1, f2, g), k, sensor)
            } else {
                let x1 = ref_fold(k1, matrix_of(sensor.color_matrix_1.as_ref().expect("cm1")));
                let x2 = ref_fold(k2, matrix_of(sensor.color_matrix_2.as_ref().expect("cm2")));
                cm_cam2xyz(
                    inv(ref_lerp(x1, x2, g)).expect("interpolated X invertible"),
                    sensor,
                )
            }
        }
        None => {
            if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
                let k = if k1 == I3 { None } else { Some(k1) };
                fm_cam2xyz(matrix_of(fm1), k, sensor)
            } else {
                let x = ref_fold(k1, matrix_of(sensor.color_matrix_1.as_ref().expect("cm1")));
                cm_cam2xyz(inv(x).expect("invertible"), sensor)
            }
        }
    };
    let c = mm(S_SRGB_FROM_XYZ_D65, B_BRADFORD_D50_TO_D65);
    mm(c, cam2xyz)
}

/// Reference quantize: clamp to `[0, 65535]` before round-half-up (`DEC-018`).
fn quantize(v: f64) -> u16 {
    (v.clamp(0.0, 65535.0) + 0.5).floor() as u16
}

/// Reference apply of a built `M` to interleaved camera RGB → linear-sRGB.
fn reference_apply(m: [[f64; 3]; 3], camera_rgb: &[u16]) -> Vec<u16> {
    let mut out = vec![0u16; camera_rgb.len()];
    for (pixel, &[r_u, g_u, b_u]) in camera_rgb.as_chunks::<3>().0.iter().enumerate() {
        let r = f64::from(r_u);
        let g = f64::from(g_u);
        let b = f64::from(b_u);
        for channel in 0..3 {
            let row = m[channel];
            out[pixel * 3 + channel] = quantize(row[0] * r + row[1] * g + row[2] * b);
        }
    }
    out
}

/// The whole reference transform: build `M` from the sensor, apply to RGB.
fn reference_to_srgb(sensor: &Sensor, camera_rgb: &[u16]) -> Vec<u16> {
    reference_apply(reference_matrix(sensor), camera_rgb)
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture builders.
// ─────────────────────────────────────────────────────────────────────────────

/// A `Sensor` carrying the colour calibration the transform reads; everything
/// else is absent/empty (mirrors `tests/demosaic_oracle.rs::fixture_sensor`).
fn fixture_sensor(width: u32, height: u32) -> Sensor {
    let mut built = Sensor::test_fixture();
    built.width = width;
    built.height = height;
    built.photometric = 32803;
    built
}

/// A 2×2 RGGB CFA with the DNG-default plane colour.
fn rggb() -> Cfa {
    // `Cfa` is `#[non_exhaustive]` (DEC-052): start from the dev-only fixture,
    // which is already 2×2 RGGB with the DNG-default plane colour and layout.
    Cfa::test_fixture()
}

/// Nine `num/10000` rationals (row-major) for a synthetic matrix.
fn matrix_rationals(rows: [[i32; 3]; 3]) -> Vec<Rational> {
    let mut out = Vec::with_capacity(9);
    for row in rows {
        for num in row {
            out.push(Rational { num, den: 10000 });
        }
    }
    out
}

/// The neutral `(1/2, 1/1, 1/2)` used across the fixtures.
fn asn_half_one_half() -> Vec<Rational> {
    vec![
        Rational { num: 1, den: 2 },
        Rational { num: 1, den: 1 },
        Rational { num: 1, den: 2 },
    ]
}

/// A synthetic `ForwardMatrix1` whose rows sum to the CIE D50 white
/// `[0.96422, 1.0, 0.82521]` — the DNG constraint that makes the neutral map to
/// grey. `num/100000`.
fn forward_matrix_d50() -> Vec<Rational> {
    let rows = [
        [60000, 30000, 6422],
        [20000, 70000, 10000],
        [10000, 10000, 62521],
    ];
    let mut out = Vec::with_capacity(9);
    for row in rows {
        for num in row {
            out.push(Rational { num, den: 100000 });
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Bit-exact matrix math — the apply matches the independent reference.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn forward_matrix_path_matches_hand_computed() {
    let mut sensor = fixture_sensor(2, 2);
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());

    // Four camera pixels: a neutral (∝ asn), a red-biased, a mid-grey, a bright.
    let camera_rgb: Vec<u16> = vec![
        16384, 32768, 16384, // neutral (R=B=asn·G-scale)
        40000, 8000, 8000, // red-biased
        20000, 20000, 20000, // equal camera channels
        60000, 60000, 60000, // bright
    ];
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("valid calibration");

    assert_eq!(
        dst,
        reference_to_srgb(&sensor, &camera_rgb),
        "ForwardMatrix path must match the independent reference to the bit"
    );
}

#[test]
fn color_matrix_path_matches_hand_computed() {
    let mut sensor = fixture_sensor(2, 2);
    // No ForwardMatrix → the inverted-ColorMatrix path. An invertible CM1.
    sensor.color_matrix_1 = Some(matrix_rationals([
        [8000, 1000, 1000],
        [2000, 9000, -1000],
        [500, -1500, 11000],
    ]));
    sensor.as_shot_neutral = Some(asn_half_one_half());

    let camera_rgb: Vec<u16> = vec![
        16384, 32768, 16384, //
        40000, 8000, 8000, //
        20000, 20000, 20000, //
        60000, 60000, 60000,
    ];
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("invertible ColorMatrix");

    assert_eq!(
        dst,
        reference_to_srgb(&sensor, &camera_rgb),
        "ColorMatrix (inverted) path must match the independent reference to the bit"
    );
}

#[test]
fn a_neutral_subject_is_grey() {
    // A neutral subject reads `∝ asn` in camera space; WB maps it to the
    // reference white, the ForwardMatrix (rows summing to D50 white) to the D50
    // white, and C to the sRGB white — so the three output channels are equal.
    let mut sensor = fixture_sensor(1, 4);
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());

    // Each pixel is proportional to asn = (0.5, 1.0, 0.5): camera (c/2, c, c/2).
    let camera_rgb: Vec<u16> = vec![
        16384, 32768, 16384, // c = 32768
        8192, 16384, 8192, // c = 16384
        4096, 8192, 4096, // c = 8192
        1000, 2000, 1000, // c = 2000
    ];
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("valid calibration");

    for &[r, g, b] in dst.as_chunks::<3>().0 {
        assert_eq!(
            (r, g),
            (g, b),
            "a neutral subject must yield R == G == B, got [{r}, {g}, {b}]"
        );
    }
    // And a sanity check that it is not the trivial all-zero / all-saturated
    // image: the mid pixels land strictly inside the window.
    assert!(
        dst[3] > 0 && dst[3] < 65535,
        "mid-grey should be interior, got {}",
        dst[3]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Clamp — off-gamut mixes pin to the window, no wrap / panic (`DEC-018`).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn off_gamut_mixes_clamp_to_the_window() {
    let mut sensor = fixture_sensor(1, 2);
    // A ColorMatrix with large off-diagonal terms drives one channel negative
    // and another well past full-scale for a saturated input.
    sensor.color_matrix_1 = Some(matrix_rationals([
        [20000, -5000, -5000],
        [-5000, 20000, -5000],
        [-5000, -5000, 20000],
    ]));
    sensor.as_shot_neutral = Some(asn_half_one_half());

    let camera_rgb: Vec<u16> = vec![
        65535, 0, 0, // a pure, saturated channel → extreme off-gamut mix
        0, 0, 65535, //
    ];
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("invertible");

    // Every output matches the reference (which clamps identically) — a u16 is
    // in range by construction, so the no-wrap guarantee is that the two agree.
    assert_eq!(dst, reference_to_srgb(&sensor, &camera_rgb));
    // The construction guarantees at least one channel hit each rail.
    assert!(dst.contains(&0), "expected a channel pinned to 0");
    assert!(dst.contains(&65535), "expected a channel pinned to 65535");
}

// ─────────────────────────────────────────────────────────────────────────────
// Red-proof — a perturbed ForwardMatrix changes the OUTPUT (`DEC-017`).
// ─────────────────────────────────────────────────────────────────────────────

/// The camera RGB the red-proof pair runs on.
fn red_proof_camera() -> Vec<u16> {
    vec![30000, 20000, 10000, 5000, 40000, 25000, 60000, 1000, 50000]
}

#[test]
fn the_honest_control_is_green() {
    let mut sensor = fixture_sensor(3, 1);
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());

    let camera_rgb = red_proof_camera();
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("valid calibration");

    // The real transform reproduces the independent reference — the control the
    // mutant below is a one-coefficient perturbation of.
    assert_eq!(dst, reference_to_srgb(&sensor, &camera_rgb));
}

#[test]
fn a_perturbed_forward_matrix_turns_the_oracle_red() {
    let mut honest_sensor = fixture_sensor(3, 1);
    honest_sensor.forward_matrix_1 = Some(forward_matrix_d50());
    honest_sensor.as_shot_neutral = Some(asn_half_one_half());

    let camera_rgb = red_proof_camera();
    let mut honest = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&honest_sensor, &camera_rgb, &mut honest).expect("valid");

    // Mutate ONE real coefficient of the ForwardMatrix — FM1[0][0] — in a copy.
    let mut mutant_fm = forward_matrix_d50();
    mutant_fm[0] = Rational {
        num: mutant_fm[0].num + 5000,
        den: mutant_fm[0].den,
    };
    let mut mutant_sensor = fixture_sensor(3, 1);
    mutant_sensor.forward_matrix_1 = Some(mutant_fm);
    mutant_sensor.as_shot_neutral = Some(asn_half_one_half());

    let mut mutant = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&mutant_sensor, &camera_rgb, &mut mutant).expect("valid");

    // The load-bearing clause (`plane_oracle.rs`'s lesson, `DEC-017`): the
    // perturbation must CHANGE THE OUTPUT, not be a no-op the oracle misses.
    assert_ne!(
        mutant, honest,
        "perturbing FM1[0][0] did NOT change the output — the red-proof caught nothing"
    );
    // And the honest reference rejects the mutant — the oracle goes red.
    assert_ne!(mutant, reference_to_srgb(&honest_sensor, &camera_rgb));
}

// ─────────────────────────────────────────────────────────────────────────────
// Compose — develop_color_into == normalize ∘ demosaic ∘ matrix.
// ─────────────────────────────────────────────────────────────────────────────

/// Mirror of `color::normalize_mosaic`: per-2×2-position level-normalize, the
/// 2×2 slot counted from the `ActiveArea` origin (`DEC-040`; `(0, 0)` when the
/// tag is absent, so every origin-free fixture is unchanged).
fn reference_normalize(sensor: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let width = sensor.width as usize;
    let (origin_top, origin_left) = sensor
        .active_area
        .map(|a| (a.top as usize, a.left as usize))
        .unwrap_or((0, 0));
    let white_default = (1u64 << sensor.bits_per_sample).saturating_sub(1) as u32;
    let level_at = |levels: &[u32], pos: usize, scalar: Option<u32>, default: u32| -> u32 {
        match levels.get(pos).or_else(|| levels.last()) {
            Some(&v) => v,
            None => scalar.unwrap_or(default),
        }
    };
    // `DEC-070`: each photosite subtracts its own slot's black but every one
    // scales by white minus the largest of the four slot blacks, clipping
    // there (DNG 1.7.1 Chapter 5, "Rescaling").
    let max_black = (0..4)
        .map(|pos| level_at(&sensor.black_levels, pos, sensor.black_level, 0))
        .max()
        .unwrap_or(0);
    let normalize = |sample: u16, black: u32, white: u32| -> u16 {
        let range = white.saturating_sub(max_black);
        let shifted = u32::from(sample).saturating_sub(black).min(range);
        let numerator = u64::from(shifted) * u64::from(u16::MAX);
        let denominator = u64::from(range);
        if denominator == 0 {
            return 0;
        }
        let half = denominator / 2;
        u16::try_from((numerator + half) / denominator).unwrap_or(u16::MAX)
    };
    mosaic
        .iter()
        .enumerate()
        .map(|(pixel, &sample)| {
            let row = (pixel / width).wrapping_sub(origin_top);
            let col = (pixel % width).wrapping_sub(origin_left);
            let pos = match (row & 1, col & 1) {
                (0, 0) => 0,
                (0, _) => 1,
                (_, 0) => 2,
                (_, _) => 3,
            };
            let black = level_at(&sensor.black_levels, pos, sensor.black_level, 0);
            let white = level_at(&sensor.white_levels, pos, sensor.white_level, white_default);
            normalize(sample, black, white)
        })
        .collect()
}

#[test]
fn develop_color_composes_normalize_demosaic_matrix() {
    let mut sensor = fixture_sensor(4, 4);
    sensor.color_filter = Some(rggb());
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    // Per-channel black (2×2 positions R,G,G,B) and a scalar white.
    sensor.black_levels = vec![100, 100, 100, 200];
    sensor.black_level_repeat_dim = Some([2, 2]);
    sensor.white_levels = vec![4000];

    #[rustfmt::skip]
    let mosaic: Vec<u16> = vec![
        300,  900,  450, 1200,
        800,  500, 1100,  650,
        360, 1000,  520, 1320,
        1200, 560, 1520,  760,
    ];

    let mut composed = vec![0u16; mosaic.len() * 3];
    develop_color_into(&sensor, &mosaic, &mut composed).expect("valid mosaic + calibration");

    // Independently: normalize → demosaic (the real public kernel) → matrix.
    let normalized = reference_normalize(&sensor, &mosaic);
    let mut camera_rgb = vec![0u16; mosaic.len() * 3];
    irradiance::demosaic::demosaic_into(&sensor, &normalized, &mut camera_rgb)
        .expect("demosaic the normalized mosaic");
    let expected = reference_to_srgb(&sensor, &camera_rgb);

    assert_eq!(
        composed, expected,
        "develop_color_into must equal normalize → demosaic → matrix"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-049 — the fuse is output-invariant, and the f32 entry point equals the
// quantised u16 divided by 65535 (DEC-042).
//
// The fuse restructures develop_color_into from three buffered passes (normalize
// → demosaic → matrix) into two memory-lean passes that normalise on read and
// apply the matrix straight into dst. Its u16 output must stay byte-identical to
// the pre-fuse composition — proven here against the independent reference
// pipeline (reference_normalize → the real demosaic_into → reference_to_srgb),
// for all four Bayer phases. develop_color_f32_into must equal
// to_linear_f32_into(develop_color_into(..)) to the bit.
// ─────────────────────────────────────────────────────────────────────────────

/// A 2×2 Bayer CFA with the DNG-default plane colour for a given phase pattern.
fn phase_cfa(pattern: &[u8]) -> Cfa {
    let mut cfa = Cfa::test_fixture();
    cfa.pattern = pattern.to_vec();
    cfa
}

/// The pre-fuse colour-develop composition, built from independent transcriptions
/// plus the real public `demosaic_into`: normalize → demosaic → matrix. This is
/// the `dst_before` the fused `develop_color_into` (`dst_after`) must match.
fn reference_develop_color(sensor: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let normalized = reference_normalize(sensor, mosaic);
    let mut camera_rgb = vec![0u16; mosaic.len() * 3];
    irradiance::demosaic::demosaic_into(sensor, &normalized, &mut camera_rgb)
        .expect("demosaic the normalized mosaic");
    reference_to_srgb(sensor, &camera_rgb)
}

/// A varied (non-flat) `width·height` mosaic so the demosaic gradients differ
/// per pixel and the clamp (values above the white level) is exercised.
fn varied_mosaic(width: u32, height: u32) -> Vec<u16> {
    let pixels = (width as usize) * (height as usize);
    (0..pixels)
        .map(|i| ((i * 211 + 37) % 4096) as u16)
        .collect()
}

/// A `width·height` sensor carrying the colour calibration and per-slot levels
/// the fuse reads, for a given Bayer phase (no ActiveArea → origin `(0, 0)`, so
/// `reference_normalize`'s plane-relative slot matches).
fn fused_fixture(width: u32, height: u32, pattern: &[u8]) -> Sensor {
    let mut sensor = fixture_sensor(width, height);
    sensor.color_filter = Some(phase_cfa(pattern));
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    sensor.black_levels = vec![100, 140, 160, 220]; // distinct per-2×2-slot black
    sensor.black_level_repeat_dim = Some([2, 2]);
    sensor.white_levels = vec![4000]; // scalar white (below some mosaic values)
    sensor
}

/// The sizes the fuse tests sweep. 9×7 is odd in both dimensions and ≥ 5 each
/// (interior fast path AND reflecting border). 20×24 is taller than the rolling
/// normalised ring (RING = 8), so it exercises the ring wrap over many rows;
/// 3×5 keeps one axis below the interior threshold (border-only rows). A tiny
/// 2×2 and a degenerate 1×1 are covered by the panic tests above.
const FUSE_SIZES: [(u32, u32); 3] = [(9, 7), (20, 24), (3, 5)];

#[test]
fn fused_develop_color_is_byte_identical_to_the_reference() {
    // The fused develop_color_into must equal the pre-fuse
    // normalize → demosaic_into → matrix output byte for byte, across all four
    // Bayer phases and a spread of sizes (incl. one taller than the ring).
    for (w, h) in FUSE_SIZES {
        for pattern in BAYER_PHASES {
            let sensor = fused_fixture(w, h, &pattern);
            let mosaic = varied_mosaic(w, h);

            let mut dst_after = vec![0u16; mosaic.len() * 3];
            develop_color_into(&sensor, &mosaic, &mut dst_after).expect("fused develop");

            let dst_before = reference_develop_color(&sensor, &mosaic);

            assert_eq!(
                dst_after, dst_before,
                "size {w}×{h} phase {pattern:?}: the fused develop_color_into must equal the \
                 pre-fuse normalize → demosaic_into → matrix output byte for byte"
            );
        }
    }
}

#[test]
fn develop_color_f32_equals_to_linear_of_u16() {
    for (w, h) in FUSE_SIZES {
        for pattern in BAYER_PHASES {
            let sensor = fused_fixture(w, h, &pattern);
            let mosaic = varied_mosaic(w, h);

            // The u16 render, then to_linear_f32_into — the exact definition the
            // f32 entry point must reproduce (quantise, then divide by 65535.0).
            let mut u16_out = vec![0u16; mosaic.len() * 3];
            develop_color_into(&sensor, &mosaic, &mut u16_out).expect("u16 develop");
            let mut expected = vec![0.0f32; mosaic.len() * 3];
            to_linear_f32_into(&u16_out, &mut expected).expect("to_linear_f32_into");

            let mut f32_out = vec![0.0f32; mosaic.len() * 3];
            develop_color_f32_into(&sensor, &mosaic, &mut f32_out).expect("f32 develop");

            assert_eq!(
                f32_out, expected,
                "size {w}×{h} phase {pattern:?}: develop_color_f32_into must equal \
                 to_linear_f32_into(develop_color_into(..)) to the bit"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Typed errors, no panic — every wrong-length and bad-calibration path.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn wrong_lengths_and_missing_calibration_are_typed_errors() {
    let mut sensor = fixture_sensor(2, 2);
    sensor.color_filter = Some(rggb());
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());

    // to_srgb_linear_into: camera RGB must be width·height·3 = 12.
    let good_rgb = vec![0u16; 12];
    let mut good_dst = vec![0u16; 12];
    let short_rgb = vec![0u16; 11];
    match to_srgb_linear_into(&sensor, &short_rgb, &mut good_dst) {
        Err(Error::ColorSourceWrongLength { expected, actual }) => {
            assert_eq!((expected, actual), (12, 11));
        }
        other => panic!("expected ColorSourceWrongLength, got {other:?}"),
    }
    let mut short_dst = vec![0u16; 10];
    match to_srgb_linear_into(&sensor, &good_rgb, &mut short_dst) {
        Err(Error::RgbBufferWrongLength { expected, actual }) => {
            assert_eq!((expected, actual), (12, 10));
        }
        other => panic!("expected RgbBufferWrongLength, got {other:?}"),
    }

    // develop_color_into: the mosaic must be width·height = 4.
    let mut rgb_dst = vec![0u16; 12];
    let short_mosaic = vec![0u16; 3];
    match develop_color_into(&sensor, &short_mosaic, &mut rgb_dst) {
        Err(Error::ColorSourceWrongLength { expected, actual }) => {
            assert_eq!((expected, actual), (4, 3));
        }
        other => panic!("expected ColorSourceWrongLength, got {other:?}"),
    }
    let good_mosaic = vec![0u16; 4];
    let mut short_rgb_dst = vec![0u16; 11];
    match develop_color_into(&sensor, &good_mosaic, &mut short_rgb_dst) {
        Err(Error::RgbBufferWrongLength { expected, actual }) => {
            assert_eq!((expected, actual), (12, 11));
        }
        other => panic!("expected RgbBufferWrongLength, got {other:?}"),
    }

    // Missing calibration (correct lengths): no ForwardMatrix and no ColorMatrix.
    let mut no_cal = fixture_sensor(2, 2);
    no_cal.as_shot_neutral = Some(asn_half_one_half());
    match to_srgb_linear_into(&no_cal, &good_rgb, &mut good_dst) {
        Err(Error::MissingColorCalibration) => {}
        other => panic!("expected MissingColorCalibration, got {other:?}"),
    }

    // Missing AsShotNeutral.
    let mut no_asn = fixture_sensor(2, 2);
    no_asn.forward_matrix_1 = Some(forward_matrix_d50());
    match to_srgb_linear_into(&no_asn, &good_rgb, &mut good_dst) {
        Err(Error::MissingColorCalibration) => {}
        other => panic!("expected MissingColorCalibration, got {other:?}"),
    }
}

#[test]
fn degenerate_dims_do_not_panic() {
    // A 1×1 Bayer mosaic: demosaic resolves all neighbours to the one pixel
    // (reflect/clamp), the matrix applies, nothing panics.
    let mut sensor = fixture_sensor(1, 1);
    sensor.color_filter = Some(rggb());
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());

    let mosaic = vec![12345u16];
    let mut dst = vec![0u16; 3];
    develop_color_into(&sensor, &mosaic, &mut dst).expect("1x1 must not panic");

    // And the one-pixel matrix path directly.
    let camera_rgb = vec![10000u16, 20000, 30000];
    let mut rgb_dst = vec![0u16; 3];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut rgb_dst).expect("1x1 matrix path");
    assert_eq!(rgb_dst, reference_to_srgb(&sensor, &camera_rgb));
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-034 — dual-illuminant interpolation (DNG NeutralToXY + McCamy, DEC-034).
// The matrix is built from the illuminant-interpolated calibration; the public
// output still matches the independent reference to the bit, dual-calibration
// files move off the illuminant-1 render, and a perturbed ColorMatrix2 goes red.
// ─────────────────────────────────────────────────────────────────────────────

const ILLUMINANT_A: u32 = 17; // Standard light A → 2856 K
const ILLUMINANT_D65: u32 = 21; // D65 → 6504 K

/// Fixture `ColorMatrix1` (warm calibration) — `num/10000`; invertible.
fn fixture_cm1() -> Vec<Rational> {
    matrix_rationals([
        [39216, -17647, -39216],
        [-24510, 73529, -100490],
        [-9804, -70588, 259804],
    ])
}

/// Fixture `ColorMatrix2` (cool calibration) — `num/10000`; invertible.
fn fixture_cm2() -> Vec<Rational> {
    matrix_rationals([
        [35893, -13393, -9821],
        [-6071, 41071, -23214],
        [-32143, -17857, 53571],
    ])
}

/// A cooler companion to [`forward_matrix_d50`] for illuminant 2 (`num/100000`).
fn forward_matrix_cool() -> Vec<Rational> {
    let rows = [
        [52000, 36000, 8422],
        [16000, 78000, 6000],
        [6000, 14000, 70521],
    ];
    let mut out = Vec::with_capacity(9);
    for row in rows {
        for num in row {
            out.push(Rational { num, den: 100000 });
        }
    }
    out
}

/// The four-pixel camera RGB the interpolation fixtures run on.
fn dual_camera_rgb() -> Vec<u16> {
    vec![
        16384, 32768, 16384, // neutral (∝ asn)
        40000, 8000, 8000, // red-biased
        20000, 20000, 20000, // equal camera channels
        60000, 60000, 60000, // bright
    ]
}

#[test]
fn forward_matrix_path_interpolates_both_matrices() {
    let mut sensor = fixture_sensor(2, 2);
    sensor.color_matrix_1 = Some(fixture_cm1());
    sensor.color_matrix_2 = Some(fixture_cm2());
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.forward_matrix_2 = Some(forward_matrix_cool());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    sensor.calibration_illuminant_1 = Some(ILLUMINANT_A);
    sensor.calibration_illuminant_2 = Some(ILLUMINANT_D65);

    let camera_rgb = dual_camera_rgb();
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("valid dual calibration");

    // cam2xyz = C · FM(g) · D, FM(g) = lerp(FM1, FM2, g) — bit-exact vs the ref.
    assert_eq!(
        dst,
        reference_to_srgb(&sensor, &camera_rgb),
        "the FM(g) interpolation must match the independent reference to the bit"
    );

    // It genuinely interpolated: the render differs from the illuminant-1-only
    // (SPEC-033) render, and lands interior (not all clamped to a rail).
    let mut single = sensor.clone();
    single.color_matrix_2 = None; // forces the single-illuminant fallback (FM1 only)
    let dst_single = reference_to_srgb(&single, &camera_rgb);
    assert_ne!(
        dst, dst_single,
        "interpolation must move the output off the illuminant-1 render"
    );
    assert!(
        dst.iter().any(|&v| v > 0 && v < 65535),
        "expected interior output, got {dst:?}"
    );
}

#[test]
fn color_matrix_path_interpolates() {
    // No ForwardMatrix → the inverted interpolated-ColorMatrix path.
    let mut sensor = fixture_sensor(2, 2);
    sensor.color_matrix_1 = Some(fixture_cm1());
    sensor.color_matrix_2 = Some(fixture_cm2());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    sensor.calibration_illuminant_1 = Some(ILLUMINANT_A);
    sensor.calibration_illuminant_2 = Some(ILLUMINANT_D65);

    let camera_rgb = dual_camera_rgb();
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("invertible interpolated CM");

    assert_eq!(
        dst,
        reference_to_srgb(&sensor, &camera_rgb),
        "the inv(CM(g)) interpolation must match the independent reference to the bit"
    );

    // Differs from the illuminant-1-only (inv(CM1)) render, and is interior.
    let mut single = sensor.clone();
    single.color_matrix_2 = None;
    assert_ne!(dst, reference_to_srgb(&single, &camera_rgb));
    assert!(dst.iter().any(|&v| v > 0 && v < 65535));
}

#[test]
fn single_illuminant_is_byte_identical_to_spec_033() {
    // A sensor with no ColorMatrix2 (even with both CalibrationIlluminants set)
    // takes the single-illuminant fallback — byte-identical to SPEC-033's
    // illuminant-1 result. The shipped SPEC-033 fixtures (which carry no CM2)
    // already assert this; here the illuminants are present to prove it is the
    // missing CM2, not missing illuminants, that triggers the fallback.
    let mut sensor = fixture_sensor(2, 2);
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    sensor.calibration_illuminant_1 = Some(ILLUMINANT_A);
    sensor.calibration_illuminant_2 = Some(ILLUMINANT_D65);

    let camera_rgb = dual_camera_rgb();
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("single-illuminant fallback");

    // Build the SPEC-033 illuminant-1 result independently (no interpolation):
    // a fresh sensor with only FM1 + asn renders exactly C · FM1 · D.
    let mut spec_033 = fixture_sensor(2, 2);
    spec_033.forward_matrix_1 = Some(forward_matrix_d50());
    spec_033.as_shot_neutral = Some(asn_half_one_half());
    assert_eq!(
        dst,
        reference_to_srgb(&spec_033, &camera_rgb),
        "a sensor with no ColorMatrix2 must be byte-identical to SPEC-033"
    );
}

#[test]
fn the_honest_dual_control_is_green() {
    let mut sensor = fixture_sensor(3, 1);
    sensor.color_matrix_1 = Some(fixture_cm1());
    sensor.color_matrix_2 = Some(fixture_cm2());
    sensor.forward_matrix_1 = Some(forward_matrix_d50());
    sensor.forward_matrix_2 = Some(forward_matrix_cool());
    sensor.as_shot_neutral = Some(asn_half_one_half());
    sensor.calibration_illuminant_1 = Some(ILLUMINANT_A);
    sensor.calibration_illuminant_2 = Some(ILLUMINANT_D65);

    let camera_rgb = red_proof_camera();
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&sensor, &camera_rgb, &mut dst).expect("valid dual calibration");

    // The real interpolating transform reproduces the independent reference —
    // the control the mutant below is a one-ColorMatrix2-coefficient perturbation of.
    assert_eq!(dst, reference_to_srgb(&sensor, &camera_rgb));
}

#[test]
fn a_perturbed_second_matrix_turns_the_oracle_red() {
    let mut honest = fixture_sensor(3, 1);
    honest.color_matrix_1 = Some(fixture_cm1());
    honest.color_matrix_2 = Some(fixture_cm2());
    honest.forward_matrix_1 = Some(forward_matrix_d50());
    honest.forward_matrix_2 = Some(forward_matrix_cool());
    honest.as_shot_neutral = Some(asn_half_one_half());
    honest.calibration_illuminant_1 = Some(ILLUMINANT_A);
    honest.calibration_illuminant_2 = Some(ILLUMINANT_D65);

    let camera_rgb = red_proof_camera();
    let mut honest_dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&honest, &camera_rgb, &mut honest_dst).expect("valid");

    // Mutate ONE real ColorMatrix2 coefficient — CM2[0][0] — in a copy. It feeds
    // the weight (shifting g) AND, since both FMs are present, the render path
    // uses FM(g): the perturbation reaches the output only through g, which is
    // exactly what makes this a real red-proof of the interpolation.
    let mut mutant = honest.clone();
    let mut mutant_cm2 = fixture_cm2();
    mutant_cm2[0] = Rational {
        num: mutant_cm2[0].num + 5000,
        den: mutant_cm2[0].den,
    };
    mutant.color_matrix_2 = Some(mutant_cm2);

    let mut mutant_dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(&mutant, &camera_rgb, &mut mutant_dst).expect("valid");

    // The load-bearing clause (DEC-017): the perturbation must CHANGE THE OUTPUT,
    // not be a no-op the oracle misses.
    assert_ne!(
        mutant_dst, honest_dst,
        "perturbing CM2[0][0] did NOT change the output — the red-proof caught nothing"
    );
    // And the honest reference rejects the mutant — the oracle goes red.
    assert_ne!(mutant_dst, reference_to_srgb(&honest, &camera_rgb));
}

// ─────────────────────────────────────────────────────────────────────────────
// Neutral invariance — a grey subject must develop to R = G = B.
//
// Regression for the ColorMatrix-only path double-counting white balance
// (`inv(CM) · diag(1/asn)`), which developed grey to (1.0, 0.49, 0.78) on the
// Nikon D300S. A flat mosaic whose every site reads `black + k·(white−black)·asn`
// is a grey subject; the developed image must be neutral on every path.
// ─────────────────────────────────────────────────────────────────────────────

/// Nikon D300S tags (CM1 = A, CM2 = D65, no ForwardMatrix), `num/10000`.
fn nikon_cm1() -> Vec<Rational> {
    matrix_rationals([
        [10354, -3401, 54],
        [-7111, 14628, 2702],
        [-1350, 2011, 8528],
    ])
}

fn nikon_cm2() -> Vec<Rational> {
    matrix_rationals([
        [7471, -1017, -1087],
        [-8285, 15951, 2357],
        [-1210, 1801, 7387],
    ])
}

/// Nikon D300S `AsShotNeutral` (0.64, 1, 0.76417).
fn nikon_asn() -> Vec<Rational> {
    vec![
        Rational {
            num: 64000,
            den: 100000,
        },
        Rational { num: 1, den: 1 },
        Rational {
            num: 76417,
            den: 100000,
        },
    ]
}

/// Develop a 16×16 flat grey mosaic (black 1000, white 65535, brightness `k`) and
/// return the largest per-pixel channel spread, in `u16` counts.
fn grey_spread(sensor: &Sensor, k: f64) -> u16 {
    let (black, white) = (1000.0f64, 65535.0f64);
    let asn: Vec<f64> = sensor
        .as_shot_neutral
        .as_ref()
        .expect("asn")
        .iter()
        .map(|r| f64::from(r.num) / f64::from(r.den))
        .collect();
    // RGGB: site colour index 0=R,1=G,2=B.
    let site = |x: usize, y: usize| match (y % 2, x % 2) {
        (0, 0) => 0,
        (1, 1) => 2,
        _ => 1,
    };
    let (w, h) = (sensor.width as usize, sensor.height as usize);
    let mut mosaic = vec![0u16; w * h];
    for y in 0..h {
        for x in 0..w {
            let v = black + k * (white - black) * asn[site(x, y)];
            mosaic[y * w + x] = v.round() as u16;
        }
    }
    let mut out = vec![0u16; w * h * 3];
    develop_color_into(sensor, &mosaic, &mut out).expect("develops");
    out.as_chunks::<3>()
        .0
        .iter()
        .map(|p| p.iter().max().unwrap() - p.iter().min().unwrap())
        .max()
        .unwrap()
}

fn grey_sensor() -> Sensor {
    let mut s = fixture_sensor(16, 16);
    s.color_filter = Some(rggb());
    s.black_level = Some(1000);
    s.white_level = Some(65535);
    s
}

#[test]
fn grey_is_neutral_on_the_forward_matrix_path() {
    let mut s = grey_sensor();
    s.forward_matrix_1 = Some(forward_matrix_d50());
    s.as_shot_neutral = Some(nikon_asn());
    // ≤ 8 counts of 65535 ≈ quantisation from the 12-bit input and the 7-digit
    // sRGB constants.
    assert!(grey_spread(&s, 0.5) <= 8);
}

#[test]
fn grey_is_neutral_on_the_color_matrix_only_path() {
    // Single calibration.
    let mut s = grey_sensor();
    s.color_matrix_1 = Some(nikon_cm1());
    s.as_shot_neutral = Some(nikon_asn());
    let spread = grey_spread(&s, 0.5);
    assert!(
        spread <= 8,
        "single-illuminant CM path tinted grey: spread {spread}"
    );

    // Dual calibration (interpolated), as on the real D300S.
    s.color_matrix_2 = Some(nikon_cm2());
    s.calibration_illuminant_1 = Some(17); // Standard light A
    s.calibration_illuminant_2 = Some(21); // D65
    let spread = grey_spread(&s, 0.5);
    assert!(
        spread <= 8,
        "interpolated CM path tinted grey: spread {spread}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-047 — the colour path honours the ActiveArea origin (DEC-040).
//
// In DNG the CFA phase AND the per-slot BlackLevel/WhiteLevel 2×2 pattern both
// start at the ActiveArea origin, not the raw plane's top-left. The colour
// develop path must index both relative to that origin, via the offset form
// `(y − active_top) & 1`, `(x − active_left) & 1`. A flat-grey subject built
// relative to the origin must therefore stay grey for every origin parity and
// Bayer phase; the pre-fix plane-relative index tints an odd-origin grey (the
// red-proof). The ground-truth oracle here is the physical fact that a neutral
// subject is colourless — no external reference needed.
// ─────────────────────────────────────────────────────────────────────────────

/// Plane size the SPEC-047 grey fixtures develop on. Even in both dimensions, so
/// every 2×2 Bayer block is whole.
const GREY_W: u32 = 8;
const GREY_H: u32 = 8;

/// The four standard 2×2 Bayer phases, as `CFAPattern` colour indices in slot
/// order (top-left, top-right, bottom-left, bottom-right): 0 = R, 1 = G, 2 = B.
/// Each has one red, one blue and two diagonal greens — the RGGB family
/// `demosaic_into` accepts.
const BAYER_PHASES: [[u8; 4]; 4] = [
    [0, 1, 1, 2], // RGGB
    [2, 1, 1, 0], // BGGR
    [1, 0, 2, 1], // GRBG
    [1, 2, 0, 1], // GBRG
];

/// The four ActiveArea origin parities `(top, left)`: even/even, even/odd,
/// odd/even, odd/odd. The pre-fix plane-relative index is correct only for
/// even/even.
const ORIGIN_PARITIES: [(u32, u32); 4] = [(2, 4), (2, 5), (3, 4), (3, 5)];

/// A per-slot (2×2) BlackLevel pattern — one value per position, all distinct so
/// a wrong slot index (the plane-relative bug at an odd origin) subtracts a wrong
/// black. Position order matches `normalize_mosaic`'s `pos`: TL, TR, BL, BR.
const SLOT_BLACK: [u32; 4] = [200, 400, 600, 800];
const GREY_WHITE: u32 = 65535;

/// The scene brightness fraction the flat-grey subject is built at.
const GREY_K: f64 = 0.5;

/// Grey tolerance, in `u16` counts of 65535. A neutral subject is colourless, so
/// the spread is only quantisation from the 12-bit-ish input and the 7-digit
/// sRGB/Bradford constants (the shipped neutral tests hold at `≤ 8`); the
/// plane-relative tint is thousands of counts, so the two are never confusable.
const GREY_TOLERANCE: u16 = 8;

/// `AsShotNeutral` as an `f64` triple — reuses the asymmetric Nikon neutral
/// `(0.64, 1, 0.76417)` (R ≠ B, so even an R↔B swap tints).
fn grey_asn_f64() -> [f64; 3] {
    let a = nikon_asn();
    [
        f64::from(a[0].num) / f64::from(a[0].den),
        f64::from(a[1].num) / f64::from(a[1].den),
        f64::from(a[2].num) / f64::from(a[2].den),
    ]
}

/// Build the `Sensor` for a SPEC-047 grey case: the D50 forward matrix, the
/// Nikon neutral, the given Bayer `pattern`, the per-slot black pattern and a
/// scalar white. `sensor_origin` is what the sensor *declares* as its ActiveArea
/// origin — `Some((top, left))` sets `active_area`, `None` leaves it absent (the
/// plane-relative origin `(0, 0)`, i.e. the pre-DEC-040 behaviour).
fn grey_origin_sensor(pattern: &[u8], sensor_origin: Option<(u32, u32)>) -> Sensor {
    let mut s = fixture_sensor(GREY_W, GREY_H);
    s.color_filter = Some(phase_cfa(pattern));
    s.forward_matrix_1 = Some(forward_matrix_d50());
    s.as_shot_neutral = Some(nikon_asn());
    s.black_levels = SLOT_BLACK.to_vec();
    s.black_level_repeat_dim = Some([2, 2]);
    s.white_level = Some(GREY_WHITE);
    s.active_area = sensor_origin.map(|(top, left)| ActiveArea {
        top,
        left,
        bottom: GREY_H,
        right: GREY_W,
    });
    s
}

/// A flat-grey mosaic (`GREY_W × GREY_H`) constructed *relative to* `build_origin`
/// for the given Bayer `pattern`: each photosite reads
/// `black_slot + k·(white − max_black)·asn[colour]`, where the 2×2 slot and its
/// colour come from the position *relative to `build_origin`*. Subtracting that
/// slot's black and scaling by `white − max_black` (`DEC-070`; before SPEC-078
/// each slot used its own `white − black_slot`) therefore recovers `k·65535·asn[colour]` for
/// every photosite of a colour — a spectrally neutral (grey) subject. Once the
/// develop path indexes from the same origin, demosaic sees a piecewise-constant
/// mosaic and the colour matrix maps the neutral to grey.
fn grey_mosaic(pattern: &[u8], build_origin: (u32, u32)) -> Vec<u16> {
    let (build_top, build_left) = build_origin;
    let asn = grey_asn_f64();
    let white = f64::from(GREY_WHITE);
    let mut mosaic = vec![0u16; (GREY_W * GREY_H) as usize];
    for y in 0..GREY_H {
        for x in 0..GREY_W {
            let row_odd = (y.wrapping_sub(build_top) & 1) == 1;
            let col_odd = (x.wrapping_sub(build_left) & 1) == 1;
            let slot = usize::from(row_odd) * 2 + usize::from(col_odd);
            let colour = usize::from(pattern[slot]);
            let black = f64::from(SLOT_BLACK[slot]);
            // `DEC-070`: every slot scales by white minus the largest black.
            let max_black = f64::from(*SLOT_BLACK.iter().max().unwrap());
            let raw = black + GREY_K * (white - max_black) * asn[colour];
            mosaic[(y * GREY_W + x) as usize] = raw.round() as u16;
        }
    }
    mosaic
}

/// The largest per-pixel channel spread `max(R,G,B) − min(R,G,B)` of an
/// interleaved RGB image, in `u16` counts.
fn max_channel_spread(rgb: &[u16]) -> u16 {
    rgb.as_chunks::<3>()
        .0
        .iter()
        .map(|p| p.iter().max().unwrap() - p.iter().min().unwrap())
        .max()
        .unwrap_or(0)
}

#[test]
fn grey_stays_grey_across_origin_parities_and_bayer_phases() {
    // 4 Bayer phases × 4 origin parities = 16 cases. A flat-grey subject built
    // relative to each origin develops to grey once the path honours that origin.
    for pattern in BAYER_PHASES {
        for origin in ORIGIN_PARITIES {
            let sensor = grey_origin_sensor(&pattern, Some(origin));
            let mosaic = grey_mosaic(&pattern, origin);
            let mut out = vec![0u16; output_rgb_len(&sensor)];
            develop_color_into(&sensor, &mosaic, &mut out)
                .expect("a grey subject develops without error");
            let spread = max_channel_spread(&out);
            assert!(
                spread <= GREY_TOLERANCE,
                "phase {pattern:?} origin {origin:?}: grey tinted — spread {spread} counts \
                 exceeds {GREY_TOLERANCE}"
            );
        }
    }
}

#[test]
fn plane_relative_phase_tints_an_odd_origin_grey() {
    // The red-proof control. RGGB at an odd-top origin: under the plane-relative
    // index the row parity flips, so every red site is read as green (and vice
    // versa) — a flat grey develops to a strong tint. The reference below *is*
    // that plane-relative render (an ActiveArea-absent sensor indexes from
    // (0, 0), exactly the pre-DEC-040 behaviour). DEC-040 must move the real,
    // origin-declaring render off it; before the fix the two are identical and
    // the `assert_ne!` fails — the proof that the offset is load-bearing.
    let pattern = BAYER_PHASES[0]; // RGGB
    let origin = (3, 4); // top odd, left even → row-parity flip
    let mosaic = grey_mosaic(&pattern, origin);

    let plane_relative = grey_origin_sensor(&pattern, None);
    let mut reference = vec![0u16; mosaic.len() * 3];
    develop_color_into(&plane_relative, &mosaic, &mut reference)
        .expect("plane-relative develop is total");
    let reference_spread = max_channel_spread(&reference);
    assert!(
        reference_spread > 1000,
        "the plane-relative path should tint an odd-origin grey — spread was only \
         {reference_spread} counts, so this fixture does not exercise the bug"
    );

    let honours_origin = grey_origin_sensor(&pattern, Some(origin));
    let mut developed = vec![0u16; output_rgb_len(&honours_origin)];
    develop_color_into(&honours_origin, &mosaic, &mut developed)
        .expect("origin-honouring develop is total");
    // SPEC-050: the origin-honouring render is cropped to the ActiveArea, so
    // compare it against the plane-relative render at the same source pixels.
    let reference = crop_whole_plane(&honours_origin, &reference);
    assert_ne!(
        developed, reference,
        "declaring the ActiveArea origin did not move the output off the \
         plane-relative render — DEC-040 is not load-bearing"
    );
    assert!(
        max_channel_spread(&developed) <= GREY_TOLERANCE,
        "the origin-honouring render should recover grey, not merely differ"
    );
}

#[test]
fn even_origin_is_byte_identical_to_plane_relative() {
    // The no-regression guard. For an even/even origin `(y − even) & 1 == y & 1`,
    // so declaring the origin changes nothing — the corpus's even-origin files
    // (Canon 5D3 (80, 122), Nikon D300S (0, 0)) keep their exact goldens. Holds
    // before AND after DEC-040.
    for pattern in BAYER_PHASES {
        let origin = (2, 4); // even/even
        let mosaic = grey_mosaic(&pattern, origin);

        let plane_relative = grey_origin_sensor(&pattern, None);
        let mut reference = vec![0u16; mosaic.len() * 3];
        develop_color_into(&plane_relative, &mosaic, &mut reference)
            .expect("plane-relative develop is total");

        let declared = grey_origin_sensor(&pattern, Some(origin));
        let mut developed = vec![0u16; output_rgb_len(&declared)];
        develop_color_into(&declared, &mosaic, &mut developed)
            .expect("origin-honouring develop is total");
        // SPEC-050: the declared render is cropped to the ActiveArea; the
        // plane-relative render, taken at the same source pixels, must match.
        let reference = crop_whole_plane(&declared, &reference);

        assert_eq!(
            developed, reference,
            "phase {pattern:?}: an even origin must be byte-identical to the \
             plane-relative render"
        );
    }
}

#[test]
fn develop_color_does_not_panic_for_an_out_of_range_active_area() {
    // An ActiveArea origin at or beyond the plane edge. Since SPEC-050 the colour
    // path resolves the full geometry (DEC-043), so such a rectangle is rejected
    // with the same typed `InvalidActiveArea` the mono `develop_into` returns —
    // never a panic, for any origin value, and whatever `dst` length is passed.
    let pattern = BAYER_PHASES[0];
    for origin in [
        (GREY_H, GREY_W),          // exactly at the far edge
        (GREY_H + 7, GREY_W + 13), // beyond it, with odd offsets
        (u32::MAX, u32::MAX),      // the extreme
    ] {
        let sensor = grey_origin_sensor(&pattern, Some(origin));
        let mosaic: Vec<u16> = (0..GREY_W * GREY_H).map(|i| (i * 997) as u16).collect();
        let mut out = vec![0u16; mosaic.len() * 3];
        match develop_color_into(&sensor, &mosaic, &mut out) {
            Err(Error::InvalidActiveArea { .. }) => {}
            other => panic!("origin {origin:?}: expected InvalidActiveArea, got {other:?}"),
        }
        let mut out_f32 = vec![0.0f32; mosaic.len() * 3];
        match develop_color_f32_into(&sensor, &mosaic, &mut out_f32) {
            Err(Error::InvalidActiveArea { .. }) => {}
            other => panic!("origin {origin:?}: expected InvalidActiveArea, got {other:?}"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-050 — the colour path crops and orients to develop_into's geometry
// (DEC-043): ActiveArea → DefaultCrop → Orientation, output at
// output_dimensions(sensor), the masked border dropped. The crop moves the
// frame, not the colours: every kept pixel is byte-identical to the whole-plane
// develop (the SPEC-049 fused render, reproduced here by the independent
// reference_develop_color) at its source pixel.
// ─────────────────────────────────────────────────────────────────────────────

/// The interleaved-RGB `dst` length the colour develop expects for `sensor`:
/// `output_dimensions(sensor)`'s product × 3, the same size callers of the mono
/// `develop_into` compute (× 1 there).
fn output_rgb_len(sensor: &Sensor) -> usize {
    let (w, h) = output_dimensions(sensor).expect("fixture geometry is valid");
    w as usize * h as usize * 3
}

/// Independent transcription of TIFF tag 274's eight `Orientation` cases as the
/// output→crop map (where in the stored crop rectangle each displayed pixel
/// comes from). Re-derived from the TIFF "row 0 / column 0" table here rather
/// than read from `src/develop.rs`.
fn ref_crop_source(orientation: u32, ox: u32, oy: u32, cw: u32, ch: u32) -> (u32, u32) {
    match orientation {
        1 => (ox, oy),
        2 => (cw - 1 - ox, oy),
        3 => (cw - 1 - ox, ch - 1 - oy),
        4 => (ox, ch - 1 - oy),
        5 => (oy, ox),
        6 => (oy, ch - 1 - ox),
        7 => (cw - 1 - oy, ch - 1 - ox),
        8 => (cw - 1 - oy, ox),
        _ => unreachable!("fixture orientations are 1..=8"),
    }
}

/// Crop + orient a **whole-plane** interleaved-RGB render (`width·height·3`) to
/// `sensor`'s geometry by gathering, for each output pixel, the triple at its
/// source plane pixel — `ActiveArea` origin + `DefaultCropOrigin` (relative to
/// the ActiveArea, `DEC-019`) + the orientation map. This is the expected
/// SPEC-050 output: the same values, the frame moved.
fn crop_whole_plane(sensor: &Sensor, whole: &[u16]) -> Vec<u16> {
    let (top, left) = sensor
        .active_area
        .map(|a| (a.top, a.left))
        .unwrap_or((0, 0));
    let (active_w, active_h) = sensor
        .active_area
        .map(|a| (a.right - a.left, a.bottom - a.top))
        .unwrap_or((sensor.width, sensor.height));
    let (cox, coy) = sensor
        .default_crop_origin
        .map(|o| (o.x, o.y))
        .unwrap_or((0, 0));
    let (cw, ch) = sensor
        .default_crop_size
        .map(|s| (s.width, s.height))
        .unwrap_or((active_w, active_h));
    let orientation = sensor.orientation.unwrap_or(1);
    let (ow, oh) = if orientation <= 4 { (cw, ch) } else { (ch, cw) };
    let mut out = Vec::with_capacity((ow * oh * 3) as usize);
    for oy in 0..oh {
        for ox in 0..ow {
            let (cx, cy) = ref_crop_source(orientation, ox, oy, cw, ch);
            let px = (left + cox + cx) as usize;
            let py = (top + coy + cy) as usize;
            let i = (py * sensor.width as usize + px) * 3;
            out.extend_from_slice(&whole[i..i + 3]);
        }
    }
    out
}

/// Plane size of the SPEC-050 geometry fixture.
const GEO_W: u32 = 20;
const GEO_H: u32 = 16;

/// A 20×16 colour sensor with a non-trivial geometry: an **odd-top** ActiveArea
/// `(3, 2)–(15, 19)` (17×12, so the SPEC-047 phase offset is load-bearing), a
/// `DefaultCrop` inset `(2, 1)` of size 13×9 inside it, and the given
/// `orientation` (`None` = absent = 1). Per-slot blacks + a scalar white so the
/// level slot matters too.
fn geometry_fixture(pattern: &[u8], orientation: Option<u32>) -> Sensor {
    let mut sensor = fused_fixture(GEO_W, GEO_H, pattern);
    sensor.active_area = Some(ActiveArea {
        top: 3,
        left: 2,
        bottom: 15,
        right: 19,
    });
    sensor.default_crop_origin = Some(DefaultCropOrigin { x: 2, y: 1 });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 13,
        height: 9,
    });
    sensor.orientation = orientation;
    sensor
}

/// Every TIFF orientation, plus "absent" (which must behave as 1).
const ALL_ORIENTATIONS: [Option<u32>; 9] = [
    None,
    Some(1),
    Some(2),
    Some(3),
    Some(4),
    Some(5),
    Some(6),
    Some(7),
    Some(8),
];

#[test]
fn colour_output_has_output_dimensions_geometry() {
    // Orientation 6 (a 90° rotation) over the cropped fixture: the output is the
    // 13×9 crop transposed to 9×13, so dst is 9·13·3 — the whole 20×16 plane
    // (and its masked border) is gone.
    let sensor = geometry_fixture(&BAYER_PHASES[0], Some(6));
    assert_eq!(output_dimensions(&sensor).expect("valid geometry"), (9, 13));
    let mosaic = varied_mosaic(GEO_W, GEO_H);

    let mut out = vec![0u16; 9 * 13 * 3];
    develop_color_into(&sensor, &mosaic, &mut out)
        .expect("dst sized to output_dimensions·3 must be accepted");

    // The whole-plane length is now the WRONG length (a typed error, no panic).
    let mut whole_len = vec![0u16; mosaic.len() * 3];
    match develop_color_into(&sensor, &mosaic, &mut whole_len) {
        Err(Error::RgbBufferWrongLength { expected, actual }) => {
            assert_eq!((expected, actual), (9 * 13 * 3, mosaic.len() * 3));
        }
        other => panic!("expected RgbBufferWrongLength, got {other:?}"),
    }

    // The masked border is gone: output [0, 0] is NOT the plane's (0, 0) border
    // pixel, it is the oriented crop's first pixel.
    let whole = reference_develop_color(&sensor, &mosaic);
    assert_ne!(
        out[0..3],
        whole[0..3],
        "output [0,0] still carries the masked-border pixel (0,0)"
    );
    assert_eq!(out, crop_whole_plane(&sensor, &whole));
}

#[test]
fn colour_geometry_matches_the_mono_path() {
    // For every orientation, the colour output has develop_into's dimensions AND
    // takes each output pixel from the same source photosite develop_into does.
    // The mono path is the oracle for WHERE: a mosaic whose sample is its own
    // plane index, developed by develop_into with an identity level map (black 0,
    // white 65535), returns at each output slot the index of its source pixel.
    for orientation in ALL_ORIENTATIONS {
        let sensor = geometry_fixture(&BAYER_PHASES[0], orientation);
        let (ow, oh) = output_dimensions(&sensor).expect("valid geometry");
        let (cw, ch) = (13, 9);
        let expected_dims = if orientation.unwrap_or(1) <= 4 {
            (cw, ch)
        } else {
            (ch, cw)
        };
        assert_eq!((ow, oh), expected_dims, "orientation {orientation:?}");

        // Where: the mono source map.
        let mut mono_sensor = sensor.clone();
        mono_sensor.black_level = Some(0);
        mono_sensor.white_level = Some(65535);
        let index_plane: Vec<u16> = (0..GEO_W * GEO_H).map(|i| i as u16).collect();
        let mut source_of = vec![0u16; (ow * oh) as usize];
        develop_into(&mono_sensor, &index_plane, &mut source_of).expect("mono develop");

        // What: the colour develop, cropped by the colour path itself.
        let mosaic = varied_mosaic(GEO_W, GEO_H);
        let mut colour = vec![0u16; (ow * oh * 3) as usize];
        develop_color_into(&sensor, &mosaic, &mut colour)
            .expect("colour develop at output_dimensions");

        let whole = reference_develop_color(&sensor, &mosaic);
        for (slot, &src) in source_of.iter().enumerate() {
            let s = usize::from(src) * 3;
            assert_eq!(
                colour[slot * 3..slot * 3 + 3],
                whole[s..s + 3],
                "orientation {orientation:?}: output slot {slot} must come from the                  source photosite develop_into maps it to ({src})"
            );
        }
    }
}

#[test]
fn colour_crop_pixels_equal_the_whole_plane_develop_at_their_source() {
    // The crop moves the frame, not the colours: across all four Bayer phases
    // and every orientation, each kept pixel is byte-identical to the whole-plane
    // develop (SPEC-047's origin-honouring colours, via the independent
    // reference pipeline) at its DefaultCrop + inverse-orient source.
    for pattern in BAYER_PHASES {
        for orientation in ALL_ORIENTATIONS {
            let sensor = geometry_fixture(&pattern, orientation);
            let mosaic = varied_mosaic(GEO_W, GEO_H);
            let mut out = vec![0u16; output_rgb_len(&sensor)];
            develop_color_into(&sensor, &mosaic, &mut out).expect("cropped develop");

            let whole = reference_develop_color(&sensor, &mosaic);
            assert_eq!(
                out,
                crop_whole_plane(&sensor, &whole),
                "phase {pattern:?} orientation {orientation:?}: a kept pixel moved value"
            );
        }
    }
    // And SPEC-047's grey-stays-grey holds on every kept pixel of a cropped,
    // rotated odd-origin grey.
    for pattern in BAYER_PHASES {
        for origin in ORIGIN_PARITIES {
            let mut sensor = grey_origin_sensor(&pattern, Some(origin));
            sensor.default_crop_origin = Some(DefaultCropOrigin { x: 1, y: 1 });
            sensor.default_crop_size = Some(DefaultCropSize {
                width: 2,
                height: 3,
            });
            sensor.orientation = Some(7);
            let mosaic = grey_mosaic(&pattern, origin);
            let mut out = vec![0u16; output_rgb_len(&sensor)];
            develop_color_into(&sensor, &mosaic, &mut out).expect("cropped grey develops");
            assert!(
                max_channel_spread(&out) <= GREY_TOLERANCE,
                "phase {pattern:?} origin {origin:?}: the cropped grey tinted"
            );
        }
    }
}

#[test]
fn develop_color_f32_still_equals_to_linear_at_new_dims() {
    for pattern in BAYER_PHASES {
        for orientation in ALL_ORIENTATIONS {
            let sensor = geometry_fixture(&pattern, orientation);
            let mosaic = varied_mosaic(GEO_W, GEO_H);
            let len = output_rgb_len(&sensor);

            let mut u16_out = vec![0u16; len];
            develop_color_into(&sensor, &mosaic, &mut u16_out).expect("u16 develop");
            let mut expected = vec![0.0f32; len];
            to_linear_f32_into(&u16_out, &mut expected).expect("to_linear_f32_into");

            let mut f32_out = vec![0.0f32; len];
            develop_color_f32_into(&sensor, &mosaic, &mut f32_out)
                .expect("f32 develop at output_dimensions");
            assert_eq!(
                f32_out, expected,
                "phase {pattern:?} orientation {orientation:?}: f32 must equal                  to_linear_f32_into(develop_color_into(..)) at the new dimensions"
            );

            let mut whole_len = vec![0.0f32; mosaic.len() * 3];
            match develop_color_f32_into(&sensor, &mosaic, &mut whole_len) {
                Err(Error::RgbBufferWrongLength { expected, actual }) => {
                    assert_eq!((expected, actual), (len as u64, mosaic.len() * 3));
                }
                other => panic!("expected RgbBufferWrongLength, got {other:?}"),
            }
        }
    }
}

#[test]
fn trivial_geometry_is_byte_identical_to_the_whole_plane() {
    // The pin: a full-plane ActiveArea, a full DefaultCrop and orientation 1
    // resolve to the whole plane, so the SPEC-050 output equals the pre-SPEC-050
    // whole-plane develop byte for byte (and equals the all-tags-absent render).
    for (w, h) in FUSE_SIZES {
        for pattern in BAYER_PHASES {
            let absent = fused_fixture(w, h, &pattern);
            let mut trivial = absent.clone();
            trivial.active_area = Some(ActiveArea {
                top: 0,
                left: 0,
                bottom: h,
                right: w,
            });
            trivial.default_crop_origin = Some(DefaultCropOrigin { x: 0, y: 0 });
            trivial.default_crop_size = Some(DefaultCropSize {
                width: w,
                height: h,
            });
            trivial.orientation = Some(1);
            let mosaic = varied_mosaic(w, h);

            let mut from_absent = vec![0u16; mosaic.len() * 3];
            develop_color_into(&absent, &mosaic, &mut from_absent).expect("absent tags");
            let mut from_trivial = vec![0u16; mosaic.len() * 3];
            develop_color_into(&trivial, &mosaic, &mut from_trivial).expect("trivial tags");

            let whole = reference_develop_color(&absent, &mosaic);
            assert_eq!(from_absent, whole, "{w}×{h} {pattern:?}: absent tags");
            assert_eq!(from_trivial, whole, "{w}×{h} {pattern:?}: trivial tags");
        }
    }
}

#[test]
fn hostile_geometry_is_a_typed_error_on_the_colour_path() {
    // DefaultCrop that does not fit, and an out-of-range Orientation: the colour
    // path reuses resolve_geometry's typed rejections, never a panic.
    let mosaic = varied_mosaic(GEO_W, GEO_H);
    let mut dst = vec![0u16; mosaic.len() * 3];

    let mut bad_crop = geometry_fixture(&BAYER_PHASES[0], None);
    bad_crop.default_crop_size = Some(DefaultCropSize {
        width: 17,
        height: 9,
    });
    match develop_color_into(&bad_crop, &mosaic, &mut dst) {
        Err(Error::InvalidDefaultCrop { .. }) => {}
        other => panic!("expected InvalidDefaultCrop, got {other:?}"),
    }

    let bad_orientation = geometry_fixture(&BAYER_PHASES[0], Some(9));
    match develop_color_into(&bad_orientation, &mosaic, &mut dst) {
        Err(Error::UnsupportedOrientation { orientation: 9 }) => {}
        other => panic!("expected UnsupportedOrientation, got {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-085 — CameraCalibration and AnalogBalance (DEC-076). The reference above
// carries the DEC-076 transform: `X_i = AB · CC_i · CM_i` lerped on the
// ColorMatrix path, `K(g) = AB · lerp(CC_1, CC_2)` on the ForwardMatrix path, the
// signature rule, and the convergent NeutralToXY loop. Every fixture asserts the
// public API equals it to the bit AND that the clause under test moves the
// output (so a mutant dropping it goes red).
// ─────────────────────────────────────────────────────────────────────────────

/// A `num/10000` diagonal calibration `diag(a, 1, c)` (the Pixel shapes).
fn cc_diag(a: i32, c: i32) -> Vec<Rational> {
    matrix_rationals([[a, 0, 0], [0, 10000, 0], [0, 0, c]])
}

/// A non-diagonal calibration: the shape where the ForwardMatrix path's `K` is
/// observable (for a diagonal `K`, `FM · diag(1/R) · inv(K)` equals `FM · D`).
fn cc_skew_1() -> Vec<Rational> {
    matrix_rationals([[10235, -820, 400], [350, 10000, -250], [150, 700, 9555]])
}

/// A second non-diagonal calibration, different from [`cc_skew_1`].
fn cc_skew_2() -> Vec<Rational> {
    matrix_rationals([[9340, 600, -300], [-450, 10000, 500], [-200, -900, 10764]])
}

/// The identity as the held files write it (`n/10000`).
fn cc_identity() -> Vec<Rational> {
    cc_diag(10000, 10000)
}

/// An `AnalogBalance` of `(1.1, 1, 0.9)`.
fn ab_tilted() -> Vec<Rational> {
    vec![
        Rational { num: 11, den: 10 },
        Rational { num: 1, den: 1 },
        Rational { num: 9, den: 10 },
    ]
}

/// An `AnalogBalance` of `(1, 1, 1)` as the held files write it.
fn ab_unity() -> Vec<Rational> {
    vec![
        Rational {
            num: 1_000_000,
            den: 1_000_000,
        };
        3
    ]
}

fn sig(text: &str) -> Option<Vec<u8>> {
    Some(text.as_bytes().to_vec())
}

/// The sensor without any of the five SPEC-085 tags.
fn without_calibration(sensor: &Sensor) -> Sensor {
    let mut s = sensor.clone();
    s.camera_calibration_1 = None;
    s.camera_calibration_2 = None;
    s.analog_balance = None;
    s.camera_calibration_signature = None;
    s.profile_calibration_signature = None;
    s
}

fn api_to_srgb(sensor: &Sensor, camera_rgb: &[u16]) -> Vec<u16> {
    let mut dst = vec![0u16; camera_rgb.len()];
    to_srgb_linear_into(sensor, camera_rgb, &mut dst).expect("valid calibration");
    dst
}

/// The single-illuminant ColorMatrix fixture (the Pixel 7 Pro's path).
fn single_cm_sensor() -> Sensor {
    let mut s = fixture_sensor(2, 2);
    s.color_matrix_1 = Some(fixture_cm1());
    s.as_shot_neutral = Some(asn_half_one_half());
    s
}

/// The single-illuminant ForwardMatrix fixture.
fn single_fm_sensor() -> Sensor {
    let mut s = single_cm_sensor();
    s.forward_matrix_1 = Some(forward_matrix_d50());
    s
}

/// The dual-illuminant ColorMatrix fixture (A / D65).
fn dual_cm_sensor() -> Sensor {
    let mut s = single_cm_sensor();
    s.color_matrix_2 = Some(fixture_cm2());
    s.calibration_illuminant_1 = Some(ILLUMINANT_A);
    s.calibration_illuminant_2 = Some(ILLUMINANT_D65);
    s
}

/// The dual-illuminant ForwardMatrix fixture.
fn dual_fm_sensor() -> Sensor {
    let mut s = dual_cm_sensor();
    s.forward_matrix_1 = Some(forward_matrix_d50());
    s.forward_matrix_2 = Some(forward_matrix_cool());
    s
}

/// Assert the API equals the DEC-076 reference to the bit, and that the
/// calibration moved the output off the uncalibrated render.
fn assert_calibrated(sensor: &Sensor, what: &str) {
    let camera_rgb = dual_camera_rgb();
    let got = api_to_srgb(sensor, &camera_rgb);
    assert_eq!(
        got,
        reference_to_srgb(sensor, &camera_rgb),
        "{what}: the API must equal the DEC-076 reference to the bit"
    );
    assert_ne!(
        got,
        api_to_srgb(&without_calibration(sensor), &camera_rgb),
        "{what}: the calibration must move the output"
    );
    assert!(
        got.iter().any(|&v| v > 0 && v < 65535),
        "{what}: expected interior output, got {got:?}"
    );
}

/// On the ColorMatrix path, a neutral input `c · AsShotNeutral` stays grey.
fn assert_cm_neutral_is_grey(sensor: &Sensor, what: &str) {
    // asn = (1/2, 1, 1/2): camera (c/2, c, c/2) for a few even c.
    let camera_rgb: Vec<u16> = vec![
        16384, 32768, 16384, 4000, 8000, 4000, 500, 1000, 500, 30000, 60000, 30000,
    ];
    let got = api_to_srgb(sensor, &camera_rgb);
    for &[r, g, b] in got.as_chunks::<3>().0 {
        let spread = r.max(g).max(b) - r.min(g).min(b);
        assert!(
            spread <= 1,
            "{what}: a neutral input must stay grey, got [{r}, {g}, {b}]"
        );
    }
}

#[test]
fn ac2a_single_illuminant_colour_matrix_with_a_diagonal_calibration() {
    let mut s = single_cm_sensor();
    s.camera_calibration_1 = Some(cc_diag(8594, 3408));
    assert_calibrated(&s, "AC2 (a)");
    assert_cm_neutral_is_grey(&s, "AC2 (a)");
}

#[test]
fn ac2b_single_illuminant_forward_matrix_with_a_non_diagonal_calibration() {
    let mut s = single_fm_sensor();
    s.camera_calibration_1 = Some(cc_skew_1());
    assert_calibrated(&s, "AC2 (b)");
    // The K clause is what moved it: today's `FM · D` at this sensor is the
    // uncalibrated render, already asserted different above. A diagonal
    // calibration on the same path does NOT move it (the FM-path trap).
    let mut diagonal = single_fm_sensor();
    diagonal.camera_calibration_1 = Some(cc_diag(8594, 3408));
    let camera_rgb = dual_camera_rgb();
    let got = api_to_srgb(&diagonal, &camera_rgb);
    assert_eq!(got, reference_to_srgb(&diagonal, &camera_rgb));
    let delta = got
        .iter()
        .zip(api_to_srgb(&without_calibration(&diagonal), &camera_rgb))
        .map(|(&a, b)| a.abs_diff(b))
        .max()
        .unwrap_or(0);
    assert!(
        delta <= 1,
        "a diagonal K on a single-illuminant FM file equals FM · D up to rounding, got {delta}"
    );
}

#[test]
fn ac2c_analog_balance_applies_with_mismatched_signatures_on_both_paths() {
    // On the FM path a diagonal `K = AB` equals `FM · D` up to rounding when the
    // weight is fixed, so the FM row is dual: there `AB` moves the weight.
    for (path, base) in [("CM", single_cm_sensor()), ("FM", dual_fm_sensor())] {
        let mut s = base;
        s.analog_balance = Some(ab_tilted());
        s.camera_calibration_1 = Some(cc_skew_1());
        s.camera_calibration_2 = Some(cc_skew_2());
        s.camera_calibration_signature = sig("a");
        s.profile_calibration_signature = sig("b");
        let what = format!("AC2 (c) {path}");
        assert_calibrated(&s, &what);

        let camera_rgb = dual_camera_rgb();
        let got = api_to_srgb(&s, &camera_rgb);
        // AB applied: dropping it changes the output.
        let mut no_ab = s.clone();
        no_ab.analog_balance = None;
        assert_ne!(got, api_to_srgb(&no_ab, &camera_rgb), "{what}: AB ignored");
        // CC not applied: the same file with matching signatures differs.
        let mut cc_on = s.clone();
        cc_on.profile_calibration_signature = sig("a");
        assert_ne!(
            got,
            reference_to_srgb(&cc_on, &camera_rgb),
            "{what}: CC applied"
        );
        if path == "CM" {
            assert_cm_neutral_is_grey(&s, &what);
        }
    }
}

#[test]
fn ac2d_dual_colour_matrix_lerps_the_folded_products() {
    let mut s = dual_cm_sensor();
    s.camera_calibration_1 = Some(cc_diag(8594, 3408));
    s.camera_calibration_2 = Some(cc_diag(6162, 5703));
    assert_calibrated(&s, "AC2 (d)");
    assert_cm_neutral_is_grey(&s, "AC2 (d)");

    // The lerp of the FACTORS, `AB · lerp(CC) · lerp(CM)`, at the same weight is
    // a different matrix, and the API is not it.
    let g = ref_weight(&s).expect("dual weight");
    let factors = {
        let k = mm(ref_ab(&s), ref_lerp(ref_cc(&s, 1), ref_cc(&s, 2), g));
        let cm = ref_lerp(matrix_of(&fixture_cm1()), matrix_of(&fixture_cm2()), g);
        let c = mm(S_SRGB_FROM_XYZ_D65, B_BRADFORD_D50_TO_D65);
        mm(c, cm_cam2xyz(inv(mm(k, cm)).expect("invertible"), &s))
    };
    let camera_rgb = dual_camera_rgb();
    assert_ne!(
        api_to_srgb(&s, &camera_rgb),
        reference_apply(factors, &camera_rgb),
        "AC2 (d): the products, not the factors, are lerped"
    );
}

#[test]
fn ac2e_dual_forward_matrix_with_non_diagonal_calibrations() {
    let mut s = dual_fm_sensor();
    s.camera_calibration_1 = Some(cc_skew_1());
    s.camera_calibration_2 = Some(cc_skew_2());
    assert_calibrated(&s, "AC2 (e)");
}

/// AC2 (f), the midpoint branch: on [`dual_cm_sensor`] the step shrinks by about
/// 0.76 a pass and is still about 3.5e-6 at pass 30, so the loop ends on the
/// pass-30 midpoint, never on the tolerance (verify FU-1). The tolerance stop
/// is pinned separately, by
/// [`ac2f_the_loop_stops_by_tolerance_well_before_30_passes`].
#[test]
fn ac2f_the_loop_ends_on_the_midpoint_and_its_weight_is_not_the_three_pass_one() {
    let s = dual_cm_sensor();
    let three = ref_weight_three_pass(&s).expect("three-pass weight");
    let (converged, passes) = ref_weight_and_passes(&s).expect("converged weight");
    let ended = ref_neutral_to_xy(&s, REF_TOLERANCE).expect("loop");
    assert!(
        passes == 30 && !ended.by_tolerance,
        "this fixture must exercise the pass-30 midpoint, ran {passes} passes"
    );
    // Both weights, hand-pinned from the reference (SPEC-085 AC2 (f)). The
    // three-pass one is DEC-034's value, which SPEC-034's unit test pinned.
    assert!(
        (three - 0.5202685520213226).abs() < 1e-12,
        "three-pass weight {three}"
    );
    assert!(
        (converged - CONVERGED_WEIGHT).abs() < 1e-12,
        "converged weight {converged}"
    );
    assert!(
        passes > 3,
        "the fixture must need more than 3 passes, ran {passes}"
    );
    assert!((converged - three).abs() > 1e-4);

    let camera_rgb = dual_camera_rgb();
    let got = api_to_srgb(&s, &camera_rgb);
    assert_eq!(
        got,
        reference_apply(reference_matrix_at(&s, Some(converged)), &camera_rgb),
        "the API develops at the converged weight"
    );
    assert_ne!(
        got,
        reference_apply(reference_matrix_at(&s, Some(three)), &camera_rgb),
        "the API must not develop at the three-pass weight"
    );
}

/// AC2 (f)'s converged weight for [`dual_cm_sensor`], from the reference loop.
const CONVERGED_WEIGHT: f64 = 0.6708871184066116;

/// The Nikon D300S dual-illuminant tags (A / D65, ColorMatrix only) with its own
/// `AsShotNeutral`, on a 64×64 frame: a loop that stops by tolerance.
fn nikon_dual_sensor() -> Sensor {
    let mut s = fixture_sensor(64, 64);
    s.color_matrix_1 = Some(nikon_cm1());
    s.color_matrix_2 = Some(nikon_cm2());
    s.calibration_illuminant_1 = Some(ILLUMINANT_A);
    s.calibration_illuminant_2 = Some(ILLUMINANT_D65);
    s.as_shot_neutral = Some(nikon_asn());
    s
}

/// A spread of camera RGB over the full `u16` range, one triple per pixel of a
/// `width × height` frame (a multiplicative hash, so no two rows repeat).
fn spread_camera_rgb(width: u32, height: u32) -> Vec<u16> {
    (0..width * height * 3)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 16) as u16)
        .collect()
}

/// AC2 (f), the tolerance stop (verify FU-1): on the Nikon tags the step falls
/// under 1e-7 at pass 9, well before the cap. The pass count, the final `xy`
/// and the weight are hand-pinned from the reference. The API develops at that
/// weight, and both a 10× looser and a 10× tighter tolerance stop on a
/// different pass and develop to different codes, so a wrong tolerance in
/// `src/color.rs` turns this red.
#[test]
fn ac2f_the_loop_stops_by_tolerance_well_before_30_passes() {
    let s = nikon_dual_sensor();
    let ended = ref_neutral_to_xy(&s, REF_TOLERANCE).expect("loop");
    assert!(ended.by_tolerance, "must stop by tolerance: {ended:?}");
    assert_eq!(ended.passes, 9, "pass count: {ended:?}");
    assert_eq!(ended.xy, (0.3530156291392405, 0.34551698155535704));
    assert_eq!(ended.weight, 0.6954583018786118);

    let camera_rgb = spread_camera_rgb(64, 64);
    let got = api_to_srgb(&s, &camera_rgb);
    assert_eq!(
        got,
        reference_apply(reference_matrix_at(&s, Some(ended.weight)), &camera_rgb),
        "the API develops at the tolerance-stopped weight"
    );
    for tolerance in [REF_TOLERANCE * 10.0, REF_TOLERANCE / 10.0] {
        let other = ref_neutral_to_xy(&s, tolerance).expect("loop");
        assert_ne!(other.passes, ended.passes, "tolerance {tolerance:e}");
        assert_ne!(
            got,
            reference_apply(reference_matrix_at(&s, Some(other.weight)), &camera_rgb),
            "tolerance {tolerance:e} must develop differently, or the fixture cannot see it"
        );
    }
}

// ── AC3 — the signature rule, the shapes, the identity ──────────────────────

/// A mosaic fixture on the ColorMatrix path (no ForwardMatrix) with the Pixel
/// 7 Pro's diagonal calibration, so the signature rule is observable.
fn cm_mosaic_fixture() -> Sensor {
    let mut s = fused_fixture(9, 7, &[0, 1, 1, 2]);
    s.forward_matrix_1 = None;
    s.color_matrix_1 = Some(fixture_cm1());
    s.camera_calibration_1 = Some(cc_diag(8594, 3408));
    s
}

fn api_develop(sensor: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let mut dst = vec![0u16; mosaic.len() * 3];
    develop_color_into(sensor, mosaic, &mut dst).expect("develops");
    dst
}

#[test]
fn ac3_mismatched_signatures_develop_as_the_uncalibrated_file() {
    let base = cm_mosaic_fixture();
    let mosaic = varied_mosaic(9, 7);
    let uncalibrated = api_develop(&without_calibration(&base), &mosaic);
    for (camera, profile) in [
        (sig("a"), sig("b")),
        (sig("com.adobe"), None),
        (None, sig("com.adobe")),
    ] {
        let mut s = base.clone();
        s.camera_calibration_signature = camera.clone();
        s.profile_calibration_signature = profile.clone();
        assert_eq!(
            api_develop(&s, &mosaic),
            uncalibrated,
            "camera {camera:?} / profile {profile:?}: the calibration must not apply"
        );
    }
}

#[test]
fn ac3_matching_signatures_develop_as_the_calibrated_reference() {
    let base = cm_mosaic_fixture();
    let mosaic = varied_mosaic(9, 7);
    let uncalibrated = api_develop(&without_calibration(&base), &mosaic);
    for (camera, profile) in [(None, None), (sig("com.adobe"), sig("com.adobe"))] {
        let mut s = base.clone();
        s.camera_calibration_signature = camera.clone();
        s.profile_calibration_signature = profile.clone();
        let got = api_develop(&s, &mosaic);
        assert_eq!(
            got,
            reference_develop_color(&s, &mosaic),
            "camera {camera:?} / profile {profile:?}: the calibrated reference"
        );
        assert_ne!(got, uncalibrated, "{camera:?} / {profile:?}: not applied");
    }
}

/// Every public colour entry refuses `sensor` with `MissingColorCalibration`
/// and leaves `dst` untouched.
fn assert_refused_on_every_entry(sensor: &Sensor, what: &str) {
    use irradiance::color::{develop_color_f32_into_threaded, develop_color_into_threaded};
    let mosaic = varied_mosaic(sensor.width, sensor.height);
    let n = mosaic.len() * 3;
    let sentinel = 0xABCDu16;

    let mut dst = vec![sentinel; n];
    let r = to_srgb_linear_into(sensor, &vec![1000u16; n], &mut dst);
    assert!(
        matches!(r, Err(Error::MissingColorCalibration)),
        "{what}: to_srgb {r:?}"
    );
    assert!(dst.iter().all(|&v| v == sentinel), "{what}: to_srgb wrote");

    let mut dst = vec![sentinel; n];
    let r = develop_color_into(sensor, &mosaic, &mut dst);
    assert!(
        matches!(r, Err(Error::MissingColorCalibration)),
        "{what}: u16 {r:?}"
    );
    assert!(dst.iter().all(|&v| v == sentinel), "{what}: u16 wrote");

    let mut dst = vec![sentinel; n];
    let r = develop_color_into_threaded(sensor, &mosaic, &mut dst, 4);
    assert!(
        matches!(r, Err(Error::MissingColorCalibration)),
        "{what}: threaded {r:?}"
    );
    assert!(dst.iter().all(|&v| v == sentinel), "{what}: threaded wrote");

    let mut dst = vec![-1.0f32; n];
    let r = develop_color_f32_into(sensor, &mosaic, &mut dst);
    assert!(
        matches!(r, Err(Error::MissingColorCalibration)),
        "{what}: f32 {r:?}"
    );
    assert!(dst.iter().all(|&v| v == -1.0), "{what}: f32 wrote");

    let mut dst = vec![-1.0f32; n];
    let r = develop_color_f32_into_threaded(sensor, &mosaic, &mut dst, 4);
    assert!(
        matches!(r, Err(Error::MissingColorCalibration)),
        "{what}: f32 threaded {r:?}"
    );
    assert!(dst.iter().all(|&v| v == -1.0), "{what}: f32 threaded wrote");
}

#[test]
fn ac3_an_unusable_applied_calibration_is_refused_before_any_write() {
    let base = fused_fixture(9, 7, &[0, 1, 1, 2]);

    let mut short_cc = base.clone();
    let mut cc = cc_skew_1();
    cc.pop();
    short_cc.camera_calibration_1 = Some(cc);
    assert_refused_on_every_entry(&short_cc, "CC_1 of 8 entries");

    let mut zero_den = base.clone();
    let mut cc = cc_skew_1();
    cc[4] = Rational { num: 1, den: 0 };
    zero_den.camera_calibration_1 = Some(cc);
    assert_refused_on_every_entry(&zero_den, "CC_1 with a zero denominator");

    let mut short_ab = base.clone();
    let mut ab = ab_tilted();
    ab.pop();
    short_ab.analog_balance = Some(ab);
    assert_refused_on_every_entry(&short_ab, "AB of 2 entries");

    let mut zero_ab = base.clone();
    let mut ab = ab_tilted();
    ab[1] = Rational { num: 0, den: 1 };
    zero_ab.analog_balance = Some(ab);
    assert_refused_on_every_entry(&zero_ab, "AB with a zero numerator");

    // inv(K) = [[1,0,0],[0,1,0],[0,-2,1]] maps asn (1/2, 1, 1/2) to a reference
    // neutral with blue -3/2.
    let mut bad_neutral = base.clone();
    bad_neutral.camera_calibration_1 = Some(matrix_rationals([
        [10000, 0, 0],
        [0, 10000, 0],
        [0, 20000, 10000],
    ]));
    assert_refused_on_every_entry(&bad_neutral, "a non-positive reference neutral");
}

/// SPEC-085 pinned DEC-076 §3's "only what is used is checked": an unused
/// ill-shaped calibration developed as if absent. DEC-078 §7.2 (SPEC-089)
/// reverses it: a **present** calibration that cannot be read is refused, used
/// or not, never silently the identity. On a file the reader refuses it first as
/// `MalformedTag`; on these hand-built sensors the colour code's second line
/// refuses it as `MissingColorCalibration`.
#[test]
fn ac3_an_unused_ill_shaped_calibration_refuses() {
    // The signature rule does not apply it, and it is still checked.
    let mut unapplied = cm_mosaic_fixture();
    let mut cc = cc_skew_1();
    cc.pop();
    unapplied.camera_calibration_1 = Some(cc);
    unapplied.camera_calibration_signature = sig("a");
    unapplied.profile_calibration_signature = sig("b");
    assert_refused_on_every_entry(&unapplied, "an unapplied 8-entry CameraCalibration1");

    // A single-illuminant file never uses CC_2, and it is still checked.
    let mut single = cm_mosaic_fixture();
    let mut cc2 = cc_skew_2();
    cc2.pop();
    single.camera_calibration_2 = Some(cc2);
    assert_refused_on_every_entry(&single, "an unused 8-entry CameraCalibration2");
}

#[test]
fn ac3_an_identity_calibration_is_byte_identical_to_none() {
    let camera_rgb = dual_camera_rgb();
    for (what, base) in [
        ("single CM", single_cm_sensor()),
        ("single FM", single_fm_sensor()),
        ("dual CM", dual_cm_sensor()),
        ("dual FM", dual_fm_sensor()),
    ] {
        let mut s = base.clone();
        s.camera_calibration_1 = Some(cc_identity());
        s.camera_calibration_2 = Some(cc_identity());
        s.analog_balance = Some(ab_unity());
        assert_eq!(
            api_to_srgb(&s, &camera_rgb),
            api_to_srgb(&base, &camera_rgb),
            "{what}: an identity calibration must be byte-identical to none"
        );
    }
}

/// An `AsShotNeutral` red of 32263125/46358789, where `den/num` (DEC-033's `D`,
/// built from the rationals) and `1/(num/den)` (the calibrated path's `1/R`
/// with `R = inv(I) · asn`) differ in the last bit of `f64`.
fn asn_last_bit() -> Vec<Rational> {
    vec![
        Rational {
            num: 32_263_125,
            den: 46_358_789,
        },
        Rational { num: 1, den: 1 },
        Rational { num: 1, den: 2 },
    ]
}

/// DEC-076 §4's bit-identity on the ForwardMatrix path (verify FU-2): an
/// identity calibration must keep DEC-033's `D` from the rationals, not fold
/// through `1/R`. The pixels `(r, 0, 0)` for r = 3, 9, 15, 21 were found by
/// search to land on a rounding boundary where the two forms of `D` quantize
/// differently, so dropping the identity shortcut turns this red.
#[test]
fn ac3_an_identity_calibration_keeps_the_rational_white_balance_on_the_fm_path() {
    let mut base = single_fm_sensor();
    base.as_shot_neutral = Some(asn_last_bit());
    let red = &asn_last_bit()[0];
    let (num, den) = (f64::from(red.num), f64::from(red.den));
    assert_ne!(den / num, 1.0 / (num / den), "the two forms must differ");

    let camera_rgb: Vec<u16> = vec![3, 0, 0, 9, 0, 0, 15, 0, 0, 21, 0, 0];
    let c = mm(S_SRGB_FROM_XYZ_D65, B_BRADFORD_D50_TO_D65);
    let fm = matrix_of(&forward_matrix_d50());
    let rational = reference_apply(mm(c, fm_cam2xyz(fm, None, &base)), &camera_rgb);
    let folded = reference_apply(mm(c, fm_cam2xyz(fm, Some(I3), &base)), &camera_rgb);
    assert_ne!(rational, folded, "the fixture must see the difference");

    let mut s = base.clone();
    s.camera_calibration_1 = Some(cc_identity());
    s.camera_calibration_2 = Some(cc_identity());
    s.analog_balance = Some(ab_unity());
    let got = api_to_srgb(&s, &camera_rgb);
    assert_eq!(
        got,
        api_to_srgb(&base, &camera_rgb),
        "byte-identical to none"
    );
    assert_eq!(got, rational, "DEC-033's D from the rationals");
}

/// The f32 entry with a non-diagonal calibration applied (verify FU-4): it
/// equals `to_linear_f32_into` of the independent reference's `u16` develop to
/// the bit, single-threaded and threaded, on both the ForwardMatrix and the
/// ColorMatrix path.
#[test]
fn a_calibrated_f32_develop_equals_the_reference_to_the_bit() {
    use irradiance::color::develop_color_f32_into_threaded;
    let (w, h) = (20, 24);
    let mut fm = fused_fixture(w, h, &[0, 1, 1, 2]);
    fm.color_matrix_1 = Some(fixture_cm1());
    fm.color_matrix_2 = Some(fixture_cm2());
    fm.forward_matrix_2 = Some(forward_matrix_cool());
    fm.calibration_illuminant_1 = Some(ILLUMINANT_A);
    fm.calibration_illuminant_2 = Some(ILLUMINANT_D65);
    fm.camera_calibration_1 = Some(cc_skew_1());
    fm.camera_calibration_2 = Some(cc_skew_2());
    fm.analog_balance = Some(ab_tilted());
    let mut cm = fm.clone();
    cm.forward_matrix_1 = None;
    cm.forward_matrix_2 = None;

    let mosaic = varied_mosaic(w, h);
    for (path, s) in [("FM", fm), ("CM", cm)] {
        let reference = reference_develop_color(&s, &mosaic);
        assert_ne!(
            reference,
            reference_develop_color(&without_calibration(&s), &mosaic),
            "{path}: the calibration must move the reference"
        );
        let mut expected = vec![0.0f32; reference.len()];
        to_linear_f32_into(&reference, &mut expected).expect("to_linear_f32_into");

        let mut got = vec![0.0f32; reference.len()];
        develop_color_f32_into(&s, &mosaic, &mut got).expect("f32 develop");
        assert_eq!(got, expected, "{path}: f32 must equal the reference");

        let mut threaded = vec![0.0f32; reference.len()];
        develop_color_f32_into_threaded(&s, &mosaic, &mut threaded, 4).expect("threaded");
        assert_eq!(threaded, expected, "{path}: threaded f32");
    }
}

#[test]
fn a_calibrated_threaded_develop_equals_single_thread() {
    use irradiance::color::develop_color_into_threaded;
    let mut s = fused_fixture(20, 24, &[0, 1, 1, 2]);
    s.color_matrix_1 = Some(fixture_cm1());
    s.color_matrix_2 = Some(fixture_cm2());
    s.forward_matrix_2 = Some(forward_matrix_cool());
    s.calibration_illuminant_1 = Some(ILLUMINANT_A);
    s.calibration_illuminant_2 = Some(ILLUMINANT_D65);
    s.camera_calibration_1 = Some(cc_skew_1());
    s.camera_calibration_2 = Some(cc_skew_2());
    s.analog_balance = Some(ab_tilted());
    let mosaic = varied_mosaic(20, 24);
    let single = api_develop(&s, &mosaic);
    let mut threaded = vec![0u16; single.len()];
    develop_color_into_threaded(&s, &mosaic, &mut threaded, 4).expect("threaded");
    assert_eq!(threaded, single);
    assert_eq!(single, reference_develop_color(&s, &mosaic));
}
