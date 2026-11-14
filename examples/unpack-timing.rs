//! `unpack-timing` — how long the sensor-plane unpack takes, single-threaded
//! and as SPEC-068 units on scoped threads (`SPEC-069`, `DEC-064` §3). Run it
//! as `just unpack-timing`, which builds in release; a debug timing means
//! nothing.
//!
//! It reads `LEICA-Q2-MONO/L1021223.DNG` (14-bit, one uncompressed strip,
//! 8424 × 5632) from `$IRRADIANCE_CORPUS_DIR` (the same root resolution as
//! `corpus-status`) and times:
//!
//! - `unpack_into` on one thread;
//! - 8 bands of 704 rows, from `plane::units`, each decoded by
//!   `unpack_unit_into` on its own `std::thread::scope` thread into a
//!   `chunks_mut` band of one plane. The assembled plane is asserted
//!   byte-identical to the single-thread plane.
//!
//! Each case has one warm-up and then `RUNS` timed runs; the median and the
//! minimum are printed. This is a measurement, not a gate: CI machines are
//! shared and noisy, so nothing here fails on a slow number. If the file is
//! absent it prints a SKIP line and exits 0.

#[path = "../tests/support/corpus.rs"]
mod corpus;

use std::num::NonZeroU32;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::{ByteOrder, Container, Sensor};
use irradiance::plane::{units, unpack_into, unpack_unit_into, Unit};

const FILE: &str = "LEICA-Q2-MONO/L1021223.DNG";
const RUNS: usize = 7;
/// 5632 rows / 8 threads.
const BAND_ROWS: u32 = 704;
const THREADS: usize = 8;

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

/// Decode every unit on its own scoped thread, each into its own
/// `chunks_mut` band of `plane`. Every unit here is a full band of
/// `BAND_ROWS` rows (5632 is a multiple of 704), so the bands line up with
/// `chunks_mut(BAND_ROWS · width)` exactly.
fn unpack_threaded(
    sensor: &Sensor,
    order: ByteOrder,
    file: &[u8],
    units: &[Unit],
    plane: &mut [u16],
) {
    let band_len = (BAND_ROWS as usize) * (sensor.width as usize);
    std::thread::scope(|scope| {
        for (unit, band) in units.iter().zip(plane.chunks_mut(band_len)) {
            scope.spawn(move || {
                unpack_unit_into(sensor, order, file, unit, band)
                    .unwrap_or_else(|e| panic!("{FILE}: unit at row {}: {e}", unit.row_start));
            });
        }
    });
}

fn main() -> ExitCode {
    let manifest = match Manifest::load() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("unpack-timing: MANIFEST UNREADABLE — {e}");
            return ExitCode::FAILURE;
        }
    };
    let root = CorpusRoot::resolve();
    let Some(path) = manifest.get(FILE).and_then(|entry| entry.require(&root)) else {
        println!("unpack-timing: SKIP L1021223 absent");
        return ExitCode::SUCCESS;
    };

    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{FILE}: {e}"));
    let order = container.byte_order();
    let n = (sensor.width as usize) * (sensor.height as usize);

    let mut single = vec![0u16; n];
    let (single_median, single_min) = time(|| {
        unpack_into(&sensor, order, &bytes, &mut single)
            .unwrap_or_else(|e| panic!("{FILE}: unpack: {e}"));
    });

    let max_band_rows = NonZeroU32::new(BAND_ROWS).expect("non-zero");
    let plane_units: Vec<Unit> = units(&sensor, &bytes, max_band_rows)
        .unwrap_or_else(|e| panic!("{FILE}: units: {e}"))
        .collect();
    assert_eq!(
        plane_units.len(),
        THREADS,
        "{FILE}: expected {THREADS} bands of {BAND_ROWS} rows"
    );
    let mut threaded = vec![0u16; n];
    let (threaded_median, threaded_min) =
        time(|| unpack_threaded(&sensor, order, &bytes, &plane_units, &mut threaded));
    assert!(
        threaded == single,
        "{FILE}: the 8-thread plane differs from the single-thread plane"
    );

    let cpu = command_line("sysctl", &["-n", "machdep.cpu.brand_string"])
        .unwrap_or_else(|| "unknown".to_owned());
    let model = command_line("sysctl", &["-n", "hw.model"]).unwrap_or_else(|| "unknown".to_owned());
    let built_from = command_line("git", &["describe", "--always", "--dirty"])
        .unwrap_or_else(|| "unknown".to_owned());
    println!(
        "unpack-timing: {FILE} ({} x {}, {}-bit)",
        sensor.width, sensor.height, sensor.bits_per_sample
    );
    println!("machine: {model}, {} ({cpu})", std::env::consts::ARCH);
    println!("built from: {built_from}");
    println!("release, 1 warm-up + median of {RUNS}, ms");
    println!();
    println!("| case | median | min |");
    println!("|---|---|---|");
    println!(
        "| unpack_into, 1 thread | {:.1} | {:.1} |",
        ms(single_median),
        ms(single_min)
    );
    println!(
        "| {THREADS} units of {BAND_ROWS} rows, {THREADS} threads | {:.1} | {:.1} |",
        ms(threaded_median),
        ms(threaded_min)
    );
    ExitCode::SUCCESS
}
