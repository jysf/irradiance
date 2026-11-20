---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-024                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-19
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Puts STAGE-016, STAGE-022 and STAGE-023's unreleased work on crates.io as a compatible 0.2.x release, so the monorail/onyx consumer and other dependents get Pixel phone DNG support and the black-level fixes with `cargo update`.
  delivers:
    - "irradiance 0.2.3 on crates.io: Pixel 3a, 4a and 7 Pro DNGs develop (OpcodeList2 GainMap), the public apply_gain_maps, fractional BlackLevel rounding, colour BlackLevelRepeatDim, Sensor::masked_areas and black_levels_exact"
  explicitly_does_not:
    - "Any library code change beyond release metadata and documentation wording"
    - "A 0.3.0 or any breaking change"
    - "The carried STAGE-016 follow-ups (max-black normalisation, IFD-walking SDK recipe)"

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

# STAGE-024: irradiance 0.2.3 release

## What This Stage Is

The 0.2.3 release of irradiance: one release spec, SPEC-077, that takes everything
shipped to `main` since 0.2.2 to crates.io as a compatible patch. That is STAGE-022,
STAGE-023 and STAGE-016. Phase 1 is reversible prep in a CI-gated PR: the semver check,
the version bump, the CHANGELOG promotion, the release notes, tier-B evidence and the
pre-flight. Phase 2, the tag and `cargo publish`, is the user's.

## Why Now

STAGE-016 closed on 2026-11-19 with the Pixel phones developing, the outcome PROJ-004
has carried since STAGE-013. The user asked for a 0.2.x cut on 2026-11-19. The design's
`cargo semver-checks` run against the published 0.2.2 reports no update required, so a
patch release is sound.

## Success Criteria

- `just semver-check` is clean against crates.io 0.2.2 on the candidate.
- `Cargo.toml` is `0.2.3`, and the CHANGELOG's `[Unreleased]` is promoted to
  `[0.2.3]`, with an empty `[Unreleased]` on top.
- The release notes name the behaviour changes: `DEVELOP_VERSION` 6, the new
  `InvalidBlackLevelPattern` refusal, and which files' output changes.
- `just test` with the corpus is 12/12 with every pin held; `cargo publish --dry-run` is
  clean; and every gate is green.
- The Phase-1 PR merges, the user tags `v0.2.3` and publishes, and the Cut Record is
  filled.

## Scope

### In scope
- SPEC-077: the release prep and its pre-flight.

### Explicitly out of scope
- Library code changes beyond documentation wording.
- STAGE-016's carried follow-ups.
- A breaking release. A semver-check failure stops the build for the user's decision.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-077 (design) [S] irradiance 0.2.3 release: semver check against crates.io 0.2.2, version bump, CHANGELOG promotion, release notes with the behaviour changes, tier-B evidence, two-phase cut (user tags and publishes)

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- **Version: 0.2.3, a patch.** The user asked for 0.2.x. The design ran `cargo
  semver-checks check-release --baseline-version 0.2.2 --all-features` at `b515520`:
  202 pass, 58 skip, no update required.
- **The behaviour changes are fixes, released compatibly as in DEC-052's precedent.**
  - `DEVELOP_VERSION` moving 4 → 6 tells caches to re-develop.
  - The colour-path `InvalidBlackLevelPattern` refusal replaces a wrong guess.
  - Both are stated in the release notes.

## Dependencies

### Depends on
- STAGE-016, STAGE-022 and STAGE-023, all shipped.
- External: the user's crates.io token for Phase 2.

### Enables
- monorail/onyx and other dependents developing Pixel DNGs from a registry release.

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
