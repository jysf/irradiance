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
  id: HANDOFF-196
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-094

project:
  id: PROJ-005
  stage: STAGE-017
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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 4815284            # REAL combined count — what cost-audit reads
  estimated_usd: 3.75              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: claude/spec-094-verify-base
  pr: 339
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript feabd2f5-7a5e-476b-a230-5264f1068e0b.jsonl (35 unique message.ids, all claude-opus-5-5, no subagents): input 69 + cache-creation 100,834 (1-hour) + cache-read 4,698,805 + output 15,576, priced at 5 and 25 USD per million. Re-sum at ship. Approved at e833553's src (no src change in the verify)."
  synced_at: 2026-11-25
---

# HANDOFF-196: VERIFY: STAGE-017's three follow-ups, bundled (SPEC-094)

## Delegation Summary

You are the **verifier** for SPEC-094. Review it cold: you did not build it.

- **Under review:** PR 339 (`claude/spec-094-build`, head `e833553`), built under
  HANDOFF-195 by a separate session. CI was 23 of 23 green at that head.
- **Your branch:** `claude/spec-094-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-094 verify`.

SPEC-094 closes three items, one commit each:
1. `da50aed`: `WriteInputError::EmptyOpcodeList { list }` refuses `Some(&[])` for any opcode
   list. It runs under every framing, first in the opcode step, before the `OpcodeList1`
   walk, in `plan`.
2. `5f411c2`: a `rust / fuzz smoke — dng_write (60s)` CI job.
3. `e26aeb6`: three writer doc links written as plain code, so a no-feature
   `--document-private-items` build no longer fails on them.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md`.
  Judge it against the spec **as designed** (`git show 6eab1b8:<spec path>`): AC1 to AC7,
  Design §1 to §3, the rule-clause map, and the distinguishing fixtures.
- **Decisions:** DEC-081 §2 and its Follow-up; DEC-075 §5, §7 and §9; DEC-080; DEC-051 and
  DEC-052.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `library-not-application`, `no-copyleft-dependencies`.
- **Build handback:** HANDOFF-195's `## Handback`, in the same handoffs directory. It has the
  red proof, the before and after doc-command table, the one moved assertion, and three
  deviations.
- **Prior verifies:** HANDOFF-190, 192 and 194. Their mutant tables are the model.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`.
  - Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
    `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Use `/usr/bin/git` for branch and HEAD.
  - Give each mutant its own source state. Plain cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC is met against the spec as designed,** and every rule-clause-map row reaches a
   test that fails when its clause is wrong.
2. **Mutants.** Each must fail a named test; give a table. At least these:
   - check only list 1;
   - check only under the cutting framings, so FullSensor accepts an empty list;
   - check only in `serialize`, which breaks parity;
   - move the check after the `OpcodeList1` walk;
   - check the lists in the order 3, 2, 1;
   - refuse `len < 4` instead of `len == 0`;
   - refuse a 4-byte count-0 list;
   - report the wrong `list` number;
   - drop the check entirely.
3. **The build's three deviations.** Rule on each:
   - (a) One existing assertion moved: `opcode_list_1_allow_list_under_cutting_framings` went
     from `MalformedOpcodeList` to `EmptyOpcodeList`. The design said no such assertion
     existed. Is this the only one? Grep for every way a zero-length opcode list can be built,
     including through variables and helpers, not only the literal `Some(&[])`.
   - (b) There was no CI-job list to update, so the AGENTS.md recipe block now names the job.
     Is anything else stale, such as a list of fuzz targets, of CI jobs, or of gates in a
     handoff template?
   - (c) `every_refusal_is_typed` covers only StandardCrop. Is the per-framing coverage in the
     new test enough?
4. **Red proof.** The build added the variant first, without the check, and saw 5 failures.
   Re-run that: revert only the check from `da50aed` and show which tests fail.
5. **The CI job.** Compare it line by line with `fuzz-ifd`, `fuzz-plane` and `fuzz-ljpeg`:
   toolchain, `cargo-fuzz --locked`, corpus dir, seeds dir and `-max_total_time=60`. Does it
   build the target with the features the target needs, as the local recipe does? Read its
   log on the PR and confirm it ran for about 60 s on the real seeds, and was not a no-op.
6. **Doc links.** Re-run AC6's command yourself, without features and with
   `--all-features`. Confirm that no `crate::dng` link is left, and that the 13 `ljpeg`
   errors are the only ones. Rule on the plain-code choice for `src/lib.rs`: is it right that
   the comment never reaches public docs?
7. **No panics, no allocation.** `dng_write_alloc` still passes. The fuzz target reaches the
   new refusal: the `refuse-empty-opcode-list` seed, and empty lists from the fuzzer's own
   length byte.
8. **Docs.** Check the module docs' "Checks and their order", the reason's doc comment and
   `Display`, the CHANGELOG, and DEC-081's Follow-up bullet ("resolved by SPEC-094").
9. **Gates.** Read `gh pr checks 339` first. Then run:
   - `just test` with the corpus and `dng_validate` set;
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`;
   - `deny`, `deny-fuzz`, `lint-fuzz`, **`lint-fuzz-red-proof`**, `msrv`;
   - `validate`, `gate-audit`, `cost-audit`, `semver-check`;
   - `just decisions-audit --changed`;
   - 60 s of `just fuzz-dng-write`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and your rulings on items 3,
  5 and 6.
- **Commit** this handoff's filled handback, and any tests or docs fixes, to
  `claude/spec-094-verify-base`, and push it. If you approve:
  1. Run `just advance-cycle SPEC-094 ship --verdict approved`.
  2. Run `just handback-sync SPEC-094` in the same push, so CI's cost audit passes.
  3. Fast-forward `claude/spec-094-build` to the verify tip, so PR 339 carries the verify
     commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` or `.github/` change goes in the
  punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** before minting a DEC or HANDOFF, compute the next free one from `origin/main`
  plus the open PRs. Right now those are DEC-083 and HANDOFF-197.

## Out of Scope

- The 13 `ljpeg` private doc links, and any CI gate on `--document-private-items`.
- README's fuzz recipe list. The build named it as a follow-up for the release spec.
- Walking `OpcodeList2` or `OpcodeList3`, or changing the `OpcodeList1` allow-list.
- Any reader change.
- The STAGE-017 close. The orchestrator does it.

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

- **Branch / PR:** `claude/spec-094-verify-base`, fast-forwarded into `claude/spec-094-build`, https://github.com/jysf/irradiance/pull/339
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes, AC1 to AC7, against the spec as designed at `6eab1b8`.
- **Verdict:** ✅ APPROVED. The code under review is `e833553` (PR 339's build head). The verify changes no `src/` or `.github/` file; it adds one test assertion (below) and this handback.

**Gates.** `gh pr checks 339` read first: 23 of 23 pass at `e833553`, `rust / fuzz smoke — dng_write (60s)` among them. Re-run locally:
- `just test` with `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` set: exit 0, 55 binaries, 923 passed, 1 ignored, corpus 12/12 present, `dng_validate` present and its acceptance tests ran. (`IRRADIANCE_SDK_STAGE2_DIR` was unset, so the SPEC-060 stage-2 cross-check skipped, as in prior verifies.)
- `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check` (2 feature surfaces, 0 warnings): all exit 0.
- `deny`, `deny-fuzz`, `lint-fuzz`, `lint-fuzz-red-proof` (control clean, injection exit 101 with the flag, exit 0 without), `msrv`: all exit 0.
- `validate` (98 artifacts), `gate-audit` (14 scripts), `cost-audit`, `semver-check` ("no semver update required"): all exit 0.
- `just decisions-audit --changed`: advisory only, flags DEC-004 for `tests/dng_write.rs`. Consistent: the tests read tag bytes back, no image comparison.
- `just fuzz-dng-write`: 3,835,115 runs in 61 s, 33 seeds, no crash.

**Red proof (item 4).** Dropping only the check from `plan` (mutant M9, the rest of `da50aed` kept) gives 29 passed, 5 failed in `tests/dng_write.rs`: `checks_run_in_the_documented_order`, `empty_opcode_list_is_refused_under_every_framing`, `every_refusal_is_typed`, `fuzz_seeds_reach_their_named_outcome`, `opcode_list_1_allow_list_under_cutting_framings`. That is the build's five, exactly.

**Mutants (item 2).** Each run on its own copy of `src/dng/write.rs` with `cargo test --all-features --test dng_write --no-fail-fast`. All killed.

| # | Mutant | Killed by |
|---|---|---|
| M1 | check only list 1 | `empty_opcode_list_…`, `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…` |
| M2 | check only under cutting framings (FullSensor accepts) | `empty_opcode_list_is_refused_under_every_framing` |
| M3 | check only on the write path (`write_into`, `write_to_vec`), not in `plan` | `empty_opcode_list_…`, `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…`, `opcode_list_1_allow_list_…` (`written_len` returns `Ok`) |
| M4 | check after the `OpcodeList1` walk | `checks_run_…` (gets `UnsupportedOpcodeForFraming`), `empty_opcode_list_…`, `every_refusal_is_typed`, `opcode_list_1_allow_list_…` (gets `MalformedOpcodeList`) |
| M5 | lists in order 3, 2, 1 | `checks_run_in_the_documented_order` (gets `{ list: 3 }`) |
| M6 | refuse `len < 4` | `opcode_list_1_allow_list_…` (its 3-byte list 2); after the verify's assertion also `count_zero_opcode_list_header_is_still_written_verbatim` |
| M7 | also refuse a 4-byte count-0 list | `count_zero_opcode_list_header_is_still_written_verbatim`, `opcode_list_1_allow_list_…` |
| M8 | wrong `list` number (`list % 3 + 1`) | `empty_opcode_list_…`, `every_refusal_is_typed`, `checks_run_…`, `fuzz_seeds_…`, `opcode_list_1_allow_list_…` |
| M9 | drop the check | the five red-proof tests above |

**Rule-clause map.** Every row reaches a test that fails when its clause is wrong. One row was mis-pointed by the design: "only zero bytes" names `count_zero_…` as the killer of a `len < 4` mutant, but that test passes only 4-byte lists, so `len < 4` never fired there (M6 was caught by `opcode_list_1_allow_list_…`'s 3-byte list 2 instead). **Verify-added, tests only:** `count_zero_opcode_list_header_is_still_written_verbatim` now also writes a 1-byte list 2 and list 3 at FullSensor and StandardCrop and reads each tag back as exactly that byte, so the named test kills M6 itself.

**Rulings on the build's deviations (item 3).**
- (a) **Accepted.** The moved assertion in `opcode_list_1_allow_list_under_cutting_framings` is the only one. I grepped every writer-side builder: `tests/dng_write.rs` (literals and the `list_1/2/3()` helpers, all non-empty; `[0xFF; 3]`, `[0xFF; 7]`, `[0, 0, 0, 0]` are non-empty), `tests/dng_write_alloc.rs` (built by `opcode_list`, non-empty), `tests/dng_write_corpus.rs` and `tests/support/dng_write_oracle.rs` (copied from corpus sensors, which the reader never yields empty), and the fuzz decoder, where `bytes()` with a length byte of 0 gives `Some(vec![])`. That last one reaches the refusal through the new seed (`EmptyOpcodeList { list: 2 }`), and every other seed still reaches its named outcome (`fuzz_seeds_reach_their_named_outcome`, 33 seen). The `Some(Vec::new())` at `tests/support/dng_write_fuzz.rs:474` is a descriptive string, not a list.
- (b) **Accepted.** No file counts CI jobs. `guidance/toolchain-brief.md` names none; no handoff template lists gates by job; `lint-fuzz-red-proof` copies no job or seed list. The only stale list is README's `just fuzz` line, already a named follow-up (out of scope).
- (c) **Accepted.** `every_refusal_is_typed` is the reason-catalogue test at one framing; the per-framing claim is `empty_opcode_list_is_refused_under_every_framing` (3 lists × 3 framings × `written_len`, `write_into`, `write_to_vec` via `assert_refused`), which is what killed M2.

**The CI job (item 5).** Line for line the same as `fuzz-ifd`, `fuzz-plane` and `fuzz-ljpeg`: `ubuntu-latest`, `dtolnay/rust-toolchain@nightly`, `cargo install cargo-fuzz --locked`, `mkdir -p fuzz/corpus/dng_write`, then `cargo +nightly fuzz run dng_write fuzz/corpus/dng_write fuzz/seeds/dng_write -- -max_total_time=60`. Features: `fuzz/Cargo.toml` depends on irradiance with `features = ["fuzzing", "write"]`, so the target builds with `write` exactly as the local recipe does. Its log on PR 339 (job 139024455713): "seed corpus: files: 33", INITED at cov 1802, "Done 2921496 runs in 61 second(s)" at cov 2173. It ran on the real seeds and was not a no-op.

**Doc links (item 6).** `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items`, re-run at the build's tree: without features, 13 unresolved links; with `--all-features`, 13 unresolved links. In both they are exactly the `ljpeg` ones (`HuffTable` ×3, `HuffTable::build`, `ScanReader` ×3, `ScanReader::restart`, `parse_headers` ×2, `Frame`, `decode_into`, `reconstruct`). No `crate::dng` link remains. `just doc-check` passes. **Ruling on `src/lib.rs`:** plain code is right. The comment is the doc of `mod write_types;`, a private module, so rustdoc shows it only in a private-items build, never in public docs of any feature set. A `cfg_attr` pair would buy nothing a reader of the public docs could see.

**No panics, no allocation (item 7).** `dng_write_alloc` passes in `just test`; `lint-no-allow` passes. The fuzz target reaches the refusal through the `refuse-empty-opcode-list` seed and through its own length byte (`bytes()` with n = 0).

**Docs (item 8).** The module docs' "Checks and their order" now reads "the opcode lists (none empty, then `OpcodeList1`'s walk under a cut)". The reason's doc comment is the spec's text verbatim, and `Display` is "OpcodeList{list} is empty" (asserted per list in the new test). The CHANGELOG `[Unreleased]` entry and DEC-081's Follow-up ("Resolved by SPEC-094") are present and accurate.

### Cost self-report

- **Tokens (total):** 4,815,284 (floor, measured before the handback commit)
- **Estimated USD:** 3.75
- **Duration (minutes):** about 45
- **Source of the number:** the transcript, deduplicated by `message.id` per docs/cost-tracking.md. There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. No IDs minted.
- **Deviations from spec:** none in the verify. The build's three are accepted above.
- **Follow-up work identified:**
  - The spec's Context says every other fuzz target has a CI smoke job. Two do not: `develop` and `demosaic` are in `fuzz/Cargo.toml` but have no job in `ci.yml`. This predates SPEC-094 and is not a defect of it. Worth weighing at the STAGE-017 close.
  - Already known: README's `just fuzz` line omits `fuzz-dng-write`, and the 13 `ljpeg` private links.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocked. One rule-clause-map row named a test that could not kill its mutant (a 4-byte list never trips `len < 4`); the mutant run showed it, and one assertion fixed it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — Nothing material. Running the mutants and code gates in a scratch copy with its own target directory, while `just test` ran in the worktree, kept the whole verify under an hour.
