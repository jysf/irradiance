//! `SPEC-066` — per-position black levels on the mono path (`DEC-061`).
//!
//! [`DevelopOptions::black_levels`] chooses where each photosite's black comes
//! from: [`BlackLevels::FirstValue`] (the default, `develop_into`'s output) or
//! [`BlackLevels::PerPosition`] (`BlackLevel` indexed by `BlackLevelRepeatDim`,
//! phased from the `ActiveArea` origin, taken on raw samples before the warp).
//!
//! **No expected value here comes from calling the crate** (`AC2`–`AC5`). Byte
//! identity between the two modes runs both through the same core, so it
//! cannot see a bug in the pattern arm (`shared-path-oracle-tautology`). The
//! oracle is [`normalize`] below — the DNG affine map with `DEC-018`'s clamp and
//! round-half-up, written out again in the test — composed with a crop and
//! orientation also written out here from the TIFF/Exif definitions.
//!
//! Every fixture has an **odd** `ActiveArea` origin and a `DefaultCrop` origin
//! that puts the crop's absolute phase on a third parity, because every held
//! file's `ActiveArea` origin is even: on real data the three plausible origins
//! (plane, `ActiveArea`, `DefaultCrop`) are indistinguishable.
//!
//! Tier B (`AC8`) reads the held corpus when `IRRADIANCE_CORPUS_DIR` points at
//! it and skips by name otherwise.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/opcode.rs"]
mod opcode_support;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions_with,
    scratch_len, BlackLevels, DevelopOptions, Framing,
};
use irradiance::ifd::{ActiveArea, Cfa, Container, DefaultCropOrigin, DefaultCropSize, Sensor};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind, DEVELOP_VERSION};

const SENTINEL: u16 = 0xABCD;

/// The Q2M's measured `WarpRectilinear` radial coefficients: a real,
/// non-identity, in-bounds warp at the image centre.
const Q2M_KR: [f64; 4] = [
    0.999_251_106,
    -0.061_376_512_877_263_58,
    -0.093_915_541_393_360_16,
    0.055_882_009_215_291_75,
];

// ─────────────────────────────────────────────────────────────────────────────
// The oracle, written out independently of the crate
// ─────────────────────────────────────────────────────────────────────────────

/// DNG's `[black, white] -> [0, 65535]` map: clamp (`DEC-018`), scale, round
/// half up in integers.
fn normalize(sample: u32, black: u32, white: u32) -> u16 {
    let clamped = sample.clamp(black, white);
    let numerator = u64::from(clamped - black) * 65535;
    let denominator = u64::from(white - black);
    u16::try_from((numerator + denominator / 2) / denominator).unwrap()
}

/// The black DNG assigns raw photosite `(x, y)`: `BlackLevel` indexed by
/// `BlackLevelRepeatDim [rows, cols]`, the pattern's origin at the `ActiveArea`
/// top-left `(left, top)`. The phase is extended (Euclidean remainder) over
/// the padding outside `ActiveArea`, which fixtures fill but no pass reads.
fn pattern_black(blacks: &[u32], rows: u32, cols: u32, left: u32, top: u32, x: u32, y: u32) -> u32 {
    let row = (i64::from(y) - i64::from(top)).rem_euclid(i64::from(rows));
    let col = (i64::from(x) - i64::from(left)).rem_euclid(i64::from(cols));
    blacks[(row * i64::from(cols) + col) as usize]
}

/// The geometry a fixture is built from, kept beside the `Sensor` so the
/// expected image is computed from these numbers, not from the crate.
#[derive(Clone, Copy)]
struct Shape {
    width: u32,
    height: u32,
    /// `ActiveArea` (left, top, right, bottom).
    active: (u32, u32, u32, u32),
    /// `DefaultCrop` origin within `ActiveArea`, and size.
    crop: (u32, u32, u32, u32),
}

impl Shape {
    fn active_size(&self) -> (u32, u32) {
        (self.active.2 - self.active.0, self.active.3 - self.active.1)
    }
}

/// The expected developed image: `value(raw_x, raw_y)` for every output pixel,
/// framed and oriented by hand. Orientation 6 is "rotate 90° clockwise": the
/// output is `crop_height` wide, and output `(ox, oy)` shows crop pixel
/// `(oy, crop_height − 1 − ox)`.
fn expected_image(
    shape: &Shape,
    framing: Framing,
    orientation: u32,
    value: impl Fn(u32, u32) -> u16,
) -> (u32, u32, Vec<u16>) {
    let (active_w, active_h) = shape.active_size();
    let (origin_x, origin_y, crop_w, crop_h) = match framing {
        Framing::StandardCrop => shape.crop,
        _ => (0, 0, active_w, active_h),
    };
    let (out_w, out_h) = match orientation {
        1 => (crop_w, crop_h),
        6 => (crop_h, crop_w),
        other => panic!("orientation {other} is not exercised here"),
    };
    let mut out = Vec::with_capacity((out_w * out_h) as usize);
    for oy in 0..out_h {
        for ox in 0..out_w {
            let (cx, cy) = match orientation {
                1 => (ox, oy),
                _ => (oy, crop_h - 1 - ox),
            };
            let raw_x = shape.active.0 + origin_x + cx;
            let raw_y = shape.active.1 + origin_y + cy;
            out.push(value(raw_x, raw_y));
        }
    }
    (out_w, out_h, out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A 12×11 plane, `ActiveArea` 8×9 at (3, 1) — odd on both axes — and an
/// odd-x `DefaultCrop` origin (1, 2), size 5×6. The three candidate pattern
/// origins then sit on three different parities: plane (0, 0), `ActiveArea`
/// (1, 1), `DefaultCrop` (4, 3) → (0, 1).
const SHAPE: Shape = Shape {
    width: 12,
    height: 11,
    active: (3, 1, 11, 10),
    crop: (1, 2, 5, 6),
};

fn sensor_with(shape: &Shape, cfa: bool, orientation: u32) -> Sensor {
    let mut sensor = Sensor::test_fixture();
    sensor.width = shape.width;
    sensor.height = shape.height;
    sensor.bits_per_sample = 16;
    sensor.active_area = Some(ActiveArea {
        left: shape.active.0,
        top: shape.active.1,
        right: shape.active.2,
        bottom: shape.active.3,
    });
    sensor.default_crop_origin = Some(DefaultCropOrigin {
        x: shape.crop.0,
        y: shape.crop.1,
    });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: shape.crop.2,
        height: shape.crop.3,
    });
    sensor.orientation = Some(orientation);
    if cfa {
        sensor.photometric = 32803;
        sensor.color_filter = Some(Cfa::test_fixture());
    }
    sensor
}

/// Set `BlackLevel` / `BlackLevelRepeatDim` / `WhiteLevel` the way the reader
/// fills them: the scalar is the first value.
fn set_levels(sensor: &mut Sensor, blacks: &[u32], repeat: Option<[u32; 2]>, white: u32) {
    sensor.black_level = blacks.first().copied();
    sensor.black_levels = blacks.to_vec();
    sensor.black_level_repeat_dim = repeat;
    sensor.white_level = Some(white);
    sensor.white_levels = vec![white];
}

/// The AC3 fixture: four distinct blacks on a `[2, 2]` pattern, white 4095.
const AC3_BLACKS: [u32; 4] = [100, 200, 300, 400];
const AC3_WHITE: u32 = 4095;

fn ac3_sensor(orientation: u32) -> Sensor {
    let mut sensor = sensor_with(&SHAPE, true, orientation);
    set_levels(&mut sensor, &AC3_BLACKS, Some([2, 2]), AC3_WHITE);
    sensor
}

/// A hand-built ramp, distinct at every photosite, inside `[0, 4095]`.
fn ramp(shape: &Shape) -> Vec<u16> {
    let mut plane = Vec::new();
    for y in 0..shape.height {
        for x in 0..shape.width {
            plane.push(u16::try_from(500 + 37 * x + 11 * y).unwrap());
        }
    }
    plane
}

/// `black(x, y) + k` at every photosite of the plane, the padding included.
fn black_plus_k(shape: &Shape, black: impl Fn(u32, u32) -> u32, k: u32) -> Vec<u16> {
    let mut plane = Vec::new();
    for y in 0..shape.height {
        for x in 0..shape.width {
            plane.push(u16::try_from(black(x, y) + k).unwrap());
        }
    }
    plane
}

fn options(framing: Framing, apply_orientation: bool, black_levels: BlackLevels) -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = framing;
    options.apply_orientation = apply_orientation;
    options.black_levels = black_levels;
    options
}

fn per_position() -> DevelopOptions {
    options(Framing::StandardCrop, true, BlackLevels::PerPosition)
}

/// Develop through `develop_with_into` AND `develop_with_scratch_into`, assert
/// they agree, and return `(width, height, pixels)`.
fn develop(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> (u32, u32, Vec<u16>) {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions");
    let mut dst = vec![SENTINEL; (w * h) as usize];
    develop_with_into(sensor, src, &mut dst, options).expect("develop_with_into");
    let mut scratch = vec![0u16; scratch_len(sensor, options).expect("scratch_len")];
    let mut via_scratch = vec![SENTINEL; dst.len()];
    develop_with_scratch_into(sensor, src, &mut via_scratch, &mut scratch, options)
        .expect("develop_with_scratch_into");
    assert_eq!(dst, via_scratch, "the two entry points disagree");
    (w, h, dst)
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — the default is unchanged
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn default_is_first_value_and_pins_hold() {
    assert_eq!(
        DevelopOptions::default().black_levels,
        BlackLevels::FirstValue
    );
    assert_eq!(BlackLevels::default(), BlackLevels::FirstValue);
    assert_eq!(DEVELOP_VERSION, "4");

    // On a tier-A plane with a real pattern, `develop_into` is the
    // first-value develop: every photosite takes `BlackLevel[0]`.
    let sensor = ac3_sensor(1);
    let src = ramp(&SHAPE);
    let (w, h) = output_dimensions_with(&sensor, &DevelopOptions::default()).unwrap();
    let mut dst = vec![SENTINEL; (w * h) as usize];
    develop_into(&sensor, &src, &mut dst).unwrap();
    let (_, _, want) = expected_image(&SHAPE, Framing::StandardCrop, 1, |x, y| {
        normalize(
            u32::from(src[(y * SHAPE.width + x) as usize]),
            100,
            AC3_WHITE,
        )
    });
    assert_eq!(dst, want, "develop_into takes the first value everywhere");

    // The SPEC-063 MD5 pins of all three entry points are asserted in
    // tests/develop_options.rs and tests/develop_scratch.rs, unedited.
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — the uniform-plane oracle (request 6)
// ─────────────────────────────────────────────────────────────────────────────

/// The Pixel 3a/4a values.
const PIXEL_BLACKS: [u32; 4] = [1023, 1023, 1022, 1022];
const PIXEL_WHITE: u32 = 16368;

#[test]
fn uniform_cfa_plane_develops_uniform_under_per_position() {
    let (left, top) = (SHAPE.active.0, SHAPE.active.1);
    let black = |x, y| pattern_black(&PIXEL_BLACKS, 2, 2, left, top, x, y);
    for k in [0u32, 200, 15345] {
        let src = black_plus_k(&SHAPE, black, k);
        for orientation in [1, 6] {
            let mut cfa = sensor_with(&SHAPE, true, orientation);
            set_levels(&mut cfa, &PIXEL_BLACKS, Some([2, 2]), PIXEL_WHITE);
            for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
                for apply_orientation in [true, false] {
                    let label = format!(
                        "k {k} orientation {orientation} {framing:?} oriented {apply_orientation}"
                    );
                    let shown = if apply_orientation { orientation } else { 1 };
                    let opts = options(framing, apply_orientation, BlackLevels::PerPosition);
                    let (w, h, got) = develop(&cfa, &src, &opts);

                    // The analytic value: each photosite maps `k` against its
                    // own range `[0, 16368 − black]`.
                    let (ew, eh, want) = expected_image(&SHAPE, framing, shown, |x, y| {
                        normalize(k + black(x, y), black(x, y), PIXEL_WHITE)
                    });
                    assert_eq!((w, h), (ew, eh), "{label}: size");
                    assert_eq!(got, want, "{label}: analytic");

                    // For k = 0 and 200 the two ranges (15345 and 15346) round
                    // to the same code value, so the plane is uniform and
                    // equals a LinearRaw develop of `1023 + k`. For k = 15345
                    // they do not (65535 against 65531): the ranges genuinely
                    // differ by one, so only the analytic value is asserted.
                    if k != 15345 {
                        let constant = normalize(k, 0, PIXEL_WHITE - 1023);
                        assert!(got.iter().all(|&v| v == constant), "{label}: uniform");
                        let mut mono = sensor_with(&SHAPE, false, orientation);
                        set_levels(&mut mono, &[1023], Some([1, 1]), PIXEL_WHITE);
                        let mono_src = black_plus_k(&SHAPE, |_, _| 1023, k);
                        let mono_opts =
                            options(framing, apply_orientation, BlackLevels::FirstValue);
                        let (_, _, mono_out) = develop(&mono, &mono_src, &mono_opts);
                        assert_eq!(got, mono_out, "{label}: equals the LinearRaw develop");
                    }

                    // FirstValue on the same plane is NOT uniform for k > 0:
                    // the fixture tells the two modes apart. (At k = 15345
                    // both modes happen to round positions 2 and 3 to 65531,
                    // so only k = 200 separates them sample by sample.)
                    if k > 0 {
                        let first = options(framing, apply_orientation, BlackLevels::FirstValue);
                        let (_, _, first_out) = develop(&cfa, &src, &first);
                        if k == 200 {
                            assert_ne!(first_out, got, "{label}: FirstValue differs");
                        }
                        assert!(
                            first_out.iter().any(|&v| v != first_out[0]),
                            "{label}: FirstValue is not uniform"
                        );
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — per-position honoured, phased from the ActiveArea origin
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn per_position_matches_hand_computed_block() {
    let sensor = ac3_sensor(1);
    let src = ramp(&SHAPE);
    let (w, h, got) = develop(&sensor, &src, &per_position());
    assert_eq!((w, h), (5, 6));

    // Hand-computed (tests/black_levels.rs's normalize, run outside the
    // crate): raw = 500 + 37x + 11y, black = [100, 200, 300, 400] at
    // ((y − 1) mod 2, (x − 3) mod 2). The other candidates give, at (0, 0):
    // plane origin 6579, DefaultCrop origin 9531, transposed index 6579,
    // entries 0 and 1 swapped 9531, FirstValue 9531 — all different.
    let want: [[u16; 4]; 4] = [
        [8093, 10138, 9338, 11352],
        [5179, 7408, 6491, 8686],
        [8463, 10499, 9708, 11713],
        [5569, 7788, 6882, 9066],
    ];
    for (oy, row) in want.iter().enumerate() {
        for (ox, &value) in row.iter().enumerate() {
            assert_eq!(got[oy * w as usize + ox], value, "output ({ox}, {oy})");
        }
    }

    // And the whole image against the formula, under every framing and
    // orientation.
    for orientation in [1, 6] {
        let sensor = ac3_sensor(orientation);
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            for apply_orientation in [true, false] {
                let shown = if apply_orientation { orientation } else { 1 };
                let opts = options(framing, apply_orientation, BlackLevels::PerPosition);
                let (_, _, got) = develop(&sensor, &src, &opts);
                let (_, _, want) = expected_image(&SHAPE, framing, shown, |x, y| {
                    let black = pattern_black(&AC3_BLACKS, 2, 2, 3, 1, x, y);
                    normalize(
                        u32::from(src[(y * SHAPE.width + x) as usize]),
                        black,
                        AC3_WHITE,
                    )
                });
                assert_eq!(
                    got, want,
                    "orientation {orientation} {framing:?} oriented {apply_orientation}"
                );
            }
        }
    }

    // White is per sample, not per position: a second WhiteLevel value is
    // never read (DEC-061 §3).
    let mut two_whites = ac3_sensor(1);
    two_whites.white_levels = vec![AC3_WHITE, 1000];
    let (_, _, with_two) = develop(&two_whites, &src, &per_position());
    let (_, _, with_one) = develop(&ac3_sensor(1), &src, &per_position());
    assert_eq!(with_two, with_one, "only the first WhiteLevel is used");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — non-square and 3×3 patterns
// ─────────────────────────────────────────────────────────────────────────────

/// (label, blacks, repeat dim, the hand-computed top-left 4×4).
type Ac4Block = (&'static str, Vec<u32>, [u32; 2], [[u16; 4]; 4]);

#[test]
fn non_square_and_3x3_patterns() {
    // Active 8×9 at (3, 1): neither side a multiple of 3.
    let src = ramp(&SHAPE);
    let cases: [(&str, Vec<u32>, [u32; 2]); 3] = [
        ("[1, 2]", vec![100, 300], [1, 2]),
        ("[2, 1]", vec![100, 300], [2, 1]),
        ("[3, 3]", (1..=9).map(|i| 40 * i).collect(), [3, 3]),
    ];
    for (label, blacks, [rows, cols]) in cases {
        let mut sensor = sensor_with(&SHAPE, true, 1);
        set_levels(&mut sensor, &blacks, Some([rows, cols]), AC3_WHITE);
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let opts = options(framing, true, BlackLevels::PerPosition);
            let (_, _, got) = develop(&sensor, &src, &opts);
            let (_, _, want) = expected_image(&SHAPE, framing, 1, |x, y| {
                let black = pattern_black(&blacks, rows, cols, 3, 1, x, y);
                normalize(
                    u32::from(src[(y * SHAPE.width + x) as usize]),
                    black,
                    AC3_WHITE,
                )
            });
            assert_eq!(got, want, "{label} {framing:?}");
        }
    }

    // The hand-computed StandardCrop top-left 4×4 of each, as literals
    // (this file's normalize, run outside the crate). The crop starts at raw
    // (4, 3), active-relative (1, 2). A fixed 2×2 slot gives a different
    // block for every one of the three.
    let blocks: [Ac4Block; 3] = [
        (
            "[1, 2]: by column only",
            vec![100, 300],
            [1, 2],
            [
                [6579, 10138, 7857, 11352],
                [6769, 10318, 8047, 11532],
                [6959, 10499, 8237, 11713],
                [7149, 10679, 8427, 11893],
            ],
        ),
        (
            "[2, 1]: by row only",
            vec![100, 300],
            [2, 1],
            [
                [9531, 10138, 10745, 11352],
                [6769, 7408, 8047, 8686],
                [9892, 10499, 11106, 11713],
                [7149, 7788, 8427, 9066],
            ],
        ),
        (
            "[3, 3]",
            (1..=9).map(|i| 40 * i).collect(),
            [3, 3],
            [
                [6267, 6282, 8160, 8194],
                [9989, 10040, 11733, 11801],
                [8463, 8500, 10276, 10331],
                [6840, 6861, 8727, 8767],
            ],
        ),
    ];
    for (label, blacks, [rows, cols], want) in blocks {
        let mut sensor = sensor_with(&SHAPE, true, 1);
        set_levels(&mut sensor, &blacks, Some([rows, cols]), AC3_WHITE);
        let (w, _, got) = develop(&sensor, &src, &per_position());
        for (oy, row) in want.iter().enumerate() {
            for (ox, &value) in row.iter().enumerate() {
                assert_eq!(
                    got[oy * w as usize + ox],
                    value,
                    "{label}: output ({ox}, {oy})"
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — the black is applied before the warp, by raw position
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn black_is_applied_before_the_warp() {
    let warp = opcode_support::single_warp_rectilinear_list(Q2M_KR, [0.0, 0.0], 0.5, 0.5, 1, 0);
    // (blacks, white, k, the uniform value): the AC3 pattern at k = 0, and a
    // narrow-spread pattern at k = 10 whose four ranges round alike.
    let cases: [([u32; 4], u32, u32); 2] = [
        (AC3_BLACKS, AC3_WHITE, 0),
        ([1000, 1001, 1002, 1003], 65535, 10),
    ];
    for (blacks, white, k) in cases {
        let expected = normalize(k, 0, white - blacks[0]);
        for b in blacks {
            assert_eq!(
                normalize(k + b, b, white),
                expected,
                "fixture: ranges round alike"
            );
        }
        let mut sensor = sensor_with(&SHAPE, true, 1);
        set_levels(&mut sensor, &blacks, Some([2, 2]), white);
        sensor.opcode_list_3 = Some(warp.clone());
        sensor.opcode_lists = [false, false, true];
        let src = black_plus_k(&SHAPE, |x, y| pattern_black(&blacks, 2, 2, 3, 1, x, y), k);
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            for alpha in [1.0, 0.5] {
                let mut opts = options(framing, true, BlackLevels::PerPosition);
                opts.warp_alpha = alpha;
                let (_, _, got) = develop(&sensor, &src, &opts);
                assert!(
                    got.iter().all(|&v| v == expected),
                    "blacks {blacks:?} k {k} {framing:?} α {alpha}: not uniform {expected}: {got:?}"
                );
                // The warp really ran: FirstValue on the same plane is not
                // uniform for k > 0 and differs from PerPosition.
                if k > 0 {
                    let mut first = opts;
                    first.black_levels = BlackLevels::FirstValue;
                    let (_, _, first_out) = develop(&sensor, &src, &first);
                    assert_ne!(first_out, got);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — refusals, only when the pattern is used
// ─────────────────────────────────────────────────────────────────────────────

/// Develop through both entry points into sentinel buffers, assert both give
/// the same error and leave `dst` untouched, and return the error.
fn refused(sensor: &Sensor, src: &[u16], options: &DevelopOptions, label: &str) -> Error {
    let (w, h) = output_dimensions_with(sensor, options).unwrap();
    let mut dst = vec![SENTINEL; (w * h) as usize];
    let err = develop_with_into(sensor, src, &mut dst, options)
        .expect_err(&format!("{label}: develop_with_into must refuse"));
    assert!(dst.iter().all(|&v| v == SENTINEL), "{label}: dst written");
    let mut scratch = vec![SENTINEL; scratch_len(sensor, options).unwrap()];
    let err2 = develop_with_scratch_into(sensor, src, &mut dst, &mut scratch, options)
        .expect_err(&format!("{label}: develop_with_scratch_into must refuse"));
    assert!(
        dst.iter().all(|&v| v == SENTINEL),
        "{label}: dst written (scratch)"
    );
    assert!(
        scratch.iter().all(|&v| v == SENTINEL),
        "{label}: scratch written"
    );
    assert_eq!(
        format!("{err:?}"),
        format!("{err2:?}"),
        "{label}: entries disagree"
    );
    err
}

/// (label, blacks, repeat dim, the expected (rows, cols, count)).
type PatternCase = (&'static str, Vec<u32>, Option<[u32; 2]>, (u32, u32, usize));

#[test]
fn bad_patterns_are_refused_only_when_used() {
    let src = ramp(&SHAPE);
    let pattern_cases: [PatternCase; 5] = [
        (
            "4 values, [1, 1]",
            vec![100, 200, 300, 400],
            Some([1, 1]),
            (1, 1, 4),
        ),
        (
            "3 values, [2, 2]",
            vec![100, 200, 300],
            Some([2, 2]),
            (2, 2, 3),
        ),
        ("[0, 2]", vec![100, 200], Some([0, 2]), (0, 2, 2)),
        (
            "overflow",
            vec![100, 200],
            Some([u32::MAX, u32::MAX]),
            (u32::MAX, u32::MAX, 2),
        ),
        (
            "absent dim, 4 values",
            vec![100, 200, 300, 400],
            None,
            (1, 1, 4),
        ),
    ];
    for (label, blacks, repeat, (rows, cols, count)) in pattern_cases {
        let mut sensor = sensor_with(&SHAPE, true, 1);
        set_levels(&mut sensor, &blacks, repeat, AC3_WHITE);
        let err = refused(&sensor, &src, &per_position(), label);
        match err {
            Error::InvalidBlackLevelPattern {
                repeat_rows,
                repeat_cols,
                count: c,
            } => assert_eq!(
                (repeat_rows, repeat_cols, c),
                (rows, cols, count),
                "{label}"
            ),
            other => panic!("{label}: expected InvalidBlackLevelPattern, got {other:?}"),
        }
        assert_eq!(err_kind(&sensor, &src), ErrorKind::Corrupt, "{label}: kind");
        // The same sensor under FirstValue develops: validated only when used.
        let first = options(Framing::StandardCrop, true, BlackLevels::FirstValue);
        let _ = develop(&sensor, &src, &first);
    }

    // A value ≥ white: today's InvalidLevels, carrying the first offender.
    let mut over = sensor_with(&SHAPE, true, 1);
    set_levels(&mut over, &[100, 5000, 4095, 300], Some([2, 2]), AC3_WHITE);
    match refused(&over, &src, &per_position(), "value ≥ white") {
        Error::InvalidLevels {
            black_level,
            white_level,
        } => assert_eq!((black_level, white_level), (5000, AC3_WHITE)),
        other => panic!("value ≥ white: got {other:?}"),
    }
    let _ = develop(
        &over,
        &src,
        &options(Framing::StandardCrop, true, BlackLevels::FirstValue),
    );

    // BlackLevel absent or unreadable (the Pixel 7 Pro shape): both modes use
    // black 0 and agree.
    let mut absent = sensor_with(&SHAPE, true, 1);
    set_levels(&mut absent, &[], Some([2, 2]), AC3_WHITE);
    assert_eq!(absent.black_level, None);
    let first = options(Framing::StandardCrop, true, BlackLevels::FirstValue);
    assert_eq!(
        develop(&absent, &src, &per_position()),
        develop(&absent, &src, &first)
    );

    // BlackLevelRepeatDim absent (or malformed, as on the K-3 III) with one
    // value: the DNG default [1, 1] meets one value.
    let mut one = sensor_with(&SHAPE, false, 1);
    set_levels(&mut one, &[220], None, AC3_WHITE);
    assert_eq!(
        develop(&one, &src, &per_position()),
        develop(&one, &src, &first)
    );
}

fn err_kind(sensor: &Sensor, src: &[u16]) -> ErrorKind {
    let (w, h) = output_dimensions_with(sensor, &per_position()).unwrap();
    let mut dst = vec![0u16; (w * h) as usize];
    develop_with_into(sensor, src, &mut dst, &per_position())
        .unwrap_err()
        .kind()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — error order
// ─────────────────────────────────────────────────────────────────────────────

fn bad_pattern_sensor() -> Sensor {
    let mut sensor = sensor_with(&SHAPE, true, 1);
    set_levels(&mut sensor, &[100, 200, 300], Some([2, 2]), AC3_WHITE);
    sensor
}

fn develop_err(sensor: &Sensor, src: &[u16], dst_len: usize, options: &DevelopOptions) -> Error {
    let mut dst = vec![SENTINEL; dst_len];
    let err = develop_with_into(sensor, src, &mut dst, options).unwrap_err();
    let mut scratch = vec![0u16; 4096];
    let err2 = develop_with_scratch_into(sensor, src, &mut dst, &mut scratch, options).unwrap_err();
    assert_eq!(format!("{err:?}"), format!("{err2:?}"));
    assert!(dst.iter().all(|&v| v == SENTINEL));
    err
}

#[test]
fn pattern_error_order() {
    let src = ramp(&SHAPE);
    let good_dst = 5 * 6;

    let mut alpha = per_position();
    alpha.warp_alpha = 1.5;
    assert!(matches!(
        develop_err(&bad_pattern_sensor(), &src, good_dst, &alpha),
        Error::InvalidWarpAlpha { .. }
    ));

    let mut mandatory = bad_pattern_sensor();
    mandatory.opcode_list_1 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        0xDEAD_BEEF,
        0x0104_0000,
        0,
        &[1, 2, 3, 4],
    )]));
    mandatory.opcode_lists = [true, false, false];
    assert!(matches!(
        develop_err(&mandatory, &src, good_dst, &per_position()),
        Error::UnsupportedMandatoryOpcode { .. }
    ));

    assert!(matches!(
        develop_err(&bad_pattern_sensor(), &src[1..], good_dst, &per_position()),
        Error::SourcePlaneWrongLength { .. }
    ));

    let mut bad_area = bad_pattern_sensor();
    bad_area.active_area = Some(ActiveArea {
        left: 3,
        top: 1,
        right: 13,
        bottom: 10,
    });
    assert!(matches!(
        develop_err(&bad_area, &src, good_dst, &per_position()),
        Error::InvalidActiveArea { .. }
    ));

    // The pattern is checked where the levels always were: before dst.
    assert!(matches!(
        develop_err(&bad_pattern_sensor(), &src, good_dst + 1, &per_position()),
        Error::InvalidBlackLevelPattern { .. }
    ));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — the error's Display
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn pattern_error_display_names_all_three() {
    let message = Error::InvalidBlackLevelPattern {
        repeat_rows: 7,
        repeat_cols: 13,
        count: 29,
    }
    .to_string();
    for number in ["7", "13", "29"] {
        assert!(message.contains(number), "{message:?} lacks {number}");
    }
    assert!(message.contains("BlackLevel"), "{message:?}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 — tier B
// ─────────────────────────────────────────────────────────────────────────────

fn corpus_plane(path: &str) -> Option<(Sensor, Vec<u16>)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(&root)?;
    let bytes = std::fs::read(&file_path).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    Some((sensor, plane))
}

fn md5_le(samples: &[u16]) -> String {
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    md5::to_hex(&md5::hash(&bytes))
}

#[test]
fn canon_per_position_differs_only_at_position_3() {
    let path = "CANON-EOS-5D-MARK-III/5G4A9394.DNG";
    let Some((sensor, src)) = corpus_plane(path) else {
        eprintln!("SKIP {path}: not present");
        return;
    };
    assert_eq!(sensor.black_levels, vec![2047, 2047, 2047, 2048]);
    assert_eq!(sensor.black_level_repeat_dim, Some([2, 2]));
    let mut first = DevelopOptions::default();
    first.framing = Framing::FullActiveArea;
    first.apply_orientation = false;
    let mut per = first;
    per.black_levels = BlackLevels::PerPosition;

    let (w, h) = output_dimensions_with(&sensor, &first).unwrap();
    assert_eq!((w, h), (5796, 3870));
    let mut a = vec![0u16; (w * h) as usize];
    let mut b = vec![0u16; a.len()];
    develop_with_into(&sensor, &src, &mut a, &first).unwrap();
    develop_with_into(&sensor, &src, &mut b, &per).unwrap();

    let (mut differing, mut largest) = (0usize, 0u16);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let position = (y % 2) * 2 + x % 2;
            if position == 3 {
                assert!(b[i] <= a[i], "({x}, {y}): PerPosition above FirstValue");
                if a[i] != b[i] {
                    differing += 1;
                    largest = largest.max(a[i] - b[i]);
                }
            } else {
                assert_eq!(a[i], b[i], "({x}, {y}) position {position}");
            }
        }
    }
    assert_eq!(differing, 5_602_332, "{path}: differing samples");
    assert_eq!(largest, 6, "{path}: largest difference");
}

#[test]
fn one_by_one_files_are_identical_under_both_modes() {
    // SPEC-063's pins (tests/develop_options.rs): the FirstValue output.
    let pinned: [(&str, &str); 7] = [
        (
            "LEICA-Q2-MONO/L1021223.DNG",
            "feb4910588d189bd5bf89620b480fe6c",
        ),
        (
            "LEICA-Q2-MONO/L1026016.DNG",
            "2bc53a6337b582a8080ac1206e62434b",
        ),
        (
            "LEICA-Q2-MONO/L1026192.DNG",
            "f9a9c56c6255436ac7e764cb0a7c21fb",
        ),
        (
            "LEICA-M-MONOCHROM/L1000622.DNG",
            "fddf0ea0dec6a66bf1fd1cb8c787cc79",
        ),
        (
            "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
            "4aa013f64317412111c9d298300dcb28",
        ),
        (
            "PENTAX-K3III-MONO/K3III.DNG",
            "82d2d51957922587ec55ee594b2a6b14",
        ),
        (
            "NIKON-D300S/_DSC4615.DNG",
            "09faddddd24a11497cc072bcf3a42ce9",
        ),
    ];
    let mut options = DevelopOptions::default();
    options.black_levels = BlackLevels::PerPosition;
    for (path, md5) in pinned {
        let Some((sensor, src)) = corpus_plane(path) else {
            eprintln!("SKIP {path}: not present");
            continue;
        };
        let (w, h) = output_dimensions_with(&sensor, &options).unwrap();
        let mut dst = vec![0u16; (w * h) as usize];
        develop_with_into(&sensor, &src, &mut dst, &options)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(md5_le(&dst), md5, "{path}: PerPosition MD5");
    }
}
