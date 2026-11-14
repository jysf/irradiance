//! `SPEC-069` — the packed 12/14-bit unpack decodes whole byte groups
//! (`DEC-064`): 14-bit reads 7 bytes as 4 samples, 12-bit reads 3 bytes as 2,
//! and the bit reader decodes only the tail. These tests pin that the planes
//! are unchanged on every layout the groups can meet (AC2).
//!
//! Tier A only (`DEC-003`): every fixture is built in-test. The oracle never
//! runs the code under test (AGENTS.md §16 rule 5): samples come from a seeded
//! PRNG over the full range, including 0 and `2^bits − 1`, are packed by this
//! file's own MSB-first packer ([`pack_msb`], a copy of
//! `tests/plane_unpack.rs`'s), and must unpack to exactly the samples that
//! went in.
//!
//! Every tail residue appears: widths 1–17 and heights 1–5 put every sample
//! count modulo 4 (14-bit) and modulo 2 (12-bit) at both a row end and a strip
//! end, in one strip, in byte-padded strips of 1 and 2 rows, in SPEC-068 bands,
//! and in tiles with clipped right and bottom edges.
//!
//! These tests pass on the per-sample reader as well as on the groups: they
//! pin behaviour, not speed. Their teeth are SPEC-069's red-proof list (a
//! wrong shift, mask, dispatch or tail position turns them red).

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use std::num::NonZeroU32;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, RawTag};
use irradiance::ifd::{ByteOrder, Container, Sensor, TAG_ROWS_PER_STRIP};
use irradiance::plane::{units, unpack_into, unpack_unit_into, Unit};
use tiff::Order;

/// The packed depths the groups cover.
const DEPTHS: [u32; 2] = [12, 14];
const WIDTHS: std::ops::RangeInclusive<u32> = 1..=17;
const HEIGHTS: std::ops::RangeInclusive<u32> = 1..=5;
/// Above every 12/14-bit sample, so a sample the decoder never wrote shows.
const POISON: u16 = u16::MAX;

// ─────────────────────────────────────────────────────────────────────────────
// The independent packer and the samples
// ─────────────────────────────────────────────────────────────────────────────

/// Pack `samples` MSB-first at `bits` per sample, padding the final byte with
/// zero bits (`DEC-008`). A copy of `tests/plane_unpack.rs`'s `pack_msb`, with
/// the accumulator masked so it never carries more than the pending bits.
fn pack_msb(samples: &[u16], bits: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut nbits: u32 = 0;
    let mask = (1u32 << bits) - 1;
    for &s in samples {
        acc = ((acc << bits) | (u32::from(s) & mask)) & 0x00FF_FFFF;
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

/// `count` samples below `2^bits` from a seeded xorshift32, with 0 and
/// `2^bits − 1` forced in often (one draw in four each way) so both extremes
/// land in every group position.
fn samples(count: usize, bits: u32, seed: u32) -> Vec<u16> {
    let max = (1u32 << bits) - 1;
    let mut state = 0x0069_0000_u32 ^ seed.wrapping_mul(0x9E37_79B9) ^ bits;
    if state == 0 {
        state = 1;
    }
    (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let value = match state % 8 {
                0 | 1 => 0,
                2 | 3 => max,
                _ => (state >> 3) & max,
            };
            value as u16
        })
        .collect()
}

fn seed(width: u32, height: u32, extra: u32) -> u32 {
    width | (height << 8) | (extra << 16)
}

// ─────────────────────────────────────────────────────────────────────────────
// The strip builder (a minimal copy of tests/plane_units.rs's `Dng`)
// ─────────────────────────────────────────────────────────────────────────────

/// Where the out-of-line strip arrays go.
const ARRAYS_AT: u32 = 256;
/// Where the strip payload starts.
const PAYLOAD_BASE: u32 = 8192;

/// A one-IFD, uncompressed `LinearRaw` DNG of `strips`, laid out contiguously
/// from `PAYLOAD_BASE`.
fn strip_dng(
    (width, height): (u32, u32),
    bits: u32,
    rows_per_strip: Option<u32>,
    strips: &[Vec<u8>],
) -> Vec<u8> {
    let order = Order::Little;
    let mut offsets = Vec::new();
    let mut counts = Vec::new();
    let mut payload = Vec::new();
    for s in strips {
        offsets.push(PAYLOAD_BASE + payload.len() as u32);
        counts.push(s.len() as u32);
        payload.extend_from_slice(s);
    }
    let mut entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, width),
        tiff::long(tiff::IMAGE_LENGTH, height),
        tiff::short(tiff::BITS_PER_SAMPLE, bits as u16, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
    ];
    if let Some(rows) = rows_per_strip {
        entries.push(tiff::long(TAG_ROWS_PER_STRIP, rows));
    }
    let counts_at = ARRAYS_AT + 4 * offsets.len() as u32;
    for (tag, vals, at) in [
        (tiff::STRIP_OFFSETS, &offsets, ARRAYS_AT),
        (tiff::STRIP_BYTE_COUNTS, &counts, counts_at),
    ] {
        match vals.len() {
            1 => entries.push(tiff::long(tag, vals[0])),
            n => entries.push(tiff::at_offset(tag, 4, n as u32, at)),
        }
    }
    entries.sort_by_key(|e| e.tag);
    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
    for (vals, at) in [(&offsets, ARRAYS_AT), (&counts, counts_at)] {
        if vals.len() > 1 {
            for (i, v) in vals.iter().enumerate() {
                let o = at as usize + i * 4;
                if data.len() < o + 4 {
                    data.resize(o + 4, 0);
                }
                data[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
    }
    assert!(
        data.len() <= PAYLOAD_BASE as usize,
        "the strip arrays overran the payload region"
    );
    data.resize(PAYLOAD_BASE as usize, 0);
    data.extend_from_slice(&payload);
    data
}

/// An uncompressed strip plane of seeded samples: one strip when
/// `rows_per_strip` is `None`, else strips of that many rows, each packed as
/// its own byte-padded stream.
fn strip_plane(
    (width, height): (u32, u32),
    bits: u32,
    rows_per_strip: Option<u32>,
) -> (Vec<u8>, Vec<u16>) {
    let plane = samples(
        (width * height) as usize,
        bits,
        seed(width, height, rows_per_strip.unwrap_or(0)),
    );
    let rows = rows_per_strip.unwrap_or(height) as usize;
    let strips: Vec<Vec<u8>> = plane
        .chunks(rows * width as usize)
        .map(|c| pack_msb(c, bits))
        .collect();
    (
        strip_dng((width, height), bits, rows_per_strip, &strips),
        plane,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

fn sensor_of(file: &[u8]) -> (Sensor, ByteOrder) {
    let container = Container::parse(file).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor plane");
    (sensor, container.byte_order())
}

/// `unpack_into` over a plane pre-filled with [`POISON`].
fn unpack_whole(file: &[u8]) -> Vec<u16> {
    let (sensor, order) = sensor_of(file);
    let mut plane = vec![POISON; (sensor.width * sensor.height) as usize];
    unpack_into(&sensor, order, file, &mut plane).expect("the fixture unpacks");
    plane
}

/// Assert `got == want`, naming the first differing sample.
fn assert_plane(label: &str, got: &[u16], want: &[u16]) {
    assert_eq!(got.len(), want.len(), "{label}: plane length");
    if let Some(i) = got.iter().zip(want).position(|(g, w)| g != w) {
        panic!(
            "{label}: sample {i} is {} but {} was packed",
            got[i], want[i]
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — strips
// ─────────────────────────────────────────────────────────────────────────────

/// One strip, and strips of 1 and 2 rows (each byte-padded), 12- and 14-bit,
/// widths 1–17, heights 1–5.
#[test]
fn packed_strips_round_trip_on_every_residue() {
    for bits in DEPTHS {
        for width in WIDTHS {
            for height in HEIGHTS {
                for rows_per_strip in [None, Some(1u32), Some(2)] {
                    let (file, plane) = strip_plane((width, height), bits, rows_per_strip);
                    let label =
                        format!("{bits}-bit {width}x{height} rows_per_strip {rows_per_strip:?}");
                    assert_plane(&label, &unpack_whole(&file), &plane);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — SPEC-068 bands
// ─────────────────────────────────────────────────────────────────────────────

/// SPEC-068 units at `max_band_rows` 1 and 2, each decoded by
/// `unpack_unit_into` into its own band of a poisoned plane. A band of a strip
/// whose rows are whole bytes starts mid-strip, so its first group is not the
/// strip's first group.
#[test]
fn packed_bands_round_trip_on_every_residue() {
    for bits in DEPTHS {
        for width in WIDTHS {
            for height in HEIGHTS {
                for rows_per_strip in [None, Some(2u32)] {
                    let (file, plane) = strip_plane((width, height), bits, rows_per_strip);
                    let (sensor, order) = sensor_of(&file);
                    for max_band_rows in [1u32, 2] {
                        let label = format!(
                            "{bits}-bit {width}x{height} rows_per_strip {rows_per_strip:?} \
                             max_band_rows {max_band_rows}"
                        );
                        let rows = NonZeroU32::new(max_band_rows).expect("non-zero");
                        let all: Vec<Unit> = units(&sensor, &file, rows)
                            .unwrap_or_else(|e| panic!("{label}: units: {e}"))
                            .collect();
                        let mut got = vec![POISON; plane.len()];
                        for unit in &all {
                            let at = unit.row_start as usize * width as usize;
                            let n = unit.samples(sensor.width).expect("band fits usize");
                            unpack_unit_into(&sensor, order, &file, unit, &mut got[at..at + n])
                                .unwrap_or_else(|e| {
                                    panic!("{label}: unit at row {}: {e}", unit.row_start)
                                });
                        }
                        assert_plane(&label, &got, &plane);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — tiles
// ─────────────────────────────────────────────────────────────────────────────

/// A padding sample below `2^bits` for the clipped part of edge tiles.
fn fill(bits: u32) -> u16 {
    (0x2A5A & ((1u32 << bits) - 1)) as u16
}

/// Full-size uncompressed tiles of `plane`, left to right then top to bottom,
/// each packed by [`pack_msb`] as one stream. `tile_width · bits` is a whole
/// number of bytes, so each tile row starts on a byte, as `check_tile_rows`
/// requires.
fn packed_tiles(
    plane: &[u16],
    (width, height): (u32, u32),
    bits: u32,
    (tile_width, tile_length): (u32, u32),
) -> Vec<Vec<u8>> {
    let (w, h) = (width as usize, height as usize);
    let (tw, tl) = (tile_width as usize, tile_length as usize);
    let mut chunks = Vec::new();
    for ty in 0..h.div_ceil(tl) {
        for tx in 0..w.div_ceil(tw) {
            let mut tile = Vec::with_capacity(tw * tl);
            for r in 0..tl {
                for c in 0..tw {
                    let (y, x) = (ty * tl + r, tx * tw + c);
                    tile.push(if y < h && x < w {
                        plane[y * w + x]
                    } else {
                        fill(bits)
                    });
                }
            }
            chunks.push(pack_msb(&tile, bits));
        }
    }
    chunks
}

fn tiled_dng(
    (width, height): (u32, u32),
    bits: u32,
    (tile_width, tile_length): (u32, u32),
    chunks: Vec<Vec<u8>>,
) -> Vec<u8> {
    let d = DngDescription {
        order: Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width,
        height,
        bits_per_sample: bits,
        kind: dng_writer::PlaneKind::LinearRaw,
        organisation: Organisation::Tiles {
            tile_width,
            tile_length,
            edge_fill: fill(bits),
        },
        payload: Payload::Encoded {
            compression: 1,
            chunks,
        },
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::<RawTag>::new(),
        fault: None,
    };
    write_dng(&d).expect("the writer accepts the tiled description")
}

/// Tiled planes with clipped right and bottom edges. Tile widths keep tile
/// rows whole bytes: 4 and 8 for both depths (multiples of 4 for 14-bit, of 2
/// for 12-bit). A clipped tile row hands the decoder fewer samples than the
/// row holds, so its tail ends mid-row.
#[test]
fn packed_tiles_round_trip_with_clipped_edges() {
    for bits in DEPTHS {
        for tile_width in [4u32, 8] {
            for tile_length in [1u32, 2, 3] {
                for width in WIDTHS {
                    for height in HEIGHTS {
                        let plane =
                            samples((width * height) as usize, bits, seed(width, height, 0x7));
                        let chunks =
                            packed_tiles(&plane, (width, height), bits, (tile_width, tile_length));
                        let file =
                            tiled_dng((width, height), bits, (tile_width, tile_length), chunks);
                        let label = format!(
                            "{bits}-bit {width}x{height} tiles of {tile_width}x{tile_length}"
                        );
                        assert_plane(&label, &unpack_whole(&file), &plane);
                    }
                }
            }
        }
    }
}

/// The packer is the inverse the tests assume: the Q2M strip head
/// (`DEC-008`, 8 × 14-bit) packs to its measured bytes.
#[test]
fn pack_msb_packs_the_measured_q2m_head() {
    let samples = [746u16, 725, 711, 752, 646, 705, 772, 686];
    let q2m_head: [u8; 14] = [
        0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
    ];
    assert_eq!(pack_msb(&samples, 14), q2m_head.to_vec());
}
