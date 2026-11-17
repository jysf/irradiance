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
  id: HANDOFF-153
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-16
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-073

project:
  id: PROJ-004
  stage: STAGE-021
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
  tokens_total: 10790137           # REAL combined count — what cost-audit reads
  estimated_usd: 9.04              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 80
  branch: claude/spec-073-build
  pr: 253
  completed_at: 2026-11-16         # YYYY-MM-DD
  notes: "Blocked at step 1 on a cargo-semver-checks 0.51.0 report (enum_no_repr_variant_discriminant_changed on Error), which the user accepted as a false positive on 2026-11-16 (Accept, resume 0.2.2); then the full Phase-1 prep. Floor from transcript 34da6ff5-811a-4f41-afac-30c5186c9017.jsonl, 64 message.ids, no subagents, opus-5-5 with 1-hour cache writes, measured before the final commit; re-sum at ship." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-153: BUILD: the irradiance 0.2.2 release prep (SPEC-073, Phase 1)

## Delegation Summary

You are the **builder** for SPEC-073, the 0.2.2 release. Cut `claude/spec-073-build`
from `origin/main` after the design PR merges. **This is Phase 1 only.** That means the
reversible prep:
- a semver check, then the version bump;
- the CHANGELOG promotion and the release notes;
- the evidence and the pre-flight.

**You never tag, push a tag, or publish.** The user cuts 0.2.2 after your PR merges.

**The user decided** the version (0.2.2, a compatible patch release) and **approved
installing `cargo-semver-checks`** from crates.io for a mechanical semver check.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-073-the-next-irradiance-release-semver-check-changelog-promotion.md`.
  Read all of it:
  - Release Scope (the design's API audit);
  - Build work items 1–8;
  - Release Notes (the drafted headline);
  - Release Pre-Flight;
  - the Notes (the Phase 2 runbook and the gotchas).
- **Precedent:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md`,
  the 0.2.0 release. Mirror its filled pre-flight and its notes style.
- **Decisions:** DEC-052 (semver and the sealed types), DEC-028 (publishing), DEC-006
  (release specs and the CHANGELOG), DEC-066 (`just test` now takes about 3 min).
- **Files:** `Cargo.toml`, `CHANGELOG.md`, `README.md`, the crate docs in `src/lib.rs`,
  and `.repo-context.yaml` (scheme semver, already set).
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.

## Expected Deliverables

1. **The semver check first.**
   - `cargo install cargo-semver-checks --locked`, then
     `cargo semver-checks check-release --baseline-rev v0.2.1` (record the tool version
     and its whole output).
   - **If it reports any breaking change, stop.** Write it up in the handback and set
     the spec `blocked: true` with a question. Do not bump to 0.3.0 or "fix" the API
     yourself.
2. **The prep** (spec Build work items 2–5): the version, the CHANGELOG, the release
   notes (from `just release-notes PROJ-004`, edited into the spec), and README or
   crate-doc accuracy fixes, wording only.
3. **The evidence** (item 6): `just test` with the corpus, and the three timing
   examples.
4. **The package guards** (item 7):
   - `just package-targets-check`;
   - `just deny` and `just deny-fuzz`;
   - `cargo publish --dry-run`, whose packaged manifest must have no self
     dev-dependency.
5. **The pre-flight** (item 8): fill every item `[now]`, `[cut]` or `N/A`, with the
   evidence.
6. **Gates, all green:**
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `lint-fuzz`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cost-audit`
   - `gate-audit`
7. **PR.** Open it against `main` from `claude/spec-073-build`, referencing SPEC-073
   and HANDOFF-153. Its body carries the release notes and the semver-checks result.
   Confirm CI is green on its head.

## Out of Scope

- Tagging, pushing a tag, `cargo publish`, a GitHub release, and `cargo yank`. All of
  these are Phase 2, for the user.
- Any API change, and any version other than 0.2.2.
- Library code changes beyond doc wording.
- New features.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, **tag or publish**.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

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

- **Branch / PR:** `claude/spec-073-build`, https://github.com/jysf/irradiance/pull/253. The PR is left unmerged for the cold verify.
- **Completed at:** 2026-11-16
- **All acceptance criteria met?** Yes, after one stop. The build first stopped at step 1, with commit `f1a2884` recording the block. The user answered "Accept, resume 0.2.2", and the build then completed every step.
- **The semver check:**
  - `cargo-semver-checks 0.51.0` ran 202 checks against v0.2.1: 201 passed and 1 failed.
  - The failure is `enum_no_repr_variant_discriminant_changed` on `Error`: 37 variants' implicit discriminants shifted because new variants were inserted mid-enum.
  - I showed it cannot be observed. `Error` has fields and no `#[repr]`, so a downstream `as isize` is rustc error E0605, which a probe crate against the v0.2.1 tree confirms. The same lint fires on the published v0.2.0 -> v0.2.1.
  - The user accepted it as a false positive on 2026-11-16. The question `spec-073-semver-checks-flags-error-discriminants` is answered, and the spec's internal release record carries it.
- **Workaround:** `--baseline-rev v0.2.1` crashes with `File name too long (os error 63)`. The clone writes a `.tmp-…` file beside a handoff whose name is already near the 255-byte limit. The check ran with `--baseline-root` on a `git archive v0.2.1` of the crate files.
- **The prep:**
  - `version = "0.2.2"`; no other manifest key changed.
  - The CHANGELOG `[Unreleased]` is promoted to `## [0.2.2] - 2026-11-16`, with a short summary paragraph, and a fresh, empty `[Unreleased]` sits on top.
  - The release notes are in the spec, seeded from `just release-notes PROJ-004` and edited against the CHANGELOG. Every known limitation was checked against the code.
  - Four wording fixes:
    - the README no longer lists tiled planes as unimplemented;
    - the README `validate` comment says "strip or tile layout";
    - the README fuzz list adds `fuzz-ljpeg`;
    - the `src/lib.rs` crate doc no longer says compressed-plane decode is absent.
- **The evidence, on candidate `439ecf7`:**
  - `just test` with the corpus: 12/12 files present, 40 binaries, 710 passed, 0 failed, 1 ignored, 185 s.
  - `cargo test --doc`: 4 passed.
  - The timings confirm the CHANGELOG figures: unpack 9.9 ms on 1 thread and 1.6 ms in 8 units; ljpeg 104.4, 92.0 and 62.4 ms.
  - `cargo publish --dry-run` packages v0.2.2: 19 files, `.cargo_vcs_info.json` sha1 `439ecf7…`, no self dev-dependency.
  - `package-targets-check`, `deny` and `deny-fuzz` are green.
- **Gates, all green:** `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `msrv`, `cost-audit`, `gate-audit`.
- **No tag, no publish:** `git tag -l v0.2.2` is empty.

### Cost self-report

- **Tokens (total):** 10,790,137 (input 131, output 37,820, cache write 286,464, cache read 10,465,722). This is a floor, measured just before the final commit; the CI check after it is not counted.
- **Estimated USD:** 9.04, at opus-5-5 rates per component, with cache writes priced as 1-hour.
- **Duration (minutes):** about 80 in all. That covers both turns, including the tool install and the 3-minute test run.
- **Source of the number:** the transcript `34da6ff5-811a-4f41-afac-30c5186c9017.jsonl`, deduped by `message.id` (64 ids, the largest output per id). There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. The false-positive acceptance is the user's answer, recorded in `guidance/questions.yaml` and in the spec's internal release record.
- **Deviations from spec:**
  - `--baseline-root` instead of `--baseline-rev`, for the reason above.
  - The build stopped once, as instructed, before resuming.
  - The PR head is a docs-only commit after the dry-run candidate `439ecf7`. Phase 2's runbook re-runs the dry-run on the merged commit anyway.
- **Follow-up work identified:**
  - Append new `Error` variants at the end of the enum, as a convention or a DEC, so the lint stays quiet.
  - Add a `just semver-check` recipe that pins the `--baseline-root` workaround.
  - Shorten handoff filenames: the longest ones break any tool that writes a `.tmp` sibling.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — "Stop on any reported breaking change" doesn't separate a breaking change from a tool finding that cannot be observed. It cost one round trip to the user, which was the right call under the rule as written.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-052 covers new `Error` variants only as "`#[non_exhaustive]`". It does not address variant order, the one thing the semver tool objects to.

3. **If you did this task again, what would you do differently?**
   — Run the tool on the previous patch release (v0.2.0 -> v0.2.1) at design time, so a known false positive is decided before the build.
