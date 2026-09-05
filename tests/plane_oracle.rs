//! `SPEC-013` — the bit-exact plane oracle against `dnglab analyze
//! --raw-checksum`, and its red-proof. `DEC-017` records the red-proof's
//! mechanism.
//!
//! `SPEC-012`'s unpacker already matches `dnglab`'s checksum on all four
//! decodable corpus files, verified twice outside this repo
//! (`docs/oracle-contract.md`). This file makes that a fact the repo asserts
//! on every run, in three tiers:
//!
//! - **Tier A** (`md5_matches_the_rfc_1321_test_vectors`,
//!   `a_mismatch_names_the_first_differing_sample`,
//!   `dnglab_raw_pixel_pgm_parses`, `compressed_files_are_skipped_by_name`,
//!   `hand_built_fixtures_plane_matches_its_known_md5`) run everywhere, no
//!   corpus, no tools — `AC5`, and the only half CI ever sees (`DEC-003`).
//! - **Tier B** (`plane_md5_matches_the_pinned_raw_checksum`) needs the real
//!   corpus and skips loudly, per-entry, when absent.
//! - **The red-proof** (`an_injected_unpacker_fault_turns_the_oracle_red`,
//!   `the_honest_tree_is_the_negative_control`) needs the real corpus too, but
//!   is its own category: it does not merely READ the unpacker, it rebuilds a
//!   MUTATED copy of it and proves the mutation changes the plane MD5. See
//!   "The red-proof" below for why that needs a real, separate compilation
//!   rather than an in-process trick.
//!
//! ## ⚠⚠ The one sentence this file exists for
//!
//! A red-proof must assert that the injected fault changed the file, AND
//! compiled, AND **changed the OUTPUT** — control digest ≠ mutant digest —
//! **before** concluding anything about what it caught. `SPEC-013`'s design
//! probe injected `remaining.min(bits_left)` → `remaining.min(bits_left).max(1)`
//! into `BitReader::read`: the file changed, it compiled, and the plane
//! digest came back **byte-identical** to the honest one — `.max(1)` differs
//! only when the min is zero, which never happens in this module, so it was a
//! semantic no-op that satisfied every check this repo's rules require. The
//! fault this file injects instead (`inject_chunk_extraction_fault`, below)
//! is chosen so it does NOT have that property, and
//! `an_injected_unpacker_fault_turns_the_oracle_red` asserts
//! `mutant_digest != honest_digest` directly, every run, rather than trusting
//! that any given mutation must be real.
//!
//! ⚠ **A first attempt at this fault was measured and rejected.** Starting
//! `BitReader`'s cursor at bit 1 instead of bit 0 changed the file and
//! compiled, but the strip is packed with **zero slack** (the layer-0
//! invariant: `width * height * bits == StripByteCounts * 8`, exactly), so a
//! CONSTANT shift to the total bit budget consumes one bit more than the
//! buffer holds by the very last sample — measured to produce
//! `Error::Truncated` at the final read, not a wrong digest, on this file.
//! The fault below instead swaps which end of a partial byte's remaining
//! bits gets kept as the sample value — it changes VALUES, never the total
//! bits consumed, so it cannot hit that boundary.
//!
//! ## The red-proof's mechanism (`DEC-017`)
//!
//! Rust has no way to swap `unpack_into`'s behaviour at runtime — proving a
//! fault in it requires a real, separate compilation, the same reason
//! `scripts/lint-red-proof.sh` (`DEC-006`/`DEC-009`) copies the crate to a
//! temp dir rather than mutating a struct in memory. This file does the same
//! thing for a *decode* fault instead of a *lint* fault: copy `Cargo.toml`,
//! `Cargo.lock` and `src/` to a temp dir, inject one line into the copy's
//! `src/plane.rs`, drop in a small synthesized probe binary that runs the
//! copy's own `unpack_into` against a real corpus file and prints the plane's
//! MD5, `cargo build --release` it (release, not debug — a 47-megapixel
//! decode in debug is the timeout the design probe hit twice), and run it.
//! The working tree's `src/plane.rs` is never touched.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/tiff.rs"]
mod tiff;

use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::Container;
use irradiance::plane::unpack_into;

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — every corpus file, accounted for by name
// ─────────────────────────────────────────────────────────────────────────────

/// The four files this oracle actually decodes and hashes.
const DECODABLE: [&str; 4] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "LEICA-Q2-MONO/L1026016.DNG",
    "LEICA-Q2-MONO/L1026192.DNG",
    "LEICA-M-MONOCHROM/L1000622.DNG",
];

/// The reference frame for the red-proof: the same file every other spec in
/// this repo treats as "the" Q2M frame, 14-bit — the sub-byte `BitReader`
/// path the injected fault targets — and the smallest of the three Q2M files.
const REFERENCE_FILE: &str = "LEICA-Q2-MONO/L1021223.DNG";

/// The three corpus files this oracle does NOT decode, and why — named here
/// rather than left as "whatever `DECODABLE` doesn't mention", so an entry
/// that falls through neither list fails `compressed_files_are_skipped_by_name`
/// instead of silently vanishing (AC2: "skipped by name with a stated reason,
/// not silently").
const SKIPPED_COMPRESSED: [(&str, &str); 3] = [
    (
        "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
        "Compression 7 (JPEG) - lossless JPEG SOF-3 decode is out of scope for PROJ-001",
    ),
    (
        "PENTAX-K3III-MONO/K3III.DNG",
        "Compression 7 (JPEG) - same SOF-3 gap, a different camera",
    ),
    (
        "PENTAX-K3III-MONO/K3III.PEF",
        "Compression 65535 (vendor PEF) - a different container entirely",
    ),
];

#[test]
fn compressed_files_are_skipped_by_name() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");

    assert_eq!(
        DECODABLE.len() + SKIPPED_COMPRESSED.len(),
        manifest.files.len(),
        "DECODABLE ({}) + SKIPPED_COMPRESSED ({}) must account for every [[file]] in the \
         manifest ({}) -- an entry that falls through both lists is a silent skip",
        DECODABLE.len(),
        SKIPPED_COMPRESSED.len(),
        manifest.files.len()
    );

    for path in DECODABLE {
        assert!(
            manifest.get(path).is_some(),
            "{path} is in DECODABLE but not in the manifest"
        );
        assert!(
            !SKIPPED_COMPRESSED.iter().any(|(p, _)| *p == path),
            "{path} is in both DECODABLE and SKIPPED_COMPRESSED"
        );
    }
    for (path, reason) in SKIPPED_COMPRESSED {
        assert!(
            manifest.get(path).is_some(),
            "{path} is in SKIPPED_COMPRESSED but not in the manifest"
        );
        assert!(!reason.trim().is_empty(), "{path} has an empty skip reason");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 -- MD5 from RFC 1321, proven against its own published test vectors
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md5_matches_the_rfc_1321_test_vectors() {
    // RFC 1321 Appendix A.5 "Test suite" -- all seven vectors, verbatim.
    let vectors: [(&str, &str); 7] = [
        ("", "d41d8cd98f00b204e9800998ecf8427e"),
        ("a", "0cc175b9c0f1b6a831c399e269772661"),
        ("abc", "900150983cd24fb0d6963f7d28e17f72"),
        ("message digest", "f96b697d7cb7938d525a2f31aaf161d0"),
        (
            "abcdefghijklmnopqrstuvwxyz",
            "c3fcd3d76192e4007dfb496cca67e13b",
        ),
        (
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            "d174ab98d277d9f5a5611c2c9f419d9f",
        ),
        (
            "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
            "57edf4a22be3c955ac49da2e2107b67a",
        ),
    ];

    for (input, expected) in vectors {
        let actual = md5::to_hex(&md5::hash(input.as_bytes()));
        assert_eq!(actual, expected, "MD5({input:?})");
    }
}

#[test]
fn md5_streaming_matches_one_shot() {
    // Mirrors `sha256_streaming_matches_one_shot`'s discipline: a hasher fed
    // in two pieces must equal the same input fed in one, split across a
    // block boundary (`BLOCK` = 64) so the buffered-tail path is exercised.
    let data: Vec<u8> = (0u32..5000).map(|i| (i % 256) as u8).collect();
    let one_shot = md5::hash(&data);

    let mut streamed = md5::Md5::new();
    streamed.update(&data[..4999]);
    streamed.update(&data[4999..]);
    let streamed = streamed.finish();

    assert_eq!(one_shot, streamed);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 -- a mismatch is locatable: the pure locator, tier A
// ─────────────────────────────────────────────────────────────────────────────

/// Where two planes first disagree, and by how much. MD5 says "different",
/// never "where" (`docs/oracle-contract.md`) -- this is the "where".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SampleMismatch {
    index: usize,
    ours: u16,
    reference: u16,
}

/// The first index at which `ours` and `reference` disagree, or `None` if
/// every sample they share in common agrees. A length mismatch is not
/// reported here -- our decoded plane and dnglab's `--raw-pixel` plane are
/// always the same declared `width x height` whenever both parse at all
/// (`dnglab_reference_plane` checks that separately), so this function's
/// only job is locating a SAMPLE disagreement between two same-shaped planes.
fn locate_first_difference(ours: &[u16], reference: &[u16]) -> Option<SampleMismatch> {
    ours.iter()
        .zip(reference.iter())
        .enumerate()
        .find_map(|(index, (&o, &r))| {
            (o != r).then_some(SampleMismatch {
                index,
                ours: o,
                reference: r,
            })
        })
}

#[test]
fn a_mismatch_names_the_first_differing_sample() {
    let identical = [1u16, 2, 3, 4, 5];
    assert_eq!(locate_first_difference(&identical, &identical), None);

    let a = [1u16, 2, 3, 4, 5];
    let b = [1u16, 2, 30, 4, 5];
    assert_eq!(
        locate_first_difference(&a, &b),
        Some(SampleMismatch {
            index: 2,
            ours: 3,
            reference: 30
        })
    );

    // The very first sample -- no off-by-one hiding a difference at index 0.
    let c = [9u16, 2, 3];
    let d = [1u16, 2, 3];
    assert_eq!(
        locate_first_difference(&c, &d),
        Some(SampleMismatch {
            index: 0,
            ours: 9,
            reference: 1
        })
    );

    // Only the FIRST difference is reported, even with several later ones.
    let e = [1u16, 2, 3, 4];
    let f = [1u16, 20, 3, 40];
    assert_eq!(
        locate_first_difference(&e, &f),
        Some(SampleMismatch {
            index: 1,
            ours: 2,
            reference: 20
        })
    );

    let empty: [u16; 0] = [];
    assert_eq!(locate_first_difference(&empty, &empty), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 -- the reference route: parsing `dnglab analyze --raw-pixel`'s PGM
// ─────────────────────────────────────────────────────────────────────────────

/// Parse dnglab's `--raw-pixel` output: a `P5 <w> <h> <maxval>\n` header
/// followed by `w * h` **big-endian** `u16` samples (the PNM spec mandates
/// big-endian for `maxval > 255`; `docs/oracle-contract.md`'s endianness
/// proof is why this reads big-endian while `raw_checksum` is native LE --
/// they are the same plane, two different serializations of it).
fn parse_raw_pixel_pgm(stdout: &[u8]) -> Result<(u32, u32, Vec<u16>), String> {
    let header_end = stdout
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| "no newline in output -- not a PGM header".to_string())?;
    let header = std::str::from_utf8(&stdout[..header_end])
        .map_err(|e| format!("PGM header is not UTF-8: {e}"))?;

    let mut fields = header.split_whitespace();
    let magic = fields
        .next()
        .ok_or_else(|| "empty PGM header".to_string())?;
    if magic != "P5" {
        return Err(format!("expected P5 magic, got {magic:?}"));
    }
    let width: u32 = fields
        .next()
        .ok_or_else(|| "PGM header missing width".to_string())?
        .parse()
        .map_err(|e| format!("PGM width does not parse: {e}"))?;
    let height: u32 = fields
        .next()
        .ok_or_else(|| "PGM header missing height".to_string())?
        .parse()
        .map_err(|e| format!("PGM height does not parse: {e}"))?;
    let maxval: u32 = fields
        .next()
        .ok_or_else(|| "PGM header missing maxval".to_string())?
        .parse()
        .map_err(|e| format!("PGM maxval does not parse: {e}"))?;
    if maxval <= 255 {
        return Err(format!(
            "PGM maxval {maxval} <= 255 -- payload would be 8-bit, not the 16-bit plane this \
             oracle expects"
        ));
    }

    let payload = &stdout[header_end + 1..];
    let pixel_count = usize::try_from(u64::from(width).saturating_mul(u64::from(height)))
        .map_err(|_| "width * height overflows this host's usize".to_string())?;
    let expected_bytes = pixel_count
        .checked_mul(2)
        .ok_or_else(|| "pixel_count * 2 overflows this host's usize".to_string())?;
    if payload.len() != expected_bytes {
        return Err(format!(
            "PGM payload is {} bytes, expected {expected_bytes} ({width}x{height}x2)",
            payload.len()
        ));
    }

    let (chunks, _remainder) = payload.as_chunks::<2>();
    let samples = chunks.iter().map(|c| u16::from_be_bytes(*c)).collect();
    Ok((width, height, samples))
}

#[test]
fn dnglab_raw_pixel_pgm_parses() {
    // The exact endianness proof from `docs/oracle-contract.md`: file bytes
    // `02 EA` read big-endian are 746 -- `L1021223.DNG`'s real first sample.
    let mut stdout = b"P5 2 1 65535\n".to_vec();
    stdout.extend_from_slice(&[0x02, 0xEA, 0x00, 0x64]);

    let (width, height, samples) = parse_raw_pixel_pgm(&stdout).expect("parses");
    assert_eq!((width, height), (2, 1));
    assert_eq!(samples, vec![746, 100]);
}

#[test]
fn dnglab_raw_pixel_pgm_rejects_malformed_input() {
    assert!(
        parse_raw_pixel_pgm(b"P6 2 1 65535\nxxxx").is_err(),
        "wrong magic"
    );
    assert!(
        parse_raw_pixel_pgm(b"not a header at all").is_err(),
        "no newline"
    );
    assert!(
        parse_raw_pixel_pgm(b"P5 2 1 255\n\x01\x02").is_err(),
        "maxval <= 255 must be rejected, not silently treated as 16-bit"
    );
    assert!(
        parse_raw_pixel_pgm(b"P5 2 1 65535\n\x00\x01").is_err(),
        "payload short by one sample must be rejected, not silently truncated"
    );
}

/// Fetch dnglab's own plane for `file_path` and check its declared shape
/// against `(width, height)`. Best-effort: used only as a diagnostic when
/// `plane_md5_matches_the_pinned_raw_checksum` finds a real mismatch (which
/// this oracle has never yet observed) -- if dnglab is not on `PATH` or its
/// output does not parse, the caller still fails the test on the MD5
/// mismatch itself, just without a locator attached.
fn dnglab_reference_plane(file_path: &Path, width: u32, height: u32) -> Result<Vec<u16>, String> {
    let output = std::process::Command::new("dnglab")
        .args(["analyze", "--raw-pixel"])
        .arg(file_path)
        .output()
        .map_err(|e| format!("dnglab: could not run ({e}) -- is it on PATH?"))?;
    if !output.status.success() {
        return Err(format!(
            "dnglab exited {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let (pgm_width, pgm_height, samples) = parse_raw_pixel_pgm(&output.stdout)?;
    if (pgm_width, pgm_height) != (width, height) {
        return Err(format!(
            "dnglab's plane is {pgm_width}x{pgm_height}, ours is {width}x{height}"
        ));
    }
    Ok(samples)
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 -- a hand-built fixture whose plane AND digest are both known, tier A
// ─────────────────────────────────────────────────────────────────────────────

/// `L1021223.DNG`'s real strip head (`DEC-008`/`SPEC-012`'s
/// `## Implementation Context`, also `tests/plane_unpack.rs::Q2M_STRIP_HEAD`)
/// -- 14-bit MSB-first, decoding to `[746, 725, 711, 752, 646, 705, 772,
/// 686]`. Reused here (not re-derived) because it is already independently
/// measured two ways: by hand, and against `dnglab --raw-pixel`'s own plane.
const FIXTURE_STRIP: [u8; 14] = [
    0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
];

/// The MD5 of `[746, 725, 711, 752, 646, 705, 772, 686]` as eight native-LE
/// `u16`s -- computed independently with Python's `hashlib.md5` at design
/// time (not with this file's own MD5, which would make the check circular),
/// over the exact byte sequence
/// `ea 02 d5 02 c7 02 f0 02 86 02 c1 02 04 03 ae 02`.
const FIXTURE_PLANE_MD5: &str = "d1d83299c631541fac68da1051b19a23";

/// A minimal 4x2, 14-bit sensor-only TIFF (IFD0 IS the sensor plane) with
/// [`FIXTURE_STRIP`] planted at a chosen offset -- the same construction
/// `tests/plane_unpack.rs::Fixture` uses, trimmed to the one shape AC5 needs.
fn hand_built_fixture() -> Vec<u8> {
    const STRIP_OFFSET: u32 = 512;
    let entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, 4),
        tiff::long(tiff::IMAGE_LENGTH, 2),
        tiff::short(tiff::BITS_PER_SAMPLE, 14, tiff::Order::Little),
        tiff::short(tiff::COMPRESSION, 1, tiff::Order::Little),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, tiff::Order::Little),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, tiff::Order::Little),
        tiff::long(tiff::STRIP_OFFSETS, STRIP_OFFSET),
        tiff::long(
            tiff::STRIP_BYTE_COUNTS,
            u32::try_from(FIXTURE_STRIP.len()).expect("14 fits u32"),
        ),
    ];
    let mut data = tiff::tiff(tiff::Order::Little, 8, &[tiff::Ifd::new(8, entries, 0)]);

    let at = STRIP_OFFSET as usize;
    let end = at + FIXTURE_STRIP.len();
    if data.len() < end {
        data.resize(end, 0);
    }
    data[at..end].copy_from_slice(&FIXTURE_STRIP);
    data
}

#[test]
fn hand_built_fixtures_plane_matches_its_known_md5() {
    let data = hand_built_fixture();
    let container = Container::parse(&data).expect("hand-built fixture parses");
    let sensor = container
        .sensor()
        .expect("hand-built fixture has a sensor plane");
    let mut plane = [0u16; 8];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane).expect("unpack");

    assert_eq!(
        plane,
        [746, 725, 711, 752, 646, 705, 772, 686],
        "the hand-built fixture's plane must match its known, independently-measured values"
    );

    let digest = md5::to_hex(&plane_md5(&plane));
    assert_eq!(
        digest, FIXTURE_PLANE_MD5,
        "the hand-built fixture's plane MD5 must match the value computed independently \
         (Python hashlib.md5) at design time"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared: hash a decoded plane the way `dnglab analyze --raw-checksum` does
// ─────────────────────────────────────────────────────────────────────────────

/// `docs/oracle-contract.md`: "MD5 of the uncropped u16 plane, native
/// little-endian". Streamed sample-by-sample rather than serialized into a
/// second ~95 MB buffer first -- `DEC-016` already puts one caller-owned
/// plane buffer on the peak-RSS budget; this does not add a second.
fn plane_md5(plane: &[u16]) -> [u8; 16] {
    let mut hasher = md5::Md5::new();
    for sample in plane {
        hasher.update(&sample.to_le_bytes());
    }
    hasher.finish()
}

/// Read, parse, and unpack one corpus file's plane, then hash it. Shared by
/// the tier-B oracle test and the red-proof's in-process honest side.
fn decode_and_hash(file_path: &Path) -> Result<(u32, u32, Vec<u16>, String), String> {
    let bytes = std::fs::read(file_path).map_err(|e| format!("read: {e}"))?;
    let container = Container::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    let sensor = container.sensor().map_err(|e| format!("sensor: {e}"))?;
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .map_err(|_| "plane too large for this host".to_string())?;
    let mut plane = vec![0u16; pixel_count];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .map_err(|e| format!("unpack: {e}"))?;
    let digest = md5::to_hex(&plane_md5(&plane));
    Ok((sensor.width, sensor.height, plane, digest))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 -- the live oracle: all four decodable files against the pinned digest
// ─────────────────────────────────────────────────────────────────────────────

fn assert_plane_matches(label: &str, file_path: &Path, expected_hex: &str) {
    let (width, height, plane, actual_hex) =
        decode_and_hash(file_path).unwrap_or_else(|e| panic!("{label}: {e}"));

    if actual_hex == expected_hex {
        return;
    }

    // AC3: a mismatch must be locatable, not just "digests differ". This
    // branch has never fired against an honest corpus file -- it exists for
    // SPEC-014, which will debug a 47-megapixel plane against this oracle.
    let locator = match dnglab_reference_plane(file_path, width, height) {
        Ok(reference) => match locate_first_difference(&plane, &reference) {
            Some(m) => format!(
                "first differing sample at index {}: ours={} dnglab={}",
                m.index, m.ours, m.reference
            ),
            None => "dnglab's plane agreed with ours sample-for-sample; the digests still \
                     differ, which should not be possible -- investigate the hasher itself"
                .to_string(),
        },
        Err(e) => format!("could not localize via dnglab: {e}"),
    };

    panic!(
        "{label}: plane MD5 mismatch\n  expected (manifest): {expected_hex}\n  actual:              \
         {actual_hex}\n  locator: {locator}"
    );
}

#[test]
fn plane_md5_matches_the_pinned_raw_checksum() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    for path in DECODABLE {
        let entry = manifest
            .get(path)
            .unwrap_or_else(|| panic!("{path} must be in the manifest"));
        let Some(file_path) = entry.require(&root) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        assert_plane_matches(path, &file_path, &entry.oracle.raw_checksum);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 -- the red-proof, and its negative control
// ─────────────────────────────────────────────────────────────────────────────

/// A directory removed on drop, even if the test panics first -- mirrors
/// `scripts/lint-red-proof.sh`'s `trap cleanup EXIT`.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-plane-oracle-{label}-{}-{nanos}",
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

/// The ONE injected fault this red-proof exists to catch, and the reason it
/// is not the design probe's `.max(1)` or this file's own first attempt (see
/// the module doc comment): `BitReader::read`'s final chunk of a partial byte
/// is meant to be the TOP `take` bits of the `bits_left` bits remaining in
/// that byte (`in_byte.checked_div(pow2(bits_left - take))`). This swaps it
/// for the BOTTOM `take` bits instead (`in_byte.checked_rem(pow2(take))`) --
/// wrong whenever `take < bits_left` (every partial-byte read that does not
/// exhaust the byte), same total bits consumed either way, so it corrupts
/// values throughout the plane without ever touching the buffer's length.
fn inject_chunk_extraction_fault(plane_rs: &Path) {
    let src = std::fs::read_to_string(plane_rs)
        .unwrap_or_else(|e| panic!("read {}: {e}", plane_rs.display()));

    let needle = ".checked_div(pow2(bits_left.saturating_sub(take)))";
    let occurrences = src.matches(needle).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one `{needle}` in src/plane.rs (BitReader::read's chunk \
         extraction); found {occurrences} -- the injection point moved, update this test"
    );

    let mutated = src.replacen(
        needle,
        ".checked_rem(pow2(take)) /* RED-PROOF INJECTION -- tests/plane_oracle.rs, never in the real tree */",
        1,
    );
    std::fs::write(plane_rs, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", plane_rs.display()));
}

/// The probe's `main()`, spliced together with [`MD5_SOURCE`] (the exact
/// content of `tests/support/md5.rs`) into one `.rs` file dropped into the
/// staged copy's `src/bin/`. Runs the copy's OWN `irradiance::plane::unpack_into`
/// -- mutated or not, depending on which copy this is -- against a real file
/// and prints the plane's MD5.
const MD5_SOURCE: &str = include_str!("support/md5.rs");

const PROBE_MAIN: &str = r#"
fn main() {
    let path = std::env::args().nth(1).expect("usage: plane_oracle_probe <file>");
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let container = irradiance::ifd::Container::parse(&data)
        .unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("sensor {path}: {e}"));
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .unwrap_or_else(|_| panic!("plane too large for this host"));
    let mut plane = vec![0u16; pixel_count];
    irradiance::plane::unpack_into(&sensor, container.byte_order(), &data, &mut plane)
        .unwrap_or_else(|e| panic!("unpack {path}: {e}"));

    let mut hasher = md5_probe::Md5::new();
    for sample in &plane {
        hasher.update(&sample.to_le_bytes());
    }
    println!("{}", md5_probe::to_hex(&hasher.finish()));
}
"#;

fn probe_source() -> String {
    format!("mod md5_probe {{\n{MD5_SOURCE}\n}}\n\n{PROBE_MAIN}")
}

/// Copy the crate to `dest`, optionally injecting the fault, and drop in the
/// synthesized probe binary as a second, explicit `[[bin]]` target.
fn stage_probe_crate(dest: &Path, mutate: bool) {
    let repo_root = corpus::crate_root();

    std::fs::create_dir_all(dest.join("src/bin")).expect("mkdir src/bin");
    std::fs::copy(repo_root.join("Cargo.toml"), dest.join("Cargo.toml")).expect("copy Cargo.toml");
    let lock = repo_root.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, dest.join("Cargo.lock")).expect("copy Cargo.lock");
    }
    copy_dir_recursive(&repo_root.join("src"), &dest.join("src"));

    if mutate {
        inject_chunk_extraction_fault(&dest.join("src/plane.rs"));
    }

    std::fs::write(dest.join("src/bin/plane_oracle_probe.rs"), probe_source())
        .expect("write probe binary source");

    // Explicit `[[bin]]` rather than relying on autobins discovery, so this
    // does not silently stop working if `autobins` is ever turned off.
    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml.push_str(
        "\n[[bin]]\nname = \"plane_oracle_probe\"\npath = \"src/bin/plane_oracle_probe.rs\"\n",
    );
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

/// Build the staged crate in **release** mode (debug makes a 47-megapixel
/// decode slow enough to risk a tool-call timeout -- measured: the design
/// probe was killed by exactly this, twice) and run the probe against
/// `file`, returning its printed MD5 hex digest.
fn build_and_run_probe(dir: &Path, file: &Path) -> String {
    let build = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "plane_oracle_probe",
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

    let bin = dir.join("target/release/plane_oracle_probe");
    let run = std::process::Command::new(&bin)
        .arg(file)
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "plane_oracle_probe failed on {}:\n{}",
        file.display(),
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .to_string()
}

#[test]
fn an_injected_unpacker_fault_turns_the_oracle_red() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(REFERENCE_FILE)
        .unwrap_or_else(|| panic!("{REFERENCE_FILE} must be in the manifest"));
    let Some(file_path) = entry.require(&root) else {
        return; // SKIP already announced by CorpusFile::require
    };

    // The honest digest, computed IN-PROCESS via the real, UNMUTATED
    // unpack_into already linked into this test binary -- recomputed here
    // (not merely read from the manifest) so this test does not depend on
    // `plane_md5_matches_the_pinned_raw_checksum` having run first.
    let (_, _, _, honest_digest) =
        decode_and_hash(&file_path).unwrap_or_else(|e| panic!("{REFERENCE_FILE}: {e}"));
    assert_eq!(
        honest_digest, entry.oracle.raw_checksum,
        "the honest tree must match the pinned raw_checksum before this red-proof means anything"
    );

    let dir = TempDir::new("mutant");
    stage_probe_crate(&dir.0, true);
    let mutant_digest = build_and_run_probe(&dir.0, &file_path);

    // The one sentence this spec exists for: assert the OUTPUT changed
    // before concluding anything about what the test caught.
    assert_ne!(
        mutant_digest, honest_digest,
        "the injected chunk-extraction fault did NOT change the plane MD5 -- it is a semantic no-op \
         like the design probe's `.max(1)`, not a real fault. This red-proof has caught \
         NOTHING; do not conclude it works."
    );

    eprintln!(
        "RED-PROOF ({REFERENCE_FILE}): honest={honest_digest} mutant={mutant_digest} -- the \
         injected fault turned the oracle red"
    );
}

#[test]
fn the_honest_tree_is_the_negative_control() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(REFERENCE_FILE)
        .unwrap_or_else(|| panic!("{REFERENCE_FILE} must be in the manifest"));
    let Some(file_path) = entry.require(&root) else {
        return; // SKIP already announced by CorpusFile::require
    };

    let dir = TempDir::new("control");
    stage_probe_crate(&dir.0, false);
    let apparatus_digest = build_and_run_probe(&dir.0, &file_path);

    // If this fails, a red result from the mutation test above is
    // attributable to the copy-and-rebuild apparatus itself, not the
    // injection -- exactly the distinction `lint-red-proof.sh`'s control run
    // exists for (`DEC-009`).
    assert_eq!(
        apparatus_digest, entry.oracle.raw_checksum,
        "the UNMUTATED copy-and-rebuild apparatus must reproduce the pinned digest"
    );
}
