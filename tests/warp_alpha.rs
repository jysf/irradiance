//! `SPEC-064` — `DevelopOptions::warp_alpha` over `WarpRect::lerp`, and the
//! `FullActiveArea` bounds rule that replaces `SPEC-063`'s blanket refusal
//! (`DEC-059`, amending `DEC-058` §4).
//!
//! The load-bearing oracle is `AC5`: on every held Q2M frame and at five
//! values of α, `FullActiveArea` develops and its output **contains** the
//! `StandardCrop` output of the same α at the crop origin. The warp runs over
//! the active area under both framings, so the two must agree exactly. At
//! α = 1 the `StandardCrop` output also matches `SPEC-063`'s MD5 pin, which
//! was taken from main before either spec changed anything.
//!
//! Every other assertion is a **relation** between public-API calls, or an
//! analytic expectation computed from the coefficients in the test itself
//! (`AC4`'s impulse, `AC6`'s expected `scale_max`). None re-implements the
//! develop arithmetic.
//!
//! Two lanes, as `tests/develop_options.rs` establishes: tier A builds a
//! `Sensor` from `Sensor::test_fixture()` (needs the `test-fixtures` feature,
//! which `just test`'s `--all-features` turns on); tier B reads the held files
//! under `$IRRADIANCE_CORPUS_DIR` in place (`DEC-003`) and skips per entry, by
//! name, when one is absent (`CorpusFile::require`).
//!
//! The Pixel 3a and Pixel 7 Pro coefficients below are the full-precision
//! values the `SPEC-064` design probe read through `parse_warp_rectilinear`
//! (spec §1). Pixel files do not develop (an unrecognised mandatory opcode 9),
//! so their real polynomials enter here as numbers in tier-A fixtures, never
//! as corpus bytes.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/opcode.rs"]
mod opcode_support;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{
    develop_into, develop_with_into, output_dimensions, output_dimensions_with, DevelopOptions,
    Framing,
};
use irradiance::ifd::{
    ActiveArea, ByteOrder, Container, DefaultCropOrigin, DefaultCropSize, Sensor,
};
use irradiance::opcode::WarpRect;
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind, DEVELOP_VERSION};

// ─────────────────────────────────────────────────────────────────────────────
// Coefficients (spec §1)
// ─────────────────────────────────────────────────────────────────────────────

/// The three held Q2M frames' `kr`, as `tests/warp.rs` carries them. Each is
/// centred (`cx = cy = 0.5`).
const Q2M_KR: [(&str, [f64; 4]); 3] = [
    (
        "L1021223",
        [
            0.999_251_106,
            -0.061_376_512_877_263_58,
            -0.093_915_541_393_360_16,
            0.055_882_009_215_291_75,
        ],
    ),
    (
        "L1026016",
        [
            0.999_251_106,
            -0.041_845_169_276_595_75,
            -0.102_311_478_808_510_63,
            0.057_876_719_170_212_77,
        ],
    ),
    (
        "L1026192",
        [
            0.999_251_106,
            -0.032_331_453_292_576_424,
            -0.104_204_110_174_672_49,
            0.056_364_293_537_117_91,
        ],
    ),
];

/// L1021223's `kr` alone — the frame with the largest corner pull.
const L1021223_KR: [f64; 4] = Q2M_KR[0].1;

/// Pixel 3a: a real over-unity lens. Its scale exceeds 1 near the corners at
/// every α > 0 (spec §1: α = 1 max 1.001529254).
const PIXEL_3A_KR: [f64; 4] = [
    0.980_158_686_637_878_4,
    0.091_988_243_162_631_99,
    -0.177_878_528_833_389_28,
    0.107_260_853_052_139_28,
];
const PIXEL_3A_CX: f64 = 0.501_746_677_217_029_4;
const PIXEL_3A_CY: f64 = 0.502_773_708_767_361_2;
/// The probe's α = 1 `scale_max` for the Pixel 3a, at its printed precision.
const PIXEL_3A_SCALE_MAX: f64 = 1.001_529_254;

/// Pixel 7 Pro: an off-centre lens whose scale stays in `[0, 1]`.
const PIXEL_7_PRO_KR: [f64; 4] = [
    0.989_696_025_848_388_7,
    0.009_945_581_667_125_225,
    -0.003_183_106_426_149_606_7,
    0.003_037_228_714_674_711_2,
];
const PIXEL_7_PRO_CX: f64 = 0.499_835_510_096_680_2;
const PIXEL_7_PRO_CY: f64 = 0.491_674_211_869_976_9;

/// The α values `AC5` / `AC6` run over.
const ALPHAS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

// ─────────────────────────────────────────────────────────────────────────────
// Shared fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A value no develop of these fixtures produces at the positions checked, so
/// an untouched `dst` is distinguishable from a written one.
const SENTINEL: u16 = 0xABCD;

/// The tier-A fixture `tests/develop_options.rs` uses: a 10×8 plane with a
/// 7×6 `ActiveArea` at `(left 2, top 1)` and a non-square 4×3 `DefaultCrop` at
/// `(1, 1)` inside it. Identity levels keep raw values recognisable.
fn bordered_sensor() -> Sensor {
    let mut sensor = Sensor::test_fixture();
    sensor.width = 10;
    sensor.height = 8;
    sensor.bits_per_sample = 16;
    sensor.black_level = Some(0);
    sensor.white_level = Some(u32::from(u16::MAX));
    sensor.active_area = Some(ActiveArea {
        top: 1,
        left: 2,
        bottom: 7,
        right: 9,
    });
    sensor.default_crop_origin = Some(DefaultCropOrigin { x: 1, y: 1 });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 4,
        height: 3,
    });
    sensor.orientation = Some(1);
    sensor
}

/// [`bordered_sensor`] carrying a `WarpRectilinear` with these coefficients
/// and a mandatory `FixBadPixelsConstant` in `OpcodeList1`.
fn warped_sensor(kr: [f64; 4], kt: [f64; 2], cx: f64, cy: f64) -> Sensor {
    let mut sensor = bordered_sensor();
    sensor.opcode_list_1 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    sensor.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        kr, kt, cx, cy, 1, 0,
    ));
    sensor
}

/// The same sensor without its warp.
fn without_warp(sensor: &Sensor) -> Sensor {
    let mut plain = sensor.clone();
    plain.opcode_list_3 = None;
    plain
}

/// A `FixBadPixelsConstant` with a 4-byte parameter block: it passes the
/// required-processing check (which reads no parameters) but fails its own
/// parse with `MalformedOpcodeParams`. `SPEC-063`'s precedence probe.
fn malformed_fix_bad_pixels() -> Vec<u8> {
    opcode_support::opcode_list(&[opcode_support::raw_opcode(4, 0x0103_0000, 0, &[0, 0, 0, 0])])
}

/// Distinct sample values for the sensor's plane (no two equal, none 0).
fn distinct_plane(sensor: &Sensor) -> Vec<u16> {
    let len = (sensor.width * sensor.height) as usize;
    (0..len).map(|i| 100 + 7 * i as u16).collect()
}

/// Options for `framing` at `warp_alpha`, set by field assignment on
/// `Default` (`AC11`: the only way an external caller can build them).
fn opts(framing: Framing, warp_alpha: f64) -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = framing;
    options.warp_alpha = warp_alpha;
    options
}

/// Develop with `options` into a freshly sized buffer.
fn develop_with(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> (u32, u32, Vec<u16>) {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions resolve");
    let mut dst = vec![0u16; (w as usize) * (h as usize)];
    develop_with_into(sensor, src, &mut dst, options)
        .unwrap_or_else(|e| panic!("develops under {options:?}: {e}"));
    (w, h, dst)
}

/// A `WarpRect` with these `kr`, built from the dev-only fixture
/// (`#[non_exhaustive]`, `DEC-052`).
fn warp_rect(kr: [f64; 4]) -> WarpRect {
    let mut warp = WarpRect::test_fixture();
    warp.kr = kr;
    warp
}

/// Assert that `inner` (`inner_w` wide) sits inside `outer` (`outer_w` wide)
/// with its top-left at `(x0, y0)`.
fn assert_contains_at(
    label: &str,
    outer: &[u16],
    outer_w: u32,
    inner: &[u16],
    inner_w: u32,
    (x0, y0): (u32, u32),
) {
    let inner_h = inner.len() as u32 / inner_w;
    for y in 0..inner_h {
        let inner_row = &inner[(y * inner_w) as usize..((y + 1) * inner_w) as usize];
        let start = ((y0 + y) * outer_w + x0) as usize;
        let outer_row = &outer[start..start + inner_w as usize];
        assert!(
            inner_row == outer_row,
            "{label}: row {y} of the StandardCrop output is not at ({x0}, {y0} + {y}) of the \
             FullActiveArea output"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B helpers (as tests/develop_options.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// The `Sensor` for one held file, or `None` with the skip announced by name.
fn corpus_sensor(path: &str) -> Option<(Sensor, ByteOrder, Vec<u8>)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(&root)?;
    let bytes = std::fs::read(&file_path).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));
    let byte_order = container.byte_order();
    Some((sensor, byte_order, bytes))
}

/// The held file's sensor and its unpacked raw plane.
fn corpus_plane(path: &str) -> Option<(Sensor, Vec<u16>)> {
    let (sensor, byte_order, bytes) = corpus_sensor(path)?;
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, byte_order, &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    Some((sensor, plane))
}

/// MD5 of `samples` as little-endian `u16` bytes — `SPEC-063` §1's encoding.
fn md5_le(samples: &[u16]) -> String {
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    md5::to_hex(&md5::hash(&bytes))
}

/// `SPEC-063` §1's `develop_into` MD5 pins, copied unedited from
/// `tests/develop_options.rs`. `(path, width, height, md5)`.
const PINNED: [(&str, u32, u32, &str); 8] = [
    (
        "LEICA-Q2-MONO/L1021223.DNG",
        8368,
        5584,
        "feb4910588d189bd5bf89620b480fe6c",
    ),
    (
        "LEICA-Q2-MONO/L1026016.DNG",
        5584,
        8368,
        "2bc53a6337b582a8080ac1206e62434b",
    ),
    (
        "LEICA-Q2-MONO/L1026192.DNG",
        8368,
        5584,
        "f9a9c56c6255436ac7e764cb0a7c21fb",
    ),
    (
        "LEICA-M-MONOCHROM/L1000622.DNG",
        5212,
        3468,
        "fddf0ea0dec6a66bf1fd1cb8c787cc79",
    ),
    (
        "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
        5976,
        3992,
        "4aa013f64317412111c9d298300dcb28",
    ),
    (
        "PENTAX-K3III-MONO/K3III.DNG",
        6192,
        4128,
        "82d2d51957922587ec55ee594b2a6b14",
    ),
    (
        "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
        5760,
        3840,
        "f533e4052026097e799d46bc8bc794bc",
    ),
    (
        "NIKON-D300S/_DSC4615.DNG",
        4288,
        2848,
        "09faddddd24a11497cc072bcf3a42ce9",
    ),
];

/// The three held Q2M frames (the first three pins).
const Q2M_PATHS: [&str; 3] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "LEICA-Q2-MONO/L1026016.DNG",
    "LEICA-Q2-MONO/L1026192.DNG",
];

/// The SPEC-063 pin for `path`.
fn pin_for(path: &str) -> &'static str {
    PINNED
        .iter()
        .find(|(p, _, _, _)| *p == path)
        .map(|(_, _, _, md5)| *md5)
        .unwrap_or_else(|| panic!("{path} has no SPEC-063 pin"))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — default
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn default_warp_alpha_is_one_and_pins_hold() {
    let default = DevelopOptions::default();
    assert_eq!(default.warp_alpha.to_bits(), 1.0f64.to_bits());
    assert_eq!(default.framing, Framing::StandardCrop);
    assert!(default.apply_orientation);
    assert_eq!(
        DEVELOP_VERSION, "4",
        "SPEC-064 does not bump DEVELOP_VERSION"
    );
    // An explicit 1.0 is the default value, not merely an equivalent one.
    assert_eq!(opts(Framing::StandardCrop, 1.0), default);
    assert_ne!(opts(Framing::StandardCrop, 0.5), default);

    // Tier A: an explicit α = 1 equals develop_into on a warped sensor.
    let sensor = warped_sensor(L1021223_KR, [0.0, 0.0], 0.5, 0.5);
    let src = distinct_plane(&sensor);
    let (w, h) = output_dimensions(&sensor).unwrap();
    let mut plain = vec![0u16; (w * h) as usize];
    develop_into(&sensor, &src, &mut plain).unwrap();
    let (_, _, explicit) = develop_with(&sensor, &src, &opts(Framing::StandardCrop, 1.0));
    assert_eq!(explicit, plain);

    // Tier B: every SPEC-063 pin, through an explicit warp_alpha = 1.0.
    for (path, width, height, pinned) in PINNED {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue; // SKIP announced by name by CorpusFile::require
        };
        let options = opts(Framing::StandardCrop, 1.0);
        assert_eq!(
            output_dimensions_with(&sensor, &options).unwrap(),
            (width, height),
            "{path}"
        );
        let mut dst = vec![SENTINEL; (width as usize) * (height as usize)];
        develop_with_into(&sensor, &plane, &mut dst, &options)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(md5_le(&dst), pinned, "{path}: explicit warp_alpha = 1.0");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — lerp is exact at its ends
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lerp_is_bit_exact_at_one_and_identity_at_zero() {
    let mut sets: Vec<(&str, WarpRect)> = Q2M_KR
        .iter()
        .map(|&(name, kr)| (name, warp_rect(kr)))
        .collect();
    let mut pixel_3a = warp_rect(PIXEL_3A_KR);
    pixel_3a.cx = PIXEL_3A_CX;
    pixel_3a.cy = PIXEL_3A_CY;
    sets.push(("Pixel 3a", pixel_3a));
    // A set with non-zero tangential terms and more than one plane, so kt
    // and planes are exercised too.
    let mut tangential = warp_rect(L1021223_KR);
    tangential.kt = [0.01, -0.02];
    tangential.planes = 3;
    sets.push(("tangential", tangential));

    for (name, w) in &sets {
        // α = 1: bit for bit.
        let one = w.lerp(1.0);
        for i in 0..4 {
            assert_eq!(
                one.kr[i].to_bits(),
                w.kr[i].to_bits(),
                "{name}: lerp(1).kr[{i}]"
            );
        }
        for i in 0..2 {
            assert_eq!(
                one.kt[i].to_bits(),
                w.kt[i].to_bits(),
                "{name}: lerp(1).kt[{i}]"
            );
        }
        assert_eq!(one.cx.to_bits(), w.cx.to_bits(), "{name}: cx");
        assert_eq!(one.cy.to_bits(), w.cy.to_bits(), "{name}: cy");
        assert_eq!(one.planes, w.planes, "{name}: planes");

        // α = 0: the identity coefficients (`-0.0` compares equal to 0.0).
        let zero = w.lerp(0.0);
        assert_eq!(zero.kr, [1.0, 0.0, 0.0, 0.0], "{name}: lerp(0).kr");
        assert_eq!(zero.kt, [0.0, 0.0], "{name}: lerp(0).kt");

        // α = 0.5: kr0 moves halfway to 1, every other coefficient halves.
        let half = w.lerp(0.5);
        assert_eq!(half.kr[0], 0.5 + 0.5 * w.kr[0], "{name}: lerp(0.5).kr0");
        for i in 1..4 {
            assert_eq!(half.kr[i], 0.5 * w.kr[i], "{name}: lerp(0.5).kr[{i}]");
        }
        for i in 0..2 {
            assert_eq!(half.kt[i], 0.5 * w.kt[i], "{name}: lerp(0.5).kt[{i}]");
        }

        // cx, cy and planes are unchanged at every α.
        for alpha in ALPHAS {
            let l = w.lerp(alpha);
            assert_eq!(l.cx.to_bits(), w.cx.to_bits(), "{name}: cx at {alpha}");
            assert_eq!(l.cy.to_bits(), w.cy.to_bits(), "{name}: cy at {alpha}");
            assert_eq!(l.planes, w.planes, "{name}: planes at {alpha}");
        }
    }

    // α = 0 through the develop path: a sensor whose warp lerps to identity
    // takes the warp-free path, even under FullActiveArea and even for the
    // over-unity Pixel 3a polynomial.
    let sensor = warped_sensor(PIXEL_3A_KR, [0.0, 0.0], PIXEL_3A_CX, PIXEL_3A_CY);
    let src = distinct_plane(&sensor);
    let (_, _, lerped) = develop_with(&sensor, &src, &opts(Framing::FullActiveArea, 0.0));
    let (_, _, plain) = develop_with(
        &without_warp(&sensor),
        &src,
        &opts(Framing::FullActiveArea, 1.0),
    );
    assert_eq!(lerped, plain, "lerp(0) is the identity warp");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — α = 0 equals no warp
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn alpha_zero_equals_the_warp_free_develop() {
    // Tier A: both framings, with and without orientation.
    let sensor = warped_sensor(L1021223_KR, [0.0, 0.0], 0.5, 0.5);
    let src = distinct_plane(&sensor);
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        for apply_orientation in [true, false] {
            let mut zero = opts(framing, 0.0);
            zero.apply_orientation = apply_orientation;
            let mut full = zero;
            full.warp_alpha = 1.0;

            let (_, _, at_zero) = develop_with(&sensor, &src, &zero);
            let (_, _, no_warp) = develop_with(&without_warp(&sensor), &src, &full);
            assert_eq!(
                at_zero, no_warp,
                "{framing:?}, oriented {apply_orientation}"
            );
        }
    }
    // The fixture's warp is not a no-op at α = 1, so the relation above is
    // not vacuous.
    let (_, _, at_one) = develop_with(&sensor, &src, &opts(Framing::FullActiveArea, 1.0));
    let (_, _, no_warp) = develop_with(
        &without_warp(&sensor),
        &src,
        &opts(Framing::FullActiveArea, 1.0),
    );
    assert_ne!(at_one, no_warp, "the warp moves pixels at α = 1");

    // Tier B: the three Q2M frames, both framings. An MD5 comparison of two
    // outputs computed here, not a pin. Under FullActiveArea this is also the
    // first real-file success on the Q2M.
    for path in Q2M_PATHS {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue;
        };
        assert!(sensor.opcode_list_3.is_some(), "{path}: carries a warp");
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let (w, h, at_zero) = develop_with(&sensor, &plane, &opts(framing, 0.0));
            let (_, _, no_warp) = develop_with(&without_warp(&sensor), &plane, &opts(framing, 1.0));
            if framing == Framing::FullActiveArea {
                let swapped = sensor.orientation.unwrap_or(1) >= 5;
                let expected = if swapped { (5632, 8392) } else { (8392, 5632) };
                assert_eq!((w, h), expected, "{path}: FullActiveArea dimensions");
            }
            assert_eq!(
                md5_le(&at_zero),
                md5_le(&no_warp),
                "{path} {framing:?}: α = 0 equals the warp-free develop"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — displacement is linear in α (analytic oracle, tier A)
// ─────────────────────────────────────────────────────────────────────────────

/// The Q2M crop size `tests/warp.rs`'s corner-impulse fixture uses.
const IMPULSE_W: u32 = 8368;
const IMPULSE_H: u32 = 5584;

/// A whole-plane-active, uncropped `IMPULSE_W × IMPULSE_H` sensor with
/// identity levels, carrying L1021223's warp — so `develop_with_into`'s only
/// pixel transform is the (lerped) warp.
fn impulse_sensor() -> Sensor {
    let mut sensor = Sensor::test_fixture();
    sensor.width = IMPULSE_W;
    sensor.height = IMPULSE_H;
    sensor.bits_per_sample = 16;
    sensor.black_level = Some(0);
    sensor.white_level = Some(u32::from(u16::MAX));
    sensor.orientation = Some(1);
    sensor.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        L1021223_KR,
        [0.0, 0.0],
        0.5,
        0.5,
        1,
        0,
    ));
    sensor
}

/// The bottom-right corner's source coordinate at `alpha`, computed here from
/// DNG 1.7.0.0 §6.4.1 and DEC-059 §1, independent of `src/`: at the corner
/// `r = 1`, so the lerped scale is `(1 - α) + α·f(1)`.
fn corner_source_at(alpha: f64) -> (f64, f64) {
    let cx = f64::from(IMPULSE_W - 1) / 2.0;
    let cy = f64::from(IMPULSE_H - 1) / 2.0;
    let f1: f64 = L1021223_KR.iter().sum();
    let f_alpha = (1.0 - alpha) + alpha * f1;
    (
        cx + f_alpha * (f64::from(IMPULSE_W - 1) - cx),
        cy + f_alpha * (f64::from(IMPULSE_H - 1) - cy),
    )
}

/// Develop an impulse placed at α's predicted corner source, and return the
/// distance from the impulse to the developed peak — the detected
/// displacement.
fn detected_displacement(sensor: &Sensor, alpha: f64) -> f64 {
    let (sx, sy) = corner_source_at(alpha);
    let (px, py) = (sx.round() as u32, sy.round() as u32);
    let mut src = vec![0u16; (IMPULSE_W as usize) * (IMPULSE_H as usize)];
    src[(py * IMPULSE_W + px) as usize] = u16::MAX;
    let (w, _, dst) = develop_with(sensor, &src, &opts(Framing::StandardCrop, alpha));
    let (index, _) = dst
        .iter()
        .enumerate()
        .max_by_key(|&(_, &v)| v)
        .expect("non-empty");
    let peak_x = (index % w as usize) as f64;
    let peak_y = (index / w as usize) as f64;
    eprintln!("α = {alpha}: impulse at ({px}, {py}), developed peak at ({peak_x}, {peak_y})");
    (peak_x - f64::from(px)).hypot(peak_y - f64::from(py))
}

#[test]
fn displacement_is_linear_in_alpha() {
    let sensor = impulse_sensor();
    let full = detected_displacement(&sensor, 1.0);
    // The analytic α = 1 corner pull at this size: m·(1 − f(1)), with m the
    // half-diagonal. ≈ 503.8 px at the 8368×5584 crop size (SPEC-064 §1's
    // 506.1 px is the same lens over the 8392×5632 active area).
    let cx = f64::from(IMPULSE_W - 1) / 2.0;
    let cy = f64::from(IMPULSE_H - 1) / 2.0;
    let analytic = cx.hypot(cy) * (1.0 - L1021223_KR.iter().sum::<f64>());
    assert!(
        (full - analytic).abs() <= 1.5,
        "α = 1 corner displacement {full:.1} px, analytic {analytic:.1}"
    );
    for alpha in [0.5, 0.25] {
        let got = detected_displacement(&sensor, alpha);
        let want = alpha * full;
        assert!(
            (got - want).abs() <= 1.0,
            "α = {alpha}: displacement {got:.2} px, expected {want:.2} (α × {full:.2}) within 1 px"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — the bounds oracle on the held frames (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn full_active_area_develops_every_q2m_frame_at_five_alphas() {
    for path in Q2M_PATHS {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue; // SKIP announced by name by CorpusFile::require
        };
        let origin = sensor
            .default_crop_origin
            .map(|o| (o.x, o.y))
            .expect("Q2M carries DefaultCropOrigin");
        assert_eq!(origin, (12, 24), "{path}: crop origin");

        for alpha in ALPHAS {
            // Both outputs in stored order, so containment is a plain
            // sub-rectangle relation.
            let mut full_opts = opts(Framing::FullActiveArea, alpha);
            full_opts.apply_orientation = false;
            let mut crop_opts = opts(Framing::StandardCrop, alpha);
            crop_opts.apply_orientation = false;

            let (fw, fh, full) = develop_with(&sensor, &plane, &full_opts);
            assert_eq!(
                (fw, fh),
                (8392, 5632),
                "{path} α {alpha}: FullActiveArea size"
            );
            let (sw, sh, standard) = develop_with(&sensor, &plane, &crop_opts);
            assert_eq!(
                (sw, sh),
                (8368, 5584),
                "{path} α {alpha}: StandardCrop size"
            );
            assert_contains_at(
                &format!("{path} α {alpha}"),
                &full,
                fw,
                &standard,
                sw,
                origin,
            );

            // At α = 1 the StandardCrop output is SPEC-063's pin. Orientation
            // 1 frames already hold it in stored order; L1026016 (Orientation
            // 6) is pinned oriented, so develop it once more with the default.
            if alpha == 1.0 {
                let oriented = if sensor.orientation.unwrap_or(1) == 1 {
                    standard
                } else {
                    develop_with(&sensor, &plane, &opts(Framing::StandardCrop, 1.0)).2
                };
                assert_eq!(md5_le(&oriented), pin_for(path), "{path}: α = 1 pin");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the over-unity red-proof and the refusal
// ─────────────────────────────────────────────────────────────────────────────

/// The Pixel 3a fixture (an over-unity lens) with a `FixBadPixelsConstant`.
fn pixel_3a_sensor() -> Sensor {
    warped_sensor(PIXEL_3A_KR, [0.0, 0.0], PIXEL_3A_CX, PIXEL_3A_CY)
}

/// Run `develop_with_into` into a sentinel-filled buffer of the right size and
/// return the error, asserting the buffer stayed untouched.
fn refused(label: &str, sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> Error {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions resolve");
    let mut dst = vec![SENTINEL; (w as usize) * (h as usize)];
    let err = develop_with_into(sensor, src, &mut dst, options)
        .expect_err(&format!("{label}: must be refused"));
    assert!(
        dst.iter().all(|&v| v == SENTINEL),
        "{label}: dst must be untouched"
    );
    err
}

#[test]
fn over_unity_pixel_3a_warp_is_refused_before_any_write() {
    let sensor = pixel_3a_sensor();
    let src = distinct_plane(&sensor);
    let mut malformed = sensor.clone();
    malformed.opcode_list_1 = Some(malformed_fix_bad_pixels());

    for alpha in [0.25, 0.5, 0.75, 1.0] {
        let options = opts(Framing::FullActiveArea, alpha);
        let err = refused(&format!("Pixel 3a α {alpha}"), &sensor, &src, &options);
        assert_eq!(err.kind(), ErrorKind::Unsupported);
        match err {
            Error::UnsupportedWarpedFullActiveArea {
                active_width,
                active_height,
                warp_alpha,
                scale_min,
                scale_max,
            } => {
                assert_eq!((active_width, active_height), (7, 6), "α {alpha}");
                assert_eq!(warp_alpha.to_bits(), alpha.to_bits(), "α {alpha}");
                assert!(scale_max > 1.0, "α {alpha}: scale_max {scale_max}");
                // §1: at each α the range is 1 − α·(1 − range at α = 1).
                let want_max = 1.0 - alpha * (1.0 - PIXEL_3A_SCALE_MAX);
                assert!(
                    (scale_max - want_max).abs() < 1e-9,
                    "α {alpha}: scale_max {scale_max}, probe {want_max}"
                );
                assert!(scale_min > 0.0 && scale_min < 1.0, "α {alpha}: {scale_min}");
            }
            other => panic!("α {alpha}: got {other:?}"),
        }

        // Precedence: the refusal comes before OpcodeList1 is parsed (and so
        // before the FixBadPixelsConstant copy).
        let err = refused(
            &format!("Pixel 3a α {alpha}, malformed OpcodeList1"),
            &malformed,
            &src,
            &options,
        );
        assert!(
            matches!(err, Error::UnsupportedWarpedFullActiveArea { .. }),
            "α {alpha}: the bounds refusal precedes OpcodeList1's parse: got {err:?}"
        );
    }

    // α = 0 develops, warp-free.
    let (w, h, at_zero) = develop_with(&sensor, &src, &opts(Framing::FullActiveArea, 0.0));
    assert_eq!((w, h), (7, 6));
    let (_, _, no_warp) = develop_with(
        &without_warp(&sensor),
        &src,
        &opts(Framing::FullActiveArea, 1.0),
    );
    assert_eq!(at_zero, no_warp);
}

#[test]
fn over_unity_warp_still_develops_under_standard_crop() {
    // DEC-059 §4: no new refusal under StandardCrop, at any α.
    let sensor = pixel_3a_sensor();
    let src = distinct_plane(&sensor);
    let (w, h) = output_dimensions(&sensor).unwrap();
    let mut plain = vec![0u16; (w * h) as usize];
    develop_into(&sensor, &src, &mut plain).expect("develop_into develops the Pixel 3a warp");
    let (_, _, with) = develop_with(&sensor, &src, &opts(Framing::StandardCrop, 1.0));
    assert_eq!(with, plain);
    for alpha in ALPHAS {
        develop_with(&sensor, &src, &opts(Framing::StandardCrop, alpha));
    }
}

#[test]
fn passing_warps_develop_under_full_active_area() {
    let cases = [
        (
            "Pixel 7 Pro (off-centre)",
            PIXEL_7_PRO_KR,
            PIXEL_7_PRO_CX,
            PIXEL_7_PRO_CY,
        ),
        ("L1021223", L1021223_KR, 0.5, 0.5),
    ];
    for (label, kr, cx, cy) in cases {
        let sensor = warped_sensor(kr, [0.0, 0.0], cx, cy);
        let src = distinct_plane(&sensor);
        for alpha in ALPHAS {
            let (fw, fh, full) = develop_with(&sensor, &src, &opts(Framing::FullActiveArea, alpha));
            assert_eq!((fw, fh), (7, 6), "{label} α {alpha}");
            let (sw, _, standard) =
                develop_with(&sensor, &src, &opts(Framing::StandardCrop, alpha));
            assert_contains_at(
                &format!("{label} α {alpha}"),
                &full,
                fw,
                &standard,
                sw,
                (1, 1),
            );
        }
    }
}

#[test]
fn each_bounds_clause_refuses() {
    // (label, kr, cx, cy): each fails exactly one clause of DEC-059 §3.
    let cases: [(&str, [f64; 4], f64, f64); 6] = [
        ("f < 0", [-0.5, 0.0, 0.0, 0.0], 0.5, 0.5),
        // f(0) = f(1) = 0.9, but f(0.5) = 1.05: a corner-only check misses it.
        ("interior peak", [0.9, 0.6, -0.6, 0.0], 0.5, 0.5),
        ("cx outside", L1021223_KR, 1.5, 0.5),
        ("cy outside", L1021223_KR, 0.5, -0.1),
        ("NaN kr1", [0.999, f64::NAN, 0.0, 0.0], 0.5, 0.5),
        ("infinite kr3", [0.999, 0.0, 0.0, f64::INFINITY], 0.5, 0.5),
    ];
    for (label, kr, cx, cy) in cases {
        let sensor = warped_sensor(kr, [0.0, 0.0], cx, cy);
        let src = distinct_plane(&sensor);
        let err = refused(label, &sensor, &src, &opts(Framing::FullActiveArea, 1.0));
        assert!(
            matches!(
                err,
                Error::UnsupportedWarpedFullActiveArea {
                    active_width: 7,
                    active_height: 6,
                    ..
                }
            ),
            "{label}: got {err:?}"
        );
        assert_eq!(err.kind(), ErrorKind::Unsupported, "{label}");
    }

    // The interior peak is reported as the scale it really reaches.
    let sensor = warped_sensor([0.9, 0.6, -0.6, 0.0], [0.0, 0.0], 0.5, 0.5);
    let src = distinct_plane(&sensor);
    match refused(
        "interior peak",
        &sensor,
        &src,
        &opts(Framing::FullActiveArea, 1.0),
    ) {
        Error::UnsupportedWarpedFullActiveArea {
            scale_min,
            scale_max,
            ..
        } => {
            assert!((scale_max - 1.05).abs() < 1e-12, "scale_max {scale_max}");
            assert!((scale_min - 0.9).abs() < 1e-12, "scale_min {scale_min}");
        }
        other => panic!("interior peak: got {other:?}"),
    }

    // The same polynomials with a centre inside the image and no NaN pass
    // once the failing clause is removed — the clauses are the refusal, not
    // something else about the fixture.
    let sensor = warped_sensor(L1021223_KR, [0.0, 0.0], 0.0, 1.0);
    let src = distinct_plane(&sensor);
    develop_with(&sensor, &src, &opts(Framing::FullActiveArea, 1.0));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — the warp_alpha domain
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn warp_alpha_outside_the_domain_is_a_caller_error() {
    let no_warp = bordered_sensor();
    let warped = warped_sensor(L1021223_KR, [0.0, 0.0], 0.5, 0.5);
    // Step 0 is first: before a malformed OpcodeList1 / OpcodeList3 ...
    let mut malformed = warped.clone();
    malformed.opcode_list_1 = Some(malformed_fix_bad_pixels());
    malformed.opcode_list_3 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        1,
        0x0103_0000,
        0,
        &[0, 0, 0, 1],
    )]));
    // ... and before the required-processing check refuses an unrecognised
    // mandatory opcode.
    let mut unrecognised = no_warp.clone();
    unrecognised.opcode_list_2 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        9,
        0x0103_0000,
        0,
        &[],
    )]));
    unrecognised.opcode_lists = [false, true, false];

    let sensors = [
        ("no warp", &no_warp),
        ("warped", &warped),
        ("malformed opcode lists", &malformed),
        ("unrecognised mandatory opcode", &unrecognised),
    ];
    let src = distinct_plane(&no_warp);

    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let options = opts(framing, bad);
            for (label, sensor) in sensors {
                let label = format!("{label}, {framing:?}, α {bad}");
                let check = |err: Error, call: &str| {
                    assert_eq!(err.kind(), ErrorKind::CallerError, "{label} {call}");
                    match err {
                        Error::InvalidWarpAlpha { warp_alpha } => assert_eq!(
                            warp_alpha.to_bits(),
                            bad.to_bits(),
                            "{label} {call}: carries the value"
                        ),
                        other => panic!("{label} {call}: got {other:?}"),
                    }
                };
                check(
                    output_dimensions_with(sensor, &options).unwrap_err(),
                    "output_dimensions_with",
                );
                // Every dst length: the argument check comes before the
                // buffer checks too.
                for len in [0usize, 12, 42] {
                    let mut dst = vec![SENTINEL; len];
                    check(
                        develop_with_into(sensor, &src, &mut dst, &options).unwrap_err(),
                        "develop_with_into",
                    );
                    assert!(dst.iter().all(|&v| v == SENTINEL), "{label}: dst untouched");
                }
            }
        }
    }

    // The ends of the domain, -0.0 included, are accepted.
    for good in [-0.0, 0.0, 1.0] {
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let options = opts(framing, good);
            for sensor in [&no_warp, &warped] {
                develop_with(sensor, &src, &options);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 — tangential terms at α = 0
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn alpha_zero_skips_tangential_terms() {
    let sensor = warped_sensor(L1021223_KR, [0.01, 0.0], 0.5, 0.5);
    let src = distinct_plane(&sensor);
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        let (_, _, at_zero) = develop_with(&sensor, &src, &opts(framing, 0.0));
        let (_, _, no_warp) = develop_with(&without_warp(&sensor), &src, &opts(framing, 1.0));
        assert_eq!(at_zero, no_warp, "{framing:?}: α = 0 is warp-free");
    }
    // α = 1 under StandardCrop is refused as today; under FullActiveArea the
    // radial check passes and the applier's refusal is reached.
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        let err = refused(
            &format!("{framing:?} kt"),
            &sensor,
            &src,
            &opts(framing, 1.0),
        );
        assert!(
            matches!(err, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if kt0 == 0.01 && kt1 == 0.0),
            "{framing:?}: got {err:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — error surface
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn refusal_displays_name_the_values() {
    let err = Error::InvalidWarpAlpha { warp_alpha: 1.5 };
    let text = err.to_string();
    assert!(text.contains("1.5"), "{text}");
    assert!(text.contains("warp_alpha"), "{text}");
    assert!(!text.contains("SPEC-"), "{text}");
    assert_eq!(err.kind(), ErrorKind::CallerError);

    let err = Error::UnsupportedWarpedFullActiveArea {
        active_width: 8392,
        active_height: 5632,
        warp_alpha: 0.75,
        scale_min: 0.25,
        scale_max: 1.125,
    };
    let text = err.to_string();
    for needle in ["8392", "5632", "warp_alpha", "0.75", "0.25", "1.125"] {
        assert!(text.contains(needle), "missing {needle}: {text}");
    }
    assert!(!text.contains("SPEC-"), "{text}");
    assert_eq!(err.kind(), ErrorKind::Unsupported);
}
