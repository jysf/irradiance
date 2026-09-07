---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-045
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session
                                    # (correction from tier_map.design's
                                    # claude-opus-5 prediction, per DEC-004
                                    # rule 3 — silent cost-surprise trap).
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify, not a
                                    # measurement. CORRECT THIS in the handback
                                    # to what your own system prompt reports
                                    # as `message.model`.
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-020

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` MUST be a real number from your interface (/cost in Claude
# Code; usage in the API). `notes:` MUST be ONE PHYSICAL LINE — handback-sync
# truncates multi-line YAML scalars and leaves the spec unparseable while
# every gate reports green (`handback-sync-truncates-multi-line-scalars`).
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 10366144           # REAL combined count — what cost-audit reads
  estimated_usd: 8.25              # tokens_total × your rate, or your harness's number
  duration_minutes: 16
  branch: feat/spec-020-develop-oracle-vs-dnglab-srgb
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: 2026-09-06         # YYYY-MM-DD
  notes: "Deduped by message.id from own transcript identified by this session's scratchpad UUID d83664a6-d3a9-4ac2-b681-bb222d00d0a7 (not text-matched, per this project's identify-own-transcript-for-cost-handback memory); 153 usage objects, 65 distinct ids, all message.model claude-opus-5 (tier_map.verify's prediction was RIGHT this time), raw combined 8,638,453 (input 130 / cache-write(1h) 161,424 / cache-read 8,435,217 / output 41,682 — 97.6% cache-read), priced per-component at published claude-opus-5 rates ($5/$25/$10/$0.50 per Mtok input/output/1h-write/read) = $6.87, +20% uplift for the turns writing this handback = $8.25 and 10,366,144 tokens; verify edited no repo file except this handback block — the one red-proof mutation (identity warp in tests/support/perturb.rs) was reverted and the tree confirmed clean."
  synced_at: 2026-09-06
  verdict: approved                # approved | punch-list | rejected — mirror the
                                   #   verdict banner from the end of your review.
                                   #   Copies into spec.task.verify_verdict at
                                   #   ship, so the "how often does verify
                                   #   reject anything" question stops being a
                                   #   hunch.
---

# HANDOFF-045: Verify SPEC-020 — the develop oracle vs dnglab srgb

## Delegation Summary

Verify the SPEC-020 build shipped by HANDOFF-043 (build handback:
`status: completed`, `tokens_total: 36,917,935`, three FUs raised).
Branch is at `61e5eb4` (pushed, tracked, three CI runs all green).

Your job: reconcile the build's claims against actual git+disk state
(DEC-004 rule 1), run the 11 acceptance criteria's named tests
yourself, run the 12 verify checks (`AGENTS.md §15` — 8 generic + 4
repo-specific bars), and return `✅ APPROVED` / `⚠ PUNCH LIST` /
`❌ REJECTED` with per-finding `SB-N` / `FU-N` labels.

## ⚠ Read this before the build's claims

- **The build was HANDOFF-043's sub-agent, and its handback carries
  the reasoning for three FUs already proposed `closed:`.** Read the
  handback (`handoffs/HANDOFF-043-*.md` → `## Cost self-report`,
  `## Drift and new artifacts`, `## Reflection`) before you form your
  own opinion of any finding. Your job is not to re-raise the build's
  own honest self-report; it is to catch what the build could not see.

- **DEC-004 rule 1 is not optional.** Trust git/disk over the
  handback. The build claimed `src/` untouched — verify with
  `git diff --name-only 303a8f6..HEAD -- src/` (should print
  nothing). The build claimed 20 new tests in
  `tests/perceptual_oracle.rs` — verify with `cargo test --all-features
  --test perceptual_oracle` (should report 20 passing). The build
  claimed a 46.7 Mpixel end-to-end SSIMULACRA2 score would take
  ~9.3 s in release — you don't need to reproduce that timing; but
  verify the standalone `dnglab_srgb_reader_parses_a_real_corpus_file`
  substitute test exists and runs.

- **The four repo-specific verify bars for this spec (`§15` bars 9–12):**
  9. **Did the oracle go red?** Run
     `red_proof_missing_warp_scores_below_the_pre_registered_gate`
     yourself and *watch it produce a score below 85*. `oracle-must-
     be-shown-red` is why. The handback reports the missing-warp
     mutation lands at −82.338; confirm.
  10. **Fuzz target exists and ran?** N/A — this spec adds no
      library-side input surface (SPEC-020 spec `## Outputs`:
      "no new fuzz target — the PNM reader consumes bytes it produced
      itself from dnglab, not from an attacker"). Confirm no fuzz
      target was added anyway.
  11. **Provenance ledger row for the new algorithm?** Check
      `docs/provenance-ledger.md` — the SSIMULACRA2 metric is a
      transcription of the crate's published algorithm; class 2 (a
      permissive-crate reading). The build should have added the
      row; if it didn't, that is a **finding**.
  12. **Is the new dependency permissive, and not a RAW decoder?**
      Check `DEC-023-ssimulacra2-devdep-sanction.md`. The handback
      corrected the design guess from BSD-3 to BSD-2-Clause — run
      `cargo deny check licenses` yourself. Also verify `[dev-
      dependencies]` in `Cargo.toml` (not `[dependencies]`); a
      runtime dep would violate `library-not-application` even at
      a permissive licence.

## The four codified lessons apply to this verify

Read `AGENTS.md` §16. All four have caught real defects in earlier
verifies:

1. **`measurement-over-generalised`** — the four measured scores
   (`identity 100.000`, `1-px shift 61.823`, `missing warp −82.338`,
   `gamma 1.05 90.038`) are claims from **one probe on one fixture**.
   Do not read them as "SSIMULACRA2 always scores these boundaries";
   confirm each by running the specific test.
2. **`attribute-text-inside-doc-comments`** — if any of your verify
   scripts grep source text (e.g. to confirm `src/` untouched),
   assert the match count and exclude doc comments. `git diff
   --name-only` is the correct scanner here; `grep` for "src/" in
   a diff is not.
3. **`a-gate-that-fails-mutely-is-a-gate-that-never-ran`** — every
   gate you run reports its own pass/fail; do not conclude "green"
   from a silent exit. `just lint-ci` prints the clippy version it
   used (PATCH-004); confirm 0.1.98, not 0.1.97.
4. **`unrun-docs-carry-errors`** — the build's `DEC-023` cites 38
   crates in `ssimulacra2`'s resolved graph. If you cite that
   number in your review, re-run `cargo tree` first.

## The `a-fix-inherits-the-precondition-of-the-thing-it-fixes` lesson

New this week (STAGE-002 close). When you evaluate the build's
three proposed `closed:` dispositions, don't just check that the
CLOSE reasoning is sound; check that the CLOSE actually addresses
the finding's underlying assumption:

- **`FU-1` — closed on the grounds that the substitute test
  (`dnglab_srgb_reader_parses_a_real_corpus_file`) covers the
  load-bearing claim.** The assumption is "parsing the PNM shape is
  what matters; scoring it end-to-end doesn't yet". Verify by
  running the substitute test yourself and confirming it exercises
  a real dnglab output (not a hand-built fixture).

- **`FU-2` — closed on the grounds that `DEC-023` carries the
  corrected 38-crate count and the exact `cargo tree` method.** The
  assumption is "future cites will re-run `cargo tree`". Verify by
  reading `DEC-023` and confirming it names the method.

- **`FU-3` — closed on the grounds that `just deny` / CI's
  `licenses` job will fail loudly on any future licence drift.**
  The assumption is "cargo-deny reads the actual licence, not
  DEC-023's cited one". Verify by running `cargo deny check
  licenses` yourself; the output should name `ssimulacra2` and
  its actual licence.

## The eight generic verify checks (§15)

1. All 11 acceptance criteria met and tested? (AC1..AC11 — the
   spec's `## Acceptance Criteria` numbers them.)
2. Failing tests from the spec now pass? (`## Failing Tests` in
   SPEC-020, sub-agent added `tests/perceptual_oracle.rs` with
   20 tests.)
3. No drift from referenced decisions? (DEC-002, DEC-004, DEC-005,
   DEC-011, DEC-013, DEC-016, DEC-018 plus the new DEC-023.)
4. No constraint violations? (blocking constraints: `no-copyleft-
   dependencies`, `no-panics-on-untrusted-input`, `oracle-must-be-
   shown-red`, `provenance-recorded-per-algorithm`, `library-not-
   application`, `test-before-implementation`.)
5. Non-trivial implementer choices have accompanying DEC-\*? (DEC-023
   covers the ssimulacra2 dev-dep; the build cites deviations 1–3
   in its handback — verify each is either self-explanatory or has
   a DEC.)
6. Implementer reflection answered (not mailed in)? (build's `##
   Reflection` answered all three questions with substance;
   confirm.)
7. `cost.sessions` has entries for prior cycles? (After
   `handback-sync SPEC-020` runs, the build session should be
   there. Confirm.)
8. For any acceptance criterion claiming **runtime behavior**, was
   the *behavioral* surface actually exercised — not just the shape
   validated (§12 behavioral pre-flight)? Applies here to: the PNM
   parser (does it consume dnglab's real defect shape? — the
   substitute test), SSIMULACRA2 scoring (does it produce the
   expected magnitudes on real inputs? — the four measured scores
   in the fixture test), and the license claim (does cargo-deny
   actually accept `ssimulacra2`? — `cargo deny check licenses`).

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` / `⚠ PUNCH LIST` /
   `❌ REJECTED`, with the branch SHA it applies to.

2. **Every finding labelled `SB-N` (ship-blocker) or `FU-N`
   (follow-up), per spec** (numbering restarts at 1 for SPEC-020;
   §15's per-spec convention). Each row: id, one-line finding,
   one-sentence severity reasoning, one-sentence proposed fix or
   punch-list note.

3. **The four repo-specific bars 9–12 explicitly answered**, each
   with the command you ran and its observation. Bar 10 is `n/a` on
   this spec; state so and why (no library-side parser).

4. **The three build-proposed `closed:` dispositions** either
   confirmed (with your own one-sentence read) or objected to
   (with a `FU-N` re-raising the concern for ship).

5. **The 11 named tests run and each observed passing** — sum
   across all targets, quote `cargo test --all-features --test
   perceptual_oracle` output. ⚠ A zero-match `cargo test <name>`
   exits 0 silently (`named-tests-can-pass-vacuously`); confirm
   each test *ran*, not just that its command exited 0.

6. **`just lint-ci` run locally, clippy version asserted** (should
   be 0.1.98). PATCH-004 made this recipe print its own clippy
   version; quote it.

7. **CI observed green on `61e5eb4`**, run id and job count
   (`gh run list --branch feat/spec-020-develop-oracle-vs-dnglab-
   srgb --limit 1`).

8. **`cargo deny check licenses`** run against the root manifest
   (`--manifest-path Cargo.toml`) — one line quoted for
   `ssimulacra2`. `cargo deny --manifest-path fuzz/Cargo.toml check
   licenses` also (the second graph — SPEC-003's shipping-week
   lesson).

9. **`cost.sessions[build]` present** after handback-sync (already
   run by orchestrator this session). Report `tokens_total` you
   observed in the spec.

10. **PR NOT opened by verify** — the orchestrator opens the PR
    after your verdict lands. Do not push to the branch (verify
    edits nothing) unless you have a punch-list fix so small it is
    faster to apply than to file; state either way.

## Cost

Fill the `handback:` block above with real numbers from your own
interface. `just handback-sync SPEC-020` (from an orchestrator's
session) transcribes them. `notes:` is ONE PHYSICAL LINE.
`verdict:` mirrors the top-of-review banner.

## References

- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-020-develop-oracle-vs-dnglab-srgb.md`
  — the spec (`## Acceptance Criteria`, `## Failing Tests`, `##
  Implementation Context`).
- `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-043-build-develop-oracle-vs-dnglab-srgb.md`
  — the build handoff and its filled handback + three proposed
  closes.
- `decisions/DEC-023-ssimulacra2-devdep-sanction.md` — the new
  dev-dep sanction; contains the corrected licence and cargo-tree
  count.
- `tests/perceptual_oracle.rs`, `tests/support/pnm.rs`,
  `tests/support/ssimulacra2.rs`, `tests/support/perturb.rs` — the
  four new files.
- `AGENTS.md` §15 (verify's 8 generic + 4 repo-specific bars), §16
  (four codified lessons + a-fix-inherits), §12 (four testing bars
  including oracle-must-be-shown-red), §11 (unread-field discipline
  — DEC-023's `[dev-dependencies]` sits at the edge of this).
- `guidance/constraints.yaml` — the five blocking constraints;
  `no-copyleft-dependencies` is why bar 12 matters and why the
  BSD-3 → BSD-2-Clause correction was load-bearing.

## What this verify does NOT ask for

- **Re-designing anything.** Findings that would require a design
  change are `FU-N`, not `SB-N`, unless they let bad data or a
  panic reach a consumer. §15's ship-blocker definition applies.
- **Iterating with the build sub-agent.** Verify is a cold-eyes
  review, not a conversation. If a finding is unclear, name it
  clearly enough that the ship cycle (or the next build) can act
  on it without more context.
- **Opening the PR.** Orchestrator's step.
- **Running full corpus tier-B tests you don't have the corpus
  for.** The three decodable Q2M frames are at
  `$IRRADIANCE_CORPUS_DIR/LEICA-Q2-MONO/` on this machine; if you
  don't have `IRRADIANCE_CORPUS_DIR` set, mark corpus-dependent
  checks as `[?]` in your review and say the corpus was unavailable.
