---
# Maps to ContextCore handoff.* semantic conventions.

handoff:
  id: HANDOFF-050
  cycle: verify                # build | verify — which cycle is delegated
  from_agent: claude-opus-4-7     # orchestrator's actual model this session.
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.verify. Prior
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: feat/spec-017-fixbadpixels-opcode
  pr: null                         # verify does not open the PR; orchestrator does
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one PHYSICAL, DOUBLE-QUOTED line
  synced_at: null                  # stamped by `just handback-sync` — do not edit
  verdict: null                    # approved | punch-list | rejected
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
