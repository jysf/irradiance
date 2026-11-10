---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-012                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-06
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "Makes the Bayer colour-develop path correct and efficient for real downstream use — the colour that PROJ-002 produces must be right for every DNG geometry, not only even-origin ones, and cheap enough to run per image."
  delivers:
    - "Correct colour for DNGs with an odd ActiveArea origin (today they come out magenta), with output cropped to the active area"
    - "A materially faster demosaic and a leaner-memory colour develop, output bit-identical"
    - "An f32 colour-develop entry point for float consumers (the Python bindings, ML dataloaders)"
  explicitly_does_not:
    - "Change the colour math (the ColorMatrix / Bradford path, DEC-038) or the tone/mono develop path"
    - "Add any runtime dependency, panic on untrusted input, or break determinism"
    - "Introduce threads by default (P3 threading is opt-in, default 1)"

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

# STAGE-012: colour-develop ActiveArea-origin correctness and demosaic performance

## What This Stage Is

The Bayer colour-develop path, made correct for every DNG geometry and cheap
enough to run per image. Today `develop_color_into` picks each pixel's CFA
colour and its BlackLevel/WhiteLevel slot from the **plane's** top-left corner
and emits the whole uncropped plane; it should take both from the **ActiveArea**
origin and crop to it, so a file with an odd active-area origin develops to the
right colour instead of magenta. On top of that correctness fix, the stage makes
`demosaic_into` and `develop_color_into` faster and leaner — output bit-identical
— and adds an f32 colour-develop entry point for float consumers. Raised by the
pyirradiance rawpy/LibRaw benchmark (`docs/reviews/`), and the external 0.2.0
staff review's §3B (geometry) and §5 (perf).

## Why Now

The colour correctness fix (SPEC-047) is a **pre-0.2.0 blocker**: a wrong-colour
image raises no error, LibRaw handles these files, and 0.2.0 is the first tagged
colour release — it must not ship magenta grey for odd-origin cameras. The perf
and memory work is the biggest remaining develop win (demosaic is ~90% of develop
time and the colour path holds ~190 MB of avoidable scratch at 23 MP) and the f32
entry point is what the Python bindings and ML dataloaders actually call; all
three are worth landing in the 0.2.0 surface while the colour path is open.

## Success Criteria

- A DNG with an **odd ActiveArea origin** develops to correct colour — grey stays
  grey (per-channel spread < 1e-3) for all four origin parities on all four Bayer
  phases — and the colour output is cropped to the active area.
- `demosaic_into` is materially faster, output **bit-identical** (the existing
  golden hashes pass), with a measured before/after in `docs/provenance-ledger.md`.
- `develop_color_into` drops its two scratch buffers (~140 MB at 23 MP), output
  bit-identical, with a measured peak-RSS before/after.
- A `develop_color_f32_into` entry point exists whose values equal
  `to_linear_f32_into(develop_color_into(..))` exactly.
- Optional: opt-in row-band threading, default 1 thread, output bit-identical.
- Throughout: zero new dependencies, no panic on untrusted input, determinism held.

## Scope

### In scope
- Indexing the CFA phase and the level-pattern slot from the ActiveArea origin,
  and cropping the colour path to the active area (SPEC-047, P0).
- A demosaic interior fast path and the fused-scratch / f32 memory work, each
  bit-identical and measured (P1, P2).
- Opt-in row-band threading for demosaic (P3, optional).

### Explicitly out of scope
- The colour math itself — the ColorMatrix / Bradford-adaptation path (DEC-038)
  and the ColorChecker ΔE work (SPEC-035) are untouched.
- The mono/tone develop path (`develop_into`), which already resolves geometry.
- Tiled or multi-component layouts beyond what already ships.

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-047 (shipped on 2026-11-06) [M] — **P0, ship first:** the colour path indexes the CFA phase + level slot from the ActiveArea origin (offset, no crop — DEC-040), so odd-origin DNGs develop to correct colour; DEVELOP_VERSION 2→3 + a grey-stays-grey red-proof.
- [ ] SPEC-050 (build) [M] — **P0-follow / §3B:** crop the colour output to the active area, remove the masked border, and bring full DefaultCrop/Orientation parity with `develop_into` (the contract-changing geometry the P0 offset deferred, DEC-040 → DEC-043). Design shipped: DEC-043 crops **at the sink** (scatter each developed pixel to its output slot, kernel byte-for-byte, SPEC-049's memory shape preserved), DEVELOP_VERSION 3→4, goldens regenerated.
- [x] SPEC-048 (shipped on 2026-11-07) [M] — **P1:** demosaic interior fast path (per-row slices, no per-pixel reflect / checked-div on the interior); output bit-identical, before/after ms recorded (DEC-041).
- [x] SPEC-049 (shipped on 2026-11-07) [M] — **P2:** fuse `develop_color_into`'s scratch passes (normalise each row ONCE into a rolling row cache — #147's normalise-on-read regressed time 2.76x and was revised in #148 — write linear sRGB in pass 2) to drop ~134 MB at no speed cost, and add `develop_color_f32_into`; output bit-identical, peak-RSS recorded (DEC-042).
- [ ] (not yet written) [S] — **P3 (optional):** opt-in row-band demosaic threading via `std::thread::scope`, default 1 thread; output bit-identical.

**Count:** 3 shipped / 1 active / 1 pending

## Design Notes

- **DEC-040 (to be formalized at SPEC-047 design):** the CFA phase and the
  BlackLevel/WhiteLevel 2×2 slot are indexed from the ActiveArea origin, not the
  plane origin. Recommended implementation: **crop the colour path to ActiveArea
  before normalize/demosaic**, reusing `develop.rs`'s `resolve_geometry`, so the
  origin becomes (0,0) and both plane-relative computations become correct at
  once — this also removes the masked-border work. The alternative is to offset
  the index (`((y - top) & 1, (x - left) & 1)`) in both `normalize_mosaic` and
  `demosaic_into` without cropping; the design cycle chooses and records why.
  Either way the output changes for odd-origin and masked-border files, so
  **DEVELOP_VERSION bumps 2 → 3** and the affected goldens are regenerated with
  the reason in the commit.
- **Perf discipline (P1–P3), from the staff review §5:** every optimisation is
  gated on a `dst_after == dst_before` proof (the existing golden hashes) plus a
  before/after ms and peak-RSS line in `docs/provenance-ledger.md` and its own
  `DEC-*`. No optimisation changes the integer ops or their order.

## Dependencies

### Depends on
- STAGE-007 (demosaic) and STAGE-008 (colour transform, DEC-038) — the colour
  path this stage corrects and optimises.
- The mono path's `resolve_geometry` (`src/develop.rs`, SPEC-017/018) — reused to
  crop the colour path to the active area.
- External: the pyirradiance rawpy/LibRaw benchmark that raised these (reference
  only; LibRaw is not linked).

### Enables
- PROJ-002's 0.2.0 release (SPEC-042, STAGE-010) — SPEC-047 is a pre-release
  correctness blocker; the f32 entry point belongs in the 0.2.0 API surface.

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
