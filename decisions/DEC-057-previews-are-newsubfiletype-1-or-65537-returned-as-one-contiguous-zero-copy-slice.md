---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-057
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-12
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/lib.rs
  - fuzz/fuzz_targets/ifd.rs
  - tests/corpus/manifest.toml
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-062-preview-slice-accessors-on-container-zero-copy-a-u8-dimensio.md

tags:
  - previews
  - container
  - zero-copy
  - stage-018
---

# DEC-057: A preview is an IFD with NewSubfileType 1 or 65537, returned as one contiguous zero-copy slice

## Decision

`Container::previews()` walks the IFDs `Container::parse` already collected, in walk order
(IFD0, then each IFD's SubIFDs depth-first). It reports an IFD as a preview **only** when its
`NewSubfileType` is exactly:

- `1`, a reduced-resolution image (a primary preview), or
- `65537` (`0x10001`), the DNG 1.2 alternate preview.

No other value counts, whatever its bits:

- `0`: the main image, which is the sensor plane in a DNG;
- `4`, `5`: transparency masks;
- `8`, `9`: a depth map and a reduced-resolution depth map;
- `16`: an enhanced image;
- `65540`: a semantic mask;
- any value the DNG spec does not name.

An absent `NewSubfileType` means `0` (TIFF 6.0 default), so it is not a preview. **Size is
never used.**

Each preview is returned as **one byte slice borrowed from the caller's input**
(`&'a [u8]`), plus its dimensions, `Compression`, `PhotometricInterpretation`,
`SamplesPerPixel`, `BitsPerSample` and file offset. The range comes from
`StripOffsets`/`StripByteCounts`:

- **One strip:** that strip's range.
- **Several strips, each starting exactly where the previous one ends:** the one range
  that spans them all.
- **Gaps, overlaps or out-of-order strips:** refused as **Unsupported**. A consumer would
  have to glue the pieces together, and "zero-copy" would be a lie.
- **Tiled previews (`TileOffsets` present) and old-style `JPEGInterchangeFormat`
  previews:** refused as Unsupported, too. Neither appears in the held corpus.
- **Mismatched strip-tag counts, zero strips, or a range outside the file:** Corrupt.

irradiance never decodes a preview, and never checks that its bytes are a valid JPEG.

The iterator yields `Result<Preview<'a>, Error>` per candidate IFD. One bad preview IFD is
an `Err` item, and iteration **continues** to the next IFD: a broken thumbnail must not hide
a good full-size preview. An IFD whose `NewSubfileType` can't be read is an `Err` item, not
a silent skip (DEC-012's "unknown is not no").

Every tag read in the preview path is allocation-free: a new `pub(crate)` indexed reader
replaces `uints()`, which builds a `Vec`. The accessor performs no heap allocation.

## Context

monorail-000 / onyx-000 (the macOS monochrome app suite) asked for zero-copy preview access
for its Preview Lie Detector (`feedback/2026-11-11-monorail-000-requests.md`, item 5).
`docs/future-plans.md` § PROJ-004 locked these rules on 2026-11-02:
- select on `NewSubfileType`;
- return `&'a [u8]` tied to the input, not to `&self` as `Container::payload` does;
- the host decodes, never irradiance.

The SPEC-062 design probe walked all 11 held tier-B DNGs with an independent Python TIFF
walker and checked them with exiftool 13.55 (`-a -G1 -s -n`). It found 21 preview IFDs:
- every one has `NewSubfileType = 1` and exactly one strip;
- each is either uncompressed RGB (`Compression 1`, `Photometric 2`) or baseline JPEG
  YCbCr (`Compression 7`, `Photometric 6`);
- none use tiles or `JPEGInterchangeFormat`.

The sensor IFD always has `NewSubfileType = 0`.

The full-size previews are nearly as large as the sensor plane. On the Q2M, 8368×5584 sits
next to the 8424×5632 plane; on the K-3 III, 6192×4128 sits next to 6304×4224. A
size-based rule passes on these files only by luck.

`Compression = 7` is shared by the K-3 III's lossless-JPEG **sensor plane** and every
baseline-JPEG **preview**, so compression can't tell them apart either.

## Alternatives Considered

- **Any non-zero `NewSubfileType`, or bit 0 set.** Rejected. Bit 0 is also set in `5`
  (reduced transparency mask) and `9` (reduced depth map). Neither is a preview of the
  image, and a consumer comparing "the preview" with the sensor would get a mask.
- **Select by size: everything but the largest image is a preview.** Rejected, for the
  reasons in Context. It also breaks the moment a writer stores a 1:1 preview of an
  uncropped plane.
- **Return strips as an iterator of slices**, so non-contiguous strips can be supported.
  Rejected for v1: no held file has a multi-strip preview, and every consumer named so far
  hands one buffer to a JPEG decoder. It can be added later without breaking anything,
  since `Preview` is `#[non_exhaustive]` (DEC-052).
- **Stop at the first bad preview IFD** (iterator of `Preview`, wrapped in an outer
  `Result`). Rejected: one malformed thumbnail would hide every other preview.
- **Decode the JPEG, or validate its SOI/EOI markers.** Rejected: that is out of scope for
  a decoder-free crate (`library-not-application`). A future, opt-in check could live
  beside it.
- **Follow the MakerNote preview** (Pentax writes one into its MakerNote; exiftool lists it
  as `[Pentax] PreviewImageStart`). Rejected: a MakerNote is vendor-private and is not part
  of the IFD tree irradiance walks.

## Consequences

- Two new `Error` variants (Unsupported and Corrupt, named in SPEC-062), each classified in
  `Error::kind()` (DEC-051). Range errors reuse `Truncated` / `ValueOverflow`.
- `Preview<'a>` and `PreviewKind` are `#[non_exhaustive]` (DEC-052), so fields and kinds
  can be added in a 0.2.x release.
- The corpus manifest pins every held file's preview list (index, kind, dimensions,
  compression, photometric, offset, length), taken from exiftool. Like `raw_checksum`, a
  change in exiftool's answer or in ours shows up by name.
- Adding tiled or non-contiguous previews later means relaxing one Unsupported arm. The
  selection rule is unaffected.
