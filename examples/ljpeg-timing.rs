//! `ljpeg-timing` — how long a single-threaded Compression 7 (lossless-JPEG)
//! sensor-plane decode takes (`SPEC-070`, `DEC-065` §4). Run it as
//! `just ljpeg-timing`, which builds in release; a debug timing means nothing.
//!
//! It times `plane::unpack_into` on three held Compression 7 files from
//! `$IRRADIANCE_CORPUS_DIR` (the same root resolution as `corpus-status`):
//!
//! - `PENTAX-K3III-MONO/K3III.DNG`: one restart-free two-component strip;
//! - `LEICA-M-MONOCHROM-TYP246/M2462362.DNG`: the same shape;
//! - `GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng`: 192 lossless-JPEG tiles.
//!
//! Each file has one warm-up and then `RUNS` timed runs; the median and the
//! minimum are printed. This is a measurement, not a gate: CI machines are
//! shared and noisy, so nothing here fails on a slow number. An absent file
//! prints a SKIP line, and the example still exits 0.

#[path = "../tests/support/corpus.rs"]
mod corpus;

use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::Container;
use irradiance::plane::unpack_into;

const FILES: [&str; 3] = [
    "PENTAX-K3III-MONO/K3III.DNG",
    "LEICA-M-MONOCHROM-TYP246/M2462362.DNG",
    "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng",
];
const RUNS: usize = 7;

/// The output of `program args…` trimmed, or `None` if it did not run cleanly.
fn command_line(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// `(median, min)` of `RUNS` timed calls of `f` after one warm-up.
fn time(mut f: impl FnMut()) -> (Duration, Duration) {
    f();
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .collect();
    times.sort();
    (times[RUNS / 2], times[0])
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() -> ExitCode {
    let manifest = match Manifest::load() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ljpeg-timing: MANIFEST UNREADABLE — {e}");
            return ExitCode::FAILURE;
        }
    };
    let root = CorpusRoot::resolve();

    let cpu = command_line("sysctl", &["-n", "machdep.cpu.brand_string"])
        .unwrap_or_else(|| "unknown".to_owned());
    let model = command_line("sysctl", &["-n", "hw.model"]).unwrap_or_else(|| "unknown".to_owned());
    let built_from = command_line("git", &["describe", "--always", "--dirty"])
        .unwrap_or_else(|| "unknown".to_owned());
    println!("ljpeg-timing: plane::unpack_into, 1 thread, Compression 7");
    println!("machine: {model}, {} ({cpu})", std::env::consts::ARCH);
    println!("built from: {built_from}");
    println!("release, 1 warm-up + median of {RUNS}, ms");
    println!();

    let mut rows: Vec<String> = Vec::new();
    for file in FILES {
        let Some(path) = manifest.get(file).and_then(|entry| entry.require(&root)) else {
            println!("ljpeg-timing: SKIP {file} absent");
            continue;
        };
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{file}: {e}"));
        let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{file}: {e}"));
        let sensor = container.sensor().unwrap_or_else(|e| panic!("{file}: {e}"));
        let order = container.byte_order();
        let n = (sensor.width as usize) * (sensor.height as usize);

        let mut plane = vec![0u16; n];
        let (median, min) = time(|| {
            unpack_into(&sensor, order, &bytes, &mut plane)
                .unwrap_or_else(|e| panic!("{file}: unpack: {e}"));
        });
        rows.push(format!(
            "| {file} | {} x {} | {:.1} | {:.1} |",
            sensor.width,
            sensor.height,
            ms(median),
            ms(min)
        ));
    }

    if !rows.is_empty() {
        println!("| file | size | median | min |");
        println!("|---|---|---|---|");
        for row in rows {
            println!("{row}");
        }
    }
    ExitCode::SUCCESS
}
