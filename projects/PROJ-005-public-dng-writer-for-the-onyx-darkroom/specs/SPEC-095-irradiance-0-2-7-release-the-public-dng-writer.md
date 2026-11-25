---
# A RELEASE SPEC is the spec for cutting a release — tagging, building the
# artifact, publishing it through each channel, and confirming it actually runs
# on a machine that didn't build it. See AGENTS.md "During ship" and
# docs/decisions/DEC-006.
#
# It uses the same task.* schema as a normal spec (so `just validate`,
# `just cost-audit`, and `just status` treat it as first-class), with
# `task.type: release`. The value it adds over a plain spec is the
# ## Release Pre-Flight checklist below: a generic, portable set of
# runtime/operational categories that every release hits, so each project
# stops re-earning the same gotchas one release at a time. This variant
# delegates the cut to an implementer via handoffs/HANDOFF-*.md.

task:
  id: SPEC-095
  type: release                    # epic | story | task | bug | chore | release
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-005
  stage: STAGE-032
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-006, DEC-028, DEC-055, DEC-067, DEC-071, DEC-080, DEC-082]
  constraints: [library-not-application, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-061, SPEC-081, SPEC-082, SPEC-094, SPEC-090]

value_link: "Puts the STAGE-017 writer on crates.io, so onyx's Darkroom v1 can depend on a published irradiance with the write feature."

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 25000000       # S; SPEC-090 (0.2.6) ran 27.2M with two SDK lanes this one drops, plus a packaged-crate writer check it adds
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-25
      notes: "main-loop, not separately metered (AGENTS.md section 4); measured at the stage close as orchestration"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-095: irradiance 0.2.7 release: the public DNG writer

## Context

The release spec of STAGE-032 (PROJ-005). irradiance 0.2.6 was published on 2026-11-23
(annotated tag `v0.2.6` on `50d533d`). Since then `main` has shipped, to `153f517`, the whole
of STAGE-017 (closed 2026-11-25, PR 342):
- **SPEC-061** (PR 329): the public writer behind the new `write` feature: `written_len`,
  `write_into`, `write_to_vec`, `WriteOptions`, three framings, levels and opcode lists
  copied, required `UniqueCameraModel`, and two `Error` variants present in every build
  (DEC-080).
- **SPEC-081** (PR 332): the EXIF block at a fixed base offset, XMP, eight descriptive
  ASCII tags and four rendering hints.
- **SPEC-082** (PR 336): an optional caller JPEG preview, stored as a SubIFD.
- **SPEC-094** (PR 339): an empty opcode list is refused, CI fuzzes `dng_write`, three
  doc links fixed.

Measured at design on `153f517`: `git diff --stat v0.2.6 153f517 -- src/ Cargo.toml` touches
only `src/dng/mod.rs`, `src/dng/write.rs`, `src/write_types.rs` (all new), `src/lib.rs` (the
two modules, two `Error` variants with their `Display` and `kind()` arms) and `Cargo.toml`
(the `write = []` feature). No reader, develop, colour or `irr` code changed. The
CHANGELOG's `[Unreleased]` section already carries all four specs' entries, written by their
builds.

onyx's Darkroom v1 (`monorail-000/DEC-075`) consumes a published crate, so the writer
reaches onyx only through this release. This spec ends prep-complete; a cold verify
reviews the prep PR; the user then tags and runs `cargo publish`, as for 0.2.6.

## Release Scope

### Decided for 0.2.7 (design, 2026-11-25)

- **Version / tag:** `0.2.7`, annotated tag `v0.2.7`, a **patch** over the live 0.2.6.
  Dependents on `irradiance = "0.2"` receive it with `cargo update`; the writer stays off
  until they enable `write`.
- **Why it is compatible: the design's mechanical check.** On `153f517`, `just
  semver-check` (crates.io 0.2.6 baseline, DEC-067) reports "202 checks: 202 pass, 58 skip
  … no semver update required", and so does `cargo semver-checks --all-features`. Every
  change is an addition: a new opt-in feature, a new module behind it, and two variants on
  `Error`, which is `#[non_exhaustive]`, so no caller's match can be exhaustive. 0.2.1 to
  0.2.6 shipped additions as patches too. **If the build's semver check reports any
  breaking change, the build stops for the user's decision.** The tool cannot see the
  `write` surface as new API (the baseline has none to compare); that is expected, not a
  gap (SPEC-081's reflection).
- **Only the writer ships. PR 343 (SPEC-091, PROJ-006's linear sRGB plane) must not be in
  this release.** It is open and changes reader-side `src/` and `Error`. The prep is cut
  from `origin/main` and **the build first checks `git diff --stat 153f517 <cut> -- src/
  Cargo.toml` is empty**. If anything has landed, the build stops and the user decides:
  hold the release until it is re-scoped, or re-cut from `153f517`. The user is asked to
  hold PR 343's merge until `v0.2.7` is tagged.
- **docs.rs documents the writer (the one manifest change besides the version).** docs.rs
  builds default features unless told otherwise, so 0.2.7 as it stands would publish the
  writer with no documentation on docs.rs. Add to `Cargo.toml`:

  ```toml
  [package.metadata.docs.rs]
  features = ["write"]
  ```

  `write` only: never `--all-features`, which would document the dev-only `fuzzing`,
  `irr` and `test-fixtures` items. No `doc_cfg` badges (nightly only); the module docs
  already say "behind the `write` feature". The build adds a one-line comment above it in
  the file's own style.
- **Every input whose result changes since `v0.2.6`.** The design's list, from the
  `src/` diff above. The build re-derives it from `git diff v0.2.6 <candidate> -- src/`
  and reconciles; a difference is a finding.
  - **Reader, develop, colour and `irr`: no change.** No line outside the writer's three
    files and `src/lib.rs`'s additions changed. Every existing input develops to the same
    output, `DEVELOP_VERSION` stays `"8"`, every existing refusal keeps its error, and no
    check is reordered. `ErrorKind` gains no variant.
  - **`Error`: two new variants in every build.** `InvalidWriteInput(WriteInputError)`
    (`CallerError`) and `UnsupportedOpcodeForFraming { list, opcode_id, framing }`
    (`Unsupported`). Only the writer returns them, so a reader-only caller never sees
    them. Without `write` their payload types have no public path (DEC-080). **The build
    measures `size_of::<irradiance::Error>()` at `v0.2.6` and on the candidate** and
    records both; a larger `Error` is a note for the release notes, not a stop.
  - **Newly accepted: everything the writer does,** with `write` on. Its refusals are
    `WriteInputError`'s 22 reasons and `UnsupportedOpcodeForFraming`; the build lists them
    from the code and checks the CHANGELOG names each.
- **What's included:** the CHANGELOG's `[Unreleased]` section (SPEC-061, 081, 082, 094) and
  the README's writer section.
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1: reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes. A defect found
in prep is reported in the handback, not fixed here.

1. **The cut point.** Confirm `git diff --stat 153f517 <cut> -- src/ Cargo.toml` is empty
   (Release Scope). Otherwise stop.
2. **The changed-input diff.** Reconcile the design's list with `git diff v0.2.6 <candidate>
   -- src/`: every new `Err(` path, every changed line outside the three writer files.
   Record the result, and both `size_of::<Error>()` figures, in the internal record.
3. **The semver check:** `just semver-check` and `cargo semver-checks --all-features`,
   recording the tool version and both summaries, after the version bump.
4. **Manifest:** `version = "0.2.7"` and the `[package.metadata.docs.rs]` table. Every other
   field is unchanged (`git diff v0.2.6 -- Cargo.toml` before the bump shows only the
   `write` feature).
5. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.7] - <date>` with a
   short lead paragraph like 0.2.6's and a fresh empty `## [Unreleased]`. Check each entry
   against the shipped code; the SPEC-061 entry's reason count ("fourteen reasons, nineteen
   with SPEC-081's") is stale: SPEC-082 and SPEC-094 brought it to 22. Correct anything
   the reconciliation contradicts.
6. **Release notes:** run `just release-notes PROJ-005`, then write `## Release Notes`
   below in 0.2.6's shape (Headline, Added, Behaviour changes, Breaking / action
   required, Known limitations, then the internal record). The writer's own limits go
   under Known limitations: monochrome `u16` only; uncompressed, one strip, little-endian;
   no CFA, tiles or float; the preview is caller-encoded; the EXIF block is placed, never
   parsed; under a cutting framing `OpcodeList1` may hold only `FixBadPixelsConstant`;
   the Lightroom Classic / Capture One open is not recorded. 0.2.6's reader limitations
   carry over unchanged.
7. **README and crate docs:** the README's writer section matches the shipped API (it
   predates SPEC-081's and SPEC-082's options and SPEC-094's refusal); wording fixes only.
   `cargo test --doc --all-features` and `just doc-check`.
8. **Evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` and
   `IRRADIANCE_DNG_VALIDATE` set:
   - `just test`: every held file present, every pin held, and the writer's
     `dng_validate` acceptance tests ran rather than skipped (say which).
   - `just fuzz-dng-write` for 60 s, no crash.
   - **Not re-run, by design:** the SDK colour and stage-2 lanes, the `irr ifd` diff and
     the timing examples. They measure reader code that step 2 confirms is byte-identical
     to `v0.2.6`, whose figures SPEC-090 recorded. If step 2 finds any reader change, run
     them as SPEC-090 did.
9. **The packaged crate, built as a consumer would (new for this release).** `cargo publish
   --dry-run` builds the package with default features only, so it never compiles the
   writer. In a scratch directory outside the repository:
   - `cargo package` on a clean tree, then in the unpacked `target/package/irradiance-0.2.7`:
     `cargo build --features write`, `cargo build --no-default-features`, and
     `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --features write` (what docs.rs runs);
   - a scratch consumer crate depending on that unpacked path with `features = ["write"]`:
     parse one held Monochrom DNG, write it at each framing with `write_to_vec`, re-parse
     each, and compare the samples over the written region with the source. Run
     `dng_validate` on the three outputs. Record the output MD5s;
   - the same consumer without `features = ["write"]` must fail to compile a use of
     `irradiance::dng` (the feature gate holds in the package).
10. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`, `cargo
    publish --dry-run`. The package list must be 0.2.6's files plus exactly
    `src/dng/mod.rs`, `src/dng/write.rs` and `src/write_types.rs`.
11. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), with SPEC-090's filled
    pre-flight as the model, then run every gate: `just test`, `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `lint-fuzz-red-proof`, `fmt-check`, `validate`, `doc-check`, `deny`,
    `deny-fuzz`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check`,
    `decisions-index --check`, `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing, written for whoever uses the crate. `just release-notes PROJ-005` was run
at build: it lists 4 notes from 4 shipped specs (SPEC-061, SPEC-081, SPEC-082, SPEC-094),
all STAGE-017's. As for 0.2.6, these notes are written from the final CHANGELOG instead,
with SPEC ids dropped, and checked line by line against the reconciled changed-input list
in the internal record. The GitHub release text for `v0.2.7` is this section, down to the
internal record.*

**Headline:** irradiance can now write DNG. The opt-in `write` feature turns a monochrome
`u16` sensor plane, with its levels, opcode lists, EXIF block, XMP, descriptive tags and an
optional JPEG preview, into an uncompressed DNG in a buffer you size first. Same input,
same bytes; no allocation in `write_into`; no new dependency. Adobe `dng_validate` 1.7.1
accepts the output with no warnings on the held monochrome files. Nothing changes for a
reader-only caller: no reader, develop or colour code changed, and `DEVELOP_VERSION` stays
`"8"`.

A compatible 0.2.x release: run `cargo update -p irradiance`, then enable the writer with
`irradiance = { version = "0.2.7", features = ["write"] }`.

**Added** (all behind the `write` feature unless noted)
- **`irradiance::dng::write`.** `written_len` (the size query, and a preflight that runs
  every check), `write_into` (into your buffer, no allocation, every byte written,
  deterministic) and `write_to_vec`. The output is one uncompressed little-endian strip in
  IFD0, the plane's samples unchanged.
- **`WriteOptions`:**
  - the framing: `FullSensor` (the whole stored plane, margin included),
    `FullActiveArea`, or `StandardCrop` (the default). Samples are cut only at the
    `ActiveArea` edge; the default crop and the orientation are written as tags, so the
    pixels are never rotated or cropped inside the active area;
  - `ActiveArea`, the default crop, `Orientation`, the black levels and their repeat
    pattern, `WhiteLevel`, `MaskedAreas` (full-sensor framing only), and
    `OpcodeList1`/`2`/`3`, copied verbatim;
  - `UniqueCameraModel`, required: printable ASCII, 1 to 1024 bytes;
  - the source's metadata, all optional: an `ExifBlock` of bytes you built, placed verbatim
    at file offset `EXIF_BLOCK_BASE` (8) with `ExifIFD` and `GPSInfo` pointing into it
    (never parsed or rebased; `exif_block_base` names the base to build for); an XMP
    packet; eight descriptive ASCII tags (`Make`, `Model`, `Software`, `DateTime`,
    `Artist`, `Copyright`, `LocalizedCameraModel`, `CameraSerialNumber`); and four
    rendering hints (`BaselineExposure`, `BaselineNoise`, `BaselineSharpness`,
    `LinearResponseLimit`), written as given. Carrying `BaselineExposure` makes the
    Pentax K-3 III Monochrome's export render like its source in Adobe's reader;
  - `preview`: a `JpegPreview` of JPEG bytes you encoded, with their width, height and
    colour (`YCbCr`, the default, or `Gray`). It is stored as a reduced-resolution SubIFD
    of IFD0 and never decoded. Without a preview the output is byte-identical to a file
    written without the field.
- **Refusals,** each checked by `written_len` too, before anything is written:
  - `Error::PlaneTooLarge` (an existing variant) when `width × height` exceeds your
    `max_pixels`, checked first;
  - `Error::InvalidWriteInput(WriteInputError)`, kind `CallerError`, with 22 reasons
    naming the argument: an empty plane or a sample count that does not match, an invalid
    `ActiveArea`, default crop or masked area, masked areas under a cutting framing, an
    invalid orientation, a black-level count or repeat pattern that does not fit, a
    missing, empty or all-space `UniqueCameraModel`, an ASCII field that is not
    printable ASCII of at most 1024 bytes, a `DateTime` not in TIFF's
    `YYYY:MM:DD HH:MM:SS` form, a rendering hint with a zero denominator (or, for the
    three unsigned hints, a negative part), an EXIF block with neither an Exif nor a GPS
    IFD offset, or an offset that is odd or leaves no room inside the block, an empty
    XMP packet, a preview that does not start `FF D8` or whose dimensions are not 1 to
    65535, an empty opcode list, an `OpcodeList1` whose headers do not add up under a
    cutting framing, an offset or total length that would not fit TIFF's 32 bits, and
    (for `write_into`) a buffer of the wrong length;
  - `Error::UnsupportedOpcodeForFraming`, kind `Unsupported`, when a cutting framing
    meets an `OpcodeList1` opcode other than `FixBadPixelsConstant`, whose stored-plane
    positions the cut would shift. The file can still be written at `FullSensor`.
- **In every build, writer or not:** the two `Error` variants above exist, so `Error`'s
  shape does not depend on features. `Error` is `#[non_exhaustive]`, so no match on it
  changes. Its size is unchanged at 40 bytes. Only the writer returns these variants.
- **docs.rs** now builds the crate with the `write` feature, so it documents
  `irradiance::dng::write`.

**Behaviour changes:** none. No reader, develop, colour or `irr` code changed: every input
reads, inspects and develops exactly as in 0.2.6, every existing refusal keeps its error,
and `ErrorKind` gains no variant.

**Breaking / action required:** none. `cargo semver-checks` against crates.io 0.2.6 reports
no semver update required, with and without all features. The new public items are
reachable only with `write` on; the default build gains only the two `Error` variants on a
`#[non_exhaustive]` enum. There is no new dependency, and the MSRV stays 1.90.

**Known limitations**
- **The writer, v1:**
  - monochrome `u16` planes only: no CFA (Bayer) plane, no float or other sample types;
  - uncompressed, one strip, little-endian: no compression and no tiles;
  - the preview is yours to encode: the writer checks only the JPEG start marker and the
    dimensions;
  - the EXIF block is placed, never parsed: offsets inside it must already be built for
    `EXIF_BLOCK_BASE`;
  - under a cutting framing, `OpcodeList1` may hold only `FixBadPixelsConstant`;
  - opening a written file in Lightroom Classic or Capture One has not been checked; Adobe
    `dng_validate`, `dnglab`, `exiftool` and irradiance's own reader and develop have.
- **The reader** (0.2.6's, unchanged):
  - no colour-accuracy claim yet: the SDK match is a transform check on probe values, not
    a measured colour error, and the ColorChecker ΔE2000 measurement is blocked on
    capturing the chart;
  - tags that change what the samples mean (`LinearizationTable`, a non-zero
    `BlackLevelDeltaH` or `BlackLevelDeltaV`, a `SampleFormat` other than 1, a
    `SubTileBlockSize` other than `[1, 1]`, an interleave factor other than 1) are
    refused, not applied;
  - a staggered CFA (SuperCCD-style `CFALayout` 2 to 9) or a non-RGB `CFAPlaneColor` is
    refused, not demosaiced;
  - an unknown `CalibrationIlluminant` code sends a dual-illuminant file to
    single-illuminant colour rather than refusing it;
  - the Pixel 7 Pro differs from the SDK by up to 2 codes before the `GainMap`, because
    irradiance rounds its fractional black to the nearest integer;
  - the colour develop runs the `GainMap` but not the other `OpcodeList` stages that the
    monochrome develop runs; a file that requires them is refused with a typed error, and
    the develop options are mono-only;
  - a `GainMap` in `OpcodeList1` or `OpcodeList3` is recognised but not applied; a
    mandatory one is refused;
  - a Compression 7 strip is one plane unit, so a single-scan lossless-JPEG file cannot be
    split.

### Internal release record (not part of the GitHub release text)

- **The candidate.** Branch `release/0.2.7-prep`, cut from `origin/main` at `3f9df0b` (the
  PR 344 design merge). Step 1's guard, `git diff --stat 153f517 origin/main -- src
  Cargo.toml`, was empty, so PR 343 (SPEC-091) is not in this release. The build's commits
  touch only `Cargo.toml` (the version line and the docs.rs table), `CHANGELOG.md`,
  `README.md` (wording) and this project's records. `src/` is byte-identical to `153f517`
  and to `3f9df0b`. Where the evidence ran:
  - `just test` with the corpus and `dng_validate`: on `3f9df0b` (`src/`-identical), then
    again on the PR head (handback);
  - semver after the bump, the package checks, the packaged-crate writer check, the fuzz
    run and the gates: on `252360b` (`252360bcea981dc8dfb46179a9a2be91ee0b6933`), the
    version bump.
- **Mechanical semver check:** `cargo-semver-checks 0.51.0`, crates.io 0.2.6 to the
  candidate (DEC-067). Clean both ways: no failure, nothing to accept. Verbatim, after the
  version bump (`252360b`), `just semver-check`:

  ```
  Checking irradiance v0.2.6 -> v0.2.7 (minor change)
   Checked [   0.014s] 202 checks: 202 pass, 58 skip
   Summary no semver update required
  ```

  and `cargo semver-checks --all-features`:

  ```
  Checking irradiance v0.2.6 -> v0.2.7 (minor change)
   Checked [   0.010s] 202 checks: 202 pass, 58 skip
   Summary no semver update required
  ```

  As for 0.2.4 to 0.2.6, the tool labels a 0.x patch bump a "minor change"; no update is
  required. It cannot see the `write` surface as new API, since the baseline has none to
  compare (SPEC-081's reflection); the two `Error` variants are additions to a
  `#[non_exhaustive]` enum, which it passes.
- **`size_of::<irradiance::Error>()`:** 40 bytes (align 8) at `v0.2.6` (`50d533d`), and
  40 bytes (align 8) on the candidate both with and without `write`. Measured by one scratch
  binary built against a detached `v0.2.6` worktree and against the candidate. `Error`
  does not grow, so the release notes say so instead of carrying a size note.
- **Changed-input reconciliation** (step 2). `git diff --stat v0.2.6 3f9df0b -- src/
  Cargo.toml` is 5 files, +1688/−3: `src/dng/mod.rs`, `src/dng/write.rs` and
  `src/write_types.rs` (all new), `src/lib.rs` (+47/−3) and `Cargo.toml`. Against the
  design's list:
  - **Confirmed as designed:**
    - `src/lib.rs`'s changes are exactly the `#[cfg(feature = "write")] pub mod dng`, the
      private always-compiled `mod write_types`, the two `Error` variants, their two
      `Display` arms and their two `kind()` arms (`InvalidWriteInput` in `CallerError`,
      `UnsupportedOpcodeForFraming` in `Unsupported`). The three removed lines are the
      `kind()` arms' line endings, rewritten to add a variant. No reader, develop, colour,
      `irr` or `ErrorKind` line changed, so no existing input changes its result and
      `DEVELOP_VERSION` stays `"8"`;
    - `WriteInputError` has exactly 22 reasons (counted from the enum, 22 variants), each
      with a matching `Err(invalid(WriteInputError::…))` site in `src/dng/write.rs`, plus
      the one `UnsupportedOpcodeForFraming` site. The CHANGELOG names every reason across
      its four entries (fourteen in SPEC-061's, five in SPEC-081's, two in SPEC-082's,
      one in SPEC-094's).
  - **Corrected (the design's refusal list was short by one).** The writer's first check
    returns the existing `Error::PlaneTooLarge { pixels, limit }` when `width × height`
    exceeds `max_pixels` (`plan`, before any size arithmetic). The design listed only
    `WriteInputError`'s reasons and `UnsupportedOpcodeForFraming`. The CHANGELOG's
    SPEC-061 entry and the notes now name it. Not a defect: the module docs and
    `written_len`'s `# Errors` already said so.
  - **Corrected (stale count).** The CHANGELOG's "fourteen reasons, nineteen with
    SPEC-081's" now says 22, with each spec's share.
  - **A Cargo.toml line the design did not name.** `git diff v0.2.6 3f9df0b -- Cargo.toml`
    is the `write = []` feature and its comment, as designed, plus `"write"` added to the
    self dev-dependency's features (`irradiance = { path = ".", features = [...] }`), so
    the tests compile the writer. `cargo package` strips that path-only dev-dependency:
    the packaged `Cargo.toml`'s only dev-dependencies are `ssimulacra2` and `toml`. No
    effect on the release.
  - **No reader change, so the reader lanes stay off.** The SDK colour and stage-2 lanes,
    the `irr ifd` diff and the timing examples were not run, as designed: they measure
    code that is byte-identical to `v0.2.6`, whose figures SPEC-090 recorded.
- **The packaged crate, as a consumer builds it** (step 9, new for this release). In the
  build's scratchpad, from a clean detached worktree of `252360b`:
  - `cargo package`: `Packaged 22 files, 1.9MiB (416.6KiB compressed)`, verified; the
    `.crate` unpacked into a separate directory. Its `.cargo_vcs_info.json` carries
    `git.sha1` `252360bcea981dc8dfb46179a9a2be91ee0b6933`; its manifest shows `version =
    "0.2.7"`, `rust-version = "1.90"`, `write = []` and the
    `[package.metadata.docs.rs] features = ["write"]` table.
  - In the unpacked `irradiance-0.2.7`: `cargo build --features write` exit 0, `cargo
    build --no-default-features` exit 0, and `RUSTDOCFLAGS='-D warnings' cargo doc
    --no-deps --features write` exit 0 with no warning, generating
    `irradiance/dng/write/` (`write_into`, `write_to_vec`, `written_len`,
    `exif_block_base`, `EXIF_BLOCK_BASE`, `Framing`, `JpegPreviewColor`,
    `WriteInputError` and the structs).
  - A scratch consumer crate on `irradiance = { path = <unpacked>, features = ["write"] }`,
    release build: parse the Leica Q2 Monochrom `LEICA-Q2-MONO/L1021223.DNG` (8424×5632,
    14-bit, `ActiveArea` 0,0 to 5632,8392), unpack it, and `write_to_vec` it at each
    framing with options built from its `Sensor` as the corpus tests build them
    (`UniqueCameraModel` "irradiance 0.2.7 package check"). Each output was re-parsed and
    unpacked, and its samples compared with the source over the written region: equal
    every time.

    | Framing | Bytes | Written plane | Samples equal | MD5 |
    |---|---|---|---|---|
    | `FullSensor` | 94,888,406 | 8424×5632 | 47,443,968 | `26a380eb4b8cfbedcab521ba152c2349` |
    | `FullActiveArea` | 94,527,930 | 8392×5632 | 47,263,744 | `3e64fa3b185517c81e660e7d5ddd16c5` |
    | `StandardCrop` | 94,527,930 | 8392×5632 | 47,263,744 | `fc60668e99ddee1440efe993071e641e` |

    A second run of the same binary wrote the same three MD5s. Adobe `dng_validate` 1.7.1
    (the local build at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/`,
    DEC-069 §6) on each output: exit 0, "Validation complete", no `Warning` or `Error`
    line. The outputs stayed in the scratchpad.
  - The same consumer with `features = ["write"]` removed fails to compile:
    `error[E0433]: cannot find `dng` in `irradiance``. The feature gate holds in the
    package.
- **Package list.** `cargo package --list` on clean detached worktrees of `v0.2.6` and
  `252360b`, diffed: 0.2.6's 19 paths plus exactly `src/dng/mod.rs`, `src/dng/write.rs`
  and `src/write_types.rs`, 22 in all. There is no `scripts/`, `tests/`, SDK or corpus
  file. The dry run prints 54 "ignoring example/test … not included" warnings where 0.2.6
  had 51; diffed, the three extra are `tests/dng_write.rs`, `tests/dng_write_alloc.rs` and
  `tests/dng_write_corpus.rs`.
- **Fuzz.** `just fuzz-dng-write 60` on `252360b`: `Done 3597321 runs in 61 second(s)`, no
  crash, cov 2150, corpus 549 entries; the working tree stayed clean.
- **Release notes against the reconciled list.** Each class above has a line in the notes:
  the writer's functions and options, every refusal (the 22 reasons, the framing
  refusal and `PlaneTooLarge`), the two variants in every build with `Error`'s size
  unchanged, docs.rs, no behaviour change for a reader-only caller, and the writer's v1
  limits. The reader's known limitations are 0.2.6's, carried over unchanged.

## Release cut is two-phase

A release session **cannot finish in one pass** — the irreversible tag/publish is
gated by a human/coordinator. Structure it so the reversible work lands first:
- **Phase 1 — reversible prep (CI-gated PR):** CHANGELOG entry, version bump
  (`just next-version`), stage-backlog tick. Fully revertable — land it as a
  normal PR.
- **Phase 2 — irreversible cut (human/coordinator-gated):** create the tag and
  publish to each channel, only after Phase 1 merges. The spec ships
  **"prep-complete, cut-deferred."** Before tagging, confirm the checked-out tree
  is the merged Phase-1 commit: the version in the manifest is the new one. A tag
  cut on the commit before the merge points at the old version (irradiance 0.2.4's
  first attempt).
  Make the tag annotated (`git tag -a`), then confirm `git cat-file -t <tag>` prints
  `tag` and, after the push, that `git ls-remote --tags origin '<tag>*'` shows a `^{}`
  line. A lightweight tag (irradiance 0.2.5's) still publishes, but `git describe`
  skips it.

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled
2026-11-25 by the Phase-1 build (HANDOFF-197) on candidate `252360b`
(`252360bcea981dc8dfb46179a9a2be91ee0b6933`, the version bump). The commits after it
change only project records, and `src/` is byte-identical throughout. `just test` was
repeated on the PR head; the handback has that result.
- `[now]`: verified on the candidate in the build session.
- `[cut]`: verified by the user at Phase 2.
- `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance** — `[now]` done, the rest `[cut]`.
  - `[now]` ✔:
    - `Cargo.toml` says `version = "0.2.7"`. `cargo publish --dry-run` ran in a detached
      scratch worktree of `252360b`, on a clean tree with no `--allow-dirty`. It reports
      `Packaged 22 files, 1.9MiB (416.6KiB compressed)` and `Uploading irradiance
      v0.2.7`, then "aborting upload due to dry run".
    - `git tag -l 'v0.2.7'` is empty, locally and on origin.
    - The packaged `.cargo_vcs_info.json` carries `git.sha1`
      `252360bcea981dc8dfb46179a9a2be91ee0b6933`, the candidate.
    - `git diff v0.2.6 252360b -- Cargo.toml` is the `write` feature (and `"write"` on the
      self dev-dependency, stripped from the package), the version line and the docs.rs
      table. So `license`, `description`, `keywords`, `categories`, `repository`,
      `include` and `rust-version = "1.90"` are unchanged; the packaged manifest shows
      `rust-version = "1.90"`.
    - `v0.2.6` on origin is annotated (`git cat-file -t v0.2.6` prints `tag`).
  - `[cut]`:
    - `git tag -a v0.2.7 -m "irradiance 0.2.7"` on the merged prep commit, after
      confirming its `Cargo.toml` shows 0.2.7 and that PR 343 has not merged.
    - `git tag --points-at HEAD` prints exactly `v0.2.7`, and `git cat-file -t v0.2.7`
      prints `tag`. After the push, `git ls-remote --tags origin 'v0.2.7*'` shows both
      `refs/tags/v0.2.7` and `refs/tags/v0.2.7^{}`.
    - The published `.crate`'s `.cargo_vcs_info.json` sha1 equals the tagged commit:
      `curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.7.crate | tar -xzO irradiance-0.2.7/.cargo_vcs_info.json`
      against `git rev-parse v0.2.7^{commit}`.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[now]` proxy done, the rest `[cut]`.
  - `[now]` ✔:
    - `cargo publish --dry-run` packaged and verified the crate in isolation (default
      features). Its only warnings are the expected "ignoring example/test … not
      included in the published package" lines (54: 0.2.6's 51 plus the three writer
      test files).
    - **The packaged crate with `write` on** (new for this release, because the dry run
      builds default features only): in the unpacked package, `cargo build --features
      write`, `cargo build --no-default-features` and `RUSTDOCFLAGS='-D warnings' cargo
      doc --no-deps --features write` (what docs.rs runs, given the new metadata table)
      all exit 0 (internal record).
    - The packaged `Cargo.toml` has no self dev-dependency; its only dev-dependencies
      are `ssimulacra2` and `toml`. `[dependencies]` is empty.
    - `just package-targets-check` passed: no executable in the default build or in the
      package (`irradiance-0.2.7`), `irr` builds under `--features irr`, and its red case
      is caught. `just deny` and `just deny-fuzz` both report `licenses ok`.
    - The package list is 0.2.6's 19 files plus exactly `src/dng/mod.rs`,
      `src/dng/write.rs` and `src/write_types.rs`. No `scripts/`, `tests/`, SDK or corpus
      bytes.
  - `[cut]`: on a clean host, `cargo add irradiance@=0.2.7 --features write && cargo
    build`. crates.io shows 0.2.7, and docs.rs is green and shows `irradiance::dng::write`.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` done on the candidate, `[cut]` on the
  published crate.
  - `[now]` ✔:
    - `just test` (on `3f9df0b`, `src/`-identical) ran with `IRRADIANCE_CORPUS_DIR` and
      `IRRADIANCE_DNG_VALIDATE` set. It printed "corpus: 12/12 present", exiftool and
      dnglab present, and "dng-validate: present … the DNG writer's dng_validate
      acceptance will run". The 55 test results, doctests included, total 923 passed, 0
      failed and 1 ignored, exit 0. The three writer `dng_validate` tests
      (`dng_validate_accepts_and_renders_like_the_source`,
      `dng_validate_accepts_metadata_and_the_k3_renders_like_its_source`,
      `dng_validate_accepts_a_written_preview`) ran and passed. The SDK colour and
      stage-2 lanes printed SKIP, by design (reader unchanged).
    - `cargo test --doc --all-features`: 5 passed. `just doc-check`: 2 feature surfaces,
      0 rustdoc warnings.
    - The packaged-crate consumer wrote the Q2 Monochrom at all three framings from the
      unpacked package, re-read each sample-for-sample against the source, and Adobe
      `dng_validate` 1.7.1 accepted all three with no warning (internal record, with the
      MD5s). Without `write` the same consumer does not compile.
  - `[cut]`: in a clean-host probe crate on `irradiance = { version = "=0.2.7", features =
    ["write"] }`:
    - parse a held monochrome DNG, `write_to_vec` it at `StandardCrop`, and re-parse it:
      the samples equal the source's active area, and the output's MD5 equals the same
      program built against the unpacked package
      (`fc60668e99ddee1440efe993071e641e` for the Q2 Monochrom `L1021223.DNG` with
      `UniqueCameraModel` "irradiance 0.2.7 package check");
    - `dng_validate` accepts the output with no warning;
    - `develop_into` on the Pentax K-3 III Monochrome returns `Ok`, with an output hash
      equal to the same program built against 0.2.6, and
      `irradiance::DEVELOP_VERSION == "8"`.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.7`, and dependents on `"0.2"` fall back to
    0.2.6 (which has no `write` feature, so a dependent that enabled it stops resolving
    until it pins 0.2.7 or drops the feature). `cargo yank --undo --version 0.2.7`
    reverses it.
  - **Git rollback:** revert the Phase-1 PR.

**Gates on the candidate, all green** (2026-11-25, `IRRADIANCE_CORPUS_DIR` and
`IRRADIANCE_DNG_VALIDATE` set):
- `just test` (above)
- `lint-ci`
- `lint-no-allow`
- `lint-fuzz`
- `lint-fuzz-red-proof`
- `fmt-check`
- `validate`
- `doc-check`
- `deny`, plus `deny-fuzz`
- `msrv` (1.90.0)
- `cost-audit`
- `gate-audit`
- `name-length-check`
- `decisions-index --check`
- `semver-check` (202 of 202)
- `package-targets-check`
- `cargo publish --dry-run`

## Notes for the Implementer

- The corpus and `dng_validate` are local only (DEC-069 §6): never commit or package them,
  or anything they write. Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  and `IRRADIANCE_DNG_VALIDATE` to the local build under `tools/dng_sdk`; `just test`
  already passes `--all-features`.
- Do the step-9 scratch crates in the session's scratchpad, not in the repository.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).
- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, clean tree; confirm `Cargo.toml` shows
     `version = "0.2.7"` and that PR 343 has not merged (or the release was re-scoped).
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.7 -m "irradiance 0.2.7"`, then `git push origin v0.2.7`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5 (the orchestrator can run the
     clean-host crate), reported back for the Cut Record. PR 343 can merge after this.
  6. Optional: a GitHub release on `v0.2.7` with the `## Release Notes` text.

## Cut Record

*One line, appended after Phase 2 by whoever runs the cut.* The user-facing
outcome now live, plus the upgrade/clean-install check that confirms the
publish landed — the one line a review or work-log would quote. E.g.
*"v0.4.0→v0.5.0 `brew upgrade` clean; MCP-install now first-class; prod DB
opened, 189 entries intact."* If the release is pure infra with no user-facing
change, say so — a real, greppable outcome, not a blank.

---

## Reflection

*Appended during **ship**. Four questions, short answers.*

1. **What would I do differently next release?**
   — <answer>

2. **Does any template, constraint, or decision need updating?** (e.g. a new
   pre-flight category to add to this template)
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(A release-phase `escaped` defect means a pre-flight category above was
   missing or skipped — grow the checklist.)*
