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
  status: rejected                 # pending | accepted | completed | rejected

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
  status: blocked                  # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 4574961            # REAL combined count — what cost-audit reads
  estimated_usd: 3.77              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 30
  branch: claude/spec-073-build
  pr: null
  completed_at: 2026-11-16         # YYYY-MM-DD
  notes: "BLOCKED at step 1. cargo-semver-checks 0.51.0, v0.2.1 to main: 201 of 202 checks pass, one major-level failure, enum_no_repr_variant_discriminant_changed on Error (37 variants renumbered by mid-enum inserts). Shown unobservable downstream (Error has fields and no repr, so an as-cast is E0605) and the same lint fires on the shipped v0.2.0 to v0.2.1. No prep done, no version or API change. Floor from transcript 34da6ff5-811a-4f41-afac-30c5186c9017.jsonl, 31 message.ids, no subagents, opus-5-5 with 1-hour cache writes; re-sum at ship." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
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

- **Branch / PR:** `claude/spec-073-build`, a local commit with this bookkeeping only. Not pushed and no PR, because the build stopped at step 1.
- **Completed at:** 2026-11-16 (blocked)
- **All acceptance criteria met?** No. The handoff's first deliverable says any reported breaking change stops the build, and one was reported. Nothing past step 1 was started: no version bump, no CHANGELOG, no release notes, no README edit.
- **The result:**
  - `cargo-semver-checks 0.51.0`, installed with `cargo install cargo-semver-checks --locked`.
  - `--baseline-rev v0.2.1` cannot run on this repo. The clone writes a `.tmp-…` file beside a handoff whose name is already near the 255-byte limit and fails with `File name too long (os error 63)`. The run used `--baseline-root` on a `git archive v0.2.1` of the crate files instead (`Cargo.toml`, `src`, `README.md`, the licences, `tests`, `examples`). That is the same tag content.
  - The run checked 202: 201 passed, 1 failed, 0 warned, 58 skipped. The summary says "semver requires new major version: 1 major and 0 minor checks failed".
  - The failing check is `enum_no_repr_variant_discriminant_changed` on `Error`. The implicit discriminants of 37 variants shifted, from `MissingRowsPerStrip 19 -> 24` through `LjpegUnsupportedSampling 55 -> 65`, because new variants were inserted mid-enum.
- **Why I think it is a false positive (for the user to judge):**
  - `Error` has no `#[repr]`, is `#[non_exhaustive]`, and has struct variants with fields. rustc rejects `e as isize` on it with E0605 ("only if the enum type is unit-only or field-less"). I proved that with a throwaway probe crate against the v0.2.1 tree, so no dependent can observe the number. `std::mem::discriminant` is opaque.
  - The same lint fires on the already-published v0.2.0 -> v0.2.1 patch (22 `Error` variants), so 0.2.1 shipped with this exact pattern.
  - Every other check passes, including removed items, signatures, struct fields and trait impls.
- **The decision for the user:** accept it as a false positive and resume the 0.2.2 build as designed, or release as 0.3.0. Raised as `guidance/questions.yaml` `spec-073-semver-checks-flags-error-discriminants`, and the spec is set to `blocked: true`. Both tool outputs are in the session notes folder (`spec-073-semver-checks-v0.2.1-to-main.log`, `spec-073-semver-checks-v0.2.0-to-v0.2.1.log`).

### Cost self-report

- **Tokens (total):** 4,574,961 (input 61, output 13,182, cache write 121,653, cache read 4,440,065). This is a floor re-summed at handback.
- **Estimated USD:** 3.77, at opus-5-5 rates per component, with cache writes priced as 1-hour.
- **Duration (minutes):** about 30, most of it the tool's compile.
- **Source of the number:** the transcript `34da6ff5-811a-4f41-afac-30c5186c9017.jsonl`, deduped by `message.id` (31 ids, the largest output per id). There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - `--baseline-root` instead of `--baseline-rev`, for the filename-length reason above.
  - The build stopped at step 1, as the handoff requires.
- **Follow-up work identified:**
  - If the user accepts the false positive, a resumed build can record this run as the semver evidence and carry on from step 2.
  - Future releases could avoid the noise by appending new `Error` variants at the end of the enum. That would be a convention, or a DEC, not a code change here.
  - A `semver-checks` recipe could pin the `--baseline-root` workaround.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The handoff does not separate a breaking change from a tool finding that cannot be observed. The rule as written is "any reported" change, so I stopped, as instructed.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The design's audit and DEC-052 cover new variants only as "`Error` is `#[non_exhaustive]`". Neither anticipated the discriminant lint, which also fired on 0.2.1.

3. **If you did this task again, what would you do differently?**
   — Run the tool on the previous patch release (v0.2.0 -> v0.2.1) at design time, so a known false positive is either pre-accepted or decided before the build.
