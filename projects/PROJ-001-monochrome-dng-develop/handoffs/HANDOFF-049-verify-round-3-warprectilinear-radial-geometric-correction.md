---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. This is verify round 3 — matches
# PATCH-003's pattern of a fresh handoff per verify round.

handoff:
  id: HANDOFF-049
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session.
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify. Rounds 1
                                    # and 2 verify both confirmed opus-5 as
                                    # the actual model; prediction likely
                                    # right this cycle too.
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-09-07
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-018

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# `notes:` MUST be ONE PHYSICAL LINE and MUST BE DOUBLE-QUOTED — FU-5's
# silent-truncate-at-# bug is still live (still open, ship dispositions it).
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one PHYSICAL, DOUBLE-QUOTED line
  synced_at: null                  # stamped by `just handback-sync` — do not edit
  verdict: null                    # approved | punch-list | rejected
---

# HANDOFF-049: Verify SPEC-018 round 3 — SB-3 closure

## Delegation Summary

Verify SPEC-018 round 3. HANDOFF-048's `⚠ PUNCH LIST (round 3)` raised
one ship blocker (SB-3: false DEC-024 citation in `src/warp.rs:53-55`)
and one follow-up (FU-11: cost.sessions agent-field mismatch). Round-3
build (HANDOFF-046 round-3 handback) claims SB-3 closed at ship SHA
`b92b30c` (branch tip `e976564` bookkeeping-only). 206/0/2 tests, CI
green (runs 34173566347 + 34173569505).

Your job: reconcile the round-3 build's claims against actual git+disk
state (DEC-004 rule 1), verify SB-3 is truly closed, confirm no NEW SB
introduced, and return one of:
  ✅ APPROVED (with SHA) / ⚠ PUNCH LIST (round 4) / ❌ REJECTED

FUs 1-11 are dispositioned at ship, not here.

## ⚠ This is even TIGHTER than round 2.

Round-3 build touched only `src/warp.rs` (10 insertions, 3 deletions, all
inside `//!`) and `HANDOFF-046` (handback). All 51 metered messages this
build were on claude-opus-5, priced $15.02 at Opus rates. If you find
yourself running the whole round-1 or round-2 verification pass, stop.

## The one SB — SB-3 fix

**Round-3 claim:** `src/warp.rs:53-55` (and surrounding block, expanded to
lines 47-77) now:
  1. Names only what DEC-024 actually holds (clamp-to-edge, in the
     Consequences "Neutral" line — no Alternatives-Considered record).
  2. Cites DNG § 6.4.1's SILENCE (not § 6.4.1 as the source of the rule)
     plus this spec's AC7 — the judgment call the round-3 build made to
     avoid becoming a third `unrun-docs-carry-errors` instance inside the
     correction pass.
  3. Corrects the AC8 measured-score imprecision — now names
     `-60.169 on L1021223.DNG` explicitly (per DEC-024:97-98).

**Your checks:**

1. **Read the current `src/warp.rs:47-77` rustdoc.** Does it:
   a. Correctly cite DEC-024's structure (Consequences "Neutral" line for
      the choice, Alternatives-Considered's Options A/B/C for kernel and
      threshold only, no out-of-extent alternative)?
   b. Cite DNG § 6.4.1's silence rather than § 6.4.1 as the source of the
      clamp rule? (Verify against DEC-024:195-202's own text about §6.4.1's
      silence.)
   c. Name `-60.169 on L1021223.DNG` for the AC8 measurement?

2. **Judge the round-3 build's judgment call.** The dispatch suggested
   wording "clamps to edge per DNG § 6.4.1" but the sub-agent recognised
   this would have introduced a THIRD `unrun-docs-carry-errors` instance
   (DEC-024:195-196 records § 6.4.1 is SILENT on out-of-extent). Their
   solution: cite § 6.4.1's silence plus AC7. Is this the right call?
   Does the current rustdoc's phrasing correctly reflect what DEC-024
   holds without over-generalising? If you would have written it
   differently, that is a taste finding (FU), not a defect (SB).

3. **grep the current tree for the false "zero-fill" claim.** It should NOT
   appear in `src/warp.rs`. `grep -rn zero-fill src/` should return 0
   hits.

## FU-11 stays open

The `cost.sessions` round-2 build entry still says `agent: claude-sonnet-5`
even though the session was opus-5. Round-3 build's entry will have the
SAME mis-attribution (`sonnet-5` when opus-5 actually ran) because
`HANDOFF-046`'s `to_agent` is still the round-1 stale value — the round-3
build sub-agent flagged this in their handback but respected the "don't
edit cost.sessions" boundary. This is expected; FU-11 remains open for
ship. Do NOT re-raise as SB.

## The 10 round-1 FUs stay open

- **FU-5** (handback-sync silent-truncate at bare `#`) — still live in the
  round-1 cost.sessions entry (`12/14 ACs green; AC8/AC9`).
- **FU-9** (LGPL source-reading question) — no new source reading in round 3.
- **FU-1..4, 6, 7, 8, 10** — untouched by round 3.

If round 3 accidentally re-opened any FU by mistake, THAT is a new SB.
Otherwise leave them.

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` (SB-3 closed, no new SB), `⚠ PUNCH
   LIST (round 4)` (SB-3 not closed or new SB found), or `❌ REJECTED`
   (round 3's diff broke something). Cite ship SHA (`b92b30c` or your
   measured SHA).

2. **The current `src/warp.rs:47-77` rustdoc quoted** — one block of the
   actual current text.

3. **DEC-024 clauses verified** — cite the lines you read to check each of
   the three fix rules (Consequences "Neutral" line for the out-of-extent
   choice; § 6.4.1 silence text at DEC-024:195-202; AC8 measurement at
   DEC-024:97-98).

4. **CI observed green** on the ship SHA. Round-3 handback cites runs
   34173566347 and 34173569505. Confirm.

5. **`cost.sessions` has 5 entries** — build round 1 (116,480,125,
   sonnet-5), build round 2 (14,298,209, sonnet-5 per FU-11), build round 3
   (5,579,812, sonnet-5 per FU-11), verify round 1 (22,631,969, opus-5),
   verify round 2 (6,790,688, opus-5). Round-3 build's entry will show
   sonnet-5 by FU-11 — that is the expected mis-attribution, not a
   defect.

6. **grep verification.** `grep -rn zero-fill src/` = 0 hits.

7. **PR NOT opened by verify.** Orchestrator handles PR #16 after your
   verdict.

## Cost

Fill `handback:` with real numbers. `notes:` MUST BE double-quoted, ONE
PHYSICAL LINE, NO bare `#`.

## References

- HANDOFF-048 (round-2 verify punch list — the SB-3 finding).
- HANDOFF-046's `## Completion` blocks — round 1, round 2, and round 3
  handbacks.
- DEC-024 — READ the specific line ranges above.
- `src/warp.rs` — the file that changed.

## Out of scope

- Round 1 and round 2 findings — closed.
- FUs 1-11 — ship.
- PR ops — orchestrator.
