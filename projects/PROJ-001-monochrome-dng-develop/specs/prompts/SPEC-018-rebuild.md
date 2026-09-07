# SPEC-018 — build round 2 (punch-list) dispatch

Hand this to a fresh CLI session in this repo (`claude` from the repo root, on
the SPEC-018 branch). Verify (HANDOFF-047, `⚠ PUNCH LIST` at `40f5d45`) raised
two ship blockers and 10 follow-ups; **this build round addresses only the two
SBs**. The FUs are dispositioned at ship, not here — do not scope-creep.

⚠ **The verifier's judgement of SPEC-018's Finding 2 was `FU-10` (ship with
`#[ignore]`s + a follow-up spec narrowing SPEC-020's oracle scope), not
`SB-N`.** Do not re-open that decision here.

---

```
Cycle: build round 2 (punch-list). You are the implementer for SPEC-018's
verify punch list. You did not build round 1 — read the round-1 handback and
the verify review cold before touching code.

Read first, in this order:
  1. AGENTS.md — §15 (cycle contract, punch-list send-back), §16 (four codified
     lessons; SB-2 is a `unrun-docs-carry-errors` instance in its own way).
  2. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-047-verify-warprectilinear-radial-geometric-correction.md
     — the verify handback. Read `## Delegation Summary` and every SB/FU
     finding. SB-1 and SB-2 are your entire mandate; FUs are dispositioned
     at ship.
  3. projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-046-build-warprectilinear-radial-geometric-correction.md
     — round-1 build handback. Read its `## Completion` section so you know
     what round 1 shipped.
  4. decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md
     — the decision record. Do NOT re-open the oracle-narrowing question
     (that's FU-10 for a future spec).
  5. projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction.md
     — the spec (`## Acceptance Criteria` numbers ACs; SB-1 fixes a gap in
     the test surface, not a spec addition).

Before reading further, keep build `[~]` in
  projects/PROJ-001-monochrome-dng-develop/specs/SPEC-018-warprectilinear-radial-geometric-correction-timeline.md
(orchestrator already flipped it from verify → build round 2 via advance-cycle.)

Branch: feat/spec-018-warprectilinear-radial-geometric-correction (already
checked out; work continues on top of 27c54bd — the verify handback commit).

Environment — this bites every session:
  export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images

Four things this repo has paid for, in order of how often they bite:
  1. `just lint-ci`, NOT `just lint` — local clippy is 0.1.97, CI floats at
     0.1.98. Both recipes print their own clippy version (PATCH-004).
  2. Every mutation (including your own SB-1 red-proof reproduction) must
     change the file AND compile AND change the output. Five false red-proofs
     across earlier specs.
  3. Tier-B tests pass whether or not the corpus is present.
  4. Do NOT hand-write cost.sessions. Fill your handback in a new commit on
     HANDOFF-046 (see "Handback" below).
     ⚠ `notes:` MUST BE QUOTED and MUST NOT CONTAIN A BARE `#`. FU-5
     established that handback-sync silently truncated round 1's notes at
     `#[ignore]d`. If you need to reference `#[ignore]` in your notes,
     quote the whole scalar with double-quotes and escape any real `#`
     usage — or better, describe it without the `#` sigil.

# ── SB-1: `develop_into`'s warp branch has no live test ──────────────

Verify proved this holds under mutation: reverting `develop.rs` to
crop-then-warp compiled, changed real output, and kept the suite at 205
passed / 0 failed / 2 ignored. `opcode_list_3` is non-`None` at exactly one
call site in the whole test tree (`tests/warp.rs:216`), and that `Sensor`
is only passed to `output_dimensions()`, which by design ignores it.
Every other develop test — including `tests/develop_oracle.rs::
decode_and_develop` — clears `opcode_list_3` on the `Sensor`. So the
develop-side `Some(warp)` branch has NO passing test that reaches it; the
two that would are the `#[ignore]d` AC8/AC9.

The fix, pre-registered so you do not have to design it:
  1. Add a TIER-A test to tests/warp.rs (NOT a corpus-dependent test).
  2. The test hand-builds a `Sensor` with:
       - `ActiveArea != DefaultCropOrigin/Size` (so the pipeline seam is
         actually observable)
       - `opcode_list_3: Some(...)` carrying a real WarpRectilinear (the
         L1021223 or a hand-built small-plane variant)
     and asserts that the developed output has the crop taken from the
     WARPED ActiveArea — i.e. two pixels chosen from the DefaultCropOrigin
     region show the effect of the warp.
  3. Add a SECOND assertion: identity-warp (kr0=1.0, kr1=kr2=kr3=0.0)
     through the develop pipeline produces bit-identical output to the
     warp-free path (no `opcode_list_3` present at all). This catches a
     `Some(warp)` branch that structurally works but semantically no-ops
     when it should not.
  4. The test MUST turn red if src/develop.rs is reverted to
     crop-then-warp — this is your acceptance test for the fix. Prove it
     before you commit: mutate, run, watch it fail; revert; run, watch it
     pass. Show the mutation md5 change in your handback.

# ── SB-2: two false claims in shipped rustdoc ──────────────────────────

Verify names exact locations:
  - `src/warp.rs:59-61` — the comment claims the bilinear kernel "shipped
    because all three decodable Q2M frames scored >= 85 through SPEC-020's
    oracle with it (see the kernel-choice DEC-* for the measured per-frame
    numbers)". No such numbers exist — DEC-024 records the opposite (AC8
    unmeasurable, #[ignore]d). Rewrite this comment to state the actual
    reason bilinear was chosen (the pre-registered rule + AC4/AC10's
    analytic verification with corpus-free teeth), and cite DEC-024 by id.
  - `src/lib.rs:56-57` — the comment says the warp is "wired into
    develop_into between Orientation and the... tone curve". That is the
    PRE-Finding-1 order, in the crate-root doc of the commit that
    corrected it. Rewrite to state the actual shipped order: after
    normalize, over the full ActiveArea, BEFORE DefaultCrop extracts the
    final image. Cite DEC-024 §Finding 1.

Both rewrites are documentation-only; no logic changes. Run `just fmt` and
`just lint-ci` after them.

# ── What this round 2 does NOT touch ─────────────────────────────────

- **FUs 1-10.** They are dispositioned at ship, not here. In particular:
  - FU-1 (docs/measured-q2m-dng.md citation): defer.
  - FU-2 (SPEC-018 ## Implementation Context stale facts): defer.
  - FU-3, FU-4 (DEC-024 spec-clause gaps): defer.
  - FU-5 (handback-sync silent-truncate): defer, but write your OWN
    handback notes so this bug doesn't fire again (see above).
  - FU-6 (L1026016 orientation-6 second reason for AC8 failure): defer.
  - FU-9 (LGPL source-reading question): defer.
- **DEC-024 rewrite.** Do not amend the DEC.
- **SPEC-018 spec surgery.** Do not amend ACs.
- **PR modifications.** PR #16 is open; do not push through GitHub, only
  through git-push to the branch. Do not close/reopen.

# ── Return Criteria ─────────────────────────────────────────────────────

1. **SB-1 fix committed.** A tier-A test in tests/warp.rs that (a) exercises
   the develop_into warp branch, (b) turns red when the pipeline order is
   reverted to crop-then-warp, and (c) has an identity-warp assertion.
   Quote the mutation md5 before/after + the test-suite delta as evidence
   the mutation changed the output. Test name goes in your handback.

2. **SB-2 fix committed.** Both rustdoc claims rewritten to match reality.
   Cite DEC-024 by id in each. Quote the before/after text in the handback.

3. **All gates green.** `just lint-ci`, `just test`, `just fmt`, `just deny`,
   `just msrv`. Test count should be 206 passed / 0 failed / 2 ignored (+1
   from SB-1's tier-A test). ⚠ CI observed green on your ship SHA.

4. **Fuzz smoke run.** `just fuzz-warp` for 60 s. Round-1 build wired it;
   just confirm it still runs.

5. **Handback filled** (a new entry appended to HANDOFF-046's `## Completion`
   naming this as `round 2`) or a new HANDOFF-046 handback block — pick
   whichever is cleaner. tokens_total real, deduped by message.id from
   your OWN transcript (identify by scratchpad UUID, not text-match).
   ⚠ notes: MUST BE QUOTED. Handback-sync truncates at bare `#` (FU-5).

6. **No PR ops.** Orchestrator handles the PR after your handback lands.

7. **State in the handback:** the test name for SB-1, the mutation md5s,
   the exact text of the two SB-2 rewrites, and any incidental fixes you
   made that touch the two SBs' surroundings (e.g., if fixing SB-2's
   src/lib.rs comment naturally exposes an unrelated typo, note it but
   do not chase FUs).
```
