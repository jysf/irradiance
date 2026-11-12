---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-015                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: 2026-11-11

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Widens what irradiance can decode from strip-organised DNGs to tile-organised ones, the layout phones and many newer bodies write. That is the precondition for checking the GainMap stage against real files.
  delivers:
    - "plane::unpack_into / plane::validate decode a TILED DNG (TileWidth/TileLength/TileOffsets/TileByteCounts): uncompressed tiles and lossless-JPEG (Compression 7) tiles, including partial edge tiles"
    - "The three held Pixel files (4a, 3a, 7 Pro) decode to a plane whose MD5 equals dnglab --raw-checksum, exactly"
    - "Hostile tile tables are typed errors: offset/count mismatches, overlapping or out-of-bounds tiles, and tile-count overflow never panic or over-allocate (validate-before-allocate, DEC-050)"
  explicitly_does_not:
    - "Applying the Pixel files' GainMap or WarpRectilinear (STAGE-016 / existing opcode paths)"
    - "Any new compression (deflate, JPEG XL, lossy)"
    - "Fractional (rational) BlackLevel or the Pixel 7 Pro's malformed BlackLevel tag (50714) beyond what decode needs; recorded as a finding if it bites"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions:
    - tokens_total: 16682664
      estimated_usd: 11.58
      recorded_at: 2026-11-11
      notes: "Stage orchestration, measured from the orchestrator transcript (session 1833c79a, claude-opus-5-5) from the user's 'Design SPEC-058' message at 2026-11-12T05:19Z through this ship commit: 81 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (docs/cost-tracking.md); no subagents. Covers the SPEC-058 design and Pixel probes, DEC-056, build and verify dispatch and reconciliation, and this ship and stage-ship. A FLOOR: the final wrap-up messages after this commit are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-015: Tiled DNG plane reader: uncompressed and lossless-JPEG tiles

## What This Stage Is

Tiled-DNG support in the plane decoder. Today `src/plane.rs` reads strips only: the
Pixel files fail with "unsupported strip layout: 0 StripOffsets". This stage adds
the tile path for both uncompressed and lossless-JPEG tiles, routes each
lossless-JPEG tile through the existing SOF-3 decoder (PROJ-003), and stitches the
tiles into the same canonical `u16` plane the strip path produces.

## Why Now

The GainMap stage (STAGE-016) needs real files, and every real GainMap file we hold
is tiled. Measured 2026-11-11 against the raw.pixls.us mirror (`data.lfs`): the
Pixel 4a, 3a and 7 Pro carry four **mandatory** 30x40 GainMaps and are tiled. Tiled
layout is also common in phone and newer camera DNGs, so this widens coverage on its
own merits.

## Success Criteria

- All three Pixel files decode bit-exact against `dnglab --raw-checksum`.
- Every held strip file is unchanged, byte for byte.
- Tile-table hostility is fuzzed (a fuzz target covers the new input surface, §12 bar 2), with typed errors and no panics.
- The writer-built tiled fixtures (STAGE-013) round-trip, including partial edge tiles and both byte orders.

## Scope

### In scope
- Tile tags, tile-to-plane placement including partial edge tiles, uncompressed
  and Compression-7 tiles, and validate-before-allocate guards.
- Corpus manifest entries for the Pixel files (tier B: data.lfs, CC0, never
  committed).

### Explicitly out of scope
- GainMap/opcode application (STAGE-016).
- New codecs.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-058 (shipped on 2026-11-11) [M] Tiled plane decode: tile tags, placement and edge tiles, uncompressed and Compression-7 tiles, validate-before-allocate guards, fuzz target; bit-exact against dnglab on the three Pixel files

**Count:** 1 shipped / 0 active / 0 pending

## Design Notes

**Design (2026-11-11, SPEC-058 + DEC-056).** The probe confirmed the codec needs no change: single Pixel tiles wrapped as one-strip DNGs decode bit-exact against dnglab today, and all 192 tiles per file are full 256x256 frames (edge tiles included). Tiles decode in place through a clipped, strided window with no scratch buffer (DEC-056), keeping DEC-016. P=16 is in range and the "false FFC3" was the naive marker scan.

**Design-time probe (2026-11-11).** Pixel 4a tiles are 256x256. Each tile is a
lossless-JPEG frame with P=16, Y=256, X=128, Nf=2. That is the same two-component
column-interleave shape SPEC-046 decodes, so the codec should need no change. Two
things to re-probe at design: whether P=16 is within the decoder's precision range,
and the Pixel 7 Pro's frame (a naive marker scan hit a false FFC3).

The Pixel 7 Pro also carries a malformed `BlackLevel` (50714) and fractional black
levels. Plane decode doesn't need either, but develop will.

## Dependencies

### Depends on
- STAGE-013 (test-only writer): tiled fixtures.
- PROJ-003 (shipped): the SOF-3 decoder.

### Enables
- STAGE-016 (GainMap) on real files.

## Stage-Level Reflection

*Filled at stage ship, 2026-11-11.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes.
  - `plane::unpack_into` and `plane::validate` decode tiled planes:
    - uncompressed tiles;
    - Compression-7 tiles, with Nf=1 and Nf=2;
    - partial right and bottom edge tiles;
    - both II and MM byte orders.
  - Decoding is allocation-free through DEC-056's clipped, strided window.
  - The Pixel 4a, 3a and 7 Pro unpack bit-exact against `dnglab --raw-checksum`.
  - Hostile tile tables are typed errors, refused before anything is written.
  - The held strip files' checksums are unchanged.
- **How many specs did it actually take?** 1 of 1 planned (SPEC-058, complexity M,
  47.43M tokens / $32.57 against a 30M estimate).
- **How many outlines survived unchanged?** 1 of 1. SPEC-058's frame scope held.
- **What changed between starting and shipping?** The design probe removed the codec
  risk: single Pixel tiles already decoded bit-exact, and every edge tile is a full
  256x256 frame. The open question narrowed to where a clipped tile's out-of-plane
  samples go, and DEC-056 answered it without a scratch buffer.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - None codified.
  - DEC-016's wording should say that the ljpeg header parse allocates small
    Huffman tables. That allocation predates this stage; it is recorded as a
    follow-up, not done here.
- **Signals dispositioned at this close?** Stage-close lesson signals were walked:
  - [[tolerant-oracle-cannot-pin-structure]]: N=1 -> N=2, stays `watch` (bar N=3).
    The verify's item 1 is a refusal test indifferent to a partial write.
  - [[probe-the-hard-assumption-at-frame-time]]: applied, not violated. The STAGE-015
    frame probe flagged P=16 and the 7 Pro's frame, and the design probe settled both
    before the build. Still N=1.
  - No other stage-close lesson was touched by this stage.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - No. SPEC-058's follow-ups go to STAGE-016 (a Pixel metadata and colour oracle,
    which needs per-channel BlackLevel) and to a later DEC-016 wording fix.
