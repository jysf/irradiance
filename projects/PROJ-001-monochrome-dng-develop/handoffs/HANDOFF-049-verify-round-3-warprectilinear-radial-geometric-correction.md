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
  status: completed                # completed | blocked | rejected
  tokens_total: 3275009            # REAL, deduped by message.id from my OWN transcript (scratchpad UUID 431a772f-caec-4323-b122-b8231db1ab28), 37 metered messages, all message.model claude-opus-5
  estimated_usd: 11.64             # per-component at Opus list rates: in 74 x $15 + out 27537 x $75 + cache-write 97014 x $30 (ALL 1h TTL this session, ephemeral_5m = 0, so $30 not $18.75) + cache-read 3150384 x $1.50 per Mtok = $9.70, + 20% handback uplift
  duration_minutes: 71             # wall clock; one 57-min idle gap awaiting continue, so 14 min active
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: 2026-09-07
  notes: "APPROVED on b92b30c - SB-3 CLOSED, no new SB, round 3's diff is comment-only. Reconciled against git and disk rather than the handback (DEC-004 rule 1): git diff 32599d2..b92b30c is src/warp.rs ALONE, 13 changed lines, 0 non-comment (asserted, not eyeballed), so no decoded pixel can move and SB-1's test, SB-2's src/lib.rs rewrite, src/develop.rs, tests/ and decisions/ are all byte-untouched this round. The three SB-3 defects are each fixed and each verified against the SOURCE document, not against the handback. (A) The invented alternatives are gone: grep -rn zero-fill src/ returns 0 hits (asserted), the only surviving source hits are tests/ifd_reader.rs 333 and tests/support/perturb.rs 14, both unrelated, and DEC-024 contains the string zero-fill 0 times. I asserted the ABSENCE claim itself rather than trusting it, since an absence is the easiest thing to assert falsely: 'alternativ' appears in DEC-024 EXACTLY ONCE, at line 138, the Alternatives Considered heading, and 'out-of-extent' appears EXACTLY ONCE, at line 195, so 'records the choice in a single Consequences Neutral line' and 'It records no alternative against it' are both literally true rather than merely plausible. Alternatives Considered spans 138-168 and holds exactly Options A (bicubic or Lanczos-3), B (relax DEC-005's 85 threshold) and C (chosen, keep bilinear plus the analytic checks), so 'entirely kernel and oracle threshold' is accurate. (B) AC11 is gone from src/ entirely - grep -rn AC11 src/ returns 0 hits - and AC7 is the right criterion: SPEC-018 at b92b30c lines 387-395 pre-registers the out-of-extent rule using AC7's own word 'pre-registered', and DEC-024 195-202 names AC7 (outside_pixels_follow_the_dng_spec_rule) as what pins the choice, on simplicity and determinism grounds, so the rustdoc matches the record clause for clause. The build's SPEC-018 line citations read 8 low against HEAD only because f9c43ee later inserted the round-3 cost entry above them; at the SHA the build actually read they are exact. (C) I RE-MEASURED AC8 instead of reading it back: cargo test --test warp -- --ignored --exact warp_scores_at_least_eightyfive_via_spec_020_oracle printed 'AC8 LEICA-Q2-MONO/L1021223.DNG: SSIMULACRA2 score = -60.169' in 135.92 s against dnglab 0.7.2, matching DEC-024 97-98 exactly. Worth recording that the round-3 handback's own verification story for this number is weaker than it reads - 'just test prints it back in the ignore reason' prints a STATIC string in tests/warp.rs, not a fresh measurement - but the number is right, I ran the reader, and the shipped sentence cites DEC-024, which does hold it. THE JUDGMENT CALL IS CORRECT AND I WOULD HAVE MADE IT. The dispatch's suggested 'clamps to edge per DNG 6.4.1' would have asserted a rule that DEC-024 195-196 records the specification does NOT hold, one screen below src/warp.rs line 36's own reverify-approved heading 'DNG 1.7.0.0 is silent; this build clamps' - unrun-docs-carry-errors landing inside the pass that exists to fix unrun-docs-carry-errors, which is exactly the instance-4 shape signals.yaml calls the one that settles it. The rewrite rests the rule on 6.4.1's SILENCE plus AC7 and on nothing else, and it did so without opening a specification it could not verify: it leaned on the two in-repo documents it did read. src/warp.rs cites 6.4.1 seven times and not once as the source of a clamp rule. The closing clause, 'those measurements are what established that the oracle cannot validate this stage at all', is honest rather than over-general: DEC-024's Finding 2 puts the measurements first and labels the dnglab/rawler source inspection at 116-130 as the ROOT CAUSE, and the rustdoc defers the why to the Kernel section below, which states it. No clause of the ten new lines over-generalises, so no section 16 rule 1 finding either. Gates re-run by me on b92b30c: just lint-ci green on the PINNED clippy 0.1.98 while this machine's default is 0.1.97; just test 206 passed / 0 failed / 2 ignored run TWICE, once with IRRADIANCE_CORPUS_DIR at the real corpus (7/7 present) and once with it UNSET (7 SKIP lines, default root absent), identical counts both ways, so the tier-B discipline holds; decisions-audit clean; DEC-024 confidence 0.85, so no section 16 yellow flag. CI GREEN on b92b30c confirmed by me from the API rather than inferred from a green predecessor: push run 34173566347 and pull_request run 34173569505, headSha b92b30cea0db66eeb0d2324fe7816d2365bc362f on both, conclusion success, 10 jobs each all success including fuzz smoke warp_opcode. cost.sessions holds 5 entries summing to 165,780,803, matching totals exactly, and 168.25 USD likewise; the round-3 build entry does carry agent claude-sonnet-5 while that session ran opus-5, which is FU-11 landing a second time exactly as HANDOFF-049 predicted, NOT a new finding. FU-1 through FU-11 are all still open and none was re-opened or silently closed by this round; FU-5's truncation is still visible verbatim at SPEC-018 line 111 as an unquoted notes value ending at 'AC8/AC9'. ONE OBSERVATION for ship, deliberately unnumbered because it predates round 3 and sits outside this handoff's scope: AGENTS.md line 1394 still states unrun-docs-carry-errors at N=5 while guidance/signals.yaml line 76 records N=6, instance 6 dated 2026-09-06 from SPEC-020 verify - the rule's own instance count is stale in the file agents read first, and this dispatch cited N=6. Cost method: 37 metered messages deduped by message.id from my OWN transcript, identified by the scratchpad UUID 431a772f-caec-4323-b122-b8231db1ab28 rather than by text-matching, all reporting message.model claude-opus-5, so tier_map.verify's prediction and this handoff's to_agent were both right for the third round running. Duration is WALL CLOCK 71 minutes, of which a single 57-minute idle gap awaited the user's continue, so active work was 14 minutes - recorded unhidden so calibration can discount it. PR not opened, handback-sync not run, cost.sessions not hand-edited, nothing repaired."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
  verdict: approved                # approved | punch-list | rejected
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
