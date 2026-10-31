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
  id: HANDOFF-074
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model ACTUALLY running this build session (message.model), NOT tier_map's claude-opus-5 hint
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  tokens_total: null               # /cost is NOT exposed to this desktop build session — see notes
  estimated_usd: null              # derivable only once tokens_total is read from the transcript
  duration_minutes: 30
  branch: claude/spec-030-demosaic-568a93
  pr: null                         # filled when the PR is opened
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: >
    /cost is not available as a tool in this Claude Code desktop build session, so the REAL
    cumulative build-cycle total cannot be self-reported here (not invented, per the contract).
    The orchestrator should read /cost from THIS session's transcript at ship and fill
    cost.sessions directly — anchor on the scratchpad-dir UUID 93026a80-0784-4ed6-9408-46a32be4a9f4,
    not a text grep (memory: identify-own-transcript-for-cost-handback). Cross-check figure: the
    session context window measured 389,365 tokens at hand-off (mcp get_usage) — a floor, not the
    cumulative /cost sum. Model: claude-opus-4-8.
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-074: Demosaic foundation — the camera-space linear RGB

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-030`
to `claude-opus-4-8` (acting as implementer) for the **build** cycle — stand up
the mosaic→RGB demosaic path end to end with the bilinear baseline kernel, its
bit-exact analytic oracle, and the `irr demosaic` surfacing.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-007-demosaic-clean-room-malvar-he-cutler-mosaic-to-camera-linear-rgb.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-030-demosaic-foundation-the-camera-space-linear-rgb.md`
  — its Design / Inputs / Outputs / Acceptance Criteria / Failing Tests / Notes
  are load-bearing; the four RGGB bilinear cases and the exact arithmetic
  (round-half-up, `u32`) live in `## Notes for the Implementer`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework (built-in
  harness), the `+toolchain` traps, `just lint-ci` vs `just lint`, and `irr` as a
  dev binary where `unwrap()` is fine.

### What to build (mirrors the SPEC)

- **`src/demosaic.rs` (new):** `pub fn demosaic_into(sensor: &Sensor, src: &[u16],
  dst: &mut [u16]) -> Result<(), Error>`. `src.len() == width*height` (mosaic),
  `dst.len() == width*height*3` (INTERLEAVED R,G,B), caller-owned, no allocation.
  Mirror `src/plane.rs::unpack_into` (`src/plane.rs:261`) and
  `src/develop.rs::develop_into` (`src/develop.rs:744`) exactly for the
  length-check + typed-error shape.
- **Kernel = bilinear, phase-driven** by `Sensor.color_filter.pattern`
  (`src/ifd.rs:598` `Cfa`) — NOT hard-coded RGGB. Integer averages, round-half-up,
  computed in `u32`: two neighbours `(a+b+1)/2`, four `(a+b+c+d+2)/4`.
- **Border = phase-preserving reflect** (`x<0 → -x`, `x≥w → 2w-2-x`; same y),
  with a degenerate-dimension clamp so 1×1 cannot panic.
- **Typed `Error` (no panic)** on wrong `src`/`dst` length, `color_filter == None`
  (mono), and unsupported CFA (`repeat_dim != [2,2]` / non-RGGB-family).
- **`src/bin/irr.rs`:** new `irr demosaic <file>` → `unpack_into` → `demosaic_into`
  → **P6 PPM** (`"P6\n{w} {h}\n65535\n"` then big-endian `u16` RGB) on stdout, the
  stats on stderr (stdout is the binary image); wire into `USAGE`.
- **`tests/demosaic_oracle.rs` (new):** the Tier-A analytic-fixture bit-exact
  oracle + the perturbed-kernel red-proof (assert mutant OUTPUT ≠ honest, control
  green) + the no-panic + typed-error tests + the `irr demosaic` P6 smoke.

### Decisions that apply

- `DEC-016` (`./decisions/`) — `unpack_into` writes a **caller-owned** buffer and
  allocates nothing; `demosaic_into` mirrors it (both `src` and `dst` are the
  caller's). A wrong length is a typed error, the one precondition checked.
- `DEC-017` (`./decisions/`) — the plane oracle's red-proof **asserts the OUTPUT
  changed** (mutant ≠ honest), never merely that the mutation compiled. SPEC-030's
  red-proof heeds that lesson with a perturbed in-test kernel (the spec's
  sanctioned "injected neighbour swap" alternative to a crate-copy rebuild).
- `DEC-018` (`./decisions/`) — developed output is `u16`, full-scale; the demosaic
  stays in the same `u16` scene-linear representation. Bilinear cannot overshoot
  (average of `[0,65535]` samples), so SPEC-030 needs **no clamp** (SPEC-031's MHC
  gradient term can, and will clamp — recorded so it does not inherit a false
  assumption).
- `DEC-024` (`./decisions/`) — `dnglab` implements no DNG opcodes and emits only a
  full colour render, so it **cannot isolate demosaic**: there is NO dnglab
  comparison here. Correctness is established analytically (bit-exact fixtures),
  exactly as SPEC-015/SPEC-020 did where no comparison oracle exists.

### Constraints that apply

Full text in `./guidance/constraints.yaml`.

- `oracle-must-be-shown-red` — the analytic oracle ships with a deliberate-fault
  (perturbed-kernel) test proving it goes red, plus a green negative control.
- `no-panics-on-untrusted-input` — `src/demosaic.rs` is a library decode path: no
  `unwrap`/`expect`/`panic`/panicking index/overflowing arithmetic. Use `.get()`,
  `checked_*`, `saturating_*`, `try_from` (as `src/develop.rs` does). The five
  `#![deny]` lints hold; `irr` and `#[cfg(test)]` are the only exceptions.
- `library-not-application` — the demosaic is pure (bytes/samples in, samples
  out); all I/O and the PPM encode stay in `src/bin/irr.rs`.
- `provenance-recorded-per-algorithm` — bilinear is textbook (no external source);
  a one-line `docs/provenance-ledger.md` note. MHC's ledger row lands with SPEC-031.

### Prior related work

- `HANDOFF-064`/`HANDOFF-070` — the CFA parse (`Sensor.color_filter`) and the
  bit-exact mosaic plane this demosaic reads.
- `PR #61` (STAGE-007 frame), `PR #62` (SPEC-030 design, merged to `main`).

## Expected Deliverables

- `src/demosaic.rs`, `tests/demosaic_oracle.rs`, `irr demosaic` in `src/bin/irr.rs`,
  the new `Error` variants in `src/lib.rs`, a P6-RGB reader added to
  `tests/support/pnm.rs`, the `src/demosaic` module wired in `src/lib.rs`, and a
  `docs/provenance-ledger.md` bilinear note.
- All SPEC-030 `## Failing Tests` passing; gates green (`just test`, `just lint-ci`,
  `cargo fmt --check`, `just validate`).
- A PR against `main` referencing this handoff, the spec/stage/project, and the DECs.

## Out of Scope

- Malvar-He-Cutler (SPEC-031); white balance / colour matrix / ColorChecker ΔE
  (STAGE-008); levels/black-subtract, crop, orientation (demosaic is level-agnostic
  pure interpolation); non-2×2 / X-Trans CFA (a typed error); the fuzz corpus
  (SPEC-032); any real-image dnglab comparison (DEC-024).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (incl. a real `tokens_total`), fill
`## Handback` + the reflection, set statuses `completed`, open the PR.

---

## Handback

*Filled in by the receiving agent (`claude-opus-4-8`).*

### Execution notes

- **Branch / PR:** `claude/spec-030-demosaic-568a93` → PR TBD (filled on open)
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** yes — see the PR body for the AC-by-AC map.

### Cost self-report

- **Tokens (total):** null — `/cost` is not exposed as a tool in this Claude Code desktop
  build session, so the cumulative total cannot be self-reported (not invented, per the
  contract). Floor cross-check: 389,365 tokens of session context at hand-off (`get_usage`),
  which undercounts the cumulative `/cost` sum.
- **Estimated USD:** null (derivable once tokens_total is read from the transcript).
- **Duration (minutes):** ~30.
- **Source of the number:** the orchestrator reads `/cost` from THIS session's transcript at
  ship and fills `cost.sessions` — anchor on scratchpad UUID
  `93026a80-0784-4ed6-9408-46a32be4a9f4` (memory: `identify-own-transcript-for-cost-handback`).

### Drift and new artifacts

- **New decisions emitted:** none — SPEC-030 builds entirely within DEC-016/017/018/024.
  (The in-test perturbed-kernel red-proof is the spec's own sanctioned alternative,
  not a new decision.)
- **Deviations from spec:** none of substance. Notes for verify:
  - The red-proof uses an **in-test mutant kernel** (a dropped-neighbour
    reimplementation) and asserts `mutant_rgb != honest_rgb` with a green control,
    rather than the DEC-017 crate-copy-rebuild apparatus. The spec explicitly
    offers this ("a mutated `demosaic` per DEC-017, **or** an injected neighbour
    swap"); it keeps the oracle Tier-A and instant, and still proves the oracle can
    go red while heeding DEC-017's "assert the OUTPUT changed" lesson.
  - Source/dst length errors are their own typed variants
    (`DemosaicSourceWrongLength`, `RgbBufferWrongLength`), matching the repo idiom
    of one length error per function (`SourcePlaneWrongLength`, `WarpSourceWrongLength`, …).
  - `irr demosaic` writes the PPM to **stdout** and all stats to **stderr** so the
    binary image is not corrupted by text (cmd_develop prints stats to stdout, but
    its stdout carries no binary payload).
  - Added a general `read_p6_rgb` to `tests/support/pnm.rs` — the existing
    `read_dnglab_srgb` reads dnglab's malformed grayscale P6 only.
- **Follow-up work identified:** none new beyond the stage's existing SPEC-031 (MHC)
  / SPEC-032 (fuzz) backlog.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The spec's "reuse `tests/support/pnm.rs`" needed a judgement
   call: the existing reader is dnglab-grayscale-specific, so the genuine-P6-RGB
   reader is a new sibling function in that module, not a reuse of the existing one.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-016/017/018/024 + the two named constraints covered it exactly.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The phase-driven kernel (classify each site via the
   pattern + a same-row neighbour-colour lookup) reproduces the spec's four RGGB
   formulas while passing the BGGR fixture with identical code; writing the fixtures
   with integer-only averages first, then one half-hitting fixture, kept the
   hand-computed tables easy to verify.
