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
  id: HANDOFF-140
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier
  created_at: 2026-11-13
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-066

project:
  id: PROJ-004
  stage: STAGE-019
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
  tokens_total: 7346027            # REAL combined count — what cost-audit reads
  estimated_usd: 5.75               # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 36
  branch: claude/spec-066-verify-51c60b
  pr: null                         # approved with no code changes: no PR; verdict commented on #227
  completed_at: 2026-11-13         # YYYY-MM-DD
  notes: "Floor at handback: transcript d56548c4-3951-45c3-9fdc-3ad6d2e57f89.jsonl, deduped by message.id, max output per message, no subagents; excludes the commit, push and PR-comment turns. Re-sum at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-140: VERIFY: per-position black levels on the mono path, CFA-as-mono (SPEC-066)

## Delegation Summary

You are the **cold verifier** for SPEC-066. You did **not** implement it. Review it
against SPEC-066 and DEC-061, and return exactly one verdict.

**This verify runs after the merge.** The build PR **#227** (branch
`claude/spec-066-build`, head `ddbbf81`, CI 44/44 green) was **already squash-merged to
`main` as `399acf4`** before verify. Your worktree is cut from
`claude/spec-066-verify-base`, which is `main` at `399acf4` plus this handoff and the
spec's `cycle: verify`. The code under review is the code on `main`. Diff it against the
design merge with `git diff 06d776a 399acf4`.

The build delivers:
- **`BlackLevels { FirstValue, PerPosition }`**, the `DevelopOptions::black_levels`
  field, level resolution, and the two normalize sites in `src/develop.rs`. The level
  dispatch lives in the new `crop_orient_normalize_for` and `normalize_active_area_for`
  wrappers. The loops stay in `crop_orient_normalize_into` and
  `normalize_active_area_into`, because `tests/develop_oracle.rs` scans those bodies.
- **`Error::InvalidBlackLevelPattern`** (Corrupt) in `src/lib.rs`.
- **Tests:**
  - `tests/black_levels.rs`;
  - `per_position_allocates_nothing` in `tests/develop_scratch_alloc.rs`;
  - an `error_kind` row.
- **Fuzz:** option bit 5, a collapse differential, and three `black-levels-*` seeds.
- **Docs:** CHANGELOG, the develop module docs, and the field and enum docs.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-066-cfa-as-mono-option-develop-a-cfa-declared-plane-on-the-mono.md`.
  Read all of it: the §1 probe, Design §2–§6, AC1–AC13, the red-proof list, and the
  Notes.
- **Decision:** `./decisions/DEC-061-the-mono-develop-honours-blacklevelrepeatdim-under-an-opt-in-per-position-black-level-option-and-white-stays-per-sample.md`.
- **Also:**
  - DEC-058 (options; the version rule; options never skip validation);
  - DEC-060 (the scratch path allocates nothing; the shared core);
  - DEC-040 (the `ActiveArea`-origin phase);
  - DEC-018 (clamp, rounding);
  - DEC-012;
  - DEC-051, DEC-052, DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-139-build-cfa-as-mono-option-develop-a-cfa-declared-plane-on-the-mono.md`.
  Treat its claims as claims. The build found three places where **my design was wrong**.
  Judge whether each fix keeps the criterion's intent:
  - **AC2's "uniform `normalize(k)`" is not uniform at every k.** Distinct per-position
    blacks give distinct ranges, and these round differently: at k = 15345, positions 0
    and 1 give 65535 and positions 2 and 3 give 65531. The build asserts only the
    analytic per-position value there, and checks uniformity and equality with the
    LinearRaw develop at k = 0 and 200. Check that at **every** k the test still asserts
    the exact analytic value for each position, so no k becomes a weaker check.
  - **AC3's crop origin is (1, 2), not odd on both axes.** With the `ActiveArea` origin
    at (3, 1), a both-odd crop origin lands on the plane origin's parity, and the
    "phase from the DefaultCrop origin" mutant could not be seen. Re-derive by hand that
    the three candidate origins (`ActiveArea`, plane, `DefaultCrop`) give three different
    expected 4×4 blocks with this fixture. Check one block's values by hand.
  - **AC5 uses two cases so that uniformity is well defined**, and asserts its own
    rounding premise. Confirm that the premise assertion would fail if the premise broke,
    and that both cases still separate "black before the warp" from "black after the
    warp".
  - **The per-position-white red-proof was modelled as reading the last `WhiteLevel`
    value.** Decide whether the two-`WhiteLevel` fixture also kills a true per-pixel
    `white_levels[pos]` read.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Run long binaries in the background.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC13 met by a test that would fail if the property broke? Is the build
  reflection honest?
- Run the gate suite yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`
- Run `just decisions-audit --changed`.
- Confirm that `main`'s CI is green at `399acf4` (`gh run list --branch main`).

### The behavioural surface (this is where defects escape)

**Independence of the oracles.** AC2–AC5's expected values must come from a formula or
literals in the test, never from calling `develop`. Read each helper in
`tests/black_levels.rs`. If one reuses a crate function (for example a re-exported
`normalize`), that is a shared-path oracle (`shared-path-oracle-tautology`, N=3), and it
is a finding.

**DEC-061 §3, read line by line in `src/develop.rs`:**
- `FirstValue`, or `PerPosition` with `black_levels` empty, takes today's path exactly.
- `[R, C]` defaults to `[1, 1]` when the dimension is absent or malformed.
- Zero, overflow and a count mismatch all give `InvalidBlackLevelPattern` with the
  correct fields.
- The first offending `B[i] ≥ white` gives `InvalidLevels` carrying that value.
- An all-equal `B` takes `Uniform`.
- The phase origin is the `ActiveArea` top-left, or `(0, 0)` when the tag is absent.
- The index is `row × C + col`.
- Arithmetic must not panic under clippy's arithmetic lints.

**The `Uniform` fast path really is unchanged.**
- Confirm the `Uniform` arm computes no per-pixel modulo.
- Confirm the eight SPEC-063 MD5 pins in `tests/develop_options.rs` are unedited
  (`git diff 06d776a 399acf4 -- tests/develop_options.rs tests/develop_scratch.rs`).

**Mutate it yourself**, outside any test knob, and **run tier A only** (CI has no corpus):
- the 12 in the build's list;
- the pattern origin taken as the `DefaultCrop` origin under `FullActiveArea`, where the
  crop origin is 0 (does a test still see it?);
- `R` and `C` swapped when the index is formed from a non-square `[1, 2]` / `[2, 1]`
  dimension;
- the empty-`B` case refused instead of falling back;
- `InvalidLevels` reporting the last offending value instead of the first.

A mutant that stays green, or that only tier B catches, is a finding.

**Order and no-write guarantees.**
- With a bad pattern **and** each earlier error (`warp_alpha`, a mandatory opcode,
  `src` length, `ActiveArea`), the earlier error wins.
- With a bad pattern and a wrong `dst` length, the pattern error wins.
- Refusals leave `dst` and `scratch` untouched (sentinels), from both
  `develop_with_into` and `develop_with_scratch_into`.

**The warp path.** Read `normalize_active_area_into`. The black is chosen by **raw**
coordinates in the active grid, before the warp. Confirm the `FixBadPixelsConstant` copy
still runs on raw values first.

**Allocation.** `per_position_allocates_nothing` covers a pattern with and without a warp
and `FixBadPixelsConstant`, and the error paths. Confirm the counter is still the
thread-local one with the negative control intact.

**Real files.** Confirm these tier-B tests **ran** (no SKIP line) and asserted what the
spec says:
- the Canon relation: positions 0–2 equal; position 3 ≤, 5,602,332 differences, at most
  6;
- the 1×1 files' MD5 equality.

**Fuzz.**
- The three seeds are synthetic (built by `examples/fuzz-seeds.rs`), with no corpus bytes
  (DEC-003).
- The collapse differential asserts byte equality and does not just run both modes.
- Run `develop` for at least 60 s and report the run count.

**Error surface.**
- `InvalidBlackLevelPattern` is in `Error::kind()` with no wildcard.
- Its `Display` names all three numbers, and a test checks that.
- `DEVELOP_VERSION` is still `"4"`.
- The CHANGELOG entry is accurate.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

#227 is already merged, so there is no build PR to supersede.
- **Approved with no code changes:**
  - Commit the filled HANDOFF-140 handback (and the timeline's verify line) on your
    `claude/spec-066-verify-<suffix>` branch and push it.
  - Post the verdict as a comment on PR #227, naming that branch.
  - Open no PR. The orchestrator lands those docs in the ship PR.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` from your verify branch, carrying the fixes and the
    filled handback.
  - Reference HANDOFF-140, SPEC-066, DEC-061 and PR #227.
- **Rejected:** comment on #227 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
  Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- Per-position white.
- `BlackLevelDeltaH` and `BlackLevelDeltaV`.
- RATIONAL `BlackLevel`.
- The colour path's `BlackLevelRepeatDim` handling. These are DEC-061 §5 follow-ups.
- Routing changes, SPEC-067, a release, and a `DEVELOP_VERSION` bump.

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

- **Branch / PR:** `claude/spec-066-verify-51c60b`. Approved with no code changes, so no PR
  was opened. The verdict is commented on https://github.com/jysf/irradiance/pull/227.
- **Completed at:** 2026-11-13
- **All acceptance criteria met?** yes. AC1–AC13 each have a test that goes red when the
  property breaks (see the mutants below).
- **For `verify`:** ✅ **APPROVED at `399acf4`** (main; same tree as #227's head `ddbbf81`,
  `d56dc33`).

**Code review against DEC-061 §3–§4, `src/develop.rs` line by line.**
- `FirstValue`, or `PerPosition` with `black_levels` empty, returns through the unchanged
  `resolve_levels`, so it takes today's path exactly.
- `[R, C]` defaults to `[1, 1]`. A zero dimension, an overflowing product or a count
  mismatch gives `InvalidBlackLevelPattern` carrying the resolved `R`, `C` and the count.
- `find` reports the first `B[i] ≥ white`. An all-equal `B` gives `Uniform`.
- The phase origin is the `ActiveArea` top-left, or `(0, 0)` without the tag. The index is
  `row × C + col`.
- `Blacks::at` is all checked arithmetic. Clippy's arithmetic lints are green.
- The `Uniform` arm passes a constant closure `|_, _| black` into the generic loop. That
  monomorphises to the scalar loop and does no per-pixel modulo.
- The call sits exactly where `resolve_levels` was: after geometry and options, before the
  `dst` length. The loops stay in `crop_orient_normalize_into` and
  `normalize_active_area_into`. On the warp path the black is chosen by raw coordinates
  before the warp, and `copy_and_fix_bad_pixels` still runs first on raw values.
- `git diff 06d776a 399acf4 -- tests/develop_options.rs tests/develop_scratch.rs` is empty,
  so the eight SPEC-063 pins are unedited.
- `DEVELOP_VERSION` is `"4"`.
- `InvalidBlackLevelPattern` is in `kind()` with no wildcard.
- `Display` names all three numbers, and `pattern_error_display_names_all_three` pins that.
- `error_kind` moves to 65 variants, with Corrupt at 30.

**Oracle independence.** Every expected value in AC2–AC5 comes from the test's own
`normalize`, `pattern_black` and `expected_image`, or from literals. None calls `develop` or
a crate re-export. Byte identity is used only where the spec asks for it: AC2's LinearRaw
equality and AC6's absent-`B` / 1×1 cases. In both, an analytic assertion on the same data
either sits beside it or isn't needed. No shared-path tautology.

**The build's three fixture changes keep their ACs' intent.**
- **AC2.** The analytic per-position assertion (`got == want`) runs at **every** k
  (0, 200 and 15345), under both framings, both `apply_orientation` values and
  orientations 1 and 6. It sits before the k-conditional block, so no k is weaker.
  Uniformity and equality with LinearRaw are asserted at k = 0 and 200. At k = 15345 the
  two ranges (15345 and 15346) genuinely round to 65535 and 65531, so uniformity is not a
  property there. The FirstValue-is-not-uniform check runs at k = 200 and 15345.
- **AC3.** Re-derived by hand. Output (0, 0) is raw (4, 3), with raw value
  500 + 37·4 + 11·3 = 681.
  - Phased from the `ActiveArea` origin (3, 1), the slot is (0, 1), so the black is 200 and
    the value is ⌊(481·65535 + 1947) / 3895⌋ = **8093**. That matches the literal.
  - Phased from the plane origin, the slot is (1, 0), so the black is 300 and the value is
    **6579**.
  - Phased from the `DefaultCrop` origin (4, 3), the slot is (0, 0), so the black is 100 and
    the value is **9531**.

  The three candidate origins give three different blocks. With an odd/odd crop origin the
  `DefaultCrop` phase would equal the plane's parity, so (1, 2) is the right fix.
- **AC5.** The premise assertion is a hard `assert_eq!` against the test's own `normalize`,
  so it fails if a case stops rounding alike. I checked each case alone against a true
  "black after the warp, by output position" mutant: normalize raw values with an identity
  map, warp, then subtract the per-output-position black. **Both cases go red on their
  own.** The AC3 pattern at k = 0 fails under `StandardCrop`. The narrow pattern at k = 10
  fails only under `FullActiveArea`, because the centre crop barely moves.
- **Per-position white.** The two-`WhiteLevel` fixture kills a **true per-pixel**
  `white_levels[pos]` read (mutant 12b below) as well as the build's "last value" model.

**Gates (corpus env set, 12/12 present, no SKIP line).**
- `just test`: 668 passed, 0 failed, 1 ignored. The tier-B
  `canon_per_position_differs_only_at_position_3` and
  `one_by_one_files_are_identical_under_both_modes` ran and passed. They assert the
  5,602,332 / ≤ 6 / positions-0–2-equal relation and the seven MD5 pins.
- Green: lint-ci, lint-no-allow, lint-fuzz, fmt-check, validate, doc-check, deny, msrv,
  cost-audit, gate-audit.
- `decisions-audit --changed` is advisory and names only DEC-061.
- main CI at `399acf4`: run 43500667607, success. #227's checks were 44/44.

**Mutants: 18, applied by hand to `src/develop.rs`, tier A only (no corpus env), all
reverted.** Every one is red.

| # | Mutant | Tests that went red |
|---|---|---|
| 1 | Pattern ignored | AC2, AC3, AC4, AC5, AC6, AC7, AC9 |
| 2 | Entries 0 and 1 swapped | AC3, AC4, AC5 |
| 3 | Phase from the plane origin | AC2, AC3, AC4, AC5 |
| 4 | Phase from the `DefaultCrop` origin (sensor tag) | AC2, AC3, AC4, AC5 |
| 5 | Index transposed | AC2, AC3, AC4, AC5 |
| 6 | Fixed 2×2 slot | AC4 |
| 7a | Warp site, black by active-grid position | AC5 |
| 7b | Black truly after the warp | AC5 |
| 8 | Warp site, pattern ignored | AC5 |
| 9 | No count check, index clamped | AC6, AC7, AC9 |
| 10 | Validated under `FirstValue` too | AC6 |
| 11 | Pattern check after `dst` | AC7 |
| 12a | Last `WhiteLevel` | AC3, AC9 |
| 12b | True per-pixel `white_levels[pos]` | AC3 |
| 13 | Phase from `geometry.crop_origin` (0 under `FullActiveArea`) | AC3, AC4, AC5, through the `StandardCrop` assertions |
| 14 | `R` and `C` swapped | AC4, AC6 |
| 15 | Empty `B` refused instead of falling back | AC6 |
| 16 | `InvalidLevels` reports the last offender | AC6 |

**Order and no-write.**
- `pattern_error_order` covers `warp_alpha`, a mandatory opcode, `src` length and
  `ActiveArea` ahead of the pattern error, and the pattern error ahead of `dst`.
- `refused()` checks the `dst` and scratch sentinels from both entry points.

**Allocation.** `per_position_allocates_nothing` uses the thread-local `COUNTING` /
`ALLOCATIONS` counter. Its negative control is intact. It covers all four
fbp × warp arms under both framings, and five refusals.

**Fuzz.**
- `develop`: **4,654,282 runs in 63 s**, clean.
- The three `black-levels-*` seeds are built by `examples/fuzz-seeds.rs` from numbers in the
  source. Re-running it leaves the tree byte-identical, so there are no corpus bytes
  (DEC-003).
- The collapse differential asserts `collapsed.is_ok() && developed_first == developed_with`,
  which is byte equality.

**CHANGELOG.** It is accurate. One nit: "every held LinearRaw file declares a 1×1 pattern"
is loose for the K-3 III, whose `BlackLevelRepeatDim` is malformed and defaults to
`[1, 1]`. DEC-061 words it the same way. No change needed.

### Cost self-report

- **Tokens (total):** 7,346,027. This is a floor; the commit, push and comment
  turns are excluded.
- **Estimated USD:** 5.75
- **Duration (minutes):** 36, by the transcript's timestamps.
- **Source of the number:** transcript `d56548c4-3951-45c3-9fdc-3ad6d2e57f89.jsonl`, found
  by the scratchpad UUID, deduped by message.id, max output per message. Priced per
  component at opus-5-5 rates (5 in, 25 out, cache read ×0.10, 1-hour cache write ×2.0).
  No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the code. The three fixture changes the build made
  (AC2, AC3, AC5) are judged above and keep their criteria's intent.
- **Follow-up work identified:** two XS test-strength nits. Neither blocks.
  1. AC5's comment "the warp really ran: FirstValue … differs" proves nothing about the
     warp, because the two modes differ even without one. The narrow-pattern case also
     catches the after-warp mutant only under `FullActiveArea`. An assertion that the
     warped `FirstValue` output differs from an unwarped develop would make the comment
     true.
  2. `develop_err` (AC7) checks the `dst` sentinel but not the scratch sentinel. AC6's
     `refused()` does check it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The handoff's question list was precise. One small gap: the
   `decisions-audit --changed` gate needs the flag passed unquoted to `just`. A loop that
   quotes "recipe --flag" as a single word errors out with "no recipe".
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The build's note about `develop_oracle` scanning the text of `src/develop.rs` was
   already in the hand-over. It mattered for writing mutants: I edited only inside the
   function bodies and never renamed one.
3. **If you did this task again, what would you do differently?**
   — Run the mutants against a copy in a separate `CARGO_TARGET_DIR`, so they can start
   while the 15-minute `just test` debug corpus run is still going. Sharing the target
   directory forced me to run them one after the other.
