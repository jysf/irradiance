---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-044
  type: decision
  confidence: 0.75
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

created_at: 2026-11-09
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/demosaic.rs
  - src/color.rs
  - src/lib.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-051-opt-in-row-band-demosaic-threading.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md
  - docs/provenance-ledger.md

tags:
  - threading
  - demosaic
  - performance
  - opt-in
  - determinism
  - stage-012
  - spec-051
---

# DEC-044: Opt-in output-row-band threading for the demosaic hot loop

## Decision

Add opt-in parallelism to the demosaic / colour-develop hot loop by splitting the
**output** into contiguous row bands and running one `std::thread::scope` thread
per band, each over a **disjoint `dst` slice**. A caller-specified thread count
defaults to 1, and **`threads <= 1` is the existing single-thread path, byte for
byte** — no scope, no banding, no extra allocation. The threaded output is
byte-identical to the single-thread output and independent of the thread count and
of scheduling. Standard library only — **no dependency, no `unsafe`, no SIMD**.

## Why output bands (and not input bands)

Bit-identity needs each thread to own a region of `dst` that no other thread
touches — the borrow checker proves disjointness through `split_at_mut` /
`chunks_mut`, and that same disjointness is the guarantee. Output bands give each
thread a contiguous `dst` slice directly. Input bands do not: `SPEC-050`'s colour
sink scatters each developed pixel to its `DefaultCrop`+`Orientation` output
position, so an input row-band's pixels can land anywhere in `dst`.

Each thread then works **backwards from its output band** to the source rows it
needs, runs the **existing single-thread traversal** over those rows (with the ±2
halo), and writes only its own `dst` band. Same neighbours, same order, same `f64`
matrix per pixel, no cross-thread reduction — so every pixel is identical to the
single-thread render regardless of N.

## The three entry points and how each bands

Add threaded siblings of the existing functions, additive, the single-thread
signatures unchanged (shape `(sensor, src, dst, threads: usize)`; `threads` 0 or 1
means single-thread). `threads >= 2` partitions the **output rows** into that many
near-equal contiguous bands.

- **`demosaic_into` → `demosaic_into_threaded`.** Output rows map 1:1 to source
  rows, so output band `[oy0, oy1)` needs source rows `[oy0-2, oy1+2)` (clamped,
  reflected at the true edges). Every input threads cleanly. (The single-thread
  `demosaic_into` keeps green in `dst`'s own green slots and allocates nothing;
  the threaded path cannot — pass 2 would read green across a band boundary while a
  neighbour writes red/blue there — so each thread uses a **local** green buffer
  for its `[oy0-2, oy1+2)` source rows. That is opt-in memory, only when `N >= 2`.)

- **`develop_color_into` / `develop_color_f32_into` → `*_threaded`.** The resolved
  `Orientation` splits into two cases:
  - **Non-transposing (orientations 1–4).** The sink maps each source row to a
    single output row (identity or vertically flipped), so output band
    `[oy0, oy1)` is fed by a **contiguous** source row band. Thread it: each thread
    develops that source band (local green + the `SPEC-049` rolling normalise cache)
    and scatters into its disjoint `dst` band.
  - **Transposing (orientations 5–8, the 90° rotations).** An output row band is
    fed by source **columns** spanning every source row, so no contiguous source
    band feeds it and the bands cannot carve `dst` without a gather that defeats the
    rolling cache. **Fall back to single-thread** for these (correct, just not
    parallel). Record it as a known limit; a later spec can add a transposed
    output-band gather if the portrait case proves worth it.

## Memory: no new persistent buffer

The threaded colour path reuses `SPEC-049`'s shape — each band holds a **local**
green buffer for its own source rows plus the ±2 halo, which sums to about one
`width·height` green plane across all bands (the single-thread plane, partitioned)
plus a few halo rows per band. **No active-area or full-plane RGB intermediate is
introduced** (that would undo `SPEC-049`/`SPEC-050`). `demosaic_into_threaded`'s
local green is the one genuinely new allocation, and only when `N >= 2`.

## Determinism and safety

- **Deterministic:** the band decomposition is a pure function of
  `(output_height, N)`; each pixel is computed in the same order and arithmetic as
  the single-thread path; there is no floating-point reduction across threads. So
  the output does not depend on N or on how the OS schedules the threads.
- **Panic-free:** `std::thread::scope` joins all threads before returning, and a
  panic inside a scoped thread is re-raised on join — but the kernels are already
  panic-free (`DEC-041`), so no new panic path is introduced; reuse
  `resolve_geometry`'s typed geometry errors (validate once, before spawning).
- **`threads <= 1`** dispatches straight to the existing function — no banding
  overhead, no behavioural change, the default for every current caller.

## Scope and non-goals

- **In:** `*_threaded` entry points; output-row-band decomposition via
  `std::thread::scope`; `demosaic_into` for all inputs and the colour path for
  non-transposing orientations; the single-thread fallback for transposing colour
  orientations; a measured before/after in the ledger.
- **Out:** `rayon` or any threadpool/parallelism **dependency**; `unsafe`; SIMD;
  auto-detecting the thread count (the caller passes it); a transposed-orientation
  colour gather (possible later spec); any change to the single-thread output, the
  colour maths (`DEC-038`), the kernel, or `DEVELOP_VERSION` (threading is
  output-invariant, so the develop contract and version do not move).

## Addendum (ship of SPEC-051, from the verify FU-1)

The build (`SPEC-051`, PR #157) settled the mechanics this decision left loose; the
verify confirmed them bit-identical, so they are the decided design:

- **Halo radius is 3, not "±2".** The body above says "±2 halo", but a band needs
  3 rows of halo, and its **top is rounded down to an even row** so the Bayer /
  `ActiveArea` phase at the seam is identical to the single-thread traversal. The
  kernel's ±2 neighbour reach plus the green frontier the fused pass advances is
  why 3 (even-aligned) is the correct figure. Mutating it to 2, or dropping the
  even rounding, turns the byte-equality tests red.
- **Band count is `min(threads, output_rows)`**, split with `split_at_mut_checked`;
  workers are `std::thread::Builder::spawn_scoped` and the **calling thread drains
  the same band queue**, so a refused spawn degrades to fewer workers, never a
  panic.
- **`demosaic_into_threaded` bands run the fused traversal** (the same kernel the
  colour path uses), with a local green buffer, rather than the in-`dst` green of
  the single-thread `demosaic_into` — a neighbour band would otherwise be writing
  red/blue into the green a band reads.

Still open (non-blocking, not done here): **FU-2** — floor the band height or cap
`threads`, since a very large `threads` spawns one OS thread per row and does about
7× the work; worth a small follow-up spec. **FU-3** — an optional non-zero
even-origin colour fixture.
