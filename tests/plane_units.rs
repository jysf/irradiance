//! `SPEC-068` — plane units (`DEC-063`): `plane::units` yields full-width row
//! bands, and `plane::unpack_unit_into` decodes one into its own band of the
//! caller's plane, byte-identically in any order.
//!
//! Tier A (everything but the two `held_files` tests): every fixture is built
//! in-test, uncompressed and Compression-7 strips with the [`Dng`] builder below
//! (a copy of `tests/plane_validate.rs`'s, which can write a strip whose rows
//! are not whole bytes) and tiled planes with the SPEC-057 DNG writer.
//!
//! The oracles never run the code under test (AGENTS.md §16 rule 5):
//!
//! - tier A compares against the fixture's own input samples, which the
//!   writer packed independently of the reader;
//! - tier B compares against `unpack_into`'s plane only after proving that
//!   plane's MD5 equals the manifest's pre-existing `dnglab --raw-checksum`
//!   pin, so the assembled plane is held to the dnglab pin, not to itself.
//!
//! The unit counts asserted here are DEC-063's arithmetic (§1 of the spec), not
//! measurements.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/md5.rs"]
#[allow(dead_code)]
mod md5;
#[path = "support/tiff.rs"]
mod tiff;

use std::num::NonZeroU32;

use corpus::{CorpusRoot, Manifest};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, RawTag};
use irradiance::ifd::{ByteOrder, Container, Sensor, TAG_ROWS_PER_STRIP};
use irradiance::plane::{units, unpack_into, unpack_unit_into, Unit};
use irradiance::{Error, ErrorKind};
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// The strip builder (a copy of tests/plane_validate.rs's `Dng`)
// ─────────────────────────────────────────────────────────────────────────────

/// Where the out-of-line strip arrays go.
const ARRAYS_AT: u32 = 256;
/// Where the strip payload starts.
const PAYLOAD_BASE: u32 = 8192;

/// A one-IFD DNG whose strip tables are written exactly as given, so a test
/// can mutate any count, offset or `RowsPerStrip`.
#[derive(Clone, Debug)]
struct Dng {
    order: Order,
    compression: u16,
    width: u32,
    height: u32,
    bits: u16,
    rows_per_strip: Option<u32>,
    offsets: Vec<u32>,
    counts: Vec<u32>,
    payload: Vec<u8>,
}

impl Dng {
    fn contiguous(
        order: Order,
        compression: u16,
        (width, height): (u32, u32),
        bits: u16,
        rows_per_strip: Option<u32>,
        strips: &[Vec<u8>],
    ) -> Dng {
        let mut offsets = Vec::new();
        let mut counts = Vec::new();
        let mut payload = Vec::new();
        for s in strips {
            offsets.push(PAYLOAD_BASE + payload.len() as u32);
            counts.push(s.len() as u32);
            payload.extend_from_slice(s);
        }
        Dng {
            order,
            compression,
            width,
            height,
            bits,
            rows_per_strip,
            offsets,
            counts,
            payload,
        }
    }

    fn build(&self) -> Vec<u8> {
        let order = self.order;
        let mut entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, self.width),
            tiff::long(tiff::IMAGE_LENGTH, self.height),
            tiff::short(tiff::BITS_PER_SAMPLE, self.bits, order),
            tiff::short(tiff::COMPRESSION, self.compression, order),
            tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        ];
        if let Some(rows) = self.rows_per_strip {
            entries.push(tiff::long(TAG_ROWS_PER_STRIP, rows));
        }
        let counts_at = ARRAYS_AT + 4 * self.offsets.len() as u32;
        for (tag, vals, at) in [
            (tiff::STRIP_OFFSETS, &self.offsets, ARRAYS_AT),
            (tiff::STRIP_BYTE_COUNTS, &self.counts, counts_at),
        ] {
            match vals.len() {
                0 => {}
                1 => entries.push(tiff::long(tag, vals[0])),
                n => entries.push(tiff::at_offset(tag, 4, n as u32, at)),
            }
        }
        entries.sort_by_key(|e| e.tag);
        let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
        for (vals, at) in [(&self.offsets, ARRAYS_AT), (&self.counts, counts_at)] {
            if vals.len() > 1 {
                put_u32_array(&mut data, at as usize, vals, order);
            }
        }
        assert!(
            data.len() <= PAYLOAD_BASE as usize,
            "the strip arrays overran the payload region"
        );
        data.resize(PAYLOAD_BASE as usize, 0);
        data.extend_from_slice(&self.payload);
        data
    }
}

fn put_u32_array(data: &mut Vec<u8>, at: usize, vals: &[u32], order: Order) {
    let end = at + vals.len() * 4;
    if data.len() < end {
        data.resize(end, 0);
    }
    for (i, v) in vals.iter().enumerate() {
        let bytes = match order {
            Order::Little => v.to_le_bytes(),
            Order::Big => v.to_be_bytes(),
        };
        data[at + i * 4..at + i * 4 + 4].copy_from_slice(&bytes);
    }
}

/// A deterministic LCG ramp below `2^bits`.
fn samples(count: usize, bits: u32, seed: u32) -> Vec<u16> {
    let modulo = 1u32 << bits;
    let mut state = 0x0068_u32 ^ seed ^ bits;
    (0..count)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 9) % modulo) as u16
        })
        .collect()
}

/// Pack one strip: 8/16-bit as container-order integers, 12/14-bit as one
/// MSB-first bitstream with only the strip's final byte padded (`DEC-008`).
fn pack(samples: &[u16], bits: u32, order: Order) -> Vec<u8> {
    match bits {
        8 => samples.iter().map(|&s| s as u8).collect(),
        16 => samples
            .iter()
            .flat_map(|&s| match order {
                Order::Little => s.to_le_bytes(),
                Order::Big => s.to_be_bytes(),
            })
            .collect(),
        _ => {
            let mut out = Vec::new();
            let (mut acc, mut nbits) = (0u32, 0u32);
            for &s in samples {
                acc = (acc << bits) | u32::from(s);
                nbits += bits;
                while nbits >= 8 {
                    nbits -= 8;
                    out.push(((acc >> nbits) & 0xFF) as u8);
                }
            }
            if nbits > 0 {
                out.push(((acc << (8 - nbits)) & 0xFF) as u8);
            }
            out
        }
    }
}

/// An uncompressed strip plane and the samples it encodes.
fn uncompressed(
    order: Order,
    (width, height): (u32, u32),
    bits: u32,
    rows_per_strip: Option<u32>,
) -> (Dng, Vec<u16>) {
    let plane = samples((width * height) as usize, bits, width ^ (height << 8));
    let band = rows_per_strip.unwrap_or(height) as usize * width as usize;
    let strips: Vec<Vec<u8>> = plane
        .chunks(band.max(1))
        .map(|c| pack(c, bits, order))
        .collect();
    let dng = Dng::contiguous(
        order,
        1,
        (width, height),
        bits as u16,
        rows_per_strip,
        &strips,
    );
    (dng, plane)
}

/// A Compression-7 strip plane (one SOF-3 datastream per strip).
fn compression7(
    order: Order,
    (width, height): (u32, u32),
    precision: u8,
    nf: u8,
    rows_per_strip: Option<u32>,
) -> (Dng, Vec<u16>) {
    let plane = samples((width * height) as usize, u32::from(precision), 7);
    let rows = rows_per_strip.unwrap_or(height) as usize;
    let strips: Vec<Vec<u8>> = plane
        .chunks(rows * width as usize)
        .map(|band| {
            let band_rows = band.len() / width as usize;
            tiff::lj92_strip_components(band, width as usize, band_rows, precision, nf, 0x11)
        })
        .collect();
    let dng = Dng::contiguous(
        order,
        7,
        (width, height),
        u16::from(precision),
        rows_per_strip,
        &strips,
    );
    (dng, plane)
}

// ─────────────────────────────────────────────────────────────────────────────
// The tile builders (SPEC-057's writer, as tests/plane_tiles.rs uses it)
// ─────────────────────────────────────────────────────────────────────────────

/// A padding sentinel below `2^bits`.
fn fill(bits: u32) -> u16 {
    (0xBEEF & ((1u32 << bits) - 1)) as u16
}

#[allow(clippy::too_many_arguments)]
fn tiled(
    order: Order,
    width: u32,
    height: u32,
    bits: u32,
    tile_width: u32,
    tile_length: u32,
    payload: Payload,
) -> Vec<u8> {
    let d = DngDescription {
        order,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width,
        height,
        bits_per_sample: bits,
        kind: dng_writer::PlaneKind::LinearRaw,
        organisation: Organisation::Tiles {
            tile_width,
            tile_length,
            edge_fill: fill(bits.min(16)),
        },
        payload,
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::<RawTag>::new(),
        fault: None,
    };
    write_dng(&d).expect("the writer accepts the tiled description")
}

/// Full-size LJ92 tiles of `plane`, padded past the right and bottom edges.
#[allow(clippy::too_many_arguments)]
fn lj92_tiles(
    plane: &[u16],
    width: u32,
    height: u32,
    tile_width: u32,
    tile_length: u32,
    pad: u16,
    precision: u8,
    nf: u8,
) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    for ty in 0..height.div_ceil(tile_length) {
        for tx in 0..width.div_ceil(tile_width) {
            let mut tile = Vec::with_capacity((tile_width * tile_length) as usize);
            for r in 0..tile_length {
                for c in 0..tile_width {
                    let (x, y) = (tx * tile_width + c, ty * tile_length + r);
                    tile.push(if x < width && y < height {
                        plane[(y * width + x) as usize]
                    } else {
                        pad
                    });
                }
            }
            chunks.push(tiff::lj92_strip_components(
                &tile,
                tile_width as usize,
                tile_length as usize,
                precision,
                nf,
                0x11,
            ));
        }
    }
    chunks
}

// ─────────────────────────────────────────────────────────────────────────────
// The AC2 layouts
// ─────────────────────────────────────────────────────────────────────────────

/// One tier-A layout: its file and the plane it must decode to.
struct Fixture {
    label: String,
    file: Vec<u8>,
    expected: Vec<u16>,
}

fn fixture(label: impl Into<String>, file: Vec<u8>, expected: Vec<u16>) -> Fixture {
    Fixture {
        label: label.into(),
        file,
        expected,
    }
}

/// Every AC2 layout: uncompressed single strips at each depth; multi-strip
/// planes with a short last strip at 12 and 16 bits in both byte orders; a
/// 12-bit odd-width plane whose rows are not whole bytes; uncompressed tiles
/// with a partial last tile row and column; Compression-7 single and multi
/// strip; Compression-7 tiles with partial edges.
fn ac2_layouts() -> Vec<Fixture> {
    let mut out = Vec::new();
    for order in [Order::Little, Order::Big] {
        // Single strips, every depth; width 8 makes every depth whole-byte rows.
        for bits in [8u32, 12, 14, 16] {
            let (d, plane) = uncompressed(order, (8, 13), bits, None);
            out.push(fixture(
                format!("{order:?} {bits}-bit single strip 8x13"),
                d.build(),
                plane,
            ));
        }
        // Multi-strip with a short last strip (13 = 4+4+4+1, and 5+5+3).
        for bits in [12u32, 16] {
            for rps in [4u32, 5] {
                let (d, plane) = uncompressed(order, (8, 13), bits, Some(rps));
                out.push(fixture(
                    format!("{order:?} {bits}-bit 8x13 rps {rps}"),
                    d.build(),
                    plane,
                ));
            }
        }
        // 12-bit, width 5: 60 bits a row, NOT whole bytes — the whole-strip
        // fallback, single strip and multi-strip (7 = 3+3+1).
        for rps in [None, Some(3u32)] {
            let (d, plane) = uncompressed(order, (5, 7), 12, rps);
            out.push(fixture(
                format!("{order:?} 12-bit odd-width 5x7 rps {rps:?}"),
                d.build(),
                plane,
            ));
        }
        // Uncompressed tiles, partial last tile row and column.
        for bits in [8u32, 12, 14, 16] {
            let (w, h, tw, tl) = (21u32, 13u32, 8u32, 4u32);
            let plane = samples((w * h) as usize, bits, 21);
            let file = tiled(order, w, h, bits, tw, tl, Payload::Samples(plane.clone()));
            out.push(fixture(
                format!("{order:?} {bits}-bit tiles 21x13 of 8x4"),
                file,
                plane,
            ));
        }
    }
    // Compression-7 strips: single, and multi with a short last strip.
    for (rps, nf) in [(None, 1u8), (None, 2), (Some(2u32), 1), (Some(3), 2)] {
        let (d, plane) = compression7(Order::Little, (6, 7), 12, nf, rps);
        out.push(fixture(
            format!("c7 6x7 nf {nf} rps {rps:?}"),
            d.build(),
            plane,
        ));
    }
    // Compression-7 tiles with partial right and bottom edges, Nf 1 and 2.
    for nf in [1u8, 2] {
        let (w, h, tw, tl) = (21u32, 13u32, 8u32, 4u32);
        let plane = samples((w * h) as usize, 12, 33);
        let chunks = lj92_tiles(&plane, w, h, tw, tl, fill(12), 12, nf);
        let file = tiled(
            Order::Little,
            w,
            h,
            12,
            tw,
            tl,
            Payload::Encoded {
                compression: 7,
                chunks,
            },
        );
        out.push(fixture(
            format!("c7 tiles 21x13 of 8x4 nf {nf}"),
            file,
            plane,
        ));
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

fn sensor_of(file: &[u8]) -> (Sensor, ByteOrder) {
    let container = Container::parse(file).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor plane");
    (sensor, container.byte_order())
}

fn band(rows: u32) -> NonZeroU32 {
    NonZeroU32::new(rows).expect("band rows are non-zero")
}

const BANDS_A: [u32; 5] = [1, 2, 3, 7, u32::MAX];

/// Decode `units[order[k]]` for each `k`, each into its own band of a plane
/// pre-filled with `poison` (so a unit that writes nothing cannot pass).
fn assemble(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    units: &[Unit],
    order: &[usize],
    poison: &[u16],
) -> Result<Vec<u16>, Error> {
    let width = sensor.width as usize;
    let mut plane = poison.to_vec();
    for &k in order {
        let u = &units[k];
        let at = u.row_start as usize * width;
        let n = u.samples(sensor.width).expect("band fits usize");
        unpack_unit_into(sensor, byte_order, file, u, &mut plane[at..at + n])?;
    }
    Ok(plane)
}

/// A seeded Fisher–Yates shuffle (xorshift32; no `rand` dependency).
fn shuffled(n: usize, seed: u32) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    let mut s = seed | 1;
    for i in (1..n).rev() {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        let j = (s as usize) % (i + 1);
        v.swap(i, j);
    }
    v
}

/// Every permutation of `0..n` (Heap's algorithm), for small `n`.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    fn heap(k: usize, a: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if k <= 1 {
            out.push(a.clone());
            return;
        }
        for i in 0..k {
            heap(k - 1, a, out);
            if k.is_multiple_of(2) {
                a.swap(i, k - 1);
            } else {
                a.swap(0, k - 1);
            }
        }
    }
    let mut a: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    heap(n, &mut a, &mut out);
    out
}

/// The orders AC2 checks: every permutation up to 6 units, else forward,
/// reverse and a seeded shuffle.
fn orders_for(n: usize) -> Vec<Vec<usize>> {
    if n <= 6 {
        permutations(n)
    } else {
        vec![
            (0..n).collect(),
            (0..n).rev().collect(),
            shuffled(n, 0x5EC0_0068),
        ]
    }
}

/// AC3's coverage invariant: in order, no gap, no overlap, exactly `0..height`.
fn assert_covers(label: &str, units: &[Unit], height: u32) {
    let mut next = 0u32;
    for (k, u) in units.iter().enumerate() {
        assert_eq!(
            u.row_start, next,
            "{label}: unit {k} starts at row {} but the previous unit ended at {next}",
            u.row_start
        );
        next = u.row_start + u.row_count;
    }
    assert_eq!(
        next, height,
        "{label}: the units must end at the plane height"
    );
}

/// `units(..)` collected, with `len()` checked against the count yielded (and
/// again part-way through).
fn collect_units(label: &str, sensor: &Sensor, file: &[u8], rows: u32) -> Vec<Unit> {
    let it = units(sensor, file, band(rows)).unwrap_or_else(|e| panic!("{label}: units: {e}"));
    let len = it.len();
    let mut it = it;
    let first = it.next();
    if first.is_some() {
        assert_eq!(it.len(), len - 1, "{label}: len() after one next()");
    }
    let all: Vec<Unit> = first.into_iter().chain(it).collect();
    assert_eq!(all.len(), len, "{label}: ExactSizeIterator::len()");
    all
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — any-order assembly on tier-A layouts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn units_assemble_in_any_order_on_tier_a_layouts() {
    let mut assemblies = 0usize;
    for f in ac2_layouts() {
        let (sensor, order) = sensor_of(&f.file);
        // The whole-plane decode is itself held to the fixture's own samples.
        let mut whole = vec![0u16; f.expected.len()];
        unpack_into(&sensor, order, &f.file, &mut whole)
            .unwrap_or_else(|e| panic!("{}: unpack_into: {e}", f.label));
        assert_eq!(whole, f.expected, "{}: unpack_into", f.label);
        let poison: Vec<u16> = f.expected.iter().map(|s| s ^ 0x5555).collect();

        for rows in BANDS_A {
            let label = format!("{} band {rows}", f.label);
            let us = collect_units(&label, &sensor, &f.file, rows);
            assert_covers(&label, &us, sensor.height);
            for perm in orders_for(us.len()) {
                let plane = assemble(&sensor, order, &f.file, &us, &perm, &poison)
                    .unwrap_or_else(|e| panic!("{label} order {perm:?}: {e}"));
                assert!(
                    plane == f.expected,
                    "{label}: units decoded in order {perm:?} differ from the input samples \
                     (first difference at {:?})",
                    plane.iter().zip(&f.expected).position(|(a, b)| a != b)
                );
                assemblies += 1;
            }
        }
    }
    assert!(assemblies > 500, "the sweep ran {assemblies} assemblies");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — unit counts and coverage
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn unit_counts_and_coverage() {
    // Every AC2 fixture covers 0..height in order (also asserted inside the
    // AC2 sweep; repeated here so AC3 stands alone).
    for f in ac2_layouts() {
        let (sensor, _) = sensor_of(&f.file);
        for rows in BANDS_A {
            let label = format!("{} band {rows}", f.label);
            let us = collect_units(&label, &sensor, &f.file, rows);
            assert_covers(&label, &us, sensor.height);
        }
    }

    // DEC-063 §1's arithmetic on tier-A shapes, `(label, file, band, count)`.
    let single16 = uncompressed(Order::Little, (8, 13), 16, None).0.build();
    let multi12 = uncompressed(Order::Big, (8, 13), 12, Some(4)).0.build();
    let odd12 = uncompressed(Order::Little, (5, 7), 12, Some(3)).0.build();
    let c7 = compression7(Order::Little, (6, 7), 12, 1, Some(2))
        .0
        .build();
    let tiles = tiled(
        Order::Little,
        21,
        13,
        16,
        8,
        4,
        Payload::Samples(samples(21 * 13, 16, 1)),
    );
    let cases: [(&str, &[u8], u32, usize); 14] = [
        ("single 13 rows / 1", &single16, 1, 13),
        ("single 13 rows / 3", &single16, 3, 5),
        ("single 13 rows / 7", &single16, 7, 2),
        ("single 13 rows / MAX", &single16, u32::MAX, 1),
        // Strips 4+4+4+1: a band never crosses a strip.
        ("rps 4 / 3", &multi12, 3, 2 + 2 + 2 + 1),
        ("rps 4 / 7", &multi12, 7, 4),
        ("rps 4 / MAX", &multi12, u32::MAX, 4),
        // Rows not whole bytes: one unit per strip whatever the band.
        ("odd width / 1", &odd12, 1, 3),
        ("odd width / MAX", &odd12, u32::MAX, 3),
        // A Compression-7 strip is one unit whatever the band.
        ("c7 rps 2 / 1", &c7, 1, 4),
        ("c7 rps 2 / MAX", &c7, u32::MAX, 4),
        // One unit per tile row: 13 rows of 4-row tiles is 4 tile rows.
        ("tiles / 1", &tiles, 1, 4),
        ("tiles / 2", &tiles, 2, 4),
        ("tiles / MAX", &tiles, u32::MAX, 4),
    ];
    for (label, file, rows, want) in cases {
        let (sensor, _) = sensor_of(file);
        let us = collect_units(label, &sensor, file, rows);
        assert_eq!(us.len(), want, "{label}: unit count");
    }

    // The odd-width fallback's units are exactly the strips.
    let (sensor, _) = sensor_of(&odd12);
    let us = collect_units("odd width", &sensor, &odd12, 1);
    let rows: Vec<(u32, u32)> = us.iter().map(|u| (u.row_start, u.row_count)).collect();
    assert_eq!(rows, [(0, 3), (3, 3), (6, 1)]);

    // A tiled plane's last unit is the clipped last tile row.
    let (sensor, _) = sensor_of(&tiles);
    let us = collect_units("tiles", &sensor, &tiles, u32::MAX);
    assert_eq!((us[3].row_start, us[3].row_count), (12, 1));

    // Nikon-shaped (rps 256, height 2868, 12 strips) at 100: 11 strips of
    // 3 bands (100+100+56) and a last strip of 52 rows, one band: 34 units,
    // each inside one strip.
    let (d, _) = uncompressed(Order::Little, (2, 2868), 16, Some(256));
    assert_eq!(d.offsets.len(), 12);
    let nikon = d.build();
    let (sensor, _) = sensor_of(&nikon);
    let us = collect_units("nikon-shaped", &sensor, &nikon, 100);
    assert_eq!(us.len(), 34);
    assert_covers("nikon-shaped", &us, 2868);
    for u in &us {
        let first_strip = u.row_start / 256;
        let last_strip = (u.row_start + u.row_count - 1) / 256;
        assert_eq!(
            first_strip, last_strip,
            "unit rows {}..+{} crosses a strip boundary",
            u.row_start, u.row_count
        );
    }
    assert_eq!(
        collect_units("nikon-shaped", &sensor, &nikon, 704).len(),
        12
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — refusals
// ─────────────────────────────────────────────────────────────────────────────

/// The malformed and the well-formed layouts `tests/plane_validate.rs` sweeps
/// (each strip count ±1, an offset past EOF, `RowsPerStrip` changes, strip
/// lists truncated, an extra zero-row strip, Compression-7 rps and count
/// changes), plus the `tests/plane_unpack.rs` / `tests/plane_tiles.rs`
/// refusals: an unsupported compression and depth, both layouts at once, a
/// tile table of the wrong length, a tile row of sub-byte width, and aliased
/// strips.
fn refusal_sweep() -> Vec<(String, Vec<u8>)> {
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();
    for bits in [12u32, 16] {
        let (good, _) = uncompressed(Order::Big, (5, 6), bits, Some(2));
        cases.push((format!("{bits} good"), good.build()));
        for i in 0..good.counts.len() {
            for delta in [-1i64, 1] {
                let mut d = good.clone();
                d.counts[i] = (i64::from(d.counts[i]) + delta) as u32;
                cases.push((format!("{bits} count[{i}] {delta:+}"), d.build()));
            }
            let mut d = good.clone();
            d.offsets[i] = u32::MAX - 3;
            cases.push((format!("{bits} offset[{i}] past EOF"), d.build()));
        }
        for rps in [0u32, 1, 3, 4, 6, 7, u32::MAX] {
            let mut d = good.clone();
            d.rows_per_strip = Some(rps);
            cases.push((format!("{bits} rps {rps}"), d.build()));
        }
        for keep in 0..good.offsets.len() {
            let mut d = good.clone();
            d.offsets.truncate(keep);
            d.counts.truncate(keep);
            cases.push((format!("{bits} keep {keep} strips"), d.build()));
        }
        let mut d = good.clone();
        d.offsets.push(PAYLOAD_BASE);
        d.counts.push(0);
        cases.push((format!("{bits} extra zero-row strip"), d.build()));
        // Aliased strips: every strip points at the first one's bytes.
        let mut d = good.clone();
        for o in d.offsets.iter_mut() {
            *o = PAYLOAD_BASE;
        }
        d.payload.truncate(d.counts[0] as usize);
        cases.push((format!("{bits} aliased strips"), d.build()));
    }
    let (c7, _) = compression7(Order::Little, (4, 6), 12, 1, Some(2));
    cases.push(("c7 good".into(), c7.build()));
    for rps in [0u32, 1, 3, 6, 9] {
        let mut d = c7.clone();
        d.rows_per_strip = Some(rps);
        cases.push((format!("c7 rps {rps}"), d.build()));
    }
    for i in 0..c7.counts.len() {
        let mut d = c7.clone();
        d.counts[i] += 4_000;
        cases.push((format!("c7 count[{i}] past EOF"), d.build()));
    }
    let mut d = c7.clone();
    d.offsets.pop();
    d.counts.pop();
    cases.push(("c7 strips short of the plane".into(), d.build()));

    // Unsupported compression and depth.
    let (mut d, _) = uncompressed(Order::Little, (4, 4), 16, None);
    d.compression = 34713;
    cases.push(("unsupported compression".into(), d.build()));
    let (mut d, _) = uncompressed(Order::Little, (4, 4), 16, None);
    d.bits = 10;
    cases.push(("unsupported depth".into(), d.build()));

    // Tiled refusals, through the writer's raw-tag escape hatch.
    let tile_plane = samples(21 * 13, 16, 3);
    let tiled_with = |extra: Vec<RawTag>| {
        let d = DngDescription {
            order: Order::Little,
            layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
            width: 21,
            height: 13,
            bits_per_sample: 16,
            kind: dng_writer::PlaneKind::LinearRaw,
            organisation: Organisation::Tiles {
                tile_width: 8,
                tile_length: 4,
                edge_fill: fill(16),
            },
            payload: Payload::Samples(tile_plane.clone()),
            meta: DngMeta::default(),
            opcode_lists: [None, None, None],
            extra_tags: extra,
            fault: None,
        };
        write_dng(&d).expect("writer accepts")
    };
    let long_tag = |tag: u16, values: &[u32]| RawTag {
        ifd: dng_writer::TagIfd::Sensor,
        tag,
        field_type: 4,
        count: values.len() as u32,
        bytes: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    };
    cases.push(("tiles good".into(), tiled_with(vec![])));
    cases.push((
        "tiles and strips".into(),
        tiled_with(vec![
            long_tag(tiff::STRIP_OFFSETS, &[8]),
            long_tag(tiff::STRIP_BYTE_COUNTS, &[8]),
        ]),
    ));
    cases.push((
        "tile table too short".into(),
        tiled_with(vec![long_tag(dng_writer::TAG_TILE_OFFSETS, &[8, 8])]),
    ));
    cases.push((
        "tile width zero".into(),
        tiled_with(vec![long_tag(dng_writer::TAG_TILE_WIDTH, &[0])]),
    ));
    // A 12-bit plane whose 5-sample tile rows are 60 bits: UnsupportedTileRowBits.
    let d = DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: 10,
        height: 4,
        bits_per_sample: 12,
        kind: dng_writer::PlaneKind::LinearRaw,
        organisation: Organisation::Tiles {
            tile_width: 5,
            tile_length: 4,
            edge_fill: 0,
        },
        payload: Payload::Encoded {
            compression: 1,
            chunks: vec![vec![0u8; 30], vec![0u8; 30]],
        },
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    };
    cases.push((
        "sub-byte tile row".into(),
        write_dng(&d).expect("writer accepts"),
    ));
    cases
}

#[test]
fn units_refuse_what_unpack_into_refuses() {
    let (mut structural, mut decode_time, mut oks) = (0, 0, 0);
    for (label, file) in refusal_sweep() {
        let (sensor, order) = sensor_of(&file);
        let n = sensor.width as usize * sensor.height as usize;
        let mut whole = vec![0u16; n];
        let unpacked = unpack_into(&sensor, order, &file, &mut whole);
        let rows = band(2);
        match (&unpacked, units(&sensor, &file, rows)) {
            (Ok(()), Ok(it)) => {
                let us: Vec<Unit> = it.collect();
                assert_covers(&label, &us, sensor.height);
                let order_rev: Vec<usize> = (0..us.len()).rev().collect();
                let poison = vec![0xA5A5u16; n];
                let plane = assemble(&sensor, order, &file, &us, &order_rev, &poison)
                    .unwrap_or_else(|e| panic!("{label}: a unit failed on an Ok plane: {e}"));
                assert!(plane == whole, "{label}: reverse assembly differs");
                oks += 1;
            }
            (Ok(()), Err(e)) => panic!("{label}: unpack_into Ok but units refused: {e:?}"),
            (Err(ue), Err(e)) => {
                assert_eq!(
                    format!("{e:?}"),
                    format!("{ue:?}"),
                    "{label}: units must refuse with unpack_into's error"
                );
                structural += 1;
            }
            (Err(ue), Ok(it)) => {
                // Only a decode-time failure gets past the layout check; then
                // the units in forward order fail with that error, first.
                let us: Vec<Unit> = it.collect();
                let mut first_err = None;
                let mut scratch = vec![0u16; n];
                let width = sensor.width as usize;
                for u in &us {
                    let at = u.row_start as usize * width;
                    let k = u.samples(sensor.width).unwrap();
                    if let Err(e) =
                        unpack_unit_into(&sensor, order, &file, u, &mut scratch[at..at + k])
                    {
                        first_err = Some(e);
                        break;
                    }
                }
                let e = first_err
                    .unwrap_or_else(|| panic!("{label}: unpack_into {ue:?} but every unit Ok"));
                assert_eq!(format!("{e:?}"), format!("{ue:?}"), "{label}");
                decode_time += 1;
            }
        }
    }
    assert!(
        structural >= 20 && oks >= 4,
        "the sweep must exercise both refusals ({structural}) and accepted layouts ({oks}); \
         decode-time {decode_time}"
    );
}

/// A unit of `from` that is not a unit of `onto`, decoded into a sentinel band
/// of `len` samples: the mismatch, its rows, its kind, and an untouched `dst`.
fn assert_foreign(label: &str, from: &[u8], from_rows: u32, onto: &[u8], pick_last: bool) {
    let (s_from, _) = sensor_of(from);
    let (s_onto, order) = sensor_of(onto);
    let us = collect_units(label, &s_from, from, from_rows);
    let u = if pick_last {
        *us.last().unwrap()
    } else {
        us[0]
    };
    let right_len = u.samples(s_onto.width).unwrap();
    // Right length for the unit, and a wrong one: either way the mismatch is
    // reported first, and nothing is written.
    for len in [right_len, right_len + 3, right_len.saturating_sub(1)] {
        let mut dst = vec![0x7E7Eu16; len];
        let e = unpack_unit_into(&s_onto, order, onto, &u, &mut dst)
            .expect_err(&format!("{label}: a foreign unit must be refused"));
        assert!(
            matches!(e, Error::PlaneUnitMismatch { row_start, row_count }
                if row_start == u.row_start && row_count == u.row_count),
            "{label} (dst {len}): expected PlaneUnitMismatch with the unit's rows, got {e:?}"
        );
        assert_eq!(e.kind(), ErrorKind::CallerError, "{label}");
        assert!(
            dst.iter().all(|&s| s == 0x7E7E),
            "{label}: dst written on refusal"
        );
    }
}

#[test]
fn foreign_unit_is_a_typed_mismatch_before_any_write() {
    let tall = uncompressed(Order::Little, (8, 13), 16, None).0.build();
    let short = uncompressed(Order::Little, (8, 6), 16, None).0.build();
    let rps4 = uncompressed(Order::Little, (8, 13), 16, Some(4)).0.build();
    let odd = uncompressed(Order::Little, (5, 13), 12, None).0.build();
    let tiles = tiled(
        Order::Little,
        8,
        13,
        16,
        8,
        4,
        Payload::Samples(samples(8 * 13, 16, 4)),
    );
    let c7 = compression7(Order::Little, (8, 13), 12, 1, None).0.build();

    // A band past the end of a shorter plane (the Q2M-704-on-Canon shape).
    assert_foreign("band past the end", &tall, 7, &short, true);
    // A band that would cross a strip of the other plane (rows 0..7 vs rps 4).
    assert_foreign("band across a strip", &tall, 7, &rps4, false);
    // A tile row is not a unit of a strip plane, and back.
    assert_foreign("tile row onto strips", &tiles, 1, &tall, true);
    assert_foreign("band onto tiles", &rps4, 1, &tiles, false);
    // A whole-strip unit (rows not whole bytes) is not a band, and back.
    assert_foreign("whole strip onto bands", &odd, 1, &short, false);
    assert_foreign("band onto whole strips", &short, 1, &odd, false);
    // A Compression-7 whole strip is not a unit of a multi-strip plane.
    assert_foreign("c7 strip onto rps 4", &c7, 1, &rps4, false);

    // Tier B: the Q2M's last 704-row band on the Canon 5D III.
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();
    let paths = [
        "LEICA-Q2-MONO/L1021223.DNG",
        "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
    ];
    let files: Vec<_> = paths
        .iter()
        .filter_map(|p| manifest.get(p).expect("in manifest").require(&root))
        .collect();
    if let [q2m, canon] = files.as_slice() {
        let q2m = std::fs::read(q2m).expect("read Q2M");
        let canon = std::fs::read(canon).expect("read Canon");
        assert_foreign("Q2M 704 band on Canon", &q2m, 704, &canon, true);
    }
}

#[test]
fn wrong_length_band_is_refused_before_any_write() {
    let layouts = ac2_layouts();
    for f in &layouts {
        let (sensor, order) = sensor_of(&f.file);
        for u in collect_units(&f.label, &sensor, &f.file, 3) {
            let n = u.samples(sensor.width).unwrap();
            for len in [n.wrapping_sub(1), n + 1] {
                if len > n + 1 {
                    continue; // n == 0 cannot be one short
                }
                let mut dst = vec![0x7E7Eu16; len];
                let e = unpack_unit_into(&sensor, order, &f.file, &u, &mut dst)
                    .expect_err("a wrong-length band must be refused");
                assert!(
                    matches!(e, Error::PlaneBufferWrongLength { expected, actual }
                        if expected == n as u64 && actual == len),
                    "{}: unit rows {}..+{} dst {len}: got {e:?}",
                    f.label,
                    u.row_start,
                    u.row_count
                );
                assert!(dst.iter().all(|&s| s == 0x7E7E), "{}: dst written", f.label);
            }
        }
    }
}

#[test]
fn a_failing_unit_fails_alone() {
    // Three Compression-7 strips (rps 2 over 6 rows); the middle one is a
    // frame of the wrong shape (3 rows, not 2).
    let (width, height) = (4u32, 6u32);
    let plane = samples((width * height) as usize, 12, 9);
    let good = |rows: std::ops::Range<usize>| {
        tiff::lj92_strip_components(
            &plane[rows.start * 4..rows.end * 4],
            4,
            rows.len(),
            12,
            1,
            0x11,
        )
    };
    let bad = tiff::lj92_strip_components(&plane[..12], 4, 3, 12, 1, 0x11);
    let strips = [good(0..2), bad, good(4..6)];
    let file = Dng::contiguous(Order::Little, 7, (width, height), 12, Some(2), &strips).build();
    let (sensor, order) = sensor_of(&file);

    let mut whole = vec![0u16; 24];
    let whole_err = unpack_into(&sensor, order, &file, &mut whole).expect_err("strip 1 is bad");

    let us = collect_units("c7 with a bad strip", &sensor, &file, u32::MAX);
    assert_eq!(us.len(), 3);
    for (k, u) in us.iter().enumerate() {
        let mut dst = vec![0u16; u.samples(width).unwrap()];
        let got = unpack_unit_into(&sensor, order, &file, u, &mut dst);
        if k == 1 {
            let e = got.expect_err("unit 1 must fail");
            assert_eq!(format!("{e:?}"), format!("{whole_err:?}"));
        } else {
            got.unwrap_or_else(|e| panic!("unit {k} must decode on its own: {e}"));
            let at = u.row_start as usize * width as usize;
            assert_eq!(dst, plane[at..at + dst.len()], "unit {k}");
        }
    }
}

#[test]
fn unit_mismatch_display_names_both_rows() {
    let e = Error::PlaneUnitMismatch {
        row_start: 4928,
        row_count: 704,
    };
    let shown = e.to_string();
    assert!(
        shown.contains("4928") && shown.contains("704"),
        "Display must name both numbers: {shown}"
    );
    assert_eq!(e.kind(), ErrorKind::CallerError);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 + AC3 — the held files (tier B)
// ─────────────────────────────────────────────────────────────────────────────

/// Every held file `plane_oracle` pins, with DEC-063 §1's unit counts at
/// bands 704, 100 and MAX.
const HELD: [(&str, usize, usize, usize); 11] = [
    ("LEICA-Q2-MONO/L1021223.DNG", 8, 57, 1),
    ("LEICA-Q2-MONO/L1026016.DNG", 8, 57, 1),
    ("LEICA-Q2-MONO/L1026192.DNG", 8, 57, 1),
    ("LEICA-M-MONOCHROM/L1000622.DNG", 5, 35, 1),
    ("CANON-EOS-5D-MARK-III/5G4A9394.DNG", 6, 40, 1),
    ("NIKON-D300S/_DSC4615.DNG", 12, 34, 12),
    ("LEICA-M-MONOCHROM-TYP246/M2462362.DNG", 1, 1, 1),
    ("PENTAX-K3III-MONO/K3III.DNG", 1, 1, 1),
    ("GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng", 12, 12, 12),
    ("GOOGLE-PIXEL-3A/IMG_20190918_164153.dng", 12, 12, 12),
    ("GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng", 12, 12, 12),
];

/// `docs/oracle-contract.md`: MD5 of the uncropped plane, native LE `u16`s.
fn plane_md5(plane: &[u16]) -> String {
    let mut hasher = md5::Md5::new();
    for s in plane {
        hasher.update(&s.to_le_bytes());
    }
    md5::to_hex(&hasher.finish())
}

#[test]
fn unit_counts_on_held_files() {
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();
    for (path, at704, at100, at_max) in HELD {
        let entry = manifest.get(path).expect("held file is in the manifest");
        let Some(file_path) = entry.require(&root) else {
            continue; // SKIP announced by name
        };
        let file = std::fs::read(&file_path).expect("read held file");
        let (sensor, _) = sensor_of(&file);
        for (rows, want) in [(704, at704), (100, at100), (u32::MAX, at_max)] {
            let us = collect_units(path, &sensor, &file, rows);
            assert_eq!(us.len(), want, "{path} at {rows}");
            assert_covers(path, &us, sensor.height);
        }
    }
}

#[test]
fn units_assemble_in_any_order_on_held_files() {
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();
    for (path, _, _, _) in HELD {
        let entry = manifest.get(path).expect("held file is in the manifest");
        let Some(file_path) = entry.require(&root) else {
            continue; // SKIP announced by name
        };
        let file = std::fs::read(&file_path).expect("read held file");
        let (sensor, order) = sensor_of(&file);
        let n = sensor.width as usize * sensor.height as usize;

        // The reference is held to the PRE-EXISTING dnglab pin first, so the
        // byte comparisons below are comparisons against that pin.
        let mut reference = vec![0u16; n];
        unpack_into(&sensor, order, &file, &mut reference).expect("unpack_into");
        assert_eq!(
            plane_md5(&reference),
            entry.oracle.raw_checksum,
            "{path}: unpack_into no longer matches the manifest's dnglab checksum"
        );
        let poison: Vec<u16> = reference.iter().map(|s| s ^ 0x5555).collect();

        for rows in [1u32, 100, 704, u32::MAX] {
            let us = collect_units(path, &sensor, &file, rows);
            let reverse: Vec<usize> = (0..us.len()).rev().collect();
            for (what, perm) in [("reverse", reverse), ("shuffle", shuffled(us.len(), rows))] {
                let plane = assemble(&sensor, order, &file, &us, &perm, &poison)
                    .unwrap_or_else(|e| panic!("{path} band {rows} {what}: {e}"));
                assert!(
                    plane == reference,
                    "{path} band {rows} {what}: assembled plane differs from the pinned plane"
                );
            }
        }

        // The Q2M at 704: eight units on eight scoped threads, each into its
        // own band carved with split_at_mut.
        if path == "LEICA-Q2-MONO/L1021223.DNG" {
            let us = collect_units(path, &sensor, &file, 704);
            assert_eq!(us.len(), 8);
            let mut plane = poison.clone();
            let started = std::time::Instant::now();
            std::thread::scope(|scope| {
                let mut rest: &mut [u16] = &mut plane;
                let mut handles = Vec::new();
                for u in &us {
                    let (mine, tail) = rest.split_at_mut(u.samples(sensor.width).unwrap());
                    rest = tail;
                    let (sensor, file) = (&sensor, &file);
                    handles
                        .push(scope.spawn(move || unpack_unit_into(sensor, order, file, u, mine)));
                }
                for h in handles {
                    h.join().expect("thread").expect("unit decodes");
                }
            });
            eprintln!(
                "{path}: 8 units on 8 threads in {:?} (this build's profile)",
                started.elapsed()
            );
            assert_eq!(
                plane_md5(&plane),
                entry.oracle.raw_checksum,
                "{path}: the 8-thread assembly must hash to the dnglab pin"
            );
        }
    }
}

/// Found by the `plane` fuzz target during the SPEC-068 build: a zero-width
/// tiled plane has an empty tile table (`ceil(0 / TW) × down` = 0 entries) and
/// so nothing to decode, but a tile row per `TileLength` rows — about a
/// billion of them at `height` near `u32::MAX`. It must yield no units, so
/// `unpack_into` stays as cheap as the zero-tile loop it replaced.
#[test]
fn a_zero_width_tiled_plane_has_no_units() {
    let order = Order::Little;
    let mut entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, 0),
        tiff::long(tiff::IMAGE_LENGTH, u32::MAX - 7),
        tiff::short(tiff::BITS_PER_SAMPLE, 16, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::long(dng_writer::TAG_TILE_WIDTH, 8),
        tiff::long(dng_writer::TAG_TILE_LENGTH, 4),
    ];
    entries.sort_by_key(|e| e.tag);
    let file = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
    let (sensor, byte_order) = sensor_of(&file);
    assert_eq!(sensor.width, 0);
    for rows in [1u32, u32::MAX] {
        let it = units(&sensor, &file, band(rows)).expect("an empty tile table is a valid grid");
        assert_eq!(it.len(), 0, "band {rows}");
        assert_eq!(it.count(), 0, "band {rows}");
    }
    let started = std::time::Instant::now();
    unpack_into(&sensor, byte_order, &file, &mut []).expect("nothing to decode");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "unpack_into walked empty tile rows ({:?})",
        started.elapsed()
    );
}
