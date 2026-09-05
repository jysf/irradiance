//! `SPEC-014` — level normalization, `ActiveArea` -> `DefaultCrop` ->
//! `Orientation`, `DEC-018`/`DEC-019`.
//!
//! ⚠ **This spec has no oracle** (`DEC-004`): `dnglab --raw-checksum`
//! attaches before any of this, and no comparison oracle can see a levels or
//! geometry error (`docs/oracle-contract.md`). Every assertion here is
//! analytic, against tag values **read from the file**, never a hardcoded
//! constant — the same discipline `tests/plane_unpack.rs` uses for
//! `bits_per_sample`.
//!
//! Two lanes, as `tests/plane_unpack.rs` establishes the pattern:
//!
//! - **Tier A** (`values_outside_the_level_range_are_handled_as_decided`'s
//!   synthetic half, `hostile_geometry_does_not_panic`) build a `Sensor`
//!   directly — this module's public API takes `&Sensor` + a plane slice, not
//!   file bytes, so a hand-built fixture is a struct literal, not a TIFF byte
//!   sequence. `crop_origin_is_relative_to_active_area`'s hand-built fixture
//!   (`AC4`) lives as a unit test in `src/develop.rs` instead — it exercises
//!   the module's own internal geometry resolution and needs no file I/O,
//!   which is exactly the "unit test in the module it tests" case AGENTS.md
//!   §12 describes; `cargo test crop_origin_is_relative_to_active_area`
//!   matches it there.
//! - **Tier B** (`black_and_white_levels_map_to_the_endpoints`,
//!   `the_three_stage_crop_produces_the_measured_dimensions`,
//!   `orientation_six_swaps_the_output_dimensions`,
//!   `an_unrotated_sibling_keeps_its_dimensions`, and this test's tier-B
//!   half) need real files under `$IRRADIANCE_CORPUS_DIR` and skip loudly,
//!   per-entry, when absent.

#[path = "support/corpus.rs"]
mod corpus;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{
    ActiveArea, Compression, Container, DefaultCropOrigin, DefaultCropSize, Sensor,
};
use irradiance::Error;

/// A minimal, valid `Sensor` covering the whole `width x height` plane (no
/// `ActiveArea`/crop/orientation tags) — a test overrides what it needs.
fn minimal_sensor(width: u32, height: u32) -> Sensor {
    Sensor {
        ifd_index: 0,
        width,
        height,
        bits_per_sample: 14,
        samples_per_pixel: 1,
        photometric: 34892,
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
        malformed_tags: vec![],
    }
}

/// The real `Sensor` for one corpus file, or `None` with the skip already
/// announced by `CorpusFile::require`.
fn corpus_sensor(path: &str) -> Option<Sensor> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(&root)?;
    let bytes = std::fs::read(&file_path).expect("read corpus file");
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    Some(container.sensor().unwrap_or_else(|e| panic!("{path}: {e}")))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — levels normalize analytically, on values read from the file
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn black_and_white_levels_map_to_the_endpoints() {
    for (path, expected_black, expected_white) in [
        ("LEICA-Q2-MONO/L1021223.DNG", 512u32, 16383u32),
        ("LEICA-M-MONOCHROM/L1000622.DNG", 220u32, 16383u32),
    ] {
        let Some(real_sensor) = corpus_sensor(path) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        let black = real_sensor
            .black_level
            .unwrap_or_else(|| panic!("{path}: must carry BlackLevel"));
        let white = real_sensor
            .white_level
            .unwrap_or_else(|| panic!("{path}: must carry WhiteLevel"));
        assert_eq!(
            black, expected_black,
            "{path}: BlackLevel read from the file"
        );
        assert_eq!(
            white, expected_white,
            "{path}: WhiteLevel read from the file"
        );

        // A tiny synthetic plane carrying exactly BlackLevel, WhiteLevel, and
        // an interior point — the arithmetic under test, not a 47-megapixel
        // decode. The LEVELS are read from the real file; the plane is not.
        let mut sensor = minimal_sensor(3, 1);
        sensor.black_level = Some(black);
        sensor.white_level = Some(white);
        let black_u16 = u16::try_from(black).expect("14/16-bit level fits u16");
        let white_u16 = u16::try_from(white).expect("14/16-bit level fits u16");
        let interior = black + (white - black) / 2;
        let interior_u16 = u16::try_from(interior).expect("interior point fits u16");

        let src = [black_u16, white_u16, interior_u16];
        let mut dst = [0u16; 3];
        develop_into(&sensor, &src, &mut dst).expect("fits");

        assert_eq!(dst[0], 0, "{path}: BlackLevel must map to 0");
        assert_eq!(
            dst[1],
            u16::MAX,
            "{path}: WhiteLevel must map to full scale"
        );
        assert!(
            dst[2] > 0 && dst[2] < u16::MAX,
            "{path}: an interior point must land strictly between the endpoints, got {}",
            dst[2]
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — values outside [BlackLevel, WhiteLevel] are clamped (DEC-018)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn values_outside_the_level_range_are_handled_as_decided() {
    // Tier A: a value below BlackLevel and a value above WhiteLevel,
    // synthetic — `develop_into` takes the plane directly, so this reaches
    // the edge even though `plane::unpack_into` would itself reject a sample
    // above WhiteLevel before `develop_into` ever saw it.
    let mut sensor = minimal_sensor(2, 1);
    sensor.black_level = Some(512);
    sensor.white_level = Some(16383);
    let src = [0u16, u16::MAX]; // far below BlackLevel, far above WhiteLevel
    let mut dst = [0u16; 2];
    develop_into(&sensor, &src, &mut dst).expect("fits");
    assert_eq!(dst[0], 0, "below BlackLevel clamps to 0, does not wrap");
    assert_eq!(
        dst[1],
        u16::MAX,
        "above WhiteLevel clamps to full scale, does not wrap"
    );

    // Tier B: AC2 is not hypothetical — it fires on the FIRST file. Both
    // measured real planes contain samples below BlackLevel (min 2 and 108),
    // read from the real file's tags, not hardcoded.
    for (path, measured_min) in [
        ("LEICA-Q2-MONO/L1021223.DNG", 2u16),
        ("LEICA-M-MONOCHROM/L1000622.DNG", 108u16),
    ] {
        let Some(real_sensor) = corpus_sensor(path) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        let black = real_sensor
            .black_level
            .unwrap_or_else(|| panic!("{path}: must carry BlackLevel"));
        let white = real_sensor
            .white_level
            .unwrap_or_else(|| panic!("{path}: must carry WhiteLevel"));
        assert!(
            u32::from(measured_min) < black,
            "{path}: the measured min must actually be below BlackLevel, or this proves nothing"
        );

        let mut sensor = minimal_sensor(1, 1);
        sensor.black_level = Some(black);
        sensor.white_level = Some(white);
        let src = [measured_min];
        let mut dst = [0u16; 1];
        develop_into(&sensor, &src, &mut dst).expect("fits");
        assert_eq!(
            dst[0], 0,
            "{path}: the measured below-BlackLevel minimum must clamp to 0"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — the three-stage crop, on the real measured geometry
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_three_stage_crop_produces_the_measured_dimensions() {
    // 8424x5632 -> ActiveArea 8392x5632 -> DefaultCrop 8368x5584.
    if let Some(sensor) = corpus_sensor("LEICA-Q2-MONO/L1021223.DNG") {
        assert_eq!(
            output_dimensions(&sensor).expect("real file geometry always fits"),
            (8368, 5584)
        );
    }
    // 5216x3472 -> (no ActiveArea) -> 5212x3468.
    if let Some(sensor) = corpus_sensor("LEICA-M-MONOCHROM/L1000622.DNG") {
        assert_eq!(
            sensor.active_area, None,
            "measured: ActiveArea is absent on this file"
        );
        assert_eq!(
            output_dimensions(&sensor).expect("real file geometry always fits"),
            (5212, 3468)
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — orientation is per-frame; the rotated file swaps output dimensions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn orientation_six_swaps_the_output_dimensions() {
    let Some(sensor) = corpus_sensor("LEICA-Q2-MONO/L1026016.DNG") else {
        return; // SKIP already announced
    };
    assert_eq!(
        sensor.orientation,
        Some(6),
        "measured: this frame is Rotate 90 CW"
    );
    assert_eq!(
        output_dimensions(&sensor).expect("real file geometry always fits"),
        (5584, 8368),
        "Orientation 6 must swap width and height"
    );
}

#[test]
fn an_unrotated_sibling_keeps_its_dimensions() {
    let Some(sensor) = corpus_sensor("LEICA-Q2-MONO/L1021223.DNG") else {
        return; // SKIP already announced
    };
    assert_eq!(
        sensor.orientation,
        Some(1),
        "measured: this frame is Horizontal (normal)"
    );
    assert_eq!(
        output_dimensions(&sensor).expect("real file geometry always fits"),
        (8368, 5584),
        "Orientation 1 must not swap width and height"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — hostile geometry is a typed error, never a panic
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hostile_geometry_does_not_panic() {
    let src = vec![0u16; 16]; // a plausible 4x4 raw plane, reused across cases

    // DefaultCropSize larger than ActiveArea.
    let mut sensor = minimal_sensor(4, 4);
    sensor.active_area = Some(ActiveArea {
        top: 0,
        left: 0,
        bottom: 4,
        right: 4,
    });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 10,
        height: 10,
    });
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ),
        "DefaultCropSize larger than ActiveArea must be a typed error"
    );

    // Crop origin outside the plane.
    let mut sensor = minimal_sensor(4, 4);
    sensor.default_crop_origin = Some(DefaultCropOrigin { x: 3, y: 3 });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 3,
        height: 3,
    });
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ),
        "a crop origin that leaves no room for its size must be a typed error"
    );

    // Zero dimensions.
    let mut sensor = minimal_sensor(4, 4);
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 0,
        height: 4,
    });
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ),
        "a zero-width crop must be a typed error"
    );

    // Absent tags altogether must NOT be an error — every default applies
    // and the whole plane develops.
    let sensor = minimal_sensor(4, 4);
    let mut dst = vec![0u16; 16];
    develop_into(&sensor, &src, &mut dst).expect("every tag absent still develops the whole plane");

    // Orientation outside 1..=8.
    let mut sensor = minimal_sensor(4, 4);
    sensor.orientation = Some(0);
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::UnsupportedOrientation { orientation: 0 })
        ),
        "Orientation 0 must be a typed error"
    );
    sensor.orientation = Some(9);
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::UnsupportedOrientation { orientation: 9 })
        ),
        "Orientation 9 must be a typed error"
    );

    // ActiveArea inverted / past the raw plane.
    let mut sensor = minimal_sensor(4, 4);
    sensor.active_area = Some(ActiveArea {
        top: 2,
        left: 0,
        bottom: 1,
        right: 4,
    });
    assert!(
        matches!(
            output_dimensions(&sensor),
            Err(Error::InvalidActiveArea { .. })
        ),
        "an inverted ActiveArea must be a typed error"
    );

    // BlackLevel >= WhiteLevel.
    let mut sensor = minimal_sensor(1, 1);
    sensor.black_level = Some(100);
    sensor.white_level = Some(100);
    let mut dst = [0u16; 1];
    assert!(
        matches!(
            develop_into(&sensor, &[0u16], &mut dst),
            Err(Error::InvalidLevels { .. })
        ),
        "BlackLevel == WhiteLevel must be a typed error, not a division by zero"
    );
}
