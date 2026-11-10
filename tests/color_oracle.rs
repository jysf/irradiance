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
use irradiance::ifd::{
    ActiveArea, Cfa, Compression, DefaultCropOrigin, DefaultCropSize, Rational, Sensor,
};
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

/// Reference scene weight by the DNG NeutralToXY method (`None` ⇒ fallback).
fn ref_weight(sensor: &Sensor) -> Option<f64> {
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
    let (m1, m2) = (matrix_of(cm1), matrix_of(cm2));
    let mut g = 0.0f64;
    for _ in 0..3 {
        let ic = inv(ref_lerp(m1, m2, g))?;
        let w = ref_mul_vec(ic, neutral);
        let sum = w[0] + w[1] + w[2];
        if sum <= 0.0 {
            return None;
        }
        let t = ref_mccamy(w[0] / sum, w[1] / sum)?;
        g = ((inv_t1 - 1.0 / t) / denom).clamp(0.0, 1.0);
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

/// Reference build of `M = C · cam2xyz`. When the dual-calibration preconditions
/// hold (`ref_weight` is `Some`), `cam2xyz` is the illuminant-interpolated
/// `FM(g) · D` (both FMs present) else `inv(CM(g)) · D`; otherwise it is the
/// SPEC-033 single-illuminant path (`FM1` else `inv(CM1)`).
fn reference_matrix(sensor: &Sensor) -> [[f64; 3]; 3] {
    let wb = wb_of(sensor.as_shot_neutral.as_ref().expect("asn"));
    let cam2xyz = match ref_weight(sensor) {
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
                mm(ref_lerp(f1, f2, g), wb)
            } else {
                let cm = ref_lerp(
                    matrix_of(sensor.color_matrix_1.as_ref().expect("cm1")),
                    matrix_of(sensor.color_matrix_2.as_ref().expect("cm2")),
                    g,
                );
                cm_cam2xyz(inv(cm).expect("interpolated CM invertible"), sensor)
            }
        }
        None => {
            if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
                mm(matrix_of(fm1), wb)
            } else {
                let cm = matrix_of(sensor.color_matrix_1.as_ref().expect("cm1"));
                cm_cam2xyz(inv(cm).expect("invertible"), sensor)
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
    Sensor {
        ifd_index: 0,
        width,
        height,
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
        color_matrix_1: None,
        color_matrix_2: None,
        forward_matrix_1: None,
        forward_matrix_2: None,
        as_shot_neutral: None,
        calibration_illuminant_1: None,
        calibration_illuminant_2: None,
        black_levels: vec![],
        white_levels: vec![],
        malformed_tags: vec![],
    }
}

/// A 2×2 RGGB CFA with the DNG-default plane colour.
fn rggb() -> Cfa {
    Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 1, 2],
        plane_color: vec![0, 1, 2],
        layout: 1,
    }
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
    let normalize = |sample: u16, black: u32, white: u32| -> u16 {
        let value = u32::from(sample);
        let clamped = value.clamp(black.min(white), black.max(white));
        let numerator = u64::from(clamped.saturating_sub(black)) * u64::from(u16::MAX);
        let denominator = u64::from(white.saturating_sub(black));
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
    Cfa {
        repeat_dim: [2, 2],
        pattern: pattern.to_vec(),
        plane_color: vec![0, 1, 2],
        layout: 1,
    }
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
    s.color_filter = Some(Cfa {
        repeat_dim: [2, 2],
        pattern: pattern.to_vec(),
        plane_color: vec![0, 1, 2],
        layout: 1,
    });
    s.forward_matrix_1 = Some(forward_matrix_d50());
    s.as_shot_neutral = Some(nikon_asn());
    s.black_levels = SLOT_BLACK.to_vec();
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
/// `black_slot + k·(white − black_slot)·asn[colour]`, where the 2×2 slot and its
/// colour come from the position *relative to `build_origin`*. Normalizing by
/// that slot's `[black, white]` therefore recovers `k·65535·asn[colour]` for
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
            let raw = black + GREY_K * (white - black) * asn[colour];
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
