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
    - "Preview-slice accessors and `WarpRect::lerp` + two framing modes ship with analytic oracles and red-proofs (the raw-ADU histogram/zone kernel moved to onyx's mono-instruments, onyx DEC-010)"
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
- [x] STAGE-015 (shipped 2026-11-11) — Tiled DNG plane reader: uncompressed and lossless-JPEG tiles (SPEC-058). It unlocks the real GainMap files.
- [ ] STAGE-016 (proposed) — OpcodeList2 GainMap: parse and apply on the mono and colour paths (SPEC-059 analytic, SPEC-060 real-file)
- [ ] STAGE-017 (proposed) — Public DNG writer behind a `write` feature (SPEC-061). Design waits for a named consumer.
- [ ] STAGE-004 (proposed) — crustyimg integration: `raw-develop` feature, adapter, metadata truthfulness (moved from PROJ-001 2026-11-11; implemented crustyimg-side as its PROJ-016/STAGE-056)
- [x] STAGE-018 (shipped 2026-11-12) — Instrument layer A: preview slice accessors (SPEC-062 shipped, DEC-057; 25.00M / $18.87, verify punch list fixed in PR 209). It is first in the order agreed with monorail-000 / onyx-000 on 2026-11-12. The ADU histogram/zones and bad-pixel listing from future-plans Stage A are not planned here. onyx computes the histogram itself (onyx DEC-010), so crustyimg STAGE-062 is the only possible caller left.
- [ ] (parked) — Instrument layer B: optical black. Its only named caller, the onyx / Monochrom app, now computes the optical-black reductions itself (onyx DEC-010, 2026-11-12). Revisit only if irradiance's own develop needs optical-black-based black subtraction.
- [ ] (unframed, low priority) — `MaskedAreas` pass-through: read DNG tag 50830 into a new `Sensor` field, rectangles verbatim, no strip naming, no develop change. onyx request 5, item 8 in `feedback/2026-11-11-monorail-000-requests.md`. No held DNG has the tag, so fixtures come from the SPEC-057 writer. Fits STAGE-014 (0.2.x maintenance) or beside Stage C. `[XS]`/`[S]`.
- [ ] STAGE-019 (active, framed 2026-11-12) — Instrument layer C: develop options, warp alpha, caller scratch, CFA-as-mono, blocked orientation. Five specs: SPEC-063 `DevelopOptions` + `develop_with_into` [M] **shipped 2026-11-12** (DEC-058); SPEC-064 `warp_alpha` / `WarpRect::lerp` [M] **shipped 2026-11-12** (DEC-059; FullActiveArea now develops every Q2M frame); SPEC-065 caller scratch [M] **shipped 2026-11-13** (DEC-060; `develop_with_scratch_into` allocates nothing, `develop_into` now allocates once); SPEC-066 CFA-as-mono [M] **shipped 2026-11-13** (DEC-061; opt-in `BlackLevels::PerPosition` honours `BlackLevelRepeatDim`; default unchanged); SPEC-067 tiled Orientation 5–8 gather [S] **shipped 2026-11-13** (DEC-062; byte-identical; warped O6 785 -> 630 ms against O1 615). All five specs shipped; stage close pending. monorail-000 requests 2, 3, 6, 4 plus future-plans Stage C geometry; mono path only; the displacement-field sampler is not planned (no caller).
- [ ] (not yet framed) — Parallel-friendly plane decode: `plane::units` + `unpack_unit_into` so the caller can parallelise unpack (monorail-000 request 1). The real win is row bands in uncompressed strips. The held Compression 7 files are one strip each, with no restart markers, so they can't be split; speeding those up means faster single-stream lossless-JPEG decode. Depends on SPEC-058.

Spec chain: SPEC-057 → (SPEC-058, SPEC-059, SPEC-061); SPEC-060 depends on SPEC-058 + SPEC-059.

**Count:** 1 shipped / 1 active / 5 pending (+3 unframed)

Consumer requests feeding the unframed stages: `feedback/2026-11-11-monorail-000-requests.md` (triaged; nothing minted).

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
