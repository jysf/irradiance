# SPEC-017 — build round 2 (punch-list) dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root, on
the SPEC-017 branch). Verify (HANDOFF-050) returned `⚠ PUNCH LIST` with ONE
ship blocker (SB-2: `develop_into` wiring untested — the mirror of SPEC-018's
SB-1) and several follow-ups. **Round 2 addresses only SB-2**; FUs are
dispositioned at ship.

⚠ The verifier already sketched the one-line fix and reproduced the red-proof
personally. Round 2 is small: extend
`develop_output_is_bit_identical_across_two_runs` with a centre-pixel
assertion so the mutant that severs `effective_src → src` turns red.

---

```
Cycle: build round 2 (punch-list). You are the implementer for SPEC-017's verify
punch list. You did not build round 1 — read the round-1 handback and the
verify review cold before touching code.

Read first, in this order:
  1. AGENTS.md — §15 (cycle contract, punch-list send-back), §16 (four codified
     lessons; SB-2 is the same shape as SPEC-018's SB-1 —
     `a-fix-inherits-the-precondition-of-the-thing-it-fixes` doesn't apply
     because SPEC-017 wasn't a fix, but the STAGE-003 discipline "assert the
     branch was HIT, not merely that the image came out unchanged" is the
     recurring pattern this SB names).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-050-verify-fixbadpixelsconstant-opcode.md
     — the verify handback. The `notes:` field has the exact SB-2 mutation
     (severing `effective_src` to `src`), the mutant md5 pair, and the
     one-line fix. That's your entire mandate.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-044-build-fixbadpixelsconstant-opcode.md
     — round-1 build's `## Completion`. Confirm the applier and pipeline
     integration you inherit.
  4. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md
     — the spec. AC9 is `develop_output_is_bit_identical_across_two_runs`;
     the fixture there is where the assertion lands.
  5. projects/PROJ-001-monochrome-dng-develop/specs/done/SPEC-018-warprectilinear-radial-geometric-correction.md
     `## Follow-ups` SB-1 row — SPEC-018's parallel case, closed by a
     tier-A test that exercises `develop_into`. SPEC-017's fix is smaller
     (extending an existing test, not adding a new one).
  6. src/develop.rs — `develop_into` and the effective_src / src plumbing.
  7. src/opcode.rs — the applier lives here; unchanged by round 2.
  8. tests/opcode.rs or wherever AC9's `develop_output_is_bit_identical_
     across_two_runs` lives.

Before reading further, keep build `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode-timeline.md
(orchestrator already flipped it from verify → build round 2.)

Branch: feat/spec-017-fixbadpixels-opcode (already checked out; work
continues on top of 033a7cf — the verify handback commit).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy 0.1.97, CI 0.1.98.
  2. Every mutation must change the file AND compile AND change the output.
     This applies to your SB-2 red-proof: reproduce it before shipping.
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill your handback in HANDOFF-044's
     `## Completion` block as round 2 (matches SPEC-018's round-2 pattern).
     ⚠ notes: MUST BE DOUBLE-QUOTED, NO bare `#` (signal
     handback-sync-treats-hash-as-comment-in-unquoted-notes). ⚠ CORRECT
     HANDOFF-044's top-level to_agent to your actual message.model
     BEFORE handback-sync runs — round 1's build did this correctly; keep
     the discipline.

# ── SB-2: `develop_into` wiring untested ──────────────────────────

Verifier proved this holds under mutation: severing `effective_src → src`
compiles (md5 b3b922d0 → 548e240f), leaves the tier-A suite at 231/0/3
green, and the mutant AC7 passes with delta +0.000 on both comparable
frames — because HIT=0 makes before-and-after images bit-identical, so no
test observes the wiring at all.

Pre-registered fix (verifier's one-liner, no design work needed):

  1. Extend `develop_output_is_bit_identical_across_two_runs` (AC9's test)
     with a **centre-pixel assertion**. The fixture in that test already
     makes normalize an identity map, so `dst1[centre]` should equal 1000
     if the fixed plane flows through `develop_into` — and 0 (or the
     replaced value chosen by the fix) if the wiring is severed.
  2. The assertion MUST turn red under the `effective_src = src` mutation
     verifier documented. Reproduce that red-proof yourself: mutate,
     compile, run — watch the assertion fail. Revert; md5-verify; watch it
     pass. Show the mutation md5 change in your handback.
  3. Do NOT modify any other test, any src/ file, or DEC-024.

# ── What this round 2 does NOT touch ─────────────────────────────────

- **FUs 1-9.** All dispositioned at ship, not here. Even the small ones
  (FU-4 false rustdoc, FU-5 provenance row placement, FU-6 memory ledger
  drift, FU-8 stale AC text) wait for ship.
- **DEC-024 amendments.** Untouched.
- **SPEC-017 spec surgery.** Untouched — the AC text is stale (FU-8) but
  that's a ship-cycle correction.
- **PR modifications.** PR #17 open; push to branch only.
- **The applier or the parser.** Both are correct per verify.

# ── Return Criteria ─────────────────────────────────────────────────────

1. **SB-2 fix committed.** The one-line assertion added to AC9's test.
   Quote the before/after diff (test source lines) in your handback. Quote
   the mutation md5 change (b3b922d0 → whatever after severing → back).
   Test-suite delta under mutation: 231/0/3 → 230/1/3.

2. **All gates green.** `just lint-ci` (clippy 0.1.98), `just test`,
   `just fmt` (or `just lint` if fmt is inside it), `just deny`,
   `just msrv`, `just fuzz-opcode` (60 s). Test count 232/0/3 (+1 from
   the new assertion? or same 231/0/3 if you extend an existing test —
   either is fine, state which). CI observed green on your ship SHA.

3. **Handback filled** — append round-2 to HANDOFF-044's `## Completion`.
   tokens_total real, deduped by message.id from your OWN transcript
   (identify by scratchpad UUID). ⚠ notes: DOUBLE-QUOTED, NO bare `#`.
   ⚠ Update HANDOFF-044's top-level to_agent to your actual model.

4. **No PR ops.** Orchestrator handles PR #17.

5. **State in the handback:** the exact assertion added, the file/line
   where it landed, the mutation md5s before and after, and the test-suite
   delta under the mutation.
```
