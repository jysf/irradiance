---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-082
  type: decision
  confidence: 0.8                    # placement measured with dng_validate, exiftool, irradiance and Quick Look; Lightroom and Windows not tested
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-005
repo:
  id: irradiance

created_at: 2026-11-24
supersedes: null
superseded_by: null

status: proposed                     # SPEC-082 design; jysf accepts at the design PR or at ship

deciders: [claude]

affected_scope:
  - src/dng/write.rs
  - src/write_types.rs

tags:
  - writer
  - preview
  - layout
---

# DEC-082: The writer stores a caller JPEG preview as a SubIFD of the raw IFD0

## Decision

DEC-071 §5 allows zero or one caller-encoded JPEG preview, stored as a reduced-resolution
subfile. SPEC-082 places and describes it like this:

1. **Placement.** The raw plane stays IFD0 (DEC-075 §1). A preview is one IFD with
   `NewSubFileType` 1, pointed to by a `SubIFDs` (330, LONG, count 1) entry in IFD0. There is
   still no next-IFD chain: IFD0's next offset stays 0.
2. **Layout.** Header, EXIF block (DEC-072), IFD0 and its out-of-line values, the preview IFD
   and its out-of-line values, the JPEG bytes, then the raw strip, last as before. Every IFD,
   value and strip starts at an even offset, and every pad byte is 0.
3. **The preview IFD's tags**, all fixed by the writer: `NewSubFileType` 1, `ImageWidth`,
   `ImageLength`, `BitsPerSample` (8, or 8, 8, 8), `Compression` 7, `PhotometricInterpretation`
   (1 for gray, 6 for YCbCr), `StripOffsets`, `SamplesPerPixel` (1 or 3), `RowsPerStrip`
   (the height), `StripByteCounts`, `PlanarConfiguration` 1. One strip.
4. **The caller describes the JPEG, the writer does not decode it.** The caller passes the
   bytes, the width, the height and the colour (gray or YCbCr), because a JPEG's
   dimensions and component count live in its SOF header, which the writer does not read.
5. **Checks.** The bytes must be non-empty and start with the JPEG SOI marker (`FF D8`),
   else `PreviewNotJpeg`. That is the only content check, two bytes, so the file never claims
   `Compression` 7 for a PNG or nothing. Width and height must be 1 to 65535 (JPEG's own
   limit), else `PreviewDimensionsInvalid`. Both run in `plan`, after the XMP check and
   before the `OpcodeList1` walk (DEC-081 §4), so `written_len` keeps its parity.
6. **Under every framing** the preview is written the same way. Its dimensions are the
   caller's and are not checked against the plane.

## Context

SPEC-082's design probe (2026-11-24) rebuilt a SPEC-081 output of the K-3 III with a JPEG
preview in three layouts and checked each with Adobe `dng_validate` 1.7.1, exiftool 13.55,
irradiance's `Container::previews` and macOS Quick Look (`qlmanage -t`, with a solid red
preview so a red thumbnail proves the preview was used):

| Layout | dng_validate | irradiance | exiftool | Quick Look |
|---|---|---|---|---|
| Raw IFD0, preview as IFD1 (next-IFD chain) | warns `This file has Chained IFDs, which will be ignored by DNG readers` | (not tried) | | |
| Raw IFD0, preview as its SubIFD | no warnings | sensor unchanged, one preview, exact bytes | `SubIFD:PreviewImage` byte-identical | uses the preview (red) |
| Preview IFD0, raw as its SubIFD (the cameras' shape) | no warnings | sensor unchanged, one preview, exact bytes | as above | uses the preview (red) |

A one-component (gray) JPEG with `PhotometricInterpretation` 1 and `SamplesPerPixel` 1 is
also accepted in both SubIFD layouts. `dng_validate` does not decode the preview: a gray
JPEG declared as YCbCr was accepted too, which is why the caller's declaration must be
right and the writer cannot check it.

## Alternatives Considered

- **Preview in IFD0, raw in a SubIFD**, the shape the held cameras write. Equally clean in
  every probe. Rejected for v1: it moves every raw tag SPEC-061 and SPEC-081 put in IFD0
  into a SubIFD, so it restructures the shipped layout and its tests for no measured gain.
  It stays open as a later layout if a reader is found that shows only IFD0's image.
- **A next-IFD chain (IFD1).** Rejected: `dng_validate` warns that DNG readers ignore it.
- **The writer reads the SOF header** for the dimensions and component count. Rejected: it
  parses caller bytes, which DEC-071 §5 and the frame's non-goals rule out, and it is a new
  fuzz surface for a value the caller already has.
- **No content check at all.** Rejected: a file whose preview IFD claims JPEG but holds
  something else is broken for every reader. Two bytes is the cheapest check that catches
  the likely mistake (passing PNG or an empty buffer).

## Consequences

- **Positive:** SPEC-061's and SPEC-081's IFD0 is unchanged, a file without a preview is
  byte-identical to SPEC-081's output, and Quick Look shows the caller's preview.
- **Negative:** a reader that shows only IFD0's image (some generic TIFF viewers) shows the
  raw plane, not the preview. Lightroom and Windows were not tested.
- **Neutral:** `PreviewColorSpace` and the other DNG preview tags are left out; adding one is
  additive.

## Validation

- SPEC-082's oracles: the tag table, `Container::previews` returning the exact bytes,
  exiftool's `SubIFD:PreviewImage`, `dng_validate` with no warnings on the K-3 III and Q2M.

## References

- Related specs: SPEC-061, SPEC-062, SPEC-081, SPEC-082
- Related decisions: DEC-057, DEC-071, DEC-072, DEC-075, DEC-081
