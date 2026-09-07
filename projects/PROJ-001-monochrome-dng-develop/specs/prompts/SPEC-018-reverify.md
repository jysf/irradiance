# SPEC-018 — verify round 2 (reverify) dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root,
on the SPEC-018 branch). Round-2 build (HANDOFF-046 round-2 handback,
`957612e` tip / `08ad42e` ship SHA) claims SB-1 and SB-2 both closed with
206/0/2 tests and CI green. This session confirms — tightly.

⚠ **Do NOT re-audit round 1.** Findings 1 and 2 stand. The 10 FUs from
round 1 stay OPEN and get dispositioned at ship. HANDOFF-048's focus is
narrow: are SB-1 and SB-2 truly closed, and did round 2 introduce any NEW
SBs?

---

```
Cycle: verify round 2 (reverify). You are the reviewer for SPEC-018's punch-list
closure. Round 1's verify (HANDOFF-047) established the substantive findings;
your job is verifying that round-2's fix landed cleanly and no new defects
were introduced.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (checks 1-12); apply the SUBSET that
     round-2's diff actually touches. §16's four codified lessons.
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-048-verify-round-2-warprectilinear-radial-geometric-correction.md
     — your contract. It carries the two SBs to verify closed and the
     boundary that keeps you out of round-1 territory.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-047-verify-warprectilinear-radial-geometric-correction.md
     — the round-1 verify punch list. Read the SB-1 and SB-2 findings so
     you know exactly what round 2 was supposed to close.
  4. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md
     — the round-2 build handback (`## Completion — round 2`). Read the
     claimed fix and the red-proof md5s.
  5. decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md
     — round-2 rustdoc cites this repeatedly; confirm citations are honest.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md

Branch: feat/spec-018-warprectilinear-radial-geometric-correction
Code SHA under review: 08ad42e (round-2 build's ship SHA). Branch tip 957612e
is bookkeeping-only. Approve whichever SHA you measured, and observe CI green
on that SHA (round-2 handback cites run 34164609782 on 08ad42e — verify with
`gh run list --branch feat/spec-018-warprectilinear-radial-geometric-correction`).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation (including your SB-1 red-proof reproduction) must change
     the file AND compile AND change the output.
  3. Tier-B tests pass whether or not the corpus is present. SB-1's fix is
     TIER A — it does not need corpus.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-048's handback:.
     ⚠ notes: MUST BE DOUBLE-QUOTED (FU-5 established handback-sync silently
     truncates at bare `#`).

The TIGHT scope — HANDOFF-048's boundary:

  ✓ Check SB-1 closure — the new test
    `develop_into_crops_from_the_warped_active_area_not_the_warped_crop`
    exists (cargo test with --exact --nocapture, exactly one match, passes),
    the fixture separates 2837/2880 pixels between orders, the identity-warp
    assertion holds bit-identically, and the red-proof (revert to
    crop-then-warp) turns THIS test red while leaving round 1's 205 green.
    Show mutation md5s.

  ✓ Check SB-2 closure — src/warp.rs:52-77 and src/lib.rs:56-58 now cite
    DEC-024 and match reality (no fabricated per-frame scores; pipeline
    order as actually shipped, over full ActiveArea, before DefaultCrop).
    The incidental "third-caller-supplied-buffer" correction: confirm
    DEC-024 does not hold that alternative and the current rustdoc matches.

  ✗ Do NOT re-verify Finding 1 (pipeline order) against DNG 1.7 — round 1
    already did.

  ✗ Do NOT re-verify Finding 2 (dnglab-doesn't-implement-opcodes) against
    dnglab source or behavioural NCC — round 1 already did.

  ✗ Do NOT touch FU-1..10. If you see one live, it stays live — ship
    dispositions them.

  ✓ IF round-2's diff introduced a NEW defect (any src/ change beyond the
    two SBs' rustdoc, or a spec/DEC-024 amendment beyond the incidental
    correction), raise that as a NEW SB and reject / punch-list. This is
    the only reason to grow the punch list.

Do not fix anything you find. Report; do not repair. Do not open the PR
(orchestrator opens it), do not run handback-sync.

Return: `✅ APPROVED (with SHA)` if SB-1 and SB-2 are cleanly closed and no
new SBs / `⚠ PUNCH LIST` if SB-1 or SB-2 not closed or a new SB appeared /
`❌ REJECTED` if round 2's diff broke something. Fill HANDOFF-048's
handback: with:
  - status: completed | blocked | rejected
  - REAL tokens_total deduped by message.id from your OWN transcript
    (identify by scratchpad UUID, not text-match — identify-own-transcript-
    for-cost-handback in this project's memory)
  - estimated_usd priced per-component at your model's rates, +20% uplift
  - notes: ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
  - verdict: approved | punch-list | rejected
```
