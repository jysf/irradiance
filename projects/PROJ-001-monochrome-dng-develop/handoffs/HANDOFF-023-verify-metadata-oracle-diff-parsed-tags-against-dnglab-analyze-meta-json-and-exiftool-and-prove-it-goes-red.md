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
  id: HANDOFF-023
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT from tier_map.verify, NOT a record.
                                   #   tier_map is 1 for 4 (SPEC-007/FU-6) — the ONE hit was
                                   #   SPEC-005's own verify round 1. Check your own
                                   #   message.model and CORRECT this; do not inherit it
                                   #   because it happened to be right last round.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-22
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-005

project:
  id: PROJ-001
  stage: STAGE-001
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

# HANDOFF-023: Re-verify SPEC-005's punch-list fix — round 2, at `5b1aef7`

## Delegation Summary

**Round 2 of verify.** Round 1 (`HANDOFF-022`) returned ⚠ PUNCH LIST at
`418be15` — one ship-blocker (`SB-1`, `DEC-013` wrong on three counts), seven
follow-ups, nothing else holding the spec. This round reviews **only the fix**,
at **`5b1aef7`** on `feat/spec-005-metadata-oracle`.

**⚠ Read this first: the architect wrote the fix.** The orchestrator made the
`SB-1` change, rejected `DEC-013`, wrote the new doc comment, and took the
judgement call the reviewer explicitly delegated. That is the whole reason this
round exists — it is the architect grading their own homework, and the round
before it is the only reason we know the last self-graded artefact was wrong on
three counts.

**Findings continue this spec's sequence** (§15): round 1 used `SB-1` and
`FU-1`…`FU-7`, so start at **`FU-8`** / **`SB-2`**.

## What changed since the reviewed SHA — the entire delta

`git diff 418be15..5b1aef7` — six files, and only one is code:

| file | change |
|---|---|
| `tests/support/tools.rs` | the **only** functional change: one condition, one import, one doc comment |
| `decisions/DEC-013-…md` | `status: accepted` → **`rejected`**, rewritten, original text preserved verbatim below a line |
| `guidance/signals.yaml` | `tier-map-…` corrected to 1-for-4; a floor-bias measurement added |
| `SPEC-005`, `HANDOFF-022`, `HANDOFF-023` | records |

`src/`, `Cargo.toml`, `Cargo.lock` are untouched — verify that yourself.

## The fix, and the judgement inside it

**What changed in code.** `diff()`'s `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM) &&`
is gone; the now-unused import went with it. All eleven fields are compared
unconditionally.

**The judgement.** Round 1 said: *"decide there whether the guard stays (dead
until FU-1) or goes."* The orchestrator **removed** it, over the alternative of
correcting it to be genuinely generic. The stated reason:

> A dead guard is not neutral: it is a decision made in advance, on evidence that
> does not exist yet, that disarms the alarm which would have demanded it.
> Removing it makes `FU-1`'s fix **self-forcing** — fix `FU-1` and `K3III.DNG`
> reds immediately, so the real question gets decided deliberately and with a
> test, rather than absorbed silently.

**Round 1's own reviewer noted the opposite pull** — *"Fix that and the guard
becomes necessary. The record's conclusion may be right; its stated premise
isn't."* Both readings are defensible. **You are invited to disagree with the
call, not merely to check that it was executed.** If you think a corrected,
genuinely-generic guard was the better answer, say so and say why — that is a
legitimate `FU-8`, and possibly an `SB-2` if you think removing it loses a real
guarantee.

## THE ONE CLAIM YOU MUST NOT INHERIT

The new doc comment asserts, as a **measurement**:

> with the `FU-1` fix simulated — a one-element reading mapped to `Some([a, a])`
> instead of `None` — `metadata_matches_exiftool_on_every_corpus_file` fails
> immediately with `PENTAX-K3III-MONO/K3III.DNG: BlackLevelRepeatDim: ours=None,
> theirs=Some([1, 1])`

**Reproduce it.** The orchestrator ran that and nobody else observed it, which
makes it a self-report by exactly the rule this repo applies to everyone else
(§15 check 9). Patch `black_level_repeat_dim`'s parse in
`reading_from_fields` so a one-element reading survives, **assert the mutation
changed the file and compiled**, run the oracle, and confirm the failure names
that file and that field. Then restore byte-identical.

If it does *not* red, the doc comment is false and the whole
self-forcing-alarm argument for removing the guard collapses — that is an
`SB-2`, because it would mean the fix was justified by a measurement that does
not hold.

## Also check

1. **Is the removal actually behaviour-neutral today?** 87 tests summed across
   six targets, corpus present. Confirm nothing else moved.
2. **Does `DEC-013` now match what shipped?** It is `status: rejected` with the
   original preserved. Round 1's three counts should each be stated accurately —
   including count 3, which round 1 settled by *test* (malformed
   `BlackLevelRepeatDim` diffs `[]`; identically malformed `ActiveArea` still
   reds), not by reading. Check the record did not soften that.
3. **Did rejecting a decision leave dangling references?** `just decisions-audit`
   and `just decisions-index --check`. Does anything still cite `DEC-013` as
   though it were live — the spec, the CHANGELOG, `docs/`?
4. **`just validate`, `just cost-audit`, and the eight remaining gates.**
5. **Did the architect quietly widen scope?** Round 1 approved everything except
   `SB-1`. Anything in the delta that is not `SB-1`'s fix or its records is scope
   creep and should be called.

## Do NOT re-litigate

Round 1's `FU-1`…`FU-7` are **settled findings** and are dispositioned at ship,
not here. Do not re-argue them. The two claims round 1 killed — that `diff()` is
narrower than it looks, and that the dnglab uniqueness assertion is fake — are
**closed**; they were measured false (31 perturbations; six planted duplicates
all refused). Do not spend the round re-proving them.

`AC2` is met as written. Round 1 established that and the orchestrator accepted
the correction.

## Return Criteria

1. Ten gates plus `just oracle-meta` and `just decisions-audit`, re-run **by
   you**, pasted. Sum across **all six targets**.
2. **Both red-proofs watched failing by you** (§15 check 9) — the tier-A
   perturbation and the tier-B patched-tag, plus the simulated-`FU-1` alarm above.
3. **Fuzz** — the delta is test-only and `tests/support/tiff.rs` did not move, so
   a short run and a seed-hash comparison is enough. Say which you did.
4. Every mutation: **assert it changed the file and compiled** before concluding.
5. Fill the `handback:` with a real `tokens_total` **deduped by `message.id`**,
   and say you deduped. Compute `estimated_usd` **per-component at the rates for
   the model that actually ran** — read `message.model`, do not trust `tier_map`.
   ⚠ Round 1 recorded `8,500,000` as a floor; measured after that session closed,
   the same transcript gives `10,203,870` over 74 distinct ids — the convention
   runs ~17% low, always in the same direction. Capture as late as you can and
   say it is a floor.
6. **Correct `handoff.to_agent`** to what actually ran.
7. Do **not** run `just handback-sync`.
8. Findings labelled `SB-N` / `FU-N` continuing from **`FU-8`** / **`SB-2`**, each
   with which of the four dispositions you think it wants (§15).
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

**A clean ✅ is a fine outcome here and so is a second punch list.** The delta is
one condition, one import and one doc comment — but the last artefact the
architect self-graded was wrong three ways, and that is the prior you should
carry in.

## Handback

*(Filled by the reviewer. Mirror the `handback:` front-matter above.)*
