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
  id: HANDOFF-192
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-24
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-081

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

# HANDOFF-192: VERIFY: EXIF block, XMP, descriptive tags and rendering hints (SPEC-081)

## Delegation Summary

You are the **verifier** for SPEC-081. Review it cold: you did not build it.

- **Under review:** PR 332 (`claude/spec-081-build`, head `1ed1865`), built under
  HANDOFF-191 by a separate session. CI was 22 of 22 green at that head.
- **Your branch:** `claude/spec-081-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-081 verify`.

SPEC-081 extends the shipped writer (SPEC-061) with: a caller EXIF block copied verbatim at
file offset 8 with IFD0 after it, `ExifIFD`/`GPSInfo` as `8 + offset` (DEC-072);
`EXIF_BLOCK_BASE` and `exif_block_base`; `XMLPacket`; eight typed ASCII descriptive tags
(DEC-071 §3a); four rendering hints as `Rational` (DEC-074); every new check in `plan`, after
`UniqueCameraModel` and before the `OpcodeList1` walk (DEC-081 §4); five new
`WriteInputError` reasons; no new `Error` variant.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-081-exif-block-and-xmp-placement-at-a-fixed-base-offset.md`.
  Judge the ACs against the spec **as designed** (`git show f5c1b9c:<spec path>`): AC1 to
  AC10, Design §1 to §7, and the rule-clause map.
- **Decisions:** DEC-081 (proposed, the design's choices), DEC-072 with its amendment,
  DEC-071 §2 and §3a, DEC-074, DEC-075 §1 and §7, DEC-080; DEC-051, DEC-052, DEC-058,
  DEC-016, DEC-069 §6.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`,
  `library-not-application`.
- **Build handback:** HANDOFF-191's `## Handback` (same handoffs dir): the exiftool and
  `dng_validate` results and three deviations.
- **Prior verify:** HANDOFF-190 (SPEC-061). Its mutant table is a model; its punch list was a
  CI job missing from the handoff's gate list.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`. Run `just test`, never
  `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Use `/usr/bin/git` for branch and HEAD. Give each mutant run its own source state; plain
  cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC met against the spec as designed,** and every rule-clause-map row reaching a
   test that fails when the clause is wrong.
2. **Mutants.** At least: drop the pad byte (odd block), write the pointer as the in-block
   offset (no `+ 8`), write `GPSInfo` when only an Exif offset is given, accept an odd offset,
   accept `offset + 2 == len + 1`, skip `DateTime`'s form check, let a RATIONAL hint take a
   negative, write `BaselineExposure` as RATIONAL, treat `Some("")` as absent, accept an empty
   XMP, move one new check after the `OpcodeList1` walk, run one new check only in
   `serialize` (parity), and break `exif_block_base` to return 10. Each must fail a named
   test. Give a table.
3. **The three build deviations.** (a) `InvalidAscii` before `DateTimeInvalid` for a
   non-ASCII `DateTime`: is that DEC-081 §4's order, and is it documented in the module docs?
   (b) The exiftool oracles on a synthetic plane: does oracle 5 alone still prove the block
   on real files? (c) All 22 fuzz seeds regenerated: do they still reach their named
   outcomes, with the same cases as before?
4. **Red-proofs.** Oracle 4 (a base-10 block is caught, after the base-8 negative control,
   and the two blocks differ) and the hints half of oracle 5 (the K-3 render differs without
   `BaselineExposure`). Run the new tests against `f5c1b9c`'s `src/` too: they must fail.
5. **The test block builder** (`tests/support/exif_block.rs`): are its offsets really
   absolute for the given base, including the Interop pointer? A builder bug that writes
   block-relative offsets would make oracle 3 pass for the wrong reason only if exiftool
   tolerates it. Check one block by hand (hex) against the TIFF and Exif rules.
6. **No panics, no allocation.** The new arithmetic (block length, pad, pointer sums, XMP
   length) on caller input; `written_len` and `write_into` allocate nothing with every field
   set. Does the fuzz target reach every new refusal, or only `Ok`?
7. **Docs.** The module docs ("What is written", "Checks and their order") list every new tag
   and check in the order the code runs them; CHANGELOG, provenance row, AGENTS.md block.
8. **Gates:** `just test` with the corpus and `dng_validate` set, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`, **`lint-fuzz-red-proof`**,
   `msrv`, `validate`, `gate-audit`, `cost-audit`, `just decisions-audit --changed`,
   `cargo semver-checks`, 60 s of `just fuzz-dng-write`, and **`gh pr checks 332` first**.
9. **DEC-081.** Is each of its seven items sound and followed by the code? Should it be
   accepted at ship?

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and rulings on items 3 and 9.
- **Commit** this handoff's filled handback (and any tests or docs fixes) to
  `claude/spec-081-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-081 ship --verdict approved`, then `just handback-sync SPEC-081`
  (so CI's cost audit passes at ship), and fast-forward `claude/spec-081-build` to the
  verify tip so PR 332 carries the verify commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** a PROJ-006 session also mints IDs. Before minting a DEC or HANDOFF, compute the next
  free one from `origin/main` plus open PRs.

## Out of Scope

- Previews (SPEC-082); UTF-8 `LocalizedCameraModel`; `Copyright`'s editor part.
- The empty-opcode-list follow-up in DEC-081 (SPEC-061 behaviour).
- Any reader change. If a round trip exposes a reader bug, report it.
- Accepting DEC-081 (the orchestrator does it at ship).

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
