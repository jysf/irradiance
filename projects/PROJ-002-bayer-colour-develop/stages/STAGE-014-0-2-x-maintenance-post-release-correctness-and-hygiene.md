---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-014                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: null

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
  sessions: []                      # - tokens_total: N
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

- [ ] SPEC-056 (design) — [M] 0.2.1 housekeeping: refuse unreadable OpcodeLists (DEC-053), guard the Sensor-taking stage primitives + public check, rustdoc private links + doc gate, nested-cargo tests honour CARGO_TARGET_DIR

- [ ] (not yet written) [XS] `corpus-status` states only what it checked — from PROJ-001 STAGE-005 (SPEC-016 pre-rescope AC1; measurements at git 3238dcb)
- [ ] (not yet written) [XS] `req()`'s single-value assumption is enforced, not documented — from STAGE-005 (SPEC-016 pre-rescope AC2; carries SPEC-005/FU-2, SPEC-010/FU-2)
- [ ] (not yet written) [XS] `SUPPORTED_BITS` cannot outrun its tests — from STAGE-005 (SPEC-016 pre-rescope AC3; carries SPEC-012/FU-1)
- [ ] (not yet written) [XS] plane fuzz seeds reach `SampleExceedsWhiteLevel` — from STAGE-005 (SPEC-016 pre-rescope AC4; carries SPEC-012/FU-2)

**Count:** 0 shipped / 1 active / 4 pending

> The four [XS] harness outlines moved here from PROJ-001 STAGE-005 at PROJ-001's
> close (2026-11-11). Their measurements are from git 3238dcb; re-measure before
> designing. They are candidates for one bundled harness spec after SPEC-056, and
> they do not gate 0.2.1.

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
