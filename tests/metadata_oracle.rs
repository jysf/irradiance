//! `SPEC-005` — the live metadata oracle: diff `Sensor` against `exiftool`
//! and `dnglab analyze --meta --json`, and prove the diff goes red.
//!
//! Replaces `tests/ifd_reader.rs`'s hand-transcribed `Expected` table (the
//! tag-value columns) with a comparison that runs the tools every time,
//! rather than trusting a table one past session typed by hand.
//!
//! Three tiers, and they run under different conditions on purpose:
//!
//! - **Tier A** (`oracle_is_clean_on_an_unmodified_reading`,
//!   `oracle_names_the_one_field_that_was_perturbed`) needs no tool and no
//!   corpus — it replays a committed sample of `exiftool`'s output
//!   (`tests/oracle-fixtures/`) through the SAME parsing code a real run
//!   uses. This is the only half that runs in CI.
//! - **Tier B, tool-gated** (everything else that shells out) needs
//!   `exiftool`/`dnglab` on `PATH` and skips loudly, naming the tool, when
//!   absent (AC6).
//! - **Tier B, corpus-gated** additionally needs `$IRRADIANCE_CORPUS_DIR`
//!   populated and skips loudly per file, naming it (`SPEC-002`'s idiom).
//!
//! `dnglab` is excluded from `PENTAX-K3III-MONO/K3III.PEF` throughout (AC4.2)
//! — its values there come from rawler's camera database, not the file.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tools.rs"]
mod tools;

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::{
    ActiveArea, Compression, Container, DefaultCropOrigin, DefaultCropSize, Sensor,
    TAG_ACTIVE_AREA, TAG_BLACK_LEVEL_REPEAT_DIM,
};

// ─────────────────────────────────────────────────────────────────────────────
// The exiftool group per file — MEASURED, not derived (SPEC-005
// Implementation Context: "the mapping is not positional-obvious"). Six
// files select sensor IFD #1 in our walk, which happens to be the FIRST
// SubIFD — exiftool's `-g1` leaves the first SubIFD unnumbered (`SubIFD`,
// then `SubIFD1`, `SubIFD2`, ...). `K3III.PEF` has no SubIFDs at all; its
// plane is `IFD0`.
// ─────────────────────────────────────────────────────────────────────────────

const EXIFTOOL_SENSOR_GROUP: &[(&str, &str)] = &[
    ("LEICA-Q2-MONO/L1021223.DNG", "SubIFD"),
    ("LEICA-Q2-MONO/L1026016.DNG", "SubIFD"),
    ("LEICA-Q2-MONO/L1026192.DNG", "SubIFD"),
    ("LEICA-M-MONOCHROM/L1000622.DNG", "SubIFD"),
    ("LEICA-M-MONOCHROM-TYP246/M2462362.DNG", "SubIFD"),
    ("PENTAX-K3III-MONO/K3III.DNG", "SubIFD"),
    ("PENTAX-K3III-MONO/K3III.PEF", "IFD0"),
];

fn exiftool_group(manifest_path: &str) -> &'static str {
    EXIFTOOL_SENSOR_GROUP
        .iter()
        .find(|(p, _)| *p == manifest_path)
        .unwrap_or_else(|| {
            panic!(
                "no exiftool group mapping for {manifest_path} — add one, MEASURED against the \
                 real file (`exiftool -n -G1 <file>`), before trusting a comparison"
            )
        })
        .1
}

/// Read and parse one corpus file's sensor plane, or `None` (already having
/// printed a SKIP line via `CorpusFile::require`) when it is absent.
fn sensor_at(
    root: &CorpusRoot,
    entry: &corpus::CorpusFile,
) -> Option<(std::path::PathBuf, Sensor)> {
    let path = entry.require(root)?;
    let data = std::fs::read(&path).expect("read corpus file");
    let sensor = Container::parse(&data)
        .unwrap_or_else(|e| panic!("{}: container did not parse: {e}", entry.path))
        .sensor()
        .unwrap_or_else(|e| panic!("{}: no sensor plane: {e}", entry.path));
    Some((path, sensor))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1, AC2 — exiftool is the tag-level oracle, on all seven files
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn metadata_matches_exiftool_on_every_corpus_file() {
    if !tools::exiftool_available() {
        eprintln!("SKIP metadata_matches_exiftool_on_every_corpus_file — exiftool not on PATH");
        return;
    }
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let mut checked = 0;

    for file in &manifest.files {
        let Some((path, sensor)) = sensor_at(&root, file) else {
            continue;
        };
        let group = exiftool_group(&file.path);
        let reading = tools::exiftool_reading(&path, group)
            .unwrap_or_else(|e| panic!("{}: exiftool failed: {e}", file.path));

        let mismatches = tools::diff(&sensor, &reading);
        assert!(
            mismatches.is_empty(),
            "{}: {} field(s) disagree with exiftool:\n{}",
            file.path,
            mismatches.len(),
            mismatches
                .iter()
                .map(|m| format!("  {m}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        checked += 1;
    }

    eprintln!(
        "metadata_matches_exiftool_on_every_corpus_file: {checked}/{} corpus files present",
        manifest.files.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — dnglab cross-checks the six unique scalars, on the six DNG files only
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dnglab_scalars_agree_on_the_six_dng_files() {
    if !tools::dnglab_available() {
        eprintln!("SKIP dnglab_scalars_agree_on_the_six_dng_files — dnglab not on PATH");
        return;
    }
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let mut checked = 0;

    for file in &manifest.files {
        if file.path.ends_with(".PEF") {
            continue; // AC4.2 — excluded, its own dedicated test
        }
        let Some((path, sensor)) = sensor_at(&root, file) else {
            continue;
        };
        let meta = tools::dnglab_meta(&path)
            .unwrap_or_else(|e| panic!("{}: dnglab failed: {e}", file.path));

        assert_eq!(meta.raw_width, sensor.width, "{}: rawWidth", file.path);
        assert_eq!(meta.raw_height, sensor.height, "{}: rawHeight", file.path);
        assert_eq!(
            meta.bit_depth, sensor.bits_per_sample,
            "{}: bitDepth",
            file.path
        );
        assert_eq!(
            Some(meta.white_level),
            sensor.white_level,
            "{}: whitelevels",
            file.path
        );
        assert_eq!(
            Some(meta.orientation),
            sensor.orientation,
            "{}: rawMetadata.exif.orientation",
            file.path
        );
        assert_eq!(
            Some(meta.black_level),
            sensor.black_level,
            "{}: blacklevels.levels",
            file.path
        );
        checked += 1;
    }

    eprintln!("dnglab_scalars_agree_on_the_six_dng_files: {checked}/6 DNG files present");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4.1 — dnglab's cropArea.p is sensor-absolute: ActiveArea origin + crop
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dnglab_crop_origin_is_active_area_plus_default_crop_origin() {
    if !tools::dnglab_available() {
        eprintln!(
            "SKIP dnglab_crop_origin_is_active_area_plus_default_crop_origin — dnglab not on PATH"
        );
        return;
    }
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let mut checked = 0;

    for file in &manifest.files {
        if file.path.ends_with(".PEF") {
            continue; // AC4.2
        }
        let Some((path, sensor)) = sensor_at(&root, file) else {
            continue;
        };
        let meta = tools::dnglab_meta(&path)
            .unwrap_or_else(|e| panic!("{}: dnglab failed: {e}", file.path));

        // An absent ActiveArea reads as (0, 0) — SPEC-005 AC4.1.
        let (active_left, active_top) = match sensor.active_area {
            Some(a) => (a.left, a.top),
            None => (0, 0),
        };
        let crop_origin = sensor.default_crop_origin.unwrap_or_else(|| {
            panic!(
                "{}: has no DefaultCropOrigin — AC4.1 does not apply",
                file.path
            )
        });

        let expected = (active_left + crop_origin.x, active_top + crop_origin.y);
        assert_eq!(
            meta.crop_area_p, expected,
            "{}: dnglab cropArea.p must equal ActiveArea's origin plus DefaultCropOrigin \
             (dnglab's is sensor-absolute; ours and exiftool's are DNG-relative)",
            file.path
        );
        checked += 1;
    }

    eprintln!(
        "dnglab_crop_origin_is_active_area_plus_default_crop_origin: {checked}/6 DNG files present"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4.2 — K3III.PEF is excluded from the dnglab comparison, by name and reason
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn pef_is_excluded_from_dnglab_because_its_values_are_not_in_the_file() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let file = manifest
        .get("PENTAX-K3III-MONO/K3III.PEF")
        .expect("manifest must carry PENTAX-K3III-MONO/K3III.PEF");
    let Some((path, sensor)) = sensor_at(&root, file) else {
        return;
    };

    // The file itself carries none of these DNG tags — AC2 already proves
    // our reader agrees with exiftool that they are absent.
    assert_eq!(sensor.black_level, None, "K3III.PEF: BlackLevel");
    assert_eq!(sensor.white_level, None, "K3III.PEF: WhiteLevel");
    assert_eq!(sensor.active_area, None, "K3III.PEF: ActiveArea");
    assert_eq!(
        sensor.default_crop_origin, None,
        "K3III.PEF: DefaultCropOrigin"
    );
    assert_eq!(sensor.default_crop_size, None, "K3III.PEF: DefaultCropSize");
    assert_eq!(
        sensor.bits_per_sample, 14,
        "K3III.PEF: BitsPerSample per the file"
    );

    if !tools::dnglab_available() {
        eprintln!(
            "SKIP pef_is_excluded_from_dnglab_because_its_values_are_not_in_the_file (dnglab \
             half) — dnglab not on PATH"
        );
        return;
    }
    let meta =
        tools::dnglab_meta(&path).unwrap_or_else(|e| panic!("K3III.PEF: dnglab failed: {e}"));

    // dnglab still answers, from rawler's camera database — the evidence:
    // its bitDepth (output depth, 16) disagrees with the file's own
    // BitsPerSample (14).
    assert_eq!(
        meta.bit_depth, 16,
        "K3III.PEF: dnglab's bitDepth is output depth, not the tag — the evidence this file is \
         excluded from the dnglab comparison"
    );
    assert_ne!(
        meta.bit_depth, sensor.bits_per_sample,
        "K3III.PEF: dnglab's bitDepth must diverge from the file's BitsPerSample, or this \
         test's premise for excluding the file no longer holds"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4.3 — K3III.DNG's malformed BlackLevelRepeatDim, read three different ways
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn malformed_black_level_repeat_dim_reads_three_different_ways() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let file = manifest
        .get("PENTAX-K3III-MONO/K3III.DNG")
        .expect("manifest must carry PENTAX-K3III-MONO/K3III.DNG");
    let Some((path, sensor)) = sensor_at(&root, file) else {
        return;
    };

    // Ours: DEC-012 — the tag is dropped, not the file, and the drop is
    // recorded rather than silent.
    assert_eq!(
        sensor.black_level_repeat_dim, None,
        "K3III.DNG: BlackLevelRepeatDim"
    );
    assert!(
        sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM),
        "K3III.DNG: tag 50713 must be recorded in malformed_tags, got {:?}",
        sensor.malformed_tags
    );

    if !tools::exiftool_available() {
        eprintln!(
            "SKIP malformed_black_level_repeat_dim_reads_three_different_ways (exiftool half) — \
             exiftool not on PATH"
        );
    } else {
        let fields = tools::exiftool(&path, "SubIFD", &["BlackLevelRepeatDim"])
            .unwrap_or_else(|e| panic!("K3III.DNG: exiftool failed: {e}"));
        assert_eq!(
            tools::values_for(&fields, "BlackLevelRepeatDim"),
            Some(&vec![1u32]),
            "K3III.DNG: exiftool must read a BARE 1 for a count-1 BlackLevelRepeatDim — not \
             \"1 1\" (well-formed) and not absent"
        );
    }

    if !tools::dnglab_available() {
        eprintln!(
            "SKIP malformed_black_level_repeat_dim_reads_three_different_ways (dnglab half) — \
             dnglab not on PATH"
        );
    } else {
        let run = tools::dnglab_analyze_meta(&path)
            .unwrap_or_else(|e| panic!("K3III.DNG: dnglab failed: {e}"));
        assert!(
            run.stderr.contains("BlackLevelRepeatDim") && run.stderr.contains("invalid length"),
            "K3III.DNG: dnglab must warn on stderr that it substituted an invalid \
             BlackLevelRepeatDim; got stderr: {:?}",
            run.stderr
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5, tier A — the comparator's red-proof: runs everywhere, no tool, no
// corpus. Replays a COMMITTED sample of exiftool's real output for
// LEICA-Q2-MONO/L1021223.DNG (measured 2026-08-21) through the exact parsing
// code `exiftool_reading` uses against a live process.
// ─────────────────────────────────────────────────────────────────────────────

const FIXTURE_LINE: &str = include_str!("oracle-fixtures/exiftool-l1021223-sensor.txt");

/// A `Sensor` matching [`FIXTURE_LINE`] field-for-field — the honest tree.
fn fixture_sensor() -> Sensor {
    Sensor {
        ifd_index: 1,
        width: 8424,
        height: 5632,
        bits_per_sample: 14,
        samples_per_pixel: 1,
        photometric: 34892,
        compression: Compression::Uncompressed,
        rows_per_strip: Some(5632),
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: Some(512),
        white_level: Some(16383),
        black_level_repeat_dim: Some([1, 1]),
        active_area: Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        }),
        default_crop_origin: Some(DefaultCropOrigin { x: 12, y: 24 }),
        default_crop_size: Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        }),
        orientation: Some(1),
        opcode_lists: [true, false, true],
        malformed_tags: vec![],
    }
}

fn fixture_reading() -> tools::ToolReading {
    let tags = tools::sensor_reading_tags("SubIFD");
    let fields = tools::parse_fields(&tags, FIXTURE_LINE).expect("fixture line parses");
    tools::reading_from_fields(&fields).expect("fixture fields build a ToolReading")
}

#[test]
fn oracle_is_clean_on_an_unmodified_reading() {
    let mismatches = tools::diff(&fixture_sensor(), &fixture_reading());
    assert!(
        mismatches.is_empty(),
        "an honest tree must diff clean, got {mismatches:?}"
    );
}

#[test]
fn oracle_names_the_one_field_that_was_perturbed() {
    let mut sensor = fixture_sensor();
    sensor.bits_per_sample = 13; // one field, deliberately wrong

    let mismatches = tools::diff(&sensor, &fixture_reading());
    assert_eq!(
        mismatches.len(),
        1,
        "exactly one field must disagree, got {mismatches:?}"
    );
    assert_eq!(mismatches[0].field, "BitsPerSample");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5, tier B — the oracle goes red on a patched tag in a REAL file
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn oracle_goes_red_on_a_patched_tag_in_a_real_file() {
    if !tools::exiftool_available() {
        eprintln!("SKIP oracle_goes_red_on_a_patched_tag_in_a_real_file — exiftool not on PATH");
        return;
    }
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let file = manifest
        .get("LEICA-Q2-MONO/L1021223.DNG")
        .expect("manifest must carry LEICA-Q2-MONO/L1021223.DNG");
    let Some(path) = file.require(&root) else {
        return;
    };
    let original = std::fs::read(&path).expect("read corpus file");

    // Locate ActiveArea's payload bytes — four LONGs, always stored
    // EXTERNALLY (never inline), so `Container::payload` returns a slice
    // that truly borrows `original`, and pointer arithmetic against it finds
    // the real file offset to patch.
    let container = Container::parse(&original).expect("parses");
    let ifd = container.sensor_ifd().expect("has a sensor plane");
    let entry = ifd
        .entry(TAG_ACTIVE_AREA)
        .expect("LEICA-Q2-MONO/L1021223.DNG has ActiveArea");
    let payload = container
        .payload(entry)
        .expect("ActiveArea payload readable");
    assert!(
        payload.len() > 4,
        "ActiveArea (4 LONGs = 16 bytes) must be stored externally for this patch to hit real \
         file bytes, not a copy inside Entry"
    );
    let offset = payload.as_ptr() as usize - original.as_ptr() as usize;
    let len = payload.len();

    let mut patched = original.clone();
    for b in patched
        .get_mut(offset..offset + len)
        .expect("payload range is in bounds")
    {
        *b ^= 0xFF;
    }
    assert_ne!(
        patched[offset..offset + len],
        original[offset..offset + len],
        "the patch must actually change the buffer before any conclusion is drawn from it — \
         this repo has concluded from a mutation that never applied, more than once"
    );

    let patched_sensor = Container::parse(&patched)
        .expect("patched bytes still parse — only a VALUE changed, not the structure")
        .sensor()
        .expect("patched bytes still have a sensor plane");

    // The tool reading is of the ORIGINAL, unpatched file.
    let original_reading =
        tools::exiftool_reading(&path, "SubIFD").expect("exiftool reading of the original file");

    let mismatches = tools::diff(&patched_sensor, &original_reading);
    assert_eq!(
        mismatches.len(),
        1,
        "patching ActiveArea must produce exactly one mismatch, got {mismatches:?}"
    );
    assert_eq!(mismatches[0].field, "ActiveArea");

    // Restore and re-run: the buffer was never written to disk (git status
    // stays clean by construction), and re-diffing the UNPATCHED bytes
    // proves the red above was caused by the patch, not by a bug in diff().
    let clean_sensor = Container::parse(&original)
        .expect("parses")
        .sensor()
        .expect("has a sensor plane");
    let clean_mismatches = tools::diff(&clean_sensor, &original_reading);
    assert!(
        clean_mismatches.is_empty(),
        "re-running on the unpatched bytes must be clean, got {clean_mismatches:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — a missing tool skips loudly, naming the tool
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_missing_tool_skips_loudly_naming_it() {
    // A binary name guaranteed absent, independent of what happens to be
    // installed on the host running this suite — AC6 must be provable
    // without physically uninstalling exiftool or dnglab. `run_tool` is the
    // exact guard both `exiftool()` and `dnglab_analyze_meta()` call.
    const ABSENT: &str = "irradiance-oracle-tool-that-does-not-exist";
    let err = tools::run_tool(ABSENT, &[], std::path::Path::new("/nonexistent"))
        .expect_err("a binary that cannot exist on any PATH must be a loud, typed Err");

    match err {
        tools::ToolError::NotOnPath(name) => {
            assert_eq!(name, ABSENT, "the error must name the missing tool");
        }
        other => panic!(
            "expected ToolError::NotOnPath, got {other:?} — a spawn failure must be \
                          CLASSIFIED, not just surfaced raw"
        ),
    }
}
