//! `SPEC-053` — the Compression-7 decoded-frame shape binding (`DEC-047`), the
//! fix for pre-0.2.0 code-review finding #2.
//!
//! Before this guard, `unpack_jpeg` sized each strip's row band to
//! `rows_in_strip × ImageWidth` and relied on `decode_into`'s
//! `out.len() == width × height` check to pin the frame's dimensions. That is an
//! AREA check: a SOF-3 frame that is an equal-area transpose of the band passed
//! it, was written row-major with the wrong stride, and returned `Ok` with a
//! silently garbled plane. Separately, `parse_frame_header` discarded each
//! component's `Hi/Vi` byte, so a subsampled frame decoded as `H = V = 1`.
//! `DEC-047` binds the frame's effective shape (`samples_per_line × components`
//! by `lines`) to the strip before decoding, and rejects `Hi/Vi != 0x11`.
//!
//! Every fixture is hand-built tier A (`tests/support/tiff.rs`); the one tier-B
//! check (`corpus_comp7_files_still_bit_exact`) skips loud, naming the file,
//! when the corpus is absent. The two red-proofs replay mutated copies of the
//! crate with each guard disabled and show the corresponding case go red —
//! control run first (`DEC-009`).

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
use irradiance::Error;
use tiff::Order;

/// Deterministic samples below `2^precision` for a `count`-sample plane.
fn samples(count: usize, precision: u8) -> Vec<u16> {
    let modulo = 1u32 << precision;
    let mut state = 0x053b_u32 ^ u32::from(precision);
    (0..count)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 9) % modulo) as u16
        })
        .collect()
}

/// Parse `data` and run the public `unpack_into` into a `width × height` plane.
fn unpack_synthetic(data: &[u8], width: u32, height: u32) -> Result<Vec<u16>, Error> {
    let container = Container::parse(data).expect("the synthetic container parses");
    let sensor = container.sensor().expect("IFD0 is the sensor plane");
    let mut dst = vec![0u16; (width * height) as usize];
    unpack_into(&sensor, container.byte_order(), data, &mut dst).map(|()| dst)
}

/// A Compression-7 DNG declaring `width × height` whose single strip is an
/// `nf`-component SOF-3 frame of `frame_plane_cols × frame_rows` plane samples
/// (so `X = frame_plane_cols / nf`, `Y = frame_rows`), every component
/// declaring sampling byte `hv`.
fn comp7_with_frame(
    order: Order,
    (width, height): (u32, u32),
    (frame_plane_cols, frame_rows): (usize, usize),
    nf: u8,
    hv: u8,
) -> Vec<u8> {
    let precision = 12u8;
    let plane = samples(frame_plane_cols * frame_rows, precision);
    let strip =
        tiff::lj92_strip_components(&plane, frame_plane_cols, frame_rows, precision, nf, hv);
    tiff::compression7_dng_with_strip(order, width, height, precision, &strip)
}

// The transpose fixture shared with the shape red-proof: the IFD declares
// 6 × 4, the single-component frame is 4 wide × 6 lines — the same 24 samples.
const DECLARED: (u32, u32) = (6, 4);
const TRANSPOSED: (usize, usize) = (4, 6);

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — equal-area shape mismatch rejected
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn equal_area_transposed_frame_is_rejected() {
    let (w, h) = DECLARED;
    let (fc, fr) = TRANSPOSED;
    assert_eq!(
        (w * h) as usize,
        fc * fr,
        "the transpose must be EQUAL AREA, or the old length check would already catch it"
    );
    assert_ne!(w as usize, fc, "and it must actually be a different shape");

    for order in [Order::Little, Order::Big] {
        let data = comp7_with_frame(order, DECLARED, TRANSPOSED, 1, 0x11);
        match unpack_synthetic(&data, w, h) {
            Err(Error::LjpegPlaneShapeMismatch {
                expected_width,
                expected_height,
                decoded_width,
                decoded_height,
            }) => {
                assert_eq!((expected_width, expected_height), (w, h), "{order:?}");
                assert_eq!(
                    (decoded_width, decoded_height),
                    (fc as u32, fr as u32),
                    "{order:?}"
                );
            }
            other => panic!("{order:?}: expected Err(LjpegPlaneShapeMismatch), got {other:?}"),
        }
    }
}

/// A frame whose width matches but whose line count does not (unequal area)
/// is also reported as a shape mismatch, before `decode_into`'s length check.
#[test]
fn line_count_mismatch_is_a_shape_mismatch() {
    let data = comp7_with_frame(Order::Little, (6, 4), (6, 3), 1, 0x11);
    let err = unpack_synthetic(&data, 6, 4).expect_err("3 lines into a 4-row strip");
    assert!(
        matches!(
            err,
            Error::LjpegPlaneShapeMismatch {
                expected_width: 6,
                expected_height: 4,
                decoded_width: 6,
                decoded_height: 3,
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn shape_mismatch_display_names_both_shapes() {
    let msg = Error::LjpegPlaneShapeMismatch {
        expected_width: 6,
        expected_height: 4,
        decoded_width: 4,
        decoded_height: 6,
    }
    .to_string();
    assert!(msg.contains("4x6"), "{msg}");
    assert!(msg.contains("6x4"), "{msg}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 + AC3 — both component counts; correct shapes unaffected
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn matching_shape_still_decodes() {
    let precision = 12u8;
    let (w, h) = (6u32, 4u32);
    let plane = samples((w * h) as usize, precision);

    // The generalised helper is byte-identical to the single-component one for
    // `nf = 1, hv = 0x11`, so this file's Nf=1 fixtures are today's fixtures.
    assert_eq!(
        tiff::lj92_strip(&plane, w as usize, h as usize, precision),
        tiff::lj92_strip_components(&plane, w as usize, h as usize, precision, 1, 0x11)
    );

    for order in [Order::Little, Order::Big] {
        for nf in [1u8, 2] {
            let strip =
                tiff::lj92_strip_components(&plane, w as usize, h as usize, precision, nf, 0x11);
            let data = tiff::compression7_dng_with_strip(order, w, h, precision, &strip);
            let got = unpack_synthetic(&data, w, h)
                .unwrap_or_else(|e| panic!("{order:?} Nf={nf}: a matching shape must decode: {e}"));
            assert_eq!(
                got, plane,
                "{order:?} Nf={nf}: the plane must round-trip exactly"
            );
        }
    }
}

#[test]
fn nf2_effective_width_is_twice_samples_per_line() {
    let (w, h) = (6u32, 4u32);

    // Passes: X = 3, Nf = 2, so 2·X = 6 = ImageWidth. A guard comparing bare
    // `samples_per_line` (3) to the width would wrongly reject this.
    let ok = comp7_with_frame(Order::Little, (w, h), (6, 4), 2, 0x11);
    unpack_synthetic(&ok, w, h).expect("2 × samples_per_line == ImageWidth must decode");

    // Rejected: X = 2, Nf = 2, 6 lines — effective 4 × 6, equal area with the
    // declared 6 × 4.
    let bad = comp7_with_frame(Order::Little, (w, h), (4, 6), 2, 0x11);
    let err = unpack_synthetic(&bad, w, h).expect_err("an Nf=2 transpose must be rejected");
    assert!(
        matches!(
            err,
            Error::LjpegPlaneShapeMismatch {
                expected_width: 6,
                expected_height: 4,
                decoded_width: 4,
                decoded_height: 6,
            }
        ),
        "got {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — H/V != 1 rejected, on both component counts
// ─────────────────────────────────────────────────────────────────────────────

/// The sampling fixture shared with the sampling red-proof: a correctly-shaped
/// single-component frame whose component claims `H = V = 2`.
const SUBSAMPLED_HV: u8 = 0x22;

#[test]
fn subsampled_component_is_rejected() {
    let (w, h) = (6u32, 4u32);
    for nf in [1u8, 2] {
        for hv in [SUBSAMPLED_HV, 0x21, 0x12, 0x00] {
            let data = comp7_with_frame(Order::Little, (w, h), (6, 4), nf, hv);
            let err = unpack_synthetic(&data, w, h)
                .expect_err("a non-0x11 sampling byte must not decode");
            assert!(
                matches!(err, Error::LjpegUnsupportedSampling { factor } if factor == hv),
                "Nf={nf} hv={hv:#04x}: got {err:?}"
            );
        }
    }

    // Only the SECOND component subsampled: every component spec is checked,
    // not just the first.
    let precision = 12u8;
    let plane = samples(24, precision);
    let mut strip = tiff::lj92_strip_components(&plane, 6, 4, precision, 2, 0x11);
    // SOI(2) then SOF3: marker(2) Lf(2) P(1) Y(2) X(2) Nf(1) = offset 12 for
    // comp0's (Ci, Hi/Vi, Tq); comp1's Hi/Vi is at 12 + 3 + 1.
    assert_eq!(&strip[2..4], &[0xFF, 0xC3], "SOF3 where expected");
    assert_eq!(strip[16], 0x11, "comp1 Hi/Vi where expected");
    strip[16] = 0x21;
    let data = tiff::compression7_dng_with_strip(Order::Little, w, h, precision, &strip);
    let err = unpack_synthetic(&data, w, h).expect_err("comp1 subsampled");
    assert!(
        matches!(err, Error::LjpegUnsupportedSampling { factor: 0x21 }),
        "got {err:?}"
    );
}

#[test]
fn sampling_display_names_the_factor() {
    let msg = Error::LjpegUnsupportedSampling { factor: 0x22 }.to_string();
    assert!(msg.contains("0x22"), "{msg}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 (tier B) — the held Compression-7 files still decode bit-exact
// ─────────────────────────────────────────────────────────────────────────────

const CORPUS_COMP7: &[&str] = &[
    "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
    "PENTAX-K3III-MONO/K3III.DNG",
];

#[test]
fn corpus_comp7_files_still_bit_exact() {
    let root = CorpusRoot::resolve();
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    for manifest_path in CORPUS_COMP7 {
        let entry = manifest
            .get(manifest_path)
            .unwrap_or_else(|| panic!("{manifest_path} must be in the manifest"));
        let Some(path) = entry.require(&root) else {
            eprintln!(
                "SKIP corpus_comp7_files_still_bit_exact for {manifest_path} — not present under \
                 {} ({})",
                root.path.display(),
                root.origin()
            );
            continue;
        };
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let container =
            Container::parse(&bytes).unwrap_or_else(|e| panic!("{manifest_path}: parse: {e}"));
        let sensor = container
            .sensor()
            .unwrap_or_else(|e| panic!("{manifest_path}: sensor: {e}"));
        let mut plane = vec![0u16; sensor.width as usize * sensor.height as usize];
        unpack_into(&sensor, container.byte_order(), &bytes, &mut plane).unwrap_or_else(|e| {
            panic!("{manifest_path}: the shape/sampling guards rejected a real file: {e}")
        });
        // `docs/oracle-contract.md`: MD5 of the u16 plane, native little-endian.
        let le_bytes: Vec<u8> = plane.iter().flat_map(|s| s.to_le_bytes()).collect();
        let digest = md5::to_hex(&md5::hash(&le_bytes));
        assert_eq!(
            digest, entry.oracle.raw_checksum,
            "{manifest_path}: plane MD5 drifted from the pinned dnglab checksum"
        );
        eprintln!(
            "{manifest_path}: {}x{} decodes Ok, plane MD5 {digest} == manifest raw_checksum",
            sensor.width, sensor.height
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the two red-proofs (oracle-must-be-shown-red, DEC-009)
// ─────────────────────────────────────────────────────────────────────────────
//
// A copy of the crate is built in a temp dir with a small probe binary that
// parses a fixture DNG, runs the public `unpack_into`, and prints `ok <md5>`
// or `err <message>`. The CONTROL copy is unmutated and must reproduce the
// honest tree's rejection; each MUTANT copy has one guard's condition forced
// false and must accept its fixture. Mirrors `tests/ifd_resource_guard.rs`'s
// `stage_probe_crate` (copied, not shared, as the spec asks).

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-ljpeg-shape-{label}-{}-{nanos}",
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

/// One guard to disable: the source file it lives in and its exact condition.
/// If the condition moves, the red-proof fails loudly rather than silently
/// mutating nothing.
struct Guard {
    file: &'static str,
    needle: &'static str,
}

/// `DEC-047` rule 1, in `src/plane.rs::check_strip_shape` (called by
/// `unpack_jpeg` before each strip is decoded).
const SHAPE_GUARD: Guard = Guard {
    file: "src/plane.rs",
    needle: "if decoded_width != expected_width || decoded_height != expected_height {",
};

/// `DEC-047` rule 2, in `src/ljpeg.rs::parse_frame_header`.
const SAMPLING_GUARD: Guard = Guard {
    file: "src/ljpeg.rs",
    needle: "if hv != SAMPLING_H1_V1 {",
};

/// Disable `guard` in the staged copy: the condition is kept but can never be
/// true, so the guard is "removed" without touching anything else.
fn inject_guard_removed(crate_dir: &Path, guard: &Guard) {
    let path = crate_dir.join(guard.file);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let occurrences = src.matches(guard.needle).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one guard condition {:?} in {}; found {occurrences} — the guard moved, \
         update the needle",
        guard.needle, guard.file
    );
    let disabled = guard.needle.replacen("if ", "if false && (", 1).replacen(
        " {",
        ") { // RED-PROOF INJECTION -- tests/ljpeg_shape_binding.rs",
        1,
    );
    let mutated = src.replacen(guard.needle, &disabled, 1);
    assert_ne!(mutated, src, "the injection must change the source");
    std::fs::write(&path, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", path.display()));
}

const PROBE_MAIN: &str = r#"
fn main() {
    let path = std::env::args().nth(1).expect("fixture path");
    let data = std::fs::read(&path).expect("read fixture");
    let container = irradiance::ifd::Container::parse(&data).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor plane");
    let mut dst = vec![0u16; sensor.width as usize * sensor.height as usize];
    match irradiance::plane::unpack_into(&sensor, container.byte_order(), &data, &mut dst) {
        Ok(()) => {
            let samples: Vec<String> = dst.iter().map(|s| s.to_string()).collect();
            println!("ok {}", samples.join(","));
        }
        Err(e) => println!("err {e}"),
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

    std::fs::write(dest.join("src/bin/ljpeg_shape_probe.rs"), PROBE_MAIN)
        .expect("write probe binary source");
    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml.push_str(
        "\n[[bin]]\nname = \"ljpeg_shape_probe\"\npath = \"src/bin/ljpeg_shape_probe.rs\"\n",
    );
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

/// Build the staged crate and run the probe on `fixture`, returning its line.
fn build_and_run_probe(dir: &Path, fixture: &Path) -> String {
    let build = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "ljpeg_shape_probe",
            "--quiet",
        ])
        // Isolate the staged crate's build in its own `target/` (SPEC-056): an
        // inherited CARGO_TARGET_DIR would put the probe elsewhere, and would
        // build a mutated staged crate into the caller's shared target dir.
        .env_remove("CARGO_TARGET_DIR")
        .current_dir(dir)
        .output()
        .expect("spawn cargo build");
    assert!(
        build.status.success(),
        "cargo build --release failed in {}:\n{}",
        dir.display(),
        String::from_utf8_lossy(&build.stderr)
    );
    let bin = dir.join("target/release/ljpeg_shape_probe");
    let run = std::process::Command::new(&bin)
        .arg(fixture)
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "ljpeg_shape_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .to_string()
}

/// Control then mutant for one guard. `honest_err_prefix` is the start of the
/// honest tree's rejection message; the mutant must instead decode `Ok`.
fn red_proof(label: &str, data: &[u8], guard: &Guard, honest_err_prefix: &str) {
    let fixture_dir = TempDir::new(&format!("{label}-fixture"));
    let fixture = fixture_dir.0.join(format!("{label}.dng"));
    std::fs::write(&fixture, data).expect("write fixture");

    // CONTROL first (DEC-009): the unmutated copy-and-rebuild apparatus must
    // reproduce the honest rejection, so a red below is attributable to the
    // injection and not to a broken copy or build.
    let control_dir = TempDir::new(&format!("{label}-control"));
    stage_probe_crate(&control_dir.0, None);
    let control = build_and_run_probe(&control_dir.0, &fixture);
    assert!(
        control.starts_with(honest_err_prefix),
        "{label}: the UNMUTATED copy must reject the fixture with {honest_err_prefix:?}; got \
         {control:?}"
    );

    // MUTANT: the guard disabled. The corresponding test's assertion — "returns
    // the typed error" — must now FAIL: the fixture decodes to a garbled Ok.
    let mutant_dir = TempDir::new(&format!("{label}-mutant"));
    stage_probe_crate(&mutant_dir.0, Some(guard));
    let mutant = build_and_run_probe(&mutant_dir.0, &fixture);
    assert_ne!(
        mutant, control,
        "{label}: disabling the guard did NOT change the output — the injection is a semantic \
         no-op and this red-proof has caught NOTHING"
    );
    assert!(
        mutant.starts_with("ok "),
        "{label}: with the guard disabled the fixture must decode Ok (the silent-garble red); \
         got {mutant:?}"
    );

    eprintln!("RED-PROOF ({label}): control={control:?} mutant={mutant:?}");
}

#[test]
fn shape_red_proof() {
    let (w, h) = DECLARED;
    let data = comp7_with_frame(Order::Little, DECLARED, TRANSPOSED, 1, 0x11);

    // The honest tree, in-process: the transpose test's own expectation.
    assert!(
        matches!(
            unpack_synthetic(&data, w, h),
            Err(Error::LjpegPlaneShapeMismatch { .. })
        ),
        "the honest tree must reject the transposed frame, or the red below proves nothing"
    );

    red_proof(
        "shape",
        &data,
        &SHAPE_GUARD,
        "err lossless JPEG: strip frame decodes to 4x6",
    );
}

#[test]
fn sampling_red_proof() {
    let (w, h) = DECLARED;
    let data = comp7_with_frame(Order::Little, DECLARED, (6, 4), 1, SUBSAMPLED_HV);

    assert!(
        matches!(
            unpack_synthetic(&data, w, h),
            Err(Error::LjpegUnsupportedSampling {
                factor: SUBSAMPLED_HV
            })
        ),
        "the honest tree must reject the subsampled frame, or the red below proves nothing"
    );

    red_proof(
        "sampling",
        &data,
        &SAMPLING_GUARD,
        "err lossless JPEG: component sampling factors 0x22",
    );
}
