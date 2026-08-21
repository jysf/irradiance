//! `SPEC-003` — the TIFF/IFD reader, against real files and against hostile ones.
//!
//! Two lanes, and they fail for different reasons on purpose:
//!
//! - **`ifd_rejects_hostile_input`** and its neighbours run **everywhere**. They
//!   are hand-built byte fixtures (`tests/support/tiff.rs`), so a machine with
//!   no corpus still proves the guards.
//! - **`ifd_reaches_sensor_plane`** and the `exiftool` table need tier-B files,
//!   which are never committed (`DEC-003`). They go through `SPEC-002`'s reader
//!   — no hardcoded paths — and skip per-entry when a file is absent, with
//!   `just test` printing the corpus-status lines that make the skip visible
//!   before this suite runs.
//!
//! ## Where the expected values came from
//!
//! `exiftool 13.55`, run on 2026-08-20 against each corpus file, cross-checked
//! against an independent byte-level dump of the same IFDs. They are pinned as
//! literals here rather than shelled out to, because `SPEC-005` is the spec that
//! builds the *live* metadata oracle — this table is what `SPEC-003`'s
//! acceptance criterion 6 asks for, and it runs with no external tool installed.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tiff.rs"]
mod tiff;

use corpus::{CorpusFile, CorpusRoot, Manifest};
use irradiance::ifd::{
    ActiveArea, Container, DefaultCropOrigin, DefaultCropSize, Sensor, TAG_PHOTOMETRIC,
};
use irradiance::Error;

// ─────────────────────────────────────────────────────────────────────────────
// The exiftool table
// ─────────────────────────────────────────────────────────────────────────────

/// One corpus file's expected container reading.
struct Expected {
    /// Manifest path — the key into `SPEC-002`'s reader, never a real path.
    path: &'static str,
    /// `II` or `MM`, from the file's own header.
    big_endian: bool,
    /// Total IFDs the walk should reach (chain + SubIFDs).
    ifds: usize,
    /// Index of the sensor IFD in walk order.
    sensor_index: usize,
    width: u32,
    height: u32,
    bits: u32,
    compression: u32,
    black_level: Option<u32>,
    white_level: Option<u32>,
    active_area: Option<ActiveArea>,
    crop_origin: Option<DefaultCropOrigin>,
    crop_size: Option<DefaultCropSize>,
    orientation: Option<u32>,
    opcode_lists: [bool; 3],
    strip_byte_counts: u32,
    /// Tags present but shaped wrong. Only the Pentax DNG has one.
    malformed: &'static [u16],
}

const EXPECTED: &[Expected] = &[
    // ── Leica Q2 Monochrom: 14-bit, uncompressed, both opcode lists ─────────
    Expected {
        path: "LEICA-Q2-MONO/L1021223.DNG",
        big_endian: false,
        ifds: 4,
        sensor_index: 1,
        width: 8424,
        height: 5632,
        bits: 14,
        compression: 1,
        black_level: Some(512),
        white_level: Some(16383),
        active_area: Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        }),
        crop_origin: Some(DefaultCropOrigin { x: 12, y: 24 }),
        crop_size: Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        }),
        orientation: Some(1),
        opcode_lists: [true, false, true],
        strip_byte_counts: 83026944,
        malformed: &[],
    },
    // THE ROTATED FRAME. Same body, same firmware, different Orientation —
    // the pair that makes "Orientation is per-frame" a test rather than a note.
    Expected {
        path: "LEICA-Q2-MONO/L1026016.DNG",
        big_endian: false,
        ifds: 4,
        sensor_index: 1,
        width: 8424,
        height: 5632,
        bits: 14,
        compression: 1,
        black_level: Some(512),
        white_level: Some(16383),
        active_area: Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        }),
        crop_origin: Some(DefaultCropOrigin { x: 12, y: 24 }),
        crop_size: Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        }),
        orientation: Some(6),
        opcode_lists: [true, false, true],
        strip_byte_counts: 83026944,
        malformed: &[],
    },
    Expected {
        path: "LEICA-Q2-MONO/L1026192.DNG",
        big_endian: false,
        ifds: 4,
        sensor_index: 1,
        width: 8424,
        height: 5632,
        bits: 14,
        compression: 1,
        black_level: Some(512),
        white_level: Some(16383),
        active_area: Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        }),
        crop_origin: Some(DefaultCropOrigin { x: 12, y: 24 }),
        crop_size: Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        }),
        orientation: Some(1),
        opcode_lists: [true, false, true],
        strip_byte_counts: 83026944,
        malformed: &[],
    },
    // ── Leica M Monochrom: a THIRD bit depth, NO ActiveArea, no opcodes ─────
    Expected {
        path: "LEICA-M-MONOCHROM/L1000622.DNG",
        big_endian: false,
        ifds: 2,
        sensor_index: 1,
        width: 5216,
        height: 3472,
        bits: 16,
        compression: 1,
        black_level: Some(220),
        white_level: Some(16383),
        active_area: None,
        crop_origin: Some(DefaultCropOrigin { x: 2, y: 2 }),
        crop_size: Some(DefaultCropSize {
            width: 5212,
            height: 3468,
        }),
        orientation: Some(1),
        opcode_lists: [false, false, false],
        strip_byte_counts: 36219904,
        malformed: &[],
    },
    // ── The ONE big-endian file, and JPEG-compressed ────────────────────────
    Expected {
        path: "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
        big_endian: true,
        ifds: 2,
        sensor_index: 1,
        width: 5984,
        height: 4000,
        bits: 12,
        compression: 7,
        black_level: Some(0),
        white_level: Some(3750),
        active_area: None,
        crop_origin: Some(DefaultCropOrigin { x: 4, y: 4 }),
        crop_size: Some(DefaultCropSize {
            width: 5976,
            height: 3992,
        }),
        orientation: Some(1),
        opcode_lists: [false, false, false],
        strip_byte_counts: 21311750,
        malformed: &[],
    },
    // ── Pentax DNG: the malformed BlackLevelRepeatDim dnglab warns about ────
    Expected {
        path: "PENTAX-K3III-MONO/K3III.DNG",
        big_endian: false,
        ifds: 3,
        sensor_index: 1,
        width: 6304,
        height: 4224,
        bits: 14,
        compression: 7,
        black_level: Some(64),
        white_level: Some(16378),
        active_area: Some(ActiveArea {
            top: 34,
            left: 26,
            bottom: 4194,
            right: 6250,
        }),
        crop_origin: Some(DefaultCropOrigin { x: 28, y: 24 }),
        crop_size: Some(DefaultCropSize {
            width: 6192,
            height: 4128,
        }),
        orientation: Some(1),
        opcode_lists: [false, false, false],
        strip_byte_counts: 31047287,
        malformed: &[50713],
    },
    // ── Pentax PEF: no SubIFDs at all, a 3-IFD CHAIN, and the plane in IFD0
    //    with NO NewSubfileType tag — the file that makes TIFF's absent-means-0
    //    default load-bearing rather than decorative.
    Expected {
        path: "PENTAX-K3III-MONO/K3III.PEF",
        big_endian: false,
        ifds: 3,
        sensor_index: 0,
        width: 6304,
        height: 4224,
        bits: 14,
        compression: 65535,
        black_level: None,
        white_level: None,
        active_area: None,
        crop_origin: None,
        crop_size: None,
        orientation: Some(1),
        opcode_lists: [false, false, false],
        strip_byte_counts: 30916224,
        malformed: &[],
    },
];

/// Entry count is asserted against a literal, so adding a corpus file without
/// deciding what the reader should say about it is a failure, not a no-op.
const EXPECTED_FILES: usize = 7;

/// Read one corpus file's bytes, or `None` when it is absent (skip).
fn bytes_of(entry: &CorpusFile, root: &CorpusRoot) -> Option<Vec<u8>> {
    let path = entry.require(root)?;
    std::fs::read(&path).ok()
}

/// Pair each `EXPECTED` row with its manifest entry, failing loudly if the
/// manifest and this table have drifted apart.
fn manifest_pairs() -> (Manifest, CorpusRoot) {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    assert_eq!(
        EXPECTED.len(),
        EXPECTED_FILES,
        "EXPECTED table length changed — update EXPECTED_FILES"
    );
    for e in EXPECTED {
        assert!(
            manifest.get(e.path).is_some(),
            "{} is in this test's EXPECTED table but not in the manifest",
            e.path
        );
    }
    assert_eq!(
        manifest.files.len(),
        EXPECTED.len(),
        "the manifest has entries this test says nothing about — a corpus file \
         with no expected container reading is a file nothing checks"
    );
    (manifest, CorpusRoot::resolve())
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. the reader reaches the sensor plane on every corpus file that is present
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ifd_reaches_sensor_plane() {
    let (manifest, root) = manifest_pairs();
    let mut checked = 0;

    for expect in EXPECTED {
        let Some(entry) = manifest.get(expect.path) else {
            continue;
        };
        let Some(data) = bytes_of(entry, &root) else {
            continue;
        };

        let container = Container::parse(&data)
            .unwrap_or_else(|e| panic!("{}: container did not parse: {e}", expect.path));

        assert_eq!(
            container.byte_order() == irradiance::ifd::ByteOrder::Big,
            expect.big_endian,
            "{}: byte order",
            expect.path
        );
        assert_eq!(
            container.ifds().len(),
            expect.ifds,
            "{}: IFD count reached by the walk",
            expect.path
        );

        // The selection rule must identify EXACTLY one IFD. A rule that
        // happens to pick the right one because it picks the first of several
        // is not the rule doing the work — and on a Q2M, the runner-up would
        // be a full-resolution JPEG preview only 56 px narrower than the plane.
        let candidates = container.sensor_candidates();
        assert_eq!(
            candidates,
            vec![expect.sensor_index],
            "{}: sensor-IFD selection",
            expect.path
        );

        let sensor = container
            .sensor()
            .unwrap_or_else(|e| panic!("{}: no sensor plane: {e}", expect.path));
        assert_eq!(
            (sensor.width, sensor.height),
            (expect.width, expect.height),
            "{}: dimensions",
            expect.path
        );
        assert_eq!(sensor.samples_per_pixel, 1, "{}: samples", expect.path);
        assert_eq!(
            sensor.photometric, 34892,
            "{}: photometric must be LinearRaw",
            expect.path
        );
        checked += 1;
    }

    eprintln!("ifd_reaches_sensor_plane: {checked}/{EXPECTED_FILES} corpus files present");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. every tag matches exiftool
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tag_model_matches_exiftool() {
    let (manifest, root) = manifest_pairs();

    for expect in EXPECTED {
        let Some(entry) = manifest.get(expect.path) else {
            continue;
        };
        let Some(data) = bytes_of(entry, &root) else {
            continue;
        };
        let container = Container::parse(&data)
            .unwrap_or_else(|e| panic!("{}: container did not parse: {e}", expect.path));
        let s = container
            .sensor()
            .unwrap_or_else(|e| panic!("{}: no sensor plane: {e}", expect.path));

        let at = expect.path;
        assert_eq!(s.bits_per_sample, expect.bits, "{at}: BitsPerSample");
        assert_eq!(
            s.compression.code(),
            expect.compression,
            "{at}: Compression"
        );
        assert_eq!(s.black_level, expect.black_level, "{at}: BlackLevel");
        assert_eq!(s.white_level, expect.white_level, "{at}: WhiteLevel");
        assert_eq!(s.active_area, expect.active_area, "{at}: ActiveArea");
        assert_eq!(
            s.default_crop_origin, expect.crop_origin,
            "{at}: DefaultCropOrigin"
        );
        assert_eq!(
            s.default_crop_size, expect.crop_size,
            "{at}: DefaultCropSize"
        );
        assert_eq!(s.orientation, expect.orientation, "{at}: Orientation");
        assert_eq!(s.opcode_lists, expect.opcode_lists, "{at}: OpcodeList1/2/3");
        assert_eq!(
            s.strip_byte_counts,
            vec![expect.strip_byte_counts],
            "{at}: StripByteCounts"
        );
        assert_eq!(
            s.malformed_tags, expect.malformed,
            "{at}: tags present but shaped wrong"
        );
        // StripOffsets is read as a TAG here. Reading the strip it points at
        // is STAGE-002 — but it must at least point inside the file.
        for offset in &s.strip_offsets {
            assert!(
                (*offset as usize) < data.len(),
                "{at}: StripOffsets {offset} is outside the file"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2b. Orientation is read from the file every time, and absence is not zero
// ─────────────────────────────────────────────────────────────────────────────

/// One corpus file's sensor, or `None` when the file is absent (skip).
fn sensor_of(manifest: &Manifest, root: &CorpusRoot, path: &str) -> Option<Sensor> {
    let entry = manifest.get(path)?;
    let data = bytes_of(entry, root)?;
    Some(
        Container::parse(&data)
            .unwrap_or_else(|e| panic!("{path}: container did not parse: {e}"))
            .sensor()
            .unwrap_or_else(|e| panic!("{path}: no sensor plane: {e}")),
    )
}

/// `Orientation` is per-frame, not a camera constant (SPEC-004 acceptance
/// criterion 2): two frames from the SAME Leica Q2 Mono body disagree, and a
/// reader that hardcoded either value would pass on one frame and fail on
/// the other.
#[test]
fn orientation_is_per_frame() {
    let (manifest, root) = manifest_pairs();
    let unrotated = sensor_of(&manifest, &root, "LEICA-Q2-MONO/L1021223.DNG");
    let rotated = sensor_of(&manifest, &root, "LEICA-Q2-MONO/L1026016.DNG");
    let (Some(unrotated), Some(rotated)) = (unrotated, rotated) else {
        eprintln!(
            "orientation_is_per_frame: one or both Leica Q2 Mono frames are \
             absent from the corpus, skipping"
        );
        return;
    };

    assert_eq!(unrotated.orientation, Some(1), "L1021223.DNG");
    assert_eq!(rotated.orientation, Some(6), "L1026016.DNG");
    assert_ne!(
        unrotated.orientation, rotated.orientation,
        "same body, different frames — Orientation must be read from the \
         file every time, not hardcoded"
    );
}

/// An absent optional tag must read as `None`, never silently as a
/// present-and-zero value (SPEC-004 acceptance criterion 3) — TIFF's
/// absent-means-0 default exists for OTHER tags (`NewSubfileType`,
/// `SamplesPerPixel`), and conflating the two would make a real
/// `ActiveArea { top: 0, .. }` indistinguishable from "this tag was never
/// written".
#[test]
fn absent_tag_is_absent_not_zero() {
    // Synthetic proof, runs everywhere: ActiveArea absent must differ from
    // ActiveArea present-and-genuinely-zero.
    let absent_data = tiff::tiff(
        tiff::Order::Little,
        8,
        &[tiff::Ifd::new(
            8,
            tiff::sensor_entries(tiff::Order::Little),
            0,
        )],
    );
    let absent = Container::parse(&absent_data)
        .expect("parses")
        .sensor()
        .expect("has a sensor plane");
    assert_eq!(absent.active_area, None, "ActiveArea was never written");

    let mut zero_entries = tiff::sensor_entries(tiff::Order::Little);
    zero_entries.push(tiff::at_offset(irradiance::ifd::TAG_ACTIVE_AREA, 4, 4, 600));
    let mut zero_data = tiff::tiff(
        tiff::Order::Little,
        8,
        &[tiff::Ifd::new(8, zero_entries, 0)],
    );
    // 600..616 stay zero-filled by `resize` — four zeroed LONGs, i.e. an
    // ActiveArea that is PRESENT and genuinely (0, 0, 0, 0).
    zero_data.resize(616, 0);
    let zero = Container::parse(&zero_data)
        .expect("parses")
        .sensor()
        .expect("has a sensor plane");
    assert_eq!(
        zero.active_area,
        Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0
        }),
        "ActiveArea present and genuinely zero"
    );
    assert_ne!(absent.active_area, zero.active_area);

    // The real-world case, when the corpus is present: the M Monochrom
    // genuinely omits ActiveArea — no synthetic stand-in needed.
    let (manifest, root) = manifest_pairs();
    match sensor_of(&manifest, &root, "LEICA-M-MONOCHROM/L1000622.DNG") {
        Some(sensor) => assert_eq!(sensor.active_area, None, "M Monochrom has no ActiveArea"),
        None => eprintln!(
            "absent_tag_is_absent_not_zero: LEICA-M-MONOCHROM/L1000622.DNG is \
             absent from the corpus, skipping the real-file half"
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. compressed planes are rejected cleanly, not decoded and not panicked on
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ifd_rejects_compressed_planes_cleanly() {
    let (manifest, root) = manifest_pairs();

    for expect in EXPECTED {
        let Some(entry) = manifest.get(expect.path) else {
            continue;
        };
        let Some(data) = bytes_of(entry, &root) else {
            continue;
        };
        let container = Container::parse(&data).expect("parses");
        let sensor = container.sensor().expect("has a sensor plane");

        match sensor.require_uncompressed() {
            Ok(()) => assert_eq!(
                expect.compression, 1,
                "{}: accepted a compressed plane",
                expect.path
            ),
            Err(Error::UnsupportedCompression { compression }) => {
                assert_ne!(
                    expect.compression, 1,
                    "{}: rejected an uncompressed plane",
                    expect.path
                );
                assert_eq!(compression, expect.compression, "{}", expect.path);
                // The rejection must be the ONLY consequence: the tags of a
                // file this library cannot decode are still readable.
                assert_eq!(sensor.width, expect.width, "{}", expect.path);
            }
            Err(other) => panic!("{}: wrong error: {other}", expect.path),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. the layer-0 packing arithmetic closes on every plane we can unpack
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ifd_layer0_packing_closes_on_uncompressed_planes() {
    let (manifest, root) = manifest_pairs();

    for expect in EXPECTED {
        if expect.compression != 1 {
            continue;
        }
        let Some(entry) = manifest.get(expect.path) else {
            continue;
        };
        let Some(data) = bytes_of(entry, &root) else {
            continue;
        };
        let sensor = Container::parse(&data)
            .expect("parses")
            .sensor()
            .expect("has a sensor plane");

        // AGENTS.md §12 bar 3: needs no oracle tooling, no network, no corpus
        // beyond the file itself. width x height x bits == StripByteCounts x 8.
        let packed = sensor.packed_bits().expect("packing arithmetic");
        let declared = u64::from(expect.strip_byte_counts) * 8;
        assert_eq!(
            packed, declared,
            "{}: {} x {} x {} bits is {packed}, but StripByteCounts says {declared}",
            expect.path, sensor.width, sensor.height, sensor.bits_per_sample
        );
        assert_eq!(
            sensor.rows_per_strip,
            Some(expect.height),
            "{}",
            expect.path
        );
        assert_eq!(sensor.strip_offsets.len(), 1, "{}", expect.path);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. hostile input — runs everywhere, needs no corpus
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ifd_rejects_hostile_input() {
    // Every hostile shape must produce a typed Err, and the two valid ones
    // must still parse — a reader that rejects everything is not a reader.
    let valid = [
        "valid-sensor-ii",
        "valid-sensor-mm",
        "valid-subifd-ii",
        "valid-subifd-mm",
        // These are malformed in ways TIFF tolerates: the file parses, and
        // only the offending tag or lookup fails.
        //
        // `ifd0-inside-header` is deliberately NOT here. Pointing IFD0 at
        // offset 2 makes the reader decode the version word 42 as an entry
        // count, which then wants 504 bytes out of an 8-byte file — so it
        // fails as `Truncated`. That is the bounds check doing the work, with
        // no sanity rule about where an IFD is allowed to live.
        "unknown-field-type",
        "malformed-black-level-repeat-dim",
        // Malformed identifying tags (SPEC-004 FU-11): the WALK still
        // succeeds — only sensor SELECTION is affected, and that is a
        // separate, dedicated pair of tests below.
        "malformed-photometric-on-thumbnail",
        "malformed-photometric-on-only-candidate",
        "zero-entries",
        "no-ifd0",
        "payload-offset-past-eof",
        "count-overflow",
    ];

    for (name, data) in tiff::all() {
        let result = Container::parse(&data);
        if valid.contains(&name) {
            assert!(
                result.is_ok(),
                "{name}: expected this shape to parse, got {:?}",
                result.err()
            );
            continue;
        }
        let err = match result {
            Ok(_) => panic!("{name}: hostile input parsed as valid"),
            Err(e) => e,
        };
        // The point is not merely that it failed — it is that it failed with a
        // typed error naming what was wrong, and did not panic getting there.
        assert!(!err.to_string().is_empty(), "{name}: empty error message");
    }
}

/// Each guard is named, so a regression says which one stopped working.
#[test]
fn ifd_guards_each_fire_for_their_own_reason() {
    assert!(matches!(
        Container::parse(&tiff::self_referential_subifd()),
        Err(Error::CyclicIfd { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::subifd_cycle_two()),
        Err(Error::CyclicIfd { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::chain_cycle()),
        Err(Error::CyclicIfd { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::long_cycle()),
        Err(Error::CyclicIfd { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::deep_subifd_nest()),
        Err(Error::IfdDepthExceeded { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::bad_magic()),
        Err(Error::NotTiff { .. })
    ));
    assert!(matches!(
        Container::parse(&tiff::bad_version()),
        Err(Error::UnsupportedTiffVersion { .. })
    ));
    for truncated in [
        tiff::empty(),
        tiff::truncated_header(),
        tiff::truncated_ifd0_offset(),
        tiff::ifd0_past_eof(),
        tiff::entry_count_past_eof(),
    ] {
        assert!(matches!(
            Container::parse(&truncated),
            Err(Error::Truncated { .. })
        ));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5b. FU-11 — a malformed identifying tag costs the CANDIDATE, not the file
// ─────────────────────────────────────────────────────────────────────────────
//
// `is_sensor_ifd`'s three identifying tags are read over EVERY IFD, from
// `sensor_candidates`, `sensor_ifd` and `sensor` alike. DEC-012 / SPEC-004's
// FU-11: a malformed one must be skipped and recorded, never allowed to
// abort the scan of the OTHER IFDs — and the two hand-built fixtures below
// (`tests/support/tiff.rs`) are chosen so the malformed tag lands on
// different IFDs and the outcomes are DIFFERENT and both asserted.

/// The malformed tag is on an unrelated (thumbnail) IFD, and the real sensor
/// plane is a SubIFD elsewhere. It must still be reachable.
#[test]
fn malformed_on_thumbnail_does_not_lose_the_plane() {
    let data = tiff::malformed_photometric_on_thumbnail(tiff::Order::Little);
    let container =
        Container::parse(&data).expect("the walk itself is fine — DEC-012 walk vs interpret");
    let sensor = container
        .sensor()
        .expect("a malformed tag on the thumbnail must not hide the real plane (FU-11)");
    assert_eq!(sensor.ifd_index, 1);
    assert_eq!((sensor.width, sensor.height), (4, 2));
}

/// The malformed tag is on the file's ONLY candidate — the plane itself.
/// `sensor()` must fail with an error that SAYS a candidate was malformed,
/// not a bare `NoSensorIfd` indistinguishable from "this file has no raw
/// plane at all" — the obvious-looking fix (silently treat it as
/// `NotSensor`) is exactly what FU-11 forbids.
#[test]
fn malformed_on_the_sensor_ifd_is_reported_not_hidden() {
    let data = tiff::malformed_photometric_on_the_only_candidate(tiff::Order::Little);
    let container = Container::parse(&data).expect("the walk itself is fine");
    match container.sensor() {
        Err(Error::NoSensorIfdCandidatesMalformed { candidates }) => {
            assert_eq!(candidates, vec![(0, TAG_PHOTOMETRIC)]);
        }
        other => panic!("expected NoSensorIfdCandidatesMalformed naming the tag, got {other:?}"),
    }
    // Same discipline via sensor_ifd(): the caller who only needs the IFD,
    // not the typed Sensor, gets the same explained failure.
    assert!(matches!(
        container.sensor_ifd(),
        Err(Error::NoSensorIfdCandidatesMalformed { .. })
    ));
}

/// Every prefix of a valid container: the truncation sweep a fuzzer would find
/// eventually, run deterministically on every commit.
#[test]
fn ifd_survives_every_truncation_of_a_valid_container() {
    for source in [
        tiff::valid_subifd(tiff::Order::Little),
        tiff::valid_subifd(tiff::Order::Big),
    ] {
        for cut in 0..=source.len() {
            let prefix = source.get(..cut).unwrap_or(&[]);
            // Any answer is acceptable except a panic, and the harness catches
            // a panic as a test failure naming `cut`.
            if let Ok(container) = Container::parse(prefix) {
                let _ = container.sensor();
                for ifd in container.ifds() {
                    for entry in ifd.entries() {
                        let _ = container.payload(entry);
                        let _ = container.uints(entry);
                    }
                }
            }
        }
    }
}

/// A byte-flip sweep over a valid container. Same contract: no panic, ever.
#[test]
fn ifd_survives_single_byte_corruption() {
    let source = tiff::valid_subifd(tiff::Order::Little);
    for position in 0..source.len() {
        for patch in [0x00u8, 0x01, 0x7f, 0x80, 0xff] {
            let mut data = source.clone();
            if let Some(slot) = data.get_mut(position) {
                *slot = patch;
            }
            if let Ok(container) = Container::parse(&data) {
                let _ = container.sensor();
                for ifd in container.ifds() {
                    for entry in ifd.entries() {
                        let _ = container.payload(entry);
                        let _ = container.uints(entry);
                    }
                }
            }
        }
    }
}
