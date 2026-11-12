---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-056
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-11
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/plane.rs
  - src/ljpeg.rs
  - src/ifd.rs
  - src/lib.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-058-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md

tags:
  - tiles
  - allocation
  - ljpeg
  - plane
  - stage-015
---

# DEC-056: Tiled planes decode in place through a clipped, strided window, with no scratch buffer

## Decision

`plane::unpack_into` decodes a tiled sensor plane **directly into the caller's `dst`**.
Each tile writes to the rectangle it covers in the plane. It allocates nothing, so
DEC-016 holds unchanged for tiled planes.

1. **Uncompressed tiles** (Compression 1) are copied row by row into their
   rectangle. Columns and rows past the plane's right or bottom edge are skipped.
   The DEC-008 rule applies **per tile row**:
   - **byte-aligned depths:** container-order integers;
   - **sub-byte depths:** an MSB-first stream.

   A tile row that is not a whole number of bytes
   (`TileWidth × BitsPerSample` not divisible by 8) is refused with a typed error.
   This follows SPEC-057's probe: those rows are unsettled across TIFF 6.0, this
   reader and dnglab.

2. **Compression-7 tiles** (SOF-3 lossless JPEG) are reconstructed through a
   **destination window**: `dst`, a row stride (the plane width), the tile's origin,
   and the visible width and height (the tile clipped to the plane). The existing
   strip path becomes the degenerate window: stride = width, origin (0, 0), fully
   visible. Its output is byte-identical.

   Inside the window:
   - **A visible sample** is reconstructed exactly as today and written to `dst`.
   - **A sample right of the plane edge** has its Huffman-coded difference **read
     and discarded**. Nothing is stored. This is sound for two reasons:
     - In lossless JPEG (ITU-T T.81 Annex H), the entropy-coded difference does not
       depend on the predicted value. So the bitstream can be advanced without
       reconstructing.
     - Predictors 1–7 reference only the left, above and above-left neighbours. A
       visible sample's neighbours are therefore always visible. Visible columns are
       a left-aligned prefix of the tile and visible rows a top-aligned prefix. For
       the `Nf = 2` column interleave, a sample's left neighbour is the same
       component two columns back, which is still in the prefix.
   - **Rows below the plane edge are not decoded.** Decoding of a tile stops after
     its last visible row. Its remaining bytes are never read.
   - **The point transform** (`Pt`) is applied to visible samples only.

3. **Each tile's frame shape is bound to the tile, not to the plane** (DEC-047,
   applied to tiles): `samples_per_line × components == TileWidth` and
   `lines == TileLength`. That refuses an equal-area transposed tile before
   decoding. Real edge tiles are coded at full tile size: all 192 tiles of each
   Pixel file are full 256 × 256 frames, measured 2026-11-11. A clipped-size edge
   frame is refused, not guessed at.

4. **The tile table is checked before anything is written**, by the same shared
   pass `validate` runs (DEC-050):
   - **Count and geometry:**
     - `TileWidth` and `TileLength` are non-zero.
     - `TileOffsets` and `TileByteCounts` have equal length, and that length equals
       `ceil(W / TileWidth) × ceil(H / TileLength)`, computed in `u64`.
     - A plane carrying both strip and tile tags is refused.
   - **File bounds:**
     - Every tile lies inside `file`.
     - For uncompressed tiles, each byte count equals the packed tile size, and the
       counts must not sum to more than `file.len()` (the existing alias bound for
       strips).

   No `TileWidth`/`TileLength` cap is needed, because nothing is sized from them.

## Context

STAGE-015 adds tiled reading so the Pixel phone DNGs can decode (they are the only
held files with a mandatory GainMap, STAGE-016). The design-time probe (SPEC-058)
found three things:
- The existing SOF-3 decoder already decodes every Pixel tile shape bit-exact
  against dnglab. Interior, right-edge and corner tiles of the Pixel 4a and the
  corner of the 7 Pro were each wrapped as a one-strip DNG.
- Edge tiles are coded at the full 256 × 256 size.
- So the only new work is placing tiles and clipping them, and the open question
  was where a full-size edge tile's decoded samples go when part of it lies outside
  the plane.

DEC-016 promises that `unpack_into` allocates nothing. That promise is part of the
public contract crustyimg builds on, and it keeps DEC-002's `no_std` option open.

## Alternatives Considered

- **One tile-sized scratch buffer per call.**
  - Decode each tile whole into a reusable `Vec<u16>`, then copy the visible part
    into `dst`.
  - Rejected. It is the simplest code, but it breaks DEC-016 for every tiled file.
    It also needs a new bound on `TileWidth × TileLength` so a hostile header cannot
    size the buffer. The window costs one parameterisation of the reconstruct loops,
    which the in-crate LJ92 round-trip oracle, the held Compression-7 files and the
    Pixel files all cover.
- **Scratch only for edge tiles; interior tiles strided straight into `dst`.**
  - Rejected. It needs the strided decoder anyway and keeps the allocation.
- **A caller-supplied scratch slice** (a new `unpack_into` parameter or a sibling
  entry point).
  - Rejected. It is a public API change to work around a problem the window removes.
- **Read-and-discard bottom rows too, instead of stopping.**
  - Rejected as pointless work. Each tile is an independent datastream, so nothing
    after its last visible row is needed. The scan-length rule (DEC-050 §4, complete
    on sample count) is a strip-path property and is not weakened for strips.

## Consequences

- **Positive:**
  - DEC-016 holds for every plane layout.
  - Tiles add no allocation and no new size cap.
  - The Pixel files decode through the same decoder the held Compression-7 files
    already pass.
- **Negative:**
  - The reconstruct loops of the bit-exact decoder change shape (a window instead of
    a dense buffer). The strip path must be proven byte-identical. That proof is the
    unchanged held-file `raw_checksum` oracles, the LJ92 round-trip suite, and the
    fuzz corpus.
- **Follow-ups:**
  - A per-tile `(origin, visible)` window is also the natural seam for future
    parallel tile decode (DEC-044's threading pattern). Not in scope.
