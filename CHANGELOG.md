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
