---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-038                     # stable, zero-padded, continuous across the repo
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
  advances: "The colour writer reaches crates.io, so the demo (a published-crate consumer) and any converter can depend on it"
  delivers:
    - "A 0.2.x release (the next free patch after whatever PROJ-006 cuts) carrying CFA writing, colour tags, invented patterns, tiles and Compression 7, all behind the existing `write` feature"
    - "docs.rs showing the new options; release notes that say mono output and every reader, develop and colour result are unchanged"
  explicitly_does_not:
    - "Any library behaviour change; a defect found during prep is reported, not fixed here"

# Orchestration cost — see STAGE-035's header comment; the orchestrator fills this at close.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-038: The colour writer release

## What This Stage Is

The release that publishes PROJ-007. It uses a two-phase release spec, as for 0.2.7: the
build preps, a cold verify reviews, and the maintainer tags the prep merge by SHA and
publishes.

## Why Now

When STAGE-035 to STAGE-037 have shipped. The demo depends on a published crate. If the demo
wants the uncompressed colour writer sooner, a release can be cut after STAGE-036 instead
(see the brief's cost note). STAGE-037 then gets its own release.

## Success Criteria

- `just semver-check` against the latest published 0.2.x decides the version. Everything is
  additive under `#[non_exhaustive]`, so a patch is expected.
- A packaged-crate check with `write` on writes a CFA file and a compressed file that
  dng_validate accepts. With `write` off, nothing is visible (the SPEC-095 lane).
- `DEVELOP_VERSION` is unchanged. Mono writer output hashes are unchanged.
- The maintainer tags and publishes.

## Scope

### In scope
- One release spec (`just new-release-spec`), the version bump, the CHANGELOG promotion and
  the README writer section.

### Explicitly out of scope
- Any behaviour change.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [S] irradiance 0.2.x release: the colour (CFA) DNG writer, two-phase, packaged-crate writer check with `write` on, tag by SHA, parallel PRs rebase their CHANGELOG entries after the prep merges

**Count:** 0 shipped / 0 active / 1 pending

**Estimate.** About 45M / $30 with orchestration, from STAGE-032's 44.6M / $30.69.

## Design Notes

- Mint it with `just new-release-spec`, not `frame-stage`. Settle the version number at
  design, because PROJ-006 cuts releases in parallel.

## Dependencies

### Depends on
- STAGE-035, STAGE-036, STAGE-037.

### Enables
- The demo on crates.io; any converter consumer.

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
