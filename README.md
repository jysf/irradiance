# irradiance

A permissively-licensed, pure-Rust library that reads camera RAW files and
develops them into images: **sensor data in, pixels and metadata out.**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

```text
RAW bytes ──► ifd ──► plane ──► develop ──► tone ──► displayable image
           (container)  (unpack)  (levels +     (sRGB
                                   geometry)     encode)
```

## What it is

`irradiance` turns a camera's raw sensor capture into an image. It parses the
TIFF/IFD container, locates and unpacks the sensor plane, normalizes levels and
applies the DNG geometry pipeline (defect correction, warp, crop, orientation),
and can display-encode the result to sRGB.

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

> **Status:** early. The current wave (`PROJ-001`) develops **monochrome DNG**.
> Compressed-plane decode and signed/ASCII tag types are intentionally not yet
> implemented. See [Scope](#scope-and-roadmap).

## The pipeline

| Module | Responsibility |
|---|---|
| [`ifd`](src/ifd.rs) | TIFF/IFD container reader; locates the sensor IFD and reads its tags into a `Sensor`. |
| [`plane`](src/plane.rs) | Unpacks the raw sensor plane into a `u16` buffer (two-path unpacker selected by bit depth). |
| [`develop`](src/develop.rs) | Plane → image: bad-pixel fix, black/white levels normalization, `WarpRectilinear`, then `DefaultCrop` → `Orientation`. Output stays scene-linear. |
| [`opcode`](src/opcode.rs) | `OpcodeList` byte-stream parser (DNG 1.7.0.0 Chapter 7): `FixBadPixelsConstant`, `WarpRectilinear`. |
| [`warp`](src/warp.rs) | The `WarpRectilinear` geometric resampler. |
| [`tone`](src/tone.rs) | Display-encode layer: the sRGB OETF (IEC 61966-2-1) as table-driven encoders to linear `f32` or 8-/16-bit sRGB. |

## Usage

Add the crate (this repo is not yet published to crates.io; use a path or git
dependency):

```toml
[dependencies]
irradiance = { path = "../irradiance" }
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

    // 2. Unpack the raw sensor plane into a u16 buffer.
    let mut raw = vec![0u16; sensor.width as usize * sensor.height as usize];
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
current wave:

- **`PROJ-001` — monochrome DNG develop** *(in progress)*: read, unpack,
  normalize levels, apply the DNG geometry/opcode pipeline, and sRGB-encode a
  monochrome DNG end to end.

Intentionally **not** implemented yet (by design, not omission): compressed-plane
decode (`PROJ-003`), and the `ASCII`/signed DNG tag types (no tag `PROJ-001`
reads needs them). See [`projects/`](projects/) for active briefs and
[`docs/ROADMAP.md`](docs/ROADMAP.md) for the long view.

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
