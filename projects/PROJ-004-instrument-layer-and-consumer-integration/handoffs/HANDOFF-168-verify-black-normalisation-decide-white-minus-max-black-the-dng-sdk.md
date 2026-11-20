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
  id: HANDOFF-168
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-078

project:
  id: PROJ-004
  stage: STAGE-025
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

# HANDOFF-168: VERIFY: black normalisation scales by white minus the maximum black (SPEC-078)

## Delegation Summary

You are the **verifier** for SPEC-078. Review it cold: you did not build it.

- **Under review:** PR #276 (`claude/spec-078-build`, head 23ece24), built under
  HANDOFF-167. CI is green on all 22 checks.
- **Your branch:** `claude/spec-078-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-078 implements DEC-070. Colour develop (all four entries) and mono
`BlackLevels::PerPosition` now scale by `W − B`, where `B` is the maximum black the path
applies, and subtract each position's own black. `FirstValue` and uniform-black files are
meant to be byte-identical. `DEVELOP_VERSION` is 7. `scripts/sdk-stage2.sh` also writes a
Canon stage-2 reference, with a new Canon cross-check.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-078-black-normalisation-decide-white-minus-max-black-the-dng-sdk.md`
  (Design §1–5, AC1–AC7, and AC4's pre-registered bounds).
- **Decisions:** DEC-070 (the rule, §3 validation, §Context's probe table), DEC-069 (the
  GainMap step and the SDK reference, §6), DEC-068 (the 7 Pro's fractional black, why its
  differences are negative), DEC-061 (per-position black), DEC-018 (rounding).
- **Build handback:** HANDOFF-167's `## Handback`, with its AC4 table, the 8 re-pins, the
  13 oracle updates, the mutants and the deviations. Check each claim yourself.
- **The SDK:**
  - The binary is at
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate your **own** stage-2 outputs with the committed script into your scratchpad,
    and point `IRRADIANCE_SDK_STAGE2_DIR` at them. Don't reuse the builder's directory.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them into the repo.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The rule, by hand.** Recompute AC1's three rows yourself (W = 15000, blacks
   [2047, 2047, 2047, 2048], B = 2048: 8523 → 32768, 4000 → 9882, 14999 → 65535; today's
   column 32765, 9881, 65530). Confirm the tests assert them on mono `PerPosition` (both
   `develop_with_into` and the scratch entry) and on colour. On colour the values are
   checked on the private normaliser plus a shifted-uniform equivalence on the public
   entries. Is that equivalence actually strong enough to pin the colour rule, or could a
   wrong colour denominator still pass it?
2. **`B` is computed once, over the right blacks.** Read every resolution site: mono
   `PerPosition`, colour slots, colour general pattern, `u16`, `f32` and banded threaded.
   - `B` is the max over the blacks the path *applies*, not over the whole tag.
   - It is never recomputed per sample (a speed check, not only correctness).
   - `FirstValue` and uniform-black paths keep their old code path; their pins held.
3. **The ruling the build left open: the colour path refuses nothing when `W ≤ B`.**
   Spec Design §3 and DEC-070 §3 say `W ≤ B` is `Error::InvalidLevels`. The builder applied
   that only on mono `PerPosition`, arguing the colour path never had a levels refusal, so
   on colour `W ≤ B` gives range 0 and every sample maps to 0
   (`colour_with_no_range_maps_to_zero_without_panicking`).
   - Check whether that's true of the colour path before this PR (`git show origin/main`).
     Is there really no colour levels check, or did the build miss one?
   - Rule on it: an acceptable reading of DEC-070 §3 (record it as a DEC-070 amendment in
     the punch list), or a `src/` fix (punch list). Either way, say which and why.
   - Confirm no division by zero or panic is reachable on colour for `W ≤ B`, `W = B`, or
     `W < every black`.
4. **AC4 reproduces from scratch.** Run `scripts/sdk-stage2.sh` cold into a new scratchpad
   directory, then the cross-checks. Do your numbers equal the build's table and the
   DEC-070 probe? Compare the bounds in the tests with the spec as designed in PR #275
   (`git show origin/main:<spec path>`), not as edited on the build branch:
   - no gain: Canon, 3a and 4a ≤ 1, Δ ∈ {0, +1}, at least 99.5% exact; 7 Pro ≤ 2;
   - end to end: 3a and 4a ≤ 4, at least 99.7% exact; 7 Pro ≤ 4;
   - the gain-step gate unchanged (≤ 1, ≥ 99.9%).

   Any change must be an approved deviation. The Canon stage-2 file is new: check the
   script writes it outside the repo and the Canon test reads the right comparison
   (no-gain stage 2 against our mono `PerPosition` `FullActiveArea`, unoriented).
5. **The 13 oracle updates (approved, on four conditions).** For each, confirm:
   - only the denominator changed, and the per-position `b` is still subtracted;
   - each oracle computes `B` from its own fixture's literals, with no crate helper;
   - the re-derived literal blocks in `tests/black_levels.rs` are right — recompute at
     least two by hand or with an independent script;
   - the `k ≠ 15345` carve-out removal is justified as the handback says.

   Run `git diff origin/main -- tests/ src/color.rs` and confirm no assertion moved that
   isn't in the handback's list.
6. **Indexing is still tested.** Re-run the builder's wrong-position mutants (mono
   transposed `Blacks::at`, colour transposed `pattern_index`, colour slots 1 and 2
   swapped). Each must go red. With the denominator now shared by every position, a
   position bug shows only through the subtracted `b`: check that enough tests have
   blacks that differ by more than 1 for that to be visible.
7. **Mutants.** Re-run the build's (a), (b), (d) and (e), and check (c)'s equivalence
   proof (no clip at `W − B`) yourself. Then try at least:
   - `B` computed over the whole `BlackLevel` tag rather than the applied pattern;
   - `B` from the first slot only on the colour slot path;
   - the f32 colour entry left on the old rule while `u16` moves;
   - the threaded colour path left on the old rule.

   Each should go red somewhere. Record what goes red.
8. **The pins (AC2, AC5).**
   - Exactly the 8 Pixel 3a/4a pins listed moved, plus the four `DEVELOP_VERSION` pins
     and the pre-approved Canon test; every other pin holds (`git diff origin/main --
     tests/`).
   - The new pins are stable: run twice, threaded against single-thread.
   - The Canon test's counts (6,780,144 newly differing at positions 0–2; 5,602,332 at
     position 3; max 6) — spot-check them.
9. **The release surface.** `DEVELOP_VERSION` is `"7"` and documented; the CHANGELOG
   `[Unreleased]` entry names the output change and which files it affects; `semver-check`
   is clean against crates.io 0.2.3. The provenance-ledger row is accurate and cites the
   DNG specification's Rescaling text and the SDK as class 4, with no SDK code copied.
10. **Nothing forbidden is committed:** no SDK bytes, no SDK outputs, no corpus bytes.
11. **Gates:**
    - `just test` with the corpus, both with and without the SDK variable;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants and your own
  cross-check numbers.
- **An explicit ruling on item 3.**
- **Commit this handoff's filled handback** to `claude/spec-078-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-078-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. Any
  `src/` change goes in the punch list. A measured number outside a pre-registered bound
  is reported, never fixed by widening the bound.

## Out of Scope

- SPEC-079's work: the `sdk-stage2.sh` IFD walk and mkdir order, the Adobe licence line,
  the monorail triage refresh.
- `BlackLevelDeltaH/V` (DEC-070 §5 carries it).
- Running the SDK in CI, vendoring it, or committing its outputs.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

## Handback` section below.** An unfilled handback is an
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
