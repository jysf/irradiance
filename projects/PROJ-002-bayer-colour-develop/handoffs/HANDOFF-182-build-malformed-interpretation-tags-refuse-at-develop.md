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
  id: HANDOFF-182
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-087

project:
  id: PROJ-002
  stage: STAGE-027
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
  agent: claude-opus-5-5                    # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10156524         # REAL combined count — what cost-audit reads
  estimated_usd: 8.05              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 13
  branch: feat/spec-087-malformed-tags
  pr: https://github.com/jysf/irradiance/pull/312
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "Floor, summed live from this session's transcript (cdca0400), 54 messages deduplicated by message.id, model claude-opus-5-5; in 110, cache creation 173,097 (1-hour), cache read 9,929,215, out 54,102. USD at Opus list rates $5/$25 per MTok with 1-hour cache writes. Re-sum after the session for the final figure. Duration is the transcript's timestamp span."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-22
---

# HANDOFF-182: Malformed interpretation tags refuse at develop

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-087` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle.

## Context the Receiving Agent Needs

Read these, in this order, before writing code:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-087-malformed-interpretation-tags-refuse-at-develop.md`.
  The Acceptance Criteria, the Failing Tests and the rule-clause map are the contract.
- **Decision:** `./decisions/DEC-078-a-malformed-interpretation-tag-the-develop-path-consumes-refuses-the-develop.md`.
  The whole rule: the four paths and their tag sets, the repeat-dimension exception, the
  malformed `DNGVersion`, the order, the new variant.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-027-honour-or-refuse-what-we-parse.md`.
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`.
- **Audit:** `docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md` §1 (the source).
- **`AGENTS.md`**: build, test and lint commands, coding and testing conventions.

### Relevant decisions

- `DEC-078` — this spec's rule.
- `DEC-053` — the precedent: unreadable opcode lists refuse at develop; the public
  `check_required_processing` and `DevelopPath` for composers.
- `DEC-077` — SPEC-086's checks, which run first; the malformed-50707 case is the model for
  50706.
- `DEC-012` — the tolerant read, amended by DEC-078 (the note is already in DEC-012).
- `DEC-051` — append the variant at the end; `kind()` has no wildcard; `error_kind.rs` and
  DEC-051 updated together.
- `DEC-058` §3 — an option never skips a tag's validation (the repeat-dimension exception
  holds under both `BlackLevels` options).
- `DEC-076` §3 — the singular unused matrix, confirmed unchanged by DEC-078 §4.

### Relevant constraints

- `no-panics-on-untrusted-input` — the check reads `malformed_tags` and `black_levels`
  only; no indexing.
- `oracle-must-be-shown-red` — AC9's mutants (a) to (i), each shown red then reverted,
  with a green control run first.

### Prior work

- `PR 309` / `HANDOFF-180` and `HANDOFF-181` — SPEC-086. The verify found that fixtures
  using only the easy shape left mutants alive (P1, P2). AC1 here is generated as a cross
  product from one (tag, sets) list for that reason: keep it that way.

## Expected Deliverables

- A PR against `main` from branch `feat/spec-087-malformed-tags`.
- Write the failing tests first (`tests/malformed_tags.rs`, the `error_kind.rs` counts),
  see them red, then implement.
- The Outputs list in the spec: `src/opcode.rs`, `src/demosaic.rs`, `src/color.rs`,
  `src/lib.rs`, the tests, DEC-051's row, `CHANGELOG.md` `[Unreleased]`.
- AC9's red-proof table in the handback: each mutant, the test that went red, and the
  green control.
- Gates before every push: `just test` (no extra arguments; it already passes
  `--all-features`) with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  so AC7's Tier-B pins run (the Pentax K-3 III Monochrome must keep developing),
  `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just validate`.
- No re-pin and no `DEVELOP_VERSION` change. If a held file refuses or a Tier-B pin moves,
  stop and report it in the handback: DEC-078's corpus measurement says neither happens.

## Out of Scope

- Changing DEC-012's read rule.
- Refusing a well-formed singular unused matrix.
- Audit ranks 8, 9, 11 and 12.
- Re-wording existing errors other than adding the 50706 `Display` arm.
- Commit-message bodies with code tokens: the repo ruleset rejects them. Plain prose only.

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

- **Branch / PR:** `feat/spec-087-malformed-tags`, https://github.com/jysf/irradiance/pull/312
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** yes. AC1 to AC6 and AC8 are in `tests/malformed_tags.rs` and `tests/error_kind.rs`. They were written first and seen red (they did not compile: no `MalformedTag`, no `DevelopPath::Demosaic` or `ColourTransform`), then turned green. AC7: `IRRADIANCE_CORPUS_DIR=… just test` gave 845 passed and 0 failed, with the corpus present. No Tier-B pin moved. The Pentax K-3 III Monochrome develops on every pin that develops it, and `DEVELOP_VERSION` is untouched. `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just validate` and `just doc-check` are all green.

**AC9 red-proof.** The green control ran first: `malformed_tags` 9/9 and `black_levels` 12/12, under the corpus profile. Each mutant was applied, run, and reverted with `git checkout`. `src` was confirmed clean afterwards.

| Mutant | Test that went red |
|---|---|
| (a) the malformed check not called | `ac1_each_path_refuses_its_own_malformed_set_on_every_entry` |
| (b) `demosaic_into` left on `Colour` | `ac1_a_tag_outside_the_path_set_behaves_as_absent` |
| (c) `to_srgb_linear_into` left on `Colour` | `ac1_a_tag_outside_the_path_set_behaves_as_absent` |
| (d) repeat-dimension exception dropped | `ac2_a_malformed_repeat_dim_with_one_black_value_develops`; AC7: `black_levels::one_by_one_files_are_identical_under_both_modes` (the K-3 III, Tier B) |
| (e) exception taken as any count | `ac2_a_malformed_repeat_dim_with_four_black_values_refuses` |
| (f) `CameraCalibration2` removed from the colour set | `ac1_each_path_refuses_its_own_malformed_set_on_every_entry` |
| (g) check placed after the opcode lists | `ac3_order_dec077_then_malformed_then_opcode_lists` |
| (h) malformed-50706 case dropped | `ac4_a_malformed_dng_version_without_a_backward_version_refuses` |
| (i) a write to `dst` before the check in `demosaic_into` | `ac1_each_path_refuses_its_own_malformed_set_on_every_entry`, on the sentinel assertion |

### Cost self-report

- **Tokens (total):** 10,156,524 (floor, re-summed live)
- **Estimated USD:** 8.05
- **Duration (minutes):** 13 (transcript timestamp span)
- **Source of the number:** this session's transcript (`cdca0400-…jsonl`), deduplicated by `message.id`, largest `output_tokens` per message, no subagents

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The check takes the lowest malformed tag across the path's sets, not the first in walk order. The sets' numbers interleave (Orientation 274, CFA 50710/50711, the rest of the levels set 50713 to 50829), and this keeps "ascending" exact for `Colour`.
  - AC1's row for `BlackLevelRepeatDim` carries four `BlackLevel` values so the exception does not apply. Its behaves-as-absent comparator has the same four values and no repeat dimension.
  - "Absent" for `ForwardMatrix1` is the CFA fixture with its `ForwardMatrix1` removed. `CFAPlaneColor` cannot be omitted by the test writer, so its comparator keeps `[0, 1, 2]`, which is also its DNG default.
  - The two calibration signatures are written malformed as `SHORT`, because `ASCII` is their valid type.
  - AC2's option runs: the option-taking entries (`develop_with_into`, `develop_with_scratch_into`) run under both `BlackLevels` options. The other entries take no options and run once.
  - `unhonoured_tag_name` is renamed `tag_name`, since it now serves both variants.
- **Follow-up work identified:** none new.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The spec did not say which entries AC2's "under both options" applies to (only two entries take options), or how to make `CFAPlaneColor` absent with a writer that always emits it. Both are settled as described above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The notes on the signatures (ASCII is their valid type) and on Orientation (IFD0 only) were enough.

3. **If you did this task again, what would you do differently?**
   — I would match each mutant pattern against the source before running the script. Mutant (c)'s pattern also matched a doc comment, and a rerun was needed.
