---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-032                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-005                      # parent project
repo:
  id: irradiance

created_at: 2026-11-25
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "puts the STAGE-017 writer on crates.io, so onyx's Darkroom v1 can export Linear DNG from a published irradiance"  # one sentence; which part of project.value.thesis
  delivers: ["irradiance 0.2.7 on crates.io with the opt-in write feature", "docs.rs documentation of irradiance::dng::write", "release notes that say what the writer does, its limits, and that nothing changes for a reader-only caller"]  # user-visible capabilities this stage delivers
  explicitly_does_not: ["change library behaviour (a defect found in prep is reported, not fixed)", "PROJ-006's linear sRGB plane (SPEC-091, PR 343), which gets its own release", "the two process-debt items recorded at the STAGE-017 close"]  # what this stage is NOT trying to do

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

# STAGE-032: the 0.2.7 release: the public DNG writer

## What This Stage Is

The release that takes STAGE-017 to crates.io. When it ships, irradiance 0.2.7 is published
with the public DNG writer behind the opt-in `write` feature, docs.rs documents
`irradiance::dng::write`, and a crate on `irradiance = { version = "=0.2.7", features =
["write"] }` writes a DNG that Adobe `dng_validate` accepts. Nothing changes for a caller
that does not enable `write`.

## Why Now

STAGE-017 closed on 2026-11-25 (PR 342). onyx's Darkroom v1 consumes a published crate
(`monorail-000/DEC-075`), so the writer is no use to onyx until it is on crates.io. No
reader code has changed since 0.2.6, so the release is only the writer.

## Success Criteria

- `just semver-check` against crates.io 0.2.6 decides the version. At design (on `main` at
  `153f517`) it reports 202 pass, 58 skip, no semver update required, with and without
  `--all-features`, so the version is the patch 0.2.7.
- The packaged crate compiles and runs the writer as a consumer would, with `write` on,
  and hides it with `write` off. `cargo publish --dry-run` alone builds default features
  only, so it cannot show this.
- docs.rs builds with `write` (`[package.metadata.docs.rs] features = ["write"]`) and
  shows the writer.
- The release notes say what the writer does and what it does not, and that no reader,
  develop or colour result changes. `DEVELOP_VERSION` stays `"8"`.
- A cold verify reviews the prep before the user tags. The user tags and publishes.

## Scope

### In scope
- The 0.2.7 release spec (SPEC-095), minted with `just new-release-spec`, two-phase like
  0.2.6.
- The `docs.rs` metadata table, the version bump, the CHANGELOG promotion and wording
  fixes in the README and CHANGELOG.

### Explicitly out of scope
- Any library behaviour change; a defect found during prep is reported, not fixed here.
- PROJ-006's SPEC-091 (PR 343, open): it is held until `v0.2.7` is tagged, or this
  release is re-scoped.
- The STAGE-017 close's two process-debt items (CI fuzz jobs for `develop` and
  `demosaic`, the 13 `ljpeg` private doc links).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-095 (design done, build HANDOFF-197) — [S] irradiance 0.2.7 release: the public DNG writer

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- Placement: a stage of its own in PROJ-005, as STAGE-028 and STAGE-030 were in PROJ-002,
  because STAGE-017 is closed.
- Version: 0.2.7, a patch. Every change is an addition: an opt-in feature, its module,
  and two variants on the `#[non_exhaustive]` `Error`. 0.2.1 to 0.2.6 shipped additions
  as patches.
- Evidence: the reader's SDK lanes, `irr ifd` diff and timings are not re-run, because
  the reader code is byte-identical to `v0.2.6`; the build confirms that first and runs
  them if it is not. A packaged-crate writer check takes their place.
- Verify: a cold verify. This release adds the most public surface since 0.2.0.

## Dependencies

### Depends on
- STAGE-017 (shipped 2026-11-25): SPEC-061, SPEC-081, SPEC-082, SPEC-094.
- External: the maintainer's tag and `cargo publish`; PR 343 held until the tag.

### Enables
- onyx Darkroom v1's Linear DNG export on a published irradiance.
- PROJ-006's release, after this one.

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
