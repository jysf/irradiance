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
  id: HANDOFF-183
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-22
  status: pending                  # pending | accepted | completed | rejected

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

# HANDOFF-183: VERIFY: malformed interpretation tags refuse at develop (SPEC-087)

## Delegation Summary

You are the **verifier** for SPEC-087. Review it cold: you did not build it.

- **Under review:** PR 312 (`feat/spec-087-malformed-tags`, head `c8211c5`), built under
  HANDOFF-182.
- **Your branch:** `claude/spec-087-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-087 implements DEC-078:
- each develop path refuses, before writing, with `Error::MalformedTag { tag }` (`Corrupt`)
  when a tag it reads is in `malformed_tags`;
- `DevelopPath` gains `Demosaic` and `ColourTransform`, so demosaic checks only the CFA
  tags and `to_srgb_linear_into` only the colour tags;
- a malformed `BlackLevelRepeatDim` with one black value stays exempt (the Pentax K-3 III
  Monochrome);
- a malformed `DNGVersion` with no backward version refuses `UnsupportedTag { tag: 50706 }`;
- `Error` goes from 75 to 76 variants; `DEVELOP_VERSION` is unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-087-malformed-interpretation-tags-refuse-at-develop.md`.
  Judge the ACs against the spec **as designed in PR 311**
  (`git show origin/main:<spec path>`), not as edited on the build branch.
- **Decisions:** DEC-078 (the rule and its sets), DEC-053 (the precedent and the public
  composer check), DEC-077 (SPEC-086's checks, which run first), DEC-012 (the tolerant
  read and its amendment note), DEC-051 (variant order, no wildcard), DEC-058 §3 (options
  never skip validation), DEC-076 §3 (the singular unused matrix, unchanged).
- **Build handback:** HANDOFF-182's `## Handback`: the AC9 mutant table and six
  deviations. Check each claim yourself.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, with touched sources, and read each
    panic message. (The SPEC-086 verify once ran a stale binary by reusing a target dir.)

## What to check hard

1. **The sets against DEC-078 §2, and against what each path really reads.** For each of
   the four paths, list the tags the code checks and compare them with the table. Then
   read the code each entry runs and confirm that every interpretation tag it reads is in
   its set, or is already refused another way. Look in particular for:
   - a colour entry reading `white_levels`, which is not costed, before the check;
   - `MaskedAreas` or another costed tag that some entry reads after all;
   - demosaic or the transform reading a level, geometry or orientation tag.
2. **The path split.** `demosaic_into`, `demosaic_into_threaded` and `to_srgb_linear_into`
   pass their new variants. The four colour develops keep `Colour`. Both new variants
   behave exactly as `Colour` for the opcode lists: show a mandatory opcode refusing the
   same way on each. Are the new variants additive for `semver-check`?
3. **The repeat-dimension exception.** It holds with exactly one `BlackLevel` value under
   both `BlackLevels` options on every mono and colour entry, and refuses with two or more.
   What does "one value" mean in code: `black_levels.len()`, the scalar, or both? Try a
   malformed `BlackLevel` together with a malformed repeat dimension. Does the K-3 III
   still develop on every held pin?
4. **Order.** DEC-077, then the malformed check in ascending tag order, then the opcode
   lists. **Rule on the build's deviation:** it reports the lowest malformed tag across the
   path's sets rather than the first in walk order. Is that what DEC-078 §3's "ascending
   tag order" means, and is it tested where the sets interleave (Orientation 274 against
   the CFA tags 50710/50711 on `Colour`)?
5. **50706.** It refuses on every path with no backward version present, and develops when
   a well-formed backward version is present. Its `Display` wording.
6. **The AC1 cross product.** Is it really generated from one (tag, sets) list, so that
   adding a tag to a set without a row fails? Does "behaves as absent" compare real bytes
   or errors on the entries outside the set? Rule on the fixture deviations: the
   four-black-value repeat-dimension row, `ForwardMatrix1` absent, `CFAPlaneColor`'s
   comparator, and the signatures written as `SHORT`. Does each malformed fixture really
   land in `malformed_tags` (the reader may accept several types)?
7. **Already-refused tags keep their errors (AC5).** Compare each error with `origin/main`
   on the same fixture.
8. **The composer check (AC6).** `check_required_processing(sensor, path)` called directly
   agrees with the entry for each path.
9. **Nothing held changes (AC7).** Run `just test` with the corpus. Confirm that no Tier-B
   pin moved (`git diff origin/main -- tests/`), that `DEVELOP_VERSION` is `"8"`, and,
   independently, that `irr ifd` shows `malformed_tags` empty on every held DNG except
   the K-3 III's `[50713]`.
10. **Errors (AC8).** `MalformedTag` is appended last, `Corrupt`, with an explicit `kind()`
    arm. `error_kind.rs` is 76 / 33 / 26 / 17, and DEC-051 is updated. `Display` names each
    tag and has a fallback. The `unhonoured_tag_name` → `tag_name` rename is private.
11. **Mutants.** Re-run the build's (a) to (i). Then try at least:
    - the exception keyed on the scalar `black_level` instead of the value count;
    - `ColourTransform` given the levels set;
    - `Demosaic` given an empty set;
    - one tag removed from the levels set (for example `DefaultCropSize`);
    - the 50706 case refusing even when a backward version is present;
    - walk order instead of lowest tag.

    Record what goes red, or argue equivalence.
12. **Release surface.** The CHANGELOG `[Unreleased]` entry states the refusal, the new
    variants and the entry-path change accurately. `semver-check` is additive.
13. **The build's cost figure.** The handback reports 10.16M tokens over 13 minutes. Note
    whether that looks plausible; the orchestrator re-sums it at ship.
14. **Gates:**
    - `just test` with the corpus;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
      `semver-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants.
- **A ruling on item 4's ordering deviation and item 6's fixture deviations.**
- **Commit this handoff's filled handback** to `claude/spec-087-verify-base` and push it.
  Do not open a PR, and do not push to `feat/spec-087-malformed-tags`.
- **Fixes:** a punch-list fix is fine on your branch if it is tests, scripts or docs only.
  Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

## Out of Scope

- Changing DEC-012's read rule.
- Refusing a well-formed singular unused matrix (DEC-078 §4).
- Audit ranks 8, 9, 11 and 12.

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
