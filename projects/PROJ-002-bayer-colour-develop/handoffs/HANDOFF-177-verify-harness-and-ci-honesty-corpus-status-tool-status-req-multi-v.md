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
  id: HANDOFF-177
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 4004160            # REAL combined count — what cost-audit reads
  estimated_usd: 3.17              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-084-verify-base
  pr: 298
  completed_at: 2026-11-21         # YYYY-MM-DD
  notes: "Floor, taken live before the handback commit, from transcript eccda050-37dc-4827-82bf-4ae01132914a.jsonl: 32 unique message ids, all claude-opus-5-5, no subagents. Re-sum at ship. Approved at 42890c2; no verify fix was needed."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-177: Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-084` to `claude-opus-5-5` (acting
as verifier) for the **`verify`** cycle. You did not build this; review it cold.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-084-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md` (AC1 to AC7, the
  failing tests with hand-computed bytes, the exact outputs).
- **Build handoff and its handback:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-176-build-harness-and-ci-honesty-corpus-status-tool-status-req-multi-v.md`.
  The builder's claims to check are there: seven mutants red, 793 passed locally,
  22 of 22 CI checks on `2419b34`, three deviations.
- **The build:** PR 298 (`claude/spec-084-build`). Your base branch,
  `claude/spec-084-verify-base`, is that branch plus this handoff.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-014-0-2-x-maintenance-post-release-correctness-and-hygiene.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-003` (tier-B never committed), `DEC-008` (the two unpack paths AC3 covers),
  `DEC-014` (the tri-state shape AC2 extends to `req`), `DEC-029` (why SPEC-016's old
  AC4 was dropped).

### Constraints that apply

- `oracle-must-be-shown-red`: re-run at least the AC1, AC2 and AC3 mutants
  yourself; don't trust the pasted output alone. Try at least one mutant the
  builder did not.
- `library-not-application`: confirm no non-test line under `src/` changed.

## Expected Deliverables (verify contract)

Check, at minimum:
1. **AC1.** The promise sentence is written only by `preflight_summary`, and only
   for a complete corpus with both tools. `corpus-status` uses the same two probes
   the tests use. Run `cargo run --all-features --example corpus-status` with a tool
   hidden from PATH (for example a PATH without `/opt/homebrew/bin`) and confirm the
   real output stops making the promise and names the tool. That is the behavioural
   check this AC exists for.
2. **AC2.** `req` refuses zero or several values with a message naming the tag; the
   live `metadata_oracle` passes with
   `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
   and `exiftool` on PATH. Use the corpus in place; never copy it.
3. **AC3.** The table's bytes match the spec's hand computation; the depth-set
   assertion really enumerates `SUPPORTED_BITS` rather than a copied list.
4. **AC4.** `continue-on-error` is gone from `determinism` and nowhere else changed in
   `ci.yml`; the macOS leg ran and passed on the PR head; the signal is `done`.
5. **AC5.** The exact message; `bash -n` clean.
6. **The builder's deviations and its optional follow-up** (the hard-coded
   `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` lists): judge whether each is acceptable.
7. Decision drift: `just decisions-audit --changed` if available.

Fix small doc or test issues yourself on `claude/spec-084-verify-base` (commit,
push); anything larger is a punch-list item. Output exactly ONE of ✅ APPROVED (at
a commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Repo rules a cold session trips on

- Commit-message bodies are plain prose: no code tokens. A push with them is
  rejected; reword, don't retry.
- Never add `Cargo.lock`; stage explicit paths. Run `just test`, not
  `just test --all-features`. `cargo test --no-fail-fast` for totals.
- Use `/usr/bin/git` for authoritative branch state.
- Don't force-push, merge, tag or publish, and don't push to `claude/spec-084-build`;
  push only to `claude/spec-084-verify-base`.
- Keep working until the verdict is written and the handback is filled and pushed.
  If you start a long background run, wait for it; do not end your turn while it runs.

## Out of Scope

- Any `src` behaviour change, release or CI change beyond AC4.
- Re-designing the spec. Disagreement with the design is a punch-list note, not a rewrite.

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

- **Branch / PR:** `claude/spec-084-verify-base`, which is PR 298's head `172b0b1` plus the verify setup `42890c2`. https://github.com/jysf/irradiance/pull/298. No verify fix was needed. `claude/spec-084-build` is untouched.
- **Completed at:** 2026-11-21
- **All acceptance criteria met?** yes.
- **For `verify`:** ✅ **APPROVED at `42890c2`.** `src/`, `tests/`, `examples/`, `scripts/` and `ci.yml` there are byte-identical to the PR head `172b0b1`.

**The seven checks.** Corpus `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images` was used in place. `exiftool` and `dnglab` are at `/opt/homebrew/bin`.

1. **AC1.** The sentence "no tier-B test will skip" appears once in the code, in `preflight_summary`, behind `missing == 0 && exiftool && dnglab`. The example probes with `tools::exiftool_available` and `tools::dnglab_available`, which are the probes the tier-B tests skip on. **Behavioural check:**
   - Run with `PATH=~/.cargo/bin:/usr/bin:/bin`: the output is `corpus: 12/12 present`, then two `tools:  SKIP … not on PATH` lines naming each tool and its tests. No promise.
   - Run with a PATH that holds only an `exiftool` symlink: `exiftool` shows as present, `dnglab` gets its SKIP line, and there is no promise.
   - Run with the full PATH, inside `just test`: `corpus: 12/12 present, exiftool and dnglab on PATH — no tier-B test will skip`.
2. **AC2.** `req` takes exactly one value through the pattern `Some([value])`. Zero or several values give a `ToolError::Parse` that names the tag and its values. Absent keeps its old error. **The live oracle**, `metadata_matches_exiftool_on_every_corpus_file` run with `--nocapture`, checked 7 files through the stricter `req` with no mismatch and no Parse refusal. Those are the 7 monochrome files. The 5 Bayer files are skipped by design (`scalar_oracle_skips`, the colour oracle's claim). All 32 `metadata_oracle` tests passed.
3. **AC3.** I recomputed the table's bytes in Python:
   - `0004002000FFFF` in 14-bit fields gives `[1, 2, 3, 16383]`.
   - `ABCDEF` gives `0xABC`/`0xDEF`, which is `2748`/`3567`.
   - `0x1234` is `4660`.

   The depth-set assertion builds its right-hand side from `SUPPORTED_BITS.to_vec()`, the constant itself, not from a copied list. The diff in `src/plane.rs` is entirely inside `mod tests`.
4. **AC4.** `continue-on-error` is gone from `ci.yml`: `grep` finds none anywhere. The only other `ci.yml` change is the comment. On the PR head `172b0b1`, the job `rust / determinism (macos-latest, arm64)` (job 137003949943) ran every step and passed. That includes both `cargo test --test determinism` and its `--release` run. The signal `macos-arm64-runner-unavailable` is `status: done`, with a dated note.
5. **AC5.** The message is exactly as the spec gives it. `bash -n` and `shellcheck` are clean. The script's diff is two lines: one echo string and one comment.
6. **Deviations.** All three are acceptable.
   - The AC2 mutant written as a slice pattern behaves like `.first()`. I also ran a literal `.first()` form, and it went red.
   - The extra `low12` mutant is fine.
   - The pre-flight wording is fine. A complete corpus now prints its count twice (the count line, then the promise line). That is cosmetic.

   **The optional follow-up**, the hard-coded `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` lists: I checked them against today's code with `grep`. The 3 exiftool users and the 7 dnglab users in `tests/` are exactly the listed ones. That is accurate now, and nothing fails if it drifts. I agree it is not worth a spec. It is noted as FU-1 below.
7. **Decision drift.** `just decisions-audit --changed origin/main` gives advisory flags only:
   - DEC-056, DEC-063 and DEC-064 for `src/plane.rs`. The change is test-only, so it is consistent with all three. The test reaches DEC-064's whole-group path and not the `BitReader` tail, as the spec intends.
   - DEC-066 and DEC-067 for `ci.yml`. The test job and the name-length gate are untouched.

**AC6 mutants, re-run by me.** Each was applied with a one-occurrence replace, compiled, and run with `cargo test --all-features` on its test. Each was restored with `git checkout`, and the tree was clean afterwards. All 9 went **RED**.

- Builder's six:
  - AC1, the promise ignores exiftool: red at `corpus_manifest.rs:541`, `missing=0 exiftool=false dnglab=true`, left true, right false.
  - AC1, the promise ignores dnglab: red, `exiftool=true dnglab=false`.
  - AC2, `req` back to `.first()` (literal `v.first()` form): red at `metadata_oracle.rs:737`, `got Ok(ToolReading { … bits_per_sample: 16 … })`.
  - AC3(a), 10 added to `SUPPORTED_BITS`: red, left `[8, 12, 14, 16]`, right `[8, 10, 12, 14, 16]`.
  - AC3(a), the 14-bit row deleted: red, left `[8, 12, 16]`, right `[8, 12, 14, 16]`.
  - AC3(b), 3567 flipped to 3568: red, `12-bit Little bytes [AB, CD, EF]`.
- **New ones the builder did not try:**
  - **AC3(b), `unpack_groups14`'s second shift changed from 28 to 27:** red, `14-bit … left [1, 4, 3, 16383] right [1, 2, 3, 16383]`.
  - **AC1, the promise ignores the corpus** (`if exiftool && dnglab`): red, `missing=1 exiftool=true dnglab=true`.
  - **AC1, the tool line inverted** (`if !on_path`): red at `corpus_manifest.rs:550`, `exiftool must be named in a SKIP line exactly when it is absent`.

**AC7 gates, re-run locally at `42890c2`:**
- `just test` with the corpus and both tools: pre-flight `12/12 present`, the promise line printed, 793 passed, 0 failed, 1 ignored over 46 result lines.
- `cargo test --all-features --profile corpus --no-fail-fast`: 779 / 0 / 1 over 43 result lines. Both equal the builder's figures.
- `just lint-ci`, `just validate`, `just doc-check`, `just typecheck`, `cargo fmt --check` and `scripts/decisions-index.sh --check` are all green.
- I did not re-run the builder's other gates (lint-no-allow, msrv, the red-proofs, name-length, cost-audit). CI ran them green on the PR head.

**CI on the PR head `172b0b1`:** 22 of 22 pass, mergeable.

### Cost self-report

- **Tokens (total):** 4,004,160. This is a floor, taken live before the handback commit.
- **Estimated USD:** 3.17. That is opus-5-5 at $5 input and $25 output per MTok, cache read ×0.10, and 1-hour cache write ×2.0 (every write was 1-hour).
- **Duration (minutes):** about 25.
- **Source of the number:** this session's transcript, `eccda050-37dc-4827-82bf-4ae01132914a.jsonl`, measured per docs/cost-tracking.md "Measuring a cycle":
  - 32 unique message ids, all claude-opus-5-5, no subagents;
  - input 67, cache creation 87,835, cache read 3,902,688, output 13,570.

  Re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the verify cycle. The builder's three deviations are judged acceptable above.
- **Follow-up work identified:**
  - **FU-1 (nit, no spec):** `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` in `tests/support/corpus.rs` are text, accurate today. A new tier-B test that skips on a tool would not be named, and nothing fails. Revisit only if a tool-gated test is added.
  - **FU-2 (nit, cosmetic):** with a complete corpus, the pre-flight states `N/N present` on two lines. That is harmless, and it can be folded into any future edit of `preflight_summary`.
  - Two notes on how `corpus-status` is tested. The example's wiring (which probe feeds which argument) has no unit test; only the behavioural run above covers it. Swapping the two probes in the example would pass every test. That is in the nature of an example, and it is not a punch-list item.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec. One thing looked wrong but was not: the live oracle prints "7/12 corpus files present" even with 12/12 on disk. The 7 is what it checked, because the Bayer files are excluded by design. The message wording invites a false alarm.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-064 (whole-group unpack plus the `BitReader` tail), as context for AC3. The table reaches only the whole-group path, which is intended, but a verifier should know the tail path exists and is not this test's claim.

3. **If you did this task again, what would you do differently?**
   — Write the mutant runner as one script from the start, as I did. Also avoid shell variables in Bash calls in this environment: the rtk hook rewrote one compound command, and it failed confusingly.
