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
  id: HANDOFF-164
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-060

project:
  id: PROJ-004
  stage: STAGE-016
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

# HANDOFF-164: VERIFY: Real-file GainMap evidence and the dng_validate cross-check (SPEC-060)

## Delegation Summary

You are the **verifier** for SPEC-060. Review it cold: you did not build it.

- **Under review:** PR #269 (`claude/spec-060-build`, head e4e4537), built under
  HANDOFF-162. CI was green on all 22 checks.
- **Your branch:** `claude/spec-060-verify-base`. It is that head plus one commit that adds
  this handoff.

SPEC-060 changes no `src/`. It adds:
- `scripts/sdk-stage2.sh`, which makes the Adobe `dng_validate -2` stage-2 outputs
  before and after `OpcodeList2`;
- a test-support stage-2 TIFF reader;
- `tests/gain_map_corpus.rs`, holding the Pixel develop tests, 15 MD5 pins, the amended
  AC2, and the env-gated SDK cross-check with pre-registered bounds;
- a `corpus-status` SKIP line, an AGENTS.md §6 entry, and the provenance evidence.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-060-real-file-gainmap-evidence-pixel-files-develop-cross-checked.md`
  (Design §1–5, AC1–AC7, and AC2 as amended on the build branch).
- **Decisions:** DEC-069 (§2 the rule, §6 the SDK reference, §Context's probe table),
  DEC-068 (why the 7 Pro's no-gain differences are negative), DEC-059 (warped
  `FullActiveArea` refusal), DEC-003 (corpus).
- **Build handback:** HANDOFF-162's `## Handback`, with its numbers, red-proof and
  deviations. Check each claim yourself.
- **The SDK:**
  - The binary is at
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Generate your **own** stage-2 outputs with the committed script into your scratchpad,
    and point `IRRADIANCE_SDK_STAGE2_DIR` at them. Don't reuse the builder's directory.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The recipe reproduces from scratch.** Run `scripts/sdk-stage2.sh` cold into a new
   directory, then run the cross-check against it.
   - Do your numbers match the build's table?
   - Does the script write nothing inside the repo?
   - Does it refuse when the tag pattern is missing or appears twice? Test both, on a
     scratch copy.
   - Is the 51009 → 51010 rewrite safe? It must not match a value or offset that merely
     contains the bytes. Check how it locates the IFD entry.
2. **The bounds are pre-registered, not fitted.** Compare the bounds in the test with
   SPEC-060 Design §4 as designed in PR #266:
   - gain step ≤ 1 and ≥ 99.9% exact;
   - end to end ≤ 8;
   - no gain ≤ 5, with every difference ≥ 0 on the 4a and 3a.

   Any change must be a deviation the orchestrator approved. Confirm the comparisons use
   the right inputs, as Design §4 lists them:
   - `pre.tif` + `apply_gain_maps` against `post.tif`;
   - mono `FullActiveArea`, unoriented, warp 0, `PerPosition`, against `post`;
   - no-gain against `pre`.
3. **The cross-check can fail.** Re-run the build's corner-sampling red-proof. Then try at
   least:
   - per-map rounding removed;
   - one Bayer phase's map skipped;
   - the gain applied in raw coordinates rather than `ActiveArea` coordinates.

   Each should turn the gate red on some file. Also mutate the **test** to show the
   comparison isn't vacuous. Would a reader that returns all zeros, a wrong
   width/height pairing, or a byte-order swap be caught?
4. **The TIFF reader.** It must refuse anything that isn't uncompressed, single-sample,
   16-bit and stripped. It must handle both byte orders and multi-strip files, and
   bounds-check its offsets. Read it for panics on hostile input. It's test support, but
   a silent misread would make AC4 meaningless.
5. **AC2 as amended.** Confirm three things:
   - the corner and centre windows and the phase are named;
   - "corner gain > centre gain" is computed from the map's own parsed gains, not from
     the output;
   - "toward 1" is asserted only for the 3a and 7 Pro.

   Recompute one file's two ratios yourself.
6. **The AC1 warp deviation.**
   - At `warp_alpha` 1.0, `FullActiveArea` is refused on the 3a but develops on the 4a and
     7 Pro. Is that consistent with DEC-059's rule, given each file's warp?
   - Does the test assert each file's outcome exactly?
7. **The pins (AC3).**
   - The 15 MD5 pins are new; no existing pin changed (`git diff origin/main -- tests/`).
   - Pins are stable: run them twice, and threaded against single-thread.
   - They don't duplicate SPEC-059's `plane_tiles` and `develop_options` Pixel
     assertions in a way that drifts.
8. **CI visibility.** With `IRRADIANCE_SDK_STAGE2_DIR` unset, the cross-check prints a
   visible SKIP and passes. `corpus-status` shows the SKIP line. Nothing in CI tries to
   run the SDK.
9. **Nothing forbidden is committed:** no SDK bytes, no SDK outputs, no corpus bytes. The
   provenance-ledger text is accurate.
10. **Gates:**
    - `just test` with the corpus, both with and without the SDK variable;
    - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
    - `cost-audit`, `gate-audit`, `name-length-check`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including a table of your own mutants and your own
  cross-check numbers.
- **Commit this handoff's filled handback** to `claude/spec-060-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-060-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. Any
  `src/` change goes in the punch list. A measured number outside a pre-registered bound
  is reported, never fixed by widening the bound.

## Out of Scope

- The max-black normalisation follow-up (DEC-069 §Context).
- Running the SDK in CI, vendoring it, or committing its outputs.
- The Samsung SM-G960F.
- A colour comparison with the SDK's stage 3.
- SPEC-059 rework.
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
