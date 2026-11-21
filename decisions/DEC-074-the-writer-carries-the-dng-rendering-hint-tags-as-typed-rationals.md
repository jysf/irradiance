---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-074
  type: decision
  confidence: 0.85                   # measured: dropping BaselineExposure changes the SDK render of a held file
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
  - src/dng/write.rs

tags:
  - dng-writer
  - rendering
  - stage-017
  - onyx
---

# DEC-074: The writer carries DNG's rendering-hint tags as typed rationals

## Decision

DEC-071 §3a covers IFD0's descriptive tags. Four more IFD0 tags are neither structural
nor descriptive: they change how an editor renders the file. The writer takes them as
optional typed values, written verbatim, and never invents them:

| Tag | Number | Type | Absent means (DNG default) |
|---|---|---|---|
| `BaselineExposure` | 50730 | SRATIONAL | 0 EV |
| `BaselineNoise` | 50731 | RATIONAL | 1.0 |
| `BaselineSharpness` | 50732 | RATIONAL | 1.0 |
| `LinearResponseLimit` | 50734 | RATIONAL | 1.0 |

- Each is a `(numerator, denominator)` pair, written exactly as given; a zero
  denominator is a `CallerError`.
- `mono-core` copies them from the source (they are IFD0 tags, so they are not in the
  EXIF block).
- They land in SPEC-081, with the other typed IFD0 tags.

## Agreement

onyx agreed in `monorail-000/DEC-078` item 1 (2026-11-21). `mono-core` reads each tag
from the source's IFD0 through irradiance's own reader (`Container::ifd0`, `Ifd::entry`,
`Container::rationals`, all in 0.2.2) and passes it exactly as stored. A tag the source
lacks is left out, so the DNG default applies. A malformed source value (a zero
denominator, or a RATIONAL past `i32::MAX`, which `Container::rationals` refuses) is left
out and noted in the export's provenance, so `mono-core` never sends a value the writer
refuses. onyx's export spec adds a fixture with a non-default `BaselineExposure`.

## Context

At SPEC-061's design probe (2026-11-21), the held PENTAX K-3 III Monochrome has
`BaselineExposure` = −33177/65536 (about −0.51 EV). Writing its plane as a standard-crop
DNG without that tag, then rendering both through Adobe `dng_validate` 1.7.1 (`-16 -tif`),
changed 76,681,557 of 76,681,728 output samples, by up to 7,058 codes. With the tag
carried verbatim, the two renders were identical. The held Leicas have
`BaselineExposure` 0 and the other three at 1.0, so they render identically either way,
which is why `monorail-000/DEC-075` and DEC-071 did not notice.

`AntiAliasStrength`, `BestQualityScale` and `DefaultUserCrop` are also in the held files'
raw IFDs. They are left out: the first two are demosaic hints that do not apply to a
`LinearRaw` plane, and `DefaultUserCrop` is a user crop inside `DefaultCrop`, which onyx's
framing already expresses.

## Alternatives Considered

- **Leave them out.** Rejected: an export of a K-3 III Monochrome would open about half
  a stop brighter than its source in any DNG editor.
- **Raw caller tag bytes for "other" tags.** Rejected: DEC-071 §3 lets no caller bytes
  into IFD0.

## Consequences

- **Positive:** exports render like their sources in DNG editors.
- **Negative:** four more fields, and `mono-core` has to read them from the source with
  `diamond-exif`'s reader or irradiance's container.
- **Neutral:** irradiance's reader does not model these tags today. Reading them is
  `mono-core`'s concern, not this DEC's.

## Validation

- SPEC-081: the K-3 III Monochrome standard-crop export with `BaselineExposure` renders
  identically to the source through `dng_validate`; the same export without it does not
  (the red-proof).

## References

- Related specs: SPEC-061 (probe), SPEC-081 (lands it)
- Related decisions: DEC-071, DEC-073
- External: `monorail-000/DEC-075`, `monorail-000/DEC-076`, `monorail-000/DEC-078`
