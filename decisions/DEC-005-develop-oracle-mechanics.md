---
insight:
  id: DEC-005
  type: decision
  confidence: 0.80
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-18
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - docs/oracle-contract.md

tags:
  - oracle
  - develop
  - dnglab
  - spike-001
---

# DEC-005: The develop oracle reads `--srgb` as P5, and its tolerance is SSIMULACRA2 ≥ 85

## Decision

Two things, decided together because the second is meaningless without the first:

1. **`dnglab analyze --srgb` output is read as a `P5` 16-bit grayscale PNM, by
   overriding the header dnglab writes.** It is documented as a TIFF and is not
   one; on a monochrome file it emits a **`P6` (RGB) header over a `P5`
   (grayscale) payload**. The oracle harness rewrites the header to
   `P5 <w> <h> 65535` and consumes the payload directly.

2. **The develop-layer tolerance is SSIMULACRA2 ≥ 85**, and it is scoped: it
   verifies **geometry and gross tonal correctness only**. Levels are **out of its
   scope** and belong to `DEC-004`.

## Context

### The format defect

`dnglab analyze --srgb` on a Q2 Monochrom file produces exactly
`19 + w*h*2` bytes — a 19-byte `P6 8368 5584 65535\n` header followed by
`w*h*2` bytes of payload, when a `P6` at that size and depth requires `w*h*3*2`.
**One third of the declared data.** Reproduced twice at byte-identical size.

Any conforming PNM reader either errors on truncation or reads garbage, so oracle
layer 3 **as written in `docs/oracle-contract.md` does not work on the files this
project targets.** Rewriting the header to `P5` yields a valid 16-bit grayscale
image, verified to decode as a real photograph (mean 0.237, stddev 0.114).

This is dnglab's bug, on the monochrome path. We work around it rather than
waiting for a fix, because dnglab is a *tool we run*, never a dependency
(`no-copyleft-dependencies`), and pinning our schedule to someone else's release
is worse than a one-line header rewrite. The workaround must be **loud** — if a
future dnglab emits a correct `P6`, blindly forcing `P5` would silently halve the
image. So the harness asserts the payload length matches `w*h*2` before
overriding, and fails otherwise.

### The tolerance

**85 was pre-registered in writing before any score was computed**, together with
its falsifier: *a missing WarpRectilinear must land far below 85, or the metric
cannot catch what we most need it to.* Both survived contact with the data.
Calibration against known perturbations of dnglab's own render (quarter
resolution; scores are resolution-dependent, so treat as relative):

| perturbation | score | at ≥85 |
|---|---|---|
| identical | 100.00 | sanity check passes |
| gamma 1.01 | 95.03 | passes |
| gamma 1.05 | 88.51 | passes (just) |
| 1-pixel shift | 62.96 | **caught** |
| missing warp | −68.05 | **caught, emphatically** |

85 sits in a genuinely useful place: **geometry errors cannot hide** (a
*one-pixel* shift scores 63), while honest tone-curve implementation differences
up to roughly gamma 1.05 still pass. That is the correct shape — geometry must be
exact, tone may differ slightly between implementations.

## Alternatives Considered

- **Option A: demand ≥ 95, or 100.**
  - Why rejected: gamma 1.01 — a difference no one could see — already costs 5
    points. A bar at 95 fires on legitimate implementation divergence in the tone
    curve while still missing a +64 levels error. It would train us to ignore it.

- **Option B: wait for dnglab to fix the PNM header.**
  - Why rejected: it blocks STAGE-003 on someone else's release cycle, for a
    defect a one-line header rewrite fully neutralises.

- **Option C: parse dnglab's output with an image library.**
  - Why rejected: the file is *malformed*; a conforming library is precisely what
    fails on it. And `no `image` crate` is a standing constraint
    (`library-not-application`) — though this is test-side, reaching for a decoder
    to read a known-shape raw payload is unnecessary either way.

## Consequences

- **Positive.** STAGE-003's develop oracle is implementable now, with a
  calibrated, pre-registered threshold rather than a number picked after seeing
  the results.

- **Negative.** We depend on a **workaround for someone else's bug**, which is a
  standing liability: a dnglab upgrade could change the output shape and silently
  invalidate the harness. Mitigated by asserting the payload length before
  overriding the header, and by DEC-003's pinned `raw_checksum` catching dnglab
  behaviour changes generally.

- **Negative.** The tolerance is calibrated at quarter resolution. Absolute
  SSIMULACRA2 scores are resolution-dependent, so the 85 figure should be
  re-confirmed at full resolution when a real render exists. Confidence is 0.80
  rather than higher for exactly this reason.

- **Neutral.** `docs/oracle-contract.md`'s layer-3 row is now wrong as written and
  is corrected alongside this decision.

## Validation

Right if the first real develop implementation scores ≥ 85 against
`dnglab --srgb` without the threshold being renegotiated. **If it scores below
85, the render is wrong — the bar does not move.** That was the point of
registering it in advance.

Revisit if: a dnglab release changes the `--srgb` output shape (the length
assertion will say so); or the full-resolution re-confirmation lands materially
different from the quarter-resolution calibration.

## References

- Evidence: `spikes/done/SPIKE-001-*.md`, session of 2026-08-18
- Companion: `DEC-004` — levels are analytic and explicitly outside this oracle's scope
- Corrected by this decision: `docs/oracle-contract.md` layer 3
