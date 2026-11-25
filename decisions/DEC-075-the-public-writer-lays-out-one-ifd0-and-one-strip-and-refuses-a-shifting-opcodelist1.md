---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-075
  type: decision
  confidence: 0.8                    # probed against Adobe dng_validate on the held Q2M and K-3 III Monochrome
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-005
repo:
  id: irradiance

created_at: 2026-11-21
supersedes: null
superseded_by: null

status: accepted                     # jysf, 2026-11-21, at the SPEC-061 design PR

deciders: [jysf, claude]

affected_scope:
  - src/dng/**
  - src/lib.rs

tags:
  - dng-writer
  - layout
  - validation
  - stage-017
---

# DEC-075: The public writer lays out one IFD0 and one strip, and refuses an OpcodeList1 a cut would shift

## Decision

SPEC-061's writer (`irradiance::dng::write`, feature `write`) follows these rules.

1. **File layout.** `II*\0` header, then (from SPEC-081) the EXIF block at offset 8,
   then IFD0, then its out-of-line values, then the plane. The raw plane **is IFD0**
   (`NewSubFileType` 0); there is no SubIFD and no next IFD in SPEC-061. Every value and
   the strip start at an even offset. Every byte of the output is written by the writer,
   so the caller's buffer contents never leak into the file.
2. **One strip.** `RowsPerStrip` = the written height, one `StripOffsets` entry. The
   held Q2M and M Monochrom store one strip too.
3. **Versions.** `DNGVersion` 1.4.0.0. `DNGBackwardVersion` 1.3.0.0 when any opcode list
   is written (opcode lists are a DNG 1.3 feature), otherwise 1.1.0.0.
4. **`Orientation` is always written**, default 1. `dng_validate` warns when it is
   missing (probe).
5. **Framing cuts and opcode lists** (DEC-073's refusal clause, now made concrete):
   - full sensor: every list is copied verbatim; nothing moves;
   - full active area and standard crop drop the margin. `OpcodeList2` and
     `OpcodeList3` work on the active-area image (DNG stage 2/3; DEC-069 already applies
     `GainMap` over `ActiveArea`), so they are copied verbatim. The probe proved it for
     the Q2M's `WarpRectilinear`;
   - `OpcodeList1` works on the stored plane, so a cut moves its coordinates whenever
     the active area's top or left is not 0, and drops any area it names in the margin.
     Under a cutting framing the writer walks the list's opcode headers (id, version,
     flags, size; never the parameters) and accepts it only if every opcode is in a
     position-independent allow-list. The allow-list is `FixBadPixelsConstant` (id 4),
     which names no coordinates and is the only list-1 opcode in the held files. Any
     other opcode is refused with `Error::UnsupportedOpcodeForFraming`
     (`ErrorKind::Unsupported`), naming the list, the opcode id and the framing;
   - a list whose headers cannot be walked is a `CallerError`.
   - **Known limit (SPEC-061 verify, 2026-11-24).** `FixBadPixelsConstant` names no
     coordinates, but its effect is not fully position-independent: a reader replaces a
     bad pixel from neighbours on the stored plane (irradiance: SPEC-017 rule 1,
     `apply_fix_bad_pixels_constant`). A bad pixel on the active area's edge borrows
     margin neighbours in the source but cannot after a cut, so that pixel can develop
     differently. This is a property of the format, not of irradiance: any reader runs
     `OpcodeList1` on the stored plane. It is accepted, not refused: refusing would mean
     scanning sample values on the edge, which the header-only walk is designed not to
     do. The held files are unaffected (SPEC-061 oracle 4 is exact on all four), and the
     test `fix_bad_pixels_constant_on_the_active_area_edge_develops_differently_after_a_cut`
     pins the behaviour. A caller that needs bit-identical edge pixels writes full
     sensor.
6. **`MaskedAreas` only under full sensor.** Under the cutting framings the margin is not
   in the file, so a non-empty masked list is a `CallerError`, not silently dropped.
7. **Two error variants.** Every caller-input refusal is
   `Error::InvalidWriteInput(WriteInputError)`, with `WriteInputError` a
   `#[non_exhaustive]` enum of reasons, classified `CallerError`. The one refusal that is
   about the input's content, not the call, is `Error::UnsupportedOpcodeForFraming`,
   classified `Unsupported`. The caller's pixel budget reuses `Error::PlaneTooLarge`
   (DEC-050). Two new arms in the wildcard-free match (DEC-051).
8. **The pixel budget is an argument**, as `plane::validate` takes it (DEC-050), not an
   options field: a `Default` options value then cannot silently mean "no budget".
9. **No allocation.** `written_len` and `write_into` allocate nothing (the DEC-016 promise
   for `unpack_into`, applied to the writer). The IFD entry table is a fixed-size array.
   A `write_to_vec` convenience allocates exactly the output.

## Agreement

onyx agreed in `monorail-000/DEC-078` items 2 to 4 (2026-11-21):

- **§5:** before it shows the export sheet, `mono-core` calls `written_len` once per
  framing and disables any framing refused with `UnsupportedOpcodeForFraming`, giving the
  reason. Full sensor always stays available. This relies on `written_len` running every
  check `write_into` runs except the `dst` length, in the same order, with the same
  error (SPEC-061 §3, AC4, the fuzz agreement property). The preflight must use the
  options that framing would really be written with. An earlier check fails first: for
  example, masked areas at a cutting framing give `InvalidWriteInput` before the opcode
  walk.
- **§6:** `mono-core` passes masked areas only at full sensor, so the `CallerError` can
  only be a `mono-core` bug.
- **§4, §7, §8:** nothing new. `mono-core` passes `max_pixels` and the source's
  `Orientation`, and it logs `InvalidWriteInput` as a `mono-core` bug.

## Context

These are the choices SPEC-061's design had to make that DEC-071 to DEC-073 leave open.
The probe (2026-11-21, Adobe `dng_validate` 1.7.1, the held Q2M `L1021223.DNG` and
PENTAX K-3 III Monochrome) wrote each framing with the test-only writer in this layout:
`dng_validate` accepted every file with no warnings once `Orientation` and
`UniqueCameraModel` were present; full-sensor stage 3 matched the source's exactly on
both files; and the Q2M standard crop (active-area samples, crop as a tag, verbatim
`OpcodeList1` and `OpcodeList3`) rendered identically to the source. Cutting the samples
to the crop instead changed 95% of the render, by up to 64,441 codes.

## Alternatives Considered

- **Raw plane in a SubIFD under a preview IFD0** (the Leica layout). Rejected for
  SPEC-061: it needs a preview IFD0 image before SPEC-082 exists. SPEC-082 decides where
  its preview goes.
- **Several strips.** Rejected: no reader or probe needed them, and one strip makes the
  size query trivial.
- **Rewriting `OpcodeList1` coordinates on a cut.** Rejected by DEC-073 (the writer never
  edits opcode bytes).
- **One error variant per reason.** Rejected: about fifteen reasons, all from one call;
  a reason enum keeps `Error` readable and the kind mapping to two arms.

## Consequences

- **Positive:** a small, testable layout; every framing proven against Adobe's reader.
- **Negative:** a source with a coordinate-bearing `OpcodeList1` (for example
  `FixBadPixelsList`) can only be exported at full sensor. None of the held mono files has
  one.
- **Neutral:** the allow-list grows by DEC when a real file needs it.

## Validation

- SPEC-061's tier-B tests and its env-gated `dng_validate` check.

## References

- Related specs: SPEC-061, SPEC-081, SPEC-082
- Related decisions: DEC-016, DEC-051, DEC-055, DEC-069, DEC-071, DEC-072, DEC-073
- External: `monorail-000/DEC-078`
