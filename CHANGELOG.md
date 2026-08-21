# Changelog

All notable changes to this app. Newest at top.

Keep an `## [Unreleased]` section on top and add entries under it as you work —
the patch lane files fixes here (`[Unreleased] → Fixed`), and a release spec
(DEC-006) promotes the section to a version heading when it cuts the tag.

## [Unreleased]

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
  - Verified against `exiftool 13.55` on all **7** corpus files (5 `II` / 1 `MM`
    / 1 PEF), including the Pentax's malformed `BlackLevelRepeatDim`, which
    costs the tag and not the file.
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
- CI: `fmt --check`, `clippy -D warnings`, `test`, `cargo deny check licenses`,
  an MSRV check, and the lint red-proof.
  ⚠ CI **cannot** verify decode correctness — tier-B corpus files are never
  committed (DEC-003), so a green badge does not mean the decoder is bit-exact.

### Known gaps

- No pixel decode or unpack yet — `StripOffsets`/`StripByteCounts` are read as
  **tags** only. The unpacker and DEC-008's two-path (`bits % 8`) rule are
  STAGE-002.
- No typed tag model: `irradiance::ifd` widens `BYTE`/`SHORT`/`LONG` to `u32`
  and returns `Error::UnexpectedFieldType` for `RATIONAL` and the signed types.
  SPEC-004 owns that.
- `cargo deny` does not reach `fuzz/`; its dependencies' licences were checked
  by hand and recorded in DEC-011.

### Changed

### Fixed
