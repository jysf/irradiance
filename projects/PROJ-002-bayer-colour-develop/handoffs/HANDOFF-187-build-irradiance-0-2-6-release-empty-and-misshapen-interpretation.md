---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-187
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-23
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-090

project:
  id: PROJ-002
  stage: STAGE-030
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model)
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 12869053           # REAL combined count — what cost-audit reads
  estimated_usd: 9.37              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 30
  branch: release/0.2.6-prep
  pr: 322
  completed_at: 2026-11-23         # YYYY-MM-DD
  notes: "FLOOR from this session's own transcript (f6e5f73d-413a-4acd-bfe6-b8fd9a9cf071.jsonl, found by the scratchpad UUID; 75 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with the largest output_tokens: input 156 + cache-creation 168,265 (1-hour) + cache-read 12,646,102 + output 54,530, opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so the last turns are not in it; re-sum at ship. Duration is an estimate: the transcript timestamps span 20 minutes with long background runs overlapping."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-187: irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-090` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle: Phase 1 of the 0.2.6 release, reversible prep
only. You never tag or publish.

## Context the Receiving Agent Needs

Read these, in this order:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`.
  The Release Scope's changed-input list (with its two **confirm** items) and the nine
  build steps are the contract.
- **The model:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-088-irradiance-0-2-5-release-calibration-converged-weight-unhono.md`,
  the 0.2.5 release spec. Use its release notes, internal record and filled pre-flight
  as the shape.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-030-the-0-2-6-release.md`.
- **What is being released:** SPEC-089 in `specs/done/` (and its build and verify
  handbacks, HANDOFF-185 and HANDOFF-186), DEC-078 §7, and the `[Unreleased]` CHANGELOG
  entry. `git diff v0.2.5 origin/main -- src/` touches only `src/ifd.rs` and
  `src/color.rs`.
- **`AGENTS.md`**: §15 "Cutting a release?" and the gates.

### Decisions that apply

- `DEC-067` — `just semver-check` against the crates.io baseline decides compatibility.
- `DEC-006` — the release pre-flight categories.
- `DEC-069` §6 — the SDK stays local: never committed, packaged or run in CI.
- `DEC-078` §7 (and the DEC-076 §3 amendment note) — what changed, for the notes.

### Constraints that apply

- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

### Prior related work

- `HANDOFF-184` — the 0.2.5 prep, the same nine-step shape.
- `PR #319` — SPEC-089's build and verify.

## Expected Deliverables

- A PR against `main` from branch `release/0.2.6-prep`, carrying build steps 1 to 9:
  - the changed-input reconciliation, with both **confirm** items answered;
  - the version bump and the promoted `## [0.2.6]` section, with any CHANGELOG wording
    the reconciliation contradicts corrected;
  - the release notes and the internal record;
  - the filled pre-flight.
- Use `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  The SDK objects and `dng_validate` are under
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk`.
- Run `scripts/sdk-colour.sh` and `scripts/sdk-stage2.sh` into your scratchpad, never
  into the repository. Build the `v0.2.5` `irr` for the `irr ifd` diff in a scratch
  worktree of the tag.
- **Stop and report, do not fix,** if:
  - `semver-check` reports a breaking change;
  - a held pin moves, or `irr ifd` differs from `v0.2.5` on a held file;
  - the SDK colour maxima differ from 6 / 6 / 1 / 2 / 5;
  - the stage-2 table differs from 0.2.5's;
  - the changed-input diff finds a behaviour change that SPEC-089 does not describe.

## Out of Scope

- Any library behaviour change. The stage is release-only.
- Tagging, `cargo publish`, and the GitHub release: the user's Phase 2.
- The cold verify: a separate handoff after your PR is up.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

### Execution notes

- **Branch / PR:** `release/0.2.6-prep`, https://github.com/jysf/irradiance/pull/322
  (candidate `857959c`, the version bump; later commits are records only).
- **Completed at:** 2026-11-23
- **All acceptance criteria met?** Yes. All nine build steps are done, and no stop condition fired:
  - semver is clean (202 of 202, no update required);
  - no pin moved, and `irr ifd` is identical to `v0.2.5` on all 11 held DNGs;
  - the SDK colour maxima are 6 / 6 / 1 / 2 / 5, with every delta equal;
  - the stage-2 table equals 0.2.5's;
  - the changed-input diff found no behaviour outside SPEC-089.

  `just test` passed with the corpus and both SDK lanes, 859 passed, 0 failed, 1 ignored, on `c025e67` and on the PR head. Every gate in step 9 is green, and so is `cargo publish --dry-run` on a clean worktree.

### Cost self-report

- **Tokens (total):** 12,869,053 (a floor; measured before this commit)
- **Estimated USD:** 9.37
- **Duration (minutes):** about 30 (estimate)
- **Source of the number:** this session's transcript, deduped by `message.id` per
  docs/cost-tracking.md (basis: transcript)

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **The CHANGELOG's SPEC-089 entry is corrected beyond the confirm items.** The
    design's and the CHANGELOG's "changed refusals" bucket was too broad. In 0.2.5 a
    misshapen `ColorMatrix1` beside a well-formed `ForwardMatrix1` developed `Ok`
    (single-illuminant ForwardMatrix path). So did a misshapen calibration that was not
    applied (signatures differ, or `CameraCalibration2` off the dual path), and a bad
    `ForwardMatrix1` on the dual path (ColorMatrix path). I probed this against both
    versions on the Canon's parsed `Sensor`. These are now listed as new refusals.
    Only `AnalogBalance` and a misshapen matrix or calibration that 0.2.5 actually used
    are "changed". I judged this a wording correction under step 4, not the
    "behaviour SPEC-089 does not describe" stop, because DEC-078 §7 specifies the new
    behaviour for every one of these tags.
  - Known limitations gain one line: an unknown `CalibrationIlluminant` code still
    falls back to single-illuminant colour. It was already true, but without it the
    headline's "no silent fallback" would overclaim.
  - The first `orient-timing` run was contended (load 7 to 12 from another session).
    I recorded a back-to-back `v0.2.5`/candidate A/B instead of a bare comparison with
    0.2.5's old figures.
- **Follow-up work identified:**
  - `v0.2.5` on origin is a **lightweight** tag (v0.2.0 to v0.2.4 are annotated), so
    `git describe` skips it. The pre-flight `[cut]` now checks
    `git cat-file -t v0.2.6`. Whether to re-push `v0.2.5` as annotated is the user's
    call.
  - Consider making the unknown-`CalibrationIlluminant` fallback a refusal (SPEC-089's
    audit rank 8, left out of scope).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The design's changed-input list classified "what 0.2.5 did" from SPEC-089's probe
   fixtures, which had no `ForwardMatrix1`. Reading `cam_to_xyz_single_illuminant` at
   `v0.2.5` was what exposed it. The spec could say outright that "was X" claims need
   the old code path read per tag, not inherited from the feature spec's probes.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `rtk` hook mangles shell variables that hold long paths in some commands (an
   empty `$L` and odd `tar` errors). Literal paths or `/usr/bin/…` avoid it. Also,
   timing examples on a shared machine need an A/B at the old tag, not a comparison
   with the last release's recorded numbers.

3. **If you did this task again, what would you do differently?**
   — Build the two-version probe crate first. One scratch crate on path dependencies
   answered the confirm items and the "was" column in minutes, and the same crate is
   the `[cut]` probe the user will run.
