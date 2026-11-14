//! `SPEC-068` AC6 — `plane::units` and `plane::unpack_unit_into` allocate
//! nothing (`DEC-016`, `DEC-063`).
//!
//! Its own test binary on purpose: the counting `#[global_allocator]` below
//! sees every allocation in this process (the `tests/previews_alloc.rs`
//! pattern). Fixtures are built and parsed BEFORE counting starts; only the
//! unit calls are counted, on the test thread.
//!
//! One pre-existing allocation is outside this spec: the Compression-7 header
//! parse (`ljpeg::parse_headers`) collects its Huffman tables into a small
//! header-sized `Vec`, on every strip or tile, through `unpack_into` exactly as
//! through a unit. So for Compression-7 planes the check is "no NEW
//! allocation": decoding every unit allocates exactly what `unpack_into`
//! allocates on the same plane, and `units` plus its iteration allocates
//! nothing.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};

use corpus::{CorpusRoot, Manifest};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload};
use irradiance::ifd::{ByteOrder, Container, Sensor, TAG_ROWS_PER_STRIP};
use irradiance::plane::{units, unpack_into, unpack_unit_into};
use tiff::Order;

struct Counting;

// Thread-local, so only allocations made BY THE TEST THREAD between the two
// switches count. A `const` thread-local needs no lazy initialisation, so
// reading it never allocates.
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

fn counting() -> bool {
    COUNTING.try_with(Cell::get).unwrap_or(false)
}

fn set_counting(on: bool) {
    COUNTING.with(|c| c.set(on));
}

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// The library's `forbid(unsafe_code)` binds the library crate, not this test
// crate. `GlobalAlloc` is an unsafe trait; every method forwards to `System`
// unchanged, which upholds its contract exactly as `System` does.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Allocations made by `f` on this thread.
fn count<R>(f: impl FnOnce() -> R) -> (usize, R) {
    let before = ALLOCATIONS.load(Ordering::SeqCst);
    set_counting(true);
    let r = f();
    set_counting(false);
    (ALLOCATIONS.load(Ordering::SeqCst) - before, r)
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures (minimal one-IFD strips, and tiles via the SPEC-057 writer)
// ─────────────────────────────────────────────────────────────────────────────

const ARRAYS_AT: u32 = 256;
const PAYLOAD_BASE: u32 = 8192;

fn strips_dng(
    compression: u16,
    (width, height): (u32, u32),
    bits: u16,
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
        tiff::short(tiff::BITS_PER_SAMPLE, bits, order),
        tiff::short(tiff::COMPRESSION, compression, order),
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
    data.resize(PAYLOAD_BASE as usize, 0);
    for (vals, at) in [(&offsets, ARRAYS_AT), (&counts, counts_at)] {
        if vals.len() > 1 {
            for (i, v) in vals.iter().enumerate() {
                let o = at as usize + i * 4;
                data[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
    }
    data.extend_from_slice(&payload);
    data
}

fn ramp(count: usize, bits: u32) -> Vec<u16> {
    (0..count)
        .map(|i| ((i * 2_654_435_761usize) >> 5) as u16 & ((1u32 << bits) - 1) as u16)
        .collect()
}

fn tiled(bits: u32, payload: Payload) -> Vec<u8> {
    write_dng(&DngDescription {
        order: Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width: 21,
        height: 13,
        bits_per_sample: bits,
        kind: dng_writer::PlaneKind::LinearRaw,
        organisation: Organisation::Tiles {
            tile_width: 8,
            tile_length: 4,
            edge_fill: 0,
        },
        payload,
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    })
    .expect("writer accepts")
}

/// `(label, file, compressed)`.
fn fixtures() -> Vec<(&'static str, Vec<u8>, bool)> {
    let le16 = |s: &[u16]| s.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>();
    let single = strips_dng(1, (8, 13), 16, None, &[le16(&ramp(8 * 13, 16))]);
    let multi_rows: Vec<Vec<u8>> = ramp(8 * 13, 16).chunks(8 * 4).map(le16).collect();
    let multi = strips_dng(1, (8, 13), 16, Some(4), &multi_rows);
    // 8-bit, rows of 5 bytes, then 12-bit at width 5 (rows not whole bytes).
    let eight = strips_dng(1, (5, 6), 8, Some(4), &[vec![7u8; 20], vec![9u8; 10]]);
    let odd12 = strips_dng(1, (5, 2), 12, None, &[vec![0xA5u8; 15]]);
    let tiles = tiled(16, Payload::Samples(ramp(21 * 13, 16)));
    let c7_plane = ramp(6 * 7, 12);
    let c7_strips: Vec<Vec<u8>> = c7_plane
        .chunks(6 * 2)
        .map(|b| tiff::lj92_strip_components(b, 6, b.len() / 6, 12, 1, 0x11))
        .collect();
    let c7 = strips_dng(7, (6, 7), 12, Some(2), &c7_strips);
    let tile_plane = ramp(21 * 13, 12);
    let mut chunks = Vec::new();
    for ty in 0..4u32 {
        for tx in 0..3u32 {
            let mut tile = Vec::new();
            for r in 0..4u32 {
                for c in 0..8u32 {
                    let (x, y) = (tx * 8 + c, ty * 4 + r);
                    tile.push(if x < 21 && y < 13 {
                        tile_plane[(y * 21 + x) as usize]
                    } else {
                        0
                    });
                }
            }
            chunks.push(tiff::lj92_strip_components(&tile, 8, 4, 12, 1, 0x11));
        }
    }
    let c7_tiles = tiled(
        12,
        Payload::Encoded {
            compression: 7,
            chunks,
        },
    );
    vec![
        ("uncompressed single strip", single, false),
        ("uncompressed multi-strip", multi, false),
        ("8-bit multi-strip", eight, false),
        ("12-bit odd width", odd12, false),
        ("uncompressed tiles", tiles, false),
        ("c7 multi-strip", c7, true),
        ("c7 tiles", c7_tiles, true),
    ]
}

fn sensor_of(file: &[u8]) -> (Sensor, ByteOrder) {
    let c = Container::parse(file).expect("parses");
    let s = c.sensor().expect("sensor");
    (s, c.byte_order())
}

/// Allocations of `units` + full iteration, and of decoding every unit into a
/// pre-allocated plane; plus the error paths (a foreign unit, a wrong length).
fn unit_allocations(sensor: &Sensor, order: ByteOrder, file: &[u8], rows: u32) -> (usize, usize) {
    let rows = NonZeroU32::new(rows).unwrap();
    let n = sensor.width as usize * sensor.height as usize;
    let mut plane = vec![0u16; n];
    let mut last = None;
    let (iter_allocs, yielded) = count(|| {
        let mut k = 0usize;
        for u in units(sensor, file, rows).expect("units") {
            std::hint::black_box(u);
            last = Some(u);
            k += 1;
        }
        k
    });
    assert!(yielded > 0);
    let all: Vec<_> = units(sensor, file, rows).unwrap().collect();
    let width = sensor.width as usize;
    let (decode_allocs, ()) = count(|| {
        for u in &all {
            let at = u.row_start as usize * width;
            let k = u.samples(sensor.width).unwrap();
            unpack_unit_into(sensor, order, file, u, &mut plane[at..at + k]).expect("unit");
        }
    });
    // Error paths: a wrong-length band.
    let u = last.unwrap();
    let (err_allocs, ()) = count(|| {
        let mut short = [0u16; 0];
        let e = unpack_unit_into(sensor, order, file, &u, &mut short);
        std::hint::black_box(e.is_err());
    });
    assert_eq!(err_allocs, 0, "the wrong-length refusal allocated");
    (iter_allocs, decode_allocs)
}

#[test]
fn units_and_unit_decode_allocate_nothing() {
    let fx = fixtures();
    for (label, file, compressed) in &fx {
        let (sensor, order) = sensor_of(file);
        for rows in [1u32, 3, u32::MAX] {
            let (iter_allocs, decode_allocs) = unit_allocations(&sensor, order, file, rows);
            assert_eq!(
                iter_allocs, 0,
                "{label} band {rows}: units + iteration allocated"
            );
            if *compressed {
                // No NEW allocation: the units allocate what unpack_into does
                // (the pre-existing Huffman-table parse, per strip or tile).
                let mut plane = vec![0u16; sensor.width as usize * sensor.height as usize];
                let (whole_allocs, r) = count(|| unpack_into(&sensor, order, file, &mut plane));
                r.expect("unpack_into");
                assert_eq!(
                    decode_allocs, whole_allocs,
                    "{label} band {rows}: the units allocated more than unpack_into"
                );
            } else {
                assert_eq!(
                    decode_allocs, 0,
                    "{label} band {rows}: unit decode allocated"
                );
            }
        }
    }

    // The mismatch path: a unit of one fixture refused by another.
    let (s_from, _) = sensor_of(&fx[1].1); // multi-strip, 13 rows
    let (s_onto, order) = sensor_of(&fx[2].1); // 6 rows
    let foreign = units(&s_from, &fx[1].1, NonZeroU32::new(4).unwrap())
        .unwrap()
        .last()
        .unwrap();
    let mut dst = vec![0u16; foreign.samples(s_onto.width).unwrap()];
    let (mismatch_allocs, r) =
        count(|| unpack_unit_into(&s_onto, order, &fx[2].1, &foreign, &mut dst));
    assert!(r.is_err(), "a foreign unit must be refused");
    assert_eq!(mismatch_allocs, 0, "the mismatch refusal allocated");
}

#[test]
fn counter_counts() {
    // Negative control: without this, a broken allocator hook would make every
    // zero above vacuous.
    let (n, v) = count(|| std::hint::black_box(Vec::<u8>::with_capacity(64)));
    drop(v);
    assert!(n > 0, "the counting allocator missed a Vec allocation");
}

#[test]
fn q2m_units_allocate_nothing() {
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();
    let path = "LEICA-Q2-MONO/L1021223.DNG";
    let Some(file_path) = manifest.get(path).expect("in manifest").require(&root) else {
        return; // SKIP announced by name
    };
    let file = std::fs::read(file_path).expect("read Q2M");
    let (sensor, order) = sensor_of(&file);
    let (iter_allocs, decode_allocs) = unit_allocations(&sensor, order, &file, 704);
    assert_eq!(iter_allocs, 0, "{path}: units + iteration allocated");
    assert_eq!(decode_allocs, 0, "{path}: decoding the 8 units allocated");
}
