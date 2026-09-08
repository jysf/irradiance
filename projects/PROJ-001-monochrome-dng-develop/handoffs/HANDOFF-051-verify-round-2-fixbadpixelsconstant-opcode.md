---
# Maps to ContextCore handoff.* semantic conventions.
# ROUND-2 verify handoff — matches SPEC-018 HANDOFF-048 pattern.

handoff:
  id: HANDOFF-051
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session.
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify. Prior
                                    # SPEC-017 verify (HANDOFF-050) confirmed
                                    # opus-5. ⚠ CORRECT to your actual
                                    # message.model BEFORE handback-sync runs
                                    # (signal handback-sync-inherits-stale-
                                    # to-agent-across-punch-list-rounds).
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count
  estimated_usd: null              # tokens_total × your rate
  duration_minutes: null
  branch: feat/spec-017-fixbadpixels-opcode
  pr: null                         # verify does not open the PR
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one PHYSICAL, DOUBLE-QUOTED line, NO `#`
  synced_at: null                  # stamped by `just handback-sync`
  verdict: null                    # approved | punch-list | rejected
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
