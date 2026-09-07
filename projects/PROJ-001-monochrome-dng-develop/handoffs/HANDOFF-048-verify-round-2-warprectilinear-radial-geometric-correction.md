---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. This is the ROUND-2 verify handoff —
# matches HANDOFF-041's PATCH-003 pattern (a fresh handoff for round-2 verify).

handoff:
  id: HANDOFF-048
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session
                                    # (correction from tier_map.design's
                                    # claude-opus-5 prediction).
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify. Note:
                                    # round 1's verify (HANDOFF-047) confirmed
                                    # opus-5 as the actual model, so the
                                    # prediction is likely right this time too.
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
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `notes:` MUST be ONE PHYSICAL LINE and MUST BE QUOTED (or MUST NOT contain
# a bare `#`, per FU-5 which established handback-sync silently truncates
# at `#` in unquoted YAML).
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 6790688            # REAL, deduped by message.id from my OWN transcript, identified by scratchpad UUID 533f8ae7-59f2-4439-9c49-11c160ee2ff4 (not text-matched). All 64 metered messages report message.model claude-opus-5, so tier_map.verify's prediction and this handoff's to_agent were both right.
  estimated_usd: 17.38             # per-component (in 128, out 30994, cache-write 117179, cache-read 6642387) at Opus-tier rates ($15/$75/$18.75/$1.50 per Mtok) = $14.49, + 20% handback uplift
  duration_minutes: 16
  branch: feat/spec-018-warprectilinear-radial-geometric-correction
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: 2026-09-07
  notes: "PUNCH LIST (round 3) on 08ad42e - CI green there, run 34164609782, 10 jobs all success including the warp_opcode fuzz smoke. SB-1 CLOSED. Ran the test with --exact: exactly one match, passed, and the fixture prints 2837 of 2880 developed pixels separating the two orders, reproducing the build claim exactly. Reproduced the red-proof myself on the SHIPPED file: src/develop.rs md5 11118242d58a04ce60351593b2e437f1 honest -> f620d615c80da5474f46bb9d11f7ec44 crop-then-warp -> 11118242d58a04ce60351593b2e437f1 reverted byte-identical; the mutation compiles and moves real output (irr develop L1021223.DNG samples[0..8] [2028,1855,1818,1868,1805,1889,1943,2013] max 51764 -> [1801,2001,2036,1972,1819,1689,2142,2185] max 60918), suite 206/0/2 -> 205 passed / 1 FAILED / 2 ignored with the one failure being the new test at develop_into = 4130 vs warped-ActiveArea 3948, and round 1s 205 all green under it. Note the build handbacks md5 pair 318977b6 -> cf714e8c is src/develop.rs at 40f5d45, i.e. before round 2s own rustdoc edit to that file, so the shipped file was never itself the subject of the recorded pair - my pair above is on the shipped file and is the one that satisfies DEC-004 rule 1. SB-2 CLOSED for both claims it named: src/warp.rs Kernel section no longer fabricates >=85 per-frame scores and now rests the choice on AC4/AC10 citing DEC-024 Finding 2 (AC10 re-run here: 339.5 px against the 20 px bar; AC4 re-run: 503.7 / 437.7 / 407.0 px per-frame, within 1 px of SPIKE-001), and src/lib.rs 56-62 now states the shipped order - after normalize, over the full ActiveArea, before DefaultCrop and Orientation - citing DEC-024 Finding 1, verified against src/develop.rs 575-613 and against git show 40f5d45 for its self-reference. The incidental third-caller-supplied-buffer correction is HONEST: DEC-024 Alternatives records only Options A, B and C (kernel and threshold), no caller-supplied-buffer option, and its Consequences does record the two ActiveArea-sized scratch buffers at 465,010,688 bytes peak RSS, matching docs/provenance-ledger.md src/warp.rs row. NEW: SB-3, ship-blocking, same species as SB-2 and introduced by SB-2s own edit. src/warp.rs 53-55 now reads The alternatives (zero-fill, error) are recorded in the kernel-choice decision, DEC-024 (AC11) - DEC-024 records NEITHER; repo-wide grep for zero-fill returns exactly 3 hits, two unrelated (tests/ifd_reader.rs, tests/support/perturb.rs) and the third being this claim itself, and DEC-024s only out-of-extent text is one Consequences-Neutral line naming clamp-to-edge with nothing it was chosen over. Round 2 resolved this dangling DEC-* placeholder to a concrete id without reading DEC-024 to check the neighbouring claim - the exact check it DID perform for the develop.rs placeholder - converting an unverifiable pointer into a verifiably false citation with more authority, which is unrun-docs-carry-errors (AGENTS.md 16 rule 4) landing inside the correction pass itself. AC11 is also the wrong AC to tag: it governs the KERNEL choice (bilinear/bicubic/Lanczos and measured scores), not the out-of-extent rule. Same sentences which records no per-frame oracle scores is imprecise too - DEC-024 line 98 records -60.169 on L1021223.DNG - and the SB-3 fix should rewrite the whole sentence. NEW: FU-11, follow-up. cost.sessions round-2 build entry carries agent: claude-sonnet-5 while that sessions own handback documents claude-opus-5 across 97 metered messages and prices $34.26 at Opus rates; scripts/handback-sync.sh line 97 reads to_agent from the handoff, and HANDOFF-046s to_agent was left at round 1s claude-sonnet-5 when round 2 reused the file. Bad model attribution reaches just calibration silently; one-field fix at ship, gates nothing, so FU not SB. Round-2 diff is otherwise clean: src/ changes are comment-only (0 non-comment changed lines, asserted), no existing test weakened (only 4 doc-header lines removed from tests/warp.rs), DEC-024 byte-unchanged since 40f5d45, SPEC-018.md body and ACs untouched, decisions-audit 0 structural errors, lint-ci green on pinned clippy 0.1.98, fmt clean, DEC-024 confidence 0.85 so no 16 yellow flag. FU-1 through FU-10 all verifiably still live and still owed a ship disposition - FU-5s truncation is still visible as notes: 12/14 ACs green; AC8/AC9 in the round-1 build session. cost.sessions structure is otherwise coherent: 3 entries totalling 153,410,303 which sums correctly. PR not opened; nothing repaired."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
  verdict: punch-list              # approved | punch-list | rejected
---

# HANDOFF-048: Verify SPEC-018 round 2 — SB-1 and SB-2 closure

## Delegation Summary

Verify SPEC-018 round 2. HANDOFF-047's `⚠ PUNCH LIST` at `40f5d45`
raised two SBs and 10 FUs; round-2 build (HANDOFF-046 round-2
handback) claims SB-1 and SB-2 are closed at ship SHA `08ad42e`
(branch tip `957612e` bookkeeping-only). 206/0/2 tests, CI green,
fuzz-warp smoke green.

Your job: reconcile the round-2 build's claims against actual
git+disk state (DEC-004 rule 1), verify that BOTH ship blockers are
truly closed, confirm no NEW SBs were introduced, and return
`✅ APPROVED` / `⚠ PUNCH LIST` (round 3) / `❌ REJECTED`. The 10
FUs from round 1 STAY OPEN and are dispositioned at ship — do not
re-open them here.

## ⚠ This is a TIGHT verify. Do not re-audit round 1.

HANDOFF-047 already verified rounds 1's substantive findings (Finding 1
pipeline order, Finding 2 dnglab-doesn't-implement-opcodes), the four
repo-specific bars, the eight generic checks, and the code + fuzz +
license + provenance. Round 2 is docs + tests only — `irr develop
L1021223.DNG samples[0..8]` is byte-identical before and after per
the round-2 handback. Focus your verify on the TWO SBs; if you find
yourself running the whole round-1 verification pass, stop.

## The two SBs — what to check

### SB-1 — `develop_into`'s warp branch has a live test

**Round-2 claim:** `develop_into_crops_from_the_warped_active_area_
not_the_warped_crop` is the only test in the tree that develops a
`Sensor` with `opcode_list_3: Some(...)` all the way to pixels.
Hand-built 96×72 raw plane, ActiveArea 80×64, DefaultCrop 60×48 at
origin (7,5) — strictly off-centre so the two candidate orders use
different optical centres and half-diagonals. Real L1021223
OpcodeList3 bytes, affine ramp plane. Three assertions: (a) fixture
separates 2837/2880 pixels between orders, (b) `develop_into` equals
`crop(warp(active))` and not `warp(crop(active))`, (c) identity warp
develops bit-identically to no opcode list. Red-proof: reverting
`src/develop.rs` to crop-then-warp turned the test red (205 passed /
1 FAILED / 2 ignored) while round 1's 205 all stayed green under the
same mutation.

**Your checks:**

1. **The test EXISTS and RAN.** `cargo test --all-features --test warp
   develop_into_crops_from_the_warped_active_area_not_the_warped_crop
   -- --exact --nocapture` — confirm exactly one test matched and it
   passed. The `named-tests-can-pass-vacuously` trap fires when a
   partial match runs zero tests.

2. **The test has teeth.** Mutate `src/develop.rs` to crop-then-warp
   yourself. It MUST compile, change output, and turn the new test
   red — while leaving round 1's 205 green. Show mutation md5s before
   and after. This is where §16 rule 2 (three-clause mutation) bites.

3. **The fixture separates.** Round-2 build asserted 2837/2880
   pixels differ between the two orders — if the fixture is degenerate
   (an order-agnostic input like all zeros or a symmetric pattern),
   the test passes vacuously. Verify by reading the fixture in the
   test and running it under the crop-then-warp mutation to observe
   the 43-pixel identity subset (2880 - 2837).

4. **The identity-warp assertion.** Set the WarpRectilinear
   coefficients to `kr0 = 1.0, kr1 = kr2 = kr3 = 0.0, kt0 = kt1 = 0.0`
   in a hand-built opcode and confirm `develop_into` output is
   BIT-IDENTICAL to the same input with `opcode_list_3: None`.

### SB-2 — two false rustdoc claims corrected, citing DEC-024

**Round-2 claim:** `src/warp.rs:59-61` (fabricated ≥85 oracle scores)
and `src/lib.rs:56-57` (pre-Finding-1 pipeline order) both rewritten
to match reality, both cite DEC-024. Additionally, three dangling
`DEC-*` placeholders resolved to DEC-024 as incidental cleanup;
one exposed a further false claim of the same species (a
third-caller-supplied-buffer alternative DEC-024 does not hold), also
corrected.

**Your checks:**

1. Read the current `src/warp.rs:52-77` (post-shift) rustdoc — does
   it now correctly attribute the kernel choice to AC4/AC10's
   analytic verification and cite DEC-024 by id? Does it NOT
   fabricate per-frame oracle scores?

2. Read the current `src/lib.rs:56-58` — does it state the actual
   pipeline order (over full ActiveArea, before DefaultCrop) and
   cite DEC-024 Finding 1?

3. Read the incidental "third-caller-supplied-buffer" correction:
   verify DEC-024 does NOT record a third alternative for that
   caller shape, and the current rustdoc matches DEC-024's actual
   holdings. (This is the small-scope FU-1-shape scan the round-2
   dispatch's "do not chase FUs" boundary explicitly allowed.)

## The FUs from round 1 stay open

FU-1 through FU-10 are dispositioned at SHIP, not here. If you see
one of them still live in the code, DO NOT re-raise as SB. Do NOT
open new FUs unless round-2's changes introduced NEW defects. In
particular:

- **FU-5 (handback-sync silent truncate at `#`)** — round-2's own
  handback notes are properly quoted with double-quotes and contain
  no `#`; the round-1 truncation persists in cost.sessions[build]'s
  first entry, but that is fixed downstream by ship-cycle scripts
  or by a follow-up spec, not by round-2 verify.
- **FU-9 (LGPL source-reading question)** — round-2 did no new
  source reading of any copyleft implementation.

If round 2 accidentally re-opens FU-N by mistake (e.g., overwriting
one of DEC-024's clauses), THAT is a new SB, not a rehash of the
old FU.

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` (SB-1 and SB-2 closed
   cleanly, no new SBs) / `⚠ PUNCH LIST` (SB-1 or SB-2 not closed,
   or a NEW SB found in round-2's changes) / `❌ REJECTED` (the
   round-2 diff itself broke something). Cite the ship SHA
   (`08ad42e` code + `957612e` bookkeeping tip, or whichever you
   measured).

2. **SB-1 mutation red-proof reproduced.** Show the mutation md5
   before/after and the test-suite delta (206 → 205 passed +
   1 FAILED).

3. **SB-2 doc-fix diff read.** Cite the current rustdoc text at
   `src/warp.rs:52-77` and `src/lib.rs:56-58` and confirm both
   cite DEC-024 correctly.

4. **CI observed green** on the ship SHA. Round-2 handback cites
   run 34164609782 on `08ad42e` (10 jobs, fuzz smoke included).
   Confirm.

5. **`cost.sessions` has TWO build entries** (round 1: 116,480,125
   and round 2: 14,298,209 both cycle: build) plus one verify
   entry (22,631,969, round 1) — round 2 build total 130,778,334
   tokens. Verify this cycle's cost-sessions structure is
   coherent.

6. **The 10 FUs from round 1 are recognisably still there.** Skim
   the SPEC-018.md and DEC-024 — none of FU-1..10's defects were
   secretly closed as an "incidental" side effect of round 2.

7. **PR NOT opened by verify.** Orchestrator handles PR #16 after
   your verdict.

## Cost

Fill the `handback:` block with real numbers from your interface.
`notes:` is ONE PHYSICAL LINE and MUST BE QUOTED double-quotes
(FU-5). Price per-component + 20% uplift.

## References

- `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-047-verify-warprectilinear-radial-geometric-correction.md`
  — round-1 verify's punch list (SB-1, SB-2, FU-1..10).
- `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md`
  — round-1 AND round-2 build handbacks. `## Completion — round 2`
  is at the bottom.
- `decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md`
  — the decision record; unchanged in round 2.
- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md`
  — the spec; unchanged in round 2 (ACs untouched).
- `tests/warp.rs` — SB-1's new test.
- `src/warp.rs`, `src/lib.rs`, `src/develop.rs` — SB-2's rewrites.

## What this round-2 verify does NOT ask for

- **Re-verifying round 1.** Finding 1 and Finding 2 stand as
  round 1 established them.
- **Dispositioning FUs.** Ship cycle does that.
- **Opening the PR.** Orchestrator's step.
- **Reading dnglab source.** Finding 2 stands; behavioural verification
  suffices (as HANDOFF-047 established).
