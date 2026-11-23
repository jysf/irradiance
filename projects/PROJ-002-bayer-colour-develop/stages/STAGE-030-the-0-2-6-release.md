---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-030                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-23
shipped_at: 2026-11-23

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
  sessions:
    - tokens_total: 13167720
      estimated_usd: 8.53
      recorded_at: 2026-11-23
      notes: "Floor, measured from the orchestrator transcript (session 38903934, claude-opus-5-5) from the user's request to design 0.2.6 (2026-11-23T13:52Z) to 2026-11-23T16:31Z: 59 unique message.ids, deduped by message.id, no subagents, opus-5-5 rates (input 142 + cache-creation 96,281 (1-hour) + cache-read 13,029,175 + output 42,122). Covers the semver check, the stage frame and SPEC-090's design (PR 321), dispatching the build and verify, reviewing their PRs, the cut runbook, the post-publish checks and clean-host crates, and this ship and close. The last ship messages were not yet flushed to the transcript when this was summed. The build and verify sessions are on SPEC-090's own rows."
                                    # - tokens_total: N
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

- [x] SPEC-090 (shipped on 2026-11-23) — [S] irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

**Count:** 1 shipped / 0 active / 0 pending

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

- **Did we deliver the outcome in "What This Stage Is"?** Yes. irradiance 0.2.6 is on
  crates.io (2026-11-23, annotated tag `v0.2.6` on `50d533d`) with SPEC-089. A clean-host
  crate on `=0.2.6` develops the Pixel 7 Pro, the K-3 III and the Canon exactly as `main`
  and 0.2.5 do, and refuses the Canon with a truncated `ColorMatrix2`, which 0.2.5 develops.
  The notes name every changed input, including one output change (a multi-value IFD0
  `Orientation`) that the design had wrongly ruled out.
- **How many specs did it actually take?** One, SPEC-090 [S], as planned; complexity_actual S.
- **How many outlines survived unchanged?** Not framed with `just frame-stage`.
- **What changed between starting and shipping?** The changed-input list: the build moved
  two classes to new refusals and the cold verify added three, so "no output change" became
  "one output change, on no held file". docs.rs had not built 0.2.6 at close.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Done: `projects/_templates/release-spec.md` Phase 2 asks for an annotated tag and a
    remote `^{}` check (0.2.5 was pushed lightweight). AGENTS.md names 0.2.6 as live.
- **Signals dispositioned at this close?** All `watch` signals with `disposition_at:
  stage-close` were walked; none gained same-outcome evidence and all stay `watch`. The
  codified `release-notes-diff-every-changed-refusal-and-what-it-preempts` held, through
  the build's and verify's probes rather than the design's list; a note records it.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Keep a cold verify on every release with new refusals: this one found three missed
    changed inputs, 0.2.4's found one, and 0.2.5 (no cold verify) is unknown.
- **Cost.** SPEC-090 27.18M / $20.15 (build 14.42M, verify 12.76M, estimate 22M).
  Orchestration 13.17M / $8.53. Stage about 40.3M / $28.68.
