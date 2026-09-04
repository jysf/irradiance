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
  id: HANDOFF-026
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5         # CORRECTED from the claude-opus-5 dispatch hint — this
                                   #   build actually ran on claude-sonnet-5 (this message's
                                   #   own model). tier_map is now 1 for 6.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-03
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-009

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: null               # see notes — this interface exposes no /cost-equivalent call
  estimated_usd: null
  duration_minutes: null
  branch: feat/spec-009-pin-structure-class-membership
  pr: null                         # not opened per HANDOFF-026 instruction 8 — orchestrator's job
  completed_at: 2026-09-03
  notes: "Executed as a direct CLI session (per user's standing preference to dispatch build/verify to a separate session, not an in-process sub-agent), so no Agent-tool subagent_tokens is available either. This interface exposes no /cost or usage-object call I can invoke programmatically. Orchestrator should read /cost from this session's own UI and fill tokens_total/estimated_usd (model that ran: claude-sonnet-5, not the opus tier_map hint) — do not leave cost.sessions null-with-note for this build cycle without checking that first, since build is metered per cost-captured-per-cycle."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-026: Pin the Structure-class membership, table-driven over all eleven tags

## Delegation Summary

Build `SPEC-009` — **STAGE-002's gate on its own inputs**, and the last piece
before the unpack.

`is_structural_tag()` names eleven tags. **Exactly one is enforced by any test.**
Delete the other ten and all 96 tests stay green — re-measured on `main` at
`024eaae`, thirty tests after the finding was raised. The hazard is the next
spec's: `Compression` as `RATIONAL 2/2` reads `1`, `require_uncompressed()`
passes, and `SPEC-012`'s unpack reads **JPEG bytes as raw samples** — a wrong
image from a file that parsed cleanly.

This spec carries four `SPEC-008` findings that share one shape: **a correct fix
with a one-point guard.**

## ⚠ Two things that will decide whether this spec works

**1. The test must carry its own list.** Eleven tags, written out. It must
**not** iterate `is_structural_tag()`. A test that reads the list it is checking
is a tautology — delete a tag and you delete its own coverage, and the suite
stays green exactly as it does today. `AGENTS.md` §16 rule 1 is the general form.

**2. `AC2` needs both directions.** Every structural tag **rejects** `RATIONAL`,
*and* a paired interpretation tag still **accepts** one. A test proving only
rejection would also pass if `uints()` rejected `RATIONAL` universally — which
would silently undo `SPEC-007` and nothing would notice.

## Context the Receiving Agent Needs

Read `SPEC-009` in full — its `## Implementation Context` is a measured probe,
not background. Then `AGENTS.md` **§16** (three codified rules, all three bear on
this spec), §12, §15; `guidance/constraints.yaml`; `DEC-012` (the
Structure/Interpretation split) and **`DEC-014`** — read that one before touching
`AC4`, it is why `AC4` is no longer a free choice.

Corpus: `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.

## The judgement call — `AC4`

A well-formed `IFD0` `Orientation` with an **erroring** sensor-IFD read currently
yields `Some(v)` and an **empty** `malformed_tags`. The tag is present, is shaped
wrong, and is not recorded — which contradicts the field's own documented
contract at `src/ifd.rs:553-560`.

⚠ **`DEC-014` changed the stakes since this was raised.** `malformed_tags` is no
longer just a report: `diff()` treats a tag named in it as **exempt from
comparison with the tool**. So recording more tags *widens the oracle's blind
spot*. The spec sets out both options and gives the orchestrator's read (**B** —
value-found-means-silence — and narrow the contract text instead), **offered as
input, not as the answer.**

**Write the `DEC` either way** — including if you choose B and change no code.
The contract and the code disagree today and nobody has decided which wins; that
is a decision even when the outcome is "keep what we have".

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you and pasted. Sum across **all six
   targets**; a zero-match `cargo test <name>` exits 0.
2. ⚠ **`just lint-ci`, not `just lint`** — local clippy is 0.1.97, CI floats at
   0.1.98. Then **push and read CI**: `constraints.yaml` requires the gate
   *observed* green on your SHA, not asserted from your laptop.
3. **`AC6`'s red-proof watched by you** — delete each of the eleven memberships
   in turn and watch the suite fail each time, with the unmutated tree as the
   control. Eleven mutations; **assert each changed the file and compiled**
   before concluding. This repo has drawn conclusions from mutations that never
   applied, and the count matters here more than usual.
4. ⚠ **Stage your work before running mutate-and-revert experiments on a file you
   have edited.** `SPEC-010`'s build lost its entire change to
   `git checkout --` because nothing was staged, and shipped a reconstruction.
5. **Fuzz** — no new parser, but `src/` moves if `AC4` goes that way. Seeds
   unchanged is a fine result; say so.
6. **Branch and commit before reporting done** (`feat/spec-009-…`), and fill the
   `handback:` — a real `tokens_total` **deduped by `message.id`**, said so, and
   `estimated_usd` per-component at the rates for the model that **actually ran**
   (`message.model`, not `tier_map` — it is 1 for 5). Capture late; the floor
   convention measures ~17% low.
7. **Correct `handoff.to_agent`.**
8. Do **not** run `just handback-sync`; do **not** open the PR.
9. Findings `SB-N` / `FU-N` from 1, each with which of §15's four dispositions
   you think it wants. ⚠ **A `spec:` disposition now requires naming an
   acceptance criterion in that spec which would FAIL if the finding were left
   undone** — `SPEC-010` shipped without closing a follow-up routed to it because
   its AC was narrower than the finding.
10. Verdict is the reviewer's; your job is the build and an honest handback,
    including the §15 reflection questions. `SPEC-010`'s build left none and
    verify could not run check 6 at all.

## Handback

*(Filled by the implementer.)*

### Summary

All seven ACs closed on `feat/spec-009-pin-structure-class-membership`,
commit `3b50964`. `src/ifd.rs` gained five tests (four new, one existing test
extended with a precondition assertion) and a narrowed doc comment on
`Sensor::malformed_tags`; `decisions/DEC-015-*.md` records `AC4`'s answer
(Option B — a value found means silence); `docs/provenance-ledger.md`'s
`src/ifd.rs` row was extended, same class, no new algorithm. Zero behavior
change except the one line AC4 decided to keep (i.e. none — the doc comment
narrowing is the only text change, the code path was already correct).

Full suite: 100 tests across all six targets (was 96; +4 new), corpus set
(`IRRADIANCE_CORPUS_DIR`), 0 failed. All eleven gates + `just lint-ci` run
locally and pasted in the commit message; CI observed green on `3b50964`
(`gh run view 33842214431` — 9/9 jobs). `AC6`'s red-proof watched personally:
all eleven mutations applied (each asserted changed by `git diff --stat` and
compiled before conclusion), each turned `every_structural_tag_rejects_a_rational`
red; the restored, unmutated tree passed the full 100-test suite as the
control. Work was staged (`git add -A`) before any mutate-and-revert
experiment, per this handoff's instruction 4. Fuzz: no new parser, `src/`
moved by a doc comment only — ran anyway, 60s / 13,541,962 executions / zero
crashes; seeds unchanged, which is a fine result.

### Findings

None ship-blocking or follow-up from the build itself. Two administrative
gaps were found and corrected directly, not raised as findings, since they
are spec/handoff bookkeeping rather than code defects:

- `SPEC-009`'s own `references.decisions` listed only `DEC-012`, though the
  spec's text leans on `DEC-014` throughout (it is what makes `AC4` "no
  longer a free choice"). Added `DEC-014` and this build's own `DEC-015`.
- This handoff's and the spec's `handoff.to_agent` both said `claude-opus-5`
  (the `tier_map.build` dispatch hint) — corrected to `claude-sonnet-5`, the
  model that actually ran (return criterion 7; `tier_map` is now 1 for 6).

One spec-authoring inconsistency, resolved rather than raised as a finding
(see Reflection Q1): the `## Failing Tests` section's suggested name for
`AC5`'s test (`wellformed_orientation_test_pins_its_own_precondition`)
disagreed with `AC5`'s own text, which names the *existing*
`wellformed_orientation_is_not_recorded_malformed` and asks for one line
added to it. Followed `AC5`'s text; the `## Failing Tests` section now says
so explicitly rather than silently picking one.

### Reflection (§15)

1. **What would I do differently next time?** Nothing on the code side. On
   the spec side: I'd flag `AC5`'s and `## Failing Tests`' naming
   disagreement (above) back to design *before* touching the file, since a
   handoff has no channel to ask; this time I resolved it in favor of the
   more specific text (`AC5`'s own paragraph) and documented the choice
   in-place, which I think is the right default but is worth a second
   opinion from verify.
2. **Does any template, constraint, or decision need updating?** No —
   `DEC-012`'s table needed no change (this spec pins it, doesn't redraw
   it), and `AC4`'s resolution matches the orchestrator's own recommendation
   exactly, so no decision is contested.
3. **Is there a follow-up spec I should write now before I forget?** No.
   `SPEC-009` was itself the terminal spec in a three-spec recursion
   (`SPEC-007` → `SPEC-008` → `SPEC-009`) that the design cycle's own text
   argued should end here, on the strength of the fix's shape (a table with
   no remaining "one point" to be narrow at) — I have no reason to disagree
   after building it.
4. **Where was the worst defect caught?** `none` — no defect was introduced;
   this spec closes pre-existing coverage gaps `SPEC-008`'s verify measured
   and carried forward. If the question means "where was the *measured*
   coverage gap caught": `verify` (SPEC-008's verify cycle raised FU-1/2/3/5)
   and `design` (SPEC-009's design re-measured all four against current
   `main` before scoping this build).
5. **What can a user do now that they couldn't before?** Before: deleting 10
   of `is_structural_tag()`'s 11 memberships left the suite green (measured,
   `SPEC-009`'s own `## Context`, `024eaae`). After: deleting any single one
   of the eleven turns the suite red, watched directly for all eleven
   (`AC6`). A consumer of this library gets no new capability directly —
   this is `STAGE-002`'s own gate on its inputs, `value_link`'s
   "infrastructure enabling the unpack" — but the class of silent-wrong-image
   hazard the spec's `## Context` names (`Compression` as `RATIONAL 2/2`
   reading `1`, `StripByteCounts` as `RATIONAL 28/2` reading `[14]`) is now
   provably caught before `SPEC-012`'s unpack could ever see it.
