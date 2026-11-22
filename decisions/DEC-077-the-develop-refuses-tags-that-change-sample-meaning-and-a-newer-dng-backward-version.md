---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-077
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-22
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/opcode.rs
  - src/plane.rs
  - src/lib.rs
  - tests/error_kind.rs
  - decisions/DEC-051-error-kind-three-way-classification-and-the-wildcard-free-match.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-086-refuse-tags-the-develop-cannot-honour-yet.md

tags:
  - refusal
  - develop
  - plane
  - stage-027
---

# DEC-077: the develop refuses tags that change what the samples mean, and a newer DNG backward version

## Decision

### 1. What is read

New public fields on the `#[non_exhaustive]` `Sensor` (DEC-049), so the change is
additive. Each is read under DEC-012's contract: absent is `None`; a malformed value is
`None` and its tag is pushed onto `malformed_tags`.

From IFD0:
- `dng_version: Option<[u8; 4]>`, `DNGVersion` (50706), `BYTE` × 4.
- `dng_backward_version: Option<[u8; 4]>`, `DNGBackwardVersion` (50707), `BYTE` × 4.

From the sensor IFD (the one `sensor()` already picks):
- `sample_format: Option<Vec<u16>>`, `SampleFormat` (339), `SHORT` × `SamplesPerPixel`.
- `linearization_table: Option<Vec<u16>>`, `LinearizationTable` (50712), `SHORT` × N.
- `black_level_delta_h: Option<Vec<f64>>` and `black_level_delta_v: Option<Vec<f64>>`,
  `BlackLevelDeltaH/V` (50715/50716), `SRATIONAL` × N, each as `num / den`. A zero
  denominator is malformed.
- `sub_tile_block_size: Option<[u32; 2]>`, `SubTileBlockSize` (50974), `SHORT` or `LONG` × 2.
- `row_interleave_factor: Option<u32>`, `RowInterleaveFactor` (50975), `SHORT` or `LONG`.
- `column_interleave_factor: Option<u32>`, `ColumnInterleaveFactor` (51157), `SHORT` or `LONG`.

The values are stored, not only their presence, so a later spec can apply the table and
the deltas without a second parse change.

### 2. What the develop refuses

Every develop entry refuses, before it writes to `dst`, a sensor carrying any of the
following. The check runs inside `opcode::check_required_processing`, the one gate every
mono, colour and demosaic entry already calls first. So the refusal lands on every path
without a per-entry call.

**The version guard** (first):
- The effective backward version is `DNGBackwardVersion` when present. When it is absent,
  it is `DNGVersion` with its last two bytes zeroed, the specification's default. When
  both are absent there is no check.
- It is refused when it is greater than `[1, 7, 1, 0]`, compared byte by byte from the
  first, the same supported version as `MAX_SUPPORTED_OPCODE_VERSION` (`0x0107_0100`).
- The error is `Error::UnsupportedDngVersion { backward_version: [u8; 4] }`, carrying the
  effective version.

**The tag set** (then, in ascending tag order):

| Tag | Honoured (an exact no-op) | Refused |
|---|---|---|
| `SampleFormat` (339) | every value 1 | any value other than 1 |
| `LinearizationTable` (50712) | never | present |
| `BlackLevelDeltaH` (50715) | every entry 0 | any entry not 0 |
| `BlackLevelDeltaV` (50716) | every entry 0 | any entry not 0 |
| `SubTileBlockSize` (50974) | `[1, 1]` | anything else |
| `RowInterleaveFactor` (50975) | 1 | anything else |
| `ColumnInterleaveFactor` (51157) | 1 | anything else |

- A tag in `malformed_tags` is refused the same as a value outside the honoured column.
  Its presence alone is the problem, so it does not wait for SPEC-087's general rule.
- The error is `Error::UnsupportedTag { tag: u16 }`, the first offending tag.
- An honoured value is a no-op. The develop of such a file is byte-identical to the
  same file without the tag.
- `LinearizationTable` has no honoured value, not even the identity. Telling an identity
  table from a near-identity one is the apply spec's work, and no held file has a table.

**Not refused at develop:** metadata (`ifd::sensor()` and the CLI's `irr ifd`) and plane
reads (`plane::unpack_into`, `PlaneDecoder`, `units`) still succeed on these files. A plane
caller gets the stored samples as they are and can see the new fields. Only a develop,
which would give them a meaning they do not have, refuses.

### 3. The Compression 7 precision must equal `BitsPerSample` (decode time)

The develop entries take an already-decoded plane, so they never see the SOF-3 frame
header. This check therefore sits at decode, beside `check_strip_shape`:
- every Compression 7 strip or tile whose frame precision `P` is not the sensor's
  `BitsPerSample` is refused;
- the error is `Error::LjpegPrecisionMismatch { precision: u8, bits_per_sample: u32 }`;
- it is raised on every plane path that decodes a Compression 7 unit (`unpack_into`,
  `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`), after the shape check.

This is a plane-read refusal, the one exception to §2's "plane reads succeed". The frame
and the tags disagree about the sample range, and `resolve_white` defaults white from
`BitsPerSample`. So a P = 16 frame tagged 12-bit would clip at 4095 with `Ok`. It is the
same class as `LjpegPlaneShapeMismatch`, the frame disagreeing with the tags about the
plane.

### 4. Errors and kinds

Three variants, appended at the end of `Error` in this order (cargo-semver-checks; DEC-051):

| Variant | `ErrorKind` |
|---|---|
| `UnsupportedDngVersion { backward_version: [u8; 4] }` | `Unsupported` |
| `UnsupportedTag { tag: u16 }` | `Unsupported` |
| `LjpegPrecisionMismatch { precision: u8, bits_per_sample: u32 }` | `Corrupt` |

`UnsupportedTag`'s `Display` names the tag (for example "LinearizationTable (50712) changes
what the samples mean and is not applied yet"). One variant for the tag set, not one per
tag, keeps the refusal open-ended: a later tag joins the table without a semver change.

### 5. `DEVELOP_VERSION` is unchanged

These are refusals only. Every develop that still succeeds gives the same bytes, so
`DEVELOP_VERSION` stays `"8"`, as DEC-053 set for refusals.

## Context

**The gap.** The STAGE-027 audit (`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md`
§3 and §4, ranks 4, 4b, 5, 6, 7 and 10) found that none of these tags is read, so a file
carrying one develops with `Ok` and wrong pixels. That is the class PR 301 fixed for
`CFALayout`, `CFAPlaneColor` and the opcode version.

**Re-measured at SPEC-086's design (2026-11-22, `exiftool -n`, every held DNG):**
- No held file has `LinearizationTable`, `BlackLevelDeltaH/V`, `RowInterleaveFactor`,
  `ColumnInterleaveFactor` or `SubTileBlockSize`.
- `SampleFormat` appears only on the Nikon D300S, as 1.
- `DNGBackwardVersion` is 1.1 to 1.4 where present. The Leica M Monochrom has
  `DNGVersion` 1.0.0.0 and no backward version, so its effective version is 1.0.0.0.
- The SOF-3 precision equals `BitsPerSample` on every Compression 7 file: the three Pixels
  16 / 16, the M Monochrom Typ 246 12 / 12, the Pentax K-3 III Monochrome 14 / 14
  (a scan of each file's SOF-3 headers).

So no held file is refused, and every held develop is byte-identical.

## Provenance

- **Class 1:** Adobe DNG Specification 1.7.1.0: `DNGBackwardVersion` (the reader-version
  guard and its default from `DNGVersion`), `SampleFormat` (TIFF 6.0 §Section 19),
  `LinearizationTable`, `BlackLevelDeltaH/V`, `SubTileBlockSize`, `RowInterleaveFactor`,
  `ColumnInterleaveFactor`, and the lossless JPEG Compression 7 definition.
- No SDK code was read for this decision.

## Alternatives Considered

- **Apply `LinearizationTable` and `BlackLevelDelta*` instead of refusing.** Deferred to a
  spec by demand (the audit's "Later"). No held file carries either, and an apply needs its
  own oracle.
- **One variant per tag.** Rejected. Seven variants for one rule, and every later tag
  would be a semver change.
- **Refuse these tags at plane read too.** Rejected. The stored samples are what they are;
  a plane caller may want them (for example to apply a table itself). Only the develop
  gives them a meaning.
- **Default white from `P` instead of refusing a mismatch.** Rejected. The develop never
  sees `P`, and a mismatched file is inconsistent about its own range.
- **A higher supported version.** Rejected. 1.7.1.0 is the version the library is written
  against, and the opcode guard already uses it.

## Consequences

- A file carrying any of the seven tags with a value the develop cannot honour, or a
  backward version above 1.7.1.0, refuses on every develop entry with a typed error and
  `dst` untouched. Metadata and plane reads still work.
- A Compression 7 plane whose frame precision disagrees with `BitsPerSample` refuses at
  decode.
- `Error` grows from 72 to 75 variants: `Unsupported` 24 to 26, `Corrupt` 31 to 32.
- `DEVELOP_VERSION` stays `"8"`; no held output changes.
- Applying `LinearizationTable` and `BlackLevelDelta*` is a later spec, by demand.
