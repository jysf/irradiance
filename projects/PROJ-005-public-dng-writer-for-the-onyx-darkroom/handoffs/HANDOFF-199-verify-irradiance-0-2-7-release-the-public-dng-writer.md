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
  id: HANDOFF-199
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-199: VERIFY: irradiance 0.2.7 release, the public DNG writer (SPEC-095)

## Delegation Summary

You are the **cold verifier** for SPEC-095's Phase 1, the reversible prep of the 0.2.7
release. You did not build it. Nothing is tagged or published, and you must not tag or
publish either.

- **Under review:** PR 345 (`release/0.2.7-prep`, head `cdf42ed`), built under HANDOFF-197.
  The candidate is `252360b` (the version bump); the commits after it are records only.
  CI was 23 of 23 green on `f561181`; the run on `cdf42ed` (handback only) was in progress
  at dispatch.
- **Your branch:** `claude/spec-095-verify-base`, which is that head plus one commit adding
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-095
  verify`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-095-irradiance-0-2-7-release-the-public-dng-writer.md`.
  Judge it against the spec **as designed** (`git show 3f9df0b:<spec path>`): Release
  Scope, Build work steps 1 to 11, and the Phase 2 runbook. The build filled the Release
  Notes, the internal record and the Pre-Flight; those are what you review.
- **The model:** SPEC-090 (the 0.2.6 release, `projects/PROJ-002-bayer-colour-develop/specs/done/`)
  and its cold verify HANDOFF-188, which found missed changed inputs.
- **Decisions:** DEC-006, DEC-028, DEC-067, DEC-055, DEC-071, DEC-080, DEC-082, DEC-050
  (the pixel budget behind `PlaneTooLarge`).
- **Constraints:** `library-not-application`, `no-new-top-level-deps-without-decision`.
- **Build handback:** HANDOFF-197's `## Handback`, with three deviations.
- **Toolchain:**
  - `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`;
  - `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
    (outside the irradiance checkout; never commit it or its outputs);
  - run `just test`, never `just test --all-features`; use `/usr/bin/git` for branch and
    HEAD; scratch crates and unpacked packages go in your scratchpad.

## What to check hard

1. **The cut.** PR 343 (SPEC-091) is not in: `git diff --stat 153f517 252360b -- src
   Cargo.toml` shows only the version and the docs.rs table. Confirm PR 343 is still
   unmerged and say so.
2. **The changed-input list, re-derived yourself** from `git diff v0.2.6 252360b -- src/`:
   is any reader, develop, colour or `irr` line changed? Does anything outside the writer
   return the two new `Error` variants? Rule on the build's `PlaneTooLarge` addition: is
   the writer's budget check really the existing variant with its existing kind, and are
   the notes right about it? Is any other writer refusal missing from the CHANGELOG or the
   notes (count them against `WriteInputError` and every `Err(` in `src/dng/write.rs`)?
3. **Semver,** re-run: `just semver-check` and `cargo semver-checks --all-features`, after
   the bump. Both must say no update required.
4. **The packaged crate, independently.** Run `cargo package` yourself on a clean tree of
   the candidate. Then:
   - the packaged `Cargo.toml` keeps `[package.metadata.docs.rs] features = ["write"]`
     after cargo's normalisation, and has no self dev-dependency;
   - in the unpacked package, build and doc as docs.rs would (`RUSTDOCFLAGS='-D warnings'
     cargo doc --no-deps --features write`) and confirm `dng::write` is in the output;
   - your own consumer crate, written cold, writes a **different** held monochrome DNG
     than the build used (it used the Q2 Monochrom `L1021223.DNG`) at all three framings
     with the EXIF block, XMP, descriptive tags and a JPEG preview set where the file
     gives you them. Re-read each output, compare samples, and run `dng_validate` on it.
     Determinism: two runs give identical bytes;
   - without `write` the consumer cannot name `irradiance::dng`.
   - The package list is 0.2.6's 19 files plus exactly the three writer files.
5. **Release Notes, line by line.** Check every claim against the code and the tests:
   the headline ("same input, same bytes", "no allocation in `write_into`", "no new
   dependency", "dng_validate accepts"), each Added item, "Breaking / action required:
   none", and every Known limitation, writer and reader. Flag anything overstated or
   missing, as HANDOFF-188 did for 0.2.6.
6. **CHANGELOG.** `[Unreleased]` promoted to `## [0.2.7] - <date>` with a lead paragraph
   and a fresh empty `[Unreleased]`; the stale reason count corrected; every entry true of
   the shipped code.
7. **README.** The writer section matches the shipped API: options from SPEC-081 and
   SPEC-082, SPEC-094's refusal. If its example is not compiled anywhere, compile it
   yourself in the consumer crate.
8. **The Pre-Flight and the Phase 2 runbook.** Every `[now]` has evidence you could
   reproduce; every `[cut]` is something the user can actually run. The runbook uses an
   annotated tag and the `ls-remote` `^{}` check, says to confirm PR 343 is unmerged, and
   its `[cut]` smoke on `irradiance = "=0.2.7"` with `write` would tell 0.2.7 from 0.2.6.
9. **The build's three deviations.** Rule on each: `PlaneTooLarge` (item 2); `write` added
   to the self dev-dependency's features (stripped by `cargo package`? confirm in the
   packaged manifest); the wrong `dng_validate` path in the brief (records only).
10. **Gates.** Read `gh pr checks 345` first. Then run `just test` with the corpus and
    `dng_validate` set, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `lint-fuzz-red-proof`,
    `fmt-check`, `validate`, `doc-check`, `deny`, `deny-fuzz`, `msrv`, `cost-audit`,
    `gate-audit`, `name-length-check`, `decisions-index --check`,
    `package-targets-check`, and `cargo publish --dry-run` on a clean tree.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED, with
  evidence for every item above.
- **Fixes on your branch:** wording in the CHANGELOG, README, Release Notes, Pre-Flight
  and records only. A change to `src/`, tests, fuzz, `.github/` or any `Cargo.toml` line
  goes in a punch list.
- Commit this handoff's filled handback and any fixes to `claude/spec-095-verify-base`,
  and push it. If you approve:
  1. Run `just advance-cycle SPEC-095 ship --verdict approved`.
  2. Run `just handback-sync SPEC-095` in the same push, so CI's cost audit passes.
  3. Fast-forward `release/0.2.7-prep` to the verify tip, so PR 345 carries the verify
     commits. Do not merge.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** compute any next-free DEC or HANDOFF from `origin/main` plus the open PRs.

## Out of Scope

- Tagging, `cargo publish`, a GitHub release: Phase 2 is the user's.
- PROJ-006's SPEC-091 (PR 343), and the STAGE-017 close's two process-debt items.
- Any library behaviour change.

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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
