---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-017                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-005                      # parent project
repo:
  id: irradiance

created_at: 2026-11-11
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Gives onyx's Darkroom v1 its Linear DNG export: a public, opt-in `write` feature that turns a u16 monochrome sensor plane, its levels and opcode lists, a caller-built EXIF block and XMP into a deterministic, uncompressed DNG in a caller-owned buffer, which re-parses to the source samples bit for bit.
  delivers:
    - "A public writer behind the `write` cargo feature (DEC-055, DEC-071): size query, then a write into a caller `&mut [u8]`; one `#[non_exhaustive]` options struct; `Error::kind()` errors; a `max_pixels` budget checked first; identical bytes for identical input. The default build is unchanged."
    - "Framing (full sensor, full active area, standard crop) and orientation written as tags over sensor-order samples (DEC-073), so a verbatim `OpcodeList3` stays correct"
    - "Opaque EXIF block placement at a fixed base offset with IFD0 pointer tags (DEC-072), an opaque XMP packet (IFD0 tag 700), and IFD0's descriptive tags as typed, ASCII-validated options (DEC-071 §3a)"
    - "An optional caller-encoded JPEG preview as a reduced-resolution subfile"
    - "Output that round-trips bit-exact through irradiance and is accepted by dnglab, exiftool and Adobe dng_validate"
    - "A provenance row: written from the Adobe DNG specification and TIFF 6.0 only"
  explicitly_does_not:
    - "Compression of any kind in v1. Lossless (Compression 7, deflate) is a later spec on request; lossy is never wanted (monorail-000/DEC-075 item 3)."
    - "Float samples, CFA planes, tiles or big-endian output (DEC-071 §4)"
    - "Apply, strip or rewrite any opcode list, or crop or rotate samples except dropping the margin outside ActiveArea (DEC-073)"
    - "Parse, edit or rebase the EXIF block; decide what EXIF survives (diamond-exif's job, DEC-072)"
    - "Judge whether a margin is optically masked; MaskedAreas comes only from the caller (DEC-071 §6)"
    - "Encode a preview; re-emit or edit-and-save an existing DNG; any I/O or CLI (library-not-application)"

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

# STAGE-017: Public DNG writer behind a write feature

## What This Stage Is

The public DNG writer, sized to onyx's Darkroom v1 Linear DNG export. It writes one
monochrome, scene-linear DNG of a u16 sensor plane: samples unchanged, the source's
levels and `OpcodeList3` copied, framing and orientation as tags, an optional caller
JPEG preview, and the caller's ready-made EXIF block and XMP placed verbatim. Bytes
in, bytes out, into a caller-owned buffer, behind the opt-in `write` feature.

## Why Now

onyx (`monorail-000`) answered PROJ-005's six writer questions on 2026-11-20 with
`monorail-000/DEC-075` (accepted there by jysf). That is the named consumer and the
concrete requirements this stage was waiting for. onyx's PROJ-002 (Darkroom v1) lists
this writer as an external gate, together with `diamond-exif`'s EXIF block serialiser.

## Requirements traced from `monorail-000/DEC-075`

| `monorail-000/DEC-075` | Here | Spec |
|---|---|---|
| 1 / O1: u16 ADU unchanged, source levels copied | DEC-071 §1 | SPEC-061 |
| 1: no warp; framing + orientation are the caller's | DEC-073 (tags, not pixel ops) | SPEC-061 |
| 1 / O2: `OpcodeList3` kept | DEC-071 §2, DEC-073 | SPEC-061 |
| 1 / O3: full sensor writes margins + `ActiveArea`; `MaskedAreas` only if masked | DEC-071 §6 (caller's list) | SPEC-061 |
| 1: same input, same bytes | DEC-071 §7 | SPEC-061 |
| 2 / O5: optional caller JPEG preview | DEC-071 §5 | SPEC-082 |
| 3 / O5: uncompressed; never lossy | DEC-071 §4 | SPEC-061 |
| 4: no float | DEC-071 §4 | — |
| 5 / O4: opaque EXIF block placed, pointers written, offsets fixed; opaque XMP; IFD0 structural tags the writer's own | DEC-072, DEC-071 §3 | SPEC-081 (EXIF, XMP), SPEC-061 (structural tags) |
| 6: memory in/out, size query + caller buffer, plain options, `Error::kind()`, budget before allocation, no panics, no global state | DEC-071 §7 | SPEC-061 |
| `monorail-000/DEC-076` 1: EXIF contract agreed; base read from irradiance's public constant | DEC-072 | SPEC-081 |
| `monorail-000/DEC-076` 2: framing as tags agreed (Linear DNG only) | DEC-073 | SPEC-061 |
| `monorail-000/DEC-076` 3: caller masked list agreed, empty for the Q2M | DEC-071 §6 | SPEC-061 |
| `monorail-000/DEC-076` 4 (widened): `Make`, `Model`, `UniqueCameraModel`, `LocalizedCameraModel`, `Artist`, `Copyright`, `DateTime`, `Software`, `CameraSerialNumber` as typed ASCII options; `UniqueCameraModel` required (`CallerError` if absent) | DEC-071 §3a | SPEC-061 (options struct + `UniqueCameraModel`), SPEC-081 (the other eight) |
| `monorail-000/DEC-076` 5: XMP packet in v1 | DEC-071 §2 | SPEC-081 |
| `monorail-000/DEC-078` 1: `mono-core` supplies the four rendering hints, read through irradiance's `Container::rationals`; a malformed source value is left out | DEC-074 | SPEC-081 |
| `monorail-000/DEC-078` 2: export-sheet preflight calls `written_len` per framing and disables a framing refused with `UnsupportedOpcodeForFraming` | DEC-075 §5, §7 | SPEC-061 (already: `written_len` runs every `write_into` check but `dst` length), SPEC-081 (keeps that parity for its new fields) |
| `monorail-000/DEC-078` 3, 4: masked areas only at full sensor; budget, orientation, verbatim lists and error handling need nothing new | DEC-075 §6, §8, §4, §7 | SPEC-061 |

## Success Criteria

- **Feature gating.** The default build gains nothing: `cargo package` and
  `--no-default-features` are unchanged; `write` is additive.
- **Round trip.** On the held Monochrom files, at each framing, a written file
  re-parses through irradiance to the source's samples (over the written region),
  levels, `OpcodeList3` bytes, `Orientation` and crop, bit for bit.
- **Acceptance.** dnglab, exiftool and Adobe `dng_validate` (local, DEC-069 §6)
  accept every framing; exiftool reads the same Exif and GPS values from a written
  file as from the block's source.
- **Determinism.** Two writes of the same input are byte-identical (a CI test, plus
  the existing macOS determinism leg).
- **Panic-freedom.** No panics on any caller input; typed errors for inconsistent
  options. A fuzz target covers the writer's option and geometry surface.
- **Semver.** New public types are `#[non_exhaustive]` (DEC-052).
- **Manual check.** A Lightroom Classic / Capture One open is recorded, not gated
  (onyx's own validation repeats it on its export).

## Scope

### In scope
- The `write` feature and the public API (DEC-071).
- Mono u16, uncompressed strips, little-endian.
- Framing and orientation as tags (DEC-073); caller-listed `MaskedAreas`.
- Opcode-list, XMP and EXIF-block pass-through (DEC-072).
- One optional caller-encoded JPEG preview.

### Explicitly out of scope
- Everything in `value_contribution.explicitly_does_not`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-061 (shipped on 2026-11-24, PR 329; 35.7M / $26.09, est. 35M) [M] Public `write` feature core: mono u16 uncompressed writer into a caller buffer, size query, options struct, framing and orientation as tags, levels/opcode pass-through, caller MaskedAreas, required `UniqueCameraModel` (`CallerError` if absent), `Error::kind()`, max_pixels, determinism, fuzz target, round-trip + dnglab/exiftool/dng_validate acceptance
- [ ] SPEC-081 (design done, build HANDOFF-191) [M] EXIF block, XMP and IFD0 descriptive tags: opaque block at the fixed base offset with ExifIFD/GPSInfo pointers (DEC-072), XMLPacket, typed ASCII `Make`/`Model`/`LocalizedCameraModel`/`Artist`/`Copyright`/`DateTime`/`Software`/`CameraSerialNumber` (DEC-071 §3a), the rendering hints `BaselineExposure`/`BaselineNoise`/`BaselineSharpness`/`LinearResponseLimit` (DEC-074), exiftool parity on a hand-built block, red-proofs
- [ ] SPEC-082 (frame) [S] Optional caller JPEG preview stored as a reduced-resolution subfile, read back by irradiance's preview accessors (DEC-057) and accepted by dng_validate
- [ ] [XS] CI `fuzz smoke — dng_write (60s)` job, like every other fuzz target's (SPEC-061 verify F3)
- [ ] [XS] The three intra-doc links to `crate::dng` from always-compiled private items break under `--document-private-items` without `write`; use plain text or `cfg_attr` (SPEC-061 verify F4)

**Count:** 1 shipped / 1 active / 1 pending (plus 2 [XS])

## Design Notes

DEC-055 governs the crate layout: the writer stays in the crate behind a feature.
DEC-071 fixes v1's scope, DEC-072 the EXIF offset contract, DEC-073 the framing rule.
SPEC-061's design (2026-11-21) added DEC-075 (layout and validation rules) and DEC-074
(the rendering-hint tags, which land in SPEC-081), both proposed.
All three are `proposed` until jysf accepts them. onyx agreed to all three in
`monorail-000/DEC-076` and its two asks (descriptive tags, explicit XMP) are folded into
DEC-071. `diamond-exif` accepted DEC-072's contract in `diamond-exif/DEC-015`
(2026-11-21); DEC-072 was amended the same day so a GPS-only block is valid.

**2026-11-24, SPEC-081 designed.** DEC-081 (proposed) settles what DEC-071 §3a, DEC-072
and DEC-074 left open: `ExifBlock` and the base constant and function, an empty
descriptive string written as an empty tag (held files carry them), an empty XMP packet
refused, `Rational` for the hints with unsigned tags refusing negatives, the new checks
between `UniqueCameraModel` and the `OpcodeList1` walk, and "inside the block" as room for
the IFD's entry count. Re-estimated from S to M (25M).

The requirement source is `monorail-000/DEC-075`. It is an external input: specs
reference it in prose, never in `depends_on`.

**2026-11-20, re-framed from `monorail-000/DEC-075`.** The 2026-11-11 frame scoped
mono and CFA planes, strips and tiles, both byte orders, colour metadata pass-through,
and no previews. onyx needs mono only, uncompressed strips, a caller preview, and
EXIF/XMP carry-over, so CFA, tiles, big-endian and colour matrices are out and the
preview and EXIF/XMP specs are in. SPEC-061 keeps its ID and is re-scoped to the core.

**Release.** onyx consumes a published crate. When this backlog ships, a release spec
(`just new-release-spec`) cuts the version that carries `write`; it is not framed yet.

## Dependencies

### Depends on
- STAGE-013 (PROJ-004): the clean-room test-only writer and its verification plan.
- The local `dng_validate` reference (DEC-069 §6), for acceptance.

### External inputs (not `depends_on`)
- `monorail-000/DEC-075`: the requirements.
- `monorail-000/DEC-076`: onyx's answer to this framing, amending DEC-075.
- `monorail-000/DEC-077`: onyx's choice of the GPS-only block (DEC-072's 2026-11-21
  amendment) and how `mono-core` prepares the writer's inputs.
- `monorail-000/DEC-078`: onyx's agreement to DEC-074 and DEC-075 (2026-11-21).
- `diamond-exif`'s block serialiser: SPEC-081 is testable with a hand-built block, but
  `diamond-exif` accepted DEC-072 in `diamond-exif/DEC-015` (2026-11-21); its builder is
  `diamond-exif/SPEC-025`, not yet built.

### Enables
- onyx Darkroom v1's Linear DNG export; later crustyimg RAW-in/DNG-out.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m — only if this stage was
  framed with `just frame-stage`. Counts outlines whose scope held from frame to
  ship. A low number means framing ran ahead of what was knowable; a high one
  means it's paying for itself.>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
