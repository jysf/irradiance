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
  id: HANDOFF-029
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. The BUILD hint is now 0 for 6.
                                   #   Read your own message.model and CORRECT this.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-04
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-012

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

# HANDOFF-029: Verify SPEC-012 — the unpack, at `1606d4b`

## Delegation Summary

Verify `SPEC-012` at **`1606d4b`** on `feat/spec-012-strip-location-and-sample-unpack`
(pushed, not merged; `main` at `a36582d`). **This is the first spec in the project
that produces pixels.**

⚠ **A worktree for this branch is live at
`~/PSeven/experiments/crustimg_redo_plus/irradiance-build-spec-012`.** Work there,
or in your own — do **not** try to check the branch out in the main checkout.

## ⚠ The headline result, measured by the orchestrator — and it changes your job

`SPEC-012` deliberately did **not** build the MD5 oracle (that is `SPEC-013`), so
the in-repo evidence for correctness is the first eight samples plus min/max.
The orchestrator built a throwaway probe against the shipped `unpack_into` and
compared the **whole plane** to `dnglab analyze --raw-checksum`:

| file | shape | whole-plane MD5 |
|---|---|---|
| `L1021223.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1026016.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1026192.DNG` | 8424×5632, 14-bit | ✅ **match** |
| `L1000622.DNG` | 5216×3472, 16-bit | ✅ **match** |

Four for four, both `DEC-008` paths, two camera bodies, and every digest equals
the value already pinned in `tests/corpus/manifest.toml`. **The unpacker is
bit-exact today.**

So do not spend your round hunting for a wrong plane — it is right. Spend it on
**everything the checksum cannot see**: hostile input, the fuzz target's actual
reach, the error paths, and the claims that go beyond what was measured.

## What else the orchestrator reconciled

| claim | reconciled |
|---|---|
| branch + both commits on `origin`, CI green | ✅ `731a891`, `1606d4b` |
| first samples on both paths | ✅ `[746, 725, 711, 752, …]` and `[4761, 4591, 4622, 4363, …]` — identical to the design-time probe |
| `max <= WhiteLevel` holds | ✅ both files report `16383 <= 16383` — at the boundary, which is the interesting case |
| `DEC-016` shape | ✅ `unpack_into(&mut [u16])`, no allocation, length checked |
| fuzz seeds reach both paths | ✅ `valid-fourteen-bit.tiff` and `valid-sixteen-bit.tiff` both present |
| 110 tests, 0 failed | ✅ summed across targets |

## Where to look

1. **`AC7` — panic-freedom is the half a checksum cannot certify.** The plane is
   right on four good files; the spec's real risk is the other input space.
   Drive the fuzz target yourself and **say how you know it reached the 16-bit
   path**, not just that seeds exist. `SPIKE-001`'s blind spot was exactly a
   parameter that was always 14.
2. **`AC3` is the assertion that would have caught `SPIKE-002`.** Confirm it
   asserts the *measured impossible values* (43019, 39186) and not merely that
   two outputs differ. A test asserting "differs" passes for the wrong reason
   forever.
3. **`AC4` at the boundary.** Both real files hit `max == WhiteLevel` exactly.
   Does the check use `>` or `>=`? A `>=` would reject every honest Q2M frame.
   This is one character and the corpus sits right on it.
4. **`AC8`'s 182 MB.** The build measured peak RSS rather than estimating, which
   is what was asked. Sanity-check the method: plane is 94.9 MB and the input is
   86 MB, which sums suspiciously close to 182 — is the file being held entirely
   in memory alongside the plane, and is that a finding for `DEC-002`?
5. **The build reports catching a local/CI clippy-version gap.** Confirm what it
   was; that is `just lint-ci` doing the job `PATCH-001` created it for, and it
   is worth recording as evidence either way.

## One thing the orchestrator did to this branch

`HANDOFF-028`'s `synced_at` is **hand-stamped**, not written by `handback-sync`.
The build had already hand-written this cycle's cost session with the correct
figure, and running the script would have appended a **second identical entry** —
the bug `SPEC-003` first warned about, `SPEC-010/FU-2` hit with two identical
figures and `SPEC-009/FU-2` with a null beside a real one. **Fourth occurrence.**
Prevented rather than merged after the fact. The reason is inline in the field.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Observe CI green on the SHA you approve.**
2. **Watch a red-proof fail yourself** (§15 check 9).
3. **Fuzz (§12 bar 2 / §15 check 10) is the centre of this round**, not a
   formality — build claims 19.2 M combined runs across both paths.
4. **Provenance (§15 check 11):** confirm the ledger row exists with an honest
   class, and that `SPIKE-001`'s discarded decoder was **not** consulted — the
   handoff forbade it and `provenance-recorded-per-algorithm` is blocking.
5. Every mutation: **assert it changed the file and compiled** first. Stage your
   work before mutate-and-revert.
6. Handback with a real `tokens_total` **deduped by `message.id`** (read your own
   transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`), priced
   per-component at the rates for the model `message.model` reports.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Handback

*(Filled by the reviewer.)*
