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
  id: HANDOFF-176
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-084

project:
  id: PROJ-002
  stage: STAGE-014
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7679810               # REAL combined count — what cost-audit reads
  estimated_usd: 5.68              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 20
  branch: claude/spec-084-build
  pr: 298
  completed_at: 2026-11-21               # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 3cd2d69c-b494-4704-b5d2-2d67be4e4ed3.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 109 + cache-creation 128,696 (1-hour) + cache-read 7,525,974 + output 25,031 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Re-sum at ship."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-21
---

# HANDOFF-176: Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-084` to `claude-opus-5-5` (acting
as implementer) for the **`build`** cycle.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-084-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md`. It is the
  contract. Its Inputs section has the line numbers and the measurements taken at
  design; its Failing Tests section has the hand-computed AC3 bytes.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-014-0-2-x-maintenance-post-release-correctness-and-hygiene.md`
  (the "Second batch" note says where each item came from and why AC4 was dropped).
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-003`: tier-B corpus files are never committed; every new test here is tier A.
- `DEC-008`: the two uncompressed unpack paths (byte-aligned 8/16, MSB-first
  bitstream 12/14); AC3 covers both.
- `DEC-014`: the optional exiftool fields' absent / unreadable / value shape; AC2
  gives `req` the same refusal to keep a truncated head.
- `DEC-029`: a sample above WhiteLevel is legal, which is why SPEC-016's old AC4 is
  dropped. Do not resurrect it.

### Constraints that apply

- `oracle-must-be-shown-red`: AC6's mutants, each watched red then restored.
- `test-before-implementation`: write the three tests first and see them fail on
  `main`'s code (AC1 and AC2 fail; AC3 passes on `main` and must go red under its
  mutants).
- `one-spec-per-pr`.
- `library-not-application`: no `src` behaviour change; the only `src/` edit is a
  `#[cfg(test)]` test.

### Prior related work

- SPEC-016 (pre-rescope, git 3238dcb): the original AC1 to AC4 text.
- SPEC-005/FU-2 and SPEC-010/FU-2: the `req` truncation, left open twice because the
  check was written narrower than the finding. Write AC2's test to the finding.
- SPEC-083 verify FU-3 (HANDOFF-174): the `sdk-stage2.sh` wording.
- Signal `macos-arm64-runner-unavailable` in `guidance/signals.yaml`.

## Expected Deliverables

- The changes in the spec's Outputs, meeting AC1 to AC7.
- The three named tests passing, and the AC6 mutants each shown red, with the command
  and the failing assertion pasted into the Handback.
- A PR against `main` from `claude/spec-084-build`, referencing HANDOFF-176, SPEC-084,
  STAGE-014, PROJ-002 and the DECs above.
- Keep working until the PR is open and the handback is filled. If you start a long
  background run, wait for it; do not stop and report while it is still running.

### Repo rules a cold session trips on

- Commit-message bodies are plain prose: no code tokens (no backticks, `#[...]`, `->`,
  `--flag`). A push with them is rejected. PR bodies and files are fine.
- `Cargo.lock` is gitignored; never add it. Stage explicit paths, never `git add -A`.
- Run `just test`, not `just test --all-features` (the recipe already passes it).
- Use `cargo test --no-fail-fast` for totals; a plain run stops at the first failing
  binary.
- Corpus: `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place; never copy them into the repo.
- Don't force-push, merge, tag or publish.

## Out of Scope

- Any `src` behaviour change, `DEVELOP_VERSION` bump or release.
- Requiring or installing `exiftool` or `dnglab` in CI.
- Any CI change beyond the `determinism` job's `continue-on-error` and its comment.
- The dropped SPEC-016 AC4 (fuzz seeds and `SampleExceedsWhiteLevel`).

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

- **Branch / PR:** `claude/spec-084-build`, https://github.com/jysf/irradiance/pull/298. Build commit `2419b34`, cut from `origin/main` at `4011d6b` (PR 297).
- **Completed at:** 2026-11-21
- **All acceptance criteria met?** yes.
  - **AC1:** `preflight_summary` in `tests/support/corpus.rs` is the only writer of "no tier-B test will skip", and writes it only for a complete corpus with both tools on PATH. Each absent tool gets a `tools:  SKIP     <tool> not on PATH — the tier-B tests needing it (…) will SKIP` line. `corpus-status` includes `tests/support/tools.rs` by `#[path]` and probes with `exiftool_available` / `dnglab_available`. Live, with the corpus and both tools: `corpus: 12/12 present, exiftool and dnglab on PATH — no tier-B test will skip`. The built example run with `PATH=/usr/bin:/bin` prints both tool SKIP lines and no promise.
  - **AC2:** `req` matches exactly one value; several (or zero) values give `ToolError::Parse` "BitsPerSample is required to be single-valued but exiftool reported [16, 16, 16]". The live `metadata_oracle` stayed green on the corpus with both tools present (inside the `just test` run below).
  - **AC3:** `every_supported_bit_depth_is_exercised` in `src/plane.rs` `mod tests`, with the spec's hand-computed rows (8 bits in both orders, 12, 14, 16 in LE and BE) through `unpack_packed`, plus the sorted, deduplicated depth set compared with `SUPPORTED_BITS`. No non-test line in `src/` changed.
  - **AC4:** `continue-on-error` deleted from `determinism`, and the comment replaced with two lines. Signal `macos-arm64-runner-unavailable` is `done`, with a dated note naming SPEC-084 and PR 298. On the PR head, `rust / determinism (macos-latest, arm64)` ran and passed (21 s).
  - **AC5:** the message is exactly `sdk-stage2: cannot enter worktree $wt; keeping its recorded path as a refused location`, and the comment says "still refused". `bash -n` and `shellcheck` (Homebrew) are clean. Only the echo string and a comment changed.
  - **Test-first:** I wrote `preflight_summary` first as a copy of main's corpus-only logic. AC1 then failed: `the promise must appear exactly when nothing can skip — missing=0 exiftool=false dnglab=false: ["corpus: 9/9 present — no tier-B test will skip"]`. AC2 failed on main's `req`: `a three-valued BitsPerSample must be ToolError::Parse, got Ok(ToolReading { … bits_per_sample: 16 … })`. AC3 passed on main, as predicted.
- **AC6 mutants.** Each was applied with a one-occurrence replace, compiled, watched red, then restored with `git checkout`; the tree was clean afterwards and the test was green again.
  - **AC1, the promise ignores exiftool** (`if missing == 0 && dnglab {`). `cargo test --all-features --test corpus_manifest the_preflight` failed with: `assertion left == right failed: the promise must appear exactly when nothing can skip — missing=0 exiftool=false dnglab=true: [..., "corpus: 9/9 present, exiftool and dnglab on PATH — no tier-B test will skip"]`, left true, right false.
  - **AC1, the promise ignores dnglab** (`if missing == 0 && exiftool {`). Same command; it failed with: `… missing=0 exiftool=true dnglab=false …`, left true, right false.
  - **AC2, `req` back to the head** (`Some([value, ..]) => Ok(*value)`, which behaves like `.first()` on a non-empty list). `cargo test --all-features --test metadata_oracle a_multivalued_required` panicked at `tests/metadata_oracle.rs:737` with: `a three-valued BitsPerSample must be ToolError::Parse, got Ok(ToolReading { width: 8424, height: 5632, bits_per_sample: 16, … })`.
  - **AC3(a), 10 added** (`const SUPPORTED_BITS: [u32; 5] = [8, 10, 12, 14, 16];`). `cargo test --all-features --lib every_supported_bit_depth` failed with: `every SUPPORTED_BITS depth needs a row here, and every row a supported depth`, left `[8, 12, 14, 16]`, right `[8, 10, 12, 14, 16]`.
  - **AC3(a), the 14-bit row deleted.** Same command; left `[8, 12, 16]`, right `[8, 12, 14, 16]`.
  - **AC3(b), a 12-bit expected sample flipped** (3567 to 3568). Same command; it failed with `12-bit Little bytes [AB, CD, EF]`, left `[2748, 3567]`, right `[2748, 3568]`.
  - **AC3(b) extra, `low12` broken** (mask `0x07FF`). Same command; left `[700, 1519]`, right `[2748, 3567]`.
  - **AC4 and AC5 have no meaningful mutant.** A runner outage cannot be staged, and a message has nothing to falsify it.
- **AC7 gates, local.** Corpus `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`, used in place; `exiftool` and `dnglab` are on PATH at `/opt/homebrew/bin`.
  - **`just test`:** pre-flight `12/12 present … no tier-B test will skip`; 793 passed, 0 failed, 1 ignored over 46 result lines; about 3.5 min. That is 790 before plus the 3 new tests.
  - **`cargo test --all-features --profile corpus --no-fail-fast`** with the corpus: 779 passed, 0 failed, 1 ignored. The 14-test gap to 793 is the three debug `*_alloc` binaries, which `just test` runs separately.
  - **Green:** `just lint-ci`, `just lint`, `cargo fmt --check`, `just lint-no-allow`, `just doc-check` (2 surfaces, 0 warnings), `just typecheck`, `just msrv`, `just validate` (88 artifacts, 387 parse), `scripts/validate-red-proof.sh`, `scripts/gate-audit.sh` (13 scripts), `just gate-audit-red-proof`, `just lint-red-proof`, `scripts/name-length-check.sh` (911 paths), `scripts/cost-audit.sh`, `scripts/decisions-index.sh --check` (75 decisions).
  - **Not run:** `semver-check`, because nothing public changed (the only `src/` change is `#[cfg(test)]`), and `test-debug`, which CI's test job mirrors.
- **CI on the PR head `2419b34`:** 22 of 22 checks SUCCESS, including `rust / determinism (macos-latest, arm64)`, which now blocks, and `rust / test`.

### Cost self-report

- **Tokens (total):** 7,679,810. This is a floor, taken before the handback commit.
- **Estimated USD:** 5.68 (opus-5-5: input $5 and output $25 per MTok; cache read ×0.10; 1-hour cache write ×2.0).
- **Duration (minutes):** about 20.
- **Source of the number:** this session's own transcript, `3cd2d69c-b494-4704-b5d2-2d67be4e4ed3.jsonl`, measured per docs/cost-tracking.md "Measuring a cycle": 52 unique message.ids, all claude-opus-5-5, no subagents. Input 109, cache creation 128,696 (all 1-hour), cache read 7,525,974, output 25,031 (the largest per message). Re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **AC2 mutant.** I wrote "`req` back to `.first()`" as the slice pattern `Some([value, ..])`. On a non-empty list it behaves the same as `.first()`, and it is a one-token change in the new match.
  - **Extra mutant.** I added a code mutant for AC3(b) (`low12` broken) beside the expectation flip. The spec allowed either; I ran both.
  - **Pre-flight wording.** The line for a complete corpus is now `corpus: N/N present`, and the promise moved to its own final line, which also names the two tools. The MISSING line is unchanged.
- **Follow-up work identified:** none required.
  - **Optional.** The `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` lists in `preflight_summary` are the design-time measurement, written as text. If a new tier-B test starts skipping on a tool, the lists go stale, and nothing fails. That only matters if someone wants them derived, and it is not worth a spec now.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The hand-computed AC3 bytes and the exact AC5 message meant nothing had to be re-derived. The one judgement call was how to "see AC1 fail on main" when the function does not exist on main. I wrote the function first as a faithful copy of main's corpus-only branch, and that is worth saying in the template for "new pure function" specs.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — I would push the branch and open the PR before the local `just test`, which I did here. CI took about 4 min of wall clock and ran in parallel with the local gates.
