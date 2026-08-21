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
use irradiance::ifd::Container;
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
    active_area: Option<[u32; 4]>,
    crop_origin: Option<[u32; 2]>,
    crop_size: Option<[u32; 2]>,
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
        active_area: Some([0, 0, 5632, 8392]),
        crop_origin: Some([12, 24]),
        crop_size: Some([8368, 5584]),
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
        active_area: Some([0, 0, 5632, 8392]),
        crop_origin: Some([12, 24]),
        crop_size: Some([8368, 5584]),
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
        active_area: Some([0, 0, 5632, 8392]),
        crop_origin: Some([12, 24]),
        crop_size: Some([8368, 5584]),
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
        crop_origin: Some([2, 2]),
        crop_size: Some([5212, 3468]),
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
        crop_origin: Some([4, 4]),
        crop_size: Some([5976, 3992]),
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
        active_area: Some([34, 26, 4194, 6250]),
        crop_origin: Some([28, 24]),
        crop_size: Some([6192, 4128]),
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
        let candidates = container
            .sensor_candidates()
            .unwrap_or_else(|e| panic!("{}: candidate scan failed: {e}", expect.path));
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
fn ifd_tags_match_exiftool() {
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
