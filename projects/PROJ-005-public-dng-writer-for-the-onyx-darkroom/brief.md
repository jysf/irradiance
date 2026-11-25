---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-005
  status: shipped
  activity: requirements
  priority: high
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-20
shipped_at: 2026-11-25

closed_reason: shipped

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
  thesis_held: partly
  signals_observed:
    - "irradiance 0.2.7 (crates.io, 2026-11-25) publishes the writer behind the opt-in `write` feature; docs.rs documents `irradiance::dng::write`"
    - "Written files re-parse bit-exact through irradiance at all three framings, exiftool reads the EXIF, XMP and preview back, and Adobe dng_validate 1.7.1 reports no warnings on every held mono file (SPEC-061, 081, 082) and on a published-crate clean-host write (SPEC-095)"
    - "The default build gains nothing: semver-checks clean against 0.2.6, `[dependencies]` still empty, and the K-3 III develops identically on 0.2.6 and 0.2.7"
  evidence: >
    Two of three success signals are met. The writer exists, is public, is
    validated by Adobe's own tool and is additive. The first signal, onyx
    Darkroom v1 writing DNGs through it, is not observed yet: onyx has not
    integrated, and its EXIF path waits on diamond-exif's block builder
    (diamond-exif/SPEC-025). The thesis also names Lightroom and darktable,
    and no open in either is recorded.
  notes: >
    The library half of the thesis held: a clean-room writer on STAGE-013's
    core, sized to one named consumer's answers, shipped in four specs and
    one release with no runtime dependency. The consumer half is too early to
    score; score it at onyx's Darkroom v1 integration. Asking the consumer six
    concrete questions before framing (monorail-000/DEC-075) is why the scope
    held after the 2026-11-20 re-frame.

roadmap: []
---

# PROJ-005: Public DNG writer for the onyx Darkroom

> **SHIPPED 2026-11-25** (irradiance 0.2.7). History: STAGE-017 re-framed 2026-11-20 from onyx's answers** (`monorail-000/DEC-075`).
> Created on 2026-11-20 so STAGE-017 (the public DNG writer) has an owner once PROJ-004
> closes. Project status stays `proposed` until jysf activates it; DEC-071..073 are
> `proposed` until accepted.
>
> The ID is the next free one. `docs/future-plans.md` uses "PROJ-005" for the spectral
> monochrome mixer, which was never minted; that plan is now PROJ-006 (2026-11-21).

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

- [x] STAGE-017 (shipped 2026-11-25, PR 342) — Public DNG writer behind a `write` feature: SPEC-061 core, SPEC-081 EXIF/XMP and descriptive tags, SPEC-082 preview, SPEC-094 follow-ups. Moved from PROJ-004 and re-framed from `monorail-000/DEC-075` on 2026-11-20.
- [x] STAGE-032 (shipped 2026-11-25, PR 348) — The 0.2.7 release (SPEC-095): the writer on crates.io, docs.rs with `write`.

**Count:** 2 shipped / 0 active / 0 pending

## Dependencies

### Depends on
- PROJ-004 STAGE-013: the clean-room writer core and its verification plan (shipped).
- PROJ-004 STAGE-025: the local `dng_validate` reference (DEC-069 §6), for acceptance.
- External inputs (not dependencies): onyx's answers, `monorail-000/DEC-075`, and its answer to the STAGE-017 framing, `monorail-000/DEC-076`.

### Enables
- onyx Darkroom v1's DNG output; crustyimg RAW-in/DNG-out.

## Project-Level Reflection

- **Did the thesis hold?** Partly. The library half held: irradiance 0.2.7 writes a
  monochrome DNG that re-parses bit-exact and that Adobe `dng_validate` accepts with no
  warnings, behind an additive feature with no dependency. The consumer half is too
  early to score: onyx's Darkroom v1 has not integrated yet, its EXIF block waits on
  `diamond-exif`'s builder, and no Lightroom or darktable open is recorded.
- **What changed between brief and reality?** Little after the 2026-11-20 re-frame, which
  cut CFA, tiles, big-endian and colour metadata and added the preview and EXIF/XMP specs.
  STAGE-017 took four specs, not three: SPEC-094 bundled the three [XS] follow-ups the
  verifies found. The release became its own stage (STAGE-032). DEC-080 (the error
  payload types always compiled) was the one design decision the build forced.
- **How did the stages go?** STAGE-017 was smooth after SPEC-061: every later spec was
  green in CI on the first run, and the verifies caught a defect in each spec except
  SPEC-081 before ship. STAGE-032 was clean through build and verify. Its one slip came at
  the cut: PR 343 merged two minutes after the prep, so its CHANGELOG entries landed under
  `[0.2.7]` (fixed by PR 347). The tag was still right, because it was cut on the prep
  merge.
- **Cost.** STAGE-017 about 148.8M tokens / $109.77, STAGE-032 about 44.6M / $30.69:
  about 193.4M / $140.46 in all (specs 107.6M / $81.13, orchestration measured where
  observable). The 2026-11-11 frame, the 2026-11-20 re-frame, SPEC-061's design and this
  close were not measured.
- **Signals dispositioned at this close** (all three `project-close` signals):
  - `fuzz-targets-without-a-ci-smoke-job` and
    `ljpeg-private-doc-links-break-the-private-items-doc-build`: **accepted**, scheduled
    together as one hygiene patch (`docs/future-plans.md`, "Later: the DNG writer"). Both
    predate PROJ-005.
  - `parallel-pr-changelog-entries-land-under-the-promoted-version`: **done** at this
    close. The release-spec template's two-phase section now says to tag the prep merge
    by SHA and to have open PRs that touch the CHANGELOG rebase their entries after the
    prep merges.
- **What carries to the next project?**
  - For onyx: a recorded Lightroom Classic, Capture One and Windows open of written files
    (DEC-082's confidence is 0.8 until then); one real export with `diamond-exif`'s block;
    and the question whether `mono-core` already builds `WriteOptions` from a `Sensor`.
  - Later, each on a consumer's request: a `WriteOptions` helper from a `Sensor`, lossless
    Compression 7 output, tiled output, CFA planes with colour tags (a raw-to-DNG
    converter, crustyimg and Sony ARW), and float or LinearRaw RGB output.
  - The two process-debt items as one hygiene patch.
