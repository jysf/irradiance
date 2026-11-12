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
- **Panic-free on untrusted input** — RAW is attacker-influenced binary data.
  Every fallible read on a parse/decode path returns a typed
  [`Error`](src/lib.rs) instead of panicking, enforced by five `deny`-level
  clippy lints and a red-proof CI job (`DEC-009`), not just by review.
- **Edition 2021**, MSRV `1.90` (the measured floor; the true floor is lower
  but unmeasured — see [`Cargo.toml`](Cargo.toml)).

> **Status:** early (0.2.x, on crates.io). Uncompressed **monochrome** and
> **Bayer** DNG develop work end to end, and monochrome DNG `Compression 7`
> (lossless-JPEG, SOF-3) planes decode bit-exact through the public
> `plane::unpack_into` — single- and two-component frames (`SPEC-043`/`046`/`044`).
> Size the plane buffer with `plane::validate` first: it checks the layout and
> your own pixel budget before you allocate. See
> [tested DNG modes](#tested-dng-modes) and [Scope](#scope-and-roadmap).

## The pipeline

| Module | Responsibility |
|---|---|
| [`ifd`](src/ifd.rs) | TIFF/IFD container reader; locates the sensor IFD and reads its tags into a `Sensor`. |
| [`plane`](src/plane.rs) | Unpacks the raw sensor plane into a `u16` buffer (two-path unpacker selected by bit depth). |
| [`develop`](src/develop.rs) | Plane → image: bad-pixel fix, black/white levels normalization, `WarpRectilinear`, then `DefaultCrop` → `Orientation`. Output stays scene-linear. |
| [`opcode`](src/opcode.rs) | `OpcodeList` byte-stream parser (DNG 1.7.0.0 Chapter 7): `FixBadPixelsConstant`, `WarpRectilinear`. |
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

    // 2. Preflight BEFORE allocating: `validate` checks the strip layout and
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

The colour path applies `ActiveArea` → `DefaultCrop` → `Orientation` (the
masked border is dropped) but not yet the `OpcodeList` stages (`WarpRectilinear`,
`FixBadPixelsConstant`) that `develop_into` runs.

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
                     #   fuzz-develop, fuzz-warp, fuzz-opcode)
```

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

- **`PROJ-001` — monochrome DNG develop**: read, unpack, normalize levels,
  apply the DNG geometry/opcode pipeline, and sRGB-encode a monochrome DNG end
  to end.
- **`PROJ-002` — Bayer colour develop** *(in progress)*: demosaic and the
  camera→sRGB colour transform are shipped and released (0.2.x); ColorChecker
  ΔE validation (SPEC-035) is pending the chart capture.
- **`PROJ-003` — compressed planes** *(shipped)*: a clean-room, patent-clean
  lossless-JPEG (SOF-3) decoder, fuzzed, and bound to DNG `Compression 7`
  through `plane::unpack_into` (`SPEC-043` routing, `SPEC-046` two-component
  decode, `SPEC-044` bit-exact against `dnglab` on both held files).

Intentionally **not** implemented yet (by design, not omission): tiled planes,
Bayer (CFA) planes inside lossless JPEG, compressions other than `1` and `7`,
and the `ASCII`/signed DNG tag types. The long view — more vendor formats and beyond — is in
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
