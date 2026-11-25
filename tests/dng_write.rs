//! `SPEC-061` tier A — the public mono `u16` DNG writer
//! (`irradiance::dng::write`, feature `write`) against synthetic, own-work
//! planes (`DEC-003`).
//!
//! - Oracle 1, the round trip (`round_trip_every_framing_and_geometry`):
//!   irradiance's own reader gets back the samples and every field `SPEC-061`
//!   §3/§4 fixes ([`dng_write_oracle::check_round_trip`]).
//! - The tag table, the backward-version rule and `Orientation`, walked on the
//!   raw IFD0 (`tags_match_the_spec_table` and its two siblings).
//! - Every refusal, typed, from both entry points (`every_refusal_is_typed`),
//!   the `OpcodeList1` allow-list, and the size query.
//! - Oracle 2, develop equivalence against a test-writer fixture of the source.
//! - Oracle 3, the red-proof of oracle 1, negative control first.
//! - Oracle 9, determinism over pre-filled buffers.
//! - `SPEC-081` oracles 1, 2, 6, 7 and 8: the EXIF block at its base, the
//!   fifteen metadata tags, the round trip with every metadata option set,
//!   the pinned base, determinism, and the new refusals in their order.
//!
//! Nothing is written to the repo.

#[path = "support/dng_write_fuzz.rs"]
mod dng_write_fuzz;
#[path = "support/dng_write_oracle.rs"]
mod dng_write_oracle;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/exif_block.rs"]
mod exif_block;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/opcode.rs"]
mod opcode;
#[path = "support/tiff.rs"]
mod tiff;

use dng_write_oracle::{check_round_trip, cut, expected, read_back};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::develop::{
    develop_into, develop_with_into, output_dimensions_with, DevelopOptions,
};
use irradiance::dng::write::{
    exif_block_base, write_into, write_to_vec, written_len, ExifBlock, Framing, JpegPreview,
    JpegPreviewColor, WriteInputError, WriteOptions, EXIF_BLOCK_BASE,
};
use irradiance::ifd::{
    ActiveArea, Compression, Container, DefaultCropOrigin, DefaultCropSize, PreviewKind, Rational,
};
use irradiance::{Error, ErrorKind};
use opcode::{
    opcode_list, raw_opcode, single_fix_bad_pixels_constant_list, single_warp_rectilinear_list,
};

const BUDGET: usize = 1 << 24;
const MODEL: &str = "irradiance test mono";
const FRAMINGS: [Framing; 3] = [
    Framing::FullSensor,
    Framing::FullActiveArea,
    Framing::StandardCrop,
];

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A deterministic plane with every sample distinct from its neighbours and,
/// for almost every index, from its own byte swap.
fn plane(width: u32, height: u32) -> Vec<u16> {
    (0..width * height)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 9) as u16 | 1)
        .collect()
}

/// A 20×14 plane's active area with a non-zero origin and a margin on all
/// four sides.
fn margin_area() -> ActiveArea {
    ActiveArea {
        top: 2,
        left: 3,
        bottom: 12,
        right: 17,
    }
}

/// Two masked strips in the 20×14 plane's margin: the top rows and the left
/// columns, neither touching `margin_area()`.
fn masked() -> [ActiveArea; 2] {
    [
        ActiveArea {
            top: 0,
            left: 0,
            bottom: 2,
            right: 20,
        },
        ActiveArea {
            top: 2,
            left: 0,
            bottom: 12,
            right: 3,
        },
    ]
}

fn list_1() -> Vec<u8> {
    single_fix_bad_pixels_constant_list(0, 0, 0)
}

/// An optional opcode the reader stores and no develop needs: enough for a
/// byte-for-byte round trip of list 2.
fn list_2() -> Vec<u8> {
    opcode_list(&[raw_opcode(99, 0x0103_0000, 1, &[1, 2, 3, 4, 5])])
}

/// An identity `WarpRectilinear`, the shape the Q2M's list 3 takes.
fn list_3() -> Vec<u8> {
    single_warp_rectilinear_list([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5, 1, 0)
}

fn base_options() -> WriteOptions<'static> {
    let mut o = WriteOptions::default();
    o.unique_camera_model = Some(MODEL);
    o
}

fn write(samples: &[u16], w: u32, h: u32, o: &WriteOptions<'_>) -> Vec<u8> {
    write_to_vec(samples, w, h, o, BUDGET).unwrap_or_else(|e| panic!("write failed: {e}"))
}

/// IFD0's raw entries: `(tag, type, count, payload bytes)`.
fn ifd0_entries(file: &[u8]) -> Vec<(u16, u16, u32, Vec<u8>)> {
    let container = Container::parse(file).expect("written file parses");
    let ifd = container.ifds().first().expect("IFD0");
    ifd.entries()
        .iter()
        .map(|e| {
            (
                e.tag(),
                e.field_type(),
                e.count(),
                container.payload(e).expect("payload").to_vec(),
            )
        })
        .collect()
}

fn le_u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&c| u32::from_le_bytes(c))
        .collect()
}

fn le_u16s(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_le_bytes(c))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — oracle 1
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn round_trip_every_framing_and_geometry() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    let masked = masked();
    let mut cases = 0;
    for framing in FRAMINGS {
        for active in [None, Some(margin_area())] {
            for with_lists in [false, true] {
                let mut o = base_options();
                o.framing = framing;
                o.active_area = active;
                o.black_level = &[64];
                o.white_level = 60000;
                o.orientation = 6;
                o.default_crop = Some((
                    DefaultCropOrigin { x: 1, y: 2 },
                    DefaultCropSize {
                        width: 9,
                        height: 5,
                    },
                ));
                if with_lists {
                    o.opcode_list_1 = Some(&l1);
                    o.opcode_list_2 = Some(&l2);
                    o.opcode_list_3 = Some(&l3);
                }
                if framing == Framing::FullSensor && active.is_some() {
                    o.masked_areas = &masked;
                }
                let file = write(&samples, w, h, &o);
                let want = expected(&samples, w, h, &o);
                if let Err(bad) = check_round_trip(&file, &want) {
                    panic!("{framing:?} active={active:?} lists={with_lists}: {bad:#?}");
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 12);
}

#[test]
fn round_trip_black_level_pattern() {
    let (w, h) = (8, 6);
    let samples = plane(w, h);
    let mut o = base_options();
    o.framing = Framing::FullSensor;
    o.black_level_repeat_dim = Some([2, 2]);
    o.black_level = &[10, 20, 30, 40];
    let file = write(&samples, w, h, &o);
    check_round_trip(&file, &expected(&samples, w, h, &o)).unwrap_or_else(|b| panic!("{b:#?}"));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — the tags
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tags_match_the_spec_table() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    let masked = masked();
    let mut o = base_options();
    o.framing = Framing::FullSensor;
    o.active_area = Some(margin_area());
    o.black_level_repeat_dim = Some([1, 1]);
    o.black_level = &[64];
    o.white_level = 60000;
    o.orientation = 3;
    o.masked_areas = &masked;
    o.opcode_list_1 = Some(&l1);
    o.opcode_list_2 = Some(&l2);
    o.opcode_list_3 = Some(&l3);
    let file = write(&samples, w, h, &o);
    assert_eq!(&file[..8], &[b'I', b'I', 42, 0, 8, 0, 0, 0]);
    let entries = ifd0_entries(&file);

    let (short, long, byte, ascii, undefined) = (3u16, 4u16, 1u16, 2u16, 7u16);
    let mut model = MODEL.as_bytes().to_vec();
    model.push(0);
    let aa = margin_area();
    let strip_offset = le_u32s(&entries[6].3)[0];
    let want: Vec<(u16, u16, u32, Vec<u8>)> = vec![
        (254, long, 1, 0u32.to_le_bytes().to_vec()),
        (256, long, 1, w.to_le_bytes().to_vec()),
        (257, long, 1, h.to_le_bytes().to_vec()),
        (258, short, 1, 16u16.to_le_bytes().to_vec()),
        (259, short, 1, 1u16.to_le_bytes().to_vec()),
        (262, short, 1, 34892u16.to_le_bytes().to_vec()),
        (273, long, 1, strip_offset.to_le_bytes().to_vec()),
        (274, short, 1, 3u16.to_le_bytes().to_vec()),
        (277, short, 1, 1u16.to_le_bytes().to_vec()),
        (278, long, 1, h.to_le_bytes().to_vec()),
        (279, long, 1, (w * h * 2).to_le_bytes().to_vec()),
        (284, short, 1, 1u16.to_le_bytes().to_vec()),
        (50706, byte, 4, vec![1, 4, 0, 0]),
        (50707, byte, 4, vec![1, 3, 0, 0]),
        (50708, ascii, model.len() as u32, model.clone()),
        (
            50713,
            short,
            2,
            [1u16, 1].iter().flat_map(|v| v.to_le_bytes()).collect(),
        ),
        (50714, long, 1, 64u32.to_le_bytes().to_vec()),
        (50717, long, 1, 60000u32.to_le_bytes().to_vec()),
        (
            50719,
            long,
            2,
            [0u32, 0].iter().flat_map(|v| v.to_le_bytes()).collect(),
        ),
        (
            50720,
            long,
            2,
            [aa.right - aa.left, aa.bottom - aa.top]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
        ),
        (
            50829,
            long,
            4,
            [aa.top, aa.left, aa.bottom, aa.right]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
        ),
        (
            50830,
            long,
            8,
            masked
                .iter()
                .flat_map(|m| [m.top, m.left, m.bottom, m.right])
                .flat_map(u32::to_le_bytes)
                .collect(),
        ),
        (51008, undefined, l1.len() as u32, l1.clone()),
        (51009, undefined, l2.len() as u32, l2.clone()),
        (51022, undefined, l3.len() as u32, l3.clone()),
    ];
    assert_eq!(entries, want);

    // The strip is the last thing in the file, at an even offset, and every
    // out-of-line value starts at an even offset.
    assert_eq!(strip_offset % 2, 0);
    assert_eq!(strip_offset as usize + (w * h * 2) as usize, file.len());
    let container = Container::parse(&file).unwrap();
    for e in container.ifds()[0].entries() {
        if e.byte_len().unwrap() > 4 {
            let raw = container.payload(e).unwrap().as_ptr() as usize - file.as_ptr() as usize;
            assert_eq!(raw % 2, 0, "tag {} value at odd offset {raw}", e.tag());
        }
    }
    // The inline SHORT is left-justified: the low half holds it (TIFF §2).
    let orientation_entry = 8 + 2 + 7 * 12;
    assert_eq!(
        &file[orientation_entry..orientation_entry + 2],
        &[0x12, 0x01]
    );
    assert_eq!(
        &file[orientation_entry + 8..orientation_entry + 12],
        &[3, 0, 0, 0]
    );
}

#[test]
fn cutting_framings_omit_active_and_masked_areas() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    for framing in [Framing::FullActiveArea, Framing::StandardCrop] {
        let mut o = base_options();
        o.framing = framing;
        o.active_area = Some(margin_area());
        let tags: Vec<u16> = ifd0_entries(&write(&samples, w, h, &o))
            .iter()
            .map(|e| e.0)
            .collect();
        assert!(
            !tags.contains(&50829) && !tags.contains(&50830),
            "{framing:?}: {tags:?}"
        );
        // Without levels or lists only the always-written tags remain.
        assert_eq!(
            tags,
            [
                254, 256, 257, 258, 259, 262, 273, 274, 277, 278, 279, 284, 50706, 50707, 50708,
                50717, 50719, 50720
            ]
        );
    }
}

#[test]
fn backward_version_follows_opcode_lists() {
    let (w, h) = (6, 4);
    let samples = plane(w, h);
    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    let backward = |o: &WriteOptions<'_>| {
        ifd0_entries(&write(&samples, w, h, o))
            .into_iter()
            .find(|e| e.0 == 50707)
            .map(|e| e.3)
            .expect("DNGBackwardVersion written")
    };
    let none = base_options();
    assert_eq!(backward(&none), [1, 1, 0, 0]);
    for i in 0..3 {
        let mut o = base_options();
        o.framing = Framing::FullSensor;
        match i {
            0 => o.opcode_list_1 = Some(&l1),
            1 => o.opcode_list_2 = Some(&l2),
            _ => o.opcode_list_3 = Some(&l3),
        }
        assert_eq!(backward(&o), [1, 3, 0, 0], "list {}", i + 1);
    }
    // The version is also what the reader sees.
    let (sensor, _) = read_back(&write(&samples, w, h, &none)).unwrap();
    assert_eq!(sensor.dng_version, Some([1, 4, 0, 0]));
    assert_eq!(sensor.dng_backward_version, Some([1, 1, 0, 0]));
}

#[test]
fn orientation_is_always_written() {
    let (w, h) = (6, 4);
    let samples = plane(w, h);
    let o = base_options();
    assert_eq!(o.orientation, 1, "the default is 1");
    let entries = ifd0_entries(&write(&samples, w, h, &o));
    let orientation = entries
        .iter()
        .find(|e| e.0 == 274)
        .expect("Orientation written");
    assert_eq!(le_u16s(&orientation.3), [1]);
    for value in 1..=8u16 {
        let mut o = base_options();
        o.orientation = value;
        let (sensor, _) = read_back(&write(&samples, w, h, &o)).unwrap();
        assert_eq!(sensor.orientation, Some(u32::from(value)));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 / AC5 — refusals and the size query
// ─────────────────────────────────────────────────────────────────────────────

/// Both entry points, the same error (compared by `Debug`, `Error` has no
/// `PartialEq`), and the kind.
fn assert_refused(
    name: &str,
    samples: &[u16],
    w: u32,
    h: u32,
    o: &WriteOptions<'_>,
    budget: usize,
    want: &Error,
) {
    let from_len = written_len(samples, w, h, o, budget).expect_err(name);
    let mut dst = vec![0u8; 64];
    let from_write = write_into(samples, w, h, o, budget, &mut dst).expect_err(name);
    let from_vec = write_to_vec(samples, w, h, o, budget).expect_err(name);
    for got in [&from_len, &from_write, &from_vec] {
        assert_eq!(format!("{got:?}"), format!("{want:?}"), "{name}");
        assert_eq!(got.kind(), want.kind(), "{name}");
        assert!(!got.to_string().is_empty());
    }
}

fn reason(r: WriteInputError) -> Error {
    Error::InvalidWriteInput(r)
}

#[test]
fn every_refusal_is_typed() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let good = base_options();
    let malformed_list = vec![0, 0, 0, 1, 0, 0]; // claims one opcode, holds two bytes
    let id5 = opcode_list(&[raw_opcode(5, 0x0103_0000, 0, &[0; 8])]);
    let masked = masked();
    let square = ActiveArea {
        top: 0,
        left: 0,
        bottom: 1,
        right: 1,
    };

    type Case<'a> = (&'static str, u32, u32, WriteOptions<'a>, usize, Error);
    let with = |f: &dyn Fn(&mut WriteOptions<'static>)| {
        let mut o = good.clone();
        f(&mut o);
        o
    };
    let long_model: &'static str = Box::leak("M".repeat(1025).into_boxed_str());
    let cases: Vec<Case<'_>> = vec![
        (
            "budget",
            w,
            h,
            good.clone(),
            279,
            Error::PlaneTooLarge {
                pixels: 280,
                limit: 279,
            },
        ),
        (
            "empty width",
            0,
            h,
            good.clone(),
            BUDGET,
            reason(WriteInputError::EmptyPlane),
        ),
        (
            "empty height",
            w,
            0,
            good.clone(),
            BUDGET,
            reason(WriteInputError::EmptyPlane),
        ),
        (
            "sample count",
            w,
            h - 1,
            good.clone(),
            BUDGET,
            reason(WriteInputError::SampleCountMismatch {
                expected: 260,
                actual: 280,
            }),
        ),
        (
            "active area inverted",
            w,
            h,
            with(&|o| {
                o.active_area = Some(ActiveArea {
                    top: 5,
                    left: 0,
                    bottom: 5,
                    right: 4,
                })
            }),
            BUDGET,
            reason(WriteInputError::ActiveAreaInvalid),
        ),
        (
            "active area outside",
            w,
            h,
            with(&|o| {
                o.active_area = Some(ActiveArea {
                    top: 0,
                    left: 0,
                    bottom: 15,
                    right: 20,
                })
            }),
            BUDGET,
            reason(WriteInputError::ActiveAreaInvalid),
        ),
        (
            "crop empty",
            w,
            h,
            with(&|o| {
                o.default_crop = Some((
                    DefaultCropOrigin { x: 0, y: 0 },
                    DefaultCropSize {
                        width: 0,
                        height: 3,
                    },
                ))
            }),
            BUDGET,
            reason(WriteInputError::DefaultCropInvalid),
        ),
        (
            "crop outside the active area",
            w,
            h,
            with(&|o| {
                o.active_area = Some(margin_area());
                o.default_crop = Some((
                    DefaultCropOrigin { x: 1, y: 0 },
                    DefaultCropSize {
                        width: 14,
                        height: 10,
                    },
                ))
            }),
            BUDGET,
            reason(WriteInputError::DefaultCropInvalid),
        ),
        (
            "masked area at a cutting framing",
            w,
            h,
            with(&|o| {
                o.active_area = Some(margin_area());
                o.masked_areas = &[ActiveArea {
                    top: 0,
                    left: 0,
                    bottom: 2,
                    right: 20,
                }];
            }),
            BUDGET,
            reason(WriteInputError::MaskedAreasNeedFullSensor),
        ),
        (
            "masked area empty",
            w,
            h,
            with(&|o| {
                o.framing = Framing::FullSensor;
                o.active_area = Some(margin_area());
                o.masked_areas = &[ActiveArea {
                    top: 0,
                    left: 0,
                    bottom: 0,
                    right: 20,
                }];
            }),
            BUDGET,
            reason(WriteInputError::MaskedAreaInvalid { index: 0 }),
        ),
        (
            "orientation 0",
            w,
            h,
            with(&|o| o.orientation = 0),
            BUDGET,
            reason(WriteInputError::OrientationInvalid { value: 0 }),
        ),
        (
            "orientation 9",
            w,
            h,
            with(&|o| o.orientation = 9),
            BUDGET,
            reason(WriteInputError::OrientationInvalid { value: 9 }),
        ),
        (
            "black level count",
            w,
            h,
            with(&|o| {
                o.black_level_repeat_dim = Some([2, 2]);
                o.black_level = &[1, 2, 3];
            }),
            BUDGET,
            reason(WriteInputError::BlackLevelCount {
                expected: 4,
                actual: 3,
            }),
        ),
        (
            "black level missing under a repeat dim",
            w,
            h,
            with(&|o| o.black_level_repeat_dim = Some([1, 1])),
            BUDGET,
            reason(WriteInputError::BlackLevelCount {
                expected: 1,
                actual: 0,
            }),
        ),
        (
            "two black levels without a repeat dim",
            w,
            h,
            with(&|o| o.black_level = &[1, 2]),
            BUDGET,
            reason(WriteInputError::BlackLevelCount {
                expected: 1,
                actual: 2,
            }),
        ),
        (
            "repeat dim zero",
            w,
            h,
            with(&|o| {
                o.black_level_repeat_dim = Some([0, 2]);
                o.black_level = &[];
            }),
            BUDGET,
            reason(WriteInputError::BlackLevelRepeatDimInvalid),
        ),
        (
            "model missing",
            w,
            h,
            with(&|o| o.unique_camera_model = None),
            BUDGET,
            reason(WriteInputError::MissingUniqueCameraModel),
        ),
        (
            "model empty",
            w,
            h,
            with(&|o| o.unique_camera_model = Some("")),
            BUDGET,
            reason(WriteInputError::MissingUniqueCameraModel),
        ),
        (
            "model blank",
            w,
            h,
            with(&|o| o.unique_camera_model = Some("   ")),
            BUDGET,
            reason(WriteInputError::MissingUniqueCameraModel),
        ),
        (
            "model not printable",
            w,
            h,
            with(&|o| o.unique_camera_model = Some("Caf\u{e9}")),
            BUDGET,
            reason(WriteInputError::InvalidAscii {
                field: "UniqueCameraModel",
            }),
        ),
        (
            "model with a control byte",
            w,
            h,
            with(&|o| o.unique_camera_model = Some("A\nB")),
            BUDGET,
            reason(WriteInputError::InvalidAscii {
                field: "UniqueCameraModel",
            }),
        ),
        (
            "model too long",
            w,
            h,
            with(&|o| o.unique_camera_model = Some(long_model)),
            BUDGET,
            reason(WriteInputError::InvalidAscii {
                field: "UniqueCameraModel",
            }),
        ),
    ];
    let mut n = 0;
    for (name, cw, ch, o, budget, want) in &cases {
        assert_refused(name, &samples, *cw, *ch, o, *budget, want);
        n += 1;
    }

    // Masked areas: overlapping the active area, outside the plane, and an
    // active area of None (the whole plane is active, so any masked area
    // overlaps). Index names the offending rectangle.
    let overlapping = [masked[0], margin_area()];
    let outside = [ActiveArea {
        top: 0,
        left: 18,
        bottom: 2,
        right: 21,
    }];
    for (label, list, active, index) in [
        ("overlaps", &overlapping[..], Some(margin_area()), 1),
        ("outside", &outside[..], Some(margin_area()), 0),
        ("no active area", &[square][..], None, 0),
    ] {
        let mut o = good.clone();
        o.framing = Framing::FullSensor;
        o.active_area = active;
        o.masked_areas = list;
        assert_refused(
            label,
            &samples,
            w,
            h,
            &o,
            BUDGET,
            &reason(WriteInputError::MaskedAreaInvalid { index }),
        );
        n += 1;
    }

    // OpcodeList1: malformed (CallerError) and outside the allow-list
    // (Unsupported), at a cutting framing.
    let mut o = good.clone();
    o.opcode_list_1 = Some(&malformed_list);
    assert_refused(
        "malformed list 1",
        &samples,
        w,
        h,
        &o,
        BUDGET,
        &reason(WriteInputError::MalformedOpcodeList { list: 1 }),
    );
    o.opcode_list_1 = Some(&id5);
    assert_refused(
        "list 1 id 5",
        &samples,
        w,
        h,
        &o,
        BUDGET,
        &Error::UnsupportedOpcodeForFraming {
            list: 1,
            opcode_id: 5,
            framing: Framing::StandardCrop,
        },
    );
    n += 2;

    // OutputTooLarge: a 4 GiB OpcodeList2 over a tiny plane. `vec![0; n]`
    // asks the allocator for zeroed memory, which the OS maps lazily, and
    // planning never reads list 2's bytes, so this costs no real memory.
    let huge = vec![0u8; u32::MAX as usize];
    let mut o = good.clone();
    o.opcode_list_2 = Some(&huge);
    assert_refused(
        "output too large",
        &samples,
        w,
        h,
        &o,
        BUDGET,
        &reason(WriteInputError::OutputTooLarge),
    );
    drop(huge);
    n += 1;

    // SPEC-081 §4: the new reasons, one case each (and one per descriptive
    // field for InvalidAscii).
    let block: &'static [u8] = &[0u8; 8];
    let meta = |f: &dyn Fn(&mut WriteOptions<'_>)| {
        let mut o = good.clone();
        f(&mut o);
        o
    };
    let long_text: &'static str = Box::leak("a".repeat(1025).into_boxed_str());
    type Set = fn(&mut WriteOptions<'static>, &'static str);
    let fields: [(&str, Set); 8] = [
        ("Make", |o, v| o.make = Some(v)),
        ("Model", |o, v| o.model = Some(v)),
        ("Software", |o, v| o.software = Some(v)),
        ("DateTime", |o, v| o.date_time = Some(v)),
        ("Artist", |o, v| o.artist = Some(v)),
        ("Copyright", |o, v| o.copyright = Some(v)),
        ("LocalizedCameraModel", |o, v| {
            o.localized_camera_model = Some(v)
        }),
        ("CameraSerialNumber", |o, v| {
            o.camera_serial_number = Some(v)
        }),
    ];
    for (field, set) in fields {
        for bad in ["caf\u{e9}", "tab\there", long_text] {
            let mut o = good.clone();
            set(&mut o, bad);
            assert_refused(
                field,
                &samples,
                w,
                h,
                &o,
                BUDGET,
                &reason(WriteInputError::InvalidAscii { field }),
            );
            n += 1;
        }
    }
    let half = Rational { num: 1, den: 2 };
    let zero_den = Rational { num: 1, den: 0 };
    let new_cases: Vec<(&str, WriteOptions<'_>, Error)> = vec![
        (
            "date time form",
            meta(&|o| o.date_time = Some("2026-11-24 10:00:00")),
            reason(WriteInputError::DateTimeInvalid),
        ),
        (
            "baseline exposure zero denominator",
            meta(&|o| o.baseline_exposure = Some(zero_den)),
            reason(WriteInputError::RationalInvalid {
                field: "BaselineExposure",
            }),
        ),
        (
            "baseline noise negative",
            meta(&|o| o.baseline_noise = Some(Rational { num: -1, den: 2 })),
            reason(WriteInputError::RationalInvalid {
                field: "BaselineNoise",
            }),
        ),
        (
            "baseline sharpness zero denominator",
            meta(&|o| o.baseline_sharpness = Some(zero_den)),
            reason(WriteInputError::RationalInvalid {
                field: "BaselineSharpness",
            }),
        ),
        (
            "linear response limit negative denominator",
            meta(&|o| o.linear_response_limit = Some(Rational { num: 1, den: -2 })),
            reason(WriteInputError::RationalInvalid {
                field: "LinearResponseLimit",
            }),
        ),
        (
            "exif block without an ifd",
            meta(&|o| {
                let mut b = ExifBlock::default();
                b.bytes = block;
                o.exif_block = Some(b);
            }),
            reason(WriteInputError::ExifBlockWithoutIfd),
        ),
        (
            "exif ifd offset odd",
            meta(&|o| {
                let mut b = ExifBlock::default();
                b.bytes = block;
                b.exif_ifd = Some(1);
                o.exif_block = Some(b);
            }),
            reason(WriteInputError::ExifIfdOffsetInvalid {
                ifd: "Exif",
                offset: 1,
            }),
        ),
        (
            // offset + 2 == len + 1: one byte short of the entry count.
            "gps ifd offset past the block",
            meta(&|o| {
                let mut b = ExifBlock::default();
                b.bytes = &block[..7];
                b.gps_ifd = Some(6);
                o.exif_block = Some(b);
            }),
            reason(WriteInputError::ExifIfdOffsetInvalid {
                ifd: "GPS",
                offset: 6,
            }),
        ),
        (
            "gps ifd offset odd beside a good exif offset",
            meta(&|o| {
                let mut b = ExifBlock::default();
                b.bytes = block;
                b.exif_ifd = Some(0);
                b.gps_ifd = Some(3);
                o.exif_block = Some(b);
            }),
            reason(WriteInputError::ExifIfdOffsetInvalid {
                ifd: "GPS",
                offset: 3,
            }),
        ),
        (
            "empty xmp",
            meta(&|o| o.xmp = Some(&[])),
            reason(WriteInputError::EmptyXmp),
        ),
        // SPEC-082 §4: the preview's two reasons.
        (
            "preview empty",
            meta(&|o| o.preview = Some(preview(&[], JpegPreviewColor::YCbCr))),
            reason(WriteInputError::PreviewNotJpeg),
        ),
        (
            "preview starts FF D9",
            meta(&|o| o.preview = Some(preview(&[0xFF, 0xD9, 0xFF, 0xD8], JpegPreviewColor::Gray))),
            reason(WriteInputError::PreviewNotJpeg),
        ),
        (
            "preview a single FF",
            meta(&|o| o.preview = Some(preview(&[0xFF], JpegPreviewColor::YCbCr))),
            reason(WriteInputError::PreviewNotJpeg),
        ),
        (
            "preview width 0",
            meta(&|o| {
                let mut p = preview(JPEG_EVEN, JpegPreviewColor::YCbCr);
                p.width = 0;
                o.preview = Some(p);
            }),
            reason(WriteInputError::PreviewDimensionsInvalid),
        ),
        (
            "preview height 65536",
            meta(&|o| {
                let mut p = preview(JPEG_EVEN, JpegPreviewColor::Gray);
                p.height = 65536;
                o.preview = Some(p);
            }),
            reason(WriteInputError::PreviewDimensionsInvalid),
        ),
    ];
    for (name, o, want) in &new_cases {
        assert_refused(name, &samples, w, h, o, BUDGET, want);
        assert_eq!(want.kind(), ErrorKind::CallerError, "{name}");
        n += 1;
    }
    // The boundaries that pass: offset + 2 == len, and BaselineExposure of
    // either sign.
    let mut b = ExifBlock::default();
    b.bytes = block;
    b.gps_ifd = Some(6);
    let ok = meta(&|o| {
        o.exif_block = Some(b);
        o.baseline_exposure = Some(Rational { num: -1, den: 2 });
        o.baseline_noise = Some(half);
    });
    assert!(written_len(&samples, w, h, &ok, BUDGET).is_ok());

    // OutputBufferWrongLength: write_into only.
    let len = written_len(&samples, w, h, &good, BUDGET).unwrap();
    let mut short = vec![0u8; len - 1];
    let err = write_into(&samples, w, h, &good, BUDGET, &mut short).unwrap_err();
    assert_eq!(
        format!("{err:?}"),
        format!(
            "{:?}",
            reason(WriteInputError::OutputBufferWrongLength {
                expected: len,
                actual: len - 1
            })
        )
    );
    assert_eq!(err.kind(), ErrorKind::CallerError);
    n += 1;
    assert_eq!(n, 29 + 24 + 10 + 5);
}

#[test]
fn checks_run_in_the_documented_order() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    // Everything wrong at once: each refusal must be the earliest one.
    let id5 = opcode_list(&[raw_opcode(5, 0x0103_0000, 0, &[0; 8])]);
    let inside = [margin_area()];
    let mut o = base_options();
    o.unique_camera_model = None;
    o.orientation = 0;
    o.black_level = &[1, 2];
    o.masked_areas = &inside;
    o.default_crop = Some((
        DefaultCropOrigin { x: 0, y: 0 },
        DefaultCropSize {
            width: 0,
            height: 0,
        },
    ));
    o.active_area = Some(ActiveArea {
        top: 9,
        left: 0,
        bottom: 3,
        right: 1,
    });
    o.opcode_list_1 = Some(&id5);
    // SPEC-081 §4: every new check fails too, between UniqueCameraModel and
    // the OpcodeList1 walk, each in its documented place.
    let block: &'static [u8] = &[0u8; 8];
    let mut bad_block = ExifBlock::default();
    bad_block.bytes = block;
    o.make = Some("tab\t");
    o.copyright = Some("caf\u{e9}");
    o.date_time = Some("2026:11:24T10:00:00");
    o.baseline_exposure = Some(Rational { num: 1, den: 0 });
    o.linear_response_limit = Some(Rational { num: -1, den: 1 });
    o.exif_block = Some(bad_block);
    o.xmp = Some(&[]);
    // SPEC-082 §4: a preview that fails both of its checks, after EmptyXmp.
    let mut bad_preview = preview(&[0xFF], JpegPreviewColor::YCbCr);
    bad_preview.width = 0;
    o.preview = Some(bad_preview);
    let mut odd_block = bad_block;
    odd_block.exif_ifd = Some(1);
    type Fix = Box<dyn Fn(&mut WriteOptions<'_>)>;
    let steps: Vec<(Fix, &str)> = vec![
        (Box::new(|_| {}), "ActiveAreaInvalid"),
        (Box::new(|o| o.active_area = None), "DefaultCropInvalid"),
        (
            Box::new(|o| o.default_crop = None),
            "MaskedAreasNeedFullSensor",
        ),
        (Box::new(|o| o.masked_areas = &[]), "BlackLevelCount"),
        (Box::new(|o| o.black_level = &[]), "OrientationInvalid"),
        (Box::new(|o| o.orientation = 1), "MissingUniqueCameraModel"),
        (
            Box::new(|o| o.unique_camera_model = Some(MODEL)),
            "InvalidAscii { field: \"Make\" }",
        ),
        (Box::new(|o| o.make = None), "DateTimeInvalid"),
        (
            Box::new(|o| o.date_time = None),
            "InvalidAscii { field: \"Copyright\" }",
        ),
        (
            Box::new(|o| o.copyright = None),
            "RationalInvalid { field: \"BaselineExposure\" }",
        ),
        (
            Box::new(|o| o.baseline_exposure = None),
            "RationalInvalid { field: \"LinearResponseLimit\" }",
        ),
        (
            Box::new(|o| o.linear_response_limit = None),
            "ExifBlockWithoutIfd",
        ),
        (
            Box::new(move |o| o.exif_block = Some(odd_block)),
            "ExifIfdOffsetInvalid",
        ),
        (Box::new(|o| o.exif_block = None), "EmptyXmp"),
        (Box::new(|o| o.xmp = None), "PreviewNotJpeg"),
        (
            Box::new(|o| {
                if let Some(p) = &mut o.preview {
                    p.jpeg = JPEG_EVEN;
                }
            }),
            "PreviewDimensionsInvalid",
        ),
        (
            Box::new(|o| {
                if let Some(p) = &mut o.preview {
                    p.width = 1;
                }
            }),
            "UnsupportedOpcodeForFraming",
        ),
    ];
    // The budget and the sample count come before all of them.
    let err = written_len(&samples, w, h, &o, 1).unwrap_err();
    assert!(matches!(err, Error::PlaneTooLarge { .. }), "{err:?}");
    let err = written_len(&samples[1..], w, h, &o, BUDGET).unwrap_err();
    assert!(
        format!("{err:?}").contains("SampleCountMismatch"),
        "{err:?}"
    );
    for (fix, want) in steps {
        fix(&mut o);
        // Both entry points report the same, earliest refusal.
        let err = written_len(&samples, w, h, &o, BUDGET).unwrap_err();
        assert!(
            format!("{err:?}").contains(want),
            "want {want}, got {err:?}"
        );
        let again = write_into(&samples, w, h, &o, BUDGET, &mut []).unwrap_err();
        assert_eq!(format!("{err:?}"), format!("{again:?}"));
    }
    o.opcode_list_1 = None;
    assert!(written_len(&samples, w, h, &o, BUDGET).is_ok());
}

#[test]
fn opcode_list_1_allow_list_under_cutting_framings() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let fix_constant = list_1();
    let fix_list = opcode_list(&[raw_opcode(5, 0x0103_0000, 0, &[0; 8])]);
    // FixBadPixelsConstant followed by FixBadPixelsList: the first unlisted id
    // is reported.
    let mixed = opcode_list(&[
        raw_opcode(4, 0x0103_0000, 0, &[0; 8]),
        raw_opcode(5, 0x0103_0000, 0, &[0; 8]),
    ]);
    for framing in FRAMINGS {
        let mut o = base_options();
        o.framing = framing;
        o.active_area = Some(margin_area());
        o.opcode_list_1 = Some(&fix_constant);
        assert!(
            written_len(&samples, w, h, &o, BUDGET).is_ok(),
            "{framing:?} id 4"
        );
        for list in [&fix_list, &mixed] {
            o.opcode_list_1 = Some(list);
            let got = written_len(&samples, w, h, &o, BUDGET);
            if framing == Framing::FullSensor {
                let file = write(&samples, w, h, &o);
                check_round_trip(&file, &expected(&samples, w, h, &o))
                    .unwrap_or_else(|b| panic!("{b:#?}"));
            } else {
                let err = got.unwrap_err();
                assert_eq!(
                    format!("{err:?}"),
                    format!(
                        "{:?}",
                        Error::UnsupportedOpcodeForFraming {
                            list: 1,
                            opcode_id: 5,
                            framing
                        }
                    )
                );
                assert_eq!(err.kind(), ErrorKind::Unsupported);
            }
        }
    }
    // Lists 2 and 3 are never walked: garbage passes at every framing.
    for framing in FRAMINGS {
        let mut o = base_options();
        o.framing = framing;
        o.opcode_list_2 = Some(&[0xFF; 3]);
        o.opcode_list_3 = Some(&[0xFF; 7]);
        assert!(
            written_len(&samples, w, h, &o, BUDGET).is_ok(),
            "{framing:?}"
        );
    }
    // An empty list 1 has no count word: malformed under a cut.
    let mut o = base_options();
    o.opcode_list_1 = Some(&[]);
    let err = written_len(&samples, w, h, &o, BUDGET).unwrap_err();
    assert!(
        format!("{err:?}").contains("MalformedOpcodeList"),
        "{err:?}"
    );
    // A list of zero opcodes is well-formed.
    o.opcode_list_1 = Some(&[0, 0, 0, 0]);
    assert!(written_len(&samples, w, h, &o, BUDGET).is_ok());
    // Verify-added (HANDOFF-190 mutant M11): the walk must account for EVERY
    // byte, so a well-formed opcode followed by stray bytes is malformed too,
    // not only a truncated one.
    for extra in [&[0u8][..], &[0, 0, 0, 0]] {
        let mut trailing = fix_constant.clone();
        trailing.extend_from_slice(extra);
        let mut o = base_options(); // StandardCrop: a cut, so list 1 is walked
        o.opcode_list_1 = Some(&trailing);
        let err = written_len(&samples, w, h, &o, BUDGET).unwrap_err();
        assert_eq!(
            format!("{err:?}"),
            format!(
                "{:?}",
                Error::InvalidWriteInput(WriteInputError::MalformedOpcodeList { list: 1 })
            ),
            "{} trailing byte(s)",
            extra.len()
        );
    }
}

/// Verify-added (HANDOFF-190 mutant M10): the budget refuses only
/// `width × height > max_pixels`, so a plane of exactly `max_pixels` writes.
#[test]
fn a_plane_of_exactly_max_pixels_is_within_the_budget() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let o = base_options();
    let exact = (w * h) as usize;
    assert!(written_len(&samples, w, h, &o, exact).is_ok());
    assert!(write_to_vec(&samples, w, h, &o, exact).is_ok());
    let err = written_len(&samples, w, h, &o, exact - 1).unwrap_err();
    assert!(matches!(err, Error::PlaneTooLarge { .. }), "{err:?}");
}

#[test]
fn size_query_matches_write() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    for framing in FRAMINGS {
        for with_lists in [false, true] {
            let mut o = base_options();
            o.framing = framing;
            o.active_area = Some(margin_area());
            if with_lists {
                o.opcode_list_1 = Some(&l1);
                o.opcode_list_2 = Some(&l2);
                o.opcode_list_3 = Some(&l3);
            }
            let len = written_len(&samples, w, h, &o, BUDGET).unwrap();
            for wrong in [len - 1, len + 1] {
                let mut dst = vec![0u8; wrong];
                let err = write_into(&samples, w, h, &o, BUDGET, &mut dst).unwrap_err();
                assert!(
                    format!("{err:?}").contains("OutputBufferWrongLength"),
                    "{framing:?} {wrong}: {err:?}"
                );
            }
            let mut dst = vec![0u8; len];
            write_into(&samples, w, h, &o, BUDGET, &mut dst).unwrap();
            let vec = write_to_vec(&samples, w, h, &o, BUDGET).unwrap();
            assert_eq!(vec.len(), len);
            assert_eq!(vec, dst);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — oracle 2, develop equivalence
// ─────────────────────────────────────────────────────────────────────────────

fn develop(file: &[u8], options: &DevelopOptions) -> Result<Vec<u16>, String> {
    let (sensor, plane) = read_back(file).map_err(|e| e.to_string())?;
    let (ow, oh) = output_dimensions_with(&sensor, options).map_err(|e| e.to_string())?;
    let mut dst = vec![0u16; ow as usize * oh as usize];
    develop_with_into(&sensor, &plane, &mut dst, options).map_err(|e| format!("{e:?}"))?;
    Ok(dst)
}

fn full_active_area() -> DevelopOptions {
    let mut o = DevelopOptions::default();
    o.framing = irradiance::develop::Framing::FullActiveArea;
    o
}

#[test]
fn develop_equivalence_synthetic() {
    let (w, h) = (20, 14);
    let mut samples: Vec<u16> = plane(w, h).iter().map(|s| 100 + s % 3000).collect();
    // Two bad pixels inside the active area, away from its edge, so the
    // FixBadPixelsConstant median sees the same neighbours before and after a
    // cut. (A bad pixel ON the edge borrows margin neighbours in the source,
    // SPEC-017 rule 1 of `apply_fix_bad_pixels_constant`, and the cut file would not: see the handback.)
    samples[(5 * w + 8) as usize] = 0;
    samples[(9 * w + 12) as usize] = 0;
    let aa = margin_area();
    let list_1 = list_1();
    let source = write_dng(&DngDescription {
        order: tiff::Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width: w,
        height: h,
        bits_per_sample: 16,
        kind: PlaneKind::LinearRaw,
        organisation: Organisation::Strips { rows_per_strip: 4 },
        payload: Payload::Samples(samples.clone()),
        meta: DngMeta {
            black_level: Some(vec![100]),
            white_level: Some(vec![4000]),
            active_area: Some([aa.top, aa.left, aa.bottom, aa.right]),
            default_crop_origin: Some([2, 1]),
            default_crop_size: Some([9, 7]),
            orientation: Some(6),
            ..DngMeta::default()
        },
        opcode_lists: [Some(list_1.clone()), None, None],
        extra_tags: Vec::new(),
        fault: None,
    })
    .unwrap();
    let (sensor, plane) = read_back(&source).unwrap();
    assert_eq!(plane, samples);
    let want_crop = develop(&source, &DevelopOptions::default()).unwrap();
    let want_full = develop(&source, &full_active_area()).unwrap();
    assert_ne!(want_crop.len(), want_full.len());

    for framing in FRAMINGS {
        let o = dng_write_oracle::options_from_sensor(&sensor, framing, MODEL);
        let written = write(&plane, w, h, &o);
        let got = develop(&written, &DevelopOptions::default()).unwrap();
        let want = if framing == Framing::StandardCrop {
            &want_crop
        } else {
            &want_full
        };
        assert_eq!(&got, want, "{framing:?}");
    }
    // And the plain entry point agrees with the options one.
    let o = dng_write_oracle::options_from_sensor(&sensor, Framing::StandardCrop, MODEL);
    let (s2, p2) = read_back(&write(&plane, w, h, &o)).unwrap();
    let mut dst = vec![0u16; want_crop.len()];
    develop_into(&s2, &p2, &mut dst).unwrap();
    assert_eq!(dst, want_crop);
}

/// Pins a known limit of `DEC-075` §5, found at the SPEC-061 build and
/// confirmed at verify (HANDOFF-190 item 4). `FixBadPixelsConstant` names no
/// coordinates, but its effect is not position-independent: a bad pixel ON
/// the active-area edge takes its median from neighbours across the whole
/// stored plane (SPEC-017 rule 1), margin included, and a cut removes those.
/// The cut file is a valid DNG; it develops differently at that one pixel.
///
/// This test asserts the divergence as it stands, and that it is confined to
/// the edge pixel. If a later spec refuses or flags such input, or `DEC-075`
/// changes the rule, this test is the one to rewrite.
#[test]
fn fix_bad_pixels_constant_on_the_active_area_edge_develops_differently_after_a_cut() {
    let (w, h) = (20, 14);
    let aa = margin_area();
    // The margin reads bright, the active area dark, so the median of a
    // corner pixel's neighbours depends on whether the margin is there.
    let mut samples: Vec<u16> = (0..h)
        .flat_map(|y| {
            (0..w).map(move |x| {
                let inside = (aa.top..aa.bottom).contains(&y) && (aa.left..aa.right).contains(&x);
                if inside {
                    200
                } else {
                    3000
                }
            })
        })
        .collect();
    // The active area's top-left corner: five of its eight neighbours are in
    // the margin.
    let corner = (aa.top * w + aa.left) as usize;
    samples[corner] = 0;
    let l1 = list_1(); // FixBadPixelsConstant, constant 0

    let mut o = base_options();
    o.active_area = Some(aa);
    o.white_level = 4000;
    o.opcode_list_1 = Some(&l1);

    o.framing = Framing::FullSensor;
    let full_sensor = develop(&write(&samples, w, h, &o), &full_active_area()).unwrap();
    o.framing = Framing::FullActiveArea;
    let cut_file = develop(&write(&samples, w, h, &o), &full_active_area()).unwrap();

    assert_eq!(full_sensor.len(), cut_file.len());
    let differing: Vec<usize> = full_sensor
        .iter()
        .zip(&cut_file)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    // Output index 0 is the active area's top-left corner.
    assert_eq!(differing, [0], "only the edge pixel differs");
    assert!(
        full_sensor[0] > cut_file[0],
        "the source's median took the bright margin, the cut file's could not"
    );

    // Negative control: the same bad pixel one step inside the edge has all
    // eight neighbours in the active area, so the cut changes nothing.
    samples[corner] = 200;
    samples[corner + w as usize + 1] = 0;
    o.framing = Framing::FullSensor;
    let full_sensor = develop(&write(&samples, w, h, &o), &full_active_area()).unwrap();
    o.framing = Framing::FullActiveArea;
    let cut_file = develop(&write(&samples, w, h, &o), &full_active_area()).unwrap();
    assert_eq!(full_sensor, cut_file);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — oracle 3, the red-proof of oracle 1
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn red_proof_round_trip_catches_a_flipped_sample_and_a_moved_crop() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let mut o = base_options();
    o.framing = Framing::StandardCrop;
    o.active_area = Some(margin_area());
    o.default_crop = Some((
        DefaultCropOrigin { x: 1, y: 2 },
        DefaultCropSize {
            width: 9,
            height: 5,
        },
    ));
    let file = write(&samples, w, h, &o);
    let want = expected(&samples, w, h, &o);

    // Negative control first: the honest output passes.
    check_round_trip(&file, &want).unwrap_or_else(|b| panic!("honest output failed: {b:#?}"));

    // One sample byte flipped.
    let entries = ifd0_entries(&file);
    let strip = le_u32s(&entries.iter().find(|e| e.0 == 273).unwrap().3)[0] as usize;
    let mut flipped = file.clone();
    flipped[strip + 7] ^= 0x01;
    let bad = check_round_trip(&flipped, &want).expect_err("a flipped sample must be caught");
    assert!(bad.iter().any(|b| b.starts_with("samples")), "{bad:?}");

    // DefaultCropOrigin rewritten in a copy: find its value's offset in the
    // entry table and change x.
    let container = Container::parse(&file).unwrap();
    let crop = container.ifds()[0]
        .entries()
        .iter()
        .find(|e| e.tag() == 50719)
        .unwrap();
    let at = container.payload(crop).unwrap().as_ptr() as usize - file.as_ptr() as usize;
    let mut moved = file.clone();
    moved[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
    let bad = check_round_trip(&moved, &want).expect_err("a moved crop must be caught");
    assert!(
        bad.iter().any(|b| b.starts_with("default_crop_origin")),
        "{bad:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — oracle 9, determinism
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn write_is_deterministic_whatever_the_buffer_held() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let odd_block = exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    assert_eq!(odd_block.bytes.len() % 2, 1);
    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    let masked = masked();
    for framing in FRAMINGS {
        let mut o = base_options();
        o.framing = framing;
        o.active_area = Some(margin_area());
        o.black_level = &[7];
        // An odd-length model and odd-length lists force pad bytes.
        o.unique_camera_model = Some("odd");
        o.opcode_list_1 = Some(&l1);
        o.opcode_list_2 = Some(&l2);
        o.opcode_list_3 = Some(&l3);
        if framing == Framing::FullSensor {
            o.masked_areas = &masked;
        }
        // SPEC-081 oracle 7: every metadata option, an odd-length block (its
        // pad byte) and an odd-length XMP packet.
        set_all_metadata(&mut o, &odd_block, XMP_ODD);
        // SPEC-082 oracle 8: an odd-length preview (its pad byte).
        o.preview = Some(preview(JPEG_ODD, JpegPreviewColor::YCbCr));
        let len = written_len(&samples, w, h, &o, BUDGET).unwrap();
        let mut zeros = vec![0x00u8; len];
        let mut ones = vec![0xFFu8; len];
        write_into(&samples, w, h, &o, BUDGET, &mut zeros).unwrap();
        write_into(&samples, w, h, &o, BUDGET, &mut ones).unwrap();
        assert_eq!(zeros, ones, "{framing:?}");
        let (_, back) = read_back(&zeros).unwrap();
        let want = if framing == Framing::FullSensor {
            samples.clone()
        } else {
            cut(&samples, w, margin_area())
        };
        assert_eq!(back, want);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC11 — the committed fuzz seeds reach what their names say
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fuzz_seeds_reach_their_named_outcome() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/seeds/dng_write");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).expect("fuzz/seeds/dng_write exists (just fuzz-seeds)") {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let case = dng_write_fuzz::decode(&std::fs::read(&path).unwrap());
        let owned = case.owned();
        let o = case.options(&owned);
        let got = written_len(
            &case.samples,
            case.width,
            case.height,
            &o,
            usize::from(case.max_pixels),
        );
        assert_eq!(got.is_err(), name.starts_with("refuse-"), "{name}: {got:?}");
        // SPEC-081's seeds name the reason they reach.
        let reasons = [
            ("refuse-date-time", "DateTimeInvalid"),
            (
                "refuse-rational",
                "RationalInvalid { field: \"BaselineNoise\" }",
            ),
            ("refuse-exif-without-ifd", "ExifBlockWithoutIfd"),
            (
                "refuse-exif-offset",
                "ExifIfdOffsetInvalid { ifd: \"GPS\", offset: 11 }",
            ),
            ("refuse-empty-xmp", "EmptyXmp"),
            // SPEC-082's.
            ("refuse-preview-not-jpeg", "PreviewNotJpeg"),
            ("refuse-preview-dimensions", "PreviewDimensionsInvalid"),
        ];
        if let Some((_, want)) = reasons.iter().find(|(n, _)| *n == name) {
            let err = format!("{:?}", got.as_ref().unwrap_err());
            assert!(err.contains(want), "{name}: want {want}, got {err}");
        }
        if name == "all-metadata" {
            assert!(o.exif_block.is_some() && o.xmp.is_some() && o.make.is_some());
            assert!(o.baseline_exposure.is_some() && o.linear_response_limit.is_some());
        }
        if name.starts_with("preview") {
            let p = o.preview.expect("a preview seed carries a preview");
            let gray = name == "preview-gray";
            let color = if gray {
                JpegPreviewColor::Gray
            } else {
                JpegPreviewColor::YCbCr
            };
            assert_eq!((p.width, p.height, p.color), (40, 30, color), "{name}");
            let file = write_to_vec(
                &case.samples,
                case.width,
                case.height,
                &o,
                usize::from(case.max_pixels),
            )
            .unwrap();
            let container = Container::parse(&file).unwrap();
            let got: Vec<_> = container.previews().collect();
            assert!(
                matches!(got.as_slice(), [Ok(q)] if q.bytes == p.jpeg),
                "{name}"
            );
        }
        seen += 1;
    }
    assert_eq!(seen, 32);
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-081 — the EXIF block, XMP, descriptive tags and rendering hints
// ─────────────────────────────────────────────────────────────────────────────

/// An XMP packet of even length, and one of odd length (its pad observable).
const XMP: &[u8] = b"<x:xmpmeta xmlns:x='adobe:ns:meta/'></x:xmpmeta>";
const XMP_ODD: &[u8] = b"<x:xmpmeta xmlns:x='adobe:ns:meta/'/>";

const MAKE: &str = "IRRADIANCE";
const MODEL_NAME: &str = "Mono Test";
const SOFTWARE: &str = "irradiance SPEC-081";
const DATE_TIME: &str = "2026:11:24 10:20:30";
const ARTIST: &str = "A. Photographer";
const COPYRIGHT: &str = "(c) 2026 nobody";
const LOCALIZED: &str = "Mono Test (local)";
const SERIAL: &str = "0042";
const EXPOSURE: Rational = Rational { num: -7, den: 10 };
const NOISE: Rational = Rational { num: 3, den: 4 };
const SHARPNESS: Rational = Rational { num: 5, den: 4 };
const LIMIT: Rational = Rational { num: 9, den: 10 };

fn exif_of(block: &exif_block::Block) -> ExifBlock<'_> {
    let mut b = ExifBlock::default();
    b.bytes = &block.bytes;
    b.exif_ifd = block.exif_ifd;
    b.gps_ifd = block.gps_ifd;
    b
}

/// Every SPEC-081 option, set to the constants above.
fn set_all_metadata<'a>(o: &mut WriteOptions<'a>, block: &'a exif_block::Block, xmp: &'a [u8]) {
    o.exif_block = Some(exif_of(block));
    o.xmp = Some(xmp);
    o.make = Some(MAKE);
    o.model = Some(MODEL_NAME);
    o.software = Some(SOFTWARE);
    o.date_time = Some(DATE_TIME);
    o.artist = Some(ARTIST);
    o.copyright = Some(COPYRIGHT);
    o.localized_camera_model = Some(LOCALIZED);
    o.camera_serial_number = Some(SERIAL);
    o.baseline_exposure = Some(EXPOSURE);
    o.baseline_noise = Some(NOISE);
    o.baseline_sharpness = Some(SHARPNESS);
    o.linear_response_limit = Some(LIMIT);
}

fn nul(text: &str) -> Vec<u8> {
    let mut v = text.as_bytes().to_vec();
    v.push(0);
    v
}

fn rational_bytes(r: Rational) -> Vec<u8> {
    let mut v = r.num.to_le_bytes().to_vec();
    v.extend_from_slice(&r.den.to_le_bytes());
    v
}

/// The header's IFD0 offset.
fn header_ifd0(file: &[u8]) -> u32 {
    u32::from_le_bytes(file[4..8].try_into().unwrap())
}

#[test]
fn exif_block_base_is_pinned_to_eight() {
    assert_eq!(EXIF_BLOCK_BASE, 8);
    assert_eq!(exif_block_base(&WriteOptions::default()), 8);
    let block = exif_block::build(8, exif_block::Shape::Both);
    let mut o = base_options();
    set_all_metadata(&mut o, &block, XMP);
    o.framing = Framing::FullSensor;
    o.active_area = Some(margin_area());
    o.black_level = &[1];
    assert_eq!(exif_block_base(&o), 8);
    // Every new field defaults to None.
    let d = WriteOptions::default();
    assert!(d.exif_block.is_none() && d.xmp.is_none());
    for s in [
        d.make,
        d.model,
        d.software,
        d.date_time,
        d.artist,
        d.copyright,
        d.localized_camera_model,
        d.camera_serial_number,
    ] {
        assert!(s.is_none());
    }
    for r in [
        d.baseline_exposure,
        d.baseline_noise,
        d.baseline_sharpness,
        d.linear_response_limit,
    ] {
        assert!(r.is_none());
    }
    let b = ExifBlock::default();
    assert!(b.bytes.is_empty() && b.exif_ifd.is_none() && b.gps_ifd.is_none());
}

#[test]
fn every_new_tag_matches_the_spec_table() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let (byte, ascii, long, rational, srational) = (1u16, 2u16, 4u16, 5u16, 10u16);
    for odd in [false, true] {
        let block = if odd {
            exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both)
        } else {
            exif_block::build(EXIF_BLOCK_BASE, exif_block::Shape::Both)
        };
        assert_eq!(block.bytes.len() % 2 == 1, odd);
        let xmp = if odd { XMP_ODD } else { XMP };
        let mut o = base_options();
        set_all_metadata(&mut o, &block, xmp);
        let file = write(&samples, w, h, &o);
        let entries = ifd0_entries(&file);
        // Ascending: the 18 SPEC-061 tags these options fill, plus the 15.
        let tags: Vec<u16> = entries.iter().map(|e| e.0).collect();
        assert!(tags.windows(2).all(|p| p[0] < p[1]), "{tags:?}");
        assert_eq!(entries.len(), 18 + 15, "{tags:?}");
        let want: Vec<(u16, u16, u32, Vec<u8>)> = vec![
            (271, ascii, 11, nul(MAKE)),
            (272, ascii, 10, nul(MODEL_NAME)),
            (305, ascii, 20, nul(SOFTWARE)),
            (306, ascii, 20, nul(DATE_TIME)),
            (315, ascii, 16, nul(ARTIST)),
            (700, byte, xmp.len() as u32, xmp.to_vec()),
            (33432, ascii, 16, nul(COPYRIGHT)),
            (
                34665,
                long,
                1,
                (8 + block.exif_ifd.unwrap()).to_le_bytes().to_vec(),
            ),
            (
                34853,
                long,
                1,
                (8 + block.gps_ifd.unwrap()).to_le_bytes().to_vec(),
            ),
            (50709, ascii, 18, nul(LOCALIZED)),
            (50730, srational, 1, rational_bytes(EXPOSURE)),
            (50731, rational, 1, rational_bytes(NOISE)),
            (50732, rational, 1, rational_bytes(SHARPNESS)),
            (50734, rational, 1, rational_bytes(LIMIT)),
            (50735, ascii, 5, nul(SERIAL)),
        ];
        for row in &want {
            let got = entries
                .iter()
                .find(|e| e.0 == row.0)
                .unwrap_or_else(|| panic!("tag {} missing (odd={odd})", row.0));
            assert_eq!(got, row, "tag {} (odd={odd})", row.0);
        }
        // Every out-of-line value sits at an even offset.
        let container = Container::parse(&file).unwrap();
        for e in container.ifds()[0].entries() {
            let len = container.payload(e).unwrap().len();
            if len > 4 {
                let at = u32::from_le_bytes(file_entry_word(&file, e.tag()));
                assert_eq!(at % 2, 0, "tag {} at {at}", e.tag());
            }
        }
    }
}

/// The 4-byte value field of IFD0's entry for `tag`, raw.
fn file_entry_word(file: &[u8], tag: u16) -> [u8; 4] {
    let ifd0 = header_ifd0(file) as usize;
    let count = u16::from_le_bytes([file[ifd0], file[ifd0 + 1]]) as usize;
    (0..count)
        .map(|i| ifd0 + 2 + 12 * i)
        .find(|&at| u16::from_le_bytes([file[at], file[at + 1]]) == tag)
        .map(|at| file[at + 8..at + 12].try_into().unwrap())
        .unwrap_or_else(|| panic!("tag {tag} not in IFD0"))
}

#[test]
fn exif_block_sits_at_the_base_and_ifd0_follows_it() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    use exif_block::Shape;
    let mut cases = 0;
    for shape in [Shape::ExifOnly, Shape::GpsOnly, Shape::Both] {
        for odd in [false, true] {
            let block = if odd {
                exif_block::build_odd(EXIF_BLOCK_BASE, shape)
            } else {
                exif_block::build(EXIF_BLOCK_BASE, shape)
            };
            let len = block.bytes.len();
            let mut o = base_options();
            o.exif_block = Some(exif_of(&block));
            let file = write(&samples, w, h, &o);
            let ifd0 = header_ifd0(&file) as usize;
            assert_eq!(ifd0, 8 + len + len % 2, "{shape:?} odd={odd}");
            assert_eq!(&file[8..8 + len], &block.bytes[..], "{shape:?} odd={odd}");
            if odd {
                assert_eq!(file[8 + len], 0, "{shape:?}: the pad byte is 0");
            }
            let entries = ifd0_entries(&file);
            let pointer = |tag: u16| {
                entries
                    .iter()
                    .find(|e| e.0 == tag)
                    .map(|e| (e.1, e.2, le_u32s(&e.3)[0]))
            };
            assert_eq!(
                pointer(34665),
                block.exif_ifd.map(|off| (4, 1, 8 + off)),
                "{shape:?} ExifIFD"
            );
            assert_eq!(
                pointer(34853),
                block.gps_ifd.map(|off| (4, 1, 8 + off)),
                "{shape:?} GPSInfo"
            );
            // The pointed-at IFD's entry count is the builder's.
            for (tag, want_count) in [(34665u16, 6u16), (34853, 3)] {
                if let Some((_, _, at)) = pointer(tag) {
                    let at = at as usize;
                    assert_eq!(u16::from_le_bytes([file[at], file[at + 1]]), want_count);
                }
            }
            // And the plane still reads back.
            let (_, back) = read_back(&file).unwrap();
            assert_eq!(back, samples);
            cases += 1;
        }
    }
    assert_eq!(cases, 6);
    // Without a block, IFD0 follows the header as before.
    let file = write(&samples, w, h, &base_options());
    assert_eq!(header_ifd0(&file), 8);
}

#[test]
fn absent_options_write_no_tag_and_empty_strings_write_count_one() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let new_tags = [
        271u16, 272, 305, 306, 315, 700, 33432, 34665, 34853, 50709, 50730, 50731, 50732, 50734,
        50735,
    ];
    let file = write(&samples, w, h, &base_options());
    let entries = ifd0_entries(&file);
    for tag in new_tags {
        assert!(entries.iter().all(|e| e.0 != tag), "tag {tag} written");
    }
    let mut o = base_options();
    o.make = Some("");
    o.model = Some("");
    o.software = Some("");
    o.artist = Some("");
    o.copyright = Some("");
    o.localized_camera_model = Some("");
    o.camera_serial_number = Some("");
    o.copyright = Some("");
    let entries = ifd0_entries(&write(&samples, w, h, &o));
    for tag in [271u16, 272, 305, 315, 33432, 50709, 50735] {
        let e = entries.iter().find(|e| e.0 == tag).expect("written");
        assert_eq!((e.1, e.2, e.3.clone()), (2, 1, vec![0]), "tag {tag}");
    }
    // Four spaces, as the Q2M holds, are written as given.
    o.artist = Some("    ");
    let entries = ifd0_entries(&write(&samples, w, h, &o));
    let e = entries.iter().find(|e| e.0 == 315).unwrap();
    assert_eq!((e.2, e.3.clone()), (5, b"    \0".to_vec()));
}

#[test]
fn round_trip_holds_with_every_new_option_set() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let block = exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let masked = masked();
    let mut cases = 0;
    for framing in FRAMINGS {
        let mut o = base_options();
        o.framing = framing;
        o.active_area = Some(margin_area());
        o.black_level = &[64];
        o.white_level = 60000;
        o.orientation = 6;
        o.default_crop = Some((
            DefaultCropOrigin { x: 1, y: 2 },
            DefaultCropSize {
                width: 9,
                height: 5,
            },
        ));
        if framing == Framing::FullSensor {
            o.masked_areas = &masked;
        }
        set_all_metadata(&mut o, &block, XMP_ODD);
        let file = write(&samples, w, h, &o);
        let want = expected(&samples, w, h, &o);
        if let Err(bad) = check_round_trip(&file, &want) {
            panic!("{framing:?}: {bad:#?}");
        }
        // The hints read back through the reader as onyx reads them.
        let container = Container::parse(&file).unwrap();
        let ifd0 = container.ifd0().unwrap();
        for (tag, want) in [
            (50730u16, EXPOSURE),
            (50731, NOISE),
            (50732, SHARPNESS),
            (50734, LIMIT),
        ] {
            let entry = ifd0.entry(tag).unwrap();
            assert_eq!(container.rationals(entry).unwrap(), vec![want], "tag {tag}");
        }
        cases += 1;
    }
    assert_eq!(cases, 3);
}

#[test]
fn date_time_form_is_checked_not_the_calendar() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let check = |text: &'static str| {
        let mut o = base_options();
        o.date_time = Some(text);
        written_len(&samples, w, h, &o, BUDGET)
    };
    for good in ["2026:13:40 25:61:61", "0000:00:00 00:00:00", DATE_TIME] {
        assert!(check(good).is_ok(), "{good}");
    }
    for bad in [
        "2026-11-24 10:00:00",
        "2026:11:24 10:00:0",
        "2026:11:24 10:00:000",
        "2026:1a:24 10:00:00",
        "2026:11:24T10:00:00",
        "2026:11:24  10:00:0",
        "",
    ] {
        let err = check(bad).unwrap_err();
        assert_eq!(
            format!("{err:?}"),
            format!("{:?}", reason(WriteInputError::DateTimeInvalid)),
            "{bad:?}"
        );
    }
}

#[test]
fn rational_hints_refuse_zero_and_unsigned_negatives() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    type Set = fn(&mut WriteOptions<'static>, Rational);
    let hints: [(&str, Set, bool); 4] = [
        (
            "BaselineExposure",
            |o, r| o.baseline_exposure = Some(r),
            true,
        ),
        ("BaselineNoise", |o, r| o.baseline_noise = Some(r), false),
        (
            "BaselineSharpness",
            |o, r| o.baseline_sharpness = Some(r),
            false,
        ),
        (
            "LinearResponseLimit",
            |o, r| o.linear_response_limit = Some(r),
            false,
        ),
    ];
    let r = |num, den| Rational { num, den };
    for (field, set, signed) in hints {
        let run = |value: Rational| {
            let mut o = base_options();
            set(&mut o, value);
            written_len(&samples, w, h, &o, BUDGET)
        };
        let refused = format!("{:?}", reason(WriteInputError::RationalInvalid { field }));
        assert!(run(r(1, 2)).is_ok(), "{field} 1/2");
        assert!(run(r(0, 1)).is_ok(), "{field} 0/1");
        for zero in [r(1, 0), r(0, 0), r(-1, 0)] {
            assert_eq!(format!("{:?}", run(zero).unwrap_err()), refused, "{field}");
        }
        for negative in [r(-1, 2), r(1, -2)] {
            if signed {
                assert!(run(negative).is_ok(), "{field} {negative:?}");
            } else {
                assert_eq!(
                    format!("{:?}", run(negative).unwrap_err()),
                    refused,
                    "{field} {negative:?}"
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-082 — the optional caller JPEG preview
// ─────────────────────────────────────────────────────────────────────────────

/// The three no-preview outputs oracle 2 pins: no metadata, every SPEC-081
/// field set, and a FullSensor output with masked areas and all three opcode
/// lists. Each is `(name, file)`.
fn oracle_2_outputs() -> Vec<(&'static str, Vec<u8>)> {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let mut out = Vec::new();

    let mut o = base_options();
    o.active_area = Some(margin_area());
    out.push(("no metadata", write(&samples, w, h, &o)));

    let block = exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let mut o = base_options();
    o.active_area = Some(margin_area());
    o.black_level = &[64];
    o.white_level = 60000;
    set_all_metadata(&mut o, &block, XMP_ODD);
    out.push(("every SPEC-081 field", write(&samples, w, h, &o)));

    let (l1, l2, l3) = (list_1(), list_2(), list_3());
    let masked = masked();
    let mut o = base_options();
    o.framing = Framing::FullSensor;
    o.active_area = Some(margin_area());
    o.black_level_repeat_dim = Some([1, 1]);
    o.black_level = &[64];
    o.white_level = 60000;
    o.orientation = 3;
    o.masked_areas = &masked;
    o.opcode_list_1 = Some(&l1);
    o.opcode_list_2 = Some(&l2);
    o.opcode_list_3 = Some(&l3);
    out.push(("FullSensor, masked, three lists", write(&samples, w, h, &o)));
    out
}

/// Oracle 2 (AC3): with no preview the output is byte-identical to
/// SPEC-081's. The MD5s were recorded on `main` at 165a096 (whose `src/` is
/// identical to 386b4a7's) before any `src/` change for SPEC-082.
#[test]
fn no_preview_output_is_byte_identical_to_spec_081() {
    const PINNED: [&str; 3] = [
        "fb81830e8abeee795eb1c5ecb660209f",
        "36160d2e0d02d0b303716dd9a342fff9",
        "e5081b7db946fdc4461307e78b0ab592",
    ];
    let got: Vec<(&str, String)> = oracle_2_outputs()
        .into_iter()
        .map(|(name, file)| (name, md5::to_hex(&md5::hash(&file))))
        .collect();
    for (name, hex) in &got {
        println!("{name}: {hex}");
    }
    assert_eq!(got.len(), PINNED.len());
    for ((name, hex), want) in got.iter().zip(PINNED) {
        assert_eq!(hex, want, "{name}");
    }
}

/// Synthetic tier-A "JPEGs": `FF D8`, filler, `FF D9`. The writer never looks
/// past the first two bytes, so these need not decode. One even and one odd
/// length (the odd one makes the pad before the raw strip observable).
const JPEG_EVEN: &[u8] = &[0xFF, 0xD8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0xFF, 0xD9];
const JPEG_ODD: &[u8] = &[
    0xFF, 0xD8, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0xFF, 0xD9,
];
/// Dimensions no real JPEG here has: they come from the caller, never the
/// stream (DEC-082 §4).
const PREVIEW_W: u32 = 37;
const PREVIEW_H: u32 = 23;

fn preview(jpeg: &[u8], color: JpegPreviewColor) -> JpegPreview<'_> {
    let mut p = JpegPreview::default();
    p.jpeg = jpeg;
    p.width = PREVIEW_W;
    p.height = PREVIEW_H;
    p.color = color;
    p
}

fn le16(file: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([file[at], file[at + 1]])
}

fn le32(file: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(file[at..at + 4].try_into().unwrap())
}

/// One IFD read straight from the bytes, without the crate's reader:
/// `(entries as (tag, type, count, value-or-offset word), next offset)`.
fn raw_ifd(file: &[u8], at: u32) -> (Vec<(u16, u16, u32, u32)>, u32) {
    let at = at as usize;
    let n = usize::from(le16(file, at));
    let entries = (0..n)
        .map(|i| {
            let e = at + 2 + 12 * i;
            (
                le16(file, e),
                le16(file, e + 2),
                le32(file, e + 4),
                le32(file, e + 8),
            )
        })
        .collect();
    (entries, le32(file, at + 2 + 12 * n))
}

/// The preview IFD's offset, from IFD0's `SubIFDs` word, read raw.
fn sub_ifd_offset(file: &[u8]) -> u32 {
    let (ifd0, _) = raw_ifd(file, header_ifd0(file));
    let subs: Vec<_> = ifd0.iter().filter(|e| e.0 == 330).collect();
    assert_eq!(subs.len(), 1, "one SubIFDs entry");
    assert_eq!((subs[0].1, subs[0].2), (4, 1), "SubIFDs is LONG, count 1");
    subs[0].3
}

/// The preview IFD as SPEC-082 §2's table writes it, hand-computed:
/// `(tag, type, count, inline value)`; `None` for the two offsets, which
/// are checked separately.
fn preview_table(color: JpegPreviewColor, jpeg_len: u32) -> Vec<(u16, u16, u32, Option<u32>)> {
    let (bits_count, photometric, spp) = match color {
        JpegPreviewColor::YCbCr => (3, 6, 3),
        _ => (1, 1, 1),
    };
    vec![
        (254, 4, 1, Some(1)),
        (256, 4, 1, Some(PREVIEW_W)),
        (257, 4, 1, Some(PREVIEW_H)),
        // Gray's one SHORT sits inline in the low half; YCbCr's three are
        // out of line.
        (258, 3, bits_count, (bits_count == 1).then_some(8)),
        (259, 3, 1, Some(7)),
        (262, 3, 1, Some(photometric)),
        (273, 4, 1, None),
        (277, 3, 1, Some(spp)),
        (278, 4, 1, Some(PREVIEW_H)),
        (279, 4, 1, Some(jpeg_len)),
        (284, 3, 1, Some(1)),
    ]
}

/// Oracle 1 (AC2): IFD0's `SubIFDs` points at an IFD whose entries are
/// exactly §2's table, the JPEG sits at its `StripOffsets`, both IFDs end
/// the chain, and the crate's reader finds the same IFD as IFD0's child.
#[test]
fn preview_ifd_matches_the_spec_table() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    for color in [JpegPreviewColor::YCbCr, JpegPreviewColor::Gray] {
        let mut o = base_options();
        o.active_area = Some(margin_area());
        o.preview = Some(preview(JPEG_EVEN, color));
        let file = write(&samples, w, h, &o);

        let (ifd0, ifd0_next) = raw_ifd(&file, header_ifd0(&file));
        assert_eq!(ifd0_next, 0, "{color:?}: IFD0 has no next IFD");
        // SubIFDs sits between Artist (315) and XMLPacket (700): the tags
        // stay ascending.
        assert!(ifd0.windows(2).all(|p| p[0].0 < p[1].0), "{color:?}");
        let at = sub_ifd_offset(&file);
        let (entries, next) = raw_ifd(&file, at);
        assert_eq!(next, 0, "{color:?}: the preview IFD has no next IFD");
        let table = preview_table(color, JPEG_EVEN.len() as u32);
        assert_eq!(entries.len(), table.len(), "{color:?}");
        for (got, want) in entries.iter().zip(&table) {
            assert_eq!((got.0, got.1, got.2), (want.0, want.1, want.2), "{color:?}");
            if let Some(v) = want.3 {
                assert_eq!(got.3, v, "{color:?}: tag {}", got.0);
            }
        }
        if color == JpegPreviewColor::YCbCr {
            let bits_at = entries[3].3 as usize;
            assert_eq!(&file[bits_at..bits_at + 6], &[8, 0, 8, 0, 8, 0]);
        }
        let jpeg_at = entries[6].3 as usize;
        assert_eq!(&file[jpeg_at..jpeg_at + JPEG_EVEN.len()], JPEG_EVEN);

        // The reader walks the same IFD as IFD0's SubIFD.
        let container = Container::parse(&file).unwrap();
        let ifds = container.ifds();
        assert_eq!(ifds.len(), 2, "{color:?}");
        assert_eq!(ifds[1].offset(), at);
        assert_eq!(ifds[1].parent(), Some(0));
        assert_eq!(ifds[1].depth(), 1);
    }
}

/// Oracle 1, layout (AC2, DEC-082 §2): IFD0's values, then the preview IFD
/// and its values, then the JPEG, then the raw strip last; every offset even
/// and the pad after an odd-length JPEG 0. With and without an EXIF block.
#[test]
fn preview_sits_between_ifd0_values_and_the_raw_strip() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let block = exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let mut cases = 0;
    for with_block in [false, true] {
        for jpeg in [JPEG_EVEN, JPEG_ODD] {
            for color in [JpegPreviewColor::YCbCr, JpegPreviewColor::Gray] {
                let mut o = base_options();
                o.active_area = Some(margin_area());
                o.unique_camera_model = Some("odd"); // an odd value forces a pad
                if with_block {
                    set_all_metadata(&mut o, &block, XMP_ODD);
                }
                o.preview = Some(preview(jpeg, color));
                let file = write(&samples, w, h, &o);
                let name = format!("block {with_block}, jpeg {}, {color:?}", jpeg.len());

                // IFD0's out-of-line values all end before the preview IFD.
                let ifd0_at = header_ifd0(&file);
                let (ifd0, _) = raw_ifd(&file, ifd0_at);
                let unit = |t: u16| match t {
                    3 => 2,
                    4 => 4,
                    5 | 10 => 8,
                    _ => 1,
                };
                let ifd0_end = ifd0
                    .iter()
                    .filter(|e| unit(e.1) * e.2 > 4)
                    .map(|e| e.3 + unit(e.1) * e.2)
                    .max()
                    .unwrap();
                let at = sub_ifd_offset(&file);
                assert_eq!(at % 2, 0, "{name}");
                assert_eq!(
                    at,
                    ifd0_end + ifd0_end % 2,
                    "{name}: right after IFD0's values"
                );
                assert!(at > ifd0_at, "{name}");

                // The preview IFD, then its one out-of-line value (YCbCr's
                // BitsPerSample), then the JPEG.
                let (entries, _) = raw_ifd(&file, at);
                let ifd_end = at + 2 + 12 * entries.len() as u32 + 4;
                let values_end = if color == JpegPreviewColor::YCbCr {
                    assert_eq!(entries[3].3, ifd_end, "{name}: BitsPerSample's value");
                    ifd_end + 6
                } else {
                    ifd_end
                };
                let jpeg_at = entries[6].3;
                assert_eq!(jpeg_at, values_end + values_end % 2, "{name}");
                assert_eq!(jpeg_at % 2, 0, "{name}");

                // The raw strip, last, at the next even offset; the pad is 0.
                let raw_at = ifd0.iter().find(|e| e.0 == 273).unwrap().3;
                let jpeg_end = jpeg_at + jpeg.len() as u32;
                assert_eq!(raw_at, jpeg_end + jpeg_end % 2, "{name}");
                if jpeg.len() % 2 == 1 {
                    assert_eq!(file[jpeg_end as usize], 0, "{name}: the pad byte");
                }
                let raw_len = ifd0.iter().find(|e| e.0 == 279).unwrap().3;
                assert_eq!(raw_at as usize + raw_len as usize, file.len(), "{name}");
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 8);
}

/// Oracle 3 (AC4): SPEC-061's round trip holds at every framing with a
/// preview set, alone and beside every SPEC-081 option.
#[test]
fn round_trip_holds_with_a_preview() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let block = exif_block::build_odd(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let masked = masked();
    let mut cases = 0;
    for framing in FRAMINGS {
        for with_meta in [false, true] {
            for color in [JpegPreviewColor::YCbCr, JpegPreviewColor::Gray] {
                let mut o = base_options();
                o.framing = framing;
                o.active_area = Some(margin_area());
                o.black_level = &[64];
                o.white_level = 60000;
                o.default_crop = Some((
                    DefaultCropOrigin { x: 1, y: 2 },
                    DefaultCropSize {
                        width: 9,
                        height: 5,
                    },
                ));
                if framing == Framing::FullSensor {
                    o.masked_areas = &masked;
                }
                if with_meta {
                    set_all_metadata(&mut o, &block, XMP_ODD);
                }
                o.preview = Some(preview(JPEG_ODD, color));
                let file = write(&samples, w, h, &o);
                let want = expected(&samples, w, h, &o);
                if let Err(bad) = check_round_trip(&file, &want) {
                    panic!("{framing:?} meta {with_meta} {color:?}: {bad:#?}");
                }
                // The preview IFD is there under every framing (DEC-082 §6).
                assert_eq!(Container::parse(&file).unwrap().previews().count(), 1);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 12);
}

/// Oracle 4's comparison: `Ok` when the file's previews are exactly one
/// `Ok` preview matching the caller's, otherwise what differs.
fn check_preview(file: &[u8], jpeg: &[u8], color: JpegPreviewColor) -> Result<(), String> {
    let container = Container::parse(file).map_err(|e| e.to_string())?;
    let all: Vec<_> = container.previews().collect();
    let [Ok(p)] = all.as_slice() else {
        return Err(format!("want one Ok preview, got {all:?}"));
    };
    let (photometric, spp) = match color {
        JpegPreviewColor::YCbCr => (6, 3),
        _ => (1, 1),
    };
    let fields = (
        p.kind,
        p.ifd_index,
        p.width,
        p.height,
        p.compression,
        p.photometric,
        p.samples_per_pixel,
        p.bits_per_sample,
    );
    let want = (
        PreviewKind::Primary,
        1,
        PREVIEW_W,
        PREVIEW_H,
        Compression::Jpeg,
        Some(photometric),
        spp,
        Some(8),
    );
    if fields != want {
        return Err(format!("fields {fields:?}, want {want:?}"));
    }
    if p.bytes != jpeg {
        return Err(format!("bytes {:02X?}, want {jpeg:02X?}", p.bytes));
    }
    Ok(())
}

/// Oracle 4 (AC4, DEC-057): `Container::previews` returns the caller's JPEG
/// byte for byte, at every framing.
#[test]
fn previews_returns_the_callers_jpeg() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let mut cases = 0;
    for framing in FRAMINGS {
        for jpeg in [JPEG_EVEN, JPEG_ODD] {
            for color in [JpegPreviewColor::YCbCr, JpegPreviewColor::Gray] {
                let mut o = base_options();
                o.framing = framing;
                o.active_area = Some(margin_area());
                o.preview = Some(preview(jpeg, color));
                let file = write(&samples, w, h, &o);
                check_preview(&file, jpeg, color)
                    .unwrap_or_else(|e| panic!("{framing:?} {color:?}: {e}"));
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 12);
    // Without a preview, none.
    let file = write(&samples, w, h, &base_options());
    assert_eq!(Container::parse(&file).unwrap().previews().count(), 0);
}

/// Oracle 5 (AC6): oracle 4's comparison passes on a fresh output (the
/// negative control), fails once the preview's `StripOffsets` moves by 2, and
/// a preview IFD whose `NewSubFileType` is 0 is no preview at all.
#[test]
fn red_proof_a_moved_preview_offset_is_caught() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let mut o = base_options();
    o.preview = Some(preview(JPEG_EVEN, JpegPreviewColor::YCbCr));
    let file = write(&samples, w, h, &o);

    // Negative control first.
    check_preview(&file, JPEG_EVEN, JpegPreviewColor::YCbCr).expect("fresh output passes");

    // Each entry's position, read raw: 2-byte count, then 12 bytes each.
    let at = sub_ifd_offset(&file) as usize;
    let (entries, _) = raw_ifd(&file, at as u32);
    let slot = |tag: u16| at + 2 + 12 * entries.iter().position(|e| e.0 == tag).unwrap() + 8;

    let mut moved = file.clone();
    let offset_at = slot(273);
    let shifted = le32(&moved, offset_at) + 2;
    moved[offset_at..offset_at + 4].copy_from_slice(&shifted.to_le_bytes());
    let err = check_preview(&moved, JPEG_EVEN, JpegPreviewColor::YCbCr)
        .expect_err("a moved StripOffsets must be caught");
    assert!(err.contains("bytes"), "{err}");

    let mut not_preview = file.clone();
    let nst_at = slot(254);
    not_preview[nst_at..nst_at + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        Container::parse(&not_preview).unwrap().previews().count(),
        0,
        "NewSubFileType 0 is not a preview"
    );
}

/// AC5, the boundaries that pass: 65535 in both dimensions, a two-byte JPEG.
#[test]
fn preview_dimension_and_marker_boundaries_pass() {
    let (w, h) = (20, 14);
    let samples = plane(w, h);
    let mut o = base_options();
    let mut p = preview(&[0xFF, 0xD8], JpegPreviewColor::Gray);
    p.width = 65535;
    p.height = 65535;
    o.preview = Some(p);
    let file = write(&samples, w, h, &o);
    let at = sub_ifd_offset(&file);
    let (entries, _) = raw_ifd(&file, at);
    assert_eq!((entries[1].3, entries[2].3), (65535, 65535));
    p.width = 1;
    p.height = 1;
    o.preview = Some(p);
    assert!(written_len(&samples, w, h, &o, BUDGET).is_ok());
}
