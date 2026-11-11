//! `SPEC-039` — `plane::validate`, the unified validate-before-allocate
//! preflight (`DEC-050`, group E of the pre-0.2.0 wave).
//!
//! `validate(sensor, byte_order, file, max_pixels)` must answer, before the
//! caller allocates anything, "how many samples, and will `unpack_into`
//! succeed?" — checking the caller's pixel budget FIRST on both the
//! uncompressed and the Compression-7 path, then running the very same
//! check helpers `unpack_into` runs. Every fixture here is hand-built tier A
//! (`tests/support/tiff.rs` plus the [`Dng`] builder below); the one tier-B
//! check (`corpus_files_validate_and_trip_neither_new_error`) skips loud,
//! naming each file, when the corpus is absent.
//!
//! The three red-proofs replay mutated copies of the crate with one
//! `validate` check disabled each (full coverage, aliasing, budget) and show
//! the corresponding assertion go red — control run first (`DEC-009`).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/tiff.rs"]
mod tiff;

use std::mem::discriminant;
use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::{ByteOrder, Container, Sensor, TAG_ROWS_PER_STRIP};
use irradiance::plane::{unpack_into, validate};
use irradiance::Error;
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Tier-A fixture builder
// ─────────────────────────────────────────────────────────────────────────────

/// Where the external `StripOffsets` / `StripByteCounts` arrays go (for more
/// than one value), and where the strip payload region starts. The IFD block
/// itself ends well under 256; `ARRAYS_AT + 8·N <= PAYLOAD_BASE` holds for the
/// strip counts used here (at most a few hundred).
const ARRAYS_AT: u32 = 256;
const PAYLOAD_BASE: u32 = 8192;

/// A sensor-only TIFF (IFD0 *is* the plane) whose strip tags are given
/// verbatim — `offsets` and `counts` may disagree in length, be empty, point
/// past EOF or alias each other — with `payload` planted at [`PAYLOAD_BASE`].
#[derive(Clone)]
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
    /// `strips` laid out back to back from [`PAYLOAD_BASE`], each strip's
    /// `StripByteCounts` its own length — a well-formed file.
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

/// Write a `u32` array into `data` at `at`, in `order` — resizing to fit.
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

/// Deterministic samples below `2^bits`.
fn samples(count: usize, bits: u32, seed: u32) -> Vec<u16> {
    let modulo = 1u32 << bits;
    let mut state = 0x0390_u32 ^ seed ^ bits;
    (0..count)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 9) % modulo) as u16
        })
        .collect()
}

/// Pack `samples` in the uncompressed wire form for `bits` (`DEC-008`): plain
/// bytes for 8, `u16`s in `order` for 16, an MSB-first bitstream padded to a
/// byte for 12/14.
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

/// A well-formed uncompressed plane of `plane` samples split into strips of
/// `rows_per_strip` rows (`None` = one strip), plus the plane itself.
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

/// A well-formed Compression-7 plane: each `rows_per_strip` band an
/// independent `nf`-component SOF-3 strip (`None` = one strip).
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

/// Parse `data` into its sensor and byte order.
fn sensor_of(data: &[u8]) -> (Sensor, ByteOrder) {
    let container = Container::parse(data).expect("the synthetic container parses");
    let sensor = container.sensor().expect("IFD0 is the sensor plane");
    (sensor, container.byte_order())
}

/// `validate` with no budget, then `unpack_into` with a `width × height` dst.
fn both(data: &[u8]) -> (Result<usize, Error>, Result<Vec<u16>, Error>) {
    let (sensor, order) = sensor_of(data);
    let v = validate(&sensor, order, data, usize::MAX);
    let mut dst = vec![0u16; sensor.width as usize * sensor.height as usize];
    let u = unpack_into(&sensor, order, data, &mut dst).map(|()| dst);
    (v, u)
}

// ─────────────────────────────────────────────────────────────────────────────
// Ok(n) — and then unpack_into Ok
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn validate_ok_n_matches_pixel_count() {
    for order in [Order::Little, Order::Big] {
        for bits in [8u32, 12, 14, 16] {
            for (dims, rps) in [
                ((4u32, 2u32), None),
                ((5, 7), None),
                ((6, 8), Some(2)),
                ((5, 7), Some(3)), // uneven last strip
                ((3, 4), Some(1)),
            ] {
                let (dng, plane) = uncompressed(order, dims, bits, rps);
                let data = dng.build();
                let (sensor, bo) = sensor_of(&data);
                let n = validate(&sensor, bo, &data, usize::MAX).unwrap_or_else(|e| {
                    panic!("{order:?} {bits}-bit {dims:?} rps={rps:?}: validate: {e}")
                });
                assert_eq!(n, (dims.0 * dims.1) as usize);
                // Exactly at the budget is still Ok.
                assert_eq!(validate(&sensor, bo, &data, n).ok(), Some(n));

                let mut dst = vec![0u16; n];
                unpack_into(&sensor, bo, &data, &mut dst).unwrap_or_else(|e| {
                    panic!("{order:?} {bits}-bit {dims:?} rps={rps:?}: validate said Ok: {e}")
                });
                assert_eq!(dst, plane, "{order:?} {bits}-bit {dims:?} rps={rps:?}");
                // The uncompressed bound DEC-050 §2 promises.
                assert!(n as u64 * u64::from(bits) <= 8 * data.len() as u64);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Err(e) ⇒ unpack_into Err of the same variant
// ─────────────────────────────────────────────────────────────────────────────

/// A labelled fixture and the predicate its rejection must satisfy.
type BadLayout = (&'static str, Vec<u8>, fn(&Error) -> bool);

/// The layouts `validate` and `unpack_into` must both reject, each with the
/// variant it must be rejected with.
fn bad_layouts() -> Vec<BadLayout> {
    let order = Order::Little;
    let (good, _) = uncompressed(order, (4, 4), 16, Some(1));
    let mut out: Vec<BadLayout> = Vec::new();

    let mut d = good.clone();
    d.bits = 10;
    out.push(("bad bit depth", d.build(), |e| {
        matches!(e, Error::UnsupportedBitDepth { bits: 10 })
    }));

    let mut d = good.clone();
    d.compression = 34713; // a vendor compression code
    out.push(("unsupported compression", d.build(), |e| {
        matches!(e, Error::UnsupportedCompression { compression: 34713 })
    }));

    let mut d = good.clone();
    d.counts.pop();
    out.push(("strip-count mismatch", d.build(), |e| {
        matches!(
            e,
            Error::UnsupportedStripLayout {
                strip_offsets: 4,
                strip_byte_counts: 3
            }
        )
    }));

    let mut d = good.clone();
    d.offsets.clear();
    d.counts.clear();
    out.push(("zero strips", d.build(), |e| {
        matches!(
            e,
            Error::UnsupportedStripLayout {
                strip_offsets: 0,
                strip_byte_counts: 0
            }
        )
    }));

    let mut d = good.clone();
    d.rows_per_strip = None;
    out.push(("missing RowsPerStrip", d.build(), |e| {
        matches!(e, Error::MissingRowsPerStrip { strips: 4 })
    }));

    let mut d = good.clone();
    d.counts[1] += 2;
    out.push(("wrong StripByteCounts", d.build(), |e| {
        matches!(e, Error::PackedSizeMismatch { .. })
    }));

    let mut d = good.clone();
    d.offsets[3] = PAYLOAD_BASE + 10_000;
    out.push(("strip past EOF", d.build(), |e| {
        matches!(e, Error::Truncated { .. })
    }));

    let mut d = good.clone();
    d.offsets.truncate(2);
    d.counts.truncate(2);
    out.push(("undershoot (strips cover half the rows)", d.build(), |e| {
        matches!(e, Error::PackedSizeMismatch { .. })
    }));

    // The Compression-7 structural rejections.
    let (c7, _) = compression7(order, (4, 4), 12, 1, Some(1));
    let mut d = c7.clone();
    d.offsets.truncate(2);
    d.counts.truncate(2);
    out.push(("Compression-7 undershoot", d.build(), |e| {
        matches!(e, Error::UnsupportedStripLayout { .. })
    }));
    let mut d = c7.clone();
    d.rows_per_strip = None;
    out.push(("Compression-7 missing RowsPerStrip", d.build(), |e| {
        matches!(e, Error::MissingRowsPerStrip { strips: 4 })
    }));
    let mut d = c7.clone();
    d.counts[1] += 1_000;
    out.push(("Compression-7 strip past EOF", d.build(), |e| {
        matches!(e, Error::Truncated { .. })
    }));
    let mut d = c7;
    d.offsets.clear();
    d.counts.clear();
    out.push(("Compression-7 zero strips (tiled)", d.build(), |e| {
        matches!(e, Error::UnsupportedStripLayout { .. })
    }));
    out
}

#[test]
fn validate_err_implies_unpack_err_same_variant() {
    for (label, data, expected) in bad_layouts() {
        let (v, u) = both(&data);
        let ve = v.expect_err(label);
        let ue = u.expect_err(label);
        assert!(expected(&ve), "{label}: validate gave {ve:?}");
        assert_eq!(
            discriminant(&ve),
            discriminant(&ue),
            "{label}: validate {ve:?} vs unpack_into {ue:?}"
        );
    }
}

/// The no-drift check (`SPEC-039` AC "No drift"): mutate a good layout every
/// way a hostile file can — each strip's count ±1, each offset past EOF, the
/// `RowsPerStrip` value, the strip list's length — and require `validate` and
/// `unpack_into` to agree on every one: `Ok` ⇔ `Ok`, and the same variant on
/// `Err`.
#[test]
fn validate_and_unpack_agree_on_mutated_layouts() {
    let mut cases: Vec<Dng> = Vec::new();
    for bits in [12u32, 16] {
        let (good, _) = uncompressed(Order::Big, (5, 6), bits, Some(2));
        cases.push(good.clone());
        for i in 0..good.counts.len() {
            for delta in [-1i64, 1] {
                let mut d = good.clone();
                d.counts[i] = (i64::from(d.counts[i]) + delta) as u32;
                cases.push(d);
            }
            let mut d = good.clone();
            d.offsets[i] = u32::MAX - 3;
            cases.push(d);
        }
        for rps in [0u32, 1, 3, 4, 6, 7, u32::MAX] {
            let mut d = good.clone();
            d.rows_per_strip = Some(rps);
            cases.push(d);
        }
        for keep in 0..good.offsets.len() {
            let mut d = good.clone();
            d.offsets.truncate(keep);
            d.counts.truncate(keep);
            cases.push(d);
        }
        let mut d = good.clone();
        d.offsets.push(PAYLOAD_BASE);
        d.counts.push(0); // an extra zero-row strip
        cases.push(d);
    }
    let (c7, _) = compression7(Order::Little, (4, 6), 12, 1, Some(2));
    cases.push(c7.clone());
    for rps in [0u32, 1, 3, 6, 9] {
        let mut d = c7.clone();
        d.rows_per_strip = Some(rps);
        cases.push(d);
    }
    for i in 0..c7.counts.len() {
        let mut d = c7.clone();
        d.counts[i] += 4_000;
        cases.push(d);
    }

    let mut oks = 0;
    for (i, dng) in cases.iter().enumerate() {
        let data = dng.build();
        let (v, u) = both(&data);
        match (&v, &u) {
            (Ok(n), Ok(plane)) => {
                assert_eq!(*n, plane.len(), "case {i}");
                oks += 1;
            }
            (Err(ve), Err(ue)) => {
                assert_eq!(
                    discriminant(ve),
                    discriminant(ue),
                    "case {i}: {ve:?} vs {ue:?}"
                );
            }
            (Ok(_), Err(ue)) if dng.compression == 7 => {
                // validate Ok, decode failed: only the decode-time family may
                // do that (DEC-050 §3).
                assert!(
                    matches!(
                        ue,
                        Error::LjpegPlaneShapeMismatch { .. } | Error::LjpegPlaneWrongLength { .. }
                    ),
                    "case {i}: validate Ok but unpack_into {ue:?}"
                );
            }
            _ => panic!("case {i}: validate {v:?} disagrees with unpack_into {u:?}"),
        }
    }
    assert!(oks >= 3, "the sweep must include accepted layouts too");
}

// ─────────────────────────────────────────────────────────────────────────────
// The budget — first, on both paths
// ─────────────────────────────────────────────────────────────────────────────

/// A tiny file claiming `width × height`, with a 64-byte strip.
fn forged(compression: u16, width: u32, height: u32) -> Vec<u8> {
    let strip = if compression == 7 {
        tiff::lj92_strip(&samples(4, 12, 1), 2, 2, 12)
    } else {
        vec![0u8; 64]
    };
    Dng::contiguous(
        Order::Little,
        compression,
        (width, height),
        if compression == 7 { 12 } else { 16 },
        None,
        &[strip],
    )
    .build()
}

#[test]
fn budget_rejects_forged_dims_both_paths() {
    const BUDGET: usize = 1 << 24; // a caller's 16-megapixel budget
    for compression in [1u16, 7] {
        for (w, h) in [(65_535u32, 65_535u32), (u32::MAX, u32::MAX)] {
            let data = forged(compression, w, h);
            let (sensor, order) = sensor_of(&data);
            let claim = u64::from(w) * u64::from(h);
            // No allocation proportional to the claim happens anywhere here:
            // validate takes no buffer, and the claim is never materialised.
            match validate(&sensor, order, &data, BUDGET) {
                Err(Error::PlaneTooLarge { pixels, limit }) => {
                    assert_eq!(
                        (pixels, limit),
                        (claim, BUDGET),
                        "compression {compression}"
                    );
                }
                other => panic!("compression {compression} {w}x{h}: {other:?}"),
            }
        }
    }

    // FIRST: an over-budget plane whose layout is ALSO broken in every other
    // way still reports PlaneTooLarge — the budget runs before any other check.
    for (label, data, _) in bad_layouts() {
        let (sensor, order) = sensor_of(&data);
        let n = sensor.width as usize * sensor.height as usize;
        match validate(&sensor, order, &data, n - 1) {
            Err(Error::PlaneTooLarge { pixels, limit }) => {
                assert_eq!((pixels, limit), (n as u64, n - 1), "{label}");
            }
            other => panic!("{label}: the budget must fire first; got {other:?}"),
        }
    }

    // Why the budget exists: with NO budget, a forged Compression-7 plane is
    // structurally fine — the file-bytes bound does not apply to compressed
    // strips — so validate returns the attacker's n. The caller's max_pixels
    // is the only pre-allocation bound.
    let data = forged(7, 65_535, 65_535);
    let (sensor, order) = sensor_of(&data);
    assert_eq!(
        validate(&sensor, order, &data, usize::MAX).ok(),
        Some(65_535 * 65_535)
    );
    // The uncompressed forgery is refused by layout even with no budget.
    let data = forged(1, 65_535, 65_535);
    let (sensor, order) = sensor_of(&data);
    assert!(validate(&sensor, order, &data, usize::MAX).is_err());

    // One under the budget is refused, exactly at it is accepted.
    let (dng, _) = uncompressed(Order::Little, (4, 2), 14, None);
    let data = dng.build();
    let (sensor, order) = sensor_of(&data);
    assert!(matches!(
        validate(&sensor, order, &data, 7),
        Err(Error::PlaneTooLarge {
            pixels: 8,
            limit: 7
        })
    ));
    assert_eq!(validate(&sensor, order, &data, 8).ok(), Some(8));
}

// ─────────────────────────────────────────────────────────────────────────────
// Aliased strips — the uncompressed file-bytes bound
// ─────────────────────────────────────────────────────────────────────────────

/// `strips` one-row strips of a `width`-wide 16-bit plane, every one pointing
/// at the SAME `2·width` bytes. Each strip is individually well-formed and in
/// bounds; together they claim `strips × 2·width` bytes of a much smaller file.
fn aliased(width: u32, strips: u32) -> Dng {
    let one = pack(&samples(width as usize, 16, 3), 16, Order::Little);
    Dng {
        order: Order::Little,
        compression: 1,
        width,
        height: strips,
        bits: 16,
        rows_per_strip: Some(1),
        offsets: vec![PAYLOAD_BASE; strips as usize],
        counts: vec![one.len() as u32; strips as usize],
        payload: one,
    }
}

#[test]
fn aliased_strips_are_rejected() {
    let dng = aliased(64, 200);
    let data = dng.build();
    let declared: u64 = dng.counts.iter().map(|&c| u64::from(c)).sum();
    assert!(
        declared > data.len() as u64,
        "the fixture must actually alias: {declared} declared vs {} file bytes",
        data.len()
    );
    let (v, u) = both(&data);
    match v {
        Err(Error::StripBytesExceedFile {
            declared: d,
            file_len,
        }) => {
            assert_eq!((d, file_len), (declared, data.len()));
        }
        other => panic!("aliased strips must be StripBytesExceedFile; got {other:?}"),
    }
    assert!(
        matches!(u, Err(Error::StripBytesExceedFile { .. })),
        "unpack_into shares the check: {u:?}"
    );

    // Aliasing that stays within the file length is not what this guards
    // (the plane is still bounded by the bytes supplied), so it is accepted;
    // the bound n·bits <= 8·len is what matters, and it holds.
    let small = aliased(64, 4).build();
    let (sensor, order) = sensor_of(&small);
    let n = validate(&sensor, order, &small, usize::MAX).expect("in-bound aliasing");
    assert!(n as u64 * 16 <= 8 * small.len() as u64);

    // No Ok(n) violates the bound, across every aliasing width and count.
    for width in [1u32, 8, 64, 200] {
        for strips in [1u32, 2, 16, 64, 300] {
            let data = aliased(width, strips).build();
            let (sensor, order) = sensor_of(&data);
            if let Ok(n) = validate(&sensor, order, &data, usize::MAX) {
                assert!(
                    n as u64 * 16 <= 8 * data.len() as u64,
                    "width {width} strips {strips}: Ok({n}) breaks n·bits <= 8·len"
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compression-7
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn compression7_valid_plane_validates_then_unpacks() {
    for order in [Order::Little, Order::Big] {
        for nf in [1u8, 2] {
            for rps in [None, Some(2)] {
                let (dng, plane) = compression7(order, (6, 4), 12, nf, rps);
                let data = dng.build();
                let (sensor, bo) = sensor_of(&data);
                let n = validate(&sensor, bo, &data, usize::MAX)
                    .unwrap_or_else(|e| panic!("{order:?} Nf={nf} rps={rps:?}: {e}"));
                assert_eq!(n, 24);
                let mut dst = vec![0u16; n];
                unpack_into(&sensor, bo, &data, &mut dst)
                    .unwrap_or_else(|e| panic!("{order:?} Nf={nf} rps={rps:?}: {e}"));
                assert_eq!(dst, plane, "{order:?} Nf={nf} rps={rps:?}");
                // A compressed plane is bounded only by the budget.
                assert!(matches!(
                    validate(&sensor, bo, &data, 23),
                    Err(Error::PlaneTooLarge {
                        pixels: 24,
                        limit: 23
                    })
                ));
            }
        }
    }

    // The SPEC-053 transpose: structurally fine (validate Ok — it reads no
    // strip bytes), refused at decode by the shape binding.
    let plane = samples(24, 12, 9);
    let strip = tiff::lj92_strip(&plane, 4, 6, 12);
    let data = tiff::compression7_dng_with_strip(Order::Little, 6, 4, 12, &strip);
    let (v, u) = both(&data);
    assert_eq!(v.ok(), Some(24), "validate does not decode");
    assert!(
        matches!(
            u,
            Err(Error::LjpegPlaneShapeMismatch {
                expected_width: 6,
                expected_height: 4,
                decoded_width: 4,
                decoded_height: 6,
            })
        ),
        "the shape binding still fires at decode: {u:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Precedence and purity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn wrong_dst_and_bad_layout_keeps_old_precedence() {
    for (label, data, _) in bad_layouts() {
        let (sensor, order) = sensor_of(&data);
        let n = sensor.width as usize * sensor.height as usize;
        // Every bad layout whose compression and depth are recognised: a
        // wrong-length dst must win over the layout error, as before SPEC-039.
        if matches!(
            validate(&sensor, order, &data, usize::MAX),
            Err(Error::UnsupportedBitDepth { .. } | Error::UnsupportedCompression { .. })
        ) {
            continue;
        }
        for wrong in [n - 1, n + 1] {
            let mut dst = vec![0u16; wrong];
            let err = unpack_into(&sensor, order, &data, &mut dst).expect_err(label);
            assert!(
                matches!(err, Error::PlaneBufferWrongLength { expected, actual }
                    if expected == n as u64 && actual == wrong),
                "{label}: dst {wrong}: got {err:?}"
            );
        }
    }
    // The aliasing rejection too.
    let data = aliased(64, 200).build();
    let (sensor, order) = sensor_of(&data);
    let mut dst = vec![0u16; 3];
    assert!(matches!(
        unpack_into(&sensor, order, &data, &mut dst),
        Err(Error::PlaneBufferWrongLength { .. })
    ));
}

#[test]
fn validate_is_pure() {
    // Takes only shared references and a budget; returns a count, not a buffer.
    let f: fn(&Sensor, ByteOrder, &[u8], usize) -> Result<usize, Error> = validate;

    let mut fixtures: Vec<Vec<u8>> = bad_layouts().into_iter().map(|(_, d, _)| d).collect();
    fixtures.push(uncompressed(Order::Big, (6, 8), 14, Some(3)).0.build());
    fixtures.push(compression7(Order::Little, (6, 4), 12, 2, None).0.build());
    fixtures.push(aliased(64, 200).build());
    for data in fixtures {
        let before = data.clone();
        let (sensor, order) = sensor_of(&data);
        let other = match order {
            ByteOrder::Little => ByteOrder::Big,
            ByteOrder::Big => ByteOrder::Little,
        };
        for budget in [usize::MAX, 3] {
            let a = format!("{:?}", f(&sensor, order, &data, budget));
            let b = format!("{:?}", f(&sensor, order, &data, budget));
            // byte_order is parity-only: it never changes the answer.
            let c = format!("{:?}", f(&sensor, other, &data, budget));
            assert_eq!(a, b);
            assert_eq!(a, c);
        }
        assert_eq!(data, before, "validate must not write the input");
    }
}

#[test]
fn new_errors_display_their_numbers() {
    let msg = Error::StripBytesExceedFile {
        declared: 25_600,
        file_len: 8_448,
    }
    .to_string();
    assert!(msg.contains("25600") && msg.contains("8448"), "{msg}");
    let msg = Error::PlaneTooLarge {
        pixels: 4_294_836_225,
        limit: 16_777_216,
    }
    .to_string();
    assert!(
        msg.contains("4294836225") && msg.contains("16777216"),
        "{msg}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EOI optional (DEC-050 §4)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn eoi_optional() {
    let (w, h) = (6u32, 4u32);
    for nf in [1u8, 2] {
        let plane = samples((w * h) as usize, 12, u32::from(nf));
        let full = tiff::lj92_strip_components(&plane, w as usize, h as usize, 12, nf, 0x11);
        assert_eq!(
            &full[full.len() - 2..],
            &[0xFF, 0xD9],
            "the encoder ends in EOI"
        );

        // The trailing EOI removed: the scan completes on sample count.
        let no_eoi = &full[..full.len() - 2];
        let data = tiff::compression7_dng_with_strip(Order::Little, w, h, 12, no_eoi);
        let (v, u) = both(&data);
        assert_eq!(v.ok(), Some(24), "Nf={nf}");
        assert_eq!(
            u.unwrap_or_else(|e| panic!("Nf={nf}: a missing EOI must not be an error: {e}")),
            plane,
            "Nf={nf}"
        );

        // A scan cut before width·height samples is Truncated, EOI or not.
        let (_, _, entropy_at) = sof3_entropy_offset(&full);
        let cut = &full[..entropy_at + 2];
        let data = tiff::compression7_dng_with_strip(Order::Little, w, h, 12, cut);
        let (v, u) = both(&data);
        assert_eq!(v.ok(), Some(24), "Nf={nf}: structurally fine");
        assert!(
            matches!(u, Err(Error::Truncated { .. })),
            "Nf={nf}: a truncated scan must be Truncated; got {u:?}"
        );
    }
}

/// The byte offset just past the `SOS` segment of a well-formed SOF-3 stream
/// (where the entropy-coded scan starts), found by walking the marker
/// segments. Returns `(sos_at, sos_len, entropy_at)`.
fn sof3_entropy_offset(stream: &[u8]) -> (usize, usize, usize) {
    let mut at = 2; // past SOI
    loop {
        assert_eq!(stream[at], 0xFF, "marker expected at {at}");
        let marker = stream[at + 1];
        let len = usize::from(u16::from_be_bytes([stream[at + 2], stream[at + 3]]));
        if marker == 0xDA {
            return (at, len, at + 2 + len);
        }
        at += 2 + len;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B — the corpus validates, and trips neither new error
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn corpus_files_validate_and_trip_neither_new_error() {
    let root = CorpusRoot::resolve();
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let mut checked = 0;
    for entry in &manifest.files {
        let Some(path) = entry.require(&root) else {
            eprintln!(
                "SKIP corpus_files_validate_and_trip_neither_new_error for {} — not present \
                 under {} ({})",
                entry.path,
                root.path.display(),
                root.origin()
            );
            continue;
        };
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let container =
            Container::parse(&bytes).unwrap_or_else(|e| panic!("{}: parse: {e}", entry.path));
        let Ok(sensor) = container.sensor() else {
            eprintln!("{}: no sensor plane — not a plane fixture", entry.path);
            continue;
        };
        let order = container.byte_order();
        let v = validate(&sensor, order, &bytes, usize::MAX);
        assert!(
            !matches!(
                v,
                Err(Error::StripBytesExceedFile { .. } | Error::PlaneTooLarge { .. })
            ),
            "{}: a real file tripped a SPEC-039 guard: {v:?}",
            entry.path
        );
        let mut plane = vec![0u16; sensor.width as usize * sensor.height as usize];
        let u = unpack_into(&sensor, order, &bytes, &mut plane);
        match (&v, &u) {
            (Ok(n), Ok(())) => {
                assert_eq!(*n, plane.len(), "{}", entry.path);
                let le: Vec<u8> = plane.iter().flat_map(|s| s.to_le_bytes()).collect();
                let digest = md5::to_hex(&md5::hash(&le));
                assert_eq!(
                    digest, entry.oracle.raw_checksum,
                    "{}: plane MD5 drifted from the pinned dnglab checksum",
                    entry.path
                );
                eprintln!(
                    "{}: validate Ok({n}), unpack_into bit-exact (MD5 {digest})",
                    entry.path
                );
            }
            (Err(ve), Err(ue)) => {
                assert_eq!(discriminant(ve), discriminant(ue), "{}", entry.path);
                eprintln!("{}: validate and unpack_into both refuse: {ve}", entry.path);
            }
            _ => panic!(
                "{}: validate {v:?} disagrees with unpack_into {u:?}",
                entry.path
            ),
        }
        checked += 1;
    }
    eprintln!("corpus: {checked} file(s) checked");
}

// ─────────────────────────────────────────────────────────────────────────────
// The three red-proofs (oracle-must-be-shown-red, DEC-009)
// ─────────────────────────────────────────────────────────────────────────────
//
// A copy of the crate is built in a temp dir with a small probe binary that
// parses a fixture DNG, runs the public `validate` with a given budget, and
// prints `ok <n>` or `err <Debug>`. The CONTROL copy is unmutated and must
// reproduce the honest tree's rejection; each MUTANT copy has one check's
// condition forced false and must accept its fixture. Copied from
// `tests/ljpeg_shape_binding.rs`'s `stage_probe_crate` (copied, not shared, as
// the spec asks).

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-plane-validate-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("create temp dir {}: {e}", dir.display()));
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {}: {e}", dst.display()));
    for entry in
        std::fs::read_dir(src).unwrap_or_else(|e| panic!("read_dir {}: {e}", src.display()))
    {
        let entry = entry.expect("dir entry");
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type().expect("file_type");
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {}: {e}", target.display()));
        }
    }
}

/// One `validate` check to disable: its exact condition in `src/plane.rs`. If
/// the condition moves, the red-proof fails loudly rather than silently
/// mutating nothing.
struct Guard {
    needle: &'static str,
}

/// (a) Full row coverage, in the shared `check_strips`.
const COVERAGE_GUARD: Guard = Guard {
    needle: "if walk.remaining_samples != 0 {",
};

/// (b) The sum-of-`StripByteCounts` aliasing bound, in `check_strips`.
const ALIASING_GUARD: Guard = Guard {
    needle: "if declared > file_len {",
};

/// (c) The caller's pixel budget, first in `validate`.
const BUDGET_GUARD: Guard = Guard {
    needle: "if pixels > budget {",
};

/// Disable `guard` in the staged copy: the condition is kept but can never be
/// true, so the check is "removed" without touching anything else.
fn inject_guard_removed(crate_dir: &Path, guard: &Guard) {
    let path = crate_dir.join("src/plane.rs");
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let occurrences = src.matches(guard.needle).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one guard condition {:?} in src/plane.rs; found {occurrences} — the \
         guard moved, update the needle",
        guard.needle
    );
    let disabled = guard.needle.replacen("if ", "if false && (", 1).replacen(
        " {",
        ") { // RED-PROOF INJECTION -- tests/plane_validate.rs",
        1,
    );
    let mutated = src.replacen(guard.needle, &disabled, 1);
    assert_ne!(mutated, src, "the injection must change the source");
    std::fs::write(&path, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", path.display()));
}

const PROBE_MAIN: &str = r#"
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("fixture path");
    let budget: usize = args.next().expect("budget").parse().expect("budget is a usize");
    let data = std::fs::read(&path).expect("read fixture");
    let container = irradiance::ifd::Container::parse(&data).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor plane");
    match irradiance::plane::validate(&sensor, container.byte_order(), &data, budget) {
        Ok(n) => println!("ok {n}"),
        Err(e) => println!("err {e:?}"),
    }
}
"#;

fn stage_probe_crate(dest: &Path, guard: Option<&Guard>) {
    let repo_root = corpus::crate_root();

    std::fs::create_dir_all(dest.join("src/bin")).expect("mkdir src/bin");
    std::fs::copy(repo_root.join("Cargo.toml"), dest.join("Cargo.toml")).expect("copy Cargo.toml");
    let lock = repo_root.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, dest.join("Cargo.lock")).expect("copy Cargo.lock");
    }
    copy_dir_recursive(&repo_root.join("src"), &dest.join("src"));

    if let Some(guard) = guard {
        inject_guard_removed(dest, guard);
    }

    std::fs::write(dest.join("src/bin/plane_validate_probe.rs"), PROBE_MAIN)
        .expect("write probe binary source");
    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml.push_str(
        "\n[[bin]]\nname = \"plane_validate_probe\"\npath = \"src/bin/plane_validate_probe.rs\"\n",
    );
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

/// Build the staged crate and run the probe on `fixture` at `budget`.
fn build_and_run_probe(dir: &Path, fixture: &Path, budget: usize) -> String {
    let build = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "plane_validate_probe",
            "--quiet",
        ])
        .current_dir(dir)
        .output()
        .expect("spawn cargo build");
    assert!(
        build.status.success(),
        "cargo build --release failed in {}:\n{}",
        dir.display(),
        String::from_utf8_lossy(&build.stderr)
    );
    let bin = dir.join("target/release/plane_validate_probe");
    let run = std::process::Command::new(&bin)
        .arg(fixture)
        .arg(budget.to_string())
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "plane_validate_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .to_string()
}

/// Control then mutant for one check. `honest_err_prefix` is the start of the
/// honest tree's rejection; the mutant must instead return `ok <n>`.
fn red_proof(label: &str, data: &[u8], budget: usize, guard: &Guard, honest_err_prefix: &str) {
    let fixture_dir = TempDir::new(&format!("{label}-fixture"));
    let fixture = fixture_dir.0.join(format!("{label}.dng"));
    std::fs::write(&fixture, data).expect("write fixture");

    // CONTROL first (DEC-009): the unmutated copy-and-rebuild apparatus must
    // reproduce the honest rejection, so a red below is attributable to the
    // injection and not to a broken copy or build.
    let control_dir = TempDir::new(&format!("{label}-control"));
    stage_probe_crate(&control_dir.0, None);
    let control = build_and_run_probe(&control_dir.0, &fixture, budget);
    assert!(
        control.starts_with(honest_err_prefix),
        "{label}: the UNMUTATED copy must reject the fixture with {honest_err_prefix:?}; got \
         {control:?}"
    );

    // MUTANT: the check disabled. The corresponding test's assertion —
    // "validate returns the typed error" — must now FAIL: validate says Ok.
    let mutant_dir = TempDir::new(&format!("{label}-mutant"));
    stage_probe_crate(&mutant_dir.0, Some(guard));
    let mutant = build_and_run_probe(&mutant_dir.0, &fixture, budget);
    assert_ne!(
        mutant, control,
        "{label}: disabling the check did NOT change the output — the injection is a semantic \
         no-op and this red-proof has caught NOTHING"
    );
    assert!(
        mutant.starts_with("ok "),
        "{label}: with the check disabled validate must return Ok (the red); got {mutant:?}"
    );

    eprintln!("RED-PROOF ({label}): control={control:?} mutant={mutant:?}");
}

#[test]
fn validate_red_proof_coverage_removed() {
    // Strips that cover only half the rows of a 4-row plane.
    let (mut dng, _) = uncompressed(Order::Little, (4, 4), 16, Some(1));
    dng.offsets.truncate(2);
    dng.counts.truncate(2);
    let data = dng.build();
    let (sensor, order) = sensor_of(&data);
    assert!(
        matches!(
            validate(&sensor, order, &data, usize::MAX),
            Err(Error::PackedSizeMismatch { .. })
        ),
        "the honest tree must reject the undershoot, or the red below proves nothing"
    );
    red_proof(
        "coverage",
        &data,
        usize::MAX,
        &COVERAGE_GUARD,
        "err PackedSizeMismatch",
    );
}

#[test]
fn validate_red_proof_aliasing_removed() {
    let data = aliased(64, 200).build();
    let (sensor, order) = sensor_of(&data);
    assert!(
        matches!(
            validate(&sensor, order, &data, usize::MAX),
            Err(Error::StripBytesExceedFile { .. })
        ),
        "the honest tree must reject the aliased strips, or the red below proves nothing"
    );
    red_proof(
        "aliasing",
        &data,
        usize::MAX,
        &ALIASING_GUARD,
        "err StripBytesExceedFile",
    );
}

#[test]
fn validate_red_proof_budget_removed() {
    // A valid 4×2 plane against a 4-pixel budget.
    let data = uncompressed(Order::Little, (4, 2), 14, None).0.build();
    let (sensor, order) = sensor_of(&data);
    assert!(
        matches!(
            validate(&sensor, order, &data, 4),
            Err(Error::PlaneTooLarge {
                pixels: 8,
                limit: 4
            })
        ),
        "the honest tree must refuse the over-budget plane, or the red below proves nothing"
    );
    red_proof("budget", &data, 4, &BUDGET_GUARD, "err PlaneTooLarge");
}
