---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-041
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

created_at: 2026-11-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/demosaic.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-048-demosaic-interior-fast-path.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md
  - docs/provenance-ledger.md

tags:
  - demosaic
  - performance
  - bayer
  - bit-identical
  - stage-012
  - spec-048
---

# DEC-041: Demosaic interior fast path, and the STAGE-012 perf discipline

## Decision

Give `demosaic_into` a **row-wise interior fast path**. Both passes (pass 1
green, pass 2 red/blue) keep their current reflecting, bounds-checked path for
the **2-pixel border** — the outermost two rows and columns, the only region
where a neighbour read at offset ±1 or ±2 can leave the plane. For the
**interior** (every pixel at least two rows and two columns from each edge),
read the neighbours by **direct index** into per-row slices of `src` (and, in
pass 2, `dst`), skipping `reflect()`, `plane_index()`'s recomputation, and the
per-read bounds-checked `get`. The `(x, y)` recovery moves out of the per-pixel
body too: iterate row by row so `y` is the loop variable and the row base is
computed once per row, not via a `checked_div`/`checked_rem` per pixel.

The **kernel arithmetic is reused verbatim** — the same classifiers, second
differences, predictors, rounding and clamp, applied to the same neighbour
values in the same order. The fast path changes only *how a neighbour value is
fetched*, never *what is computed from it*, so the output is **bit-identical**.

The border stays on the existing code precisely because that is where
`reflect()` earns its keep; duplicating the reflection logic into the fast path
would be the error this split exists to avoid.

## Why

The pyirradiance rawpy/LibRaw benchmark measured `demosaic_into` at ~520 ms of a
~587 ms develop on a 23.4 MP frame (LibRaw's whole develop: 477 ms). The cost is
per-pixel machinery the interior never needs: a `checked_div` + `checked_rem` to
recover `(x, y)`, and ~9 neighbour reads each routed through `reflect()` (i64
saturating), `plane_index()`, and a bounds-checked `get`. Reflection only
matters within two pixels of an edge. Moving the interior off that machinery is
the single biggest develop win available, and it costs nothing in correctness
because the result is unchanged.

## Safety and scope

- **Panic-free without per-read checks.** Per-row slices (`src[row-2 ..= row+2]`
  windows, bounds-sliced once per row) make the interior neighbour reads
  in-bounds *by construction*, so the fast path needs no per-element `get` and
  introduces no `unsafe` — it stays under `#![forbid(unsafe_code)]` and the
  five-lint deny policy.
- **No public API, no dependency, no output change.** `demosaic_into`'s
  signature and the developed bytes are unchanged; this is an internal
  restructuring.
- **Out of scope:** SIMD or `unsafe`; the colour-develop memory / f32 work (P2);
  threading (P3); the colour-geometry crop follow-up — each its own STAGE-012
  spec.

## The STAGE-012 perf discipline (this and every later optimisation)

Recorded here because it governs P1–P3, not just this spec (the external 0.2.0
staff review's §5): **every performance change in this stage is gated on all
three** —

1. a `dst_after == dst_before` proof (an equivalence test against a
   straightforward reference, plus the existing golden hashes), so bit-identity
   is demonstrated, not asserted;
2. a before/after wall-time (and peak-RSS where the change is about memory) line
   in `docs/provenance-ledger.md`, so the win is measured, not claimed;
3. its own `DEC-*`.

No optimisation changes the integer operations or their order. A change that
cannot show bit-identity is not a performance change — it is a behaviour change
and needs a different spec.
