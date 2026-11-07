---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-049
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L              # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-103 (build); tier_map.build
  created_at: 2026-11-07

references:
  decisions: [DEC-042, DEC-038, DEC-016, DEC-018] # DEC-042 = the fuse + f32 entry point (this spec's decision); DEC-038 = the colour develop path; DEC-016 = caller-owned buffers (the f32 entry point extends this); DEC-018 = clamp/modular integer discipline.
  constraints: []                  # design fills exact ids: no-panic, determinism, no-new-deps
  related_specs: [SPEC-047, SPEC-048] # same colour path

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-048]             # builds on the demosaic fast path (same develop_color_into flow)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-012's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-012's memory + float goals: a leaner colour develop and the f32 entry point the Python/ML callers want, output bit-identical."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 18000000
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 40
      recorded_at: 2026-11-07
      notes: "Main-session design (un-metered). Formalized DEC-042 (fuse to two passes + a public develop_color_f32_into that quantises then divides), wrote the byte-identity + f32-equality failing tests, HANDOFF-103 and the build prompt. The design's normalise-on-read call was wrong (see build) — the rolling-row cache it named as the fallback is what shipped."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 49237604
      estimated_usd: 106.77
      duration_minutes: 110
      recorded_at: 2026-11-07
      notes: "ONE session, TWO build turns: #147 (the fuse, normalise-on-read) then #148 (the rolling-row cache revision). #147's normalise-on-read measured ~2.76x slower develop (it re-normalised each sample ~9x and bought no memory over a rolling cache); the ledger discipline surfaced it, the maintainer chose the fix, and #148's rolling cache kept the ~134 MB win at ~1.04x (near baseline), bit-identical. Recovered 49,237,604 tok / $106.77 (claude-opus-4-8, transcript 160a4ff6, 119 deduped messages) across both turns — ~3.3x the 18M estimate, the cost of the wrong first approach + rework."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 10460829
      estimated_usd: 23.18
      duration_minutes: 45
      recorded_at: 2026-11-07
      notes: "Verdict APPROVED at main 212eb98 (HANDOFF-104, PR #149). Cold review of the rolling-cache state: bit-identity proven against an independent reference and confirmed by two adversarial mutations watched go red; demosaic kernel untouched; the ~134 MiB win is structural (green plane + ~91 KB ring); f32 equals the quantised u16 / 65535. Recovered 10,460,829 tok / $23.18 (claude-opus-4-8, transcript d3910903, 59 deduped messages); handback floor superseded."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 30
      recorded_at: 2026-11-07
      notes: "Orchestrator ship bookkeeping (un-metered main session). The build PR #147 was merged directly (skipping verify + ship), so this cycle caught up: recovered the true build+verify costs from transcripts, wrote the reflection and totals, archived the spec, and flipped the STAGE-012 backlog."
  totals:
    tokens_total: 59698433
    estimated_usd: 129.95
    session_count: 4
shipped_at: 2026-11-07
---

# SPEC-049: colour-develop memory fuse and f32 entry point

## Context

The colour develop path holds more memory than it needs. For one 23.4 MP
develop, `develop_color_into` allocates two scratch buffers that are live at the
same time (the pyirradiance benchmark): a `normalized` mosaic (`w*h` `u16`,
~47 MB) and a `camera_rgb` intermediate (`w*h*3` `u16`, ~140 MB), on top of the
caller's `src` plane and `dst`. Peak on the corrupt-file benchmark was 993 MB
against LibRaw's 488 MB. Separately, every float consumer (the Python bindings,
ML dataloaders) currently has to hold the `u16` result **and** its `f32` copy at
once, because the only float path is `to_linear_f32_into(develop_color_into(..))`
as two passes. Third spec of `STAGE-012`'s performance work (after P1, SPEC-048);
the pyirradiance report's P2.

## Goal

Two output-preserving changes to the colour path: (1) **fuse the scratch passes**
— level-normalise as samples are read and write final linear sRGB directly into
`dst`, keeping only a `w*h` green plane instead of the full `camera_rgb`, which
removes both scratch buffers (~140 MB saved at 23 MP); (2) add a public
**`develop_color_f32_into`** entry point that writes `f32` directly, equal to
`to_linear_f32_into(develop_color_into(..))` exactly. The `u16` output stays
**bit-identical** (same integer ops, same order).

## Inputs

- **Files to read:** `src/color.rs` (`develop_color_into`, `normalize_mosaic`,
  `to_srgb_linear_into`, `apply_matrix_into`, and `to_linear_f32_into`),
  `src/demosaic.rs` (`demosaic_into` — pass 1 writes green; the fuse keeps green
  in a `w*h` plane), `src/lib.rs` (the public API surface, `DEVELOP_VERSION`).
- **Reference only:** the pyirradiance benchmark (`bench/README.md` and the
  report) for the measured buffer sizes and peak RSS.
- **Prior decisions:** `DEC-038` (the colour path), `DEC-016` (caller-owned
  buffers — the f32 entry point extends it), `DEC-018` (clamp/integer discipline).

## Outputs

- **Files modified:** `src/color.rs` (fuse normalise-on-read + a camera-RGB-free
  pass-2 that applies the 3×3 matrix straight into `dst`; a `w*h` green plane
  replaces `camera_rgb`), possibly `src/demosaic.rs` (green-into-a-plane), and
  `src/lib.rs` (export `develop_color_f32_into`).
- **New public function:** `develop_color_f32_into(sensor: &Sensor, src: &[u16],
  dst: &mut [f32]) -> Result<(), Error>` — additive (semver-minor); its values
  equal `to_linear_f32_into(develop_color_into(..))` exactly.
- **New decision:** `DEC-042` — the fuse structure and the f32 entry point, under
  the STAGE-012 perf discipline (`DEC-041`). Written at this spec's **design**.
- **Provenance ledger:** a before/after **peak-RSS** row (and the f32 path noted).

## Acceptance Criteria

- [ ] After the fuse, `develop_color_into`'s `u16` output is **bit-identical** to
  the pre-change implementation (the existing colour goldens / oracle unchanged)
  — same integer ops in the same order.
- [ ] The `normalized` and `camera_rgb` scratch buffers are gone; a measured
  **peak-RSS before/after** for a representative develop is in
  `docs/provenance-ledger.md` (target ~140 MB lower at 23 MP).
- [ ] `develop_color_f32_into` exists and its output **equals**
  `to_linear_f32_into(develop_color_into(..))` exactly (bit-for-bit `f32`), for
  every Bayer phase.
- [ ] Panic-free; determinism held; no new dependency. The new function is the
  only public-surface change (additive).

## Failing Tests

*Sketched at frame; the **design** cycle writes them in full (and formalises
`DEC-042`).*

- **`tests/color_oracle.rs`** (and the existing colour goldens)
  - `"fused_develop_color_is_byte_identical_to_the_reference"` — the fused path
    equals the pre-fuse output byte-for-byte (the `dst_after == dst_before` proof).
  - `"develop_color_f32_equals_to_linear_of_u16"` — `develop_color_f32_into(..)`
    equals `to_linear_f32_into(develop_color_into(..))` exactly, all four phases.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- Any change to the `u16` colour output, the colour maths (`DEC-038`), or the
  demosaic kernel — bit-identity is the gate.
- The colour-geometry crop follow-up (§3B) and P3 threading — separate specs.
- Changing `develop_color_into`'s existing signature — the f32 path is a **new**
  function, not a breaking change.
- SIMD / `unsafe` / new dependencies.

## Notes for the Implementer

- **The shape (DEC-042, at design):** pass 1 computes green into a `w*h` plane
  (not into `dst`), normalising each sample on read; pass 2 interpolates red/blue
  and applies the 3×3 colour matrix per pixel, writing the final linear sRGB
  straight into `dst` — so neither `normalized` nor `camera_rgb` is allocated. The
  arithmetic and its order must match today's path exactly (bit-identical).
- **`develop_color_f32_into`** shares that fused pipeline, but pass 2 writes `f32`
  into `dst` so a float caller never holds a `u16` copy. It must equal
  `to_linear_f32_into(develop_color_into(..))` **to the bit** — and that function
  (`src/tone.rs`) divides the **quantised `u16`** by `65535.0`. So the f32 path
  **quantises to `u16` first (identical to the integer path), then divides by
  `65535.0`**; do NOT divide the raw `f64` matrix result, which would differ by
  the clamp-and-round (`quantize`, `DEC-018`). The equality test is the gate.
- This is a bigger restructuring than P1 — keep the STAGE-012 perf discipline
  (bit-identical proof + measured peak-RSS + the `DEC-*`). Keep zero-dependency,
  no-panic, determinism. LibRaw/rawpy are the pyirradiance yardstick — reference
  only, never linked.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Reach for the rolling-row cache at **design**, not after a measured
   regression. The design chose normalise-on-read; it re-normalised each sample
   ~9× and, per the ledger, bought no memory over a rolling cache — so it only
   added a 2.76× develop slowdown. DEC-042 had even named the rolling cache as the
   fallback. The ledger's before/after discipline caught it (the system working),
   but a full extra build turn later — the spec ran ~3.3× its token estimate
   (`complexity_actual: L` vs estimated M). Also: don't merge a perf build PR
   before its verify and a before/after review — the direct #147 merge put a known
   2.76× regression transiently on `main`.

2. **Does any template, constraint, or decision need updating?**
   — No new rule; `DEC-042` was amended to the rolling-row cache and the STAGE-012
   perf discipline (`DEC-041`) did its job. A watch-worthy pattern: a memory
   optimisation that removes a buffer by recomputing a value on every read can
   regress time badly — prefer a rolling/windowed cache that recomputes once. It
   is captured in DEC-042's narrative; worth a `signals.yaml` lesson if it recurs.

3. **Is there a follow-up spec I should write now before I forget?**
   — The remaining `STAGE-012` backlog: the colour-geometry crop follow-up (§3B)
   and P3 (opt-in threading). The one-row-cache follow-up the first build flagged
   is already done — it is this spec's shipped approach (#148).

4. **Where was the worst defect caught?**
   — build
   *(The normalise-on-read time regression — a design-level wrong call — was
   surfaced by the build cycle's own before/after measurement, before verify; the
   fix shipped in the same build session's second turn.)*

5. **What can a user do now that they couldn't before?**
   — Colour-develop a 23 MP frame with ~134 MB less peak memory (358.7 → 224.9
   MiB) at no speed cost, and call `develop_color_f32_into` for a direct `f32`
   result — before, a float caller had to hold the `u16` result and its `f32`
   copy at once.
