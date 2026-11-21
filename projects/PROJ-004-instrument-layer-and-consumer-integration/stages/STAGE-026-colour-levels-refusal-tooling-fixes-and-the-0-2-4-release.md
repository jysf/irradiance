---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-026                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-20
shipped_at: 2026-11-21

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Puts STAGE-025's DNG-conformant black normalisation on crates.io, and makes the colour develop refuse malformed levels the way mono already does, so consumers get correct output and a typed error instead of a silently black image.
  delivers:
    - "The four colour develop entries refuse WhiteLevel at or below the maximum applied black with Error::InvalidLevels, before writing anything"
    - "irradiance 0.2.4 (or the version the semver check requires) on crates.io, carrying DEC-070 and the colour refusal"
  explicitly_does_not:
    - "The public DNG writer (STAGE-017): being discussed separately"
    - "crustyimg integration (STAGE-004): gated on crustyimg's STAGE-056"
    - "Any other output change: every file that develops today develops byte-identically"

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
    - tokens_total: 65062260
      estimated_usd: 40.49
      recorded_at: 2026-11-21
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'Close after merge' message at 2026-11-20T18:29Z (the PROJ-004 close attempt that became this stage) through this close: 161 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 384 + cache-creation 563,721 (1-hour) + cache-read 64,392,159 + output 105,996); no subagents. Covers the PROJ-004 close dry run and the finish-or-move discussion, the DNG-writer discussion and the move of STAGE-017 to a new PROJ-005, framing STAGE-026, designing SPEC-080 and SPEC-083, four handoffs, two builds and two verifies dispatched, three verify-base fast-forwards, two ships, the misplaced-tag recovery, the clean-host smoke on the published 0.2.4, and this close. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-026: Colour levels refusal, tooling fixes and the 0.2.4 release

## What This Stage Is

The release that takes STAGE-025 to crates.io, with the one behaviour gap its verify
found closed first. When it ships:
- the four colour develop entries (`develop_color_into`, `develop_color_f32_into` and both
  threaded entries) refuse a plane whose `WhiteLevel` is at or below the maximum black the
  path applies, with `Error::InvalidLevels`, as mono `BlackLevels::PerPosition` already does.
  Today they develop such a file silently to 0;
- two small tooling gaps from SPEC-079 are closed;
- irradiance 0.2.4 (or whatever the semver check requires) is on crates.io with DEC-070's
  normalisation and the colour refusal.

## Why Now

`main` carries a behaviour change that crates.io does not: DEC-070 (`DEVELOP_VERSION` 7)
changes the Canon 5D Mark III, Pixel 3a and Pixel 4a colour and `PerPosition` output, and a
consumer on 0.2.3 still gets the old normalisation. FU-1 is a behaviour change too, so
releasing both together means one notice and one version bump. The tooling fixes are small
and would otherwise be carried out of PROJ-004.

## Success Criteria

- On every colour entry, `W ≤ B` returns `Error::InvalidLevels` before any output is
  written; the refusal is checked against `B`, the maximum applied black, as DEC-070 §3
  defines. DEC-070 §3's SPEC-078 amendment is superseded by a second amendment.
- Every file that develops today develops byte-identically: no pin moves, and
  `DEVELOP_VERSION` stays 7 unless the design shows an input that changes output.
- `scripts/sdk-stage2.sh` finds the repository through git, so a copy run from another
  worktree is judged correctly.
- `just validate` parses `feedback/` front matter.
- `just semver-check` against crates.io 0.2.3 decides the version; the release notes name
  DEC-070's output change and the new colour refusal; the user tags and publishes.

## Scope

### In scope
- FU-1, the colour levels refusal, and its DEC-070 amendment.
- The two SPEC-079 tooling follow-ups.
- The release, as a release spec minted with `just new-release-spec` at its design.

### Explicitly out of scope
- STAGE-017, the public DNG writer. Under discussion with the maintainer; onyx may want to
  write DNGs.
- STAGE-004, crustyimg integration.
- `BlackLevelDeltaH/V` (DEC-070 §5).
- Any change to output for a file that develops today.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-080 (shipped on 2026-11-20) [S] Colour levels refusal (FU-1), designed 2026-11-20: the four colour entries refuse W ≤ B with InvalidLevels reporting B, after the pattern check and before any write (DEC-070 §3, second amendment); no output change, DEVELOP_VERSION stays 7; sdk-stage2.sh refuses an output dir inside any worktree (via git); just validate parses feedback/ front matter and checks status

- [x] SPEC-083 (shipped on 2026-11-21) [S] irradiance 0.2.4 release, designed 2026-11-21: semver clean against crates.io 0.2.3 (202 pass, no update required); one newly refused input (colour W ≤ B) and one changed error field (PerPosition reports the largest black); DEVELOP_VERSION 6 → 7; the two SPEC-080 verify script fixes; two-phase cut, the user tags and publishes

**Count:** 2 shipped / 0 active / 0 pending

## Design Notes

- **Carried from SPEC-080's verify (2026-11-20), for the release spec.**
  - `scripts/validate.sh`: strip a trailing CR and surrounding quotes from a feedback
    `status` before the vocabulary check, and treat a leading UTF-8 BOM as part of the
    `---` test (or refuse it loudly).
  - `scripts/sdk-stage2.sh`: a worktree directory that exists but cannot be entered makes
    the script exit under `set -e` with no message; add a fallback that says why.
  - Release-notes wording: colour now refuses what mono `PerPosition` refuses. Mono's
    default `FirstValue` checks its first black, not the maximum, so don't claim the two
    are identical.
- **IDs.** PR 284 (PROJ-005's STAGE-017 re-frame) holds SPEC-081, SPEC-082 and DEC-071 to
  DEC-073. Mint the release spec after checking open PRs, so it does not collide.
- **Order.** The refusal spec ships first; the release spec follows it, so the release
  carries both behaviour changes.
- **Semver.** 0.2.3 was a patch under DEC-052's precedent for behaviour fixes. A new refusal
  on an existing error variant is the same shape as 0.2.3's `InvalidBlackLevelPattern`; the
  semver check and the release design decide, not this frame.
- **Diff refusal paths against the last tag.** STAGE-024's lesson: semver-checks cannot see a
  newly refused input, so the release notes must list every input 0.2.3 developed and 0.2.4
  refuses.

## Dependencies

### Depends on
- STAGE-025 (DEC-070, SPEC-078, SPEC-079), shipped.
- External: the user's crates.io token for the publish.

### Enables
- Consumers (monorail/onyx, crustyimg) get DNG-conformant black normalisation from a registry
  release.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes.
  - SPEC-080 (PR 283): the four colour develop entries refuse `W ≤ B` with
    `InvalidLevels`, as mono `PerPosition` does; no output changed; `just validate` parses
    `feedback/`; `sdk-stage2.sh` refuses any git worktree.
  - SPEC-083 (PR 287): **irradiance 0.2.4 published to crates.io on 2026-11-21**, tag
    `v0.2.4` on `ef37e6b`, docs.rs green, clean-host smoke passed. DEC-070's
    DNG-conformant black normalisation and the colour refusal are now on the registry.
- **How many specs did it actually take?** 2, as planned (one outline plus the release spec
  minted after it).
- **How many outlines survived unchanged?** 1 of 1. SPEC-080's scope held.
- **Cost.** SPEC-080 27.10M tokens / $19.10 (estimate 15M, actual M); SPEC-083 26.13M /
  $18.70 (estimate 15M, actual M); orchestration 65.06M / $40.49 (floor, and it includes the
  PROJ-005 writer move). About $78.29 for the stage.
- **What changed between starting and shipping?** The stage began as a PROJ-004 close that
  `close-project` refused; the user chose to finish PROJ-004's work and moved the public DNG
  writer to a new PROJ-005 once onyx named Darkroom v1 as its consumer. The release design
  missed one changed result (the colour error precedence), caught by the verify; the first
  tag landed on the pre-merge commit and was replaced before anything was published.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Codified: the release-spec template's Phase 2 text now says to confirm the checked-out
    manifest shows the new version before tagging.
  - Watch: a release design diffs every changed refusal, including the errors a new refusal
    now pre-empts (N=2).
- **Signals dispositioned at this close?**
  - New `watch` at N=2: `release-notes-diff-every-changed-refusal-and-what-it-preempts`
    (0.2.3, 0.2.4).
  - `a-changed-formula-lists-every-assertion-it-moves`: N=2, no new instance (SPEC-080's
    throwaway-refusal sweep found only the pre-approved test). Left `watch`.
  - `a-designed-mutant-must-be-able-to-fail-on-its-fixture`: N=1. SPEC-080's AC3 fixture
    ambiguity is a near relative (a fixture that could not reach the path it tested) but
    not the same outcome, so not counted. Left `watch`.
  - `validate-does-not-parse-feedback-or-docs`: resolved by SPEC-080 for `feedback/`;
    `docs/` has no front matter to parse. Disposition at the PROJ-004 close.
  - The other open `watch` lessons were walked; none has a new instance.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - No; both are recorded above.
- **Carried to the next project:** the `sdk-stage2.sh` warning wording nit (SPEC-083 verify
  FU-3).
