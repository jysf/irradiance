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
  id: HANDOFF-081
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  duration_minutes: 50
  branch: claude/spec-033-verify-28f03a
  pr: 80                           # verify PR against main (docs-only; code landed via #79)
  completed_at: 2026-11-02         # YYYY-MM-DD
  notes: >
    ✅ APPROVED at commit bb4aab11 (PR #79's build; byte-identical at worktree HEAD
    d05ace4 — the only later commit, #78, is docs-only). tokens_total is null because
    this is a desktop Claude Code (app) session: `/cost` is a terminal-dialog command
    the app session cannot run — the recurring `desktop-cost-tool-unavailable-handbacks-null`
    gap (same as the SPEC-030/031/032 builds+verifies, and the SPEC-033 build). The
    orchestrator must recover the REAL count from THIS verify session's transcript at
    ship — anchor on scratchpad UUID 790c21da-873f-437d-9be9-d67eb55a46dd (model
    claude-opus-4-8). Do NOT invent a number. (`get_usage` reports only the
    context-window snapshot, the wrong metric, so it is deliberately not used.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-081: Verify — Colour-transform core + first colour develop (WB + colour matrix → linear-sRGB)

## Delegation Summary

The orchestrator (`claude-opus-5`, architect) hands `SPEC-033` to a cold verifier
(`claude-opus-4-8`) for the **verify** cycle: review the SPEC-033 build (on `main`,
PR #79 @ `bb4aab1`) without having written it — independently re-derive the bit-exact
oracle, prove the red-proof bites against the real kernel, audit `DEC-033` adherence
and the panic-free contract, run the gates cold, exercise `irr color` end-to-end, and
return exactly one verdict.

## Context the Receiving Agent Needed

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-033-colour-transform-core-first-colour-develop-wb.md` — the contract (`## Design`, `## Acceptance Criteria`, `## Failing Tests`).
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-080-build-colour-transform-core-first-colour-develop-wb.md`.
- **Build:** `src/color.rs`, `src/lib.rs` (two new `Error` variants), `src/bin/irr.rs` (`irr color`), `tests/color_oracle.rs`, `docs/provenance-ledger.md`, `CHANGELOG.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — `just test` (NOT `--all-features`); the Tier-A oracle needs no corpus; `irr color` on a corpus DNG needs the real images dir.

### Decisions that apply

- `DEC-033` — the colour transform is a committed-constant `f64` pipeline (`+ − × ÷` only, no FMA/transcendental); the audited arithmetic decision.
- `DEC-018` — full-scale `u16 ≡ 1.0` (scale-free apply) + the load-bearing clamp.
- `DEC-025` — the sRGB OETF stays `tone`'s committed table; `color` emits linear-sRGB.
- `DEC-016` — caller-owned `src`/`dst`; internal scratch allowed.
- `DEC-017` / `oracle-must-be-shown-red` — the red-proof must mutate a REAL coefficient and change the OUTPUT.
- `no-panics-on-untrusted-input` — den-0 rationals, singular `ColorMatrix`, missing metadata, off-gamut mixes, wrong lengths are typed errors / clamped.

### Prior related work

- `HANDOFF-080` — the SPEC-033 build (claims: built whole, all AC met, gates green).
- `PR #79` — the build, merged to `main` @ `bb4aab1`.

## Expected Deliverables (verify contract)

Review cold; re-derive the oracle; prove the red-proof bites; audit DEC-033 + panic
policy; run the gates; exercise the behavioural surface; return ONE verdict.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-033-verify-28f03a` → verify PR against `main` (docs-only; the code already landed via #79).
- **Completed at:** 2026-11-02
- **All acceptance criteria met?** **yes — every one, independently confirmed (not merely re-running the build's own tests).**
- **For `verify`:** **✅ APPROVED at commit `bb4aab11`.** (The reviewed colour code — `src/color.rs`, `tests/color_oracle.rs`, the `src/lib.rs` variants, `src/bin/irr.rs` — is byte-identical at `bb4aab1` and at worktree HEAD `d05ace4`; the only intervening commit, `#78`, touches `docs/` only.)

#### What I verified, and how (the cold-read discipline, not a re-run)

1. **Independently re-derived the bit-exact oracle.** I wrote my own reference in
   Python (IEEE-754 double): transcribed the sRGB (IEC 61966-2-1) and Bradford
   (D50→D65) constants from the published standards myself, built `M = C·cam2xyz·D`
   from the spec's own formula, and replicated the exact `f64` operation order
   (`mat_mul`'s left-to-right dot; `clamp(0,65535)` then `(v+0.5).floor()`). On the
   FM-path fixture, the inv(CM)-path fixture, the neutral→grey fixture, the off-gamut
   fixture, and the red-proof camera, **my re-derivation reproduced the real Rust
   kernel's output to the bit** (e.g. FM `[32768,32768,32768, 65535,0,24811, …]`;
   CM `[33632,31812,40901, …]`). This is a genuine second source — I did not read
   `src/color.rs`'s private helpers to produce it.
2. **Proved the red-proof bites by mutating the REAL kernel ≥2 independent ways:**
   - **(a) perturbed committed `S[0][0]`** `3.2404542→3.2504542` → **6 oracle tests
     RED**, incl. neutral→grey (`[33079,32768,32768]`).
   - **(b) removed the WB `1/asn` inversion** (`reciprocal_rational` `den/num →
     num/den`) → **7 oracle tests RED**, incl. `the_honest_control_is_green` and
     neutral→grey.
   - The honest (reverted) tree is green — the negative control. And the oracle's own
     in-file red-proof perturbs a real `FM1[0][0]` coefficient and I confirmed (both
     in Rust and my Python) that it CHANGES the output (`mutant != honest`), so it is
     not a no-op (the `plane_oracle.rs`/`DEC-017` lesson is honoured).
3. **Audited `DEC-033` adherence** (grep + read): the transform path is `f64`,
   `+ − × ÷` only — **no `mul_add`/FMA, no `powf`/`exp`/`ln`/`sqrt`**; the only `pow`
   is the integer `2u32.checked_pow(bits)` white-level default, off the float path.
   The OETF is **not** recomputed in `color` (it stays `tone`'s table; `irr color`
   calls `tone::encode_srgb16_into` separately). `M` is built **once** in
   `to_srgb_linear_into` before the pixel loop; the apply is scale-free (no `/65535`
   per pixel). The clamp is **before** the round in `quantize`.
4. **Panic-free (the escape-prone class):** den-0 rationals (`rational_to_f64`),
   non-positive / den-0 neutrals (`reciprocal_rational`), a singular `ColorMatrix`
   (`inv` returns `None` → `MissingColorCalibration`), absent metadata, off-gamut
   mixes (clamped), and wrong `camera_rgb`/mosaic/`dst` lengths are all typed errors /
   clamped. `just lint-ci` (`-D warnings`, the panic-free lint set) is green, so this
   is enforced mechanically, not just by eye.
5. **Pipeline order + neutral→grey:** `develop_color_into` normalizes the **mosaic
   before** `demosaic_into` (per-channel black-subtract doesn't commute with
   interpolation); the neutral fixture yields `R==G==B` on every pixel, interior
   (`[32768,32768,32768] … [2000,2000,2000]`), confirmed bit-exact by my reference.
6. **Behavioural surface (AGENTS.md §12):** ran `irr color` end-to-end on **both**
   real Bayer frames — Canon 5D-III (FM1 path, 5920×3950) and Nikon D300S (inv(CM1)
   path, 4352×2868). Both exit 0 and emit a well-formed `P6 … 65535` PPM of exactly
   `w·h·3·2 + header` bytes, with **genuine non-degenerate colour** (mid-region
   triples R≠G≠B, thousands of distinct values; min 0 / max 65535). Not just "it
   compiles".
7. **Gates cold:** `just test` (whole suite, incl. STAGE-006/007 oracles + the mono
   `develop_into`) ✅; `just lint-ci` ✅; `cargo fmt --check` ✅; `just validate` ✅
   (39 artefacts); `just decisions-audit --changed main` → no changed files in scope
   (the build is already on `main`, so there is no delta — not a failure); `just msrv`
   (1.90.0) ✅ (the new `slice::as_chunks` usage compiles at MSRV).
8. **Build matches its handback:** built **whole** (no silent scope drop), CHANGELOG
   `[Unreleased] Added` entry present (public API, correctly — unlike the test-only
   025/027/032), provenance-ledger `src/color.rs` row present, both `Error` variants
   + `irr color` present. The `dnglab` cross-check was (correctly) deferred as loose
   Tier-B, not masquerading as an accuracy gate.

### Cost self-report

- **Tokens (total):** **null** — this is a desktop Claude Code (app) session; `/cost`
  is a terminal-dialog command the app session cannot run (the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap). **Recover the REAL count from
  THIS session's transcript at ship** — anchor on scratchpad UUID
  `790c21da-873f-437d-9be9-d67eb55a46dd` (model `claude-opus-4-8`), NOT a text grep
  (memory: `identify-own-transcript-for-cost-handback`). Do NOT invent a number.
- **Estimated USD:** null (follows from tokens_total).
- **Duration (minutes):** ~50.
- **Source of the number:** none available (recover from transcript).

### Drift and new artifacts

- **New decisions emitted:** None. `DEC-033` was minted at design and implemented
  unchanged; no supersession.
- **Deviations from spec:** None material. The build's one judgment call — indexing
  per-channel `black_levels`/`white_levels` by 2×2 mosaic position `(y&1,x&1)` (the
  DNG repeat-block layout), scalar fallback — is a faithful reading of the spec's
  "per CFA colour" for a Bayer plane; the compose oracle's independent
  `reference_normalize` pins it.
- **Follow-up work identified:**
  - **Non-blocking observation (test strength, not a defect):** removing the explicit
    `value.clamp(0.0, 65535.0)` from `quantize` is a **no-op** — the whole suite
    (incl. `quantize_clamps_before_rounding`) stays green, because Rust's `f64 as u16`
    cast already saturates (negative→0, >65535→65535). The code is correct and the
    clamp acceptance criterion holds either way; the clamp is still worth keeping
    (explicit intent, robust to a future cast change, matches DEC-018). But that one
    unit test cannot distinguish "clamp present" from "clamp absent". Optional
    hardening: assert the clamp on the float path, or comment that the saturating cast
    is the real backstop. Not worth its own spec — a `signals.yaml` `lesson`
    (`saturating-as-cast-masks-explicit-clamp-in-quantize-tests`) at most.
  - **The Tier-B `dnglab`-render cross-check** stays deferred (the build's own
    follow-up); it is NOT an accuracy gate (that is SPEC-035's ΔE) and did not block.
  - **SPEC-035 (ColorChecker ΔE)** remains blocked on the maintainer's ColorChecker
    shot — the only thing that judges whether the single-illuminant Nikon render is
    colorimetrically *right* (DEC-033's honest cost; the bit-exact oracle does not and
    cannot catch that, by design).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing slowed the review. The spec's `## Design` gave the exact `M = C·cam2xyz·D`
   composition and the committed `S`/`B` values, so re-deriving the oracle by hand was
   transcription, not discovery. The one thing I had to reason through myself was
   whether "skip the clamp" should go red — it doesn't, because of Rust's saturating
   cast; the spec calls the clamp "load-bearing" which is true semantically but not
   numerically on this target. Worth a line in the spec for the next colour spec.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The references (`DEC-033/018/025/016/017`, the two constraints) were
   complete and exactly the ones the review needed. The build's note that
   `clippy::arithmetic_side_effects` doesn't fire on `f64` `+ − × ÷` matches what I
   saw — the matrix math is lint-clean with no `#[allow]` on the library path.

3. **If you did this task again, what would you do differently?**
   — I'd reach for the saturating-cast reasoning on the clamp mutation *before*
   running it, rather than discovering it empirically — though running it is the right
   confirmation and I'd still run it. Everything else (independent Python re-derivation
   first, then two real-kernel mutations, then the end-to-end `irr color` behavioural
   check) was the right order and I'd keep it.
