---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-031                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-006                      # parent project
repo:
  id: irradiance

created_at: 2026-11-23
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >-
    The core of the thesis: a colour Bayer DNG becomes a deterministic monochrome
    plane onyx can use in both its viewport and its export, with highlights that stay
    monotone through clipping.
  delivers:
    - "a public white-balanced, demosaiced linear RGB plane at sensor geometry, in the working space the mixer uses (onyx: DEC-059 colour reference view, its own candidate mixes, the quality study)"
    - "a public mixer from that plane to a monochrome u16 plane at sensor geometry, weights only, carrying its own black and white levels"
    - "a mix option on develop_with_into giving the developed mono at output_dimensions_with geometry, through the existing crop, orientation, warp and resampling"
  explicitly_does_not:
    - "ship filter presets (STAGE-032; the API takes weights only, presets are named weight sets)"
    - "publish a release (the PROJ-006 release stage)"
    - "change the output bits of any existing develop, demosaic or unpack call"
    - "reconstruct clipped highlights spatially"

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

# STAGE-031: Colour to mono: the linear RGB step, the mixer, and the developed mix

## What This Stage Is

The three public steps onyx asked for (its PROJ-006 answers, 2026-11-23), with
the decisions that make them hold together: the working space, how clipping is
detected and handled, and the arithmetic. When it ships, a colour Bayer DNG can
be turned into (1) a white-balanced linear RGB plane, (2) a mixed monochrome
plane at sensor geometry for a GPU viewport, and (3) a developed monochrome
image for export, all additive to the existing API.

## Why Now

PROJ-006's first stage. onyx answered the four framing questions: Q3 and M11
first, weights-only API, two output shapes, and a crates.io release with no
change to existing output bits. Nothing in PROJ-002 or PROJ-005 is in these
files' way: PROJ-005's writer touches the write side, and PROJ-002's only open
spec (SPEC-035) waits on a photograph.

## Success Criteria

- The working space is decided in a DEC, with a test run on held colour files.
  Linear is the expected answer (onyx prefers it: one preset table, the same look
  on every body); the DEC records why if the test run says otherwise.
- The linear RGB step, at sensor geometry, matches today's
  demosaic-then-to-linear composition exactly when the working space is linear
  sRGB, or states the exact relationship if another space is chosen.
- The mixer is bit-exact against an analytic oracle (known RGB and weights in,
  exact mono out), and a red-proof shows the oracle can fail.
- A pixel with a clipped raw channel never comes out darker than the same pixel
  just below clipping: a monotone ramp through clipping on a tier-A fixture.
- The developed mix equals the sensor-geometry mix pushed through the same
  geometry the mono develop uses, so onyx's GPU-to-develop warp parity check
  (its DEC-062) carries over unchanged.
- Every existing output is unchanged: the corpus hashes for develop_into,
  develop_with_into without a mix, develop_color_into, demosaic_into and
  unpack_into do not move, and `just semver-check` reports additions only.
- Serial and threaded variants are byte-identical, and every new entry point has
  a fuzz target and is panic-free on untrusted input.

## Scope

### In scope
- The working-space DEC: linear sRGB primaries (what `to_srgb_linear_into`
  produces today), another linear space, or white-balanced camera RGB.
- The arithmetic of the new path. Today's colour matrix is f64 with each
  multiply and add rounded separately, and CI's determinism leg proves it
  byte-identical across platforms. onyx asked for fixed-point. Design decides
  whether the new path reuses the f64 matrix (reusing a proven step) or a
  fixed-point one (only for the new path; the existing colour output may not
  change).
- Clip detection on the raw camera channels, before white balance and before any
  matrix (onyx's point: after a matrix you cannot tell which channel clipped), and
  what the mixer does with that mask in the working space.
- The levels the mixed sensor-geometry plane carries, so onyx's single normalise
  step applies to it as it does to a mono sensor plane.
- Fuzz targets and tier-A fixtures for each new entry point.

### Explicitly out of scope
- Presets (STAGE-032) and the release (the PROJ-006 release stage).
- Spatial highlight reconstruction.
- X-Trans and other non-Bayer mosaics: refused as the colour path refuses them.
- An `irr` hook is not in or out yet: the mixer spec's design decides whether it is the
  cheapest way to look at a mix on a real file.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

- [ ] SPEC-091 (frame) — [M] The white-balanced linear RGB step at sensor geometry, with the working-space DEC and the arithmetic decision for the new path
- [ ] SPEC-092 (frame) — [M] The mixer at sensor geometry: weights-only rational API, raw-channel clip mask, the clip rule, the plane's black and white levels
- [ ] SPEC-093 (frame) — [M] The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling

**Count:** 0 shipped / 3 active / 0 pending

## Design Notes

- **Order.** The three specs run one at a time, each depending on the one before.
  The first fixes the working space and arithmetic the others build on; the
  second defines the mixed plane the third resamples. They will touch
  `src/color.rs`, `src/demosaic.rs`, `src/develop.rs` and `src/lib.rs`, and may
  append `Error` variants (at the end of the enum, classified by `Error::kind()`,
  DEC-051).
- **The hard question is clipping in a linear space.** The clip mask comes from
  raw camera channels, but the weights apply after the matrix, where every
  output channel mixes all three inputs. "Drop the clipped channel and
  renormalise" (`docs/future-plans.md`) is only well defined in camera RGB.
  The mixer spec's design has to state the rule for a pixel with one, two or three
  clipped raw channels, and show the result is monotone. Candidates to weigh:
  treat the pixel as at white; mix that pixel in camera RGB with
  renormalised weights mapped from the working-space weights; or clamp the
  clipped raw channel before the matrix. Pre-register the oracle before choosing.
- **Weights.** Rational, fixed-point, summing to one (DEC-018). The caller
  passes resolved weights; onyx stores those in its recipe, never a preset name,
  so a later preset-table change cannot alter an existing print.
- **Existing paths stay put.** `develop_with_into` already accepts a CFA plane
  as mono (SPEC-066, DEC-061, "CFA as mono"). The mix option is a separate,
  opt-in field on the `#[non_exhaustive]` `DevelopOptions`; with it absent the
  develop is byte-identical to today.
- **Corpus.** Held colour files: Canon 5D Mark III, Pixel 3a, 4a and 7 Pro,
  Nikon D300S. No colour Leica yet; the Q3 and M11 come first by jysf's call
  (2026-11-23). This stage can prove correctness without them; STAGE-032's
  presets cannot be judged without them.

## Dependencies

### Depends on
- PROJ-002 STAGE-007 and STAGE-008's SPEC-033/034: demosaic, white balance,
  matrix, illuminant interpolation (shipped).
- PROJ-002 STAGE-027..029: the colour path's refusals (shipped).
- PROJ-004: DevelopOptions, framing, warp_alpha and orientation control (shipped).
- External input: onyx's answers to the PROJ-006 questions (2026-11-23).

### Enables
- STAGE-032 (presets as named weight sets).
- The PROJ-006 release, and onyx moving its pin.

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
