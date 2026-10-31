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
  id: HANDOFF-075
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model ACTUALLY running this verify session (message.model), NOT tier_map's claude-opus-5 hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-31
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-030

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
  tokens_total: null               # /cost is NOT exposed to this desktop verify session — see notes
  estimated_usd: null              # derivable only once tokens_total is read from the transcript
  duration_minutes: 35
  branch: claude/spec-030-verify-04f726
  pr: https://github.com/jysf/irradiance/pull/64
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: >
    VERDICT: ✅ APPROVED — ship at commit 9e908f7 (the merged build), no changes needed.
    /cost is not available as a tool in this Claude Code desktop verify session, so the REAL
    cumulative verify-cycle total cannot be self-reported here (not invented, per the contract).
    The orchestrator should read /cost from THIS session's transcript at ship and fill
    cost.sessions directly — anchor on the scratchpad-dir UUID eae8c836-5e74-4969-a9b0-f5a132fad651,
    not a text grep (memory: identify-own-transcript-for-cost-handback). Cross-check figure: the
    session context window measured 197,792 tokens mid-verify (mcp get_usage) — a floor, not the
    cumulative /cost sum. Model: claude-opus-4-8. SEPARATELY: the BUILD handback (HANDOFF-074) also
    carries tokens_total: null for the same reason — the orchestrator must recover BOTH the build
    and the verify cost from their transcripts at ship (build anchor: 93026a80-0784-4ed6-9408-46a32be4a9f4).
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-075: Verify — demosaic foundation, the camera-space linear RGB

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-030` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — an
independent cold review of the SPEC-030 bilinear demosaic build (PR #63,
commit `b545feb`), already merged to `main` as `9e908f7` (the maintainer merged
early), confirming bit-exactness, a genuinely-red oracle, and spec compliance.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-007-demosaic-clean-room-malvar-he-cutler-mosaic-to-camera-linear-rgb.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-030-demosaic-foundation-the-camera-space-linear-rgb.md`
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-074-build-demosaic-foundation-the-camera-space-linear-rgb.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-016` — caller-owned buffer, no allocation. **Followed** (`demosaic_into`
  mirrors `unpack_into`/`develop_into`).
- `DEC-017` — red-proof asserts the OUTPUT changed. **Lesson followed**; the
  build uses the spec-sanctioned in-test injected-neighbour-swap variant, not
  the crate-copy rebuild — the spec's `## Failing Tests` explicitly offers this.
- `DEC-018` — u16 scene-linear. **Followed** (output is u16, no clamp needed for
  bilinear, recorded for SPEC-031's MHC gradient term).
- `DEC-024` — dnglab cannot isolate demosaic; no comparison oracle. **Followed**
  (Tier-A analytic fixtures only).

### Constraints that apply

- `oracle-must-be-shown-red` — **met and independently confirmed red** (see below).
- `no-panics-on-untrusted-input` — **met** (`lint-ci` green; all `.get()`/
  `checked_*`/`saturating_*`/`try_from`; the five `#![deny]` lints hold).
- `library-not-application` — **met** (demosaic is pure; PPM encode + I/O in `irr.rs`).
- `provenance-recorded-per-algorithm` — **met** (bilinear ledger row added, class 1).

### Prior related work

- `HANDOFF-074` — the build this reviews.
- `PR #63` (build, merged `9e908f7`), `PR #62` (design), `PR #61` (frame).

## Expected Deliverables

### The verify contract

Reviewed cold; did not assume the build correct.

- Acceptance criteria met? **Yes, all 8.** Tests actually pass? **Yes.** Build
  reflection answered honestly? **Yes — specific, not mailed-in.**
- Decision drift (`just decisions-audit --changed`): **none** (no active
  decision's `affected_scope` matches; the merged build sits within DEC-016/017/018/024).
- Constraint violations: **none.** Non-trivial choices missing a DEC: **none**
  (the in-test red-proof is spec-sanctioned; the typed-error variant names and
  stdout/stderr split follow repo idiom — not DEC-worthy).
- Runtime-behaviour surface exercised, not just shape validated: **yes** — the
  `irr demosaic` P6 path is driven end-to-end (synthetic DNG in the Tier-A test;
  I additionally ran it on both corpus Bayer DNGs).

## Out of Scope

- Rewriting the code (verify flags, does not patch).
- Malvar-He-Cutler (SPEC-031); WB/matrix/ΔE (STAGE-008); fuzz corpus (SPEC-032).

## Return Criteria — how to hand back

Completed: handback front-matter + `## Handback` filled, statuses `completed`,
verdict returned, verify PR opened.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-030-verify-04f726` → PR #64 (docs-only verify PR)
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** yes — all 8, mapped below.
- **Verdict:** ✅ **APPROVED — ship at commit `9e908f7`** (the merged build), no changes needed.

#### What I reproduced (did not take on trust)

- `just test` → **all green** (lib 105; demosaic_oracle 10; every STAGE-006
  regression oracle — plane, develop, metadata, colour_metadata, warp, tone,
  perceptual — green; 1 corpus-gated test ignored, as usual).
- `just lint-ci` → **green** (pinned stable clippy; the five panic-free lints hold).
- `cargo fmt --check` → **green.** `just validate` → **green** (front-matter).
- **Bit-exactness, independently:** I wrote a from-scratch bilinear reference
  (Python, from the spec's `## Notes` formulas — not copied from `src/demosaic.rs`
  nor from the test's `local_demosaic`) and it reproduces **every** hand table
  bit-exactly: EXPECTED_A (RGGB interior), EXPECTED_B (RGGB edges/corners, halves
  pinning round-half-up), EXPECTED_C (BGGR phase), and the 1×1 / 2×2 / 3×5
  degenerate fixtures. The tables are the true bilinear output, not vacuous.
- **THE LOAD-BEARING CHECK — the red-proof genuinely goes red.** Beyond the
  in-test red-proof, I mutated the **real** `src/demosaic.rs` kernel three ways
  (each reverted immediately; tree clean):
  1. `avg4` round-half-up weight `+2 → +1` → oracle **RED** (`edges_and_corners`,
     `degenerate_dims`, control). Fixture A stays green by design (all-exact
     averages); Fixture B is the half-pinning one.
  2. Hard-code the RGGB phase (ignore `Cfa.pattern`) → **only**
     `phase_is_driven_by_the_cfa_pattern` RED, all RGGB tests green — the BGGR
     fixture is precisely what catches a hard-coded kernel (AC2's claim has teeth).
  3. Drop the `down` neighbour in the real R/B green average (the red-proof's own
     fault, in the real kernel) → oracle **RED across the board**, including the
     red-proof's `assert_ne!(mutant, honest)` (the broken real kernel coincided
     with the in-test mutant, so the red-proof caught it too).
  I could not make any genuinely broken kernel pass — the one thing this cycle
  exists to catch is absent. The oracle is not a green-that-cannot-fail.
- **`irr demosaic` P6 on corpus:** Nikon D300S (4352×2868, RGGB) and Canon
  5D-III (5920×3950, RGGB) both emit exit-0 valid P6 (`P6\n{w} {h}\n65535\n`),
  payload exactly `w*h*3*2` bytes, honestly uncorrected (Canon min=1882 — black
  level not subtracted, as designed).

#### AC-by-AC

1. Interior+edges+corners bit-exact, ≥2 fixtures — ✅ (A+B, independently reproduced).
2. Phase read from `Cfa` (BGGR) proving RGGB not hard-coded — ✅ (mutation 2 confirms teeth).
3. Oracle red on perturbed kernel, control green, OUTPUT changed — ✅ (confirmed 3 ways).
4. Typed error (no panic) on mono/absent, unsupported CFA, wrong src/dst len — ✅.
5. 1×1 / 2×2 / 3×5 no panic, reflect border applied — ✅ (independently reproduced).
6. `irr demosaic <corpus Bayer DNG>` → valid P6 of w×h — ✅ (Tier-A synthetic test + my corpus smoke on both files).
7. Mono + STAGE-006 oracles stay green; no panic-capable public-API call — ✅.
8. Oracle Tier-A (synthetic, no corpus/tools), CI runs it in full — ✅.

### Cost self-report

- **Tokens (total):** null — `/cost` is not exposed as a tool in this Claude Code
  desktop verify session, so the cumulative total cannot be self-reported (not
  invented, per the contract). Floor cross-check: 197,792 tokens of session
  context mid-verify (`get_usage`), which undercounts the cumulative `/cost` sum.
- **Estimated USD:** null (derivable once tokens_total is read from the transcript).
- **Duration (minutes):** ~35.
- **Source of the number:** the orchestrator reads `/cost` from THIS session's
  transcript at ship and fills `cost.sessions` — anchor on scratchpad UUID
  `eae8c836-5e74-4969-a9b0-f5a132fad651` (memory: `identify-own-transcript-for-cost-handback`).
  The BUILD cost (HANDOFF-074) is likewise null and must be recovered at ship.

### Drift and new artifacts

- **New decisions emitted:** none — the build sits within DEC-016/017/018/024.
- **Deviations from spec:** none of substance. The red-proof uses the spec's own
  sanctioned in-test injected-neighbour-swap (Tier-A, instant) rather than the
  DEC-017 crate-copy rebuild — the spec's `## Failing Tests` explicitly offers
  this ("a mutated `demosaic` per DEC-017, **or** an injected neighbour swap").
- **Follow-up work identified (non-blocking, NOT a punch-list — verdict is APPROVED):**
  - **Untested phase paths GRBG / GBRG.** Fixtures cover RGGB and BGGR — both
    *anti-diagonal* green phases. GRBG and GBRG (greens on the *main* diagonal)
    are not fixture-verified. The spec AC2 only requires "BGGR (or GRBG)", so the
    build **meets its contract**, and the kernel is provably phase-driven and
    correct for all four phases by inspection. But STAGE-007's Design Notes
    aspired to "cover the other three Bayer phases" and flagged the
    `measurement-over-generalised` trap; a one-line GRBG or GBRG fixture would
    close the only untested phase path. Natural home: SPEC-032 (edge-phase
    hardening) or an added fixture in SPEC-031's MHC oracle. Recorded here, not
    as a blocker.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's four RGGB formulas + the sanctioned red-proof
   alternative made the review mechanical; the build's AC-by-AC PR body and the
   inline-pinned arithmetic in the fixtures let me re-derive by hand quickly.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-016/017/018/024 + the two named constraints were exactly the set
   that governs this surface.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Mutating the *real* kernel (not just trusting the
   in-test red-proof) is the check that actually settles "can this oracle fail?",
   and doing all three mutations — weight, phase, dropped-neighbour — mapped
   precisely which fixture guards which property; I would lead with that again.
