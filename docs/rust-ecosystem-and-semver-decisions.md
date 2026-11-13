# Rust image ecosystem and semver: decision memo

Status: research memo, 2026-11-12. **Nothing here is a decision.** Each item that
needs one is listed in §4 with a draft DEC title. Next-free IDs, computed from
`origin/main` at `bd0e0df`, are **DEC-059** and **SPEC-068**. Open PR #217 adds no
DEC or new SPEC. Mint the IDs when the DECs are written, not from this memo.

Baseline: irradiance **0.2.1** (crates.io), `DEVELOP_VERSION = "4"`, zero
`[dependencies]`, MSRV 1.90. This supersedes the draft
`docs/audiences-and-semver.md` on `claude/pyradiance-use-cases-af310c`. That draft
was written at `80b6783` (2026-11-05), before 0.2.0, and several of its hazards are
now resolved (§2.1).

Two labels are used throughout:

- **[V]** means verified from a source: repo code at `bd0e0df`, the crates.io API,
  crate source tarballs, or GitHub. Research was done 2026-11-12.
- **[J]** means my judgment.
- Anything I could not check is marked *unverified*.

---

## 1. Part A: how irradiance plugs into the Rust image ecosystem

### 1.1 What exists for RAW in Rust today [V]

Downloads are all-time / last 90 days. Dates are the latest release.

| Crate | Latest | Downloads | License | Deps | DNG coverage | `image` integration |
|---|---|---|---|---|---|---|
| rawloader | 0.37.2 (2026-08-15) | 335k / 24k | LGPL-2.1 | 5 (incl. rayon) | Compression 1, 7, 0x884c; others refused | None |
| rawler (dnglab) | 0.8.0 (2026-08-30) | 223k / 118k | LGPL-2.1 | 27, incl. a non-optional `image` and jxl-oxide | 1, 7, 8 (Deflate), 0x884c, 52546 (JXL), plus CR3/NEF/ARW/… | Returns `DynamicImage`; does not implement `ImageDecoder` |
| quickraw | 0.2.1-alpha.1 (2023-06-19) | 25k / 2k | LGPL-2.1 | 8 | unverified | Optional, wasm only |
| imagepipe | 0.5.1 (2026-08-15) | 123k / 6k | LGPL-3.0-only | 12 (rawloader, image) | via rawloader | Uses `image` |
| zenraw | 0.2.0 (2026-04-17) | 1.1k | AGPL-3.0-only OR commercial | 13 (+ rawler/rawloader optional) | via backends | Own `zencodec` |
| libraw-rs / rsraw | 0.0.4 (2021) / 0.1.1 (2026-03-21) | — | MIT/Apache, MIT | C++ FFI (LibRaw) | LibRaw's | None |
| **gamut-dng** | **1.0.0** (2026-07-21) | 956 | **MIT OR Apache-2.0** | 5 (own sub-crates + miniz_oxide) | Strips and tiles; uncompressed, Deflate, lossless JPEG, **JXL**; README claims full Apple ProRAW decode, checked against the Adobe SDK | None |
| **filmcraft-dng** | 0.9.0 (2026-10-23) | 467 | **MIT OR Apache-2.0** | 2 (thiserror, rayon opt.) | DNG and CinemaDNG; strips, tiles, SOF-3; clean-room, wasm | None |
| **flashraw** | 0.2.0 (2026-11-11) | 886 | **MIT OR Apache-2.0** | 9 (5 optional) | About 40 formats; README claims 1,980 of 2,016 raw.pixls.us files; **no JPEG-in-DNG** | None |
| dng (apertus) | 1.6.0 (2026-07-28) | 25k | AGPL-3.0 | 5 optional | Tag/IFD level read and write | None |

Sources: `crates.io/api/v1/crates/<name>` and `/<ver>/dependencies`.
DNG compression lists come from `rawloader-0.37.2/src/decoders/dng.rs:35-46` and
`rawler-0.8.0/src/decoders/dng.rs:219`.

**The headline correction [V]: irradiance is no longer the only permissive,
pure-Rust DNG decoder.** gamut-dng, filmcraft-dng and flashraw all appeared in
2026. gamut-dng is at 1.0 and already claims JXL and ProRAW. The adoption
checklist's opening line ("the entire Rust RAW ecosystem is copyleft") was true on
2026-10-29 for the established crates. It is no longer true of the category.

What I did not verify: the gamut-dng and flashraw coverage claims beyond their
READMEs, and filmcraft-dng's repository (crates.io lists none).

**What [J] still differentiates irradiance:**

- zero runtime dependencies;
- `forbid(unsafe)` plus panic-free on untrusted input, enforced by fuzz smoke in CI;
- oracles that are shown red, and bit-exact plane MD5s against dnglab;
- a provenance ledger with clean-room discipline;
- `DEVELOP_VERSION` output identity and cross-arch determinism evidence;
- monochrome and instrument-grade depth.

These are quality claims, not coverage claims, so the pitch has to change.

**The `image` crate's stance [V].** Latest is 0.25.10 (2026-03-10); there is no
0.26.

- Issue #2475 (DNG decoding, closed 2025-05-27). The maintainer said the decoder "is
  best done in a separate crate / project."
- Discussion #2570 ("Discuss new file formats") holds open DNG, CR2 and NEF entries.
  It points to `image-extras` (0.1.1, 2026-05-02) for formats enabled through hooks.
- `image-extras`' criteria are pure Rust, no patent or licensing restrictions, a
  public spec, and test images. Today it holds only small formats (dds, pcx, xpm, …),
  no RAW.
- I found no statement that "RAW is out of scope."

### 1.2 How an `ImageDecoder` adapter would work

**The integration point [V].** `image::hooks` arrived in 0.25.7, which was yanked, so
0.25.8 is the first usable release. It provides:

- `register_decoding_hook(ext: OsString, hook: DecodingHook) -> bool`;
- `register_format_detection_hook(ext, signature, mask)`.

`DecodingHook` is `Box<dyn Fn(GenericReader) -> ImageResult<Box<dyn ImageDecoder>>>`.
The trait requires `dimensions`, `color_type`, `read_image` and `read_image_boxed`.
`icc_profile`, `exif_metadata`, `orientation`, `set_limits` and the rest have
defaults.

The precedent is jxl-oxide 0.12.6 and libheif-rs 3.0.0. Both use an optional `image`
feature inside the codec crate, plus a `register_*_decoding_hook()` function.

**Type mapping [V for the types, J for the choices].** The `ColorType` and
`DynamicImage` variants are L8, La8, Rgb8, Rgba8, L16, La16, Rgb16, Rgba16, Rgb32F
and Rgba32F. All are `#[non_exhaustive]`. There is no Bayer/CFA variant and no L32F.

| irradiance output | Adapter `ColorType` | Notes |
|---|---|---|
| `develop::develop_into` / `develop_with_into` (mono, u16, display-encoded) | **L16** | Clean fit. Dimensions from `output_dimensions(_with)`. |
| `color::develop_color_into` (u16, 3 samples) | **Rgb16** | Clean fit. |
| `color::develop_color_f32_into` | **Rgb32F** | Optional second mode. |
| `tone::to_linear_f32_into` (mono linear f32) | none (no L32F) | Must widen to Rgb32F (3× memory) or be dropped. Drop it. |
| `plane::unpack_into` (raw CFA mosaic) | none | `image` has no type for a mosaic. Labelling it L16 would be a lie the consumer cannot detect. **Do not expose it through `image`.** |

Other mismatches:

- **`read_image` takes `&mut [u8]`, but irradiance writes `&mut [u16]` / `&mut [f32]`.**
  The adapter needs a bytemuck-style cast of the caller's buffer, which needs
  alignment handling, or an intermediate allocation. jxl-oxide pulls in `bytemuck`
  for exactly this.
- **Orientation.** irradiance applies `Orientation` inside develop. The adapter must
  report `Orientation::NoTransforms`, or `image` will rotate a second time.
  Alternatively it can develop with `apply_orientation: false` and report the tag;
  that option exists for mono only, since SPEC-063.
- **Limits.** `set_limits` maps onto irradiance's validate-before-allocate pixel limit
  (DEC-050).
- **Format detection.** A DNG's magic bytes are TIFF's (`II*\0` / `MM\0*`), so a
  signature hook cannot tell DNG from TIFF. The adapter can register only by the
  `dng` extension. `image::load_from_memory` on DNG bytes will still route to
  `image`'s own TIFF decoder. This is inherent and worth stating in the README.
- **Coverage errors.** Every irradiance refusal becomes
  `ImageError::Unsupported`/`Decoding`, mapped from `Error::kind()`.

**Where it can live [V for policy, J for the answer].** An optional `image` feature
inside irradiance would add `image` (plus its own deps, and probably `bytemuck`) to
irradiance's dependency graph. That breaks:

- the stated "Zero dependencies by design" (`Cargo.toml`, `[dependencies]` comment);
- the `no-new-top-level-deps-without-decision` constraint;
- `cargo-deny` licence checks, which would then cover `image`'s whole default graph.

It also ties irradiance's semver to `image`'s: a public `impl ImageDecoder` exposes
`image` types, so an `image` 0.26 would force an irradiance breaking release. That
coupling is the real cost. The deny graph is secondary.

**A separate crate, `irradiance-image`, avoids all of it** [J]. It would depend on
`irradiance` and `image`, live in this repo as a workspace member or in its own repo,
and have its own semver that tracks `image`'s major. irradiance itself stays
zero-dependency. This deliberately departs from the jxl-oxide and libheif-rs
precedent; the reason is the zero-dependency policy. A workspace conversion is its
own small change (CI, `cargo package`, deny over two graphs).

**Minimal scope [J]:**

- `IrradianceDecoder::new(bytes) -> Result<Self>`, which parses and validates and
  picks mono (L16) or colour (Rgb16) from the sensor;
- `impl ImageDecoder`;
- `register_dng_decoding_hook()`;
- `exif_metadata` returning `None` for now;
- tests: the held corpus round-trips through `image::ImageReader` byte-identically to
  calling `develop_*` directly, plus one red-proof (a corrupted DNG must error, not
  produce an image).

**Effort estimate [J]:** one S/M spec of about 300–500 lines with tests. Using this
repo's cost history, that is roughly 25–45M tokens across design, build and verify.
That number excludes the workspace or second-repo setup.

### 1.3 Who would actually consume it [V unless marked]

| Project | RAW path today (HEAD 2026-10/11) | License | Would a DNG-only, permissive adapter help as is? |
|---|---|---|---|
| **crustyimg** | Has its own adapter; calls irradiance directly behind `raw-develop` (crustyimg DEC-107, preview fallback) | — | **No.** It does not need `image` integration. It is already consumer #1. |
| **oculante** | `quickraw = "0.1.6"`, non-optional, for 15 RAW extensions | MIT | Partly. It would replace quickraw (LGPL, last pushed 2025-03) only for `.dng`. Its open issues are RAF/CR3/RW2. |
| **czkawka** | `rawler = "0.8.0"` in czkawka_core | MIT (core) | **Weakly.** Its two recent DNG bugs were iPhone ProRAW and Android DNGs (#2007, closed 2026-11-03) and **JXL DNG 1.7** (#1967). irradiance cannot decode JXL, and likely not ProRAW either (below). |
| **immich** | Embedded preview, then sharp/libvips with LibRaw 0.22.2 | AGPL-3.0 | No. It is a Node/C stack and would need a sidecar. Its DNG pain is Samsung JXL and ProRAW. |
| **PhotoPrism** | sips / darktable / RawTherapee / embedded preview | AGPL-3.0 | No (Go, shells out). |
| **zenraw** | rawloader (default) / rawler | AGPL or commercial | A possible backend, but it already covers DNG through rawler. |
| Permissive crates on LGPL raw crates | rawler: rawbit, reveal, media-decode, contact-sheet, smugmug-cli, …; rawloader: simp, image-hdr, **stardetect** (astro), img4avif, skia-rs-codec | MIT/Apache | The best fit for a "license upgrade" pitch, if they mostly see DNG. I did not check per-crate whether they do. |

**Coverage reality for the users who actually show up [V from the conformance
matrix and code, J where marked]:**

- **Pixel / Android DNGs.** The plane decodes bit-exact, but **develop refuses all
  three held files** (mandatory `OpcodeList2` GainMap, DEC-049) until STAGE-016 lands.
- **JXL-compressed DNG (52546).** This is DNG 1.7, and Samsung and newer phones write
  it. It is not supported: it falls to `Compression::Other`.
- **Apple ProRAW** [J, unverified against a file]. ProRAW is LinearRaw with 3 samples
  per pixel. irradiance's sensor-IFD rule requires `SamplesPerPixel == 1`
  (`src/ifd.rs:942`), so it would be rejected. No ProRAW file is in the corpus.
- **What does develop:** Leica and Pentax monochrome DNGs, and uncompressed or SOF-3
  Bayer DNGs from converted or native-DNG cameras.

That is exactly the set the phone-DNG issues are *not* about. A permissive pure-Rust
competitor (gamut-dng) claims both ProRAW and JXL.

### 1.4 Head-to-head with the three new permissive DNG crates

**How this was checked:** a static reading of the published crate sources
(`gamut-dng-1.0.0`, `filmcraft-dng-0.9.0`, `flashraw-0.2.0` from static.crates.io),
their READMEs, and gamut-dng's `STATUS.md`. Nothing was built or run.

**Gaps in what I could see:**

- Only the top-level crates were read. gamut-dng's `gamut-ifd`/`gamut-jxl` and
  flashraw's `flashraw-core`/`-codecs`/`-formats` sub-crates were not downloaded, so
  their fuzzing, `unsafe` use and provenance are *unverified*.
- None of the crates' README claims (ProRAW, the 98% corpus figure) were run against
  the held corpus.

| | **irradiance 0.2.1** | **gamut-dng 1.0.0** | **filmcraft-dng 0.9.0** | **flashraw 0.2.0** |
|---|---|---|---|---|
| What it is [V] | DNG **developer**: bytes in, finished pixels out | DNG **container codec**, encode and decode | DNG and CinemaDNG decoder plus a simple develop | Multi-format RAW decoder and converter, ~40 formats |
| Compression [V] | uncompressed, SOF-3; strips and tiles | uncompressed, Deflate, SOF-3, **JXL**; tiles; BigTIFF | uncompressed, SOF-3; strips and tiles | CR2, **CR3**, NEF, ARW, RAF, DNG, …; README says JPEG-in-DNG is unsupported |
| Develops to pixels [V] | yes (demosaic, colour, tone, crop, orientation) | **no**: demosaic, rendering and opcode execution are out of scope (README, `STATUS.md`) | yes, to linear Rec.709 f32 RGBA | yes, RGB with ICC |
| Opcodes [V] | **applies** WarpRectilinear and FixBadPixelsConstant; **refuses** unapplied mandatory ones (DEC-049) | parses and round-trips them; does not execute them | **ignored** ("opcode lists … ignored"); no opcode code in `src/` | *unverified* |
| Demosaic [V] | Hamilton-Adams (DEC-031 replaced Malvar-He-Cutler on patent grounds) | none | **Malvar-He-Cutler** (README spec table) | *unverified* |
| Runtime deps [V] | **0** | 5 (four own sub-crates + miniz_oxide) | 2 (thiserror, rayon opt.) | 4 internal sub-crates + optional rayon, memmap2, serde, moxcms, gpu |
| `unsafe` [V] | `forbid(unsafe_code)` | `forbid(unsafe_code)` | none in `src/` (no `forbid` attribute) | yes: AVX2 kernels, mmap, a `from_raw_parts` |
| Hostile-input testing [V] | cargo-fuzz targets with CI smoke runs; red-proofs; dnglab MD5 oracles | Adobe DNG SDK and libtiff differential oracles; no fuzz dir in the package | 12,000 mutations under `catch_unwind`, every truncation, cyclic IFDs | `Limits` API; no `tests/` in the package |
| Provenance [V] | per-algorithm ledger, clean-room rule | spec plus Adobe SDK as oracle | explicit clean-room statement; LibRaw run only as an oracle tool | no provenance statement found in the facade crate |
| MSRV [V] | 1.90 | 1.92 | 1.95 | 1.85 |

**Reading of the comparison [J]:**

- **gamut-dng is a complement, not a competitor.** Its decode envelope is far wider
  and better oracled than irradiance's: JXL, ProRAW-shaped LinearRaw, BigTIFF, all
  checked against the Adobe SDK. But it stops at stored samples and leaves developing
  to downstream "raw processors."
  - That makes it a candidate **front end** for the formats irradiance does not read.
  - It would also be a cheaper answer to DEC-064 than a clean-room JXL decoder.
  - The catch: it is a dependency, so it needs a DEC and probably an optional crate or
    feature to keep the zero-dependency default.
- **filmcraft-dng is the direct competitor, and irradiance's edges are concrete:**
  - It silently ignores opcode lists. So a Pixel DNG would develop without its
    mandatory GainMap and produce wrong pixels with no error, where irradiance refuses.
  - It ships Malvar-He-Cutler, the demosaic DEC-031 rejected on patent grounds. That
    matters to commercial embedders.
  - It wins on CinemaDNG and on a one-call `decode()`.
- **flashraw plays a different game:** breadth and speed (SIMD, GPU, Python, C and
  WASM bindings). It includes CR3, which is on the patent-avoid list. It is not
  `unsafe`-free, and its provenance is unstated.
  - irradiance should not chase its format count.
  - Zero dependencies, no `unsafe`, refuse-rather-than-guess, and output identity are
    the contrast.
- **To settle it with evidence:** run all three over the held corpus.
  - Compare plane MD5s against dnglab.
  - Run a short mutation/fuzz pass on each to count panics.
  - Check what each produces for the Pixel GainMap files.

  That is about half a day. It is the "re-pitch" evidence §1.5 calls for.

### 1.5 Recommendation for Part A [J]

**Do not build the adapter yet.** Plan it as a separate `irradiance-image` crate and
gate it on two things:

1. **STAGE-016 (GainMap) ships**, so Pixel DNGs develop.
2. **A coverage decision on JXL-in-DNG and 3-sample LinearRaw (ProRAW).** JPEG XL is
   royalty-free and not on the patent-avoid list, but a pure-Rust decoder would be a
   dependency (jxl-oxide is MIT/Apache) or a large clean-room project. Either way,
   that is a DEC.

Shipping the adapter before then puts irradiance in front of `image` users with a
DNG decoder that refuses most phone DNGs, the very files those users bring. That
first impression is worse than no listing.

**What to do now instead (cheap):**

- **Retire "the only permissive RAW crate" from the pitch.** Lead with zero-dep,
  panic-free/fuzzed, bit-exact oracles and the determinism contract (`DEVELOP_VERSION`).
- **Do a half-day comparison against gamut-dng and filmcraft-dng** on the held corpus
  (§1.4): bit-exact plane checks plus panic-on-fuzz. If irradiance is not measurably
  better on correctness or safety, the adapter's value proposition is weak, whatever
  its coverage.
- **Evaluate gamut-dng as an optional decode front end** for JXL and ProRAW, with
  irradiance developing its output. This feeds DEC-064.

**First outreach target, once gated:**

- **oculante** (MIT; quickraw is LGPL and stale; the maintainer merges community PRs),
  as a `.dng`-only swap behind a feature.
- **stardetect** and the other astro or permissive rawloader dependants, where linear,
  monochrome and bit-exact output is what they need.
- Post in **image-rs Discussion #2570**'s DNG entry when the crate exists.
- Hold czkawka and immich until JXL and ProRAW are covered.

---

## 2. Part B: public API and semver policy

### 2.1 The draft's hazards, re-verified on `bd0e0df`

| Item | State now [V] | Status |
|---|---|---|
| `opcode::Opcode` "non_exhaustive-free by choice" | `#[non_exhaustive]` since DEC-052 (0.2.0); the comment was rewritten (`src/opcode.rs:101-110`) | **Resolved** |
| `ifd::Sensor`, `ifd::Cfa`, `opcode::WarpRect` | `#[non_exhaustive]` (DEC-049, DEC-052); constructors behind the dev-only `test-fixtures` feature | **Resolved** |
| `Error`, `ErrorKind`, `UnsupportedPreview`, `PreviewKind`, `Preview`, `DevelopPath`, `DevelopOptions`, `Framing` | `#[non_exhaustive]` | Fine |
| `Entry`, `Ifd`, `Container`, `Previews` | Private fields with accessors | Fine |
| **`ifd::Compression`** (`src/ifd.rs:492`) | **Not sealed.** Variants are `Uncompressed`, `Jpeg`, `Other(u32)`. The doc comment is stale ("Jpeg … rejected cleanly here"); PROJ-003 decodes it. | **Open** |
| **`ifd::SensorMatch`** (`:952`) | Not sealed: `Yes` / `No` / `Unreadable(u16)` | Open, but see below |
| **`ifd::ByteOrder`** (`:274`) | Not sealed: `Little` / `Big` | Open, but see below |
| **`Rational {num, den}`, `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize`** | Public fields, not sealed, externally constructible | Open, but see below |
| **`Error`'s 58 struct-like variants** (new finding) | The enum is sealed, but no variant is. Adding a field to an existing variant is breaking, and outsiders can construct variants. | **Open** |
| **Public field *types* on sealed `Sensor`** (new finding) | `#[non_exhaustive]` protects adding fields, not changing a type. Examples: `strip_offsets: Vec<u32>` (BigTIFF would want u64), the scalar `black_level: Option<u32>` beside the per-channel `black_levels: Vec<u32>`. | Open, a policy item |
| `DEVELOP_VERSION` as the output signal | Exists ("4"). Introduced in DEC-028, bumped by DEC-038 (2), DEC-040 (3), and the orientation change (4). The CHANGELOG notes "unchanged" per release. No single written contract; its scope across entry points is implicit. | **Open** (§2.3) |
| `cargo-semver-checks` | Not in `.github/workflows/ci.yml` | Open |

### 2.2 Q1: which types get `#[non_exhaustive]` or getters

In 0.x, adding `#[non_exhaustive]` to an existing public enum or struct is a breaking
change, so all of these belong in the **next breaking minor (0.3.0)** as one batch,
not in a 0.2.x patch.

| Type | Recommendation [J] | Cost of being wrong |
|---|---|---|
| `Compression` | **Seal it.** Fix the stale doc. Named variants are coming: Deflate (8) and JXL (52546) are real DNG 1.7 codes, and promoting a code out of `Other(n)` is both a type break and a silent match change. | High. Without the seal, every new codec is a forced breaking release. |
| `Error` variants | **Seal each struct-like variant** with variant-level `#[non_exhaustive]`, in the same 0.3.0. External matches then need `..`. If that is too noisy, adopt the policy rule instead: "never add a field to an existing variant; add a new variant." | Medium. Without either, enriching an error message is breaking. |
| `SensorMatch` | **Leave exhaustive, deliberately.** A closed tri-state is the point (DEC-012): callers must handle `Unreadable`. Document "closed by design." | Low. A 4th state would be a semantic change that deserves a break anyway. |
| `ByteOrder` | **Leave exhaustive.** TIFF and BigTIFF define exactly `II` and `MM`. | Negligible. |
| `Rational`, `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize` | **Leave as plain value types** with public fields. Document them as frozen, spec-defined shapes. Sealing would forbid callers building their own crop rectangles, which is a legitimate use. | Low. These shapes are fixed by TIFF/DNG. |
| Sensor field types | **Policy, not code.** Changing a public field's type is a breaking change. New representations arrive as *new* fields (the `black_levels` precedent), and old ones are deprecated, not retyped. | Medium if ignored. BigTIFF or u64 offsets would force a break. |

### 2.3 Q2: is `DEVELOP_VERSION` part of the public contract?

**Yes, and it already is in practice** [V]. crustyimg keys its build cache on it
(`src/lib.rs:146-157`), and every release note already states whether it moved.
What is missing is a written definition and a scope. Draft wording for
`docs/stability.md`:

> **Output identity.** For a fixed tuple (`DEVELOP_VERSION`, entry point, input
> bytes, options value), every *output-producing* public function returns
> bit-identical output on every supported platform and thread count. This holds
> across all crate versions that carry the same `DEVELOP_VERSION`.
>
> The output-producing functions are:
> - `develop::develop_into` and `develop_with_into`;
> - `color::develop_color_into`, `develop_color_f32_into` and their `_threaded`
>   variants;
> - `demosaic::demosaic_into` and `demosaic_into_threaded`;
> - `color::to_srgb_linear_into`;
> - `tone::*_into`;
> - `warp::apply_warp_into`;
> - `develop::apply_fix_bad_pixels_constant`;
> - `plane::unpack_into`.
>
> **An output-changing release** is one where any of those functions returns
> different bytes, or different output dimensions, for an input it previously
> accepted, under the same options. This includes bug fixes.
>
> **Not output-changing:**
> - an input that was previously *refused* with an `Error` and now succeeds (new
>   coverage);
> - an input that was accepted and is now refused, which is a behavioural fix
>   recorded under **Fixed**;
> - new options whose default reproduces the old output (the DEC-058 pattern).
>
> **Rules:**
> 1. Every output-changing release increments `DEVELOP_VERSION` by one, in the
>    same commit as the change.
> 2. It is **never a patch release.** Before 1.0 it ships in the next `0.y.0`, which
>    Cargo treats as breaking, so `^0.2` users are not moved silently. After 1.0 it
>    ships in a minor.
> 3. It is recorded in `CHANGELOG.md` under a dedicated heading,
>    `### Output changed (DEVELOP_VERSION N → N+1)`, listing each affected entry
>    point and which inputs move.
> 4. A release with no such heading asserts that `DEVELOP_VERSION` is unchanged.

Points to decide:

- **Rule 2 is the real choice** [J]. It is strict: a correctness fix to output must
  wait for a 0.y.0. The alternative ("bump `DEVELOP_VERSION`, allow it in a patch")
  is weaker for the reproducibility audience (ML and archives) that is the point of
  the contract.
  - Cost of choosing strict wrongly: some fixes sit a few weeks longer.
  - Cost of choosing loose wrongly: a researcher's `^0.2` dataset silently changes
    pixels on `cargo update`.
- **Including `plane::unpack_into`** [J]. Its output is pinned to dnglab MD5s, so a
  change there is almost certainly a bug fix, but it is still an output change.
- **The 0.2.1 history [V].** It changed no output; the CHANGELOG says so explicitly.
  The 3 → 4 bump shipped in 0.2.0, a minor. So rule 2 matches the history so far.

### 2.4 Q3: the pre-1.0 policy, and `cargo-semver-checks`

**1.0 gates [J]:**

1. At least two independent consumers have used a 0.y line for at least one full
   release without asking for a break. crustyimg is #1. A second must be
   non-maintainer: pyirradiance counts only partly, monorail/onyx is the maintainer's
   own, so an external crate such as oculante or astro would count.
2. The 0.3.0 seal batch from §2.2 has landed.
3. `docs/stability.md` exists, with the output-identity contract, the non-promises
   (`fuzzing`/`test-fixtures`/`irr` features, `#[doc(hidden)]` and `__`-prefixed
   items, error *message* text, performance, camera coverage) and an MSRV rule
   ("raised only in a minor, noted in the CHANGELOG").
4. The coverage story is honest at the README's first screen, including phone DNG
   status (§1.3).
5. `cargo-semver-checks` has been green across at least one minor release.

**`cargo-semver-checks` [V for the facts, J for the recommendation].**

- Facts: v0.51.0 (2026-10-03); the action is `obi1kenobi/cargo-semver-checks-action@v2`
  (v2.9, 2026-06-29). It diffs rustdoc JSON against the published crates.io baseline.
- Known false negatives: field and parameter *type* changes, generics and lifetimes,
  and feature-combination-only breaks. It does not see behaviour or output changes
  (that last point is my inference from its design).

**Recommendation: add it as a CI job.** It is low cost, and it is the only gate that
catches an accidental signature break before `cargo publish`.

Configuration:

- Baseline is the latest crates.io release.
- Run with default features only: `fuzzing`/`test-fixtures`/`irr` are outside the
  promise.
- Fail on a break unless the PR's Cargo version is a 0.y bump.

It needs a red-proof per `oracle-must-be-shown-red`: a fixture branch that removes a
public fn must turn the job red. It does **not** cover `DEVELOP_VERSION`; the
existing MD5-pinned corpus tests do. The two together are the contract.

### 2.5 Q4: pyirradiance versioning

The task calls it "pyradiance"; the repo's feedback file calls the binding
`pyirradiance` (`feedback/2026-11-02-pyirradiance-requests.md`). Use one name.

What the precedents show [V, PyPI and crates.io, 2026-11-12]:

- **Independent semver is the norm** when the Python package wraps a reusable crate:
  - polars: Python 2.1.0 wraps crate 0.55.2;
  - tokenizers: Python 0.23.3 wraps crate 1.0.0-rc.4;
  - rpds-py: calver Python wraps crate 1.2.1.
- **Lockstep** only happens where the crate exists for the binding (jiter, orjson).
- **Exposing the wrapped version is rare.** Of the projects checked, only pydantic does
  it (`version_info()`, plus an exact pin enforced at import).
- **abi3** is used where the build matrix matters:
  - polars `cp311-abi3`, cryptography `cp39-abi3`/`cp311-abi3`, tokenizers `cp310-abi3`;
  - free-threaded builds need separate `cp314t` wheels (maturin cannot build abi3t
    for 3.14);
  - performance-tuned crates (orjson, jiter, pydantic-core) ship per-version wheels.
  - The limited API gates the buffer protocol to ≥3.11 (pyo3 docs).

**Recommendation [J]:**

1. **Independent semver for pyirradiance.** Pin irradiance **exactly** (`=0.y.z`) in
   its `Cargo.toml`, the pydantic pattern, so each wheel maps to one crate version.
2. **Expose both `pyirradiance.__irradiance_version__` and `pyirradiance.DEVELOP_VERSION`.**
   Precedent for this is thin, but it is right here: output identity is the product
   for the ML and archive audience, and the crate version alone does not carry it.
3. **Mirror the bump rule.** An irradiance output change is at least a pyirradiance
   minor and repeats the `Output changed` heading. An irradiance 0.y bump that leaves
   the Python surface intact needs only a pyirradiance minor.
4. **Ship abi3 wheels at `abi3-py311`.** Use 3.11 if numpy zero-copy goes through the
   buffer protocol; otherwise 3.10 suffices. The work is memcpy-bound Rust, so abi3's
   lost interpreter-specific optimizations do not matter. Add a separate `cp314t`
   wheel only when someone asks.

I did not check pyirradiance's current version or its wheel tags. This is a
pyirradiance-repo decision; irradiance should only record the contract it exports.

---

## 3. Summary

**Part A.** The Rust RAW landscape changed under the adoption plan. Three permissive,
pure-Rust DNG decoders shipped in 2026: gamut-dng (1.0, claims JXL and ProRAW),
filmcraft-dng and flashraw. "The only permissive RAW crate" is no longer a true pitch.

`image` explicitly wants RAW decoders out of tree and offers `image::hooks`
(≥0.25.8) as the plug. An adapter must be a **separate `irradiance-image` crate**: an
in-crate `image` feature would break the zero-dependency rule and tie irradiance's
semver to `image`'s. It maps cleanly only for develop output (L16 mono, Rgb16 or
Rgb32F colour), never for the raw mosaic, and can register only by file extension,
because DNG's magic bytes are TIFF's.

But the users who would find it bring phone DNGs. Those are Pixel (develop refused
until the GainMap stage), JXL DNG 1.7 (unsupported) and ProRAW (likely refused:
3-sample LinearRaw). So the honest answer is **not yet**:

- build it after STAGE-016 and a JXL/ProRAW coverage decision, about one S/M spec;
- meanwhile, re-pitch on zero-dep, panic-free and bit-exact determinism, and
  benchmark correctness and safety against gamut-dng and filmcraft-dng;
- on a source reading (§1.4), gamut-dng is a decode-only codec that could front
  irradiance, not a rival developer. filmcraft-dng is the direct rival, and it
  silently ignores opcodes and ships the patented Malvar-He-Cutler demosaic;
- first outreach target is oculante's `.dng` path, then astro rawloader dependants
  and image-rs Discussion #2570.

**Part B.** 0.2.0 already sealed `Opcode`, `Sensor`, `Cfa` and `WarpRect`, so most of
the old draft's hazards are gone. What remains:

- **`Compression` must be sealed.** Deflate and JXL variants are coming.
- **`Error`'s 58 struct-like variants** need variant-level seals or a "new variant,
  never a new field" rule.
- `SensorMatch`, `ByteOrder` and the TIFF value structs should stay open on purpose,
  and say so.
- Public field *types* on `Sensor` need a "never retype, add a new field" rule.

All of that goes in one 0.3.0 batch.

`DEVELOP_VERSION` is already a de facto contract (crustyimg's cache key). It needs
written scope and rules:

- bump on any byte change from a previously accepted input;
- never in a patch;
- record it under `### Output changed (DEVELOP_VERSION N → N+1)`.

Add `cargo-semver-checks` to CI with a red-proof. Gate 1.0 on a second external
consumer, the seal batch, `docs/stability.md`, and one green semver-checks minor.

pyirradiance should use independent semver, pin irradiance exactly, expose
`__irradiance_version__` and `DEVELOP_VERSION`, and ship abi3-py311 wheels.

## 4. Decisions that need the maintainer's call

The IDs are indicative only. DEC-059 is next-free on `bd0e0df`; mint at write time.

| # | Draft DEC title | Decision needed |
|---|---|---|
| 1 | **DEC-059: `DEVELOP_VERSION` is a public output-identity contract; output changes never ship in a patch** | Strict rule 2 vs bump-in-patch; whether `plane::unpack_into` is in scope; the CHANGELOG heading |
| 2 | **DEC-060: The 0.3.0 seal batch: `Compression` and the `Error` variants sealed; `SensorMatch`, `ByteOrder` and the TIFF value structs closed by design** | Variant-level seal vs policy-only for `Error`; the "never retype a public field" rule |
| 3 | **DEC-061: `cargo-semver-checks` joins the CI gate against the crates.io baseline** | Default features only; red-proof shape |
| 4 | **DEC-062: Stability policy and 1.0 criteria** (`docs/stability.md`) | What counts as an "independent consumer"; the MSRV rule; the non-promise list. Could fold into #1. |
| 5 | **DEC-063: The `image` integration is a separate `irradiance-image` crate, deferred until phone-DNG coverage** | Workspace member vs separate repo; the gate (STAGE-016 plus a JXL/ProRAW decision) |
| 6 | **DEC-064: JXL-compressed DNG and 3-sample LinearRaw (ProRAW): in scope or not** | A dependency (jxl-oxide, MIT/Apache, or gamut-dng as a decode front end, §1.4) vs clean-room vs out of scope; a ProRAW file is needed first |
| — | pyirradiance versioning (exact pin, `__irradiance_version__`, abi3-py311) | Belongs to the pyirradiance repo, not an irradiance DEC |
| — | Adoption pitch rewrite (drop "only permissive") | Not a DEC: README and checklist edits |
