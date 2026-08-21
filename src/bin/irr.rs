//! `irr` — internal dev/oracle binary for `irradiance`.
//!
//! Constraint `library-not-application` (`guidance/constraints.yaml`): this
//! binary is a development tool only, never a shipped product surface, and
//! the library crate must not grow features around it. It is not part of
//! `irradiance`'s public API.
//!
//! The panic-free lints denied in `src/lib.rs` do not apply here — `irr`
//! runs on developer-controlled input, not attacker-influenced RAW bytes
//! (`guidance/toolchain-brief.md`: "`irr` is a dev/oracle binary, not a
//! product surface. `unwrap()` is fine there.").
//!
//! # Subcommands
//!
//! - `irr ifd <file>` — walk the container and print every IFD, then the
//!   sensor plane's tags. This is the surface that makes `SPEC-003`'s parsed
//!   fields *visible* (AGENTS.md §11, "ship the reader with the field"): a
//!   tag nothing reports is a tag nobody can check against `exiftool`.
//!
//! `irr ifd` reads a file, which the library never does — the library takes
//! bytes. The I/O is here, on purpose, and stays here.

use std::process::ExitCode;

use irradiance::ifd::{Compression, Container};

const USAGE: &str = "\
irr — irradiance's internal dev/oracle binary

usage:
  irr ifd [--entries] <file>   walk the TIFF/IFD container and report the sensor plane
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match refs.as_slice() {
        ["ifd", rest @ ..] => cmd_ifd(rest),
        [] | ["-h"] | ["--help"] => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("irr: unknown command {other:?}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_ifd(args: &[&str]) -> ExitCode {
    let mut entries = false;
    let mut path = None;
    for arg in args {
        match *arg {
            "--entries" => entries = true,
            other => path = Some(other),
        }
    }
    let Some(path) = path else {
        eprintln!("irr ifd: expected a file\n\n{USAGE}");
        return ExitCode::FAILURE;
    };

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("irr ifd: cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let container = match Container::parse(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("irr ifd: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("file            {path}");
    println!("bytes           {}", data.len());
    println!("byte_order      {:?}", container.byte_order());
    println!("ifd0_offset     {}", container.ifd0_offset());
    println!("ifds            {}", container.ifds().len());
    for (i, ifd) in container.ifds().iter().enumerate() {
        let parent = match ifd.parent() {
            Some(p) => format!("sub of #{p}"),
            None => "chain".to_string(),
        };
        println!(
            "  #{i} @{} depth {} {} — {} entries, next {}",
            ifd.offset(),
            ifd.depth(),
            parent,
            ifd.entries().len(),
            ifd.next()
        );
        if entries {
            for e in ifd.entries() {
                println!(
                    "      tag {:>5}  type {:>2}  count {:>10}  bytes {:?}",
                    e.tag(),
                    e.field_type(),
                    e.count(),
                    e.byte_len()
                );
            }
        }
    }

    println!("sensor_matches  {:?}", container.sensor_candidates());

    let sensor = match container.sensor() {
        Ok(s) => s,
        Err(e) => {
            println!("sensor          <none: {e}>");
            return ExitCode::FAILURE;
        }
    };

    println!("sensor_ifd      #{}", sensor.ifd_index);
    println!("dimensions      {} x {}", sensor.width, sensor.height);
    println!("bits_per_sample {}", sensor.bits_per_sample);
    println!("samples         {}", sensor.samples_per_pixel);
    println!("photometric     {}", sensor.photometric);
    println!(
        "compression     {} ({})",
        sensor.compression.code(),
        match sensor.compression {
            Compression::Uncompressed => "uncompressed",
            Compression::Jpeg => "JPEG — not decodable by PROJ-001",
            Compression::Other(_) => "vendor/other — not decodable by PROJ-001",
        }
    );
    println!("rows_per_strip  {:?}", sensor.rows_per_strip);
    println!("strip_offsets   {:?}", sensor.strip_offsets);
    println!("strip_bytes     {:?}", sensor.strip_byte_counts);
    println!("black_level     {:?}", sensor.black_level);
    println!("white_level     {:?}", sensor.white_level);
    println!("black_repeat    {:?}", sensor.black_level_repeat_dim);
    println!("active_area     {:?}", sensor.active_area);
    println!("crop_origin     {:?}", sensor.default_crop_origin);
    println!("crop_size       {:?}", sensor.default_crop_size);
    println!("orientation     {:?}", sensor.orientation);
    println!(
        "opcode_lists    1:{} 2:{} 3:{}",
        sensor.opcode_lists[0], sensor.opcode_lists[1], sensor.opcode_lists[2]
    );
    println!("malformed_tags  {:?}", sensor.malformed_tags);

    // The layer-0 oracle, free and available before any tooling: a tightly
    // packed plane with no row padding must reproduce StripByteCounts exactly
    // (AGENTS.md §12 bar 3).
    match (sensor.packed_bits(), sensor.strip_byte_counts.first()) {
        (Ok(bits), Some(declared)) => {
            let expect = u64::from(*declared) * 8;
            println!(
                "layer0          {} bits packed vs {} bits declared — {}",
                bits,
                expect,
                if bits == expect { "CLOSES" } else { "differs" }
            );
        }
        _ => println!("layer0          <not applicable>"),
    }

    match sensor.require_uncompressed() {
        Ok(()) => println!("unpackable      yes"),
        Err(e) => println!("unpackable      no — {e}"),
    }

    ExitCode::SUCCESS
}
