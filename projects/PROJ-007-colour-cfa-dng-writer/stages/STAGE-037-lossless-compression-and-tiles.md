---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-037                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-007                      # parent project
repo:
  id: irradiance

created_at: 2026-11-27
shipped_at: null

value_contribution:
  advances: "Written files become a practical size: lossless Compression 7 in tiles, for CFA and mono alike, decoding bit for bit"
  delivers:
    - "A public-crate lossless JPEG (ITU-T T.81 process 14) encoder, clean-room, patent-clean (DEC-035), grown from `src/ljpeg.rs`'s test-only encoder"
    - "A tiled layout (`TileWidth`/`TileLength`/`TileOffsets`/`TileByteCounts`) for uncompressed and compressed output"
    - "CFA planes encoded as DNG stores them (two interleaved components per row), mono as one component"
    - "A size-query contract that still lets a caller own the buffer when compressed length is unknown before encoding"
  explicitly_does_not:
    - "Deflate (it needs zlib, a new dependency), lossy JPEG or JXL"
    - "Change uncompressed-strip output, which stays the default and byte-identical"
    - "Parallel encoding (a later `parallel`-feature question; determinism first)"

# Orchestration cost — see STAGE-035's header comment; the orchestrator fills this at close.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-037: Lossless compression and tiles

## What This Stage Is

The writer can store a plane as lossless JPEG (Compression 7) in tiles, the way cameras and
Adobe's converter do. That covers mono and CFA. Each output decodes through irradiance's own
SOF-3 decoder and through dng_validate to the uncompressed samples, bit for bit. Uncompressed
strips stay the default.

## Why Now

The maintainer wants compression and tiles in v1. A 61 MP Bayer file is about 120 MB
uncompressed, and a Q2 Monochrom is 95 MB. It comes after the CFA stages because the CFA
two-component encoding should be designed against a general CFA model, not retrofitted. It
also gives onyx smaller mono exports.

## Success Criteria

- **Lossless.** For every held plane (mono and CFA) and each predictor the encoder offers,
  the SOF-3 decoder and dng_validate return the input samples exactly. A red-proof shows that
  an encoder with an off-by-one predictor fails it.
- **Tiles.** Uncompressed tiled output re-parses bit for bit and passes dng_validate. Edge
  tiles are padded per TIFF 6.0 §15, and the padding is deterministic.
- **Size.** Compressed held files are recorded against the cameras' own compressed sizes
  (reported, not gated).
- **Budget and safety.** Encoding allocates nothing beyond what the size contract states. No
  panics. The fuzz target covers compression options, and identical input gives identical
  bytes.
- **Mono unchanged by default.** No existing output hash moves.

## Scope

### In scope
- The encoder, tiles, the options that select them, and the size-query decision.
- Provenance rows for the encoder (ITU-T T.81 Annex H and DNG 1.7.1's Compression 7 rules).

### Explicitly out of scope
- Everything in `value_contribution.explicitly_does_not`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [M] Tiled layout, uncompressed: tile options, tile geometry checks, `Tile*` tags, deterministic edge padding, round trip and dng_validate for mono and CFA. A DEC for the tile rules and how tiles meet framing cuts
- [ ] (not yet written) — [L] Lossless JPEG encoder and Compression 7 output: the encoder moved from `#[cfg(test)]` into the library (one and two components, predictor 1 at least), Huffman tables built deterministically, the size-query contract for a length unknown before encoding (an upper bound from `written_len` and the actual length returned by `write_into`, or an encode-twice exact size; the DEC decides), SOF-3 and dng_validate oracles with red-proofs, fuzz. **Flagged L.** Split at design into encoder and binding if it reads over ~50M
- [ ] (not yet written) — [S] STAGE-037 follow-ups, bundled (reserve)

**Count:** 0 shipped / 0 active / 3 pending

**Estimate.** Specs about 92M (30 + 50 + 12), orchestration about 60M: about 150M / $110.
The highest-variance stage in the project.

## Design Notes

- DEC-071 §7 promises an exact `written_len`. Compression breaks "exact before writing",
  so this stage needs a DEC that amends §7 without changing what an uncompressed caller
  sees. onyx's export-sheet preflight (DEC-075 §5) relies on `written_len` running every
  check `write_into` runs, and that parity must survive.
- The SOF-3 decoder already reads the cameras' CFA layout (two components, half width), so
  it is the encoder's oracle as written. Reading a copyleft encoder is not allowed
  (`provenance-recorded-per-algorithm`).
- **Added 2026-11-28.** The demo runs as wasm in the browser, where an uncompressed 120 MB
  file is the user's download. If the demo goes public before this stage ships, it writes a
  smaller resolution. A compressed file's `NewRawImageDigest` is computed over the samples as
  DNG defines it, so STAGE-035's digest rule carries over unchanged.
- If the L will not fit one spec, the natural split is the encoder (a pure function, bytes
  in, bytes out) and then its binding into the writer.

## Dependencies

### Depends on
- STAGE-035 (and STAGE-036 for the general CFA model).
- PROJ-003: the SOF-3 decoder and DEC-035.

### Enables
- Compressed exports for onyx; a converter whose output is the size of the camera's file.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?**
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
