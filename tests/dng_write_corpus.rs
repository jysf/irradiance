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
//! Every written file goes to the OS temp dir, never the repo.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_write_oracle.rs"]
mod dng_write_oracle;
#[path = "support/dng_writer.rs"]
mod dng_writer;
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
use irradiance::dng::write::{write_to_vec, Framing};
use irradiance::ifd::{ActiveArea, Sensor};

/// The four held monochrome DNGs (`SPEC-061` §7 item 4).
const FILES: [&str; 4] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "PENTAX-K3III-MONO/K3III.DNG",
    "LEICA-M-MONOCHROM/L1000622.DNG",
    "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
];
const Q2M: &str = "LEICA-Q2-MONO/L1021223.DNG";

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
