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
  id: HANDOFF-031
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. The BUILD hint is 0 for 7.
                                   #   Read your own message.model and CORRECT this.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-04
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-013

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

# HANDOFF-031: Verify SPEC-013 — the plane oracle and its red-proof, at `88cc343`

## Delegation Summary

Verify `SPEC-013` at **`88cc343`** on `feat/spec-013-bit-exact-plane-oracle-red-proof`
(pushed, not merged; `main` at `9f269ed`). **This is a strong build — verify it
on that basis.** The risk is not sloppiness; it is a well-made oracle with a
coverage gap.

## What the orchestrator reconciled — reproduce, do not inherit

| claim | reconciled |
|---|---|
| branch + CI green on three SHAs | ✅ `f162a39`, `1f1bbbc`, `905a68a`, `88cc343` |
| `src/`, `Cargo.toml`, `Cargo.lock` untouched | ✅ `git diff main...HEAD` is **0 lines** on all three |
| 120 tests, 0 failed | ✅ summed, corpus present |
| **the red-proof works** | ✅ **run by the orchestrator**, watched: `honest=cb653b5bec24d166eef2fd258ee61ac4 mutant=59b032fe4320a27989ce61f3e3da7ff2` |
| the red-proof leaves the tree untouched | ✅ `git status` empty and `git diff HEAD` empty **after** running it |

⚠ **Credit where it is due, and it is a design the reviewer should understand
before critiquing:** the red-proof mutates a **temp-dir copy** of the crate and
rebuilds *that*, so the working tree is never touched and the whole thing runs in
**10.5 s**. The design session's own probe rebuilt in place, took minutes, timed
out twice, and left a stale process holding a mutated `src/plane.rs`. This is
strictly better than what the spec asked for.

**And the build did the thing the spec was written to force.** Its *first*
candidate fault changed the file, compiled, and produced `Error::Truncated`
rather than a wrong digest — because the strip is packed with **zero slack**
(`width × height × bits == StripByteCounts × 8` exactly), so any constant
additive shift runs one bit past the buffer on the final sample. It rejected
that fault and recorded why in `DEC-017`. That is `AC4`'s third clause working
on its first use.

## ⚠ The finding to confirm or kill — the orchestrator's, measured

**The red-proof passes vacuously where CI runs it.**

```
corpus present : an_injected_unpacker_fault_turns_the_oracle_red ... ok   (10.50s)
corpus absent  : all 10 tests "pass"                                      (0.00s)
```

`AC5` asked for a tier-A half and **got a real one** — the RFC vectors, the
streaming check, a hand-built fixture plane with a known digest, the locator, and
two PGM-parser tests all do genuine work with no corpus and no tools. That
criterion is met.

**But the red-proof is not in it.** The half CI can see contains no proof that
the oracle can fail. `constraints.yaml` was amended at STAGE-001's close to say
*a job that exists and has never passed is a deleted job*; the sibling case is a
**red-proof that exists and never runs**, and this project has now met that shape
four times (`SPEC-005/FU-3`, `SPEC-010/F-b`, `SPEC-012`, here).

Judge it. It is arguably **not** a defect — the red-proof genuinely works for
anyone holding the corpus, and `DEC-003` means CI can never hash a real plane.
But if it is acceptable, say so **with the reason**, because the alternative is
that a tier-A red-proof over the hand-built fixture is cheap and nobody thought
to ask for it.

## Your own checks

1. **Does the rebuild actually rebuild?** `DEC-017`'s mechanism copies, mutates,
   and rebuilds in release mode. If that rebuild silently failed or a stale
   artifact were reused, the test would compare a digest against itself. **Break
   the rebuild deliberately and confirm the test notices** — a red-proof whose
   apparatus can no-op is the exact defect `SPEC-013` exists to prevent, one
   level up.
2. **Is `the_honest_tree_is_the_negative_control` load-bearing?** Mutate it and
   see what dies. A control that cannot fail is not a control.
3. **Is `a_mismatch_names_the_first_differing_sample` exercised on a REAL
   mismatch**, or only a synthetic one? `AC3` exists because `SPEC-014` will
   debug 47 megapixels against this.
4. **MD5 beyond the RFC vectors.** Seven published vectors are the floor.
   Cross-check the implementation against the system `md5`/`md5sum` on something
   large and irregular — the corpus planes are right there.
5. **`compressed_files_are_skipped_by_name`** — does it assert the *reason*, or
   just that three files were skipped?

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Observe CI green on the SHA you approve.**
2. **Watch the red-proof fail yourself** (§15 check 9) and paste **both digests**.
3. **Fuzz** (§15 check 10) — `tests/` gained a lane; seeds unchanged is a fine
   result, say so.
4. **Provenance (§15 check 11):** MD5 row, class 1, RFC 1321, written from the
   standard and not from an implementation. Confirm it.
5. Every mutation: file changed **and** compiled **and** *output changed*. Stage
   your work before mutate-and-revert.
6. Handback with a real `tokens_total` **deduped by `message.id`** from your own
   transcript, priced **per-component** at the rates for the model
   `message.model` reports. ⚠ **Do not hand-write `cost.sessions`** — fill the
   handback block only, so `handback-sync` runs once cleanly. Hand-writing it has
   caused four duplicate-entry cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

*(Filled by the reviewer.)*
