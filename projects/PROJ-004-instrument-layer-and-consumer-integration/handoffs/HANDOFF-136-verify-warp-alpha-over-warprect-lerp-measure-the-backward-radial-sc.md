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
  id: HANDOFF-136
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-12
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-064

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

# HANDOFF-136: VERIFY: warp_alpha over WarpRect::lerp, and the FullActiveArea bounds rule (SPEC-064)

## Delegation Summary

You are the **cold verifier** for SPEC-064. You did **not** implement it. Review it
against SPEC-064 and DEC-059, and return exactly one verdict.

- **The build:** **PR #219**, branch `claude/spec-064-build`, cut from `origin/main` at
  `1720c66`. The head when this handoff was minted is `863b857` (build commit `75ea6d0`,
  then the handback commit).
- **Your branch:** your worktree is cut from `claude/spec-064-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers:
- **`WarpRect::lerp`** in `src/opcode.rs`.
- **The bounds function and the `full_active_area_admits` predicate** in `src/warp.rs`.
  The risk is here: closed-form cubic extrema, degenerate cubics, NaN handling.
- **`DevelopOptions::warp_alpha`**, step-0 α validation, `non_identity_warp(sensor,
  alpha)`, and the bounds check in place of SPEC-063's blanket refusal, all in
  `src/develop.rs`.
- **`Error::InvalidWarpAlpha`** and three new fields on the unreleased
  `UnsupportedWarpedFullActiveArea`, in `src/lib.rs`.
- **Tests:**
  - `tests/warp_alpha.rs` (12 tests; tier B runs 33 Q2M develops);
  - three unit tests in `src/warp.rs`;
  - edits to `tests/develop_options.rs` and `tests/error_kind.rs`.
- **Fuzz:** the option byte is extended, and there are six new seeds.
- **Docs:** CHANGELOG `[Unreleased]` and a provenance-ledger row.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-064-warp-alpha-over-warprect-lerp-measure-the-backward-radial-sc.md`.
  Read all of it: the §1 probe table and coefficients, Design §2–§5, AC1–AC13, the
  red-proof list, and the Notes.
- **Decisions:**
  - `./decisions/DEC-059-warp-alpha-lerps-the-radial-scale-toward-identity-and-full-active-area-admits-a-warp-whose-scale-stays-in-0-1.md`;
  - DEC-058, which this amends at §4;
  - DEC-024 (clamp-to-edge);
  - DEC-051 (wildcard-free `Error::kind`);
  - DEC-052 (`#[non_exhaustive]`);
  - DEC-003 (tier-B bytes are never committed).
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-135-build-warp-alpha-over-warprect-lerp-measure-the-backward-radial-sc.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation is sound:
  - **AC4 now computes its expected pull.** The spec quotes 506.1 px for the active area;
    the crop-size fixture's value is 503.7 px, so the test computes `m·(1 − f(1))`
    instead of hard-coding either number. Confirm the relation the test asserts is still
    "linear in α within 1 px", and that computing the expectation did not make it
    circular.
  - **`displacement_is_linear_in_alpha` gained a "the impulse survives" guard** after
    three lerp mutants left it green. They mapped the impulse off-image, and an all-zero
    argmax happened to land on the expected corner. Confirm the guard is enough. Look for
    any other way the argmax can pass vacuously.
  - **The predicate also requires `kt` to be finite.**
  - **Red-proof 10.** NaN-unsafe comparisons are caught only by the AC9 unit test; AC6's
    NaN case is still refused through the finiteness clause.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - The tier-B `warp_alpha` binary takes about 7 min in debug; run it in the background.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC13 met by a test that would fail if the property broke?
- Is the build reflection honest?
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

### The behavioural surface (this is where defects escape)

**The bounds rule is exactly DEC-059 §3, computed independently.**
- Outside the test code, write a few lines of standalone Rust or Python in your
  scratchpad. Re-derive the scale extrema at α = 1 for the three Q2M sets and the Pixel
  3a / 7 Pro sets, by dense sampling. Compare them with the spec's §1 table and with what
  `src/warp.rs` returns.
- Then, for at least one Q2M frame and the Pixel 3a set, count the out-of-image source
  pixels **pixel-exactly** at α = 0.5 and 1. Confirm that the predicate admits exactly
  the zero-count cases.
- Read the cubic-extrema code line by line:
  - the quadratic roots (including `a = 0`, `b = 0`, a negative discriminant, and roots
    outside `(0, 1)`);
  - catastrophic cancellation;
  - no division by zero;
  - NaN never admitted;
  - no arithmetic that can panic under clippy's arithmetic lints.

**`lerp` is bit-exact at α = 1.** Read `lerp`. Confirm the formula is
`(1.0 - alpha) + alpha * kr0` and nothing that rounds differently. Confirm AC2 compares
with `to_bits`. Confirm the eight SPEC-063 MD5 pins in `tests/develop_options.rs` are
**unedited**, with `git diff origin/main -- tests/develop_options.rs`.

**Ordering and no-write guarantees.** Mutate them yourself, outside any test knob, and
confirm a test goes red:
- α validation moved after `check_required_processing`, or after the opcode parse;
- the bounds refusal moved after the `FixBadPixelsConstant` copy, or after a `dst` write;
- the bounds check applied under `StandardCrop`;
- the blanket SPEC-063 refusal restored.

Also confirm that under `FullActiveArea` the warp is parsed once, not twice.

**Selection mutants on the predicate.** Confirm a test goes red for each:
- corner only (`s = 1`);
- interior roots dropped;
- `f ≥ 0` dropped;
- centre clause dropped;
- finiteness dropped;
- `<` swapped for `<=` at a bound. Decide whether a boundary case (`f` exactly 1, as at
  α = 0's identity, or a centre exactly 0 or 1) is pinned. If it is not, say whether it
  needs to be.

**α semantics.**
- α = 0 equals the warp-free develop on real Q2M frames.
- A non-zero-`kt` warp develops at α = 0 and is refused at α = 1.
- `-0.0` is accepted.
- NaN, ±∞, -0.01 and 1.01 are `CallerError` from both `develop_with_into` and
  `output_dimensions_with`, on a sensor with no warp too.

**Real-file success.** Confirm the tier-B AC5 test **ran** (no SKIP line) and asserts the
containment relation at all five α on all three Q2M frames, including L1026016
(Orientation 6) with `apply_orientation: false`.

**Fuzz seeds.**
- The six new `fuzz/seeds/develop/options-*.tiff` files must be synthetic (built by
  `examples/fuzz-seeds.rs` from coefficients). Confirm none is derived from corpus bytes
  (DEC-003).
- Run the `develop` target for at least 60 s and report the run count.

**No regression.**
- Account for every deleted or changed line in `tests/develop_options.rs` and
  `tests/error_kind.rs` against the allowance in AC6/AC10.
- `git diff origin/main -- src/` adds no `Vec`, `vec!`, `Box`, `collect`, `to_vec`,
  `String` or `format!` on the develop path.
- `DEVELOP_VERSION` is still `"4"`.

**Error surface.**
- `InvalidWarpAlpha` is in `Error::kind()` with no wildcard.
- Both variants' `Display` texts name their values, and a test checks each.
- CHANGELOG `[Unreleased]` no longer says the Q2M is refused.

**CI is observed green on the PR head** with `gh pr checks 219`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

**Approved with no code changes:**
- Do **not** open a PR that supersets #219. Such a PR conflicts once #219 is
  squash-merged.
- Commit the filled HANDOFF-136 handback (and the timeline's verify line) on a branch
  named `claude/spec-064-verify-<suffix>` and push it.
- Post the verdict as a comment on PR #219, with the branch name.
- The orchestrator lands those docs in the ship PR.

**Punch list with fixes you make yourself:**
- Open a NEW PR against `main` that supersets #219.
- It must reference HANDOFF-136, SPEC-064, DEC-059 and PR #219.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.

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
