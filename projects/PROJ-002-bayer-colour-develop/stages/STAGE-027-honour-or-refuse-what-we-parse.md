---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-027                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-22
shipped_at: 2026-11-22

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >-
    The "correct COLOUR image" half of the thesis: a develop either honours every
    tag that changes what its pixels mean, or refuses with a typed error, so a
    file never develops with Ok and silently wrong pixels.
  delivers:
    - "CameraCalibration1/2 and AnalogBalance applied on the colour develop (the Pixel 7 Pro blue channel is corrected)"
    - "typed refusals for tags the develop cannot honour yet (LinearizationTable, DNGBackwardVersion, SampleFormat, BlackLevelDeltaH/V, row/column interleave, Compression 7 precision mismatch)"
    - "a malformed interpretation tag refuses at develop instead of silently becoming its absent-tag default"
  explicitly_does_not:
    - "apply LinearizationTable or BlackLevelDeltaH/V (refuse only; applying is later, by demand)"
    - "refuse a CFA plane on the mono develop (CFA-as-mono is by design, SPEC-066 / DEC-061)"
    - "staggered-layout (SuperCCD) demosaic"

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
    - tokens_total: 60064402
      estimated_usd: 43.88
      recorded_at: 2026-11-22
      notes: "Measured from the orchestrator transcript (session a0013105, claude-opus-5-5) from the user's request to give the audit's specs proper IDs (2026-11-22T04:03Z) through this close: 302 unique message.ids across the main transcript and its Explore subagents, deduped by message.id with the max output per message, priced per component at Opus 5.5 list rates (input 668 + cache-creation 902,510 (681,252 1-hour) + cache-read 58,912,294 + output 248,930). Covers the frame (PR 304), the SPEC-085 SDK probe and design (PR 305), the SPEC-086 and SPEC-087 designs (PRs 308, 311), dispatching and checking three cold builds and three cold verifies (two with second rounds), the fast-forwards, the three ships (PRs 307, 310, 313) and this close. A FLOOR: the final messages after the measurement are not in it. The audit itself (PR 303) ran before the window and is not counted here."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-027: honour or refuse what we parse

## What This Stage Is

Every tag the library reads, or ought to read, either changes the develop the
way the DNG specification says, or makes the develop refuse with a typed
`Error`. PR 301 fixed three cases where a parsed field's meaning was never
checked (CFALayout, CFAPlaneColor, opcode version) plus non-finite warp values.
The follow-up audit (`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md`,
PR 303) found the rest of the class. This stage closes it: one colour
correctness fix on files we hold, one batch of cheap refusals, and one
systemic rule for malformed tags.

## Why Now

The audit's top gap is live on a held file. The Pixel 7 Pro carries
`CameraCalibration1 = diag(0.8594, 1, 0.3408)`, which irradiance never reads,
so its blue channel develops uncorrected with `Ok`. The Pixel 4a and Canon 5D
Mark III carry smaller calibrations. SPEC-035's ColorChecker ΔE oracle (STAGE-008)
would measure that error rather than the develop's own, so calibration should
land before the chart is shot. The refusals are cheap and stop the next
review from finding the same class again.

## Success Criteria

- The Pixel 7 Pro, 4a and 5D Mark III colour develops apply CameraCalibration
  and AnalogBalance, moving closer to Adobe `dng_validate` than today (the
  amount is pre-registered at design), with a red-proof that forcing identity
  calibration changes the output.
- Every tag in the audit's "refuse" list produces a typed error at develop on a
  tier-A fixture; no held corpus file that develops today stops developing
  (or the spec names the file and why).
- A malformed interpretation tag that the chosen develop path consumes refuses
  at develop, and PR 301's two CFA refusals can no longer be bypassed by a
  malformed `CFALayout` or `CFAPlaneColor`.
- New `Error` variants are appended at the end of the enum, classified by
  `Error::kind()`, and counted in `tests/error_kind.rs` (DEC-051);
  `just semver-check` stays clean.

## Scope

### In scope
- Applying `CameraCalibration1/2` and `AnalogBalance` on both colour develop
  paths, with the `CameraCalibrationSignature` / `ProfileCalibrationSignature`
  rule (a DEC at design).
- Refusing `LinearizationTable`, `DNGBackwardVersion` above what we support,
  `SampleFormat` other than 1, `BlackLevelDeltaH/V`, `RowInterleaveFactor` /
  `ColumnInterleaveFactor` / `SubTileBlockSize`, and a Compression 7 sample
  precision that disagrees with `BitsPerSample`.
- Extending the DEC-053 "tolerant parse, refusing develop" rule from opcode
  lists to every interpretation tag a develop path consumes, amending DEC-012.

### Explicitly out of scope
- Applying `LinearizationTable` or `BlackLevelDeltaH/V` (later, by demand, once
  a held file carries one).
- Mono develop of a CFA plane: intended behaviour (SPEC-066, DEC-061), withdrawn
  from the audit after review.
- Staggered CFA layouts (SuperCCD): refused since PR 301; support is deferred.
- `BaselineExposure`: a rendering intent, not a pixel-meaning gap.
- The audit's ranks 8, 9, 11 and 12 (illuminant fallback, several sensor IFDs,
  degenerate finite warp and gains, DefaultScale and friends): they ride along
  with whichever spec touches the file, or wait.

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

- [x] SPEC-085 (shipped on 2026-11-22) — [L] Apply CameraCalibration and AnalogBalance on the colour develop, and run NeutralToXY to convergence (DEC-076)
- [x] SPEC-086 (shipped on 2026-11-22) — [M] Refuse tags the develop cannot honour yet (DEC-077)
- [x] SPEC-087 (shipped on 2026-11-22) — [M] Malformed interpretation tags refuse at develop (DEC-078)

**Count:** 3 shipped / 0 active / 0 pending

## Design Notes

- Source of the backlog: the audit report in PR 303, as corrected after review
  (its rank 3 / "Spec C" is withdrawn). Its §4 ranks and §3 corpus evidence are
  the starting point for each spec's Context; re-measure at design, do not cite.
- The three specs touch overlapping files (`src/ifd.rs`, `src/color.rs`,
  `src/develop.rs`) and each appends `Error` variants, so they run one at a
  time, not in parallel, to keep the enum tail and `tests/error_kind.rs`
  counts conflict-free.
- Oracle for the calibration spec: the local Adobe `dng_validate` build
  (env-gated, never in CI) applies CameraCalibration, so the existing
  SPEC-060 cross-check is the natural real-file comparison.

## Dependencies

### Depends on
- STAGE-014 / PR 301 (merged): the CFA and opcode-version refusals that the
  malformed-tag spec must not let a bad tag bypass.
- STAGE-016 (PROJ-004, shipped): the Pixel files develop at all (GainMap).

### Enables
- STAGE-008 / SPEC-035: the ColorChecker ΔE oracle measures a develop with
  calibration applied.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes. Every tag the audit found
  parsed-but-unhonoured now either changes the develop as DNG specifies or refuses with a
  typed `Error`:
  - SPEC-085 (DEC-076) applies `CameraCalibration` and `AnalogBalance` and runs
    `NeutralToXY` to convergence. The Pixel 7 Pro went from 1,160 codes off Adobe's DNG
    SDK to 6, and every held colour file is within 7 (gate 12). `DEVELOP_VERSION` 8.
  - SPEC-086 (DEC-077) refuses tags that change what the samples mean, a newer DNG
    backward version, and a lossless-JPEG precision that disagrees with `BitsPerSample`.
  - SPEC-087 (DEC-078) refuses a malformed tag that the develop path reads, so PR 301's
    CFA refusals and SPEC-085's calibration cannot be bypassed through a bad field type.
  - No held file refuses, and no held output moved except SPEC-085's intended colour
    fixes.
- **How many specs did it actually take?** Three, as framed: SPEC-085 [L], SPEC-086 [M]
  (framed S), SPEC-087 [M]. Each verify found something: SPEC-085 approved with four
  test and doc follow-ups closed before ship; SPEC-086 and SPEC-087 each took a punch-list
  round.
- **How many outlines survived unchanged?** 1 of 3 in full. SPEC-085's scope grew at
  design, when the SDK probe found a second cause (DEC-034's 3-pass weight loop, wrongly
  attributed to the SDK). SPEC-086 grew from S to M (seven tags, a version guard, a
  decode-time check). SPEC-087's scope held, and it absorbed one question from each
  sibling.
- **What changed between starting and shipping?** The stage was framed as "apply the
  calibration, then refuse the rest", and the probe turned the first spec into a fix of
  two compounding causes, one of them in an accepted decision.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Codified: the spec template's Failing Tests section gains "Distinguishing fixture"
    (signal `a-designed-mutant-must-be-able-to-fail-on-its-fixture`, N=5). It is the
    lesson every STAGE-027 verify found.
  - Applied: `probe-the-hard-assumption-at-frame-time` paid off at SPEC-085's design.
    The SDK probe overturned DEC-034's pass count.
- **Signals dispositioned at this close?** All 17 `watch` signals with
  `disposition_at: stage-close` were walked.
  - `a-designed-mutant-must-be-able-to-fail-on-its-fixture`: CODIFIED (N=5, bar N=3).
  - `probe-the-hard-assumption-at-frame-time`: applied, not violated. It stays `watch`.
  - `release-notes-diff-every-changed-refusal-and-what-it-preempts`: no release in this
    stage. It is due at the next release spec, which now has three specs of new refusals
    and a colour change to describe. It stays `watch`.
  - The other 14 gained no same-outcome evidence and stay `watch`.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - SPEC-086's "name the reader for every scalar tag in the design" (P1: the interleave
    factors read through the first-value `scalar()`). N=1; not a signal yet.
- **Left unscheduled:** the audit's ranks 8 (unknown illuminant), 9 (several sensor IFDs),
  11 (degenerate warp and gains) and 12, and the by-demand applies of
  `LinearizationTable` and `BlackLevelDelta*`.
- **Cost.**
  - Specs: SPEC-085 58.91M / $41.34, SPEC-086 27.76M / $20.58, SPEC-087 21.39M / $16.31.
    About 108.1M / $78.23 in total.
  - Orchestration: 60.06M / $43.88 (floor), 36% of the stage.
  - Stage total about 168.1M / $122.11.
