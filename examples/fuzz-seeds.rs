//! Write the fuzz seed corpus for `fuzz/fuzz_targets/ifd.rs` and
//! `fuzz/fuzz_targets/plane.rs`.
//!
//! `SPEC-003`, extended by `SPEC-012`. The `ifd` seeds are the hand-built
//! tier-A fixtures in `tests/support/tiff.rs` — the same list
//! `tests/ifd_reader.rs` asserts against, so a fixture cannot be added to the
//! test lane and forgotten by the fuzz lane. The `plane` seeds are built
//! locally in [`plane_seeds`] from the strip bytes `tests/plane_unpack.rs`
//! also uses — `DEC-008`/`SPEC-012`'s measured `## Implementation Context`,
//! not any corpus file. Everything here is **own work built from TIFF 6.0
//! §2**: tier B is never committed (`DEC-003`), and a truncated 86 MB Leica
//! frame would still be a Leica frame.
//!
//! ```text
//! cargo run --example fuzz-seeds          # rewrite fuzz/seeds/{ifd,plane}/
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
}
