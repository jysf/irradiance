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
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT from tier_map.build, NOT a record.
                                   #   tier_map is 1 FOR 5 (SPEC-007/FU-6) — SPEC-010's build
                                   #   ran sonnet against this same hint. Read your own
                                   #   message.model and CORRECT this before handing back.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-03
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
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
