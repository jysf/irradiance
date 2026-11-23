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
  id: HANDOFF-184
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-088

project:
  id: PROJ-002
  stage: STAGE-028
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
  tokens_total: 16659440           # REAL combined count — what cost-audit reads
  estimated_usd: 11.62             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: release/0.2.5-prep
  pr: 316
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "FLOOR from this session's own transcript (f05d5494-5971-4ca8-a038-a75160f617c3.jsonl, found by the scratchpad UUID; 85 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with the largest output_tokens: input 174 + cache-creation 199,419 (1-hour) + cache-read 16,402,949 + output 56,898, opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so the last few turns are not in it; re-sum at ship. Duration is an estimate: the transcript timestamps span 21 minutes, less than the measured command times, so they are not used."
  synced_at: 2026-11-22
---

# HANDOFF-184: irradiance 0.2.5 release prep (SPEC-088, Phase 1)

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-088` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle: Phase 1 of the 0.2.5 release, reversible prep
only. You never tag or publish.

## Context the Receiving Agent Needs

Read these, in this order:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-088-irradiance-0-2-5-release-calibration-converged-weight-unhono.md`.
  The Release Scope's changed-input list and the ten build steps are the contract.
- **The model:** `projects/PROJ-004-instrument-layer-and-consumer-integration/specs/done/SPEC-083-irradiance-0-2-4-release-dng-conformant-black-normalisation.md`,
  the 0.2.4 release spec. Use its release notes, internal record and filled pre-flight
  as the shape.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-028-the-0-2-5-release.md`.
- **What is being released:** DEC-076, DEC-077 and DEC-078, and the three STAGE-027 specs
  in `specs/done/` (SPEC-085, SPEC-086, SPEC-087), with their CHANGELOG entries. Also PR
  301 (`gh pr view 301 --repo jysf/irradiance`, and `git show dfffd75`), which has no
  spec and no CHANGELOG entry.
- **`AGENTS.md`**: §15 "Cutting a release?" and the gates.

### Relevant decisions

- `DEC-067` — `just semver-check` against the crates.io baseline decides compatibility.
- `DEC-006` — the release pre-flight categories.
- `DEC-069` §6 — the SDK stays local: never committed, packaged or run in CI.
- `DEC-076`, `DEC-077`, `DEC-078` — what changed, for the notes.

### Relevant signals

- `release-notes-diff-every-changed-refusal-and-what-it-preempts` — the reason for build
  step 2: derive every changed input from the source diff, including refusals that now
  pre-empt a later error.

## Expected Deliverables

- A PR against `main` from branch `release/0.2.5-prep`, carrying build steps 1 to 10:
  - the PR 301 CHANGELOG entries;
  - the version bump and the promoted `## [0.2.5]` section;
  - the release notes and the internal record;
  - the filled pre-flight.
- Use `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  The SDK objects and `dng_validate` are under
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk`.
- Run `scripts/sdk-colour.sh` and `scripts/sdk-stage2.sh` into your scratchpad, never
  into the repository.
- **Stop and report, do not fix,** if:
  - `semver-check` reports a breaking change;
  - a held pin moves;
  - the SDK colour maxima differ from 6 / 6 / 1 / 2 / 5;
  - the stage-2 table differs from 0.2.4's;
  - the changed-input diff finds a behaviour change that no spec or PR describes.

## Out of Scope

- Any library behaviour change. The stage is release-only.
- Tagging, `cargo publish`, and the GitHub release: the user's Phase 2.
- Commit-message bodies with code tokens: the repo ruleset rejects them. Plain prose only.

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

- **Branch / PR:** `release/0.2.5-prep`, [PR 316](https://github.com/jysf/irradiance/pull/316), head `4dbecce` before this handback commit.
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** Yes. All ten build steps are done, and none of the stop conditions fired:
  - `semver-check`: no breaking change. `cargo-semver-checks 0.51.0` against crates.io 0.2.4: "202 checks: 202 pass, 58 skip … no semver update required", before and after the bump.
  - Held pins: no pin moved. `just test` ran with the corpus and both SDK lanes: 845 passed, 0 failed, 1 ignored, 12 of 12 held files. It ran on `2724fff` (233 s) and again on the PR head `4dbecce` (227 s). `src/` is byte-identical across both, which `git diff --quiet` confirmed.
  - SDK colour maxima: 6 / 6 / 1 / 2 / 5, from a fresh `sdk-colour.sh` run.
  - Stage-2 table: equal to 0.2.4's digit for digit, from a fresh `sdk-stage2.sh` run. All 7 TIFF sha256s match HANDOFF-169 and HANDOFF-171.
  - Changed-input diff: no behaviour change outside every spec and PR.
- **Gates:** all green on the candidate: lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, deny-fuzz, msrv, cost-audit, gate-audit, name-length-check, decisions-index --check, semver-check and package-targets-check. `cargo publish --dry-run` passed on a clean detached worktree of `92ad58f` and again of `4dbecce`: 19 files, and the packaged `vcs_info` sha1 is the commit. CI on PR 316 was 21 pass with 1 pending at handback.
- **The reconciliation, in short** (the full record is in SPEC-088's internal record):
  - It confirms the design's list.
  - It answers the open question: 0.2.4 did apply an optional opcode newer than 1.7.1.0, because it discarded the version and dispatched on the ID.
  - It corrects one design example. A malformed `CFAPattern` with a malformed `CFALayout` still gives `NotAColorFilterArray`, because `parse_cfa` returns before reading `CFALayout`. The CHANGELOG uses another example instead.
  - It adds seven classes, each described by its spec or PR:
    - PR 301's already-refused-different-error cases: a mandatory too-new opcode, a bad CFA with no usable matrix, and a non-finite warp under `FullActiveArea`;
    - PR 301's newly-developing case: an optional too-new opcode with unparseable parameters;
    - PR 301's output case: a later same-ID opcode is now the one applied;
    - SPEC-086/087: the buffer, options and `GainMap` errors are pre-empted by the new tag checks;
    - SPEC-085: the `ForwardMatrix1` fallback edge;
    - `malformed_tags` growth;
    - the `irr` print lines.
- **What else changed:**
  - PR 301's CHANGELOG entries were added from the PR description and diff.
  - The SPEC-085 entry's "left to SPEC-087" was corrected to DEC-078 §4.
  - The README now names 0.2.5. Wording only.

### Cost self-report

- **Tokens (total):** 16,659,440. This is a floor, measured before this handback commit.
- **Estimated USD:** 11.62
- **Duration (minutes):** 60, an estimate. The transcript's timestamp span (21 minutes) is shorter than the measured command times, so it is not used.
- **Source of the number:** the session transcript `f05d5494-5971-4ca8-a038-a75160f617c3.jsonl`, found by the scratchpad UUID. It has 85 unique message ids, all claude-opus-5-5, and no subagents. Each message id is counted once, at its largest output.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - `just release-notes PROJ-002` lists no STAGE-027 or PR 301 notes, so the notes were written from the CHANGELOG. 0.2.4 did the same.
  - The README wording fix makes the package bytes differ from the first dry run's, so the dry run was repeated on the PR head.
- **Follow-up work identified:**
  - Warp-free orientation O7 and O8 timings are 5% slower than 0.2.4 on two runs (150 against 143 ms). `src/` changed only by a once-per-develop tag check on that path. Worth one look if it persists; not investigated.
  - `just release-notes` does not pick up STAGE-027's specs, which suggests their Reflection Q5 is not in the shape the tool reads. A tooling or records fix.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The one design example that was wrong (the CFA pair) cost a few minutes. The parse short-circuit is easy to miss without reading `parse_cfa`.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The README is in the package, so a wording fix after the dry run means repeating the dry run. Worth one line in the release template's step order: README before the package guards.

3. **If you did this task again, what would you do differently?**
   — Make the README pass before the dry run, and run the timings while nothing else is building. The first ljpeg run had a 166 ms outlier.
