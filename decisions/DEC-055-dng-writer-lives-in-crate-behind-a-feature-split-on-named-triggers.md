---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-055
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - docs/future-plans.md
  - projects/PROJ-004-instrument-layer-and-consumer-integration/stages/STAGE-013-test-fixture-dng-writer-clean-room-uncompressed.md

tags:
  - dng-writer
  - crate-layout
  - workspace
  - patent-isolation
  - stage-013
---

# DEC-055: The DNG writer lives in the `irradiance` crate behind a feature; split only on named triggers

> Drafted 2026-11-06 as an uncommitted "DEC-041" in the DNG-writer session; that id was
> later taken on main, so it lands here as DEC-055 (2026-11-11), unchanged in substance.

## Decision

Keep DNG writing **inside the `irradiance` crate**, not in a separate crate or
repo, and do not turn the project into a multi-crate workspace yet.

- The test-only writer (PROJ-004 STAGE-013) lives under `tests/support` and is
  not published.
- A future public writer lives in `src/dng/write.rs` behind a `write` cargo
  feature. It depends only on the container types in `src/ifd.rs`, never on the
  develop or colour code.
- `src/ifd.rs` stays free of develop and colour imports, so it can be lifted out
  as a crate later without untangling anything.

## Split triggers

Revisit this DEC and split into a facade plus children (`irradiance` as the
facade, `irradiance-container`, `irradiance-ljpeg`, further `irradiance-<codec>`
crates) when any one of these holds:

1. **Patent or licence isolation.** A codec with an uncertain or encumbered
   patent position (for example JPEG XL, or any lossy encoder) is added. It goes
   in its own opt-in crate, because a cargo feature still ships the code in the
   source tarball. Each such codec also needs its own patent DEC (DEC-035 pattern).
2. **A second consumer of the container layer** (crustyimg, a metadata tool, a
   validator) that wants TIFF/DNG parsing without the develop pipeline.
3. **Divergent release cadence**, for example a churning writer against a frozen
   develop API.
4. **A dependency** that a codec needs, which would break the zero-runtime-
   dependency promise of the main crate.

## Why not split now

- The writer shares tag constants, `ByteOrder`, `Rational` and CFA types with the
  reader. A split forces a third core crate to hold them, before the real
  boundary is known.
- The adoption pitch is "the permissive pure-Rust RAW crate"; one `cargo add` is
  the easiest story. Several crates add publish ordering, version lockstep and
  docs for a solo maintainer.
- crustyimg's bare path dependency already made it unpublishable once; a
  workspace multiplies that class of problem.
- LJ92 (SOF-3) is patent-clean (DEC-035), so nothing today needs isolating.

## Consequences

- `cargo deny` and the provenance ledger are kept per module, so they carry over
  to separate crates if a split happens.
- Keep the lossless-container property (STAGE-013, "Reader fidelity") when
  touching `src/ifd.rs`.
- Rejected: separate repo now (no consumer, doubles release overhead); a
  workspace of many small crates now (boundary unknown, speculative).
