---
# Maps to ContextCore handoff.* semantic conventions.

handoff:
  id: HANDOFF-050
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session.
  to_agent: claude-opus-5           # ✅ VERIFIED, not inherited: this verify session's
                                    # own transcript (identified by the scratchpad UUID
                                    # 2d7bb44d-8fa7-4343-a8e0-bd16ec0414e0, not by text
                                    # match) reports message.model = claude-opus-5 on all
                                    # 306 assistant records. The prediction was correct;
                                    # left as-is deliberately (FU-11 discipline held for a
                                    # SECOND spec running). Prior
                                    # verifies for SPEC-018 all confirmed
                                    # opus-5 as the actual model.
                                    # ⚠ CORRECT this to your actual message.model
                                    # BEFORE handback-sync runs (signal
                                    # `handback-sync-inherits-stale-to-agent-
                                    # across-punch-list-rounds`).
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-09-07
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-017

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# `notes:` MUST BE ONE PHYSICAL LINE and MUST BE DOUBLE-QUOTED with NO bare `#`
# (signal `handback-sync-treats-hash-as-comment-in-unquoted-notes` filed at
# SPEC-018 ship: unquoted values with a bare `#` truncate silently at hash).
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 27235932           # MEASURED, deduped by message.id from this session's own
                                    # transcript, identified by the scratchpad UUID
                                    # 2d7bb44d-8fa7-4343-a8e0-bd16ec0414e0 (NOT by text match --
                                    # `identify-own-transcript-for-cost-handback`). 145 deduped
                                    # assistant messages: 290 input / 93,298 output /
                                    # 250,578 cache-write / 26,891,766 cache-read (98.7% cache
                                    # read). Snapshot taken at handback; the report tail after
                                    # this point is not counted.
  estimated_usd: 65.83              # PER-COMPONENT at published Opus list rates, no cache discount
                                    # on the write tier: $15/$75/$30/$1.50 per MTok for
                                    # input/output/1h-cache-write/cache-read = $54.86, x1.20
                                    # uplift = $65.83. Deliberately NOT AGENTS.md §4's flat
                                    # fallback, which on a 98.7%-cache-read session would report
                                    # ~$409 -- signal `flat-rate-overstates-cached-sessions` (N=5)
                                    # measures that class of error at 2.6x-14.7x.
  duration_minutes: 30
  branch: feat/spec-017-fixbadpixels-opcode
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: 2026-09-08         # YYYY-MM-DD
  notes: "PUNCH LIST on cd82ca8 (tip 5c1ba36 bookkeeping-only). CI run 34191738033 observed green 11/11 via gh, including fuzz smoke - opcode whose own CI log says Done 11549674 runs in 61 second(s), 0 crashes (the 13.7M figure is the BUILD LOCAL run, not CI). SB-1 DOWNGRADED to FU-3: independently verified with a C 14-bit unpacker written from the DNG packing rule (7 bytes -> 4 samples, MSB-first), which reproduces irr unpack first8 and minima exactly on all three frames - ZEROS inside ActiveArea = 0 and whole-plane minima 2/30/2, so the applier correctly returns 0 and there is NO code bug; AC5 rerun with --ignored reproduces constant=0 HIT-count=0 left-at-constant=0. NEW SB-2, SHIP-BLOCKING: nothing asserts develop_into USES the fixed plane - severing effective_src to src (md5 b3b922d0 -> 548e240f) compiles and leaves the whole tier-A suite green at 231 passed / 0 failed / 3 ignored, and tier-B is blind too, MEASURED not assumed: the mutant AC7 PASSES 1/0/0 on BOTH comparable frames - before=-60.169 after=-60.169 delta=+0.000 on L1021223 and before=-40.281 after=-40.281 delta=+0.000 on L1026192, both byte-identical to the honest tree's own numbers, L1026016 skipped for the orientation reason - because HIT=0 makes before and after the same image. Identical shape to SPEC-018/SB-1 and a direct hit on STAGE-003 the test must assert the branch was HIT, not merely that the image came out unchanged. One-line fix: assert dst1 centre == 1000 in develop_output_is_bit_identical_across_two_runs, whose fixture already makes normalize an identity map. Bar 9 discharged PERSONALLY, both directions: md5 b3b922d0 honest -> 74af14a4 mutant, compiles, AC4 RED at left 0 right 1000 -> reverted b3b922d0, AC4 green; all mutation in a scratchpad copy, working tree never touched and still clean. Bar 10 pass. Bar 11 passes on substance - class 1 verified by reading the DNG 1.6.0.0 PDF myself with pdftotext: Chapter 7 Opcode List Processing, FixBadPixelsConstant Opcode ID 4, DNG Version 1.3.0.0, params Constant/BayerPhase LONG, p.95, and Chapter 6 is Mapping Camera Color Space to CIE XYZ Space, so the build chapter correction is RIGHT - but the row is filed under src/opcode.rs while the median kernel lives in src/develop.rs, whose own row never mentions it (FU-5). Bar 12 pass: dependencies empty, cargo tree -e normal is irradiance v0.1.0 alone, Cargo.toml 0 lines changed. lint-ci green on PINNED clippy 0.1.98 (local unpinned 0.1.97), fmt clean, deny + deny-fuzz both licences ok, full suite 231/0/3 with corpus present. Build FU-1 confirmed accurate against the shipped parser. Build FU-2 is a RE-INSTANCE of SPEC-018/FU-6, reproduced: dnglab --srgb prints 8368 5584 for BOTH an Orientation-6 and an Orientation-1 frame, i.e. it ignores Orientation entirely; already absorbed by SPEC-021 Context, no new spec needed. FU-4 false rustdoc: every hand-built test Sensor carries opcode_list_1 None is wrong - AC9 own test sets Some and takes the copy path. FU-6 peak RSS measured 541261824 bytes post-SPEC-017 vs the ledger 465010688 the module doc points readers at. FU-7 in-place neighbour semantics undocumented and spec-silent, measured in-place by probe. FU-8 AC3/AC5/AC10 spec text now stale vs shipped reality. FU-9 tokens_total 570000 is a remaining-budget delta, not comparable with this repo transcript-sum figures (SPEC-018 build rounds were 116M/14.3M/5.6M) - evidence for token-counts-not-comparable. PR not opened, nothing repaired, handback-sync not run."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
  verdict: punch-list              # approved | punch-list | rejected
---

# HANDOFF-050: Verify SPEC-017 — FixBadPixelsConstant opcode

## Delegation Summary

Verify the SPEC-017 build shipped by HANDOFF-044 (build handback:
`status: completed`, `tokens_total: 570,000`, 90 min on
claude-sonnet-5, two substantive findings raised). Branch tip is at
`5c1ba36` (bookkeeping-only) atop code SHA `cd82ca8`. CI run
34191738033 green on 11 jobs including the new `fuzz smoke — opcode`
(60s, 13.7M executions, zero crashes).

Your job: reconcile the build's claims against actual git+disk state
(DEC-004 rule 1), run the 11 acceptance criteria's named tests
yourself, run the 12 verify checks (§15 — 8 generic + 4 repo-specific
bars), judge the two findings the build raised, and return
`✅ APPROVED` / `⚠ PUNCH LIST` / `❌ REJECTED` with per-finding
`SB-N` / `FU-N` labels.

## ⚠ The two findings — the crux of this verify

### SB-1 (build's label): AC5's HIT count is 0 on all three real Q2M frames

**Build's claim:** `apply_fix_bad_pixels_constant` measures **zero
replaced pixels** on L1021223.DNG, L1026016.DNG, and L1026192.DNG
respectively — the raw sensor data never contains a sample at
`Constant = 0`. AC5 pre-registered `count > 0` on each frame; the
build measured `count = 0` on each. Build labeled this SB-1 and
`#[ignore]`d AC5 with the finding recorded rather than adjusting the
assertion.

**Build's evidence that this is a MEASUREMENT, not a CODE BUG:**
AC1 (parser round-trip), AC4 (synthetic 5×5 plane returns Ok(1) with
the centre pixel replaced), and AC8 (tier-A red-proof: mutating the
applier to skip the replacement makes AC4 fail) all pass. So the
applier is reachable and correct on synthetic input.

**Your check:**
1. **Reproduce the zero HIT count** on all three decodable Q2M frames
   yourself. Use whatever measurement path is exposed (the tier-B
   test, or a scratch `irr` probe if one exists).
2. **INDEPENDENTLY verify that Constant=0 never occurs in the raw
   plane.** Read the raw u16 plane bytes for one Q2M frame directly
   (SPEC-012's `plane::unpack` or an equivalent) and count how many
   samples equal 0 across the whole ActiveArea. If the count is
   **strictly 0**, the build's finding is real and SB-1 downgrades to
   FU-N (AC5's assumption was wrong; disposition = closed with
   reason, or spec amendment). If the count is **> 0 but the applier
   returns 0**, there IS a real code bug and SB-1 stands.
3. **Judge the disposition.** If SB-1 is real-measurement-not-code-bug,
   is `#[ignore]` the right disposition, or should AC5 be amended to
   assert "count ≥ 0" and the finding filed as a follow-up? Either
   way, the applier ships. The build made a defensible call; judge it.

### FU-2 (build's label): `dnglab --srgb` never applies EXIF Orientation

**Build's claim:** `dnglab --srgb` emits pixels in sensor-native
orientation (5584×8368 for a landscape-mounted sensor), while
`develop_into` applies Orientation. On L1026016 (which has
Orientation = 6), the two outputs disagree in dimensions before any
scoring can happen — the same species of oracle-scope finding
SPEC-018's FU-6 flagged (which is folded into SPEC-021's Context).

**Your check:**
1. **Reproduce.** Run `dnglab analyze --srgb L1026016.DNG` and
   observe the output dimensions match the sensor-native shape
   (5584×8368 or 8368×5584?), then run `develop_into` on the same
   frame and observe the oriented output.
2. **Judge whether this is a NEW finding or evidence for SPEC-018's
   FU-6.** SPEC-018's ship dispositioned FU-6 as evidence for
   SPEC-021 (narrow SPEC-020 oracle scope). If SPEC-017/FU-2 is the
   same class, its disposition is `signal: <SPEC-021's target>` or
   simply "evidence for SPEC-021 — folded in Context".

## The four repo-specific verify bars (§15 bars 9–12)

**Bar 9 — did the oracle go red?** AC8 is the tier-A red-proof — mutating
the applier to skip the replacement turns AC4 red. Run it yourself,
watch AC4 fail on the mutation, revert, watch it pass. Show the
mutation md5s.

**Bar 10 — fuzz target exists and ran?** YES — build added
`fuzz/fuzz_targets/opcode.rs` (or extended the existing one). Confirm:
(1) target exists, (2) `just fuzz-opcode` recipe in `app.just` uses the
`PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly` prefix,
(3) AGENTS.md §6 code block gained the recipe, (4) CI on `cd82ca8`
shows the `fuzz smoke — opcode` job green (13.7M executions per the
handback), (5) `fuzz/seeds/opcode/` has the 28-byte real Q2M payload
plus the three hand-crafted adversarial variants.

**Bar 11 — provenance row for the new algorithm?** Check
`docs/provenance-ledger.md` — expected ONE new row for `src/opcode.rs`'s
`apply_fix_bad_pixels_constant` (or a row that specifically calls out
the applier, since SPEC-018 already added the parser row). Source: DNG
1.7.0.0 § **Chapter 7** (build corrected the design's "Chapter 6"
mis-citation — verify this against the actual DNG spec). Class 1
(published specification).

**Bar 12 — no new dependencies?** `Cargo.toml` `[dependencies]` MUST
remain empty. Run `cargo tree -e normal` — expect `irradiance v0.1.0`
alone. `cargo deny check licenses` on both graphs.

## The four codified lessons + a-fix-inherits

1. **`measurement-over-generalised`** — the "zero replaced pixels" is
   ONE measurement on ONE probe path. Independently confirm with a
   different measurement (raw-plane byte scan).
2. **`attribute-text-inside-doc-comments`** — the panic-free scanner
   still applies.
3. **`a-gate-that-fails-mutely-is-a-gate-that-never-ran`** — every new
   gate you run reports its own pass/fail explicitly.
4. **`unrun-docs-carry-errors` (N=6 as of SPEC-018 ship)** — the build
   corrected a Chapter 6 → Chapter 7 mis-citation by fetching the DNG
   spec PDF; verify by opening the same authoritative source (or
   quoting the exact text the build cited).

## The eight generic verify checks (§15)

1. All 11 acceptance criteria met and tested? AC5 is `#[ignore]`d per
   SB-1; judge whether that is "met" per SB-1's disposition above.
2. Failing tests from the spec now pass? 10 named in `## Failing
   Tests`; run each with `cargo test <name> -- --exact --nocapture`.
3. No drift from referenced decisions? Run `just decisions-audit
   --changed main`. SPEC-017 references DEC-002, DEC-004, DEC-005,
   DEC-011, DEC-016, DEC-018; DEC-024 (from SPEC-018) is context.
4. No constraint violations? The five blocking constraints apply.
5. Non-trivial implementer choices have DEC-\*? Judge whether any
   choice needed a DEC (build may not have emitted one; that is fine
   if the choices were spec-registered).
6. Implementer reflection answered (not mailed in)? Read HANDOFF-044's
   `## Completion` and confirm substance.
7. `cost.sessions[build]` present after handback-sync? Confirm
   `agent: claude-sonnet-5` (build's own correction from opus-5
   prediction; matches SPEC-018's FU-11 discipline).
8. Behavioural surfaces exercised? Parser (real bytes), applier
   (synthetic AC4 fixture with count assertion), fuzz (60s smoke).

## Return Criteria

1. **Verdict at the top:** `✅ APPROVED` / `⚠ PUNCH LIST` /
   `❌ REJECTED` with the ship SHA. Judge SB-1 explicitly: is it a
   ship-blocker or downgraded to FU-N?

2. **SB-1's independent verification.** Show the raw-plane byte scan
   count for at least one Q2M frame. If 0 samples equal `Constant`,
   quote that number and the command that produced it.

3. **Every finding labelled** `SB-N` / `FU-N`, per-spec numbering.
   SPEC-017's numbering restarts at 1.

4. **The four repo-specific bars (9–12) explicitly answered.**

5. **The 10 named tests run and each observed passing** — sum across
   all targets. `named-tests-can-pass-vacuously` protection.

6. **`just lint-ci` run locally, clippy version quoted** (0.1.98 pinned).

7. **CI observed green on `cd82ca8`** — run 34191738033 per the
   handback, 11 jobs including `fuzz smoke — opcode`.

8. **`cargo deny check licenses`** and
   `cargo deny --manifest-path fuzz/Cargo.toml check licenses` both
   quoted.

9. **`cost.sessions[build]` present** after handback-sync — 570,000
   tokens on claude-sonnet-5. Note that build correctly set `to_agent`
   to sonnet-5 before handback-sync ran (per SPEC-018's FU-11
   lesson) — this is the first spec where FU-11 discipline held.

10. **PR NOT opened by verify.** Orchestrator handles the PR.

11. **CORRECT this handoff's top-level `to_agent`** to your actual
    `message.model` BEFORE handback-sync runs, so the verify session
    is correctly attributed (per FU-11 signal).

## Cost

Fill the `handback:` block with real numbers. `notes:` MUST BE
double-quoted and CANNOT contain a bare `#`. Identify your own
transcript by scratchpad UUID. Price per-component + 20% uplift.

## References

- HANDOFF-044 build handoff and `## Completion` block.
- SPEC-017 spec (`## Acceptance Criteria`, `## Failing Tests`).
- SPEC-018 done spec — its FU-6 is SPEC-017/FU-2's sibling.
- SPEC-021 frame — where FU-2 evidence lands if you route it there.
- DEC-024 — SPEC-018's decision record (unchanged, referenced).
- `src/opcode.rs` (extended), `src/develop.rs` (integration point),
  `tests/opcode.rs` (or wherever the new tests live).
- DNG 1.7.0.0 specification § Chapter 7 (opcode list;
  FixBadPixelsConstant, OpcodeID 4).

## Out of scope

- Ship-cycle dispositions of FUs (SPEC-018's cycle handles those at
  ship, not verify).
- SPEC-020's oracle scope narrowing (that is SPEC-021's job).
- Opening the PR.
