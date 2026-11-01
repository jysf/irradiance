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
  id: HANDOFF-077
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model ACTUALLY running this verify (message.model), not the tier hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-31
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-031

project:
  id: PROJ-002
  stage: STAGE-007
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/spec-031-verify-311cc7
  pr: "https://github.com/jysf/irradiance/pull/68"  # verify PR to main (contains build + verify; #67 closes as included)
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: "✅ APPROVED — ship at 5450c70. /cost is NOT exposed to this desktop Claude Code (Code tab) verify session either, so tokens_total is null (not invented, AGENTS §13). Recover the real number from THIS verify session's transcript at ship — anchor on the scratchpad-dir UUID 21f3a4a2-4f9e-4e86-9dba-37e22e208e34, not a text grep (identify-own-transcript-for-cost-handback). This is the SAME handling the build used (HANDOFF-076, its own UUID 9b462a6e-5ff6-421d-be8a-1b6ea3098b04), so ship recovers TWO build/verify nulls for SPEC-031. Because build PR #67 is unmerged, this verify branch carries build+verify together; merging the verify PR lands both and closes #67 as included."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-077: Hamilton-Adams directional demosaic (patent-clean quality kernel) — verify

## Delegation Summary

One sentence: `claude-opus-5` (acting as `architect`) hands `SPEC-031` to
`claude-opus-4-8` (acting as `verifier`) for the **`verify`** cycle — an
independent cold review of the two-pass **Adams-Hamilton** directional demosaic
built in `HANDOFF-076` (PR #67, open), reproducing the gates and adversarially
breaking the kernel to prove the bit-exact oracle is live.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-031-malvar-he-cutler-gradient-corrected-5-5-kernels.md`
  (slug historical; algorithm is Hamilton-Adams per `DEC-031`).
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-076-build-*.md`.
- **Code under review:** `src/demosaic.rs`, `tests/demosaic_oracle.rs`,
  `docs/provenance-ledger.md`, `CHANGELOG.md`, `decisions/DEC-031`, `DEC-032`.

### Decisions that apply

- `DEC-031` — the quality kernel is Hamilton-Adams (US 5,629,734, expired), NOT
  Malvar-He-Cutler (US 7,502,505, active ~2027-01-25). Honoured: the code is HA.
- `DEC-016` — `demosaic_into` is caller-owned, zero-alloc. Honoured via `DEC-032`.
- `DEC-017` — the red-proof must assert `mutant != honest`, not merely compile.
- `DEC-018` — output is clamped `u16`; HA's predictor can overshoot, so the
  clamp is load-bearing (bilinear's was not).
- `DEC-032` — the two passes share the green plane in `dst`'s own green slots
  (zero scratch). Independently audited for aliasing (see below).

### Constraints that apply

- `oracle-must-be-shown-red` — satisfied and INDEPENDENTLY RE-PROVEN (below).
- `no-panics-on-untrusted-input` — `.get`/`checked_*`/`saturating_*` throughout,
  no `#[allow]` on the library path; `lint-ci` (pinned clippy) green.

### Prior related work

- `HANDOFF-074`/`HANDOFF-075` — SPEC-030 bilinear foundation + the oracle harness.
- `PR #67` (OPEN) — the SPEC-031 build this reviews; this verify branch is cut
  from it, so this verify PR contains build + verify together.

## Expected Deliverables

*(verify handoff — see the verdict below.)*

### If `cycle: verify` — the verify contract

- Acceptance criteria met? **Yes — all eight.** Tests pass? **Yes — 16/16
  binaries, demosaic 22/22.** Reflection honest? **Yes — substantive, not
  mailed-in; it names the one genuinely unclear point (where the inter-pass
  green lives under DEC-016) and records it as DEC-032.**
- Decision drift — `just decisions-audit --changed` → clean. No undocumented
  non-trivial choice (the `floor_div`-vs-`div_euclid` lint refactor is pinned
  behaviourally-equal by `floor_div_matches_div_euclid`).
- Constraint violations — none found.
- Behavioral surface exercised — yes: the real `demosaic_into` is driven on
  synthetic mosaics and `irr demosaic` is spawned end-to-end to a P6 PPM; and I
  broke the real kernel three ways to confirm the oracle rejects it.
- Verdict: **✅ APPROVED — ship at commit `5450c70`.**

## Out of Scope

- White balance / colour matrix / ΔE / display encode — STAGE-008.
- GRBG/GBRG fixture — SPEC-030/FU-1, correctly deferred to SPEC-032 (still open;
  not a regression — the main-diagonal-green path exists by construction).
- CFA-demosaic fuzz — SPEC-032 (on the STAGE-007 backlog).

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-031-verify-311cc7` → verify PR to `main`
  (contains build + verify, because build PR #67 is unmerged; link recorded in
  the timeline + `handback.pr` once opened). Reviewed at `HEAD = 5450c70`.
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** **Yes — all eight SPEC-031 ACs.**
- **Verdict:** **✅ APPROVED — ship at commit `5450c70` (branch tip).**

#### What I reproduced (not taken on trust)

- **Gates, all green locally in this worktree:**
  - `just test` — the whole suite: **16/16 test binaries `ok`**, demosaic oracle
    **22/22**. The tier-B corpus SKIPs are **loud** (printed by `corpus-status`),
    not silently green; SPEC-031's own oracle is Tier-A and runs fully.
  - `just lint-ci` — pinned `~/.cargo/bin/cargo +stable clippy 0.1.98`, `-D
    warnings`, all targets/features: **clean**.
  - `cargo fmt --check` — clean. `just validate` — 36 + 161 artifacts parse.
- **The load-bearing red-proof — I broke the REAL kernel, three ways, each went
  red** (the `tests/plane_oracle.rs` lesson: prove the oracle CAN fail):
  1. **Pass-1 green predictor** — dropped the `(WW−2C+EE)` correction from `gh`.
     → `adams_hamilton_rggb_matches_hand_computed`, `phase_is_driven_by_the_pattern_bggr`,
     `the_honest_control_is_green`, `degenerate_dims_do_not_panic` all FAILED.
  2. **Pass-2 diagonal classifier** — flipped `cl_p < cl_n` to `cl_p > cl_n`
     (R/B at red/blue sites). → the RGGB hand-table FAILED.
  3. **Pass-2 green-site colour-difference** — dropped the `2·` weight on
     `(p1+p2)`. → RGGB + BGGR hand-tables FAILED.
     All three reverted (`/usr/bin/git checkout`), tree clean, oracle green again
     (22/22). The oracle is genuinely live across all three kernel paths — NOT a
     no-op the way `plane_oracle.rs` once was.
- **Bit-exactness, hand-recomputed independently.** Pixel (2,2) of the RGGB 6×6:
  RED site, CLh=32<CLv=44 ⇒ horizontal green gh=464 ⇒ G=rnd4=116; blue from
  diagonals with CLn=17<CLp=59 ⇒ rnd4(219)=55; and the diagonal green at (1,1) is
  itself a genuine classifier **tie** (CLh=CLv=52 ⇒ rnd8(676)=85) — the nested
  tie that feeds the blue. Result (52,116,55) matches `EXPECTED_HA` exactly. The
  directionality fixture (CLv=0≪CLh=7000 ⇒ vertical ⇒ G=4000≠bilinear 2750) and
  the tie fixture (CLh=CLv=400 ⇒ rnd8=975 ≠ rnd4(gh)=1000 ≠ rnd4(gv)=950) both
  hand-check out.
- **DEC-032 aliasing — independently audited and sound.** Pass 1 writes only
  green (middle of each `as_chunks_mut::<3>` triple), reads only `src`. Pass 2
  takes `dst: &[u16]` (shared borrow reads only `+1` green slots); that borrow
  ends when `interpolate_red_blue` returns, THEN `get_mut(base)`/`get_mut(base+2)`
  write only `+0`/`+2`. Green read-only in pass 2, R/B write-only, disjoint — no
  read-after-write hazard, order-independent. The borrow checker accepting this
  is itself the proof the borrows don't overlap.
- **No regression.** Every STAGE-006 oracle green (`colour_metadata_oracle`,
  `plane_oracle`, `plane_unpack`, `metadata_oracle`) + develop/tone/perceptual/
  warp/opcode/ifd/corpus + lib unittests. Mono path returns
  `NotAColorFilterArray`; `irr demosaic` emits a valid P6 PPM.

### Cost self-report

- **Tokens (total):** null — `/cost` is not exposed to this desktop Claude Code
  (Code tab) verify session, exactly as for the build (HANDOFF-076). Not invented
  (AGENTS §13). Recover from THIS session's transcript at ship (scratchpad UUID
  `21f3a4a2-4f9e-4e86-9dba-37e22e208e34`; `identify-own-transcript-for-cost-handback`).
- **Estimated USD:** null (follows from tokens_total).
- **Duration (minutes):** ~40 (estimate).
- **Source of the number:** none available — see above.

### Drift and new artifacts

- **New decisions emitted:** none (verify cycle).
- **Deviations from spec:** none. Full HA (green + R/B) shipped, not the green-only
  fallback. One intentional, documented naming artifact worth stating plainly:
  the spec/branch slug still reads "malvar-he-cutler" while the code is
  Hamilton-Adams — the frame-stage historical slug (DEC-031), not a defect.
- **Follow-up work identified:** none new.
  - `SPEC-030/FU-1` (GRBG/GBRG main-diagonal-green fixture) — **still open**,
    correctly deferred to `SPEC-032`'s edge-phase hardening. The build's RGGB +
    BGGR fixtures cover only the two anti-diagonal-green phases; GRBG/GBRG work by
    construction (`BayerPhase::from_cfa`'s `greens_main_diagonal` branch + the
    phase-driven R/B assignment) but are not yet fixture-pinned. Not a blocker:
    not a SPEC-031 AC, and the stage note itself sanctions the deferral ("fixture
    coverage, not a bug").
  - `SPEC-032` (CFA-demosaic fuzz + panic-free hardening) — on the backlog.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's `## Design` pins the exact arithmetic and the build's
   handback flagged the one choice worth auditing (DEC-032's green-slot reuse) up
   front, so the review went straight to reproducing gates and attacking the
   oracle rather than reverse-engineering intent.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-031`/`016`/`017`/`018`/`032` + the two constraints were the right
   and complete set.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Mutating three *distinct* kernel paths (not just
   re-running the build's own injected fault) is what turns "the red-proof passes"
   into "the oracle provably rejects a wrong kernel everywhere it matters"; I'd
   keep that as the default shape for any bit-exact-oracle verify.
