---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-042
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-07
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/color.rs
  - src/lib.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-049-colour-develop-memory-fuse-and-f32-entry-point.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md
  - docs/provenance-ledger.md

tags:
  - colour
  - demosaic
  - memory
  - f32
  - bit-identical
  - stage-012
  - spec-049
---

# DEC-042: Fuse the colour-develop passes, and add an f32 entry point

## Decision

Restructure `develop_color_into` from three buffered passes into two, removing
both scratch buffers, and add a public `develop_color_f32_into` that shares the
fused pipeline. Two parts, one cohesive change in `src/color.rs`.

**The fuse.** Today the path is: `normalize_mosaic` → a `normalized` mosaic
(`w*h` `u16`, ~47 MB); `demosaic_into` → a `camera_rgb` intermediate (`w*h*3`
`u16`, ~140 MB); `apply_matrix_into` reads `camera_rgb` and writes `dst`. Fuse it:

- **Pass 1** computes green at every pixel into a `w*h` green plane,
  **normalising each sample on read** — so the `normalized` mosaic is never
  allocated.
- **Pass 2** interpolates red and blue per pixel (reading the green plane and
  normalising the needed `src` samples on read), assembles the camera-RGB triple
  **in registers**, applies the 3×3 colour matrix in `f64`, quantises, and writes
  the final linear-sRGB `u16` straight into `dst` — so `camera_rgb` is never
  allocated and `apply_matrix_into`'s separate pass disappears.

Only the `w*h` green plane remains as scratch: ~47 MB, down from ~187 MB
(`normalized` + `camera_rgb`) — about **140 MB saved at 23 MP**.

**The f32 entry point.** Add `pub fn develop_color_f32_into(sensor: &Sensor, src:
&[u16], dst: &mut [f32]) -> Result<(), Error>` running the same fused pipeline,
but pass 2 writes `f32` into `dst`. It must equal
`to_linear_f32_into(develop_color_into(..))` **exactly**, and that function
(`src/tone.rs`) divides the **quantised `u16`** by `65535.0`. So the f32 path
**quantises to `u16` first (identical to the integer path), then divides by
`65535.0`** — it does NOT divide the raw `f64` matrix result, which would differ
by the clamp-and-round. This lets a float caller (the Python bindings, ML
dataloaders) skip holding a `u16` result and its `f32` copy at once.

## Bit-identity is the gate

The `u16` output must not move: the fused path performs the **same integer
operations in the same order** — the same per-slot normalisation
(`normalize_sample`), the same two-pass green-then-red/blue demosaic (reading the
same neighbours), the same `f64` matrix and the same `quantize` (clamp to
`[0, 65535]`, round half up, `DEC-018`). Reordering *which buffer* holds an
intermediate changes nothing the arithmetic sees. The existing colour goldens
plus a byte-for-byte equivalence test against the pre-fuse path are the proof.

## The normalise-on-read tradeoff

Removing the `normalized` buffer means each photosite is normalised **every time
it is read as a neighbour** (up to ~9×), rather than once. `normalize_sample` is
a cheap, deterministic integer map, so the result is bit-identical, but it is
extra compute layered on P1's (SPEC-048) fast path. Memory is the goal here
(peak-RSS), not speed: the build records **both** peak-RSS and `demosaic`/develop
wall-time before and after in `docs/provenance-ledger.md`, and the RSS drop is
what this spec is judged on. If the recompute regresses time materially, that is
a design note for a later pass (e.g. a one-row normalised cache), not a reason to
keep the full-plane buffer.

## Scope and perf discipline

- **New public API:** `develop_color_f32_into` only — additive, semver-minor.
  `develop_color_into`'s signature is unchanged.
- Governed by the STAGE-012 perf discipline (`DEC-041`): a `dst_after ==
  dst_before` proof (goldens + equivalence test), a measured peak-RSS (and time)
  line in the provenance ledger, and this `DEC`.
- **Out:** any `u16` output change, the colour maths (`DEC-038`), the demosaic
  kernel, the colour-geometry crop follow-up (§3B), P3 threading, SIMD/`unsafe`,
  and new dependencies.
