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
  id: HANDOFF-186
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-23
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-089

project:
  id: PROJ-002
  stage: STAGE-029
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
  tokens_total: 10230218           # REAL combined count — what cost-audit reads
  estimated_usd: 7.69              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 16
  branch: claude/spec-089-verify-base
  pr: https://github.com/jysf/irradiance/pull/319
  completed_at: 2026-11-23         # YYYY-MM-DD
  notes: "Floor, summed live from this session transcript (8ef6d792), 58 messages deduplicated by message.id, model claude-opus-5-5; in 115, cache creation 168,356 (1-hour), cache read 10,021,976, out 39,771. USD at Opus list rates 5 and 25 dollars per MTok with 1-hour cache writes. Taken before the handback commit and push, so re-sum after the session. APPROVED at 727d471; V1 fixed there, tests only."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-23
---

# HANDOFF-186: VERIFY: empty and misshapen interpretation tags refuse at develop (SPEC-089)

## Delegation Summary

You are the **verifier** for SPEC-089. Review it cold: you did not build it.

- **Under review:** PR 319 (`feat/spec-089-misshapen-tags`, head `b2cc3b6`), built under
  HANDOFF-185.
- **Your branch:** `claude/spec-089-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-089 implements DEC-078 §7:
- a present interpretation tag is malformed (recorded in `malformed_tags`, field `None`)
  when its count is zero, when a fixed-count tag has another count, or when a colour
  rational has a zero denominator; ColorPlanes is the `CFAPlaneColor` count on a CFA plane
  (3 when absent or malformed) and `SamplesPerPixel` otherwise;
- DEC-078's existing check then refuses it (`MalformedTag`, or `UnsupportedTag` for
  DEC-077's set), with no change to the sets or the order;
- the colour code no longer falls back from a present matrix it cannot use
  (`dual_illuminant`, the ForwardMatrix branch, `resolve_calibration`);
- no new `Error` variant (76), `DEVELOP_VERSION` stays `"8"`, no Tier-B re-pin, and the
  public `Container::scalar` is unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-089-empty-and-misshapen-interpretation-tags-refuse-at-develop.md`.
  Judge the ACs against the spec **as designed in PR 318**
  (`git show origin/main:<spec path>`): ACs 1 to 9, the rule-clause map, the
  distinguishing fixtures.
- **Decisions:** DEC-078 §7 (the amendment this spec implements) and §2 to §4, DEC-077 §3
  (its set refuses as `UnsupportedTag`), DEC-076 §3 (the "only what is used is checked"
  rule that §7.2 reverses), DEC-012 and DEC-015 (tolerant read; absent is not recorded),
  DEC-051 (variant count).
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`.
- **Build handback:** HANDOFF-185's `## Handback`: the red-on-main table, the AC9 mutant
  table, and three deviations (four changed existing tests, `dual_illuminant` validating
  matrices before the illuminant check, a lone misshapen `ForwardMatrix1` now an error).
- **Prior related work:** SPEC-087 (HANDOFF-182 build, HANDOFF-183 verify) built the
  refusal and the `tests/malformed_tags.rs` machinery that `tests/misshapen_tags.rs` copies.
- **Toolchain:** AGENTS.md §5 and §6. Run `just test`, never `just test --all-features`.
  Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use
  `/usr/bin/git` to establish branch and HEAD. Give each mutant run its own source state
  and read the panic message.

## What to check hard

1. **Every AC met, every rule-clause row reaching a test where the output changes.**
2. **AC9 mutants.** Can each fail on its fixture? Re-run those in doubt, at least (d),
   (h), (i) and (j), plus any the spec's distinguishing fixtures make delicate.
3. **Red on main.** Run the new test file against `origin/main`'s `src/`.
4. **The four changed existing tests:** `tests/color_oracle.rs`
   `ac3_an_unused_ill_shaped_calibration_develops` renamed `..._refuses`; the
   zero-denominator `ColorMatrix2` case in `color::tests::degenerate_interpolation_inputs_do_not_panic`;
   and the two `ifd::tests` reader tests whose fixtures moved to a 3-plane CFA sensor.
   Each must follow from DEC-078 §7, not be edited to pass.
5. **No panics on untrusted input** in the new reader code (count arithmetic, ColorPlanes).
6. **`just decisions-audit --changed`.**
7. **DEC-076 amendment note.** §7.2 reverses DEC-076 §3 and nothing recorded it on DEC-076.
   Add a short note pointing at DEC-078 §7.2; run `just decisions-index`.
8. **Gates:** `just test` with the corpus, `lint-ci`, `lint-no-allow`, `fmt-check`,
   `doc-check`, `validate`; SDK lanes SKIPPED if unset; `gh pr checks 319`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including a table of the mutants you re-ran.
- **Commit** this handoff's filled handback and the DEC-076 note to
  `claude/spec-089-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-089 ship --verdict approved` and fast-forward
  `feat/spec-089-misshapen-tags` to the verify tip so PR 319 carries the verify commits.
  Do not merge.
- **Fixes:** tests or docs only on your branch; any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.

## Out of Scope

- An unknown `CalibrationIlluminant` code (audit rank 8), several sensor IFDs (rank 9),
  degenerate warp and gains (rank 11), rank 12.
- Extra values beyond the DNG count for `BlackLevel` and `WhiteLevel`.
- Refusing a well-formed singular matrix (DEC-078 §4).
- The 0.2.6 release.

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

- **Branch / PR:** `claude/spec-089-verify-base`, fast-forwarded onto `feat/spec-089-misshapen-tags` so https://github.com/jysf/irradiance/pull/319 carries the verify commits. Cut from PR 319's head `b2cc3b6` (checked with `/usr/bin/git`).
- **Completed at:** 2026-11-23
- **All acceptance criteria met?** yes. AC1 to AC9 hold. Three test gaps in the §7.1(b) ColorPlanes and the `WhiteLevel` count rule were closed here, tests only (below).
- **For `verify`:** ✅ APPROVED at `727d471` (the reviewed code plus the verify tests; the handback commit after it is docs only).

**1. ACs and the rule-clause map.** Each AC was checked against the spec as designed in PR 318 (`git show origin/main:<spec>`).

| AC | Evidence |
|---|---|
| AC1 | `ac1_the_reader_records_each_empty_or_misshapen_tag` runs all 29 tags (§2's three sets with 20 tags, plus DEC-077's set and the version pair). It covers count zero, both sides of every fixed count, and a zero denominator in the last entry. Each row checks: recorded exactly once, field equal to the absent read, `unpack_into` unchanged. The row count is asserted. `ac1_valid_and_absent_controls_are_not_recorded` covers the controls. `ac1_colour_planes_come_from_the_plane` covers ColorPlanes. |
| AC2 | A cross product of one `TAGS` table with every shape and all 10 entries (both `BlackLevels` options on the two option-taking entries). It asserts `MalformedTag` (DEC-077: `UnsupportedTag`) with dst and scratch untouched, or byte-equal to the absent control. |
| AC3 | Three-way check on a dual-illuminant CFA fixture: valid differs from absent, and the 8-entry and zero-denominator forms refuse 50722 on all five colour readers with dst untouched. The same for `ForwardMatrix2` (also 10 entries and count zero) and count-zero `CalibrationIlluminant2`. |
| AC4 | The three `color::tests` unit tests on hand-built sensors, each with an absent or valid control. |
| AC5 | Covers recovery (no record, value 6, output equal to the sensor-IFD-only control and different from unrotated) and count zero in both IFDs (recorded once, refuses on mono and colour). |
| AC6 | K-3 III shape and count-zero repeat-dim develop byte-identical to the control under both options. `AsShotNeutral` and `CFAPattern` keep their errors. An unknown illuminant code is byte-identical to single-illuminant. The opcode-list part is not re-asserted here; SPEC-087's tests hold it and `src/opcode.rs` is untouched. |
| AC7 | Exactly the four listed refusals, now `MalformedTag`. The CHANGELOG lists exactly these as changed refusals. |
| AC8 | Shown independently: `irr ifd` (built from origin/main and from this branch) on all 11 held DNGs gives identical `malformed_tags`, empty except K-3 III `[50713]`. `DEVELOP_VERSION` is `"8"`, `tests/error_kind.rs` is 76, and no Tier-B test or fixture changed (`git diff origin/main...HEAD -- tests/` touches only `color_oracle.rs` and the new file). |

Every rule-clause map row reaches a test where the output changes. The one weak row was "§7.1(b) ColorPlanes from `CFAPlaneColor`": it was exercised only through `ColorMatrix1`. See finding V1.

**2. Red on main's behaviour (re-run here).** `tests/misshapen_tags.rs` against origin/main's `src/` in a scratch copy: 7 of 9 red with the build's messages. Examples: `274 count zero on DevelopInto: … got Ok(())`, `50722 8 entries … got Ok(())`, `50965 8 entries … got Ok(())`, `50721 … got Err(MissingColorCalibration)`. The two green tests are the controls. The new verify test `ac1_colour_planes_size_the_calibration_and_the_balance` is also red on main. `ac1_a_multi_value_white_level_is_not_misshapen` is green on main, as a control pinning an existing rule should be.

**3. AC9 mutants (re-run here, all of them).** Each mutant was run in a scratch copy, with the pristine `src` restored before each. The green control ran first: lib 212, `color_oracle` 43, `misshapen_tags` 9.

| Mutant | Red on |
|---|---|
| (a) count zero back to `Ok(None)` (and "more than one" for the single-value tags) | `ac1_the_reader…`, `ac2…`, `ac5…`, `ac3…colour_matrix` |
| (b) length check removed | `ac1_the_reader…`, `ac1_colour_planes…`, `ac2…`, both `ac3`, `ac7` |
| (c) denominator check removed | `ac1_the_reader…`, `ac2…`, both `ac3` |
| (d) `dual_illuminant` `.ok()` on both ColorMatrix reads | `ac4_dual_illuminant…`, `degenerate_interpolation_inputs_do_not_panic` |
| (e) FM branch `.ok()` on both ForwardMatrix reads | `ac4_forward_matrix_error_propagates` |
| (f) `resolve_calibration` validates only the used calibration | `ac4_unused_calibration_is_validated`, `color_oracle::ac3_an_unused_ill_shaped_calibration_refuses` |
| (g) Orientation recovery dropped | `ac5…`, plus two existing `ifd::tests` Orientation tests |
| (h) "at least" length | `ac1_the_reader…`, `ac1_colour_planes…`, `ac2…`, `ac3…forward_matrix` |
| (i) ColorPlanes fixed at 3 | `ac1_colour_planes_come_from_the_plane` (and the new V1 test) |
| (j) zero denominator checked on the first entry only | `ac1_the_reader…`, `ac2…`, both `ac3` |
| extra: count-zero `BlackLevel` check removed | `ac1_the_reader…`, `ac2…` |
| extra: count-zero signature check removed | `ac1_the_reader…`, `ac2…` |
| extra: empty `CFAPlaneColor` check removed | `ac1_the_reader…`, `ac1_colour_planes…`, `ac2…`, `ac7` |
| extra: `SamplesPerPixel` ignored (non-CFA ColorPlanes fixed at 3) | `ac1_colour_planes_come_from_the_plane` |
| extra: `WhiteLevel` read as exactly one value | **survived** the build's tests. Red after V1 on `ac1_a_multi_value_white_level_is_not_misshapen` |
| extra: `AnalogBalance` count fixed at 3 | **survived**. Red after V1 on `ac1_colour_planes_size_the_calibration_and_the_balance` |
| extra: `CameraCalibration` count as 3 × ColorPlanes | **survived**. Red after V1 on the same test |

**4. The four changed existing tests.** Each follows from DEC-078 §7. None was edited just to pass.
- `tests/color_oracle.rs` `ac3_an_unused_ill_shaped_calibration_develops` → `…_refuses`. §7.2 explicitly forbids "identity for an unused calibration", which reverses DEC-076 §3. It is a hand-built-sensor test, so `MissingColorCalibration` is the right variant. Mutant (f) turns it red.
- `color::tests::degenerate_interpolation_inputs_do_not_panic` case (a). A zero-denominator `ColorMatrix2` used to fall back to single-illuminant colour, which §7.2 names ("neither single-illuminant colour"). It now asserts `MissingColorCalibration`, and the test's purpose (no panic) holds. Mutant (d) turns it red.
- `ifd::tests::srational_matrix_reads_signed_and_fractional_verbatim` and `the_five_calibration_tags_read_on_a_writer_fixture`. Their 9-entry matrices sat on a 1-sample LinearRaw plane, where §7.1(b) now requires 3 (and `AnalogBalance` 1). Moving them to the existing 3-plane `cfa_sensor` is the minimal change. Their assertions (verbatim signed and fractional values, NUL cut) are unchanged.

**5. Constraints.** `no-panics-on-untrusted-input`: the new reader code uses `checked_mul` for both counts, `usize::try_from(..).unwrap_or(usize::MAX)` (no count then fits), and no indexing. Every new `unwrap`/`expect` in `src/` is inside `#[cfg(test)]`. `lint-no-allow` is green. `oracle-must-be-shown-red`: see 2 and 3. No new `Error` variant; the public `Container::scalar` is unchanged apart from its doc sentence.

**6. `just decisions-audit --changed`** (and `--changed origin/main` for the whole branch) is advisory only. It names DEC-000 and DEC-004 for my changes, and DEC-012, DEC-015, DEC-033, DEC-034, DEC-038 and others by file scope for the branch. Each is consistent: DEC-015's "lost value" holds (field `None` and recorded), DEC-034's preconditions are unchanged apart from propagating a present matrix's error, and DEC-004's tests are analytic.

**7. DEC-076 amendment note.** Added under §3, pointing at DEC-078 §7.2. `just decisions-index` left `decisions/INDEX.md` unchanged (the note is body-only), so there was nothing to commit there.

**Gates (final tree):**
- `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images just test`: green, 859 passed, 0 failed across 51 result lines (857 from the build plus the 2 verify tests). corpus-status shows every corpus file present.
- `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just doc-check` (2 surfaces, 0 warnings), `just validate` (standing SPEC-061 advisory only): green.
- **SKIPPED:** `sdk-stage2` and `sdk-colour` (`IRRADIANCE_SDK_STAGE2_DIR` and `IRRADIANCE_SDK_COLOUR_DIR` unset).
- `gh pr checks 319` on `b2cc3b6`: all 22 checks pass. The verify commits will re-run them after the fast-forward.

### Findings

- **V1 (fixed here, tests only).** Three plausible regressions passed every build test: `WhiteLevel` read as exactly one value (it would refuse a valid multi-value `WhiteLevel`), `AnalogBalance` fixed at 3, and `CameraCalibration` sized as 3 × ColorPlanes. Two tests were added to `tests/misshapen_tags.rs`:
  - `ac1_colour_planes_size_the_calibration_and_the_balance`: `CameraCalibration1/2` with 16 against 12 entries on the 4-plane CFA, 1 against 3 on the 1-sample plane, and `AnalogBalance` with 4 against 3 and 1 against 3;
  - `ac1_a_multi_value_white_level_is_not_misshapen`: 3 values, not recorded, scalar the first, array all three, on both planes.
  All three mutants are now red, and the controls are green.
- **Nit (no action).** On a hand-built `Sensor` that takes the single-illuminant path, a present misshapen `ForwardMatrix2` is never examined, because only the dual path reads it. The reader refuses that shape on any parsed file first. §7.2's wording is about matrices a helper "receives", so this is arguably in scope. Worth a line if the second line is ever tightened.
- **Ruling on the build's deviations.** Accepted. `dual_illuminant` validating present ColorMatrixes before the illuminant check, and a lone misshapen `ForwardMatrix1` erroring on the dual path, are both what §7.2 says ("a colour helper that receives a present matrix it cannot use returns an error"). Neither is reachable from a parsed file.

### Cost self-report

- **Tokens (total):** 10,230,218 (floor; see `notes`)
- **Estimated USD:** 7.69
- **Duration (minutes):** 16 (transcript span)
- **Source of the number:** this session's transcript, measured per `docs/cost-tracking.md` "Measuring a cycle"

### Drift and new artifacts

- **New decisions emitted:** none. DEC-076 gained an amendment note pointing at DEC-078 §7.2.
- **Deviations from spec:** none in the build beyond the three it reported, which are ruled on above.
- **Follow-up work identified:**
  - None required. If the colour code's second line is ever tightened, validate a present `ForwardMatrix2` on the single-illuminant path too (the nit above).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The rule-clause map's ColorPlanes row did not say which tags it had to reach. "ColorPlanes from `CFAPlaneColor`" reads as satisfied by a single tag, which is how three count rules went unpinned.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The spec's Non-Goal "`WhiteLevel` keeps its count rules" is a rule the reader must preserve, but no AC asked for a positive control on it. A pre-registered "existing rules preserved" control row would have caught that mutant at build.

3. **If you did this task again, what would you do differently?**
   — Write my own mutants from the rule text (one per count in §7.1(b)) before reading the build's mutant table. Every survivor here was a clause the build's table did not list.
