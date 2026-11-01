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
  id: HANDOFF-078
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model that ACTUALLY ran this build (message.model), not the tier hint
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-31
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-032

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
  tokens_total: null               # desktop session — /cost NOT exposed; see notes. Recover from transcript at ship.
  estimated_usd: null
  duration_minutes: null
  branch: claude/spec-032-fuzz-689903
  pr: "https://github.com/jysf/irradiance/pull/71"
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: "Claude desktop app (Code tab) session — the `/cost` terminal dialog is not available here, so tokens_total is null by the repo convention (NOT invented). Orchestrator recovers it from this session's transcript at ship; scratchpad/session UUID 0d68a8ef-a193-413b-8680-dfdf10ad0777 (model claude-opus-4-8). Test+fuzz only, no src change → no CHANGELOG (SPEC-025/027 convention)."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-078: CFA-demosaic fuzz + panic-free hardening + the GRBG/GBRG phase fixture

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-032` to `claude-opus-4-8` (implementer)
for the **build** cycle: add a `demosaic` libFuzzer target + tier-A seeds, and
the GRBG/GBRG bit-exact phase fixtures closing SPEC-030/FU-1 — **test + fuzz
only, no `src/demosaic.rs` change** (Hamilton-Adams is already phase-driven and
panic-free).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-007-demosaic-clean-room-malvar-he-cutler-mosaic-to-camera-linear-rgb.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-032-cfa-demosaic-fuzz-panic-free-hardening-on.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` + AGENTS.md §5 — the
  `cargo +nightly` / `+toolchain` trap (the rustup shim must be FIRST on PATH).
- **The template to mirror:** `fuzz/fuzz_targets/develop.rs` (`MAX_PIXELS` cap,
  "rejected = SUCCESS"); the fuzzed surface `src/demosaic.rs::demosaic_into`
  (read, don't change); the oracle `tests/demosaic_oracle.rs`; the CFA builders
  in `tests/support/tiff.rs` (`valid_cfa_sensor`, `cfa_dng_16bit`,
  `cfa_pattern_past_eof`, SPEC-027).

### Decisions that apply

- `DEC-003` — no corpus file as a fuzz seed; seeds are hand-built tier-A.
- `DEC-016` — `demosaic_into` allocates nothing; the harness caps `pixel*3`.
- `DEC-027` — the fuzz crate is its own cargo graph; it must lint clean under
  default clippy + `-D warnings` (`just lint-fuzz`), not the library's five
  panic-free lints.
- `DEC-017` — oracle red-proof discipline (NOT re-done here; the existing RGGB
  perturbation red-proof already covers the shared kernel).
- `DEC-031` — the kernel is Hamilton-Adams (patent-clean); unchanged.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — the whole point of the fuzz target + the
  tier-A `demosaic_is_panic_free_on_hostile_cfa` CI stand-in.
- `oracle-must-be-shown-red` — the GRBG/GBRG fixtures are bit-exact, each
  cross-checked against the independent `local_hamilton_adams` reference whose
  perturbation already turns the oracle red.

### Prior related work

- `SPEC-030` (PR #65) — bilinear foundation + the Tier-A oracle harness.
- `SPEC-031` (PR #69) — the Hamilton-Adams kernel this hardens.
- `SPEC-027` (PR #54) — the CFA fuzz-seed pattern + `valid_cfa_sensor`.

## Expected Deliverables

- `fuzz/fuzz_targets/demosaic.rs` (new) + `[[bin]] name = "demosaic"` in
  `fuzz/Cargo.toml`.
- `examples/fuzz-seeds.rs`: `demosaic_seeds()` + a `write_seeds("demosaic", …)`
  call; `fuzz/seeds/demosaic/` regenerated.
- `tests/demosaic_oracle.rs`: `GRBG`/`GBRG` constants + bit-exact fixtures,
  DC-preservation over all four phases, `demosaic_is_panic_free_on_hostile_cfa`.
- All SPEC-032 Failing Tests passing; the full gate set green.
- A PR against `main` referencing this handoff, the spec/stage/project, and the
  DECs; **no CHANGELOG** (test + fuzz only — confirm at ship).

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

- Any `src/demosaic.rs` or other `src/` change. The HA kernel is already
  phase-driven + panic-free; a src change would mean the fuzz/tests found a real
  defect — STOP and flag at verify, don't silently patch.
- The demosaic algorithm (HA stays), white balance / colour / ΔE / levels / crop
  (all STAGE-008), MHC (DEC-031).
- Re-running SPEC-031's red-proof (this adds phase coverage + fuzz, not a new
  oracle).
- Non-2×2 / X-Trans CFA support; real-image perceptual comparison.
- A CHANGELOG entry — test + fuzz only, no public API change (SPEC-025/027
  convention; confirm at ship).

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

- **Branch / PR:** `claude/spec-032-fuzz-689903` → PR filled in the spec timeline
  / `handback.pr` once opened.
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** yes.
  - `demosaic` fuzz target exists + registered (`fuzz list` shows it); seeds
    generated into `fuzz/seeds/demosaic/` (5 seeds).
  - **Nightly fuzz ran clean:** `cargo +nightly fuzz run demosaic` →
    **11,273,543 runs in 61s, 0 crashes / 0 panics** (cov 1174, corp 487).
  - GRBG **and** GBRG bit-exact fixtures pass (`adams_hamilton_grbg_matches_hand_computed`,
    `adams_hamilton_gbrg_matches_hand_computed`) — hand-computed tables, each
    also reproduced by the independent `local_hamilton_adams` reference.
  - DC-preservation now loops all four phases (`constant_mosaic_is_preserved_on_all_four_phases`).
  - Tier-A CI stand-in `demosaic_is_panic_free_on_hostile_cfa` passes (no nightly
    needed), with an Ok control guarding against a vacuous all-errors pass.
  - `src/demosaic.rs` unchanged; gates green: `just test` (all pass, 1 tier-B
    skip loud), `just lint-ci` (exit 0), `just lint-fuzz` (exit 0, DEC-027),
    `cargo fmt --check` (clean), `just validate` (clean).

### Cost self-report

- **Tokens (total):** null — see below.
- **Estimated USD:** null.
- **Duration (minutes):** ~30 (incl. the 61s nightly fuzz run + a cold build).
- **Source of the number:** none available. This ran in the **Claude desktop
  app (Code tab)**, where the `/cost` terminal dialog is not exposed, so no real
  token count is reachable from here. Per the repo convention the number is left
  `null` (NOT invented); the orchestrator recovers it from this session's
  transcript at ship (session/scratchpad UUID `0d68a8ef-a193-413b-8680-dfdf10ad0777`,
  model `claude-opus-4-8`).

### Drift and new artifacts

- **New decisions emitted:** none. SPEC-032 is test + fuzz hardening of an
  already-decided kernel; no new design choice warranted a `DEC-*`.
- **Deviations from spec:**
  - The spec asked for "≥1 bit-exact GRBG (and/or GBRG) fixture"; I added **both**
    GRBG and GBRG (cheap, and it makes "all four Bayer phases covered" literal
    rather than one-of-two).
  - Seeds: added a real-strip `cfa_dng_16bit` seed (reaches `demosaic_into`) and
    both byte orders of `valid_cfa_sensor`, in addition to the degenerate/truncated
    CFA the spec named — a strictly richer seed set, same builders.
  - The existing DC test was **renamed** `..._on_all_channels` →
    `..._on_all_four_phases` (the Failing-Tests name) and its loop extended; no
    behaviour lost.
- **Follow-up work identified:** none. This is the last STAGE-007 spec; the
  obvious next step is the STAGE-007 stage-ship then STAGE-008 (colour + ΔE),
  which is already on the backlog.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The spec's Design/Outputs were precise and the `develop.rs`
   template + the in-test `local_hamilton_adams` reference made the fuzz target and
   the expected-table derivation near-mechanical. The one judgement call was how to
   derive the GRBG/GBRG tables honestly: I transcribed SPEC-031's HA arithmetic into
   a standalone scratch program, then asserted both the real kernel **and** the
   independent in-test reference reproduce the pinned literals — so the table is
   twice-derived, not copied from the code under test.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-003/016/017/027 + the two constraints were exactly the relevant set.
   Worth noting for verify: the fuzz harness caps `pixel_count * 3 ≤ MAX_PIXELS`,
   which is a stricter effective pixel bound than the plane target's — intentional,
   since the RGB buffer is 3× the plane and `demosaic_into` allocates nothing (DEC-016).

3. **If you did this task again, what would you do differently?**
   — Nothing structural. If I were optimising, I'd have generated the expected
   tables and run the nightly fuzz in parallel with writing the oracle text rather
   than sequentially.
