---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-020
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-05
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - tests/develop_oracle.rs
  - tests/support/oracle.rs

tags:
  - testing
  - oracle
  - develop
  - spec-015
---

# DEC-020: the analytic oracle compares by RANK and FREQUENCY, never by position

## Decision

`tests/support/oracle.rs`'s checks for `SPEC-015`'s AC1 (`bound_check`), AC2
(the same function), and AC3 (`multiset_equal`) compare `develop_into`'s
actual output against an expectation derived from the crop window **without
ever computing a per-output-pixel source position** — the one thing that
would require reimplementing `Orientation`'s eight-case table. Two
techniques, both resting on the same two facts (`Orientation` only
*permutes* positions; `normalize`/`exact_affine` are monotonic, strictly so
on `[BlackLevel, WhiteLevel]` — `AC4`):

- **AC3 (the permutation property):** a value → count frequency table
  (`multiset_equal`) over `develop_into`'s actual output must equal the
  frequency table of the crop window mapped through `rounded_affine`. A
  permutation cannot change a multiset.
- **AC1/AC2 (the per-pixel bound and the truncation trap):** `bound_check`
  merges two frequency tables **by rank**, in a single counting-sort-style
  pass — the i-th smallest raw sample (counting repeats) is paired with the
  i-th smallest actual output sample, which is provably the SAME pairing the
  true positional map would produce, because both sides are sorted by a
  monotonic function of the same underlying value.

`AC4`'s own tier-A test (`normalization_is_strictly_monotonic_and_injective`)
proves the monotonicity/injectivity fact these two techniques both lean on,
directly against `exact_affine`/`rounded_affine`, with no file needed.

## Context

`SPEC-015`'s central finding (`## The design decision this spec rests on`) is
that an oracle reimplementing the transform is a mirror. The spec's own text
resolves this for AC3 by name (histogram equality, position-independent by
construction). It does not fully resolve it for AC1, whose stated form —
"every output pixel is within 0.5 of the exact real-valued affine map" —
reads as if it needs to know WHICH raw sample fed WHICH output pixel, i.e.
the orientation mapping.

It does not, because of a fact worth stating precisely: `Orientation` is a
**bijection on positions that never touches values**, and `exact_affine` is
monotonic. Sorting `actual_output` and `{exact_affine(v) : v in crop window}`
independently and pairing by rank reconstructs the TRUE positional pairing
exactly, without this file ever stating what the permutation IS. This is not
an approximation — for a monotonic function, "sorted order" and "the
function's own preimage order" are the same order.

The build session's first attempt at this optimized the wrong direction: it
paired the **i-th smallest DISTINCT value** on each side (weighting only the
truncation tally by count), not the i-th smallest sample counting repeats.
That is a different, WEAKER claim, and it was caught by its own honest-tree
assertion (`the_oracle_is_red_on_a_levels_fault` reported the honest tree's
own `max_deviation` as infinite): many raw values below `BlackLevel`
legitimately collapse to output `0`, so a bare distinct-value list can see
that `0` occurs but not that the WRONG NUMBER of pixels collapsed there —
exactly the shape of a `BlackLevel` fault. `bound_check`'s actual mechanism
(a two-pointer merge over runs, i.e. `(value, remaining count)` pairs in each
frequency table) is what restores full rank-fidelity: run length IS the
"counting repeats" that distinguishes it from the broken attempt.

## Alternatives Considered

- **Option A: compute the true positional map (reimplement `Orientation`).**
  - Why rejected: this is the exact failure mode `SPEC-015` exists to avoid —
    a second copy of the eight-case table fails and succeeds for the same
    reasons as the first (`DEC-004`'s own limit, restated for this spec).

- **Option B: sort both sides (`sort_unstable`) and zip by index.**
  - What it is: materialize `actual_output` and the crop-window-derived
    expectation as `Vec`s, sort each, and pair element-wise.
  - Why rejected: mathematically correct (this is what `bound_check`'s merge
    is EQUIVALENT to), but measured too slow at ~47-megapixel scale in a
    debug build — a single tier-B test alone took 91.78s with a closure-based
    `f64` comparator, and still 79.32s after switching to a primitive `u16`
    sort. The comparison SORT itself, not the comparator, was `AC8`'s
    pre-registered 60s bound's dominant cost.

- **Option C: pair by DISTINCT value only, weighting nothing but the
  truncation tally by count.**
  - Why rejected: measured wrong (see Context) — it cannot see a fault that
    shifts the WRONG NUMBER of pixels into an otherwise-valid, already-occurring
    bucket, only a fault that introduces an impossible value.

- **Option D (chosen): a rank-preserving merge over two frequency tables.**
  - What it is: `bound_check`'s two-pointer walk over `(value, remaining
    count)` runs in each of `crop_freq`/`actual_freq` (65536-entry arrays,
    index == value).
  - Why selected: provably equivalent to Option B (sorting and zipping) —
    same pairing, same weights — computed in O(n + 65536) instead of O(n log
    n), and it never sorts a ~47-megapixel array at all. Measured: the full
    switch (frequency tables plus the rank-merge) brought the tier-B suite
    from 95.72s to 14.92s, reproducing IDENTICAL numbers to the naive
    per-pixel/sorted version on every one of the three real files (max
    deviation 0.499968/0.499968/0.499969; truncation disagreement
    50.1%/49.1%/45.0%; distinct levels 15872/16164) — the optimization
    changed performance, not the answer.

## Consequences

- **Positive.** AC1, AC2 and AC3 all avoid the orientation table by
  construction, not by discipline — there is no eight-case table anywhere in
  `tests/support/oracle.rs` or `tests/develop_oracle.rs` for a future editor
  to accidentally reach for.
- **Positive.** The tier-B suite runs in ~15s in a debug build, comfortably
  under `AC8`'s pre-registered 60s bound, with no loss of exactness (every
  pixel is still accounted for, weighted by its true multiplicity).
- **Negative.** The merge in `bound_check` is less obviously correct on
  first read than a sort-and-zip would be; DEC-020 (this record) and the
  module doc's worked explanation of the rejected Option C are the mitigation
  — a future editor tempted to "simplify" it to per-distinct-value pairing
  should read the rejected option first.
- **Neutral.** `histogram`'s `HashMap`-based implementation is kept (not
  replaced by a frequency table) for the two red-proof tests' tiny,
  6-element fixtures, where a `HashMap`'s per-value diagnostic clarity matters
  more than the several-orders-of-magnitude performance difference that only
  shows up at real-corpus scale.

## Validation

**Right if** a future change to `develop_into`'s levels or geometry handling
that should be caught by AC1/AC2/AC3 still turns these checks red, and the
tier-B suite stays comfortably under 60s as corpus files are added.

**Wrong if** a future fault shape exists that a rank-preserving merge cannot
distinguish from an honest tree while a true per-pixel positional check
could — revisit the merge's correctness argument (monotonicity + permutation)
against that fault's specific shape before assuming Option B (sort-and-zip)
is needed after all.

## References

- Governing spec: `SPEC-015`, `## The design decision this spec rests on`
- Related decisions: `DEC-004` (why no oracle may reimplement the transform),
  `DEC-017` (the OTHER oracle's own mechanism decision, for a code-path
  fault rather than a data-shape one — see `DEC-021`), `DEC-018` (the
  rounding rule AC1 is deliberately agnostic to)
- Evidence: `tests/develop_oracle.rs`'s `eprintln!` trails, matching the
  spec's own `## Implementation Context` measurements exactly
