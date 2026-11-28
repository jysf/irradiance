---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-036                     # stable, zero-padded, continuous across the repo
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
  advances: "The demo's \"invent a sensor\": the writer describes any colour filter array DNG allows, not only 2×2 Bayer, and the file develops in Adobe's reference"
  delivers:
    - "CFA patterns with any `CFARepeatPatternDim` DNG and the reference accept (up to the SDK's limit, settled at design), including X-Trans-like 6×6 and Quad-Bayer-like 4×4"
    - "`CFAPlaneColor` with three or four colours (for example CMYG or RGBE), with colour-matrix sizes that follow the colour count (3×n)"
    - "An oracle for patterns irradiance cannot develop: the parser round trip plus dng_validate's render, compared against a test-support mosaic of a known image"
    - "A recorded Lightroom Classic open of an invented-CFA file (manual, not gated)"
    - "A local-only reader lane: every PROJ-007 output (and the PROJ-005 mono outputs) opened in darktable, RawTherapee, rawpy/LibRaw and Quick Look, results recorded. These are run as tools, never linked"
  explicitly_does_not:
    - "Develop a non-2×2 or four-colour CFA in irradiance (`UnsupportedCfaPattern` stays)"
    - "Ship named vendor presets (an X-Trans layout) without a patent re-check"
    - "`CFALayout` other than 1"

# Orchestration cost — see STAGE-035's header comment; the orchestrator fills this at close.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-036: Invented sensors

## What This Stage Is

The writer accepts any rectangular colour filter array that DNG can describe: a repeat
pattern larger than 2×2, and three or four plane colours. A caller who invents a sensor
gets a file that dng_validate accepts and renders. In the demo, a user sees the demosaic
struggle with their invented pattern.

## Why Now

It is the demo's distinctive half. It is also the cheapest way to learn whether the writer's
CFA model is general before the compression stage fixes a tile layout around it. It comes
after STAGE-035 because it widens that stage's checks and has no new layout.

## Success Criteria

- Each test pattern round-trips through irradiance's parser bit for bit: 2×2 Bayer, a 4×4,
  an X-Trans-like 6×6, a four-colour 2×2, and one deliberately odd pattern such as 3×3. The
  `Cfa` model is already general.
- dng_validate gives no warnings and writes a final render for each pattern. Against a
  test-support mosaic of a known image, the render's mean colour per patch is within a
  tolerance set at design. That is enough to tell a correct pattern from a mis-indexed one,
  and the red-proof is a transposed pattern.
- A pattern DNG or the reference refuses is a typed `CallerError`, and the limit is named in
  the module docs.
- The Lightroom Classic result is recorded either way. It decides what the demo promises.

## Scope

### In scope
- Widening STAGE-035's pattern checks to general repeat dimensions and colour counts.
- Matrix-size rules for four-colour planes.
- A render oracle through dng_validate (local only, DEC-069 §6, as the SDK stage-2 lane is).

### Explicitly out of scope
- Everything in `value_contribution.explicitly_does_not`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [S] Spike, timeboxed to one session: what dng_validate 1.7.1 and Lightroom Classic accept and render (repeat dimensions up to 8×8, four colours, a 3×3, whether `ColorMatrix` must be 4×3 for four colours), with a hand-built file from the STAGE-035 writer. Settles the limits the next spec enforces
- [ ] (not yet written) — [M] General CFA patterns: any repeat dimension and three or four plane colours within the spike's limits, matrix sizes by colour count, refusals named and typed, parser round trip and the dng_validate render oracle with its red-proof, fuzz target widened. A DEC for the accepted pattern space

- [ ] (not yet written) — [S] Local reader lane: a script that opens written files in darktable, RawTherapee, rawpy/LibRaw and Quick Look and records whether each develops, including whether any reader minds the raw plane in IFD0 (DEC-082). Skips loudly when a tool is absent, like `sdk-stage2`. Never in CI
- [ ] (not yet written) — [S] STAGE-036 follow-ups, bundled (reserve)

**Count:** 0 shipped / 0 active / 4 pending

**Estimate.** Specs about 50M (12 + 25 + 13), orchestration about 30M: about 80M / $60.

## Design Notes

- The spike comes first because the success criteria depend on what the reference and
  Lightroom do. That is the PROJ-001 rule: do not frame ahead of what is knowable.
- The render oracle is local-only, as `sdk-stage2` is. CI covers the parser round trip and
  the refusals.
- Gated on an outside input: the Lightroom check is manual. If Lightroom refuses non-Bayer
  patterns, the stage still ships (dng_validate is the bar), and the demo says which apps
  develop invented sensors.

## Dependencies

### Depends on
- STAGE-035: the CFA plane core.

### Enables
- The demo's "invent a sensor"; X-Trans re-wraps if a RAF reader ever exists.

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
