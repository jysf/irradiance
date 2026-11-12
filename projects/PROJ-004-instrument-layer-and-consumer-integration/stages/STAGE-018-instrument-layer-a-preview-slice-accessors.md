---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-018                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-12
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The first piece of the instrument surface: a consumer can reach a DNG's embedded previews as zero-copy byte slices beside the sensor plane, with no I/O and no decoding in irradiance.
  delivers:
    - "A zero-copy Container previews iterator: one entry per embedded preview IFD, with dimensions, compression, photometric interpretation and a byte slice tied to the input buffer's lifetime"
    - "Preview selection by NewSubfileType and PhotometricInterpretation, never by size (docs/measured-q2m-dng.md), so the sensor IFD is never reported as a preview"
    - "Hostile layouts (offsets past the end, overlapping or non-contiguous strips) become typed errors, with a fuzz target"
  explicitly_does_not:
    - "Decode any preview: the host decodes the JPEG"
    - "The raw-ADU histogram and zones, or bad-pixel listing. They are the rest of future-plans Stage A and wait for a named caller (crustyimg STAGE-062, or a spike)"
    - "Compare the preview with the sensor plane: that is the consumer's Preview Lie Detector"

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

# STAGE-018: Instrument layer A: preview slice accessors

## What This Stage Is

Read-only access to a DNG's embedded previews. `Container` gets an iterator over the
preview IFDs that yields, for each one, its dimensions, `Compression`,
`PhotometricInterpretation`, `NewSubfileType` and its bytes as `&'a [u8]` borrowed from the
input. irradiance never decodes them.

This is the forensics part of PROJ-004's instrument layer (`docs/future-plans.md` § PROJ-004
Stage A), cut down to the one piece a consumer has asked for.

## Why Now

There is a named caller. monorail-000 (also called onyx-000, the macOS monochrome app suite)
asked for it as request 5 in `feedback/2026-11-11-monorail-000-requests.md`. Its Preview Lie
Detector compares the Q2M's full-size embedded JPEG (SubIFD2) with the sensor plane. The
maintainer and the requester agreed on 2026-11-12 that this stage comes first. It is small,
additive, and needs no new decoder, so it works on every file irradiance already parses.

## Success Criteria

- **Matches exiftool.** For every held tier-B corpus file, each preview's dimensions,
  compression, offset and length match what exiftool reports. The spec pre-registers how
  exiftool is run and which tags it compares. Evidence is recorded in the manifest; the
  corpus is never committed (DEC-003).
- **Correct selection.** The sensor IFD (`NewSubfileType == 0`, `LinearRaw` or CFA) is
  never reported as a preview, including on the Q2M, where SubIFD2 (8368 x 5584) is nearly
  the size of the sensor plane (8424 x 5632). A red-proof shows a size-based selector
  failing that test.
- **Zero-copy.** Each slice is a sub-slice of the caller's input buffer (checked by
  pointer range in tests), and the accessor does not allocate per preview.
- **Typed errors, never panics.** Offsets or lengths outside the file, overflow,
  multi-strip previews whose strips are not contiguous, and unreadable identifying tags
  all return typed errors, classified in `ErrorKind` (DEC-051). Tier-A fixtures
  cover each case, and a fuzz target covers the accessor.
- **No collateral change.** Existing `Container`, `Sensor` and develop outputs are
  byte-identical, and there is no new dependency.

## Scope

### In scope
- The accessor, a `Preview<'a>` type (`#[non_exhaustive]`, following DEC-052), and
  any new error variants.
- Tier-A preview fixtures (`tests/support/tiff.rs` and the STAGE-013 writer). The design
  probe found no `JPEGInterchangeFormat` preview in the corpus, so it is refused, not
  supported (DEC-057).
- The exiftool cross-check on the tier-B corpus, a fuzz target, a provenance row
  (TIFF 6.0 / DNG 1.7 for preview IFDs) and a CHANGELOG entry.

### Explicitly out of scope
- Decoding, colour-managing or validating the JPEG bitstream itself.
- The ADU histogram and zones, `collect_bad_pixels`, optical black (Stage B) and
  geometry (Stage C).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-062 (build) [S] Preview slice accessors on Container: zero-copy, allocation-free `previews()` iterator; NewSubfileType 1/65537 only (DEC-057); contiguous strips only; typed errors per IFD; exiftool-pinned corpus check (21 previews); fuzz. HANDOFF-131.

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

Facts to pin at design time:
- **Compression 7 does not mean raw.** DNG previews are often `Compression = 7`
  (baseline JPEG), the same code the K-3 III sensor plane uses for lossless JPEG.
  Select on `NewSubfileType` and `PhotometricInterpretation` (YCbCr or RGB for previews),
  as DEC-012 and `docs/measured-q2m-dng.md` already require for the sensor.
- **The Q2M has three candidates:**
  - IFD0: a small 8-bit preview;
  - SubIFD1: 1620 x 1080;
  - SubIFD2: a full-size JPEG, 8368 x 5584.

  Before design, inventory every held file with `irr ifd` (all preview IFDs, their
  compression, and strip count). One-strip previews should be the norm; the spec decides
  whether to support multi-strip previews only when they are contiguous, or to refuse them.
- **Lifetime.** Slices borrow the input (`'a`), not `&self`, unlike today's
  `Container::payload` (locked rule, future-plans 2026-11-02).
- **IFD walk.** The SubIFD walk, cycle guard and depth limit already exist
  (`MAX_IFD_DEPTH`, DEC-012). The accessor reuses `Container::ifds()` and adds no
  second walker.

## Dependencies

### Depends on
- STAGE-013 (shipped): the test-only writer, for preview fixtures.
- DEC-012 (selection rules), DEC-051 (`ErrorKind`), DEC-052 (sealing convention).

### Enables
- monorail-000 / onyx-000's Preview Lie Detector.
- crustyimg's truthful `info` path (STAGE-004 / its STAGE-062): preview vs sensor
  metadata.

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
