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
  id: HANDOFF-198
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-091

project:
  id: PROJ-006
  stage: STAGE-031
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

# HANDOFF-198: VERIFY: the white-balanced linear RGB step at sensor geometry (SPEC-091)

## Delegation Summary

You are the **verifier** for SPEC-091. Review it cold: you did not build it.

- **Under review:** PR 343 (`feat/spec-091-srgb-linear-plane`). The code is commit
  `ea65f2dc`; CI run 45666936351 on it was green, including both `determinism` legs. The
  two later build commits (`3ba0966`, `f4210b8`) touch the handoff, spec and timeline only.
  The build ran under HANDOFF-189 in a separate session.
- **Your branch:** `claude/spec-091-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already advanced the spec to `verify`.

SPEC-091 adds, in `src/color.rs`:
- `srgb_linear_plane_into` and `srgb_linear_plane_f32_into`, which write the white-balanced
  linear-sRGB plane at sensor geometry (every pixel of `width × height`).
- The row-range forms `srgb_linear_plane_rows_into` and `srgb_linear_plane_rows_f32_into`,
  built on `develop_rows_banded`.
- One new `Error::InvalidRowRange`, classified `CallerError`.

The `u16` form must equal today's colour pixels; the `f32` form is the unclamped `f64` sum
divided by 65535, cast once. It also adds a Tier-A colour fixture to `tests/determinism.rs`
with committed plane hashes, and golden digests of the five held files. No existing output
may change. The design took two asks from onyx (the downstream app): the `f32` plane must
be bit-identical on aarch64 and x86_64, and a band form must exist for iPad memory.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md`.
  Judge the ACs against the spec **as designed**: `git show 3ecdad74:<spec path>` (PR 340's
  merge). That is Design §1 to §8, AC1 to AC13, mutants (a) to (k), and the Failing Tests.
- **Decisions:** DEC-079 (accepted: working space, fold, arithmetic, clip-rule evidence);
  DEC-033 (f64 basic operations only); DEC-018 (`quantize`); DEC-016; DEC-040 (CFA phase from
  the ActiveArea origin); DEC-042; DEC-044 (row bands); DEC-051 (`Error::kind`, no wildcard);
  DEC-053, DEC-078 (required processing, malformed tags first); DEC-069 (GainMaps); DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-189's `## Handback`, in the same handoffs directory. It has the
  AC1 byte counts, the AC3 counts, the AC11 partitions, the committed hashes, a mutant table,
  and the deviations listed below.
- **Background:** `docs/determinism.md` and `.github/workflows/ci.yml`'s `determinism` job
  (SPEC-041).
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`.
  - Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  - Use `/usr/bin/git` for branch and HEAD.
  - Give each mutant run its own source state: touch the file after restoring it, or cargo
    may reuse a mutant build (the build hit this). Plain cargo totals need `--no-fail-fast`.

## What to check hard

1. **Every AC against the spec as designed.** Each Failing Tests entry must exist and fail
   when its clause is wrong. Re-run the gates yourself; do not trust the build's numbers.
2. **The refactor of existing entries.** The build moved `develop_color_into`,
   `develop_color_f32_into` and both threaded colour entries onto a shared generic sink
   (`develop_color_with`, `PixelSlot`, `store_u16`, `store_f32_quantised`). This is the
   highest-risk change because it touches shipped outputs.
   - Confirm every existing colour output is unchanged: the existing Tier-B pins and the
     colour tests pass, serial equals threaded, and the four expressions keep today's
     operation order.
   - Diff the old and new sink bodies by eye.
3. **The four build deviations.** Rule on each:
   - (a) **AC4's slack** was widened from `0.5 + 1e-3` to `0.5 + 65535/2^25 + 1e-9`. Is the
     build's argument right (one f32 ulp in `[0.5, 1)`)? Is the new bound still tight
     enough to catch mutant (c)?
   - (b) **AC2 gained a check (iv).** It compares against the independent `demosaic_into`
     then `to_srgb_linear_into` composition, so mutant (f) is catchable. Is that composition
     really independent of `develop_color_fused`'s phase handling? Are the fixture's levels
     really the identity, so the comparison is exact?
   - (c) **Mutant (k)** (`mul_add` in the f32 sink) survives every hash. A unit test with a
     cancelling matrix was added for it. Does that test exercise the real sink, or a copy?
     Would it catch `mul_add` in any of the three channel sums?
   - (d) **`docs/determinism.md`** splits the colour row in two. Does each row claim no more
     than AC12 measures?
4. **Mutants.** Re-run (a) to (k) yourself, plus at least these extras:
   - The band forms skip the `InvalidRowRange` check, so `end > height` slips through.
   - `InvalidRowRange` is checked after `RgbBufferWrongLength`, not before.
   - The `f32` whole-plane form clamps only the negatives.
   - The u16 sink's operation order is swapped (`m02·b` first): AC1 must catch it if f64
     rounding differs.
   - `Error::kind` classifies `InvalidRowRange` as `Corrupt`.
   Give a table: mutant, how applied, which named test failed.
5. **Determinism evidence.**
   - Read CI run 45666936351's two `determinism` job logs. Confirm the arm64 leg really ran
     on arm64 and both legs asserted the new colour hashes.
   - Confirm the fixture's f32 plane holds a negative and an above-1.0 value, and that
     dual-illuminant interpolation really runs: a fixture that falls back to a single
     illuminant would not exercise it.
   - Is the red-proof real?
6. **Band memory.** The point of the band form is that internal scratch scales with the band,
   not the plane. Read `develop_rows_banded` and the band body. List every allocation and
   its size. Flag any `width·height`-sized buffer in the band path.
7. **No panics.** Check the new range arithmetic (`rows.end − rows.start`, multiplication by
   `width·3`, `u32` to `usize`, ranges near `u32::MAX`). Does the fuzz target reach
   `InvalidRowRange`, `RgbBufferWrongLength` and `Ok` for the band forms? Run 60 s of the
   `demosaic` fuzz target.
8. **Docs and API.**
   - The public doc comments must state the geometry, the unclamped f32 rule (and that it is
     not `develop_color_f32_into`'s), the refusal order and the band equality.
   - Check the CHANGELOG and DEC-051's added sentence.
   - `cargo semver-checks` must report no breakage. Confirm the public diff is exactly five
     items.
9. **Gates.** Read `gh pr checks 343` first. Then run:
   - `just test` with the corpus
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`
   - `deny`, `lint-fuzz`, `msrv`
   - `validate`, `gate-audit`, `cost-audit`, `just decisions-audit --changed`
   - `cargo semver-checks`

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and rulings on items 2, 3
  and 6.
- **Commit** this handoff's filled handback, and any tests or docs fixes, to
  `claude/spec-091-verify-base`, and push it. If you approve:
  1. Run `just advance-cycle SPEC-091 ship --verdict approved`.
  2. Run `just handback-sync SPEC-091` in the same push, so CI's cost audit passes.
  3. Fast-forward `feat/spec-091-srgb-linear-plane` to the verify tip, so PR 343 carries
     the verify commits. Do not merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose: the repo ruleset rejects code tokens.
- **IDs:** other sessions also mint IDs. Before minting a DEC or HANDOFF, compute the next
  free one from `origin/main` plus the open PRs.

## Out of Scope

- The mixer, the fold, the fixed-point row and the clip fade (SPEC-092). The developed mix
  (SPEC-093).
- Threaded band forms; a band form of the cropped `develop_color_into`.
- A reusable plan so many small strips don't re-resolve levels, GainMaps and the matrix.
  The build noted this for SPEC-092; note it, don't fix it.
- Any change to `DEVELOP_VERSION` or an existing hash. If one seems necessary, that is a
  rejection.

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
