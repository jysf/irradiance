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
//!
//! Nothing is written to the repo.

#[path = "support/dng_write_fuzz.rs"]
mod dng_write_fuzz;
#[path = "support/dng_write_oracle.rs"]
mod dng_write_oracle;
#[path = "support/dng_writer.rs"]
mod dng_writer;
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
    write_into, write_to_vec, written_len, Framing, WriteInputError, WriteOptions,
};
use irradiance::ifd::{ActiveArea, Container, DefaultCropOrigin, DefaultCropSize};
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
    assert_eq!(n, 29);
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
        let err = written_len(&samples, w, h, &o, BUDGET).unwrap_err();
        assert!(
            format!("{err:?}").contains(want),
            "want {want}, got {err:?}"
        );
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
        seen += 1;
    }
    assert_eq!(seen, 22);
}
