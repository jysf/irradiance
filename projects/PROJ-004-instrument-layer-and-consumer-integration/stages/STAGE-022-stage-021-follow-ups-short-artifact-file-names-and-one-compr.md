---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-022                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-18
shipped_at: 2026-11-18

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Clears STAGE-021's last two follow-ups, so the repo's own files stop breaking tools that copy the tree and the Compression 7 decode stops parsing each header twice.
  delivers:
    - "Every generated spec, handoff, stage, patch and spike file named within the 60-character slug cap, with a CI gate on tracked name length and a just semver-check recipe that uses the crates.io baseline"
    - "One SOF-3 header parse per Compression 7 strip or tile: 5 allocations per chunk instead of 10, output and errors unchanged"
  explicitly_does_not:
    - "Change any decode output, error, or public item"
    - "Change the template-managed slugify or rewrite old tags"
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
    - tokens_total: 12316807
      estimated_usd: 9.65
      recorded_at: 2026-11-18
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'lets do the followups' message at 2026-11-18T23:05Z through this close at 02:58Z: 80 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 182 + cache-creation 239,896 (1-hour) + cache-read 12,027,260 + output 49,469); no subagents. Covers framing STAGE-022, the SPEC-074 design probe (semver crash repro, one-parse prototype), DEC-067, the build and verify dispatches, the verify base, this ship and close. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-022: STAGE-021 follow-ups: short artifact file names and one Compression 7 header parse

## What This Stage Is

A one-spec clean-up stage. STAGE-021 closed with three [XS] follow-ups in the PROJ-004
brief. The CI push trigger was done directly (PRs 256 and 257). This stage does the other
two in one bundled spec, SPEC-074: rename the legacy artifact files whose names break
tools that copy the repo, gate tracked name length in CI, and make each Compression 7
strip or tile parse its header once.

## Why Now

- The long names broke SPEC-073's semver check, and they will break the next release's
  `--baseline-rev` run too. Fixing them before the next release spec avoids the
  workaround.
- The double parse was found and explained in SPEC-071; the probe for this stage showed
  a one-parse version is small and keeps every plane test green.
- The user asked for the follow-ups before the next PROJ-004 stage (2026-11-18).

## Success Criteria

- No tracked generated artifact has a slug over 60 characters, and no tracked path
  component is over 150 bytes, enforced in CI.
- `cargo semver-checks` runs clean through `just semver-check`, and a git baseline of the
  renamed tree gets past the clone.
- `C7_ALLOCATIONS_PER_CHUNK` is 5, with every Compression 7 oracle and pin unedited and
  green.

## Scope

### In scope
- SPEC-074, both parts.

### Explicitly out of scope
- A `cargo-semver-checks` CI gate (open item in `docs/rust-ecosystem-and-semver-decisions.md`).
- Renaming DECs or other hand-named files.
- The next release.

## Spec Backlog

- [x] SPEC-074 (shipped on 2026-11-18) [S] — STAGE-021 follow-ups, bundled: (A) rename the 83 legacy generated artifacts whose slug exceeds the 60-character cap, rewrite references, add a 150-byte `name-length-check` CI gate and a `just semver-check` recipe on the crates.io baseline (DEC-067); (B) decode Compression 7 strips and tiles from the header already parsed for the shape check, 10 to 5 allocations per chunk.

**Count:** 1 shipped / 0 active / 0 pending (SPEC-074 19.34M tok / $13.83; orchestration 12.32M / $9.65)

**Designed 2026-11-18:** probe on `28ecd5d` reproduced the semver crash (temporary name = file name + about 45 bytes; HANDOFF-064 is 235 bytes), confirmed the crates.io baseline runs clean (202/202), and measured a throwaway one-parse build at 5 allocations per chunk with the plane tests green.

## Design Notes

- DEC-067: the renaming rule, the 150-byte gate and the crates.io semver baseline.
- Part B needs no DEC: it is a private refactor under DEC-047 (shape binding first) and
  DEC-065 (the scan reader), with errors identical by construction.

## Dependencies

### Depends on
- STAGE-021 (SPEC-071's allocation pin, SPEC-073's semver finding).

### Enables
- The next release spec runs its semver check without a workaround.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes. 83 legacy artifacts
  renamed to the slug cap with history kept, a 150-byte name-length gate in CI,
  `just semver-check` on the crates.io baseline, and one Compression 7 header parse per
  strip or tile (5 allocations per chunk, output and errors unchanged). PR 259, merged
  as 25b305a.
- **How many specs did it actually take?** 1, as planned.
- **How many outlines survived unchanged?** n/a (not framed with `just frame-stage`).
- **Cost.** SPEC-074 19.34M tokens / $13.83 (estimate 12M); orchestration 12.32M /
  $9.65 (floor). About $23.48 for the stage.
- **What went well.** The design probe settled both parts before the build: the semver
  crash came from legacy names, not the generator, and the one-parse prototype measured
  5 allocations with the plane tests green. Build and verify each ran in one turn, and
  verify had no punch list.
- **What to change.** The estimate missed the cost of the rename: 83 files and their
  references are cheap to change but expensive to read. Size rename-heavy chores by file
  count, not by lines changed.
- **Signals walked.** `push-and-pull-request-both-trigger-ci` done (PR 256);
  `macos-arm64-runner-unavailable` stays watch (the leg is non-blocking since PR 257).
  No new signals.
