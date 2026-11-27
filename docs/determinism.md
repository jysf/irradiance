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
| `color.rs` — the mono mixer `mono_mix_row` / `mono_mix_plane_into` / `mono_mix_plane_rows_into` (`SPEC-092`, `DEC-083`) | the row: the same `f64` matrix, then `v = (w_0·M[0][i] + w_1·M[1][i]) + w_2·M[2][i]` with basic ops (no FMA) and one exact quantiser `⌊v·2^24 + 0.5⌋`; the plane: exact `i64` integer arithmetic on the traversal's integer camera triples | **measured** (`SPEC-092`): the two rows and the two mono planes of the colour fixture below. |
| `develop.rs` — the developed mix, `DevelopOptions::mono_mix` on `develop_with_into` / `develop_with_scratch_into` (`SPEC-093`, `DEC-084`) | the mixer's exact integer kernel in 512-row strips (output-invariant), then the mono develop's geometry: a gather, or the `f64` warp below and a gather | **measured** (`SPEC-093`): four developed-mix hashes of the colour fixture below, two unwarped and two through an in-memory warp with Orientation 6. |
| `warp.rs` — `WarpRectilinear` bilinear resampling | `f64` coordinate map and bilinear weights | **measured through the developed mix** (`SPEC-093`): the warped pair below. No hash covers `develop_into`'s warp on its own. |

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

## The mono mixer on the colour fixture (`SPEC-092`)

`mono_mix_hashes_match_committed` mixes the same colour fixture (its bytes unchanged)
with two weight sets, luminance `[2126, 7152, 722]/10000` and the first one-hot set
whose row has a negative entry (the red set `[1, 0, 0]`, asserted), and hashes each
fixed-point row (12 little-endian bytes) and each mono plane against four committed
constants. It also asserts the fade of negative weights changes at least one pixel
(8 of the 384): the negative row's plane differs there from the plain fold of
`demosaic_into`'s triples, computed in the test in `i128`. The test and
`tests/mono_mix.rs`'s golden rows assert `MONO_MIX_VERSION == "1"` beside the
constants: a new value needs a version bump in the same change. The extended
`colour_determinism_red_proof` shows the one-bit sample flip moves both plane hashes
(and leaves the rows alone), and a one-unit change of one `AsShotNeutral` numerator
moves both row hashes.

The row's own blind spot is the one above: a fused or reordered `f64` sum moves `v`
by an ulp, far below the row's `2^-24` resolution. The rules are pinned by injected
fixtures instead (`SPEC-091/FU-10`): `color::tests::ac11_the_row_fuses_no_term_of_any_channel`
(the cancellation `0.1·3 − 0.3` at each of the nine fold sites) and
`ac11_the_row_keeps_its_summation_order` (the two wrong pairings of each channel's
three terms), each asserting the exact `f64` bits and that the mutant's value
differs. The plane's step is integer, where order cannot matter.

## The developed mix on the colour fixture (`SPEC-093`)

`developed_mix_hashes_match_committed` develops the same colour fixture (its bytes
unchanged) with `DevelopOptions::mono_mix`, for the luminance and the negative one-hot
weights, under the default options and again with an optional non-identity radial
`WarpRectilinear` (scale `1 − 0.08·r²`, centre `(½, ½)`) and Orientation 6 injected
into the parsed sensor in memory: four committed hashes. Each output equals the
`SPEC-093` §2 oracle (the mixed plane developed as a mono sensor at levels 0 and
65535), and the warped pair differs from the unwarped pair. The fixture has no
`ActiveArea`, `DefaultCrop` or Orientation, so the unwarped pair is the mixed plane
itself and its two hashes are the mixer's plane hashes above (asserted). Both tests
assert `MONO_MIX_VERSION == "1"` and `DEVELOP_VERSION == "8"`.

The extended `colour_determinism_red_proof` shows the one-bit sample flip moves all
four, and a change of `2^-26` in the warp's `kr0` moves the warped two and not the
unwarped two. A one-ulp change of any warp coefficient moves no pixel (measured at
`SPEC-093`'s build): the warped samples are rounded to `u16`, and a shift of about
`10^-16` pixel crosses no rounding boundary on this fixture. So the warped hashes
catch an `f64` difference only once it moves a rounded sample, and `2^-26` is the
smallest power-of-two step of `kr0` that does here.

## If a hash diverges

A red leg on one arch is a finding: open a new spec for it. Do not edit a kernel or a
committed constant to make the matrix green (`SPEC-041` Non-Goals). Each test prints
every fixture's actual hashes before asserting, so the CI log shows the whole
picture from either arch.

## Out of scope

Tier-B corpus files never run in CI (`DEC-003`); any corpus-file hash is a local
record only. Only the two arches above are measured.
