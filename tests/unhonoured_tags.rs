//! `SPEC-086` (`DEC-077`): the develop refuses tags that change what the
//! samples mean, and a DNG backward version newer than 1.7.1.0.
//!
//! A file carrying `SampleFormat` other than 1, a `LinearizationTable`, a
//! non-zero `BlackLevelDeltaH/V`, a `SubTileBlockSize` other than `[1, 1]`, or
//! an interleave factor other than 1 used to develop with `Ok` and wrong
//! pixels: none of the seven tags was read. Now every develop entry refuses it
//! with a typed error before writing to `dst`, through
//! `opcode::check_required_processing`, the one gate every entry already calls
//! first. Metadata and plane reads still succeed (`DEC-077` §2).
//!
//! Every fixture goes through the real container read
//! (`Container::parse → sensor → unpack_into`) from a file written by the
//! test-only DNG writer, with the sensor in a SubIFD under a preview IFD0 so
//! the IFD0 tags (the two versions) and the sensor tags sit where a camera
//! writes them.
//!
//! §12 behavioural verification: each of the nine public develop entries is
//! exercised on its own, not one representative (mono entries on a mono
//! fixture, the rest on an RGGB fixture).
//!
//! **Tier-A only**: synthetic fixtures, no corpus, no tools.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, to_srgb_linear_into,
};
use irradiance::demosaic::{demosaic_into, demosaic_into_threaded};
use irradiance::develop::{
    develop_into, develop_with_scratch_into, output_dimensions, scratch_len, DevelopOptions,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Tag numbers (DNG 1.7.1.0; TIFF 6.0 §19 for SampleFormat). Written out here
// rather than imported, so the test states the numbers the spec names.
// ─────────────────────────────────────────────────────────────────────────────

const SAMPLE_FORMAT: u16 = 339;
const DNG_BACKWARD_VERSION: u16 = 50707;
const LINEARIZATION_TABLE: u16 = 50712;
const BLACK_LEVEL_DELTA_H: u16 = 50715;
const BLACK_LEVEL_DELTA_V: u16 = 50716;
const SUB_TILE_BLOCK_SIZE: u16 = 50974;
const ROW_INTERLEAVE_FACTOR: u16 = 50975;
const COLUMN_INTERLEAVE_FACTOR: u16 = 51157;

/// The seven sensor-IFD tags of `DEC-077` §2's table, in ascending order.
const TAG_SET: [u16; 7] = [
    SAMPLE_FORMAT,
    LINEARIZATION_TABLE,
    BLACK_LEVEL_DELTA_H,
    BLACK_LEVEL_DELTA_V,
    SUB_TILE_BLOCK_SIZE,
    ROW_INTERLEAVE_FACTOR,
    COLUMN_INTERLEAVE_FACTOR,
];

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_SRATIONAL: u16 = 10;

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures: an 8×8 16-bit plane, mono (LinearRaw) or 2×2 RGGB.
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 8;
const H: u32 = 8;

fn samples() -> Vec<u16> {
    (0..W * H).map(|i| 1000 + (i as u16) * 37).collect()
}

fn base_desc(cfa: bool) -> DngDescription {
    let kind = if cfa {
        PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        }
    } else {
        PlaneKind::LinearRaw
    };
    let mut meta = DngMeta::default();
    if cfa {
        // The colour calibration the colour develop needs (the same matrix
        // and neutral `tests/required_processing.rs` uses).
        meta.forward_matrix_1 = Some(
            [
                [60000, 30000, 6422],
                [20000, 70000, 10000],
                [10000, 10000, 62521],
            ]
            .iter()
            .flatten()
            .map(|&num| (num, 100_000))
            .collect(),
        );
        meta.as_shot_neutral = Some(vec![(1, 2), (1, 1), (1, 2)]);
    }
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width: W,
        height: H,
        bits_per_sample: 16,
        kind,
        organisation: Organisation::Strips { rows_per_strip: H },
        payload: Payload::Samples(samples()),
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// A fixture, read the way a caller reads a camera file.
struct Fixture {
    sensor: Sensor,
    plane: Vec<u16>,
}

fn load(d: &DngDescription) -> Fixture {
    let file = write_dng(d).expect("the fixture writes");
    let container = Container::parse(&file).expect("the fixture parses");
    let sensor = container.sensor().expect("the fixture has a sensor IFD");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("the fixture unpacks");
    Fixture { sensor, plane }
}

/// The fixture `entry` develops: mono for the two mono entries, RGGB for the
/// rest, with `modify` applied to its description.
fn fixture_for(entry: Entry, modify: &dyn Fn(&mut DngDescription)) -> Fixture {
    let mut d = base_desc(!entry.mono());
    modify(&mut d);
    load(&d)
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw tags, little-endian (the fixture's byte order).
// ─────────────────────────────────────────────────────────────────────────────

fn shorts(tag: u16, values: &[u16]) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_SHORT,
        count: values.len() as u32,
        bytes: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    }
}

fn srationals(tag: u16, values: &[(i32, i32)]) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_SRATIONAL,
        count: values.len() as u32,
        bytes: values
            .iter()
            .flat_map(|(n, d)| n.to_le_bytes().into_iter().chain(d.to_le_bytes()))
            .collect(),
    }
}

/// `tag` written as `ASCII` "x": the wrong type for every tag here.
fn ascii(tag: u16) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_ASCII,
        count: 2,
        bytes: b"x\0".to_vec(),
    }
}

fn add(d: &mut DngDescription, tag: RawTag) {
    d.extra_tags.push(tag);
}

/// `n` SRATIONAL zeros (`0/1`).
fn zeros(n: usize) -> Vec<(i32, i32)> {
    vec![(0, 1); n]
}

// ─────────────────────────────────────────────────────────────────────────────
// The nine public develop entries, each its own door (§12).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    DevelopInto,
    DevelopWithScratchInto,
    DevelopColorInto,
    DevelopColorF32Into,
    DevelopColorIntoThreaded,
    DevelopColorF32IntoThreaded,
    ToSrgbLinearInto,
    DemosaicInto,
    DemosaicIntoThreaded,
}

const ENTRIES: [Entry; 9] = [
    Entry::DevelopInto,
    Entry::DevelopWithScratchInto,
    Entry::DevelopColorInto,
    Entry::DevelopColorF32Into,
    Entry::DevelopColorIntoThreaded,
    Entry::DevelopColorF32IntoThreaded,
    Entry::ToSrgbLinearInto,
    Entry::DemosaicInto,
    Entry::DemosaicIntoThreaded,
];

impl Entry {
    fn mono(self) -> bool {
        matches!(self, Entry::DevelopInto | Entry::DevelopWithScratchInto)
    }
}

/// A sentinel no develop writes, so an untouched buffer is recognisable.
const SENTINEL_U16: u16 = 0xA5A5;
const SENTINEL_F32: f32 = -12345.5;

/// Everything an entry wrote: `dst` (and the caller's scratch, after it).
/// `f32` output is kept as bits so equality is byte equality.
#[derive(Debug, PartialEq, Eq)]
enum Written {
    U16(Vec<u16>),
    F32(Vec<u32>),
}

impl Written {
    fn untouched(&self) -> bool {
        match self {
            Written::U16(v) => v.iter().all(|&x| x == SENTINEL_U16),
            Written::F32(v) => v.iter().all(|&x| x == SENTINEL_F32.to_bits()),
        }
    }
}

fn run(entry: Entry, fx: &Fixture) -> (Result<(), Error>, Written) {
    let sensor = &fx.sensor;
    let plane = &fx.plane;
    let out_len = || {
        let (w, h) = output_dimensions(sensor).expect("the fixture geometry is valid");
        w as usize * h as usize
    };
    let raw3 = W as usize * H as usize * 3;
    let u16s = |n: usize| vec![SENTINEL_U16; n];
    let f32s = |n: usize| vec![SENTINEL_F32; n];
    let bits = |v: Vec<f32>| Written::F32(v.into_iter().map(f32::to_bits).collect());
    match entry {
        Entry::DevelopInto => {
            let mut dst = u16s(out_len());
            let r = develop_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DevelopWithScratchInto => {
            let options = DevelopOptions::default();
            let n = scratch_len(sensor, &options).expect("the scratch length resolves");
            let mut dst = u16s(out_len());
            let mut scratch = u16s(n);
            let r = develop_with_scratch_into(sensor, plane, &mut dst, &mut scratch, &options);
            dst.extend_from_slice(&scratch);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorInto => {
            let mut dst = u16s(out_len() * 3);
            let r = develop_color_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorF32Into => {
            let mut dst = f32s(out_len() * 3);
            let r = develop_color_f32_into(sensor, plane, &mut dst);
            (r, bits(dst))
        }
        Entry::DevelopColorIntoThreaded => {
            let mut dst = u16s(out_len() * 3);
            let r = develop_color_into_threaded(sensor, plane, &mut dst, 4);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorF32IntoThreaded => {
            let mut dst = f32s(out_len() * 3);
            let r = develop_color_f32_into_threaded(sensor, plane, &mut dst, 4);
            (r, bits(dst))
        }
        Entry::ToSrgbLinearInto => {
            // Camera-space RGB of the raw plane's size; the values only need
            // to be deterministic.
            let camera: Vec<u16> = (0..raw3).map(|i| (i % 4096) as u16).collect();
            let mut dst = u16s(raw3);
            let r = to_srgb_linear_into(sensor, &camera, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DemosaicInto => {
            let mut dst = u16s(raw3);
            let r = demosaic_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DemosaicIntoThreaded => {
            let mut dst = u16s(raw3);
            let r = demosaic_into_threaded(sensor, plane, &mut dst, 4);
            (r, Written::U16(dst))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The refusal table (SPEC-086 AC1).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expected {
    Tag(u16),
    Version([u8; 4]),
}

#[track_caller]
fn assert_refused(r: &Result<(), Error>, expected: Expected, context: &str) {
    let ok = match (r, expected) {
        (Err(Error::UnsupportedTag { tag }), Expected::Tag(want)) => *tag == want,
        (Err(Error::UnsupportedDngVersion { backward_version }), Expected::Version(want)) => {
            *backward_version == want
        }
        _ => false,
    };
    assert!(ok, "{context}: expected {expected:?}, got {r:?}");
    let err = r.as_ref().expect_err("checked above");
    assert_eq!(err.kind(), ErrorKind::Unsupported, "{context}");
}

/// A change to a fixture's description.
type Modify = fn(&mut DngDescription);

struct Row {
    name: &'static str,
    modify: Modify,
    expected: Expected,
    /// The new `Sensor` field holds the written value (AC4).
    field_holds: fn(&Sensor) -> bool,
}

fn refusal_rows() -> Vec<Row> {
    vec![
        Row {
            name: "a: SampleFormat = 3",
            modify: |d| add(d, shorts(SAMPLE_FORMAT, &[3])),
            expected: Expected::Tag(SAMPLE_FORMAT),
            field_holds: |s| s.sample_format == Some(vec![3]),
        },
        Row {
            name: "b: SampleFormat = 2",
            modify: |d| add(d, shorts(SAMPLE_FORMAT, &[2])),
            expected: Expected::Tag(SAMPLE_FORMAT),
            field_holds: |s| s.sample_format == Some(vec![2]),
        },
        Row {
            name: "c: LinearizationTable = identity",
            modify: |d| {
                let identity: Vec<u16> = (0..=u16::MAX).collect();
                add(d, shorts(LINEARIZATION_TABLE, &identity));
            },
            expected: Expected::Tag(LINEARIZATION_TABLE),
            field_holds: |s| s.linearization_table == Some((0..=u16::MAX).collect::<Vec<u16>>()),
        },
        Row {
            name: "d: BlackLevelDeltaH = [0, ..., 0, 1/2]",
            modify: |d| {
                let mut v = zeros(W as usize - 1);
                v.push((1, 2));
                add(d, srationals(BLACK_LEVEL_DELTA_H, &v));
            },
            expected: Expected::Tag(BLACK_LEVEL_DELTA_H),
            field_holds: |s| {
                let mut want = vec![0.0; W as usize - 1];
                want.push(0.5);
                s.black_level_delta_h == Some(want)
            },
        },
        Row {
            name: "e: BlackLevelDeltaV = [-1/1, 0, ...]",
            modify: |d| {
                let mut v = vec![(-1, 1)];
                v.extend(zeros(H as usize - 1));
                add(d, srationals(BLACK_LEVEL_DELTA_V, &v));
            },
            expected: Expected::Tag(BLACK_LEVEL_DELTA_V),
            field_holds: |s| {
                let mut want = vec![-1.0];
                want.extend(vec![0.0; H as usize - 1]);
                s.black_level_delta_v == Some(want)
            },
        },
        Row {
            name: "f: SubTileBlockSize = [2, 2]",
            modify: |d| add(d, shorts(SUB_TILE_BLOCK_SIZE, &[2, 2])),
            expected: Expected::Tag(SUB_TILE_BLOCK_SIZE),
            field_holds: |s| s.sub_tile_block_size == Some([2, 2]),
        },
        Row {
            name: "g: RowInterleaveFactor = 2",
            modify: |d| add(d, shorts(ROW_INTERLEAVE_FACTOR, &[2])),
            expected: Expected::Tag(ROW_INTERLEAVE_FACTOR),
            field_holds: |s| s.row_interleave_factor == Some(2),
        },
        Row {
            name: "h: ColumnInterleaveFactor = 2",
            modify: |d| add(d, shorts(COLUMN_INTERLEAVE_FACTOR, &[2])),
            expected: Expected::Tag(COLUMN_INTERLEAVE_FACTOR),
            field_holds: |s| s.column_interleave_factor == Some(2),
        },
        Row {
            name: "i: DNGBackwardVersion = 1.7.1.1",
            modify: |d| d.meta.dng_backward_version = Some([1, 7, 1, 1]),
            expected: Expected::Version([1, 7, 1, 1]),
            field_holds: |s| s.dng_backward_version == Some([1, 7, 1, 1]),
        },
    ]
}

// Shown red first: written before the implementation, this file did not
// compile (no `UnsupportedTag`, no `UnsupportedDngVersion`, no new `Sensor`
// fields), and the AC9 mutants in the handback each turn one test red.

#[test]
fn ac1_every_entry_refuses_each_unhonoured_tag() {
    for row in refusal_rows() {
        for entry in ENTRIES {
            let fx = fixture_for(entry, &row.modify);
            let (r, written) = run(entry, &fx);
            let context = format!("{} on {entry:?}", row.name);
            assert_refused(&r, row.expected, &context);
            assert!(
                written.untouched(),
                "{context}: dst (and scratch) must be untouched by a refusal"
            );
        }
    }
}

#[test]
fn ac1_first_offending_tag_is_reported_in_ascending_order() {
    // Row j: two tags at once, written in DESCENDING order so the file's own
    // order cannot be what the check follows.
    let row_j = |d: &mut DngDescription| {
        add(d, shorts(COLUMN_INTERLEAVE_FACTOR, &[2]));
        add(d, shorts(SAMPLE_FORMAT, &[3]));
    };
    // Row k: the version guard runs before the tag set.
    let row_k = |d: &mut DngDescription| {
        d.meta.dng_backward_version = Some([2, 0, 0, 0]);
        add(d, shorts(LINEARIZATION_TABLE, &[0, 1, 2, 3]));
    };
    for entry in ENTRIES {
        let (r, written) = run(entry, &fixture_for(entry, &row_j));
        assert_refused(&r, Expected::Tag(SAMPLE_FORMAT), &format!("j on {entry:?}"));
        assert!(written.untouched(), "j on {entry:?}");

        let (r, written) = run(entry, &fixture_for(entry, &row_k));
        assert_refused(
            &r,
            Expected::Version([2, 0, 0, 0]),
            &format!("k on {entry:?}"),
        );
        assert!(written.untouched(), "k on {entry:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: honoured values are exact no-ops.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_honoured_values_develop_byte_identical() {
    let honoured: Vec<(&str, Modify)> = vec![
        ("SampleFormat = 1", |d| add(d, shorts(SAMPLE_FORMAT, &[1]))),
        ("BlackLevelDeltaH all zero", |d| {
            add(d, srationals(BLACK_LEVEL_DELTA_H, &zeros(W as usize)));
        }),
        ("BlackLevelDeltaV all zero", |d| {
            add(d, srationals(BLACK_LEVEL_DELTA_V, &zeros(H as usize)));
        }),
        ("SubTileBlockSize = [1, 1]", |d| {
            add(d, shorts(SUB_TILE_BLOCK_SIZE, &[1, 1]));
        }),
        ("RowInterleaveFactor = 1", |d| {
            add(d, shorts(ROW_INTERLEAVE_FACTOR, &[1]));
        }),
        ("ColumnInterleaveFactor = 1", |d| {
            add(d, shorts(COLUMN_INTERLEAVE_FACTOR, &[1]));
        }),
        ("DNGBackwardVersion = 1.7.1.0", |d| {
            d.meta.dng_backward_version = Some([1, 7, 1, 0]);
        }),
    ];
    for entry in ENTRIES {
        let (base_r, base) = run(entry, &fixture_for(entry, &|_| {}));
        assert!(base_r.is_ok(), "control {entry:?}: {base_r:?}");
        for (name, modify) in &honoured {
            let (r, written) = run(entry, &fixture_for(entry, modify));
            assert!(r.is_ok(), "{name} on {entry:?} must develop: {r:?}");
            assert!(
                written == base,
                "{name} on {entry:?}: output differs from the fixture without the tag"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: the backward version's default from DNGVersion.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac3_backward_version_defaults_from_dng_version() {
    let versions = |dng: Option<[u8; 4]>, backward: Option<[u8; 4]>| {
        move |d: &mut DngDescription| {
            d.meta.dng_version = dng;
            d.meta.dng_backward_version = backward;
        }
    };
    for entry in ENTRIES {
        // Absent backward, DNGVersion 1.8.0.0: the default is 1.8.0.0.
        let (r, written) = run(
            entry,
            &fixture_for(entry, &versions(Some([1, 8, 0, 0]), None)),
        );
        assert_refused(
            &r,
            Expected::Version([1, 8, 0, 0]),
            &format!("DNGVersion 1.8.0.0 on {entry:?}"),
        );
        assert!(written.untouched(), "{entry:?}");

        // Absent backward, DNGVersion 1.7.9.9: the default is 1.7.0.0.
        let (r, _) = run(
            entry,
            &fixture_for(entry, &versions(Some([1, 7, 9, 9]), None)),
        );
        assert!(r.is_ok(), "DNGVersion 1.7.9.9 on {entry:?}: {r:?}");

        // Both absent: no check.
        let (r, _) = run(entry, &fixture_for(entry, &versions(None, None)));
        assert!(r.is_ok(), "no versions on {entry:?}: {r:?}");

        // Present backward 1.4.0.0 wins over DNGVersion 9.0.0.0.
        let (r, _) = run(
            entry,
            &fixture_for(entry, &versions(Some([9, 0, 0, 0]), Some([1, 4, 0, 0]))),
        );
        assert!(
            r.is_ok(),
            "backward 1.4 under DNGVersion 9 on {entry:?}: {r:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4: metadata and plane reads still succeed.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac4_metadata_and_plane_reads_still_succeed() {
    for cfa in [false, true] {
        let base = load(&base_desc(cfa));
        // The untouched fixture has none of the new fields, except the two
        // versions the writer always emits.
        assert_eq!(base.sensor.sample_format, None);
        assert_eq!(base.sensor.linearization_table, None);
        assert_eq!(base.sensor.black_level_delta_h, None);
        assert_eq!(base.sensor.black_level_delta_v, None);
        assert_eq!(base.sensor.sub_tile_block_size, None);
        assert_eq!(base.sensor.row_interleave_factor, None);
        assert_eq!(base.sensor.column_interleave_factor, None);
        assert_eq!(base.sensor.dng_version, Some([1, 4, 0, 0]));
        assert_eq!(base.sensor.dng_backward_version, Some([1, 1, 0, 0]));

        for row in refusal_rows() {
            let mut d = base_desc(cfa);
            (row.modify)(&mut d);
            // `load` asserts `sensor()` and `unpack_into` are both `Ok`.
            let fx = load(&d);
            assert!(
                (row.field_holds)(&fx.sensor),
                "{} (cfa {cfa}): the field does not hold the written value",
                row.name
            );
            assert!(
                fx.sensor.malformed_tags.is_empty(),
                "{} (cfa {cfa}): {:?}",
                row.name,
                fx.sensor.malformed_tags
            );
            assert_eq!(
                fx.plane, base.plane,
                "{} (cfa {cfa}): the plane read must not change",
                row.name
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: a malformed tag of the set is refused.
// ─────────────────────────────────────────────────────────────────────────────

/// `Debug` of the field `tag` fills, so "is `None`" reads the same for every
/// field type.
fn field_debug(s: &Sensor, tag: u16) -> String {
    match tag {
        SAMPLE_FORMAT => format!("{:?}", s.sample_format),
        LINEARIZATION_TABLE => format!("{:?}", s.linearization_table),
        BLACK_LEVEL_DELTA_H => format!("{:?}", s.black_level_delta_h),
        BLACK_LEVEL_DELTA_V => format!("{:?}", s.black_level_delta_v),
        SUB_TILE_BLOCK_SIZE => format!("{:?}", s.sub_tile_block_size),
        ROW_INTERLEAVE_FACTOR => format!("{:?}", s.row_interleave_factor),
        COLUMN_INTERLEAVE_FACTOR => format!("{:?}", s.column_interleave_factor),
        DNG_BACKWARD_VERSION => format!("{:?}", s.dng_backward_version),
        other => panic!("no field for tag {other}"),
    }
}

#[test]
fn ac6_a_malformed_unhonoured_tag_is_refused() {
    let mut cases: Vec<(String, u16, RawTag)> = TAG_SET
        .iter()
        .map(|&tag| (format!("{tag} as ASCII"), tag, ascii(tag)))
        .collect();
    let mut zero_den = zeros(W as usize);
    zero_den[3] = (1, 0);
    cases.push((
        "BlackLevelDeltaH with a zero denominator".to_string(),
        BLACK_LEVEL_DELTA_H,
        srationals(BLACK_LEVEL_DELTA_H, &zero_den),
    ));
    cases.push((
        "DNGBackwardVersion with count 3".to_string(),
        DNG_BACKWARD_VERSION,
        RawTag {
            ifd: TagIfd::Ifd0,
            tag: DNG_BACKWARD_VERSION,
            field_type: TYPE_BYTE,
            count: 3,
            bytes: vec![1, 1, 0],
        },
    ));

    for (name, tag, raw) in cases {
        for entry in [Entry::DevelopInto, Entry::DevelopColorInto] {
            let raw = raw.clone();
            let fx = fixture_for(entry, &move |d| add(d, raw.clone()));
            assert!(
                fx.sensor.malformed_tags.contains(&tag),
                "{name}: malformed_tags {:?} must name {tag}",
                fx.sensor.malformed_tags
            );
            assert_eq!(field_debug(&fx.sensor, tag), "None", "{name}");
            let (r, written) = run(entry, &fx);
            assert_refused(&r, Expected::Tag(tag), &format!("{name} on {entry:?}"));
            assert!(written.untouched(), "{name} on {entry:?}");
        }
    }
}

#[test]
fn unsupported_tag_display_names_the_tag() {
    let msg = Error::UnsupportedTag { tag: 50712 }.to_string();
    assert!(msg.contains("LinearizationTable"), "{msg}");
    assert!(msg.contains("50712"), "{msg}");
    let msg = Error::UnsupportedDngVersion {
        backward_version: [1, 8, 0, 0],
    }
    .to_string();
    assert!(msg.contains("1.8.0.0"), "{msg}");
    assert!(msg.contains("1.7.1.0"), "{msg}");
}
