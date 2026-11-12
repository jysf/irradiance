---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-051
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/lib.rs
  - tests/
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-040-error-kind-errorkind-needs-a-dec-first.md

tags:
  - error-kind
  - errorkind
  - classification
  - api
  - non-exhaustive
  - stage-010
  - spec-040
---

# DEC-051: `Error::kind()` — a three-way classification and a wildcard-free match

## Decision

Add a stable `Error::kind(&self) -> ErrorKind` where

```
#[non_exhaustive]
pub enum ErrorKind { Unsupported, Corrupt, CallerError }
```

so a consumer (pyirradiance's dataloader, feedback item 3) can split "a valid file
whose format/feature I don't implement — skip it" from "a malformed or hostile file —
bad data" from "my own code called the API wrong — a bug". The three kinds are fixed
(the spec's goal); `ErrorKind` is `#[non_exhaustive]` so a fourth kind could be added
later without breaking callers, and `Error` stays `#[non_exhaustive]` and unchanged
(no new variant, no `Display` change, no change to how any error is produced).

### The classification rubric

- **`CallerError`** — the outcome is determined by an **argument the caller passed**,
  not by the file: every buffer/slice length mismatch (the caller sized a `dst`/`src`
  wrong), and `PlaneTooLarge` (the caller's own `max_pixels` budget, DEC-050). Remedy:
  fix the call (resize the buffer, or raise/lower the budget).
- **`Unsupported`** — the input is **well-formed for its format** but uses a
  format, version, feature or variant this library does not implement; a different or
  future decoder could read it. Remedy: skip it / use another tool.
- **`Corrupt`** — the bytes are **malformed, truncated, internally inconsistent, or
  hostile** (exceed a structural limit, alias, or carry out-of-range values). Remedy:
  the file is bad.

### The full classification (every current variant)

**`CallerError`** (12): `PlaneBufferWrongLength`, `PlaneTooLarge`,
`SourcePlaneWrongLength`, `DevelopBufferWrongLength`, `WarpSourceWrongLength`,
`WarpDestWrongLength`, `FixBadPixelsPlaneWrongLength`, `OutputBufferWrongLength`,
`DemosaicSourceWrongLength`, `RgbBufferWrongLength`, `ColorSourceWrongLength`,
`LjpegPlaneWrongLength`.

**`Unsupported`** (18): `NotTiff`, `UnsupportedTiffVersion`, `NoSensorIfd`,
`UnsupportedCompression`, `UnsupportedBitDepth`, `UnsupportedStripLayout`,
`UnsupportedOrientation`, `UnsupportedMandatoryOpcode`, `UnsupportedRequiredOpcode`,
`UnsupportedWarpTangentialTerms`, `NotAColorFilterArray`, `UnsupportedCfaPattern`,
`MissingColorCalibration`, `LjpegUnsupportedProcess`, `LjpegUnsupportedComponentCount`,
`LjpegUnsupportedPrecision`, `LjpegUnsupportedPredictor`, `LjpegUnsupportedSampling`.

**`Corrupt`** (25): `Truncated`, `CyclicIfd`, `IfdDepthExceeded`, `TooManyIfds`,
`IfdTableBytesExceedFile`, `UnexpectedFieldType`, `ValueOverflow`,
`MalformedRationalValue`, `RationalValueOutOfRange`, `TagTooLarge`, `MissingTag`,
`OffsetOutOfRange`, `NoSensorIfdCandidatesMalformed`, `MissingRowsPerStrip`,
`PackedSizeMismatch`, `StripBytesExceedFile`, `InvalidActiveArea`,
`InvalidDefaultCrop`, `InvalidLevels`, `MalformedOpcodeParams`,
`LjpegInvalidPointTransform`, `LjpegInvalidHuffmanTable`, `LjpegMissingMarker`,
`LjpegUnexpectedMarker`, `LjpegPlaneShapeMismatch`.

*Later additions (each classified by its own decision, recorded here so the table
stays the one reference):* `UnreadableOpcodeList` → **`Corrupt`** (`DEC-053`,
`SPEC-056`, 0.2.1), making `Corrupt` 26 and the enum 56. `UnsupportedTileRowBits` →
**`Unsupported`**, `InvalidTileLayout` and `StripsAndTiles` → **`Corrupt`** (`DEC-056`,
`SPEC-058`), making `Unsupported` 19, `Corrupt` 28 and the enum 59.

The build must re-derive this table against the live enum and pin **every** variant in
a test (one representative constructed per variant, asserted to the kind above); the
table here is authored against `src/lib.rs` at 2026-11-10 (~55 variants) and is the
reference, but the compiler is the gate (below).

### The enforcement: a wildcard-free `match`, not a convention

`kind()` is a `match self { … }` over **every** variant with **no `_ =>` arm**. `Error`
is defined in this crate, so — even though it is `#[non_exhaustive]` for downstream
crates — the defining crate's match must be exhaustive, and a newly-added variant makes
`kind()` fail to compile until it is classified. That compiler error *is* the
"a new variant cannot land unclassified" guarantee; it is stronger than a test, and it
is the reason a wildcard arm is forbidden here. A test additionally pins the kind of a
representative of each variant so the *classification* (not just totality) is checked.

### The four judgment calls (flagged so the build and verify can challenge them)

- **`PlaneTooLarge` → `CallerError`.** It fires when `width·height` exceeds the caller's
  `max_pixels` (DEC-050). The lever is the caller's argument, so the remedy is on the
  caller's side (raise the budget, or decline the file). A *hostile forged-dimension*
  file is better caught — and classified `Corrupt` — by the parser guards upstream
  (`IfdTableBytesExceedFile`, `StripBytesExceedFile`); `PlaneTooLarge` is the budget
  gate, not the corruption gate.
- **`NotTiff` / `NoSensorIfd` → `Unsupported`.** Feeding a non-TIFF, or a valid TIFF
  with no RAW sensor IFD, to a RAW reader is "not a file I decode" — the batch-loader's
  skip case — not "bad bytes". (`NoSensorIfdCandidatesMalformed` is `Corrupt`: candidates
  existed but were malformed.)
- **`MissingColorCalibration` → `Unsupported`.** A file with no colour matrices cannot
  be colour-developed by this library; that is a missing-capability skip, distinct from
  `MissingTag` (`Corrupt`: a required low-level TIFF tag absent = a malformed container).
- **`InvalidActiveArea` / `InvalidDefaultCrop` / `InvalidLevels` → `Corrupt`.** These are
  out-of-range *values in the file*, not caller arguments, so they are bad data.

## Why

`Error` mixes three populations: unsupported-but-valid input, corrupt/hostile input, and
caller misuse (`*WrongLength`). A boolean `is_unsupported` would report a caller's
wrong-length buffer as "hostile file", and a two-way unsupported/corrupt split would put
caller bugs in one of those buckets. Three kinds let the consumer route each the way its
own logic needs (skip / log-bad-file / fix-my-code) without a per-variant stability
promise. Sequenced after SPEC-039 because that spec added the last new failure paths
(`PlaneTooLarge`, `StripBytesExceedFile`) the table must classify.

## Rejected alternatives

- **A boolean `is_unsupported()`** — rejected: collapses `Corrupt` and `CallerError`
  together, so a caller bug reads as a hostile file (the exact confusion the consumer
  asked to avoid).
- **A fourth kind (e.g. `ResourceLimited` for `PlaneTooLarge`)** — rejected for 0.2.0:
  the goal fixes three kinds; `#[non_exhaustive]` leaves the door open to add one later
  without breaking callers if a real need appears.
- **A `_ =>` wildcard arm** (defaulting unknown variants to, say, `Corrupt`) — rejected:
  it silently classifies every future variant and destroys the compile-time "cannot land
  unclassified" guarantee, which is the whole point of the exhaustiveness.
- **Per-variant `kind` stability promises** — rejected: only the three kinds are stable;
  a variant may be re-classified in a later release (it is advisory routing, not a
  contract on each variant).

## Mechanics

- `src/lib.rs`: add `ErrorKind` (`#[non_exhaustive]`, derive `Debug, Clone, Copy,
  PartialEq, Eq`) and `impl Error { pub fn kind(&self) -> ErrorKind { match self { … } } }`
  with no wildcard. No change to `Error`'s variants or `Display`.
- `tests/`: an exhaustive classification test (a representative of each variant → its
  expected kind); a red-proof is unnecessary here because the compiler enforces totality,
  but the test's own value is pinning the *assignment* so a re-classification is a
  visible diff.
- CHANGELOG `[Unreleased] → Added`: `Error::kind()` / `ErrorKind`. Provenance ledger row
  (own prior code; a classification over this crate's own `Error`).

## Scope and non-goals

- **In:** `ErrorKind` + `Error::kind()` + the wildcard-free match + the per-variant test.
- **Out:** no new `Error` variant, no `Display` change, no change to how any error is
  produced (SPEC-040 non-goals); no `is_unsupported` alias; the release (SPEC-042);
  determinism (SPEC-041).
