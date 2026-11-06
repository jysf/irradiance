---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-037
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-003
repo:
  id: irradiance

created_at: 2026-11-05
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md
  - projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md
  - src/ljpeg.rs

tags:
  - sof-3
  - ljpeg
  - multi-component
  - column-interleave
  - stage-011
  - re-plan
---

# DEC-037: Decode the held Compression-7 files' two-component (Nf=2) column-interleave SOF-3

## Decision

Grow the SOF-3 decoder to **two-component (`Nf=2`) column-interleave** so both held
Compression-7 DNGs decode to one monochrome `u16` plane — reversing STAGE-009's
single-component-only scope **for the monochrome column-interleave case**. This is a
new STAGE-011 spec (`SPEC-046`), sequenced **before** the bit-exact oracle (`SPEC-044`).
Bayer / CFA *colour* multi-component interleave (3+ colour components in one frame)
stays OUT of scope — no held colour file, no bit-exact oracle.

## Context

STAGE-009 shipped a single-component (`Nf=1`) SOF-3 decoder, and the brief deferred
"multi-component" wholesale citing "no held Bayer-in-LJ92 file." That conflated two
different things and rested on an unverified assumption: the brief called both held
Compression-7 files "monochrome single-component", but only the *sensor* is monochrome.
`SPEC-043`'s build probed the real SOF3 frame headers — the design §12 probe had
measured strip LAYOUT via `irr ifd`, not the frame header inside the strip — and found
both held files are **two-component**:

- `LEICA-M-MONOCHROM-TYP246/M2462362.DNG`: P=12, lines=4000, samples_per_line=2992, **Nf=2** (≈ half the 5976 crop width).
- `PENTAX-K3III-MONO/K3III.DNG`: P=14, lines=4224, samples_per_line=3152, **Nf=2** (≈ half the 6192 crop width).

This is the DNG monochrome column-interleave encoding: one monochrome plane stored as
two SOF-3 components carrying alternating columns. The single-component decoder returns
`LjpegUnsupportedComponentCount { nf: 2 }` for both (panic-free), so it decodes
**neither** held file — and `SPEC-044`'s premise ("both held files decode bit-exact vs
dnglab") is void. (Independently confirmed: the orchestrator re-parsed both SOF3 markers.)

## Alternatives Considered

- **A: keep single-component only; park the bit-exact oracle until a single-component
  Compression-7 file is acquired** (the SPEC-035-waits-on-the-chart pattern).
  - Rejected: column-interleave is the *common* monochrome-DNG encoding (both held files
    use it; a genuinely single-component monochrome Compression-7 DNG may not be readily
    acquirable), so parking defers the thesis indefinitely with no clear unblock.
- **B: treat all multi-component as out of scope (status quo).**
  - Rejected: decodes zero held files and defeats PROJ-003's thesis (develop the
    compressed files actually held).
- **C (chosen): decode the monochrome two-component column-interleave (`SPEC-046`); keep
  Bayer / CFA colour multi-component out.**
  - Both held files decode bit-exact (restoring `SPEC-044`'s target); the genuinely
    oracle-less colour-mosaic path stays deferred, honestly.

## Consequences

- **Positive.** Both held Compression-7 files become decodable → `SPEC-044`'s bit-exact
  `dnglab` oracle has a real target again, and PROJ-003's thesis is reachable with the
  inputs in hand. STAGE-011's backlog gains `SPEC-046` (Nf=2 decode + column
  de-interleave) before `SPEC-044`.
- **Neutral.** The decoder grows a two-component scan path (per-component predictor
  state + column de-interleave into one plane); it stays `pub(crate)`, panic-free,
  zero runtime dependency, five-lint clean.
- **Negative — the honest cost.** The single-component boundary STAGE-009 shipped is
  partially reopened, and the design §12 probe that should have read the SOF3 header did
  not — a mid-stage re-plan. Recorded as the signal
  `probe-the-decode-input-not-just-the-container`.

## Validation

Right if `SPEC-046` decodes both held files to a monochrome `u16` plane whose MD5 equals
`dnglab analyze --raw-checksum` (`SPEC-044`), with an analytic two-component round-trip
red-proof, panic-free. Revisit if a held file turns out to need more than two components
or a non-column interleave the de-interleave assumption does not cover.

## References

- Triggered by `SPEC-043` build (the `stage-011-held-files-are-two-component-sof3`
  question in `guidance/questions.yaml`) and confirmed by re-parsing both SOF3 markers.
  Governs `SPEC-046`; re-scopes `SPEC-044` / `SPEC-045`; corrects the PROJ-003 brief's
  premise; amends STAGE-009's single-component scope for the monochrome
  column-interleave case. Builds on `DEC-035` (clean-room / patent-clean) and `DEC-016`
  (caller-owned buffer).
