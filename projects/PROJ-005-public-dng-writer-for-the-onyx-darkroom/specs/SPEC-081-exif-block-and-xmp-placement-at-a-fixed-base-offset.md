---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-081
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
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
  to_agent: claude-opus-5          # HANDOFF-191 (tier_map.build)
  created_at: 2026-11-24

references:
  decisions: [DEC-008, DEC-016, DEC-050, DEC-051, DEC-052, DEC-058, DEC-069, DEC-071, DEC-072, DEC-074, DEC-075, DEC-080, DEC-081]
  constraints: [no-panics-on-untrusted-input, library-not-application, provenance-recorded-per-algorithm, oracle-must-be-shown-red]
  related_specs: [SPEC-061]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-061]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-017's <capability>". Optional; null is acceptable.
value_link: "EXIF and XMP carry-over for onyx's Linear DNG export (STAGE-017)."

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
  tokens_estimate: 25000000       # framed S, designed M (five option groups, exiftool tier); SPEC-061 ran 35.7M at M
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-24
      notes: "main-loop, not separately metered (AGENTS.md section 4); includes the dng_validate metadata probe"

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-081: EXIF block, XMP and IFD0 descriptive tags (EXIF block at a fixed base offset)

## Context

STAGE-017 (PROJ-005), after SPEC-061 (shipped 2026-11-24, PR 329). SPEC-061's writer
writes the plane and its structural tags, plus `UniqueCameraModel`. It writes nothing
else that a source file carries, so an exported DNG loses its exposure data, GPS,
descriptive tags and rendering hints. On the K-3 III, losing `BaselineExposure` alone
changes 76,681,557 of 76,681,728 rendered samples (DEC-074).

`monorail-000/DEC-075` item 5 (an external input, never in `depends_on`) has
`diamond-exif` build the source's Exif/GPS/Interop IFDs as one ready-made block, and
irradiance place it, write the IFD0 pointer tags, and carry an opaque XMP packet.
DEC-072 fixes the offset contract: the block is built for a fixed base (file offset 8,
little-endian) and the writer never parses or rebases it. onyx agreed in
`monorail-000/DEC-076` item 1 and `diamond-exif` in `diamond-exif/DEC-015` (its builder is
`diamond-exif/SPEC-025`, not built yet, so this spec tests with a hand-built block).
DEC-072's amendment makes both IFD offsets optional, with at least one required.
`monorail-000/DEC-076` item 4 adds IFD0's eight descriptive tags as typed ASCII options
(DEC-071 §3a). DEC-074 adds the four rendering hints, which onyx reads through
`Container::rationals` and passes as stored (`monorail-000/DEC-078` item 1).

## Goal

Extend `WriteOptions` with an optional EXIF block (bytes plus the in-block offsets of its
Exif and GPS IFDs), an optional XMP packet, the eight optional descriptive tags and the
four optional rendering hints. Place the block at the fixed base, write `ExifIFD` /
`GPSInfo`, `XMLPacket`, the descriptive tags and the hints. Check only what DEC-072,
DEC-071 §3a, DEC-074 and DEC-081 name, in `written_len` as well as `write_into`. Prove
with exiftool that the written file carries the block's values and the options exactly,
with a red-proof that a block built for the wrong base is caught, and prove with
`dng_validate` that the K-3 III renders like its source once `BaselineExposure` is
carried.

## Design

### 1. The new options (`src/dng/write.rs`, behind `write`)

```rust
/// Where the EXIF block starts in every written file (DEC-072). Pinned by a test:
/// changing it is a breaking change that needs a DEC.
pub const EXIF_BLOCK_BASE: u32 = 8;

/// The base the caller must build its EXIF block for. Always EXIF_BLOCK_BASE today;
/// DEC-072 asks for a function of the options so a later layout can say so.
pub fn exif_block_base(options: &WriteOptions<'_>) -> u32;

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ExifBlock<'a> {
    /// The block, built for EXIF_BLOCK_BASE, little-endian, every offset in it
    /// already absolute. Copied verbatim; never parsed.
    pub bytes: &'a [u8],
    /// The Exif IFD's offset inside `bytes` (the file offset minus the base).
    pub exif_ifd: Option<u32>,
    /// The GPS IFD's offset inside `bytes`.
    pub gps_ifd: Option<u32>,
}

// New WriteOptions fields (all default None):
pub exif_block: Option<ExifBlock<'a>>,
pub xmp: Option<&'a [u8]>,
pub make: Option<&'a str>,
pub model: Option<&'a str>,
pub software: Option<&'a str>,
pub date_time: Option<&'a str>,
pub artist: Option<&'a str>,
pub copyright: Option<&'a str>,
pub localized_camera_model: Option<&'a str>,
pub camera_serial_number: Option<&'a str>,
pub baseline_exposure: Option<Rational>,     // crate::ifd::Rational
pub baseline_noise: Option<Rational>,
pub baseline_sharpness: Option<Rational>,
pub linear_response_limit: Option<Rational>,
```

`ExifBlock` follows the `WriteOptions` pattern (DEC-052, DEC-058): `#[non_exhaustive]`
with `Default`, so callers start from `ExifBlock::default()` and assign fields. The hints
use irradiance's own `Rational` (DEC-081 §3), the type `mono-core` already holds.

### 2. Tags (all in IFD0, ascending, one fixed slot each)

The 15 new slots join SPEC-061's 25 (40 in all). Each is written only when its option is
`Some`, under every framing (none of them depends on geometry).

| Tag | Number | Type | Count | Value |
|---|---|---|---|---|
| `Make` | 271 | ASCII | len + 1 | `make` |
| `Model` | 272 | ASCII | len + 1 | `model` |
| `Software` | 305 | ASCII | len + 1 | `software` |
| `DateTime` | 306 | ASCII | 20 | `date_time` |
| `Artist` | 315 | ASCII | len + 1 | `artist` |
| `XMLPacket` | 700 | BYTE | len | `xmp`, verbatim |
| `Copyright` | 33432 | ASCII | len + 1 | `copyright` |
| `ExifIFD` | 34665 | LONG | 1 | `EXIF_BLOCK_BASE + exif_ifd` |
| `GPSInfo` | 34853 | LONG | 1 | `EXIF_BLOCK_BASE + gps_ifd` |
| `LocalizedCameraModel` | 50709 | ASCII | len + 1 | `localized_camera_model` |
| `BaselineExposure` | 50730 | SRATIONAL | 1 | `num`, `den` as `i32` |
| `BaselineNoise` | 50731 | RATIONAL | 1 | `num`, `den` as `u32` |
| `BaselineSharpness` | 50732 | RATIONAL | 1 | as above |
| `LinearResponseLimit` | 50734 | RATIONAL | 1 | as above |
| `CameraSerialNumber` | 50735 | ASCII | len + 1 | `camera_serial_number` |

- `Copyright` is written as one ASCII string with its NUL (TIFF 6.0's photographer part
  only; no editor part). The pointer tags are LONG, as the held files store them.
- **An empty string writes an empty tag** (count 1, the NUL only). The held K-3 III
  stores `Artist` and `Copyright` exactly so, the Q2M stores four spaces, and the Typ 246
  a count-0 `Copyright`; `mono-core` copies what the source holds, and `None` is how it
  leaves a tag out. `UniqueCameraModel` keeps SPEC-061's rule (required, not blank).
- Rationals are 8 bytes, so they go out of line, like every value over 4 bytes.

### 3. Layout (DEC-072, DEC-075 §1)

```
offset 0   II 42 <IFD0 offset>
offset 8   the EXIF block, verbatim (when given)
           one 0 byte when the block's length is odd
           IFD0
           out-of-line values, in tag order, each at an even offset (XMLPacket among them)
           the strip, at an even offset
```

The header's IFD0 offset is `8 + align2(block length)` with a block and 8 without
(SPEC-061's `IFD0_OFFSET` becomes a planned value). `ExifIFD` and `GPSInfo` are written
only for the offsets given. No byte of the block is read or changed, and every pad byte
is 0, so the output stays deterministic and never leaks the caller's buffer.

### 4. Checks and their order (DEC-081 §4)

SPEC-061's order is kept, and the new checks are inserted after its step 6
(`UniqueCameraModel`) and before step 7 (the `OpcodeList1` walk). Every one runs in
`plan`, so `written_len` returns the same error as `write_into` for the same input
(SPEC-061 §3, `monorail-000/DEC-078` item 2):

- **6a. Descriptive strings**, in tag order (`Make`, `Model`, `Software`, `DateTime`,
  `Artist`, `Copyright`, `LocalizedCameraModel`, `CameraSerialNumber`): printable ASCII
  (0x20 to 0x7E) and at most 1024 bytes, else `InvalidAscii { field }` with the tag's
  name. `DateTime` must also be exactly `DDDD:DD:DD DD:DD:DD` (19 bytes, ASCII digits at
  the `D` positions, `:` and one space where shown), else `DateTimeInvalid`. The form is
  checked, not the calendar: month 13 passes. The writer never stamps a time.
- **6b. Rendering hints**, in tag order: a zero denominator is
  `RationalInvalid { field }`; so is a negative numerator or denominator in the three
  RATIONAL tags (`BaselineNoise`, `BaselineSharpness`, `LinearResponseLimit`), which are
  unsigned on disk. `BaselineExposure` (SRATIONAL) takes any sign.
- **6c. EXIF block:** no offset at all is `ExifBlockWithoutIfd`. Then each given offset,
  Exif first: it must be even and leave room for the IFD's 2-byte entry count inside the
  block (`offset + 2 <= bytes.len()`), else `ExifIfdOffsetInvalid { ifd, offset }` with
  `ifd` `"Exif"` or `"GPS"`. Nothing else in the block is looked at.
- **6d. XMP:** an empty packet is `EmptyXmp` (a count-0 tag is malformed; DEC-081 §2).
  Its content is never parsed.
- **8. Output size** (SPEC-061's step) now also covers the block, its pad and the XMP
  length: any offset or count past `u32` is `OutputTooLarge`.

Five new `WriteInputError` reasons, all `CallerError`: `DateTimeInvalid`,
`RationalInvalid { field: &'static str }`, `ExifBlockWithoutIfd`,
`ExifIfdOffsetInvalid { ifd: &'static str, offset: u32 }`, `EmptyXmp`. Each has rustdoc
and a `Display` arm. No new `Error` variant: `WriteInputError` is `#[non_exhaustive]`, so
this is additive (DEC-080's placement in `src/write_types.rs` stands).

### 5. Oracles and red-proofs

1. **Tags, tier A.** For a plane with every new option set, walking IFD0 with
   `Container::parse` finds every row of §2 with the exact type, count and bytes; the
   header's IFD0 offset is `8 + align2(len)`; `bytes[8..8 + len]` equals the block; the
   pad byte is 0; absent options write no tag; `Some("")` writes count 1. Run once with an
   odd-length block and once with an even one, and with a block that has only an Exif
   IFD, only a GPS IFD, and both.
2. **Round trip unchanged.** SPEC-061's oracle 1 (`check_round_trip`) holds at every
   framing with every new option set: same samples, levels and geometry, and
   `malformed_tags` empty. The reader does not follow `ExifIFD` (it walks only
   `SubIFDs`), so the block cannot change what irradiance reads.
3. **Block parity, exiftool (tool-gated).** A test-only builder,
   `tests/support/exif_block.rs`, writes a little-endian block for a given base: an Exif
   IFD (`ExposureTime` RATIONAL, `FNumber` RATIONAL, `ISO` SHORT, `DateTimeOriginal`
   ASCII, `ExifVersion` UNDEFINED, and an `InteropOffset` to an Interop IFD with
   `InteroperabilityIndex` `R98`), a GPS IFD (`GPSVersionID` BYTE, `GPSLatitudeRef`
   ASCII, `GPSLatitude` three RATIONALs), and each IFD alone. exiftool on the written
   file reads every value exactly as the builder set it (compare rationals by
   cross-multiplying, as `tests/support/tools.rs` does). exiftool also reads back all
   eight descriptive tags, the four hints and one XMP property (`XMP-dc:Description`) as
   given.
4. **Red-proof: a shifted base.** Negative control first: the oracle-3 comparison passes
   for a block built for base 8. Then the same builder at base 10, placed by the writer
   at 8: the comparison must fail (exiftool reads different or missing values). Build
   errors in the builder itself must not be able to pass this: assert the base-10 block
   differs from the base-8 one.
5. **`dng_validate` and the K-3 render (env-gated, local only, DEC-069 §6).** K-3 III and
   Q2M at `StandardCrop`, with the source's descriptive tags and hints (hints read
   through `Container::ifd0` + `Ifd::entry` + `Container::rationals`, as `mono-core`
   does; strings from the source's IFD0 with the test TIFF support), the oracle-3 block
   and an XMP packet: no `Warning` or `Error` line. The K-3 III's `-16 -tif` render
   equals the source's render sample for sample; the same file without
   `baseline_exposure` renders differently (the DEC-074 probe measured 76,681,557
   samples). The second half is the hints' red-proof.
6. **Base pinned.** `EXIF_BLOCK_BASE == 8`, and `exif_block_base` returns 8 for the
   default options and for options with every field set.
7. **Determinism and allocation.** SPEC-061's oracles 9 and 10 with every new option
   set: same bytes whatever `dst` held; `written_len` and `write_into` allocate nothing.
8. **Preflight parity and order.** SPEC-061's `every_refusal_is_typed` and
   `checks_run_in_the_documented_order` gain one case per new reason, through all three
   entry points, with `kind()`. Each new check is shown to run after `UniqueCameraModel`
   and before the `OpcodeList1` walk (an input failing both reports the earlier).

### 6. Fuzz

Extend `tests/support/dng_write_fuzz.rs`'s input format with the new fields (block bytes
and both offsets, XMP bytes, the eight strings, the four rationals) and keep the
agreement property: `written_len` and `write_into` give the same `Ok` length or the same
error. Add one seed per new refusal and one with every field set to
`examples/fuzz-seeds.rs`, and extend `fuzz_seeds_reach_their_named_outcome`. The
red-proof's copy step already includes `dng_write_fuzz.rs`; if the target includes any
new support file by path, add it to `scripts/lint-fuzz-red-proof.sh` (signal
`fuzz-red-proof-copies-files-a-target-includes`).

### 7. Provenance

Ledger row: TIFF 6.0 §2 (ASCII, RATIONAL, SRATIONAL, `DateTime` form, tags 271 to 315,
33432), Exif 2.32 §4.6.3 (the `ExifIFD` and `GPSInfo` pointer tags in IFD0), XMP
Specification Part 3 §1.1.4 (tag 700 in TIFF), Adobe DNG Specification 1.7.1 (50709,
50730 to 50735), DEC-072, DEC-074 and DEC-081. No writer implementation read; exiftool
and `dng_validate` used as black boxes.

## Implementation Context — probed 2026-11-24

Probed with SPEC-061's public writer (StandardCrop), exiftool 13.55 adding tags to its
output, and Adobe `dng_validate` 1.7.1.

| Probe | Result |
|---|---|
| K-3 III and Q2M, SPEC-061 output as shipped | no warnings |
| Same, plus the source's Exif IFD (no MakerNote), its GPS IFD, the IFD0 descriptive tags present in the source and all four hints | no warnings (both) |
| K-3 III, plus an XMP packet (`XMP-dc:Description`) and a GPS latitude | no warnings; exiftool reads both back |
| Held `Artist` / `Copyright` | K-3 III: count 1, just NUL; Q2M: four spaces + NUL; Typ 246: `Copyright` count 0 |
| Held `DateTime` | K-3 III and Q2M: `YYYY:MM:DD HH:MM:SS`; M Monochrom and Typ 246: absent |
| Held `ExifIFD` / `GPSInfo` | LONG pointers in IFD0; the K-3 III and Typ 246 have a GPS IFD, the Q2M and M Monochrom do not |
| irradiance's reader | walks `SubIFDs` only; it never follows `ExifIFD` or `GPSInfo` |

exiftool placed the Exif IFD after IFD0, not at offset 8, so the probe shows that
`dng_validate` accepts the tags and IFDs, not the placement. Oracle 5 covers placement.
The probe test was a throwaway (`tests/zz_probe_081.rs`), not committed.

## Inputs

- **Files to read:** `src/dng/write.rs` and `src/write_types.rs` (the shipped writer),
  `tests/dng_write.rs`, `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `tests/support/{dng_write_oracle,dng_write_fuzz,tools,tiff,stage2_tiff}.rs`,
  `fuzz/fuzz_targets/dng_write.rs`, `examples/fuzz-seeds.rs`, `src/ifd.rs` (`Rational`,
  `Container::rationals`, `Ifd::entry`), `scripts/lint-fuzz-red-proof.sh`.
- **Decisions:** DEC-071 §2 and §3a, DEC-072 (with its amendment), DEC-074, DEC-075 §1
  and §7, DEC-080, DEC-081; DEC-051, DEC-052, DEC-058, DEC-016, DEC-069 §6.
- **External tools:** exiftool (tool-gated), `dng_validate` (env-gated, never CI).

## Outputs

- **Files created:** `tests/support/exif_block.rs` (the test-only block builder).
- **Files modified:** `src/dng/write.rs` (options, constant, function, slots, layout,
  checks, module docs), `src/write_types.rs` (five reasons with `Display`),
  `tests/dng_write.rs`, `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `tests/support/dng_write_fuzz.rs`, `examples/fuzz-seeds.rs`, `fuzz/seeds/dng_write/*`,
  `docs/provenance-ledger.md`, `CHANGELOG.md` `[Unreleased]` Added, the AGENTS.md writer
  block.
- **New public API (behind `write`):** `EXIF_BLOCK_BASE`, `exif_block_base`,
  `ExifBlock`, the 14 `WriteOptions` fields of §1; five `WriteInputError` reasons.
  `cargo semver-checks` must report additions only.
- **New flags:** none. Every new option defaults to `None` (no tag written).

## Acceptance Criteria

- [ ] **AC1 Options and base.** The §1 API exists behind `write`; every new field defaults
      to `None`; `EXIF_BLOCK_BASE` is 8 and `exif_block_base` returns it (oracle 6).
- [ ] **AC2 Tags and layout.** Oracle 1: every §2 row exact, the §3 layout, empty
      strings as count 1, absent options write nothing, odd and even blocks, Exif-only,
      GPS-only and both.
- [ ] **AC3 Round trip.** Oracle 2 at every framing with every new option set.
- [ ] **AC4 Validation.** Every new reason through `written_len`, `write_into` and
      `write_to_vec`, with `kind()` `CallerError`, in the §4 order (oracle 8). No case
      panics.
- [ ] **AC5 exiftool parity.** Oracle 3, tool-gated.
- [ ] **AC6 Red-proofs.** Oracle 4 after its negative control; the hints half of oracle 5
      when `$IRRADIANCE_DNG_VALIDATE` is set.
- [ ] **AC7 dng_validate.** Oracle 5: no warnings on K-3 III and Q2M with every new
      field set, K-3 render equal to the source's. A SKIP line when the variable is
      unset; a failure when set but unusable.
- [ ] **AC8 Determinism and allocation.** Oracle 7.
- [ ] **AC9 Fuzz.** §6; `just fuzz-dng-write` 60 s with no crash; `just lint-fuzz`,
      `just lint-fuzz-red-proof` and `just deny-fuzz` pass.
- [ ] **AC10 Docs and gates.** Provenance row, CHANGELOG, AGENTS.md block, module docs
      (the "What is written" and "Checks and their order" sections); `just test`,
      `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just doc-check`,
      `just deny`, `just msrv`, `just validate`, `just gate-audit`, `just cost-audit` and
      every CI job on the PR pass; `cargo semver-checks` reports only additions.

## Failing Tests

- **`tests/dng_write.rs`** (tier A):
  - `every_new_tag_matches_the_spec_table` — AC2, oracle 1, with odd and even blocks.
  - `exif_block_sits_at_the_base_and_ifd0_follows_it` — AC2: header offset, block bytes,
    pad byte, pointer values; Exif-only, GPS-only and both.
  - `absent_options_write_no_tag_and_empty_strings_write_count_one` — AC2.
  - `round_trip_holds_with_every_new_option_set` — AC3, every framing.
  - `exif_block_base_is_pinned_to_eight` — AC1, oracle 6.
  - `every_refusal_is_typed` and `checks_run_in_the_documented_order` — AC4, extended.
  - `date_time_form_is_checked_not_the_calendar` — AC4: `2026:13:40 25:61:61` passes;
    `2026-11-24 10:00:00`, 18 and 20 bytes, and a letter in a digit place refuse.
  - `rational_hints_refuse_zero_and_unsigned_negatives` — AC4: `BaselineExposure`
    −1/2 and 1/−2 pass; 0 denominators refuse on all four; a negative numerator or
    denominator refuses on the three RATIONAL tags.
  - `write_is_deterministic_whatever_the_buffer_held` — AC8, extended.
- **`tests/dng_write_corpus.rs`** (tool-gated and env-gated, skip loudly):
  - `exiftool_reads_the_block_and_the_new_tags` — AC5, oracle 3.
  - `red_proof_a_block_built_for_another_base_is_caught` — AC6, oracle 4.
  - `dng_validate_accepts_metadata_and_the_k3_renders_like_its_source` — AC6/AC7,
    oracle 5.
- **`tests/dng_write_alloc.rs`:** extended with every new option set — AC8.
- **`fuzz_seeds_reach_their_named_outcome`:** a seed per new refusal — AC9.

**Rule-clause map.**

| Clause | Test | Path that changes the output |
|---|---|---|
| DEC-072: block verbatim at 8, IFD0 after it, pad even | `exif_block_sits_at_the_base_and_ifd0_follows_it` | header IFD0 offset and block bytes (odd block makes the pad observable) |
| DEC-072: pointer = 8 + offset, each only when given | same, three block shapes | `ExifIFD` / `GPSInfo` values and presence |
| DEC-072: at least one offset | `every_refusal_is_typed` | `ExifBlockWithoutIfd` |
| DEC-072: offset even and inside | same | an odd offset, and `offset + 2 == len + 1`, refuse; `offset + 2 == len` passes |
| DEC-072: the base is a pinned constant and a function | `exif_block_base_is_pinned_to_eight` | the constant and the function's value |
| DEC-072: never rebased | `red_proof_a_block_built_for_another_base_is_caught` | exiftool's reading of the block |
| DEC-071 §3a: printable ASCII, 1024 bound, per field | `every_refusal_is_typed` | `InvalidAscii { field }` names each of the eight |
| DEC-071 §3a: `DateTime` form | `date_time_form_is_checked_not_the_calendar` | `DateTimeInvalid` |
| DEC-081 §2: empty string writes count 1; empty XMP refuses | `absent_options_write_no_tag_and_empty_strings_write_count_one`, `every_refusal_is_typed` | the tag's count; `EmptyXmp` |
| DEC-074: verbatim, zero denominator refused | `rational_hints_refuse_zero_and_unsigned_negatives`, oracle 1 | stored bytes; `RationalInvalid` |
| DEC-081 §3: unsigned tags refuse negatives, SRATIONAL does not | `rational_hints_refuse_zero_and_unsigned_negatives` | `RationalInvalid` vs `Ok` |
| DEC-074: the hint changes the render | `dng_validate_accepts_metadata_and_the_k3_renders_like_its_source` | K-3 render with vs without |
| DEC-081 §4: order, and parity with `written_len` | `checks_run_in_the_documented_order`, fuzz agreement | which error an input failing two checks returns |

## Non-Goals

- Building, filtering or rebasing EXIF; MakerNote handling (all `diamond-exif`'s).
- Interpreting or validating XMP contents.
- Any change to IFD0 structural tags (SPEC-061; DEC-071 §3); any way for a descriptive
  field to name a tag number.
- Reading descriptive values from a source file (`mono-core` does that with
  `diamond-exif`'s reader); a reader accessor for ASCII tags.
- UTF-8 `LocalizedCameraModel` (DEC-071 §3a keeps it ASCII in v1); the editor part of
  `Copyright`.
- Previews (SPEC-082). Refusing an empty opcode list (SPEC-061 behaviour; see the
  follow-up note in DEC-081).

## Notes for the Implementer

- **Do not read** any writer implementation (exiftool's writer, dnglab/rawler, the DNG
  SDK source, LibTIFF). Use exiftool and `dng_validate` as black boxes only.
- **Preflight parity** is the contract onyx built its export sheet on: every new check
  goes in `plan`, never in `serialize`.
- `dng_validate` exits 0 even with warnings: the oracle is "no line containing `Warning`
  or `Error`", as in SPEC-061.
- The slot array grows from 25 to 40; keep it sorted by tag number (the IFD's order is
  the array's order) and keep `SLOTS` the only place the count lives.
- The test block builder is test-only and must not be reused by `src/`. It is the
  stand-in for `diamond-exif/SPEC-025`, not a design for it.
- Run the whole CI set before handing back, including `just lint-fuzz-red-proof`
  (SPEC-061's punch list).
- Commit messages in this repo must be plain prose: no code tokens.
- If a round trip fails because the reader is wrong, stop and report it.
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
