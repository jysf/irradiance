---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-062
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-13
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/develop.rs
  - tests/develop_oracle.rs
  - examples/orient-timing.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-067-blocked-orientation-5-8-transpose-in-crop-orient-normalize-i.md

tags:
  - develop
  - orientation
  - performance
  - stage-019
---

# DEC-062: the mono orientation gather walks the output in 64×64 tiles for Orientations 5–8, and stays byte-identical

## Decision

### 1. Both gather loops, one traversal rule

The mono develop orients in **two** places:
- `crop_orient_normalize_into`, on the warp-free path;
- `crop_and_orient_from_active_into`, after the warp.

Every held Q2M frame carries a warp, so the requester's 774 ms against 617 ms comes from
the **second** one. The stage notes named only the first. Both change, under one rule:

- **Orientations 5–8** (the output is transposed against the source) walk the output in
  **64×64 tiles**: tile rows top to bottom, tiles left to right within a tile row, and
  pixels row-major within a tile. Edge tiles are clipped to the output.
- **Orientations 1–4** keep today's row-major walk. In terms of the rule above, that is
  one tile spanning the whole output. Their reads are already sequential or reversed
  within a row, so tiling buys nothing; the probe measured a 64-tile orientation-1 gather
  slightly slower (114 ms against 110 ms).

Only the **order** in which output pixels are visited changes. Each output pixel's value
is computed by exactly today's expression:
- the same `crop_source_coords`;
- the same source index;
- the same `normalize` with the same black (SPEC-066);
- the same write.

Every pixel is written exactly once, whatever the order. The output is therefore
byte-identical by construction, and `DEVELOP_VERSION` stays `"4"`.

### 2. Shape constraints

- **One per-pixel body per function.** The tile walk wraps the existing body. It does
  not duplicate it per orientation class. `crop_orient_normalize_into` keeps exactly one
  `crop_source_coords(` call site, because `tests/develop_oracle.rs`'s
  `inject_orientation_identity_fault` asserts there is exactly one. Only that injector's
  `needle` string may change, to match the call's new indentation. Its assertion and its
  red-proof stay as they are.
- **No allocation, no `unsafe`, no new dependency.** DEC-060's promise that
  `develop_with_scratch_into` allocates nothing still holds. Tile bounds use checked or
  saturating arithmetic (clippy's arithmetic lints).
- **The tile size is a private `const` (64).** On the M4 Pro, 64 and 128 measured within
  5% of each other (124 against 120 ms). 64 is kept because 64×64×2 B = 8 KiB of output
  per tile, with 64 source rows touched, sits comfortably in L1 (64 KiB here) on smaller
  cores too. It is not tuned per machine.
- **The colour path is untouched.** It scatters through `output_coords_from_crop` as it
  emits pixels (DEC-043), and no request covers it.

### 3. How the speed is measured (the stage's open question)

- **A committed example, `examples/orient-timing.rs`, run as `just orient-timing`.** It
  reads L1021223 from `IRRADIANCE_CORPUS_DIR`, overrides `Sensor::orientation` to each of
  1–8, and times `develop_with_into` at `warp_alpha` 0 (the warp-free gather) and at 1
  (the post-warp gather). It runs a release build, does one warm-up, then takes the
  median of 7 runs. If the file is absent, it prints a SKIP line and exits 0.
- **No criterion dependency** (no licence review, no new tree), and **no CI timing
  gate**, because CI machines are shared and noisy. The numbers are recorded in the
  handback and in the spec at ship, with the machine named.
- **The target**, checked by the verify and not by CI: on the build machine, the median
  for each of orientations 5–8 is within 1.15× of orientation 1's median at the same α.
  It is a target, not a gate. Byte identity is the hard criterion.

## Context

monorail-000 / onyx-000 request 4 reported:
- develop takes 774 ms at Orientation 6 against 617 ms at Orientation 1 on the same
  plane;
- they want a blocked transpose;
- their acceptance criteria are byte-identical output plus a before/after benchmark.

The SPEC-067 design probe (main at eaa2ae3) ran on an Apple M4 Pro (64 KiB L1D, 4 MiB
L2), release build, L1021223, with the orientation overridden, median of 5:

| Path | O1 | O2 | O3 | O4 | O5 | O6 | O7 | O8 | O6, `apply_orientation: false` |
|---|---|---|---|---|---|---|---|---|---|
| warp-free (`warp_alpha` 0) | 190 | 189 | 194 | 190 | **370** | **374** | **409** | **410** | 189 |
| warped (`warp_alpha` 1) | 604 | 606 | 590 | 594 | **767** | **765** | **784** | **779** | 608 |

All figures are milliseconds.

- **The requester's figures reproduce** (765 against 604, +161 ms with the warp). The
  warp-free path pays more (+184 ms) in relative terms: 1.97× against 1.27×.
- **The extra cost is entirely the transposed gather.** With
  `apply_orientation: false`, Orientation 6 costs the same as Orientation 1 on both
  paths.
- **A standalone gather of an 8368×5584 `u16` image** (the same index arithmetic, no
  levels) took:

  | Orientation | Naive | 16 | 32 | 64 | 128 |
  |---|---|---|---|---|---|
  | 6 | 281 | 136 | 132 | 124 | 120 |
  | 5 | 286 | | | 126 | |
  | 8 | 291 | | | 135 | |
  | 1 | 110 | | | 114 | |

  All figures are milliseconds; the size columns are tile edges.

  So tiling removes about 155–160 ms of a transposed gather's roughly 170–180 ms
  premium. The tiled and naive outputs were asserted equal in the probe.

## Alternatives Considered

- **Rely on `apply_orientation: false`** (SPEC-063). Rejected as the only answer. It
  hides the cost for a caller that rotates on the GPU, but not for any other caller of
  `develop_into`.
- **Tile every orientation.** Rejected, because it is slightly slower for 1–4 and adds
  risk to the common case for no gain.
- **Gather in source order and scatter to the output.** Rejected. It moves the strided
  access from the reads to the writes, and writes are no cheaper.
- **A criterion benchmark.** Rejected for now. It needs a new dev-dependency and a
  licence review for a single measurement. The example is enough, and criterion stays
  additive later.
- **An `#[ignore]`d timing test.** Rejected, because tests run in debug by default and a
  debug timing is meaningless. A release example states its own build.
- **A tile size tuned per machine.** Rejected. It is not deterministic across machines,
  and 64 and 128 are within 5%.

## Consequences

- Orientations 5–8 should develop close to Orientation 1's time on both paths. The
  measured result is recorded at ship.
- **There is no public API change**, `DEVELOP_VERSION` stays `"4"`, and every SPEC-063
  pin holds by construction.
- `tests/develop_oracle.rs`'s injector needle is the one permitted test edit. A future
  restructure of these loops must keep a single per-pixel call site, or update the
  injector to match.
