---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-058
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
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
  stage: STAGE-015
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-056, DEC-016, DEC-008, DEC-047, DEC-050, DEC-051, DEC-012, DEC-049, DEC-003, DEC-009]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, no-copyleft-dependencies, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-057, SPEC-059, SPEC-060, SPEC-046, SPEC-039]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-057]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-015's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-015: tiled planes (uncompressed and lossless-JPEG, with edge tiles) decode allocation-free, so the three Pixel phone DNGs unpack bit-exact against dnglab and STAGE-016 can test GainMaps on real files"

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
  tokens_estimate: 30000000
  sessions:
    - cycle: build
      handoff: HANDOFF-129
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 34569832
      estimated_usd: 23.45
      duration_minutes: 120
      recorded_at: 2026-11-11
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript (1 jsonl, 119 unique message.ids, all claude-opus-5-5, no subagents): input 239 + cache-creation 343,587 (1-hour) + cache-read 34,107,458 + output 118,548 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 31,088,670 / 21.47 before the commit, push and PR turns."
    - cycle: verify
      handoff: HANDOFF-130
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 12863934
      estimated_usd: 9.12
      duration_minutes: 150
      recorded_at: 2026-11-11
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended verify transcript (1 jsonl, 71 unique message.ids, all claude-opus-5-5, no subagents): input 149 + cache-creation 179,569 (1-hour) + cache-read 12,644,236 + output 39,980 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 10,428,396 / 7.56 before the commit, push and PR turns."
  totals:
    tokens_total: 47433766
    estimated_usd: 32.57
    session_count: 2
shipped_at: 2026-11-11
---

# SPEC-058: Tiled plane decode: tile tags, placement and edge tiles, uncompressed and Compression-7 tiles, validate-before-allocate guards, fuzz target; bit-exact against dnglab on the three Pixel files

## Context

This is the only spec in **STAGE-015** (PROJ-004). STAGE-016's GainMap work needs
real files, and every held file with a mandatory GainMap is tiled. Today
`src/plane.rs` reads strips only. The Pixel files parse, and the sensor is
selected correctly, but `unpack_into` stops with
`UnsupportedStripLayout { strip_offsets: 0, strip_byte_counts: 0 }`. Its message
says "tiled planes are unsupported" (`src/lib.rs:997`).

SPEC-057 (shipped, #199/#200) provides `tests/support/dng_writer.rs`. With
`Organisation::Tiles` it writes uncompressed tiles, including partial edge tiles,
in II and MM. With `Payload::Encoded` it writes Compression-7 tiles from
`tiff.rs`'s test `ljpeg` encoder (`lj92_strip_components`).

**Design-time probe (2026-11-11, dnglab 0.7.2; main at e7a273f).** The files come
from the raw.pixls.us mirror at `crustimg_redo_plus/data.lfs` (CC0).

| File | Plane | Tiles | Tile frame (all tiles) | dnglab `--raw-checksum` |
|---|---|---|---|---|
| Pixel 4a `PXL_20201121_100251397.dng` | 4032×3024, CFA, 16-bit | 256×256, 16×12 = 192, LONG tags | P=16, Y=256, X=128, Nf=2, H/V 0x11, predictor 1, Pt 0, no DRI | `d1830234ac3f32b07ff73792f0faeabb` |
| Pixel 3a `IMG_20190918_164153.dng` | 4032×3024 | same | same | `eb736a494bdfd2fc5ec799fb798ed2c3` |
| Pixel 7 Pro `PXL_20221204_063131635.dng` | 3882×2924 | same (192) | same | `eb72cb300a27c4bcab745702542269de` |

Findings:

1. **The sensor IFD is already selected.** It is SubIFD #1 under a preview IFD0,
   with all CFA, level, crop and matrix metadata read. Only the plane decode is
   missing.
2. **Edge tiles are coded at the full 256×256 size.**
   - Pixel 4a/3a: the right edge has 192 visible columns of 256, the bottom edge 208
     visible rows.
   - Pixel 7 Pro: 42 visible columns and 172 visible rows.
3. **The existing decoder needs no codec change.** Single tiles were wrapped as
   one-strip Compression-7 DNGs and decoded by today's `unpack_into`. Its MD5
   (little-endian `u16`) equals dnglab's checksum for:
   - Pixel 4a tile 0 (interior), `a340b031…`;
   - tile 15 (right edge), `a3cf5c32…`;
   - tile 191 (bottom-right corner), `c9c984f5…`;
   - Pixel 7 Pro tile 191, `1dbd6e76…`.

   P=16 and the Nf=2 interleave already work. The "false FFC3" seen at framing was
   the naive marker scan, not the file.
4. **Tile tables are well-formed in all three files.** The offsets are ascending,
   no tile extends past the end of the file, and every tile ends with `FFD9`.

**The allocation question.** DEC-016 says `unpack_into` allocates nothing. Full-size
edge tiles therefore cannot be decoded into a scratch buffer and clipped. **DEC-056**
settles it: decode in place through a clipped, strided window. Samples outside the
plane have their entropy-coded difference read and discarded, and decoding stops
after a tile's last visible row.

## Goal

`plane::validate` and `plane::unpack_into` accept tiled planes (uncompressed and
Compression 7, including partial edge tiles), still allocation-free. The three
Pixel files unpack bit-exact against dnglab. Every held strip file is unchanged byte
for byte, and every hostile tile table is a typed error.

## Design

**Follow DEC-056.** Don't re-litigate its rejected alternatives: no scratch buffer,
no caller-supplied scratch, no new entry point.

### 1. Read the tile tags (`src/ifd.rs`)

- Add `pub const TAG_TILE_WIDTH: u16 = 322`, `TAG_TILE_LENGTH = 323`,
  `TAG_TILE_OFFSETS = 324` and `TAG_TILE_BYTE_COUNTS = 325`.
- Add four fields to `Sensor`, which is `#[non_exhaustive]`, so this is additive:
  - `tile_width: Option<u32>`;
  - `tile_length: Option<u32>`;
  - `tile_offsets: Vec<u32>`;
  - `tile_byte_counts: Vec<u32>`.
- Read them the way the strip tags are read (`src/ifd.rs:1844`). They are
  **structural** (DEC-012), so a malformed tile tag fails the read; it is not
  tolerated. SHORT or LONG are both accepted, as for strips. The Pixel files write
  LONG for all four.
- Update the `irr ifd` report to print the tile fields.

### 2. Layout dispatch (`src/plane.rs`)

The shared pre-decode pass (`check_strips`, renamed or wrapped as needed) picks
the layout:
- **Strip tags only:** today's path, byte-for-byte unchanged.
- **Tile tags only** (`TileWidth`, `TileLength` and `TileOffsets` present): the
  tile path.
- **Both, or neither:** a typed error. Neither keeps today's
  `UnsupportedStripLayout`, but its message must no longer claim tiles are
  unsupported.

The precedence `unpack_into` documents (compression → bit depth → pixel count →
`dst.len()` → layout) is unchanged. Tile checks happen at the "layout" step.

### 3. Tile-table validation (shared by `validate` and `unpack_into`)

This step allocates nothing and reads no tile bytes. Each case is a typed error,
never a panic:
- `TileWidth == 0` or `TileLength == 0`.
- `TileOffsets.len() != TileByteCounts.len()`, or either ≠
  `ceil(W/TW) × ceil(H/TL)`, computed in `u64`.
- A tile that starts or ends past `file.len()` (offset + count in `u64`).
- **Uncompressed only:**
  - `TileWidth × BitsPerSample` not divisible by 8. The reason is a whole-byte
    tile row, per SPEC-057's probe.
  - A byte count ≠ `TileWidth × TileLength × bits / 8`.
  - The counts summing to more than `file.len()` (the existing strip alias bound,
    DEC-050 / `StripBytesExceedFile`).

**Error variants.**
- Prefer reusing an existing variant where its meaning fits exactly: `Truncated`
  for a tile past EOF, `PackedSizeMismatch` for an uncompressed byte-count
  mismatch.
- Add new variants only where none fits, for example `UnsupportedTileLayout { … }`
  for zero dimensions, a count mismatch, or both layouts present.
- Every new variant is classified in `Error::kind()`. DEC-051's wildcard-free match
  refuses to compile otherwise.
- Write the `Display` text.
- Layout inconsistencies are `ErrorKind::Corrupt`. "Both strips and tiles" is
  `Corrupt` too.

### 4. Uncompressed tiles

For tile `(tx, ty)`:
- The origin is `(tx·TW, ty·TL)`.
- The visible width is `min(TW, W − x0)` and the visible height `min(TL, H − y0)`.

Decode each tile row with DEC-008's rule:
- **byte-aligned depths:** container-order integers;
- **sub-byte depths:** an MSB-first stream, starting each row on a byte boundary,
  which §3 guarantees.

Write only the first `visible_width` samples of the first `visible_height` rows to
`dst[(y0 + r)·W + x0 ..]`. Reuse the existing `unpack_bitstream` /
`unpack_byte_aligned` helpers on the visible part of each row slice.

### 5. Compression-7 tiles: the window (`src/ljpeg.rs`)

- Generalise `reconstruct` and `reconstruct_two_component` to write through a
  window: `dst`, a stride, an origin, the visible width and the visible height.
  (`decode_into` can gain a sibling, for example `decode_window_into`, or take the
  window directly; the builder chooses.)
  - **A visible sample** is reconstructed and written as today, reading its
    neighbours from `dst` at stride offsets.
  - **A sample right of the visible width** has its difference decoded through the
    same `decode_difference` and dropped. Nothing is stored. In the `Nf=2` case,
    each component's own table is used, in MCU order.
  - **After the last visible row,** return. Don't decode the rest of the tile.
  - **The point transform** is applied to visible samples only.
- **The strip path** calls the same code with a full window: stride = width,
  origin (0, 0), all visible. It must stay byte-identical. That is proven by the
  held files' unchanged `raw_checksum` and the in-crate LJ92 round-trip tests.
- **Bind each tile's frame shape before decoding** (DEC-047 for tiles):
  `samples_per_line × components == TileWidth` and `lines == TileLength`. Otherwise
  return the existing `LjpegPlaneShapeMismatch`. `check_strip_shape` can take the
  tile dimensions directly.
- **Neighbour reads must never touch the out-of-window area.** The DEC-056 argument
  says they don't. Assert it structurally: compute neighbour indices only from
  visible coordinates. The interior bit-exact result over 192 tiles is the
  behavioural proof.

### 6. Validate-before-allocate

`plane::validate(sensor, order, file, max_pixels)` runs:
1. the budget first (unchanged);
2. the compression and bit-depth check;
3. the §3 tile checks.

It reads no tile bytes and allocates nothing. Whatever `validate` accepts,
`unpack_into` cannot fail on structurally; only the decode itself can fail. The
existing `validate`/`unpack_into` parity tests must extend to tiled fixtures.

### 7. Fuzz

- Extend the `plane` fuzz target's seed set (`examples/fuzz-seeds.rs`) with
  writer-built tiled fixtures: uncompressed and Compression-7, with edge tiles, in
  II and MM. The seeds generator `#[path]`-includes `tests/support/dng_writer.rs`
  as it does `tiff.rs`.
- Keep the existing targets: `develop` and `demosaic` already reach `unpack_into`,
  so the tile path is fuzzed end to end.
- Run each fuzz target that reaches the tile path for at least 10 minutes locally
  with `cargo +nightly fuzz` (see the toolchain brief). Report the run counts and
  that there were no crashes.

### 8. Corpus and oracle

- Add manifest entries for the three Pixel files (tier B, never committed):
  - Paths: `GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng`,
    `GOOGLE-PIXEL-3A/IMG_20190918_164153.dng` and
    `GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng`.
  - Pinned `bytes`, `sha256` and `raw_checksum` from the table below.
  - `licence = "CC0-1.0"`, `source` = raw.pixls.us.
  - Notes covering the tiled layout and the mandatory GainMaps.
- To make them present locally, **copy** them from
  `~/PSeven/experiments/crustimg_redo_plus/data.lfs/Google/<model>/` into
  `$IRRADIANCE_CORPUS_DIR` (`~/PSeven/experiments/crustimg_redo_plus/images`) under
  those directory names. This is a machine-local step. The bytes are never
  committed (DEC-003).
- Update `MANIFEST_ENTRIES` in `tests/corpus_manifest.rs`.
- Extend the bit-exact plane oracle (the existing `raw_checksum` test over manifest
  entries) to cover the three Pixel files. They must match exactly.
- Add one red-proof: mutate the window clip (for example, write one extra visible
  column at the right edge, or skip discarding the right-of-plane differences) and
  show the Pixel checksum goes red. Run the honest pass first (DEC-009).

| File | bytes | sha256 | raw_checksum |
|---|---|---|---|
| Pixel 4a | 13586704 | `11b38186ee3455376291bcdb3049728a3f9dcd94b9e82ff76855d9f3430e44be` | `d1830234ac3f32b07ff73792f0faeabb` |
| Pixel 3a | 12413696 | `78c7bec867f3f739d43df6f027fad36ead73502d062570ce6ca14f59cdc4a0dd` | `eb736a494bdfd2fc5ec799fb798ed2c3` |
| Pixel 7 Pro | 12243982 | `6b921558944db5fdb4f1c63dc64919b6666ef700857a187e18500099c6e7c6c6` | `eb72cb300a27c4bcab745702542269de` |

### 9. Develop on the Pixel files (behaviour only, no new develop work)

Once the plane decodes, `develop_*` on a Pixel file must return the
required-processing refusal for its mandatory `OpcodeList2` GainMaps (DEC-049),
typed and without a panic. Assert this per file. STAGE-016 turns it into a
develop. `DEVELOP_VERSION` is unchanged, because no previously `Ok` develop
changes.

### 10. Docs

- Add a CHANGELOG `[Unreleased]` entry: "Added: tiled sensor planes (uncompressed
  and lossless-JPEG), …". No version bump; the maintainer cuts releases.
- Add a `docs/provenance-ledger.md` row: TIFF 6.0 §15 (tiles), DNG 1.7, ITU-T T.81
  Annex H (the difference coding is independent of prediction), DEC-056.
- Update the `plane.rs` module docs.
- Add a conformance-matrix line for the Pixel files.

## Inputs

- **Files to read:**
  - `decisions/DEC-056-…` (this spec's decision) and DEC-016, DEC-008, DEC-047,
    DEC-050, DEC-051, DEC-012, DEC-003.
  - `src/plane.rs`: `validate` at `:282`, `unpack_into` at `:382`, `StripWalk`,
    `check_strips` at `:623`, `unpack_jpeg` at `:810`, `check_strip_shape` at
    `:765`.
  - `src/ljpeg.rs`: `reconstruct` at `:1000`, `reconstruct_two_component` at
    `:1189`, `decode_into` at `:1324`, `decode_difference` at `:849`, and the
    in-crate encoders `encode_lj92*`.
  - `src/ifd.rs`: the strip-tag read at `:1844` and `Sensor` at `:679`.
  - `src/lib.rs`: `Error`, `Error::kind`, and `UnsupportedStripLayout`'s text at
    `:989`.
  - `tests/support/dng_writer.rs` (SPEC-057) and `tests/support/tiff.rs`
    (`lj92_strip_components`).
  - `tests/corpus/manifest.toml`, `tests/corpus_manifest.rs`, and the existing
    `raw_checksum` plane oracle tests.
  - `fuzz/fuzz_targets/plane.rs` and `examples/fuzz-seeds.rs`.
- **External tools:** dnglab 0.7.2 (black box), and `cargo +nightly fuzz`.

## Outputs

- **Files modified:**
  - `src/ifd.rs`: tile tags and `Sensor` fields.
  - `src/plane.rs`: the tile path.
  - `src/ljpeg.rs`: the window.
  - `src/lib.rs`: errors and kinds.
  - `src/bin/irr.rs`: the ifd report.
  - `tests/corpus/manifest.toml` and `tests/corpus_manifest.rs`.
  - `examples/fuzz-seeds.rs`.
  - `CHANGELOG.md`, `docs/provenance-ledger.md`, `docs/conformance-matrix.md`.
- **Files created:** `tests/plane_tiles.rs`, the tile tests.
- **Public API:**
  - Additive only: four `Sensor` fields, four `TAG_*` constants, and any new
    `Error` variants.
  - `validate` and `unpack_into` keep their signatures.
  - **No new allocation** anywhere in `plane` or `ljpeg`.
- **New flags:** none.

## Acceptance Criteria

- [x] **AC1 Uncompressed tiles.** Writer-built tiled planes unpack exactly to the
      input samples:
      - 8/12/16-bit, plus 14-bit where `TW×14` is a whole number of bytes;
      - LinearRaw and CFA;
      - II and MM;
      - partial right **and** bottom tiles, a single-tile plane smaller than its
        tile, and a plane that is an exact multiple of the tile size.

      Padding values (the writer's `edge_fill`) never reach `dst`.
- [x] **AC2 Compression-7 tiles.** Writer-built tiled planes, each tile encoded
      with `lj92_strip_components`, unpack exactly to the input:
      - `Nf=1` and `Nf=2`;
      - precisions including 16;
      - partial right and bottom tiles, including an odd visible width under
        `Nf=2`;
      - II and MM.
- [x] **AC3 Pixel files bit-exact.** All three Pixel files unpack to a plane whose
      MD5 equals the pinned `raw_checksum`. Tier B: each file skips loudly by name
      when absent.
- [x] **AC4 Strips unchanged.** Every held strip file's `raw_checksum` oracle still
      passes, and the LJ92 round-trip and existing plane tests are unchanged and
      green.
- [x] **AC5 Hostile tile tables.** Each case in Design §3 returns its typed error
      from **both** `validate` and `unpack_into`, with the same variant and no
      panic. This includes a tile-count overflow (`ceil × ceil` past `u32`) and a
      transposed tile frame (`LjpegPlaneShapeMismatch`).
- [x] **AC6 No allocation.** No `Vec`, `Box` or other allocation is introduced in
      `src/plane.rs` or `src/ljpeg.rs` by this spec. A grep-level test or review
      check is acceptable. `validate` reads no tile bytes.
- [x] **AC7 Window red-proof.** The honest Pixel (or writer) checksum passes
      first. A mutated clip, such as one extra visible column or no discard of
      right-of-plane differences, then goes red.
- [x] **AC8 Develop refuses the Pixel files.** Each Pixel file's
      `develop_color_into` (and the mono `develop_into`, if its path is reached)
      returns the DEC-049 required-processing error for the mandatory GainMap,
      typed, with no panic.
- [x] **AC9 Fuzz.** The seed set includes tiled fixtures. The plane-reaching fuzz
      targets each run for at least 10 minutes with no crash, and the run counts
      are reported.
- [x] **AC10 Error kinds.** Every new `Error` variant has an `ErrorKind`, `Display`
      text and a test (DEC-051).
- [x] **AC11 Gates green.** These all pass:
      - `just test`
      - `lint-ci`
      - `lint-no-allow`
      - `fmt-check`
      - `validate`
      - `doc-check`
      - `deny`
      - `deny-fuzz`
      - `msrv`
      - `cost-audit`
      - `gate-audit`

## Failing Tests

- **`tests/plane_tiles.rs`** (new)
  - `uncompressed_tiles_round_trip_edges_and_orders`: AC1.
  - `uncompressed_tile_padding_never_reaches_dst`: AC1. Uses a distinctive
    `edge_fill`; also covers the bottom-edge and 12-bit fill that SPEC-057's verify
    flagged.
  - `lj92_tiles_round_trip_nf1_nf2_edges`: AC2.
  - `lj92_tiles_odd_visible_width_nf2`: AC2.
  - `pixel_files_unpack_bit_exact`: AC3, tier B.
  - `hostile_tile_tables_are_typed_from_validate_and_unpack`: AC5.
  - `tile_frame_shape_is_bound_to_the_tile`: AC5 (transposed tile).
  - `window_clip_red_proof`: AC7, negative control first.
  - `pixel_files_develop_refuses_mandatory_gainmap`: AC8, tier B.
- **`src/ljpeg.rs`** unit tests: `full_window_is_byte_identical_to_dense_decode`
  (AC4), and `clipped_window_matches_dense_decode_prefix`. That test decodes the
  same stream dense and windowed, and compares the visible prefix.
- **`tests/corpus_manifest.rs`**: `MANIFEST_ENTRIES` is updated, and the three Pixel
  `raw_checksum`s are pinned.
- **`src/lib.rs`** error tests: AC10.

## Non-Goals

- **Applying the GainMap**, or any opcode (STAGE-016). The Pixel files must still
  be *refused* at develop.
- **Fractional `BlackLevel`, or the Pixel 7 Pro's malformed `BlackLevel` (50714).**
  The plane decode doesn't read them. Record anything that bites as a finding.
- **New compression** (deflate, JPEG XL, lossy), or `PlanarConfiguration = 2`.
- **Parallel tile decode.** DEC-056 names the seam; it is not used here.
- **A scratch buffer or a new public entry point** (DEC-056).
- **A version bump or release.**

## Notes for the Implementer

- **Clean-room.** The tile path is derived from TIFF 6.0 §15, DNG 1.7 and T.81
  Annex H only. Run dnglab as a black box. Do not read rawler, LibTIFF or the DNG
  SDK.
- **The probe's per-tile checksums are a debugging aid.** If a Pixel file goes red,
  wrap a single tile as a one-strip DNG (the SPEC-058 design probe did this) and
  compare one tile at a time.
- **The window argument rests on two facts.** The difference coding never depends
  on the prediction, and the predictors reach only left, up and up-left. If either
  looks false in the code, stop and report it. Do not fall back to a scratch buffer.
- **`Error` and `Sensor` are `#[non_exhaustive]`** (DEC-052), so additions are not
  breaking. Still, list them in the CHANGELOG.
- **Commit-message bodies:** plain prose only, no code tokens. `Cargo.lock` is
  gitignored.

---

## Reflection

*Appended during **ship**, 2026-11-11.*

1. **What would I do differently next time?**
   — Write the refusal ACs as "typed error AND `dst` untouched", not "typed error".
   AC5 named the variants but not atomicity. A mutation that skips the shared
   tile-table pass still refuses with the same variant from the per-tile checks, but
   only after earlier tiles were written. The cold verify found that and pinned it. Also,
   the 30M estimate was low: true cost 47.43M (build 34.57M, verify 12.86M), with the
   build's cache-read volume driven by the size of `src/ljpeg.rs` and the 26-mutation
   verify sweep.

2. **Does any template, constraint, or decision need updating?**
   — Two items, neither done this session:
   - DEC-016's "allocates nothing" is literally true only for uncompressed planes.
     `ljpeg::parse_headers` returns a `Vec<HuffTable>` per strip or tile. That predates
     this spec (the build found it, the verify confirmed SPEC-058 added nothing). Either
     a sentence in DEC-016 or a fixed-size table array, in a later spec.
   - The verify's item 1 is the second case of
     [[tolerant-oracle-cannot-pin-structure]]: a test that checks only the error
     variant is indifferent to a partial write. Signal bumped to N=2 (bar N=3).

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec now. Carried forward:
   - The Pixel files have no metadata oracle. Their 4-value per-channel `BlackLevel`
     does not fit the scalar comparison in `tests/metadata_oracle.rs`, so they are
     skipped by name. Add them to the colour and metadata oracles, cross-checked
     against exiftool and dnglab, with or before STAGE-016's GainMap work.
   - The HuffTable allocation above.
   - DEC-056's per-tile window is the seam for parallel tile decode (DEC-044 pattern);
     out of scope until someone asks for speed on tiled files.

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — Before, a tiled DNG was refused (`plane::unpack_into` read strips only); after,
   tiled planes decode, uncompressed or lossless-JPEG with partial edge tiles and no
   allocation, and the Pixel 4a, 3a and 7 Pro files unpack bit-exact against
   `dnglab --raw-checksum` (d1830234..., eb736a49..., eb72cb30...).
