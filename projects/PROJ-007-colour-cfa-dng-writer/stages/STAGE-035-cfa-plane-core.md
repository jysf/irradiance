---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-035                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-007                      # parent project
repo:
  id: irradiance

created_at: 2026-11-27
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "The writer can write a colour raw file at all: a 2×2 Bayer CFA plane with the colour tags a reader needs to develop it, re-wrapped from any held colour DNG bit for bit"
  delivers:
    - "An optional CFA value on `WriteOptions`: `PhotometricInterpretation` 32803, `CFARepeatPatternDim`, `CFAPattern`, `CFAPlaneColor` and `CFALayout` 1. With `None`, the mono write is byte-identical to 0.2.7"
    - "The colour tags as typed options, copied verbatim and shape-checked as DEC-078 reads them: `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `ForwardMatrix1/2`, `CameraCalibration1/2` and their signatures, `AnalogBalance`, `AsShotNeutral`. `ColorMatrix1` is required for a CFA write"
    - "Per-channel levels under a 2×2 `BlackLevelRepeatDim`, and a RATIONAL `BlackLevel` (the Pixel 7 Pro) written exactly"
    - "All three framings (DEC-073) with the phase kept by the `ActiveArea` origin rule (DEC-040), proven on a synthetic odd-origin fixture"
    - "The rendering and profile tags the held files carry, as typed options, plus `LinearizationTable`"
    - "`NewRawImageDigest` computed by the writer over the written samples, or not written; never copied from a caller"
    - "`WriteOptions` from a `Sensor`, for mono and CFA (promoted from future-plans extension 1)"
  explicitly_does_not:
    - "Any pattern other than 2×2 three-colour (STAGE-036)"
    - "Compression or tiles (STAGE-037)"
    - "Compute, convert or default any colour tag; read any non-DNG input"
    - "Rewrite `CFAPattern` or an opcode list; cut anywhere but the `ActiveArea` edge"
    - "Carry tags outside the typed set (`DNGPrivateData`, MakerNote, `OriginalRawFileData`, unknown tags): they are dropped, and the docs say so"
    - "Decide a synthetic file's camera name or mark it synthetic; that is the caller's policy"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-035: CFA plane core

## What This Stage Is

The public writer learns to write a Bayer CFA plane. A caller passes a `u16` mosaic, its
2×2 pattern, its levels and its colour tags. The caller gets back an uncompressed DNG that
re-parses to the same samples and tags bit for bit, and develops through irradiance's colour
path to the same bytes as the source. Adobe's validator accepts it with no warnings.
Everything else carries over unchanged from the mono writer: framing as tags, opcode-list
pass-through, the EXIF block, XMP, descriptive tags, rendering hints and the caller's
preview.

## Why Now

It is the first stage because everything else builds on it. The demo's synthetic Bayer file
and every later converter need exactly this. The held colour DNGs give it an exact oracle
(the source file) that no other stage has.

## Success Criteria

- **Round trip.** At each framing, every held colour DNG the reader decodes is written and
  re-parsed. Samples (over the written region), levels, CFA tags, colour tags, opcode-list
  bytes, `Orientation` and crop match the source bit for bit. That is the Canon 5D3, the
  Nikon D300S and the Pixel 3a/4a/7 Pro, plus the Leica M10 and Q2 once PROJ-006 STAGE-034
  lands.
- **Develop equality.** `develop_color_into` on the written file equals the source's develop
  at the same framing. The Pixels' `GainMap` (`OpcodeList2`) and the Q2's `WarpRectilinear`
  are included.
- **Acceptance.** dng_validate 1.7.1 gives no warnings on every output. exiftool and dnglab
  read them.
- **Phase.** On a synthetic file with an odd `ActiveArea` origin, both cutting framings keep
  every pixel's colour. A red-proof shows that a writer which re-phased the pattern (or
  ignored the origin) fails that develop.
- **Digest.** A written `NewRawImageDigest` is the one dng_validate computes for the file, on
  held re-wraps and on a processed plane. A red-proof shows that a digest copied from the source
  onto changed samples fails dng_validate.
- **Mono unchanged.** Every existing mono output hash is the same as 0.2.7's.
- **Panic-freedom, determinism, semver.** As for STAGE-017: the fuzz target covers the CFA
  and colour options, identical input gives identical bytes, and new types are
  `#[non_exhaustive]`.

## Scope

### In scope
- The CFA options value, the four CFA tags, and 2×2 three-colour patterns (any phase).
- The colour tags and the held files' rendering/profile tags, as typed options.
- Rational and per-channel levels.
- Framing, preview, metadata and opcode lists: re-checked with a CFA plane, unchanged in
  behaviour.

### Explicitly out of scope
- Everything in `value_contribution.explicitly_does_not`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [M] CFA plane core: the CFA options value on `WriteOptions` (`None` is the 0.2.7 mono write), `PhotometricInterpretation` 32803, `CFARepeatPatternDim`/`CFAPattern`/`CFAPlaneColor`/`CFALayout` 1 for 2×2 three-colour patterns, pattern checks in `plan`, per-channel and RATIONAL `BlackLevel`, all three framings, a synthetic odd-`ActiveArea` red-proof, mono output hashes pinned, fuzz target widened, round trip and dng_validate on held colour DNGs. A DEC for the API shape and the new checks' order. Design probe: the minimal tag set dng_validate accepts without warnings
- [ ] (not yet written) — [M] Colour tags as typed options: `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `ForwardMatrix1/2`, `CameraCalibration1/2` and both signatures, `AnalogBalance`, `AsShotNeutral` (or `AsShotWhiteXY`, never both), checked to DEC-078's shapes and `ColorMatrix1` required for CFA; develop equality through `develop_color_into` on held files at each framing. A DEC for which tags are required and the refusal order
- [ ] (not yet written) — [S] Rendering and profile tags the held files carry: `DefaultScale`, `BestQualityScale`, `BayerGreenSplit`, `AntiAliasStrength`, `ShadowScale`, `NoiseProfile`, `ProfileName`, `ProfileEmbedPolicy`, and the opaque profile tables (`ProfileHueSatMap*`, `ProfileLookTable*`, `ProfileToneCurve`) as length-checked payloads. **May be M** if the tables need per-tag shape rules; split at design if so

- [ ] (not yet written) — [S] `WriteOptions` from a `Sensor` (mono and CFA): geometry, levels, CFA, colour and rendering tags and opcode lists filled from a parsed source at a framing, never the digest or private tags. A DEC for the policy calls (white level when absent, `BlackLevelRepeatDim` narrowing, masked areas only at full sensor, malformed tags left out as DEC-078 reads them). The re-wrap oracle then uses it
- [ ] (not yet written) — [S] STAGE-035 follow-ups, bundled: whatever the verifies find, as SPEC-094 did. Reserve, not a known spec

**Count:** 0 shipped / 0 active / 5 pending

**Estimate.** Specs about 100M (35 + 25 + 15 + 15 + 10), orchestration about 65M: about 165M / $120.

## Design Notes

- Additive by construction: a mono `WriteOptions` (no CFA value) must take no new branch
  that could change its bytes. The core spec pins every existing mono output hash before
  any `src/` edit (the SPEC-082 lesson).
- The phase rule is DEC-040's, in the writing direction. `CFAPattern` is always the
  source's, and it is relative to the `ActiveArea` origin. No held file has an odd origin,
  so the odd-origin fixture is synthetic, built in `tests/support`.
- DEC-078's shape rules are the reader's. Mirroring them at write time means a file the
  writer accepts is never one the develop then calls malformed. One test per rule pairs the
  write refusal with the read refusal.
- The demo writes this stage's files too. Its tags (a chosen `ColorMatrix1`, an
  `AsShotNeutral`, its levels) come from its own inverse pipeline. The writer takes them like
  any other caller's.
- **Added 2026-11-28, from the brief's "Considerations" list:**
  - **Camera name.** Lightroom looks up its own profile by `UniqueCameraModel`, so a file that
    names a real camera renders with Adobe's profile, not the caller's matrices. The module
    docs say so, and the demo uses an invented name for invented sensors.
  - **Digest.** The colour-tags spec's DEC settles `NewRawImageDigest`: computed by the writer
    (MD5, `tests/support/md5.rs` as the oracle's reference) or not written, and never taken from
    the caller. `RawDataUniqueID` is the caller's, since it identifies the capture, not the bytes.
  - **Dropped tags.** The core DEC states that tags outside the typed set are dropped, naming
    `DNGPrivateData`, MakerNote and `OriginalRawFileData`. Adobe's converter keeps MakerNotes.
  - **`LinearizationTable`** joins the rendering-tags spec (the reader already parses it).
  - **Triple-illuminant** tags (DNG 1.6) are out; no held file carries them.
  - **wasm.** The core spec adds a CFA write to `wasm-check`'s build (SPEC-096's gate).
- The [M]/[M]/[S] split follows SPEC-061/SPEC-081: the core plus required tags first, then
  the wider tag set.

## Dependencies

### Depends on
- PROJ-005 STAGE-017: the public mono writer.
- PROJ-002 / DEC-040 / DEC-078: the colour develop, the phase origin and the tag shapes.
- PROJ-006 STAGE-034 (in flight), only for the two Leica files in the oracle set.

### Enables
- STAGE-036 (non-Bayer patterns), STAGE-037 (compressed CFA), the demo's Bayer file.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?**
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
