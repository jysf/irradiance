---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-067
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-019
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-062, DEC-058, DEC-060, DEC-061, DEC-043, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-063, SPEC-065, SPEC-066]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-019's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-019's orientation performance: Orientation 5-8 develops near Orientation 1 speed on both mono paths, byte-identical, for every caller of develop_into."

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
  tokens_estimate: 22000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-067: Tiled Orientation 5–8 gather on both mono develop paths, byte-identical, with a committed timing example

## Context

SPEC-067 is the fifth and last spec in STAGE-019 (PROJ-004 instrument layer C). It
covers monorail-000 / onyx-000 request 4 (`feedback/2026-11-11-monorail-000-requests.md`).
The develop took 774 ms at Orientation 6 against 617 ms at Orientation 1 on the same
plane. They proposed a blocked transpose, with acceptance on byte-identical output plus a
before/after benchmark.

**DEC-062** (minted with this design) records what the probe found:
- The cost is the transposed gather, on **both** mono paths.
- The requester's frames take the **post-warp** path (`crop_and_orient_from_active_into`),
  which the stage notes did not name.

DEC-062 fixes the traversal rule (64×64 output tiles for Orientations 5–8, today's row
walk for 1–4), the one permitted test edit, and how the timing is measured.

## Goal

Make `crop_orient_normalize_into` and `crop_and_orient_from_active_into` visit the output
in 64×64 tiles for Orientations 5–8, with output byte-identical to today for every input
and option. Commit a release-mode timing example that shows the before/after.

## Design

### 1. Probe results (main at eaa2ae3, 2026-11-13; recorded so build is transcription)

**Timing.** Apple M4 Pro, release build, `develop_with_into` on Q2M L1021223, with
`Sensor::orientation` overridden, median of 5 after one warm-up, in ms:

| Path | O1 | O2 | O3 | O4 | O5 | O6 | O7 | O8 | O6, `apply_orientation: false` |
|---|---|---|---|---|---|---|---|---|---|
| `warp_alpha` 0 (warp-free gather) | 190 | 189 | 194 | 190 | 370 | 374 | 409 | 410 | 189 |
| `warp_alpha` 1 (post-warp gather) | 604 | 606 | 590 | 594 | 767 | 765 | 784 | 779 | 608 |

The requester's 774/617 reproduces as 765/604. A standalone 8368×5584 gather took 281 ms
naive and 124 ms in 64×64 tiles at Orientation 6. The DEC-062 Context has the full
tile-size table.

**Byte pins** (the pre-change oracle), computed on main at eaa2ae3 with the throwaway
probe. Each is the MD5 of `develop_with_into`'s output as little-endian `u16`, for
L1021223 with `orientation` overridden. Orientation 1 at α = 1 `StandardCrop` equals
SPEC-063's existing pin `feb49105…`.

`(orientation, warp_alpha, framing, width, height, md5)`:
```
(1, 0.0, Std,  8368, 5584, "5987848b3a7d67f5eedcc15643424fe3"),
(2, 0.0, Std,  8368, 5584, "48e16c555ce7902dfe69f667b5fdb5aa"),
(3, 0.0, Std,  8368, 5584, "47a94fb041d111fd1ec933458258987f"),
(4, 0.0, Std,  8368, 5584, "e47775dac94c816b54b246c58425f567"),
(5, 0.0, Std,  5584, 8368, "1cc4629f4b01eee22e5b27644bb507a5"),
(6, 0.0, Std,  5584, 8368, "c4e27ed3c4c5db88b9ce5b90e479953e"),
(7, 0.0, Std,  5584, 8368, "7bdd2a15d81c6e038a49f6f7fe44e49c"),
(8, 0.0, Std,  5584, 8368, "018aa23a740a9709292ed4f197972e88"),
(1, 1.0, Std,  8368, 5584, "feb4910588d189bd5bf89620b480fe6c"),
(2, 1.0, Std,  8368, 5584, "bfddc03dfba0a5e11d0bccd4e8bd0776"),
(3, 1.0, Std,  8368, 5584, "c59395e798fa7e077800d263868957c9"),
(4, 1.0, Std,  8368, 5584, "2cb1916f23c45f22c412a31529f24e92"),
(5, 1.0, Std,  5584, 8368, "a4292458049cba8683bc9824374817ba"),
(6, 1.0, Std,  5584, 8368, "94059540d3da4547880ae4497bffcf16"),
(7, 1.0, Std,  5584, 8368, "3e2e9f08e1f6f9d20376c12ce5f85852"),
(8, 1.0, Std,  5584, 8368, "6ed7ac26d186433bc3c18982f737b2b8"),
(1, 0.0, Full, 8392, 5632, "70dfe17181c2e382cabba831d84866fb"),
(2, 0.0, Full, 8392, 5632, "0a4d591a282863fd5b929ea2753b4fb5"),
(3, 0.0, Full, 8392, 5632, "4ee3c4b57ff4da6adae334fc163df5c3"),
(4, 0.0, Full, 8392, 5632, "5643a83af6a697f049b5a2ec5b1e55fc"),
(5, 0.0, Full, 5632, 8392, "0ef1ae6659cf9db99de7dcb8195ae594"),
(6, 0.0, Full, 5632, 8392, "3ffb1ff599af4add4f4ced0c4c24134c"),
(7, 0.0, Full, 5632, 8392, "f28a2e0384100ab4e6755633fc2df9df"),
(8, 0.0, Full, 5632, 8392, "43dbea0e36328116738b6c32e25834f4"),
```

The pins cover:
- **`Std`** = `Framing::StandardCrop` and **`Full`** = `Framing::FullActiveArea`.
- **Every `apply_orientation`** is true. α = 1 under `Full` is omitted because the
  post-warp gather is already covered at α = 1 `Std`, and a full-active-area warped
  develop doubles the tier-B time for no new code path.

**The source-scan test.** `tests/develop_oracle.rs::inject_orientation_identity_fault`
copies the crate and finds `fn crop_orient_normalize_into(`. It asserts that **exactly
one** `crop_source_coords(` call with today's exact indentation (`needle`) is in that
body, replaces it with the identity, and expects the oracle to go red. `tests/develop.rs`
scans `apply_fix_bad_pixels_constant`, which this spec does not touch.

### 2. The traversal (`src/develop.rs`), exactly DEC-062 §1–§2

A private `const ORIENT_TILE: u32 = 64;`. In both `crop_orient_normalize_into` and
`crop_and_orient_from_active_into`:

```text
let (tile_w, tile_h) = if (5..=8).contains(&geometry.orientation) {
    (ORIENT_TILE, ORIENT_TILE)
} else {
    (out_width, out_height)        // one tile = today's row-major walk
};
for tile_y in (0..out_height).step_by(tile_h) {
    for tile_x in (0..out_width).step_by(tile_w) {
        for out_y in tile_y..min(tile_y + tile_h, out_height) {
            for out_x in tile_x..min(tile_x + tile_w, out_width) {
                <today's per-pixel body, unchanged>
            }
        }
    }
}
```

- **The per-pixel body is moved, not rewritten.** That covers `crop_source_coords`, the
  raw/active index, the SPEC-066 black, `normalize`, and the write. Keep exactly one
  `crop_source_coords(` call in each function.
- **Guards:**
  - `step_by` needs a non-zero step. A zero-width or zero-height output cannot reach
    here (`resolve_geometry`), but guard it with `.max(1)` so the function stays total.
  - `tile_y + tile_h` uses `saturating_add` (clippy's arithmetic lints).
- **No allocation, no `unsafe`.** The traversal shape for 1–4 must be today's, so a
  perf regression there is impossible by construction.

**Permitted test edit (DEC-062 §2):** update only the `needle` literal in
`tests/develop_oracle.rs`'s `inject_orientation_identity_fault` to the new indentation.
Its "exactly one" assertion, the replacement text and the expectation that the oracle goes
red stay as they are. Record the edit in the handback.

### 3. Timing example (`examples/orient-timing.rs`, `just orient-timing`)

- **Input.** It reads `IRRADIANCE_CORPUS_DIR` with the same root resolution as
  `examples/corpus-status.rs` (`#[path = "../tests/support/corpus.rs"]`). Its file is
  `LEICA-Q2-MONO/L1021223.DNG`. If the file is absent, it prints
  `orient-timing: SKIP L1021223 absent` and exits 0.
- **Measurement.** For `warp_alpha` in {0, 1}, and for orientation 1..=8 (plus O6 with
  `apply_orientation: false`), it times `develop_with_into` after one warm-up, over 7
  runs.
- **Output.** For each, it prints the median, the min and the ratio to O1 at the same α.
  First it prints the machine (`std::env::consts::ARCH` plus `sysctl -n
  machdep.cpu.brand_string` when available, else "unknown") and the git describe it was
  built from (via `option_env!` or a runtime `git` call, whichever is simpler; "unknown"
  is fine).
- **The recipe.** `orient-timing` in `justfile` runs
  `cargo run --release --all-features --example orient-timing`.
- **"Before" numbers.** Build the example on a checkout at eaa2ae3 plus only the example
  file, or reuse §1's table if the machine matches. Record both tables in the handback.

### 4. Docs

- **CHANGELOG `[Unreleased]`,** Changed: the mono develop of Orientations 5–8 visits the
  output in tiles, with the before/after medians from the example. Output is unchanged.
- **The `crop_orient_normalize_into` doc comment** gets one sentence on the traversal and
  why. `DEVELOP_VERSION` stays `"4"`.
- **No provenance row**, since this is a loop order and not an algorithm.

## Inputs

- **Files to read:**
  - `src/develop.rs`: `crop_source_coords` ~486, `crop_orient_normalize_for` ~903,
    `crop_orient_normalize_into` ~926, `crop_and_orient_from_active_into` ~1054, and
    `run_passes`;
  - `tests/develop_oracle.rs` ~578, `inject_orientation_identity_fault`;
  - `examples/corpus-status.rs` and `tests/support/corpus.rs`;
  - `tests/support/md5.rs`.
- **Tests to mirror:** `tests/develop_options.rs` (tier-B gating and MD5 pins),
  `tests/black_levels.rs` (tier-A fixtures with an odd `ActiveArea`).
- **Decisions:**
  - DEC-062 (this spec);
  - DEC-058 (the version rule);
  - DEC-060 (no allocation);
  - DEC-061 (the per-position black inside the body);
  - DEC-043 (the colour path, untouched);
  - DEC-003.

## Outputs

- **Files modified:**
  - `src/develop.rs` (two functions and one `const`);
  - `tests/develop_oracle.rs` (the needle only);
  - `justfile` (one recipe);
  - `CHANGELOG.md`.
- **Files created:**
  - `tests/orientation_tiles.rs`;
  - `examples/orient-timing.rs`.
- **No public API change.**

## Acceptance Criteria

- [ ] **AC1. Byte identity on real data (tier B).**
  - **Twenty-four pins.** All 24 §1 MD5s hold, through `develop_with_into` and
    `develop_with_scratch_into` (using the `scratch_len` for each).
  - **SPEC-063.** Its eight pins stay green, unedited.
  - **Missing files.** A missing file is reported by name.
- [ ] **AC2. Byte identity against an independent reference (tier A).**
  - **The reference.** `tests/orientation_tiles.rs` builds the expected output with its
    **own** naive gather: the eight orientation formulas written out in the test (TIFF
    6.0 / DNG Orientation), then crop, then the SPEC-066 normalize written in the test.
    It does not call `crop_source_coords` or `develop`.
  - **What it compares.** The reference against `develop_with_into` and
    `develop_with_scratch_into`:
    - for every orientation 1–8;
    - for both framings;
    - at `warp_alpha` 0, and at 1 with an identity-equivalent in-bounds warp. For the
      warped case, compare against `apply_orientation: false` output re-oriented by the
      test's own formulas, so the warp itself needs no reference.
    - on crop sizes chosen to hit every tile edge: 1×1, 63×65, 64×64, 65×63, 130×67 and
      200×129, with odd `ActiveArea` and `DefaultCrop` origins;
    - under `BlackLevels::PerPosition` with a 2×2 pattern for at least one size.
- [ ] **AC3. The tiled walk is really used.**
  - Add a `#[cfg(test)]` unit test in `src/develop.rs` that records the visit order of
    the traversal helper on a 130×67 output (or exposes the order via a small private
    iterator).
  - For orientations 5–8 the first 64 visits are row 0, columns 0–63, and visit 65 is
    row 1, column 0. For 1–4 the first 130 visits are row 0.
  - Every pixel is visited exactly once.
  - **Red-proofs:**
    - an untiled 5–8 walk;
    - a tiled 1–4 walk.
- [ ] **AC4. The fault injector still bites.**
  - `tests/develop_oracle.rs`'s orientation fault test passes. That means the injected
    fault is found and turns the oracle red.
  - The only diff in that file is the `needle` literal (`git diff` shows one changed
    string).
- [ ] **AC5. No allocation.** `tests/develop_scratch_alloc.rs`'s
  `scratch_develop_allocates_nothing` stays green. Add Orientation 6 and 8 rows (warped
  and warp-free) if none exist.
- [ ] **AC6. Timing (recorded, not a CI gate).**
  - `just orient-timing` runs, and its before and after tables are in the handback with
    the machine named.
  - **Target:** on the build machine, each of O5–O8's medians is within 1.15× of O1's at
    the same α.
  - **Floor:** O6's warp-free median improves by at least 25% from §1's 374 ms on the
    same machine class.
  - If the target is missed, report it with the numbers. The verify decides; byte
    identity still holds.
- [ ] **AC7. No regression.**
  - Every existing test passes with no assertion changed, other than AC4's needle.
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.
  - `DEVELOP_VERSION == "4"`.
  - CHANGELOG `[Unreleased]` is updated (§4).

## Failing Tests

- **`tests/orientation_tiles.rs`:**
  - `tiled_output_matches_the_pre_change_pins`: AC1, tier B, 24 pins, both entry points.
  - `every_orientation_matches_an_independent_gather`: AC2.
  - `per_position_blacks_survive_the_tiled_walk`: AC2, `PerPosition`.
- **`src/develop.rs` unit:** `orientation_traversal_is_tiled_only_for_5_to_8` (AC3).
- **`tests/develop_scratch_alloc.rs`:** the O6/O8 rows (AC5), if they are new.

The tier-B pin test is written **before** the change and must pass on main. That is the
point: it pins today. Record that in the handback. AC2's tier-A test also passes before
the change. AC3's unit test is the one that is red first.

**Red-proof expectations.** The verify mutates each of these, and each must turn a named
**tier-A** test red:
- an off-by-one at the tile's edge (`<=` in the clip, or the tile end not clipped to the
  output) → AC2 (63×65, 130×67);
- a pixel skipped at a tile boundary (`step_by(tile + 1)`) → AC2;
- the tile's x and y loops swapped so that the tile walk writes transposed positions →
  AC2;
- orientations 5–8 untiled → AC3;
- orientations 1–4 tiled → AC3;
- `crop_and_orient_from_active_into` left untiled while `crop_orient_normalize_into` is
  tiled → AC3. The unit test must cover the helper both functions use, or each
  function's order.

## Non-Goals

- The colour path's orientation scatter.
- Parallelism (no threads in develop).
- SIMD.
- A tile size tuned per machine.
- A criterion dependency or a CI timing gate.
- Any output change or a `DEVELOP_VERSION` bump.
- Unit-level unpack (the next stage).

## Notes for the Implementer

- **Both functions.** The requester's frames all have a warp, so
  `crop_and_orient_from_active_into` is the one their timing measures. Tiling only
  `crop_orient_normalize_into` would leave their number unchanged.
- **Shared traversal helper.** A private generic helper (for example
  `for_each_out_pixel(width, height, orientation, |x, y| ..)`, inlined) would let both
  functions share the traversal and let AC3 test it once. If you use one, keep the
  `crop_source_coords(` call textually inside `crop_orient_normalize_into`'s body, inside
  the closure, so the injector still finds exactly one.
- **The pins.** The §1 pins were computed on main **before** this change. If one fails
  after it, the change altered output. Fix the change, never the pin.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
