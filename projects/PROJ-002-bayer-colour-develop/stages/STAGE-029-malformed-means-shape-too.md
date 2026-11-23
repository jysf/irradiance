---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-029                     # stable, zero-padded, continuous across the repo
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
  advances: "closes the last silent-default hole in DEC-078: a tag that is present but empty or the wrong shape refuses the develop instead of acting as absent"  # one sentence; which part of project.value.thesis
  delivers: ["a DNG whose interpretation tag is present but empty, the wrong length, or has a zero denominator refuses with MalformedTag before writing, instead of developing with the absent-tag default"]  # user-visible capabilities this stage delivers
  explicitly_does_not: ["change any held file's output", "add an Error variant", "refuse a well-formed singular matrix (DEC-078 section 4)", "release 0.2.6 (a later release spec)"]  # what this stage is NOT trying to do

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
    - tokens_total: 15800133
      estimated_usd: 11.61
      recorded_at: 2026-11-23
      notes: "Floor, measured from the orchestrator transcript (session 38903934, claude-opus-5-5) from the pasted external review (2026-11-23T05:23Z) to 2026-11-23T13:44Z: 89 unique message.ids, deduped by message.id, no subagents, opus-5-5 rates (input 192 + cache-creation 218,828 (1-hour) + cache-read 15,514,648 + output 66,465). Covers the reproduction probe, the stage frame and SPEC-089's design (PR 318), dispatching the build and verify, the cost-sync fix on PR 319, and this ship and close. The last few ship messages were not yet flushed to the transcript when this was summed. The build and verify sessions are on SPEC-089's own rows, not here."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-029: malformed means shape too

## What This Stage Is

DEC-078 (SPEC-087, shipped in 0.2.5) made a malformed interpretation tag refuse the
develop that reads it. But its readers only call a tag malformed when they cannot read
it. A tag that reads cleanly with the wrong shape still passes as absent or valid:
a count of zero, a colour matrix with eight entries, a zero denominator. When this
stage ships, "malformed" also covers those shapes, every develop path refuses them
with the existing `Error::MalformedTag` before writing, and the colour code no longer
swallows a matrix error into a fallback.

## Why Now

A follow-up external review of 0.2.5 (at c78ec4c) reported it as two P2 findings, and
both were reproduced at design through the real parser (fixtures written by the test-only
DNG writer, read by `Container::parse` and `sensor()`):
- count-zero `Orientation`, `BlackLevel`, `WhiteLevel` and `CFALayout` give
  `malformed_tags = []` and `Ok(())`;
- `ColorMatrix2` with eight entries, or with nine zero-denominator entries, gives
  `malformed_tags = []`, `Ok(())`, and output byte-identical to `ColorMatrix2` absent,
  while a valid `ColorMatrix2` changes the output.

It is the exact gap DEC-078's Context names ("a malformed `ColorMatrix2` silently drops
to single-illuminant colour"), so 0.2.5 ships a hole in a rule it advertises. It is
small, it touches only the reader and the colour code, and it adds no `Error` variant,
so it does not collide with the PROJ-005 writer build.

## Success Criteria

- Every tag in DEC-078 section 2's sets, and every tag DEC-077 refuses, is recorded in
  `Sensor::malformed_tags` when present with a count of zero; the fixed-count tags
  also when their count is wrong; the colour rationals also on a zero denominator.
- Each such fixture refuses `MalformedTag { tag }` (or DEC-077's `UnsupportedTag` for
  its own set) on every entry whose path reads the tag, with `dst` untouched, and
  `sensor()` and `unpack_into` still succeed.
- Absent tags keep their defaults, and the K-3 III `BlackLevelRepeatDim` exception still
  holds; no held file refuses and no held output moves (re-measured on every held DNG).
- `dual_illuminant`, the interpolated ForwardMatrix branch and `resolve_calibration`
  propagate a present matrix's error instead of turning it into a fallback.

## Scope

### In scope
- The count, length and denominator rule for interpretation tags, in the reader
  (DEC-078 amendment, section 7).
- Removing the error-swallowing fallbacks in `src/color.rs`.
- Byte-level parsed-file fixtures with valid and absent controls, and a red-proof.

### Explicitly out of scope
- A new `Error` variant, or any `DEVELOP_VERSION` change.
- Changing the public `Container::scalar` (its count-zero `Ok(None)` is documented and
  kept; the sensor reader uses its own count-aware read).
- A well-formed singular matrix (DEC-078 section 4 stands: the SDK's behaviour).
- Audit ranks 8, 9, 11 and 12 (an unknown `CalibrationIlluminant` code stays a
  fallback; that is rank 8, not a shape question).
- The 0.2.6 release: a separate release spec after this one ships.

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-089 (shipped on 2026-11-23) — [M] Empty and misshapen interpretation tags refuse at develop (DEC-078 section 7)

**Count:** 1 shipped / 0 active / 0 pending

## Design Notes

- Source: the external review's two P2 findings, reproduced at design (see SPEC-089
  Context). The design probe also audited the neighbouring readers: the array readers
  (`ActiveArea`, `DefaultCropOrigin/Size`, `BlackLevelRepeatDim`) already record a wrong
  count; the holes are every read through `Container::scalar`, the `BlackLevel` count-zero
  case, the rational colour tags' length and denominators, and the count-zero signatures.
- A misshapen `ColorMatrix1`, `CameraCalibration` or `AnalogBalance` already refuses
  today, as `MissingColorCalibration` from the colour code. After this stage it refuses
  earlier as `MalformedTag`. That is a changed refusal, so the CHANGELOG and the 0.2.6
  release notes list it (signal release-notes-diff-every-changed-refusal).

## Dependencies

### Depends on
- STAGE-027 (shipped): DEC-078's sets, `MalformedTag`, and `check_malformed_tags`.

### Enables
- A 0.2.6 patch release carrying the fix.
- STAGE-008 / SPEC-035: the ColorChecker oracle never measures a silently
  single-illuminant develop.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes. PR 319 (merged as
  `51b21e8`) makes a present tag that is empty, the wrong length or zero-denominator a
  `MalformedTag` refusal on every develop path that reads it, before writing, with `dst`
  untouched. `dual_illuminant`, the ForwardMatrix branch and `resolve_calibration` no longer
  turn a present matrix's error into a fallback. No `Error` variant, `DEVELOP_VERSION` still
  8, and `irr ifd` and the outputs are unchanged on all 11 held DNGs.
- **How many specs did it actually take?** One, SPEC-089 [M], as planned; complexity_actual M.
- **How many outlines survived unchanged?** Not framed with `just frame-stage`.
- **What changed between starting and shipping?** The scope grew from "count zero" to
  "count, length and denominator" once the design audit found the misshapen colour tags.
  The build then also reversed DEC-076 §3 ("only what is used is checked"), which the
  design had not named; the verify added the DEC-076 amendment note.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Codified: `a-changed-formula-lists-every-assertion-it-moves` (N=3) as a "Moved
    assertions" paragraph under Failing Tests in `projects/_templates/spec.md`.
- **Signals dispositioned at this close?** All `watch` signals with `disposition_at:
  stage-close` were walked. One codified (above). `hostile-fixture-shadowed-by-an-earlier-guard`
  went N=1 to N=2 and stays `watch`. The codified `a-designed-mutant-must-be-able-to-fail-on-its-fixture`
  recurred (a note was added, wording unchanged). The rest gained no same-outcome evidence
  and stay `watch`.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - The verify's reflection: write your own mutants from the rule text before reading the
    build's table. Every survivor here was a clause the build's table did not list. This
    is already the cold verify's job; no new signal.
- **Cost.** SPEC-089 24.49M / $18.92 (build 14.26M, verify 10.23M, estimate 25M).
  Orchestration 15.80M / $11.61. Stage about 40.3M / $30.53.
- **Next.** The 0.2.6 release spec. Its notes must carry the changed refusals listed in
  SPEC-089's Reflection.
