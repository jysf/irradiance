---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-014                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: 2026-11-21

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Keeps the 0.2.x line honest after the 0.2.0 cut: the release's documented
    known limitations that are correctness gaps (a file can still develop without
    processing it declares as required) are closed, and the hygiene items the
    0.2.0 verify left as non-blocking are cleared, so 0.2.1 ships with fewer
    caveats than 0.2.0.
  delivers:
    - "A DNG whose OpcodeList tag is present but unreadable is refused by every develop entry instead of developing without it (DEC-053)"
    - "The Sensor-taking stage primitives (demosaic_into*, to_srgb_linear_into) refuse unapplied required processing the same way develop_* does, and composers get a public check"
    - "cargo doc builds warning-free, and the nested-cargo red-proof tests pass when CARGO_TARGET_DIR is set"
  explicitly_does_not:
    - "Implement any new opcode, or run OpcodeList stages on the colour path"
    - "Add a develop-anyway escape hatch (DEC-053 records it as deferred until a consumer asks)"
    - "Colour accuracy (SPEC-035 / STAGE-008) or any pixel-output change"

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
    - tokens_total: 18370956
      estimated_usd: 12.83
      recorded_at: 2026-11-21
      notes: "Second batch only, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's request to frame the STAGE-014 bundle at 2026-11-21T20:33Z through the SPEC-084 ship: 90 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus 5.5 list rates (input 206 + cache-creation 240,400 (1-hour) + cache-read 18,074,835 + output 55,515); no subagents. Covers re-measuring the four inherited outlines, the frame (PR 296), the SPEC-084 design (PR 297), dispatching and checking the cold build and verify, the fast-forward, the ship (PR 299) and this close. A FLOOR: the final messages after the measurement are not in it. The first batch (SPEC-056, 2026-11-11) was orchestrated in another session and is not measured here; its spec cost is on SPEC-056."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-014: 0.2.x maintenance: post-release correctness and hygiene

## What This Stage Is

The post-0.2.0 maintenance stage for PROJ-002: the small, already-measured items
that were deliberately deferred out of the 0.2.0 cut (SPEC-042) and recorded in
the CHANGELOG's Known limitations, closed and released as 0.2.1. No new
capability, no pixel change — it narrows the gap between what the API promises
and what it does.

## Why Now

0.2.0 is on crates.io with two documented correctness caveats about required
opcode processing. They are cheap, fully understood (SPEC-054 built the
machinery they extend), and the longer they sit the more consumers compose
around them. It is also the only unblocked work while SPEC-035 waits on the
ColorChecker capture.

## Success Criteria

- Every documented 0.2.0 "Known limitation" that is a correctness gap (not a
  scope limit) is either fixed or explicitly re-stated with its decision
- `just test` + the full gate suite green, CI observed green on the shipping SHA
- No still-succeeding develop changes its bytes (`DEVELOP_VERSION` unchanged)
- 0.2.1 is cuttable by the maintainer (tag + publish are the human step)

## Scope

### In scope
- SPEC-056: unreadable OpcodeList refusal, stage-primitive guard, public check,
  rustdoc private-link cleanup + a doc gate, CARGO_TARGET_DIR-proof nested-cargo tests
- The 0.2.1 version bump + CHANGELOG section (part of SPEC-056's ship, release
  itself human-gated)

### Explicitly out of scope
- Making the four STAGE-005 [XS] harness outlines block 0.2.1 (they are in the backlog, not on the critical path)
- Colour accuracy (SPEC-035, STAGE-008)
- Any new opcode / colour-path opcode pipeline

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-056 (shipped on 2026-11-11) — [M] 0.2.1 housekeeping: refuse unreadable OpcodeLists (DEC-053), guard the Sensor-taking stage primitives + public check, rustdoc private links + doc gate, nested-cargo tests honour CARGO_TARGET_DIR

- [x] SPEC-084 (shipped on 2026-11-21) — [S] Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg. PR 298; verify approved first round; 14.34M tokens / $10.80 build plus verify

**Count:** 2 shipped / 0 active / 0 pending

> **Second batch, framed 2026-11-21.** The four [XS] harness outlines moved here
> from PROJ-001 STAGE-005 at PROJ-001's close (2026-11-11), measured at git
> 3238dcb. Re-measured at `main` 2026-11-21, before framing:
> - **AC1 still holds.** `examples/corpus-status.rs` prints "no tier-B test will
>   skip" when every corpus file is present, and never checks for `exiftool` or
>   `dnglab`; without them, the tier-B oracle tests still skip.
> - **AC2 still holds.** `reading_from_fields` in `tests/support/tools.rs` keeps
>   `req`'s `.first()` and its doc comment still justifies the truncation.
> - **AC3 partly overtaken.** `tests/determinism.rs` now has 8-bit and 12-bit
>   cases, so the two depths are exercised; nothing yet fails when a depth is added
>   to `SUPPORTED_BITS` (private, in `src/plane.rs`) without a test. Re-run the
>   delete-a-depth mutant at design.
> - **AC4 dropped, obsolete.** DEC-029 made a sample above `WhiteLevel` legal, so
>   `SampleExceedsWhiteLevel` no longer exists and there is nothing for the plane
>   fuzz seeds to reach.
>
> Two PROJ-004 carry-overs join them: restoring the macOS arm64 determinism leg as
> blocking (the PROJ-004 close found the runner trigger met; the leg passed on PR
> 287), and the `sdk-stage2.sh` warning wording. One [S] spec, not five: each item is
> a few lines, and a spec cycle costs more than any one of them. No `src` behaviour
> change, no `DEVELOP_VERSION` bump, no release.

## Design Notes

- The refusal lives in the one shared check (`check_sensor_required_processing`,
  `src/opcode.rs`), so every caller — develop_* and the newly-guarded
  primitives — inherits it with no per-entry logic.
- Read stays tolerant, develop is strict: DEC-012's interpretation-class
  tolerance is unchanged for parsing/`info`; DEC-053 adds strictness only at the
  point the library claims a correct develop.

## Dependencies

### Depends on
- STAGE-010 (shipped) — SPEC-054's required-processing scan, DEC-049, DEC-051's ErrorKind
- External: none

### Enables
- The 0.2.1 crates.io cut (maintainer)

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes. The first batch
  (SPEC-056) closed 0.2.0's documented correctness gaps and was released as 0.2.1. The
  second batch (SPEC-084) cleared the inherited harness debt and two PROJ-004
  carry-overs. No pixel output changed, and `DEVELOP_VERSION` was not bumped.
- **How many specs did it actually take?** Two: SPEC-056 [M] and SPEC-084 [S]. The
  four [XS] outlines carried from PROJ-001 became one spec, and one of them was dropped.
- **How many outlines survived unchanged?** 0 of 4 as written. Re-measured on `main`
  before framing: two still held, one was half overtaken (`SUPPORTED_BITS`), and one
  was obsolete (`SampleExceedsWhiteLevel`, removed by DEC-029). They were reframed as
  one bundle, and that bundle's scope then held from frame to ship.
- **What changed between starting and shipping?** The stage was framed as 0.2.1
  maintenance and closed as a home for small hygiene work that the PROJ-004 close
  carried over.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - None codified. Re-measuring inherited outlines before framing paid off here; it is
    an application of `probe-the-hard-assumption-at-frame-time`, not a new rule.
- **Signals dispositioned at this close?** All 17 `watch` signals with
  `disposition_at: stage-close` were walked. None gained same-outcome evidence from
  this stage, so none reached its bar, and all stay `watch`.
  - `probe-the-hard-assumption-at-frame-time`: applied, not violated (the re-measure
    found the obsolete outline). It stays `watch`; a note was added.
  - `preflight-tool-lists-are-hand-written` (new at SPEC-084's ship, process-debt):
    stays `watch`, low priority.
  - `macos-arm64-runner-unavailable` was closed (`done`) by SPEC-084.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - No. SPEC-084's verify found no defect, and the two nits are recorded.
- **Cost.** SPEC-084: 14.34M / $10.80. Second-batch orchestration: 18.37M / $12.83
  (floor). Together about 32.7M / $23.63. SPEC-056's spec cost is on its own record.
