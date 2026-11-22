//! `SPEC-085` AC4 — the colour transform against Adobe's DNG SDK 1.7.1.
//!
//! For each held colour file, `scripts/sdk-colour.sh` writes the SDK's own
//! camera→XYZ(D50) matrix (`dng_color_spec::CameraToPCS()` at the SDK's
//! `NeutralToXY(AsShotNeutral)` white, the default profile). This lane runs six
//! probe colours through the public [`to_srgb_linear_into`] and compares the 18
//! output codes with `C · SDK · v`, where `C` is this file's own transcription of
//! the XYZ(D50)→linear-sRGB constant.
//!
//! **Pre-registered** at SPEC-085's design, from the probe (DEC-076): max |Δ| ≤
//! 12 codes over the 18 values, on every file. The probe measured 6 (Pixel 7 Pro),
//! 7 (Pixel 3a), 1 (Pixel 4a), 2 (Canon 5D Mark III), 5 (Nikon D300S). A miss is
//! reported, never widened. The residual is McCamy against the SDK's Robertson
//! CCT, the SDK's `NormalizeForwardMatrix` and its clipped Bradford, none of
//! which DEC-076 adopts.
//!
//! **Independence (AGENTS.md §16 rule 5).** The expected values come from the
//! SDK's matrix and this file's own constants, never from `src/color.rs`.
//!
//! **Set means must run (`SPEC-079`).** With `$IRRADIANCE_SDK_COLOUR_DIR` set, a
//! missing matrix file or a missing corpus file fails; unset (always in CI), the
//! lane prints a visible SKIP. The SDK is a local reference only: never
//! vendored, built in CI or linked into the crate (`DEC-069` §6).

#[path = "support/corpus.rs"]
mod corpus;

use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::color::to_srgb_linear_into;
use irradiance::ifd::{Container, Sensor};

/// The environment variable naming the directory `scripts/sdk-colour.sh` wrote.
const SDK_COLOUR_ENV: &str = "IRRADIANCE_SDK_COLOUR_DIR";

/// AC4's pre-registered bound, in u16 output codes.
const MAX_CODES: u16 = 12;

/// `(manifest path, SDK output stem)` for the five held colour files.
const FILES: [(&str, &str); 5] = [
    (
        "GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng",
        "GOOGLE-PIXEL-7-PRO",
    ),
    ("GOOGLE-PIXEL-3A/IMG_20190918_164153.dng", "GOOGLE-PIXEL-3A"),
    (
        "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng",
        "GOOGLE-PIXEL-4A",
    ),
    (
        "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
        "CANON-EOS-5D-MARK-III",
    ),
    ("NIKON-D300S/_DSC4615.DNG", "NIKON-D300S"),
];

/// The six tilts applied to `AsShotNeutral` (AC4): neutral, then five colours.
const TILTS: [[f64; 3]; 6] = [
    [1.0, 1.0, 1.0],
    [1.3, 1.0, 0.8],
    [0.8, 1.0, 1.3],
    [1.0, 1.25, 1.0],
    [1.2, 0.9, 1.1],
    [0.9, 1.1, 0.85],
];

/// Linear sRGB ← CIE XYZ (D65), IEC 61966-2-1, transcribed here.
const SRGB_FROM_XYZ_D65: [[f64; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

/// Bradford chromatic adaptation, CIE XYZ (D50) → CIE XYZ (D65), transcribed here.
const BRADFORD_D50_TO_D65: [[f64; 3]; 3] = [
    [0.9555766, -0.0230393, 0.0631636],
    [-0.0282895, 1.0099416, 0.0210077],
    [0.0122982, -0.0204830, 1.3299098],
];

fn mm(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// One SDK output file, parsed.
#[derive(Debug, PartialEq)]
struct SdkColour {
    white_xy: [f64; 2],
    neutral: [f64; 3],
    cam2pcs: [[f64; 3]; 3],
}

/// Parse `sdk-camera-to-pcs`'s five lines: `white_xy`, `neutral`, three
/// `cam2pcs` rows, in that order.
fn parse_sdk(text: &str) -> Result<SdkColour, String> {
    let mut lines = text.lines();
    let mut numbers = |key: &str, n: usize| -> Result<Vec<f64>, String> {
        let line = lines.next().ok_or_else(|| format!("missing {key} line"))?;
        let mut words = line.split_whitespace();
        if words.next() != Some(key) {
            return Err(format!("expected {key}, got {line:?}"));
        }
        let values: Vec<f64> = words
            .map(|w| w.parse::<f64>().map_err(|e| format!("{key}: {w:?}: {e}")))
            .collect::<Result<_, _>>()?;
        if values.len() != n {
            return Err(format!("{key}: {} values, expected {n}", values.len()));
        }
        Ok(values)
    };
    let w = numbers("white_xy", 2)?;
    let n = numbers("neutral", 3)?;
    let mut cam2pcs = [[0.0f64; 3]; 3];
    for row in &mut cam2pcs {
        let r = numbers("cam2pcs", 3)?;
        *row = [r[0], r[1], r[2]];
    }
    if lines.next().is_some() {
        return Err("more than five lines".to_string());
    }
    Ok(SdkColour {
        white_xy: [w[0], w[1]],
        neutral: [n[0], n[1], n[2]],
        cam2pcs,
    })
}

/// The 18 probe values (AC4): `round(0.25 · 65535 · AsShotNeutral_i · t_i)`.
fn probe_camera(neutral: [f64; 3]) -> Vec<u16> {
    let mut out = Vec::with_capacity(18);
    for tilt in TILTS {
        for ch in 0..3 {
            let v = (0.25 * 65535.0 * neutral[ch] * tilt[ch]).round();
            assert!((0.0..=65535.0).contains(&v), "probe value {v} out of range");
            out.push(v as u16);
        }
    }
    out
}

/// The expected codes: `C · SDK · v`, clamped and rounded half up.
fn expected_codes(sdk: &SdkColour, camera: &[u16]) -> Vec<u16> {
    let m = mm(mm(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65), sdk.cam2pcs);
    let mut out = Vec::with_capacity(camera.len());
    for &[r, g, b] in camera.as_chunks::<3>().0 {
        let v = [f64::from(r), f64::from(g), f64::from(b)];
        for row in m {
            let y = row[0] * v[0] + row[1] * v[1] + row[2] * v[2];
            out.push((y.clamp(0.0, 65535.0) + 0.5).floor() as u16);
        }
    }
    out
}

/// Ours: the public transform on a clone of the file's sensor sized to exactly
/// the six probe pixels.
fn our_codes(sensor: &Sensor, camera: &[u16]) -> Vec<u16> {
    let mut s = sensor.clone();
    s.width = 6;
    s.height = 1;
    let mut dst = vec![0u16; camera.len()];
    to_srgb_linear_into(&s, camera, &mut dst).expect("the colour transform builds");
    dst
}

/// The SDK output directory, or `None` (the visible skip) when the variable is
/// unset.
fn sdk_dir() -> Option<PathBuf> {
    match std::env::var_os(SDK_COLOUR_ENV) {
        Some(d) if !d.is_empty() => Some(PathBuf::from(d)),
        _ => {
            eprintln!(
                "SKIP SDK colour cross-check — ${SDK_COLOUR_ENV} unset \
                 (generate it with scripts/sdk-colour.sh)"
            );
            None
        }
    }
}

/// The held file's sensor. The SDK variable is set, so a missing file fails.
fn sdk_corpus_sensor(path: &str) -> Sensor {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let resolved = entry.resolve(&CorpusRoot::resolve());
    assert!(
        resolved.is_file(),
        "{path}: MISSING at {} — ${SDK_COLOUR_ENV} is set, so the SDK colour cross-check \
         must run: point $IRRADIANCE_CORPUS_DIR at the corpus, or unset ${SDK_COLOUR_ENV}",
        resolved.display()
    );
    let bytes = std::fs::read(&resolved).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    Container::parse(&bytes)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .sensor()
        .unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn read_sdk(dir: &Path, stem: &str) -> SdkColour {
    let file = dir.join(format!("{stem}-camera-to-pcs.txt"));
    let text = std::fs::read_to_string(&file).unwrap_or_else(|e| {
        panic!(
            "{}: {e} — ${SDK_COLOUR_ENV} is set, so every matrix file must exist \
             (run scripts/sdk-colour.sh)",
            file.display()
        )
    });
    parse_sdk(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
}

/// The signed deltas ours − expected, and their largest magnitude.
fn deltas(ours: &[u16], expected: &[u16]) -> (Vec<i32>, u16) {
    let d: Vec<i32> = ours
        .iter()
        .zip(expected)
        .map(|(&a, &b)| i32::from(a) - i32::from(b))
        .collect();
    let max = ours
        .iter()
        .zip(expected)
        .map(|(&a, &b)| a.abs_diff(b))
        .max()
        .unwrap_or(0);
    (d, max)
}

#[test]
fn the_colour_transform_matches_the_sdk_within_the_pre_registered_bound() {
    let Some(dir) = sdk_dir() else {
        return;
    };
    let mut misses = Vec::new();
    for (path, stem) in FILES {
        let sensor = sdk_corpus_sensor(path);
        let sdk = read_sdk(&dir, stem);
        // The SDK's neutral is this file's AsShotNeutral: the matrix belongs to
        // this file.
        let asn: Vec<f64> = sensor
            .as_shot_neutral
            .as_ref()
            .unwrap_or_else(|| panic!("{path}: no AsShotNeutral"))
            .iter()
            .map(|r| f64::from(r.num) / f64::from(r.den))
            .collect();
        for (ch, (&ours, &theirs)) in asn.iter().zip(&sdk.neutral).enumerate() {
            assert!(
                (ours - theirs).abs() < 1e-9,
                "{stem}: AsShotNeutral[{ch}] {ours} but the SDK file says {theirs}"
            );
        }

        let camera = probe_camera(sdk.neutral);
        let expected = expected_codes(&sdk, &camera);
        let ours = our_codes(&sensor, &camera);
        let (d, max) = deltas(&ours, &expected);
        eprintln!(
            "AC4 {stem}: max |delta| {max} codes; white xy {:.6} {:.6}; deltas {d:?}",
            sdk.white_xy[0], sdk.white_xy[1]
        );
        if max > MAX_CODES {
            misses.push(format!("{stem}: {max} codes (deltas {d:?})"));
        }
    }
    assert!(
        misses.is_empty(),
        "AC4: over the pre-registered {MAX_CODES} codes — report, never widen:\n{}",
        misses.join("\n")
    );
}

// ── Tier A: the lane's own parts ────────────────────────────────────────────

const SAMPLE: &str = "white_xy 0.319745165568 0.333330981773\n\
                      neutral 0.642636000000 1.000000000000 0.559075000000\n\
                      cam2pcs 0.689884684255 0.377995488580 0.255699911568\n\
                      cam2pcs 0.2 0.7 0.1\n\
                      cam2pcs 0.0 0.1 0.7\n";

#[test]
fn the_sdk_file_parses_and_a_malformed_one_is_refused() {
    let parsed = parse_sdk(SAMPLE).expect("sample parses");
    assert_eq!(parsed.white_xy, [0.319745165568, 0.333330981773]);
    assert_eq!(
        parsed.cam2pcs[0],
        [0.689884684255, 0.377995488580, 0.255699911568]
    );
    for (what, text) in [
        (
            "four lines",
            SAMPLE.lines().take(4).collect::<Vec<_>>().join("\n"),
        ),
        ("six lines", format!("{SAMPLE}cam2pcs 1 2 3\n")),
        (
            "a short row",
            SAMPLE.replace("cam2pcs 0.2 0.7 0.1", "cam2pcs 0.2 0.7"),
        ),
        ("a bad number", SAMPLE.replace("0.2 0.7 0.1", "0.2 x 0.1")),
        ("a wrong key", SAMPLE.replace("neutral", "neutrals")),
    ] {
        assert!(parse_sdk(&text).is_err(), "{what} must be refused");
    }
}

#[test]
fn the_comparison_goes_red_on_a_perturbed_sdk_matrix() {
    // Control: an SDK matrix compared with the codes it itself produces is 0
    // codes off. Red: one entry perturbed by 0.01 moves a code past the bound.
    let sdk = parse_sdk(SAMPLE).expect("sample parses");
    let camera = probe_camera(sdk.neutral);
    let honest = expected_codes(&sdk, &camera);
    assert_eq!(deltas(&honest, &expected_codes(&sdk, &camera)).1, 0);
    let mut perturbed = parse_sdk(SAMPLE).expect("sample parses");
    perturbed.cam2pcs[0][0] += 0.01;
    let max = deltas(&honest, &expected_codes(&perturbed, &camera)).1;
    assert!(
        max > MAX_CODES,
        "a 0.01 perturbation moved only {max} codes"
    );
}
