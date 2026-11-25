//! `SPEC-061` tier B — the public DNG writer on the held monochrome files
//! (`$IRRADIANCE_CORPUS_DIR`; each file skips loudly, by name, when absent).
//!
//! - Oracle 4: every framing of every held mono file round-trips through
//!   irradiance's reader, and develops exactly like the source: the
//!   StandardCrop output like `develop_into(source)`, the FullSensor and
//!   FullActiveArea outputs like the source's `Framing::FullActiveArea`
//!   develop. A refusal on one side must be the same refusal on the other.
//! - Oracle 5: DEC-073's red-proof on the Q2M. A test-writer file cut to the
//!   standard crop with the warp verbatim develops differently; the negative
//!   control (the public writer's StandardCrop output) is checked first.
//! - Oracle 6: dnglab's raw checksum (tool-gated).
//! - Oracle 7: exiftool reads the written tags back (tool-gated).
//! - Oracle 8: Adobe `dng_validate` 1.7.1 (`DEC-069` §6), gated on
//!   `$IRRADIANCE_DNG_VALIDATE` naming the binary. Every output validates with
//!   no `Warning` or `Error` line; on the Q2M the StandardCrop render equals
//!   the source's, and the FullSensor render equals the FullActiveArea
//!   output's (both frame the whole active area by §3's table, so neither can
//!   equal the source's cropped render; see the SPEC-061 build handback). Unset: a SKIP line (also
//!   printed by `examples/corpus-status.rs`). Set but unusable: a failure.
//!   Local only, never CI; its outputs go to the OS temp dir and are deleted.
//!
//! `SPEC-081` adds:
//!
//! - Oracle 3 (`exiftool_reads_the_block_and_the_new_tags`, tool-gated): a
//!   block from `tests/support/exif_block.rs`, placed by the writer, reads
//!   back through exiftool value for value, with the eight descriptive tags,
//!   the four hints and an XMP property. Synthetic plane; no corpus needed.
//! - Oracle 4 (`red_proof_a_block_built_for_another_base_is_caught`): the
//!   same comparison fails for a block built for base 10, after its negative
//!   control passes.
//! - Oracle 5 (`dng_validate_accepts_metadata_and_the_k3_renders_like_its_source`,
//!   env-gated like oracle 8): the K-3 III and Q2M with their own descriptive
//!   tags and hints, the oracle-3 block and an XMP packet validate with no
//!   warnings; the K-3 III renders like its source, and differently without
//!   `BaselineExposure` (the hints' red-proof).
//!
//! `SPEC-082` adds:
//!
//! - Oracle 6 (`exiftool_extracts_the_preview_byte_for_byte`, tool-gated):
//!   `exiftool -b -SubIFD:PreviewImage` returns the caller's JPEG exactly, and
//!   the preview IFD's tags read back as written. Synthetic plane.
//! - Oracle 7 (`dng_validate_accepts_a_written_preview`, env-gated like
//!   oracle 8): the K-3 III and Q2M at StandardCrop with every SPEC-081 field
//!   and their own largest JPEG preview validate with no warnings and render
//!   exactly as without the preview; so does the K-3 III with the committed
//!   gray fixture as a Gray preview.
//!
//! Every written file goes to the OS temp dir, never the repo.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_write_oracle.rs"]
mod dng_write_oracle;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/exif_block.rs"]
mod exif_block;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/stage2_tiff.rs"]
mod stage2_tiff;
#[path = "support/tiff.rs"]
mod tiff;
#[path = "support/tools.rs"]
mod tools;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use corpus::{CorpusRoot, Manifest};
use dng_write_oracle::{check_round_trip, cut, expected, options_from_sensor, read_back};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::develop::{develop_with_into, output_dimensions_with, DevelopOptions};
use irradiance::dng::write::{
    write_to_vec, ExifBlock, Framing, JpegPreview, JpegPreviewColor, WriteOptions, EXIF_BLOCK_BASE,
};
use irradiance::ifd::{ActiveArea, Compression, Container, Rational, Sensor};

/// The four held monochrome DNGs (`SPEC-061` §7 item 4).
const FILES: [&str; 4] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "PENTAX-K3III-MONO/K3III.DNG",
    "LEICA-M-MONOCHROM/L1000622.DNG",
    "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
];
const Q2M: &str = "LEICA-Q2-MONO/L1021223.DNG";
const K3III: &str = "PENTAX-K3III-MONO/K3III.DNG";

const FRAMINGS: [Framing; 3] = [
    Framing::FullSensor,
    Framing::FullActiveArea,
    Framing::StandardCrop,
];

const MODEL: &str = "irradiance SPEC-061 corpus";
const BUDGET: usize = 1 << 28;

/// The environment variable naming Adobe's `dng_validate` binary.
const DNG_VALIDATE_ENV: &str = "IRRADIANCE_DNG_VALIDATE";

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// A held file's path, or `None` after a SKIP line naming it.
fn corpus_path(path: &str) -> Option<PathBuf> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    entry.require(&root)
}

/// The held file's bytes, sensor and unpacked plane.
fn source(path: &str) -> Option<(PathBuf, Sensor, Vec<u16>)> {
    let file_path = corpus_path(path)?;
    let bytes = std::fs::read(&file_path).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    let (sensor, plane) = read_back(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    Some((file_path, sensor, plane))
}

/// The source re-written by the public writer at `framing`.
fn written(path: &str, sensor: &Sensor, plane: &[u16], framing: Framing) -> Vec<u8> {
    let o = options_from_sensor(sensor, framing, MODEL);
    write_to_vec(plane, sensor.width, sensor.height, &o, BUDGET)
        .unwrap_or_else(|e| panic!("{path} {framing:?}: write: {e}"))
}

fn full_active_area() -> DevelopOptions {
    let mut o = DevelopOptions::default();
    o.framing = irradiance::develop::Framing::FullActiveArea;
    o
}

/// A mono develop, or the refusal's `Debug` (so two refusals compare by
/// variant and fields).
fn develop(sensor: &Sensor, plane: &[u16], options: &DevelopOptions) -> Result<Vec<u16>, String> {
    let (w, h) = output_dimensions_with(sensor, options).map_err(|e| format!("{e:?}"))?;
    let mut dst = vec![0u16; w as usize * h as usize];
    develop_with_into(sensor, plane, &mut dst, options).map_err(|e| format!("{e:?}"))?;
    Ok(dst)
}

fn develop_file(file: &[u8], options: &DevelopOptions) -> Result<Vec<u16>, String> {
    let (sensor, plane) = read_back(file).map_err(|e| format!("{e:?}"))?;
    develop(&sensor, &plane, options)
}

/// Equal results, or the same refusal; a short message otherwise.
fn same(label: &str, got: &Result<Vec<u16>, String>, want: &Result<Vec<u16>, String>) {
    match (got, want) {
        (Ok(g), Ok(w)) => {
            if g != w {
                let differ = g.iter().zip(w).filter(|(a, b)| a != b).count();
                panic!(
                    "{label}: develops differ ({} vs {} samples, {differ} differ)",
                    g.len(),
                    w.len()
                );
            }
        }
        (Err(g), Err(w)) => assert_eq!(g, w, "{label}: refused differently"),
        (g, w) => panic!(
            "{label}: one side refused: written {:?}, source {:?}",
            g.as_ref().map(Vec::len),
            w.as_ref().map(Vec::len)
        ),
    }
}

/// Write `bytes` to a fresh file under the OS temp dir. Never in the repo.
fn temp_file(bytes: &[u8], label: &str) -> PathBuf {
    let path = temp_path(label, "dng");
    std::fs::write(&path, bytes).expect("write temp DNG");
    path
}

fn temp_path(label: &str, ext: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    std::env::temp_dir().join(format!(
        "irradiance-spec061-{}-{n}-{label}.{ext}",
        std::process::id()
    ))
}

/// MD5 of samples as little-endian `u16`, lowercase hex.
fn md5_le(samples: &[u16]) -> String {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    md5::to_hex(&md5::hash(&bytes))
}

// ─────────────────────────────────────────────────────────────────────────────
// Oracle 4
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn held_mono_files_round_trip_at_every_framing() {
    for path in FILES {
        let Some((_, sensor, plane)) = source(path) else {
            continue; // SKIP announced by name by CorpusFile::require
        };
        let want_full = develop(&sensor, &plane, &full_active_area());
        for framing in FRAMINGS {
            let o = options_from_sensor(&sensor, framing, MODEL);
            let file = written(path, &sensor, &plane, framing);
            let want = expected(&plane, sensor.width, sensor.height, &o);
            if let Err(bad) = check_round_trip(&file, &want) {
                panic!("{path} {framing:?}: {bad:#?}");
            }
            if framing != Framing::StandardCrop {
                let got = develop_file(&file, &DevelopOptions::default());
                same(&format!("{path} {framing:?}"), &got, &want_full);
            }
        }
    }
}

#[test]
fn standard_crop_develops_like_the_source() {
    for path in FILES {
        let Some((_, sensor, plane)) = source(path) else {
            continue;
        };
        let want = develop(&sensor, &plane, &DevelopOptions::default());
        let file = written(path, &sensor, &plane, Framing::StandardCrop);
        let got = develop_file(&file, &DevelopOptions::default());
        same(&format!("{path} StandardCrop"), &got, &want);
        assert!(want.is_ok(), "{path}: the source develops (probe)");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Oracle 5 — DEC-073's red-proof
// ─────────────────────────────────────────────────────────────────────────────

/// The Q2M with its samples cut to the STANDARD CROP (not the active area) and
/// every opcode list verbatim, by the test writer: the shape DEC-073 forbids,
/// because `WarpRectilinear`'s optical centre is relative to the active area.
fn crop_cut_with_verbatim_warp(sensor: &Sensor, plane: &[u16]) -> Vec<u8> {
    let aa = sensor.active_area.expect("the Q2M has an ActiveArea");
    let origin = sensor.default_crop_origin.expect("crop origin");
    let size = sensor.default_crop_size.expect("crop size");
    let crop = ActiveArea {
        top: aa.top + origin.y,
        left: aa.left + origin.x,
        bottom: aa.top + origin.y + size.height,
        right: aa.left + origin.x + size.width,
    };
    let samples = cut(plane, sensor.width, crop);
    write_dng(&DngDescription {
        order: tiff::Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: size.width,
        height: size.height,
        bits_per_sample: 16,
        kind: PlaneKind::LinearRaw,
        organisation: Organisation::Strips {
            rows_per_strip: size.height,
        },
        payload: Payload::Samples(samples),
        meta: DngMeta {
            black_level: Some(sensor.black_levels.clone()),
            black_level_repeat_dim: sensor
                .black_level_repeat_dim
                .map(|[r, c]| [r as u16, c as u16]),
            white_level: Some(vec![sensor.white_level.unwrap_or(65535)]),
            orientation: sensor.orientation.map(|o| o as u16),
            dng_backward_version: Some([1, 3, 0, 0]),
            ..DngMeta::default()
        },
        opcode_lists: [
            sensor.opcode_list_1.clone(),
            sensor.opcode_list_2.clone(),
            sensor.opcode_list_3.clone(),
        ],
        extra_tags: Vec::new(),
        fault: None,
    })
    .expect("test writer")
}

#[test]
fn red_proof_a_crop_cut_with_verbatim_warp_develops_differently() {
    let Some((_, sensor, plane)) = source(Q2M) else {
        return;
    };
    assert!(sensor.opcode_list_3.is_some(), "the Q2M carries its warp");
    let want = develop(&sensor, &plane, &DevelopOptions::default()).expect("source develops");

    // Negative control first: the public writer's StandardCrop output matches.
    let honest = written(Q2M, &sensor, &plane, Framing::StandardCrop);
    same(
        "negative control",
        &develop_file(&honest, &DevelopOptions::default()),
        &Ok(want.clone()),
    );

    // The forbidden shape develops to the same size, but differently.
    let forbidden = crop_cut_with_verbatim_warp(&sensor, &plane);
    let got = develop_file(&forbidden, &DevelopOptions::default()).expect("it develops");
    assert_eq!(got.len(), want.len(), "same output size");
    let differ = got.iter().zip(&want).filter(|(a, b)| a != b).count();
    assert!(
        differ > want.len() / 2,
        "the crop-cut file must develop differently from the source: {differ} of {} differ",
        want.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Oracle 6 — dnglab
// ─────────────────────────────────────────────────────────────────────────────

/// `dnglab analyze --raw-checksum`, black-box: the one 32-hex token.
fn dnglab_raw_checksum(path: &Path) -> Result<String, String> {
    let out = tools::run_tool("dnglab", &["analyze".into(), "--raw-checksum".into()], path)
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        return Err(format!(
            "dnglab exited {:?}: {stdout}{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let hexes: Vec<&str> = stdout
        .split(|c: char| !c.is_ascii_hexdigit())
        .filter(|t| t.len() == 32)
        .collect();
    match hexes.as_slice() {
        [one] => Ok(one.to_ascii_lowercase()),
        _ => Err(format!("no unique 32-hex checksum in {stdout:?}")),
    }
}

#[test]
fn dnglab_checksums_match() {
    if !tools::dnglab_available() {
        eprintln!("SKIP dnglab_checksums_match: dnglab is not on PATH — oracle 6 did not run");
        return;
    }
    for path in FILES {
        let Some((file_path, sensor, plane)) = source(path) else {
            continue;
        };
        for framing in FRAMINGS {
            let file = written(path, &sensor, &plane, framing);
            let tmp = temp_file(&file, &format!("{path}-{framing:?}"));
            let got = dnglab_raw_checksum(&tmp);
            let _ = std::fs::remove_file(&tmp);
            let got = got.unwrap_or_else(|e| panic!("{path} {framing:?}: {e}"));
            let want = if framing == Framing::FullSensor {
                // The source's own checksum, computed by dnglab from the
                // source file: never through irradiance.
                dnglab_raw_checksum(&file_path).unwrap_or_else(|e| panic!("{path} source: {e}"))
            } else {
                let aa = sensor.active_area.unwrap_or(ActiveArea {
                    top: 0,
                    left: 0,
                    bottom: sensor.height,
                    right: sensor.width,
                });
                md5_le(&cut(&plane, sensor.width, aa))
            };
            assert_eq!(got, want, "{path} {framing:?}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Oracle 7 — exiftool
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn exiftool_reads_the_written_tags() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP exiftool_reads_the_written_tags: exiftool is not on PATH — oracle 7 did not run"
        );
        return;
    }
    for path in FILES {
        let Some((_, sensor, plane)) = source(path) else {
            continue;
        };
        for framing in FRAMINGS {
            let o = options_from_sensor(&sensor, framing, MODEL);
            let want = expected(&plane, sensor.width, sensor.height, &o);
            let file = written(path, &sensor, &plane, framing);
            let tmp = temp_file(&file, &format!("{path}-{framing:?}"));
            // -n numeric; -s3 bare values; one line per tag, absent as "-"
            // under -f (so ActiveArea's absence is a line, not a gap).
            let args: Vec<String> = [
                "-n",
                "-s3",
                "-f",
                "-IFD0:DNGVersion",
                "-IFD0:UniqueCameraModel",
                "-IFD0:BitsPerSample",
                "-IFD0:ImageWidth",
                "-IFD0:ActiveArea",
                "-IFD0:DefaultCropSize",
                "-IFD0:Orientation",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let out = tools::run_tool("exiftool", &args, &tmp).expect("exiftool");
            let _ = std::fs::remove_file(&tmp);
            let text = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = text.lines().collect();
            let active = want.active_area.map_or("-".to_string(), |a| {
                format!("{} {} {} {}", a.top, a.left, a.bottom, a.right)
            });
            let crop = format!("{} {}", want.crop_size.width, want.crop_size.height);
            let (width, orientation) = (want.width.to_string(), want.orientation.to_string());
            assert_eq!(
                lines,
                vec![
                    "1 4 0 0",
                    MODEL,
                    "16",
                    width.as_str(),
                    active.as_str(),
                    crop.as_str(),
                    orientation.as_str(),
                ],
                "{path} {framing:?}: exiftool said {text:?}"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Oracle 8 — Adobe dng_validate (env-gated, local only)
// ─────────────────────────────────────────────────────────────────────────────

/// The `dng_validate` binary, or `None` with a visible SKIP line. Set means
/// must run: a set variable that names nothing runnable fails here.
fn dng_validate() -> Option<PathBuf> {
    match std::env::var_os(DNG_VALIDATE_ENV) {
        Some(p) if !p.is_empty() => {
            let p = PathBuf::from(p);
            assert!(
                p.is_file(),
                "${DNG_VALIDATE_ENV} is set to {} but no such file exists — \
                 unset it to skip oracle 8",
                p.display()
            );
            Some(p)
        }
        _ => {
            eprintln!(
                "SKIP dng_validate_accepts_and_renders_like_the_source — ${DNG_VALIDATE_ENV} unset"
            );
            None
        }
    }
}

/// Run `dng_validate [args] file`; its whole output, or a failure. It exits 0
/// even with warnings, so the verdict is read from the text, not the code.
fn run_validate(bin: &Path, args: &[String], file: &Path) -> String {
    let out = std::process::Command::new(bin)
        .args(args)
        .arg(file)
        .output()
        .unwrap_or_else(|e| panic!("${DNG_VALIDATE_ENV} {} did not run: {e}", bin.display()));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "dng_validate exited {:?} on {}: {text}",
        out.status.code(),
        file.display()
    );
    text
}

/// The `Warning`/`Error` lines of a `dng_validate` run.
fn complaints(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| l.contains("Warning") || l.contains("Error"))
        .collect()
}

/// Validate `file` and render it with `-16 -tif`: its complaints and its
/// 3-sample render. The TIFF is read then deleted.
fn validate_and_render(bin: &Path, file: &Path, label: &str) -> (Vec<String>, stage2_tiff::Stage2) {
    let base = temp_path(label, "render");
    let text = run_validate(
        bin,
        &[
            "-16".into(),
            "-tif".into(),
            base.to_string_lossy().into_owned(),
        ],
        file,
    );
    let tif = PathBuf::from(format!("{}.tif", base.display()));
    let bytes = std::fs::read(&tif)
        .unwrap_or_else(|e| panic!("{label}: dng_validate wrote no {}: {e}", tif.display()));
    let _ = std::fs::remove_file(&tif);
    let render = stage2_tiff::read_rgb(&bytes)
        .unwrap_or_else(|e| panic!("{label}: unreadable -tif render: {e}"));
    (
        complaints(&text).into_iter().map(str::to_owned).collect(),
        render,
    )
}

#[test]
fn dng_validate_accepts_and_renders_like_the_source() {
    let Some(bin) = dng_validate() else {
        return;
    };
    let mut report = Vec::new();
    for path in FILES {
        let Some((file_path, sensor, plane)) = source(path) else {
            continue;
        };
        let q2m = path == Q2M;
        let source_render = q2m.then(|| validate_and_render(&bin, &file_path, "q2m-source").1);
        // The FullActiveArea output's render, the reference for the FullSensor
        // one: both frame the whole active area (SPEC-061 §3), so the margin
        // the FullSensor file adds must not change a single rendered sample.
        let mut full_active_render = None;
        for framing in [
            Framing::FullActiveArea,
            Framing::StandardCrop,
            Framing::FullSensor,
        ] {
            let file = written(path, &sensor, &plane, framing);
            let tmp = temp_file(&file, &format!("{path}-{framing:?}"));
            let label = format!("{path} {framing:?}");
            let warnings: Vec<String> = if q2m {
                let (warnings, render) = validate_and_render(&bin, &tmp, &label);
                let (reference, name) = match framing {
                    Framing::FullActiveArea => (None, ""),
                    Framing::StandardCrop => (source_render.as_ref(), "the source's"),
                    _ => (full_active_render.as_ref(), "the FullActiveArea output's"),
                };
                if let Some(reference) = reference {
                    assert_eq!(
                        (render.width, render.height),
                        (reference.width, reference.height),
                        "{label}: render size against {name}"
                    );
                    let differ = render
                        .samples
                        .iter()
                        .zip(&reference.samples)
                        .filter(|(a, b)| a != b)
                        .count();
                    assert_eq!(
                        differ, 0,
                        "{label}: render differs from {name} in {differ} samples"
                    );
                    report.push(format!(
                        "{label}: render equals {name} ({}x{}, {} samples)",
                        render.width,
                        render.height,
                        reference.samples.len()
                    ));
                }
                if framing == Framing::FullActiveArea {
                    // Verify-added (HANDOFF-190 item 2): anchor this render
                    // to the source's too, so FullSensor's reference is not
                    // only another output. Adobe applies the warp to the
                    // whole active area and then crops, so the source's
                    // render is this render's default-crop window. Upright
                    // files only: a rotated render moves the window.
                    if let (Some(src), Some(origin), Some(1) | None) = (
                        source_render.as_ref(),
                        sensor.default_crop_origin,
                        sensor.orientation,
                    ) {
                        let (x0, y0) = (origin.x as usize, origin.y as usize);
                        let (sw, sh) = (src.width as usize, src.height as usize);
                        let rw = render.width as usize;
                        let mut differ = 0usize;
                        for row in 0..sh {
                            let at = ((y0 + row) * rw + x0) * 3;
                            let window = &render.samples[at..at + sw * 3];
                            let want = &src.samples[row * sw * 3..(row + 1) * sw * 3];
                            differ += window.iter().zip(want).filter(|(a, b)| a != b).count();
                        }
                        assert_eq!(
                            differ, 0,
                            "{label}: the default-crop window of the render differs from \
                             the source's render in {differ} samples"
                        );
                        report.push(format!(
                            "{label}: its default-crop window equals the source's render"
                        ));
                    }
                    full_active_render = Some(render);
                }
                warnings
            } else {
                complaints(&run_validate(&bin, &[], &tmp))
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            };
            let _ = std::fs::remove_file(&tmp);
            assert!(
                warnings.is_empty(),
                "{label}: dng_validate complained: {warnings:#?}"
            );
            report.push(format!("{label}: valid, no warnings"));
        }
        if let Some(src) = &source_render {
            // DEC-073's forbidden shape renders differently (oracle 5's
            // shape, judged by Adobe's reader).
            let forbidden = crop_cut_with_verbatim_warp(&sensor, &plane);
            let tmp = temp_file(&forbidden, "q2m-forbidden");
            let (_, render) = validate_and_render(&bin, &tmp, "q2m forbidden");
            let _ = std::fs::remove_file(&tmp);
            let differ = render
                .samples
                .iter()
                .zip(&src.samples)
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                (render.width, render.height) != (src.width, src.height) || differ > 0,
                "the crop-cut file must render differently"
            );
            report.push(format!(
                "{Q2M} crop cut, warp verbatim: render differs in {differ} of {} samples",
                src.samples.len()
            ));
        }
    }
    // Shown with --nocapture; the handback quotes it.
    println!("dng_validate report:\n{}", report.join("\n"));
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-081 oracles 3 and 4 — exiftool reads the block and the new tags
// ─────────────────────────────────────────────────────────────────────────────

/// An XMP packet with one `dc:description`, the property oracle 3 reads back.
const XMP_DESCRIPTION: &str = "irradiance SPEC-081 XMP";

fn xmp_packet() -> Vec<u8> {
    format!(
        "<?xpacket begin='' id='W5M0MpCehiHzreSzNTczkc9d'?>\
         <x:xmpmeta xmlns:x='adobe:ns:meta/'>\
         <rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>\
         <rdf:Description rdf:about='' xmlns:dc='http://purl.org/dc/elements/1.1/'>\
         <dc:description><rdf:Alt><rdf:li xml:lang='x-default'>{XMP_DESCRIPTION}</rdf:li>\
         </rdf:Alt></dc:description></rdf:Description></rdf:RDF></x:xmpmeta>\
         <?xpacket end='w'?>"
    )
    .into_bytes()
}

/// The descriptive values oracle 3 sets (all non-empty, so exiftool's `-`
/// for absent cannot be confused with an empty value).
const DESCRIPTIVE: [(&str, &str); 8] = [
    ("IFD0:Make", "IRRADIANCE"),
    ("IFD0:Model", "Mono Test"),
    ("IFD0:Software", "irradiance SPEC-081"),
    ("IFD0:ModifyDate", "2026:11:24 10:20:30"),
    ("IFD0:Artist", "A. Photographer"),
    ("IFD0:Copyright", "(c) 2026 nobody"),
    ("IFD0:LocalizedCameraModel", "Mono Test (local)"),
    ("IFD0:CameraSerialNumber", "0042"),
];
const HINTS: [(&str, Rational); 4] = [
    ("IFD0:BaselineExposure", Rational { num: -7, den: 10 }),
    ("IFD0:BaselineNoise", Rational { num: 3, den: 4 }),
    ("IFD0:BaselineSharpness", Rational { num: 5, den: 4 }),
    ("IFD0:LinearResponseLimit", Rational { num: 9, den: 10 }),
];

fn exif_of(block: &exif_block::Block) -> ExifBlock<'_> {
    let mut b = ExifBlock::default();
    b.bytes = &block.bytes;
    b.exif_ifd = block.exif_ifd;
    b.gps_ifd = block.gps_ifd;
    b
}

/// A 24×16 synthetic plane written with `block`, the XMP packet, the eight
/// descriptive strings and the four hints.
fn synthetic_with_metadata(block: &exif_block::Block, xmp: &[u8]) -> Vec<u8> {
    let (w, h) = (24u32, 16u32);
    let samples: Vec<u16> = (0..w * h).map(|i| (i * 97 % 4096) as u16).collect();
    let mut o = WriteOptions::default();
    o.unique_camera_model = Some(MODEL);
    o.exif_block = Some(exif_of(block));
    o.xmp = Some(xmp);
    let text = |i: usize| Some(DESCRIPTIVE[i].1);
    o.make = text(0);
    o.model = text(1);
    o.software = text(2);
    o.date_time = text(3);
    o.artist = text(4);
    o.copyright = text(5);
    o.localized_camera_model = text(6);
    o.camera_serial_number = text(7);
    o.baseline_exposure = Some(HINTS[0].1);
    o.baseline_noise = Some(HINTS[1].1);
    o.baseline_sharpness = Some(HINTS[2].1);
    o.linear_response_limit = Some(HINTS[3].1);
    write_to_vec(&samples, w, h, &o, BUDGET).expect("synthetic write")
}

/// exiftool's decimal reading `text` equals `num / den`, compared by
/// cross-multiplying (the decimal as an exact fraction), never by float.
fn decimal_equals(text: &str, num: i64, den: i64) -> bool {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (whole, frac) = digits.split_once('.').unwrap_or((digits, ""));
    let Ok(mut value) = format!("{whole}{frac}").parse::<i64>() else {
        return false;
    };
    if negative {
        value = -value;
    }
    let scale = 10i64.pow(frac.len() as u32);
    value * den == num * scale
}

/// Every place where exiftool's reading of `file` differs from what the
/// builder and the options set, for a block of `shape`. Empty means parity.
fn block_mismatches(file: &[u8], shape: exif_block::Shape, label: &str) -> Vec<String> {
    use exif_block::*;
    let tmp = temp_file(file, label);
    // `#` turns print conversion off per tag; GPSLatitude keeps it, with -c
    // printing degrees, minutes and seconds apart so each rational is seen.
    let mut args: Vec<String> = ["-s3", "-f", "-c", "%d %d %.6f"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let block_tags = [
        "ExifIFD:ExposureTime#",
        "ExifIFD:FNumber#",
        "ExifIFD:ISO#",
        "ExifIFD:ExifVersion#",
        "ExifIFD:DateTimeOriginal#",
        "InteropIFD:InteropIndex#",
        "GPS:GPSVersionID#",
        "GPS:GPSLatitudeRef#",
        "GPS:GPSLatitude",
    ];
    for t in block_tags {
        args.push(format!("-{t}"));
    }
    for (t, _) in DESCRIPTIVE {
        args.push(format!("-{t}#"));
    }
    for (t, _) in HINTS {
        args.push(format!("-{t}#"));
    }
    args.push("-XMP-dc:Description#".into());
    let out = tools::run_tool("exiftool", &args, &tmp).expect("exiftool runs");
    let _ = std::fs::remove_file(&tmp);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let lines: Vec<&str> = text.lines().collect();
    let mut bad = Vec::new();
    if lines.len() != block_tags.len() + DESCRIPTIVE.len() + HINTS.len() + 1 {
        bad.push(format!(
            "{label}: exiftool printed {} lines: {text:?}",
            lines.len()
        ));
        return bad;
    }
    let exif = shape != Shape::GpsOnly;
    let gps = shape != Shape::ExifOnly;
    let check = |bad: &mut Vec<String>, i: usize, ok: bool, want: &str| {
        if !ok {
            bad.push(format!(
                "{label}: {} read {:?}, want {want}",
                args.get(i + 4).map_or("?", |s| s.as_str()),
                lines[i]
            ));
        }
    };
    let absent = |i: usize| lines[i] == "-";
    let (en, ed) = EXPOSURE_TIME;
    let (fnn, fnd) = F_NUMBER;
    let block_checks: [(bool, bool, String); 9] = [
        (
            exif,
            decimal_equals(lines[0], en.into(), ed.into()),
            format!("{en}/{ed}"),
        ),
        (
            exif,
            decimal_equals(lines[1], fnn.into(), fnd.into()),
            format!("{fnn}/{fnd}"),
        ),
        (exif, lines[2] == ISO.to_string(), ISO.to_string()),
        (exif, lines[3].as_bytes() == EXIF_VERSION, "0232".into()),
        (
            exif,
            lines[4] == DATE_TIME_ORIGINAL,
            DATE_TIME_ORIGINAL.into(),
        ),
        (exif, lines[5] == INTEROP_INDEX, INTEROP_INDEX.into()),
        (
            gps,
            lines[6] == GPS_VERSION_ID.map(|b| b.to_string()).join(" "),
            "2 3 0 0".into(),
        ),
        (gps, lines[7] == GPS_LATITUDE_REF, GPS_LATITUDE_REF.into()),
        (
            gps,
            {
                let parts: Vec<&str> = lines[8].split(' ').collect();
                parts.len() == 3
                    && parts
                        .iter()
                        .zip(GPS_LATITUDE)
                        .all(|(p, (n, d))| decimal_equals(p, n.into(), d.into()))
            },
            format!("{GPS_LATITUDE:?}"),
        ),
    ];
    for (i, (present, ok, want)) in block_checks.into_iter().enumerate() {
        if present {
            check(&mut bad, i, ok, &want);
        } else {
            check(&mut bad, i, absent(i), "absent");
        }
    }
    let mut at = block_tags.len();
    for (_, want) in DESCRIPTIVE {
        check(&mut bad, at, lines[at] == want, want);
        at += 1;
    }
    for (_, r) in HINTS {
        let ok = decimal_equals(lines[at], r.num.into(), r.den.into());
        check(&mut bad, at, ok, &format!("{}/{}", r.num, r.den));
        at += 1;
    }
    check(&mut bad, at, lines[at] == XMP_DESCRIPTION, XMP_DESCRIPTION);
    bad
}

#[test]
fn exiftool_reads_the_block_and_the_new_tags() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP exiftool_reads_the_block_and_the_new_tags: exiftool is not on PATH — \
             SPEC-081 oracle 3 did not run"
        );
        return;
    }
    use exif_block::Shape;
    let xmp = xmp_packet();
    let mut runs = 0;
    for shape in [Shape::Both, Shape::ExifOnly, Shape::GpsOnly] {
        for odd in [false, true] {
            let block = if odd {
                exif_block::build_odd(EXIF_BLOCK_BASE, shape)
            } else {
                exif_block::build(EXIF_BLOCK_BASE, shape)
            };
            let file = synthetic_with_metadata(&block, &xmp);
            let bad = block_mismatches(&file, shape, &format!("oracle3-{shape:?}-{odd}"));
            assert!(bad.is_empty(), "{bad:#?}");
            runs += 1;
        }
    }
    assert_eq!(runs, 6);
}

#[test]
fn red_proof_a_block_built_for_another_base_is_caught() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP red_proof_a_block_built_for_another_base_is_caught: exiftool is not on PATH — \
             SPEC-081 oracle 4 did not run"
        );
        return;
    }
    use exif_block::Shape;
    let xmp = xmp_packet();
    // Negative control: built for the base the writer uses, it passes.
    let right = exif_block::build(EXIF_BLOCK_BASE, Shape::Both);
    let control = block_mismatches(
        &synthetic_with_metadata(&right, &xmp),
        Shape::Both,
        "control",
    );
    assert!(control.is_empty(), "negative control: {control:#?}");

    // Built for base 10 and placed at 8: the builder did produce a different
    // block (so a builder bug cannot pass this), and exiftool must misread it.
    let wrong = exif_block::build(10, Shape::Both);
    assert_eq!(wrong.bytes.len(), right.bytes.len());
    assert_ne!(wrong.bytes, right.bytes, "the base-10 block differs");
    let caught = block_mismatches(
        &synthetic_with_metadata(&wrong, &xmp),
        Shape::Both,
        "base10",
    );
    assert!(
        !caught.is_empty(),
        "a block built for base 10 read back unchanged: the oracle cannot see a rebase"
    );
    println!(
        "oracle 4: base-10 block caught by {} mismatches:\n{}",
        caught.len(),
        caught.join("\n")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-081 oracle 5 — dng_validate with metadata, and the K-3 III render
// ─────────────────────────────────────────────────────────────────────────────

/// A source IFD0 ASCII tag as text, up to its first NUL; `None` when absent.
fn source_ascii(container: &Container<'_>, tag: u16) -> Option<String> {
    let entry = container.ifd0()?.entry(tag)?;
    let bytes = container.payload(entry).ok()?;
    let text = bytes.split(|&b| b == 0).next().unwrap_or(&[]);
    Some(String::from_utf8_lossy(text).into_owned())
}

/// A source IFD0 hint, read as `mono-core` reads it.
fn source_hint(container: &Container<'_>, tag: u16) -> Option<Rational> {
    let entry = container.ifd0()?.entry(tag)?;
    container.rationals(entry).ok()?.first().copied()
}

/// The source's descriptive strings, in tag order.
const DESCRIPTIVE_TAGS: [u16; 8] = [271, 272, 305, 306, 315, 33432, 50709, 50735];
const HINT_TAGS: [u16; 4] = [50730, 50731, 50732, 50734];

/// The source's descriptive strings and hints, in tag order.
fn source_metadata(container: &Container<'_>) -> (Vec<Option<String>>, Vec<Option<Rational>>) {
    let strings = DESCRIPTIVE_TAGS
        .iter()
        .map(|&t| source_ascii(container, t))
        .collect();
    let hints = HINT_TAGS
        .iter()
        .map(|&t| source_hint(container, t))
        .collect();
    (strings, hints)
}

/// StandardCrop options carrying the oracle-3 block, `xmp` and the source's
/// own strings and hints (SPEC-081 oracle 5's file).
fn metadata_options<'a>(
    sensor: &'a Sensor,
    block: &'a exif_block::Block,
    xmp: &'a [u8],
    strings: &'a [Option<String>],
    hints: &[Option<Rational>],
) -> WriteOptions<'a> {
    let mut o = options_from_sensor(sensor, Framing::StandardCrop, MODEL);
    o.exif_block = Some(exif_of(block));
    o.xmp = Some(xmp);
    let s = |i: usize| strings[i].as_deref();
    o.make = s(0);
    o.model = s(1);
    o.software = s(2);
    o.date_time = s(3);
    o.artist = s(4);
    o.copyright = s(5);
    o.localized_camera_model = s(6);
    o.camera_serial_number = s(7);
    o.baseline_exposure = hints[0];
    o.baseline_noise = hints[1];
    o.baseline_sharpness = hints[2];
    o.linear_response_limit = hints[3];
    o
}

#[test]
fn dng_validate_accepts_metadata_and_the_k3_renders_like_its_source() {
    let Some(bin) = dng_validate() else {
        eprintln!(
            "SKIP dng_validate_accepts_metadata_and_the_k3_renders_like_its_source — \
             ${DNG_VALIDATE_ENV} unset"
        );
        return;
    };
    let block = exif_block::build(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let xmp = xmp_packet();
    let mut report = Vec::new();
    for path in [K3III, Q2M] {
        let Some((file_path, sensor, plane)) = source(path) else {
            continue;
        };
        let bytes = std::fs::read(&file_path).unwrap();
        let container = Container::parse(&bytes).unwrap();
        let (strings, hints) = source_metadata(&container);

        let write_with = |baseline_exposure: bool| {
            let mut o = metadata_options(&sensor, &block, &xmp, &strings, &hints);
            o.baseline_exposure = o.baseline_exposure.filter(|_| baseline_exposure);
            write_to_vec(&plane, sensor.width, sensor.height, &o, BUDGET)
                .unwrap_or_else(|e| panic!("{path}: write with metadata: {e}"))
        };
        let present: Vec<u16> = DESCRIPTIVE_TAGS
            .iter()
            .zip(&strings)
            .filter(|(_, s)| s.is_some())
            .map(|(t, _)| *t)
            .chain(
                HINT_TAGS
                    .iter()
                    .zip(&hints)
                    .filter(|(_, h)| h.is_some())
                    .map(|(t, _)| *t),
            )
            .collect();
        report.push(format!("{path}: carried source tags {present:?}"));

        let file = write_with(true);
        let tmp = temp_file(&file, &format!("{path}-metadata"));
        let label = format!("{path} StandardCrop with metadata");
        let k3 = path == K3III;
        let warnings: Vec<String> = if k3 {
            let (warnings, render) = validate_and_render(&bin, &tmp, &label);
            let (_, source_render) = validate_and_render(&bin, &file_path, "k3-source");
            assert_eq!(
                (render.width, render.height),
                (source_render.width, source_render.height),
                "{label}: render size"
            );
            let differ = render
                .samples
                .iter()
                .zip(&source_render.samples)
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(
                differ, 0,
                "{label}: render differs from the source's in {differ}"
            );
            report.push(format!(
                "{label}: render equals the source's ({}x{}, {} samples)",
                render.width,
                render.height,
                render.samples.len()
            ));

            // The hints' red-proof: without BaselineExposure it renders
            // differently (DEC-074 measured 76,681,557 samples).
            assert!(hints[0].is_some(), "the K-3 III carries BaselineExposure");
            let without = write_with(false);
            let tmp2 = temp_file(&without, "k3-no-baseline-exposure");
            let (w2, render2) = validate_and_render(&bin, &tmp2, "k3 without BaselineExposure");
            let _ = std::fs::remove_file(&tmp2);
            assert!(w2.is_empty(), "k3 without BaselineExposure: {w2:#?}");
            let differ2 = render2
                .samples
                .iter()
                .zip(&source_render.samples)
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                differ2 > 0,
                "dropping BaselineExposure must change the K-3 III render"
            );
            report.push(format!(
                "{path} without BaselineExposure: render differs in {differ2} of {} samples",
                source_render.samples.len()
            ));
            warnings
        } else {
            complaints(&run_validate(&bin, &[], &tmp))
                .into_iter()
                .map(str::to_owned)
                .collect()
        };
        let _ = std::fs::remove_file(&tmp);
        assert!(
            warnings.is_empty(),
            "{label}: dng_validate complained: {warnings:#?}"
        );
        report.push(format!("{label}: valid, no warnings"));
    }
    println!("SPEC-081 dng_validate report:\n{}", report.join("\n"));
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-082 oracles 6 and 7 — the preview through exiftool and dng_validate
// ─────────────────────────────────────────────────────────────────────────────

/// A one-component baseline JPEG, 64×48, of a synthetic XOR pattern (our own
/// content, no camera data). Made once on macOS 26.6.2: a Python script wrote
/// an 8-bit gray TIFF with sample `((x * 4) ^ (y * 5)) & 0xFF`, then
/// `sips -s format jpeg gray.tif --out preview-gray-64x48.jpg`. Its SOF0
/// header says 64×48 and one component.
const GRAY_FIXTURE: &[u8] = include_bytes!("fixtures/preview-gray-64x48.jpg");

fn jpeg_preview(jpeg: &[u8], width: u32, height: u32, color: JpegPreviewColor) -> JpegPreview<'_> {
    let mut p = JpegPreview::default();
    p.jpeg = jpeg;
    p.width = width;
    p.height = height;
    p.color = color;
    p
}

/// Oracle 6 (AC7): exiftool extracts the preview byte for byte and reads its
/// IFD's tags as written, for a Gray (the fixture) and a YCbCr preview.
#[test]
fn exiftool_extracts_the_preview_byte_for_byte() {
    if !tools::exiftool_available() {
        eprintln!(
            "SKIP exiftool_extracts_the_preview_byte_for_byte: exiftool is not on PATH — \
             SPEC-082 oracle 6 did not run"
        );
        return;
    }
    let (w, h) = (24u32, 16u32);
    let samples: Vec<u16> = (0..w * h).map(|i| (i * 97 % 4096) as u16).collect();
    // A synthetic YCbCr stand-in: the writer and exiftool's extraction never
    // decode it. Odd length, so the pad before the raw strip is exercised.
    let ycbcr: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 2, 0xFF, 0xD9, 7];
    let cases = [
        ("gray", GRAY_FIXTURE, 64, 48, JpegPreviewColor::Gray, "1"),
        ("ycbcr", ycbcr, 120, 80, JpegPreviewColor::YCbCr, "6"),
    ];
    for (label, jpeg, pw, ph, color, photometric) in cases {
        let mut o = WriteOptions::default();
        o.unique_camera_model = Some(MODEL);
        o.preview = Some(jpeg_preview(jpeg, pw, ph, color));
        let file = write_to_vec(&samples, w, h, &o, BUDGET).expect("synthetic write");
        let tmp = temp_file(&file, &format!("preview-{label}"));

        let args: Vec<String> = ["-b", "-SubIFD:PreviewImage"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let out = tools::run_tool("exiftool", &args, &tmp).expect("exiftool -b");
        assert_eq!(out.stdout, jpeg, "{label}: exiftool's PreviewImage");

        let args: Vec<String> = [
            "-n",
            "-s3",
            "-f",
            "-SubIFD:SubfileType",
            "-SubIFD:ImageWidth",
            "-SubIFD:ImageHeight",
            "-SubIFD:Compression",
            "-SubIFD:PhotometricInterpretation",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let out = tools::run_tool("exiftool", &args, &tmp).expect("exiftool");
        let _ = std::fs::remove_file(&tmp);
        let text = String::from_utf8_lossy(&out.stdout);
        let (pw, ph) = (pw.to_string(), ph.to_string());
        assert_eq!(
            text.lines().collect::<Vec<_>>(),
            vec!["1", pw.as_str(), ph.as_str(), "7", photometric],
            "{label}: exiftool said {text:?}"
        );
    }
}

/// Oracle 7 (AC8, DEC-069 §6): the K-3 III and Q2M at StandardCrop with
/// every SPEC-081 field and their own largest JPEG preview (its declared
/// size and colour are the source's) validate with no warnings, and render
/// exactly as without the preview; the K-3 III also with the gray fixture.
#[test]
fn dng_validate_accepts_a_written_preview() {
    let Some(bin) = dng_validate() else {
        eprintln!("SKIP dng_validate_accepts_a_written_preview — ${DNG_VALIDATE_ENV} unset");
        return;
    };
    let block = exif_block::build(EXIF_BLOCK_BASE, exif_block::Shape::Both);
    let xmp = xmp_packet();
    let mut report = Vec::new();
    for path in [K3III, Q2M] {
        let Some((file_path, sensor, plane)) = source(path) else {
            continue;
        };
        let bytes = std::fs::read(&file_path).unwrap();
        let container = Container::parse(&bytes).unwrap();
        let (strings, hints) = source_metadata(&container);
        let largest = container
            .previews()
            .flatten()
            .filter(|p| p.compression == Compression::Jpeg)
            .max_by_key(|p| u64::from(p.width) * u64::from(p.height))
            .unwrap_or_else(|| panic!("{path}: the source has a JPEG preview"));
        let color = match (largest.photometric, largest.samples_per_pixel) {
            (Some(1), 1) => JpegPreviewColor::Gray,
            (Some(6) | None, 3) => JpegPreviewColor::YCbCr,
            other => panic!("{path}: unexpected preview colour {other:?}"),
        };

        let base = metadata_options(&sensor, &block, &xmp, &strings, &hints);
        let write = |preview: Option<JpegPreview<'_>>| {
            let mut o = base.clone();
            o.preview = preview;
            write_to_vec(&plane, sensor.width, sensor.height, &o, BUDGET)
                .unwrap_or_else(|e| panic!("{path}: write: {e}"))
        };
        let plain = temp_file(&write(None), &format!("{path}-no-preview"));
        let (w0, render0) = validate_and_render(&bin, &plain, &format!("{path} no preview"));
        let _ = std::fs::remove_file(&plain);
        assert!(w0.is_empty(), "{path} no preview: {w0:#?}");

        let mut runs = vec![(
            format!(
                "{path} with its own {}x{} {color:?} preview ({} bytes)",
                largest.width,
                largest.height,
                largest.bytes.len()
            ),
            jpeg_preview(largest.bytes, largest.width, largest.height, color),
        )];
        if path == K3III {
            runs.push((
                format!("{path} with the gray fixture"),
                jpeg_preview(GRAY_FIXTURE, 64, 48, JpegPreviewColor::Gray),
            ));
        }
        for (label, preview) in runs {
            let file = write(Some(preview));
            // The preview reads back through irradiance too.
            let back = Container::parse(&file).unwrap();
            let got: Vec<_> = back.previews().collect();
            assert!(
                matches!(got.as_slice(), [Ok(q)] if q.bytes == preview.jpeg),
                "{label}: {got:?}"
            );
            let tmp = temp_file(&file, &label);
            let (warnings, render) = validate_and_render(&bin, &tmp, &label);
            let _ = std::fs::remove_file(&tmp);
            assert!(
                warnings.is_empty(),
                "{label}: dng_validate complained: {warnings:#?}"
            );
            assert_eq!(
                (render.width, render.height),
                (render0.width, render0.height),
                "{label}: render size"
            );
            let differ = render
                .samples
                .iter()
                .zip(&render0.samples)
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(differ, 0, "{label}: the preview changed the raw render");
            report.push(format!(
                "{label}: valid, no warnings; render equals the no-preview render ({}x{})",
                render.width, render.height
            ));
        }
    }
    println!("SPEC-082 dng_validate report:\n{}", report.join("\n"));
}
