---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-023                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-18
shipped_at: 2026-11-19

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Makes the black level right for every shape DNG allows before the Pixel files start developing in STAGE-016, closes STAGE-019's test-strength follow-ups, and answers onyx request 5.
  delivers:
    - "A fractional RATIONAL BlackLevel resolves to its nearest integer instead of 0, with the exact values kept on Sensor (DEC-068)"
    - "The colour develop indexes black by BlackLevelRepeatDim and refuses a pattern whose count disagrees"
    - "Sensor::masked_areas, MaskedAreas passed through verbatim"
    - "Five STAGE-019 test hardenings, each proved by a mutant shown red"
  explicitly_does_not:
    - "Carry a fractional black through develop"
    - "Change any held file's develop output"
    - "Make the Pixel files develop (STAGE-016)"
    - "Cut a release"

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
    - tokens_total: 22710566
      estimated_usd: 16.00
      recorded_at: 2026-11-19
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'Black-level fixes' message at 2026-11-19T03:51Z through this close: 125 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 292 + cache-creation 252,589 (1-hour) + cache-read 22,366,174 + output 91,511); no subagents. Covers framing STAGE-023, the black-level design probe, DEC-068, both specs, the 17-red fixture decision, two build and two verify dispatches, two verify-base fast-forwards, the SPEC-075 ship and this ship and close. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-023: Black-level correctness, STAGE-019 test hardenings and MaskedAreas

## What This Stage Is

A correctness and hygiene stage of two specs. SPEC-075 fixes the two black-level gaps
DEC-061 §5 left open, and adds onyx's `MaskedAreas` pass-through, which reads the same
sensor IFD. A non-integral RATIONAL `BlackLevel` stops resolving to 0. The colour path
indexes black by `BlackLevelRepeatDim` as the mono path already does. SPEC-076 closes the
five `[XS]` test-strength gaps the STAGE-019 verifies named.

## Why Now

- The user chose it as the next stage after STAGE-022 (2026-11-19).
- STAGE-016 (GainMap) will make the Pixel files develop. The Pixel 7 Pro's black is
  fractional, so without SPEC-075 it would develop against black 0.
- No held file that develops today hits either black bug, so the fix can land with
  every pinned output unchanged, and that is the regression guard.

## Success Criteria

- `sensor().black_level` is `Some(1024)` on the Pixel 7 Pro, and `black_levels_exact`
  holds its four rationals.
- A colour fixture with a `[1, 2]`, `[2, 1]` or `[4, 4]` pattern develops identically to
  its `[2, 2]` expansion. A count mismatch is refused on all four colour entry points.
- `Sensor::masked_areas` reads writer-built fixtures verbatim, and costs a malformed tag
  without failing the read.
- Each STAGE-019 hardening catches its named mutant.
- Every held file's develop output is byte-identical. `DEVELOP_VERSION` is `"5"`
  (DEC-068 §4).

## Scope

### In scope
- SPEC-075 and SPEC-076.

### Explicitly out of scope
- A fractional black in develop, `BlackLevelDeltaH`/`V`, and a colour `BlackLevels`
  option.
- HANDOFF-142's optional O7/O8 tile reorder, and the slow 24-pin test.
- GainMap (STAGE-016) and a release.

## Spec Backlog

- [x] SPEC-075 (shipped on 2026-11-19) [M] — Black-level fixes and MaskedAreas (DEC-068): round a non-integral RATIONAL `BlackLevel` to nearest and keep the exact values in `Sensor::black_levels_exact`; the colour normalizer indexes `BlackLevelRepeatDim` with the DEC-061 §3 rule and refuses a count mismatch with `InvalidBlackLevelPattern`; `Sensor::masked_areas` passes through `MaskedAreas` (interpretation class, 16-rectangle cap); `DEVELOP_VERSION` 5.
- [x] SPEC-076 (shipped on 2026-11-19) [S] — STAGE-019 verify test hardenings: an unsorted-neighbour median fixture, a develop-level malformed `OpcodeList1` alloc row, a true "warp ran" guard, the scratch sentinel in `develop_err`, and the traversal-helper argument scan. Tests only, each with a mutant shown red.

**Count:** 2 shipped / 0 active / 0 pending

**Designed 2026-11-19:** the probe on 4c5dd2c read all 11 held DNGs. Only the Pixel 7 Pro has a non-integral black (`262172/256` and so on, all rounding to 1024), and no file has a `[1, 2]`/`[2, 1]` pattern, more than one `WhiteLevel` value, or `MaskedAreas`. The test writer's `extra_tags` builds every fixture with no writer change. The user chose rounding over a fractional develop.

## Design Notes

- DEC-068 holds all three SPEC-075 choices and the version bump.
- Both specs edit `tests/black_levels.rs` (SPEC-075 the version pin, SPEC-076 H3 and H4),
  and both may edit `src/develop.rs`. Build them one after the other: SPEC-075 first,
  then SPEC-076 cut from `main` after SPEC-075 merges.

## Dependencies

### Depends on
- STAGE-019 (DEC-061's pattern rule, the hardening targets) and SPEC-057's writer.

### Enables
- STAGE-016's real-file GainMap spec develops the Pixel 7 Pro with a correct black.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes, every success criterion.
  The Pixel 7 Pro reads `black_level == Some(1024)` with its four rationals in
  `black_levels_exact`. Colour `[1, 2]`, `[2, 1]` and `[4, 4]` patterns develop as their
  `[2, 2]` expansion, and a count mismatch is refused on all four colour entry points.
  `Sensor::masked_areas` reads verbatim and costs a malformed tag only. Each STAGE-019
  hardening catches its mutant. Every held file is byte-identical, and `DEVELOP_VERSION`
  is 5. SPEC-075 in PR 262, SPEC-076 in PR 264.
- **How many specs did it actually take?** 2, as planned.
- **How many outlines survived unchanged?** n/a (not framed with `just frame-stage`).
- **Cost.** SPEC-075 33.41M tokens / $23.86 (estimate 22M); SPEC-076 15.78M / $11.79
  (estimate 9M); orchestration 22.71M / $16.00 (floor). About $51.65 for the stage.
- **What changed between starting and shipping?** DEC-068's count rule refused the shape
  17 legacy colour fixtures used (4 blacks, no repeat dim), so they gained a `[2, 2]`
  repeat dim with no assertion changed. Both verifies came back as tests-only punch
  lists (four and two surviving mutants), and both fixes were fast-forwarded into the
  build PR before merge, so no cherry-pick was needed at ship.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Probe `tests/` as well as the corpus when a DEC tightens a rule: grep for fixtures
    the new rule refuses, and grep for version pins instead of listing files.
  - Give every mutant copy its own `CARGO_TARGET_DIR` and read its panic message. Both
    builds and one verify hit a false green or a wrong-reason red.
- **Signals dispositioned at this close?** No lesson signal is owned by this stage. New
  `watch` signal `every-rule-clause-maps-to-a-reachable-test` (N=2, bar N=3).
  `macos-arm64-runner-unavailable` stays watch.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - The rule-clause-to-test mapping, recorded as the watch signal above.
  - Carried, unframed `[XS]`: a non-integral RATIONAL `BlackLevel` above `i32::MAX`
    stays malformed instead of rounding; an optional larger warp fixture so H3's guard
    runs on both AC5 cases.
