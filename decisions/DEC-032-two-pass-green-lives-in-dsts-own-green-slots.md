---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-032
  type: decision
  confidence: 0.9
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

created_at: 2026-10-31
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - src/demosaic.rs

tags:
  - demosaic
  - allocation
  - spec-031
  - hamilton-adams
---

# DEC-032: Hamilton-Adams's two passes share the green plane in `dst`'s own green slots — no scratch buffer

## Decision

`demosaic_into`'s Hamilton-Adams kernel (`SPEC-031`) is two-pass — pass 1 must
finalize green at *every* pixel before pass 2 reads interpolated green at
non-green neighbours — yet it allocates **nothing**, honouring `DEC-016`. The
complete green plane the two passes share is written into **`dst`'s own green
slots** (`dst[pixel * 3 + 1]`) by pass 1, and read back from there by pass 2
while pass 2 fills only the red and blue slots (`+0`, `+2`). The green slots
(read in pass 2, never written) and the red/blue slots (written in pass 2, never
read) are disjoint, so there is no read-after-write hazard and no aliasing: pass
1 uses `as_chunks_mut::<3>()`; pass 2 is index-based with `.get`/`.get_mut` so a
green read's shared borrow ends before a red/blue write's mutable borrow.

## Context

`DEC-016` is the shape every caller (including `crustyimg`) is built against:
`*_into(&mut [u16])`, allocator-free, so the library survives `DEC-002`'s still
open `no_std` question. SPEC-030's bilinear demosaic was single-pass and read
only `src`, so it met `DEC-016` trivially. Hamilton-Adams cannot: its red/blue
interpolation reads the *interpolated* green at neighbouring sites, which only
exists after a complete green pass. The naive implementation allocates a
`width * height` scratch green plane between passes — a second ~95 MB buffer on a
47 MP frame, allocated inside the library on the caller's behalf, exactly what
`DEC-016` exists to avoid.

The output buffer already has a `u16` green slot per pixel standing idle until
pass 2 would overwrite it — except pass 2 never overwrites green (green is final
after pass 1; pass 2 only computes the two *missing* primaries). So the output's
green channel *is* the scratch green plane, for free.

## Alternatives Considered

- **A: allocate a `Vec<u16>` green plane between the passes.**
  - Simplest to write, but it allocates inside the library (~95 MB on a real
    frame), breaking `DEC-016` and foreclosing `no_std`. Rejected.
- **B: add a caller-supplied scratch `&mut [u16]` green parameter.**
  - Keeps the library allocator-free but changes the public signature SPEC-031
    is required to leave unchanged, and pushes a demosaic-internal detail (that
    it happens to be two-pass) into every caller's call site. Rejected.
- **C (chosen): write green into `dst`'s green slots in pass 1; read them back
  in pass 2.**
  - No allocation, unchanged signature, and the disjoint-slot argument makes the
    aliasing safe and auditable. The cost is that pass 2 cannot use
    `as_chunks_mut` (it reads other pixels' chunks), so it is an indexed loop —
    a small readability cost, paid once.

## Consequences

- **Positive.** `DEC-016` holds for the quality kernel exactly as for bilinear;
  no new allocation, no signature change, `no_std` stays open.
- **Positive.** The correctness argument is local and checkable: pass 2 reads
  only index `+1` and writes only `+0`/`+2`, so no pixel's green is read after
  it could have been clobbered (it never is).
- **Neutral.** Pass 2 is an indexed `.get`/`.get_mut` loop rather than an
  `as_chunks_mut` iterator, because it reads neighbouring pixels' green slots
  while writing this pixel's red/blue.
- **Negative.** If a future kernel needed to *rewrite* green in a later pass
  (HA does not), this reuse would stop being safe and a real scratch plane (or
  option B) would be needed; that is a different algorithm and its own decision.

## Validation

Right if the bit-exact oracle (`tests/demosaic_oracle.rs`) passes with zero
allocation inside `demosaic_into` and the green channel of the output equals the
pass-1 green plane. Revisit if a demosaic kernel is added that must mutate green
after the red/blue pass, or if `DEC-002` lands on a buffer model where a
caller-supplied scratch plane is the natural shape.

## References

- Builds on: `DEC-016` (caller-owned buffer, no allocation), `SPEC-030`
  (bilinear foundation + the oracle harness), `SPEC-031` (this kernel), `DEC-031`
  (Hamilton-Adams, not MHC).
- Constraints: `no-panics-on-untrusted-input` (the `.get`/`.get_mut` pass-2 loop).
