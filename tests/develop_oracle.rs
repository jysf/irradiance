//! `SPEC-015` — the analytic levels and geometry oracle, and its red-proof.
//! `DEC-020` records the property set; `DEC-021` records the red-proof's
//! (asymmetric) mechanism.
//!
//! `SPEC-014` shipped level normalization and the three-stage `ActiveArea` →
//! `DefaultCrop` → `Orientation` geometry, and asserted **its own
//! arithmetic** (`tests/develop.rs`, `src/develop.rs`'s unit tests) —
//! `DEC-004` already names the limit of that: it verifies the arithmetic
//! this project chose, not that the choice matches DNG's intent, and no
//! comparison oracle can see it either (`dnglab --raw-checksum` attaches
//! before any of this; the develop oracle is perceptually blind to a levels
//! error up to +256). This file is the independent check: expectations
//! derived from tag values read from the file and DNG 1.7 Chapter 4's own
//! stated defaults, **never** from `src/develop.rs`'s implementation.
//!
//! ## ⚠ The one idea this file turns on
//!
//! **An oracle that reimplements the transform is a mirror.** `Orientation`'s
//! eight-case table is never written here, in any form. Two techniques make
//! that possible, and both rest on the same fact: `Orientation` only ever
//! *permutes* output positions, and `develop.rs`'s `normalize` (hence the
//! exact real-valued affine map `exact_affine`) is monotonic
//! (`AC4`) — see `tests/support/oracle.rs` and `DEC-020`.
//!
//! - **L1, the permutation property** (`the_developed_histogram_is_the_normalized_crop_windows`,
//!   `AC3`): `histogram(develop_into output)` must equal `histogram(the crop
//!   window in raster order, no orientation)`. A permutation cannot change a
//!   multiset, so this holds for ANY orientation without this file ever
//!   knowing what that orientation DOES.
//! - **L2, the per-pixel bound** (`every_pixel_is_within_half_an_lsb_of_the_exact_affine_map`,
//!   `AC1`/`AC2`): every output pixel is within `< 0.5` of the exact
//!   real-valued affine map, checked via a RANK-preserving merge of two
//!   frequency tables rather than position — monotonicity makes "rank,
//!   counting repeats" and "the true positional pairing" the same pairing,
//!   so this too never needs the orientation table
//!   (`tests/support/oracle.rs::bound_check`; see `DEC-020` for why a bare
//!   distinct-value pairing, without weighting by repeat count, is NOT
//!   equivalent and was measured wrong).
//!
//! Three lanes:
//!
//! - **Tier A** (`normalization_is_strictly_monotonic_and_injective`,
//!   `the_oracle_is_red_on_a_levels_fault`,
//!   `the_oracle_is_red_on_an_orientation_fault`,
//!   `the_orientation_fixture_oracle_control_is_green`,
//!   `rotating_orientation_is_positionally_correct_at_production_scale`,
//!   `flipping_orientation_is_positionally_correct_at_production_scale`) run
//!   everywhere, no corpus, no tools — the only lane CI ever executes
//!   (`DEC-003`). The two red-proofs and their control satisfy `AC6`:
//!   `SPEC-013/FU-1`'s corpus-gated red-proof genuinely works and CI has never
//!   once run it: it needs the corpus. These do not.
//! - **Tier B** (`every_pixel_is_within_half_an_lsb_of_the_exact_affine_map`,
//!   `the_developed_histogram_is_the_normalized_crop_windows`,
//!   `distinct_output_levels_equal_distinct_input_levels`) need the real
//!   corpus and skip loudly, per-entry, when absent.
//!
//! ## `FU-10` — the two production-scale tests
//!
//! `HANDOFF-036` measured that `DEC-020`'s rank/frequency techniques
//! (`bound_check`, `multiset_equal`) are blind to a positional fault when it
//! is size-gated (`crop_width > N` for any `N` no hand-built fixture crosses)
//! — a fault that corrupts 100% of a real 47-megapixel frame's positions
//! passed 150/150 of this repo's tests, because every positional fixture
//! anywhere in the repo has `crop_width <= 3`. CI never runs the corpus at
//! all (`DEC-003`), so no tier-B test can backstop that gap either.
//! `rotating_orientation_is_positionally_correct_at_production_scale` and
//! `flipping_orientation_is_positionally_correct_at_production_scale` are a
//! synthetic, in-test, tier-A fixture at 1024x768 — big enough to cross the
//! `> 1000` gate measured against this oracle — that checks POSITIONS, not
//! rank or frequency, for exactly the reason `DEC-020`'s techniques cannot:
//! only a positional check can see which permutation was applied. This does
//! **not** close the class (a `> 2000` gate still evades a 1024-wide fixture)
//! and does **not** touch `FU-6`'s inherent wrong-permutation blind spot in
//! the rank/frequency techniques themselves — see the residual note at the
//! tests' own definition, and `DEC-020`'s `## Consequences`.
//!
//! No fuzz target: this file adds no parser and no new input surface — it
//! consumes the same already-parsed `Sensor` the `develop` fuzz target
//! (`SPEC-014`) already exercises. `AGENTS.md` §12 bar 2 does not fire.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/oracle.rs"]
mod oracle;

use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{Compression, Container, Sensor};
use irradiance::plane::unpack_into;

/// The three decodable files `SPEC-015`'s design probe measured against
/// (111,529,040 pixels total — `Implementation Context`). Deliberately NOT
/// `tests/plane_oracle.rs`'s four-file `DECODABLE`: `L1026192.DNG` shares
/// `L1021223.DNG`'s levels, geometry and orientation exactly, so it adds no
/// new arithmetic this oracle can observe.
const DECODABLE: [&str; 3] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "LEICA-Q2-MONO/L1026016.DNG",
    "LEICA-M-MONOCHROM/L1000622.DNG",
];

/// A decoded, developed fixture: the parsed `Sensor`, the uncropped raw
/// plane, and `develop_into`'s actual output.
type DecodedFixture = (Sensor, Vec<u16>, Vec<u16>);

/// Decode `path`'s real file, unpack its plane, and run the REAL
/// `develop_into` — `None` with the skip already announced by
/// `CorpusFile::require` when the corpus is absent.
fn decode_and_develop(
    manifest: &Manifest,
    root: &CorpusRoot,
    path: &str,
) -> Option<DecodedFixture> {
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(root)?;
    let data = std::fs::read(&file_path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let container = Container::parse(&data).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let sensor = container
        .sensor()
        .unwrap_or_else(|e| panic!("sensor {path}: {e}"));

    let pixel_count = sensor.width as usize * sensor.height as usize;
    let mut plane = vec![0u16; pixel_count];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane)
        .unwrap_or_else(|e| panic!("unpack {path}: {e}"));

    let (out_width, out_height) =
        output_dimensions(&sensor).unwrap_or_else(|e| panic!("output_dimensions {path}: {e}"));
    let mut output = vec![0u16; out_width as usize * out_height as usize];
    develop_into(&sensor, &plane, &mut output)
        .unwrap_or_else(|e| panic!("develop_into {path}: {e}"));

    Some((sensor, plane, output))
}

/// [`decode_and_develop`], memoized per file. Three of this file's tests
/// each independently decode all of [`DECODABLE`]; a debug-mode decode plus
/// `develop_into` over a ~47-megapixel frame is expensive enough that doing
/// it three times over (once per test) was a real contributor to `AC8`'s
/// pre-registered 60s bound (see `tests/support/oracle.rs`'s sort-based
/// fixes for the other contributor). `OnceLock::get_or_init` is the correct
/// primitive here rather than a hand-rolled `Mutex`: concurrently-running
/// tests that want the SAME file block on one real decode instead of racing
/// to redo it, and tests wanting DIFFERENT files still proceed in parallel.
fn cached_fixture(
    manifest: &Manifest,
    root: &CorpusRoot,
    path: &str,
) -> Option<&'static DecodedFixture> {
    static L1021223: std::sync::OnceLock<Option<DecodedFixture>> = std::sync::OnceLock::new();
    static L1026016: std::sync::OnceLock<Option<DecodedFixture>> = std::sync::OnceLock::new();
    static L1000622: std::sync::OnceLock<Option<DecodedFixture>> = std::sync::OnceLock::new();

    let cell = match path {
        "LEICA-Q2-MONO/L1021223.DNG" => &L1021223,
        "LEICA-Q2-MONO/L1026016.DNG" => &L1026016,
        "LEICA-M-MONOCHROM/L1000622.DNG" => &L1000622,
        _ => panic!("cached_fixture: {path} is not in DECODABLE"),
    };
    cell.get_or_init(|| decode_and_develop(manifest, root, path))
        .as_ref()
}

/// A minimal, valid `Sensor` covering the whole `width x 1` plane (no
/// `ActiveArea`/crop/orientation tags) — a test overrides what it needs.
/// Mirrors `tests/develop.rs`'s own `minimal_sensor`; redefined locally
/// because integration test binaries do not share code across files.
fn minimal_sensor(width: u32) -> Sensor {
    Sensor {
        ifd_index: 0,
        width,
        height: 1,
        bits_per_sample: 14,
        samples_per_pixel: 1,
        photometric: 34892,
        compression: Compression::Uncompressed,
        rows_per_strip: None,
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: None,
        white_level: None,
        black_level_repeat_dim: None,
        active_area: None,
        default_crop_origin: None,
        default_crop_size: None,
        orientation: None,
        opcode_lists: [false, false, false],
        malformed_tags: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 + AC2 — L2, the per-pixel bound, on every decodable frame (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn every_pixel_is_within_half_an_lsb_of_the_exact_affine_map() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    for path in DECODABLE {
        let Some((sensor, plane, output)) = cached_fixture(&manifest, &root, path) else {
            continue; // SKIP already announced by CorpusFile::require
        };

        let (black, white) = oracle::resolve_levels(sensor);
        let crop = oracle::crop_window_samples(sensor, plane);
        let check = oracle::bound_check(output, &crop, black, white);

        // ⚠ PRE-REGISTERED (`pre-register-the-tolerance`): `< 0.5`, falsifier
        // a single pixel at `>= 0.5`. Measured at design: max 0.499968 over
        // 111,529,040 pixels, zero at or above 0.5. A max at or above 0.5
        // here is a FINDING, not a threshold to relax.
        assert!(
            check.max_deviation < 0.5,
            "{path}: AC1's pre-registered bound is < 0.5 LSB from the exact real-valued \
             affine map; got max deviation {} — this is a FINDING, not a threshold to relax",
            check.max_deviation
        );

        // AC2: the shipped output must NOT be satisfiable by truncation —
        // measured 45.0-50.1% at design. Floor of 40%, not the exact figure
        // (data-dependent). ⚠ SPEC-015/FU-8: the real margin is not "5
        // points" — in-range disagreement is structurally ~0.5006/0.5006/
        // 0.5001 regardless of content, and only CLIPPED pixels (round ==
        // floor at the endpoints) pull the total down. A CORRECT
        // implementation falls under this 40% floor once the clipped share
        // exceeds 20.09% (measured break-even) — L1000622.DNG is already at
        // 10.05% clipped. Fails loudly, safe direction (false red, never
        // false green).
        let truncation_disagreement_fraction =
            check.truncation_disagreements as f64 / check.total as f64;
        assert!(
            truncation_disagreement_fraction > 0.40,
            "{path}: AC2 requires >40% of pixels to disagree with the TRUNCATED map, so a \
             future truncating 'simplification' cannot satisfy AC1 by accident; got {:.1}% \
             ({}/{})",
            truncation_disagreement_fraction * 100.0,
            check.truncation_disagreements,
            check.total
        );

        eprintln!(
            "{path}: max |shipped - exact| = {:.6} ({}/{} px), truncation disagreement = {:.1}%",
            check.max_deviation,
            check.total,
            check.total,
            truncation_disagreement_fraction * 100.0
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — L1, the permutation property, WITHOUT the orientation table (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_developed_histogram_is_the_normalized_crop_windows() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    for path in DECODABLE {
        let Some((sensor, plane, output)) = cached_fixture(&manifest, &root, path) else {
            continue; // SKIP already announced by CorpusFile::require
        };

        let (black, white) = oracle::resolve_levels(sensor);
        let crop = oracle::crop_window_samples(sensor, plane);
        let expected: Vec<u16> = crop
            .iter()
            .map(|&v| oracle::rounded_affine(v, black, white))
            .collect();

        // `multiset_equal` (sort-based) rather than comparing two
        // `HashMap`-based histograms directly: at ~47-megapixel scale a
        // `HashMap` over every pixel was the dominant cost in a debug build
        // (`AC8`'s pre-registered 60s bound — see `tests/support/oracle.rs`).
        // Both express the SAME property; this is a performance choice, not
        // a weaker check.
        assert!(
            oracle::multiset_equal(output, &expected),
            "{path}: histogram(develop_into output) must equal histogram(normalize(crop \
             window)) taken in raster order with NO orientation applied — develop_into \
             applies only a PERMUTATION of these values, which cannot change a histogram"
        );

        eprintln!(
            "{path}: histogram property holds exactly — {} distinct levels",
            oracle::distinct_count(output)
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — L1, the injectivity property: pure math (tier A), then the corpus
// cross-check (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn normalization_is_strictly_monotonic_and_injective() {
    // Representative (BlackLevel, WhiteLevel) pairs: the two real cameras'
    // own measured levels (`tests/develop.rs`), the widest legal range AC4
    // names explicitly (`W - B == 65535`), and the narrowest non-degenerate
    // one.
    for (black, white) in [(512u32, 16383u32), (220, 16383), (0, 65535), (1, 2)] {
        // Monotonic non-decreasing over the WHOLE u16 domain, exhaustively:
        // checking every consecutive pair proves the whole function, since
        // monotonicity is transitive.
        let mut previous_exact = oracle::exact_affine(0, black, white);
        let mut previous_rounded = oracle::rounded_affine(0, black, white);
        for raw in 1..=u16::MAX {
            let exact = oracle::exact_affine(raw, black, white);
            assert!(
                exact >= previous_exact,
                "exact_affine must be monotonic non-decreasing: raw={raw}, black={black}, \
                 white={white}"
            );
            let rounded = oracle::rounded_affine(raw, black, white);
            assert!(
                rounded >= previous_rounded,
                "rounded_affine must be monotonic non-decreasing: raw={raw}, black={black}, \
                 white={white}"
            );
            previous_exact = exact;
            previous_rounded = rounded;
        }

        // Strictly increasing (hence injective) on [BlackLevel, WhiteLevel]
        // itself — AC4's actual claim, and the fact that lets the tier-B
        // check below compare COUNTS rather than needing the injection
        // re-proven per file.
        for raw in (black + 1)..=white {
            let prev_raw = u16::try_from(raw - 1).expect("W - B <= 65535, so raw fits u16");
            let this_raw = u16::try_from(raw).expect("W - B <= 65535, so raw fits u16");
            let prev = oracle::rounded_affine(prev_raw, black, white);
            let curr = oracle::rounded_affine(this_raw, black, white);
            assert!(
                curr > prev,
                "rounded normalization must be STRICTLY increasing inside \
                 [BlackLevel, WhiteLevel] (injective): black={black}, white={white}, raw={raw} \
                 got {curr} after {prev}"
            );
        }
    }
}

#[test]
fn distinct_output_levels_equal_distinct_input_levels() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    // Measured at design (`Implementation Context`) on two of the three
    // files — both the FULL in-range domain (`white - black + 1`) exactly.
    // `L1021223.DNG` shares `L1026016.DNG`'s levels and was not separately
    // reported.
    for (path, measured_distinct) in [
        ("LEICA-Q2-MONO/L1026016.DNG", Some(15872usize)),
        ("LEICA-M-MONOCHROM/L1000622.DNG", Some(16164usize)),
        ("LEICA-Q2-MONO/L1021223.DNG", None),
    ] {
        let Some((sensor, plane, output)) = cached_fixture(&manifest, &root, path) else {
            continue; // SKIP already announced by CorpusFile::require
        };

        let (black, white) = oracle::resolve_levels(sensor);
        let crop = oracle::crop_window_samples(sensor, plane);

        // A 65536-entry presence bitmap (`oracle::distinct_count`), not a
        // `HashSet<u16>`: same reasoning as `multiset_equal` above — no
        // hashing at ~47-megapixel scale.
        let distinct_output = oracle::distinct_count(output);
        let distinct_expected = oracle::distinct_count(
            &crop
                .iter()
                .map(|&v| oracle::rounded_affine(v, black, white))
                .collect::<Vec<_>>(),
        );

        assert_eq!(
            distinct_output, distinct_expected,
            "{path}: normalization is injective (AC4), so distinct output levels must equal \
             distinct (clamped, normalized) crop-window levels"
        );
        if let Some(expected_count) = measured_distinct {
            assert_eq!(
                distinct_output, expected_count,
                "{path}: measured at design — {expected_count} distinct levels, the FULL \
                 in-range domain (white - black + 1)"
            );
        }
        eprintln!("{path}: {distinct_output} distinct output levels");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5(a) + AC6 — the levels red-proof (tier A, hand-built, no corpus)
//
// `BlackLevel` is a public `Sensor` field, so this fault needs no source
// mutation at all (`DEC-021`): its effect on `develop_into` is fully
// reproduced by calling the REAL, unmutated function with a `Sensor`
// carrying the wrong value.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_oracle_is_red_on_a_levels_fault() {
    let black = 512u32; // Q2M's own measured BlackLevel (tests/develop.rs)
    let white = 16383u32;
    // Every value 0..=17407 — comprehensive and deterministic, no file
    // needed: spans below-BlackLevel, in-range, and above-WhiteLevel samples
    // in one hand-built fixture, the same three regions AC2's real measured
    // evidence spans.
    let raw: Vec<u16> = (0..=17407u32)
        .map(|v| u16::try_from(v).expect("fits u16"))
        .collect();

    let mut sensor = minimal_sensor(u32::try_from(raw.len()).expect("fits u32"));
    sensor.black_level = Some(black);
    sensor.white_level = Some(white);

    let mut honest_output = vec![0u16; raw.len()];
    develop_into(&sensor, &raw, &mut honest_output).expect("fits");

    let crop = oracle::crop_window_samples(&sensor, &raw);
    let honest_check = oracle::bound_check(&honest_output, &crop, black, white);
    assert!(
        honest_check.max_deviation < 0.5,
        "the honest tree must satisfy AC1's own bound (got {}), or this proves nothing about \
         the fault below",
        honest_check.max_deviation
    );

    let mut faulted_sensor = sensor.clone();
    faulted_sensor.black_level = Some(black + 64); // the measured fault (Implementation Context)
    let mut faulted_output = vec![0u16; raw.len()];
    develop_into(&faulted_sensor, &raw, &mut faulted_output).expect("fits");

    // The oracle does not know the fault happened — it checks the FAULTED
    // output against the expectation derived from the TRUE black level.
    let faulted_check = oracle::bound_check(&faulted_output, &crop, black, white);
    assert!(
        faulted_check.max_deviation >= 0.5,
        "a BlackLevel+64 fault must turn AC1's bound red — got max deviation {}, no better \
         than the honest tree's {}",
        faulted_check.max_deviation,
        honest_check.max_deviation
    );

    let wrong_pixels = honest_output
        .iter()
        .zip(faulted_output.iter())
        .filter(|(honest, faulted)| honest != faulted)
        .count();
    eprintln!(
        "RED-PROOF (BlackLevel+64, hand-built, no corpus): honest max_deviation={:.6} faulted \
         max_deviation={:.6} — {wrong_pixels}/{} pixels wrong, the fault turned AC1's bound red",
        honest_check.max_deviation,
        faulted_check.max_deviation,
        raw.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5(b) + AC6 — the orientation red-proof (tier A, hand-built, no corpus)
//
// Unlike the levels fault, "identity at crop_source_coords' call site"
// (SPEC-014/FU-3's historical bug) is a call-site defect no public `Sensor`
// field can express — it requires a real, separate compilation of a
// deliberately mutated copy of `src/develop.rs` (`DEC-021`, following
// `DEC-017`'s precedent for exactly this reason). The working tree's
// `src/develop.rs` is never touched.
//
// ⚠ `SPEC-015/FU-7` — scope this proof honestly. The injected fault is an
// IDENTITY at the call site, which reads outside the crop window on this
// fixture and so produces a DIFFERENT multiset (three zeros, not the honest
// tree's none) — `AC3`'s histogram property catches DEGENERACY here, never a
// permutation being the WRONG permutation (that limit is `DEC-020`'s, and it
// is inherent: see `SPEC-015/FU-6`). This red-proof is sound; its name
// ("the orientation red-proof") is broader than what it actually exercises.
// ─────────────────────────────────────────────────────────────────────────────

/// The exact fixture `develop_into_applies_orientation_to_pixels_not_only_dimensions`
/// (`tests/develop.rs`, `SPEC-014/FU-3`) pins: 3 wide x 2 tall,
/// `sample(x, y) = 10*y + x`, `BlackLevel 0` / `WhiteLevel u16::MAX` so
/// `normalize` is the identity. Kept byte-for-byte identical to
/// [`PROBE_MAIN`]'s own copy (the probe cannot literally import this
/// function, since it is compiled as a separate crate) and to the existing
/// pinned regression test, so all three stay mutually checkable.
const ORIENTATION_FIXTURE_SRC: [u16; 6] = [0, 1, 2, 10, 11, 12];

fn orientation_fixture_sensor() -> Sensor {
    let mut sensor = minimal_sensor(3);
    sensor.height = 2;
    sensor.black_level = Some(0);
    sensor.white_level = Some(u32::from(u16::MAX));
    sensor.orientation = Some(6); // Rotate 90 CW — swaps dimensions, non-square
    sensor
}

/// A directory removed on drop, even if the test panics first — mirrors
/// `tests/plane_oracle.rs`'s `TempDir` (`DEC-017`).
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-develop-oracle-{label}-{}-{nanos}",
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

/// The ONE injected fault this red-proof exists to catch: `develop_into`'s
/// inner loop resolves `crop_source_coords` correctly but then DISCARDS the
/// result, binding `(crop_x, crop_y) = (out_x, out_y)` instead —
/// `SPEC-014/FU-3`'s historical bug, the exact fault that left 141 of 141
/// tests green before that spec's own hand-built fixture closed it.
fn inject_orientation_identity_fault(develop_rs: &Path) {
    let src = std::fs::read_to_string(develop_rs)
        .unwrap_or_else(|e| panic!("read {}: {e}", develop_rs.display()));

    let needle = "let (crop_x, crop_y) = crop_source_coords(\n                geometry.orientation,\n                out_x,\n                out_y,\n                geometry.crop_width,\n                geometry.crop_height,\n            );";
    let occurrences = src.matches(needle).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one call to `crop_source_coords` in src/develop.rs's \
         `develop_into`; found {occurrences} — the call site moved, update this test"
    );

    let mutated = src.replacen(
        needle,
        "let (crop_x, crop_y) = (out_x, out_y); // RED-PROOF INJECTION -- tests/develop_oracle.rs, never in the real tree",
        1,
    );
    std::fs::write(develop_rs, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", develop_rs.display()));
}

/// The probe's `main()`. `develop_into`'s public API takes an
/// already-parsed `Sensor` and an already-unpacked plane slice — unlike
/// `SPEC-013`'s red-proof probe (`plane::unpack_into` needs real file
/// bytes), this fixture is a Rust literal: no file I/O, no corpus, at all.
/// Byte-for-byte the same fixture as [`ORIENTATION_FIXTURE_SRC`] /
/// `orientation_fixture_sensor`.
const PROBE_MAIN: &str = r#"
fn main() {
    let sensor = irradiance::ifd::Sensor {
        ifd_index: 0,
        width: 3,
        height: 2,
        bits_per_sample: 14,
        samples_per_pixel: 1,
        photometric: 34892,
        compression: irradiance::ifd::Compression::Uncompressed,
        rows_per_strip: None,
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: Some(0),
        white_level: Some(u32::from(u16::MAX)),
        black_level_repeat_dim: None,
        active_area: None,
        default_crop_origin: None,
        default_crop_size: None,
        orientation: Some(6),
        opcode_lists: [false, false, false],
        malformed_tags: vec![],
    };
    let src: [u16; 6] = [0, 1, 2, 10, 11, 12];
    let mut dst = [0u16; 6];
    irradiance::develop::develop_into(&sensor, &src, &mut dst)
        .unwrap_or_else(|e| panic!("develop_into: {e}"));
    let text: Vec<String> = dst.iter().map(|v| v.to_string()).collect();
    println!("{}", text.join(","));
}
"#;

/// Copy the crate to `dest`, optionally injecting the fault, and drop in the
/// synthesized probe binary as a second, explicit `[[bin]]` target — mirrors
/// `tests/plane_oracle.rs`'s `stage_probe_crate` (`DEC-017`).
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
        inject_orientation_identity_fault(&dest.join("src/develop.rs"));
    }

    std::fs::write(dest.join("src/bin/develop_oracle_probe.rs"), PROBE_MAIN)
        .expect("write probe binary source");

    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml.push_str(
        "\n[[bin]]\nname = \"develop_oracle_probe\"\npath = \"src/bin/develop_oracle_probe.rs\"\n",
    );
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

/// Build the staged crate in **release** mode and run the probe, returning
/// its printed `dst` array.
fn build_and_run_probe(dir: &Path) -> Vec<u16> {
    let build = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "develop_oracle_probe",
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

    let bin = dir.join("target/release/develop_oracle_probe");
    let run = std::process::Command::new(&bin)
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "develop_oracle_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .split(',')
        .map(|s| {
            s.parse::<u16>()
                .unwrap_or_else(|e| panic!("parse probe output {s:?}: {e}"))
        })
        .collect()
}

/// The oracle's own check (AC3's histogram property) applied to `output` —
/// factored out so the red-proof and its control apply IDENTICAL logic to
/// the mutant and the honest tree respectively.
fn histogram_check_passes(sensor: &Sensor, src: &[u16], output: &[u16]) -> bool {
    let (black, white) = oracle::resolve_levels(sensor);
    let crop = oracle::crop_window_samples(sensor, src);
    let expected = oracle::histogram(
        &crop
            .iter()
            .map(|&v| oracle::rounded_affine(v, black, white))
            .collect::<Vec<_>>(),
    );
    oracle::histogram(output) == expected
}

#[test]
fn the_oracle_is_red_on_an_orientation_fault() {
    let sensor = orientation_fixture_sensor();
    let src = ORIENTATION_FIXTURE_SRC;

    // The honest tree, in-process — develop_into is already linked into this
    // test binary, so no subprocess is needed for this half.
    let mut honest = [0u16; 6];
    develop_into(&sensor, &src, &mut honest).expect("3x2 develops under Orientation 6");
    assert_eq!(
        honest,
        [10, 0, 11, 1, 12, 2],
        "the honest tree must match SPEC-014/FU-3's own pinned expectation, or this proves \
         nothing about the fault below"
    );
    assert!(
        histogram_check_passes(&sensor, &src, &honest),
        "the oracle must be GREEN on the honest tree, or the red below proves nothing"
    );

    let dir = TempDir::new("mutant");
    stage_probe_crate(&dir.0, true);
    let mutant_output = build_and_run_probe(&dir.0);

    // The clause every red-proof in this repo exists for: assert the OUTPUT
    // actually changed before concluding anything about what was caught.
    assert_ne!(
        mutant_output, honest,
        "the injected identity-at-call-site fault did NOT change develop_into's output — it \
         is a semantic no-op, and this red-proof has caught NOTHING"
    );

    assert!(
        !histogram_check_passes(&sensor, &src, &mutant_output),
        "AC3's permutation property did not catch the orientation-identity fault — got \
         {mutant_output:?}"
    );

    let wrong_pixels = honest
        .iter()
        .zip(mutant_output.iter())
        .filter(|(h, m)| h != m)
        .count();
    eprintln!(
        "RED-PROOF (orientation identity at call site, hand-built, no corpus): honest={honest:?} \
         mutant={mutant_output:?} — {wrong_pixels}/6 pixels wrong, AC3's histogram property \
         correctly rejects the mutant"
    );
}

#[test]
fn the_orientation_fixture_oracle_control_is_green() {
    // `oracle-must-be-shown-red`'s other half: a red above could be the
    // copy-and-rebuild apparatus itself, not the injection (`DEC-009`'s
    // discipline). This is the negative control.
    let sensor = orientation_fixture_sensor();
    let src = ORIENTATION_FIXTURE_SRC;

    let mut honest = [0u16; 6];
    develop_into(&sensor, &src, &mut honest).expect("3x2 develops under Orientation 6");

    let dir = TempDir::new("control");
    stage_probe_crate(&dir.0, false);
    let control_output = build_and_run_probe(&dir.0);

    assert_eq!(
        control_output, honest,
        "the UNMUTATED copy-and-rebuild apparatus must reproduce develop_into's real output"
    );
    assert!(
        histogram_check_passes(&sensor, &src, &control_output),
        "the honest tree must satisfy AC3's own property, or the red-proof above proves \
         nothing about the fault"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// FU-10 — a tier-A positional fixture large enough to cross a SIZE-GATED fault
//
// Every positional test elsewhere in this repo —
// `crop_source_coords_matches_the_worked_example_for_all_eight_orientations`
// (`src/develop.rs`) and `develop_into_applies_orientation_to_pixels_not_only_dimensions`
// (`tests/develop.rs`) — uses a fixture of <= 6 pixels. Measured
// (`HANDOFF-037`): a fault written as `if crop_width > 100 { /* Orientation
// 8's mapping, where the file says 6 */ }` corrupts 100% of a real
// 47-megapixel frame's positions, and 150/150 of this repo's tests passed,
// because every hand-built fixture has `crop_width <= 3`. CI never runs the
// corpus at all (0/7 files present, run `34003871323`), so the entire
// real-data layer of `DEC-020`'s rank/frequency oracle is invisible to CI —
// this fixture is generated IN THE TEST, so it runs wherever CI does.
//
// ⚠ **Why this is allowed to encode Orientation's per-case mapping, when
// `AC3`'s property test above is forbidden to.** `DEC-020`'s prohibition is on
// the RANK/FREQUENCY oracle specifically, because a mirror there would defeat
// the whole point of an independent check on real data. This fixture is a
// different animal — a tier-A POSITIONAL regression test, the same category
// as the two <=6px tests named above, which already hand-derive the mapping
// for the orientations they cover. Positional testing is the ONLY technique
// that can see a wrong-permutation fault at all (`FU-6`); this fixture exists
// because that technique needed a fixture big enough to cross a size gate,
// not because the prohibition was relaxed.
//
// ⚠ **RESIDUAL — read before assuming this closes the class.** A fault gated
// at `crop_width > 2000` still evades a 1024-wide fixture: no finite fixture
// dominates every possible gate constant, and this one only raises the floor
// from 8px to 1024px, not to infinity. It also does nothing for `FU-6`'s
// wrong-permutation blind spot in `DEC-020`'s own techniques: `bound_check`
// and `multiset_equal` cannot distinguish one valid permutation from another
// because that correspondence IS the eight-case table — a limit that is
// INHERENT to comparing by value rather than position, independent of size,
// and this fixture does not touch it. What this fixture buys is narrower and
// real: orientations 2 and 6, specifically, are now checked positionally up
// to 1024px, tier A, in CI. Orientations 3, 4, 5, 7, 8 are not covered here at
// any size, and nothing above 1024px is covered by any test in this repo.
// ─────────────────────────────────────────────────────────────────────────────

/// `> 1000` on purpose, not merely `>= 1024`: `HANDOFF-037` measured a second,
/// STRICTER gate (`crop_width > 1000`) that a `> 100` fixture would not have
/// crossed, so this fixture's `crop_width` must clear both. 1024x768 (a 4:3
/// real-photo aspect ratio) keeps the pixel count near the ~0.79 Mpx the
/// handoff budgeted (`## Cost`, below, has the measured figure).
const LARGE_WIDTH: u32 = 1024;
const LARGE_HEIGHT: u32 = 768;

/// `sample(x, y) = (y * width + x) as u16` — every source pixel names its own
/// raster position, the same technique `crop_origin_is_relative_to_active_area_not_the_raw_plane`
/// (`src/develop.rs`) and `develop_into_applies_orientation_to_pixels_not_only_dimensions`
/// (`tests/develop.rs`) use at 100/6 pixels respectively, scaled up. This
/// WRAPS past 65,535 — the fixture has 786,432 pixels, and `y * width + x`
/// reaches 785,791 — and that is EXPECTED, not a bug: values repeating under
/// the wrap is harmless here because every check below reads a value at a
/// COMPUTED, known coordinate, never by searching for a value or assuming
/// distinctness (unlike `AC4`'s injectivity checks, which is exactly why
/// `AC4` uses real corpus files rather than a synthetic one like this).
fn large_fixture_plane() -> Vec<u16> {
    (0..LARGE_HEIGHT)
        .flat_map(|y| (0..LARGE_WIDTH).map(move |x| (y * LARGE_WIDTH + x) as u16))
        .collect()
}

/// `orientation`'s own `Sensor`, no `ActiveArea`/crop tags — so `crop_width` /
/// `crop_height` default to the full `LARGE_WIDTH` x `LARGE_HEIGHT` plane.
/// `BlackLevel 0` / `WhiteLevel u16::MAX` makes `normalize` the identity (the
/// same trick the two tier-A tests referenced above use), so this test needs
/// no access to `normalize` to state its expectation, and cannot be
/// accidentally satisfied by a LEVELS fault (`AC1`'s concern, not this one's).
fn large_fixture_sensor(orientation: u32) -> Sensor {
    let mut sensor = minimal_sensor(LARGE_WIDTH);
    sensor.height = LARGE_HEIGHT;
    sensor.black_level = Some(0);
    sensor.white_level = Some(u32::from(u16::MAX));
    sensor.orientation = Some(orientation);
    sensor
}

#[test]
fn rotating_orientation_is_positionally_correct_at_production_scale() {
    let sensor = large_fixture_sensor(6); // Rotate 90 CW — swaps dimensions
    let src = large_fixture_plane();

    // FU-10's `M7`: a fault that transposes `output_dimensions`' result for
    // orientations 5-8, size-gated. Checked directly, at production scale.
    assert_eq!(
        output_dimensions(&sensor).expect("fits"),
        (LARGE_HEIGHT, LARGE_WIDTH),
        "Orientation 6 must swap width and height"
    );

    let mut dst = vec![0u16; (LARGE_WIDTH * LARGE_HEIGHT) as usize];
    let start = std::time::Instant::now();
    develop_into(&sensor, &src, &mut dst).expect("fits");
    let elapsed = start.elapsed();

    // Physically rotating the WxH grid 90 degrees clockwise swaps the
    // dimensions to HxW; the new top-left corner (0,0) is the OLD bottom-left
    // corner (0, H-1), and the new image's rows run down the OLD image's
    // columns — derived by hand from Orientation 6's own semantics (the same
    // reasoning `develop_into_applies_orientation_to_pixels_not_only_dimensions`
    // uses for its 3x2 fixture), independently of `crop_source_coords`.
    let out_width = LARGE_HEIGHT;
    let out_height = LARGE_WIDTH;
    let mut mismatches = 0usize;
    let mut first_mismatch = None;
    for out_y in 0..out_height {
        for out_x in 0..out_width {
            let (src_x, src_y) = (out_y, LARGE_HEIGHT - 1 - out_x);
            let expected = src[(src_y * LARGE_WIDTH + src_x) as usize];
            let actual = dst[(out_y * out_width + out_x) as usize];
            if actual != expected {
                mismatches += 1;
                first_mismatch.get_or_insert((out_x, out_y, expected, actual));
            }
        }
    }
    assert_eq!(
        mismatches,
        0,
        "{mismatches}/{} output pixels positionally wrong under Orientation 6 at production \
         scale; first mismatch (out_x, out_y, expected, actual) = {first_mismatch:?}",
        dst.len(),
    );
    eprintln!(
        "rotating_orientation_is_positionally_correct_at_production_scale: {LARGE_WIDTH}x{LARGE_HEIGHT} \
         -> {out_width}x{out_height}, {} px, develop_into in {:.3}s",
        dst.len(),
        elapsed.as_secs_f64()
    );
}

#[test]
fn flipping_orientation_is_positionally_correct_at_production_scale() {
    let sensor = large_fixture_sensor(2); // Mirror horizontal — no dimension swap
    let src = large_fixture_plane();

    assert_eq!(
        output_dimensions(&sensor).expect("fits"),
        (LARGE_WIDTH, LARGE_HEIGHT),
        "Orientation 2 must NOT swap width and height"
    );

    let mut dst = vec![0u16; (LARGE_WIDTH * LARGE_HEIGHT) as usize];
    let start = std::time::Instant::now();
    develop_into(&sensor, &src, &mut dst).expect("fits");
    let elapsed = start.elapsed();

    // Mirroring horizontally reverses each row left-to-right; dimensions are
    // unchanged — derived by hand, independently of `crop_source_coords`.
    let mut mismatches = 0usize;
    let mut first_mismatch = None;
    for out_y in 0..LARGE_HEIGHT {
        for out_x in 0..LARGE_WIDTH {
            let (src_x, src_y) = (LARGE_WIDTH - 1 - out_x, out_y);
            let expected = src[(src_y * LARGE_WIDTH + src_x) as usize];
            let actual = dst[(out_y * LARGE_WIDTH + out_x) as usize];
            if actual != expected {
                mismatches += 1;
                first_mismatch.get_or_insert((out_x, out_y, expected, actual));
            }
        }
    }
    assert_eq!(
        mismatches,
        0,
        "{mismatches}/{} output pixels positionally wrong under Orientation 2 at production \
         scale; first mismatch (out_x, out_y, expected, actual) = {first_mismatch:?}",
        dst.len(),
    );
    eprintln!(
        "flipping_orientation_is_positionally_correct_at_production_scale: {LARGE_WIDTH}x{LARGE_HEIGHT} \
         px, develop_into in {:.3}s",
        elapsed.as_secs_f64()
    );
}
