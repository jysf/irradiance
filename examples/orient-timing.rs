//! `orient-timing` — how long the mono develop takes at each `Orientation`
//! (`SPEC-067`, `DEC-062` §3). Run it as `just orient-timing`, which builds in
//! release; a debug timing means nothing.
//!
//! It reads `LEICA-Q2-MONO/L1021223.DNG` from `$IRRADIANCE_CORPUS_DIR` (the
//! same root resolution as `corpus-status`), overrides `Sensor::orientation` to
//! each of 1–8, and times `develop_with_into` at `warp_alpha` 0 (the warp-free
//! gather, `crop_orient_normalize_into`) and 1 (the post-warp gather,
//! `crop_and_orient_from_active_into`). Orientation 6 with
//! `apply_orientation: false` is the control: the same plane with no
//! transposed gather. Each case has one warm-up and then `RUNS` timed runs.
//!
//! This is a measurement, not a gate: CI machines are shared and noisy, so
//! nothing here fails on a slow number. If the file is absent it prints a SKIP
//! line and exits 0.

#[path = "../tests/support/corpus.rs"]
mod corpus;

use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{develop_with_into, output_dimensions_with, DevelopOptions};
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;

const FILE: &str = "LEICA-Q2-MONO/L1021223.DNG";
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

/// `(median, min)` of `RUNS` timed develops after one warm-up.
fn time_case(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> (Duration, Duration) {
    let (w, h) = output_dimensions_with(sensor, options).unwrap_or_else(|e| panic!("{e}"));
    let mut dst = vec![0u16; (w as usize) * (h as usize)];
    develop_with_into(sensor, src, &mut dst, options).unwrap_or_else(|e| panic!("{e}"));
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            develop_with_into(sensor, src, &mut dst, options).unwrap_or_else(|e| panic!("{e}"));
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
            eprintln!("orient-timing: MANIFEST UNREADABLE — {e}");
            return ExitCode::FAILURE;
        }
    };
    let root = CorpusRoot::resolve();
    let Some(path) = manifest.get(FILE).and_then(|entry| entry.require(&root)) else {
        println!("orient-timing: SKIP L1021223 absent");
        return ExitCode::SUCCESS;
    };

    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let base = container.sensor().unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let mut src = vec![0u16; (base.width as usize) * (base.height as usize)];
    unpack_into(&base, container.byte_order(), &bytes, &mut src)
        .unwrap_or_else(|e| panic!("{FILE}: unpack: {e}"));

    let cpu = command_line("sysctl", &["-n", "machdep.cpu.brand_string"])
        .unwrap_or_else(|| "unknown".to_owned());
    let built_from = command_line("git", &["describe", "--always", "--dirty"])
        .unwrap_or_else(|| "unknown".to_owned());
    println!("orient-timing: {FILE}");
    println!("machine: {} ({cpu})", std::env::consts::ARCH);
    println!("built from: {built_from}");
    println!("develop_with_into, release, 1 warm-up + median of {RUNS}, ms");

    for alpha in [0.0, 1.0] {
        let gather = if alpha == 0.0 {
            "warp-free"
        } else {
            "post-warp"
        };
        println!();
        println!("warp_alpha {alpha} ({gather} gather)");
        println!("| case | median | min | / O1 |");
        println!("|---|---|---|---|");
        let mut baseline = None;
        let cases = (1..=8u32).map(|o| (o, true)).chain([(6, false)]);
        for (orientation, apply_orientation) in cases {
            let mut sensor = base.clone();
            sensor.orientation = Some(orientation);
            let mut options = DevelopOptions::default();
            options.warp_alpha = alpha;
            options.apply_orientation = apply_orientation;
            let (median, min) = time_case(&sensor, &src, &options);
            let o1 = *baseline.get_or_insert(median);
            let label = if apply_orientation {
                format!("O{orientation}")
            } else {
                format!("O{orientation}, apply_orientation false")
            };
            println!(
                "| {label} | {:.0} | {:.0} | {:.2}× |",
                ms(median),
                ms(min),
                median.as_secs_f64() / o1.as_secs_f64()
            );
        }
    }
    ExitCode::SUCCESS
}
