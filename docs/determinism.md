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

Since 2026-11-18 the macOS leg is non-blocking (`continue-on-error`): GitHub's
hosted macOS arm64 runners were often unavailable, and the job was cancelled without
running. The constants were recorded on arm64, so the x86_64 leg still gates on
them; read the macOS leg's own result on any change to a kernel or a constant.

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
| `color.rs` — the sensor-geometry plane `srgb_linear_plane_into` / `srgb_linear_plane_f32_into` (`SPEC-091`), whose `u16` pixels are `develop_color_into`'s pixels before the crop | `f64`, committed-constant matrices, basic ops only (`+ − × ÷`, polynomials as `n·n·n`), no FMA, no transcendental — `DEC-033`; the unclamped `f32` form adds one `f64 ÷ 65535` and one IEEE round-to-nearest-even cast to `f32` | **measured** (`SPEC-091`): the colour fixture below, dual-illuminant so the convergent interpolation (`DEC-076`) runs. |
| `color.rs` — the crop/orient sink of `develop_color_into` / `develop_color_f32_into`, and `to_srgb_linear_into` | the same `f64` arithmetic; the crop is a rearrangement, `develop_color_f32_into` one more `f32 ÷` | **not hashed separately.** Same arithmetic as the measured plane (`develop_color_into`'s pixels are the plane's, cut to the crop: `tests/linear_plane.rs` AC1), but no fixture hashes these outputs themselves. |
| `warp.rs` — `WarpRectilinear` bilinear resampling | `f64` coordinate map and bilinear weights | **unmeasured.** Named for completeness; not covered by any hash. |

Rust does not fuse a multiply and an add into an FMA unless the source calls
`mul_add`, and the library does not. That is why a basic-ops `f64` path is expected
to agree across arches — but "expected" is an argument, not a measurement. The colour
plane is now measured (below); `warp.rs` stays named as unmeasured until a spec
measures it.

## The colour fixture (`SPEC-091`)

`colour_plane_hashes_match_committed` builds a separate 24 × 16 RGGB 16-bit fixture
(so no byte an older fixture hashes changes) with `AsShotNeutral`,
`ColorMatrix1/2`, `ForwardMatrix1/2` and `CalibrationIlluminant` A and D65, and
hashes the `u16` and the unclamped `f32` sensor-geometry planes (`DEC-079`) against
two committed constants. It also asserts the `f32` plane holds a negative and an
above-1.0 value (so the hash covers values a clamp would change) and that removing
the second illuminant changes the plane. `colour_fixture_interpolates_between_its_illuminants`
(added at `SPEC-091`'s verify) shows the fixture's weight is strictly between the two
calibrations: the plane differs from both controls whose two calibrations are the same
matrices, so the interpolation itself, not a weight clamped to one end, is in the hash.
`colour_determinism_red_proof` checks the control, then flips one interior sample's
least-significant bit and requires both hashes to move. The constants were recorded
on macOS arm64; the x86_64 leg passing is the cross-arch evidence that onyx's
stored candidate mixes rely on.

What the hash cannot see: an FMA. Fusing one of the sink's multiply-adds moves the
`f64` sum by about one ulp, and the single cast to `f32` (or the `u16` quantise)
absorbs that on all but about one value in 2^29, so a fused sum would leave these
hashes unchanged (measured at `SPEC-091`'s build: the `mul_add` mutant passed every
hash and the exact-bits AC5 test). The no-FMA rule is pinned instead by
`color::tests::the_f32_sink_never_fuses_a_multiply_and_an_add`, an injected matrix
whose sum is pure cancellation (`0.1·3 − 0.3`), so a fused and a separate sum differ
by a factor of two. That test places the cancellation in channel 0's first term
only; `color::tests::the_f32_sink_fuses_no_term_of_any_channel` (added at
`SPEC-091`'s verify) places it at every term of every channel, so a `mul_add` on any
of the nine products is caught.

## If a hash diverges

A red leg on one arch is a finding: open a new spec for it. Do not edit a kernel or a
committed constant to make the matrix green (`SPEC-041` Non-Goals). Each test prints
every fixture's actual hashes before asserting, so the CI log shows the whole
picture from either arch.

## Out of scope

Tier-B corpus files never run in CI (`DEC-003`); any corpus-file hash is a local
record only. Only the two arches above are measured.
