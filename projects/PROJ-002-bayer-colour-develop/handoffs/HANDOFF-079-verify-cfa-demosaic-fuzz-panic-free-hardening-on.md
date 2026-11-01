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
  id: HANDOFF-079
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # the model ACTUALLY running this verify (message.model), not the tier hint
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: claude/spec-032-verify-cfbf9c
  pr: null                         # verify PR to main (docs only — build #71 already merged); link stamped when opened
  completed_at: 2026-10-31         # YYYY-MM-DD
  notes: "✅ APPROVED — ship at a7eae9e (already on main; build PR #71 merged early). /cost is NOT exposed to this desktop Claude Code (Code tab) verify session, so tokens_total is null (not invented, AGENTS §13). Recover the real number from THIS verify session's transcript at ship — anchor on the scratchpad-dir UUID 0b206331-7897-4011-ae34-b36b97f66c84, not a text grep (identify-own-transcript-for-cost-handback). The build HANDOFF-078 ALSO carries tokens_total: null (its own UUID 0d68a8ef-a193-413b-8680-dfdf10ad0777), so ship recovers TWO build/verify nulls for SPEC-032. This verify PR is docs-only (timeline [x], ship prompt, advance-cycle stamps) — the build is already on main, so merging it lands no code."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-079: CFA-demosaic fuzz + panic-free hardening + the GRBG/GBRG phase fixture — verify

## Delegation Summary

One sentence: `claude-opus-5` (acting as `architect`) hands `SPEC-032` to
`claude-opus-4-8` (acting as `verifier`) for the **`verify`** cycle — an
independent cold review of the CFA-demosaic fuzz target + the GRBG/GBRG bit-exact
phase fixtures + the panic-free hardening built in `HANDOFF-078` (PR #71, **already
merged to `main`**), reproducing every gate and hand-recomputing the new phase
tables from the Hamilton-Adams definition to prove they are derived, not a kernel
transcript.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-032-cfa-demosaic-fuzz-panic-free-hardening-on.md`
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-078-build-*.md`.
- **Code under review (all test + fuzz, no `src`):** `fuzz/fuzz_targets/demosaic.rs`,
  `fuzz/Cargo.toml`, `examples/fuzz-seeds.rs`, `fuzz/seeds/demosaic/`,
  `tests/demosaic_oracle.rs`. The fuzzed/oracled surface `src/demosaic.rs` is read,
  NOT changed.

### Decisions that apply

- `DEC-003` — no corpus file as a fuzz seed; seeds are hand-built tier-A. Honoured:
  all 5 seeds come from `tests/support/tiff.rs` CFA builders, none from the corpus.
- `DEC-016` — `demosaic_into` allocates nothing; the harness caps `pixel*3`.
  Honoured: the fuzz target bounds `rgb_count = pixel_count*3` against MAX_PIXELS.
- `DEC-017` — oracle red-proof discipline (NOT re-done per spec; the existing RGGB
  perturbation red-proof covers the shared kernel). Confirmed present + live.
- `DEC-027` — the fuzz crate is its own cargo graph, lints clean under default
  clippy + `-D warnings`. Honoured: `just lint-fuzz` exit 0.
- `DEC-031` — the kernel is Hamilton-Adams (patent-clean); unchanged (no `src`).

### Constraints that apply

- `no-panics-on-untrusted-input` — the fuzz target + the tier-A
  `demosaic_is_panic_free_on_hostile_cfa` CI stand-in. Satisfied + INDEPENDENTLY
  RE-RUN (8.6M fuzz runs, 0 crashes; 14-case hostile battery with an Ok control).
- `oracle-must-be-shown-red` — the GRBG/GBRG fixtures are bit-exact, each
  cross-checked against the independent `local_hamilton_adams` reference whose
  perturbation turns the oracle red. Satisfied; I hand-recomputed one interior
  pixel per new phase from the HA definition.

### Prior related work

- `SPEC-030` (PR #65) — bilinear foundation + the Tier-A oracle harness.
- `SPEC-031` (PR #69) — the Hamilton-Adams kernel this hardens.
- `SPEC-027` (PR #54) — the CFA fuzz-seed pattern + `valid_cfa_sensor`.
- `PR #71` (MERGED) — the SPEC-032 build this reviews; my worktree is cut from
  `main`, so the build is present at `HEAD = a7eae9e`. Retrospective review.

## Expected Deliverables

*(verify handoff — see the verdict below.)*

### If `cycle: verify` — the verify contract

- Acceptance criteria met? **Yes — all five.** Tests pass? **Yes — the whole
  suite; demosaic_oracle 23/23.** Reflection honest? **Yes — substantive, not
  mailed-in; it discloses three deviations (both phases not one, richer seeds, a
  test rename) and flags a verify-relevant note (the pixel*3 cap is stricter than
  plane's) up front.**
- Decision drift — `just decisions-audit --changed` → clean.
- Constraint violations — none.
- Behavioral surface exercised — yes: the fuzz target genuinely drives
  `demosaic_into` on the unpacked plane (I re-ran nightly fuzz, 8.6M runs / 0
  crashes); the panic-free test drives `demosaic_into` on 14 hostile cases with an
  Ok control; the GRBG/GBRG tables are hand-verified from the HA arithmetic.
- Verdict: **✅ APPROVED — ship at commit `a7eae9e`.**

## Out of Scope

- Any `src/` change (there is none — confirmed by `git show --stat a7eae9e`).
- The demosaic algorithm, white balance / colour / ΔE / levels — STAGE-008.
- Re-running SPEC-031's per-phase red-proof (spec explicitly scopes this out).
- Non-2×2 / X-Trans CFA; real-image perceptual comparison.
- A CHANGELOG entry — test + fuzz only, no public API change (confirmed: no `src`
  touched, so no new public surface). SPEC-025/027 convention → no CHANGELOG.

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-032-verify-cfbf9c` → verify PR to `main`
  (docs-only: the build already landed on `main` as PR #71, so this PR carries only
  the timeline `[x]`, the ship prompt, and the advance-cycle stamps). Reviewed at
  `HEAD = a7eae9e` (confirmed with `/usr/bin/git`).
- **Completed at:** 2026-10-31
- **All acceptance criteria met?** **Yes — all five SPEC-032 ACs.**
- **Verdict:** **✅ APPROVED — ship at commit `a7eae9e`.**

#### What I reproduced (not taken on trust)

- **TEST + FUZZ ONLY — confirmed.** `git show --stat a7eae9e` touches
  `examples/fuzz-seeds.rs`, `fuzz/Cargo.toml`, `fuzz/fuzz_targets/demosaic.rs`,
  `fuzz/seeds/demosaic/*`, `tests/demosaic_oracle.rs`, and the spec/handoff docs —
  **no `src/`** (`git show a7eae9e -- src/` is empty). AC5's "src unchanged" holds.
- **Gates, all green locally in this worktree at `a7eae9e`:**
  - `just test` — whole suite `ok`; **demosaic_oracle 23/23**; every STAGE-006/007
    regression oracle green; lib unittests + develop/tone/perceptual/warp/opcode/
    ifd/corpus green. The tier-B corpus SKIPs are **loud** (`corpus-status` prints
    each of the 9 missing files by name), not silently green. (The one `ignored`
    test is the pre-existing SPEC-017/SB-1 Q2M finding — unrelated to SPEC-032. The
    SPEC-032 oracle is Tier-A — "no corpus, no tools" — and runs fully.)
  - `just lint-ci` — pinned `~/.cargo/bin/cargo +stable clippy`, `-D warnings`,
    all targets/features: **exit 0**.
  - `just lint-fuzz` — `-D warnings` on `fuzz/Cargo.toml` (DEC-027): **exit 0**.
  - `cargo fmt --check` — **exit 0**. `just validate` — exit 0 (36 + 163 artifacts).
  - `just decisions-audit --changed` — clean (no active decision's scope matches).
- **GRBG/GBRG bit-exactness — hand-recomputed independently from the HA definition
  (not read off the kernel):**
  - **GRBG pixel (3,2)** = RED site (y even / x odd → slot 1 → colour 0). Green pass:
    W=125 E=145 WW=115 EE=135(reflect) → lap_h=−20, cl_h=40; N=85 S=185 NN=35 SS=235
    → lap_v=0, cl_v=100 ⇒ horizontal, gh=560 ⇒ **G=rnd4(560)=140**. Blue from
    diagonals (all four neighbours are BLUE sites; I recomputed each pass-1 green:
    g_NW=75, g_NE=95, g_SW=175, g_SE=195): lap_p=lap_n=−10, dp=dn=550, cl_p=130,
    cl_n=90 ⇒ CLn<CLp ⇒ **B=rnd4(550)=138**. R=self=135. Result **(135,140,138)** —
    matches the pinned `px(3,2)` assertion AND `EXPECTED_GRBG[39..42]` exactly.
  - **GBRG pixel (1,2)** = BLUE site (y even / x odd → slot 1 → colour 2). Green:
    W=88 E=104 WW=96(reflect) EE=112 → lap_h=16, cl_h=32; N=56 S=136 NN=16 SS=176 →
    lap_v=0, cl_v=80 ⇒ horizontal, gh=368 ⇒ **G=rnd4(368)=92**. Red from diagonals
    (g_NW=48 recomputed, greens 48/64/128/144): lap_p=lap_n=8, dp=dn=376, cl_p=104,
    cl_n=72 ⇒ CLn<CLp ⇒ **R=rnd4(376)=94**. B=self=96. Result **(94,92,96)** —
    matches the pinned `px(1,2)` assertion AND `EXPECTED_GBRG[33..36]` exactly.
  - Both fixtures genuinely exercise the **main-diagonal green** phase: under GRBG,
    (3,2) is a RED site (R=self); under RGGB the SAME coordinate is a GREEN site —
    so the table is not a copy of RGGB/BGGR, it drives the distinct colour
    assignment. The full-table `assert_eq!(out, EXPECTED_*)` passing means the real
    kernel reproduces all 75 values; the test ALSO asserts the independent
    `local_hamilton_adams` reference reproduces each table, so the table is
    twice-derived. My hand-derivation of the interior pixels from the HA definition
    is the non-circular anchor.
  - **DC-preservation now covers all four phases** — `constant_mosaic_is_preserved_on_all_four_phases`
    loops `[RGGB, BGGR, GRBG, GBRG]` over a constant 4242 mosaic; passes.
- **The fuzz target is not a no-op.** `fuzz/fuzz_targets/demosaic.rs` mirrors
  `develop.rs` exactly: `Container::parse` → `sensor()` → `require_uncompressed` →
  cap `width*height ≤ MAX_PIXELS` → `unpack_into` → **`demosaic_into(&sensor, &plane,
  &mut rgb)`** with `rgb` of `pixel_count*3`, and that product bounded against
  MAX_PIXELS too (DEC-016). A rejected container/sensor/unpack/CFA is a SUCCESS
  (early `return`). `cargo +nightly fuzz list` shows `demosaic`. I re-ran the
  nightly fuzz on the 5 seeds + corpus: **8,642,384 runs in 46s, 0 crashes / 0
  panics** (cov 1208, corp 529 — the corpus GREW, so inputs are reaching the kernel).
  Seeds are byte-reproducible: `cargo run --example fuzz-seeds` regenerates
  `fuzz/seeds/demosaic/` identical to the committed files (`git status` clean after).
- **The panic-free test is real.** `demosaic_is_panic_free_on_hostile_cfa` drives
  `demosaic_into` over 14 hand-built cases — mono plane, non-Bayer + non-2×2
  patterns, odd/zero dims, wrong-length buffers, the full 1×1/2×2 Bayer quad, odd
  rows/columns — and asserts each RETURNS (a panic would abort the test). It is
  **not vacuous**: a 4×4 RGGB Ok control both opens the battery and is re-asserted
  `.is_ok()` at the end, so the suite can't quietly pass all-errors. Several cases
  (3×5, 5×3 GRBG, 1×1 BGGR, 2×2 GBRG, 7×1, 1×7) feed correctly-sized buffers and
  run the real kernel.
- **No regression.** Mono path returns `NotAColorFilterArray`; the typed-error
  tests (`mono_or_absent_cfa`, `an_unsupported_cfa`, `wrong_buffer_lengths`), the
  RGGB/BGGR bit-exact tables, the directionality + classifier-tie fixtures, the
  red-proof (`a_perturbed_hamilton_adams_step_turns_the_oracle_red` +
  `the_honest_control_is_green`), and `irr_demosaic_emits_a_valid_p6_ppm` all stay
  green.

### Cost self-report

- **Tokens (total):** null — `/cost` is not exposed to this desktop Claude Code
  (Code tab) verify session, exactly as for the build (HANDOFF-078). Not invented
  (AGENTS §13). Recover from THIS session's transcript at ship (scratchpad UUID
  `0b206331-7897-4011-ae34-b36b97f66c84`; `identify-own-transcript-for-cost-handback`
  — anchor on the UUID, not a text grep).
- **Estimated USD:** null (follows from tokens_total).
- **Duration (minutes):** ~35 (incl. the cold build + a 46s nightly fuzz run).
- **Source of the number:** none available — see above.

### Drift and new artifacts

- **New decisions emitted:** none (verify cycle; and the build emitted none — this
  is test + fuzz hardening of an already-decided kernel).
- **Deviations from spec:** none introduced at verify. The build's three disclosed
  deviations are all strict improvements and spec-compliant:
  - Added **both** GRBG and GBRG fixtures where the spec asked for "≥1" — makes
    "all four Bayer phases covered" literal.
  - A richer seed set (a real-strip `cfa_dng_16bit` that reaches `demosaic_into`,
    plus both byte orders of `valid_cfa_sensor`) on top of the degenerate/truncated
    CFA the spec named — same builders, DEC-003 intact.
  - Renamed the DC test `..._on_all_channels` → `..._on_all_four_phases` (the
    Failing-Tests name) and extended its loop; no coverage lost.
- **Follow-up work identified:** none new. SPEC-030/FU-1 (the GRBG/GBRG fixture) is
  **closed** by this spec — all four Bayer phases are now fixture-pinned. SPEC-032
  is the last STAGE-007 spec; shipping it completes STAGE-007, so the orchestrator
  runs the STAGE-007 stage-ship (Prompt 1d) next, then STAGE-008 (colour + ΔE).
- **Cost note (do NOT block):** HANDOFF-078's `tokens_total` is `null` because the
  desktop `/cost` dialog is unavailable; the orchestrator recovers it from the
  build transcript at ship (UUID `0d68a8ef-a193-413b-8680-dfdf10ad0777`). This verify
  handback is `null` for the same reason (UUID above). Per the repo convention this
  is not a verify blocker.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec pinned the GRBG/GBRG phase layouts and the derivation recipe
   ("same way EXPECTED_BGGR was built"), and the build pinned one interior pixel per
   new phase inline with its arithmetic — so the review went straight to
   independently re-deriving those pixels from the HA definition and re-running the
   gates, rather than reverse-engineering intent.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-003/016/017/027 (+ 031 for the unchanged kernel) and the two
   constraints were the right and complete set. The build's own note — that the
   fuzz harness caps `pixel*3`, a stricter effective pixel bound than the plane
   target's — is the one thing worth carrying forward, and it is correct and
   intentional (the RGB buffer is 3× the plane; DEC-016).

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Hand-recomputing one interior pixel per NEW phase from the
   algorithm definition (not just re-reading the pinned arithmetic) is what turns
   "the new tables pass" into "the new tables are the HA result, independently
   derived" — the right default for a bit-exact phase-coverage verify.
