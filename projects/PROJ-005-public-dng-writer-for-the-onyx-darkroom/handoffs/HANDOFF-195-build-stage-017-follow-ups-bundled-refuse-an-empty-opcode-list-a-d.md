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
  id: HANDOFF-195
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 8051192            # REAL combined count — what cost-audit reads
  estimated_usd: 5.80              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: claude/spec-094-build
  pr: 339
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 60060586-abbf-424f-aa19-b005f940fab7.jsonl (53 unique message.ids, all claude-opus-5-5, no subagents): input 112 + cache-creation 128,968 (1-hour) + cache-read 7,899,577 + output 22,535, priced at 5 and 25 USD per million as HANDOFF-193 did. Re-sum at ship. fuzz-dng-write 2,811,502 runs in 61 s, no crash, 33 seeds."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-195: Build SPEC-094, STAGE-017's three follow-ups, bundled

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-094` to `claude-opus-5` (implementer) for the
**build** cycle, on a branch cut from `main`. Make the spec's failing tests pass and close
three small items:
1. the writer refuses an empty opcode list;
2. CI gains a 60 s `dng_write` fuzz smoke job;
3. three private doc links stop breaking a no-feature `--document-private-items` build.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md`.
  The reason's name and payload, the check's place in the order, the job's name, and the
  rule-clause map are binding.
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
- **Prior specs:** SPEC-061, SPEC-081 and SPEC-082 in `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/done/`. These are the
  writer you extend. Their verify handoffs (HANDOFF-190, 192, 194) list the findings this
  spec closes (F3, F4) and the mutant style the verify will use.
- **Precedent for a bundled follow-up spec:** SPEC-074 (PROJ-004).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.
- **Environment:** set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
  so `just test` runs the corpus tests. Never commit `dng_validate` or its outputs.

### Decisions that apply

- `DEC-081` §2 and its Follow-up: an empty XMP is refused, and this spec does the same for
  opcode lists. At build, record "resolved by SPEC-094" in that Follow-up bullet.
- `DEC-075` §5, §7 and §9: the opcode-list rules, the two error variants, and no
  allocation.
- `DEC-080`: the payload types in `src/write_types.rs`, compiled in every build.
- `DEC-051` (no wildcard match) and `DEC-052` (`#[non_exhaustive]`).

### Constraints that apply

- `no-panics-on-untrusted-input`: `just lint-no-allow` must pass, with no `#[allow]` for
  the doc links either.
- `oracle-must-be-shown-red`: show each new test failing on `main`'s `src/` before the fix.
  Say how in the handback.
- `library-not-application`: no I/O in `src/dng`.
- `no-copyleft-dependencies`: no new dependency.

## Expected Deliverables

- Every file in SPEC-094's `## Outputs`, and every test in `## Failing Tests` passing.
- **Three commits, one per item,** so the verify can mutate each item on its own.
- **Gates, all green:**
  - `just test`, with the corpus and `IRRADIANCE_DNG_VALIDATE` set;
  - `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just doc-check`;
  - `just deny`, `just deny-fuzz`, `just lint-fuzz`, `just lint-fuzz-red-proof`, `just msrv`;
  - `just validate`, `just gate-audit`, `just cost-audit`, `just semver-check`;
  - 60 s of `just fuzz-dng-write`.

  **Before handing back, confirm every job in `gh pr checks` for your PR passes. There
  should be 23 now, the new `dng_write` smoke job included.**
- **The PR:** against `main` from `claude/spec-094-build`. It names HANDOFF-195, SPEC-094,
  STAGE-017 and PROJ-005, plus DEC-075 and DEC-081. Commit messages are plain prose.
- **In the handback:**
  - The AC6 doc command's output, before and after, without features and with
    `--all-features`.
  - Which form you chose for the `src/lib.rs` link, and why.
  - Every fuzz seed or test that changed outcome because of the refusal.
  - The token count.

## Out of Scope

- The 13 pre-existing `ljpeg` private doc links, and any CI gate on
  `--document-private-items`.
- Walking `OpcodeList2` or `OpcodeList3`; refusing a count-0 opcode header; changing the
  `OpcodeList1` allow-list.
- Any reader change.
- The STAGE-017 close and the release spec. The orchestrator does both.

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

- **Branch / PR:** `claude/spec-094-build`, https://github.com/jysf/irradiance/pull/339
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes. AC1 to AC4 and AC6 to AC7 are green locally; AC5 (23 of 23 CI checks) is confirmed on the PR before reporting.
- **Commits, one per item:** `da50aed` (the refusal), `5f411c2` (the CI job), `e26aeb6` (the doc links).
- **Red proof (oracle-must-be-shown-red):** the variant and its `Display` were added first, with no check, so the tests compiled against `main`'s behaviour. `cargo test --all-features --test dng_write` then gave 29 passed, 5 failed: `empty_opcode_list_is_refused_under_every_framing`, `every_refusal_is_typed`, `checks_run_in_the_documented_order`, `opcode_list_1_allow_list_under_cutting_framings` (the moved assertion) and `fuzz_seeds_reach_their_named_outcome` (the new seed reached `Ok`). With the check in `plan`: 34 passed. `count_zero_opcode_list_header_is_still_written_verbatim` passes on both sides, as a preservation test should. Its red is the verify's mutant (refuse `len < 4`).
- **Tests and seeds whose outcome the refusal changed:**
  - `opcode_list_1_allow_list_under_cutting_framings`: `Some(&[])` for list 1 at StandardCrop expected `MalformedOpcodeList`; it now expects `EmptyOpcodeList { list: 1 }`. This is the only existing assertion that moved.
  - No existing fuzz seed holds an empty list. `just fuzz-seeds` rewrote all 32 old seeds byte-identically, so none was renamed or re-pointed. One seed is new: `refuse-empty-opcode-list` (empty list 2 at StandardCrop, reaches `EmptyOpcodeList { list: 2 }`). The seed count is now 33.
- **AC6, the doc command** (`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items`; it exits 101 in all four cases because of the `ljpeg` links):

  | | without features | `--all-features` |
  |---|---|---|
  | before (`5f411c2`) | 16 unresolved: `crate::dng`, `crate::dng::write::Framing`, `crate::dng::write::WriteInputError`, and the 13 `ljpeg` | 13 unresolved, all `ljpeg` |
  | after (`e26aeb6`) | 13 unresolved, all `ljpeg`; no `crate::dng` | 13 unresolved, all `ljpeg` |

  `just doc-check`: "2 feature surfaces documented with -D warnings, 0 rustdoc warnings".
- **`src/lib.rs` link form:** plain code (`` `dng::write` ``), not a `cfg_attr` pair. The comment sits on `mod write_types;`, which is private, so it never appears in the public docs of any feature set. A `cfg_attr` pair would add two doc variants for text only the private-items build shows. Plain code reads correctly in both.
- **CI job count:** `ci.yml` now has 22 jobs; `determinism` is a 2-way matrix, so the PR shows 23 checks.
- **Fuzz:** `just fuzz-dng-write` ran 2,811,502 runs in 61 s with no crash (33 seeds).
- **Gates run locally, all green:**
  - `just test`, with `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` set (55 binaries, 923 passed, 1 ignored; corpus 12/12 present)
  - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`
  - `deny`, `deny-fuzz`, `lint-fuzz`, `lint-fuzz-red-proof`, `msrv`
  - `validate`, `gate-audit`, `cost-audit`, `semver-check` (no semver update required)
- `lint-fuzz-red-proof` copies no list of seeds or jobs, so it needed no change.

### Cost self-report

- **Tokens (total):** 8,051,192 (floor, measured before the handback commit)
- **Estimated USD:** 5.80
- **Duration (minutes):** about 60
- **Source of the number:** the transcript, deduplicated by `message.id` per docs/cost-tracking.md. There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The spec says no test in `tests/dng_write*.rs` passes `Some(&[])` for an opcode list. One does: `opcode_list_1_allow_list_under_cutting_framings` (list 1, expecting `MalformedOpcodeList`). Its assertion moved to the new reason, so AC3's "every existing writer test passes unchanged" holds for every test except this one assertion.
  - AGENTS.md has no list that counts the CI jobs, and neither does `guidance/toolchain-brief.md`. The `fuzz-dng-write` recipe block in AGENTS.md now names the CI job instead.
  - `every_refusal_is_typed` checks the three empty lists at StandardCrop. The per-framing coverage is in the new test.
- **Follow-up work identified:**
  - The 13 `ljpeg` private doc links still fail a private-items doc build (a non-goal here; for the stage close to weigh).
  - README's `just fuzz` line lists the other fuzz recipes but not `fuzz-dng-write`. One line, for the release spec.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The spec's claim that no test passes an empty opcode list was wrong by one assertion, but its "Moved assertions" rule covered what to do.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — I would grep for the empty-list shape in every opcode-list test before trusting the spec's inventory, not after the red run.
