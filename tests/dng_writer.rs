//! `SPEC-057` — the test-only DNG writer (`tests/support/dng_writer.rs`) and
//! its oracles.
//!
//! The writer is checked three ways, cheapest-independent first (STAGE-013
//! verification plan):
//!
//! 1. **dnglab** (`dnglab analyze --raw-checksum`), which never goes through
//!    irradiance's reader: its checksum must equal the MD5 of the input samples
//!    as little-endian `u16` (`docs/oracle-contract.md` §"The plane contract"),
//!    for strips and for tiles with padded edge tiles, under II and MM.
//!    Tool-gated: when dnglab is absent the test skips loudly, naming it.
//! 2. **Round-trip through irradiance** for strip layouts:
//!    `Container::parse → sensor → plane::unpack_into` gives back the input
//!    samples, and the `Sensor` fields equal the description.
//! 3. **exiftool** reads the DNG identity tags back. Tool-gated.
//!
//! The red-proof (`oracle-must-be-shown-red`, `DEC-009`) runs the honest
//! negative control first, then shows each `WriterFault` turning the oracles
//! red on the same description.
//!
//! Tier A: every fixture is synthetic, own work (`DEC-003`); nothing is
//! written to the repo — tool-gated cases write to the OS temp dir.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/tiff.rs"]
mod tiff;
#[path = "support/tools.rs"]
mod tools;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag,
    TagIfd, WriteError, WriterFault, TAG_TILE_BYTE_COUNTS, TAG_TILE_LENGTH, TAG_TILE_OFFSETS,
    TAG_TILE_WIDTH,
};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{
    ActiveArea, Compression, Container, DefaultCropOrigin, DefaultCropSize, Rational, Sensor,
    TAG_OPCODE_LIST_2,
};
use irradiance::plane::unpack_into;
use irradiance::Error;
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// A deterministic, non-palindromic sample ramp below `2^bits`: every sample is
/// distinct from its neighbours and from its own byte- and bit-reversal for
/// almost every index, so a byte-order or bit-order fault cannot hide.
fn samples(width: u32, height: u32, bits: u32) -> Vec<u16> {
    let max = (1u32 << bits) - 1;
    (0..width * height)
        .map(|i| ((i.wrapping_mul(2_654_435_761) >> 7) % (max + 1)) as u16)
        .collect()
}

fn rggb() -> PlaneKind {
    PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 1, 2],
        plane_color: vec![0, 1, 2],
    }
}

/// A strip description with default metadata.
fn strip_desc(
    order: Order,
    bits: u32,
    kind: PlaneKind,
    width: u32,
    height: u32,
    rows_per_strip: u32,
) -> DngDescription {
    DngDescription {
        order,
        layout: IfdLayout::SensorInIfd0,
        width,
        height,
        bits_per_sample: bits,
        kind,
        organisation: Organisation::Strips { rows_per_strip },
        payload: Payload::Samples(samples(width, height, bits)),
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

fn tile_desc(order: Order, bits: u32, width: u32, height: u32, tile: u32) -> DngDescription {
    DngDescription {
        organisation: Organisation::Tiles {
            tile_width: tile,
            tile_length: tile,
            edge_fill: 0xBEEF & ((1u32 << bits) - 1) as u16,
        },
        ..strip_desc(order, bits, rggb(), width, height, height)
    }
}

fn input_samples(d: &DngDescription) -> &[u16] {
    match &d.payload {
        Payload::Samples(s) => s,
        Payload::Encoded { .. } => panic!("not a Samples payload"),
    }
}

/// MD5 of samples as little-endian `u16`, lowercase hex — the oracle's form.
fn md5_le(samples: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    md5::to_hex(&md5::hash(&bytes))
}

/// Parse the file, pick its sensor, unpack it.
fn read_back(file: &[u8]) -> Result<(Sensor, Vec<u16>), Error> {
    let container = Container::parse(file)?;
    let sensor = container.sensor()?;
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), file, &mut plane)?;
    Ok((sensor, plane))
}

/// Write `bytes` to a fresh file under the OS temp dir. Never in the repo.
fn temp_file(bytes: &[u8], label: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "irradiance-spec057-{}-{n}-{label}.dng",
        std::process::id()
    ));
    std::fs::write(&path, bytes).expect("write temp DNG");
    path
}

/// `dnglab analyze --raw-checksum`, black-box. `Err` carries why there is no
/// checksum (a non-zero exit or an unparseable output) — for the red-proof an
/// `Err` on a faulted file is as red as a wrong checksum.
fn dnglab_raw_checksum(bytes: &[u8], label: &str) -> Result<String, String> {
    let path = temp_file(bytes, label);
    let out = tools::run_tool(
        "dnglab",
        &["analyze".into(), "--raw-checksum".into()],
        &path,
    )
    .map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&path);
    let out = out?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        return Err(format!(
            "dnglab exited {:?}: {}{}",
            out.status.code(),
            stdout,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    // The checksum is the one 32-hex-digit token on stdout.
    let hexes: Vec<&str> = stdout
        .split(|c: char| !c.is_ascii_hexdigit())
        .filter(|t| t.len() == 32)
        .collect();
    match hexes.as_slice() {
        [one] => Ok(one.to_ascii_lowercase()),
        _ => Err(format!(
            "no unique 32-hex checksum in dnglab output: {stdout:?}"
        )),
    }
}

/// `true` when dnglab is on PATH; otherwise prints a loud skip naming it.
fn dnglab_or_skip(test: &str) -> bool {
    if tools::dnglab_available() {
        true
    } else {
        eprintln!("SKIP {test}: dnglab is not on PATH — the independent writer oracle did not run");
        false
    }
}

/// The AC1/AC4 case table: {8, 12, 14, 16} × {LinearRaw, Cfa2x2} × {II, MM} ×
/// {1 strip, 3 strips}. 8×6 keeps every row a whole number of bytes at every
/// depth (8 × 12 = 96, 8 × 14 = 112 bits) and gives a remainder strip
/// (rows_per_strip 2 → 3 strips; also exercised with 4 → 2 strips + remainder
/// by the metadata tests' larger planes).
fn strip_cases() -> Vec<(String, DngDescription)> {
    let mut cases = Vec::new();
    for bits in [8u32, 12, 14, 16] {
        for (kind_name, cfa) in [("mono", false), ("cfa", true)] {
            for order in [Order::Little, Order::Big] {
                for (strips_name, rps) in [("1strip", 6u32), ("3strips", 2)] {
                    let kind = if cfa { rggb() } else { PlaneKind::LinearRaw };
                    let name = format!("{bits}bit-{kind_name}-{order:?}-{strips_name}");
                    cases.push((name, strip_desc(order, bits, kind, 8, 6, rps)));
                }
            }
        }
    }
    cases
}

/// A fully-populated `DngMeta` exercising every field the reader models.
fn full_meta() -> DngMeta {
    let m = |v: &[(i32, i32)]| Some(v.to_vec());
    DngMeta {
        black_level: Some(vec![256, 257, 258, 259]),
        black_level_repeat_dim: Some([2, 2]),
        white_level: Some(vec![15000]),
        active_area: Some([2, 4, 14, 28]),
        default_crop_origin: Some([2, 1]),
        default_crop_size: Some([20, 10]),
        orientation: Some(6),
        color_matrix_1: m(&[
            (6722, 10000),
            (-635, 10000),
            (-963, 10000),
            (-4287, 10000),
            (12460, 10000),
            (2028, 10000),
            (-908, 10000),
            (2162, 10000),
            (5668, 10000),
        ]),
        color_matrix_2: m(&[
            (1, 2),
            (-3, 4),
            (5, 6),
            (-7, 8),
            (9, 10),
            (-11, 12),
            (13, 14),
            (-15, 16),
            (17, 18),
        ]),
        forward_matrix_1: m(&[
            (7978, 10000),
            (1352, 10000),
            (313, 10000),
            (2880, 10000),
            (7099, 10000),
            (21, 10000),
            (0, 10000),
            (8, 10000),
            (8243, 10000),
        ]),
        forward_matrix_2: m(&[
            (1, 1),
            (0, 1),
            (0, 1),
            (0, 1),
            (1, 1),
            (0, 1),
            (0, 1),
            (0, 1),
            (1, 1),
        ]),
        as_shot_neutral: Some(vec![(4875, 10000), (10000, 10000), (6521, 10000)]),
        calibration_illuminant_1: Some(17),
        calibration_illuminant_2: Some(21),
        ..DngMeta::default()
    }
}

fn rationals(v: &[(i32, i32)]) -> Vec<Rational> {
    v.iter().map(|&(num, den)| Rational { num, den }).collect()
}

fn assert_meta_round_trips(layout: IfdLayout) {
    for order in [Order::Little, Order::Big] {
        let (w, h) = (32u32, 16u32);
        let meta = full_meta();
        let d = DngDescription {
            layout,
            meta: meta.clone(),
            ..strip_desc(order, 14, rggb(), w, h, 6)
        };
        let file = write_dng(&d).expect("write");
        let container = Container::parse(&file).expect("parse");
        let candidates = container.sensor_candidates();
        assert_eq!(candidates.len(), 1, "{layout:?} {order:?}: one sensor IFD");
        if layout == IfdLayout::PreviewIfd0WithSensorSubIfd {
            assert_ne!(
                candidates[0], 0,
                "{order:?}: the sensor is the SubIFD, not IFD0"
            );
        }
        let (s, plane) = read_back(&file).expect("read back");
        assert_eq!(plane, input_samples(&d), "{layout:?} {order:?}: plane");

        let ctx = format!("{layout:?} {order:?}");
        assert_eq!((s.width, s.height, s.bits_per_sample), (w, h, 14), "{ctx}");
        assert_eq!(s.compression, Compression::Uncompressed, "{ctx}");
        assert_eq!(s.rows_per_strip, Some(6), "{ctx}");
        assert_eq!(s.strip_offsets.len(), 3, "{ctx}: 16 rows / 6 = 3 strips");
        assert_eq!(s.black_level, Some(256), "{ctx}");
        assert_eq!(s.black_levels, vec![256, 257, 258, 259], "{ctx}");
        assert_eq!(s.black_level_repeat_dim, Some([2, 2]), "{ctx}");
        assert_eq!(s.white_level, Some(15000), "{ctx}");
        assert_eq!(s.white_levels, vec![15000], "{ctx}");
        assert_eq!(
            s.active_area,
            Some(ActiveArea {
                top: 2,
                left: 4,
                bottom: 14,
                right: 28
            }),
            "{ctx}"
        );
        assert_eq!(
            s.default_crop_origin,
            Some(DefaultCropOrigin { x: 2, y: 1 }),
            "{ctx}"
        );
        assert_eq!(
            s.default_crop_size,
            Some(DefaultCropSize {
                width: 20,
                height: 10
            }),
            "{ctx}"
        );
        assert_eq!(s.orientation, Some(6), "{ctx}");
        let cfa = s.color_filter.as_ref().expect("CFA");
        assert_eq!(cfa.repeat_dim, [2, 2], "{ctx}");
        assert_eq!(cfa.pattern, vec![0, 1, 1, 2], "{ctx}");
        assert_eq!(cfa.plane_color, vec![0, 1, 2], "{ctx}");
        assert_eq!(cfa.layout, 1, "{ctx}");
        assert_eq!(
            s.color_matrix_1,
            meta.color_matrix_1.as_deref().map(rationals),
            "{ctx}"
        );
        assert_eq!(
            s.color_matrix_2,
            meta.color_matrix_2.as_deref().map(rationals),
            "{ctx}"
        );
        assert_eq!(
            s.forward_matrix_1,
            meta.forward_matrix_1.as_deref().map(rationals),
            "{ctx}"
        );
        assert_eq!(
            s.forward_matrix_2,
            meta.forward_matrix_2.as_deref().map(rationals),
            "{ctx}"
        );
        let neutral: Vec<(i32, i32)> = meta
            .as_shot_neutral
            .as_ref()
            .unwrap()
            .iter()
            .map(|&(n, d)| (n as i32, d as i32))
            .collect();
        assert_eq!(s.as_shot_neutral, Some(rationals(&neutral)), "{ctx}");
        assert_eq!(s.calibration_illuminant_1, Some(17), "{ctx}");
        assert_eq!(s.calibration_illuminant_2, Some(21), "{ctx}");
        assert!(s.malformed_tags.is_empty(), "{ctx}: {:?}", s.malformed_tags);
        assert_eq!(s.opcode_lists, [false, false, false], "{ctx}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — strip round-trip, every depth
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn strip_round_trip_all_depths_both_orders() {
    let cases = strip_cases();
    assert_eq!(
        cases.len(),
        32,
        "4 depths × 2 kinds × 2 orders × 2 strip counts"
    );
    for (name, d) in &cases {
        let file = write_dng(d).unwrap_or_else(|e| panic!("{name}: write failed: {e:?}"));
        let (sensor, plane) = read_back(&file).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(plane, input_samples(d), "{name}: plane differs");
        assert_eq!(sensor.bits_per_sample, d.bits_per_sample, "{name}");
        let expected_strips = if name.ends_with("3strips") { 3 } else { 1 };
        assert_eq!(sensor.strip_offsets.len(), expected_strips, "{name}");
        assert_eq!(
            sensor.color_filter.is_some(),
            matches!(d.kind, PlaneKind::Cfa { .. }),
            "{name}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — metadata round-trip, both layouts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn metadata_round_trips_in_ifd0_layout() {
    assert_meta_round_trips(IfdLayout::SensorInIfd0);
}

#[test]
fn metadata_round_trips_in_subifd_layout() {
    assert_meta_round_trips(IfdLayout::PreviewIfd0WithSensorSubIfd);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — opcode bytes
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn opcode_lists_round_trip_including_inline() {
    // List 1 is 3 bytes (inline, and an odd length), list 2 exactly 4 (inline,
    // the boundary), list 3 longer than 4 (out of line, odd length so the next
    // value's word alignment is exercised).
    let lists: [Vec<u8>; 3] = [
        vec![0xA1, 0xB2, 0xC3],
        vec![0x01, 0x02, 0x03, 0x04],
        (0u8..=40).map(|b| b.wrapping_mul(37)).collect(),
    ];
    for layout in [
        IfdLayout::SensorInIfd0,
        IfdLayout::PreviewIfd0WithSensorSubIfd,
    ] {
        for order in [Order::Little, Order::Big] {
            let d = DngDescription {
                layout,
                opcode_lists: [
                    Some(lists[0].clone()),
                    Some(lists[1].clone()),
                    Some(lists[2].clone()),
                ],
                ..strip_desc(order, 16, PlaneKind::LinearRaw, 4, 4, 4)
            };
            let file = write_dng(&d).expect("write");
            let (s, plane) = read_back(&file).expect("read back");
            let ctx = format!("{layout:?} {order:?}");
            assert_eq!(plane, input_samples(&d), "{ctx}");
            assert_eq!(s.opcode_lists, [true, true, true], "{ctx}");
            assert_eq!(s.opcode_list_1.as_deref(), Some(&lists[0][..]), "{ctx}");
            assert_eq!(s.opcode_list_2.as_deref(), Some(&lists[1][..]), "{ctx}");
            assert_eq!(s.opcode_list_3.as_deref(), Some(&lists[2][..]), "{ctx}");
            assert!(s.malformed_tags.is_empty(), "{ctx}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 / AC5 — dnglab, the independent oracle
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dnglab_checksum_matches_strips() {
    if !dnglab_or_skip("dnglab_checksum_matches_strips") {
        return;
    }
    let cases = strip_cases();
    for (name, d) in &cases {
        let file = write_dng(d).expect("write");
        let expected = md5_le(input_samples(d));
        let got = dnglab_raw_checksum(&file, name)
            .unwrap_or_else(|e| panic!("{name}: dnglab gave no checksum: {e}"));
        assert_eq!(got, expected, "{name}: dnglab checksum differs");
    }
    // Quoted in the HANDOFF-127 handback: dnglab's own output, not ours.
    let (name, d) = &cases[31];
    let got = dnglab_raw_checksum(&write_dng(d).unwrap(), name).unwrap();
    eprintln!("dnglab strip checksum, {name}: {got}");
}

#[test]
fn dnglab_checksum_matches_tiles_with_edge_tiles() {
    if !dnglab_or_skip("dnglab_checksum_matches_tiles_with_edge_tiles") {
        return;
    }
    // 40×20 in 16×16 tiles: 3×2 tiles, the right column 8 wide and the bottom
    // row 4 tall are partial and padded with the edge fill. 12-bit needs
    // `tile_width × 12` divisible by 8 — 16 × 12 = 192 bits.
    for bits in [16u32, 12] {
        for order in [Order::Little, Order::Big] {
            let name = format!("tiles-{bits}bit-{order:?}");
            let d = tile_desc(order, bits, 40, 20, 16);
            let file = write_dng(&d).expect("write");
            let expected = md5_le(input_samples(&d));
            let got = dnglab_raw_checksum(&file, &name)
                .unwrap_or_else(|e| panic!("{name}: dnglab gave no checksum: {e}"));
            assert_eq!(got, expected, "{name}: dnglab checksum differs");
            eprintln!("dnglab tile checksum, {name}: {got}");
        }
    }
}

#[test]
fn tile_tags_read_back_from_container() {
    for order in [Order::Little, Order::Big] {
        for bits in [16u32, 12] {
            let d = tile_desc(order, bits, 40, 20, 16);
            let file = write_dng(&d).expect("write");
            let c = Container::parse(&file).expect("parse");
            let candidates = c.sensor_candidates();
            assert_eq!(candidates.len(), 1);
            let ifd = &c.ifds()[candidates[0]];
            let ctx = format!("{order:?} {bits}-bit");
            assert_eq!(c.values(ifd, TAG_TILE_WIDTH).unwrap(), vec![16], "{ctx}");
            assert_eq!(c.values(ifd, TAG_TILE_LENGTH).unwrap(), vec![16], "{ctx}");
            let offsets = c.values(ifd, TAG_TILE_OFFSETS).unwrap();
            let counts = c.values(ifd, TAG_TILE_BYTE_COUNTS).unwrap();
            assert_eq!(offsets.len(), 6, "{ctx}: 3 × 2 tiles");
            let tile_bytes = 16 * 16 * bits / 8;
            assert_eq!(counts, vec![tile_bytes; 6], "{ctx}");
            // Every tile lies inside the file, word aligned, no two overlap.
            let mut spans: Vec<(u32, u32)> = offsets
                .iter()
                .zip(&counts)
                .map(|(&o, &n)| (o, o + n))
                .collect();
            spans.sort();
            for (o, end) in &spans {
                assert_eq!(o % 2, 0, "{ctx}: tile offset {o} not word aligned");
                assert!(*end as usize <= file.len(), "{ctx}: tile past EOF");
            }
            for w in spans.windows(2) {
                assert!(w[0].1 <= w[1].0, "{ctx}: tiles overlap {w:?}");
            }
            // The strip tags are absent on a tiled plane.
            assert!(
                c.values(ifd, 273).unwrap().is_empty(),
                "{ctx}: StripOffsets"
            );
            // The first tile's first row is the plane's first 16 samples,
            // container-order integers at 16-bit (DEC-008).
            if bits == 16 {
                let at = offsets[0] as usize;
                let first: Vec<u16> = (0..16)
                    .map(|i| {
                        let b = [file[at + 2 * i], file[at + 2 * i + 1]];
                        match order {
                            Order::Little => u16::from_le_bytes(b),
                            Order::Big => u16::from_be_bytes(b),
                        }
                    })
                    .collect();
                assert_eq!(first, input_samples(&d)[..16], "{ctx}");
                // The right edge tile (index 2) is 8 samples of plane then 8 of fill.
                let at = offsets[2] as usize + 2 * 8;
                let fill = [file[at], file[at + 1]];
                let fill = match order {
                    Order::Little => u16::from_le_bytes(fill),
                    Order::Big => u16::from_be_bytes(fill),
                };
                assert_eq!(fill, 0xBEEF, "{ctx}: edge fill");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — Encoded pass-through
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn encoded_payload_passes_through_lj92() {
    let (w, h, precision) = (16u32, 8u32, 14u8);
    let plane = samples(w, h, u32::from(precision));
    let chunk = tiff::lj92_strip(&plane, w as usize, h as usize, precision);
    for order in [Order::Little, Order::Big] {
        let d = DngDescription {
            payload: Payload::Encoded {
                compression: 7,
                chunks: vec![chunk.clone()],
            },
            ..strip_desc(order, u32::from(precision), PlaneKind::LinearRaw, w, h, h)
        };
        let file = write_dng(&d).expect("write");
        let (sensor, decoded) = read_back(&file).expect("read back");
        assert_eq!(sensor.compression, Compression::Jpeg, "{order:?}");
        assert_eq!(decoded, plane, "{order:?}: decoded samples");
        // The chunk is in the file byte for byte, and is the whole strip.
        assert_eq!(sensor.strip_byte_counts, vec![chunk.len() as u32]);
        let at = sensor.strip_offsets[0] as usize;
        assert_eq!(&file[at..at + chunk.len()], &chunk[..], "{order:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn write_errors_are_typed() {
    let base = || strip_desc(Order::Little, 16, rggb(), 8, 4, 4);

    // Sample count ≠ width × height.
    let mut d = base();
    d.payload = Payload::Samples(vec![0; 31]);
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::SampleCountMismatch {
            expected: 32,
            actual: 31
        })
    ));

    // A sample above 2^bits − 1.
    let mut d = strip_desc(Order::Little, 12, rggb(), 8, 4, 4);
    if let Payload::Samples(s) = &mut d.payload {
        s[5] = 4096;
    }
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::SampleOutOfRange {
            index: 5,
            value: 4096,
            bits: 12
        })
    ));

    // Bit depth outside {8, 12, 14, 16} with Samples.
    for bits in [0u32, 1, 10, 15, 17, 32] {
        let mut d = base();
        d.bits_per_sample = bits;
        assert!(
            matches!(write_dng(&d), Err(WriteError::UnsupportedBitDepth { bits: b }) if b == bits),
            "bits {bits}"
        );
    }

    // CFA pattern length ≠ repeat_dim product.
    let mut d = base();
    d.kind = PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 2],
        plane_color: vec![0, 1, 2],
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::CfaPatternLength {
            expected: 4,
            actual: 3
        })
    ));

    // Encoded chunk count ≠ strip count (8×4, rows_per_strip 2 → 2 strips).
    let mut d = strip_desc(Order::Little, 16, rggb(), 8, 4, 2);
    d.payload = Payload::Encoded {
        compression: 7,
        chunks: vec![vec![1, 2, 3]],
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::ChunkCountMismatch {
            expected: 2,
            actual: 1
        })
    ));
    // … and ≠ tile count (40×20 in 16×16 → 6 tiles).
    let mut d = tile_desc(Order::Little, 16, 40, 20, 16);
    d.payload = Payload::Encoded {
        compression: 7,
        chunks: vec![vec![0]; 5],
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::ChunkCountMismatch {
            expected: 6,
            actual: 5
        })
    ));

    // An offset or size that overflows u32, checked before anything is
    // allocated: a 65536 × 65536 plane in 1 × 1 tiles is 2^32 tiles, whose
    // count alone does not fit the u32 a TIFF count field holds.
    let mut d = base();
    d.width = 65_536;
    d.height = 65_536;
    d.payload = Payload::Encoded {
        compression: 7,
        chunks: vec![Vec::new()],
    };
    d.organisation = Organisation::Tiles {
        tile_width: 1,
        tile_length: 1,
        edge_fill: 0,
    };
    assert!(
        matches!(write_dng(&d), Err(WriteError::Overflow { .. })),
        "{:?}",
        write_dng(&d).map(|v| v.len())
    );

    // A row that is not a whole number of bytes, strips and tiles.
    let d = strip_desc(Order::Little, 14, PlaneKind::LinearRaw, 6, 4, 4);
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::RowNotWholeBytes {
            row_samples: 6,
            bits: 14
        })
    ));
    let mut d = tile_desc(Order::Little, 12, 40, 20, 16);
    d.organisation = Organisation::Tiles {
        tile_width: 5,
        tile_length: 16,
        edge_fill: 0,
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::RowNotWholeBytes {
            row_samples: 5,
            bits: 12
        })
    ));
    // …but 12-bit with an even row length is fine (6 × 12 = 72 bits).
    let d = strip_desc(Order::Little, 12, PlaneKind::LinearRaw, 6, 4, 4);
    assert!(write_dng(&d).is_ok());

    // Degenerate geometry is refused, never a divide-by-zero.
    let mut d = base();
    d.organisation = Organisation::Strips { rows_per_strip: 0 };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::InvalidGeometry { .. })
    ));
    let mut d = base();
    d.width = 0;
    d.payload = Payload::Samples(Vec::new());
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::InvalidGeometry { .. })
    ));
    let mut d = tile_desc(Order::Little, 16, 40, 20, 16);
    d.organisation = Organisation::Tiles {
        tile_width: 0,
        tile_length: 16,
        edge_fill: 0,
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::InvalidGeometry { .. })
    ));

    // An edge fill that is itself out of range for the depth.
    let mut d = tile_desc(Order::Little, 12, 40, 20, 16);
    d.organisation = Organisation::Tiles {
        tile_width: 16,
        tile_length: 16,
        edge_fill: 0xBEEF,
    };
    assert!(matches!(
        write_dng(&d),
        Err(WriteError::EdgeFillOutOfRange {
            value: 0xBEEF,
            bits: 12
        })
    ));

    // TIFF 6.0's "tile dimensions are multiples of 16" is NOT enforced.
    let mut d = tile_desc(Order::Little, 16, 40, 20, 16);
    d.organisation = Organisation::Tiles {
        tile_width: 10,
        tile_length: 7,
        edge_fill: 0,
    };
    assert!(write_dng(&d).is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 — the hostile escape hatch
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn extra_tag_overrides_generated_tag() {
    for layout in [
        IfdLayout::SensorInIfd0,
        IfdLayout::PreviewIfd0WithSensorSubIfd,
    ] {
        for order in [Order::Little, Order::Big] {
            let ctx = format!("{layout:?} {order:?}");
            // A generated, well-formed OpcodeList2 ...
            let honest = DngDescription {
                layout,
                opcode_lists: [None, Some(vec![0, 0, 0, 0, 9, 9, 9, 9]), None],
                ..strip_desc(order, 16, PlaneKind::LinearRaw, 4, 4, 4)
            };
            let (s, _) = read_back(&write_dng(&honest).unwrap()).unwrap();
            assert!(s.malformed_tags.is_empty(), "{ctx}: control is clean");

            // ... replaced by a malformed one: UNDEFINED, a count far past
            // what the file holds, and an offset word pointing at 0xFFFFFFFF.
            let hostile = DngDescription {
                extra_tags: vec![RawTag {
                    ifd: TagIfd::Sensor,
                    tag: TAG_OPCODE_LIST_2,
                    field_type: 7,
                    count: 0x00FF_FFFF,
                    bytes: vec![0xFF; 4],
                }],
                ..honest
            };
            let file = write_dng(&hostile).expect("write");
            let c = Container::parse(&file).expect("parse");
            let ifd = &c.ifds()[c.sensor_candidates()[0]];
            let entries: Vec<_> = ifd
                .entries()
                .iter()
                .filter(|e| e.tag() == TAG_OPCODE_LIST_2)
                .collect();
            assert_eq!(entries.len(), 1, "{ctx}: replaced, not appended");
            assert_eq!(entries[0].count(), 0x00FF_FFFF, "{ctx}");

            let (s, plane) = read_back(&file).expect("the plane still reads");
            assert!(
                s.malformed_tags.contains(&TAG_OPCODE_LIST_2),
                "{ctx}: {:?}",
                s.malformed_tags
            );
            assert_eq!(s.opcode_list_2, None, "{ctx}");
            let (ow, oh) = output_dimensions(&s).unwrap();
            let mut dst = vec![0u16; (ow * oh) as usize];
            assert!(
                matches!(
                    develop_into(&s, &plane, &mut dst),
                    Err(Error::UnreadableOpcodeList { list: 2 })
                ),
                "{ctx}: develop refuses (DEC-053)"
            );

            // An Ifd0-scoped extra tag that names a tag nothing generates is
            // appended (and, under the SubIFD layout, lands in IFD0).
            let appended = DngDescription {
                extra_tags: vec![RawTag {
                    ifd: TagIfd::Ifd0,
                    tag: 50_000,
                    field_type: 1,
                    count: 2,
                    bytes: vec![7, 8],
                }],
                ..strip_desc(order, 16, PlaneKind::LinearRaw, 4, 4, 4)
            };
            let appended = DngDescription { layout, ..appended };
            let file = write_dng(&appended).unwrap();
            let c = Container::parse(&file).unwrap();
            let ifd0 = c.ifd0().unwrap();
            let e = ifd0.entry(50_000).expect("appended to IFD0");
            assert_eq!(c.payload(e).unwrap(), &[7, 8], "{ctx}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC9 — red-proof
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn red_proof_writer_faults_turn_oracles_red() {
    // (name, honest description, fault, does irradiance read this layout?)
    let cases: Vec<(&str, DngDescription, WriterFault, bool)> = vec![
        (
            "16bit-strips-II-swap",
            strip_desc(Order::Little, 16, rggb(), 8, 6, 2),
            WriterFault::SwapByteAlignedOrder,
            true,
        ),
        (
            "16bit-strips-MM-swap",
            strip_desc(Order::Big, 16, rggb(), 8, 6, 2),
            WriterFault::SwapByteAlignedOrder,
            true,
        ),
        (
            "12bit-strips-lsb",
            strip_desc(Order::Little, 12, PlaneKind::LinearRaw, 8, 6, 2),
            WriterFault::LsbFirstPacking,
            true,
        ),
        (
            "14bit-strips-lsb",
            strip_desc(Order::Big, 14, rggb(), 8, 6, 6),
            WriterFault::LsbFirstPacking,
            true,
        ),
        (
            "16bit-tiles-offset",
            tile_desc(Order::Little, 16, 40, 20, 16),
            WriterFault::TileOffsetOffByOne,
            false,
        ),
        (
            "12bit-tiles-offset-MM",
            tile_desc(Order::Big, 12, 40, 20, 16),
            WriterFault::TileOffsetOffByOne,
            false,
        ),
        (
            "16bit-tiles-swap",
            tile_desc(Order::Big, 16, 40, 20, 16),
            WriterFault::SwapByteAlignedOrder,
            false,
        ),
        (
            "12bit-tiles-lsb",
            tile_desc(Order::Little, 12, 40, 20, 16),
            WriterFault::LsbFirstPacking,
            false,
        ),
    ];
    let have_dnglab = dnglab_or_skip("red_proof_writer_faults_turn_oracles_red (dnglab half)");

    // 1. Negative control FIRST (DEC-009): every honest file passes both oracles.
    for (name, d, _, readable) in &cases {
        assert!(d.fault.is_none());
        let file = write_dng(d).expect("write honest");
        if *readable {
            let (_, plane) = read_back(&file).expect("read honest");
            assert_eq!(plane, input_samples(d), "{name}: honest round-trip");
        }
        if have_dnglab {
            let got = dnglab_raw_checksum(&file, name).expect("honest dnglab");
            assert_eq!(got, md5_le(input_samples(d)), "{name}: honest dnglab");
        }
    }

    // 2. Each fault turns each applicable oracle red.
    for (name, d, fault, readable) in cases {
        let faulted = DngDescription {
            fault: Some(fault),
            ..d
        };
        let honest = DngDescription {
            fault: None,
            ..faulted.clone()
        };
        let bad = write_dng(&faulted).expect("write faulted");
        assert_ne!(
            bad,
            write_dng(&honest).unwrap(),
            "{name}: fault changed nothing"
        );
        if readable {
            // The read itself may succeed (it usually does: the fault keeps
            // every size); what must not happen is the honest samples.
            match read_back(&bad) {
                Ok((_, plane)) => {
                    assert_ne!(
                        plane,
                        input_samples(&faulted),
                        "{name}: round-trip stayed green"
                    )
                }
                Err(e) => eprintln!("{name}: faulted file refused by the reader: {e}"),
            }
        }
        if have_dnglab {
            match dnglab_raw_checksum(&bad, name) {
                Ok(sum) => assert_ne!(
                    sum,
                    md5_le(input_samples(&faulted)),
                    "{name}: dnglab stayed green"
                ),
                Err(e) => eprintln!("{name}: faulted file refused by dnglab: {e}"),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Design §2 — entries sorted by tag (added at verify, HANDOFF-128)
// ─────────────────────────────────────────────────────────────────────────────

/// TIFF 6.0 §2 requires each IFD's entries in ascending tag order, and Design
/// §2 makes it the writer's job. Neither irradiance's reader nor dnglab
/// rejects an unsorted IFD, so without this test removing the writer's sort
/// left every other test green (verify mutation M10). The parser keeps entries
/// in file order, so reading them back observes what was written. Covers both
/// layouts, both byte orders, strips and tiles, and an `extra_tags` append
/// (a tag the generator never writes, which must still land in order).
#[test]
fn ifd_entries_are_written_in_ascending_tag_order() {
    for layout in [
        IfdLayout::SensorInIfd0,
        IfdLayout::PreviewIfd0WithSensorSubIfd,
    ] {
        for order in [Order::Little, Order::Big] {
            let strips = DngDescription {
                layout,
                meta: full_meta(),
                opcode_lists: [Some(vec![1, 2]), None, Some(vec![3; 9])],
                extra_tags: vec![RawTag {
                    ifd: TagIfd::Sensor,
                    tag: 300,
                    field_type: 3,
                    count: 1,
                    bytes: vec![0, 0],
                }],
                ..strip_desc(order, 14, rggb(), 32, 16, 6)
            };
            let tiles = DngDescription {
                layout,
                ..tile_desc(order, 12, 40, 20, 16)
            };
            for (what, d) in [("strips", strips), ("tiles", tiles)] {
                let file = write_dng(&d).expect("write");
                let c = Container::parse(&file).expect("parse");
                let expected_ifds = match layout {
                    IfdLayout::SensorInIfd0 => 1,
                    IfdLayout::PreviewIfd0WithSensorSubIfd => 2,
                };
                assert_eq!(c.ifds().len(), expected_ifds, "{layout:?} {order:?} {what}");
                for (i, ifd) in c.ifds().iter().enumerate() {
                    let tags: Vec<u16> = ifd.entries().iter().map(|e| e.tag()).collect();
                    assert!(
                        tags.windows(2).all(|w| w[0] < w[1]),
                        "{layout:?} {order:?} {what} IFD {i}: entries not strictly ascending: {tags:?}"
                    );
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10 — determinism
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn write_is_deterministic() {
    let mut descs: Vec<DngDescription> = strip_cases().into_iter().map(|(_, d)| d).collect();
    descs.push(tile_desc(Order::Big, 12, 40, 20, 16));
    descs.push(DngDescription {
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        meta: full_meta(),
        opcode_lists: [Some(vec![1, 2]), None, Some(vec![3; 9])],
        ..strip_desc(Order::Little, 14, rggb(), 32, 16, 6)
    });
    for d in &descs {
        let a = write_dng(d).unwrap();
        let b = write_dng(&d.clone()).unwrap();
        assert_eq!(a, b);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC11 — exiftool sanity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn exiftool_reads_dng_identity_tags() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP exiftool_reads_dng_identity_tags: exiftool is not on PATH — the identity-tag sanity check did not run"
        );
        return;
    }
    for layout in [
        IfdLayout::SensorInIfd0,
        IfdLayout::PreviewIfd0WithSensorSubIfd,
    ] {
        for order in [Order::Little, Order::Big] {
            let ctx = format!("{layout:?} {order:?}");
            let mut d = strip_desc(order, 14, rggb(), 8, 6, 2);
            d.layout = layout;
            d.meta.unique_camera_model = Some("spec-057 writer".into());
            d.meta.dng_version = Some([1, 6, 0, 0]);
            let file = write_dng(&d).unwrap();
            let path = temp_file(&file, "exif");
            // -n numeric; -s3 bare values; one line per tag (no -T), so the
            // ASCII model, which contains spaces, is read whole.
            let args: Vec<String> = [
                "-n",
                "-s3",
                "-DNGVersion",
                "-UniqueCameraModel",
                "-BitsPerSample",
                "-CFAPattern2",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let out = tools::run_tool("exiftool", &args, &path).expect("exiftool");
            let _ = std::fs::remove_file(&path);
            let text = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = text.lines().collect();
            assert_eq!(
                lines,
                vec!["1 6 0 0", "spec-057 writer", "14", "0 1 1 2"],
                "{ctx}: exiftool said {text:?}"
            );
        }
    }
}
