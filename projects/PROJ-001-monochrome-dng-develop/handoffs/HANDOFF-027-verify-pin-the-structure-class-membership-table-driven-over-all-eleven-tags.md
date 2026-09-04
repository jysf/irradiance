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
  id: HANDOFF-027
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. tier_map is 1 FOR 6 — SPEC-009's
                                   #   build ran sonnet against this same hint. Read your
                                   #   own message.model and CORRECT this.
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-027: Verify SPEC-009 — the Structure-class membership, at `55a25f8`

## Delegation Summary

Verify `SPEC-009` at **`55a25f8`** on `feat/spec-009-pin-structure-class-membership`
(pushed, not merged; `main` at `e6cc561`). It closes four `SPEC-008` findings and
is **STAGE-002's gate on its own inputs** — the next spec is the unpack, and the
hazard this closes is a `Compression` of `RATIONAL 2/2` reading `1`, passing
`require_uncompressed()`, and the unpack reading JPEG as raw samples.

**This is a strong build. Verify it accordingly — the risk here is not sloppiness,
it is a well-made thing with a gap nobody thought to look for.**

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + CI green on `55a25f8` and `3b50964` | ✅ read off the runs |
| 100 tests, 0 failed | ✅ summed across targets, corpus present |
| the table is **independent** of `is_structural_tag()` | ✅ `const STRUCTURAL_TAGS: [u16; 11]`, hand-written |
| `AC5`'s precondition assertion | ✅ `assert_eq!(c.sensor_candidates(), vec![1]);` present |
| `src/` behaviour change | ✅ **none** — the only non-test edit is the `malformed_tags` doc comment |

**⚠ The eleven-way red-proof, run by the orchestrator, not taken on report:**
every membership deleted in turn, each mutation asserted applied by `diff` and
asserted to compile, tree restored byte-identical after each.

```
control (unmutated)                    100 passed, 0 failed
TAG_NEW_SUBFILE_TYPE … TAG_STRIP_BYTE_COUNTS   1 failed each  (10 tags)
TAG_SUB_IFDS                                    2 failed
```

**Eleven for eleven.** `SPEC-008/FU-1` — "one of eleven enforced" — is closed.

## Where to actually look

The mechanical claims hold. Spend your round on judgement, not re-counting.

1. **`AC2`'s other direction is the load-bearing half.** The eleven-way proof
   shows each tag *rejects* `RATIONAL`. It would pass identically if `uints()`
   rejected `RATIONAL` **universally**, silently undoing `SPEC-007`.
   `an_interpretation_tag_still_accepts_a_rational` is the only thing standing
   between us and that. **Mutate it: make `uints()` reject `RATIONAL`
   unconditionally and confirm that test — and ideally only that test — dies.**
2. **`DEC-015` chose Option B with zero code change.** Read whether the narrowed
   contract (`src/ifd.rs:553-569`) actually states the property `DEC-014`'s
   oracle exemption depends on: *a tag named in `malformed_tags` is one whose
   value the reader genuinely does not have, never one it recovered.* If that
   sentence is true, the exemption is sound; if it is only nearly true, `DEC-014`
   inherits the gap. This is the coupling the finding could not have known about
   when it was raised.
3. **`AC3`'s fixture.** `orientation_malformed_on_both_ifds_is_costed_once` is
   the guard `SPEC-008/FU-2` never had. Does it die when the combined
   `malformed.push` is split into one per erroring read? Measured on `main`
   before this spec: that split compiles and leaves everything green.
4. **A twelfth tag.** Adding one to `is_structural_tag()` without a table row is
   the *strict* direction and is explicitly a non-goal — but confirm the table
   would not silently drift out of sync with the predicate, and say whether that
   matters.

## ⚠ One finding that is the ORCHESTRATOR's, disclosed so you do not file it against the build

The build could not self-report `tokens_total` and asked the orchestrator to run
`/cost`. **That is not the build's fault.** `HANDOFF-024` named the transcript
method five times and its build self-reported without difficulty; **this
handoff mentioned it zero times.** Same requirement, method dropped between two
handoffs written by the same author.

Recovered from the build's own transcript instead: **20,412,565** deduped by
`message.id` (196 usage objects → 105 distinct ids, 1.86×, 98.4 % cache-read),
priced per-component at **Sonnet** rates because `message.model` reads
`claude-sonnet-5` on all 196 — so `tier_map` is now **1 for 6**. At Opus rates
the same session computes $43.45 (5.0×); at the repo's flat rate, $134.72
(15.5×).

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you and pasted; sum across all six
   targets. **Observe CI green on the SHA you approve.**
2. **Watch a red-proof fail yourself** (§15 check 9) — the eleven-way, or at
   minimum the `AC2` mutation in point 1 above, which nobody has run.
3. **Fuzz** (§15 check 10) — build claims 13.5 M runs, seeds unchanged.
4. Every mutation: **assert it changed the file and compiled** before concluding.
   Stage your work first.
5. Handback with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`, the session
   id is in the scratchpad path in your system prompt. Price per-component at the
   rates for the model `message.model` reports.
6. **Correct `handoff.to_agent`.** Do **not** run `handback-sync` or open the PR.
7. Findings `SB-N`/`FU-N` from 1, each with a §15 disposition. ⚠ A `spec:`
   disposition must **name an AC in that spec which would fail if the finding
   were left undone**.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

*(Filled by the reviewer.)*
