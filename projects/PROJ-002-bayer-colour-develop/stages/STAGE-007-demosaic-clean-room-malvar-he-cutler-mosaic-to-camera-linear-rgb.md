---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-007                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-10-31
shipped_at: 2026-11-01

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The demosaic half of PROJ-002's thesis — turning STAGE-006's bit-exact Bayer
    mosaic plane into three camera-space colour channels, the interpolation step
    every colour image needs. It is the one colour step whose math is exact (a
    fixed linear filter), so it is verified bit-exact — no tolerance — before
    STAGE-008's genuinely tolerance-based colour transform. No colour is corrected
    here; the output is demosaiced camera-space RGB, the trusted input STAGE-008
    turns into a recognisable picture.
  delivers:
    - "A Bayer mosaic (STAGE-006's unpacked plane, driven by the parsed `Cfa` pattern) becomes a 3-channel camera-space linear RGB image via a clean-room Malvar-He-Cutler demosaic"
    - "The demosaic is verified BIT-EXACT against hand-computed output on synthetic analytic fixtures, with a perturbed-kernel red-proof — not a tolerance against a foreign demosaic"
    - "`irr` emits the demosaiced camera-space RGB of a corpus Bayer DNG — the first MULTI-CHANNEL output irradiance produces (honestly uncorrected)"
  explicitly_does_not:
    - "White balance (`AsShotNeutral`), colour-matrix / `ForwardMatrix` transform, illuminant handling, display colour encode — all STAGE-008"
    - "ColorChecker ΔE or any real-image perceptual oracle — STAGE-008 (dnglab emits only a full colour render, so a real-image demosaic-only comparison would be theatre)"
    - "X-Trans / non-Bayer CFA demosaic (the Fuji x100f's 6×6 DHT) — a later stage"
    - "Highlight reconstruction, denoise, edge-aware refinement beyond MHC's own gradient-corrected linear step"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions:
    - tokens_total: 122077684
      estimated_usd: 249.64
      recorded_at: 2026-11-01
      notes: "Stage grain (DEC-013 §5). ONE orchestrator session (work-status-next-steps, scratchpad uuid c0b7ec9b-2be4-41ad-9dea-77f3b6690343) ran STAGE-007 end to end in the main session: framing + the MHC→Hamilton-Adams patent pivot (deep-dive + DEC-031) + the design of all three specs + their 3 ships + dispatching/reconciling 6 cold build/verify sessions + this close. Measured FLOOR from that session's OWN transcript (scratchpad-uuid anchored, NOT text grep), deduped by message.id, all claude-opus-4-8, 98.0% cache-read: 195 usage-bearing messages dated >= 2026-10-31T02:59:41Z (the 'Frame STAGE-007' turn) -> in 390 / out 439,413 / cache-write 1,983,941 / cache-read 119,653,940 = 122,077,684 tokens, priced per-component at Opus list rates (15/75/18.75-write/1.50-read per MTok) = 249.64 USD. SCOPED: STAGE-006's entry already claimed the session through its close; the STAGE-006-close turn between that measurement and this boundary is STAGE-006's, excluded here. Floor: the turns writing this close post-date the measurement. RATIO: 122.1M orchestration against 77.3M delegated spec cost (SPEC-030/031/032 grand totals) = ~61% orchestration — high because all three bit-exact demosaic DESIGNS + the patent pivot were main-session work, while the builds reused SPEC-030's oracle harness cheaply (SPEC-031 build 14.3M, SPEC-032 build 12.7M)."
---

# STAGE-007: Demosaic: clean-room Malvar-He-Cutler, mosaic to camera-linear RGB

> **⚠ Algorithm changed to Hamilton-Adams — see DEC-031.** The quality demosaic is
> **Adams-Hamilton** (US 5,629,734, expired), NOT Malvar-He-Cutler: MHC is
> patent-encumbered (US 7,502,505, active to ~2027-01-25), which defeats the
> permissive-crate thesis (clean-room beats copyright, not patents). SPEC-030's
> shipped bilinear baseline + the bit-exact oracle are unchanged; SPEC-031 is
> Hamilton-Adams. References to "MHC" / "Malvar-He-Cutler" below (including this
> file's slug) are superseded by DEC-031 and get rewritten at the STAGE-007
> stage-ship.

## What This Stage Is

When this stage ships, `irradiance` turns a Bayer mosaic into colour — the
interpolation step, and nothing else. STAGE-006 made the mosaic plane bit-exact
and the CFA pattern truthful; STAGE-007 runs a clean-room **Malvar-He-Cutler**
demosaic over that plane to produce a 3-channel, **camera-space, scene-linear
RGB** image. No white balance, no colour matrix, no display encode — those are
STAGE-008. Because MHC is a fixed set of linear 5×5 FIR kernels, its output on a
synthetic mosaic is **exactly computable**, so the oracle is **bit-exact on
analytic fixtures** (not a tolerance against a foreign demosaic), with a
perturbed-kernel red-proof. The deliverable is the first multi-channel image
irradiance produces — demosaiced but colour-uncorrected (so it reads greenish and
flat) — and the trusted RGB input STAGE-008's colour transform turns into a
recognisable photo.

## Why Now

**STAGE-006 just shipped the trusted input.** The mosaic unpacks bit-exact and
the `Cfa` pattern is parsed into `Sensor`. Demosaic is the only step between a
verified mosaic and colour; it is the natural next move and the project brief's
stage plan has it next.

**It is the one colour step with no tolerance risk.** MHC is linear and fixed, so
its output is exactly computable on fixtures — the same "verify where the math is
exact" discipline that carried STAGE-006, applied once more before STAGE-008's
genuinely tolerance-based ColorChecker ΔE. Doing demosaic first isolates the step
that CAN be bit-exact from the step that cannot, so each stage's oracle is honest
about which it is.

**It unblocks the first colour picture.** STAGE-007 + STAGE-008 together produce
the first recognisable colour image; neither alone does. Landing demosaic now
leaves STAGE-008 as purely the colour transform — white balance, matrix, encode —
with a trusted RGB buffer already in hand.

## Success Criteria

- A clean-room Malvar-He-Cutler demosaic turns a Bayer mosaic (RGGB, both corpus
  files, driven by the parsed `Cfa` pattern) into a 3-channel **camera-space
  linear RGB** image, with a **documented, deterministic border policy**
- On synthetic analytic fixtures the demosaiced RGB equals the hand-computed MHC
  output **exactly** (bit-exact, no tolerance), and the oracle demonstrably goes
  **red** on a perturbed kernel coefficient
- The MHC algorithm carries a **provenance-ledger entry** naming its source
  (Malvar / He / Cutler 2004) and its published-reference status
- `irr` emits the demosaiced camera-space RGB of a corpus Bayer DNG — the first
  multi-channel output, honestly uncorrected
- The monochrome path and STAGE-006's plane + colour-metadata oracles stay green
  (demosaic is additive; mono never demosaics), and the demosaic never panics on
  degenerate CFA input (tiny / 1×1 / odd-dim-at-phase planes; fuzz corpus extended)

## Scope

### In scope
- **Clean-room MHC demosaic kernel:** the gradient-corrected 5×5 linear
  interpolation, RGGB Bayer mosaic → 3-channel camera-space linear RGB,
  implemented from the published algorithm (no copyleft source read).
- **The 3-channel camera-space RGB buffer type** — a new representation (a 3-plane
  or interleaved `u16` buffer, scene-linear per DEC-018, MHC overshoot clamped to
  the level range). It lands **with** its first producer (the demosaic) to satisfy
  the unread-field rule (AGENTS.md §11); STAGE-008's colour transform consumes it.
- **A documented border policy** for the 5×5 kernel running off the edge
  (reflect / mirror), baked into the analytic fixtures so edges are verified.
- **The analytic-fixture bit-exact oracle** + perturbed-kernel red-proof.
- **`irr` surfacing** — emit the demosaiced camera-space RGB (e.g. a PPM) for the
  demo and for a panic-free pass over the CFA demosaic path.
- **Provenance-ledger entry** for the MHC algorithm.

### Explicitly out of scope
- **White balance, colour-matrix / `ForwardMatrix` transform, illuminant,
  display colour encode, ColorChecker ΔE** — all STAGE-008.
- **Any real-image perceptual / tolerance comparison vs `dnglab`** — deferred to
  STAGE-008's full-render ΔE; `dnglab` cannot isolate demosaic (it emits only a
  full colour render, and implements no DNG opcodes — DEC-024).
- **X-Trans / non-Bayer CFA** (Fuji x100f, 6×6 DHT) — a later stage.
- **Highlight reconstruction, denoise, edge-aware refinement** beyond MHC itself.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-007` promotes the `(not yet written)` lines below into
real specs at `cycle: frame` with stable IDs; the `[S]`/`[M]`/`[L]` tag is carried
onto each as `task.complexity`.

- [x] SPEC-030 (shipped on 2026-10-31) — [M] Demosaic foundation: the camera-space linear RGB
      buffer type + a **bilinear** baseline demosaic (exactly computable, the
      simplest red-provable kernel) + the analytic-fixture **bit-exact oracle
      harness** (+ perturbed-kernel red-proof) + border policy + `irr` RGB
      surfacing. Stands up the whole mosaic→RGB path end-to-end before the hard
      kernel lands.
- [x] SPEC-031 (shipped on 2026-10-31) — [M] **Hamilton-Adams** directional demosaic (NOT MHC — patent, DEC-031)
      replacing the bilinear baseline: directional green + colour-difference red/blue;
      HA analytic fixtures + directionality + DC-preservation + red-proof; HA
      provenance entry. ⚠ **L-risk** — if full HA (green + R/B) exceeds [M], ship the
      **green** interpolation first (R/B stay bilinear) and the **red/blue**
      colour-difference as a follow-on spec.
- [x] SPEC-032 (shipped on 2026-10-31) — [S] CFA-demosaic **fuzz + panic-free hardening** + the
      **GRBG/GBRG phase fixture** (closes SPEC-030/FU-1): a `demosaic` fuzz target
      over degenerate / edge-phase planes (tiny, 1×1, odd-dim at the CFA phase) +
      all-four-phase bit-exact oracle coverage. Test + fuzz only, no `src` change.

**Count:** 3 shipped / 0 active / 0 pending

> **SPEC-030/FU-1 (✅ CLOSED by SPEC-032, 2026-11-01; raised at SPEC-030 verify):** SPEC-030's demosaic oracle
> fixture-verifies RGGB + BGGR only (both anti-diagonal green phases); add a **GRBG
> or GBRG** fixture — in SPEC-031's MHC fixtures or SPEC-032's edge-phase hardening —
> to close the one unverified Bayer phase. The kernel is phase-driven and correct for
> all four phases by construction; this is fixture coverage, not a bug.

> **Framing notes for review (Prompt 1c steps 3–4):**
> - **Lean but not thin.** 3 specs is the brief's floor; the stage is one
>   algorithm + its oracle + its hardening, so this is honest, not padded. It
>   becomes 4 if MHC splits (below).
> - **The one L-risk is the MHC spec.** The split is pre-named (green vs red/blue)
>   so the build can take it without a re-frame.
> - **Order may flip.** The oracle and the kernel share the MHC coefficient
>   definition; the build may prefer to write the analytic fixtures first (failing)
>   and make them pass. Either way the fixtures and the kernel cite one source of
>   truth for the coefficients.

## Design Notes

- **Why the oracle is bit-exact, not a tolerance.** MHC is a fixed set of linear
  5×5 FIR kernels (Malvar, He & Cutler, ICASSP 2004). Constant kernels + a linear
  operation ⇒ the output on any synthetic mosaic is exactly computable by hand, so
  the oracle asserts **equality** and a perturbed coefficient is a clean red-proof.
  There is **no** bit-exact external reference: `dnglab` emits only a full colour
  render (and implements no opcodes — DEC-024), so an isolated real-image demosaic
  comparison is impossible. The real-image perceptual check belongs to STAGE-008's
  ColorChecker ΔE. *(Maintainer decision, this frame: bit-exact analytic fixtures;
  real-image check deferred to STAGE-008.)*
- **Output is uncorrected camera-space linear RGB.** No WB, no matrix (STAGE-008),
  so a STAGE-007 image reads greenish/flat; the first recognisable photo is the
  STAGE-007+008 milestone. *(Maintainer decision, this frame: ship the uncorrected
  demosaic as the STAGE-007 demo; do not borrow WB forward.)*
- **Buffer representation.** 3-channel `u16`, scene-linear, consistent with
  DEC-018; MHC overshoot/undershoot clamped to the level range. New type, landed
  with its producer (the demosaic) per AGENTS.md §11; STAGE-008 reads it. Whether
  it is `demosaic_into` producing the buffer or a widened `develop_color_into` is a
  spec-level call — the brief allows either.
- **Phase, not hard-coding.** Both corpus DNGs are RGGB; drive the demosaic from
  the parsed `Cfa` pattern (STAGE-006) rather than hard-coding RGGB, corpus-verify
  RGGB, and cover the other three Bayer phases with synthetic fixtures in the
  oracle — an unverified phase path is a [[measurement-over-generalised]] trap.
- **Bilinear-baseline-then-MHC is the recommended build path** and mirrors
  DEC-024's precedent (ship the simple correct thing, then the better kernel):
  bilinear stands up the type + oracle + `irr` + border end-to-end with the
  simplest exactly-computable kernel and a provably-red oracle, then MHC is a
  focused kernel swap against the same harness.
- **Watch [[probe-the-hard-assumption-at-frame-time]] (filed at STAGE-006 close):**
  before speccing, spike that `plane::unpack_into` + the `Cfa` pattern actually
  feed a demosaic on BOTH corpus files (dims, phase, clamp behaviour) — the hardest
  assumption this stage rests on — rather than discovering it mid-design as the
  capstone surprise STAGE-006 had.
- **Provenance-ledger entry** for MHC (source: Malvar/He/Cutler 2004; a published
  algorithm, not copyleft code), per the brief and PROJ-001's provenance discipline.

## Dependencies

### Depends on
- **STAGE-006** (shipped) — the bit-exact mosaic plane (`plane::unpack_into`) and
  the parsed `Cfa` pattern (`Sensor.color_filter`); the demosaic reads both.
- **PROJ-001** (shipped) — the develop representation (DEC-018, `u16` scene-linear),
  the `irr` harness, and the analytic-oracle + red-proof discipline (DEC-004 /
  DEC-017), all reused.
- External: the published **Malvar-He-Cutler** algorithm, implemented clean-room
  from the paper. `dnglab` is **not** a demosaic oracle here (it cannot isolate
  demosaic) — it returns for STAGE-008's ΔE.

### Enables
- **STAGE-008** (colour transform + output) — white balance (`AsShotNeutral`) +
  `ColorMatrix` / `ForwardMatrix` → XYZ / linear-sRGB → display encode, plus the
  ColorChecker ΔE oracle. It consumes STAGE-007's camera-space RGB; together they
  render the first recognisable colour picture.

## Stage-Level Reflection

*Filled 2026-11-01 at ship (SPEC-032 merged; all three specs in specs/done/).
Drafted via Prompt 1d (Stage Ship).*

- **Did we deliver the outcome in "What This Stage Is"?** Yes, in full. A Bayer
  mosaic now demosaics to a 3-channel **camera-space linear RGB** image via a
  **patent-clean Hamilton-Adams** directional demosaic (directional green on the
  lower-gradient axis + colour-difference red/blue), verified **bit-exact** on
  analytic fixtures across **all four Bayer phases** (RGGB/BGGR/GRBG/GBRG), with a
  perturbed-kernel red-proof (independently broken three ways at verify) and a
  fuzz target (11.27M runs, 0 crashes). SPEC-030 (bilinear foundation + the
  analytic-oracle harness + `irr demosaic`) → SPEC-031 (the Hamilton-Adams quality
  kernel, zero-alloc via DEC-032) → SPEC-032 (fuzz + all-phase coverage). No colour
  correction (that is STAGE-008); the monochrome path is byte-for-byte unchanged.
- **How many specs did it actually take?** 3 (SPEC-030/031/032), exactly as framed.
  No spec splits — SPEC-031's pre-named green/RB split was NOT needed (full
  Hamilton-Adams fit as one [M] build). The one real change was SPEC-031's
  **algorithm** (below), not the spec count.
- **How many outlines survived unchanged?** 3 of 3 framed outlines shipped in their
  slots. SPEC-031 kept its slot but its **approach was re-scoped mid-stage**
  (Malvar-He-Cutler → Hamilton-Adams, DEC-031) — the backlog structure held 3/3;
  one spec's algorithm was changed by a mid-design finding.
- **What changed between starting and shipping?** The quality kernel changed from
  **Malvar-He-Cutler to Hamilton-Adams** (DEC-031) after a mid-design patent check
  found MHC encumbered (US 7,502,505, active to ~2027-01-25) — clean-room beats
  copyright, not patents, so a patented demosaic would have defeated the
  permissive-crate thesis.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **Patent-check a named algorithm or dependency at FRAME time.** The brief +
    STAGE-007 named MHC with no patent check; caught only mid-design (SPEC-031),
    forcing a pivot + DEC-031. The stage-frame prompt (Prompt 1c) wants a one-line
    "if the stage names an algorithm/dependency, patent-check it before designing."
    Filed as lesson [[patent-check-named-algorithms-at-frame]] (watch, N=1).
  - **A reusable bit-exact oracle harness makes kernel swaps cheap and predictable.**
    SPEC-030 built the RGB type + analytic oracle + red-proof + `irr` once; SPEC-031
    (HA) and SPEC-032 (fuzz/phases) reused it wholesale and landed at/under estimate
    (23.6M, 19.5M). Filed as golden-path [[reusable-bit-exact-oracle-harness]]
    (watch, N=2) — capture, don't promote yet.
  - **Desktop Claude Code sessions cannot read `/cost`, so every delegated handback
    returns `tokens_total: null`** and the orchestrator recovers each from the
    session transcript at ship — 6/6 cold sessions this stage. Filed as process-debt
    [[desktop-cost-tool-unavailable-handbacks-null]] (project-close).
- **Signals dispositioned at this close?** (Prompt 1d step 7)
  - [[a-fix-inherits-the-precondition-of-the-thing-it-fixes]] (watch, N=2) →
    **deferred**: no STAGE-007 instance (demosaic was new-capability on a reused
    harness; DEC-032 was a build-time design choice, not a fix hardening failing
    logic). `last_touched` bumped.
  - [[probe-the-hard-assumption-at-frame-time]] (watch, N=1) → **deferred**: no new
    corpus/code-assumption instance (the demosaic built on the corpus as framed).
    The MHC-patent-not-probed-at-frame is adjacent but a different axis — captured as
    the new patent-check lesson instead. `last_touched` bumped.
  - The four codified stage-close lessons ([[unrun-docs-carry-errors]],
    [[attribute-text-inside-doc-comments]], [[measurement-over-generalised]],
    [[a-gate-that-fails-mutely-is-a-gate-that-never-ran]]) are terminal and none
    recurred.
  - [[spec-reflection-shipped-unfilled]] (STAGE-006, project-close) did **not**
    recur — all three STAGE-007 spec reflections were filled at their ships.
  - New this close: lesson [[patent-check-named-algorithms-at-frame]] (N=1 watch),
    golden-path [[reusable-bit-exact-oracle-harness]] (N=2 watch), process-debt
    [[desktop-cost-tool-unavailable-handbacks-null]].
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Yes — the **patent-check** lesson (SPEC-031's reflection) and the
    **harness-reuse golden-path** (SPEC-032's reflection) are both promoted to the
    stage-level signals filed above.
