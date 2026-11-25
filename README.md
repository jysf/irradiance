# irradiance

A permissively-licensed, pure-Rust library that reads camera RAW files and
develops them into images: **sensor data in, pixels and metadata out.**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

```text
RAW bytes ──► ifd ──► plane ──► develop ──► tone ──► displayable image
           (container)  (unpack)  (levels +     (sRGB
                                   geometry)     encode)

Bayer DNG:  plane ──► demosaic ──► color ──► linear sRGB   (develop_color_into)
```

## What it is

`irradiance` turns a camera's raw sensor capture into an image. It parses the
TIFF/IFD container, locates and unpacks the sensor plane, normalizes levels and
applies the DNG geometry pipeline (defect correction, warp, crop, orientation),
and can display-encode the result to sRGB. Monochrome DNGs go through
`develop`; Bayer DNGs are additionally demosaiced and colour-transformed
(camera RGB → linear sRGB, with dual-illuminant interpolation).

It is deliberately a **library, not an application**
(constraint `library-not-application`):

- **Pure Rust, zero runtime dependencies** — `[dependencies]` in
  [`Cargo.toml`](Cargo.toml) is empty by design. Dev-only crates are used for
  testing/oracles and are never compiled into the library.
- **No I/O, no CLI, no async runtime** — you hand it bytes, it hands you
  pixels. The `irr` binary in this repo is an internal dev/oracle tool and is
  **not** part of the public API.
- **`#![forbid(unsafe_code)]`.**
- **Designed not to panic on untrusted input** — RAW is attacker-influenced
  binary data. Every fallible read on a parse/decode path returns a typed
  [`Error`](src/lib.rs) instead of panicking. Five `deny`-level clippy lints
  (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
  `arithmetic_side_effects`) and a red-proof CI job (`DEC-009`) keep those
  panic sources out of library code, and fuzz targets exercise the parsers.
  What that does not prove: a standard-library call can still panic on its
  own preconditions (each such call is guarded by a check before it, not by
  a lint), fuzzing samples inputs rather than covering them all, and a failed
  allocation aborts the process rather than returning an error. Call
  `plane::validate` with your own `max_pixels` budget first, so an oversized
  plane is refused before anything is allocated for it.
- **Edition 2021**, MSRV `1.90` (the measured floor; the true floor is lower
  but unmeasured — see [`Cargo.toml`](Cargo.toml)).

> **Status:** early (0.2.6, on crates.io). Uncompressed **monochrome** and
> **Bayer** DNG develop work end to end. Sensor planes stored as strips or
> **tiles**, uncompressed or `Compression 7` (lossless-JPEG, SOF-3), decode
> bit-exact through the public `plane::unpack_into`, and can be split into
> independent row bands for your own threads. 0.2.2 adds develop options
> (framing, orientation, partial lens correction, per-position black levels),
> an allocation-free develop with caller-owned scratch, and zero-copy access to
> embedded previews. 0.2.3 applies `OpcodeList2` `GainMap`s on both develop
> paths, so the Google Pixel 3a, 4a and 7 Pro DNGs develop, and reads a
> fractional `BlackLevel` and a colour `BlackLevelRepeatDim` correctly. 0.2.4
> scales a file whose black differs by position by white minus the largest
> black, as the DNG specification and Adobe's SDK do, and refuses a colour file
> whose white is at or below that black. 0.2.5 applies the camera calibration
> and runs the illuminant weight to convergence, as Adobe's SDK does, and
> refuses tags it cannot honour yet, a newer DNG backward version and a
> malformed tag it reads, instead of developing them wrongly. 0.2.6 also
> refuses a tag that reads cleanly but is empty or the wrong shape. Size the plane
> buffer with `plane::validate` first: it checks the layout and your own pixel budget
> before you allocate. See
> [tested DNG modes](#tested-dng-modes), [`CHANGELOG.md`](CHANGELOG.md) and
> [Scope](#scope-and-roadmap).

**See it in the browser:** [Anatomy of a RAW](https://jysf.github.io/irradiance-demo/)
runs the published crate compiled to WebAssembly and shows every pipeline stage on a
real DNG ([source](https://github.com/jysf/irradiance-demo)). CI builds the library for
`wasm32-unknown-unknown` on every change (`just wasm-check`).

## The pipeline

| Module | Responsibility |
|---|---|
| [`ifd`](src/ifd.rs) | TIFF/IFD container reader; locates the sensor IFD and reads its tags into a `Sensor`; iterates embedded previews as zero-copy slices (`Container::previews`). |
| [`plane`](src/plane.rs) | Unpacks the raw sensor plane (strips or tiles; uncompressed or lossless-JPEG) into a `u16` buffer, whole (`unpack_into`) or band by band (`units` + `unpack_unit_into`). |
| [`develop`](src/develop.rs) | Plane → image: bad-pixel fix, black/white levels normalization, `OpcodeList2` `GainMap`s, `WarpRectilinear`, then `DefaultCrop` → `Orientation`. Output stays scene-linear. `DevelopOptions` and caller-owned scratch via `develop_with_into` / `develop_with_scratch_into`. |
| [`opcode`](src/opcode.rs) | `OpcodeList` byte-stream parser (DNG 1.7.0.0 Chapter 7): `FixBadPixelsConstant`, `GainMap`, `WarpRectilinear`; the required-processing check every develop entry runs first. |
| [`warp`](src/warp.rs) | The `WarpRectilinear` geometric resampler. |
| [`demosaic`](src/demosaic.rs) | Bayer (RGGB family) → camera-space linear RGB, Hamilton-Adams directional kernel. |
| [`color`](src/color.rs) | Camera RGB → linear sRGB via the DNG colour matrices (dual-illuminant interpolation); `develop_color_into` runs levels → demosaic → colour in one call. |
| [`tone`](src/tone.rs) | Display-encode layer: the sRGB OETF (IEC 61966-2-1) as table-driven encoders to linear `f32` or 8-/16-bit sRGB. |

## Usage

Add the crate from crates.io:

```toml
[dependencies]
irradiance = "0.2"
```

End to end — read a DNG's bytes, develop, and display-encode to 8-bit sRGB:

```rust
use irradiance::{ifd::Container, plane, develop, tone};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The crate does no I/O — you read the file.
    let bytes = std::fs::read("photo.dng")?;

    // 1. Parse the TIFF/IFD container and read the sensor-plane tags.
    let container = Container::parse(&bytes)?;
    let sensor = container.sensor()?;

    // 2. Preflight BEFORE allocating: `validate` checks the strip or tile layout and
    //    your own pixel budget (here 100 MP; `usize::MAX` = no limit), so a
    //    forged width x height is a typed error, not an out-of-memory abort.
    let n = plane::validate(&sensor, container.byte_order(), &bytes, 100_000_000)?;

    //    Then unpack the raw sensor plane into a u16 buffer of exactly n samples.
    let mut raw = vec![0u16; n];
    plane::unpack_into(&sensor, container.byte_order(), &bytes, &mut raw)?;

    // 3. Develop into a scene-linear image (levels + geometry pipeline).
    let (w, h) = develop::output_dimensions(&sensor)?;
    let mut image = vec![0u16; w as usize * h as usize];
    develop::develop_into(&sensor, &raw, &mut image)?;

    // 4. Optional: display-encode to 8-bit sRGB for viewing.
    let mut srgb = vec![0u8; image.len()];
    tone::encode_srgb8_into(&image, &mut srgb)?;

    Ok(())
}
```

`develop_into` leaves the image **scene-linear** (`DEC-018`); the `tone`
encoders are the additive, display-referred layer on top.

For a **Bayer** DNG, replace step 3 with `develop_color_into`, which takes the
`width × height` mosaic and writes linear-sRGB `u16` cropped and oriented to the
same geometry as `develop_into` — `output_dimensions(&sensor)`, × 3 channels
(it reports `Error::NotAColorFilterArray` for a monochrome plane):

```rust
let (w, h) = irradiance::develop::output_dimensions(&sensor)?;
let mut rgb = vec![0u16; w as usize * h as usize * 3];
irradiance::color::develop_color_into(&sensor, &raw, &mut rgb)?;
```

The colour path applies `OpcodeList2`'s `GainMap`s and `ActiveArea` →
`DefaultCrop` → `Orientation` (the masked border is dropped) but not yet the
other `OpcodeList` stages (`WarpRectilinear`, `FixBadPixelsConstant`) that
`develop_into` runs.

### Develop options (monochrome)

`develop_with_into` and `output_dimensions_with` take a `DevelopOptions`. It is
`#[non_exhaustive]`, so start from the default and assign fields; the default
reproduces `develop_into` exactly.

```rust
use irradiance::develop::{self, BlackLevels, DevelopOptions, Framing};

let mut options = DevelopOptions::default();
options.framing = Framing::FullActiveArea;   // every recorded photosite, not DefaultCrop
options.apply_orientation = false;           // stored pixel order
options.warp_alpha = 0.5;                    // half the WarpRectilinear correction (0.0..=1.0)
options.black_levels = BlackLevels::PerPosition;

let (w, h) = develop::output_dimensions_with(&sensor, &options)?;
let mut image = vec![0u16; w as usize * h as usize];
develop::develop_with_into(&sensor, &raw, &mut image, &options)?;
```

To develop with **no heap allocation at all**, ask for the scratch size and pass
the buffer yourself (reuse it across frames):

```rust
let mut scratch = vec![0u16; develop::scratch_len(&sensor, &options)?];
develop::develop_with_scratch_into(&sensor, &raw, &mut image, &mut scratch, &options)?;
```

### Decoding the plane in parallel

irradiance stays single-threaded and dependency-free, but `plane::units` splits
the plane into full-width row bands that decode independently. Each band is a
contiguous run of the plane, so `split_at_mut` hands each thread its own slice
with no `unsafe`, and the result equals `unpack_into` byte for byte:

```rust
use std::num::NonZeroU32;
use irradiance::plane::{units, unpack_unit_into};

let order = container.byte_order();
let mut plane = vec![0u16; n];
std::thread::scope(|scope| -> Result<(), irradiance::Error> {
    let mut rest: &mut [u16] = &mut plane;
    let mut workers = Vec::new();
    for unit in units(&sensor, &bytes, NonZeroU32::new(512).unwrap())? {
        let len = unit.samples(sensor.width).expect("the band fits usize");
        let (band, tail) = rest.split_at_mut(len);
        rest = tail;
        let (sensor, bytes) = (&sensor, &bytes);
        workers.push(scope.spawn(move || unpack_unit_into(sensor, order, bytes, &unit, band)));
    }
    for worker in workers {
        worker.join().expect("a worker panicked")?;
    }
    Ok(())
})?;
```

A single-scan `Compression 7` strip is one unit, so it does not split.

### Embedded previews

`Container::previews()` yields each embedded preview's metadata and its bytes as
a slice borrowed from your input — never copied, never decoded:

```rust
for preview in container.previews() {
    let preview = preview?; // one bad preview is an Err; iteration continues
    println!("{:?} {}x{} {:?}, {} bytes",
        preview.kind, preview.width, preview.height,
        preview.compression, preview.bytes.len());
}
```

### Writing a monochrome DNG (feature `write`)

Behind the opt-in `write` feature (no dependency), `irradiance::dng::write`
writes one monochrome `u16` plane, unchanged, as an uncompressed little-endian
DNG into a buffer you size first. Framing and orientation are tags; samples are
cut only at the `ActiveArea` edge:

```rust
use irradiance::dng::write::{write_into, written_len, Framing, WriteOptions};

let mut options = WriteOptions::default();
options.framing = Framing::StandardCrop;
options.active_area = sensor.active_area;
options.default_crop = sensor.default_crop_origin.zip(sensor.default_crop_size);
options.black_level = &sensor.black_levels;
options.white_level = sensor.white_level.unwrap_or(65535);
options.opcode_list_3 = sensor.opcode_list_3.as_deref();
options.unique_camera_model = Some("My Monochrome");

let len = written_len(&plane, sensor.width, sensor.height, &options, max_pixels)?;
let mut file = vec![0u8; len];
write_into(&plane, sensor.width, sensor.height, &options, max_pixels, &mut file)?;
```

`written_len` and `write_into` allocate nothing and run the same checks in the
same order, so `written_len` is also a preflight: a framing the source cannot
be written at (an `OpcodeList1` a cut would shift) is refused before any buffer
exists. Each written file is checked by irradiance's own reader and develop,
`dnglab`, `exiftool` and Adobe's `dng_validate` 1.7.1 on the held mono files.

## Tested DNG modes

What the public `plane::validate` → `plane::unpack_into` route accepts today,
and how each mode is tested. Tier A = hand-built fixtures, committed and run in
CI; tier B = the real-file corpus (never committed, `DEC-003`), bit-exact against
`dnglab --raw-checksum`.

| Plane | Compression | Layout | Status | Tested by |
|---|---|---|---|---|
| Monochrome (`LinearRaw`), 8/12/14/16-bit | `1` (uncompressed) | one or many strips | ✔ supported | tier A + tier B (Leica Q2 Monochrom, Leica M Monochrom), bit-exact |
| Bayer (CFA, RGGB family), 12/14/16-bit | `1` (uncompressed) | one or many strips | ✔ supported | tier A + tier B (Canon 5D Mark III, Nikon D300S 12-strip, …), bit-exact |
| Monochrome, SOF-3 single-component (`Nf = 1`) | `7` (lossless JPEG) | one or many strips | ✔ supported | tier A (round-trip encoder fixtures, fuzzed) |
| Monochrome, SOF-3 two-component column interleave (`Nf = 2`) | `7` (lossless JPEG) | one or many strips | ✔ supported | tier A + tier B (Leica M Monochrom Typ 246, Pentax K-3 III Monochrome DNG), bit-exact |
| Monochrome or Bayer, 8/12/14/16-bit | `1` (uncompressed) | **tiles**, including partial edge tiles (`SPEC-058`) | ✔ supported | tier A (writer-built fixtures, II and MM, fuzzed) |
| Monochrome or Bayer, SOF-3 `Nf = 1` or `Nf = 2` | `7` (lossless JPEG) | **tiles**, edge tiles coded full-size and clipped (`SPEC-058`, `DEC-056`) | ✔ supported | tier A + tier B (Google Pixel 4a, 3a, 7 Pro: 192 tiles of 256×256, Bayer, `Nf = 2`), bit-exact |
| Uncompressed tiles whose row is not a whole number of bytes | `1` | tiles | ✘ `UnsupportedTileRowBits` | tier A |
| Bayer (CFA) inside lossless JPEG, other component layouts | `7` | — | ✘ not supported yet | — |
| Any | other codes (vendor-private, lossy, …) | — | ✘ `UnsupportedCompression` | tier A + tier B (Pentax PEF, `65535`) |

For every supported mode `validate` refuses a plane over the caller's
`max_pixels` (`PlaneTooLarge`) before any other check. For an uncompressed plane
its `Ok(n)` also guarantees the unpack succeeds and that `n` is bounded by the
file's own bytes (`StripBytesExceedFile` refuses strips or tiles that alias); for a
Compression-7 plane `n` is bounded only by `max_pixels`, and the SOF-3 frame
shape is still checked at decode (`DEC-050`).

## Building and testing

The toolchain *is* the dependency — there is nothing to install beyond Rust:

```bash
cargo fetch          # "install"
just build           # cargo build --release
just test            # the full suite (--all-features is baked in)
just lint            # clippy (both halves) + rustfmt --check
just lint-ci         # clippy AS CI SEES IT — run this before every push
just deny            # permissive-only license gate
just msrv            # compile against the pinned MSRV (1.90.0)
just fuzz            # the ifd/container fuzz target (also: fuzz-plane,
                     #   fuzz-develop, fuzz-warp, fuzz-opcode, fuzz-ljpeg,
                     #   fuzz-dng-write)
just doc-check       # cargo doc with warnings denied
just wasm-check      # build the library for wasm32-unknown-unknown (the browser)
```

Timing examples (release build, need the Tier-B corpus): `just unpack-timing`,
`just ljpeg-timing`, `just orient-timing`.

Run a single test by name:

```bash
cargo test --all-features <test_name> -- --exact --nocapture
```

The exact command for each recipe is listed in
[`AGENTS.md`](AGENTS.md) §6, "Commands (exact)". Some tests use a Tier-B RAW
corpus and skip loudly when it is absent; point `$IRRADIANCE_CORPUS_DIR` at a
directory of sample files to enable them.

## Scope and roadmap

`irradiance` is developed as a series of **projects** (waves of work). The
waves:

- **`PROJ-001` — monochrome DNG develop** *(shipped)*: read, unpack, normalize
  levels, apply the DNG geometry/opcode pipeline, and sRGB-encode a monochrome
  DNG end to end.
- **`PROJ-002` — Bayer colour develop** *(in progress)*: demosaic and the
  camera→sRGB colour transform are shipped and released (0.2.x). Since 0.2.5 the
  transform applies the camera calibration, and on 18 probe values per held
  colour file it is within 6 codes of Adobe's DNG SDK 1.7.1; ColorChecker ΔE
  validation (SPEC-035) is pending the chart capture.
- **`PROJ-003` — compressed planes** *(shipped)*: a clean-room, patent-clean
  lossless-JPEG (SOF-3) decoder, fuzzed, and bound to DNG `Compression 7`
  through `plane::unpack_into` (`SPEC-043` routing, `SPEC-046` two-component
  decode, `SPEC-044` bit-exact against `dnglab` on both held files).
- **`PROJ-004` — instrument layer and consumer integration** *(in progress)*: a
  read-only, zero-I/O, caller-allocated surface beyond the developed image, for
  real consumers. Shipped in 0.2.2: tiled planes, embedded previews, develop
  options, caller-owned scratch, plane units, and faster packed and
  lossless-JPEG decodes. Shipped in 0.2.3: `OpcodeList2` `GainMap`s (the tiled
  Pixel DNGs develop), fractional and colour per-pattern black levels, and
  `Sensor::masked_areas`. Shipped in 0.2.4: black normalisation by white minus
  the largest black, and the colour refusal of white at or below it. The public
  clean-room DNG writer moved to `PROJ-005`; optical-black instruments are
  parked until a caller needs them.
- **`PROJ-005` — public DNG writer** *(in progress)*: the mono `u16` writer
  core behind the `write` feature is built (`SPEC-061`, unreleased); the EXIF
  block, XMP, descriptive tags and rendering hints (`SPEC-081`) and an optional
  preview (`SPEC-082`) follow.

Intentionally **not** implemented yet (by design, not omission): lossless-JPEG
component layouts other than `Nf = 1` and `Nf = 2`, compressions other than `1` and `7`,
mandatory opcodes the develop does not apply (refused with a typed error, never
silently skipped), tags that change what the samples mean such as
`LinearizationTable` or a non-zero `BlackLevelDeltaH/V` (also refused), and the `ASCII`/signed DNG tag types. The long view — more vendor formats and beyond — is in
[`docs/future-plans.md`](docs/future-plans.md); see [`projects/`](projects/)
for active briefs.

## How this repo is built

This repo follows a **spec-driven multi-agent workflow**: work flows through a
Repo → Project → Stage → Spec → Cycle hierarchy, architectural choices are
recorded as `DEC-*` records, and constraints are enforced mechanically. If
you're contributing or just curious about the process, start here:

- [`docs/spec-driven-template.md`](docs/spec-driven-template.md) — the process, end to end
- [`AGENTS.md`](AGENTS.md) — conventions every agent (and human) follows
- [`guidance/constraints.yaml`](guidance/constraints.yaml) — the repo-level rules
- [`decisions/`](decisions/) — the decision log
- `just status` — current project, stage, and specs by cycle

## Where things live

| Path | Purpose |
|---|---|
| [`src/`](src/) | The library |
| [`docs/`](docs/) | Architecture, oracle contract, provenance ledger, roadmap |
| [`AGENTS.md`](AGENTS.md) | Conventions for anyone (agent or human) working in this repo |
| [`.repo-context.yaml`](.repo-context.yaml) | Structured metadata about the app |
| [`guidance/`](guidance/) | Repo-level rules, open questions, signals ledger |
| [`decisions/`](decisions/) | Decision log (`DEC-*`) |
| [`projects/`](projects/) | Each wave of work, with its briefs, stages, and specs |
| [`fuzz/`](fuzz/) | Fuzz targets (kept outside the library's dependency graph) |

## License

Licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional
terms or conditions.
