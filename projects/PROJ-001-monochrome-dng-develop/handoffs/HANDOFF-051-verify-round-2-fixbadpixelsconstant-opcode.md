---
# Maps to ContextCore handoff.* semantic conventions.
# ROUND-2 verify handoff — matches SPEC-018 HANDOFF-048 pattern.

handoff:
  id: HANDOFF-051
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session.
  to_agent: claude-opus-5           # ✅ VERIFIED, not inherited. Read from this
                                    # session's own transcript (scratchpad UUID
                                    # 0c0e3eaa-766d-473a-98c6-72479691d34a):
                                    # 76 entries carry message.model, every one
                                    # claude-opus-5. The prediction happened to
                                    # be right; it was checked before
                                    # handback-sync, per signal handback-sync-
                                    # inherits-stale-to-agent-across-punch-
                                    # list-rounds.
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-09-08
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-017

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# `notes:` MUST BE ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
# (signal handback-sync-treats-hash-as-comment-in-unquoted-notes).
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 3590211            # REAL, deduped by message.id from own transcript
  estimated_usd: 10.26             # per-component opus pricing, +20% uplift
  duration_minutes: 22
  branch: feat/spec-017-fixbadpixels-opcode
  pr: null                         # verify does not open the PR
  completed_at: 2026-09-08         # YYYY-MM-DD
  notes: "APPROVED on 6e4376b. SB-2 IS CLOSED and the closure has teeth, reproduced personally rather than read off the build's handback. SCOPE FIRST, git not prose: git diff cd82ca8..6e4376b -- src/ prints NOTHING, git log 8ae24e1..6e4376b is the single commit 6e4376b, and git show --stat 6e4376b is tests/develop.rs alone, 15 insertions 0 deletions; the branch tip 11d1c15 is bookkeeping-only, confirmed by git diff --stat 6e4376b..11d1c15 -- src/ tests/ Cargo.toml Cargo.lock printing nothing. THE ASSERTION exists at tests/develop.rs:594-599 inside develop_output_is_bit_identical_across_two_runs (AC9), after the dst1 == dst2 assert and before the direct-applier count block, and its message names the pipeline stage it protects. The index is CORRECT, checked against the fixture not assumed: minimal_sensor(5,5) with default_crop_size 5x5, orientation None and active_area None makes output_dimensions 5x5, so dst has 25 cells and 2 * 5 + 2 = 12 is row 2 col 2 - the same cell the test's own src marks with 0 at line 574. RED-PROOF REPRODUCED, all three clauses of the repo's mutation bar, in a scratchpad rsync copy minus target/ and .git/, working tree never touched and git status clean before and after. Match count asserted == 1 on the effective_src binding before substituting, per AGENTS.md section 16 rule 2. Clause one, the file CHANGED: honest md5 b3b922d0ebd499f54c58bd86b39cb3e8 -> severed 1c4f4e34eed721173b90134302ee4ca9, cmp confirms differ. That pair REPRODUCES the round-2 build's md5 pair EXACTLY, which retires the discrepancy the build reported honestly; round-1 verify's 548e240f still does not reproduce, and the build was right that its exact edit text is unrecoverable from what was recorded. Clause two, it COMPILED: cargo build --tests exit code 0, read directly rather than inferred from output text. Clause three, the OUTPUT CHANGED: AC9 panics at tests/develop.rs:594:5, left 0 right 1000, cargo test exit code 101. Then reverted to b3b922d0, cmp says byte-identical to the working tree's own src/develop.rs, AC9 green exit code 0. SUITE DELTA MEASURED with --no-fail-fast, which this repo has already paid for once: honest 231 passed / 0 failed / 3 ignored, mutant 230 / 1 / 3, and the one failure is develop_output_is_bit_identical_across_two_runs. Corpus-independence measured too, not assumed: the tier-A totals are the same 231 / 0 / 3 with IRRADIANCE_CORPUS_DIR unset as with it set, so tier-B passes either way. EVASION GREP CLEAN: dst1[2 * 5 + 2] occurs exactly once in the whole tree; src/warp.rs:385 and tests/support/perturb.rs:176 are warp-path fixtures, not develop_into; only three tier-A tests set opcode_list_1 and the other two build unknown opcode id 99, so AC9 is the only test that drives develop_into through a real FixBadPixelsConstant list. Nothing shadows or masks the new assertion. COST.SESSIONS has exactly 3 entries as briefed - build round 1 570000 on claude-sonnet-5, verify round 1 27235932 on claude-opus-5, build round 2 9026059 on claude-opus-5 - and the round-2 opus-5 attribution held, which makes three consecutive SPEC-017 handbacks (verify round 1, build round 2, this one) that set to_agent from a checked transcript; totals 36831991 and 93.77 both reconcile to the sum of the three rows. CI OBSERVED via gh, not inferred: push run 34202703543 and pull_request run 34202708489 on 6e4376b are 11 of 11 jobs success each, including rust / test, rust / clippy -D warnings and both fuzz smokes. I did NOT re-run local gates: round 2 changed one test file and CI's own PINNED clippy job passed on that exact SHA, which is the authority local unpinned 0.1.97 is not. FU-1 THROUGH FU-9 ALL STILL OPEN, spot-checked at the source rather than taken on trust: FU-4's false rustdoc line is still at src/develop.rs:90, FU-5's provenance row is still filed under src/opcode.rs at docs/provenance-ledger.md:44 while the median kernel lives in src/develop.rs, FU-8's AC5 text is still stale at the spec's line 387, src/ is byte-unchanged since cd82ca8 so FU-6 and FU-7 cannot have moved, and the spec's Follow-ups table at line 652 is still the unfilled template - ship still owes all nine dispositions. ONE NEW FINDING, FU-10, FOLLOW-UP not ship-blocking, and it is MEASURED rather than reasoned: the new assertion covers only ONE of effective_src's TWO consumers. They are crop_orient_normalize_into at src/develop.rs:826 (the no-warp branch, which AC9 takes) and normalize_active_area_into at src/develop.rs:842 (the warp branch). Severing ONLY the 842 use site - match count asserted == 1, md5 b3b922d0 -> 607c546a005ca1e1fe017a47d9bad3c2, compiles - leaves the ENTIRE tier-A suite green at 231 / 0 / 3. Zero tests anywhere set opcode_list_3, so no tier-A test drives develop_into with a fix opcode and a non-identity warp together, and tier-B is blind for round 1's own reason, HIT=0 making fixed_plane byte-identical to src on all three Q2M frames. The sting is that real Q2M frames DO carry a non-identity WarpRectilinear, so the branch real files actually take is the one still unasserted. Why FU and not SB: round 2 introduced nothing - src/ is byte-unchanged - the gap pre-dates round 2, and the build delivered EXACTLY the one-line fix round-1 verify itself pre-registered, so calling it an SB now would be re-scoping round 1 from the outside. It names one file and one fix (a synthetic warp-plus-fix fixture in tests/develop.rs), which is follow-up shape per section 15's spec-or-signal test, and it is dispositioned at ship alongside FU-1..9. Reporting it, per the standing instruction, not repairing it. NOTHING REPAIRED, PR 17 not touched, handback-sync not run, round-1 findings not re-audited, FU-1..9 not touched. Tokens are from THIS session's transcript, located by scratchpad UUID 0c0e3eaa-766d-473a-98c6-72479691d34a rather than text-matched: 41 assistant messages deduped by message.id, all 76 model-bearing entries claude-opus-5, summing input 82 + output 21505 + cache-write 91809 + cache-read 3476815 = 3590211. Priced per component at opus rates 15 / 75 / 18.75 / 1.50 per MTok = 8.55 base, plus the 20 percent uplift for the turns after measurement = 10.26."
  synced_at: 2026-09-08
  verdict: approved                # approved | punch-list | rejected
---

# HANDOFF-051: Verify SPEC-017 round 2 — SB-2 closure

## Delegation Summary

Verify SPEC-017 round 2. HANDOFF-050's `⚠ PUNCH LIST` raised one ship
blocker (SB-2: `develop_into` wiring untested — mirror of SPEC-018/SB-1)
and eight follow-ups (FU-1..9). Round-2 build (HANDOFF-044 round-2
handback) claims SB-2 closed at ship SHA `6e4376b` (branch tip `759f200`
bookkeeping-only) via a single 6-line assertion at
`tests/develop.rs:593-600`. 231/0/3 tests, CI green on both SHAs (11
jobs each including fuzz smoke opcode).

Your job: reconcile the round-2 build's claims against actual git+disk
state (DEC-004 rule 1), verify that SB-2 is truly closed and has teeth,
confirm no NEW SB was introduced, and return one of:
  ✅ APPROVED / ⚠ PUNCH LIST (round 3) / ❌ REJECTED

FUs 1-9 stay OPEN and are dispositioned at ship. Do not re-open them.

## ⚠ This is even TIGHTER than round 1.

Round-2 build touched only `tests/develop.rs` (15 insertions, 0 deletions,
all inside a single test function). No src/ changes. No DEC-024 changes.
No spec changes. If you find yourself running the whole round-1
verification pass, stop.

## The one SB — SB-2 fix

**Round-2 claim:** `develop_output_is_bit_identical_across_two_runs`
gained a centre-pixel assertion at `tests/develop.rs:594-599`:

```rust
assert_eq!(
    dst1[2 * 5 + 2],
    1000,
    "develop_into must develop the plane the applier FIXED: the centre bad \
     pixel must reach dst as its neighbours' median, not the raw marker"
);
```

The fixture in that test already makes `normalize` an identity map, so
`dst1[centre]` equals 1000 (neighbours' median) if the fixed plane
flows through `develop_into`, and 0 (the raw marker `Constant`) if the
wiring is severed.

**Round-2 red-proof (build):** mutation `effective_src = src` md5
`b3b922d0` → `1c4f4e34`, compiles, assertion turns red at line 594
("left: 0 / right: 1000"); reverted, md5 back to `b3b922d0` byte-
identical, green. Suite 231/0/3 → 230/1/3 under the mutation.

⚠ **The build noted:** verifier's original md5 (`548e240f` in
HANDOFF-050) did not reproduce — same semantic sever, different exact
text, so hashes diverge. If you reproduce the mutation, your md5 may
differ from both build's and round-1 verify's. What matters is that the
mutation semantically severs `effective_src → src` in `src/develop.rs`,
the assertion turns red, and the code reverts byte-identical. Judge on
the behaviour, not the hash.

**Your checks:**

1. **Read `tests/develop.rs:590-605`.** Confirm the assertion is there,
   the message names the pipeline stage it protects, and the index
   `2 * 5 + 2` correctly names the centre of a 5×5 plane.

2. **Reproduce the red-proof yourself.** Sever `effective_src → src` in
   `src/develop.rs`. Compile. Run the AC9 test. Watch it fail at line
   594. Revert. Confirm the tree is byte-identical (any md5 comparison,
   just show the pair). Watch it pass. §16 rule 2 three-clause discipline.

3. **grep the test tree for the assertion's evasion routes.** Does any
   other test compare `dst1[centre]` in a way that would mask the
   failure if the wiring severed? (Verifier's round-1 check established
   231 tests stayed green under the severing mutation; a NEW test that
   accidentally covers the same seam differently could shadow the fix.)

4. **Confirm src/ is untouched by round 2.** `git diff cd82ca8..6e4376b
   -- src/` should print nothing. `git log --oneline 8ae24e1..6e4376b`
   should show only tests/develop.rs.

## FU-5's applier-location note (build correctly flagged)

Round-2 build noted that the applier `apply_fix_bad_pixels_constant`
lives in `src/develop.rs`, not `src/opcode.rs` as the round-2 dispatch
brief said. FU-5 from round-1 verify already covers this split (the
provenance row is filed under `src/opcode.rs` but the applier is
actually in `src/develop.rs`). No new finding — the build's flag is
a courtesy pointer. Do NOT re-raise as SB.

## The FUs from round 1 stay open

FU-1 through FU-9 remain OPEN and are dispositioned at ship. If you see
one still live in the code (they will be), it stays live. FU-11-shape
issues (agent field on cost.sessions) will NOT fire this round because
build correctly set `to_agent: claude-opus-5` before handback-sync ran
— confirm by reading the round-2 build entry.

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` (SB-2 closed, no new SB),
   `⚠ PUNCH LIST` (SB-2 not closed or new SB found), or `❌ REJECTED`
   (round-2 diff broke something). Cite the ship SHA (`6e4376b`).

2. **SB-2 red-proof reproduced with the semantic mutation** — show the
   assertion failing under the sever, then passing after revert. The
   md5 pair need only demonstrate the tree changed and reverted; the
   exact hash needn't match build's or round-1 verify's (build noted
   this is unreproducible from what round-1 verify recorded).

3. **`cost.sessions` has 3 entries:** build round 1 (570,000 sonnet-5),
   build round 2 (9,026,059 opus-5), verify round 1 (27,235,932
   opus-5). Round-2 build's opus-5 attribution should show — the
   FU-11 discipline held for the second time. Confirm.

4. **CI observed green** on `6e4376b`. Round-2 handback cites 11/11 on
   push and PR. Verify with `gh run list`.

5. **The 8 FUs from round 1 recognisably still there** — none was
   secretly closed as a side effect of round 2.

6. **PR NOT opened by verify.** Orchestrator handles PR #17.

7. **CORRECT this handoff's `to_agent`** to your actual `message.model`
   BEFORE handback-sync runs.

## Cost

Fill `handback:` with real numbers. `notes:` DOUBLE-QUOTED, ONE LINE,
NO bare `#`. Identify transcript by scratchpad UUID.

## References

- HANDOFF-050 (round-1 verify) — the SB-2 finding.
- HANDOFF-044 `## Completion` — round 1 and round 2 build handbacks.
- `tests/develop.rs:590-605` — the assertion.
- `src/develop.rs` — the `effective_src` plumbing you mutate.

## Out of scope

- Rounds 1's findings — SB-1→FU-3 downgrade and FU-1..9 all stand.
- Ship dispositions.
- Opening PR #17.
