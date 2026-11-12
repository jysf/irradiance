---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-016                     # stable, zero-padded, continuous across the repo
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
    Turns the most common real-world required opcode, the OpcodeList2 GainMap (lens shading / flat-field) that Android phones write as mandatory, from a refused file into a correct develop on both the mono and colour paths.
  delivers:
    - "OpcodeList2 GainMap (DNG opcode id 9) parsed, with every field validated against the plane: area, plane, pitch, grid, spacing and origin"
    - "Gain applied exactly as the DNG spec defines it (bilinear interpolation of the gain grid over the area/pitch-selected pixels) on the raw plane before demosaic, on both develop paths. It is the first opcode the colour path applies."
    - "The required-processing check (DEC-049/053) treats an applied GainMap as honoured: Pixel files develop instead of being refused"
  explicitly_does_not:
    - "Other OpcodeList2 opcodes (MapTable, MapPolynomial, DeltaPerRow/Column, ScalePerRow/Column) unless the corpus inventory shows them"
    - "FixVignetteRadial or any OpcodeList3 additions"
    - "Spatial highlight reconstruction or any tuning"

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

# STAGE-016: OpcodeList2 GainMap: parse and apply on mono and colour paths

## What This Stage Is

GainMap support. Parse the opcode, then apply it at the DNG pipeline position for
OpcodeList2: after linearisation and black/white mapping, before demosaic. The mono
and colour paths share the same application. The required-processing check learns
that GainMap is applied, so files that carry it develop instead of being refused.

## Why Now

It was deferred from PROJ-003 until a real file appeared. One now has: three Pixel
files carry four **mandatory** 30x40 GainMaps each, one per CFA phase. A Samsung
SM-G960F carries four **optional**, degenerate 1x1 maps. All four were measured
2026-11-11 from `data.lfs`. Today the Pixels would be refused as soon as they decode.

## Success Criteria

- **Analytic oracle.** GainMap application is exactly computable, so writer-built
  fixtures (STAGE-013) with known grids are checked bit-exact against an
  independent transcription of the spec's interpolation. A red-proof shows the
  oracle catching a perturbed grid, wrong pitch, or off-by-one area.
- **Real files.** The Pixel files develop `Ok` with GainMap applied. They are
  cross-checked against Adobe `dng_validate`'s render, run as a black-box binary;
  dnglab applies no opcodes. The tolerance is pre-registered before measuring.
- **The trivial case.** The Samsung's optional 1x1 maps apply as a constant gain
  and stay consistent with the analytic oracle.
- **No collateral change.** Files without a GainMap develop byte-identically.
  `DEVELOP_VERSION` is bumped only if a previously-`Ok` develop changes (e.g. the
  Samsung, whose optional maps were skipped before).
- **Fuzz.** Hostile GainMap parameters are typed errors: a giant grid, zero pitch,
  or an area outside the plane.

## Scope

### In scope
- The GainMap parser and applier.
- Wiring into the mono and colour develop paths and into the required-processing
  applied set.
- A provenance row (DNG 1.7 opcode chapter).
- The black-box `dng_validate` cross-check. Its licence must be checked before it
  is named in CI or the ledger.

### Explicitly out of scope
- The other OpcodeList2 opcodes, unless the inventory shows them.
- New camera-level corrections.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-059 (frame) [M] GainMap parse + apply on mono and colour paths with the analytic oracle and red-proof (writer fixtures), required-processing wiring, fuzz, provenance
- [ ] SPEC-060 (frame) [S] Real-file GainMap evidence: Pixel files develop, cross-checked against dng_validate within a pre-registered tolerance; Samsung optional-map case; DEVELOP_VERSION decision

**Count:** 0 shipped / 2 active / 0 pending

## Design Notes

Measured layout (Pixel 4a):
- four GainMaps, one per CFA phase;
- areas (0,0), (0,1), (1,0) and (1,1) to (3024,4032);
- RowPitch = ColPitch = 2;
- MapPointsV x MapPointsH = 30 x 40;
- MapPlanes = 1.

The GainMaps are mandatory (Flags = 0), so after STAGE-015 these files are refused
until this stage ships.

## Dependencies

### Depends on
- STAGE-013 (writer): GainMap fixtures.
- STAGE-015 (tiled reader): the Pixel files decode.
- DEC-049/053: the required-processing contract this extends.

### Enables
- Phone DNGs (the most common DNG source) developing correctly.

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
