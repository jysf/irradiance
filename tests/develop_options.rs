//! `SPEC-063` — `DevelopOptions` / `develop_with_into` /
//! `output_dimensions_with` (`DEC-058`).
//!
//! The load-bearing oracle is `AC2`: the MD5 of `develop_into`'s output on
//! eight held tier-B files, pinned from main at `3458854` **before** this
//! spec's change. If one goes red, the change altered default output — fix
//! the change, never the pin.
//!
//! Every other assertion is a **relation** between two public-API calls, never
//! a re-implementation of the develop arithmetic in the test:
//!
//! - `FullActiveArea` output equals `StandardCrop` output of the same sensor
//!   with its `DefaultCrop` widened to the whole `ActiveArea` (`AC4(a)`), and
//!   it contains the default output at the crop origin (`AC4(b)`).
//! - `apply_orientation: false` output equals the default output of the same
//!   sensor with `Orientation` forced to 1 (`AC6`).
//!
//! Two lanes, as `tests/develop.rs` establishes: tier A builds a `Sensor` from
//! `Sensor::test_fixture()` (needs the `test-fixtures` feature, which
//! `just test`'s `--all-features` turns on); tier B reads the held files under
//! `$IRRADIANCE_CORPUS_DIR` in place (`DEC-003`) and skips per entry, by name,
//! when one is absent (`CorpusFile::require`; `just test` prints the
//! corpus-status lines first).

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
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind, DEVELOP_VERSION};

// ─────────────────────────────────────────────────────────────────────────────
// Shared fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A value no develop of these fixtures produces at the positions checked, so
/// an untouched `dst` is distinguishable from a written one.
const SENTINEL: u16 = 0xABCD;

/// The tier-A fixture: a 10×8 plane with a 7×6 `ActiveArea` at `(left 2,
/// top 1)` and a NON-square 4×3 `DefaultCrop` at `(1, 1)` inside it, so the
/// crop, the active area and the plane all differ, and an Orientation 5-8
/// swap is visible in the dimensions. Identity levels (`0..=65535`) keep the
/// raw values recognisable in the output.
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

/// The same shape without an `ActiveArea` tag: the whole 10×8 plane is
/// active, with a 6×5 crop at `(2, 1)`.
fn plane_active_sensor(orientation: u32) -> Sensor {
    let mut sensor = bordered_sensor(orientation);
    sensor.active_area = None;
    sensor.default_crop_origin = Some(DefaultCropOrigin { x: 2, y: 1 });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 6,
        height: 5,
    });
    sensor
}

/// Distinct sample values for a `width × height` plane (no two equal, none 0).
fn distinct_plane(sensor: &Sensor) -> Vec<u16> {
    let len = (sensor.width * sensor.height) as usize;
    (0..len).map(|i| 100 + 7 * i as u16).collect()
}

/// `FullActiveArea` options.
fn full_active_area() -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = Framing::FullActiveArea;
    options
}

/// `apply_orientation: false` options.
fn unoriented() -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.apply_orientation = false;
    options
}

/// `ActiveArea`'s size for `sensor` (the plane's when the tag is absent).
fn active_size(sensor: &Sensor) -> (u32, u32) {
    match sensor.active_area {
        Some(a) => (a.right - a.left, a.bottom - a.top),
        None => (sensor.width, sensor.height),
    }
}

/// The same sensor with its `DefaultCrop` widened to the whole `ActiveArea` —
/// `AC4(a)`'s `StandardCrop` twin of `FullActiveArea`.
fn crop_widened_to_active_area(sensor: &Sensor) -> Sensor {
    let (width, height) = active_size(sensor);
    let mut widened = sensor.clone();
    widened.default_crop_origin = Some(DefaultCropOrigin { x: 0, y: 0 });
    widened.default_crop_size = Some(DefaultCropSize { width, height });
    widened
}

/// Develop with `options` into a freshly sized buffer.
fn develop_with(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> (u32, u32, Vec<u16>) {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions resolve");
    let mut dst = vec![0u16; (w as usize) * (h as usize)];
    develop_with_into(sensor, src, &mut dst, options).expect("develops");
    (w, h, dst)
}

/// Develop with the plain `develop_into` into a freshly sized buffer.
fn develop_default(sensor: &Sensor, src: &[u16]) -> (u32, u32, Vec<u16>) {
    let (w, h) = output_dimensions(sensor).expect("dimensions resolve");
    let mut dst = vec![0u16; (w as usize) * (h as usize)];
    develop_into(sensor, src, &mut dst).expect("develops");
    (w, h, dst)
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

/// A non-identity `WarpRectilinear` `OpcodeList3` (`kr1 = -0.1`), the same
/// shape `tests/develop.rs`'s warp-branch fixture uses.
fn non_identity_warp() -> Vec<u8> {
    opcode_support::single_warp_rectilinear_list([1.0, -0.1, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5, 1, 0)
}

/// An identity `WarpRectilinear` (`kr = [1, 0, 0, 0]`, `kt = 0`), which
/// `develop_into` already skips.
fn identity_warp() -> Vec<u8> {
    opcode_support::single_warp_rectilinear_list([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5, 1, 0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B helpers
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

/// MD5 of `samples` as little-endian `u16` bytes — the §1 pin's encoding.
fn md5_le(samples: &[u16]) -> String {
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    md5::to_hex(&md5::hash(&bytes))
}

/// SPEC-063 §1: `develop_into` output MD5s from main at 3458854, before this
/// change. `(path, width, height, md5)`.
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

/// The three Pixel files `develop_into` refuses (§1).
const PIXEL_REFUSED: [&str; 3] = [
    "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng",
    "GOOGLE-PIXEL-3A/IMG_20190918_164153.dng",
    "GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng",
];

/// The three warp-free mono files with a real border between `ActiveArea` and
/// `DefaultCrop`, and their `ActiveArea` sizes (§1).
const WARP_FREE_BORDERED: [(&str, u32, u32); 3] = [
    ("PENTAX-K3III-MONO/K3III.DNG", 6224, 4160),
    ("LEICA-M-MONOCHROM/L1000622.DNG", 5216, 3472),
    ("LEICA-M-MONOCHROM-TYP246/M2462362.DNG", 5984, 4000),
];

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — defaults
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn default_options_are_standard_crop_and_oriented() {
    let options = DevelopOptions::default();
    assert_eq!(options.framing, Framing::StandardCrop);
    assert!(options.apply_orientation);
    assert_eq!(Framing::default(), Framing::StandardCrop);
    // Copy + PartialEq, as DEC-058 §1 derives.
    let copy = options;
    assert_eq!(copy, DevelopOptions::default());
    assert_ne!(full_active_area(), DevelopOptions::default());
    assert_ne!(unoriented(), DevelopOptions::default());
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — byte identity on the held files (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn develop_into_output_is_unchanged_on_held_files() {
    assert_eq!(
        DEVELOP_VERSION, "4",
        "SPEC-063 does not bump DEVELOP_VERSION"
    );

    for (path, width, height, pinned) in PINNED {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue; // SKIP announced by name by CorpusFile::require
        };
        assert_eq!(
            output_dimensions(&sensor).expect("dimensions"),
            (width, height),
            "{path}: output_dimensions"
        );
        assert_eq!(
            output_dimensions_with(&sensor, &DevelopOptions::default()).expect("dimensions"),
            (width, height),
            "{path}: output_dimensions_with(default)"
        );

        let mut dst = vec![0u16; (width as usize) * (height as usize)];
        develop_into(&sensor, &plane, &mut dst).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(md5_le(&dst), pinned, "{path}: develop_into output MD5");

        dst.fill(SENTINEL);
        develop_with_into(&sensor, &plane, &mut dst, &DevelopOptions::default())
            .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(
            md5_le(&dst),
            pinned,
            "{path}: develop_with_into(default) output MD5"
        );
    }

    for path in PIXEL_REFUSED {
        let Some((sensor, _byte_order, _bytes)) = corpus_sensor(path) else {
            continue;
        };
        // The refusal is the required-processing check, which runs before any
        // buffer is examined, so empty buffers reach it.
        let plain = develop_into(&sensor, &[], &mut []).expect_err("refused");
        let with = develop_with_into(&sensor, &[], &mut [], &DevelopOptions::default())
            .expect_err("refused");
        assert!(
            plain
                .to_string()
                .contains("opcode 9 is unrecognized and not marked optional"),
            "{path}: unexpected refusal {plain}"
        );
        assert_eq!(plain.to_string(), with.to_string(), "{path}: same refusal");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — FullActiveArea dimensions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn full_active_area_dimensions() {
    // Tier A: with an ActiveArea tag (7×6) and without (the 10×8 plane),
    // Orientation 1 and 6.
    assert_eq!(
        output_dimensions_with(&bordered_sensor(1), &full_active_area()).unwrap(),
        (7, 6)
    );
    assert_eq!(
        output_dimensions_with(&bordered_sensor(6), &full_active_area()).unwrap(),
        (6, 7),
        "Orientation 6 swaps the active size when orientation is applied"
    );
    assert_eq!(
        output_dimensions_with(&plane_active_sensor(1), &full_active_area()).unwrap(),
        (10, 8)
    );
    assert_eq!(
        output_dimensions_with(&plane_active_sensor(6), &full_active_area()).unwrap(),
        (8, 10)
    );
    // And the default stays the crop size.
    assert_eq!(output_dimensions(&bordered_sensor(1)).unwrap(), (4, 3));
    assert_eq!(output_dimensions(&bordered_sensor(6)).unwrap(), (3, 4));

    // Tier B: the warp-free bordered files, and a Q2M (which develop then
    // refuses, §4 — output_dimensions_with does not look at opcode lists).
    let tier_b =
        WARP_FREE_BORDERED
            .iter()
            .copied()
            .chain([("LEICA-Q2-MONO/L1021223.DNG", 8392, 5632)]);
    for (path, width, height) in tier_b {
        let Some((sensor, _byte_order, _bytes)) = corpus_sensor(path) else {
            continue;
        };
        assert_eq!(
            output_dimensions_with(&sensor, &full_active_area()).unwrap(),
            (width, height),
            "{path}: FullActiveArea dimensions"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — FullActiveArea pixels, by two relations
// ─────────────────────────────────────────────────────────────────────────────

/// The tier-A sensors AC4 runs over, with a label. One carries a mandatory
/// `FixBadPixelsConstant` (and a bad pixel), so the copy path runs under
/// `FullActiveArea` too.
fn ac4_tier_a_cases() -> Vec<(&'static str, Sensor, Vec<u16>)> {
    let mut cases = Vec::new();
    for orientation in [1, 6] {
        let sensor = bordered_sensor(orientation);
        let src = distinct_plane(&sensor);
        cases.push(("bordered", sensor, src));
        let sensor = plane_active_sensor(orientation);
        let src = distinct_plane(&sensor);
        cases.push(("no ActiveArea tag", sensor, src));
    }
    let mut sensor = bordered_sensor(1);
    sensor.opcode_list_1 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    let mut src = distinct_plane(&sensor);
    // A bad pixel inside the ActiveArea but OUTSIDE the DefaultCrop: only
    // FullActiveArea shows it, so the fix must have run on the plane it reads.
    src[10 + 2] = 0; // raw (x 2, y 1) in the 10-wide plane: the active origin
    cases.push(("FixBadPixelsConstant", sensor, src));
    cases
}

#[test]
fn full_active_area_equals_standard_crop_over_the_whole_active_area() {
    for (label, sensor, src) in ac4_tier_a_cases() {
        let (fw, fh, full) = develop_with(&sensor, &src, &full_active_area());
        let (sw, sh, widened) = develop_default(&crop_widened_to_active_area(&sensor), &src);
        assert_eq!((fw, fh), (sw, sh), "{label}: dimensions");
        assert_eq!(
            full, widened,
            "{label} (Orientation {:?})",
            sensor.orientation
        );
        if label == "FixBadPixelsConstant" {
            assert_ne!(full[0], 0, "the bad pixel at the active origin is fixed");
        }
    }

    for (path, width, height) in WARP_FREE_BORDERED {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue;
        };
        let (fw, fh, full) = develop_with(&sensor, &plane, &full_active_area());
        assert_eq!((fw, fh), (width, height), "{path}");
        let (_, _, widened) = develop_default(&crop_widened_to_active_area(&sensor), &plane);
        assert!(
            full == widened,
            "{path}: FullActiveArea != widened StandardCrop"
        );
    }
}

#[test]
fn full_active_area_contains_the_standard_crop() {
    for (label, sensor, src) in ac4_tier_a_cases() {
        if sensor.orientation != Some(1) {
            continue; // containment at the crop origin is an unrotated relation
        }
        let (fw, _, full) = develop_with(&sensor, &src, &full_active_area());
        let (sw, _, standard) = develop_default(&sensor, &src);
        let origin = sensor
            .default_crop_origin
            .map(|o| (o.x, o.y))
            .unwrap_or((0, 0));
        assert_contains_at(label, &full, fw, &standard, sw, origin);
        assert_ne!(full.len(), standard.len(), "{label}: a real border exists");
    }

    for (path, _, _) in WARP_FREE_BORDERED {
        let Some((sensor, plane)) = corpus_plane(path) else {
            continue;
        };
        assert_eq!(
            sensor.orientation.unwrap_or(1),
            1,
            "{path}: held at Orientation 1"
        );
        let (fw, _, full) = develop_with(&sensor, &plane, &full_active_area());
        let (sw, _, standard) = develop_default(&sensor, &plane);
        let origin = sensor
            .default_crop_origin
            .map(|o| (o.x, o.y))
            .unwrap_or((0, 0));
        assert_contains_at(path, &full, fw, &standard, sw, origin);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — the refusal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn full_active_area_with_a_warp_is_refused_before_any_write() {
    // Synthetic: a non-identity warp AND a FixBadPixelsConstant.
    let mut sensor = bordered_sensor(1);
    sensor.opcode_list_1 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    sensor.opcode_list_3 = Some(non_identity_warp());
    let src = distinct_plane(&sensor);

    let (w, h) = output_dimensions_with(&sensor, &full_active_area()).unwrap();
    assert_eq!(
        (w, h),
        (7, 6),
        "dimensions still resolve (no opcode lists read)"
    );
    let mut dst = vec![SENTINEL; (w * h) as usize];
    let err = develop_with_into(&sensor, &src, &mut dst, &full_active_area()).unwrap_err();
    assert!(
        matches!(
            err,
            Error::UnsupportedWarpedFullActiveArea {
                active_width: 7,
                active_height: 6
            }
        ),
        "got {err:?}"
    );
    assert_eq!(err.kind(), ErrorKind::Unsupported);
    assert!(dst.iter().all(|&v| v == SENTINEL), "dst must be untouched");

    // StandardCrop: the same sensor develops exactly as develop_into does.
    let (_, _, standard) = develop_with(&sensor, &src, &DevelopOptions::default());
    let (_, _, plain) = develop_default(&sensor, &src);
    assert_eq!(standard, plain);

    // Order (§4): under FullActiveArea the warp refusal comes before
    // OpcodeList1's parameters are parsed (and so before the FixBadPixels
    // copy); under StandardCrop the parse order is today's, OpcodeList1
    // first. A FixBadPixelsConstant with a 4-byte parameter block passes the
    // required-processing check (which reads no parameters) but fails its
    // own parse.
    let mut malformed_fix = sensor.clone();
    malformed_fix.opcode_list_1 = Some(opcode_support::opcode_list(&[opcode_support::raw_opcode(
        4,
        0x0103_0000,
        0,
        &[0, 0, 0, 0],
    )]));
    let mut dst = vec![SENTINEL; (w * h) as usize];
    let err = develop_with_into(&malformed_fix, &src, &mut dst, &full_active_area()).unwrap_err();
    assert!(
        matches!(err, Error::UnsupportedWarpedFullActiveArea { .. }),
        "FullActiveArea refuses the warp before parsing OpcodeList1: got {err:?}"
    );
    assert!(dst.iter().all(|&v| v == SENTINEL));
    let (sw, sh) = output_dimensions(&malformed_fix).unwrap();
    let mut dst = vec![SENTINEL; (sw * sh) as usize];
    let err = develop_into(&malformed_fix, &src, &mut dst).unwrap_err();
    assert!(
        matches!(err, Error::MalformedOpcodeParams { .. }),
        "StandardCrop keeps today's order (OpcodeList1 first): got {err:?}"
    );

    // Real file: every held Q2M frame carries a non-identity warp. The
    // refusal comes before src is read, so a zero plane of the right length
    // reaches it without a 47 MP unpack.
    let path = "LEICA-Q2-MONO/L1021223.DNG";
    if let Some((sensor, _byte_order, _bytes)) = corpus_sensor(path) {
        let src = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
        let (w, h) = output_dimensions_with(&sensor, &full_active_area()).unwrap();
        let mut dst = vec![SENTINEL; (w as usize) * (h as usize)];
        let err = develop_with_into(&sensor, &src, &mut dst, &full_active_area()).unwrap_err();
        assert!(
            matches!(
                err,
                Error::UnsupportedWarpedFullActiveArea {
                    active_width: 8392,
                    active_height: 5632
                }
            ),
            "{path}: got {err:?}"
        );
        assert!(dst.iter().all(|&v| v == SENTINEL), "{path}: dst untouched");
    }
}

#[test]
fn identity_warp_does_not_trigger_the_refusal() {
    let mut sensor = bordered_sensor(1);
    sensor.opcode_list_3 = Some(identity_warp());
    let src = distinct_plane(&sensor);
    let (_, _, full) = develop_with(&sensor, &src, &full_active_area());

    let mut no_warp = sensor.clone();
    no_warp.opcode_list_3 = None;
    let (_, _, expected) = develop_with(&no_warp, &src, &full_active_area());
    assert_eq!(full, expected, "an identity warp is not a warp");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — apply_orientation: false
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn unoriented_output_equals_orientation_one_for_all_eight() {
    for orientation in 1..=8 {
        let sensor = bordered_sensor(orientation);
        let src = distinct_plane(&sensor);
        let (w, h, unrotated) = develop_with(&sensor, &src, &unoriented());
        assert_eq!(
            (w, h),
            (4, 3),
            "Orientation {orientation}: unswapped crop size"
        );

        let mut upright = sensor.clone();
        upright.orientation = Some(1);
        let (_, _, expected) = develop_default(&upright, &src);
        assert_eq!(unrotated, expected, "Orientation {orientation}");

        // And the option is not a no-op where orientation matters.
        if orientation != 1 {
            let (_, _, oriented) = develop_default(&sensor, &src);
            assert_ne!(
                unrotated, oriented,
                "Orientation {orientation} rotates by default"
            );
        }
    }
}

#[test]
fn unoriented_q2m_orientation_six_through_the_warp_path() {
    let path = "LEICA-Q2-MONO/L1026016.DNG";
    let Some((sensor, plane)) = corpus_plane(path) else {
        return;
    };
    assert_eq!(sensor.orientation, Some(6), "{path}: held at Orientation 6");
    let (w, h, unrotated) = develop_with(&sensor, &plane, &unoriented());
    assert_eq!((w, h), (8368, 5584), "{path}: unswapped crop size");

    let mut upright = sensor.clone();
    upright.orientation = Some(1);
    let (_, _, expected) = develop_default(&upright, &plane);
    assert!(
        unrotated == expected,
        "{path}: unoriented != Orientation 1 develop"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — combination
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn full_active_area_unoriented_combination() {
    let sensor = bordered_sensor(6);
    let src = distinct_plane(&sensor);
    let mut options = full_active_area();
    options.apply_orientation = false;

    let (w, h, got) = develop_with(&sensor, &src, &options);
    assert_eq!((w, h), (7, 6), "active size, unswapped");

    let mut expected_sensor = crop_widened_to_active_area(&sensor);
    expected_sensor.orientation = Some(1);
    let (_, _, expected) = develop_default(&expected_sensor, &src);
    assert_eq!(got, expected);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 — validation is not skipped (DEC-058 §3)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn options_do_not_skip_tag_validation() {
    // FullActiveArea ignores DefaultCrop for output, but a crop that does not
    // fit is still refused.
    let mut bad_crop = bordered_sensor(1);
    bad_crop.default_crop_size = Some(DefaultCropSize {
        width: 100,
        height: 3,
    });
    let src = distinct_plane(&bad_crop);
    let mut dst = vec![SENTINEL; 7 * 6];
    for options in [full_active_area(), DevelopOptions::default()] {
        assert!(
            matches!(
                output_dimensions_with(&bad_crop, &options),
                Err(Error::InvalidDefaultCrop { .. })
            ),
            "{options:?}: output_dimensions_with"
        );
        assert!(
            matches!(
                develop_with_into(&bad_crop, &src, &mut dst, &options),
                Err(Error::InvalidDefaultCrop { .. })
            ),
            "{options:?}: develop_with_into"
        );
    }

    // apply_orientation: false ignores Orientation for output, but 9 is still
    // refused.
    let bad_orientation = bordered_sensor(9);
    let mut dst = vec![SENTINEL; 4 * 3];
    for options in [unoriented(), DevelopOptions::default()] {
        assert!(
            matches!(
                output_dimensions_with(&bad_orientation, &options),
                Err(Error::UnsupportedOrientation { orientation: 9 })
            ),
            "{options:?}: output_dimensions_with"
        );
        assert!(
            matches!(
                develop_with_into(&bad_orientation, &src, &mut dst, &options),
                Err(Error::UnsupportedOrientation { orientation: 9 })
            ),
            "{options:?}: develop_with_into"
        );
    }
    assert!(dst.iter().all(|&v| v == SENTINEL));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC9 — buffer lengths
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dst_length_is_checked_against_the_options_dimensions() {
    let sensor = bordered_sensor(1);
    let src = distinct_plane(&sensor);

    // A dst sized for the default (4×3) crop, passed with FullActiveArea (7×6).
    let mut dst = vec![SENTINEL; 4 * 3];
    let err = develop_with_into(&sensor, &src, &mut dst, &full_active_area()).unwrap_err();
    assert!(
        matches!(
            err,
            Error::DevelopBufferWrongLength {
                expected: 42,
                actual: 12
            }
        ),
        "got {err:?}"
    );
    assert!(dst.iter().all(|&v| v == SENTINEL), "dst must be untouched");

    // The reverse: a dst sized for FullActiveArea, passed with the default.
    let mut dst = vec![SENTINEL; 7 * 6];
    let err = develop_with_into(&sensor, &src, &mut dst, &DevelopOptions::default()).unwrap_err();
    assert!(
        matches!(
            err,
            Error::DevelopBufferWrongLength {
                expected: 12,
                actual: 42
            }
        ),
        "got {err:?}"
    );
    assert!(dst.iter().all(|&v| v == SENTINEL));

    // Orientation 6 with apply_orientation: false. The buffer length is a
    // product, so a swap does not change it: the default-sized buffer is the
    // right length and receives the unswapped 4×3 layout.
    let rotated = bordered_sensor(6);
    let (dw, dh) = output_dimensions(&rotated).unwrap();
    assert_eq!((dw, dh), (3, 4));
    let mut dst = vec![SENTINEL; (dw * dh) as usize];
    develop_with_into(&rotated, &src, &mut dst, &unoriented()).expect("same length");
    let mut upright = rotated.clone();
    upright.orientation = Some(1);
    let (_, _, expected) = develop_default(&upright, &src);
    assert_eq!(dst, expected);

    // A short src is still SourcePlaneWrongLength under any option.
    let mut dst = vec![SENTINEL; 7 * 6];
    let err = develop_with_into(&sensor, &src[1..], &mut dst, &full_active_area()).unwrap_err();
    assert!(
        matches!(
            err,
            Error::SourcePlaneWrongLength {
                expected: 80,
                actual: 79
            }
        ),
        "got {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — error surface
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn refusal_display_names_the_active_size() {
    let err = Error::UnsupportedWarpedFullActiveArea {
        active_width: 8392,
        active_height: 5632,
    };
    let text = err.to_string();
    assert!(text.contains("8392"), "{text}");
    assert!(text.contains("5632"), "{text}");
    assert!(
        !text.contains("SPEC-"),
        "no spec IDs in user-facing text: {text}"
    );
    assert_eq!(err.kind(), ErrorKind::Unsupported);
}
