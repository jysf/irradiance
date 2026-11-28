---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-039                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-008                      # parent project
repo:
  id: irradiance

created_at: 2026-11-28
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "the scene-linear f32 entry, its curves and its sRGB encoders: the whole of PROJ-008's thesis except the release"
  delivers:
    - "tone::tone_map_f32_into: scene-linear f32 at or above 0, values above 1.0 included, mapped into [0, 1] by extended Reinhard (SPEC-101) or Hable (SPEC-103)"
    - "[0, 1] f32 encoded to 16- and 8-bit sRGB through the existing SRGB_ENCODE_* tables (SPEC-102)"
  explicitly_does_not:
    - "change develop_into, DEC-018's u16 output, the existing tone encoders or the committed tables"
    - "AgX, a luminance mode, auto-exposure, or any I/O"

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

# STAGE-039: Tone-map core

## What This Stage Is

`tone` gains a scene-linear `f32` path. `tone_map_f32_into` maps samples at or above 0,
values above 1.0 included, into display-linear `[0, 1]` through a fixed published curve,
per channel, in basic-operations `f64` (`DEC-089`). Two encoders then quantise that `[0, 1]`
`f32` to `u16` and look it up in the existing `SRGB_ENCODE_16` / `SRGB_ENCODE_8` tables. When
the three specs ship, a caller holding `srgb_linear_plane_f32_into`'s unclamped plane
(`DEC-079`) gets an 8- or 16-bit sRGB image whose highlights roll off instead of clipping.

## Why Now

See `PROJ-008`'s brief. This is the project's only build stage; STAGE-040 is the release.

## Success Criteria

- `PROJ-008`'s first four success signals hold for both curves.
- Each spec lands in its own PR (`one-spec-per-pr`) with its oracle shown red.

## Scope

### In scope
- The options struct, the `f32` entry, its refusals and extended Reinhard (SPEC-101).
- The `[0, 1]` `f32` to sRGB16/sRGB8 encoders (SPEC-102).
- Hable (SPEC-103).

### Explicitly out of scope
- Everything `PROJ-008`'s brief puts out of scope.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-101 (design) [M] — `ToneMapOptions`, `tone_map_f32_into`, its refusals and extended Reinhard, with the oracle, red-proof, sweep test and hashes; `DEC-089`
- [ ] (not yet written) [S] — the `[0, 1]` `f32` to sRGB16/sRGB8 encoders through the existing tables
- [ ] (not yet written) [S] — Hable as a second `ToneMapCurve`; decides whether Hable's linear white `W` (11.2) is an option or fixed and documented (the app author's item 1), and documents the formula exactly

**Count:** 0 shipped / 1 active / 2 pending

## Design Notes

- **Order.** SPEC-102 depends on nothing in SPEC-101 but its `[0, 1]` contract, so it can
  design in parallel; SPEC-103 depends on SPEC-101's options struct.
- **Naming.** The new enum is `ToneMapCurve`, not `ToneCurve`: `DEC-025` and `src/tone.rs`
  already call the sRGB OETF "the tone curve".

## Dependencies

### Depends on
- PROJ-001's `tone` module and tables (`DEC-025`); PROJ-006's unclamped `f32` plane (`DEC-079`).

### Enables
- STAGE-040, the 0.2.x release of the tone mapper.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
