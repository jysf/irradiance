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
  id: HANDOFF-134
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-12
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-063

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

# HANDOFF-134: VERIFY: DevelopOptions + develop_with_into (SPEC-063)

## Delegation Summary

You are the **cold verifier** for SPEC-063. You did **not** implement it. Review it against
SPEC-063 and DEC-058 and return exactly one verdict.

- **The build:** **PR #214**, branch `claude/spec-063-build`, cut from `origin/main` at
  `0642e54`. When this handoff was minted its head was `ae232c2`, and CI was green on that
  head.
- **Your branch:** your worktree is cut from `claude/spec-063-verify-base`. That branch is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under review
  is the build's own.

What the build delivers:
- `DevelopOptions`, `Framing`, `develop_with_into` and `output_dimensions_with` in
  `src/develop.rs`. A private `apply_options` rewrites the resolved `Geometry`, and
  `develop_into` / `output_dimensions` delegate to the new functions.
- `Error::UnsupportedWarpedFullActiveArea` in `src/lib.rs`.
- `tests/develop_options.rs` (13 tests, including the tier-B AC2 MD5 pins) and rows in
  `tests/error_kind.rs`.
- An option byte in the `develop` fuzz target, plus 8 new seeds.
- A CHANGELOG entry and a provenance-ledger row.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-063-developoptions-develop-with-into-non-exhaustive-options-with.md`.
  Read all of it, especially §1 (the probe table and the MD5 pins, taken from main at
  3458854 **before** the change), Design §2–§7, AC1–AC13 and the red-proof list.
- **Decisions:**
  - `./decisions/DEC-058-developoptions-is-a-non-exhaustive-struct-whose-default-is-develop-into.md`;
  - DEC-024 (the warp clamps to the edge);
  - DEC-028 (`DEVELOP_VERSION`);
  - DEC-051 (`Error::kind` has no wildcard);
  - DEC-052 (`#[non_exhaustive]`);
  - DEC-012;
  - DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-133-build-developoptions-develop-with-into-non-exhaustive-options-with.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation holds up:
  - **AC9's second half is unsatisfiable.** Orientation swaps width and height and cannot
    change a product, so the test instead pins that the buffer is accepted and receives
    the unswapped layout. Do you agree, and is that a sufficient substitute?
  - **No allocation before the refusal, except one.** The only allocation is the existing
    small `Vec<Opcode>` from `parse_warp_rectilinear`, which now runs earlier and only
    once under `FullActiveArea`. Confirm that it is parsed only once, and that no
    plane-sized allocation happens before the refusal.
  - **AC2 takes ~3 min in debug.**
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use the files in
  place; never copy them.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC13 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run each gate yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
  - `msrv`
- Run `just decisions-audit --changed`.

### The behavioural surface (this is where defects escape)

**Byte identity is independent of the build.**
- Re-derive at least two of the §1 MD5s yourself: L1026016 (warped, Orientation 6) and
  one warp-free file. Build a few lines of standalone Rust in your scratchpad against the
  PR head, run `develop_into`, and MD5 the output as little-endian `u16`.
- Confirm that `git diff origin/main` shows no pin edited and `DEVELOP_VERSION` still `"4"`.
- Confirm that the AC2 test **ran**: no SKIP line, and the three Pixel files refused with
  identical text under both entry points.

**Validate, then override (DEC-058 §3).** Read `develop_with_into` and
`output_dimensions_with`.
- `resolve_geometry` must run unchanged before `apply_options`, on both paths.
- Mutate the code yourself so validation is skipped under each option, and confirm a test
  goes red.

**The refusal (DEC-058 §4).** Mutate each of these and confirm a named test goes red:
- refusal removed;
- refusal fires on an identity warp;
- refusal fires under `StandardCrop`;
- refusal placed after the `FixBadPixelsConstant` copy;
- refusal placed after a `dst` write.

Also confirm that, under `StandardCrop`, the opcode parse order and the error precedence
are exactly main's. Read the diff. A test showing it would be better than a reading;
judge whether one is needed.

**The relations are real oracles.**
- AC4 and AC6 compare against `StandardCrop` / Orientation-1 develops of a modified
  sensor. Check that the tests could not pass trivially: for example, both sides empty, or
  a fixture so uniform that a wrong crop gives the same pixels.
- Mutate `apply_options` so that:
  - FullActiveArea has an off-by-one origin;
  - FullActiveArea has the crop size swapped;
  - FullActiveArea uses the crop origin, not 0;
  - `apply_orientation: false` maps to orientation 2 rather than 1.

  Confirm each one goes red.

**Tier-B coverage.** Confirm that `FullActiveArea` is exercised on all three warp-free files
(K-3 III, L1000622, Typ 246), that the Q2M refusal is exercised, and that L1026016
`apply_orientation: false` runs through the warp path.

**API and error surface.**
- `DevelopOptions` derives `PartialEq` and **not** `Eq`/`Hash` (DEC-058 §1).
- Both new types are `#[non_exhaustive]`, with every field and variant documented, and the
  docs say the options are mono-only.
- An external test constructs options via `Default` plus field assignment.
- The new variant is in `Error::kind()` with no wildcard, and a test pins its `Display`
  naming both dimensions.
- CHANGELOG `[Unreleased]` names every addition and says the default output is unchanged.

**No regression.**
- `git diff origin/main -- tests/` changes no existing assertion except `error_kind`'s
  rows and counts. Account for every deleted line in `tests/error_kind.rs` and elsewhere.
- The `src/` diff adds no `Vec`, `vec!`, `to_vec`, `Box`, `String` or `format!` on the
  develop path.

**Fuzz.** Run the `develop` target for at least 60 s with the new seeds, and report the run
count.

**CI** must be observed green on the PR head, with `gh pr checks 214`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes or derived
  pixels (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript. Set `basis: transcript`, dedupe by
  `message.id`, take the max output per message, and use opus-5-5 rates. Fill the
  `## Handback` section too.
- Open a NEW PR against `main` from `claude/spec-063-verify-<suffix>`. It carries the
  handback and any punch-list fixes, must be a superset of #214, and must reference
  HANDOFF-134, SPEC-063, DEC-058 and PR #214.

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
