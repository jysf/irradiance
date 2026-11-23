---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-028                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-22
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "takes the STAGE-027 honour-or-refuse work and the PR 301 fixes to crates.io, so the published crate matches what main develops and refuses"  # one sentence; which part of project.value.thesis
  delivers: ["irradiance 0.2.5 on crates.io: Pixel 7 Pro colour within 6 codes of Adobe's SDK, and typed refusals for unhonoured and malformed tags"]  # user-visible capabilities this stage delivers
  explicitly_does_not: ["change any develop or refusal behaviour beyond what main already has"]  # what this stage is NOT trying to do

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

# STAGE-028: the 0.2.5 release

## What This Stage Is

The release that takes STAGE-027 and PR 301 to crates.io. When it ships, irradiance 0.2.5
is published with the camera calibration and converged illuminant weight (DEC-076,
`DEVELOP_VERSION` 8), the refusals of unhonoured and malformed tags (DEC-077, DEC-078),
PR 301's four refusals and `PlaneDecoder`, and release notes that name every input whose
result changed since 0.2.4.

## Why Now

`main` is 28 commits past `v0.2.4` and carries a colour change on every held
dual-illuminant file plus three batches of new refusals. A consumer on 0.2.4 still gets
the uncalibrated Pixel 7 Pro colour (1,160 codes off the SDK) and still develops files
that `main` refuses. STAGE-027 is closed, so nothing else is queued to ride along.

## Success Criteria

- `just semver-check` against crates.io 0.2.4 decides the version. At design (on `main`
  at `5a8eb61`, after PR 314) it reports 202 pass, no semver update required, so the
  version is the patch 0.2.5.
- The CHANGELOG covers PR 301, which shipped with no entry.
- The release notes name every input whose result changes since 0.2.4: the outputs that
  move, every new refusal, and every input that was already refused but now reports a
  different error (signal `release-notes-diff-every-changed-refusal-and-what-it-preempts`).
- The SDK colour cross-check and the stage-2 cross-check run on the candidate and match
  their shipped figures.
- The user tags and publishes.

## Scope

### In scope
- The 0.2.5 release spec, minted with `just new-release-spec`, two-phase as 0.2.4.
- The missing PR 301 CHANGELOG entries.

### Explicitly out of scope
- Any library behaviour change. A defect found during prep is reported, not fixed here.
- STAGE-008 (the ColorChecker measurement).
- The audit's unscheduled ranks 8, 9, 11 and 12.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-088 (design) — [S] irradiance 0.2.5 release: calibration, converged weight, unhonoured and malformed tag refusals

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- Placement: a stage of its own in PROJ-002 (as STAGE-026 was in PROJ-004), because
  STAGE-027 is closed and the release covers PR 301 too.
- Version: 0.2.5, a patch, as 0.2.4 was for a colour output change and a new refusal.
  The API additions are a new public type (`PlaneDecoder`), new fields on the
  `#[non_exhaustive]` `Sensor`, and new variants of the `#[non_exhaustive]` `Error`
  and `DevelopPath`. All are additive, and semver-checks agrees.

## Dependencies

- STAGE-027 (shipped 2026-11-22), and PR 301 (merged 2026-11-21).
- External: the maintainer's tag and `cargo publish`.

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
