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
//! - `irr unpack <file>` — locate the strip and unpack it into a linear
//!   `u16` plane (`SPEC-012`, `DEC-008`), then print the first eight samples
//!   and the decoded min/max. This is the surface `SPEC-012` acceptance
//!   criterion 8's peak-RSS measurement runs through:
//!   `/usr/bin/time -l target/release/irr unpack <file>` on macOS reports
//!   "maximum resident set size" for the whole process, caller's buffer
//!   included — see `docs/provenance-ledger.md`'s `src/plane.rs` row for the
//!   measured number.
//! - `irr develop [--format linear-f32|srgb16|srgb8] <file>` — unpack, then
//!   normalize levels and apply the `ActiveArea` -> `DefaultCrop` ->
//!   `Orientation` geometry (`SPEC-014`, `DEC-018`/`DEC-019`), then encode the
//!   developed scene-linear `u16` image into the chosen output representation
//!   (`SPEC-019`, `DEC-025`; default `srgb8`), printing the resolved geometry,
//!   the developed min/max, and the encoded head/min/max. The `--format` flag
//!   is the **reader** that keeps all three `tone` modes from being unread
//!   (AGENTS.md §11, `AC9`). This is also the surface `SPEC-014` acceptance
//!   criterion 7's peak-RSS measurement runs through — see
//!   `docs/provenance-ledger.md`'s `src/develop.rs` row for the measured
//!   number, which is TWO buffers now (the raw plane AND the developed image),
//!   unlike `irr unpack`'s one.
//!
//! `irr ifd`/`irr unpack`/`irr develop` read a file, which the library never
//! does — the library takes bytes. The I/O is here, on purpose, and stays
//! here.

use std::process::ExitCode;

use irradiance::ifd::{Compression, Container};

const USAGE: &str = "\
irr — irradiance's internal dev/oracle binary

usage:
  irr ifd     [--entries] <file>   walk the TIFF/IFD container and report the sensor plane
  irr unpack  <file>               unpack the sensor plane and report samples/min/max
  irr develop [--format linear-f32|srgb16|srgb8] <file>
                                   unpack, develop, then encode (default srgb8)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match refs.as_slice() {
        ["ifd", rest @ ..] => cmd_ifd(rest),
        ["unpack", rest @ ..] => cmd_unpack(rest),
        ["develop", rest @ ..] => cmd_develop(rest),
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
    println!("color_filter    {:?}", sensor.color_filter);
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

fn cmd_unpack(args: &[&str]) -> ExitCode {
    let Some(path) = args.first().copied() else {
        eprintln!("irr unpack: expected a file\n\n{USAGE}");
        return ExitCode::FAILURE;
    };

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("irr unpack: cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let container = match Container::parse(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("irr unpack: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let sensor = match container.sensor() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("irr unpack: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("file            {path}");
    println!("dimensions      {} x {}", sensor.width, sensor.height);
    println!("bits_per_sample {}", sensor.bits_per_sample);

    // `unwrap()` is fine here — this binary runs on developer-controlled
    // input, not attacker-influenced RAW bytes (module doc above). The
    // library itself (`irradiance::plane::unpack_into`) allocates nothing;
    // this `Vec` is the caller-owned buffer DEC-016 puts on US.
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .expect("plane too large for this host");
    let mut plane = vec![0u16; pixel_count];

    match irradiance::plane::unpack_into(&sensor, container.byte_order(), &data, &mut plane) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("irr unpack: {path}: {e}");
            return ExitCode::FAILURE;
        }
    }

    let head: Vec<u16> = plane.iter().take(8).copied().collect();
    let min = plane.iter().min().copied().unwrap_or(0);
    let max = plane.iter().max().copied().unwrap_or(0);
    println!("samples[0..8]   {head:?}");
    println!("min             {min}");
    println!("max             {max}");
    // A sample above WhiteLevel is legal (DEC-029) — print the real relation
    // rather than asserting `<=`, so the Canon 5D-III's above-white clip shows
    // as `15464 > 15000`, not a false `<=`.
    println!(
        "white_level     {:?}",
        sensor.white_level.map(|w| {
            let rel = if u32::from(max) <= w { "<=" } else { ">" };
            format!("{max} {rel} {w}")
        })
    );
    println!("plane_bytes     {}", plane.len() * 2);

    ExitCode::SUCCESS
}

fn cmd_develop(args: &[&str]) -> ExitCode {
    // `--format <mode>` picks the SPEC-019 output encoder; default `srgb8`.
    let mut format = "srgb8";
    let mut path = None;
    let mut rest = args.iter().copied();
    while let Some(arg) = rest.next() {
        match arg {
            "--format" => match rest.next() {
                Some(value) => format = value,
                None => {
                    eprintln!("irr develop: --format needs a value\n\n{USAGE}");
                    return ExitCode::FAILURE;
                }
            },
            other => path = Some(other),
        }
    }
    if !matches!(format, "linear-f32" | "srgb16" | "srgb8") {
        eprintln!(
            "irr develop: unknown --format {format:?} (want linear-f32|srgb16|srgb8)\n\n{USAGE}"
        );
        return ExitCode::FAILURE;
    }
    let Some(path) = path else {
        eprintln!("irr develop: expected a file\n\n{USAGE}");
        return ExitCode::FAILURE;
    };

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("irr develop: cannot read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let container = match Container::parse(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("irr develop: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let sensor = match container.sensor() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("irr develop: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("file            {path}");
    println!("raw_dimensions  {} x {}", sensor.width, sensor.height);

    // `unwrap()`/`expect()` are fine here — see the module doc's panic-policy
    // note. Both buffers are the caller-owned allocations `DEC-016`/`DEC-018`
    // put on US; neither `unpack_into` nor `develop_into` allocates.
    let raw_pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .expect("plane too large for this host");
    let mut plane = vec![0u16; raw_pixel_count];
    if let Err(e) =
        irradiance::plane::unpack_into(&sensor, container.byte_order(), &data, &mut plane)
    {
        eprintln!("irr develop: {path}: {e}");
        return ExitCode::FAILURE;
    }

    let (out_width, out_height) = match irradiance::develop::output_dimensions(&sensor) {
        Ok(dims) => dims,
        Err(e) => {
            eprintln!("irr develop: {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("out_dimensions  {out_width} x {out_height}");

    let out_pixel_count =
        usize::try_from(u64::from(out_width) * u64::from(out_height)).expect("fits this host");
    let mut developed = vec![0u16; out_pixel_count];
    if let Err(e) = irradiance::develop::develop_into(&sensor, &plane, &mut developed) {
        eprintln!("irr develop: {path}: {e}");
        return ExitCode::FAILURE;
    }

    let head: Vec<u16> = developed.iter().take(8).copied().collect();
    let min = developed.iter().min().copied().unwrap_or(0);
    let max = developed.iter().max().copied().unwrap_or(0);
    println!("samples[0..8]   {head:?}");
    println!("min             {min}");
    println!("max             {max}");
    println!("raw_plane_bytes {}", plane.len() * 2);
    println!("developed_bytes {}", developed.len() * 2);

    // SPEC-019: encode the developed scene-linear `u16` into the chosen output
    // representation and report it, so every `tone` mode has a reader (AC9).
    // The `expect`s are fine here (dev/oracle binary; the buffers are sized to
    // `developed.len()`, so the encoders cannot return a length mismatch).
    println!("format          {format}");
    match format {
        "linear-f32" => {
            let mut out = vec![0f32; developed.len()];
            irradiance::tone::to_linear_f32_into(&developed, &mut out).expect("length matches");
            let head: Vec<f32> = out.iter().take(8).copied().collect();
            let omin = out.iter().copied().fold(f32::INFINITY, f32::min);
            let omax = out.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            println!("encoded[0..8]   {head:?}");
            println!("encoded_min     {omin}");
            println!("encoded_max     {omax}");
            println!("encoded_bytes   {}", out.len() * 4);
        }
        "srgb16" => {
            let mut out = vec![0u16; developed.len()];
            irradiance::tone::encode_srgb16_into(&developed, &mut out).expect("length matches");
            let head: Vec<u16> = out.iter().take(8).copied().collect();
            println!("encoded[0..8]   {head:?}");
            println!("encoded_min     {}", out.iter().min().copied().unwrap_or(0));
            println!("encoded_max     {}", out.iter().max().copied().unwrap_or(0));
            println!("encoded_bytes   {}", out.len() * 2);
        }
        // Validated to be one of the three above; `srgb8` is the remaining case.
        _ => {
            let mut out = vec![0u8; developed.len()];
            irradiance::tone::encode_srgb8_into(&developed, &mut out).expect("length matches");
            let head: Vec<u8> = out.iter().take(8).copied().collect();
            println!("encoded[0..8]   {head:?}");
            println!("encoded_min     {}", out.iter().min().copied().unwrap_or(0));
            println!("encoded_max     {}", out.iter().max().copied().unwrap_or(0));
            println!("encoded_bytes   {}", out.len());
        }
    }

    ExitCode::SUCCESS
}
