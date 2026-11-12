---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-004
  status: proposed
  activity: requirements
  priority: medium
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: null

closed_reason: null

value:
  thesis: >
    The developed image is not the only thing a consumer needs from a RAW
    library. A read-only, zero-I/O, caller-allocated instrument surface over the
    same parsed file lets two real consumers build on irradiance without forking
    it: crustyimg's raw-develop and sensor-forensics stages, and the Monochrom app's
    sensitometer and optics benches. The surface is preview slices, raw-ADU
    histograms and zones, optical black, and warp/framing geometry.
  beneficiaries:
    - "crustyimg — its PROJ-016 STAGE-056 (raw-develop, truthful RAW metadata) and STAGE-062 (sensor forensics) consume this API"
    - "The Monochrom app (iPadOS/macOS) — its SENSITOMETER and OPTICS & PLANE benches"
    - "The maintainer — sensor-level evidence about his own files, not just a picture"
  success_signals:
    - "crustyimg's `raw-develop` feature builds against a published irradiance and `crustyimg web q2m.dng` produces a publishable image (carried from PROJ-001)"
    - "`crustyimg info` on a DNG reports the sensor's real dimensions, bit depth, orientation and EXIF from IFD0, not the preview's (carried from PROJ-001)"
    - "Preview-slice accessors, the raw-ADU histogram/zone kernel and `WarpRect::lerp` + two framing modes ship with analytic oracles and red-proofs"
    - "Every new API is zero-I/O and caller-allocated; no new runtime dependency"
    - "Tiled DNGs decode bit-exact against dnglab (the three held Pixel files), and their mandatory OpcodeList2 GainMaps are applied instead of refused"
    - "A clean-room DNG writer builds every fixture the oracles need, and its public `write` form is accepted by dnglab, exiftool and dng_validate"
  risks_to_thesis:
    - "Consumer timing: crustyimg's STAGE-056 is unscheduled (proposed) on its side; the integration success signals depend on another repo's calendar"
    - "Speculative API: building the instrument surface ahead of a consumer that actually calls it; each stage should wait for a named caller"
    - "Optical-black modes need an opcode/masked-column inventory spike first (docs/future-plans.md); without it Stage B is guesswork"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap: []
---

# PROJ-004: Instrument layer and consumer integration

> **PROPOSED. Not yet framed.** This project was created at PROJ-001's close
> (2026-11-11) so STAGE-004 (crustyimg integration) has an owner after PROJ-001
> ships. Run Prompt 1a/1b (project frame + brief) before any stage here goes
> active. The detailed plan for the instrument layer is in
> `docs/future-plans.md` § PROJ-004.

## What This Project Is

The consumer-facing surface beyond `develop_*`, in two parts:

- **Consumer integration.** STAGE-004 is carried over from PROJ-001. The work is
  crustyimg-side: its PROJ-016/STAGE-056 implements it. irradiance's job is to be a
  consumable, truthful dependency and to answer API needs as they arise.
- **Opcode coverage and DNG writing.** STAGE-013, 015, 016 and 017 are framed 2026-11-11:
  - a test-only writer that builds fixtures (mono, Bayer, strips, tiles, arbitrary
    opcode lists);
  - a tiled-DNG reader;
  - OpcodeList2 GainMap, the lens-shading map Android phones write as mandatory;
  - a public writer behind a `write` feature.

  Real GainMap files exist: the Pixel 4a, 3a and 7 Pro in the raw.pixls.us mirror
  (`data.lfs`). All of them are tiled, which is why the reader comes first.
- **The instrument layer** (stages to be framed), per `docs/future-plans.md`:
  - **A. Forensics:** preview-slice accessors, the raw-ADU histogram and zones,
    and bad pixels only if the spike justifies them.
  - **B. Optical black:** spike-gated.
  - **C. Geometry:** `WarpRect::lerp` and two framing modes.

## Why Now

PROJ-001 is closing with its consumer-integration stage undone. The reason is not
a gap in irradiance: the work lives in crustyimg, and crustyimg hasn't scheduled
it yet. Moving the stage here keeps it owned. It sits beside the instrument API
that crustyimg's next RAW stage (STAGE-062: `WarpRect::lerp`, the preview accessor
and the ADU histogram) will ask for.

## Success Criteria

See `value.success_signals`. These are refined at frame time.

## Scope

### In scope
- STAGE-004, which tracks and supports crustyimg's `raw-develop` integration.
- The instrument-layer stages A–C from `docs/future-plans.md`, framed later.

### Explicitly out of scope
- Any I/O, CLI, or `image`-crate dependency (`library-not-application`).
- The 8-bit pipeline or LUT work, which belongs to crustyimg.
- Colour accuracy (PROJ-002 SPEC-035).

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

Order agreed with the maintainer 2026-11-11: writer → tiled reader → GainMap → public writer → instrument stages.

- [x] STAGE-013 (shipped 2026-11-11) — Test-only DNG writer, clean-room, uncompressed (SPEC-057; DEC-055). It is the fixture source for every stage below.
- [ ] STAGE-015 (proposed) — Tiled DNG plane reader: uncompressed and lossless-JPEG tiles (SPEC-058). It unlocks the real GainMap files.
- [ ] STAGE-016 (proposed) — OpcodeList2 GainMap: parse and apply on the mono and colour paths (SPEC-059 analytic, SPEC-060 real-file)
- [ ] STAGE-017 (proposed) — Public DNG writer behind a `write` feature (SPEC-061). Design waits for a named consumer.
- [ ] STAGE-004 (proposed) — crustyimg integration: `raw-develop` feature, adapter, metadata truthfulness (moved from PROJ-001 2026-11-11; implemented crustyimg-side as its PROJ-016/STAGE-056)
- [ ] (not yet framed) — Instrument layer A: forensics (preview slices, ADU histogram/zones)
- [ ] (not yet framed) — Instrument layer B: optical black (spike-gated)
- [ ] (not yet framed) — Instrument layer C: geometry (`WarpRect::lerp`, framing modes)

Spec chain: SPEC-057 → (SPEC-058, SPEC-059, SPEC-061); SPEC-060 depends on SPEC-058 + SPEC-059.

**Count:** 0 shipped / 0 active / 5 pending (+3 unframed)

## Dependencies

### Depends on
- PROJ-001 (shipped): the container, plane, opcodes and develop.
- PROJ-002: published 0.2.x, for the colour metadata consumers will read.
- External: crustyimg scheduling its PROJ-016 stages.

### Enables
- crustyimg's RAW claim becoming true; the Monochrom app's instrument benches.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
