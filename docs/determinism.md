# Cross-arch determinism

What irradiance measures about bit-identical output across OS and CPU arch, what it
only argues, and what it leaves unmeasured. `SPEC-041` (PROJ-002, STAGE-010).

## What is measured

`tests/determinism.rs` builds four tier-A synthetic Bayer DNGs in the test itself
(own work from the `tests/support/tiff.rs` primitives, no corpus file — `DEC-003`):

| Fixture | CFA phase | Bit depth | Container byte order | Unpack path (`DEC-008`) |
|---|---|---|---|---|
| `rggb-16bit-le` | RGGB | 16 | little-endian | byte-aligned, file byte order |
| `bggr-14bit-be` | BGGR | 14 | big-endian | MSB-first bitstream |
| `grbg-12bit-le` | GRBG | 12 | little-endian | MSB-first bitstream |
| `gbrg-8bit-be` | GBRG | 8 | big-endian | byte-aligned |

Each is 24 × 16, so `demosaic_into` runs both its interior fast path and its
reflecting border path. The samples are a ramp plus integer xorshift noise, with a
`0` and a full-scale sample pinned so the extremes are covered.

Each fixture runs `plane::unpack_into` → `demosaic::demosaic_into` →
`tone::to_linear_f32_into` (on the demosaiced RGB). Each output's raw bytes are
hashed with a 64-bit FNV-1a written in the test (`u16` as little-endian bytes, `f32`
as its little-endian IEEE bytes — a bit-exact comparison), and every hash is
asserted against a committed constant. Twelve constants: four fixtures × three
outputs.

The CI `determinism` job runs that file on **`ubuntu-latest` (x86_64)** and
**`macos-latest` (arm64)**, in both the dev and release profiles, and checks each
leg's `uname -m` so the arch is pinned, not assumed. The constants were recorded on
macOS arm64; the x86_64 leg passing on the same constants is the cross-arch
evidence.

The oracle is shown red (`DEC-009`, control first): `determinism_red_proof` checks
each unperturbed fixture reproduces its constants, then flips the least-significant
bit of one interior sample and requires all three hashes to move. At build, three
local kernel mutants (each reverted) also went red: the `to_linear_f32_into` divisor
`65535 → 65534`, a one-bit XOR on the 16-bit little-endian unpack, and
`demosaic`'s round-half-up `rnd4` changed to round-half-down.

## The float paths, named

| Path | Arithmetic | Status |
|---|---|---|
| `plane::unpack_into` (uncompressed and Compression-7) | integer only — bit reads, byte-order conversion, lossless-JPEG prediction | **measured** (uncompressed; Compression-7 is not in these fixtures) |
| `demosaic::demosaic_into` (and `_threaded`, byte-identical per `DEC-044`) | integer only — signed `i32` saturating/checked ops, floor division, clamp | **measured** |
| `tone::to_linear_f32_into` | **one IEEE `f32` division per sample**: `f32::from(sample) / 65535.0`. `65535.0` and every `u16` are exact in `f32`, and IEEE 754 requires division to be correctly rounded, so the result is the same on every conforming arch. No FMA (Rust never contracts `a / b` into anything else), no transcendental. | **measured** |
| `tone::encode_srgb16_into` / `encode_srgb8_into` | integer lookup into the committed `tone_table` (the `powf` ran once, in the generator) | not hashed here; integer by construction |
| `color.rs` — `to_srgb_linear_into`, `develop_color_into`, `develop_color_f32_into` (STAGE-008) | `f64`, committed-constant matrices, basic ops only (`+ − × ÷`, polynomials as `n·n·n`), no FMA, no transcendental — `DEC-033` | **unmeasured.** The same IEEE argument applies, but no hash covers it. Out of scope for `SPEC-041`. |
| `warp.rs` — `WarpRectilinear` bilinear resampling | `f64` coordinate map and bilinear weights | **unmeasured.** Named for completeness; not covered by any hash. |

Rust does not fuse a multiply and an add into an FMA unless the source calls
`mul_add`, and the library does not. That is why a basic-ops `f64` path is expected
to agree across arches — but "expected" is an argument, not a measurement, and the
two `f64` paths above stay named as unmeasured until a spec measures them.

## If a hash diverges

A red leg on one arch is a finding: open a new spec for it. Do not edit a kernel or a
committed constant to make the matrix green (`SPEC-041` Non-Goals). Each test prints
every fixture's actual hashes before asserting, so the CI log shows the whole
picture from either arch.

## Out of scope

Tier-B corpus files never run in CI (`DEC-003`); any corpus-file hash is a local
record only. Only the two arches above are measured.
