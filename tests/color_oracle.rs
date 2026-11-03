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

use irradiance::color::{develop_color_into, to_srgb_linear_into};
use irradiance::ifd::{Cfa, Compression, Rational, Sensor};
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
                mm(inv(cm).expect("interpolated CM invertible"), wb)
            }
        }
        None => {
            if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
                mm(matrix_of(fm1), wb)
            } else {
                let cm = matrix_of(sensor.color_matrix_1.as_ref().expect("cm1"));
                mm(inv(cm).expect("invertible"), wb)
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

/// Mirror of `color::normalize_mosaic`: per-2×2-position level-normalize.
fn reference_normalize(sensor: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let width = sensor.width as usize;
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
            let row = pixel / width;
            let col = pixel % width;
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
