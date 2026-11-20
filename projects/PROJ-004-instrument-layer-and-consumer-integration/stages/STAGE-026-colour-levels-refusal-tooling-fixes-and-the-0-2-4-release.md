---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-026                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-20
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Puts STAGE-025's DNG-conformant black normalisation on crates.io, and makes the colour develop refuse malformed levels the way mono already does, so consumers get correct output and a typed error instead of a silently black image.
  delivers:
    - "The four colour develop entries refuse WhiteLevel at or below the maximum applied black with Error::InvalidLevels, before writing anything"
    - "irradiance 0.2.4 (or the version the semver check requires) on crates.io, carrying DEC-070 and the colour refusal"
  explicitly_does_not:
    - "The public DNG writer (STAGE-017): being discussed separately"
    - "crustyimg integration (STAGE-004): gated on crustyimg's STAGE-056"
    - "Any other output change: every file that develops today develops byte-identically"

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

# STAGE-026: Colour levels refusal, tooling fixes and the 0.2.4 release

## What This Stage Is

The release that takes STAGE-025 to crates.io, with the one behaviour gap its verify
found closed first. When it ships:
- the four colour develop entries (`develop_color_into`, `develop_color_f32_into` and both
  threaded entries) refuse a plane whose `WhiteLevel` is at or below the maximum black the
  path applies, with `Error::InvalidLevels`, as mono `BlackLevels::PerPosition` already does.
  Today they develop such a file silently to 0;
- two small tooling gaps from SPEC-079 are closed;
- irradiance 0.2.4 (or whatever the semver check requires) is on crates.io with DEC-070's
  normalisation and the colour refusal.

## Why Now

`main` carries a behaviour change that crates.io does not: DEC-070 (`DEVELOP_VERSION` 7)
changes the Canon 5D Mark III, Pixel 3a and Pixel 4a colour and `PerPosition` output, and a
consumer on 0.2.3 still gets the old normalisation. FU-1 is a behaviour change too, so
releasing both together means one notice and one version bump. The tooling fixes are small
and would otherwise be carried out of PROJ-004.

## Success Criteria

- On every colour entry, `W ≤ B` returns `Error::InvalidLevels` before any output is
  written; the refusal is checked against `B`, the maximum applied black, as DEC-070 §3
  defines. DEC-070 §3's SPEC-078 amendment is superseded by a second amendment.
- Every file that develops today develops byte-identically: no pin moves, and
  `DEVELOP_VERSION` stays 7 unless the design shows an input that changes output.
- `scripts/sdk-stage2.sh` finds the repository through git, so a copy run from another
  worktree is judged correctly.
- `just validate` parses `feedback/` front matter.
- `just semver-check` against crates.io 0.2.3 decides the version; the release notes name
  DEC-070's output change and the new colour refusal; the user tags and publishes.

## Scope

### In scope
- FU-1, the colour levels refusal, and its DEC-070 amendment.
- The two SPEC-079 tooling follow-ups.
- The release, as a release spec minted with `just new-release-spec` at its design.

### Explicitly out of scope
- STAGE-017, the public DNG writer. Under discussion with the maintainer; onyx may want to
  write DNGs.
- STAGE-004, crustyimg integration.
- `BlackLevelDeltaH/V` (DEC-070 §5).
- Any change to output for a file that develops today.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-080 (frame) — [S] Colour levels refusal (FU-1): the four colour entries refuse WhiteLevel at or below the maximum applied black with Error::InvalidLevels before writing, as mono PerPosition does; DEC-070 §3 amended again; no pin moves. Plus the two SPEC-079 tooling follow-ups: sdk-stage2.sh finds the repo through git, and just validate parses feedback/ front matter

The release spec is minted with `just new-release-spec` when SPEC-080 has shipped, as
SPEC-077 was for 0.2.3.

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- **Order.** The refusal spec ships first; the release spec follows it, so the release
  carries both behaviour changes.
- **Semver.** 0.2.3 was a patch under DEC-052's precedent for behaviour fixes. A new refusal
  on an existing error variant is the same shape as 0.2.3's `InvalidBlackLevelPattern`; the
  semver check and the release design decide, not this frame.
- **Diff refusal paths against the last tag.** STAGE-024's lesson: semver-checks cannot see a
  newly refused input, so the release notes must list every input 0.2.3 developed and 0.2.4
  refuses.

## Dependencies

### Depends on
- STAGE-025 (DEC-070, SPEC-078, SPEC-079), shipped.
- External: the user's crates.io token for the publish.

### Enables
- Consumers (monorail/onyx, crustyimg) get DNG-conformant black normalisation from a registry
  release.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

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
