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
  id: HANDOFF-080
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-02
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-033

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
  status: completed                # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 95
  branch: claude/spec-033-build-f223ec
  pr: null                         # filled with the PR number once opened
  completed_at: 2026-11-02         # YYYY-MM-DD
  notes: >
    Built WHOLE — did not need the pre-named [M] split (reused the demosaic/tone/irr
    harness as the design predicted). tokens_total is null because this is a desktop
    Claude Code (app) session: `/cost` is a terminal-dialog command the app session
    cannot run (the recurring `desktop-cost-tool-unavailable-handbacks-null` gap, same
    as the SPEC-030/031/032 builds). The orchestrator must recover the REAL count from
    THIS session's transcript at ship — do NOT invent a number. `get_usage` reports
    only the context-window snapshot (~345k), which is the wrong metric and a severe
    undercount, so it is deliberately not used here.
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-080: Colour-transform core + first colour develop (WB + colour matrix → linear-sRGB)

## Delegation Summary

The architect (`from_role`) hands `SPEC-033` to the implementer (`to_role`) for the
**build** cycle: implement STAGE-008's colour transform — a new `src/color.rs` with
`to_srgb_linear_into` (the matrix MATH) and `develop_color_into` (the compose), the two
`Error` variants, the `irr color` subcommand, and the analytic bit-exact oracle — making
every Failing Test in SPEC-033 pass. **Read SPEC-033 in full first; its `## Design` is
the algorithm and its `## Failing Tests` are your contract.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-033-colour-transform-core-first-colour-develop-wb.md` — **read the whole thing**; `## Design` is the exact math, `## Failing Tests` is the contract.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-008-colour-transform-white-balance-colour-matrix-display-encode-colorchecker-de.md`
- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks,
  runtime gotchas (DEC-004 rule 5). Also note: run `just test` (NOT `just test
  --all-features` — the recipe already bakes it in); corpus tests need
  `$IRRADIANCE_CORPUS_DIR` set (the Tier-A oracle does not).
- **Code to read/reuse (don't rebuild):**
  - `src/demosaic.rs` — `demosaic_into` (the camera-RGB producer you compose); match its
    house style — caller-owned buffer (`DEC-016`), `i32`/round/clamp, phase-driven,
    in-module `#[cfg(test)] mod tests` + a `tests/*_oracle.rs`.
  - `src/tone.rs` — `encode_srgb16_into` (the OETF `irr color` applies to your
    linear-sRGB output), and `require_matching_length`/`OutputBufferWrongLength` to mirror.
  - `src/develop.rs` — `normalize(sample, black, white)` (~ln 305–360) to reuse for the
    per-channel mosaic normalize; the warp path's internal-scratch precedent (~ln 844).
  - `src/ifd.rs` — `Sensor` colour fields + `Rational { num: i32, den: i32 }` (den may be 0).
  - `src/bin/irr.rs` — `cmd_demosaic` (mirror for `cmd_color`) + the P6 PPM writer + dispatch.
  - `src/lib.rs` — the `Error` enum (add two variants next to the demosaic set) + `pub mod`.

### Decisions that apply

- `DEC-033` — **this spec's arithmetic** (minted at design): committed-constant `f64`
  pipeline, `+ − × ÷` only (no FMA/transcendental), the committed `S`/`B` constants, the
  single-illuminant + identity-`AnalogBalance`/`CameraCalibration` simplifications.
- `DEC-018` — developed output is full-scale `u16` with clamped levels; `65535 ≡ 1.0`
  makes the matrix apply scale-free, and the clamp is load-bearing.
- `DEC-025` — the sRGB OETF is the committed `tone` table; `color` produces **linear**-sRGB
  and hands off to `tone`. Do NOT compute the OETF in `color`.
- `DEC-016` — `to_srgb_linear_into` / `develop_color_into` write a caller-owned `dst`;
  internal scratch is fine (as `develop_into`'s warp path allocates).
- `DEC-030` — the colour-metadata oracle is exact-rational; your transform is NOT (it is
  `f64`, per DEC-033) — don't conflate them.
- `DEC-017` — the red-proof must mutate a REAL coefficient and assert the OUTPUT changes
  (the `plane_oracle.rs` no-op lesson).

### Constraints that apply

Full text in `./guidance/constraints.yaml`:

- `no-panics-on-untrusted-input` — den-0 rationals, a singular `ColorMatrix`, absent
  metadata, off-gamut mixes, and wrong buffer lengths are **typed errors / clamped**,
  never panics and never a NaN written to `dst`.
- `oracle-must-be-shown-red` — the analytic oracle must be proven to fail on a perturbed
  matrix (mutant ≠ honest, honest control green).

### Prior related work

- `HANDOFF-074`/`-076` (SPEC-030/031 build) — the demosaic harness + oracle style you are
  mirroring; `tests/demosaic_oracle.rs` and the in-module tests are the template.
- `DEC-031` — why demosaic is Hamilton-Adams (context for the camera-RGB you consume).
- STAGE-007 shipped on `main`: `demosaic_into` is available and verified.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `src/color.rs` (new): `to_srgb_linear_into` + `develop_color_into` (public), the
  private `build_camera_to_srgb_linear` / 3×3 `inv` / `apply_matrix_into` / quantize, and
  the committed `S`/`B` `const` arrays (with their standard cited in a comment).
- `src/lib.rs`: `pub mod color;` + `Error::ColorSourceWrongLength { expected: u64, actual:
  usize }` and `Error::MissingColorCalibration`, with `Display` arms (mirror the demosaic
  variants).
- `src/bin/irr.rs`: an `irr color <file>` subcommand (mirror `cmd_demosaic`): unpack →
  `develop_color_into` → `tone::encode_srgb16_into` → P6 PPM (maxval 65535, BE RGB) on
  stdout; usage line + dispatch arm.
- `tests/color_oracle.rs` (new) + the `src/color.rs` in-module unit tests — every Failing
  Test in SPEC-033 passing (bit-exact apply, builder re-derivation, neutral→grey, clamp,
  red-proof, compose, typed-error/no-panic, degenerate dims). Tier-A (no corpus).
- `docs/provenance-ledger.md`: a colour-transform row (DNG colour model = class 1; sRGB =
  IEC 61966-2-1; Bradford CAT). `CHANGELOG.md`: `[Unreleased] Added` (public API).
- The mono develop + STAGE-006/007 oracles stay green; panic-free lints + fmt + validate pass.
- A PR against `main` (branch e.g. `claude/spec-033-build`), description referencing this
  handoff, SPEC-033, STAGE-008, PROJ-002, and `DEC-033/018/025/016`.
- **`just handback-sync SPEC-033` will read your handback — fill a REAL `tokens_total`
  from `/cost` (or note why null).**

**If it exceeds [M] — the pre-named split (SPEC-033 Notes).** Ship the **matrix MATH**
(`src/color.rs` `to_srgb_linear_into` + constants + `inv` + quantize + the two `Error`
variants + `DEC-033` + the `color_oracle.rs`/in-module analytic oracle, injecting a built
`M` — no demosaic needed) and leave `develop_color_into` + `irr color` + the dnglab
cross-check + the provenance row to a follow-on spec. Say so in the handback's "Follow-up
work" so the orchestrator mints it.

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

- **Dual-illuminant interpolation / scene-white adaptation** (SPEC-034). Use illuminant 1
  only; the `ColorMatrix` path treats its output as XYZ(D50) directly (documented
  simplification, DEC-033).
- **ColorChecker ΔE accuracy** (SPEC-035). The analytic oracle proves the MATH; it does
  not judge colorimetric correctness. The dnglab cross-check is a loose Tier-B sanity
  check only — do NOT tighten it into an accuracy gate.
- Parsing `AnalogBalance` / `CameraCalibration` (assumed identity), profile/HSV tone maps,
  `LinearizationTable`/`GainMap`, X-Trans/non-Bayer, a user-facing CLI.
- Changing `demosaic_into`, the mono `develop_into`, or any STAGE-006/007 behaviour — this
  is purely additive.

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

- **Branch / PR:** `claude/spec-033-build-f223ec` → PR (see `handback.pr`;
  opened against `main`, references SPEC-033 / STAGE-008 / PROJ-002 /
  DEC-033/018/025/016).
- **Completed at:** 2026-11-02
- **All acceptance criteria met?** **yes.** `to_srgb_linear_into` builds
  `M = C·FM1·diag(1/asn)` / `C·inv(CM1)·diag(1/asn)` once per image and applies
  per pixel; bit-exact apply (identity / diagonal / negative / overshoot, with a
  round-half-up tie) and bit-exact builder (FM path + CM path, `f64 ==` vs an
  independent re-derivation) are pinned; neutral→grey (`R == G == B`), clamp
  (off-gamut → `0`/`65535`), the FM1 red-proof (mutant ≠ honest, honest control
  green), the compose (`develop_color_into == normalize ∘ demosaic ∘ matrix`),
  every typed-error/no-panic path and the 1×1 degenerate case all pass. The mono
  develop + STAGE-006/007 oracles stay green; `just test` / `just lint-ci`
  (`-D warnings`) / `cargo fmt --check` / `just validate` all pass. Built
  **whole** — the pre-named [M] split was not needed. Behavioural pre-flight
  (AGENTS.md §12): `irr color` runs end-to-end on the real Canon 5D-III (FM1
  path) and Nikon D300S (inv(CM1) path) Bayer frames, emitting a valid P6 PPM
  with genuine, non-degenerate colour.
- **For `verify`:** n/a (this is the build handoff).

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** **null** — a desktop Claude Code (app) session cannot run
  `/cost` (a terminal-dialog command the app session does not expose). This is
  the known recurring `desktop-cost-tool-unavailable-handbacks-null` gap (same as
  the SPEC-030/031/032 builds). The orchestrator must recover the REAL count from
  THIS session's transcript at ship. **Do NOT invent a number.** (`get_usage`
  reports only the ~345k context-window snapshot — the wrong metric and a severe
  undercount — so it is deliberately not reported here.)
- **Estimated USD:** null (follows from tokens_total).
- **Duration (minutes):** ~95.
- **Source of the number:** none available (see above — recover from transcript).

### Drift and new artifacts

- **New decisions emitted:**
  - None. `DEC-033` was minted at design; this build implements it unchanged.
- **Deviations from spec:**
  - None material. The per-channel level-normalize indexes `black_levels` /
    `white_levels` by **2×2 mosaic position** (`(y&1,x&1)`), the DNG
    repeat-block layout, with the scalar `black_level`/`white_level` (then
    `0`/`2^bits-1`) as the fallback — the concrete reading of the spec's "per CFA
    colour" for a Bayer plane.
- **Follow-up work identified:**
  - **The Tier-B `dnglab`-render cross-check was deferred** (spec's optional
    `#[ignore]`/corpus-gated `"corpus_render_broadly_tracks_dnglab_srgb"`). The
    MATH is proven Tier-A bit-exact + a behavioural pre-flight on both real Bayer
    frames; a loose structural dnglab sanity check is worth adding but is NOT an
    accuracy gate (that is SPEC-035's ΔE) and did not block the build. A small
    follow-on spec (or an addition to SPEC-035's scope) could land it.
  - `SPEC-035` (ColorChecker ΔE) remains **blocked** on the maintainer's
    ColorChecker shot — already in the STAGE-008 backlog, flagged here as the
    thing that will actually judge the Nikon (single-illuminant) render's colour.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little — the spec's `## Design` was transcription-ready. The one thing
   that needed a judgment call was how to **index the per-channel black/white
   levels** ("per CFA colour" vs the DNG per-2×2-position layout); I used
   per-position (which is what real DNG `BlackLevel` arrays are), noted above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The references (DEC-033/018/025/016/030, DEC-017, the two constraints)
   were complete. Worth recording for future colour specs: `clippy::arithmetic_side_effects`
   does **not** fire on `f64` `+ − × ÷` (only integer ops), so the matrix math is
   lint-clean without an `#[allow]` — the `src/warp.rs` precedent the handoff
   cited is exactly right, and `rounded as u16` after a clamp is the sanctioned
   float→int conversion.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. I over-worried that the committed sRGB/Bradford constants
   might not compose a neutral back to `R == G == B` within ½ LSB; the Lindbloom
   constants are mutually consistent to ~1e-7, so the mid-grey neutral test holds
   bit-exact. I confirmed it empirically rather than trusting the reasoning —
   which is the right order, but I could have reached for the empirical check
   sooner instead of hand-deriving the accuracy.
