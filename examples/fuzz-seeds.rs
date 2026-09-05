//! Write the fuzz seed corpus for `fuzz/fuzz_targets/ifd.rs`,
//! `fuzz/fuzz_targets/plane.rs`, and `fuzz/fuzz_targets/develop.rs`.
//!
//! `SPEC-003`, extended by `SPEC-012` and `SPEC-014`. The `ifd` seeds are the
//! hand-built tier-A fixtures in `tests/support/tiff.rs` — the same list
//! `tests/ifd_reader.rs` asserts against, so a fixture cannot be added to the
//! test lane and forgotten by the fuzz lane. The `plane` seeds are built
//! locally in [`plane_seeds`] from the strip bytes `tests/plane_unpack.rs`
//! also uses — `DEC-008`/`SPEC-012`'s measured `## Implementation Context`,
//! not any corpus file. The `develop` seeds are built locally in
//! [`develop_seeds`], covering the geometry surface `SPEC-014` `AC6` names
//! (an out-of-range crop, an inverted `ActiveArea`, `Orientation` outside
//! `1..=8`, `BlackLevel >= WhiteLevel`) plus the two shapes that develop
//! successfully. Everything here is **own work built from TIFF 6.0 §2**:
//! tier B is never committed (`DEC-003`), and a truncated 86 MB Leica frame
//! would still be a Leica frame.
//!
//! ```text
//! cargo run --example fuzz-seeds     # rewrite fuzz/seeds/{ifd,plane,develop}/
//! ```
//!
//! The output directories are committed, and `fuzz/corpus/*` — where
//! libFuzzer writes what it discovers — is not. Run each target against both:
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
//!     fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane \
//!     fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=60
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run develop \
//!     fuzz/corpus/develop fuzz/seeds/develop -- -max_total_time=60
//! ```

#[path = "../tests/support/tiff.rs"]
mod tiff;

use std::path::PathBuf;

/// A minimal sensor-only TIFF (IFD0 *is* the sensor plane) with a strip
/// payload planted at `strip_offset`. Mirrors `tests/plane_unpack.rs`'s
/// `Fixture::build`, which cannot be imported here (it is test-only code in a
/// different compilation unit) — kept in sync by hand; both are exercised by
/// `just test` (the test) and `just fuzz-plane` (the seed), so drift shows up
/// as a fuzz crash or a test failure, not silently.
fn plane_fixture(
    order: tiff::Order,
    width: u32,
    height: u32,
    bits: u16,
    strip_offset: u32,
    strip_byte_count: u32,
    strip_bytes: &[u8],
) -> Vec<u8> {
    let entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, width),
        tiff::long(tiff::IMAGE_LENGTH, height),
        tiff::short(tiff::BITS_PER_SAMPLE, bits, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::long(tiff::STRIP_OFFSETS, strip_offset),
        tiff::long(tiff::STRIP_BYTE_COUNTS, strip_byte_count),
    ];
    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
    let at = strip_offset as usize;
    let end = at + strip_bytes.len();
    if data.len() < end {
        data.resize(end, 0);
    }
    data[at..end].copy_from_slice(strip_bytes);
    data
}

/// Seeds for `fuzz/fuzz_targets/plane.rs` — reaching BOTH of `DEC-008`'s
/// paths is the point (a target that only ever drives one recreates
/// `SPIKE-001`'s exact blind spot), plus the hostile shapes `SPEC-012`
/// `AC7` names.
fn plane_seeds() -> Vec<(&'static str, Vec<u8>)> {
    // L1021223.DNG's strip head (DEC-008/SPEC-012 Implementation Context).
    const Q2M_STRIP: [u8; 14] = [
        0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
    ];
    // L1000622.DNG's strip head.
    const M_MONO_STRIP: [u8; 16] = [
        0x99, 0x12, 0xef, 0x11, 0x0e, 0x12, 0x0b, 0x11, 0xbe, 0x11, 0x1f, 0x11, 0x00, 0x12, 0xbe,
        0x10,
    ];

    let compressed = {
        let entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, 4),
            tiff::long(tiff::IMAGE_LENGTH, 2),
            tiff::short(tiff::BITS_PER_SAMPLE, 14, tiff::Order::Little),
            tiff::short(tiff::COMPRESSION, 7, tiff::Order::Little),
            tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, tiff::Order::Little),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, tiff::Order::Little),
            tiff::long(tiff::STRIP_OFFSETS, 512),
            tiff::long(tiff::STRIP_BYTE_COUNTS, 14),
        ];
        tiff::tiff(tiff::Order::Little, 8, &[tiff::Ifd::new(8, entries, 0)])
    };

    vec![
        (
            // The sub-byte path (bits % 8 != 0).
            "valid-fourteen-bit",
            plane_fixture(tiff::Order::Little, 4, 2, 14, 512, 14, &Q2M_STRIP),
        ),
        (
            // The byte-aligned path (bits % 8 == 0).
            "valid-sixteen-bit",
            plane_fixture(tiff::Order::Little, 8, 1, 16, 512, 16, &M_MONO_STRIP),
        ),
        (
            // AC3/AC4: the 14-bit strip misdeclared as 16-bit-aligned —
            // decodes without error unless something checks the values.
            "wrong-bits-as-sixteen",
            plane_fixture(tiff::Order::Little, 7, 1, 16, 512, 14, &Q2M_STRIP),
        ),
        (
            // AC7: StripByteCounts promises bytes the file does not hold.
            "truncated-strip",
            plane_fixture(tiff::Order::Little, 4, 2, 14, 512, 14, &[]),
        ),
        (
            // AC7: bits_per_sample outside {8, 12, 14, 16}.
            "odd-bit-depth",
            plane_fixture(tiff::Order::Little, 4, 2, 10, 512, 10, &[0u8; 10]),
        ),
        (
            // AC6: Compression = 7 must never reach the unpack path at all.
            "compressed-plane",
            compressed,
        ),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Seeds for `fuzz/fuzz_targets/develop.rs` (SPEC-014)
// ─────────────────────────────────────────────────────────────────────────────

/// DNG tags `tests/support/tiff.rs` does not name — `ActiveArea`,
/// `DefaultCropOrigin`/`Size`, `Orientation`, `BlackLevel`/`WhiteLevel`
/// (`irradiance::ifd::TAG_*`, restated here as raw numbers so this file does
/// not need to depend on the library crate for constants alone).
const TAG_ORIENTATION: u16 = 274;
const TAG_BLACK_LEVEL: u16 = 50714;
const TAG_WHITE_LEVEL: u16 = 50717;
const TAG_DEFAULT_CROP_ORIGIN: u16 = 50719;
const TAG_DEFAULT_CROP_SIZE: u16 = 50720;
const TAG_ACTIVE_AREA: u16 = 50829;

/// A single-strip sensor IFD (byte-aligned, 16-bit — the simplest path,
/// since this seed set is exercising GEOMETRY, not `DEC-008`'s bit-packing)
/// with an optional geometry/levels tag set, each array written as an
/// explicit-offset LONG array after the strip — the same technique
/// `tests/support/tiff.rs`'s `rational_default_crop_size` uses for a
/// multi-value tag that cannot fit inline.
#[allow(clippy::too_many_arguments)]
fn develop_fixture(
    width: u32,
    height: u32,
    active_area: Option<[u32; 4]>,
    crop_origin: Option<[u32; 2]>,
    crop_size: Option<[u32; 2]>,
    orientation: Option<u32>,
    black_level: Option<u32>,
    white_level: Option<u32>,
) -> Vec<u8> {
    let order = tiff::Order::Little;
    let strip_offset: u32 = 512;
    let pixel_count = usize::try_from(width).unwrap_or(0) * usize::try_from(height).unwrap_or(0);
    let strip_bytes: Vec<u8> = (0..pixel_count * 2).map(|i| (i % 256) as u8).collect();
    let strip_byte_count = u32::try_from(strip_bytes.len()).unwrap_or(0);

    let mut entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, width),
        tiff::long(tiff::IMAGE_LENGTH, height),
        tiff::short(tiff::BITS_PER_SAMPLE, 16, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::long(tiff::STRIP_OFFSETS, strip_offset),
        tiff::long(tiff::STRIP_BYTE_COUNTS, strip_byte_count),
    ];
    if let Some(v) = orientation {
        entries.push(tiff::short(
            TAG_ORIENTATION,
            u16::try_from(v).unwrap_or(u16::MAX),
            order,
        ));
    }
    if let Some(v) = black_level {
        entries.push(tiff::long(TAG_BLACK_LEVEL, v));
    }
    if let Some(v) = white_level {
        entries.push(tiff::long(TAG_WHITE_LEVEL, v));
    }

    // Arrays go after the strip; each reserves 16 bytes (4 LONGs) whether or
    // not it uses all of them, so offsets stay easy to compute.
    let mut next_array_offset = strip_offset as usize + strip_bytes.len();
    // Align up to a 4-byte boundary — TIFF offsets need not be aligned, but
    // keeping them so makes this file easier to read.
    next_array_offset = next_array_offset.div_ceil(4) * 4;

    let mut arrays: Vec<(u16, u32, Vec<u32>)> = Vec::new(); // (tag, offset, values)
    if let Some(values) = active_area {
        arrays.push((TAG_ACTIVE_AREA, next_array_offset as u32, values.to_vec()));
        entries.push(tiff::at_offset(
            TAG_ACTIVE_AREA,
            4, // LONG
            4,
            next_array_offset as u32,
        ));
        next_array_offset += 16;
    }
    if let Some(values) = crop_origin {
        arrays.push((
            TAG_DEFAULT_CROP_ORIGIN,
            next_array_offset as u32,
            values.to_vec(),
        ));
        entries.push(tiff::at_offset(
            TAG_DEFAULT_CROP_ORIGIN,
            4,
            2,
            next_array_offset as u32,
        ));
        next_array_offset += 8;
    }
    if let Some(values) = crop_size {
        arrays.push((
            TAG_DEFAULT_CROP_SIZE,
            next_array_offset as u32,
            values.to_vec(),
        ));
        entries.push(tiff::at_offset(
            TAG_DEFAULT_CROP_SIZE,
            4,
            2,
            next_array_offset as u32,
        ));
        // No further array follows `DefaultCropSize` in this fixture builder,
        // so `next_array_offset` need not advance again.
    }

    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);

    let strip_at = strip_offset as usize;
    let strip_end = strip_at + strip_bytes.len();
    if data.len() < strip_end {
        data.resize(strip_end, 0);
    }
    data[strip_at..strip_end].copy_from_slice(&strip_bytes);

    for (_tag, offset, values) in arrays {
        let at = offset as usize;
        let end = at + values.len() * 4;
        if data.len() < end {
            data.resize(end, 0);
        }
        for (i, v) in values.iter().enumerate() {
            let slot = at + i * 4;
            data[slot..slot + 4].copy_from_slice(&v.to_le_bytes());
        }
    }

    data
}

/// Seeds for `fuzz/fuzz_targets/develop.rs` — reaching every geometry
/// rejection `SPEC-014` `AC6` names, plus the two shapes that actually
/// develop successfully (unrotated and rotated), so the target exercises the
/// happy path as well as the hostile one.
fn develop_seeds() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            "unrotated-full-plane",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([1, 1]),
                Some([6, 4]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            "rotated-six",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([1, 1]),
                Some([6, 4]),
                Some(6),
                None,
                None,
            ),
        ),
        (
            // AC4: a NON-ZERO ActiveArea origin — the shape no decodable
            // corpus file carries.
            "nonzero-active-area-origin",
            develop_fixture(
                8,
                6,
                Some([2, 3, 6, 8]),
                Some([1, 1]),
                Some([3, 2]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            "absent-geometry-tags",
            develop_fixture(8, 6, None, None, None, None, None, None),
        ),
        (
            // AC6: DefaultCropSize larger than ActiveArea.
            "crop-exceeds-active-area",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([0, 0]),
                Some([100, 100]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: crop origin leaves no room for its size.
            "crop-origin-out-of-plane",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([50, 50]),
                Some([2, 2]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: zero-width crop.
            "zero-size-crop",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([0, 0]),
                Some([0, 4]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: inverted ActiveArea (bottom < top).
            "inverted-active-area",
            develop_fixture(8, 6, Some([5, 0, 2, 4]), None, None, Some(1), None, None),
        ),
        (
            // AC6: Orientation outside 1..=8.
            "orientation-out-of-range",
            develop_fixture(8, 6, None, None, None, Some(9), None, None),
        ),
        (
            // AC6: BlackLevel >= WhiteLevel.
            "black-level-at-white-level",
            develop_fixture(8, 6, None, None, None, None, Some(100), Some(100)),
        ),
    ]
}

fn write_seeds(subdir: &str, seeds: Vec<(&'static str, Vec<u8>)>) -> std::io::Result<usize> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("seeds")
        .join(subdir);
    std::fs::create_dir_all(&dir)?;

    let mut written = 0usize;
    for (name, bytes) in seeds {
        let path = dir.join(format!("{name}.tiff"));
        std::fs::write(&path, &bytes)?;
        println!("{:>6} bytes  {}", bytes.len(), path.display());
        written += 1;
    }
    println!("fuzz-seeds: {written} seed(s) in {}", dir.display());
    Ok(written)
}

fn main() {
    if let Err(e) = write_seeds("ifd", tiff::all()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("plane", plane_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("develop", develop_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
}
