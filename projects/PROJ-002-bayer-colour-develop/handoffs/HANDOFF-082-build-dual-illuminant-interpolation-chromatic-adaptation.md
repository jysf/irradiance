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
  id: HANDOFF-082
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-02
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-034

project:
  id: PROJ-002
  stage: STAGE-008
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-082: Dual-illuminant interpolation — scene-adaptive colour matrix

## Delegation Summary

The architect hands `SPEC-034` to the implementer for the **build** cycle: make SPEC-033's
`color::build_camera_to_srgb_linear` **interpolate** the two DNG calibrations at the scene
colour temperature (DNG `NeutralToXY` + McCamy CCT, DEC-034), instead of using illuminant 1
only — a localized, internal, no-public-API change with a byte-identical single-illuminant
fallback. **Read SPEC-034 in full first; its `## Design` is the algorithm and its
`## Failing Tests` are your contract.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-034-dual-illuminant-interpolation-chromatic-adaptation.md` — **read the whole thing**; `## Design` is the exact algorithm (NeutralToXY iteration, McCamy CCT, the illuminant table, the matrix lerp, the fallback), `## Failing Tests` is the contract.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-008-colour-transform-white-balance-colour-matrix-display-encode-colorchecker-de.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — `just test` (NOT `just test
  --all-features`); the Tier-A oracle needs no corpus.
- **Code to read/extend (reuse, don't rebuild):**
  - `src/color.rs` — **this is the whole surface.** Extend `build_camera_to_srgb_linear`
    (SPEC-033); reuse `mat_mul`, `inv`, `wb_diagonal`, `matrix_from_rationals`,
    `rational_to_f64`, the committed `SRGB_FROM_XYZ_D65` / `BRADFORD_D50_TO_D65`, and the
    `#[cfg(test)] mod tests` + `tests/color_oracle.rs` oracle harness. `apply_matrix_into`
    and the public fns are **unchanged**.
  - `src/ifd.rs` — `Sensor.color_matrix_2`, `forward_matrix_2`,
    `calibration_illuminant_1`/`_2`, `as_shot_neutral`, `Rational`.

### Decisions that apply

- `DEC-034` — **this spec's method** (minted at design): DNG reciprocal-temperature
  `NeutralToXY` weight (3 iterations), McCamy cubic CCT (committed constants, polynomial),
  committed illuminant→CCT table, matrix lerp, single-illuminant fallback, deferred CM-path CAT.
- `DEC-033` — the f64 `+ − × ÷`-only pipeline you extend (no FMA/transcendental; McCamy is a
  polynomial — write `n*n*n`); the weight runs once per image, off the pixel path.
- `DEC-030` — the rationals the matrices/neutral are read from (den may be 0 → guard).
- `DEC-017` — the red-proof must mutate a REAL coefficient (a `ColorMatrix2` entry) and
  assert the OUTPUT changes.

### Constraints that apply

Full text in `./guidance/constraints.yaml`:

- `no-panics-on-untrusted-input` — every divide guarded (`X+Y+Z ≤ 0`, `0.1858 − y == 0`,
  `T1 == T2`, singular `CM(g)`, den-0 rational) → `MissingColorCalibration` or the graceful
  single-illuminant fallback; never a panic, never a NaN in `dst`.
- `oracle-must-be-shown-red` — prove the interpolation oracle fails on a perturbed `ColorMatrix2`.

### Prior related work

- `SPEC-033` / `HANDOFF-080` — the colour transform this extends; its `src/color.rs` +
  `tests/color_oracle.rs` are the harness and style to mirror. Shipped on `main`.
- `DEC-033` — why the arithmetic is committed-constant f64.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `src/color.rs`: `build_camera_to_srgb_linear` interpolates (`FM(g)`/`CM(g)`); new
  **private** helpers `interpolation_weight` (`Option<f64>`, `None` ⇒ fallback),
  `mccamy_cct`, `cct_of_illuminant`, `lerp_matrix`, `xyz_to_xy`, `mul_mat_vec`; a committed
  `ILLUMINANT_CCT` table with each code's source. **No public API change, no new `Error`
  variant.**
- `tests/color_oracle.rs` + `src/color.rs` in-module tests — every Failing Test passing
  (weight matches hand-computed DNG NeutralToXY to the bit; clamp outside the range;
  McCamy on known chromaticities; FM-path + CM-path interpolation; **single-illuminant
  byte-identical to SPEC-033 — re-assert the shipped SPEC-033 fixtures unchanged**;
  perturbed-`ColorMatrix2` red-proof; degenerate inputs no-panic). Tier-A (no corpus).
- `docs/provenance-ledger.md`: a row for the DNG `NeutralToXY` interpolation + McCamy CCT.
  `CHANGELOG.md`: `[Unreleased] → Changed` (dual-calibration DNGs now interpolate;
  **public API unchanged**; single-calibration files byte-identical).
- The mono develop + demosaic + STAGE-006/007 + SPEC-033 colour oracles stay green;
  panic-free lints + fmt + validate pass.
- A PR against `main` (branch e.g. `claude/spec-034-build`), description referencing this
  handoff, SPEC-034, STAGE-008, PROJ-002, and `DEC-034/033/030`.
- **`just handback-sync SPEC-034` reads your handback — fill a REAL `tokens_total` from
  `/cost` (or null + note why; the orchestrator recovers it from your transcript at ship).**

**If it exceeds [M] — the pre-named split (SPEC-034 Notes):** ship the interpolation
*machinery* first (`lerp_matrix`, `mccamy_cct`, `cct_of_illuminant` + the table,
`xyz_to_xy`, and the analytic tests with `g` **injected** — fully testable without the
iteration), and defer the iterative `NeutralToXY` weight-from-neutral + the
`build_camera_to_srgb_linear` wiring to a follow-on spec. Record it in the handback's
"Follow-up work" so the orchestrator mints it.

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

- **The ColorMatrix-path scene-white→D50 chromatic-adaptation CAT** (SPEC-034 Non-Goals /
  DEC-034). Keep SPEC-033's implicit-D50 structure for the `inv(CM(g))` path; do NOT add a
  general Bradford CAT. SPEC-035's ΔE decides whether it is needed.
- Planckian/Robertson CCT (use McCamy — DEC-034); `CameraCalibration`/`AnalogBalance`
  (identity, DEC-033); profile curves / HueSatMap; highlight recon; auto-WB.
- Any public API / signature change, or any new `Error` variant — the fallback is not an
  error, and singular/ill-shaped reuses `MissingColorCalibration`.
- Changing `apply_matrix_into`, the committed `C`, `demosaic_into`, the mono `develop_into`,
  or any STAGE-006/007 behaviour — this is a localized refinement of the matrix build.

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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
