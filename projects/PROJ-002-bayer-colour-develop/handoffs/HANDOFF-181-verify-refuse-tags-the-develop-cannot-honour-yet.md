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
  id: HANDOFF-181
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-22
  status: pending                  # pending | accepted | completed | rejected

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

# HANDOFF-181: VERIFY: refuse tags the develop cannot honour yet (SPEC-086)

## Delegation Summary

You are the **verifier** for SPEC-086. Review it cold: you did not build it.

- **Under review:** PR 309 (`feat/spec-086-unhonoured-tags`, head `258eee9`), built under
  HANDOFF-180.
- **Your branch:** `claude/spec-086-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-086 implements DEC-077:
- nine new `Sensor` fields (`DNGVersion`, `DNGBackwardVersion`, `SampleFormat`,
  `LinearizationTable`, `BlackLevelDeltaH/V`, `SubTileBlockSize`, the two interleave
  factors);
- a develop-time refusal inside `opcode::check_required_processing`: the backward-version
  guard (above 1.7.1.0, defaulting from `DNGVersion`) then the seven tags in ascending
  order, with exact no-op values honoured and a malformed tag refused;
- a decode-time refusal of a Compression 7 frame whose precision is not `BitsPerSample`;
- three `Error` variants (72 to 75), `DEVELOP_VERSION` unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-086-refuse-tags-the-develop-cannot-honour-yet.md`.
  Judge the ACs against the spec **as designed in PR 308**
  (`git show origin/main:<spec path>`), not as edited on the build branch.
- **Decisions:** DEC-077 (the rule), DEC-051 (variant order, `kind()` with no wildcard),
  DEC-053 (refuse before writing; refusals leave `DEVELOP_VERSION`), DEC-012 (malformed
  costs its field), DEC-049 (`Sensor` is `#[non_exhaustive]`).
- **Build handback:** HANDOFF-180's `## Handback`: the AC9 mutant table and five
  deviations. Check each claim yourself.
- **Specification:** DNG 1.7.1.0 for `DNGBackwardVersion` (and its default), and the
  seven tags. If you have no copy, say so; do not guess at wording.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **Every develop entry refuses before writing.** Confirm each of the nine AC1 entries
   reaches the check before any write to `dst`, including the threaded colour plans and
   `develop_with_scratch_into`. Is AC1's sentinel assertion on every entry, and does
   mutant (h) really go red? Are there public develop or demosaic entries the spec missed?
2. **The version guard against the specification.**
   - The byte-wise comparison with `[1, 7, 1, 0]`, and the boundary (1.7.1.0 develops,
     1.7.1.1 refuses).
   - The default: absent backward takes `DNGVersion` with its last two bytes zeroed.
   - **Rule on the build's deviation:** a malformed `DNGVersion` with no backward version
     is recorded and then skipped, so no guard runs. Is that acceptable under DEC-077,
     or should it refuse `UnsupportedTag { tag: 50706 }`? Give a ruling and, if it
     should refuse, put it on the punch list (it is a `src/` change).
3. **The tag table.** For each of the seven tags: the refused value, the honoured no-op,
   and a malformed value. Are the honoured values really byte-identical (AC2) on every
   entry? Is the ascending order right (rows j and k)?
4. **The read strictness deviations.** `SampleFormat` and `LinearizationTable` must be
   SHORT and non-empty; the versions BYTE × 4; the deltas accept RATIONAL as well as
   SRATIONAL; `SampleFormat`'s count is not tied to `SamplesPerPixel`. Rule on each. An
   empty `LinearizationTable` (count 0): malformed and refused, or not? A `SampleFormat`
   as LONG (TIFF 6.0 says SHORT)?
5. **No hostile-input panics.** The new reads go through bounded readers
   (`MAX_TAG_VALUES`). A huge `LinearizationTable` or delta count is bounded. A zero
   denominator in a delta is malformed, not a division. Check what the `ifd` fuzz target
   now exercises.
6. **Decode-time precision (AC5).** On the strip path, `unpack_unit_into`,
   `PlaneDecoder::unpack_unit_into` and the tile path. Order after the shape check. Is
   `validate` or `units` affected, and should it be?
7. **Metadata and plane reads still succeed (AC4)**, with the new fields holding the
   written values, and `irr ifd` working on a refused file.
8. **Nothing held changes (AC7).** Run `just test` with the corpus. Confirm no Tier-B pin
   moved (`git diff origin/main -- tests/`) and `DEVELOP_VERSION` is still `"8"`.
   Independently confirm, on the held files, that no SOF-3 precision differs from
   `BitsPerSample` and no tag of the set has a refused value.
9. **Errors (AC8).** The three variants are appended at the end in DEC-077 §4's order,
   with the right kinds; `error_kind.rs` is 75 / 26 / 32 / 17; DEC-051 is updated.
   `UnsupportedTag`'s `Display` for each tag, and its fallback.
10. **Mutants.** Re-run the build's (a) to (h). Then try at least:
    - the version default zeroing the wrong bytes;
    - the version guard placed after the tag set;
    - the tag set in descending order;
    - `SampleFormat` honoured when any (rather than every) value is 1;
    - a delta refusal reading only the first entry;
    - the precision check on the strip path only.

    Record what goes red, or argue equivalence.
11. **The files outside the plan.** `src/color.rs` and `src/develop.rs` gain nine `None`
    fields each. Confirm that is all, and that no existing assertion was weakened
    (`git diff origin/main -- tests/ src/`).
12. **Release surface.** The CHANGELOG `[Unreleased]` entry names the refusals and the
    new fields accurately. `semver-check` is additive.
13. **The build's cost figure.** The handback reports 12.95M tokens over 12 minutes.
    Note whether that looks plausible; the orchestrator re-sums it at ship.
14. **Gates:**
    - `just test` with the corpus;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
      `semver-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants.
- **A ruling on item 2's malformed-`DNGVersion` deviation and on item 4's strictness
  choices.**
- **Commit this handoff's filled handback** to `claude/spec-086-verify-base` and push it.
  Do not open a PR, and do not push to `feat/spec-086-unhonoured-tags`.
- **Fixes:** a punch-list fix is fine on your branch if it is tests, scripts or docs
  only. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

## Out of Scope

- SPEC-087's general rule for malformed interpretation tags.
- Applying `LinearizationTable` or `BlackLevelDelta*`.
- Audit ranks 8, 9 and 11.

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
