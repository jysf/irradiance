---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-004                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Delivers the thesis to a place a person can use it, and makes crustyimg's long-standing RAW claim true.
  delivers:
    - "A `raw-develop` cargo feature on crustyimg"
    - "`SourceContainer::RawDeveloped` and the `develop_version` process version"
    - "Truthful `info` output for RAW: real dimensions, orientation, EXIF"
  explicitly_does_not:
    - "Any change to crustyimg's 8-bit operation pipeline — that is its own project"
    - "LUT — it belongs in this library, in linear, and is the next wave"
    - "Publishing to crates.io — that waits until a second camera is supported"

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

# STAGE-004: crustyimg integration: raw-develop feature and metadata truthfulness

## What This Stage Is

When this stage ships, `crustyimg web q2m.dng -o photo.avif` works, and
`crustyimg info q2m.dng` reports the sensor's real properties instead of the
embedded JPEG preview's. `irradiance` is consumed as a path dependency behind a
cargo feature, and the develop output carries a process version so crustyimg's
lockfile stays meaningful.

**Estimated effort: 15–25 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

Integrating now, rather than after more cameras, is what stops the library
drifting into a science project that never lands. It also closes a documented
crustyimg defect: RAW loses 100% of its EXIF and its orientation is never read,
and the container's IFD0 is the only possible source — which this library now
parses.

## Success Criteria

- `crustyimg web q2m.dng -o photo.avif` exits 0 and produces a publishable image
- `crustyimg info q2m.dng` reports sensor dimensions and bit depth, not the preview's
- RAW orientation is applied; EXIF survives
- `--no-default-features` still builds; the feature is additive
- `develop_version` is recorded such that a change to it invalidates the build cache entry

## Scope

### In scope
- crustyimg's `raw-develop` cargo feature, as a path dependency
- The `Image` adapter (the only place the 8-bit downconvert happens)
- `SourceContainer::RawDeveloped` and `source_format` truthfulness
- RAW orientation and EXIF passthrough
- `develop_version` plumbing into the recipe and lockfile

### Explicitly out of scope
- crustyimg's 16-bit pipeline and its linear-light resize defect — both recorded in crustyimg's own backlog
- `crustyimg-lab`
- Publishing `irradiance` to crates.io

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-004` to promote these outlines into real specs.

- [ ] (not yet written) [M] crustyimg `raw-develop` feature, `Image` adapter, `SourceContainer::RawDeveloped`, `develop_version`
- [ ] (not yet written) [M] RAW metadata truthfulness: `source_format`, orientation, and EXIF from the container's IFD0

**Count:** 0 shipped / 0 active / 2 pending

## Design Notes

**The library stays float-native; only the adapter is 8-bit.** crustyimg
flattens to `to_rgba8()` in every operation today. When its 16-bit pipeline lands,
only this adapter changes — no library rework.

This stage is authored in a crustyimg worktree, not here. It is the one stage
whose PRs land in a different repo, and crustyimg's own conventions govern them.

**Stopping point D.**

## Dependencies

### Depends on
- STAGE-003 — the developed output
- External: crustyimg (a separate repo; changes land there)

### Enables
- The maintainer's photo blog building from RAW originals through `crustyimg build`
- PROJ-002 — colour arrives through the same seam

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
