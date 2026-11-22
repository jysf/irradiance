# Parsed-but-unvalidated fields — audit

- **Tree audited:** `dfffd75` (main, after PR 301).
- **By:** Claude (claude-opus-5-5), 2026-11-21. Read-only: no code was changed.
- **Trigger:** a Codex review found three bugs of one shape. A tag was parsed,
  but its meaning was never checked before it drove processing. The three were
  `CFALayout` ignored, `CFAPlaneColor` misread, and opcode versions unchecked
  (plus non-finite `WarpRectilinear` values). PR 301 fixed them. This audit looks
  for the rest of that class.
- **Method:** every tag and opcode parameter that the decode and develop paths
  read was traced from `src/ifd.rs` to where it is used, in `src/plane.rs`,
  `src/opcode.rs`, `src/develop.rs`, `src/color.rs`, `src/demosaic.rs` and
  `src/warp.rs`. The out-of-domain behaviour below comes from **reading the
  code**, except where a row says *corpus-checked*. Those rows were run against
  the Tier-B corpus with `exiftool` and `irr` (built with `--features irr`).
  Each fix spec should start with the repro, which then doubles as its red-proof
  (`oracle-must-be-shown-red`).
- **Scope widened on purpose:** the same failure also happens when a tag is
  *never read at all* but changes what the pixels mean. That case is listed
  separately (§3). The PR 301 bug `CFALayout ignored` was exactly that shape
  before the fix.

Line numbers are at `dfffd75`.

## 1. The systemic finding: a malformed interpretation tag silently becomes its default

This is the largest gap, and it cuts across the per-tag table below.

`DEC-012` makes every non-structural tag **interpretation-class**. If such a tag
is malformed (wrong count, wrong field type, a fractional `RATIONAL` where an
integer is read, a bad offset), `Container::cost_the_field`
(`src/ifd.rs:2042`) pushes the tag onto `Sensor::malformed_tags` and returns
`None`. The develop paths then apply the DNG default for an *absent* tag.

**`malformed_tags` is consulted in exactly one place:**
`opcode::check_required_processing` (`src/opcode.rs:1133`) for the three opcode
lists. That is the `SPEC-056` / `DEC-053` "tolerant parse, refusing develop"
pattern. For every other tag, a malformed value and an absent value develop
the same way, and the result is `Ok(())`:

| Tag malformed | Default the develop applies | Effect |
|---|---|---|
| `CFALayout` | `1` (rectangular) | **Bypasses PR 301's `UnsupportedCfaLayout`.** A staggered layout whose tag has an odd field type is demosaiced as rectangular. |
| `CFAPlaneColor` | `[0, 1, 2]` | **Bypasses PR 301's `UnsupportedCfaPlaneColor`.** Example: a `SHORT`-typed tag (`cfa_bytes` accepts only `BYTE`/`UNDEFINED`, `src/ifd.rs:1724`). |
| `BlackLevel` | `0` | Blacks are lifted, and the colour path gets a cast from the black offset. |
| `WhiteLevel` | `2^BitsPerSample − 1` | Scale is wrong. A 14-bit file whose white is 15000 develops about 8% dark. |
| `BlackLevelRepeatDim` | `[1, 1]` | Refused when `BlackLevel` has more than one value (`black_pattern_dims`, good). Harmless when it has one value: this is the Pentax K-3 III Monochrome case `DEC-012` was written for. |
| `ActiveArea` | the whole raw plane | Masked border photosites end up in the image. `GainMap` and black-pattern phases are anchored to the wrong origin. |
| `DefaultCropOrigin` / `DefaultCropSize` | `(0,0)` / the active size | The output has a different size and framing. A fractional `RATIONAL` origin is enough: `uints` refuses a non-integral ratio (`src/ifd.rs` `TYPE_RATIONAL` arm), so the field is dropped. |
| `Orientation` | `1` | The output is unrotated. |
| `ColorMatrix2` / `ForwardMatrix2` / `CalibrationIlluminant1/2` | absent | Falls back silently to single-illuminant colour (`interpolation_weight` returns `None`, `src/color.rs:1077`). |
| `ForwardMatrix1` | absent | Falls back silently to the `ColorMatrix1` path. That path is valid, but it gives a different rendering. |
| `CFAPattern` / `CFARepeatPatternDim` / `AsShotNeutral` | — | **Refused** (`NotAColorFilterArray`, `UnsupportedCfaPattern`, `MissingColorCalibration`). These are safe today. |

`DEC-012`'s own rationale for tolerance has aged. It accepted a malformed
`BlackLevelRepeatDim` because *"PROJ-001 does not even consume [it] yet"*. It now
is consumed (`DEC-061`, `DEC-068`), as are the CFA tags (`SPEC-024`) and the
colour tags (`SPEC-026`).

**Recommended shape (one spec, not one per tag):** keep the tolerant parse, so
metadata and previews still read and the Pentax file still opens. Then extend
`check_required_processing`'s "malformed → refuse at develop" rule to every
interpretation tag the chosen develop path consumes. Keep the existing per-tag
nuance: a malformed `BlackLevelRepeatDim` is only fatal when `BlackLevel` has
more than one value. The new variant (or a generalised `UnreadableOpcodeList`
sibling) goes at the end of `Error`, with a `kind()` arm and the
`tests/error_kind.rs` counts updated (`DEC-051`).

## 2. Per-field table — tags and opcode parameters the library reads

Key. **D** = discriminator (an enum-like value that selects behaviour).
**N** = a numeric value with a domain. "Refused" means a typed `Error`.
"Malformed → §1" means the silent default in §1 applies.

### 2a. Plane location and decode (`src/ifd.rs` `sensor()`, `src/plane.rs`)

| Field | D/N | Checked where | Out-of-domain today |
|---|---|---|---|
| `NewSubfileType` | D | `is_sensor_ifd` `src/ifd.rs:1940`, must be 0 | Non-zero means the IFD is not a candidate. **First match wins.** `sensor()` takes `matches.first()` (`:2131`) and never refuses when several IFDs qualify. `sensor_candidates` exists so tests can assert one. Silent only on a file with two qualifying planes. |
| `PhotometricInterpretation` | D | `is_sensor_ifd`, ∈ {34892, 32803} | Anything else is not a candidate. **The mono develop never checks it** (§2c). |
| `SamplesPerPixel` | D | `is_sensor_ifd`, must be 1 | Anything else is not a candidate, so a demosaiced `LinearRaw` (SPP 3) gives `NoSensorIfd`. Refused. |
| `Compression` | D | `plane_kind` `src/plane.rs:1151`; `from_code` `src/ifd.rs:524` | Only 1 and 7 are decoded; anything else is `UnsupportedCompression`. Refused. |
| `BitsPerSample` (uncompressed) | D | `plane_kind`, `SUPPORTED_BITS` | Unsupported depths are `UnsupportedBitDepth`. Refused. Only the first value is read; SPP 1 makes that sufficient. |
| `BitsPerSample` (Compression 7) | D/N | **not checked against the frame** | The SOF-3 precision `P` is used as is, and `BitsPerSample` is never compared with it. When `WhiteLevel` is absent, the default white is `2^BitsPerSample − 1` (`resolve_white`, `src/develop.rs:704`). So a P=16 frame tagged 12-bit clips at 4095. |
| `ImageWidth` / `ImageLength` | N | structural; layout checks; `check_strip_shape` `src/plane.rs:1941` | A strip or tile frame-shape mismatch is `LjpegPlaneShapeMismatch`. Byte counts are checked (`PackedSizeMismatch`, `StripBytesExceedFile`). Refused. |
| `RowsPerStrip` | N | `StripWalk::new` `src/plane.rs:1206` | Absent or 0 with several strips is `MissingRowsPerStrip`. Refused. |
| `StripOffsets` / `StripByteCounts` | N | `StripWalk`, `check_strips` | Count mismatch, overrun and truncation are all refused. |
| `TileWidth` / `TileLength` / `TileOffsets` / `TileByteCounts` | N | `tile_grid` `:1443`, `check_tile_rows` `:1594`, `check_tiles` | Zero size or a wrong tile count is `InvalidTileLayout`; sub-byte tile rows are `UnsupportedTileRowBits`. Refused. |
| SOF-3 `P`, `Nf`, `Ss` (predictor), `Pt` | D/N | `src/ljpeg.rs` | Refused, with typed `Ljpeg*` errors (SPEC-036..038). Out of scope for this table except the `P`/`BitsPerSample` row above. |

### 2b. CFA model (`parse_cfa` `src/ifd.rs:2089`, `BayerPhase::from_cfa` `src/demosaic.rs:170`)

| Field | D/N | Checked where | Out-of-domain today |
|---|---|---|---|
| `CFARepeatPatternDim` | D | `from_pattern`, must be `[2, 2]` | `UnsupportedCfaPattern`. Refused. A malformed tag defaults to `[0, 0]`, which is also refused. |
| `CFAPattern` | D | `from_pattern`: an RGGB-family multiset with diagonal greens | `UnsupportedCfaPattern`. Refused. A malformed tag gives `color_filter = None`, then `NotAColorFilterArray`. |
| `CFALayout` | D | `from_cfa` (PR 301), must be 1 | `UnsupportedCfaLayout`. Refused when well-formed. **Malformed → §1.** |
| `CFAPlaneColor` | D | `from_cfa` (PR 301), must be `[0, 1, 2]` | `UnsupportedCfaPlaneColor`. Refused when well-formed. **Malformed → §1.** |

### 2c. Levels and geometry (`src/develop.rs`, shared by the colour path)

| Field | D/N | Checked where | Out-of-domain today |
|---|---|---|---|
| `BlackLevel` (scalar, array, `RATIONAL`) | N | `resolve_levels` `:565`, `resolve_levels_with` `:635`, `resolve_colour_levels` `src/color.rs:1457` | `black >= white` is `InvalidLevels`. Refused. A fractional value rounds to nearest (`DEC-068`). **The mono default `BlackLevels::FirstValue` uses only the first value of a non-uniform pattern.** That is by design (`DEC-061`), but it is silent. **Malformed → §1.** |
| `BlackLevelRepeatDim` | D | `black_pattern_dims` `:686` | A mismatch with the count is `InvalidBlackLevelPattern`. Refused. Malformed → §1 (harmless when there is one value). |
| `WhiteLevel` | N | the same `InvalidLevels` check | `white <= black` is refused. **A `WhiteLevel` above `2^BitsPerSample − 1` is not checked**, so the output never reaches full scale (silently dim). That is benign for real files. **Malformed → §1.** |
| `WhiteLevel` array (colour) | N | `resolve_colour_levels` takes `.first()` | Same as above. |
| `ActiveArea` | N | `resolve_geometry` `:152` | Empty, inverted or out-of-plane is `InvalidActiveArea`. Refused. **Malformed → §1.** |
| `DefaultCropOrigin` / `DefaultCropSize` | N | `resolve_geometry` | Zero size or outside the active area is `InvalidDefaultCrop`. Refused. **Malformed (including a fractional `RATIONAL`) → §1.** |
| `Orientation` | D | `resolve_geometry`, `1..=8` | Anything else is `UnsupportedOrientation`. Refused. **Malformed → §1.** |
| `MaskedAreas` | N | `masked_areas`, passed through only | Not consumed by develop. No pixel effect. |
| **`PhotometricInterpretation` on the mono path** | D | **none:** `develop_core` `:1661` never looks at `photometric` or `color_filter` | `develop_into` on a Bayer (`32803`) sensor returns `Ok(())` with the mosaic rendered as grey (a checkerboard). `FixBadPixelsConstant`'s neighbour average also mixes colours. The colour path has the mirror-image guard (`NotAColorFilterArray`). The mono path has none. |

### 2d. Colour (`src/color.rs`)

| Field | D/N | Checked where | Out-of-domain today |
|---|---|---|---|
| `AsShotNeutral` | N | `wb_diagonal` `:1187`; `reciprocal_rational` | Wrong count, zero, negative or a zero denominator is `MissingColorCalibration`. Refused. |
| `ColorMatrix1/2` | N | `matrix_from_rationals` `:1211`; `inv` `:1274` | Wrong count, a zero denominator or a singular matrix is `MissingColorCalibration`. Refused. The values are `i32` ratios, so they are always finite. |
| `ForwardMatrix1/2` | N | `matrix_from_rationals` | The shape is checked. **Its content is not.** An FM that does not map the neutral `(1,1,1)` to D50 is used as is. A malformed or one-sided FM pair falls back to the CM path silently (§1). |
| `CalibrationIlluminant1/2` | D | `cct_of_illuminant` `:1105` | **An unknown code (including 0 "unknown" and 255 "other", the DNG 1.6 `IlluminantData` case) silently takes the single-illuminant fallback**, so `ColorMatrix2`/`ForwardMatrix2` are ignored. No error, and no signal. |
| Interpolation weight `g` | N | clamped to `[0, 1]`; degenerate steps fall back | Safe. |
| Matrix output | N | `quantize` clamps | Safe. |

### 2e. Opcodes (`src/opcode.rs`, `src/warp.rs`)

| Field | D/N | Checked where | Out-of-domain today |
|---|---|---|---|
| Opcode `id`, `flags`, `version` | D | `check_required_processing` `:1133`; `MAX_SUPPORTED_OPCODE_VERSION` `:760` | Unknown mandatory, unapplied mandatory, too-new and unreadable lists are all refused (PR 301 and earlier). Good. |
| `WarpRectilinear` `N` | D | length must be exactly `4 + 48N + 16` | `N = 0` fails the length check. **`N` is not checked against "1 or the number of planes".** That is harmless on one plane, since K1 is used. |
| `WarpRectilinear` `kr`, `kt`, `cx`, `cy` | N | finite (PR 301); `kt != 0` gives `UnsupportedWarpTangentialTerms` | **Finite but degenerate values are accepted**, for example `kr0 = 0` (everything samples the centre) or `cx = 1e300`. Under the default-crop framing the sampler clamps to the edge (`bilinear_sample` `src/warp.rs:226`), which gives a flat or smeared image with `Ok(())`. `FullActiveArea` refuses an off-image centre (`full_active_area_admits`). Only reachable with a hostile file. |
| `FixBadPixelsConstant` `Constant` | N | the length must be 8 | Any value is meaningful. Safe. |
| `FixBadPixelsConstant` `BayerPhase` | D | **not validated (0..3), and ignored** | Applied only on the mono path, where there is no mosaic, so ignoring it is correct. It would matter only with the §2c mono-on-CFA gap. |
| `GainMap` `Top/Left/Bottom/Right` | N | `prepare` `:691` | An empty area means the whole image (`DEC-069`). An area past the image is clipped. Safe. |
| `GainMap` `Plane` | D | `prepare` | **`Plane ≥ 1` on a one-plane image silently applies to nothing.** A mandatory correction then counts as "applied". Arguably right (there is no such plane), but it is not reported. |
| `GainMap` `Planes`, `MapPlanes`, `RowPitch`, `ColPitch`, `MapPointsV/H` | D | `parse_gain_map_params` `:500` (non-zero, exact length) | Refused (`MalformedOpcodeParams`). `MapPlanes > Planes` is legal and handled. |
| `GainMap` `MapSpacingV/H`, `MapOriginV/H` | N | finite, and spacing > 0 | Refused. |
| `GainMap` gains | N | finite only | **A negative gain is accepted** and clamped to 0 by `apply` (`:660`), so the area goes black. A zero gain does the same. The DNG spec does not forbid them; no real file is known to carry them. |
| `GainMap` count | N | `MAX_GAIN_MAPS` = 16 | More is `TooManyGainMaps`. Refused. |

## 3. Tags that change pixel meaning but are never read

The PR 301 bugs started out this way. `grep` over `src/` (2026-11-21) finds no
reader for any of the following. Each one, when present, means the pixels
differ from what the develop assumes. Today the develop returns `Ok(())`
regardless.

| Tag | What ignoring it does | Likelihood on real input |
|---|---|---|
| `CameraCalibration1/2` (50723/50724), `AnalogBalance` (50727) | Colour path only. DNG builds the camera transform from `AnalogBalance · CameraCalibration · ColorMatrix` (and the inverse on the `ForwardMatrix` path). Skipping the per-unit calibration skews the white balance and colour. Under the spec's signature rule the matrix applies when `ProfileCalibrationSignature` is absent or matches `CameraCalibrationSignature`. | **Corpus-checked, live.** Pixel 7 Pro: `CC1 = diag(0.8594, 1, 0.3408)`, `CC2 = diag(0.6162, 1, 0.5703)`, no signatures, so it applies. Pixel 4a: `diag(1.0235, 1, 0.9555)`. 5D Mark III: `diag(1.002, 1, 1.0062)`. **`irr color` develops all three with exit 0.** So the Pixel 7 Pro's blue channel is uncorrected by a factor of 2–3 and nothing reports it. `AnalogBalance` is `1 1 1` on every held file. |
| `LinearizationTable` (50712) | The samples are **table indices**, not linear values. Tone comes out wrong across the whole range. | **Not in the corpus.** The one held M Monochrom (CCD) file is uncompressed and has no table. Leica's M8/M9-era "compressed" DNG mode is widely reported to be 8-bit with a `LinearizationTable`, which would put this in the mono market. Unverified here; a fix spec should get such a file first. |
| `DNGBackwardVersion` (50707) | The spec's own guard. A reader *should not* read a file whose backward version is newer than the version it supports. Not reading it is the general form of the opcode-version bug. | Low today, and it grows with every DNG revision. A cheap, high-leverage refusal. |
| `SampleFormat` (339) | `SampleFormat = 3` (IEEE float) with `BitsPerSample = 16` and `Compression = 1` **decodes the half-floats as integers**. `= 2` (signed) is misread the same way. 24/32-bit and deflate are refused already, by depth or compression. | Low (float DNGs are usually HDR merges, mostly deflate). The output is completely wrong when it happens. |
| `BlackLevelDeltaH` / `BlackLevelDeltaV` (50715/50716) | Per-column and per-row black offsets are dropped. The result is residual banding and a slight black error. | Medium. |
| `RowInterleaveFactor` / `ColumnInterleaveFactor` (50975/51157), `SubTileBlockSize` (50974) | The plane's rows, columns or blocks are stored out of order, so the image is scrambled. | Low. |
| `DefaultScale` / `BestQualityScale` (50718/50780) | Non-square pixels come out at the wrong aspect ratio. | Low (some older Nikon and Fuji bodies). |
| `FillOrder` (266) | `2` means LSB-first bits, and sub-byte samples get scrambled. | Very low (not used in DNG practice). |
| `BaselineExposure` (50730) | A rendering intent, not a pixel meaning. The output is darker than Adobe's. | Common. **Not a correctness gap** for a linear develop. Listed so it is not mistaken for one. |
| `PlanarConfiguration` (284), `Predictor` (317) | N/A. SPP 1 makes planar configuration moot, and `Predictor` applies only to deflate/LZW, which are refused. | — |

## 4. Ranked gap list — risk of silently wrong pixels

Ranked by (chance on real files) × (how wrong) × (how invisible), with cheap
fixes for the exact PR 301 class broken out.

| Rank | Gap | Why here | Fix size |
|---|---|---|---|
| **1** | `CameraCalibration1/2` and `AnalogBalance` never read (§3) | **The only gap confirmed live on a held file.** The Pixel 7 Pro colour-develops with `Ok` and its blue calibration (0.34 / 0.57) unapplied. The 4a and 5D III carry smaller calibrations. The `SPEC-035` ColorChecker oracle would catch this once a chart is captured. Until then, nothing does. | S–M (apply: two matrix products on the existing colour path, plus the signature rule) |
| **2** | Malformed interpretation tags silently become defaults (§1) | One systemic hole. It covers black, white, active area, crop and orientation, and it **re-opens PR 301's two CFA fixes** through a malformed tag. The fix pattern already exists (`DEC-053`). No held file trips it today (`malformed_tags` is empty on the Pixel 7 Pro, corpus-checked), so it is ranked for breadth, not for a known live hit. | M (one spec, one new `Error` variant, per-path tag sets) |
| **3** | Mono develop accepts a Bayer plane (§2c) | Any consumer that calls `develop_into` on whatever `sensor()` returns gets a grey checkerboard and `Ok(())`. The colour path already has the mirror-image guard. | S (refuse when `color_filter.is_some()` or `photometric != 34892`, unless an explicit option says otherwise) |
| **4** | `LinearizationTable` never read (§3) | Wrong tone across the whole range, and `Ok(())`. It plausibly affects older Leica mono files, but is unverified in the corpus. Refusing is one tag check. Applying it is a small spec: a LUT before levels. | S (refuse) / M (apply) |
| **4b** | `DNGBackwardVersion` never read (§3) | The spec-defined "you don't understand this file" guard. It generalises the opcode-version fix to the whole file. Every held file declares 1.1–1.4, so a refusal above 1.7 breaks nothing in the corpus. | S |
| **5** | `SampleFormat` never read (§3) | Rare, but completely wrong pixels when it happens (16-bit float read as integer). | S (refuse when not 1) |
| **6** | Compression 7 `P` not tied to `BitsPerSample` (§2a) | Silent clipping when `WhiteLevel` is absent; otherwise only metadata disagrees. | XS (refuse when `P != BitsPerSample`, or default white from `P`) |
| **7** | `BlackLevelDeltaH/V` never read (§3) | Banding and black error, usually small. | S (refuse) / M (apply) |
| **8** | Unknown `CalibrationIlluminant` falls back silently (§2d) | Colour is approximately right and nothing reports it. Could be a signal rather than an error. | S |
| **9** | Several qualifying sensor IFDs: first wins (§2a) | No known file does this. A refusal (or picking by rule) closes the last "picks the first of several" case that the code's own comment warns about. | S |
| **10** | Row/column interleave and `SubTileBlockSize` never read (§3) | Rare. Scrambles the plane when present. | S (refuse) |
| **11** | Degenerate finite warp, negative/zero gains, `GainMap` `Plane ≥ 1` (§2e) | Needs a hostile file. Never a panic. | XS each |
| **12** | `DefaultScale`, `WhiteLevel` above the bit maximum, mono `FirstValue` blacks, `FillOrder` | Small effects, by-design choices, or practically absent. Record them, don't spec them yet. | — |

### Suggested spec grouping

- **Spec 0, "apply CameraCalibration and AnalogBalance" (rank 1):** a colour-path
  correctness fix on real files, independent of the refusals below. It needs a
  `DEC` for the signature rule and a red-proof (develop the Pixel 7 Pro with the
  calibration forced to identity, and show the result differs).
- **Spec A, "refuse what we cannot honour" (ranks 4-refuse, 4b, 5, 7-refuse, 10):**
  read each tag's presence or value in `sensor()` and refuse at develop with
  typed errors. All are the same shape, and one red-proof fixture can carry
  each tag.
- **Spec B, "malformed interpretation tags refuse at develop" (rank 2):**
  generalise `DEC-053` and amend `DEC-012`'s rationale.
- **Spec C, "mono develop refuses a CFA plane" (rank 3):** plus rank 6 if
  convenient, since both are one-line gates on the develop entry.
- **Later, by demand:** applying `LinearizationTable` and `BlackLevelDelta*`
  instead of refusing them. Ranks 8, 9 and 11 can ride along with whichever
  spec touches the file.

Each new `Error` variant goes at the **end** of the enum (cargo-semver-checks),
with an `Error::kind()` arm and the `tests/error_kind.rs` counts updated
(`DEC-051`).

## 5. Side finding: a stale doc note

`docs/conformance-matrix.md` (Pixel row) says the Pixel 7 Pro *"carries a
malformed `BlackLevel` (50714)"*. Since `DEC-068`, that fractional `RATIONAL`
reads and rounds to 1024. `irr ifd` reports `malformed_tags []` (corpus-checked).
The note should be corrected in the next doc touch.
