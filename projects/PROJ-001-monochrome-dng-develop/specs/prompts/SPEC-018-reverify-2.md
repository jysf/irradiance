# SPEC-018 — verify round 3 dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root,
on the SPEC-018 branch). Round-3 build closed SB-3 with a smart judgment
call (cited § 6.4.1's silence + AC7 rather than § 6.4.1 as the source of
the clamp rule — avoiding a third `unrun-docs-carry-errors` instance).
This session confirms — tightly.

⚠ **Do NOT re-audit rounds 1 or 2.** The two ship blockers and 10 FUs from
prior rounds stand as they were dispositioned. HANDOFF-049's focus is
narrow: is SB-3 truly closed, and did round 3 introduce any NEW SB?

---

```
Cycle: verify round 3. You are the reviewer for SPEC-018's SB-3 closure. This
is the third verify pass; do not re-audit rounds 1 or 2.

Read first, in this order:
  1. AGENTS.md — §15's verify rules (SUBSET that round-3's diff touches).
     §16 rule 4 (unrun-docs-carry-errors, N=6) — SB-3's fix is exactly
     the anti-pattern this rule warns against.
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-049-verify-round-3-warprectilinear-radial-geometric-correction.md
     — your contract. Round-3 verify checks SB-3 closure only.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-048-verify-round-2-warprectilinear-radial-geometric-correction.md
     — round-2 verify's SB-3 finding. Read the exact quoted false text.
  4. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md
     `## Completion — round 3` — the round-3 build handback. Read the
     judgment call the sub-agent made (silence + AC7 instead of § 6.4.1).
  5. decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md
     — READ the specific lines HANDOFF-049 names (Consequences "Neutral"
     line; § 6.4.1 silence text at ~195-202; AC8 measurement at ~97-98).
  6. src/warp.rs:47-77 — the current rustdoc under review.

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md

Branch: feat/spec-018-warprectilinear-radial-geometric-correction
Code SHA under review: b92b30c (round-3 ship SHA). Bookkeeping tip e976564.
CI runs to confirm: 34173566347 (push) + 34173569505 (PR).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation must change the file AND compile AND change the output.
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-049's handback:.
     ⚠ notes: MUST BE DOUBLE-QUOTED. FU-5 still live.

The TIGHT scope — HANDOFF-049's boundary:

  ✓ Check SB-3 closure — the current src/warp.rs:47-77 rustdoc:
    (a) Names only what DEC-024 actually holds (Consequences "Neutral" line;
        no out-of-extent alternative);
    (b) Cites § 6.4.1's SILENCE (not § 6.4.1) as source of the clamp rule;
    (c) Names -60.169 on L1021223.DNG for AC8 measurement.

  ✓ Judge the sub-agent's judgment call. The dispatch suggested "clamps
    per DNG § 6.4.1" but that would have been a THIRD unrun-docs-carry-errors
    instance because DEC-024:195-196 records § 6.4.1 is SILENT. The
    sub-agent recognised this and rephrased. Is the current phrasing
    honest and does it correctly reflect what DEC-024 holds without
    over-generalising?

  ✓ grep -rn zero-fill src/ — should return 0 hits.

  ✗ Do NOT re-verify rounds 1 or 2. SB-1 (test) and SB-2 (rustdoc) stand.

  ✗ Do NOT touch FU-1..11. All dispositioned at ship.

  ✓ IF round 3's diff introduced ANY new defect (anything outside
    src/warp.rs:47-77, or a new false claim inside those lines), raise
    as a new SB.

Do not fix anything you find. Report; do not repair. Do not open the PR
(orchestrator), do not run handback-sync.

Return: `✅ APPROVED (with SHA)` if SB-3 cleanly closed and no new SB /
`⚠ PUNCH LIST (round 4)` if SB-3 not closed or new SB / `❌ REJECTED` if
round 3's diff broke something. Fill HANDOFF-049's handback: with:
  - status: completed | blocked | rejected
  - REAL tokens_total deduped by message.id from your OWN transcript
  - estimated_usd priced per-component, +20% uplift
  - notes: ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
  - verdict: approved | punch-list | rejected
```
