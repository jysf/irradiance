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
  id: HANDOFF-150
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-16
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-072

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
  tokens_total: 3906322            # REAL combined count — what cost-audit reads
  estimated_usd: 3.26              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: claude/spec-072-build
  pr: https://github.com/jysf/irradiance/pull/249
  completed_at: 2026-11-16         # YYYY-MM-DD
  notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (33 unique message.ids, all claude-opus-5-5, no subagents): input 70 + cache-creation 88,652 (1-hour) + cache-read 3,798,711 + output 18,889 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The handback floor was 2,603,595. PR 249 was closed, superseded by the verify PR 250."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-16
---

# HANDOFF-150: BUILD: an optimised `corpus` test profile (SPEC-072)

## Delegation Summary

You are the **builder** for SPEC-072. **Cut `claude/spec-072-build` from `origin/main`
after SPEC-071's build PR merges**: both edit AGENTS.md §6. Your orchestrator will tell
you when; if you start before that, stop and ask.

**The task:**
- **`Cargo.toml`** gains `[profile.corpus]` (`inherits = "test"`, `opt-level = 3`).
- **`just test`** runs the corpus-status example and `cargo test --all-features` under
  `--profile corpus`.
- **A new `just test-debug`** keeps today's two commands.
- **AGENTS.md §6** documents both.
- **CI changes in one comment only.**

No test is deleted, ignored or filtered, and no `src/` or `tests/` file changes. The
design measured 157 s for the full corpus suite, against about 37 min in debug, with the
same 697 passed, 0 failed, 1 ignored.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-072-a-release-profile-corpus-test-lane-signal-slow-tier-b-tests.md`.
  Read all of it: the §1 probe, §2 the change, AC1–AC6 and the Notes.
- **Decision:** `./decisions/DEC-066-just-test-runs-the-whole-suite-under-an-optimised-corpus-profile-and-ci-stays-on-debug.md`.
- **Files:**
  - `Cargo.toml`;
  - `app.just` (`test`);
  - `.github/workflows/ci.yml` (the step commented "Mirrors `just test`");
  - AGENTS.md §6;
  - the signal `slow-tier-b-tests-in-debug` in `guidance/signals.yaml`. Read it; do not
    edit it.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place.

## Expected Deliverables

1. **The change** (spec §2).
2. **AC1:**
   - both lanes green with the corpus;
   - the two `-- --list` outputs diffed, with an empty diff;
   - identical totals.
3. **AC2:** the throwaway check showing `debug_assertions` on and an overflow panicking
   under `--profile corpus`. Never commit it.
4. **AC3:** the pins and red-proofs green under `just test`, with no SKIP lines.
5. **AC4:**
   - `touch src/lib.rs`, then time a full `just test` (target ≤ 5 min wall);
   - time `just test-debug` on the same machine, run in the background;
   - record both.
6. **AC5:** `git diff origin/main -- .github/` touches comments only.
7. **Gates, all green:**
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
8. **PR.** Open it against `main` from `claude/spec-072-build`, referencing SPEC-072,
   DEC-066 and HANDOFF-150. Confirm CI is green on its head with the same job set.

## Out of Scope

- Any `src/` or `tests/` change.
- CI command changes.
- `[profile.test]` or `[profile.release]`.
- Speeding up individual tests.
- CHANGELOG.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
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

### Execution notes

- **Branch / PR:** `claude/spec-072-build` → https://github.com/jysf/irradiance/pull/249
  - 1332da7 is the build.
  - The next commit is this handback plus the timeline build line.
- **Completed at:** 2026-11-16
- **All acceptance criteria met?** Yes. The machine is an Apple M4 Pro with the corpus set
  and all 12 files present.
  - **AC1:**
    - `just test` and `just test-debug` both pass with 697 passed, 0 failed and 1 ignored
      across 37 binaries.
    - `-- --list` prints 771 lines (698 tests) under both profiles, and the diff between
      them is empty.
  - **AC2:** a throwaway test in a scratch copy of the repo, run once and deleted. Under
    `--profile corpus`:
    - `cfg!(debug_assertions)` is true;
    - `black_box(u8::MAX) + 1` panics with "attempt to add with overflow", and
      `catch_unwind` returns Err;
    - `debug_assert!(false)` fires.

    As a control, the same test under `--release` fails, with
    `cfg!(debug_assertions) = false`.
  - **AC3:** every dnglab checksum, develop MD5 and pin test, and every red-proof, is ok
    under `just test`, with 0 SKIP lines.
  - **AC4:**
    - `just test` took 144.7 s wall after `touch src/lib.rs`, against the ≤ 5 min target.
    - In a fresh worktree with no `target/` at all, the first `just test` took 159.1 s.
    - `just test-debug` took 2290.3 s (38.2 min) on the same machine after a touch.
    - The debug run overlapped for about a minute with the AC2 scratch compile, so its
      time is a slight overestimate.
  - **AC5:** `git diff origin/main -- .github/` touches comment lines only.
  - **AC6:** all 10 gates are green: lint-ci, lint-no-allow, lint-fuzz, fmt-check,
    validate, doc-check, deny, msrv, cost-audit and gate-audit.

### Cost self-report

- **Tokens (total):** 2,603,595. This is a floor, measured before the handback commit.
- **Estimated USD:** 2.35
- **Duration (minutes):** about 75, most of it the 38-minute debug run.
- **Source of the number:** the transcript
  (ea3ba583-3cb3-47ff-bed0-978b23ac4035.jsonl), deduped by message.id, priced at
  opus-5-5 rates.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The ci.yml comment keeps the original "Tier B is ABSENT…" sentence, prefixed by
    three new lines. There is no `run:` change.
  - Debug info is left as `test` sets it, so nothing was measured or lowered.
  - A grep of AGENTS.md found no sentence quoting the debug suite's duration, so the §6
    block and its prose history are the only places that now name the times.
- **Follow-up work identified:**
  - The signal `slow-tier-b-tests-in-debug` should be marked resolved at ship (not
    edited here, per the spec).
  - Handoff boilerplate that says to run the full suite in the background should quote
    about 2.5 min from now on (DEC-066 Consequences).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The branch name `claude/spec-072-build` differs from the worktree's
   local branch, so I pushed to the named remote branch.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The AC2 check went into a scratch copy of the repo with its own target dir, so
   the throwaway file never touched the worktree. That was a choice, not a stated rule.

3. **If you did this task again, what would you do differently?**
   — Run the debug lane first and do nothing that compiles while it runs, so its time
   has no overlap with other work.
