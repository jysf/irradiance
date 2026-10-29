---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-002
  status: active
  activity: design
  priority: high
  target_ship: null

repo:
  id: irradiance

created_at: 2026-10-29
shipped_at: null

closed_reason: null

value:
  thesis: >
    A permissive, pure-Rust library can develop a Bayer-mosaic DNG to a correct
    COLOUR image — the plane still bit-exact against an independent
    implementation, and colour checked by ColorChecker ΔE, the one fully
    independent oracle PROJ-001 structurally could not reach — widening the
    decoder from one monochrome Leica to every Bayer camera, which is the common
    case and the gap that keeps Rust projects on copyleft RAW crates.
  beneficiaries:
    - "The maintainer — develops his own Nikon (Bayer) files into colour web assets, not only the monochrome Leica"
    - "crustyimg — its RAW-develop path covers colour cameras (the common case), not just a monochrome edge case"
    - "Rust imaging projects still forced onto copyleft RAW crates for colour develop (oculante, czkawka, image-hdr, stardetect)"
    - "The parked linear-colour direction (astro / scientific imaging) that needs bit-exact linear sensor values with colour"
  success_signals:
    - "An uncompressed Bayer DNG (Nikon via Adobe DNG Converter) decodes to a mosaic plane whose MD5 equals `dnglab analyze --raw-checksum`, EXACTLY — the bit-exact contract survives the move to CFA"
    - "Demosaic + colour transform render a colour image within a stated ColorChecker ΔE tolerance of a measured chart, and the oracle goes RED on a deliberately wrong matrix or white balance"
    - "`info` reports the CFA pattern, per-channel black/white levels, AsShotNeutral and the colour matrices parsed from the file — truthfully, not invented"
    - "The parser still never panics on hostile or truncated CFA input (fuzz corpus extended with a Bayer file)"
    - "The demosaic and colour-matrix math each carry a provenance-ledger entry naming the source and its licence"
  risks_to_thesis:
    - "Corpus again: a ColorChecker shot is the only fully independent colour oracle. Without it, colour verification falls back to dnglab's render — single-sourced on rawler, so a tolerance test could pass while both are wrong together (PROJ-001's own standing risk, sharper in colour)"
    - "Demosaic has no bit-exact oracle — a clean-room MHC will differ from dnglab's demosaic; the tolerance band must be set from analytic fixtures honestly, or the test is theatre"
    - "Colour correctness spans white balance + matrix direction + illuminant (D50/D65) + optional ForwardMatrix; a subtle error can pass a loose ΔE while being visibly off"
    - "Nikon DNGs (via the converter) may exercise tags the Leica path never did — per-channel BlackLevel, LinearizationTable, GainMap — and pull scope the plane work did not budget for"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap:
  - item: "X-Trans (Fuji x100f) — 6×6 CFA, a harder demosaic (DHT); its own stage, not this wave"
    kind: goal
    horizon: later
  - item: "Native NEF / RAF containers + lossless-JPEG SOF-3 — the compression keystone shared by many vendor decoders"
    kind: goal
    horizon: later
---

# PROJ-002: Bayer colour develop, end to end

## What This Project Is

The second wave of `irradiance`: read an uncompressed **Bayer-mosaic** DNG and
develop it into a correct **colour** image — the sensor plane still verified
bit-exact against an independent implementation, and the colour result verified
by ColorChecker ΔE. It delivers the CFA sensor model (the colour metadata the
monochrome path never parsed), a clean-room demosaic, the white-balance and
colour-matrix transform, and the ΔE oracle that PROJ-001 structurally could not
reach.

It builds directly on PROJ-001. The container reader, the IFD model, the
packed-bits unpack, level normalization, geometry, the tone curve and the oracle
harness are all reused — colour is **additive**. The monochrome `LinearRaw` path
stays intact and untouched; a Bayer file takes a new, parallel path from the
sensor gate onward.

## Why Now

**Colour is the capability that most widens who the decoder serves.** PROJ-001
proved the thesis on one monochrome Leica. Every Nikon, Canon, Sony and Fuji is
a colour camera; a decoder that cannot demosaic is a decoder for an edge case.
Colour turns "the first permissive pure-Rust RAW decoder" from a monochrome
curiosity into something the Rust ecosystem actually lacks.

**The remaining PROJ-001 stage is blocked, and this is not.** STAGE-004
(crustyimg integration) waits on the crustyimg side, roughly two weeks out.
STAGE-005 (harness hardening) is deferred by choice. The colour design work
needs no new corpus and no crustyimg — it can proceed now, in parallel with the
wait, against the code that already exists.

**The oracle method is already proven and mostly transfers.** The bit-exact
plane contract (`dnglab analyze --raw-checksum`) applies unchanged to a CFA
mosaic — the mosaic is still a single sensor plane. Only colour needs a new kind
of check, and PROJ-001's own brief already named it: ColorChecker ΔE.

**The maintainer holds the input.** Several Nikon bodies on hand; Adobe DNG
Converter turns a NEF into an uncompressed DNG the existing parser already reads,
so the whole wave bootstraps without native NEF parsing or SOF-3 decompression.

## Success Criteria

- An uncompressed Bayer DNG decodes to a mosaic plane whose MD5 equals
  `dnglab analyze --raw-checksum`, **exactly**
- The developed colour output scores within a stated ColorChecker ΔE tolerance
  of a measured chart, and the oracle demonstrably goes **red** on a deliberately
  wrong colour matrix or white balance
- `info` reports the CFA pattern, per-channel black/white levels, AsShotNeutral
  and the colour matrices parsed from the file — not invented defaults
- The parser never panics on the fuzz corpus or on truncated/hostile CFA input
- The demosaic and the colour transform each carry a provenance-ledger entry
- The same Bayer DNG produces byte-identical **plane** output on macOS and Linux
  (colour output within the stated ΔE tolerance on both)

## Scope

### In scope
- CFA sensor identity + model: accept `PhotometricInterpretation == CFA (32803)`;
  parse `CFAPattern` / `CFARepeatPatternDim` / `CFAPlaneColor`, per-channel
  `BlackLevel` / `WhiteLevel`, `AsShotNeutral`, `ColorMatrix1/2`,
  `CalibrationIlluminant1/2`, and `ForwardMatrix1/2` when present
- Mosaic-plane unpack (reuse the packed-bits unpack; the plane is still one `u16`
  buffer, now a mosaic) and the bit-exact plane oracle against a Bayer file
- Clean-room **Malvar-He-Cutler** demosaic: CFA mosaic → camera-space linear RGB
- Colour transform: white balance (`AsShotNeutral`) → camera RGB → XYZ /
  linear-sRGB via `ColorMatrix` / `ForwardMatrix` → display encode (reuse `tone`)
- A 3-channel colour develop path (new `develop_color_into` or a widened
  signature); the mono path stays intact
- ColorChecker ΔE oracle + analytic CFA fixtures; the fuzz corpus extended with a
  Bayer file
- Provenance-ledger entries for the demosaic and the colour-matrix math

### Explicitly out of scope
- **X-Trans / non-Bayer CFA** (the Fuji x100f) — a harder 6×6 demosaic, its own stage
- **Native vendor containers** — no NEF, RAF, CR2, ARW; bootstrap via uncompressed DNG
- **Lossless JPEG SOF-3** — the compression keystone, its own project
- **Highlight reconstruction, denoise, advanced rendering intents** — an unbounded
  tuning loop against a moving target
- **A CLI** — `irradiance` stays a library; `irr` remains the internal dev/oracle binary

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-006 (active) — CFA sensor model: identify + parse Bayer DNGs, unpack the mosaic, verify the plane bit-exact vs `dnglab`
- [ ] (not yet defined) — Demosaic: clean-room Malvar-He-Cutler, mosaic → camera RGB, analytic-fixture + tolerance oracle
- [ ] (not yet defined) — Colour transform + output: white balance, colour matrix, display encode, ColorChecker ΔE

**Count:** 0 shipped / 1 active / 2 pending

Each stage ends at a **stopping point** where the maintainer reviews before the
next begins. Effort estimates live in each stage file.

## Dependencies

### Depends on
- **PROJ-001** (shipped core): container reader, IFD model, packed-bits unpack,
  level normalization, geometry, tone curve, and the three-layer oracle harness —
  colour is additive on top of all of it
- External: `dnglab` (Homebrew, LGPL-2.1 — **run as a tool, never linked**) for the
  plane checksum and the reference render
- External: the Adobe DNG Specification — the CFA tags and colour-matrix model,
  implemented from the published spec
- Hardware/input: a Nikon body + Adobe DNG Converter (held); ideally a ColorChecker
  chart for the ΔE oracle

### Enables
- crustyimg developing colour cameras — the common case, not a monochrome edge case
- X-Trans and native vendor containers as later waves
- The permissive **colour** RAW primitive the Rust ecosystem currently lacks

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
