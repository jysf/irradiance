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
  id: HANDOFF-152
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model)
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 5607703            # FLOOR at handback, 42 message.ids deduped, no subagents
  estimated_usd: 4.22              # opus-5-5: in 89, out 19045, cache_read 5488650, 1-hour cache_write 99919
  duration_minutes: 50
  branch: claude/spec-072-verify-5b9684
  pr: 250
  completed_at: 2026-11-16
  notes: "Punch list fixed by the verifier in PR 250, which supersets PR 249. (1) CI red on 249: the profile was appended after dev-dependencies and deny-red-proof.sh requires that table last; moved above dependencies. (2) Under opt-level 3 an unused vec allocation is elided, so the *_alloc zero-count tests pass vacuously for that shape; documented as debug-lane-authoritative in Cargo.toml, app.just, AGENTS.md and a DEC-066 amendment. Floor from transcript 1f696876-fbe0-4f33-9c9c-15b98d7be19e.jsonl; re-sum at ship."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-152: VERIFY: an optimised `corpus` test profile (SPEC-072)

## Delegation Summary

You are the **cold verifier** for SPEC-072. You did **not** implement it. Review it
against SPEC-072 and DEC-066, and return exactly one verdict.

- **The build:** **PR #249**, branch `claude/spec-072-build`, cut from `origin/main` at
  `d49afd9`. Its head is `0754a67`: the build commit `1332da7`, then the handback.
- **Your branch:** your worktree is cut from `claude/spec-072-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`.

The build changes four files:
- **`Cargo.toml`:** `[profile.corpus]`, with `inherits = "test"` and `opt-level = 3`.
- **`app.just`:** `test` now uses `--profile corpus`, and a new `test-debug` keeps the old
  commands.
- **AGENTS.md §6:** both lanes are documented.
- **`.github/workflows/ci.yml`:** comment lines only.

No `src/` or `tests/` file changes.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-072-a-release-profile-corpus-test-lane-signal-slow-tier-b-tests.md`.
  Read all of it: the §1 probe, §2 the change, AC1–AC6, the Non-Goals and the Notes.
- **Decision:** `./decisions/DEC-066-just-test-runs-the-whole-suite-under-an-optimised-corpus-profile-and-ci-stays-on-debug.md`.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-150-build-a-release-profile-corpus-test-lane-signal-slow-tier-b-tests.md`.
  Treat its claims as claims. The build reports:
  - `just test` at 144.7 s (159.1 s from an empty `target/`);
  - `just test-debug` at 38.2 min;
  - both at 697 passed, 0 failed, 1 ignored;
  - an identical `-- --list` output (698 tests);
  - assertions and overflow checks on under `corpus`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.

## The verify contract

### What to reproduce yourself
- **AC1:**
  - Run `just test` with the corpus.
  - Diff `cargo test --all-features --profile corpus -- --list` against
    `cargo test --all-features -- --list` yourself.
  - Run `just test-debug` **once** in the background (about 38 min). Confirm the totals
    match.
- **AC2:** your own throwaway check, never committed:
  - `cfg!(debug_assertions)` is true under `--profile corpus`;
  - an arithmetic overflow panics there;
  - as a control, the same check under `--release` shows they are off.
- **AC3:** every pin and red-proof is ok under `just test`, with no SKIP lines.
- **AC4:** `touch src/lib.rs`, then time `just test` (target ≤ 5 min).
- **AC5:** `git diff d49afd9 -- .github/` touches only comment lines, and no `run:` line
  changes. `gh pr checks 249` shows the same job set green.
- **AC6:** the 10 gates:
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

### Points to judge
- **Is opt-level 3 really safe as the default lane?** Read every test that could depend
  on unoptimised behaviour. Check for:
  - timing assertions;
  - stack-depth or recursion limits;
  - `#[inline(never)]` probes;
  - allocation counts that could differ under optimisation.

  `plane_units_alloc`, `develop_scratch_alloc` and `previews_alloc` count allocations.
  Confirm that they pass under both profiles, and say whether their zero counts could
  pass vacuously when optimised.
- **The red-proof tests** stage mutated crate copies and build them. Which profile do
  they build in, and does the `corpus` profile change what they prove?
- **AGENTS.md §6:** both lanes are runnable as written. Spot-check that `just test-debug`
  is exactly the old two commands. Check any stale "about 25/37 minutes" guidance
  elsewhere in AGENTS.md.
- **`Cargo.toml`:** the comment states why the profile inherits `test` and not `release`.
  `cargo publish --dry-run` is unaffected (profiles are ignored for dependents). Say
  whether you checked.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #249.
  - Commit the filled HANDOFF-152 handback (and the timeline's verify line) on a branch
    named `claude/spec-072-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #249 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #249.
  - Reference HANDOFF-152, SPEC-072, DEC-066 and PR #249.
- **Rejected:** comment on #249 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- Any `src/` or `tests/` change.
- CI command changes.
- Speeding up individual tests.
- The release (SPEC-073).

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

- **Branch / PR:** `claude/spec-072-verify-5b9684`, https://github.com/jysf/irradiance/pull/250 (supersets #249)
- **Completed at:** 2026-11-16
- **All acceptance criteria met?** After the fixes in #250, yes. On #249 as built, no: AC5 failed (CI red) and the "none weakened" goal does not hold for the allocation-counting tests.
- **For `verify`:** ⚠ PUNCH LIST. Both items are fixed by the verifier in #250 at `3f2db30`, with the handback on top.

**Punch list**
1. **CI red on #249.** `rust / license policy red-proof — GPL dev-dep, both graphs` failed.
   - `scripts/deny-red-proof.sh` requires `[dev-dependencies]` to be the last table in `Cargo.toml`, and the build appended `[profile.corpus]` after it.
   - Reproduced locally on `1332da7`'s manifest. The build handback said CI was "not polled".
   - **Fix:** the profile is moved above `[dependencies]`, with a comment saying why. `just deny-red-proof` is green.
2. **The allocation-counting tests are weaker under `corpus`.** I injected `drop(vec![0u16; 64])` into `develop_with_scratch_into`.
   - Under `test`, three `develop_scratch_alloc` tests go red. Under `corpus` they stay green (9/9).
   - A used transient allocation (a `collect` read afterwards) goes red in both profiles.
   - **Fix (docs only):** the `Cargo.toml` comment, the `app.just` test comment, AGENTS.md §6 and a DEC-066 amendment now say the `*_alloc` tests are authoritative in the debug lane (CI and `just test-debug`).

**Reproduced** (M4 Pro, corpus set, 12/12 present)
- **AC1:**
  - Both lanes: 37 binaries, 697 passed, 0 failed, 1 ignored.
  - `-- --list` is 771 lines (698 tests) in both profiles, and the diff is empty.
- **AC2:** a throwaway test in a scratch copy, deleted afterwards.
  - Under `corpus`: `debug_assertions = true`, u8 overflow panics, `debug_assert!` fires.
  - Under `--release`: all three are false.
  - No profile environment variable reaches a test process.
- **AC3:** 0 SKIP lines, and every pin and red-proof is ok under `just test`.
- **AC4:** `just test` took 139 s after `touch src/lib.rs`, on the fixed tree with nothing else running. `just test-debug` took 2261 s (37.7 min).
- **AC5:** the `.github/` diff is comment lines only, with no `run:` change. The job set is the same as #247's (44 checks). It was red on #249 (item 1). #250's CI is the check.
- **AC6:** all 10 gates are green on the fixed tree. `cargo publish --dry-run --allow-dirty` also passes; profiles do not reach dependents.

**Points judged**
- **opt-level 3 as the default:** safe for every assertion, pin and red-proof, with the one exception of the allocation-counting tests (item 2).
  - Timing: `a_zero_width_tiled_plane_has_no_units` has a 5 s bound, but its `len() == 0` assertions catch the regression on their own. The other `Instant` uses only print.
  - No test reads `cfg(debug_assertions)` or is `#[should_panic]`.
  - The one `#[inline(never)]` (`ljpeg.rs`) is a performance hint, not a probe.
  - There are no recursion-depth tests beyond the IFD depth guard, which is a constant limit.
- **Red-proof tests:** the staged crates are built by a child `cargo build`, with `--release` (develop, develop_oracle, plane_validate, ifd_resource_guard, ljpeg_shape_binding, required_processing) or dev (previews). They build in their own `target/`, and the outer profile does not reach them, so `corpus` does not change what they prove.
- **AGENTS.md:** `just test-debug` is exactly the old two commands. The only duration quotes are in the new §6 text, and they are accurate. `guidance/signals.yaml` keeps a historical "~25 minute" note, which the ship resolves.
- **`Cargo.toml`:** the comment says why the profile inherits `test`. The publish dry run was checked.

### Cost self-report

- **Tokens (total):** 5,607,703 (a floor at handback time: 42 message.ids, no subagents)
- **Estimated USD:** 4.22 (opus-5-5 list prices, 1-hour cache writes at 2x input)
- **Duration (minutes):** about 50 (the debug suite ran in the background)
- **Source of the number:** the transcript `1f696876-fbe0-4f33-9c9c-15b98d7be19e.jsonl`, deduped by message.id with the largest output taken per id

### Drift and new artifacts

- **New decisions emitted:** none. DEC-066 gained an amendment bullet under Consequences: the alloc-test caveat and the profile placement.
- **Deviations from spec:** none beyond the punch-list fixes. There is no `src/` or `tests/` change and no CI change.
- **Follow-up work identified:**
  - A possible guard against the item-1 shape: `deny-red-proof.sh` could insert into `[dev-dependencies]` wherever it sits instead of requiring it last. It is not needed now.
  - The ship should check that #250's CI is green, then mark the signal `slow-tier-b-tests-in-debug` resolved.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing slowed me down. AC5 says "CI green on the PR head", but the build's handback read "CI is on the PR (not polled here)", and that is where the red was hiding.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The invariant that `[dev-dependencies]` is the last table (`deny-red-proof.sh`) is not recorded anywhere outside the script. Neither is the fact that LLVM may elide allocations, which limits what allocation-counting tests can prove in optimised builds.

3. **If you did this task again, what would you do differently?**
   — I would read `gh pr checks` first. It showed the red in seconds, before the 38-minute run started.
