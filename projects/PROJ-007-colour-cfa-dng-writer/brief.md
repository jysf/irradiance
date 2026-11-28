---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-007
  status: proposed
  activity: requirements
  priority: medium
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-27
shipped_at: null

closed_reason: null

value:
  thesis: >
    irradiance 0.2.7 writes a DNG only for a monochrome plane. If the same opt-in `write`
    feature also writes a colour filter array (CFA) plane with its colour tags,
    uncompressed or lossless-compressed, then irradiance becomes the only permissive
    pure-Rust crate that can produce a colour raw file Lightroom and Adobe's validator
    develop. That covers both a file made from scratch (the "turn any photo into a RAW"
    demo, invented sensors included) and a camera's sensor plane re-wrapped as DNG. The
    mono output does not change.
  beneficiaries:
    - "The maintainer's \"turn any photo into a RAW\" demo (jysf/irradiance-demo): a JPEG run backwards into a Bayer mosaic or an invented CFA, written as a DNG that Lightroom develops"
    - "A future raw-to-DNG converter (crustyimg RAW-in/DNG-out, Sony ARW, PEF, CR2): a camera's CFA plane re-wrapped with its colour tags"
    - "Processing tools that write a mosaiced result back (dark-frame subtraction, flat-field, stacking)"
    - "onyx and every mono caller: lossless compression and tiles also shrink mono output (a Q2 Monochrom is 95 MB uncompressed)"
  success_signals:
    - "Every held colour DNG (Canon 5D3, Nikon D300S, Pixel 3a/4a/7 Pro, Leica M10/Q2/Q3/M11/SL3 where the reader decodes them) re-wraps at all three framings. Each output re-parses bit for bit (samples, levels, CFA tags, colour tags, opcode lists) and develops through irradiance's colour path to the same bytes as its source"
    - "Adobe dng_validate 1.7.1 gives no warnings on every written colour file, including an invented non-Bayer CFA, and renders it"
    - "A Compression 7 file decodes through irradiance's own SOF-3 decoder and dng_validate to the uncompressed samples, bit for bit, for mono and CFA"
    - "Mono output is byte-identical to 0.2.7 for every existing input, the default build gains nothing, and `[dependencies]` stays empty"
    - "Scored at the demo, not gated: a synthetic DNG from the demo opens and develops in Lightroom Classic"
  risks_to_thesis:
    - "No named consumer. PROJ-005 held its scope because onyx answered six questions first. Here the requirements are the maintainer's demo idea, so the API may fit the first real converter badly. Mitigation: every option is additive and `#[non_exhaustive]`, and a consumer's answers re-frame the later stages"
    - "Invented sensors may develop in dng_validate and still not in Lightroom, which may refuse or mis-render non-Bayer patterns it has never seen. The demo's 'watch the demosaic struggle' needs a manual check that no gate can make"
    - "Compression changes the size query. `written_len` can no longer be exact before encoding, so the two-call API of DEC-071 §7 needs a decision, and it must not break mono callers"
    - "The LJ92 encoder is the largest new code in the writer, and tiles add a layout that the mono writer never had. Either could turn its stage from M into L"
    - "Synthetic raws can pass as real originals. Raw files are treated as proof of authenticity (contests, journalism), and the demo makes a believable DNG from any JPEG. The writer stays neutral (it writes only what the caller passes), but the demo must not claim a real camera by default and must mark its files as synthesised (added 2026-11-28)"
    - "Lightroom may ignore the written colour tags. It looks up its own profile by `UniqueCameraModel`, so a file that names a real camera renders with Adobe's profile for that camera, not the caller's `ColorMatrix`. An invented sensor needs a name of its own (added 2026-11-28)"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap:
  - item: "Inputs beyond DNG: Sony ARW (uncompressed and lossless first), colour PEF, Canon CR2, Nikon NEF; each a reader project of its own that this writer then serves"
    kind: goal
    horizon: later
    resume_when: "samples are held (ARW: expected from 2026-11-28) and a reader project is framed"
  - item: "Computed colour tags (a ColorMatrix or AsShotNeutral the writer derives rather than copies)"
    kind: goal
    horizon: later
    resume_when: "a converter needs a tag its source lacks and a camera-matrix source with clean provenance is found"
  - item: "`WriteOptions` from a `Sensor` (future-plans writer extension 1), now for mono and CFA"
    kind: goal
    horizon: next
    resume_when: "the first converter consumer, or onyx, says it would use it"
---

# PROJ-007: Colour (CFA) DNG writer

> **Proposed 2026-11-27.** This is a planning frame only. It has no specs and no DECs. Status
> stays `proposed` until jysf activates it. IDs were computed from `origin/main` (`f6c52da`)
> plus open PR 327 on 2026-11-27. PROJ-006 is running in parallel, so mint spec and DEC IDs
> with `just new-spec` at design time, never from this file.

## What This Project Is

This project extends `irradiance::dng::write` (feature `write`, 0.2.7) from one monochrome
plane to a colour filter array plane. That means `PhotometricInterpretation` 32803, the four
CFA tags, per-channel levels, and the colour tags a DNG reader needs to develop it. The
writer copies those tags from the caller and never computes them. Any CFA that DNG can
describe is in scope, invented ones included. Output can be uncompressed strips, as today,
or lossless Compression 7 in tiles. The work is additive: a mono write gives the same bytes
as 0.2.7, and the project ends in its own 0.2.x release.

## Why Now

`docs/future-plans.md` ("Later — the DNG writer") lists "CFA planes with their colour tags"
as extension 4. It names crustyimg or Sony ARW as the trigger and calls it a project of its
own. The trigger that has actually arrived is the maintainer's demo idea, recorded
2026-11-27:

> Turn any photo into a RAW. Run the pipeline backwards on a JPEG: undo the display curve,
> undo the colour matrix, put the result back into a Bayer mosaic, add a believable amount
> of sensor noise, and write a real DNG with 0.2.7's writer. Then open it in Lightroom and it
> develops. You could even let people design their own colour-filter pattern ("invent a
> sensor") and watch the demosaic struggle with it.

The 0.2.7 writer cannot do this, because it writes `LinearRaw` only. No other permissive Rust
crate writes a colour DNG, so the demo shows off the one thing only irradiance has. The same
core also serves the raw-to-DNG re-wrap later. That is the second use the maintainer named,
and ARW samples are expected from 2026-11-28.

## Answers that set the scope (jysf, 2026-11-27)

1. **Consumer and what it writes.** Both a processed or synthetic CFA plane and a re-wrapped
   sensor plane. No consumer is named yet. The first use is the "turn any photo into a RAW"
   demo. The writer does not care which of the two it is given: it writes the samples,
   levels and tags the caller passes. A held DNG re-wrapped is the oracle, because it has a
   source to compare against.
2. **Inputs first.** The held colour DNGs. ARW, the caller's own plane and colour PEF are
   considered but are not v1 input work. The writer takes a plane, so "the caller's own
   plane" comes for free, and ARW and PEF need readers first. CR2, X-Trans or other non-2×2
   CFAs, and NEF are listed as later triggers. CR3 (CRX) and Nikon HE stay out on patents.
3. **Colour tags.** Copied only, as typed options. "Perhaps compute some at some point" is a
   roadmap item (`later`). It needs a provenance source for any camera matrix.
4. **Framing.** All three, as for mono (DEC-073). The premise that an `ActiveArea` cut shifts
   the CFA phase does not hold for DNG. DEC-040 and the DNG specification put the
   `CFAPattern` origin at the `ActiveArea`'s top-left. So writing only the active area, with
   `ActiveArea` omitted, keeps every pixel's colour, and the `DefaultCrop*` tags never move
   samples. The phase moves only in two cases: a non-DNG source whose pattern is relative to
   the plane, which is a converter's job when it sets `ActiveArea`, and a pixel crop at an
   odd origin, which DEC-073 forbids. `OpcodeList1` keeps DEC-075 §5's allow-list.
   `OpcodeList2` and `OpcodeList3` work on the active-area image, including the Pixels'
   per-plane `GainMap`s, whose `Top`/`Left`/pitch are active-area-relative, so they are copied
   verbatim. The probe below confirmed this on held files.
5. **Encoding.** Uncompressed strips, lossless Compression 7, tiles and previews are all
   wanted. Previews already exist (DEC-082) and only need checking with a CFA plane.
   Compression and tiles are a stage of their own (STAGE-037), and they serve mono too.
6. **Release and API.** Additive to the existing `write` API and feature, with its own
   release (STAGE-038). The likely shape is a new optional CFA/colour options value on
   `WriteOptions`: `None` means the mono write of 0.2.7. The design fixes the exact shape in
   a DEC.

## Probes run while framing (2026-11-27, read-only, scratch outside the repo)

- **Reuse.** About two thirds of the mono writer carries over unchanged: the layout (DEC-075
  §1–3, §9), framing (DEC-073), opcode lists (DEC-075 §5), metadata and EXIF/XMP (DEC-072,
  DEC-074, DEC-081), the preview (DEC-082), the error model, determinism and the
  `written_len`/`write_into` parity.
  - **Changes.** `PhotometricInterpretation` becomes 32803 for a CFA plane. `CFARepeatPatternDim`,
    `CFAPattern`, `CFAPlaneColor` and `CFALayout` are new. The colour tags are new, with
    DEC-078's shape rules mirrored at write time (nine entries per matrix, or 3×n for
    n-colour; one per colour for `AnalogBalance` and `AsShotNeutral`; no zero denominators).
    For a CFA file `ColorMatrix1` is required, as DNG requires.
  - **Levels.** `BlackLevelRepeatDim` 2×2 is already accepted. `BlackLevel` is `u32` only, so
    the Pixel 7 Pro's RATIONAL black (262172/256) cannot round-trip until the writer takes a
    rational black. That belongs in the core spec.
  - **New checks.** The pattern's size against `CFARepeatPatternDim`, every pattern index
    below the number of plane colours, and `CFALayout` 1 only (the reader refuses others).
- **dng_validate 1.7.1** reports no warnings on the Canon 5D3, Pixel 7 Pro and Leica Q2. On
  the Nikon D300S it gives one warning, `RowsPerStrip` count 12 in the source's preview. All
  four carry `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `AsShotNeutral`, the CFA tags and
  `BestQualityScale`.
  - Three of the four carry `DefaultScale`, `BayerGreenSplit`, `AntiAliasStrength` and the
    four rendering hints.
  - Two carry `CameraCalibration1/2`, `AnalogBalance`, `NoiseProfile`, `ShadowScale`,
    `ProfileName` and `ProfileEmbedPolicy`.
  - One (the Canon) carries `ForwardMatrix1/2`, `ProfileHueSatMap*`, `ProfileLookTable*` and
    the calibration signatures.
  - The minimal tag set dng_validate accepts without a warning on a synthetic file is a
    design-time probe for the core spec.
- **CFA phase on real files.** No held colour DNG has an odd `ActiveArea` origin. The Canon's
  is (80, 122). The Nikon, Pixels and Leica Q2/Q3/SL3 have (0, 0), and the M10 and M11 have
  no `ActiveArea`. The Nikon D300S has an odd `DefaultCropOrigin` (x 15, y 10), which is a
  tag only. The odd-`ActiveArea` case can only be tested with a synthetic fixture, and the
  core spec's red-proof needs one. The patterns vary: RGGB (Canon, Nikon, Pixel 4a, Leicas),
  BGGR (Pixel 3a, 7 Pro) and GBRG (Q2).
- **The reader's `Cfa`** already models any `repeat_dim`, pattern and plane colours, so an
  invented pattern round-trips through the parser. Only the develop refuses anything that is
  not 2×2 RGB (`UnsupportedCfaPattern`), so for invented sensors the develop oracle is
  dng_validate's render.
- **The LJ92 encoder.** `src/ljpeg.rs` already has a test-only encoder
  (`encode_lj92`, `encode_lj92_two_component`, `#[cfg(test)]`). STAGE-037 starts from it,
  with the SOF-3 decoder as its oracle.
- **Patents.**
  - Lossless JPEG (ITU-T T.81 process 14) is patent-clean (DEC-035).
  - DNG, TIFF 6.0 tiles and caller JPEG previews carry Adobe's DNG patent grant for compliant
    writers and need nothing new.
  - Fujifilm's X-Trans patents cover the sensor hardware, not a file that describes a 6×6
    pattern. Re-check before any named X-Trans preset ships.
  - The "unprocessing" method the demo would use (Brooks et al., CVPR 2019, Google Research)
    turned up no patent in one search, and its reference code invites reuse. It is the
    demo's code, not the library's, so the demo checks it before it builds the inverse
    pipeline.
  - Lossy mosaic JXL (which dng_validate offers) is out (DEC-055 trigger 1).

## Success Criteria

See `value.success_signals`. In short: held colour DNGs re-wrap bit-exact and develop
identically, dng_validate accepts and renders every output (invented CFAs included),
Compression 7 decodes back bit for bit, and mono output and the default build do not change.

## Scope

### In scope
- A CFA plane in the public writer: the four CFA tags, any pattern DNG allows (2×2 Bayer
  first, then non-2×2 and 4-colour patterns), per-channel and rational levels, all three
  framings.
- Colour and profile tags as typed, shape-checked options, copied verbatim.
- An optional caller JPEG preview with a CFA plane (DEC-082, unchanged).
- Lossless Compression 7 output in tiles, for mono and CFA, with an LJ92 encoder written in
  the crate from ITU-T T.81 (clean-room, provenance row).
- Test-support synthesis of CFA planes (a mosaic from known RGB) for oracles. This is
  `tests/support` only.
- Raw-data digests: the writer computes `NewRawImageDigest` itself (MD5, patent-clean) or writes none. It never copies a caller's, because a digest copied from a source is wrong once the samples or their encoding change (added 2026-11-28).
- `LinearizationTable` carried as a typed option, since the reader already parses it (added 2026-11-28).
- `WriteOptions` from a `Sensor`, promoted from the roadmap: for a CFA re-wrap it fills about twenty fields, and the re-wrap oracle builds it anyway (added 2026-11-28).
- One 0.2.x release.

### Explicitly out of scope
- The inverse pipeline itself (undoing the curve and matrix, re-mosaicing, adding noise).
  That is the demo's code (jysf/irradiance-demo), not a library API (`library-not-application`).
  The library's job is the writer.
- Computing any colour tag (roadmap, `later`).
- New input readers: ARW, PEF, CR2, NEF (roadmap, `later`; each its own project).
- Developing non-2×2 CFAs in irradiance (the develop keeps `UnsupportedCfaPattern`).
- `CFALayout` other than 1 (staggered or SuperCCD, parked since PR 301), big-endian output,
  float or demosaiced `LinearRaw` RGB (extension 5), deflate (it needs zlib), and lossy or
  JXL codecs.
- Tags outside the typed set: `DNGPrivateData`, the MakerNote (`MakerNoteSafety`), `OriginalRawFileData` and `OriginalRawFileName`, and any unknown or private tag, are dropped. Adobe's converter keeps MakerNotes; this writer does not, and its docs say so (added 2026-11-28).
- DNG 1.6 triple-illuminant colour (`CalibrationIlluminant3`, `ColorMatrix3`, `ForwardMatrix3`, `IlluminantData`): neither read nor written today, and no held file carries it (added 2026-11-28).
- Policy for synthetic files (camera names, a "synthesised" marker in `Software` and XMP). That is the demo's, recorded under `risks_to_thesis`; the writer only writes what it is given.
- Rewriting `CFAPattern` or opcode lists for a cut. Cuts stay at the `ActiveArea` edge, where
  nothing needs rewriting.

## Considerations added 2026-11-28

After the frame merged (PR 360), a review of what a colour writer adds beyond the mono one
found eleven points. The first three change what STAGE-035 builds. The next three are lines
in STAGE-035's DECs. The last five wait for the stage they touch.

1. **Lightroom's profile lookup by `UniqueCameraModel`** (risk above). STAGE-035's module docs
   say what a caller's camera name does.
2. **Synthetic files** (risk above). The demo's policy; the writer stays neutral.
3. **Raw-data digests** (scope above). STAGE-035's colour-tags DEC.
4. **`LinearizationTable`** (scope above). STAGE-035's rendering-tags spec.
5. **Unknown and private tags are dropped** (out of scope above). STAGE-035's core DEC states it.
6. **Triple-illuminant colour** (out of scope above).
7. **`NoiseProfile` for the demo.** Already in STAGE-035's tag list. The demo should compute
   it from the noise it adds, so Lightroom's denoise treats the file correctly.
8. **The browser.** The demo runs as wasm, so an uncompressed 120 MB buffer is the user's
   download. STAGE-037 should land before the demo goes public, or the demo uses a smaller
   resolution. `wasm-check` gains a CFA write in STAGE-035.
9. **`WriteOptions` from a `Sensor`** (scope above). Promoted into STAGE-035 as an [S] spec.
10. **More readers.** PROJ-005 never recorded a Lightroom, Capture One or Windows open, and
    colour raises the stakes. STAGE-036 adds a local-only lane that opens the outputs in
    darktable, RawTherapee, rawpy/LibRaw and Quick Look. These are tools that are run, never
    linked, like dnglab.
11. **The raw plane is IFD0.** DNG recommends a thumbnail in IFD0 with the raw in a SubIFD;
    the writer puts the raw in IFD0 (DEC-082). dng_validate does not warn and no probed tool
    cared, but an old reader might. Item 10's lane checks it.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-035 (proposed) — CFA plane core: 2×2 Bayer CFA plane, CFA and colour tags as typed options, rational levels, all three framings, held colour DNGs re-wrapped bit-exact and developing identically, dng_validate clean. ~165M tokens / ~$120 (with the promoted `from_sensor` spec).
- [ ] STAGE-036 (proposed) — Invented sensors: any CFA pattern DNG allows (non-2×2 repeat, 4-colour plane colours), with dng_validate's render as the develop oracle, a recorded Lightroom check and a local multi-reader lane. ~80M / ~$60.
- [ ] STAGE-037 (proposed) — Lossless Compression 7 and tiles, for mono and CFA: a public-crate LJ92 encoder and a tiled layout, plus the size-query decision. **Has an L.** ~150M / ~$110.
- [ ] STAGE-038 (proposed) — The 0.2.x release of the colour writer. ~45M / ~$30.

**Count:** 0 shipped / 0 active / 4 pending

**Cost estimate.** About 440M tokens / $320 for the project, against PROJ-005's 193M / $140
for one stage and one release. The baseline is PROJ-005's actuals: an [M] spec 17–36M, an
[S] 13–20M, stage orchestration about 0.7× the spec total, a release stage about 45M, and
about $0.73 per million tokens. The estimate is borrowed, so re-estimate at each stage's
first design. The release could be cut after STAGE-036 instead, if the demo wants the
uncompressed colour writer before compression. That would add a second release stage
(about 45M / $30).

## Dependencies

### Depends on
- PROJ-005 (shipped): the public mono writer, DEC-071 to DEC-075 and DEC-080 to DEC-082.
- PROJ-002 / PROJ-003: the colour develop (the re-wrap's develop oracle), DEC-040 (the phase
  origin), DEC-078 (the colour tags' shape rules), and the SOF-3 decoder (STAGE-037's oracle).
- PROJ-006 STAGE-034 (in flight): the Leica M10 and Q2 colour files reach the colour path.
  The re-wrap oracle uses them only once that lands.
- The local dng_validate reference (DEC-069 §6), for acceptance.

### External inputs (not `depends_on`)
- No consumer's answers yet. If crustyimg, onyx or the demo answer writer questions in a
  DEC of their own, they re-frame STAGE-036 and STAGE-037 as onyx's did for STAGE-017.
- ARW samples (expected 2026-11-28) do not gate any stage here. They gate the ARW reader
  project that would use this writer.
- A Lightroom Classic open of a synthetic and an invented-CFA file, done by hand.

### Enables
- The "turn any photo into a RAW" demo; a permissive raw-to-DNG converter; smaller mono
  exports for onyx.

## Project-Level Reflection

*Filled in when status moves to shipped.*
