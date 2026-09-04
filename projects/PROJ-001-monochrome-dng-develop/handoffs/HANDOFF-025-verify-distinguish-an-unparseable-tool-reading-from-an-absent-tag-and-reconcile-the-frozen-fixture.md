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
  id: HANDOFF-025
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT from tier_map.verify, NOT a record.
                                   #   tier_map is 1 for 4 (SPEC-007/FU-6). Read your own
                                   #   message.model and CORRECT this before handing back.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-03
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-010

project:
  id: PROJ-001
  stage: STAGE-005
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

# HANDOFF-025: Verify SPEC-010 — the tri-state tool reading, at `f4841b3`

## Delegation Summary

Verify `SPEC-010` at **`f4841b3`** on `feat/spec-010-tri-state-tool-reading`
(pushed, not merged; `main` at `2c0aaed`). The metadata oracle can now tell an
**absent** tag from an **unreadable** one, and an unreadable reading is a
mismatch unless `Sensor::malformed_tags` names the same tag.

## ⚠ READ THIS FIRST — the shipped code is a RECONSTRUCTION

The build disclosed, unprompted, that during its own red-proof work
`git checkout -- tests/support/tools.rs` **wiped the entire SPEC-010 change**,
not just the temporary mutation, because nothing was staged. It **redid the
edits from context** and re-verified.

Self-caught and disclosed, which is the right behaviour and should be credited.
But it means **the shipped implementation is a second writing of itself**, and
the only thing standing between it and a silently dropped requirement is that
the tests pass — and those tests were written by the same session, in the same
sitting, partly before the wipe.

**Treat every acceptance criterion as unverified.** Do not spot-check. The
failure mode to hunt is not a bug; it is an *omission* that no existing test
covers because the test that would have covered it was never rewritten.

## What the orchestrator already reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + both SHAs on `origin` | ✅ `23e413f`, `f4841b3` |
| **CI green on both** | ✅ checked on the runs, not the record — `constraints.yaml` requires the observation |
| `src/`, `Cargo.toml`, `Cargo.lock` untouched | ✅ `git diff main...HEAD` empty on all three |
| 95 tests (was 87), 0 failures | ✅ summed across all six targets, corpus present |
| all 8 named tests exist **exactly once** | ✅ per-target `-- --list`, anchored match, each `1` |
| `ToolValue<T>` is a real tri-state | ✅ `Absent` / `Unreadable(Vec<u32>)` / `Value(T)`, raw values preserved |
| `compare_optional` is **one generic arm** | ✅ per-*state*, not per-tag — the guard `DEC-013` chose and shipped wrong |
| `diff()` not dangling | ✅ a one-line wrapper over `diff_with_malformed(.., &sensor.malformed_tags)`, 10 callers |
| `DEC-014` `accepted`, `DEC-013` still `rejected` | ✅ |

**Two things the build did better than the spec asked**, worth confirming rather
than assuming: the red-proof is **in-test and permanent**, not a one-off manual
mutation — and it exercises the **real shipped comparator** via
`diff_with_malformed(sensor, reading, &[])` rather than a hand-written
re-derivation of it.

## Three findings to confirm or kill

**F-a — `req()` still truncates to its head, and `AC4` may be narrower than the
finding it carries.** `tests/support/tools.rs:296`. The build **documented** the
scope call inline with reasoning, and `AC4` as I wrote it (`BlackLevel = "512
999"` must not read `Some(512)`) **is** met, because `BlackLevel` is optional.
But `SPEC-005/FU-2`'s stated hazard was *"latent on today's mono corpus, **live
at `SamplesPerPixel > 1`**"* — and `BitsPerSample` on a 3-sample file reads
`"8 8 8"`, which `req()` still silently takes as `8`. So `FU-2` is **not closed**
even though its owning AC passes. ⚠ **That is my AC's imprecision as much as the
build's scope call** — judge it as a design finding if you agree, and say so.

**F-b — the red-proof passes vacuously without a corpus.** Measured:
`IRRADIANCE_CORPUS_DIR=/nonexistent cargo test --test metadata_oracle` →
**29 passed in 0.06 s**, including
`removing_the_malformed_comparison_turns_k3iii_red` and its control. This is
`SPEC-005/FU-3`'s shape, now on the **red-proof itself**, which is worse than
where it was found: a proof that cannot run is indistinguishable from a proof
that passed. `just test` names the missing files first, so it is visible through
the recipe — confirm that, and judge whether it is enough.

**F-c — `DEC-013` kept `rejected` with a pointer rather than `superseded_by:
DEC-014`.** The build's reasoning is that `DEC-013` was *wrong on three counts*,
not merely improved upon, and `superseded` would imply the latter. That is a real
judgement about what the two states mean. Check `decisions-audit` treats the
pair sanely and that a reader landing on `DEC-013` is actually routed to
`DEC-014`.

## Your own checks — the list above is not the job

The most valuable outcome is a **fourth** finding, and the reconstruction is
where to look. Suggestions in this repo's grain:

1. **Walk `SPEC-010`'s eight ACs against the code**, not against the test names.
   `AC5` (fixture reconciled against the live tool) and `AC7` (the doc comment
   and `DEC-013` brought true) are prose-shaped and the likeliest to have been
   lost and not noticed.
2. **Mutate each arm of `compare_optional` in turn** — `Absent`, `Value`,
   `Unreadable` — and confirm each has a test that dies. One generic arm is
   elegant and is also three behaviours behind one `match`.
3. **Check `tri_state`'s classifier per field.** `Value` only for the exact
   arity: does a 3-element `ActiveArea` really become `Unreadable` and not
   `Value` of something?

## Return Criteria

1. **Ten gates + `just lint-ci` + `just oracle-meta`**, run by you, pasted.
   Sum across **all six** targets; a zero-match `cargo test <name>` exits 0.
2. ⚠ **`just lint-ci`, not `just lint`** — local clippy is 0.1.97, CI floats at
   0.1.98. And **observe CI green on the SHA you approve**, per
   `constraints.yaml` as amended at STAGE-001's close.
3. **Watch the red-proof fail yourself** (§15 check 9), with the corpus present.
4. **Fuzz** (§15 check 10) — the build claims 10.9 M execs, seeds unchanged.
5. Every mutation: **assert it changed the file and compiled** before concluding.
   ⚠ And heed the build's own lesson — **stage your work before running
   mutate-and-revert experiments on a file you have edited.**
6. Handback with a real `tokens_total` **deduped by `message.id`**, said so, and
   `estimated_usd` per-component at the rates for the model that **actually
   ran** (`message.model`, not `tier_map`). Capture late — the floor convention
   measured ~17% low.
7. **Correct `handoff.to_agent`** to what actually ran.
8. Do **not** run `just handback-sync`; do **not** open the PR.
9. Findings `SB-N` / `FU-N` for **this spec** from 1, each with which of §15's
   four dispositions you think it wants. ⚠ `SPEC-005/FU-9` (`is_active()` ignores
   `status`) is confirmed still open and is **out of this spec's `tests/`-only
   scope** — the build flagged rather than fixed it, which was correct. It needs
   a disposition at ship, not a fix here.
10. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

*(Filled by the reviewer. Mirror the `handback:` front-matter above.)*
