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
  id: HANDOFF-197
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-095

project:
  id: PROJ-005
  stage: STAGE-032
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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 13604841             # REAL combined count — what cost-audit reads
  estimated_usd: 9.52              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 95
  branch: release/0.2.7-prep
  pr: 345
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript d03f493d-2edf-45a5-aaad-7ce96beb07f7.jsonl (the scratchpad UUID; 84 unique message.ids, all claude-opus-5-5, no subagents): input 170 + cache-creation 171,470 (1-hour) + cache-read 13,388,818 + output 44,383, deduplicated by message.id with each message's largest output_tokens, priced at 5 and 25 USD per million, cache read 0.1x, 1-hour cache writes 2x, as HANDOFF-191 did. Re-sum at ship. Semver clean both ways (202 pass, 58 skip); Error 40 bytes before and after; packaged-crate writer check green, dng_validate no warnings on all three Q2M framings."
  synced_at: 2026-11-25
---

# HANDOFF-197: irradiance 0.2.7 release: the public DNG writer

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-095` to a fresh `claude-opus-5-5`
session (acting as implementer) for the **build** cycle: Phase 1 of the 0.2.7 release, the
reversible prep PR. You never tag or publish.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-032-the-0-2-7-release-the-public-dng-writer.md`
- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md`
  (its "Build work" list is your task list, steps 1 to 11, in order).
- **The model to copy:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`,
  the 0.2.6 release: its Release Notes shape, internal record and filled Release
  Pre-Flight.
- **The previous stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
  (what shipped, and its reflection's limits).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-006`: the two-phase release and the pre-flight categories.
- `DEC-028`: irradiance publishes to crates.io, honestly scoped.
- `DEC-067`: `just semver-check` against the crates.io baseline decides the version.
- `DEC-055`, `DEC-071`: the writer lives in the crate behind the `write` feature; v1 scope.
- `DEC-080`: the two `Error` variants and their payload types exist in every build.
- `DEC-082`: the preview SubIFD (for the release notes' wording).

### Constraints that apply

- `library-not-application`: no binary ships; `irr` stays behind its dev feature.
- `no-new-top-level-deps-without-decision`: `[dependencies]` stays empty; the docs.rs
  table adds none.

### Prior related work

- `HANDOFF-187` / PR 321 / SPEC-090: the 0.2.6 prep; the same Phase 1, with the SDK lanes
  this release does not need.
- PRs 329, 332, 336, 339: the four writer builds; PR 342: the STAGE-017 close.

### Environment

- `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
- `IRRADIANCE_DNG_VALIDATE` set to the local `dng_validate` under `tools/dng_sdk` (never
  commit it or its outputs; DEC-069 §6).
- Run `just test`, not `just test --all-features` (the recipe already passes it).
- Use `/usr/bin/git` to establish branch and HEAD.

## Expected Deliverables

- Branch `release/0.2.7-prep` cut from an up-to-date `origin/main`, after step 1's check
  that no `src/` or `Cargo.toml` change has landed since `153f517`. **If one has (PR 343,
  SPEC-091, is open), stop with `[?]` and report; do not re-scope.**
- One PR against `main` carrying only: `Cargo.toml` (version and the
  `[package.metadata.docs.rs]` table), `CHANGELOG.md`, README wording, and this project's
  records (the spec's Release Notes, internal record and Pre-Flight; the timeline; this
  handoff's handback).
- The spec's `## Release Notes` and internal record written, the Pre-Flight filled
  `[now]` / `[cut]` / `N/A`, and every gate in step 11 green.
- The packaged-crate writer check (step 9) recorded with its output MD5s and
  `dng_validate` results.
- PR description referencing HANDOFF-197, SPEC-095, STAGE-032, PROJ-005 and the decisions
  above.

## Out of Scope

- Any change to `src/`, tests or fuzz targets. A defect found in prep goes in the
  handback; the stage is release-only.
- Tagging, `cargo publish`, a GitHub release: Phase 2, the user's.
- PROJ-006's SPEC-091 and the STAGE-017 close's two process-debt items.

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

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `release/0.2.7-prep`, https://github.com/jysf/irradiance/pull/345
  (cut from `origin/main` at `3f9df0b`; candidate `252360b`, the version bump; head at
  handback `f561181` plus this handback commit). Not merged, not tagged, not published.
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes. Steps 1 to 11 of SPEC-095's build work are done:
  - step 1: `git diff --stat 153f517 origin/main -- src Cargo.toml` is empty (PR 343 is not in);
  - steps 2 and 3: the reconciliation, `size_of::<Error>()` (40 at `v0.2.6`, 40 on the
    candidate with and without `write`) and both semver summaries ("v0.2.6 -> v0.2.7
    (minor change)", 202 pass, 58 skip, no semver update required, by `just
    semver-check` and by `cargo semver-checks --all-features`, tool 0.51.0) are in the
    spec's internal record;
  - steps 4 to 7: the version, the docs.rs table, the CHANGELOG promotion, the Release
    Notes and the README wording; `cargo test --doc --all-features` 5 passed, `just
    doc-check` 0 warnings;
  - step 8: `just test` on `3f9df0b` and again on the head `f561181`, both 55 results,
    923 passed, 0 failed, 1 ignored, exit 0, corpus 12/12 present. The three writer
    `dng_validate` tests ran ("dng-validate: present … will run"; each `... ok`); the SDK
    colour and stage-2 lanes printed SKIP, by design. `just fuzz-dng-write 60`: 3,597,321
    runs in 61 s, no crash;
  - step 9: the packaged crate builds with `write`, with no default features, and
    documents with `-D warnings`; the consumer wrote the Q2 Monochrom `L1021223.DNG` at
    `FullSensor` / `FullActiveArea` / `StandardCrop` with MD5s
    `26a380eb4b8cfbedcab521ba152c2349` / `3e64fa3b185517c81e660e7d5ddd16c5` /
    `fc60668e99ddee1440efe993071e641e` (same on a second run), each re-read
    sample-for-sample against the source, and `dng_validate` 1.7.1 accepted all three
    with no warning or error. Without `write` the consumer fails with `E0433: cannot find
    `dng` in `irradiance``;
  - step 10: package list = 0.2.6's 19 + the three writer files; `package-targets-check`,
    `deny`, `deny-fuzz` and `cargo publish --dry-run` (22 files, 54 ignoring warnings)
    green;
  - step 11: the Pre-Flight is filled and every listed gate is green; CI on `f561181` is
    23/23 green.

### Cost self-report

- **Tokens (total):** 13,604,841 (floor: measured before the handback commit)
- **Estimated USD:** 9.52
- **Duration (minutes):** about 95
- **Source of the number:** the session transcript, `d03f493d-2edf-45a5-aaad-7ce96beb07f7.jsonl`,
  deduplicated by `message.id` per docs/cost-tracking.md (basis: transcript). 84 unique
  messages, all `claude-opus-5-5`, no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec** (where the evidence contradicted the spec or handoff):
  - **The writer's refusal list was short by one.** The writer's first check returns the
    existing `Error::PlaneTooLarge` when `width × height` exceeds `max_pixels`. The
    design listed only `WriteInputError`'s 22 reasons and `UnsupportedOpcodeForFraming`.
    The CHANGELOG and notes now name it. Not a defect: the module docs already say so.
  - **`git diff v0.2.6 -- Cargo.toml` before the bump was not only the `write` feature.**
    It also adds `"write"` to the self dev-dependency's features. `cargo package` strips
    that dev-dependency, so the release is unaffected.
  - **The `dng_validate` path in the brief was wrong.** The binary is at
    `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`, outside
    the irradiance checkout, not under `irradiance/tools/dng_sdk`.
  - **The 22-reason count, the stale CHANGELOG count and the reader's unchanged code**
    were confirmed as designed; `Error` did not grow, so the notes carry no size note.
- **Follow-up work identified:** none new. The Phase-2 runbook in SPEC-095 stands.
  Housekeeping: the release notes' `[cut]` smoke proposes reusing the step-9 consumer's
  `StandardCrop` MD5 as the published-crate check.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only the `dng_validate` location: the brief named a path inside the irradiance
   checkout that does not exist, and I found the binary from memory notes. Otherwise the
   eleven steps were exact.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-050 (the pixel budget) is behind the `PlaneTooLarge` refusal the design
   missed, but it is the reader's long-standing rule, not a release decision.

3. **If you did this task again, what would you do differently?**
   — Bump the version before starting `just test`, not during it: the bump landed while
   the suite was in its debug alloc phase and forced a rebuild. And call the consumer
   crate's imports from the crate's real module paths (`irradiance::ifd`) on the first try.
