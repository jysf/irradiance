---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-016                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: 2026-11-19

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
  sessions:
    - tokens_total: 34701700
      estimated_usd: 24.00
      recorded_at: 2026-11-19
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'stage-016' message at 2026-11-19T16:53Z through this close: 159 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 358 + cache-creation 374,510 (1-hour) + cache-read 34,200,574 + output 126,258); no subagents. Covers the GainMap design probe, the Adobe DNG SDK download and local build with dng_validate, the throwaway rule prototype and its SDK comparison, DEC-069, both specs and handoffs, two build and two verify dispatches, the two mid-build decisions (refusal-test rewrite, AC2 rewording), two verify-base fast-forwards, the SPEC-059 ship and this ship and close. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
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
- **Real files.** The Pixel files develop `Ok` with GainMap applied, on both paths.
  The `GainMap` step is cross-checked against Adobe `dng_validate`'s stage 2 (SDK 1.7.1,
  built locally, DEC-069 §6), run as a black-box binary; dnglab applies no opcodes. The
  tolerances are pre-registered in SPEC-060 (gain step ±1 code with at least 99.9% exact;
  end to end ≤ 8 codes, explained by a max-black normalisation difference).
- **The trivial case.** A 1x1 map (the Samsung's shape) applies as a constant gain,
  covered by SPEC-059's analytic oracle. The Samsung file itself is not in the held
  corpus.
- **No collateral change.** Files without a GainMap develop byte-identically.
  `DEVELOP_VERSION` goes to 6, because optional GainMaps, skipped before, now apply
  (DEC-069 §5).
- **Fuzz.** Hostile GainMap parameters are typed errors: a block that disagrees with its
  grid, zero pitch or points, and non-finite or non-positive spacing. An area outside the
  image is intersected, as the SDK does, not refused (DEC-069 §2).

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

- [x] SPEC-059 (shipped on 2026-11-19) [M] GainMap parse + apply on mono and colour paths (DEC-069), every `OpcodeList2` GainMap accepted by required processing, public `apply_gain_maps`, the analytic oracle and red-proofs on writer fixtures, fuzz, provenance, `DEVELOP_VERSION` 6
- [x] SPEC-060 (shipped on 2026-11-19) [S] Real-file GainMap evidence: the three Pixels develop on every entry and are pinned; the GainMap step cross-checked against `dng_validate` stage 2 within pre-registered tolerances (env-gated, local SDK); a reproducible SDK recipe

**Count:** 2 shipped / 0 active / 0 pending

## Design Notes

Measured layout (Pixel 4a):
- four GainMaps, one per CFA phase;
- areas (0,0), (0,1), (1,0) and (1,1) to (3024,4032);
- RowPitch = ColPitch = 2;
- MapPointsV x MapPointsH = 30 x 40;
- MapPlanes = 1.

The GainMaps are mandatory (Flags = 0), so after STAGE-015 these files are refused
until this stage ships.

**Design probe, 2026-11-19 (DEC-069).**
- All three Pixels carry four mandatory 30x40 maps spanning the image (pitch 2, gains
  1.005 to 3.43). Their only other opcode is an optional `WarpRectilinear`, which the
  colour path skips.
- With `OpcodeList2` disabled in a scratch copy, they develop `Ok` in colour and mono.
  `GainMap` is their only blocker.
- Adobe DNG SDK 1.7.1's `dng_validate` was built locally. Its `-2` output is stage 2
  after `OpcodeList2`.
- A prototype of DEC-069 §2, run on the SDK's own pre-gain stage 2, matched its
  post-gain stage 2 within ±1 code, with 99.97% exact on all three files.
- **The SDK normalises differently:** by `white − max black`, where we use `white −
  black` per position. That is 0 to 5 codes before gain. It is pre-existing and not
  changed here; carried as a follow-up.

Build SPEC-059 first. SPEC-060 depends on it.

## Dependencies

### Depends on
- STAGE-013 (writer): GainMap fixtures.
- STAGE-015 (tiled reader): the Pixel files decode.
- DEC-049/053: the required-processing contract this extends.

### Enables
- Phone DNGs (the most common DNG source) developing correctly.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes. The Pixel 3a, 4a and 7 Pro
  develop on every mono and colour entry, where both paths refused them with
  `UnsupportedMandatoryOpcode { id: 9 }`. `OpcodeList2` `GainMap` is parsed with every
  field validated and applied at stage 2 by DEC-069, the first opcode the colour path
  applies; required processing honours every list-2 `GainMap`; `apply_gain_maps` is public.
  The step matches Adobe `dng_validate` within 1 code (at least 99.966% exact) on all three
  files. Every file with no `GainMap` is byte-identical; `DEVELOP_VERSION` is 6. SPEC-059 in
  PR 267, SPEC-060 in PR 269.
- **How many specs did it actually take?** 2, as planned.
- **How many outlines survived unchanged?** n/a (not framed with `just frame-stage`).
- **Cost.** SPEC-059 59.02M tokens / $39.59 (estimate 26M, actual L); SPEC-060 27.89M /
  $20.09 (estimate 12M, actual M); orchestration 34.70M / $24.00 (floor). About $83.68 for
  the stage. Both specs ran at more than twice their estimates: fixtures, red-proof copies
  with their own target dirs, and (SPEC-060) the SDK recipe and TIFF reader.
- **What changed between starting and shipping?** The original success criterion to refuse
  an area outside the image was dropped for the SDK's intersect rule, and the reference
  became a locally built Adobe `dng_validate`, which settled five points the DNG spec leaves
  implicit. During build, two refusal pins were rewritten to `Ok` (one approved ahead, one
  after), and AC2's "toward 1" became "corners lift relative to the centre", anchored on the
  map, because the 4a's scene is brighter at the corners. Both verifies were punch lists
  (one rule-clause test gap, three TIFF-reader robustness tests), fast-forwarded into the
  build PRs before merge.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - `projects/_templates/spec.md` gains the rule-clause map paragraph (below).
  - At design, grep the tests for every pin of the behaviour a spec removes, and check a
    behavioural assertion's direction on the real scene before writing it into an AC.
- **Signals dispositioned at this close?**
  - `every-rule-clause-maps-to-a-reachable-test`: N=3 at SPEC-059 (V24, the area
    intersection, tested only where unobservable). **Codified** into the spec template's
    Failing Tests section.
  - The other watch lessons were walked and stay watch, with no new instance here.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - None beyond the two design checks above, which are N=1 each and not recorded as
    signals yet.
- **Carried follow-ups (unframed).**
  - The SDK normalises by `white − max black`, ours per position (0 to 5 codes before gain).
    It would change held colour output.
  - `[XS]` `scripts/sdk-stage2.sh` should find `OpcodeList2` by walking the IFDs, and create
    its output directory only after the in-repo check.
  - A "set means must run" rule for env-gated tier-B checks.
  - The Adobe licence agreement's text for the provenance row.
