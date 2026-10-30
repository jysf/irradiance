//! `SPEC-026` — the colour-metadata oracle: diff `Sensor`'s DNG colour tags
//! (`ColorMatrix1/2`, `ForwardMatrix1/2`, `AsShotNeutral`,
//! `CalibrationIlluminant1/2`, per-channel `BlackLevel`/`WhiteLevel`) against
//! `exiftool` and `dnglab`, and prove the diff goes red.
//!
//! This is the SPEC-005 metadata oracle extended to colour, kept deliberately
//! SEPARATE from `tests/metadata_oracle.rs` (which stays the monochrome scalar
//! oracle, byte-for-byte unchanged) — the same partition `plane_oracle.rs`
//! keeps from `metadata_oracle.rs`.
//!
//! The matrices are compared **exactly, by cross-multiplication in `i128`**,
//! with NO float tolerance: the stored rationals are exact terminating decimals
//! (`n/10000`, `n/1000000` — measured 2026-10-30), so `exiftool -n`'s decimal is
//! the exact value, not a rounded view, and a value exiftool had to round would
//! (correctly) go red. `dnglab`'s `--meta --json` carries no matrices, so
//! exiftool is their sole oracle; `dnglab` cross-checks only the per-channel
//! levels.
//!
//! Tiers, as in the mono oracle:
//! - **Tier A** (the comparator red/green and the fixture control/red) needs no
//!   tool and no corpus — it replays a committed sample of exiftool's real
//!   output (`tests/oracle-fixtures/exiftool-5g4a9394-colour.txt`) through the
//!   SAME `colour_from_fields` parse a live run uses. The only half CI runs.
//! - **Tier B** shells out and skips loudly, naming the tool/file, when absent.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tools.rs"]
mod tools;

use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::{Cfa, Compression, Container, Rational, Sensor};
use tools::{Mismatch, ToolError};

// ─────────────────────────────────────────────────────────────────────────────
// The two Bayer files this oracle owns. SPEC-005's monochrome scalar oracle
// skips them BY NAME (its `CFA_COVERED_BY_COLOUR_ORACLE`) — "covered here", not
// "deferred". The exiftool group naming each file's sensor IFD is MEASURED
// (`exiftool -n -G1 <file>`: BlackLevel/WhiteLevel are `[SubIFD]` on both); the
// matrices/illuminants/AsShotNeutral are always `IFD0:`.
// ─────────────────────────────────────────────────────────────────────────────

const CFA_SENSOR_GROUP: &[(&str, &str)] = &[
    ("CANON-EOS-5D-MARK-III/5G4A9394.DNG", "SubIFD"),
    ("NIKON-D300S/_DSC4615.DNG", "SubIFD"),
];

/// The colour tags read from `IFD0`, in the fixed order the fixture line and
/// [`colour_from_fields`] share. Two sensor-group columns (`BlackLevel`,
/// `WhiteLevel`) follow them — nine columns total.
const IFD0_COLOUR_TAGS: &[&str] = &[
    "ColorMatrix1",
    "ColorMatrix2",
    "ForwardMatrix1",
    "ForwardMatrix2",
    "AsShotNeutral",
    "CalibrationIlluminant1",
    "CalibrationIlluminant2",
];

const COLUMN_COUNT: usize = IFD0_COLOUR_TAGS.len() + 2;

// ─────────────────────────────────────────────────────────────────────────────
// The tool reading, typed to compare against `Sensor`
// ─────────────────────────────────────────────────────────────────────────────

/// Everything exiftool says about one file's colour metadata. The matrices keep
/// their raw decimal TOKENS (not parsed to `u32` — `tools::parse_field_values`
/// would panic on a signed decimal); the comparator parses each token to an
/// exact rational at compare time. `None` is exiftool's `-` (tag absent).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ColourReading {
    color_matrix_1: Option<Vec<String>>,
    color_matrix_2: Option<Vec<String>>,
    forward_matrix_1: Option<Vec<String>>,
    forward_matrix_2: Option<Vec<String>>,
    as_shot_neutral: Option<Vec<String>>,
    calibration_illuminant_1: Option<u32>,
    calibration_illuminant_2: Option<u32>,
    black_levels: Vec<u32>,
    white_levels: Vec<u32>,
}

/// Parse one `exiftool -T -n -s3` line (nine tab-separated columns, in
/// [`IFD0_COLOUR_TAGS`] order then `BlackLevel`, `WhiteLevel`) into a
/// [`ColourReading`]. Pure — shared by [`colour_reading`] (the live tool) and
/// the tier-A fixture control/red, so the red-proof exercises the real parse.
fn colour_from_fields(line: &str) -> Result<ColourReading, ToolError> {
    let line = line.trim_end_matches('\n');
    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() != COLUMN_COUNT {
        return Err(ToolError::Parse {
            tool: "exiftool",
            message: format!(
                "expected {COLUMN_COUNT} tab-separated colour columns, got {}: {line:?}",
                cols.len()
            ),
        });
    }

    // A matrix/AsShotNeutral column: the raw decimal tokens, or `None` for `-`.
    let tokens = |raw: &str| -> Option<Vec<String>> {
        if raw == "-" {
            None
        } else {
            Some(
                raw.split(' ')
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
            )
        }
    };
    // A single integer column (the illuminants), or `None` for `-`.
    let scalar = |raw: &str| -> Result<Option<u32>, ToolError> {
        if raw == "-" {
            Ok(None)
        } else {
            raw.trim().parse().map(Some).map_err(|_| ToolError::Parse {
                tool: "exiftool",
                message: format!("illuminant {raw:?} is not an integer"),
            })
        }
    };
    // The per-channel level columns: every integer, empty for `-`.
    let levels = |raw: &str| -> Result<Vec<u32>, ToolError> {
        if raw == "-" {
            return Ok(Vec::new());
        }
        raw.split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<u32>().map_err(|_| ToolError::Parse {
                    tool: "exiftool",
                    message: format!("level {s:?} is not an integer"),
                })
            })
            .collect()
    };

    Ok(ColourReading {
        color_matrix_1: tokens(cols[0]),
        color_matrix_2: tokens(cols[1]),
        forward_matrix_1: tokens(cols[2]),
        forward_matrix_2: tokens(cols[3]),
        as_shot_neutral: tokens(cols[4]),
        calibration_illuminant_1: scalar(cols[5])?,
        calibration_illuminant_2: scalar(cols[6])?,
        black_levels: levels(cols[7])?,
        white_levels: levels(cols[8])?,
    })
}

/// Shell out: run `exiftool -T -n -s3` over the seven IFD0 colour tags plus the
/// sensor group's `BlackLevel`/`WhiteLevel`, and build a [`ColourReading`].
fn colour_reading(path: &Path, sensor_group: &str) -> Result<ColourReading, ToolError> {
    let mut args: Vec<String> = vec!["-T".into(), "-n".into(), "-s3".into()];
    for tag in IFD0_COLOUR_TAGS {
        args.push(format!("-IFD0:{tag}"));
    }
    args.push(format!("-{sensor_group}:BlackLevel"));
    args.push(format!("-{sensor_group}:WhiteLevel"));
    let output = tools::run_tool("exiftool", &args, path)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    colour_from_fields(&stdout)
}

// ─────────────────────────────────────────────────────────────────────────────
// The exact comparator — no float, no tolerance (AC6)
// ─────────────────────────────────────────────────────────────────────────────

/// A signed decimal token (`"-0.1413"`, `"1"`, `"0.606276"`) as the exact
/// rational `num / 10^frac_digits`. `None` for anything that is not a plain
/// signed decimal — so a surprise like exiftool printing `"inf"` is a loud
/// mismatch, not a silent pass. Never panics: a digit string too long for
/// `i64` or a power of ten that would overflow is `None`.
fn decimal_to_rational(tok: &str) -> Option<(i64, i64)> {
    let (neg, body) = match tok.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, tok.strip_prefix('+').unwrap_or(tok)),
    };
    let (int_part, frac_part) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if !int_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let magnitude: i64 = format!("{int_part}{frac_part}").parse().ok()?;
    let mut den: i64 = 1;
    for _ in 0..frac_part.len() {
        den = den.checked_mul(10)?;
    }
    Some((if neg { -magnitude } else { magnitude }, den))
}

/// Whether our stored [`Rational`] and the tool's `(num, den)` decimal are the
/// SAME exact value, by cross-multiplication in `i128` — `ours.num * theirs.den
/// == theirs.num * ours.den`. No float, no tolerance, and correct across a
/// non-reduced denominator (`1000000/1000000` equals `1`).
fn rationals_equal(ours: Rational, theirs: (i64, i64)) -> bool {
    i128::from(ours.num) * i128::from(theirs.1) == i128::from(theirs.0) * i128::from(ours.den)
}

// ─────────────────────────────────────────────────────────────────────────────
// The diff — one arm per field class, naming the field (and matrix index)
// ─────────────────────────────────────────────────────────────────────────────

fn compare_matrix(
    field: &'static str,
    ours: &Option<Vec<Rational>>,
    theirs: &Option<Vec<String>>,
) -> Option<Mismatch> {
    match (ours, theirs) {
        (None, None) => None,
        (Some(o), Some(t)) => {
            if o.len() != t.len() {
                return Some(Mismatch {
                    field,
                    ours: format!("{} element(s)", o.len()),
                    theirs: format!("{} element(s)", t.len()),
                });
            }
            for (i, (r, tok)) in o.iter().zip(t.iter()).enumerate() {
                let equal = decimal_to_rational(tok).is_some_and(|d| rationals_equal(*r, d));
                if !equal {
                    return Some(Mismatch {
                        field,
                        ours: format!("[{i}]={}/{}", r.num, r.den),
                        theirs: format!("[{i}]={tok}"),
                    });
                }
            }
            None
        }
        (o, t) => Some(Mismatch {
            field,
            ours: format!("{o:?}"),
            theirs: format!("{t:?}"),
        }),
    }
}

fn compare_scalar(field: &'static str, ours: Option<u32>, theirs: Option<u32>) -> Option<Mismatch> {
    if ours == theirs {
        None
    } else {
        Some(Mismatch {
            field,
            ours: format!("{ours:?}"),
            theirs: format!("{theirs:?}"),
        })
    }
}

fn compare_levels(field: &'static str, ours: &[u32], theirs: &[u32]) -> Option<Mismatch> {
    if ours == theirs {
        None
    } else {
        Some(Mismatch {
            field,
            ours: format!("{ours:?}"),
            theirs: format!("{theirs:?}"),
        })
    }
}

/// Diff a `Sensor`'s colour fields against exiftool's reading, field by field.
/// Reuses `tools::Mismatch` so a disagreement names the field (and, for a
/// matrix, the element index), ours and theirs.
fn diff_colour(sensor: &Sensor, reading: &ColourReading) -> Vec<Mismatch> {
    [
        compare_matrix(
            "ColorMatrix1",
            &sensor.color_matrix_1,
            &reading.color_matrix_1,
        ),
        compare_matrix(
            "ColorMatrix2",
            &sensor.color_matrix_2,
            &reading.color_matrix_2,
        ),
        compare_matrix(
            "ForwardMatrix1",
            &sensor.forward_matrix_1,
            &reading.forward_matrix_1,
        ),
        compare_matrix(
            "ForwardMatrix2",
            &sensor.forward_matrix_2,
            &reading.forward_matrix_2,
        ),
        compare_matrix(
            "AsShotNeutral",
            &sensor.as_shot_neutral,
            &reading.as_shot_neutral,
        ),
        compare_scalar(
            "CalibrationIlluminant1",
            sensor.calibration_illuminant_1,
            reading.calibration_illuminant_1,
        ),
        compare_scalar(
            "CalibrationIlluminant2",
            sensor.calibration_illuminant_2,
            reading.calibration_illuminant_2,
        ),
        compare_levels("BlackLevel", &sensor.black_levels, &reading.black_levels),
        compare_levels("WhiteLevel", &sensor.white_levels, &reading.white_levels),
    ]
    .into_iter()
    .flatten()
    .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// dnglab's per-channel levels — AC3's second tool. Reuses the mono oracle's
// `unique_array_str` + `parse_rational_u32`. `blacklevels.levels` are `N/D`
// rational strings (`"524032/256"`); `whitelevels` are bare integers.
// ─────────────────────────────────────────────────────────────────────────────

fn dnglab_black_levels(json: &str) -> Result<Vec<u32>, ToolError> {
    let inner = tools::unique_array_str(json, "levels")?;
    inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| tools::parse_rational_u32("blacklevels.levels", s))
        .collect()
}

fn dnglab_white_levels(json: &str) -> Result<Vec<u32>, ToolError> {
    let inner = tools::unique_array_str(json, "whitelevels")?;
    inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u32>().map_err(|_| ToolError::Parse {
                tool: "dnglab",
                message: format!("whitelevels element {s:?} is not an integer"),
            })
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

fn sensor_group(manifest_path: &str) -> &'static str {
    CFA_SENSOR_GROUP
        .iter()
        .find(|(p, _)| *p == manifest_path)
        .unwrap_or_else(|| {
            panic!(
                "no exiftool sensor-group mapping for {manifest_path} — add one, MEASURED \
                 (`exiftool -n -G1 <file>`), before trusting a comparison"
            )
        })
        .1
}

/// Read and parse one Bayer file's sensor plane, or `None` (skip already
/// announced by `CorpusFile::require`) when it is absent.
fn colour_sensor_at(root: &CorpusRoot, manifest_path: &str) -> Option<(PathBuf, Sensor)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let entry = manifest
        .get(manifest_path)
        .unwrap_or_else(|| panic!("{manifest_path} must be in the manifest"));
    let path = entry.require(root)?;
    let data = std::fs::read(&path).expect("read corpus file");
    let sensor = Container::parse(&data)
        .unwrap_or_else(|e| panic!("{manifest_path}: container did not parse: {e}"))
        .sensor()
        .unwrap_or_else(|e| panic!("{manifest_path}: no sensor plane: {e}"));
    Some((path, sensor))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — the matrices parse exactly, both files (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn colour_matrices_match_exiftool_on_both_bayer_files() {
    if !tools::exiftool_available() {
        eprintln!("SKIP colour_matrices_match_exiftool_on_both_bayer_files — exiftool not on PATH");
        return;
    }
    let root = CorpusRoot::resolve();
    let mut checked = 0;
    for (manifest_path, _) in CFA_SENSOR_GROUP {
        let Some((path, sensor)) = colour_sensor_at(&root, manifest_path) else {
            continue;
        };
        let reading = colour_reading(&path, sensor_group(manifest_path))
            .unwrap_or_else(|e| panic!("{manifest_path}: exiftool failed: {e}"));
        let mismatches = diff_colour(&sensor, &reading);
        assert!(
            mismatches.is_empty(),
            "{manifest_path}: {} colour field(s) disagree with exiftool:\n{}",
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
        "colour_matrices_match_exiftool_on_both_bayer_files: {checked}/2 Bayer files present"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1/AC4 — ForwardMatrix present on Canon, absent on Nikon (the contrast)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn forward_matrix_is_present_on_canon_and_absent_on_nikon() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP forward_matrix_is_present_on_canon_and_absent_on_nikon — exiftool not on PATH"
        );
        return;
    }
    let root = CorpusRoot::resolve();

    if let Some((path, sensor)) = colour_sensor_at(&root, "CANON-EOS-5D-MARK-III/5G4A9394.DNG") {
        assert!(
            sensor.forward_matrix_1.is_some() && sensor.forward_matrix_2.is_some(),
            "Canon must carry ForwardMatrix1/2, got {:?} / {:?}",
            sensor.forward_matrix_1,
            sensor.forward_matrix_2
        );
        let reading = colour_reading(&path, "SubIFD").expect("Canon exiftool");
        assert!(
            reading.forward_matrix_1.is_some(),
            "exiftool must confirm the Canon carries ForwardMatrix1"
        );
        assert!(
            diff_colour(&sensor, &reading).is_empty(),
            "Canon colour diff must be clean"
        );
    }

    if let Some((path, sensor)) = colour_sensor_at(&root, "NIKON-D300S/_DSC4615.DNG") {
        assert_eq!(
            sensor.forward_matrix_1, None,
            "Nikon writes no ForwardMatrix1"
        );
        assert_eq!(
            sensor.forward_matrix_2, None,
            "Nikon writes no ForwardMatrix2"
        );
        // Absent is not malformed (DEC-015, AC4).
        use irradiance::ifd::{TAG_FORWARD_MATRIX_1, TAG_FORWARD_MATRIX_2};
        assert!(!sensor.malformed_tags.contains(&TAG_FORWARD_MATRIX_1));
        assert!(!sensor.malformed_tags.contains(&TAG_FORWARD_MATRIX_2));
        let reading = colour_reading(&path, "SubIFD").expect("Nikon exiftool");
        assert_eq!(
            reading.forward_matrix_1, None,
            "exiftool must confirm the Nikon's ForwardMatrix1 is absent (-)"
        );
        assert!(
            diff_colour(&sensor, &reading).is_empty(),
            "Nikon colour diff must be clean (absent ForwardMatrix agrees)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — illuminants + AsShotNeutral (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn illuminants_and_as_shot_neutral_match_exiftool() {
    if !tools::exiftool_available() {
        eprintln!("SKIP illuminants_and_as_shot_neutral_match_exiftool — exiftool not on PATH");
        return;
    }
    let root = CorpusRoot::resolve();
    let mut checked = 0;
    for (manifest_path, group) in CFA_SENSOR_GROUP {
        let Some((path, sensor)) = colour_sensor_at(&root, manifest_path) else {
            continue;
        };
        // Both corpus files: illuminant A (17) and D65 (21).
        assert_eq!(
            sensor.calibration_illuminant_1,
            Some(17),
            "{manifest_path}: CalibrationIlluminant1"
        );
        assert_eq!(
            sensor.calibration_illuminant_2,
            Some(21),
            "{manifest_path}: CalibrationIlluminant2"
        );
        let reading = colour_reading(&path, group)
            .unwrap_or_else(|e| panic!("{manifest_path}: exiftool failed: {e}"));
        for field in [
            "CalibrationIlluminant1",
            "CalibrationIlluminant2",
            "AsShotNeutral",
        ] {
            assert!(
                !diff_colour(&sensor, &reading)
                    .iter()
                    .any(|m| m.field == field),
                "{manifest_path}: {field} disagrees with exiftool: {:?}",
                diff_colour(&sensor, &reading)
            );
        }
        checked += 1;
    }
    eprintln!("illuminants_and_as_shot_neutral_match_exiftool: {checked}/2 Bayer files present");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — per-channel levels, cross-checked by BOTH exiftool and dnglab (tier B)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn per_channel_levels_agree_with_exiftool_and_dnglab() {
    let exiftool = tools::exiftool_available();
    let dnglab = tools::dnglab_available();
    if !exiftool && !dnglab {
        eprintln!(
            "SKIP per_channel_levels_agree_with_exiftool_and_dnglab — neither exiftool nor dnglab \
             on PATH"
        );
        return;
    }
    let root = CorpusRoot::resolve();
    let mut checked = 0;
    for (manifest_path, group) in CFA_SENSOR_GROUP {
        let Some((path, sensor)) = colour_sensor_at(&root, manifest_path) else {
            continue;
        };

        if exiftool {
            let reading = colour_reading(&path, group)
                .unwrap_or_else(|e| panic!("{manifest_path}: exiftool failed: {e}"));
            assert_eq!(
                sensor.black_levels, reading.black_levels,
                "{manifest_path}: BlackLevel vs exiftool"
            );
            assert_eq!(
                sensor.white_levels, reading.white_levels,
                "{manifest_path}: WhiteLevel vs exiftool"
            );
        } else {
            eprintln!("  (exiftool half of {manifest_path} skipped — exiftool not on PATH)");
        }

        if dnglab {
            let run = tools::dnglab_analyze_meta(&path)
                .unwrap_or_else(|e| panic!("{manifest_path}: dnglab failed: {e}"));
            assert!(
                run.success,
                "{manifest_path}: dnglab analyze --meta --json exited unsuccessfully: {}",
                run.stderr
            );
            let black = dnglab_black_levels(&run.stdout)
                .unwrap_or_else(|e| panic!("{manifest_path}: dnglab blacklevels: {e}"));
            let white = dnglab_white_levels(&run.stdout)
                .unwrap_or_else(|e| panic!("{manifest_path}: dnglab whitelevels: {e}"));
            assert_eq!(
                sensor.black_levels, black,
                "{manifest_path}: BlackLevel vs dnglab blacklevels.levels"
            );
            assert_eq!(
                sensor.white_levels, white,
                "{manifest_path}: WhiteLevel vs dnglab whitelevels"
            );
        } else {
            eprintln!("  (dnglab half of {manifest_path} skipped — dnglab not on PATH)");
        }
        checked += 1;
    }
    eprintln!("per_channel_levels_agree_with_exiftool_and_dnglab: {checked}/2 Bayer files present");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the comparator's own red and green, tier A (no tool, no corpus)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cross_multiply_is_exact_across_a_non_reduced_denominator() {
    // `7234/10000` (ours) vs exiftool's reduced decimal `0.7234`: equal.
    let ours = Rational {
        num: 7234,
        den: 10000,
    };
    let theirs = decimal_to_rational("0.7234").expect("plain decimal");
    assert!(rationals_equal(ours, theirs));

    // The non-reduced case the design turns on: AsShotNeutral's middle element
    // is stored `1000000/1000000`, printed `1`. Cross-multiply must still agree.
    let one = Rational {
        num: 1000000,
        den: 1000000,
    };
    assert!(rationals_equal(
        one,
        decimal_to_rational("1").expect("integer")
    ));

    // A signed, fractional matrix entry: `-6/100` vs `-0.06`.
    let neg = Rational { num: -6, den: 100 };
    assert!(rationals_equal(
        neg,
        decimal_to_rational("-0.06").expect("signed")
    ));
}

#[test]
fn a_rounded_matrix_value_goes_red() {
    // `7234/10000` is NOT `0.7235` — a value exiftool had to round would go red,
    // which is exactly what the no-tolerance comparator guarantees.
    let ours = Rational {
        num: 7234,
        den: 10000,
    };
    let theirs = decimal_to_rational("0.7235").expect("plain decimal");
    assert!(!rationals_equal(ours, theirs));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the oracle's red-proof with a control, tier A. A committed sample of
// exiftool's real Canon colour output, replayed through the SAME
// `colour_from_fields` + `diff_colour` a live run uses.
// ─────────────────────────────────────────────────────────────────────────────

const COLOUR_FIXTURE: &str = include_str!("oracle-fixtures/exiftool-5g4a9394-colour.txt");

/// A matrix column as `n/10000` rationals — the exact form the Canon's DNG
/// stores (confirmed by the live reader: `irr ifd`).
fn matrix(nums: [i32; 9]) -> Option<Vec<Rational>> {
    Some(
        nums.iter()
            .map(|&num| Rational { num, den: 10000 })
            .collect(),
    )
}

/// The Canon 5D-III's real colour metadata, hand-built so the fixture control
/// is an INDEPENDENT reading of the committed exiftool line, not a tautology.
/// `diff_colour` reads only the colour fields and per-channel levels; the
/// structural fields carry the plane's real values (measured) for honesty but
/// are not compared here.
fn colour_fixture_sensor() -> Sensor {
    Sensor {
        ifd_index: 1,
        width: 5920,
        height: 3950,
        bits_per_sample: 16,
        samples_per_pixel: 1,
        photometric: 32803,
        compression: Compression::Uncompressed,
        rows_per_strip: Some(3950),
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: Some(2047),
        white_level: Some(15000),
        black_level_repeat_dim: Some([2, 2]),
        active_area: None,
        default_crop_origin: None,
        default_crop_size: None,
        orientation: Some(1),
        opcode_lists: [false, false, false],
        opcode_list_1: None,
        opcode_list_3: None,
        color_filter: Some(Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
            layout: 1,
        }),
        color_matrix_1: matrix([7234, -1413, -600, -3631, 11150, 2850, -382, 1335, 6437]),
        color_matrix_2: matrix([6722, -635, -963, -4287, 12460, 2028, -908, 2162, 5668]),
        forward_matrix_1: matrix([7868, 92, 1683, 2291, 8615, -906, 27, -4752, 12976]),
        forward_matrix_2: matrix([7637, 805, 1201, 2649, 9179, -1828, 137, -2456, 10570]),
        as_shot_neutral: Some(vec![
            Rational {
                num: 606276,
                den: 1000000,
            },
            Rational {
                num: 1000000,
                den: 1000000,
            },
            Rational {
                num: 461885,
                den: 1000000,
            },
        ]),
        calibration_illuminant_1: Some(17),
        calibration_illuminant_2: Some(21),
        black_levels: vec![2047, 2047, 2047, 2048],
        white_levels: vec![15000],
        malformed_tags: vec![],
    }
}

#[test]
fn colour_oracle_is_clean_on_an_unmodified_reading() {
    let reading = colour_from_fields(COLOUR_FIXTURE).expect("colour fixture line parses");
    let mismatches = diff_colour(&colour_fixture_sensor(), &reading);
    assert!(
        mismatches.is_empty(),
        "an honest colour reading must diff clean, got {mismatches:?}"
    );
}

#[test]
fn colour_oracle_names_the_one_matrix_element_that_was_perturbed() {
    let reading = colour_from_fields(COLOUR_FIXTURE).expect("colour fixture line parses");
    let mut sensor = colour_fixture_sensor();
    // Perturb exactly one element of one matrix: ColorMatrix1[4] = 11150/10000
    // -> 11151/10000 (0.11% off), the kind of single-element drift the oracle
    // exists to catch.
    sensor.color_matrix_1 = matrix([7234, -1413, -600, -3631, 11151, 2850, -382, 1335, 6437]);

    let mismatches = diff_colour(&sensor, &reading);
    assert_eq!(
        mismatches.len(),
        1,
        "exactly one field must disagree, got {mismatches:?}"
    );
    assert_eq!(mismatches[0].field, "ColorMatrix1");
    assert!(
        mismatches[0].ours.contains("[4]"),
        "the mismatch must name the perturbed element index, got {:?}",
        mismatches[0]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — a present-but-malformed ColorMatrix1 costs the field, not the file.
// (The reader's DEC-012 behaviour is unit-tested in `src/ifd.rs`; this is the
// oracle-side check that a dropped field reads `None` + recorded, tier A.)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_dropped_matrix_reads_none_and_is_recorded_not_errored() {
    use irradiance::ifd::TAG_COLOR_MATRIX_1;
    // A Sensor as if ColorMatrix1 had been present-but-wrong-shaped: dropped to
    // None, its tag recorded, every other field intact. diff_colour against a
    // reading that ALSO has ColorMatrix1 present must then disagree on exactly
    // that field — the oracle sees the drop, it is not silently absorbed.
    let mut sensor = colour_fixture_sensor();
    sensor.color_matrix_1 = None;
    sensor.malformed_tags = vec![TAG_COLOR_MATRIX_1];

    let reading = colour_from_fields(COLOUR_FIXTURE).expect("colour fixture line parses");
    let mismatches = diff_colour(&sensor, &reading);
    assert_eq!(mismatches.len(), 1, "got {mismatches:?}");
    assert_eq!(mismatches[0].field, "ColorMatrix1");
    assert_eq!(mismatches[0].ours, "None");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — both Bayer files are COVERED here (not deferred). The scalar mono
// oracle names them covered-by-the-colour-oracle; this oracle iterates them.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn both_bayer_files_are_in_this_oracles_corpus_and_the_manifest() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    for (manifest_path, _) in CFA_SENSOR_GROUP {
        assert!(
            manifest.get(manifest_path).is_some(),
            "{manifest_path} must be registered in the manifest (SPEC-027) for this oracle to \
             cover it"
        );
    }
    assert_eq!(
        CFA_SENSOR_GROUP.len(),
        2,
        "the colour oracle covers exactly the two Bayer files"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — a missing tool skips loudly, naming it (the SPEC-005 idiom)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_missing_tool_skips_loudly_naming_it() {
    const ABSENT: &str = "irradiance-colour-oracle-tool-that-does-not-exist";
    let err = tools::run_tool(ABSENT, &[], Path::new("/nonexistent"))
        .expect_err("a binary that cannot exist on any PATH must be a loud, typed Err");
    match err {
        ToolError::NotOnPath(name) => {
            assert_eq!(name, ABSENT, "the error must name the missing tool")
        }
        other => panic!("expected ToolError::NotOnPath, got {other:?}"),
    }
}
