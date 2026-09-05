---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-018
  type: decision
  confidence: 0.8
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
  - src/develop.rs

tags:
  - decode
  - develop
  - levels
  - allocation
  - spec-014
---

# DEC-018: the developed image is `u16`, full-scale, with clamped out-of-range levels

## Decision

`develop::develop_into`'s output is **`u16`, rescaled into a caller-owned
buffer, full-scale at `u16::MAX` (65535)** — not `f32` in `[0, 1]`. A sample
outside `[BlackLevel, WhiteLevel]` is **clamped** to that range before
scaling (`AC2`), so it lands at exactly 0 or exactly `u16::MAX` rather than
wrapping, saturating past the type's own range, or being surfaced as an
error. The affine scale from `[BlackLevel, WhiteLevel]` to `[0, u16::MAX]`
**rounds to nearest** (`(numerator + denominator/2) / denominator`), not
truncates — `SPEC-014/FU-4`.

## Context

`SPEC-014`'s spec explicitly left this open — offered as input, not as the
answer — because `DEC-002` (target surface, parallelism, determinism) is
still `proposed` and a wrong commitment here is expensive to undo. Two
questions, and both needed an answer before `develop_into` could be written:

**1. What type is the normalized sample?** `f32 in [0, 1]` is what a develop
pipeline eventually wants and what `SPEC-015`'s analytic oracle will assert
against. But it costs **190 MB** for a 47 MP frame, stacked on top of
`SPEC-012`'s already-measured 182,435,840-byte peak for the file + raw plane
alone (`DEC-016`). `u16` has headroom over the sensor's native 14 bits, costs
half of `f32`, and keeps the crate's allocation shape unchanged regardless of
how `DEC-002`'s `no_std`/`alloc` question resolves.

**2. What happens to a sample outside `[BlackLevel, WhiteLevel]`?** Not
hypothetical: `SPEC-014`'s design-time probe measured that **both** real
files hold samples below `BlackLevel` (`L1021223.DNG` min 2, `L1000622.DNG`
min 108) and **both** reach `WhiteLevel` exactly — so this branch fires on
the first file a caller develops, not on some adversarial edge case.

## Alternatives Considered

- **Option A: `f32` in `[0, 1]`.**
  - What it is: the representation `SPEC-015`'s oracle will eventually assert
    against directly (`BlackLevel -> 0.0`, `WhiteLevel -> 1.0`).
  - Why rejected now: 190 MB on a 47 MP frame, on top of an already-measured
    182 MB peak, is a real cost for a representation nothing in PROJ-001 yet
    consumes. `SPEC-015` can assert `BlackLevel -> 0` / `WhiteLevel -> 1`
    against a `u16` full-scale mapping just as directly as against `f32`
    (`0` and `65535` are exactly `0.0` and `1.0` scaled) — the constraint the
    spec named as non-negotiable does not actually require `f32`.

- **Option B: `u16`, but left at the sensor's native bit depth (0..16383 for
  a 14-bit source) rather than rescaled to full 16-bit scale.**
  - What it is: normalize position within `[BlackLevel, WhiteLevel]` but skip
    the final rescale to `u16::MAX`.
  - Why rejected: a caller cannot tell "native 14-bit-range `u16`" from
    "full-scale `u16`" without also knowing the source bit depth out of band,
    which reintroduces exactly the kind of hidden per-camera assumption
    `docs/measured-q2m-dng.md`'s `Orientation` finding warned against.
    Full-scale is self-describing: `u16::MAX` always means "at `WhiteLevel`",
    regardless of source bit depth.

- **Option C (chosen): `u16`, rescaled to full scale, `DEC-016`'s
  caller-owned-buffer shape.**
  - Why selected: consistent with `DEC-016`'s no-allocation shape for
    `plane::unpack_into`; `DEC-002` stays undecided either way; `SPEC-015`'s
    stated constraint (`BlackLevel -> 0`, `WhiteLevel -> 1`) is expressible
    exactly, scaled.

- **AC2 — Option D: treat an out-of-range sample as an error.**
  - What it is: `develop_into` returns `Err` if any source sample falls
    outside `[BlackLevel, WhiteLevel]`.
  - Why rejected: it fires on **every real file** (`AC2`'s measured evidence
    above) — this would make the common case an error, not an edge case, and
    a caller has no useful recovery beyond "clamp it and retry," which the
    library can simply do itself.

- **AC2 — Option E (chosen): clamp.**
  - Why selected: matches how a levels error's own arithmetic already
    behaves (a sample at `BlackLevel - 1` is optically indistinguishable from
    one at `BlackLevel`; DNG treats the *sensor's* below-black samples as
    noise floor, not signal) and keeps the output in `[0, u16::MAX]` by
    construction, so every downstream consumer of the developed image can
    assume that range without checking.

## Consequences

- **Positive.** `develop_into` needs no allocator itself (`src`/`dst` are
  both caller-owned) and adds no new dependency on `DEC-002`'s resolution.
  Measured via `irr develop` on `L1021223.DNG` (`SPEC-014` `AC7`): peak RSS
  **275,890,176 bytes** — `DEC-016`'s already-measured 182,435,840 (file +
  raw plane) plus the 93,453,824-byte developed image (8368×5584×2), to
  within rounding. `develop_into`'s own working memory is `O(1)`.
- **Negative.** `SPEC-015`'s analytic oracle must scale its own
  `BlackLevel -> 0` / `WhiteLevel -> 1` assertions by `u16::MAX` rather than
  comparing against `0.0`/`1.0` directly — a small, one-time translation cost
  at that spec's design time, not a recurring one.
- **Neutral.** A future `f32` develop path (if PROJ-002/PROJ-003 ever need
  one) is a new function, not a breaking change to this one — the reverse of
  `DEC-016`'s own note about `unpack_into`/`unpack`.
- **Negative.** Round-to-nearest vs. truncation is exactly where `SPEC-015`'s
  analytic oracle can disagree with this module: measured on Q2M's own levels
  (`black 512`, `white 16383`), the two rules give different answers on 7,935
  of 15,872 in-range samples (50.0%). `SPEC-015` must derive its expected
  values against **round-to-nearest**, not the more obvious truncating
  formula, or its "independent" oracle will be pinned to the wrong rule on
  half the domain.

## Validation

**Right if** `SPEC-015`'s analytic oracle can assert `BlackLevel -> 0`,
`WhiteLevel -> u16::MAX` against real files without contortion, and if no
PROJ-001 consumer needs `f32` before PROJ-002's colour work does.

**Wrong if** a `u16`-native oracle assertion turns out uglier in practice
than a scaled `f32` one would have been, or if `DEC-002` lands in a way that
makes `f32`'s allocation cost a non-issue (e.g. a caller-supplied arena) —
reopen this then rather than defending it out of consistency.

## References

- Related specs: SPEC-012 (`DEC-016`, the caller-owned-buffer precedent),
  SPEC-014, SPEC-015 (the analytic oracle this constrains)
- Related decisions: DEC-002 (proposed, unresolved), DEC-004 (why no
  comparison oracle covers this), DEC-016
- `SPEC-014`'s `## Implementation Context` — the measured below-BlackLevel
  minimums and exact-WhiteLevel maximums this decision responds to
