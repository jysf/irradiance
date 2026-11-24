---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-061
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-005
  stage: STAGE-017
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-175 (tier_map.build)
  created_at: 2026-11-21

references:
  decisions: [DEC-008, DEC-016, DEC-050, DEC-051, DEC-052, DEC-055, DEC-058, DEC-059, DEC-069, DEC-071, DEC-072, DEC-073, DEC-074, DEC-075]
  constraints: [no-panics-on-untrusted-input, library-not-application, provenance-recorded-per-algorithm, oracle-must-be-shown-red, no-copyleft-dependencies]
  related_specs: [SPEC-081, SPEC-082]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-057]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-017's <capability>". Optional; null is acceptable.
value_link: "The writer core onyx Darkroom v1 wraps for its Linear DNG export (STAGE-017)."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 35000000       # upper M; SPEC-057 (the test writer) and SPEC-059 ran 2x their estimates
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-21
      notes: "main-loop, not separately metered (AGENTS.md section 4); includes the dng_validate probe"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-061: Public `write` feature: the mono u16 DNG writer core (size query, caller buffer, framing as tags, typed errors, fuzz target, real-file acceptance)

## Context

STAGE-017's core spec (PROJ-005). Framed 2026-11-11 as a general public writer;
re-scoped 2026-11-20 from onyx's requirements, `monorail-000/DEC-075`, as amended by
`monorail-000/DEC-076` and `monorail-000/DEC-077` (external inputs, referenced here,
never in `depends_on`). DEC-071 fixes v1's scope and API shape, DEC-073 the framing rule
(tags over sensor-order samples, cut only at the `ActiveArea` edge), and DEC-075 (new at
this design) the layout and validation rules. STAGE-013's test-only writer
(`tests/support/dng_writer.rs`, SPEC-057) is the clean-room starting point: it stays for
fixtures; this spec writes a narrower, panic-free writer in `src/`.

## Goal

Behind the `write` feature, write one monochrome u16 sensor plane, unchanged, with the
caller's levels, opcode-list bytes, framing, orientation, masked areas and the required
`UniqueCameraModel` (DEC-071 §3a; a missing or blank one is an `ErrorKind::CallerError`),
as an uncompressed little-endian DNG into a caller buffer sized by a prior query, with
typed errors, a `max_pixels` budget and identical bytes for identical input. Prove it by
round trip on the held mono files at all three framings, by irradiance's own develop
giving the source's result, and by dnglab, exiftool and Adobe `dng_validate` acceptance,
with a fuzz target over the options and geometry.

## Design

### 1. Where it lives

- `Cargo.toml`: `write = []`, an empty feature with no dependency. Not default.
- `src/lib.rs`: `#[cfg(feature = "write")] pub mod dng;`
- `src/dng/mod.rs`: `pub mod write;` (DEC-055's `src/dng/write.rs`). It uses only
  `crate::ifd` container types (`ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize`, the
  `TAG_*` constants) and `crate::Error`. It never imports develop, colour, warp or plane
  code (DEC-055). The single exception is the header walk of §5, which is local code,
  not `crate::opcode`.
- The crate root's panic-free lint policy applies (`just lint-no-allow`): no `unwrap`,
  `expect`, indexing, `panic!` or unchecked arithmetic.

### 2. The public API

Names are binding unless the builder finds a clippy or semver reason, recorded in the
handback.

```rust
// irradiance::dng::write
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Framing { FullSensor, FullActiveArea, #[default] StandardCrop }

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WriteOptions<'a> {
    pub framing: Framing,
    /// Stored-plane coordinates. None = the whole plane is active.
    pub active_area: Option<ActiveArea>,
    /// Active-area coordinates (DNG). None = the whole active area.
    pub default_crop: Option<(DefaultCropOrigin, DefaultCropSize)>,
    /// TIFF Orientation, 1..=8. Default 1. Always written (DEC-075 §4).
    pub orientation: u16,
    /// BlackLevel, one LONG per repeat position, row-major. Empty = tag omitted.
    pub black_level: &'a [u32],
    /// BlackLevelRepeatDim [rows, cols]. None = tag omitted (DNG default 1x1).
    pub black_level_repeat_dim: Option<[u16; 2]>,
    /// WhiteLevel. Default 65535 (the DNG default for 16 bits). Always written.
    pub white_level: u32,
    /// MaskedAreas, stored-plane coordinates; FullSensor only (DEC-075 §6).
    pub masked_areas: &'a [ActiveArea],
    pub opcode_list_1: Option<&'a [u8]>,
    pub opcode_list_2: Option<&'a [u8]>,
    pub opcode_list_3: Option<&'a [u8]>,
    /// Required (DEC-071 §3a). Printable ASCII, 1..=1024 bytes, not all spaces.
    pub unique_camera_model: Option<&'a str>,
}
impl Default for WriteOptions<'_> { /* orientation 1, white_level 65535, rest empty/None */ }

pub fn written_len(samples: &[u16], width: u32, height: u32,
                   options: &WriteOptions<'_>, max_pixels: usize) -> Result<usize, Error>;
pub fn write_into(samples: &[u16], width: u32, height: u32,
                  options: &WriteOptions<'_>, max_pixels: usize, dst: &mut [u8]) -> Result<(), Error>;
pub fn write_to_vec(samples: &[u16], width: u32, height: u32,
                    options: &WriteOptions<'_>, max_pixels: usize) -> Result<Vec<u8>, Error>;
```

- `samples` is the **whole stored plane**, row-major, `width × height`, exactly what
  `plane::unpack_into` returns. The writer cuts it; the caller never does.
- `written_len` runs every check `write_into` runs except the `dst` length, in the same
  order, and returns the same error for the same input. Precedence: `max_pixels`
  (`Error::PlaneTooLarge`, before any multiplication can overflow), sample count,
  geometry (active area, crop, masked areas), levels, orientation, strings, opcode
  lists, output size; then, in `write_into` only, `dst.len()`.
- `write_into` requires `dst.len() == written_len(..)`, writes every byte of `dst`, and
  allocates nothing. `write_to_vec` allocates one `Vec` of exactly that length.
- SPEC-081 adds fields (EXIF block, XMP, the other eight descriptive tags, DEC-074's
  rendering hints) and SPEC-082 a preview field; `#[non_exhaustive]` keeps that additive.

### 3. Framing (DEC-073, accepted; agreed in `monorail-000/DEC-076` item 2)

Let `aa` be `active_area`, or the whole plane when `None`.

| Framing | Samples written | `ActiveArea` tag | `DefaultCropOrigin`/`Size` | `MaskedAreas` |
|---|---|---|---|---|
| FullSensor | the whole plane | written iff `active_area` is `Some` | (0,0) and `aa`'s size | written iff the list is non-empty |
| FullActiveArea | `aa` only | omitted | (0,0) and `aa`'s size | list must be empty |
| StandardCrop | `aa` only | omitted | `default_crop`, or (0,0) and `aa`'s size | list must be empty |

- `DefaultCrop` is active-area-relative in DNG, so the caller's values carry over
  unchanged across the cut, and no crop reaches the margin.
- `Orientation` is written as given. Samples are never rotated.
- Opcode lists follow DEC-075 §5: verbatim under FullSensor; under the cutting framings
  lists 2 and 3 are verbatim and list 1 must pass the allow-list walk.

### 4. Tags (all in IFD0, ascending, DEC-075 §1)

`NewSubFileType` (254) LONG 0 · `ImageWidth` (256) LONG · `ImageLength` (257) LONG ·
`BitsPerSample` (258) SHORT 16 · `Compression` (259) SHORT 1 · `PhotometricInterpretation`
(262) SHORT 34892 · `StripOffsets` (273) LONG · `Orientation` (274) SHORT ·
`SamplesPerPixel` (277) SHORT 1 · `RowsPerStrip` (278) LONG = height · `StripByteCounts`
(279) LONG · `PlanarConfiguration` (284) SHORT 1 · `DNGVersion` (50706) BYTE 1,4,0,0 ·
`DNGBackwardVersion` (50707) BYTE 1,3,0,0 with any opcode list, else 1,1,0,0 ·
`UniqueCameraModel` (50708) ASCII · `BlackLevelRepeatDim` (50713) SHORT[2] · `BlackLevel`
(50714) LONG[n] · `WhiteLevel` (50717) LONG · `DefaultCropOrigin` (50719) LONG[2] ·
`DefaultCropSize` (50720) LONG[2] · `ActiveArea` (50829) LONG[4] · `MaskedAreas` (50830)
LONG[4n] · `OpcodeList1` (51008) / `2` (51009) / `3` (51022) UNDEFINED.

Samples are written as little-endian u16, one strip, `RowsPerStrip` = height. A 14-bit
source becomes a 16-bit file with its own `WhiteLevel` (probe: renders identically).

### 5. Validation (`Error::InvalidWriteInput(WriteInputError)`, `CallerError`)

`WriteInputError` is `#[non_exhaustive]`, `Debug + Clone + PartialEq + Eq`. Reasons:

- `EmptyPlane` (width or height 0); `SampleCountMismatch { expected: u64, actual: usize }`;
- `ActiveAreaInvalid` (empty, inverted, or outside the plane);
- `DefaultCropInvalid` (empty, or origin + size outside the active area);
- `MaskedAreaInvalid { index }` (empty, outside the plane, or intersecting the active
  area); `MaskedAreasNeedFullSensor`;
- `OrientationInvalid { value }` (not 1..=8);
- `BlackLevelCount { expected, actual }` (repeat rows × cols, or 1 without a repeat dim;
  `actual` 0 is allowed only without a repeat dim); `BlackLevelRepeatDimInvalid` (a zero);
- `MissingUniqueCameraModel` (None, empty, or only spaces);
  `InvalidAscii { field: &'static str }` (a byte outside 0x20..=0x7E, or over 1024 bytes);
- `MalformedOpcodeList { list: u8 }` (list-1 headers cannot be walked; only list 1 is
  walked, and only under a cutting framing);
- `OutputTooLarge` (any offset or the total length past `u32::MAX`);
- `OutputBufferWrongLength { expected, actual }` (`write_into` only).

Plus, outside the enum: `Error::PlaneTooLarge` for `width × height > max_pixels`, and
`Error::UnsupportedOpcodeForFraming { list: u8, opcode_id: u32, framing: Framing }`
(`ErrorKind::Unsupported`) for a list-1 opcode outside the allow-list under a cutting
framing. Both new `Error` variants get rustdoc, a `Display` arm and a `kind()` arm, and
`tests/error_kind.rs` gains a case each.

**List-1 header walk** (DNG 1.7.1 ch. 6, big-endian regardless of file order): a
4-byte count, then per opcode id (4), version (4), flags (4), parameter byte count (4)
and that many bytes. The walk checks the lengths add up exactly to the list and reads
nothing else. Allow-list: `{4}` (`FixBadPixelsConstant`).

### 6. Layout (DEC-075 §1)

`II 2A 00` + IFD0 offset; IFD0 at offset 8 (SPEC-081 moves it after the EXIF block);
entry count, entries, next-IFD 0; then out-of-line values in tag order, each at an even
offset; then the strip at an even offset. Pad bytes are 0. Inline values follow TIFF's
start-of-field rule (the test writer's `short()`/`inline_bytes()` comments explain the
trap; in little-endian the low half holds the SHORT). Reuse the layout approach of
`tests/support/dng_writer.rs` (own work, same provenance) but not its code paths for
CFA, tiles, big-endian or `extra_tags`.

### 7. Oracles and red-proofs

Tier A (synthetic, own work, always runs; `tests/dng_write.rs`):

1. **Round trip.** `Container::parse` → `sensor()` → `plane::unpack_into` of the output
   gives the expected samples (the whole plane, or the `aa` cut), and the `Sensor`
   fields equal what §3/§4 say: dims, bits 16, levels, repeat dim, `ActiveArea`,
   `MaskedAreas`, crop, orientation, the three opcode byte strings.
2. **Develop equivalence (synthetic).** For a plane with a non-zero active-area origin and
   a margin: `develop_into` of the StandardCrop output equals `develop_into` of a
   test-writer fixture of the source; FullSensor output equals the source under
   `Framing::FullActiveArea` develop.
3. **Red-proof of oracle 1** (negative control first): the honest output passes; then the
   test flips one sample byte, and separately rewrites `DefaultCropOrigin`'s value in a
   copy, and the round-trip comparison reports a mismatch for each.

Tier B (held corpus; `tests/dng_write_corpus.rs`; skips loudly per file when absent):

4. On `LEICA-Q2-MONO/L1021223.DNG` (8424×5632, `ActiveArea` right 8392: a 32-column
   margin; `OpcodeList1` `FixBadPixelsConstant`, `OpcodeList3` `WarpRectilinear`),
   `PENTAX-K3III-MONO/K3III.DNG` (`ActiveArea` 34,26,4194,6250), `LEICA-M-MONOCHROM/
   L1000622.DNG` (no `ActiveArea`, no opcodes) and `LEICA-M-MONOCHROM-TYP246/
   M2462362.DNG` (12-bit Compression 7 source): write each framing from the source's
   own sensor fields and unpacked plane; oracle 1 holds, and
   `develop_into(written StandardCrop) == develop_into(source)` exactly.
   `develop_into(written FullSensor)` and `develop_into(written FullActiveArea)` both
   equal the source's `Framing::FullActiveArea` develop (probe: all four held files,
   the Q2M's warp included, develop at FullActiveArea; size `dst` with
   `output_dimensions_with`). A develop that is refused on one side must be refused
   with the same variant on the other.
5. **DEC-073 red-proof (Q2M).** A test-writer file with the samples cut to the
   standard crop and `OpcodeList3` verbatim (the shape DEC-073 forbids) develops to a
   result that differs from `develop_into(source)`. Negative control: the public
   writer's StandardCrop output matches (oracle 4).
6. **dnglab** (tool-gated, skips loudly): `--raw-checksum` of a FullSensor output equals
   the source's own checksum (probe: equal on the Q2M and K-3 III); of a cut output, the
   MD5 of the cut plane as LE u16.
7. **exiftool** (tool-gated): `DNGVersion`, `UniqueCameraModel`, `BitsPerSample`,
   `ImageWidth`, `ActiveArea`, `DefaultCropSize`, `Orientation` read back as written.
8. **Adobe `dng_validate`** (env-gated, local only, DEC-069 §6): when
   `$IRRADIANCE_DNG_VALIDATE` names the binary, every output above validates with no
   `Warning` or `Error` line, and for the Q2M the `-16 -tif` render of the StandardCrop
   and the FullSensor outputs equals the source's render sample for sample, while the
   item-5 file's render differs. Unset: a SKIP line (and `examples/corpus-status.rs`
   prints one). **Set but unusable** (missing binary, non-zero exit, unreadable TIFF):
   the test fails (set means must run). Extend `tests/support/stage2_tiff.rs` to read the
   3-sample `-tif` output, or add a sibling reader with the same refuse-don't-guess rule.

Always (both tiers):

9. **Determinism.** Two writes of the same input into buffers pre-filled with `0x00` and
   `0xFF` are byte-identical.
10. **No allocation** (`tests/dng_write_alloc.rs`, the counting-allocator pattern of
   `tests/previews_alloc.rs`): `written_len` and `write_into` make zero allocations on a
   256×256 plane with every option set. Added to `just test`'s debug `*_alloc` line.

### 8. Fuzz target (§12 bar 2)

`fuzz/fuzz_targets/dng_write.rs` (`fuzz/Cargo.toml` gains the `write` feature and a
`[[bin]]`): derive a plane of at most 64×64 and every `WriteOptions` field (framing,
active area, crop, orientation, levels, repeat dim, masked areas, opcode-list byte
slices, a UCM byte string) plus `max_pixels` from the input. Properties: nothing
panics; `written_len` and `write_into` agree (same `Ok` length or the same `Err`); on
`Ok`, the output parses, `sensor()` succeeds, and `unpack_into` returns the expected
samples. Seeds: `examples/fuzz-seeds.rs` writes `fuzz/seeds/dng_write/` (one per
framing, one with each opcode list, one per error reason that is cheap to reach).
Recipe `just fuzz-dng-write` (same `+toolchain` trap and corpus/seed split as the
others), listed in `app.just` and AGENTS.md §6's block.

### 9. Provenance

`docs/provenance-ledger.md` row for `src/dng/write.rs`: TIFF 6.0 §2/§8, Adobe DNG
Specification 1.7.1 (tag set and placement, opcode-list header layout, `DNGBackwardVersion`
1.3 for opcode lists), DEC-008 in the writing direction; adapted from this repo's own
`tests/support/dng_writer.rs` (SPEC-057). Class 1. No writer implementation read.
`dnglab`, `exiftool` and Adobe `dng_validate` 1.7.1 run as black-box binaries only;
the SDK's source is not read (reading it would be class 4).

## Implementation Context — probed 2026-11-21

Probed with the test-only writer (raw plane in IFD0, one strip, LE, 16-bit) and Adobe
`dng_validate` 1.7.1 at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/`,
dnglab 0.7.2 and exiftool 13.55.

| Probe | Result |
|---|---|
| Synthetic 64×48, `UniqueCameraModel` present, no `Orientation` | valid; warns `Missing Orientation tag (IFD 0)` |
| Same, `UniqueCameraModel` absent / empty | valid; warns `Missing or invalid UniqueCameraModel` (both) |
| Q2M, FullSensor (`ActiveArea` 0,0,5632,8392, source crop), `OpcodeList1` + `3` verbatim, backward 1.3 | no warnings; stage 3 equals the source's (0 of 47,263,744 differ); `-tif` render equals the source's |
| Q2M, cut at `ActiveArea` (8392 wide), crop as tag | no warnings; render equals the source's (0 of 140,180,736 differ) |
| Q2M, cut to the crop, warp verbatim (DEC-073's forbidden shape) | no warnings, but 132,753,649 of 140,180,736 render samples differ, by up to 64,441 |
| K-3 III, FullSensor and cut at `ActiveArea` | valid (warn only for missing `Orientation`); FullSensor stage 3 equals the source's |
| K-3 III, StandardCrop without / with `BaselineExposure` −33177/65536 | render differs in 76,681,557 samples / identical → DEC-074, lands in SPEC-081 |
| dnglab `--raw-checksum`, Q2M and K-3 III FullSensor | equals the source's (`cb653b5b…` and `98bae133…`) |
| irradiance `develop_with_into` at `Framing::FullActiveArea` on the four held mono files | all four succeed (DEC-059 admits the Q2M's warp) |
| `DNGBackwardVersion` 1.1 vs 1.3 with opcode lists | `dng_validate` silent either way; DEC-075 §3 writes 1.3 (the spec's version for opcode lists) |

Source facts (held files): the Q2M and M Monochrom store one strip; the Q2M puts the raw
plane in SubIFD 1 under a preview IFD0 and has `DefaultUserCrop`, `AntiAliasStrength`,
`BestQualityScale` (left out, DEC-074 Context). `Sensor.default_crop_origin/size` are
`u32` in the reader. The probe test was a throwaway (`tests/zz_probe_061.rs`), not
committed.

## Inputs

- **Files to read:** `tests/support/dng_writer.rs` (layout, inline rule, determinism),
  `tests/dng_writer.rs` (oracle helpers to reuse), `src/ifd.rs` (`Sensor`,
  `ActiveArea`, `DefaultCrop*`, `TAG_*`, `Container::sensor` placement rules),
  `src/lib.rs` (`Error`, `ErrorKind::kind`, `Display`), `src/plane.rs` (`validate`'s
  budget precedence), `src/develop.rs` (`develop_into`, `develop_with_into`,
  `Framing::FullActiveArea`), `tests/previews_alloc.rs`, `tests/error_kind.rs`,
  `tests/support/{corpus,tools,md5,stage2_tiff}.rs`, `examples/fuzz-seeds.rs`,
  `examples/corpus-status.rs`, `fuzz/Cargo.toml`, `app.just`.
- **Decisions:** DEC-008, DEC-016, DEC-050, DEC-051, DEC-052, DEC-055, DEC-058, DEC-059,
  DEC-069 §6, DEC-071, DEC-072 (layout slot only), DEC-073, DEC-074 (not built here),
  DEC-075.
- **External tools:** dnglab, exiftool (tool-gated); `dng_validate` (env-gated, never CI).

## Outputs

- **Files created:** `src/dng/mod.rs`, `src/dng/write.rs`, `tests/dng_write.rs`,
  `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `fuzz/fuzz_targets/dng_write.rs`, `fuzz/seeds/dng_write/*`.
- **Files modified:** `Cargo.toml` (`write` feature), `src/lib.rs` (`mod dng`, two
  `Error` variants with `Display` and `kind()` arms), `fuzz/Cargo.toml`,
  `examples/fuzz-seeds.rs`, `examples/corpus-status.rs` (the `dng_validate` SKIP line),
  `tests/error_kind.rs`, `tests/support/stage2_tiff.rs` (3-sample read, if extended),
  `app.just` (`fuzz-dng-write`, `dng_write_alloc` in `test`), AGENTS.md §6 block,
  `docs/provenance-ledger.md`, `CHANGELOG.md` `[Unreleased]` Added, README feature list.
- **New public API:** `irradiance::dng::write::{Framing, WriteOptions, WriteInputError,
  written_len, write_into, write_to_vec}`; `Error::InvalidWriteInput`,
  `Error::UnsupportedOpcodeForFraming`. All behind `write` except the two `Error`
  variants, which exist in every build (so `Error`'s shape does not depend on features).
- **New flags:** cargo feature `write`, default off. Env var `IRRADIANCE_DNG_VALIDATE`
  (path to the binary), unset by default and in CI.

## Acceptance Criteria

- [ ] **AC1 Feature gate.** `cargo check --no-default-features` and the default build
      compile no `dng` module; `--features write` and `--all-features` do. The crate keeps
      zero runtime dependencies.
- [ ] **AC2 Round trip, tier A.** Oracle 1 holds for each framing × {no active area,
      active area with non-zero origin and margin} × {no opcodes, all three lists}, with
      masked areas under FullSensor.
- [ ] **AC3 Tags.** The tag set, order, types and values of §4 hold; `DNGBackwardVersion`
      follows the opcode rule; `Orientation` is written when left at its default.
- [ ] **AC4 Validation.** Each `WriteInputError` reason, `PlaneTooLarge` and
      `UnsupportedOpcodeForFraming` is returned for a minimal input that triggers it,
      by both `written_len` and `write_into`, with the documented `kind()`. No case panics.
- [ ] **AC5 Size query.** For every `Ok` case, `write_into` succeeds exactly when
      `dst.len() == written_len(..)`, and `write_to_vec(..).len()` equals it.
- [ ] **AC6 Develop equivalence, tier A and B.** Oracles 2 and 4.
- [ ] **AC7 Red-proofs.** Oracles 3 and 5, each after its negative control.
- [ ] **AC8 dnglab and exiftool.** Oracles 6 and 7, tool-gated.
- [ ] **AC9 dng_validate.** Oracle 8 when `$IRRADIANCE_DNG_VALIDATE` is set; a SKIP line
      when unset; a failure when set but unusable. Run locally at build and at verify;
      the handback records the result.
- [ ] **AC10 Determinism and allocation.** Oracles 9 and 10.
- [ ] **AC11 Fuzz.** The target builds, its seeds are generated by `just fuzz-seeds`, and
      `just fuzz-dng-write` runs 60 s with no crash; `just lint-fuzz` and `just deny-fuzz`
      pass.
- [ ] **AC12 Docs and gates.** Provenance row, CHANGELOG, README, AGENTS.md block;
      `just test`, `just test-debug` (or CI), `just lint-ci`, `just lint-no-allow`,
      `just fmt-check`, `just doc-check`, `just deny`, `just msrv`, `just validate`,
      `just gate-audit` and `just cost-audit` pass; `cargo semver-checks` reports only
      additions.

## Failing Tests

- **`tests/dng_write.rs`** (tier A):
  - `round_trip_every_framing_and_geometry` — AC2, the table above.
  - `tags_match_the_spec_table` — AC3, walks the output's IFD0 with `Container::parse`.
  - `backward_version_follows_opcode_lists` — AC3.
  - `orientation_is_always_written` — AC3.
  - `every_refusal_is_typed` — AC4, one minimal case per reason, both entry points, kinds.
  - `opcode_list_1_allow_list_under_cutting_framings` — AC4: `FixBadPixelsConstant`
    passes; id 5 (`FixBadPixelsList`) is `UnsupportedOpcodeForFraming` under
    FullActiveArea and StandardCrop and passes under FullSensor.
  - `size_query_matches_write` — AC5, including `dst` one byte short and one long.
  - `develop_equivalence_synthetic` — AC6, oracle 2.
  - `red_proof_round_trip_catches_a_flipped_sample_and_a_moved_crop` — AC7, oracle 3.
  - `write_is_deterministic_whatever_the_buffer_held` — AC10, oracle 9.
- **`tests/dng_write_corpus.rs`** (tier B, skips loudly):
  - `held_mono_files_round_trip_at_every_framing` — AC2/AC6, oracle 4.
  - `standard_crop_develops_like_the_source` — AC6, oracle 4.
  - `red_proof_a_crop_cut_with_verbatim_warp_develops_differently` — AC7, oracle 5 (Q2M).
  - `dnglab_checksums_match` — AC8, oracle 6, tool-gated.
  - `exiftool_reads_the_written_tags` — AC8, oracle 7, tool-gated.
  - `dng_validate_accepts_and_renders_like_the_source` — AC9, oracle 8, env-gated.
- **`tests/dng_write_alloc.rs`:** `written_len_and_write_into_allocate_nothing` — AC10.
- **`tests/error_kind.rs`:** two new cases — AC4.

## Non-Goals

- EXIF block, XMP, the other eight descriptive tags and DEC-074's rendering hints
  (SPEC-081); previews and their IFD placement (SPEC-082).
- Compression, float, CFA, tiles, big-endian, several strips (DEC-071 §4, DEC-075 §2).
- Applying, stripping or rewriting any opcode list; parsing list 2 or 3; widening the
  list-1 allow-list (a DEC first).
- Fractional (`RATIONAL`) `BlackLevel` or crop values: v1 writes integers, as every held
  mono file has.
- Migrating `tests/support/dng_writer.rs` or its tests onto the public writer.
- Reading `BaselineExposure` or other tags the reader does not model.
- A release (a release spec after STAGE-017).

## Notes for the Implementer

- **Do not read** any writer implementation (dnglab/rawler, the AGPL `dng` crate, the
  DNG SDK source, LibTIFF). Run the three tools as black boxes only.
- `dng_validate` prints its verdict to stdout/stderr and exits 0 even with warnings: the
  oracle is "no line containing `Warning` or `Error`", not the exit code.
- Size every develop `dst` with `output_dimensions_with(&sensor, &options)`; the
  plane's own dimensions are wrong whenever there is a margin (`DevelopBufferWrongLength`).
- Tier-B files are large (the Q2M is 8424×5632). Write outputs to the OS temp dir, never
  the repo. Run tier B under `just test` (corpus profile); debug is slow.
- `Error` gains two variants: update `Display`, `kind()` and `tests/error_kind.rs` in the
  same commit, or the wildcard-free match fails to compile (that is the point).
- Commit messages in this repo must be plain prose: no code tokens (see the repo's
  commit rule); PR bodies may use them.
- If a round trip fails because the reader is wrong, stop and report it; do not patch
  the reader in this spec.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
