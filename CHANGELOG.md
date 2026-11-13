# Changelog

All notable changes to this app. Newest at top.

Keep an `## [Unreleased]` section on top and add entries under it as you work —
the patch lane files fixes here (`[Unreleased] → Fixed`), and a release spec
(DEC-006) promotes the section to a version heading when it cuts the tag.

## [Unreleased]

### Added

- **CFA as mono: per-position black levels, `DevelopOptions::black_levels`**
  (`irradiance::develop`, SPEC-066, STAGE-019, PROJ-004, DEC-061). A converted
  (CFA-removed) camera still declares `CFAPattern`; its plane already develops on the mono
  path, and now it can take its declared per-position black. `black_levels:
  BlackLevels::FirstValue | BlackLevels::PerPosition` (`#[non_exhaustive]`, default
  `FirstValue`). `PerPosition` indexes `BlackLevel` by `BlackLevelRepeatDim` (`[1, 1]` when
  absent or malformed), phased from the `ActiveArea` origin, on raw samples before the
  warp; white stays per sample. No allocation on any entry point. Output is
  **unchanged by default**, and every held `LinearRaw` file declares a 1×1 pattern, so it
  develops byte-identically under both values; `DEVELOP_VERSION` stays `"4"`. On the
  Canon 5D3 (`[2047, 2047, 2047, 2048]`) `PerPosition` changes 5,602,332 of 22,430,520
  active samples, by at most 6.
- **`Error::InvalidBlackLevelPattern { repeat_rows, repeat_cols, count }`** (additive;
  `ErrorKind::Corrupt`). Under `BlackLevels::PerPosition`, a `BlackLevel` count other than
  `BlackLevelRepeatDim`'s `rows × cols` (or a zero or overflowing dimension) is refused,
  where `InvalidLevels` is reported, before any write. Validated only when the option
  uses the pattern (SPEC-066, DEC-061).
- **Develop options: full active area and unrotated output** (`irradiance::develop`,
  SPEC-063, STAGE-019, PROJ-004, DEC-058). `develop_with_into(sensor, src, dst, &options)`
  and `output_dimensions_with(sensor, &options)` take a `DevelopOptions`:
  `framing: Framing::StandardCrop | Framing::FullActiveArea` keeps the `DefaultCrop` or
  the whole `ActiveArea` (every recorded photosite), and `apply_orientation: bool`
  applies `Orientation` or leaves pixels in stored order at the unswapped size. Options
  rewrite the geometry after it is validated, so a malformed `DefaultCrop` or
  `Orientation` is still refused under every option. `DevelopOptions` and `Framing` are
  `#[non_exhaustive]`; build options from `DevelopOptions::default()` and assign fields.
  Mono only; the colour path takes no options. `develop_into` and `output_dimensions`
  now delegate with `DevelopOptions::default()` and their output is **unchanged**
  (MD5-pinned on eight held files), so `DEVELOP_VERSION` stays `"4"`.
- **`Error::UnsupportedWarpedFullActiveArea { active_width, active_height, warp_alpha,
  scale_min, scale_max }`** (additive; `ErrorKind::Unsupported`). `FullActiveArea` on a
  sensor whose `OpcodeList3` carries a `WarpRectilinear` is refused, typed, before any
  write to `dst`, only when the warp (lerped to `warp_alpha`) could sample outside the
  image, where its clamped edge samples would be visible: its radial scale leaves
  `[0, 1]` on `r² ∈ [0, 1]`, its optical centre is outside the image, or a coefficient
  is not finite (SPEC-064, DEC-059). `scale_min` and `scale_max` are the scale's exact
  extrema. Every held Leica Q2 Monochrom frame passes, so `FullActiveArea` develops
  them at every `warp_alpha`; the Pixel 3a's over-unity polynomial is refused.
- **Partial lens correction: `DevelopOptions::warp_alpha`** (`irradiance::develop`,
  SPEC-064, STAGE-019, PROJ-004, DEC-059). A value in `[0, 1]`, default `1.0` (the full
  `WarpRectilinear` correction, today's output). The develop applies
  `warp.lerp(warp_alpha)`: every pixel moves `warp_alpha` times as far as under the
  full warp, and `0.0` applies no warp at all (its tangential terms included). The
  default leaves every existing output unchanged, so `DEVELOP_VERSION` stays `"4"`.
- **`WarpRect::lerp(alpha)`** (`irradiance::opcode`, SPEC-064, DEC-059): the warp whose
  backward radial scale is `(1 − α) + α·f(r)` (`kr0' = (1 − α) + α·kr0`, the other
  coefficients times `α`, centre and plane count unchanged). Bit-identical at `α = 1`,
  the identity at `α = 0`. It does not validate `alpha`.
- **`Error::InvalidWarpAlpha { warp_alpha }`** (additive; `ErrorKind::CallerError`). A
  `warp_alpha` that is NaN, infinite or outside `[0, 1]` (`-0.0` is accepted), reported
  first by `develop_with_into` and `output_dimensions_with`, under every framing and
  whether or not the file carries a warp.

- **Caller-owned develop scratch, allocation-free** (`irradiance::develop`, SPEC-065,
  STAGE-019, PROJ-004, DEC-060). `scratch_len(sensor, &options)` reports how many `u16`
  samples of scratch a mono develop needs (0 when the file carries neither
  `FixBadPixelsConstant` nor a non-identity `WarpRectilinear`), and
  `develop_with_scratch_into(sensor, src, dst, scratch, &options)` develops into that
  caller-owned slice with **no heap allocation at all**, on success and on every error
  path. The slice must be exactly `scratch_len` long; its contents are unspecified on
  entry and return and never affect the output. Output and errors are identical to
  `develop_with_into`. Ask for `scratch_len` rather than hard-coding it: a later version
  may change it without changing the output. On a held Leica Q2 Monochrom frame it is
  94,707,712 samples (47,443,968 at `warp_alpha = 0`).
- **`Error::DevelopScratchWrongLength { expected, actual }`** (additive;
  `ErrorKind::CallerError`). A scratch whose length is not exactly `scratch_len`,
  checked after both opcode lists are parsed and before any write.

- **Embedded preview access, zero-copy** (`irradiance::ifd`, SPEC-062, STAGE-018,
  PROJ-004, DEC-057). `Container::previews()` iterates a DNG's embedded previews: every
  IFD whose `NewSubfileType` is exactly 1 (`PreviewKind::Primary`) or 65537
  (`PreviewKind::Alternate`), in walk order. Each `Preview` carries its IFD index,
  dimensions, `Compression`, `PhotometricInterpretation`, `SamplesPerPixel`,
  `BitsPerSample`, file offset, and `bytes`: one slice borrowed from the caller's
  input, never copied and never decoded. Several strips are joined when each starts
  where the previous one ends. Each item is a `Result`, so one bad preview is an `Err`
  and iteration continues. The accessor allocates nothing. All 21 previews in the 11
  held DNGs match exiftool 13.55. New public items: `ifd::Preview`, `ifd::PreviewKind`,
  `ifd::Previews` (the first two `#[non_exhaustive]`).
- **Two new `Error` variants and `UnsupportedPreview`** (additive; `Error` is
  `#[non_exhaustive]`): `UnsupportedPreviewLayout { ifd_index, layout }` for a tiled,
  `JPEGInterchangeFormat` or non-contiguous-strip preview (`ErrorKind::Unsupported`),
  and `InvalidPreviewLayout { ifd_index, strip_offsets, strip_byte_counts }` for a
  missing, mismatched or empty strip table (`ErrorKind::Corrupt`). `UnsupportedPreview`
  (`Tiled`, `JpegInterchangeFormat`, `NonContiguousStrips`) is `#[non_exhaustive]`.

- **Tiled sensor planes, uncompressed and lossless-JPEG** (`irradiance::plane`,
  SPEC-058, STAGE-015, PROJ-004, DEC-056). `plane::validate` and `plane::unpack_into`
  now read planes stored as TIFF 6.0 §15 tiles instead of strips: uncompressed tiles
  (8/12/14/16-bit, both byte orders) and Compression-7 (SOF-3 lossless JPEG, `Nf = 1`
  or `2`) tiles, including partial right and bottom edge tiles. Edge tiles are coded at
  full size and clipped: a Compression-7 tile is reconstructed in place through a
  clipped, strided window, so `unpack_into` still allocates nothing (DEC-016). The
  three held Google Pixel DNGs (4a, 3a, 7 Pro; 192 tiles of 256×256) unpack bit-exact
  against `dnglab --raw-checksum`; they still refuse to develop
  (`UnsupportedMandatoryOpcode`, their mandatory GainMaps; DEC-049). Every strip file
  decodes byte-identically.
- **`Sensor` gains `tile_width`, `tile_length`, `tile_offsets`, `tile_byte_counts`**, and
  `irradiance::ifd` gains `TAG_TILE_WIDTH`, `TAG_TILE_LENGTH`, `TAG_TILE_OFFSETS`,
  `TAG_TILE_BYTE_COUNTS` (additive; `Sensor` is `#[non_exhaustive]`). The tile tags are
  structural (DEC-012): a malformed one, e.g. a RATIONAL `TileOffsets`, fails the read.
- **Three new `Error` variants** (additive; `Error` is `#[non_exhaustive]`):
  `InvalidTileLayout` (a zero or absent tile dimension, or tile tables that do not match
  the tile grid; `ErrorKind::Corrupt`), `StripsAndTiles` (both layouts on one plane;
  `Corrupt`), and `UnsupportedTileRowBits` (an uncompressed tile row that is not a whole
  number of bytes; `Unsupported`).

### Changed

- **`develop_into` / `develop_with_into` allocate at most once** (SPEC-065, DEC-060).
  They now run the same core as `develop_with_scratch_into`, feeding it one internal
  allocation of exactly `scratch_len` samples (none when that is 0). The warp output
  reuses the dead fixed-plane copy, so peak working memory on a file with both
  `FixBadPixelsConstant` and a warp drops from P + 2A to P + A samples. On a Leica Q2
  Monochrom frame: **1 allocation of 189,415,424 bytes, down from 6 totalling
  283,943,504**. Output is unchanged (MD5-pinned), so `DEVELOP_VERSION` stays `"4"`.
- **`opcode::parse_fix_bad_pixels_constant` and `opcode::parse_warp_rectilinear` no
  longer allocate** (SPEC-065). They walk the raw opcode framing instead of building
  `parse_opcode_list`'s `Vec`, with identical results and errors: every opcode is still
  read and every recognised one validated. `develop::apply_fix_bad_pixels_constant`
  allocates nothing either.
- `UnsupportedStripLayout`'s message no longer says tiled planes are unsupported; it
  now fires only for a plane with neither valid strip tags nor tile tags.

## [0.2.1] - 2026-11-11

Post-0.2.0 maintenance (STAGE-014, SPEC-056, DEC-053). It closes two of 0.2.0's
Known limitations: the two required-processing gaps. No pixel output changes for any
file that still develops (`DEVELOP_VERSION` unchanged), and the public surface only
grows. The `⚠` entry under **Changed** is the one that can turn a former `Ok` into a
typed `Err`. Tier-B corpus evidence was re-run by the cold verify: every held file
is byte-identical.

### Fixed

- **An `OpcodeList` tag that is present but unreadable now refuses the develop**
  (`irradiance::opcode`, SPEC-056, STAGE-014, PROJ-002; closes a 0.2.0 Known
  limitation) — a list whose payload offset or length does not fit the file was read
  as absent (DEC-012), so a file declaring required processing in a list this library
  could not read developed without it. Parsing stays tolerant (the file reads, the
  field is `None`, the tag is in `Sensor::malformed_tags`), but every `develop_*`
  entry and the colour stage primitives now return the new
  **`Error::UnreadableOpcodeList { list }`** (`ErrorKind::Corrupt`; additive, `Error`
  is `#[non_exhaustive]`). Lists are checked in order 1, 2, 3. No held corpus file
  is affected: every one still develops byte-identically, and `DEVELOP_VERSION` is
  unchanged. `DEC-053`.
- **The Sensor-taking stage primitives now run the required-processing check**
  (SPEC-056; closes a 0.2.0 Known limitation) — see **Changed**.
- **`cargo doc` with warnings denied is clean** (SPEC-056) — 19 public docs linked
  private items (rendered as plain text on docs.rs); they are reworded, not
  silenced. New gate `just doc-check` (CI `docs`) and its mutation red-proof
  `just doc-check-red-proof` (CI `docs-red-proof`).
- **The nested-cargo red-proof tests work with `CARGO_TARGET_DIR` set** (SPEC-056)
  — all 7 staged probe builds now build in their own `target/`, so the probe is
  found and a mutated staged crate never builds into the caller's target dir.

### Added

- **`opcode::check_required_processing(sensor, path)`** and the
  **`opcode::DevelopPath`** enum (`Mono`, `Colour`; `#[non_exhaustive]`)
  (SPEC-056, DEC-053) — the check every develop entry asks first, public so a caller
  composing the primitives itself (in particular the `Sensor`-less opcode appliers
  `warp::apply_warp_into` and `develop::apply_fix_bad_pixels_constant`) can ask the
  same question.

### Changed

- **⚠ `demosaic::demosaic_into`, `demosaic::demosaic_into_threaded` and
  `color::to_srgb_linear_into` now refuse** a `Sensor` that declares processing the
  colour path does not apply — a mandatory opcode (`UnsupportedRequiredOpcode` /
  `UnsupportedMandatoryOpcode`) or an unreadable list (`UnreadableOpcodeList`) —
  before any work, exactly as `develop_color_into` does (SPEC-056, DEC-053). Files
  without opcode lists, or with optional-only lists, are unaffected.

## [0.2.0] - 2026-11-10

The Bayer colour release (SPEC-042, DEC-052): everything since `0.1.1` — the Bayer
mosaic, Hamilton-Adams demosaic and colour develop (PROJ-002), DNG `Compression 7`
lossless-JPEG decode (PROJ-003), and the pre-0.2.0 review hardening (STAGE-010:
SPEC-052/053/054/055/039/040/041). Semver: in `0.x` a minor bump is the
compatibility boundary, so consumers move from `"0.1"` to `"0.2"` deliberately;
the `⚠` entries under **Changed** are the ones that can need a code change. Tier-B
corpus evidence was re-run on the release candidate (every held file bit-exact).
No colour-*accuracy* claim is made yet — see **Known limitations**.

### Security

- **A forged sensor size can no longer drive the caller into an out-of-memory
  abort** (`irradiance::plane`, SPEC-039, PROJ-002, pre-0.2.0 review finding #5) —
  `plane::unpack_into` checks the layout only after the caller has allocated `dst`,
  so a forged `ImageWidth`/`ImageLength` made the caller request gigabytes first,
  and the abort is uncatchable from a host such as a Python worker. The new
  preflight **`plane::validate(sensor, byte_order, file, max_pixels)`** runs before
  any allocation: it checks the caller's own pixel budget **first** (on both the
  uncompressed and the Compression-7 path — for a compressed plane it is the only
  bound available, since a few hundred compressed bytes legitimately expand to a
  multi-megapixel plane), then every pre-decode check `unpack_into` runs, through
  the same shared helpers. `max_pixels` is the caller's number, not a library cap
  (`usize::MAX` = no limit). `DEC-050`.

- **The container parser can no longer be amplified by aliased IFD tables**
  (`irradiance::ifd`, SPEC-052, PROJ-002, pre-0.2.0 review finding #1) — IFD entry
  tables at *distinct* offsets may overlap the same file bytes, so a small hostile
  file could make `Container::parse` read and allocate millions of entries (the
  reviewer measured ~6.3M entries, ~72 MiB, from a ~770 KB file). The walk now keeps
  a running total of every IFD's `count × 12` table bytes and rejects the file, before
  allocating, once that total exceeds the file length — so the entries parsed are
  bounded by `file_len / 12`. New typed error
  **`Error::IfdTableBytesExceedFile { declared, file_len }`** (additive; `Error` is
  `#[non_exhaustive]`). Well-formed files are unaffected: their tables are disjoint
  (the corpus DNGs' tables are ~0.003 % of the file). `MAX_IFDS`, `MAX_IFD_DEPTH`,
  `MAX_TAG_VALUES` and the cycle guard are unchanged. `DEC-046`.
- **The licence gate now checks dev-dependencies** (`deny.toml`, SPEC-055, PROJ-002,
  pre-0.2.0 review finding #4) — the policy says no copyleft dependency of any kind,
  dev-dependencies included, but cargo-deny's licence check skips dev-dependencies
  by default: `just deny` gathered all 48 crates of the root graph and evaluated the
  licence of one (`irradiance`), so a GPL dev-dependency passed both `just deny` and
  `just deny-fuzz`. `deny.toml` now sets `[licenses] include-dev = true`, and the
  real tree still passes on both graphs (`toml`, `ssimulacra2` and their transitives
  are all permissive). New mutation red-proof `scripts/deny-red-proof.sh`
  (`just deny-red-proof`, CI `licenses-red-proof`): a synthetic GPL-3.0-only
  dev-dependency is rejected by name in the root and the fuzz graph, and passes with
  the knob removed. No dependency added. `DEC-048`.

- **A wrong-shape Compression-7 strip is now refused instead of silently
  mis-decoded** (`irradiance::plane`, SPEC-053, PROJ-002, pre-0.2.0 review finding
  #2) — `unpack_into` sized each lossless-JPEG strip's row band to `rows × width`
  and checked only that the SOF-3 frame's *area* matched, so a frame that is an
  equal-area transpose of the strip (say 4 × 6 in a 6 × 4 band) decoded with the
  wrong row stride and returned `Ok` with a garbled plane. The frame's effective
  shape (`samples_per_line × components` by `lines`) is now bound to the strip's
  declared width and rows before any sample is decoded; a mismatch — including an
  unequal-area one, which used to surface as `LjpegPlaneWrongLength` — is
  **`Error::LjpegPlaneShapeMismatch { expected_width, expected_height, decoded_width,
  decoded_height }`**. Separately, a SOF-3 component declaring sampling factors
  other than `H = V = 1` was decoded as if it were `H = V = 1`; it is now
  **`Error::LjpegUnsupportedSampling { factor }`**. Both variants are additive
  (`Error` is `#[non_exhaustive]`). Correctly shaped `H = V = 1` files decode
  byte-identically — both held Compression-7 DNGs still match their pinned `dnglab`
  checksums — so `DEVELOP_VERSION` is unchanged. `DEC-047`.

### Fixed

- **A DNG that requires processing the develop does not perform is now refused,
  not silently mis-developed** (`irradiance::develop`, `irradiance::color`,
  SPEC-054, PROJ-002, pre-0.2.0 review finding #3) — DNG opcodes carry a `Flags`
  word; one without the optional bit is *required* processing. Previously a
  required opcode in `OpcodeList2` (dropped on read), any required opcode on the
  colour path (which applies none), a recognized opcode in a list its applier does
  not read, or a second required instance beyond the one applied, developed to a
  plausible-looking wrong image and `Ok(())`. Every public develop entry —
  `develop_into`, `develop_color_into`, `develop_color_f32_into`,
  `develop_color_into_threaded`, `develop_color_f32_into_threaded` — now checks
  every present opcode list first, before allocating, and returns the new typed
  **`Error::UnsupportedRequiredOpcode { list, id }`** (additive; `Error` is
  `#[non_exhaustive]`). An unrecognized required opcode keeps its existing
  `Error::UnsupportedMandatoryOpcode`; a broken opcode list is `Error::Truncated`.
  Optional opcodes are still skipped. No change to any develop that still succeeds
  (no `DEVELOP_VERSION` bump): the whole tier-B corpus develops byte-identically.
  `DEC-049`.

### Added

- **`Error::kind()` and `ErrorKind { Unsupported, Corrupt, CallerError }`**
  (`irradiance`, SPEC-040, PROJ-002) — a stable three-way classification of every
  `Error`, so a batch consumer can route a failure without matching ~55 variants:
  `Unsupported` (a well-formed file this library does not decode — skip it),
  `Corrupt` (malformed, truncated or hostile bytes — a bad file), `CallerError` (a
  wrong-length buffer or the caller's own `max_pixels` budget — fix the call).
  `ErrorKind` is `#[non_exhaustive]`; only the three kinds are a stability promise,
  and an individual variant may be re-classified later. `kind()` is a wildcard-free
  match, so a new `Error` variant cannot compile until it is classified. Additive:
  no `Error` variant, `Display` text or error-production site changed. `DEC-051`.
- **Cross-arch determinism evidence** (`tests/determinism.rs`, SPEC-041, PROJ-002) —
  committed 64-bit FNV-1a hashes of `plane::unpack_into`, `demosaic::demosaic_into`
  and `tone::to_linear_f32_into` output over four tier-A synthetic Bayer fixtures
  (RGGB/BGGR/GRBG/GBRG at 16/14/12/8-bit, both byte orders), asserted by a new CI
  `determinism` job on `ubuntu-latest` (x86_64) **and** `macos-latest` (arm64), in
  the dev and release profiles. The oracle is shown red (a one-bit sample flip moves
  all three hashes). `docs/determinism.md` names every float path: `to_linear_f32_into`
  is one correctly-rounded IEEE `f32` division; the `f64` colour (`DEC-033`) and warp
  paths are named as **unmeasured**. No library code changed; no new dependency.

- **`plane::validate`** (`irradiance::plane`, SPEC-039) — the allocation-free,
  panic-free validate-before-allocate preflight: returns `Ok(n)` (`n = width ×
  height`, the exact `dst` length `unpack_into` wants) or the same typed error
  `unpack_into` would return, plus the budget refusal. For an uncompressed plane
  `Ok(n)` guarantees the unpack succeeds and `n × bits ≤ 8 × file.len()`; for a
  Compression-7 plane it covers the structural checks, and the SOF-3 frame-shape
  binding (`DEC-047`) and sample decode stay decode-time checks. Reads no strip
  bytes. `DEC-050`.
- **`Error::PlaneTooLarge { pixels, limit }`** and **`Error::StripBytesExceedFile
  { declared, file_len }`** (SPEC-039) — additive (`Error` is `#[non_exhaustive]`).
- **Fuzz smoke jobs for `ifd`, `plane` and `ljpeg`** (CI, SPEC-039) — 60 s each,
  alongside `opcode`/`warp_opcode`; these targets were only compiled before. The
  `plane` target now drives the `validate → allocate(capped) → unpack_into`
  contract both ways, with forged-dimension and aliased-strip seeds.

- **`Sensor::opcode_list_2`** (`irradiance::ifd`, SPEC-054) — `OpcodeList2`'s raw
  bytes, kept on read like `opcode_list_1`/`opcode_list_3` (a malformed tag is
  `None` and listed in `malformed_tags`, `DEC-012`).
- **`test-fixtures` cargo feature** (dev-only, SPEC-054) — compiles in a doc-hidden
  `Sensor::test_fixture()` blank constructor for this crate's own integration
  tests. Off by default; carries no dependency.

- **Opt-in threaded demosaic and colour develop** (`irradiance::demosaic`,
  `irradiance::color`, SPEC-051, PROJ-002) — `demosaic_into_threaded`,
  `develop_color_into_threaded` and `develop_color_f32_into_threaded` take a
  thread count and split the output into row bands (`std::thread::scope`, no new
  dependency, no `unsafe`). On an M4 Pro a 5712×4096 colour develop runs ~1.95×
  faster at 2 threads and ~3.69× at 4; demosaic ~1.83× / ~3.30×. Output is
  byte-identical to the single-thread functions, and `threads = 1` is unchanged.
  `DEC-044`. (Entry added at the 0.2.0 cut; it was missing from `[Unreleased]`.)
- **`develop_color_f32_into`** (`irradiance::color`, SPEC-049, PROJ-002) — a public
  colour-develop entry point that writes scene-linear `f32` directly, equal to
  `to_linear_f32_into(develop_color_into(..))` to the bit (it quantises to `u16`,
  then divides by `65535.0`). Float consumers — the Python bindings, ML dataloaders
  — no longer hold the `u16` result and its `f32` copy at once. Additive, no other
  API change.

### Changed

- **⚠ `Cfa`, `WarpRect` and `Opcode` are now `#[non_exhaustive]`** (`irradiance::ifd`,
  `irradiance::opcode`, SPEC-042, PROJ-002) — completing the parser-output seal
  `Sensor` began (SPEC-054). They are built by the parser and read by consumers, so
  a future field or opcode variant is now additive, not breaking. Action required
  only if you **construct** a `Cfa` or `WarpRect` with a struct literal (use the
  value the parser returns instead) or **match** `Opcode` exhaustively (add a `_`
  arm). Reading fields, assigning them, and `Cfa { pattern, .. }` patterns are
  unchanged; each `Opcode` variant can still be built with a literal. The dev-only
  `test-fixtures` feature gains doc-hidden `Cfa::test_fixture` /
  `WarpRect::test_fixture` constructors for the crate's own tests (not part of the
  published API), and a self dev-dependency enables it so a bare `cargo test`
  compiles. `DEC-052`.
- **`plane::unpack_into` refuses uncompressed strips that alias** (SPEC-039) — it
  now shares `validate`'s checks, including the new rule that an uncompressed
  plane's `StripByteCounts` may not sum to more than the file length
  (`Error::StripBytesExceedFile`). Non-overlapping strips always satisfy it; no
  corpus file trips it. For a Compression-7 plane every strip's layout and in-file
  bounds are now checked before the first strip is decoded (previously strip by
  strip, interleaved with the decode), so a file with both a bad strip table and a
  bad SOF-3 stream reports the layout error. Every other input keeps its error and
  its error precedence — a wrong-length `dst` still wins over a bad layout.
- **A SOF-3 scan with no trailing EOI marker is documented as valid** (SPEC-039,
  `DEC-050` §4) — the scan completes on sample count; truncation before that is
  `Error::Truncated`. Behaviour unchanged, now pinned by a test.
- **README, conformance matrix: Compression-7 is decodable** — the README status
  block and PROJ-003 scope line, and the Leica M Monochrom (Typ 246) and Pentax
  K-3 III Monochrome rows, still described Compression-7 as unwired/pending; both
  held files decode bit-exact since SPEC-043/046/044. Added a tested-DNG-modes
  table to the README.

- **`cargo install irradiance` no longer installs the internal `irr` tool**
  (`Cargo.toml`, SPEC-055, PROJ-002, pre-0.2.0 review finding #6) — `irr` is a
  dev/oracle binary, not a product surface (`library-not-application`), but it was
  declared unconditionally, so a default build, `cargo install` and the published
  package all built it. It now sits behind a new empty `irr` cargo feature
  (`required-features = ["irr"]`); build it with `cargo build --features irr`.
  `cargo test --all-features` still builds it for the demosaic oracle test. The
  library API is unchanged. New check `scripts/package-targets-check.sh`
  (`just package-targets-check`, CI `package-surface`) builds the tree and the
  packaged crate with default features and fails on any executable. `DEC-048`.
- **⚠ `Sensor` is now `#[non_exhaustive]`** (`irradiance::ifd`, SPEC-054,
  `DEC-049`) — `Sensor` is a parser-output view that `Container::sensor` builds and
  callers read. Sealing it makes this release's new field (`opcode_list_2`), and
  every later one, non-breaking. **Breaking for callers that built a `Sensor` with
  a struct literal or `..rest` update**: obtain one from `Container::sensor()`
  instead (its public fields stay readable and assignable). `Cfa`, `WarpRect` and
  `Opcode` are sealed separately before 0.2.0 (SPEC-042).
- **⚠ Colour develop now crops and orients like the mono develop — `dst` is
  smaller** (`irradiance::color`, SPEC-050, PROJ-002) — `develop_color_into` and
  `develop_color_f32_into` now apply `ActiveArea` → `DefaultCrop` → `Orientation`
  exactly as `develop_into` does, so the optical-black masked border is dropped
  and a rotated file comes out rotated. **Breaking for callers:** size `dst` from
  `develop::output_dimensions(&sensor)` × 3, not `width × height × 3` (a
  whole-plane `dst` is now `Error::RgbBufferWrongLength`), and a hostile geometry
  is the same typed `InvalidActiveArea` / `InvalidDefaultCrop` /
  `UnsupportedOrientation` the mono path returns. The kept pixels' colours are
  **unchanged** — byte-identical to the previous whole-plane render at their
  source; only the frame moves. A file with no `ActiveArea`/`DefaultCrop` and
  orientation 1 is byte-identical to before. `DEVELOP_VERSION` 3 → 4; no new
  scratch buffer (peak RSS falls by the smaller `dst`). `irr color` writes the
  cropped image. `DEC-043`.

- **Colour develop uses ~134 MB less peak memory, output unchanged**
  (`irradiance::color`, SPEC-049, PROJ-002) — `develop_color_into` was fused from
  three buffered passes into two: level-normalisation now runs once into a small
  rolling row cache as demosaic reads, and the colour matrix is applied per pixel
  straight into `dst`, so the `normalized` mosaic and the `camera_rgb` intermediate
  are gone (only a `w·h` green plane and a ~91 KB row ring remain). Measured peak
  RSS at 23 MP fell 358.7 → 224.9 MiB (−133.9 MiB) at no speed cost (develop within
  ~4% of baseline). Output is **bit-identical** — same integer ops, same order
  (the goldens and a byte-for-byte equivalence test are the gate). `DEC-042`.

- **Demosaic is about 1.69× faster, output unchanged** (`irradiance::demosaic`,
  SPEC-048, PROJ-002) — `demosaic_into` gained a row-wise **interior fast path**:
  interior pixels read their neighbours by direct index into per-row slices,
  skipping edge reflection and per-read bounds checks, while the 2-pixel border
  keeps the reflecting path. The kernel arithmetic is unchanged, so the output is
  **bit-identical** — the demosaic golden hashes plus a byte-for-byte equivalence
  test against a reflecting reference are the gate. Measured 24 MP median
  `demosaic_into` 533.0 → 316.2 ms (`DEC-041`). No public API, no dependency, no
  `unsafe`; per-row slices keep the interior panic-free by construction.

- **Colour develop now honours the DNG `ActiveArea` origin** (`irradiance::color` /
  `irradiance::demosaic`, SPEC-047, PROJ-002) — `develop_color_into` and `demosaic_into`
  now index each pixel's Bayer colour and its BlackLevel/WhiteLevel 2×2 slot from the
  `ActiveArea` origin (a shared `active_area_origin` helper plus a wrapping offset), not
  the plane's top-left. A DNG whose `ActiveArea` origin is odd previously developed a flat
  grey to **magenta** (`0.589 0.000 0.669`) with no error raised; it now stays grey,
  matching LibRaw (`~0.200` per channel), proven by a grey-stays-grey red-proof across all
  four origin parities and the four Bayer phases. **Even-origin files are byte-identical**
  (no change for the Canon 5D3 / Nikon D300S corpus), and `develop_color_into`'s buffer
  contract is unchanged — the masked-border crop and full `DefaultCrop`/`Orientation`
  parity are a separate STAGE-012 follow-up. `DEVELOP_VERSION` is bumped to `3` (`DEC-040`).
  No new public API, no new dependency. Raised by the pyirradiance rawpy/LibRaw benchmark.

- **DNG `Compression 7` sensor planes now route through the lossless-JPEG decoder**
  (`irradiance::plane`, SPEC-043, PROJ-003) — `plane::unpack_into` dispatches on the
  `Compression` tag: a `Compression 7` (SOF-3 lossless JPEG) plane now decodes each
  strip through the internal `ljpeg` decoder instead of being rejected at the gate
  with `Error::UnsupportedCompression`, while **uncompressed planes take the existing
  path byte-for-byte unchanged** (pinned by a characterization test) and any other
  compression code stays the clean typed `UnsupportedCompression` it always was. **No
  new public API, no new `Error` variant, no new dependency** (`ljpeg` stays
  `pub(crate)`; the only new surface is a `#[cfg(test)]` encoder re-export).
  Single-component SOF-3 decodes **bit-exact** (single- and multi-strip, both container
  byte orders), proven by six tier-A tests (hand-built via the STAGE-009 encoder). The
  two held monochrome files (Leica M Typ 246, Pentax K-3 III) are **two-component
  (`Nf=2`) column-interleave** SOF-3, so they now **reach** the decoder and defer
  cleanly with a typed `LjpegUnsupportedComponentCount`; end-to-end decode of them lands
  with SPEC-046 (the two-component decode, DEC-037) plus SPEC-044's bit-exact `dnglab`
  check. The binding is panic-free under `#![forbid(unsafe_code)]` + the five-lint deny
  policy.

- **Lossless-JPEG header parsing hardened for the public `Compression 7` route**
  (`irradiance::plane` / internal `ljpeg`, SPEC-045, PROJ-003) — the SOF-3 header parser
  now rejects a point transform at or above the sample precision, a non-DC or
  out-of-range Huffman table, and lets a later Huffman table with the same destination
  replace an earlier one — each a **typed `Error`, never a panic** — so a truncated or
  hostile `Compression 7` strip fed through the public `plane::unpack_into` fails cleanly.
  A held `Compression 7` file decodes to a **byte-identical** `u16` plane across two
  independent decodes (plane-level determinism, integer-only by construction); the
  `plane` / `develop` / `demosaic` fuzz targets now exercise the **public** Compression-7
  route continuously; and the DNG `Compression 7` binding gains a provenance-ledger entry.
  No new public API, no new dependency; `ljpeg` stays `pub(crate)`. This is the capstone
  of STAGE-011 — the compressed-plane lossless-JPEG binding (PROJ-003) is complete.

- **Colour matrix is now scene-adaptive — dual-illuminant interpolation**
  (`irradiance::color`, SPEC-034, PROJ-002) — for a DNG carrying **two** colour
  calibrations (`ColorMatrix1/2`, and usually `ForwardMatrix1/2`, at
  `CalibrationIlluminant1/2`), `to_srgb_linear_into` / `develop_color_into` now build the
  camera→XYZ matrix from the calibration **interpolated at the scene colour temperature**
  (the DNG reciprocal-temperature `NeutralToXY` method — `AsShotNeutral` → McCamy CCT →
  mired interpolation weight `g`, **DEC-034**), instead of using the first illuminant
  only. The result is materially more accurate across illuminants (a daylight shot is no
  longer rendered through a tungsten calibration). The **public API is unchanged** (no new
  functions, signatures, or `Error` variants — the weight is an internal, once-per-image
  refinement), and only the decoded pixel values change, **only for dual-calibration
  files**: a **single-calibration** sensor (no `ColorMatrix2`, or a missing/unknown
  `CalibrationIlluminant`) is **byte-identical** to SPEC-033. The weight is committed-constant
  **`f64`**, `+ − × ÷` only (McCamy is a polynomial, no `powf`/transcendental — byte-identical
  across platforms, **DEC-033**/**DEC-034**). Verified **bit-exact** on analytic fixtures
  against an independent `NeutralToXY` re-derivation, with a perturbed-`ColorMatrix2`
  red-proof (**DEC-017**); every divide guarded (no panic, no NaN). The
  `ColorMatrix`-path scene-white→D50 chromatic-adaptation CAT remains deferred to a
  follow-on judged by SPEC-035's ΔE (DEC-034 Non-Goal).

- **Demosaic kernel upgraded to Hamilton-Adams** (`irradiance::demosaic`, SPEC-031,
  PROJ-002) — `demosaic_into` now interpolates with the **Adams-Hamilton** directional
  kernel (directional green on the lower-gradient axis + colour-difference red/blue)
  instead of SPEC-030's bilinear baseline — a real quality step (directional green
  removes most of bilinear's edge zippering / false colour). The public signature, the
  caller-owned zero-alloc buffer contract (DEC-016 / DEC-032 — the two passes share the
  green plane in `dst`'s own green slots) and the `irr demosaic` P6 output are all
  unchanged; only the decoded pixel values change. Per **DEC-031** the algorithm is
  Hamilton-Adams (US 5,629,734, **expired**), **not** Malvar-He-Cutler — MHC is still
  patent-encumbered (US 7,502,505, active to ~2027-01-25) and a patented method in a
  permissive crate defeats the thesis: **clean-room beats copyright, not patents**.
  Verified **bit-exact** on analytic fixtures (interior / edge / corner, a
  directionality fixture, the classifier tie, DC-preservation, BGGR) with a
  perturbed-kernel red-proof (DEC-017). Output stays **uncorrected** camera-space
  linear RGB — colour is STAGE-008.

### Added

- **Both held DNG `Compression 7` files now decode bit-exact against `dnglab`**
  (`irradiance::plane`, SPEC-044, PROJ-003) — the Leica M Monochrom Typ 246 (big-endian,
  P=12) and the Pentax K-3 III Mono (little-endian, P=14) now decode through the public
  `plane::unpack_into` to a `u16` plane whose MD5 **equals** `dnglab analyze --raw-checksum`
  exactly (`c72e7d9579e2adaef9c6247841b8ac1d` and `98bae1339c25eb6e04fbac6b5f643189`), and
  develop end-to-end through the existing pipeline unchanged — a source swap, not a second
  pipeline. To get the Pentax file to decode at all, the lossless-JPEG header parser now
  **skips `APPn` (`0xFFE0..=0xFFEF`) and `COM` (`0xFFFE`) marker segments by their declared
  length** (**DEC-039**): that file carries an `APP0` segment between its Huffman table and
  the start of scan that the parser previously rejected. The skip is bounds-checked (a bad
  length is a typed `Error`, never a panic); the Leica file carries no `APPn`, so its decode
  is byte-for-byte unchanged, as is the single-component path. The real-file oracle also
  **confirms SPEC-046's column de-interleave mapping** on both byte orders — the thing the
  analytic round-trip could not prove (`shared-path-oracle-tautology`) — and is proven red
  by a staged-copy fault injected into the `ljpeg` decode. **No new public API, no new
  `Error` variant, no new dependency.**

- **SOF-3 decoder now decodes two-component (`Nf=2`) column-interleave — internal**
  (`irradiance` `ljpeg`, SPEC-046, PROJ-003) — the `pub(crate)` `src/ljpeg.rs` decoder
  grows to decode the two-component column-interleave SOF-3 frame both held monochrome
  Compression-7 DNGs actually use (Leica M Typ 246 at P=12, Pentax K-3 III at P=14):
  `reconstruct_two_component` decodes the interleaved scan with genuine **per-component**
  predictor state and de-interleaves the two column phases into one `2·X`-wide monochrome
  `u16` plane; DC tables are keyed by `Td` (both held files share `Td=0`), and the length
  check keys on `Nf`. The `Nf=1` path is **byte-for-byte unchanged**; `Nf>2` (Bayer / CFA
  colour) stays a clean typed `LjpegUnsupportedComponentCount` (DEC-037). Proven bit-exact
  at P=12 / P=14 by an analytic two-component round-trip with an `oracle-must-be-shown-red`
  red-proof (an independent reference decoder defeats the shared-helper tautology). **No
  public API, no new `Error` variant, no new dependency** — `ljpeg` stays `pub(crate)`; the
  real-file bit-exact confirmation and the exact de-interleave mapping are SPEC-044
  (`dnglab --raw-checksum`).

- **SOF-3 decoder proven panic-free on hostile input — internal, completes
  STAGE-009** (`irradiance` `ljpeg`, SPEC-038, PROJ-003) — a `ljpeg` libFuzzer
  target now drives the full single-component SOF-3 decoder (SPEC-036 + SPEC-037)
  through a dev-only `fuzzing` cargo feature (an empty `[features] fuzzing = []` +
  a `#[cfg(feature = "fuzzing")] #[doc(hidden)]` entry, **DEC-036** — a feature,
  not `cfg(fuzzing)`, so the plain-`cargo clippy` fuzz-graph CI gate still compiles
  it), and ran **3,098,220+ executions with no crash**. 14 own-work SOF-3 seeds (9
  valid across predictors 1..7 / 8-12-16-bit / point-transform / restart, 5 hostile:
  truncated header, truncated scan, oversized DHT, `Nf≠1`, bad marker) plus two
  in-crate replay unit tests assert every truncated / oversized-table /
  forged-dimension bitstream returns a typed `Ljpeg*` `Error` rather than panicking,
  under `#![forbid(unsafe_code)]` + the five-lint deny policy. **No public API change
  and no new `Error` variant** — `ljpeg` stays `pub(crate)` (STAGE-011 owns the real
  surface) and no new runtime dependency. This is the safety half of STAGE-009; with
  it the single-component SOF-3 lossless-JPEG core decoder is **complete** (bytes →
  u16 plane, bit-exact and panic-free), with the DNG `Compression 7` container
  binding + `dnglab` bit-exact check left to STAGE-011.

- **SOF-3 single-component decoder completed — internal** (`irradiance` `ljpeg`,
  SPEC-037, PROJ-003) — `ljpeg::reconstruct` / `decode_into` turn the SOF-3 entropy
  front-end's per-sample differences into the `u16` sensor plane: predictors 1–7,
  the §H.2.1 boundary rules, the point transform (`out = reconstructed << Pt`), and
  restart reset (T.81 Annex H.1), proven by an in-harness encode→decode round-trip
  oracle with a perturbed-decoder red-proof. Adds one more additive `Error` variant
  (`LjpegPlaneWrongLength`). Still `pub(crate)` and not reachable until STAGE-011
  wires `Compression 7`; SPEC-038 (fuzz) remains.

- **SOF-3 lossless-JPEG entropy front-end — internal** (`irradiance` `ljpeg`,
  SPEC-036, PROJ-003) — a new `pub(crate)` `src/ljpeg.rs` decodes the entropy
  layer of a DNG `Compression 7` (lossless JPEG, ITU-T T.81 Annex H) sensor plane:
  it parses the SOF-3 marker/segment structure into the single-component `Frame`
  (precision, dimensions, point transform, predictor, restart interval) + the DC
  `HuffTable`, then reads the `0xFF`-destuffing MSB-first entropy scan and
  `DECODE`+`RECEIVE`+`EXTEND`s each sample to a signed difference. It adds **seven
  additive `Ljpeg*` variants** to the public `#[non_exhaustive]` `Error` —
  **additive, non-breaking, and not yet producible** (nothing in the crate
  constructs them until STAGE-011 routes `Compression 7`). No public API or
  runtime behaviour is reachable yet; this is the first half of STAGE-009's
  compressed-bytes→`u16` decoder (SPEC-037 adds predictor reconstruction, SPEC-038
  the fuzz target). Implemented **clean-room** from the published T.81 standard
  (**DEC-035** — patent-clean: the lossy-DCT and arithmetic-coding JPEG patents
  expired and never read on SOF-3 Huffman), never from Adobe's DNG SDK; panic-free
  on untrusted input under `#![forbid(unsafe_code)]` + the five-lint deny policy.

- **Colour transform — first display-referred colour render** (`irradiance::color`,
  SPEC-033, PROJ-002) — new `color::to_srgb_linear_into(sensor, &[u16], &mut [u16])`
  turns camera-space linear RGB into **linear-sRGB** by white balance
  (`AsShotNeutral`) + camera→XYZ(D50) (`ForwardMatrix1`, else inverted `ColorMatrix1`;
  single illuminant) + XYZ(D50)→linear-sRGB, folded into one 3×3 matrix built once per
  image; and `color::develop_color_into(sensor, &[u16], &mut [u16])` composes the whole
  mosaic→picture path (per-channel level-normalize → `demosaic_into` → the matrix). The
  arithmetic is committed-constant **`f64`**, `+ − × ÷` only (**DEC-033**): the
  irrational sRGB (IEC 61966-2-1) and Bradford (D50→D65) constants are committed `S`/`B`
  arrays, so the transform is byte-identical across platforms — the same basis `tone`
  relies on. The sRGB **OETF** is **not** applied here (`color` outputs linear-sRGB;
  `tone::encode_srgb16_into` encodes — DEC-025). The apply is scale-free (`65535 ≡ 1.0`,
  DEC-018) and clamps before the round-half-up quantize. New `Error` variants
  `ColorSourceWrongLength` and `MissingColorCalibration` (a zero-denominator required
  rational, absent/ill-shaped `AsShotNeutral`/`ForwardMatrix`/`ColorMatrix`, or a
  singular `ColorMatrix` → typed error, never a panic), and a new **`irr color`**
  subcommand emitting a **P6 PPM** — the first recognisable colour picture. Verified
  **bit-exact** on analytic fixtures against an independent reference with a
  perturbed-`ForwardMatrix` red-proof (DEC-017). Caller-owned buffers, no alloc on the
  matrix path (DEC-016). (Note for the eventual publish — DEC-028.)
- **Bilinear Bayer demosaic** (`irradiance::demosaic`, SPEC-030, PROJ-002) — new
  `demosaic::demosaic_into(sensor, &[u16], &mut [u16])` turns a Bayer **mosaic** into
  a 3-channel **interleaved camera-space linear RGB** image (caller-owned buffer, no
  alloc — DEC-016). Bilinear interpolation, **phase-driven** by `Sensor.color_filter`
  (not hard-coded RGGB), **phase-preserving reflect** border, round-half-up integer
  averages. New `Error` variants for the mono/absent-CFA, unsupported-CFA and
  wrong-length cases, and a new **`irr demosaic`** subcommand emitting a **P6 PPM** —
  the first multi-channel output irradiance produces. Verified **bit-exact** on
  analytic fixtures with a perturbed-kernel red-proof (DEC-017), confirmed genuinely
  red by an independent reviewer. Output is **uncorrected** (no white balance, no
  colour matrix — those are STAGE-008), so it reads greenish/flat; the recognisable
  picture is STAGE-007 + STAGE-008. (Note for the eventual 0.1.0 publish — DEC-028.)
- **Colour metadata parsed** (`irradiance::ifd`, SPEC-026, PROJ-002) — `Sensor` now
  exposes the DNG colour-metadata block: `color_matrix_1`/`_2`,
  `forward_matrix_1`/`_2` and `as_shot_neutral` (a new verbatim `Rational { num, den }`,
  read by `Container::rationals()` — SRATIONAL + fractional RATIONAL, with
  `Error::RationalValueOutOfRange` on the `RATIONAL → i32` narrow), the
  `calibration_illuminant_1`/`_2` light-source codes, and the full per-channel
  `black_levels` / `white_levels`. The matrices/illuminants/`AsShotNeutral` read
  from `IFD0`, the per-channel levels from the sensor IFD; each is an
  interpretation-class field — a malformed tag costs that field, never the file
  (DEC-012). **Parsed and exposed only, not applied to pixels** (demosaic is
  STAGE-007, colour is STAGE-008). Verified **exactly** against `exiftool` +
  `dnglab` on both corpus Bayer DNGs by i128 cross-multiplication, no tolerance
  (DEC-030); `irr ifd` surfaces every field. The monochrome scalar metadata path is
  unchanged. Completes STAGE-006 — the Bayer sensor is now fully read (bit-exact
  plane + truthful colour metadata).
- **Above-WhiteLevel samples accepted** (`irradiance::plane`, SPEC-029, PROJ-002) —
  `plane::unpack_into` no longer rejects a sample above `WhiteLevel` (specular
  highlights are legal); raw values are preserved, not clamped. Amends **DEC-008**
  (via DEC-029) and removes `Error::SampleExceedsWhiteLevel`. With SPEC-028, both
  corpus Bayer DNGs (Canon single-strip, Nikon 12-strip) now decode fully —
  unblocking the bit-exact colour-plane oracle.
- **Multi-strip sensor planes** (`irradiance::plane`, SPEC-028, PROJ-002) —
  `plane::unpack_into` now assembles a plane stored as **N strips** (each strip
  unpacked into its own rows; sub-byte depths padded per-strip), not just a single
  strip — opening the common multi-strip layout (e.g. a Nikon D300S Bayer DNG, 12
  strips). Single-strip planes stay byte-for-byte unchanged; panic-free and fuzzed
  (20M+ runs, 0 crashes).
- **CFA (Bayer) sensor recognition** (`irradiance::ifd`, SPEC-024, PROJ-002) — the
  sensor gate now accepts a colour-mosaic DNG (`PhotometricInterpretation == 32803
  (CFA)`, `SamplesPerPixel == 1`) alongside the monochrome `LinearRaw` plane, and
  `Sensor` exposes a structured `Cfa` model (`CFAPattern` / `CFARepeatPatternDim` /
  `CFAPlaneColor` / `CFALayout`), `None` for a monochrome sensor. Parsed as an
  interpretation-class field — a malformed CFA tag costs that field, never the file
  (DEC-012) — and the mosaic plane still reads bit-exact. No demosaic or colour
  yet; the monochrome path is byte-for-byte unchanged.

### Known limitations

- **No colour-accuracy claim.** The colour develop path ships (white balance,
  dual-illuminant camera→XYZ, Bradford adaptation, sRGB), and it is bit-exact
  against its own independent transcriptions, but it has not yet been measured
  against a physical reference: the ColorChecker ΔE2000 oracle (SPEC-035) is
  blocked on capturing the chart. Treat colour as plausible, not calibrated.
- **The colour develop does not run the `OpcodeList` stages** (`WarpRectilinear`,
  `FixBadPixelsConstant`) that `develop_into` runs. A file that *requires* them is
  refused with `Error::UnsupportedRequiredOpcode` (SPEC-054); optional ones are
  skipped.

## [0.1.1] - 2026-10-29

First crates.io release (DEC-028): the monochrome, uncompressed-DNG develop
pipeline ("monochrome, uncompressed, one camera"), through STAGE-005. Identical
source to the yanked 0.1.0, re-published with a trimmed package (`#33` publish →
`#34` trim → `#35` bump). Everything under `[Unreleased]` above — the Bayer
plane, colour metadata, demosaic and the colour transform — landed *after* this
release and is **not** in it.

### Added

- The crate exists: `irradiance` library + `irr` dev binary, edition 2021,
  measured MSRV 1.90, **zero dependencies**, `MIT OR Apache-2.0` (SPEC-001).
- Panic-free lint policy (`unwrap_used`, `expect_used`, `indexing_slicing`,
  `panic`, `arithmetic_side_effects` at `deny`; `#![forbid(unsafe_code)]`),
  enforced by a **red-proof that is itself proven red** — it injects violations
  into a copy of the real library and requires a clean negative control, so it
  fails if the policy is deleted, weakened to `allow`, downgraded to `warn`, or
  if clippy never ran (DEC-009).
- **The TIFF/IFD container reader** (`irradiance::ifd`, SPEC-003) — the first
  code in this library to read an actual RAW file. Walks IFD0's chain, recurses
  `SubIFDs` (tag 330), and reads entry tags, types, counts and payloads, with
  **every** offset and length bounds-checked into a typed `Error`. Depth-,
  cycle- and count-guarded: a `SubIFD` that points at itself terminates with
  `Error::CyclicIfd` rather than recursing forever.
  - Sensor-IFD selection keys on `NewSubfileType == 0 &&
    PhotometricInterpretation == 34892 && SamplesPerPixel == 1` — **never on
    largest dimensions**, because a Q2M's `SubIFD2` is a full-resolution JPEG
    preview only 56 px narrower than the plane.
  - Verified against `exiftool 13.55` on all **7** corpus files — **6 `II` / 1
    `MM`** by byte order, and **6 DNG / 1 PEF** by container; the two are
    independent axes and the PEF is `II` as well. Four bodies: Leica Q2
    Monochrom ×3, Leica M Monochrom, Leica M Monochrom (Typ 246, the only `MM`),
    Pentax K-3 III Monochrome ×2. Includes the Pentax's malformed
    `BlackLevelRepeatDim`, which costs the tag and not the file.
  - Compressed planes (2 JPEG, 1 Pentax PEF) are **rejected cleanly** with
    `Error::UnsupportedCompression`; their tags stay readable.
- **A fuzz target, shipped in the same change** (`fuzz/fuzz_targets/ifd.rs`,
  AGENTS §12 bar 2) with a committed hand-built seed corpus, and **shown to
  work**: a deliberately unchecked index was caught by libFuzzer in under a
  second (exit 77 + crash artifact), then removed. 13.0 M executions clean on
  the restored code.
- `irr ifd [--entries] <file>` — dumps the container walk and the sensor
  plane's tags, including the free layer-0 packing check
  (`width x height x bits == StripByteCounts x 8`).
- `just fuzz` / `just fuzz-seeds`, which encode the `cargo fuzz` PATH trap so
  nobody rediscovers it.
- CI: `fmt --check`, `clippy -D warnings`, `test`, **two** `cargo deny check
  licenses` jobs — one for the library's graph and one for `fuzz/`'s, which is a
  separate package and a separate graph (DEC-011) — an MSRV check, and the lint
  red-proof.
  ⚠ CI **cannot** verify decode correctness — tier-B corpus files are never
  committed (DEC-003), so a green badge does not mean the decoder is bit-exact.
- **The live metadata oracle** (`tests/metadata_oracle.rs`, `tests/support/tools.rs`,
  SPEC-005) — `Sensor` is now diffed against `exiftool 13.55` and
  `dnglab analyze --meta --json`, shelled out to fresh every run, replacing a
  frozen hand-transcribed table. No new dependency: `exiftool -T` needs no
  parser, and `dnglab`'s JSON is read by asserting the handful of keys this
  oracle needs are unique in the document before trusting a match.
  - `exiftool` cross-checks all eleven tag-level fields on all **7** corpus
    files, absence included (an M Monochrom's missing `ActiveArea`, a Pentax
    PEF's five absent DNG tags).
  - `dnglab` cross-checks six scalars on the **6 DNG** files (excludes the PEF —
    its values come from rawler's camera database, not the file, evidenced by
    a `bitDepth` of 16 against the file's own `BitsPerSample` 14) and asserts,
    rather than ignores, that its `cropArea.p` is sensor-absolute
    (`ActiveArea` origin + `DefaultCropOrigin`) where ours and exiftool's are
    DNG-relative.
  - The Pentax DNG's malformed `BlackLevelRepeatDim` (`DEC-012`) is asserted
    three ways in one test: exiftool reads a bare `1`, dnglab warns on stderr
    and substitutes, and our reader reports `None` with `50713` recorded in
    `malformed_tags`.
  - **Proven red, both directions** (`oracle-must-be-shown-red`): a tier-A
    pair replays a committed `exiftool` line through the real parsing code
    with no tool and no corpus (CI's only reachable half) — clean on an
    honest reading, exactly one named `Mismatch` on one perturbed field. A
    tier-B pair patches `ActiveArea`'s payload bytes in an in-memory copy of a
    real file (mutation asserted to have actually changed the buffer first),
    diffs the patched reader against the tool reading of the *original* file,
    and confirms re-running on the unpatched bytes is clean again.
  - `just oracle-meta` runs this file alone; both tool-absence and
    corpus-absence skip loudly, naming what is missing.
- **Tone curve and the three output modes** (`src/tone.rs`, SPEC-019). The
  scene-linear `u16` develop result is encoded for display through a
  **table-driven sRGB OETF** (IEC 61966-2-1): the library runs no `powf` — the
  curve is two committed, regenerable lookup tables (`just gen-tone-table`),
  byte-identical across platforms by construction. Three caller-owned encoders:
  `to_linear_f32_into` (scene-linear, no curve), `encode_srgb16_into` (16-bit
  sRGB) and `encode_srgb8_into` (8-bit sRGB); `irr develop --format` selects
  among them. `develop_into` is unchanged — `DEC-018`'s scene-linear output
  stands; the encoders are a composable layer on top (`DEC-025`).
  - **Analytic tone oracle, proven red** (`oracle-must-be-shown-red`,
    `tests/tone_oracle.rs`): every table entry is checked against the sRGB
    formula recomputed in `f64` to within ±1 code value — independent of rawler
    — and a deliberately perturbed entry is confirmed to turn the oracle red,
    then reverted byte-identical.
- **The develop warp branch is now pinned to the fixed plane** (`tests/develop.rs`,
  SPEC-022). `develop_into`'s `WarpRectilinear` branch already developed the
  `FixBadPixelsConstant`-repaired plane, but nothing asserted it — SPEC-017's
  coverage reached only the no-warp branch, and every real Q2M frame takes the
  warp branch. A synthetic tier-A fixture (a bad pixel at the warp's
  optical-centre fixed point plus a non-identity warp) now proves the fixed
  value reaches the developed output through the warp branch, **shown red**
  (`oracle-must-be-shown-red`): severing the branch to the raw source turns the
  two new tests red (suite 243/0/3 → 241/2/3) while SPEC-017's no-warp assertion
  stays green. No `src/` change — latent hardening that closes the hole before a
  bad-pixel-bearing frame arrives.

### Known gaps

- No pixel decode or unpack yet — `StripOffsets`/`StripByteCounts` are read as
  **tags** only. The unpacker and DEC-008's two-path (`bits % 8`) rule are
  STAGE-002.
- No typed tag model: `irradiance::ifd` widens `BYTE`/`SHORT`/`LONG` to `u32`
  and returns `Error::UnexpectedFieldType` for `RATIONAL` and the signed types.
  SPEC-004 owns that.
- No multi-strip corpus file. All seven held planes are single-strip, and the
  tests **assert** that (`tests/ifd_reader.rs:352`, `:443`, `:448`) rather than
  merely not exercising the alternative — so the day a multi-strip file arrives
  it fails those assertions loudly instead of silently taking an untested path.
  That is the right way round, but it is a test to update, not a reader bug.

### Changed

### Fixed

- **Gate scripts can no longer report green without their check running**
  (SPEC-023, STAGE-005's capstone — the generalisation of SPEC-016/021/011).
  Every gate here is a `set -euo pipefail` shell script, and in that context an
  unguarded `grep` aborts the whole script on no-match while a process- or
  command-substitution's non-zero exit is invisible to `set -e` — either way a
  gate can print a green verdict (or die) with its check never having run. A new
  `just gate-audit` (`scripts/gate-audit.sh`) + a CI job scan all **8**
  verdict-bearing `scripts/*.sh` for that shape, name each offender and which
  shape, exclude comments, and — because the audit is itself source-matching
  tooling — **assert their own scan count**, so a glob matching nothing is a loud
  failure, not a silent pass. Shipped with a red-proof
  (`oracle-must-be-shown-red`, which covers gates): a shape injected into a
  **copied** gate is caught by name with a clean negative control and isolation,
  the working tree untouched. The three live offenders are fixed and each proven:
  `validate.sh` (a non-`SyntaxError` ruby fault now turns the gate RED instead of
  a green *"0 … parseable"*), `handback-sync.sh` (a `superseded` handoff is
  skipped, not treated as `pending` — closing the
  `handback-sync-trips-on-superseded-handoffs` signal and retiring the manual
  `synced_at` kludge SPEC-016's ship needed for `HANDOFF-038`), and
  `decisions-audit.sh` (a DEC missing its `# DEC-` H1 no longer aborts the lint
  silently). No `src/` change.
- **The fuzz crate is now linted** (SPEC-011). `fuzz/` is a second cargo graph
  DEC-011 keeps outside the library's `[dependencies]`, so `cargo clippy
  --all-targets -- -D warnings` at the repo root mentioned `irradiance-fuzz`
  **zero** times — a clippy regression in a fuzz target would have shipped
  silently. A new `just lint-fuzz` (`cargo clippy --manifest-path fuzz/Cargo.toml
  --all-targets -- -D warnings`) and a CI job close it, exactly as `deny-fuzz`
  did for the licence gate: the gate is now **two invocations, both required**.
  Scope is clippy's default `-D warnings`, **not** the library's five panic-free
  lints (the fuzz harness is dev-only glue, not an attacker-surface — `DEC-027`,
  extending DEC-011). Shipped with a red-proof (`oracle-must-be-shown-red`, which
  covers gates): an injected `unnecessary_cast` is caught only because
  `-D warnings` promotes it, with a clean negative control. No `src/` or
  fuzz-target change — the crate was already clippy-clean.
- **The develop oracle stops claiming warp-bearing pixels it cannot see**
  (SPEC-021). SPEC-020's SSIMULACRA2-vs-`dnglab --srgb` tolerance (≥85) was
  advertised over all pixels, but `dnglab`/`rawler` implement **no** DNG
  OpcodeList processing and `dnglab` ignores EXIF `Orientation`, so for
  warp-bearing and rotated frames the comparison is structurally meaningless (a
  *correct* `WarpRectilinear` scores −60.169 against dnglab's uncorrected
  reference, vs a +83.145 no-warp baseline). The oracle's scope is now formally
  narrowed to **warp-free, Orientation-1 pixels** (new `DEC-026` amending
  DEC-005; a carve-out in `docs/oracle-contract.md`), and SPEC-018's two dead
  `#[ignore]`'d oracle tests are **retired, not hidden**: replaced by a passing
  characterization asserting the blindness is real (−60.169 / −40.281 < +83.145,
  measured) plus an explicit `Orientation ≠ 1` skip — so the oracle's silence on
  warp can't be mistaken for a passing verdict. Warp geometry stays verified
  analytically (`tests/warp.rs` AC4/AC10). No `src/` logic change; repo-wide
  `#[ignore]` drops 3 → 1.
- **`just validate` now PARSES every artifact's front matter** instead of only
  grepping for required keys. The old gate reported *"valid required
  front-matter"* without ever running a YAML parser, so an artifact no parser
  could read was reported valid — one shipped, was archived, and survived three
  later specs. Now it parses every `*.md` under `projects/` + `decisions/`
  (**121** artifacts, a superset of the old specs+patches+spikes scope) with
  `ruby`'s `YAML.parse`, fails loudly naming the file and the parser's own
  message, and — closing the mute-gate class — **refuses to report green if the
  parser itself is missing** rather than silently skipping the check. Proven
  red (`oracle-must-be-shown-red`): the old script passes a parse-broken fixture
  the new one rejects, with the parser's message shown. Fixed the one live
  offender (`HANDOFF-039`) that broadening the scope to `handoffs/` revealed
  (SPEC-016; ships the `validate` half of the
  `handback-sync-truncates-multi-line-scalars` signal).
- **`just cost-audit` no longer scans patch/spec bodies as front matter** and no
  longer requires build+verify totals on patches whose verify handoff has not
  handed back — DEC-013 says `handback-sync` is the ledger, not the audit
  (PATCH-003, closing PATCH-002's two ship-blockers).
- **Both `just lint` and `just lint-ci` now print the clippy version they
  invoked**, with a note that `lint` is unpinned (PATH-resolved Homebrew) and
  `lint-ci` is what CI sees. A gate that does not state what produced its
  result is the defect that cost this repo 17 consecutive red CI runs (PATCH-004).
- **`just cost-audit` now enforces the stage-level `orchestration_cost:` block**
  at ship — a stage template's own `orchestration_cost` slot was rejected
  because nothing summed it, so ~31% of PROJ-001's real spend was
  documented-but-invisible in every rollup (PATCH-002, DEC-022 amending
  DEC-013 §5).

- **The licence gate now actually covers `fuzz/`.** `DEC-011` recorded that
  `cargo deny` could not reach the fuzz package and hand-wrote its licence table
  instead. Both halves were wrong: `cargo deny --manifest-path fuzz/Cargo.toml
  check licenses` runs fine, and it **failed** — `libfuzzer-sys` declares
  `(MIT OR Apache-2.0) AND NCSA` (conjunctive, so NCSA is not optional) where the
  table said `MIT OR Apache-2.0`, and `irradiance-fuzz` itself carried no
  `license` field, which is `unlicensed` and an error. NCSA is permissive
  (OSI-approved, FSF Free/Libre) so nothing copyleft was ever linked — but the
  record was wrong on a `blocking` constraint, in the document standing in for
  the gate. Now: a named per-crate exception in `deny.toml` (not a widened
  `allow`), a `license` field on `fuzz/Cargo.toml`, a corrected and re-measured
  table in DEC-011 covering all ten crates, and `just deny-fuzz` + a CI job so it
  is a gate rather than a paragraph.
- `just msrv` — the MSRV gate was the only one of the ten with no recipe, so it
  was the only one handing out a raw `cargo +1.90.0 …` that fails with
  `no such command` under the default PATH (the third instance of the
  `+toolchain` trap; `guidance/toolchain-brief.md`).
- Corpus facts in `SPEC-003` and this file: the byte-order count (**6 `II` / 1
  `MM`**, not 5 + a PEF — container and byte order are different axes), the
  compression count (**2 JPEG**, code 7; `K3III.PEF` is code **65535**,
  vendor-private, not JPEG), and "the full-resolution SubIFD" (the PEF has none —
  its plane is in `IFD0`).
- `docs/conformance-matrix.md` gained the three bodies that were held, read
  end-to-end and unlisted, against that file's own opening rule.
- The malformed-tag rule is now stated rather than implied (**DEC-012**):
  **strict on structure, tolerant on shape.** A malformedness that changes *what
  exists* — the header, an entry table, the chain's `next`, or `SubIFDs` — is
  fatal to the container; one that changes only what a known-optional fixed-length
  tag *says* costs that tag and is reported in `Sensor::malformed_tags`. No
  behaviour changed; `SPEC-004` widens the type model on top of this boundary and
  should not have had to guess it.
- **`tests/ifd_reader.rs`'s hand-transcribed tag-value table is gone** (SPEC-005).
  `EXPECTED` now carries only this reader's own structure claims — byte order,
  IFD count, sensor-IFD index, opcode-list presence, malformed tags — none of
  which any external tool reports. Every tag *value* (dimensions, levels,
  geometry, orientation) is cross-checked live by `tests/metadata_oracle.rs`
  instead of trusted from a table one past session typed by hand; the layer-0
  packing check now compares against the file's own live `StripByteCounts`
  rather than a second hand-typed copy of it.

## [0.1.0] - 2026-10-29 [YANKED]

Initial crates.io publish (DEC-028), **yanked** ~10 minutes later and superseded
by 0.1.1 — identical source, trimmed published package. It remains on crates.io
only as a yanked version; use 0.1.1.
