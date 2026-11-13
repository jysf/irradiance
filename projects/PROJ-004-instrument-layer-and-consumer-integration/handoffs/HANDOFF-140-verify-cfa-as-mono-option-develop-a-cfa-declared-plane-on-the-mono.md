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
  status: pending                  # pending | accepted | completed | rejected

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
