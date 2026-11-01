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
  id: HANDOFF-076
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model ACTUALLY running this build (message.model), not the tier hint
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  duration_minutes: 45
  branch: claude/spec-031-hamilton-adams-56ecec
  pr: null
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: "/cost is NOT exposed to this desktop Claude Code (Code tab) session, so tokens_total is left null (not invented). Recover the real number from THIS session's transcript at ship, as done for SPEC-030's build/verify (identify-own-transcript-for-cost-handback: anchor on the scratchpad-dir UUID 9b462a6e-5ff6-421d-be8a-1b6ea3098b04, not a text grep). Full Hamilton-Adams shipped (green + R/B) — NOT the green-only fallback."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-076: Hamilton-Adams directional demosaic (patent-clean quality kernel)

## Delegation Summary

One sentence: `claude-opus-5` (acting as `architect`) hands `SPEC-031`
to `claude-opus-4-8` (acting as `implementer`) for the **`build`** cycle —
replace SPEC-030's bilinear interpolation inside `demosaic::demosaic_into` with
the two-pass **Adams-Hamilton** directional demosaic (DEC-031: Hamilton-Adams,
NOT Malvar-He-Cutler — MHC is patent-encumbered).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-007-demosaic-clean-room-malvar-he-cutler-mosaic-to-camera-linear-rgb.md`
  (title slug says MHC; algorithm is Hamilton-Adams per DEC-031).
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-031-malvar-he-cutler-gradient-corrected-5-5-kernels.md`
  — its `## Design` section pins the EXACT HA arithmetic; build to it precisely.
- **Harness to extend:** `src/demosaic.rs` (`BayerPhase`, `reflect`, `sample`,
  `u32_to_u16`, `demosaic_into`) and `tests/demosaic_oracle.rs`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, the
  five panic-free lints, `just lint-ci` (the only clippy that matches CI).

### Decisions that apply

- `DEC-031` — the quality kernel is Hamilton-Adams (US 5,629,734, expired), NOT
  Malvar-He-Cutler (US 7,502,505, active to ~2027-01-25). A patented method in a
  permissive crate defeats the thesis; clean-room beats copyright, not patents.
- `DEC-016` — `demosaic_into` takes a caller-owned `&mut [u16]` and allocates
  nothing. HA is two-pass; the green plane is held in `dst`'s own green slots
  (`dst[p*3+1]`) between passes, so no new allocation is needed.
- `DEC-017` — the red-proof must assert the mutated kernel's OUTPUT differs from
  honest (`mutant != honest`), not merely that it compiled (the plane_oracle.rs
  lesson). SPEC-030's in-test local reference + perturbation is the Tier-A form.
- `DEC-018` — developed output is `u16` full-scale with clamped levels; HA's
  gradient term can overshoot `[0, 65535]` and MUST clamp (bilinear could not).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `oracle-must-be-shown-red` — the demosaic oracle ships with a deliberate-fault
  test that turns it red, plus an honest-tree negative control.
- `no-panics-on-untrusted-input` — no unwrap/expect/panic/indexing/overflowing
  arithmetic on library paths; use `.get()`, `checked_*`, `saturating_*`,
  `try_into()`. All HA intermediates are signed `i32` via saturating ops; the
  values provably can't reach i32 saturation at 16-bit input.
- `provenance-recorded-per-algorithm` — an Adams-Hamilton row in
  `docs/provenance-ledger.md` (US 5,629,734, expired; DEC-031).

### Prior related work

- `HANDOFF-074` (build) / `HANDOFF-075` (verify) — SPEC-030, the bilinear
  foundation + the bit-exact analytic oracle this extends. `PR #63`/`#64`/`#65`.

## Expected Deliverables

- `src/demosaic.rs`: the two-pass Adams-Hamilton interpolation behind the
  UNCHANGED `pub fn demosaic_into(sensor, &[u16], &mut [u16])` — pass 1 fills
  directional green at every pixel (finalized to `u16` before pass 2); pass 2
  fills red/blue (colour-difference at green sites, gradient-directed diagonals
  at the opposite-colour site). `rnd4`/`rnd8` (Euclidean) + clamp.
- `tests/demosaic_oracle.rs`: HA bit-exact hand-computed fixtures (interior +
  edges/corners on a ≥5×5 fixture), the classifier tie, DC-preservation, a
  directionality fixture, a BGGR phase fixture, and a perturbed-HA red-proof
  (mutant ≠ honest) with an honest control. Degenerate no-panic (1×1/2×2/3×5)
  and the typed-error cases stay green.
- `docs/provenance-ledger.md`: an Adams-Hamilton row beside the bilinear note.
- All SPEC-031 Failing Tests passing; gates green (`just test`, `just lint-ci`,
  `cargo fmt --check`, `just validate`).
- A PR against `main` from branch `claude/spec-031-hamilton-adams-56ecec`.
- PR description referencing: this handoff ID, SPEC-031, STAGE-007, PROJ-002,
  `DEC-031`/`DEC-016`/`DEC-017`/`DEC-018`, and any deviation (incl. green-only).

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- White balance, colour matrix / ForwardMatrix, ΔE, display encode — STAGE-008.
- Levels / black-subtract / crop / orientation.
- Malvar-He-Cutler or any `'505`-encumbered method (DEC-031); AHD/AMaZE/LMMSE/RCD.
- Non-2×2 / X-Trans CFA; real-image dnglab comparison (STAGE-008 ΔE).
- **L-risk fallback:** if the FULL HA (green + R/B) does not fit cleanly as one
  `[M]` build, ship directional GREEN only (R/B left as SPEC-030 bilinear — a
  coherent, shippable quality increment) and say so explicitly in the handback;
  the R/B colour-difference then becomes a follow-on spec. Prefer full HA if it
  fits; do NOT half-finish both.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-031-hamilton-adams-56ecec` → PR to `main` (link
  recorded in the `handback.pr` front-matter once opened).
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** **Yes — full Hamilton-Adams (green + R/B), not
  the green-only fallback.** Every SPEC-031 acceptance criterion is covered:
  - Directional green (gradient-chosen axis + second-difference correction) then
    colour-difference red/blue, phase-driven by `Sensor.color_filter`
    (`src/demosaic.rs::green_at_site` / `interpolate_red_blue` /
    `diagonal_directed` / `colour_difference`).
  - `adams_hamilton_rggb_matches_hand_computed` — a 6×6 RGGB fixture (genuine
    offset-2 neighbours interior, reflect at the border) matches a hand-computed
    table exactly, four pixels pinned inline with arithmetic.
  - `green_follows_the_low_gradient_axis` — green takes the flat (vertical) axis,
    4000, distinct from bilinear's 4-average 2750.
  - `classifier_tie_averages_both_predictors` — CLh==CLv ⇒ 975 = rnd8(gh+gv),
    distinct from rnd4(gh)=1000 and rnd4(gv)=950.
  - `constant_mosaic_is_preserved_on_all_channels` — DC preservation (RGGB+BGGR).
  - `phase_is_driven_by_the_pattern_bggr` — BGGR correct with the same code.
  - `a_perturbed_hamilton_adams_step_turns_the_oracle_red` + `the_honest_control_is_green`
    — red-proof asserts mutant ≠ honest (inverted pass-1 classifier), control green.
  - `degenerate_dims_do_not_panic` (1×1 / 2×2 / 3×5 recomputed under HA) + the
    SPEC-030 typed-error cases carry over green.
- **Gates (observed green locally):** `just test` (whole suite — 16 binaries ok;
  the tier-B corpus SKIPs loudly as it is absent, as every prior spec's suite run
  does — SPEC-031's own oracle is Tier-A and runs fully), `just lint-ci`,
  `cargo fmt --check`, `just validate`. Oracle is Tier-A (no corpus).

### Cost self-report

- **Tokens (total):** null — **`/cost` is not exposed to this desktop Claude Code
  (Code tab) session.** Not invented (AGENTS §13: an invented number is worse than
  none). Recover the real total from THIS session's transcript at ship, as was done
  for SPEC-030's build/verify — anchor on the scratchpad-dir UUID
  `9b462a6e-5ff6-421d-be8a-1b6ea3098b04`, not a text grep
  (`identify-own-transcript-for-cost-handback`).
- **Estimated USD:** null (follows from tokens_total).
- **Duration (minutes):** ~45 (estimate).
- **Source of the number:** none available — see above.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-032` — Hamilton-Adams's two passes share the green plane in `dst`'s own
    green slots, so the two-pass kernel still allocates nothing (keeps `DEC-016`).
- **Deviations from spec:**
  - None. Full HA (green + R/B) landed as one [M] build; the pre-named green-only
    fallback was not needed.
  - Implementation detail worth flagging for verify: the spec's `## Design` leaves
    rounding as `(n+2).div_euclid(4)` / `(n+4).div_euclid(8)`. I expressed these as
    a `floor_div` helper built from `checked_div`/`checked_rem` (identical result
    for the positive divisors 4/8) rather than the `div_euclid` operator, and used
    `saturating_*` for +/-/* and a `saturating_neg`-based `abs_i32`, to satisfy the
    panic-free lint set without `#[allow]`. A `floor_div_matches_div_euclid` unit
    test pins the equivalence over -40..=40.
- **Follow-up work identified:**
  - `SPEC-032` (already on the STAGE-007 backlog) — CFA-demosaic fuzz on
    degenerate/edge-phase planes + SPEC-030/FU-1 (GRBG/GBRG fixture). The HA path
    is panic-free by construction (all `.get`/`checked_*`/`saturating_*`) but has
    no fuzz target yet; it adds no new byte-input parser, so this stays SPEC-032's.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little — the spec's `## Design` pinned the exact arithmetic (classifiers,
   predictor numerators, rnd4/rnd8, the tie branch, the green-site vs opposite-site
   R/B split), so build was close to a transcription. The one thing not spelled out
   was *where* the inter-pass green plane lives given `DEC-016`'s no-alloc rule;
   resolving that (in `dst`'s own green slots) is `DEC-032`.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No; `DEC-031`/`016`/`017`/`018` + `oracle-must-be-shown-red` /
   `no-panics-on-untrusted-input` were the right set. I added `DEC-018` to the
   handoff explicitly because HA's clamp is load-bearing where bilinear's was not.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Writing the independent reference first (scratchpad +
   in-test `local_hamilton_adams`) and hand-verifying ~5 pixels before coding `src`
   made the bit-exact tables trustworthy on the first run; I'd keep that order.
