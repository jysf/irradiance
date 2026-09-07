# SPEC-018 — build round 3 (punch-list) dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root, on
the SPEC-018 branch). Reverify (HANDOFF-048) confirmed SB-1 and SB-2 closed but
raised **SB-3 NEW** — round-2's own SB-2 rewrite introduced a false DEC-024
citation in `src/warp.rs:53-55`. FU-11 (cost.sessions agent field is stale) is
a follow-up, dispositioned at ship, not here.

⚠ **This round is even tighter than round 2.** ONE ship blocker, ONE fix,
documentation only. Do NOT touch anything else.

---

```
Cycle: build round 3 (punch-list). You are the implementer for SPEC-018's
round-2 verify punch list. You did not build rounds 1 or 2. Read the reverify
review and the round-2 rustdoc that introduced SB-3, cold.

Read first, in this order:
  1. AGENTS.md — §15 (cycle contract), §16 rule 4 (unrun-docs-carry-errors,
     which is exactly why SB-3 exists). Round 2's fix inherited SB-2's
     precondition (`a-fix-inherits-the-precondition-of-the-thing-it-fixes`);
     do NOT let round 3's fix do the same to SB-3.
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-048-verify-round-2-warprectilinear-radial-geometric-correction.md
     — the reverify handback. Read the SB-3 finding in full: the exact false
     citation, what DEC-024 actually holds, and why the round-2 build introduced
     it (resolved a dangling DEC-* placeholder to a concrete id without reading
     DEC-024 to check the neighbouring claim).
  3. decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md
     — READ IN FULL. This is the source of truth SB-3's fix must match. Skim
     the Alternatives (Options A/B/C are kernel and threshold — NOT
     caller-supplied-buffer or out-of-extent) and the Consequences.
  4. src/warp.rs:53-55 — the false citation. Also read the surrounding block
     (~lines 47-77) so your rewrite fits.

Before reading further, keep build `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md
(orchestrator already flipped it from verify → build round 3.)

Branch: feat/spec-018-warprectilinear-radial-geometric-correction (already
checked out; work continues on top of da9ed08 — the reverify handback commit).

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation (this round is doc-only, so this is just fmt/lint after).
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill your handback in a new commit on
     HANDOFF-046 (appending to round-2's ## Completion, matching round 2's
     pattern — see PATCH-003's history if in doubt).
     ⚠ notes: MUST BE DOUBLE-QUOTED. FU-5 established handback-sync
     silently truncates at bare `#`. Do NOT contain `#` in your notes.

# ── SB-3: false DEC-024 citation in src/warp.rs:53-55 ────────────────

Reverify quoted the exact false text:

  "The alternatives (zero-fill, error) are recorded in the kernel-choice
   decision, DEC-024 (AC11)"

Three problems:

  A. DEC-024 records NEITHER "zero-fill" NOR "error" as alternatives. Repo
     grep for "zero-fill" returns three hits, two unrelated (tests/) and the
     third IS THIS CLAIM ITSELF. There is no alternative-considered record
     in DEC-024 for the out-of-extent rule.

  B. AC11 governs the KERNEL CHOICE, not the out-of-extent rule. Citing
     AC11 for out-of-extent conflates two decisions.

  C. The same sentence's "records no per-frame oracle scores" is imprecise —
     DEC-024:98 records -60.169 (L1021223 measured score) explicitly.

Pre-registered fix rules — apply all three, no scope creep:

  1. **Rewrite lines 53-55 to name only what DEC-024 actually holds.**
     DEC-024's out-of-extent text is one Consequences-Neutral line naming
     clamp-to-edge with no alternatives chosen against. State that in the
     rustdoc: the out-of-extent handling clamps to edge per DNG § 6.4.1, no
     alternatives were weighed (record that fact honestly rather than inventing
     one). Cite DNG § 6.4.1, not DEC-024, since DEC-024 doesn't record the
     decision — this is a place where the SPEC citation is the honest one.

  2. **Fix "records no per-frame oracle scores" imprecision.** DEC-024 records
     the one full-resolution score measured (-60.169 on L1021223.DNG). Rewrite
     to reflect that: "DEC-024 records the AC8 measurement (-60.169 on
     L1021223.DNG) that established the oracle cannot validate this stage"
     or similar.

  3. **Do NOT touch the surrounding rustdoc lines that reverify already
     approved.** Anything outside lines 53-55 is out of scope. If a nearby line
     is wrong, that is a NEW ship blocker in a future round, not your problem.

# ── FU-11 is NOT in scope ────────────────────────────────────────────

The cost.sessions agent-field mismatch (round 2 sonnet-5 vs actual opus-5)
is a follow-up dispositioned at ship. Do NOT edit cost.sessions. Do NOT edit
scripts/handback-sync.sh. Do NOT fix HANDOFF-046's to_agent field.

# ── What this round 3 does NOT touch ─────────────────────────────────

- **FUs 1-11.** All dispositioned at ship.
- **DEC-024 amendments.** Untouched.
- **SPEC-018 spec surgery.** Untouched.
- **PR modifications.** PR #16 open; push to branch only.
- **Any src/ file other than src/warp.rs.** src/lib.rs's SB-2 rewrite
  passed reverify — do NOT touch it.
- **Any test.** SB-1's test passed reverify — leave it.

# ── Return Criteria ─────────────────────────────────────────────────────

1. **SB-3 fix committed.** The three problems above corrected. Quote the
   before/after text in your handback. Cite DEC-024's exact clause you
   verified against (paragraph, section header, line number). §16 rule 4
   (unrun-docs-carry-errors) — read the DEC while writing the rustdoc, not
   from memory.

2. **All gates green.** `just lint-ci`, `just test`, `just fmt`, `just deny`,
   `just msrv`, `just fuzz-warp` (60 s). Test count stays at 206/0/2 — no
   test changes.

3. **CI observed green** on your ship SHA.

4. **Handback filled** (append to HANDOFF-046's `## Completion` as round 3).
   tokens_total real, deduped by message.id, identify by scratchpad UUID.
   ⚠ notes: DOUBLE-QUOTED, NO bare `#`.

5. **No PR ops.** Orchestrator handles PR #16 after your handback lands.

6. **State in the handback:** the exact rewritten text of src/warp.rs:53-55,
   the DEC-024 clause you cited, and confirmation that you touched no other
   file except HANDOFF-046 and (possibly) SPEC-018-timeline.md.
```
