---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-085
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-006
repo:
  id: irradiance

created_at: 2026-11-27
supersedes: null                   # amends DEC-077 §3; DEC-077 stays active
superseded_by: null
status: proposed
deciders: [claude, jysf]

affected_scope:
  - src/plane.rs
  - src/lib.rs
  - tests/ljpeg_precision_binding.rs
  - decisions/DEC-077-the-develop-refuses-tags-that-change-sample-meaning-and-a-newer-dng-backward-version.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-097-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md

tags:
  - lossless-jpeg
  - plane
  - refusal
  - white-level
  - stage-034
---

# DEC-085: a lossless-JPEG precision below BitsPerSample decodes unchanged, and white still defaults from BitsPerSample (amending DEC-077 §3)

## Decision

### 1. The rule

A `Compression = 7` strip or tile whose SOF-3 frame precision is `P` is accepted when

> `P ≤ BitsPerSample ≤ 16`

and refused otherwise, with the existing `Error::LjpegPrecisionMismatch { precision,
bits_per_sample }` (`ErrorKind::Corrupt`). The parser already bounds `P` to `2..=16`
(`Error::LjpegUnsupportedPrecision` outside it), so the accepted set is every pair with
`2 ≤ P ≤ BitsPerSample ≤ 16`:

- **`P = BitsPerSample`**: accepted, as today. Every file that unpacks today is in this set.
- **`P < BitsPerSample ≤ 16`**: accepted for **any gap**, from `P = 2` up. This is the new
  case (the Leica M10 and M11: `P = 14`, `BitsPerSample = 16`).
- **`P > BitsPerSample`**: refused, as today. DEC-077 §3's reason stands.
- **`BitsPerSample > 16` with `P < BitsPerSample`**: refused. These pairs are refused
  today too, because `P` can never equal such a `BitsPerSample`. Lossless JPEG stores at
  most 16 bits, the plane is `u16`, and the SDK reads lossless JPEG only for
  `BitsPerSample` 8 to 16.

The check stays where DEC-077 §3 put it: `check_precision`, after `check_strip_shape`,
before any sample is decoded, on every path that decodes a Compression 7 unit
(`unpack_into`, `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`, strips and tiles).

### 2. The samples are stored as decoded

A frame of precision `P` decodes to the same `u16` values whatever `BitsPerSample` says.
There is no shift up to `BitsPerSample` and no rescale. The tags give the samples their
meaning (`BlackLevel`, `WhiteLevel`), as they do for every other plane.

### 3. White still defaults from `BitsPerSample`

`resolve_white` (mono) and `resolve_colour_levels` (colour) are unchanged. White is
`WhiteLevel` when present and `2^BitsPerSample − 1` when absent. It is never `2^P − 1`.
So a P = 14 frame in a 16-bit tag with no `WhiteLevel` develops against white 65535: its
brightest code reaches 25% of full scale. That is what the specification says and what
every reader measured does (Context). Both held files carry an explicit `WhiteLevel`.

### 4. What does not change

- No `Error` variant is added, removed or renamed. `LjpegPrecisionMismatch` keeps its
  fields, its kind and its `Display`. Only its doc comment narrows, to "the precision is
  above `BitsPerSample`, or `BitsPerSample` is above 16".
- `DEVELOP_VERSION` stays `"8"`, `MONO_MIX_VERSION` stays `"1"`. Only files that are refused
  today change, from an error to an `Ok`. Every held file that unpacks today has
  `P = BitsPerSample` (DEC-077 §Context's SOF-3 scan), takes the same code path, and
  gives the same bytes.
- DEC-077 §1, §2, §4 and §5 are untouched. §3 is amended by this decision.

## Context

**The refusal.** SPEC-086 (DEC-077 §3) required `P = BitsPerSample` on every Compression 7
unit. Its reason was the case `P > BitsPerSample`: a P = 16 frame tagged 12-bit would clip
at the default white 4095 with `Ok`. The equality also refused the opposite direction,
which no held file had then. On 2026-11-26 two colour Leica files showed it:

| File | Geometry | SOF-3 | `WhiteLevel` / `BlackLevel` | Decoded min / max | Samples above `WhiteLevel` |
|---|---|---|---|---|---|
| `LEICA-M10/f5381888.dng` (raw.pixls.us, CC0) | 5984×3992, one strip | P 14, Y 3992, X 2992, Nf 2, predictor 1, Pt 0 | 15000 / 0 | 81 / 16383 | 1,243,041 (5.2%) |
| `LEICA-M11-PB/leica_m11_01.dng` (Photography Blog, no licence, local only) | 9536×6336, one strip | P 14, Y 6336, X 4768, Nf 2, predictor 1, Pt 0 | 16383 / 1023 | 1009 / 16383 | 0 (78,144 at it) |

Both are `BitsPerSample = 16`, CFA RGGB, `DNGVersion` 1.4, `DNGBackwardVersion` 1.3, with
no opcode lists. Measured at SPEC-097's design (2026-11-27) with `check_precision`
relaxed in an uncommitted prototype; see the spec's `## Implementation Context`.

**What the readers do (driven, not assumed).**

- **DNG Specification 1.7.1.0** (the copy in the SDK 1.7.1 zip; its "New Information for
  Version 1.7.1.0" lists only `ColumnInterleaveFactor` and new tags, so these sections are
  1.7.0.0's text):
  - `BitsPerSample`: "Supported values are from 8 to 32 bits/sample."
  - `Compression`: for lossless JPEG, the stream's width, length and components need not
    match the strip's. Nothing is said about the frame precision.
  - `WhiteLevel`: default `2^BitsPerSample − 1` for unsigned integer images.
  - Chapter 5: values rescaled above 1.0 "should be clipped to 1.0". So samples above
    `WhiteLevel`, like the M10's, are legal and clip, as DEC-029 already holds.
- **Adobe DNG SDK 1.7.1, read** (class 4; no code copied):
  - `dng_lossless_jpeg_shared.cpp` `GetSof` checks only that `dataPrecision` is in
    2..16. `dataPrecision` appears in no other SDK file, so it is never compared with
    `BitsPerSample`.
  - `dng_read_image.cpp` reads lossless JPEG only for `BitsPerSample` 8 to 16.
  - `dng_ifd.cpp` defaults white to `2^fBitsPerSample[0] − 1` and calls a `WhiteLevel`
    above that invalid unless there is a `LinearizationTable`.
- **`dng_validate` 1.7.1, run:**
  - It validates both Leica files. Its stage 1 equals the prototype's plane sample for
    sample (0 of 23,888,128 and 0 of 60,420,096 differ).
  - Its stage 2 equals the clamp-at-`WhiteLevel` map exactly on the M10 (1,243,116 samples
    at 65535, which is every sample ≥ 15000). On the M11 it is within 1 code on 11
    samples.
  - It validates synthetic 16×16 LinearRaw files with (P, BitsPerSample) = (8, 16),
    (12, 16), (14, 16), (16, 12) and (16, 16), with no warning. Stage 1 of the (16, 12)
    file keeps samples up to 65470, above that file's default white of 4095. So the SDK
    accepts `P > BitsPerSample` and then clips: DEC-077 §3's hazard, which irradiance keeps
    refusing.
- **dnglab 0.7.2, run** (`analyze --raw-checksum`, `--meta`):
  - Its checksums on the two Leica files equal the prototype planes' MD5:
    `a1c6050a2cbfaf7d1b9a827ee3fd5bf6` (M10) and `9ae8c096b5ed99b86a88e52d8025168c` (M11).
  - It decodes the synthetic (12, 16), (14, 16) and (16, 12) files. On the (16, 12) file it
    gives the (16, 16) file's checksum: the samples are unshifted.
  - It refuses P = 8 at any `BitsPerSample` ("ljpeg: sof.precision 8"). That is a limit of
    its decoder, not a gap rule.
- **LibRaw 0.22.3, run** (`unprocessed_raw`, `dcraw_emu`):
  - It decodes both Leica files (`lossless_dng_load_raw`) identically to the SDK's stage 1.
  - It refuses the synthetic files as unidentified cameras, so it gives no evidence on
    the gap.
- **The white default, with `WhiteLevel` absent.** A scratch copy of the M10 had tag 50717
  renamed to a private number. All three readers then used white `2^16 − 1`:
  - the SDK's stage-2 maximum is 16383, unscaled (`2^14 − 1`);
  - dnglab reports `whitelevels: [65535]`;
  - LibRaw's linear output, with unit white balance, also peaks at 16383, against 65535
    when the tag is present.
- **rawspeed:** not driven. No rawspeed build or front end is installed here, and
  AGENTS.md forbids reading its source.

## Alternatives Considered

- **Default white from `P` when `WhiteLevel` is absent.** Rejected. The specification, the
  SDK's code, and dnglab and LibRaw when run all default from `BitsPerSample`. The develop
  never sees `P` either, so this would need `P` carried on the `Sensor` from a decode-time
  header, for an outcome no reader has.
- **Shift the samples up to `BitsPerSample` (`v << (BitsPerSample − P)`).** Rejected. No
  reader does it: SDK stage 1, dnglab and LibRaw all keep the decoded values. It would
  also move the M10's samples past its `WhiteLevel` 15000.
- **Accept `P < BitsPerSample` only where every sample fits, or only small gaps.**
  Rejected. An honest P-bit frame's samples are below `2^P ≤ 2^BitsPerSample`, so every
  gap fits. A corrupt stream can yield any `u16` at any `P`, today's `P = BitsPerSample`
  included, and the develop clamps such samples at white. So a narrower rule adds no
  safety, and the SDK accepts every gap from P = 2.
- **Also accept `P > BitsPerSample`, as the SDK does.** Rejected. DEC-077 §3's reason
  stands: the SDK's stage 1 of a (16, 12) file holds samples far above the 12-bit default
  white, and they clip with no error. A file whose frame is wider than its tag is
  inconsistent about its own range.
- **Accept any `BitsPerSample` above `P`, with no 16 ceiling.** Rejected. Lossless JPEG
  cannot exceed 16 bits, the plane is `u16`, and the SDK refuses lossless JPEG above 16.
  A 20- or 32-bit tag on a Compression 7 plane is a mismatch, as it is today.
- **Edit DEC-077 §3 in place.** Rejected. DEC-077 is accepted and shipped in 0.2.5. As
  with DEC-029 (amending DEC-008) and DEC-076 (amending DEC-034), the change gets its own
  record, and DEC-077 §3 gets a one-paragraph note pointing here.
- **A new error variant for `BitsPerSample > 16`.** Rejected. It is one rule with one
  remedy, and a variant is a semver addition for a case no file has shown.

## Consequences

- The Leica M10 and M11 colour DNGs unpack. With no further change they reach
  `develop_color_into`, `srgb_linear_plane_into`, `mono_mix_plane_into`, `develop_into`
  and `develop_with_into` with `mono_mix`, all `Ok` (prototype, 2026-11-27).
- A Compression 7 plane is now refused only when its precision is above
  `BitsPerSample`, or its `BitsPerSample` is above 16.
- No held output, determinism hash or Tier-B pin changes. Both versions stay as they are.
- A P < BitsPerSample file without `WhiteLevel` develops dark, by the specification.
  irradiance does not second-guess the tag.

## Validation

SPEC-097's Failing Tests: the accepted grid (every path, both byte orders, strips and
tiles, gaps from P = 2), the refused grid (`P > BitsPerSample`, `BitsPerSample` 17 to
32), the white-default test, and the Leica files against their dnglab checksums. Its
mutant list (a) to (i) shows each clause red.

## References

- DEC-077 §3 (amended), DEC-029 (samples above `WhiteLevel` are legal), DEC-047 (the
  frame's shape binding, which still runs first), DEC-061 / DEC-070 (the white and black
  maps, unchanged).
- SPEC-086 (the check), SPEC-097 (this change).
- Adobe DNG Specification 1.7.1.0: `BitsPerSample`, `Compression`, `WhiteLevel`, Chapter 5.
- Adobe DNG SDK 1.7.1 source: `dng_lossless_jpeg_shared.cpp` (`GetSof`),
  `dng_read_image.cpp` (`CanReadTile`, `ReadLosslessJPEG`), `dng_ifd.cpp` (`PostParse`,
  `IsValidDNG`). Read under DEC-069 §6's terms; never vendored, built in CI or linked.
