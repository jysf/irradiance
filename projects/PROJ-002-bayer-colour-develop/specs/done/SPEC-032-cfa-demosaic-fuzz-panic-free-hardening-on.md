---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-032
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-007
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-003, DEC-016, DEC-017, DEC-027]    # no-corpus-seed / no-alloc / red-proof / fuzz-crate lint scope
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-030, SPEC-031, SPEC-027]      # bilinear foundation + HA kernel it hardens; CFA fuzz-seed pattern

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: "STAGE-007's demosaic hardening — a fuzz target + the GRBG/GBRG phase fixtures (closing SPEC-030/FU-1), proving demosaic_into panic-free and phase-driven on all four Bayer phases."

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
  tokens_estimate: 20000000        # prediction (design): an [S] test+fuzz spec, no src change; cf. SPEC-027 (corpus+fuzz, [S])
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-01
      notes: "un-metered main-loop design (orchestrator): the fuzz-target + GRBG/GBRG + panic-free design, in the main session's shared transcript"
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 12690825
      estimated_usd: 28.06
      duration_minutes: null
      recorded_at: 2026-11-01
      notes: "RECOVERED at ship from the build session's OWN transcript (scratchpad uuid 0d68a8ef-a193-413b-8680-dfdf10ad0777, deduped by message.id: 58 records, claude-opus-4-8, 97.7% cache-read). HANDOFF-078 handback carried null (/cost not exposed to a desktop Claude Code session). Priced per-component at Opus list rates."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 6804472
      estimated_usd: 15.84
      duration_minutes: null
      recorded_at: 2026-11-01
      notes: "RECOVERED at ship from the verify session's OWN transcript (scratchpad uuid 0b206331-7897-4011-ae34-b36b97f66c84, deduped by message.id: 43 records, claude-opus-4-8, 97.3% cache-read). HANDOFF-079 handback carried null for the same reason. Priced per-component at Opus list rates."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-01
      notes: "un-metered main-loop ship (orchestrator): cost recovery + reflection + archive in the main session's shared transcript"
  totals:
    tokens_total: 19495297
    estimated_usd: 43.90
    session_count: 4
shipped_at: 2026-10-31
---

# SPEC-032: CFA-demosaic fuzz + panic-free hardening + the GRBG/GBRG phase fixture

## Context

The last STAGE-007 spec. SPEC-030 (bilinear) + SPEC-031 (Hamilton-Adams) built
`demosaic::demosaic_into` and its Tier-A bit-exact oracle. This spec HARDENS that
surface two ways, with no algorithm change:
1. A **fuzz target** that drives `demosaic_into` on arbitrary bytes, making its
   panic-free contract a *fuzzed* guarantee (today only the 1×1 / 2×2 / 3×5 unit
   tests exercise it) — the same discipline `plane` / `develop` already have
   (AGENTS.md §12).
2. Closes **SPEC-030/FU-1**: the oracle's phase fixtures cover RGGB + BGGR (both
   anti-diagonal green phases) but not **GRBG / GBRG** (main-diagonal greens) — the
   one unverified Bayer phase path, the `measurement-over-generalised` gap
   STAGE-007's Design Notes flagged.

`demosaic_into` is already phase-driven (SPEC-031) and panic-free (SPEC-030); this
proves both *on all four phases and under fuzzing*. `depends_on` SPEC-030 + SPEC-031
(both shipped).

## Goal

Add a `demosaic` libFuzzer target (arbitrary bytes → parse → unpack → `demosaic_into`,
never panics) with tier-A seeds, and **GRBG + GBRG bit-exact** analytic fixtures
closing SPEC-030/FU-1 — so the demosaic's panic-free and phase-driven guarantees are
proven on all four Bayer phases and under fuzzing. Test + fuzz only; no `src` change.

## Design

Measured against the fuzz + oracle harness on `main` (2026-11-01):
1. **Fuzz mirrors `develop`.** `fuzz/fuzz_targets/develop.rs` is the template:
   `Container::parse` → `sensor()` → `require_uncompressed` → cap `width*height ≤
   MAX_PIXELS` → `unpack_into` → the entry point. `demosaic.rs` adds
   `demosaic_into(&sensor, &plane, &mut rgb)` with `rgb` of `len*3` (bound `len*3`
   against MAX_PIXELS too). A rejected container / sensor / unpack is a SUCCESS
   (typed error, not panic). Register a `demosaic` bin in `fuzz/Cargo.toml`; add
   `demosaic_seeds()` + a `write_seeds("demosaic", "tiff", …)` call in
   `examples/fuzz-seeds.rs`, writing `fuzz/seeds/demosaic/` (hand-built tier-A —
   reuse SPEC-027's `valid_cfa_sensor` builder + a truncated/degenerate CFA; DEC-003:
   no corpus file as a seed).
2. **Phase fixtures slot into the oracle.** `tests/demosaic_oracle.rs` already has
   `RGGB=[0,1,1,2]`, `BGGR=[2,1,1,0]`, `bayer(pattern)`, `fixture_sensor`,
   `demosaic()`, `EXPECTED_BGGR`. Add `GRBG=[1,0,2,1]`, `GBRG=[1,2,0,1]`; extend the
   DC-preservation loop to all four; add ≥1 **bit-exact** GRBG (and/or GBRG) fixture
   with a hand-computed `EXPECTED_*` table (like `EXPECTED_BGGR`), proving HA on the
   main-diagonal green phases with the same code path.
3. **No `src/demosaic.rs` change expected** — it is already phase-driven + panic-free.
   If the fuzz finds a panic, fix it and flag it at verify; none is expected. Likely
   TEST-ONLY (no public API change → no CHANGELOG, per the repo convention — confirm
   at ship).

## Inputs
- **Read:** `fuzz/fuzz_targets/develop.rs` + `plane.rs` (target template + MAX_PIXELS
  + "rejected = success"), `fuzz/Cargo.toml` (bin registration), `examples/fuzz-seeds.rs`
  (`develop_seeds` / `write_seeds`), `tests/demosaic_oracle.rs` (phase constants +
  `bayer` / `fixture_sensor` / `demosaic` + `EXPECTED_BGGR`), `tests/support/tiff.rs`
  (`valid_cfa_sensor`, SPEC-027), `src/demosaic.rs` (the fuzzed surface).
- **Decisions:** DEC-003 (no corpus as seed), DEC-016 (no-alloc), DEC-027 (fuzz-crate
  lint scope), DEC-017 (red-proof discipline — not re-done here).

## Outputs
- **`fuzz/fuzz_targets/demosaic.rs` (new)** + a `[[bin]] name = "demosaic"` entry in
  **`fuzz/Cargo.toml`**.
- **`examples/fuzz-seeds.rs`:** `demosaic_seeds()` + `write_seeds("demosaic", "tiff",
  demosaic_seeds())`; **`fuzz/seeds/demosaic/`:** generated seed files.
- **`tests/demosaic_oracle.rs`:** `GRBG` / `GBRG` constants; DC-preservation extended
  to all four phases; ≥1 bit-exact GRBG/GBRG `EXPECTED_*` fixture + its test; a tier-A
  `demosaic_is_panic_free_on_hostile_cfa` test (mirroring SPEC-027's panic-free-on-CFA
  test) so CI (no nightly fuzz) still asserts the contract.
- No `src/demosaic.rs` change expected.

## Acceptance Criteria
- [ ] A `demosaic` fuzz target exists, is registered in `fuzz/Cargo.toml`, runs
  (`cargo +nightly fuzz run demosaic …`) with no panic/crash over a bounded run, and
  its seeds are generated by `examples/fuzz-seeds.rs` into `fuzz/seeds/demosaic/`
- [ ] The oracle verifies HA **bit-exact** on a **GRBG** (and/or **GBRG**) fixture
  with a hand-computed expected table — same code path, all four Bayer phases now
  covered (closes SPEC-030/FU-1)
- [ ] DC-preservation holds on all four phases (RGGB, BGGR, GRBG, GBRG)
- [ ] A tier-A test asserts `demosaic_into` returns (typed error or ok, never panics)
  on a hand-built hostile/degenerate CFA input — CI asserts the contract without
  nightly fuzz
- [ ] `src/demosaic.rs` unchanged (test + fuzz only); mono path + every STAGE-006/007
  oracle stays green; panic-free lints hold; the fuzz crate lints clean (DEC-027)

## Failing Tests
- **`tests/demosaic_oracle.rs`**
  - `"adams_hamilton_grbg_matches_hand_computed"` (and/or `_gbrg_`) — bit-exact HA on
    a GRBG fixture vs a hand-written `EXPECTED_GRBG`.
  - `"constant_mosaic_is_preserved_on_all_four_phases"` — DC-preservation over
    RGGB / BGGR / GRBG / GBRG.
  - `"demosaic_is_panic_free_on_hostile_cfa"` — a hand-built hostile CFA (truncated
    plane / odd dims / unsupported pattern) drives `demosaic_into` and asserts it
    returns, never panics.
- **`fuzz/fuzz_targets/demosaic.rs`** — the libFuzzer target (nightly fuzz, not CI;
  the tier-A test above is CI's assertion of the same contract).

## Non-Goals
- No demosaic algorithm change (HA stays); no white balance / colour / ΔE (STAGE-008);
  no MHC.
- Not re-running SPEC-031's red-proof (this adds phase coverage + fuzz, not a new
  oracle).
- Non-2×2 / X-Trans CFA; real-image comparison (STAGE-008 ΔE).

## Notes for the Implementer
- Mirror `fuzz/fuzz_targets/develop.rs` exactly (the `MAX_PIXELS` cap + the
  "rejected container/sensor/unpack = SUCCESS" contract); the demosaic `dst` is
  `len*3` — bound it the same way. Same `+nightly` / `+toolchain` handling as the
  other targets (see `ifd.rs`'s header).
- Phases: `GRBG=[1,0,2,1]` (row 0 G,R / row 1 B,G), `GBRG=[1,2,0,1]` (row 0 G,B /
  row 1 R,G). Hand-compute `EXPECTED_*` with SPEC-031's HA arithmetic on a 5×5 fixture
  (offset-2 in range), the same way `EXPECTED_BGGR` was built.
- Almost certainly TEST-ONLY (no public API change) → **no CHANGELOG** (repo
  convention for test-only specs, cf. SPEC-025 / SPEC-027); confirm at ship.
- Seeds: reuse SPEC-027's `valid_cfa_sensor` (`tests/support/tiff.rs`) for a valid
  demosaic seed + a truncated one; `cargo run --example fuzz-seeds` regenerates
  `fuzz/seeds/`.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Nothing material — a clean [S] that landed right at estimate (19.5M vs 20M)
   by reusing SPEC-030/031's oracle harness, `fuzz/fuzz_targets/develop.rs` as the
   target template, and SPEC-027's CFA builders wholesale. The lesson is positive:
   the Nth spec on an established harness is cheap and predictable — estimate it low.

2. **Does any template, constraint, or decision need updating?**
   — No new DEC. Confirmed the test-only → no-CHANGELOG convention (cf. SPEC-025 /
   SPEC-027). The patent-check lesson from SPEC-031 (patent-check a named algorithm
   at frame time) is still pending — file it at the STAGE-007 stage-ship signal walk.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. This **closes SPEC-030/FU-1** (GRBG + GBRG fixtures → all four Bayer phases
   now bit-exact-verified). STAGE-007's backlog is drained (3 shipped / 0 open), so
   the next step is the **STAGE-007 stage-ship** (Prompt 1d), then STAGE-008.

4. **Where was the worst defect caught?**
   — `none` — clean build, approved first pass. Verify hand-recomputed the GRBG (3,2)
   and GBRG (1,2) pixels from the HA definition (both matched), re-ran the fuzz
   (8.6M runs, 0 crashes), and confirmed zero `src` change — finding no defect.

5. **What can a user do now that they couldn't before?**
   — Same demosaic API, now **hardened and fully phase-verified**: before, the
   panic-free contract was unit-tested only (1×1 / 2×2 / 3×5) and only RGGB + BGGR
   were fixture-covered; after, a `demosaic` libFuzzer target proves it panic-free
   over 11,273,543 runs (0 crashes) and GRBG + GBRG bit-exact fixtures cover all four
   Bayer phases (closing SPEC-030/FU-1). No new call — the gain is the guarantee.
