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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7184428           # REAL combined count — what cost-audit reads
  estimated_usd: 5.67              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 10
  branch: claude/spec-087-verify-base
  pr: null
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "Floor, re-summed live from transcript 9084a071 before the commit. 1 verify round: PUNCH LIST, P1 tests-only and done on this branch (interleaved-set AC3 case; the walk-order mutant was green at c8211c5). Build transcript cdca0400 re-sums to 11,571,180."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
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

### Execution notes

- **Branch / PR:** `claude/spec-087-verify-base` (no PR, as asked). Reviewed PR 312's head `c8211c5`.
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** Yes for AC1 to AC9 as written, but the ordering
  deviation that handoff item 4 asks about was untested. A walk-order mutant stayed green.
  The fix is tests-only (P1 below), and it is done on this branch.
- **For `verify`:** ⚠ **PUNCH LIST**. There is one item (P1). It touches tests only and is
  done here. Nothing in `src/` needs to change. Once P1 is cherry-picked onto
  `feat/spec-087-malformed-tags`, round 2 can approve at that SHA.

**Punch list**

- **P1 (tests, done on this branch):** no test separated "lowest tag across the path's
  sets" from "first tag in walk order". A mutant that returns the first malformed tag in
  walk order (`find` instead of `min`) left all 9 `malformed_tags` tests green. AC3 now
  has an interleaved case, a malformed `BlackLevel` (50714) with a malformed `CFALayout`
  (50711). It expects 50711 on the four colour develops and both demosaic entries, 50714
  on the three mono entries (their fixture has no CFA), and no `MalformedTag` on the
  transform. The walk-order mutant now goes red on
  `ac3_order_dec077_then_malformed_then_opcode_lists`. Cherry-pick this branch's
  `tests/malformed_tags.rs` change onto the feat branch. The handoff's suggested
  Orientation-against-CFA pair would not have separated the two orders, because 274 is
  first either way.

**Rulings**

- **Ordering deviation (item 4): accepted.** DEC-078 §3 says the path's check runs "in
  ascending tag order", and SPEC-087 Design 2 says it "returns the first tag" of that
  walk. On `Colour` the path's tags are the union of three sets whose numbers interleave.
  The only reading under which the reported tag does not depend on how the sets are
  listed is the lowest tag of the union, which is the `min()` the build chose. It was
  untested, which is why P1 exists.
- **Fixture deviations (item 6): all four accepted.**
  - The four-value `BlackLevelRepeatDim` row is needed so the tag counts at all. Its
    behaves-as-absent comparator has the same four values and no repeat dimension, so
    the comparison is fair.
  - `ForwardMatrix1` absent means really absent: the CFA base's matrix is removed in
    both fixtures.
  - `CFAPlaneColor`'s comparator `[0, 1, 2]` is the DNG default, so it is
    semantically the absent tag. The writer cannot omit the tag.
  - The two signatures are written as `SHORT` because `ASCII` (and `BYTE`) are their
    valid types.

  Every malformed fixture lands in `malformed_tags`. AC1, AC5 and AC6 assert this on
  every row, and the tests pass.
- **AC2 options deviation: accepted.** Only `develop_with_into` and
  `develop_with_scratch_into` take `DevelopOptions`. The other levels entries run once,
  under the default.
- **`unhonoured_tag_name` renamed to `tag_name`: accepted.** It stays private (`fn`, no
  `pub`).

**Evidence by handoff item**

1. **Sets against the reads.** The costed tags in `sensor()` (src/ifd.rs) are these: the
   levels set (7 tags), the four CFA tags, the 11 colour tags, `AsShotNeutral`, the three
   opcode lists, `MaskedAreas`, the version pair, and DEC-077's set. Each one is in a
   DEC-078 set or refused another way. Each path reads only its own set:
   - Mono (`develop.rs`) reads only the levels and geometry fields, plus the opcode lists
     and dimensions.
   - `demosaic.rs` reads only `color_filter` (pattern, repeat dimension, layout, plane
     colour) and dimensions.
   - `to_srgb_linear_into` reads `as_shot_neutral`, the colour tags and dimensions. It
     reads no level, geometry or orientation tag.
   - The colour develop reads `white_levels`, which is not costed, but only after
     `check_colour_required_processing`, and a malformed `WhiteLevel` is already in
     `malformed_tags` through `white_level`.

   No develop reads `MaskedAreas`; outside `ifd.rs` it appears only in struct literals.
2. **The path split.** The two demosaic entries pass `Demosaic`, and `to_srgb_linear_into`
   passes `ColourTransform`. The four colour develops keep `Colour` through
   `check_colour_required_processing`. `applied()` matches only `Mono` explicitly, so both
   new variants get `Colour`'s opcode arms. The held `tests/required_processing.rs`
   already shows all three primitives refusing a mandatory list-1 opcode as
   `UnsupportedRequiredOpcode { list: 1, id: 4 }`, the same as the colour develops.
   `semver-check` reports 202 checks passed and "no semver update required". The change
   is additive (`#[non_exhaustive]`).
3. **The exception.** In code, "one value" is `sensor.black_levels.len() > 1`, the parsed
   value count, not the scalar. I ran a probe on all 7 levels entries, under both
   options where an entry takes them:
   - A malformed `BlackLevel` with a malformed repeat dimension refuses 50714. The count
     is then 0, so the repeat dimension is exempt, and `BlackLevel` refuses on its own.
   - A malformed repeat dimension with `BlackLevel` absent develops.

   The K-3 III develops on every held pin: `just test` is green, and mutants (d) and (j)
   turn `black_levels::one_by_one_files_are_identical_under_both_modes` red on that file.
4. **Order.** DEC-077 runs first, then the malformed check, then the opcode lists. This is
   pinned by AC3 and by mutant (g). For the deviation ruling, see above and P1.
5. **50706.** It refuses on all 10 entries with no backward version. It develops on all
   10 when the backward version 1.4.0.0 is present. Its `Display` names `DNGVersion` and
   50706 and says "reader version is unknown". Mutants (h) and (n) are both red.
6. **AC1 cross product.** The test generates it from `TAG_SETS`. The outside-the-set rows
   compare real `dst` bytes, as `f32` bits for the f32 entries, or the `Debug` of the
   error.

   "A tag added to a set without a row fails" holds in effect, not by construction.
   `TAG_SETS` is the test's own list, so adding a tag to a set in `src/` is caught by
   that tag's own test: AC5 covers `CFAPattern`, `CFARepeatPatternDim`, `AsShotNeutral`,
   the opcode lists and `MaskedAreas`, and AC4 covers `DNGVersion`. Removing a tag from a
   set is caught by the cross product: mutants (f) and (m) are red.
7. **AC5 against `origin/main`.** I extracted `ac5_already_refused_tags_keep_their_errors`
   and ran it against `origin/main`'s `src/`. It passes there, so the errors are
   identical.
8. **AC6.** The direct `check_required_processing` call agrees with each entry. Mutants
   (a), (b), (c), (e), (f), (k), (l) and (m) all turn `ac6_reads_and_the_composer_check_agree`
   red as well.
9. **AC7.**
   - `IRRADIANCE_CORPUS_DIR=… just test`: 845 passed, 0 failed, 1 ignored. The corpus is
     12/12 present. The two SDK cross-checks skip, as they always have.
   - `git diff origin/main -- tests/` touches only `tests/malformed_tags.rs` (new) and
     `tests/error_kind.rs`, so no Tier-B pin moved.
   - `DEVELOP_VERSION` is `"8"`, and nothing in `src/` changes it.
   - `irr ifd` on all 11 held DNGs: `malformed_tags` is `[]` on every file except
     `PENTAX-K3III-MONO/K3III.DNG`, which has `[50713]`.
10. **Errors.**
    - `MalformedTag` is the last variant, `Corrupt`, with an explicit `kind()` arm.
    - `error_kind.rs` reads 76 / 33 / 26 / 17 and passes.
    - DEC-051 has the row.
    - `Display` names every DEC-078 §2 tag, and the fallback is "tag (N)". Both are
      tested.
    - `tag_name` is private.
11. **Mutants.** Every copy had its own fresh `CARGO_TARGET_DIR` and touched sources. The
    test command was `cargo test --all-features --profile corpus --no-fail-fast` on
    `malformed_tags`, `black_levels` and `error_kind`, with the corpus. The green control
    ran first: 9, 12 and 5 passed. I read every panic message, and each one shows the
    intended reason.

| Mutant | Result | Red on (panic reason) |
|---|---|---|
| (a) check not called | red | ac1_each, ac2_four, ac3, ac6 ("expected MalformedTag { tag: 274 }, got Ok") |
| (b) `demosaic_into` on `Colour` | red | ac1_a_tag_outside, ac3, ac6 (DemosaicInto gets MalformedTag 274 / 50717) |
| (c) `to_srgb_linear_into` on `Colour` | red | ac1_a_tag_outside, ac3, ac6 |
| (d) exception dropped | red | ac2_one_value; Tier B `black_levels::one_by_one_files_are_identical_under_both_modes` (K3III 50713) |
| (e) exception for any count | red | ac1_each, ac2_four, ac6 |
| (f) `CameraCalibration2` out of the colour set | red | ac1_each, ac6 |
| (g) check after the opcode lists | red | ac3 ("got UnreadableOpcodeList { list: 2 }") |
| (h) 50706 case dropped | red | ac4 ("DevelopInto: Ok(())") |
| (i) write before the check in `demosaic_into` | red | ac1_each on the sentinel, ac3, ac4, ac5 |
| (j) exception keyed on the scalar `black_level.is_some()` | red | ac2_one_value; Tier B K3III |
| (k) `ColourTransform` given the levels set | red | ac1_a_tag_outside, ac3, ac6 |
| (l) `Demosaic` given an empty set | red | ac1_each, ac6 ("50710 … got Ok") |
| (m) `DefaultCropSize` out of the levels set | red | ac1_each, ac6 |
| (n) 50706 refused even with a backward version | red | ac4 ("backward 1.4.0.0 … UnsupportedTag 50706") |
| (o) walk order (`find`) instead of lowest (`min`) | **green at `c8211c5`**; red after P1 | ac3 ("interleaved sets on DevelopColorInto: expected 50711, got 50714") |

After P1 the control was re-run green, and (o) is red.
12. **Release surface.** The CHANGELOG `[Unreleased]` entry states the refusal, the lowest
    tag, the three sets, the K-3 III exception, the two new variants, the entry-path
    change and the 50706 refusal, all accurately. `semver-check` is additive.
13. **The build's cost.** Its transcript (`cdca0400…`) re-sums to 11,571,180 tokens over
    60 deduplicated messages and 13.2 minutes on `claude-opus-5-5`. That is consistent
    with the reported 10,156,524, which was a live floor, so the figure is plausible.
14. **Gates.** `just test` with the corpus, `lint-ci`, `lint-no-allow`, `fmt-check` (re-run
    after P1), `validate`, `doc-check`, `cost-audit`, `gate-audit`, `name-length-check`,
    `decisions-index --check` and `semver-check` are all green.

### Cost self-report

- **Tokens (total):** see the `handback:` front matter. That figure is re-summed just
  before the commit, so it is a floor.
- **Estimated USD:** see the front matter. Each component is priced at Opus 5.5's rates:
  input $5/M, output $25/M, cache read $0.50/M, and cache write (1-hour) $10/M.
- **Duration (minutes):** see the front matter (the transcript's timestamp span).
- **Source of the number:** this session's own transcript (`9084a071-…jsonl`, matched by
  the scratchpad UUID). Lines are deduplicated by `message.id`, with the largest
  `output_tokens` per message. There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verifier. The build's deviations are ruled on
  above.
- **Follow-up work identified:**
  - P1: cherry-pick onto the feat branch before ship.
  - (nit, no action) `DevelopPath::applied` uses a `(_, 2)` wildcard, so a future
    variant inherits the colour path's `GainMap` arm silently. This is fine for the two
    variants here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The handoff's example for the interleaved case (Orientation 274 against the CFA
   tags) cannot tell walk order from lowest-tag order, because 274 comes first either
   way. A levels tag above 50711 is needed.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — I would write the walk-order mutant before reading the tests. It was the one
   question the build's AC9 table did not cover, and it was the one gap.
