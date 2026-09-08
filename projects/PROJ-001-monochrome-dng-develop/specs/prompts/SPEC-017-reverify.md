# SPEC-017 — verify round 2 (reverify) dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo
root, on the SPEC-017 branch). Round-2 build closed SB-2 with a single
6-line assertion at `tests/develop.rs:594-599`. Ship SHA `6e4376b`, tip
`759f200` bookkeeping-only, CI green on both.

⚠ **Do NOT re-audit round 1.** SB-1's downgrade to FU-3 stands, and the
8 FUs stay OPEN for ship disposition. HANDOFF-051's focus is narrow: is
SB-2 truly closed with real teeth, and did round 2 introduce anything?

---

```
Cycle: verify round 2. You are the reviewer for SPEC-017's SB-2 closure. This
is the second verify pass; do not re-audit round 1.

Read first, in this order:
  1. AGENTS.md — §15 verify rules (SUBSET that round-2's diff touches). §16
     rule 2 (three-clause mutation) is the key discipline for reproducing
     SB-2's red-proof.
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-051-verify-round-2-fixbadpixelsconstant-opcode.md
     — your contract. Round-2 verify checks SB-2 closure only.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-050-verify-fixbadpixelsconstant-opcode.md
     — round-1 verify's SB-2 finding. Read the notes field for the exact
     mutation description (severing effective_src → src) and md5 pair.
  4. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-044-build-fixbadpixelsconstant-opcode.md
     `## Completion — round 2` — the round-2 build handback. Read the
     assertion added, the mutant md5, and the two notes at the bottom
     (verifier's md5 not reproducible, applier location note).

Before reading further, mark verify `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode-timeline.md

Branch: feat/spec-017-fixbadpixels-opcode
Code SHA under review: 6e4376b (SB-2 fix commit). Tip 759f200 bookkeeping-only.
CI: 11/11 green on both SHAs (push and PR runs).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation must change the file AND compile AND change the output.
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill HANDOFF-051's handback: only.
     ⚠ notes: MUST BE DOUBLE-QUOTED, NO bare `#` (signal
     handback-sync-treats-hash-as-comment-in-unquoted-notes).
     ⚠ CORRECT this handoff's to_agent to your actual message.model
     BEFORE handback-sync runs (signal handback-sync-inherits-stale-
     to-agent-across-punch-list-rounds). Build round 2 correctly set
     to_agent to claude-opus-5; keep the discipline.

The TIGHT scope — HANDOFF-051's boundary:

  ✓ Check SB-2 closure — the assertion at tests/develop.rs:594-599
    (a) exists and is a `dst1[2 * 5 + 2] == 1000` centre-pixel check;
    (b) turns RED under the effective_src → src mutation in
        src/develop.rs. Reproduce the mutation yourself — it need not
        match build's exact md5 (build noted the round-1 verify md5
        548e240f is unreproducible from what was recorded; the SEMANTIC
        mutation is what matters, not the exact edit text).

  ✓ grep for evasion routes — does any other test compare dst1[centre]
    in a way that would mask the failure if the wiring severed?

  ✓ src/ untouched by round 2 — git diff cd82ca8..6e4376b -- src/
    should print nothing.

  ✗ Do NOT re-verify round 1. SB-1's downgrade to FU-3 stands.

  ✗ Do NOT touch FU-1..9. All dispositioned at ship.

  ✓ IF round 2's diff introduced a NEW defect (any src/ change, or a
    test change beyond the one assertion), raise as a NEW SB.

Do not fix anything you find. Report; do not repair. Do not open PR #17
(orchestrator), do not run handback-sync.

Return: `✅ APPROVED (with SHA)` if SB-2 closed cleanly and no new SB /
`⚠ PUNCH LIST` if SB-2 not closed or new SB / `❌ REJECTED` if round 2
broke something. Fill HANDOFF-051's handback: with:
  - status: completed | blocked | rejected
  - REAL tokens_total deduped by message.id from your OWN transcript
  - estimated_usd priced per-component, +20% uplift
  - notes: ONE PHYSICAL LINE, DOUBLE-QUOTED, NO bare `#`
  - verdict: approved | punch-list | rejected
  - CORRECT top-level to_agent to your actual message.model
```
