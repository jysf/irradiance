---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-015                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: null

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
  sessions: []                      # - tokens_total: N
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

- [ ] SPEC-058 (frame) [M] Tiled plane decode: tile tags, placement and edge tiles, uncompressed and Compression-7 tiles, validate-before-allocate guards, fuzz target; bit-exact against dnglab on the three Pixel files

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

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

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m — only if this stage was
  framed with `just frame-stage`. Counts outlines whose scope held from frame to
  ship. A low number means framing ran ahead of what was knowable; a high one
  means it's paying for itself.>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
