//! `SPEC-065` — caller-owned develop scratch: `scratch_len` and
//! `develop_with_scratch_into` (`DEC-060`).
//!
//! The load-bearing oracle is a **relation**: `develop_with_scratch_into`,
//! given exactly `scratch_len` samples of any content, produces the same `dst`
//! and the same error as `develop_with_into` (`AC2`, `AC6`, `AC7`). On the held
//! files the `SPEC-063` MD5 pins, taken from main before any STAGE-019 spec
//! changed anything, must hold through the scratch path too.
//!
//! The allocation contract (`AC3`, `AC4`, `AC8`, `AC9`) lives in its own binary,
//! `tests/develop_scratch_alloc.rs`, because its counting allocator sees the
//! whole process.
//!
//! Two lanes, as `tests/develop_options.rs` establishes: tier A builds a
//! `Sensor` from `Sensor::test_fixture()` (needs the `test-fixtures` feature,
//! which `just test`'s `--all-features` turns on); tier B reads the held files
//! under `$IRRADIANCE_CORPUS_DIR` in place (`DEC-003`) and skips per entry, by
//! name, when one is absent.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/opcode.rs"]
mod opcode_support;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{
    develop_with_into, develop_with_scratch_into, output_dimensions_with, scratch_len,
    DevelopOptions, Framing,
};
use irradiance::ifd::{
    ActiveArea, ByteOrder, Container, DefaultCropOrigin, DefaultCropSize, Sensor,
    TAG_OPCODE_LIST_1,
};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

// ─────────────────────────────────────────────────────────────────────────────
// Tier-A fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// L1021223's `kr`, centred: a real warp that passes `DEC-059` §3's bounds
/// rule at every α, so it develops under both framings.
const Q2M_KR: [f64; 4] = [
    0.999_251_106,
    -0.061_376_512_877_263_58,
    -0.093_915_541_393_360_16,
    0.055_882_009_215_291_75,
];

/// The Pixel 3a's over-unity lens (`tests/warp_alpha.rs`): refused under
/// `FullActiveArea` at every α > 0.
const PIXEL_3A_KR: [f64; 4] = [
    0.980_158_686_637_878_4,
    0.091_988_243_162_631_99,
    -0.177_878_528_833_389_28,
    0.107_260_853_052_139_28,
];

/// The raw plane is 10×8 = 80 samples; the active area is 7×6 = 42.
const P: usize = 80;
const A: usize = 42;

const SENTINEL: u16 = 0xABCD;

/// A 10×8 plane with a 7×6 `ActiveArea` at `(left 2, top 1)` and a non-square
/// 4×3 `DefaultCrop` at `(1, 1)` inside it (`tests/develop_options.rs`'s
/// fixture), so P ≠ A. Identity levels.
fn bordered_sensor(orientation: u32) -> Sensor {
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
    sensor.orientation = Some(orientation);
    sensor
}

/// The four cases of `DEC-060` §3: `FixBadPixelsConstant` (`fbp`) and a
/// non-identity `WarpRectilinear` (`warp`), each present or not.
#[derive(Debug, Clone, Copy)]
enum Case {
    Neither,
    FbpOnly,
    WarpOnly,
    Both,
}

const CASES: [Case; 4] = [Case::Neither, Case::FbpOnly, Case::WarpOnly, Case::Both];

fn case_sensor(case: Case, orientation: u32) -> Sensor {
    let mut sensor = bordered_sensor(orientation);
    if matches!(case, Case::FbpOnly | Case::Both) {
        sensor.opcode_list_1 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    }
    if matches!(case, Case::WarpOnly | Case::Both) {
        sensor.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
            Q2M_KR,
            [0.0, 0.0],
            0.5,
            0.5,
            1,
            0,
        ));
    }
    sensor
}

/// Distinct non-zero samples, except every fifth is 0: the
/// `FixBadPixelsConstant` marker (`constant 0`), so the fix has work to do.
fn plane(sensor: &Sensor) -> Vec<u16> {
    let len = (sensor.width * sensor.height) as usize;
    (0..len)
        .map(|i| if i % 5 == 0 { 0 } else { 100 + 7 * i as u16 })
        .collect()
}

fn opts(framing: Framing, warp_alpha: f64, apply_orientation: bool) -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = framing;
    options.warp_alpha = warp_alpha;
    options.apply_orientation = apply_orientation;
    options
}

/// Every options value `AC2` names: both framings, α 0 / 0.5 / 1, orientation
/// applied or not.
fn ac2_options() -> Vec<DevelopOptions> {
    let mut out = Vec::new();
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        for alpha in [0.0, 0.5, 1.0] {
            for apply_orientation in [true, false] {
                out.push(opts(framing, alpha, apply_orientation));
            }
        }
    }
    out
}

fn dst_len(sensor: &Sensor, options: &DevelopOptions) -> usize {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions resolve");
    (w as usize) * (h as usize)
}

/// `develop_with_into` into a fresh buffer.
fn reference(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> Vec<u16> {
    let mut dst = vec![0u16; dst_len(sensor, options)];
    develop_with_into(sensor, src, &mut dst, options)
        .unwrap_or_else(|e| panic!("develop_with_into under {options:?}: {e}"));
    dst
}

/// `develop_with_scratch_into` with exactly `scratch_len` samples of `fill`.
fn with_scratch(
    sensor: &Sensor,
    src: &[u16],
    options: &DevelopOptions,
    fill: impl Fn(usize) -> u16,
) -> Vec<u16> {
    let n = scratch_len(sensor, options).expect("scratch_len resolves");
    let mut scratch: Vec<u16> = (0..n).map(fill).collect();
    let mut dst = vec![0u16; dst_len(sensor, options)];
    develop_with_scratch_into(sensor, src, &mut dst, &mut scratch, options)
        .unwrap_or_else(|e| panic!("develop_with_scratch_into under {options:?}: {e}"));
    dst
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — scratch_len
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn scratch_len_per_case_and_alpha() {
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        for orientation in [1, 6] {
            for (case, at_one, at_zero) in [
                (Case::Neither, 0, 0),
                (Case::FbpOnly, P, P),
                (Case::WarpOnly, 2 * A, 0),
                (Case::Both, P + A, P),
            ] {
                let sensor = case_sensor(case, orientation);
                for apply_orientation in [true, false] {
                    let label = format!("{case:?} {framing:?} orientation {orientation}");
                    assert_eq!(
                        dbg(scratch_len(&sensor, &opts(framing, 1.0, apply_orientation))),
                        dbg(Ok::<usize, Error>(at_one)),
                        "{label} α 1"
                    );
                    assert_eq!(
                        dbg(scratch_len(&sensor, &opts(framing, 0.0, apply_orientation))),
                        dbg(Ok::<usize, Error>(at_zero)),
                        "{label} α 0"
                    );
                }
            }
        }
    }
    // An out-of-domain α is refused, as everywhere else.
    let sensor = case_sensor(Case::Both, 1);
    assert!(matches!(
        scratch_len(&sensor, &opts(Framing::StandardCrop, f64::NAN, true)),
        Err(Error::InvalidWarpAlpha { .. })
    ));
}

/// `AC10`: the length is computed with checked arithmetic. A plane of
/// `u32::MAX`² samples fits `u64`, but P + A and 2A do not, so the overflow is
/// reachable from `scratch_len` (which takes no buffer) on a hostile header.
#[test]
fn scratch_len_overflow_is_typed() {
    for case in [Case::WarpOnly, Case::Both] {
        let mut sensor = case_sensor(case, 1);
        sensor.width = u32::MAX;
        sensor.height = u32::MAX;
        sensor.active_area = None;
        sensor.default_crop_origin = None;
        sensor.default_crop_size = None;
        assert_eq!(
            dbg(scratch_len(&sensor, &DevelopOptions::default())),
            dbg(Err::<usize, _>(Error::DevelopScratchWrongLength {
                expected: u64::MAX,
                actual: 0,
            })),
            "{case:?}"
        );
    }
}

/// Tier B: the probe's figures (spec §1).
#[test]
fn scratch_len_on_held_files() {
    let q2m = [
        "LEICA-Q2-MONO/L1021223.DNG",
        "LEICA-Q2-MONO/L1026016.DNG",
        "LEICA-Q2-MONO/L1026192.DNG",
    ];
    let warp_free = [
        "LEICA-M-MONOCHROM/L1000622.DNG",
        "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
        "PENTAX-K3III-MONO/K3III.DNG",
        "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
        "NIKON-D300S/_DSC4615.DNG",
    ];
    for path in q2m {
        let Some((sensor, _, _)) = corpus_sensor(path) else {
            continue;
        };
        assert_eq!(
            dbg(scratch_len(&sensor, &DevelopOptions::default())),
            dbg(Ok::<usize, Error>(94_707_712)),
            "{path}: P + A at default options"
        );
        assert_eq!(
            dbg(scratch_len(&sensor, &opts(Framing::StandardCrop, 0.0, true))),
            dbg(Ok::<usize, Error>(47_443_968)),
            "{path}: P at α 0"
        );
    }
    for path in warp_free {
        let Some((sensor, _, _)) = corpus_sensor(path) else {
            continue;
        };
        assert_eq!(
            dbg(scratch_len(&sensor, &DevelopOptions::default())),
            dbg(Ok::<usize, Error>(0)),
            "{path}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — byte identity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn scratch_output_equals_develop_with_into() {
    let mut checked = 0;
    for case in CASES {
        for orientation in [1, 6] {
            let sensor = case_sensor(case, orientation);
            let src = plane(&sensor);
            for options in ac2_options() {
                let want = reference(&sensor, &src, &options);
                let got = with_scratch(&sensor, &src, &options, |_| 0xA5A5);
                assert_eq!(
                    got, want,
                    "{case:?} orientation {orientation} under {options:?}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 4 * 2 * 12);

    // The fix and the warp both did something on the "both" fixture, so the
    // identity above is not between two copies of the plain crop.
    let options = DevelopOptions::default();
    let both = reference(&case_sensor(Case::Both, 1), &plane(&bordered_sensor(1)), &options);
    let fbp = reference(
        &case_sensor(Case::FbpOnly, 1),
        &plane(&bordered_sensor(1)),
        &options,
    );
    let neither = reference(
        &case_sensor(Case::Neither, 1),
        &plane(&bordered_sensor(1)),
        &options,
    );
    assert_ne!(both, fbp, "the warp changes the output");
    assert_ne!(fbp, neither, "the fix changes the output");
}

#[test]
fn scratch_output_matches_the_pins() {
    for (path, width, height, pin) in PINNED {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let options = DevelopOptions::default();
        let n = scratch_len(&sensor, &options).unwrap();
        let mut scratch = vec![0xA5A5u16; n];
        let mut dst = vec![0u16; (width as usize) * (height as usize)];
        develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &options)
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(md5_le(&dst), pin, "{path}: SPEC-063 pin");

        if path == "LEICA-Q2-MONO/L1021223.DNG" {
            let options = opts(Framing::FullActiveArea, 0.5, true);
            let want = reference(&sensor, &src, &options);
            let got = with_scratch(&sensor, &src, &options, |_| 0xA5A5);
            assert!(got == want, "{path}: FullActiveArea α 0.5 differs");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — wrong length
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn wrong_length_scratch_is_refused_before_any_write() {
    let options = DevelopOptions::default();
    let sensor = case_sensor(Case::Both, 1);
    let src = plane(&sensor);
    let n = scratch_len(&sensor, &options).unwrap();
    assert_eq!(n, P + A);
    for len in [n - 1, n + 1, 0] {
        let mut scratch = vec![0x5A5Au16; len];
        let mut dst = vec![SENTINEL; dst_len(&sensor, &options)];
        let err = develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &options)
            .expect_err("a wrong-length scratch is refused");
        assert_eq!(
            format!("{err:?}"),
            format!(
                "{:?}",
                Error::DevelopScratchWrongLength {
                    expected: n as u64,
                    actual: len,
                }
            ),
            "length {len}"
        );
        assert_eq!(err.kind(), ErrorKind::CallerError);
        assert!(dst.iter().all(|&v| v == SENTINEL), "length {len}: dst written");
        assert!(
            scratch.iter().all(|&v| v == 0x5A5A),
            "length {len}: scratch written"
        );
    }

    // n == 0: one sample is refused, an empty slice develops.
    let sensor = case_sensor(Case::Neither, 1);
    assert_eq!(scratch_len(&sensor, &options).ok(), Some(0));
    let mut dst = vec![SENTINEL; dst_len(&sensor, &options)];
    let mut one = [0x5A5Au16; 1];
    assert_eq!(
        dbg(develop_with_scratch_into(&sensor, &src, &mut dst, &mut one, &options)),
        dbg(Err::<(), _>(Error::DevelopScratchWrongLength {
            expected: 0,
            actual: 1,
        }))
    );
    assert!(dst.iter().all(|&v| v == SENTINEL));
    assert_eq!(one, [0x5A5A]);
    develop_with_scratch_into(&sensor, &src, &mut dst, &mut [], &options)
        .expect("an empty scratch develops when scratch_len is 0");
    assert_eq!(dst, reference(&sensor, &src, &options));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — scratch contents never matter
// ─────────────────────────────────────────────────────────────────────────────

fn fills() -> [(&'static str, fn(usize) -> u16); 3] {
    [
        ("zero", |_| 0),
        ("ones", |_| 0xFFFF),
        ("ramp", |i| (i as u16).wrapping_mul(2_654_435_761u32 as u16)),
    ]
}

#[test]
fn output_does_not_depend_on_scratch_contents() {
    for orientation in [1, 6] {
        let sensor = case_sensor(Case::Both, orientation);
        let src = plane(&sensor);
        for options in ac2_options() {
            let want = reference(&sensor, &src, &options);
            for (name, fill) in fills() {
                let got = with_scratch(&sensor, &src, &options, fill);
                assert_eq!(got, want, "{name} fill, orientation {orientation}, {options:?}");
            }
        }
    }

    let path = "LEICA-Q2-MONO/L1021223.DNG";
    if let Some((sensor, src)) = corpus_plane(path) {
        let options = DevelopOptions::default();
        let outputs: Vec<String> = fills()
            .iter()
            .map(|(_, fill)| md5_le(&with_scratch(&sensor, &src, &options, fill)))
            .collect();
        assert_eq!(outputs[0], pin_for(path), "{path}: zero fill");
        assert_eq!(outputs[1], pin_for(path), "{path}: 0xFFFF fill");
        assert_eq!(outputs[2], pin_for(path), "{path}: ramp fill");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — same errors, same order
// ─────────────────────────────────────────────────────────────────────────────

/// A `FixBadPixelsConstant` with a 4-byte parameter block: it passes the
/// required-processing check but fails its own parse.
fn malformed_fix_bad_pixels() -> Vec<u8> {
    opcode_support::opcode_list(&[opcode_support::raw_opcode(4, 0x0103_0000, 0, &[0, 0, 0, 0])])
}

/// One row per distinct refusal shape in `tests/develop.rs`,
/// `tests/develop_options.rs` and `tests/warp_alpha.rs`, rebuilt here on the
/// "both" fixture (test binaries cannot share fixtures). Each row is
/// `(label, sensor, src length delta, dst length delta, options)`.
fn hostile_fixtures() -> Vec<(&'static str, Sensor, isize, isize, DevelopOptions)> {
    let both = case_sensor(Case::Both, 1);
    let default = DevelopOptions::default();
    let full = opts(Framing::FullActiveArea, 1.0, true);
    let mut out = Vec::new();

    let mut s = both.clone();
    s.opcode_list_2 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    out.push(("mandatory opcode in OpcodeList2", s, 0, 0, default));

    let mut s = both.clone();
    s.opcode_list_1 = None;
    s.malformed_tags.push(TAG_OPCODE_LIST_1);
    out.push(("unreadable OpcodeList1", s, 0, 0, default));

    let mut s = both.clone();
    s.opcode_list_1 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        0xDEAD_BEEF,
        0x0103_0000,
        0,
        &[1, 2, 3, 4],
    )]));
    out.push(("unknown mandatory opcode in OpcodeList1", s, 0, 0, default));

    let mut s = both.clone();
    s.opcode_list_3 = Some(opcode_support::opcode_list(&[
        opcode_support::raw_opcode(1, 0x0104_0000, 0, &opcode_support::warp_rectilinear_params(
            Q2M_KR,
            [0.0, 0.0],
            0.5,
            0.5,
            1,
        )),
        opcode_support::raw_opcode(1, 0x0104_0000, 0, &opcode_support::warp_rectilinear_params(
            Q2M_KR,
            [0.0, 0.0],
            0.5,
            0.5,
            1,
        )),
    ]));
    out.push(("second mandatory warp in OpcodeList3", s, 0, 0, default));

    out.push(("src one short", both.clone(), -1, 0, default));
    out.push(("src one long", both.clone(), 1, 0, full));

    let mut s = both.clone();
    s.active_area = Some(ActiveArea {
        top: 1,
        left: 2,
        bottom: 9,
        right: 9,
    });
    out.push(("active area past the plane", s, 0, 0, default));

    let mut s = both.clone();
    s.default_crop_size = Some(DefaultCropSize {
        width: 0,
        height: 3,
    });
    out.push(("zero default crop", s.clone(), 0, 0, default));
    out.push(("zero default crop under FullActiveArea", s, 0, 0, full));

    let mut s = both.clone();
    s.orientation = Some(9);
    out.push(("orientation 9", s.clone(), 0, 0, default));
    out.push((
        "orientation 9 unoriented",
        s,
        0,
        0,
        opts(Framing::StandardCrop, 1.0, false),
    ));

    let mut s = both.clone();
    s.black_level = Some(70_000);
    s.white_level = Some(100);
    out.push(("black at or above white", s, 0, 0, default));

    out.push(("dst one long", both.clone(), 0, 1, default));
    out.push(("dst one short under FullActiveArea", both.clone(), 0, -1, full));

    let mut s = both.clone();
    s.opcode_list_1 = Some(malformed_fix_bad_pixels());
    out.push(("malformed OpcodeList1", s.clone(), 0, 0, default));
    out.push(("malformed OpcodeList1 under FullActiveArea", s, 0, 0, full));

    let mut over_unity = both.clone();
    over_unity.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        PIXEL_3A_KR,
        [0.0, 0.0],
        0.501_746_677_217_029_4,
        0.502_773_708_767_361_2,
        1,
        0,
    ));
    for alpha in [0.25, 1.0] {
        out.push((
            "over-unity warp under FullActiveArea",
            over_unity.clone(),
            0,
            0,
            opts(Framing::FullActiveArea, alpha, true),
        ));
    }
    let mut s = over_unity.clone();
    s.opcode_list_1 = Some(malformed_fix_bad_pixels());
    out.push(("over-unity warp and malformed OpcodeList1", s, 0, 0, full));

    let mut s = both.clone();
    s.opcode_list_3 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        1,
        0x0104_0000,
        0,
        &[0, 0, 0, 1],
    )]));
    out.push(("malformed OpcodeList3", s.clone(), 0, 0, default));
    out.push(("malformed OpcodeList3 under FullActiveArea", s, 0, 0, full));

    let mut s = both.clone();
    s.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        Q2M_KR,
        [0.001, 0.0],
        0.5,
        0.5,
        1,
        0,
    ));
    out.push(("tangential terms (the applier refuses)", s.clone(), 0, 0, default));
    let mut no_fbp = s;
    no_fbp.opcode_list_1 = None;
    out.push(("tangential terms, warp only", no_fbp, 0, 0, default));

    for alpha in [f64::NAN, 1.5, -0.25, f64::INFINITY] {
        out.push((
            "out-of-domain warp_alpha",
            both.clone(),
            0,
            0,
            opts(Framing::StandardCrop, alpha, true),
        ));
    }
    out
}

/// Results compared by `Debug` (`Error` is not `PartialEq`).
fn dbg<T: std::fmt::Debug>(value: Result<T, Error>) -> String {
    format!("{value:?}")
}

fn resized(len: usize, delta: isize) -> usize {
    len.checked_add_signed(delta).unwrap()
}

#[test]
fn scratch_variant_reports_the_same_errors_in_the_same_order() {
    let fixtures = hostile_fixtures();
    assert!(fixtures.len() >= 25, "table shrank: {}", fixtures.len());
    for (label, sensor, src_delta, dst_delta, options) in &fixtures {
        let src = vec![7u16; resized(P, *src_delta)];
        // A dst sized from the options when they resolve, else any length:
        // the geometry refusal comes before the dst check either way.
        let base = output_dimensions_with(sensor, options)
            .map(|(w, h)| (w as usize) * (h as usize))
            .unwrap_or(12);
        let mut dst = vec![SENTINEL; resized(base, *dst_delta)];
        let want = develop_with_into(sensor, &src, &mut dst, options)
            .expect_err(&format!("{label}: develop_with_into must refuse"));

        // A correct-length scratch when the length resolves; otherwise an
        // empty one (the refusal precedes the length check).
        let n = scratch_len(sensor, options).unwrap_or(0);
        let mut scratch = vec![0x5A5Au16; n];
        let mut dst = vec![SENTINEL; resized(base, *dst_delta)];
        let got = develop_with_scratch_into(sensor, &src, &mut dst, &mut scratch, options)
            .expect_err(&format!("{label}: develop_with_scratch_into must refuse"));
        assert_eq!(format!("{got:?}"), format!("{want:?}"), "{label}");
    }

    // A wrong scratch as well: the file's refusals still come first.
    let options = DevelopOptions::default();
    let mut s = case_sensor(Case::Both, 1);
    s.opcode_list_1 = Some(malformed_fix_bad_pixels());
    let src = vec![7u16; P];
    let mut dst = vec![SENTINEL; dst_len(&s, &options)];
    let err = develop_with_scratch_into(&s, &src, &mut dst, &mut [0u16; 1], &options).unwrap_err();
    assert!(
        matches!(err, Error::MalformedOpcodeParams { .. }),
        "a malformed OpcodeList1 is reported before the scratch length: {err:?}"
    );
    let s = case_sensor(Case::Both, 1);
    let err = develop_with_scratch_into(
        &s,
        &src,
        &mut dst,
        &mut [0u16; 1],
        &opts(Framing::StandardCrop, f64::NAN, true),
    )
    .unwrap_err();
    assert!(
        matches!(err, Error::InvalidWarpAlpha { .. }),
        "an out-of-domain α is reported first: {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — Display
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn scratch_error_display_names_both_lengths() {
    let text = Error::DevelopScratchWrongLength {
        expected: 94_707_712,
        actual: 47_443_968,
    }
    .to_string();
    assert!(text.contains("94707712"), "{text}");
    assert!(text.contains("47443968"), "{text}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B helpers (as tests/warp_alpha.rs)
// ─────────────────────────────────────────────────────────────────────────────

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

fn corpus_plane(path: &str) -> Option<(Sensor, Vec<u16>)> {
    let (sensor, byte_order, bytes) = corpus_sensor(path)?;
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, byte_order, &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    Some((sensor, plane))
}

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

fn pin_for(path: &str) -> &'static str {
    PINNED
        .iter()
        .find(|(p, _, _, _)| *p == path)
        .map(|(_, _, _, md5)| *md5)
        .unwrap_or_else(|| panic!("{path} has no SPEC-063 pin"))
}
