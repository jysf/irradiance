---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-057
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: punch-list  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [punch-list]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-013
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-055, DEC-008, DEC-009, DEC-003, DEC-053]
  constraints: [oracle-must-be-shown-red, provenance-recorded-per-algorithm, no-copyleft-dependencies, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-058, SPEC-059, SPEC-061, SPEC-045, SPEC-054]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-013's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling STAGE-013's clean-room fixture writer: analytically known mono/Bayer, strip and tile DNGs that STAGE-015/016/017 build on"

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
  tokens_estimate: 20000000
  sessions:
    - cycle: build
      handoff: HANDOFF-127
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 9999310
      estimated_usd: 8.69
      duration_minutes: 20
      recorded_at: 2026-11-11
      notes: "TRUE COST re-summed at ship by the orchestrator from the ended session's transcript (spec-057-build-8bc4e4 project dir, 1 jsonl, 49 unique message.ids, all claude-opus-5-5, no subagents): input 100 + cache-creation 217,054 + cache-read 9,715,908 + output 66,248 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Supersedes the live floor of 8,428,823 / 7.72 taken before the handback commit."
    - cycle: verify
      handoff: HANDOFF-128
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 9330476
      estimated_usd: 6.77
      duration_minutes: 40
      recorded_at: 2026-11-11
      notes: "TRUE COST re-summed at ship by the orchestrator from the ended session's transcript (spec-057-verify-e8b552 project dir, 1 jsonl, 58 unique message.ids, all claude-opus-5-5, no subagents): input 137 + cache-creation 138,518 + cache-read 9,159,476 + output 32,345 (max per message). USD at opus-5-5 list rates. Verdict punch-list, its one item (no test pinned the per-IFD tag sort) fixed in place in PR 199. Supersedes the live floor of 7,793,217 / 5.31."
  totals:
    tokens_total: 19329786
    estimated_usd: 15.46
    session_count: 2
shipped_at: 2026-11-11
---

# SPEC-057: test-only DNG writer: uncompressed mono and Bayer fixtures, strip AND tile layouts, arbitrary OpcodeList1/2/3 bytes (GainMap fixtures)

## Context

This is the only spec in **STAGE-013**, the first stage of PROJ-004. It builds the
fixture writer that the next three stages depend on:

- **STAGE-015 (SPEC-058, tiled reader)** needs tiled fixtures, including partial edge
  tiles and both byte orders. No held strip file is tiled; the only tiled files are
  the Pixel phone DNGs.
- **STAGE-016 (SPEC-059, GainMap)** needs fixtures with analytically known
  GainMaps. No held strip file carries a non-trivial one.
- **STAGE-017 (SPEC-061, public writer)** promotes this core into `src/` behind the
  `write` feature (DEC-055).

**What already exists.** `tests/support/tiff.rs` (SPEC-003 onwards) is a
byte-level TIFF builder. The caller places every IFD and payload at a hand-chosen
offset. That is the right tool for hostile fixtures, such as aliased IFD chains and
absurd offsets. It is the wrong tool for valid fixtures. Each valid builder today
(`cfa_dng_16bit`, `dng_16bit_with_opcode_lists`, the Compression-7 container) is a
one-off:

- 16-bit only;
- 2×2 RGGB or LinearRaw only;
- strips only, IFD0 only;
- no DNG version tags, levels, crop, ActiveArea or colour tags.

A test-only `ljpeg` encoder also exists there (`lj92_strip`,
`lj92_strip_components`, SPEC-045/046). This matters for the design below.

**Design-time probe (2026-11-11, dnglab 0.7.2, exiftool).** Hand-built probe files
(Python, scratch only, not committed) were checked against `dnglab analyze
--raw-checksum`:

| Probe | dnglab checksum = MD5 of the input samples as LE `u16`? |
|---|---|
| 16-bit, LE, 8×4 strip, 2×2 CFA, `DNGVersion` 1.4 + `UniqueCameraModel` | **yes** |
| 12-bit, 8×4 (96 bits per row = 12 bytes), MSB-first packed | **yes** |
| 14-bit, 8×4 (112 bits per row = 14 bytes), MSB-first packed | **yes** |
| 16-bit, 40×20, **tiled** 16×16 (3×2 tiles, partial right and bottom tiles padded with `0xBEEF`), II **and** MM | **yes, both** |
| 14-bit, **6**×4 (84 bits per row, not a byte multiple), continuous stream | **no**: rows 1 and 3 come back zero |
| the same, each row padded to a byte (TIFF 6.0 §Compression=1) | **no**: rows garbled |

`exiftool` reads `DNGVersion`, `UniqueCameraModel`, `BitsPerSample` and
`CFAPattern2` from the same files.

So:

1. A minimal DNG (sensor IFD plus `DNGVersion` plus `UniqueCameraModel`) is accepted.
2. dnglab's checksum is a usable oracle for strips **and** tiles, on both byte
   orders.
3. **Rows that are not a whole number of bytes are an unresolved three-way
   disagreement.** TIFF 6.0 says Compression 1 pads each row to a byte boundary.
   irradiance's reader (DEC-008, `src/plane.rs:708`) reads each strip as one
   continuous bitstream. dnglab matches neither. No held corpus file has such rows,
   so nothing is broken today. This spec does not try to settle it (see Design §4
   and Non-Goals).

## Goal

A test-only, declarative DNG writer: describe a plane and its metadata, get
standards-shaped bytes. It produces mono and CFA DNGs with:

- 8/12/14/16-bit samples;
- strips or tiles;
- either byte order;
- the sensor in IFD0 or in a SubIFD;
- the reader's metadata;
- arbitrary opcode-list and raw-tag payloads.

The output is proven correct against `dnglab --raw-checksum`, which is independent
of irradiance's reader, and round-trips through irradiance where the reader
supports the layout.

## Design

### 1. Where it lives

`tests/support/dng_writer.rs`, `#[path]`-included the way `tiff.rs` is.

- It is not compiled into the library (DEC-055).
- It may `use` `irradiance`'s public tag constants (`irradiance::ifd::TAG_*`), but
  nothing private.
- It does not replace `tiff.rs`. Hostile, byte-placed fixtures stay there.
- Migrating the existing one-off valid builders onto the writer is **not** in scope.
  It would churn test files that are green today.

### 2. Shape: a description in, bytes out

The core is plain data plus one function. Field names are a suggestion; the
behaviour is the contract.

```rust
pub struct DngDescription {
    pub order: Order,                 // reuse tiff.rs's Order
    pub layout: IfdLayout,            // SensorInIfd0 | PreviewIfd0WithSensorSubIfd
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: u32,         // 8 | 12 | 14 | 16 for Uncompressed
    pub kind: PlaneKind,              // LinearRaw | Cfa { repeat_dim: [u16; 2], pattern: Vec<u8>, plane_color: Vec<u8> }
    pub organisation: Organisation,   // Strips { rows_per_strip } | Tiles { tile_width, tile_length, edge_fill: u16 }
    pub payload: Payload,             // Samples(Vec<u16>) | Encoded { compression: u16, chunks: Vec<Vec<u8>> }
    pub meta: DngMeta,                // every field Option; None means the tag is not written
    pub opcode_lists: [Option<Vec<u8>>; 3],
    pub extra_tags: Vec<RawTag>,      // { ifd: TagIfd, tag, field_type, count, bytes }
}

pub fn write_dng(d: &DngDescription) -> Result<Vec<u8>, WriteError>;
```

`DngMeta` covers what the reader models:

- `BlackLevel` (per-channel, plus `BlackLevelRepeatDim`), `WhiteLevel`;
- `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize`, `Orientation`;
- `ColorMatrix1/2`, `ForwardMatrix1/2`, `AsShotNeutral`, `CalibrationIlluminant1/2`;
- `DNGVersion` (default `1.4.0.0`), `DNGBackwardVersion` (default `1.1.0.0`),
  `UniqueCameraModel` (default `"irradiance-fixture"`).

Each tag goes in the IFD the reader takes it from. The colour, illuminant and
camera-model tags go in **IFD0**. The plane, CFA, level, crop and opcode tags go in
the **sensor IFD**. Where they live under `PreviewIfd0WithSensorSubIfd` is the case
real cameras write and the reader's sensor selection has to handle. Check each
placement against `Container::sensor` (`src/ifd.rs:1642`). Do not guess.

**Layout is the writer's job, not the caller's.**

- Entries are sorted by tag.
- Out-of-line values and payloads are placed after the IFDs at word-aligned
  offsets.
- Count and offset arrays (`StripOffsets`/`TileOffsets` and their byte counts) are
  written inline when they fit in 4 bytes and out of line when they don't.
- SHORT/BYTE packing follows TIFF's start-of-field rule, the one `tiff.rs`'s
  `short()` and `inline_bytes()` already encode.
- Output is deterministic: the same description always gives the same bytes.

### 3. Payload pass-through: the writer has no encoder

`Payload::Samples` is packed by the writer as Compression 1:

- **Byte-aligned depths (8 and 16):** plain integers in the container's byte order.
- **Sub-byte depths (12 and 14):** an MSB-first bitstream. Byte order does not
  apply.

This is DEC-008's rule, applied in the writing direction.

Tiles are filled with `edge_fill` beyond the plane, so a reader that wrongly reads
the padding shows a sentinel instead of a plausible zero.

`Payload::Encoded { compression, chunks }` writes caller-supplied bytes verbatim,
one chunk per strip or tile, and writes `Compression = compression`. This is how
SPEC-058 builds Compression-7 tiles: `tiff.rs`'s existing test `ljpeg` encoder
produces each chunk. The writer therefore never contains an encoder. That keeps
STAGE-013's "no compression" scope and DEC-055's codec-isolation rule intact. It
also means SPEC-061 can promote the writer without a codec decision.

### 4. Validation: refuse what would be wrong, allow what is deliberately hostile

`WriteError` is a small enum. `write_dng` returns it, and never panics, when:

- the sample count doesn't match `width × height`;
- a sample exceeds `2^bits − 1`;
- `bits_per_sample` is outside {8, 12, 14, 16} with `Payload::Samples`;
- the CFA pattern length doesn't equal `repeat_dim[0] × repeat_dim[1]`;
- the chunk count of `Encoded` doesn't equal the strip or tile count;
- an offset or size overflows `u32`;
- **a row is not a whole number of bytes.** That is `width × bits_per_sample`
  for strips, or `tile_width × bits_per_sample` for tiles, not divisible by 8 under
  `Payload::Samples`. Per the probe above, there is no agreed answer for these rows.
  The writer refuses them rather than picking one, so no fixture can silently encode
  an unsettled rule.

Hostile fixtures stay possible on purpose:

- `extra_tags` **replaces** any generated tag with the same `(ifd, tag)` and
  otherwise appends. A test can therefore write a malformed `OpcodeList2`, a
  wrong-type `BlackLevel`, or an inconsistent `TileOffsets` count without a second
  API.
- TIFF 6.0's "tile width and length are multiples of 16" is **not** enforced. The
  writer records it in a doc comment. A fixture may break it deliberately, as
  SPEC-058's hostility tests will want to.

### 5. Oracles and red-proof

Run these cheapest first (STAGE-013 verification plan):

1. **Independent oracle: dnglab.** `dnglab analyze --raw-checksum` of the written
   file must equal the MD5 of the input samples as little-endian `u16`
   (`tests/support/md5.rs`, docs/oracle-contract.md §"Sensor plane"). This does not
   go through irradiance's reader, so it checks the writer on its own terms. It is
   tool-gated: when dnglab is absent it skips loudly, naming the tool (the
   `tests/support/tools.rs` pattern). CI does not install dnglab today, so this runs
   locally and in the verify.
2. **Round-trip through irradiance**, for strip layouts only, because the reader
   has no tiles until SPEC-058. `Container::parse`, then `sensor()`, then
   `plane::unpack_into` must give back exactly the input samples. The `Sensor`
   fields must equal the description: dims, bits, levels, CFA, ActiveArea, crop,
   orientation, matrices, neutral, illuminants and opcode bytes.
3. **Container round-trip for tiles.** Until SPEC-058, check that irradiance's
   lossless container reads `TileWidth` (322), `TileLength` (323), `TileOffsets`
   (324) and `TileByteCounts` (325) back as written.
4. **exiftool sanity** (tool-gated): `DNGVersion`, `UniqueCameraModel`,
   `BitsPerSample` and `CFAPattern2` are read back as written.
5. **`dng_validate`**: not run in this spec. It is not installed, and its licence
   has not been checked (STAGE-013 plan item 3). Record that in the handback; it is
   not a gate.

**Red-proof (`oracle-must-be-shown-red`, DEC-009 negative control first).** The
writer has a test-only `fault: Option<WriterFault>` knob that defaults to `None`:

- `SwapByteAlignedOrder`: write 16-bit samples in the opposite byte order;
- `LsbFirstPacking`: pack 12/14-bit samples LSB-first;
- `TileOffsetOffByOne`: point each `TileOffsets` entry one sample late.

The honest file must pass oracles 1 and 2 first. Then each fault must turn the
dnglab checksum and, for strips, the irradiance round-trip red on the same
description. The knob lives only in `tests/support`.

### 6. Provenance

Add a `docs/provenance-ledger.md` row for the writer: written from TIFF 6.0 §2, §8
(Compression 1) and §15 (tiles), the Adobe DNG specification 1.7 (tag placement,
`DNGVersion`), and DEC-008. No writer implementation was read: not dnglab/rawler,
not the AGPL `dng` crate, not the DNG SDK.

## Inputs

- **Files to read:**
  - `tests/support/tiff.rs`: `Order`, the inline-packing helpers, the valid
    builders it generalises, and the test `ljpeg` encoder SPEC-058 will feed through
    `Payload::Encoded`.
  - `src/ifd.rs`: the `TAG_*` constants, `Sensor` (`:679`), `Container::sensor`
    (`:1642`) and the sensor-candidate rules (which IFD each tag is read from).
  - `src/plane.rs`: DEC-008's two packing paths, `SUPPORTED_BITS`.
  - `tests/support/tools.rs` and `tests/support/md5.rs`: the tool-gating and MD5
    helpers.
  - `docs/oracle-contract.md` §"Sensor plane": the `--raw-checksum` definition.
  - DEC-055 (writer placement), DEC-008 (packing), DEC-009 (negative control),
    DEC-003 (tier A is own work; no corpus bytes).
- **External tools:** `dnglab` 0.7.2 and `exiftool`, black-box, tool-gated.

## Outputs

- **Files created:**
  - `tests/support/dng_writer.rs`: the writer.
  - `tests/dng_writer.rs`: its tests.
- **Files modified:**
  - `docs/provenance-ledger.md`: one row.
  - `docs/oracle-contract.md`: one short paragraph recording the probe finding
    about rows that aren't a whole number of bytes, under the Sensor-plane section.
- **New public library API:** none. `src/` is untouched.
- **New flags or features:** none.

## Acceptance Criteria

- [ ] **AC1 Strip round-trip, every depth.** For 8, 12, 14 and 16-bit, LinearRaw
      and 2×2 CFA, under II and MM, with one strip and with several: `write_dng`,
      then irradiance's `unpack_into`, returns the input samples exactly.
- [ ] **AC2 Metadata round-trip.** A fully-populated `DngMeta` reads back through
      `Container::sensor()` field for field. This holds for both `IfdLayout`s. With
      `PreviewIfd0WithSensorSubIfd`, `sensor_candidates()` has exactly one entry,
      the SubIFD.
- [ ] **AC3 Opcode bytes.** Arbitrary `OpcodeList1/2/3` payloads read back
      byte-identical through `Sensor::opcode_list_n`, including a payload of 4 bytes
      or fewer (written inline).
- [ ] **AC4 dnglab, strips.** For every AC1 case, `dnglab --raw-checksum` equals
      MD5(input as LE `u16`). Tool-gated.
- [ ] **AC5 dnglab, tiles.** A tiled description with partial right **and** bottom
      tiles, under II and MM, at 16-bit and at 12-bit (`tile_width × 12` divisible
      by 8), matches dnglab's checksum. The container reads the four tile tags back
      as written. Tool-gated.
- [ ] **AC6 Encoded pass-through.** A `Payload::Encoded` strip built with
      `tiff.rs`'s `lj92_strip` (Compression 7) decodes through irradiance to the
      original samples. The writer added no bytes of its own to the chunk.
- [ ] **AC7 Validation.** Each `WriteError` case in Design §4 returns that variant,
      including the not-a-whole-byte row. No case panics.
- [ ] **AC8 Hostile escape hatch.** An `extra_tags` entry replaces a generated tag,
      for example a malformed `OpcodeList2`. irradiance then records it in
      `malformed_tags` and the develop refuses it (DEC-053). This shows hostile
      fixtures remain expressible.
- [ ] **AC9 Red-proof.** The honest negative control passes first. Then each
      `WriterFault` makes the dnglab checksum differ, and for strips also makes the
      irradiance round-trip differ.
- [ ] **AC10 Determinism.** Writing the same description twice gives identical
      bytes.
- [ ] **AC11 exiftool sanity.** `DNGVersion`, `UniqueCameraModel`, `BitsPerSample`
      and `CFAPattern2` read back as written. Tool-gated.
- [ ] **AC12 Gates green.** `just test`, `just lint-ci`, `just fmt-check`,
      `just validate`, `just doc-check` and `just cost-audit` all pass. `src/` has
      no diff.

## Failing Tests

All in **`tests/dng_writer.rs`**:

- `strip_round_trip_all_depths_both_orders`: AC1, a table over {8, 12, 14, 16} ×
  {LinearRaw, Cfa2x2} × {II, MM} × {1 strip, 3 strips}.
- `metadata_round_trips_in_ifd0_layout` and
  `metadata_round_trips_in_subifd_layout`: AC2.
- `opcode_lists_round_trip_including_inline`: AC3.
- `dnglab_checksum_matches_strips`: AC4, tool-gated.
- `dnglab_checksum_matches_tiles_with_edge_tiles`: AC5, tool-gated.
- `tile_tags_read_back_from_container`: AC5, the container half; no tool needed.
- `encoded_payload_passes_through_lj92`: AC6.
- `write_errors_are_typed`: AC7, one case per variant.
- `extra_tag_overrides_generated_tag`: AC8.
- `red_proof_writer_faults_turn_oracles_red`: AC9, negative control first.
- `write_is_deterministic`: AC10.
- `exiftool_reads_dng_identity_tags`: AC11, tool-gated.

## Non-Goals

- **Any encoder in the writer.** Compressed fixtures come in through
  `Payload::Encoded` from test encoders.
- **Settling rows that aren't a whole number of bytes.** The writer refuses them.
  Whether TIFF row-padding, DEC-008's continuous stream, or neither is right for DNG
  is a reader question. If a real file ever needs it, it gets its own spec and DEC.
  Do **not** change `src/plane.rs` here.
- **Migrating the existing builders in `tiff.rs`**, or the fuzz seeds, onto the
  writer.
- **A public API.** That is SPEC-061 (STAGE-017).
- **Reading tiles in irradiance.** That is SPEC-058.
- **Running `dng_validate`**, or naming it in CI or the ledger.
- **Previews.** `PreviewIfd0WithSensorSubIfd` writes a tiny valid IFD0 image only
  so the sensor sits in a SubIFD, the way real files are laid out. It is not a
  preview feature.

## Notes for the Implementer

- **Do not read** any writer implementation: dnglab/rawler source, the AGPL `dng`
  crate, the DNG SDK, or LibTIFF's writer. Run dnglab and exiftool as black boxes
  only. The provenance row depends on this.
- **The probe's checksum is MD5 of LE `u16`, whatever the file's byte order.** An
  MM file's samples are still hashed little-endian.
- **Big-endian inline SHORTs** sit in the high half of the value word. `tiff.rs`'s
  `short()` comment explains the trap. Reuse it; don't re-derive it.
- **Keep `src/` untouched.** If a round-trip fails because the *reader* is wrong,
  stop and report it as a finding; do not patch `src/` in this spec.
- **Lints:** `tests/support` is outside the crate-root panic-free policy, but
  `write_dng` should still return `WriteError` rather than panic, because SPEC-061
  promotes this core into `src/` under that policy. Prefer checked arithmetic now.

---

## Reflection

*Appended during **ship** (2026-11-11).*

1. **What would I do differently next time?**
   — Pin every structural "the writer does X" claim in the design with its own
   assertion. Design §2 said IFD entries are sorted by tag, but nothing checked it:
   irradiance and dnglab both read unsorted IFDs, so removing the sort left all 25
   tests green until the verify added `ifd_entries_are_written_in_ascending_tag_order`.
   A property no consumer enforces needs a direct test, not an indirect oracle.

2. **Does any template, constraint, or decision need updating?**
   — No. The design-time probe (§12) paid for itself: dnglab's raw checksum was
   confirmed as an independent oracle for strips and padded edge tiles before the
   build started, and the rows-not-a-whole-byte disagreement was found and fenced off
   (the writer refuses them) instead of being discovered mid-build.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. Carried into existing outlines:
   - SPEC-061 (public writer): drop the `fault` field, move `Order` out of
     `tests/support/tiff.rs`, assert or error on `serialize`'s ascending-offset
     assumption, and keep `max_sample` private.
   - SPEC-058 (tiled reader): assert the bottom-edge and 12-bit edge-fill values,
     which today are covered only by dnglab ignoring padding.
   - The rows-not-a-whole-byte question stays open (oracle-contract.md); it needs a
     spec only if a real file carries such rows.

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — none (test-only infrastructure). For the project: before, valid DNG fixtures were
   one-off 16-bit, 2×2, strip-only builders; after, one declarative writer produces mono
   or CFA DNGs at 8/12/14/16 bits, in strips or tiles, either byte order, IFD0 or SubIFD,
   with metadata and opcode payloads, matched against dnglab's checksum in all 32 strip
   cases and all tile cases.
