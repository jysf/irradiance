---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-030                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-23
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "puts DEC-078 section 7 on crates.io, so the published crate refuses an empty or misshapen interpretation tag instead of developing it with the absent-tag default"  # one sentence; which part of project.value.thesis
  delivers: ["irradiance 0.2.6 on crates.io with SPEC-089", "release notes that name every input whose result changes since 0.2.5"]  # user-visible capabilities this stage delivers
  explicitly_does_not: ["change library behaviour (a defect found in prep is reported, not fixed)", "STAGE-008, the ColorChecker measurement", "the PROJ-005 writer"]  # what this stage is NOT trying to do

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

# STAGE-030: the 0.2.6 release

## What This Stage Is

The release that takes STAGE-029 to crates.io. When it ships, irradiance 0.2.6 is
published with SPEC-089: a present interpretation tag that is empty, the wrong length or
zero-denominator refuses the develop that reads it (DEC-078 §7), and the colour code no
longer falls back silently from a present matrix it cannot use. The release notes name
every input whose result changed since 0.2.5.

## Why Now

0.2.5 advertises DEC-078 ("a malformed tag refuses the develop that reads it") but ships
the hole an external review found: a `ColorMatrix2` with eight entries or a zero
denominator develops byte-identically to `ColorMatrix2` absent, and count-zero
`Orientation`, `BlackLevel`, `WhiteLevel` and `CFALayout` read as absent. `main` fixes it
(PR 319) and nothing else in `src/` has changed since `v0.2.5`, so a small patch release
closes the gap for consumers on `irradiance = "0.2"` now, rather than waiting for the
writer.

## Success Criteria

- `just semver-check` against crates.io 0.2.5 decides the version. At design (on `main`
  at `5a5f079`, after PR 320) it reports 202 pass, 58 skip, no semver update required,
  so the version is the patch 0.2.6.
- The release notes name every input whose result changes since 0.2.5: each new refusal,
  each already-refused input that now reports `MalformedTag`, each `Sensor` field that
  now reads `None` for a misshapen tag, and the hand-built `Sensor` colour fallbacks that
  now refuse (signal `release-notes-diff-every-changed-refusal-and-what-it-preempts`,
  codified in the release-spec template).
- No held file's output or `irr ifd` output changes; `DEVELOP_VERSION` stays `"8"`.
- A cold verify reviews the prep before the user tags (0.2.5 had none).
- The user tags and publishes.

## Scope

### In scope
- The 0.2.6 release spec (SPEC-090), minted with `just new-release-spec`, two-phase as
  0.2.5.

### Explicitly out of scope
- Any library behaviour change. A defect found during prep is reported, not fixed here.
- STAGE-008 (the ColorChecker measurement) and the PROJ-005 writer (SPEC-061).
- The audit's unscheduled ranks 8, 9, 11 and 12.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-090 (design) — [S] irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- Placement: a stage of its own in PROJ-002, as STAGE-028 was, because STAGE-029 is
  closed.
- Version: 0.2.6, a patch. SPEC-089 adds no public item, no `Error` variant and no
  output change; it adds refusals and moves three refusals to an earlier error, which
  0.2.4 and 0.2.5 also shipped as patches.
- Verify: a cold verify this time. STAGE-028's reflection asked for one on any release
  with new refusals, and 0.2.4's cold verify found a missed changed input.

## Dependencies

- STAGE-029 (shipped 2026-11-23): SPEC-089, PR 319.
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
