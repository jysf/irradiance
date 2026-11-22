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
  id: HANDOFF-180
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-086

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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 15838413           # REAL combined count — what cost-audit reads
  estimated_usd: 11.67             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 51
  branch: feat/spec-086-unhonoured-tags
  pr: https://github.com/jysf/irradiance/pull/309
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "Floor, summed live from this session's transcript (ad9e0a16), build plus the verify punch-list fix, 78 messages deduplicated by message.id; in 160, cache creation 220,412, cache read 15,550,093, out 67,748. The punch-list fix is 2,886,509 of it (about $1.85). USD at Opus list rates $5/$25 per MTok with 1-hour cache writes; re-sum after the session for the final figure. Duration is the transcript's timestamp span."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-22
---

# HANDOFF-180: Refuse tags the develop cannot honour yet

## Delegation Summary

`claude-opus-5-5` (acting as architect) hands `SPEC-086` to `claude-opus-5-5` (acting
as implementer) for the **build** cycle.

## Context the Receiving Agent Needs

Read these, in this order, before writing code:

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-086-refuse-tags-the-develop-cannot-honour-yet.md`.
  The Acceptance Criteria, the Failing Tests and the rule-clause map are the contract.
- **Decision:** `./decisions/DEC-077-the-develop-refuses-tags-that-change-sample-meaning-and-a-newer-dng-backward-version.md`.
  The whole rule: what is read, the honoured / refused table, the version default, the
  decode-time precision check, the three variants and their kinds.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-027-honour-or-refuse-what-we-parse.md`.
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`.
- **Audit:** `docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md` §3 and §4 (the
  source of the tag list).
- **`AGENTS.md`**: build, test and lint commands, coding and testing conventions.

### Relevant decisions

- `DEC-077` — this spec's rule.
- `DEC-051` — `Error` variants append at the end; `kind()` has no wildcard;
  `tests/error_kind.rs` and DEC-051 itself are updated together.
- `DEC-053` — the precedent: refuse at develop before writing, and leave
  `DEVELOP_VERSION` alone when only refusals change.
- `DEC-012` — absent is `None`; malformed is `None` plus `malformed_tags`. DEC-077 §2 then
  refuses a malformed tag of its own set.
- `DEC-049` — `Sensor` is `#[non_exhaustive]`, so the new fields are additive.

### Relevant constraints

- `no-panics-on-untrusted-input` — the new reads and the check go through the existing
  fallible readers; no indexing that can panic on a hostile count.
- `oracle-must-be-shown-red` — AC9's mutants (a) to (h), each shown red then reverted,
  with a green control run first.

### Prior work

- `PR 301` — `UnsupportedCfaLayout`, `UnsupportedCfaPlaneColor`, the opcode-version guard:
  the same refusal class.
- `HANDOFF-178` — the previous STAGE-027 build, for the handback's shape.

## Expected Deliverables

- A PR against `main` from branch `feat/spec-086-unhonoured-tags`.
- Write the failing tests first (`tests/unhonoured_tags.rs`, the AC5 precision tests, the
  `error_kind.rs` counts), see them red, then implement.
- The Outputs list in the spec: `src/ifd.rs`, `src/opcode.rs`, `src/plane.rs`,
  `src/lib.rs`, the tests, DEC-051's rows, `CHANGELOG.md` `[Unreleased]`, and the
  conformance matrix if it lists tag support.
- AC9's red-proof table in the handback: each mutant, the test that went red, and the
  green control.
- Gates before every push: `just test` (no extra arguments; it already passes
  `--all-features`) with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  so AC7's Tier-B pins run, `just lint-ci`, `just lint-no-allow`, `just fmt-check`.
- No re-pin and no `DEVELOP_VERSION` change. If a held file refuses or a Tier-B pin moves,
  stop and report it in the handback: DEC-077's corpus measurement says neither happens.

## Out of Scope

- Applying `LinearizationTable` or `BlackLevelDelta*`.
- SPEC-087's general malformed-interpretation rule (black, white, area, crop, orientation,
  colour, CFA).
- Audit ranks 8, 9 and 11.
- Refusing the seven tags at plane read; only the precision check is a plane-read refusal.
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

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-086-unhonoured-tags`, https://github.com/jysf/irradiance/pull/309
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** yes.
  - AC1 to AC4, AC6: `tests/unhonoured_tags.rs`, every check on all nine develop entries
    (mono entries on a mono fixture, the rest on RGGB), sensor in a SubIFD under a
    preview IFD0.
  - AC5: `tests/ljpeg_precision_binding.rs`, the three decode paths × strip and tile ×
    both byte orders.
  - AC7: the full corpus `just test` exits 0 (836 passed, 0 failed). Only the two
    optional SDK cross-checks skip. No held file refuses, no Tier-B pin moved, and
    `DEVELOP_VERSION` is still `"8"`.
  - AC8: `tests/error_kind.rs` at 75 variants, 17 / 26 / 32, with DEC-051's paragraph
    updated.
  - Red first: the new tests did not compile before the variants and fields existed.
    With the fields read but no check wired, AC1, AC3, AC5 and AC6 failed with `Ok`.
    AC5 decoded a P = 16 frame tagged 12-bit to samples above 4095.
- **AC9 red-proof** (control first, each mutant applied by script, run, then the file
  restored byte for byte):

  | Mutant | Test | Result |
  |---|---|---|
  | control | `unhonoured_tags` (7), `ljpeg_precision_binding` (3) | green |
  | (a) check not called from `check_required_processing` | `ac1_every_entry_refuses_each_unhonoured_tag` | red: row a on `develop_into` got `Ok(())` |
  | (b) `SampleFormat` 3 honoured | `ac1_every_entry_refuses_each_unhonoured_tag` | red: row a got `Ok(())` |
  | (c) `>=` instead of `>` | `ac2_honoured_values_develop_byte_identical` | red: 1.7.1.0 refused |
  | (d) default ignores `DNGVersion` | `ac3_backward_version_defaults_from_dng_version` | red: DNGVersion 1.8.0.0 got `Ok(())` |
  | (e) `malformed_tags` refusal dropped | `ac6_a_malformed_unhonoured_tag_is_refused` | red: 339 as ASCII got `Ok(())` |
  | (f) zero-delta exemption dropped | `ac2_honoured_values_develop_byte_identical` | red: all-zero `BlackLevelDeltaH` refused |
  | (g) precision check removed (strip and tile) | `ac5_precision_must_equal_bits_per_sample_on_every_decode_path` | red: P16 / BPS12 decoded with `Ok` |
  | (h) a write to `dst` before the check in `develop_color_into_threaded` | `ac1_every_entry_refuses_each_unhonoured_tag` | red: sentinel assertion on `DevelopColorIntoThreaded` |

- **Gates:** `just test` (with the corpus), `just lint-ci`, `just lint-no-allow`,
  `just fmt-check`, `just validate` and `just doc-check` are all green.

### Cost self-report

- **Tokens (total):** 12,951,904 (a floor: summed live, so the handback commit's own
  messages are not in it).
- **Estimated USD:** 9.82 (input 136, cache creation 205,743, cache read 12,689,330,
  output 56,695; Opus $5 / $25 per MTok, 1-hour cache writes).
- **Duration (minutes):** 12 (transcript timestamp span).
- **Source of the number:** this session's transcript
  (`ad9e0a16-c10b-4af9-ba06-9b629dc10568.jsonl`), deduplicated by `message.id` with
  the largest `output_tokens`, per `docs/cost-tracking.md`. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - Read strictness DEC-077 §1 leaves open, chosen as follows. `SampleFormat` and
    `LinearizationTable` must be SHORT and non-empty, or they are malformed. The two
    versions must be BYTE with count exactly 4. `BlackLevelDeltaH/V` go through
    `rationals()`, so RATIONAL is accepted as well as SRATIONAL.
  - `SampleFormat`'s count is not checked against `SamplesPerPixel`. Every honoured
    value must be 1 whatever the count.
  - A malformed `DNGVersion` (with `DNGBackwardVersion` absent) is costed and recorded,
    then treated as absent: no check, no refusal. DEC-077 names only a malformed
    backward version as a refusal.
  - `docs/conformance-matrix.md` is unchanged. It has no per-tag support table.
  - `UnsupportedTag`'s `Display` names the tag through a small private lookup in
    `src/lib.rs`, with "tag" as the fallback.
- **Follow-up work identified:**
  - Decide whether a malformed `DNGVersion` with no backward version should refuse. It
    fits naturally into SPEC-087's general malformed rule.

### Verify punch list (HANDOFF-181, fixed in dbc2017)

- **P1, the interleave factors.** They were read through `scalar()`, which keeps the
  first of any number of values. So count 0 read as absent and `[1, 2]` read as 1;
  neither was recorded as malformed, and both developed. Each is now read by
  `single_short_or_long` in `src/ifd.rs`: exactly one SHORT or LONG, anything else costed
  into `malformed_tags`, so DEC-077 §2 refuses it.
  - Four cases added to `ac6_a_malformed_unhonoured_tag_is_refused` (count 0 and count 2
    for 50975 and 51157).
  - Red before the fix: "50975 with count 0: malformed_tags [] must name 50975". With
    the count-0 cases skipped, also "50975 with count 2: malformed_tags [] must name
    50975".
  - Green after the fix.
- **P2.** The verifier's `b2: SampleFormat = [1, 3]` row came in from 8c90c4d
  unchanged, and it is green.
- **Nit.** `UnsupportedTag { tag: 50707 }` now displays "DNGBackwardVersion (50707) is
  malformed, so the file's reader version is unknown", pinned in
  `unsupported_tag_display_names_the_tag`.
- **Gates.** `just test` with the corpus exits 0 (836 passed, 0 failed, only the two SDK
  cross-checks skip, no pin moved). `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`
  and `doc-check` are green.
- **Extra cost.** 2,886,509 tokens, about $1.85. The front matter's `tokens_total` and
  `estimated_usd` now hold the whole session (build plus fix): 15,838,413 and 11.67.
  Do not add the extra on top.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The type and count strictness of each new read was left to the build; I
   chose the strict SHORT / BYTE × 4 shapes DEC-077 lists. The malformed `DNGVersion`
   case was unstated.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The `type_complexity` clippy lint on test code (`lint-ci` covers tests) was
   the only gate surprise, fixed with a type alias.

3. **If you did this task again, what would you do differently?**
   — Write the mutant script before the first green run, so the control and the eight
   mutants run as one step straight after it.
