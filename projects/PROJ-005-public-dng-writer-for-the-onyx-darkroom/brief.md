---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-005
  status: proposed
  activity: requirements
  priority: high
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-20
shipped_at: null

closed_reason: null

value:
  thesis: >
    A darkroom app that develops a Monochrom DNG must be able to hand its result back as
    a DNG that Lightroom, darktable and Adobe's own validator accept. A public,
    feature-gated, clean-room writer built on STAGE-013's proven test core lets onyx's
    Darkroom v1 do that without a second RAW library.
  beneficiaries:
    - "onyx (the Monochrom app) — its PROJ-002, Darkroom v1, writes DNG (onyx, 2026-11-20)"
    - "crustyimg — RAW-in/DNG-out, the consumer STAGE-017 originally expected"
    - "The maintainer — a DNG round trip from his own files"
  success_signals:
    - "onyx Darkroom v1 writes DNGs through the public `write` feature"
    - "Written files round-trip bit-exact through irradiance and pass dnglab, exiftool and dng_validate"
    - "The default build gains nothing: `write` is additive, and the crate keeps zero runtime dependencies"
  risks_to_thesis:
    - "onyx's answers (monorail-000/DEC-075) settle scope, but two of irradiance's answers back still need onyx's agreement: the EXIF offset contract (DEC-072, also diamond-exif's) and framing as tags (DEC-073)"
    - "Linear DNG export also waits on diamond-exif's block serialiser, which irradiance does not control"
    - "Lossless compression, CFA or float could each add a stage if a later consumer asks"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap: []
---

# PROJ-005: Public DNG writer for the onyx Darkroom

> **PROPOSED. STAGE-017 re-framed 2026-11-20 from onyx's answers** (`monorail-000/DEC-075`).
> Created on 2026-11-20 so STAGE-017 (the public DNG writer) has an owner once PROJ-004
> closes. Project status stays `proposed` until jysf activates it; DEC-071..073 are
> `proposed` until accepted.
>
> The ID is the next free one. `docs/future-plans.md` uses "PROJ-005" for the spectral
> monochrome mixer, which was never minted; that plan takes a later number.

## What This Project Is

A public DNG writer behind the opt-in `write` feature (DEC-055), built on the clean-room
test-only writer that STAGE-013 proved (`tests/support/dng_writer.rs`), and sized to what
onyx's Darkroom v1 actually writes.

## Why Now

On 2026-11-20 onyx answered whether it needs a DNG writer: not for its PROJ-001 (Sensor
Inspector), which only reads, but yes for its PROJ-002 (Darkroom v1), which is in the near
future. That is the named consumer STAGE-017 was waiting for. The writer was moved out of
PROJ-004 so PROJ-004 can close and higher-priority work can start sooner.

## Success Criteria

See `value.success_signals`. onyx answered the six questions below on 2026-11-20 with
`monorail-000/DEC-075`: one monochrome u16 plane, samples and levels unchanged,
`OpcodeList3` kept, an optional caller JPEG preview, uncompressed, no float, an opaque
EXIF block from `diamond-exif` plus XMP, and a bytes-in/bytes-out caller-buffer API.
STAGE-017 traces each answer to a DEC and a spec. The questions were:
1. What Darkroom v1 writes: the original sensor plane with corrected metadata, a linear DNG
   of a processed image, or a result built from several frames.
2. Whether it needs embedded previews.
3. Whether it needs compression (lossless JPEG is patent-clean, DEC-035, but irradiance has
   no encoder).
4. Whether it needs floating-point samples.
5. Whether EXIF and other metadata are carried over from a source file.
6. The API shape the Swift bridge prefers (caller-supplied buffer or an owned `Vec`).

## Scope

### In scope
- STAGE-017 and SPEC-061, moved from PROJ-004 on 2026-11-20.
- Whatever onyx's answers add, framed as stages here.

### Explicitly out of scope
- Any I/O, CLI or new runtime dependency (`library-not-application`).
- Lossy or patent-uncertain codecs (DEC-055 split trigger 1).

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-017 (proposed) — Public DNG writer behind a `write` feature: SPEC-061 core, SPEC-081 EXIF/XMP placement, SPEC-082 preview (outlines). Moved from PROJ-004 and re-framed from `monorail-000/DEC-075` on 2026-11-20.
- [ ] (release) — a release spec carrying `write`, framed when STAGE-017's backlog ships.

**Count:** 0 shipped / 0 active / 1 pending

## Dependencies

### Depends on
- PROJ-004 STAGE-013: the clean-room writer core and its verification plan (shipped).
- PROJ-004 STAGE-025: the local `dng_validate` reference (DEC-069 §6), for acceptance.
- External inputs (not dependencies): onyx's answers, `monorail-000/DEC-075`, and its answer to the STAGE-017 framing, `monorail-000/DEC-076`.

### Enables
- onyx Darkroom v1's DNG output; crustyimg RAW-in/DNG-out.

## Project-Level Reflection

*Filled in at close (Prompt 1e).*

- **Did the thesis hold?** <yes | partly | no | too-early>
- **What changed between brief and reality?** <...>
- **What carries to the next project?** <...>
