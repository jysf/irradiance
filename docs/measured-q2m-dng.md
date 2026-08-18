# A measured Leica Q2 Monochrom DNG

Read with `exiftool` on 2026-08-15 (`L1025901.DNG`). This is the reference
structure STAGE-001 and STAGE-002 are built against. **Re-verify against your own
file before relying on it** — one camera, one firmware, one frame.

```
[SubIFD] PhotometricInterpretation : Linear Raw      <- 34892, NOT BlackIsZero
[SubIFD] SamplesPerPixel           : 1               <- monochrome, no CFA
[SubIFD] Compression               : Uncompressed    <- no SOF-3 needed
[SubIFD] BitsPerSample             : 14
[SubIFD] BlackLevel / WhiteLevel   : 512 / 16383
[SubIFD] ImageWidth x ImageHeight  : 8424 x 5632
[SubIFD] RowsPerStrip              : 5632            <- ONE strip, whole image
[SubIFD] StripByteCounts           : 83026944
[SubIFD] ActiveArea                : 0 0 5632 8392
[SubIFD] DefaultCropOrigin / Size  : 12 24 / 8368 5584
[SubIFD] OpcodeList1               : FixBadPixelsConstant
[SubIFD] OpcodeList3               : WarpRectilinear
[IFD0]   Orientation               : Rotate 90 CW
```

## Layer-0 oracle: the packing arithmetic closes exactly

```
8424 x 5632           = 47,443,968 px
47,443,968 x 14 bits  = 664,215,552 bits = 83,026,944 bytes  ==  StripByteCounts
```

Tightly packed 14-bit, **no row padding**, single strip. Assert this in the
unpacker: it is a free correctness check available before any oracle tooling.

## What this deletes

No demosaic. No white balance. No colour matrix. `PhotometricInterpretation:
Linear Raw` with `SamplesPerPixel: 1` means the sensor plane *is* the image.
That is what makes PROJ-001 ~550–700 lines rather than ~1,300–1,600.

## What it adds

**The DNG opcodes are real, not hypothetical.** `WarpRectilinear` is a radial
polynomial geometric correction, and the Q-series 28 mm lens is designed around
software distortion correction — skipping it means the output matches no
reference render. `FixBadPixelsConstant` runs on the raw plane before anything
else.

Geometry is three-stage: `ActiveArea` → `DefaultCropOrigin`/`Size` →
`Orientation`. Final image 8368 x 5584 (46.7 MP).

## Re-verified 2026-08-16 against a SECOND frame

`L1021223.DNG` (86 MB, `LEICA Q2 MONO`, shot 2025-12-30) was read with the same
tool. **Every structural value above reproduced exactly** — `Linear Raw`,
`SamplesPerPixel: 1`, 14-bit, `Uncompressed`, 8424 x 5632, `RowsPerStrip: 5632`
(single strip), `StripByteCounts: 83026944`, `BlackLevel/WhiteLevel: 512/16383`,
`ActiveArea: 0 0 5632 8392`, `DefaultCropOrigin/Size: 12 24 / 8368 5584`, and both
opcode lists present.

**One line does not reproduce, and it is a category error to have listed it here
as a camera constant:**

| Tag | `L1025901.DNG` | `L1021223.DNG` |
|---|---|---|
| `[IFD0] Orientation` | `Rotate 90 CW` | **`Horizontal (normal)`** |

`Orientation` records **how the camera was held for that exposure**. It is
per-frame, not per-camera. A decoder must read it from the file every time; a test
that hardcodes `Rotate 90 CW` as "the Q2M's orientation" will pass on one frame and
fail on the next. The three-stage geometry pipeline is unaffected — only its final
input varies.

Also observed on the second frame, not recorded above: **`SubIFD2` is a
full-resolution JPEG preview** at 8368 x 5584 — the `DefaultCropSize` dimensions.
So a Q2M DNG carries *two* preview sizes (1620 x 1080 in `SubIFD1`, full-size in
`SubIFD2`) alongside the sensor plane. Any "find the raw IFD" logic that picks by
largest dimensions would pick `SubIFD` correctly (8424 x 5632 is wider), but only
just — 8424 vs 8368. **Select on `SubfileType: Full-resolution image` plus
`PhotometricInterpretation: Linear Raw`, never on size.**

## Note on IFD0

`IFD0` is `YCbCr` / JPEG / 8-bit, and `SubIFD1`/`SubIFD2` likewise — those are the
embedded previews, which is all crustyimg's Tier-1 path has ever seen. The
`SubIFD` with `SubfileType: Full-resolution image` is the sensor data.
