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
  id: HANDOFF-205
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-26
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-093

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

# HANDOFF-205: VERIFY: the developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling (SPEC-093)

## Delegation Summary

You are the **verifier** for SPEC-093. Review it cold: you did not build it.

- **Under review:** PR 358 (`feat/spec-093-developed-mix`). The code is commit `6b29c48`;
  CI run 45898355595 on it was green (25 checks), including both `determinism` legs. The
  later build commit `db31cce` touches the handoff, spec and timeline only. The build ran
  under HANDOFF-204 in a separate session.
- **Your branch:** `claude/spec-093-verify`, cut from that head, plus this handoff and the
  verify prompt. The build already advanced the spec to `verify`.

SPEC-093 adds `DevelopOptions::mono_mix: Option<[Rational; 3]>` and
`opcode::DevelopPath::MonoMix`. The developed mix is the mono develop's geometry (warp
lerped to `warp_alpha`, framing, orientation) run over SPEC-092's mixed plane, mixed
before geometry in 512-row strips into caller scratch. `black_levels` is not read under
the mix. No version constant and no `Error` variant changes; no existing output may move.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md`.
  Judge the ACs against the spec **as designed**: `git show origin/main:<spec path>` (after
  PR 356), Design §1 to §11, AC1 to AC14, mutants (a) to (z), and "onyx's reply
  (2026-11-26)".
- **Decisions:** DEC-084 (accepted: the rule, the opcode set, the strips, the scratch, the
  order, the versions), DEC-083 (the row and the kernel), DEC-079, DEC-058, DEC-059,
  DEC-060, DEC-043, DEC-044, DEC-049, DEC-068, DEC-070, DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `provenance-recorded-per-algorithm`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-204's `## Handback` (mutant table, deviations, AC numbers).
- **Prior verifies:** HANDOFF-198 (SPEC-091, FU-10: a shared path makes "equals the old
  path" oracles move with the mutant) and HANDOFF-203 (SPEC-092, FU-5 and FU-6: public
  constants not tied to the kernel, an untested refusal order).
- **Toolchain:** run `just test`, never `just test --all-features`; set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; use
  `/usr/bin/git` for branch and HEAD.

## What to check hard

1. **Every AC.** AC4's digests and `scratch_len` values equal the registered table
   (compare by script). AC5 (`black_levels` ignored, documented). `mono_mix` composes with
   `framing`, `apply_orientation` and `warp_alpha` at 0, ½ and 1 (onyx's parity case).
   With the option absent, every existing output is byte-identical.
2. **Shared code.** `develop_core`, the demosaic traversal refactor (buffers from
   scratch), `RING` at module level. No existing hash or Tier-B pin moved; the oracle
   cannot move together with a mutant in the shared path.
3. **The build's deviations.** F10 removes both colour matrices; AC11's red-proof uses
   `2^-26` on `kr0`, not one ulp; fuzz option bit 6; `RING` module-level; only
   `tests/developed_mix.rs` seen failing first.
4. **FU-5 and FU-6.** Every public constant and default tied by a test; every ranked
   refusal pair covered on all four entries; the skipped pairs (F7, F8) and (F8, F13)
   truly impossible.
5. **Mutants.** A sample of (a)–(x), plus: mix after geometry, strip size, `black_levels`
   read, version bumps, an `OpcodeList1` refusal skipped, the scratch formula off by one
   strip, the optional warp skipped. Check the argued equivalents (u), (y), (z).
6. **Allocations.** Zero in `develop_with_scratch_into`, exactly one in
   `develop_with_into`.
7. **Determinism.** Both CI legs ran the four developed-mix hashes.
8. **No panics, fuzz.** The `demosaic` and `develop` targets.
9. **Docs and API.** Rustdoc on `mono_mix`, the caching note, CHANGELOG under Unreleased,
   `just semver-check` additions only.
10. **Gates.** `just test` with the corpus, validate, cost-audit, fmt, clippy, doc-check.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- Evidence for each item above, including the mutant table and the deviation rulings.
- Fixes on the verify branch are tests or docs only; `src/` changes go in a punch list.
- If approved: `just advance-cycle SPEC-093 ship --verdict approved`, `just handback-sync
  SPEC-093`, push, and fast-forward `feat/spec-093-developed-mix` only if it is a
  fast-forward. Do not merge PR 358; do not start the ship.

## Out of Scope

- STAGE-034's lifting of the colour paths' refusal of mandatory `FixBadPixelsConstant`
  and `WarpRectilinear` on Leica files.
- Threaded variants, a public strip size, an `irr` hook, any new `Error` variant.
- Any change to `DEVELOP_VERSION`, `MONO_MIX_VERSION` or an existing hash. If one seems
  necessary, that is a rejection.

## Return Criteria — how to hand back

Fill the `handback:` front matter (with a real `tokens_total`) and the `## Handback`
section, including the reflection; set `handoff.status` and `handback.status` to
`completed`; return the verdict.

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
