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
  id: HANDOFF-052
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-26
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-019

project:
  id: PROJ-001
  stage: STAGE-003
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
  tokens_total: 41313385           # REAL combined count — what cost-audit reads
  estimated_usd: 105.42            # tokens_total × your rate, or your harness's number
  duration_minutes: 25
  branch: claude/spec-019-tone-curve-13488a
  pr: https://github.com/jysf/irradiance/pull/18
  completed_at: 2026-10-26         # YYYY-MM-DD
  notes: "all 10 ACs met; combined token total (in+out+cache-create+cache-read) summed from this session's own Claude Code transcript (interface exposes no /cost); estimated_usd at per-category Opus list rates (in $15 / out $75 / cache-write $18.75 / cache-read $1.50 per Mtok)"
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-052: Tone curve and the three output modes

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-019` to
`claude-opus-5` (acting as implementer) for the **build** cycle — add a
table-driven sRGB tone curve and the three output encoders (scene-linear f32,
16-bit sRGB u16, 8-bit sRGB u8) as a composable layer over `develop_into`'s
scene-linear `u16` output.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-001-monochrome-dng-develop/brief.md`
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-003-opcodes-and-output-bad-pixels-warp-tone-curve-develop-oracle.md`
- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-019-tone-curve-and-the-three-output-modes.md`
  — read it in full; the Acceptance Criteria and Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, the `+toolchain` traps, runtime globals and gotchas.
  Read it; it exists so a cold agent doesn't rediscover them (DEC-004 rule 5).
- **The code the layer sits on:** `src/develop.rs` — `develop_into` and its
  "Output representation — `DEC-018`" module doc; the `:682` note that calls
  the tone-curve stage a no-op "for now". Your output consumes `develop_into`'s
  scene-linear `u16`; you do **not** change `develop_into`.
- **The pattern to mirror for committed generated data:** `examples/fuzz-seeds.rs`
  + the `fuzz-seeds` recipe in `app.just`. Your `examples/gen-tone-table.rs` +
  `just gen-tone-table` is the same shape.

### Decisions that apply

- `DEC-018` — develop output is scene-linear `u16`, full-scale, round-to-nearest,
  clamped. **You do not reopen it.** Your f32 mode is `v/65535.0`; your sRGB
  modes apply the curve to that scene-linear value. Match DEC-018's
  round-to-nearest when scaling the curve output.
- `DEC-002` — determinism is a design constraint from STAGE-003 on (table-driven,
  no runtime `powf`, no SIMD dispatch, pinned); this stage's output is
  `develop_version` 1 (the field itself is STAGE-004, not yours).
- `DEC-016` — caller-owned buffers: each encoder writes a `dst` the caller owns
  and sized, erroring (never panicking) on a length mismatch.
- `DEC-005` — the develop oracle's pre-registered tolerance (≥85) and why; context
  only — this spec does NOT do end-to-end perceptual scoring (see Out of Scope).
- `DEC-017` / `DEC-021` — the red-proof shapes to reuse: operate on a mutated
  **copy** so the shipped table carries no fault switch (DEC-017 rebuilds a
  mutated plane; DEC-021 is the asymmetric-mutation develop oracle).
- **You will emit `DEC-025`** — the tone-curve/output-encoding model (sRGB OETF
  as the default curve because the file carries no profile curve; three modes;
  `develop_into` left scene-linear). The spec's `## Notes for the Implementer`
  gives its content; write it under `/decisions/` with honest confidence and a
  provenance note, and reference it in the PR. (This repo's `/decisions/`, not
  `/docs/decisions/` — AGENTS.md §10.)

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply to the paths
this task touches:

- `no-panics-on-untrusted-input` — `src/tone.rs` is a library path. No
  `unwrap`/`expect`/out-of-range indexing/`panic!`/overflowing arithmetic. Use
  `.get()`, `iter().zip()`, `try_into()`, `checked_*`. The five `deny` lints are
  on. `just lint-no-allow` must stay green (no `#[allow]` of a policy lint under
  the crate root).
- `provenance-recorded-per-algorithm` — the sRGB OETF is an algorithm: it is
  **not done** until `docs/provenance-ledger.md` has its row (AC8). Source IEC
  61966-2-1, provenance class 1, nothing read from dnglab/rawler.
- `oracle-must-be-shown-red` — the analytic tone oracle ships with its
  deliberate-fault red-proof and a negative control (AC6).
- `library-not-application` — no I/O, no new runtime dependency. `include_bytes!`
  (compile-time embed) is acceptable for the table if you go that route; runtime
  file reads are not.
- `no-copyleft-dependencies` — add no dependency. The curve is arithmetic.

### Prior related work

- `HANDOFF-043`/`HANDOFF-045` (SPEC-020) — the develop oracle harness and the
  `--srgb` reference shape (16-bit sRGB PNM); `tests/perceptual_oracle.rs`'s own
  doc notes "SPEC-019 (tone curve) depends on this spec."
- `HANDOFF-035`/`HANDOFF-036` (SPEC-015) — the analytic levels/geometry oracle;
  your analytic tone oracle (AC5) is the same discipline applied to the curve.
- `docs/oracle-contract.md:22` — "Developed output | `dnglab analyze --srgb` →
  16-bit PNM | SSIMULACRA2 ≥ 85". Confirms the sRGB target your u16 mode matches.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code implementing SPEC-019's AC1–AC10:
  - `src/tone.rs` (the three encoders + sRGB lookup), `src/tone_table.rs` (the
    committed generated const tables), `examples/gen-tone-table.rs` (the
    generator), `tests/tone_oracle.rs` (analytic oracle + red-proof).
  - `src/lib.rs`: `pub mod tone;` + one typed length-mismatch `Error` variant.
  - `app.just`: a `gen-tone-table` recipe **and** its line in AGENTS.md §6's
    command block in the same change (SPEC-001 AC8 correspondence).
  - `src/bin/irr.rs`: `develop --format linear-f32|srgb16|srgb8` (default
    `srgb8`) — the reader that keeps every mode from being unread (AC9).
  - `docs/provenance-ledger.md`: the sRGB OETF row (AC8).
  - `/decisions/DEC-025-*.md`: the tone-curve/output-encoding model.
- All Failing Tests in SPEC-019 passing — and each confirmed to have actually
  *run* (zero-match `cargo test <name>` exits 0; sum across targets).
- A PR against `main` from branch
  `feat/spec-019-tone-curve-and-the-three-output-modes`.
- PR description per AGENTS.md §13: this handoff (HANDOFF-052), SPEC-019,
  STAGE-003, PROJ-001, the referenced `DEC-*` (DEC-018/002/016/005/021/017) and
  the new `DEC-025`, the provenance row added, and which oracle layer this
  touches + its red-proof.
- **Run `just lint-ci` before every push** (PATH-prefixed, pinned toolchain) —
  it is the only local command that sees CI's clippy; and confirm CI is
  observed green on the shipping SHA (AC10, PATCH-001's lesson).

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
the stage's backlog, not expand this handoff. (Full rationale in the spec's
`## Non-Goals`.)

- **Changing `develop_into`** or `DEC-018`'s scene-linear `u16` contract. The
  encode layer is purely additive; no shipped function's decoded output changes.
- **Honouring a per-file `ProfileToneCurve`/`LinearizationTable`** — the Q2M
  carries none (probe in the spec's Context); next wave.
- **Colour / white balance / `ColorMatrix` / demosaic** — monochrome; PROJ-002.
- **Auto-tone, exposure comp, histogram adaptivity** — determinism hazard; a
  STAGE-003 non-goal.
- **End-to-end SSIMULACRA2 scoring of a real render vs `dnglab --srgb`** — that
  is SPEC-020's harness, and `DEC-024`/`SPEC-021` already record the `--srgb`
  reference is uncorrected for warp (a correct warp scores *worse*). This spec's
  correctness check is the **analytic** tone oracle (AC5), not a perceptual
  end-to-end score.
- **The `develop_version` field** — STAGE-004.

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

- **Branch / PR:** `claude/spec-019-tone-curve-13488a` → https://github.com/jysf/irradiance/pull/18
- **Completed at:** 2026-10-26
- **All acceptance criteria met?** Yes — AC1–AC10.
  - **AC1** `to_linear_f32_into` maps `0→0.0`, `65535→1.0`, `v→v/65535.0`; length mismatch → `Error::OutputBufferWrongLength` (never a panic).
  - **AC2/AC3** `encode_srgb16_into`/`encode_srgb8_into`: anchors `0→0`, `65535→65535`/`255`, monotonic non-decreasing, within ±1 of the analytic OETF.
  - **AC4** no `powf`/`exp`/`ln` on the encode path — committed const tables + integer lookup; proven by compile-time const assertions (tables are `const`, not `static`) **and** a comment-stripped source scan of `src/tone.rs`/`src/tone_table.rs`.
  - **AC5** analytic oracle recomputes the OETF in `f64` for all 65536 indices, within ±1, independent of dnglab/rawler; **0 violations** on both tables.
  - **AC6** red-proof: `+40` on a **copy** of a table turns the ±1 oracle red, with the unmutated table the green negative control; the shipped table carries no fault switch (DEC-021's in-process shape; see DEC-025 for why not DEC-017's crate rebuild). Observed both halves in one run.
  - **AC7** committed const data → byte-identical across platforms; anchors pinned; `just gen-tone-table` reproduces with **no diff** (verified after commit).
  - **AC8** provenance row added (`src/tone.rs`+`src/tone_table.rs`, sRGB OETF, IEC 61966-2-1, class 1).
  - **AC9** `irr develop --format linear-f32|srgb16|srgb8` (default `srgb8`) reads every mode — smoke-tested on `L1021223.DNG` (scene-linear max 51764 → srgb8 230 / srgb16 59063 / f32 0.790), bad format exits 1; plus the three encoders' unit tests.
  - **AC10** `just lint-ci` (clippy 0.1.98), `just lint-no-allow`, `just test`, `just deny` + `just deny-fuzz`, `just msrv` (1.90.0) all green locally; CI observed on the shipping SHA (see PR #18).

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 41,313,385
- **Estimated USD:** 105.42
- **Duration (minutes):** 25
- **Source of the number:** harness report — summed from this session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read`
  across all 165 usage records; this desktop Code interface exposes no `/cost`
  slash command). This matches the all-inclusive convention prior specs used
  (SPEC-005 30.1M, SPEC-002 9.5M). `estimated_usd` is computed at per-category
  Opus list rates (in $15 / out $75 / cache-write $18.75 / cache-read $1.50 per
  Mtok) — an order-of-magnitude figure, cache-read-dominated.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-025` — the tone curve is the sRGB OETF; output has three modes; `develop_into` stays scene-linear.
- **Deviations from spec:**
  - One non-policy lint allow added in the generated table: `#[allow(clippy::large_const_arrays)]`. The lint (warn-by-default, so `-D warnings`/`lint-ci` would fail on it) wants `static`, but `const` is what AC4's const-context proof and the spec's stated shape require. It is **not** one of the five panic-policy lints, so `just lint-no-allow` stays green; rationale recorded in DEC-025 and in `examples/gen-tone-table.rs`. No behavioural deviation from the spec.
- **Follow-up work identified:**
  - None ship-blocking. (The end-to-end SSIMULACRA2 score vs `dnglab --srgb` remains out of scope per the spec / DEC-024; correctness here is the analytic oracle.)

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The one thing the spec left to the implementer that had a
   real build consequence was *table visibility*: the spec says the tables are
   `pub(crate)`, but `tests/tone_oracle.rs` is a separate crate and cannot see
   `pub(crate)` items. Reconstructing each table through the public encoder
   (`encode_srgb16_into(&[i], dst)` writes `SRGB_ENCODE_16[i]`) resolved it
   cleanly and actually tests the shipped path. Worth a sentence in the spec.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `clippy::large_const_arrays` interaction with a committed const table
   is the kind of toolchain fact `guidance/toolchain-brief.md` exists for; it is
   not currently noted there. A one-liner ("a committed `const` lookup table over
   ~16 KB trips `large_const_arrays` under `-D warnings`; allow it, don't switch
   to `static`") would save the next table-driven spec a loop. Filing as a
   lesson candidate, not blocking.

3. **If you did this task again, what would you do differently?**
   — Emit `#[rustfmt::skip]` on the tables from the first generator draft (I did),
   but also run `cargo fmt` on the *hand-written* files before the first push —
   CI's `fmt --check` caught three long lines I'd have caught locally one step
   earlier. `just lint-ci` sees CI's clippy but nothing in the recipe set sees
   CI's rustfmt; running `cargo fmt --check` (or a pinned equivalent) belongs in
   the pre-push ritual next to `just lint-ci`.
