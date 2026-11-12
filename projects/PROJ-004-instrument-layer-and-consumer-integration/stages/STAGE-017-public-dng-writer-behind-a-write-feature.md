---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-017                     # stable, zero-padded, continuous across the repo
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
    Promotes the clean-room test-only writer into a public, opt-in `write` feature, so consumers (crustyimg RAW-in/DNG-out, tooling, archives) can emit standards-conformant uncompressed DNG from a permissive pure-Rust crate.
  delivers:
    - "A public writer API behind a `write` cargo feature (DEC-055): caller-provided plane plus metadata produce DNG bytes in a caller-provided buffer. No I/O; the default build is unchanged."
    - "Output accepted by irradiance's own reader (round-trip, bit-exact), dnglab and exiftool, and Adobe dng_validate (black-box binary)"
    - "A provenance row: written from the Adobe DNG specification and TIFF 6.0 only"
  explicitly_does_not:
    - "Any compression in the writer (lossless JPEG, deflate, JPEG XL, lossy). Each needs its own patent/licence DEC first (DEC-035 pattern), and DEC-055 says an encumbered codec goes in its own crate."
    - "Re-emit / edit-and-save of an existing DNG (needs owned payloads and verbatim unknown-tag preservation; see STAGE-013's Reader fidelity note)"
    - "Any I/O or CLI (library-not-application)"

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

# STAGE-017: Public DNG writer behind a write feature

## What This Stage Is

The public DNG writer. It takes STAGE-013's test-only core, gives it a stable,
sealed API behind the opt-in `write` feature (DEC-055), and proves the output is
accepted by independent readers. Scope is uncompressed strip and tile output for
mono and CFA planes, with the metadata the reader models.

## Why Now

The maintainer asked to plan it now (2026-11-11). It was parked until after 0.2.0,
and 0.2.0 has shipped. It follows STAGE-013 directly, so the API is designed from a
writer that has already been proven on fixtures. A consumer still has to want it
before it ships: crustyimg RAW-in/DNG-out is the expected first one.

## Success Criteria

- **Feature gating.** The default build gains nothing: `cargo package` and
  `--no-default-features` are unchanged. `write` is additive.
- **Acceptance.** Written files round-trip bit-exact through irradiance and are
  accepted by dnglab, exiftool and `dng_validate`.
- **Manual check.** A manual darktable/Lightroom open is recorded, not gated.
- **Panic-freedom.** No panics on any caller input, with typed errors for
  inconsistent metadata. A fuzz target covers the writer's input surface.
- **Semver.** The new public types are `#[non_exhaustive]` (DEC-052 pattern).

## Scope

### In scope
- The `write` feature and the public API.
- Uncompressed strips and tiles, little- and big-endian.
- DNG version tags, `UniqueCameraModel`, colour metadata pass-through, and
  OpcodeList pass-through bytes.

### Explicitly out of scope
- Compression of any kind.
- Re-emitting an existing DNG.
- Embedded previews.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-061 (frame) [M] Public `write` feature: sealed API over the STAGE-013 core, caller-allocated output, validation + typed errors, fuzz target, round-trip + dnglab/exiftool/dng_validate acceptance

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

DEC-055 governs the crate layout: the writer stays in the crate behind a feature,
and splits out only on its named triggers. The decision to proceed from framing to
design waits for a named consumer ask. If none arrives by the time STAGE-016 ships,
re-ask the maintainer rather than building speculatively.

## Dependencies

### Depends on
- STAGE-013: the clean-room writer core and its verification plan.

### Enables
- crustyimg RAW-in/DNG-out; archival and tooling uses.

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
