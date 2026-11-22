---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-078
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-22
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/opcode.rs
  - src/demosaic.rs
  - src/color.rs
  - src/lib.rs
  - tests/error_kind.rs
  - decisions/DEC-012-strict-on-structure-tolerant-on-shape.md
  - decisions/DEC-051-error-kind-three-way-classification-and-the-wildcard-free-match.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-087-malformed-interpretation-tags-refuse-at-develop.md

tags:
  - refusal
  - develop
  - malformed
  - stage-027
---

# DEC-078: a malformed interpretation tag the develop path consumes refuses the develop

## Decision

### 1. The rule

The parse stays tolerant (DEC-012 is unchanged for reads): a malformed interpretation
tag is `None` plus an entry in `Sensor::malformed_tags`, and metadata, previews and
plane reads still work.

The develop becomes strict. Each develop entry refuses, before it writes to `dst`, when
a tag **its own path reads** is in `malformed_tags`. It does not fall back to the
absent-tag default. This generalises DEC-053 (unreadable opcode lists) from three tags to
every consumed interpretation tag. A malformed value and an absent value mean different
things: absent is the DNG default, but malformed is "the file says something here and we
cannot read it".

### 2. Which path consumes which tag

`DevelopPath` (public, `#[non_exhaustive]`, DEC-053 §2) gains two variants, so each entry
asks only about what it reads. Both behave as `Colour` for the opcode lists. Adding a
variant to a `#[non_exhaustive]` enum is additive.

| Path | Entries | Tags refused when malformed |
|---|---|---|
| `Mono` | `develop_into`, `develop_with_into`, `develop_with_scratch_into` | the **levels and geometry set** |
| `Colour` | `develop_color_into`, `develop_color_f32_into`, both threaded | the levels and geometry set, the **CFA set**, the **colour set** |
| `Demosaic` (new) | `demosaic_into`, `demosaic_into_threaded` | the CFA set |
| `ColourTransform` (new) | `to_srgb_linear_into` | the colour set |

- **Levels and geometry set:** `Orientation` (274), `BlackLevel` (50714), `WhiteLevel`
  (50717), `DefaultCropOrigin` (50719), `DefaultCropSize` (50720), `ActiveArea` (50829),
  and `BlackLevelRepeatDim` (50713) **only when `BlackLevel` has more than one value**.
  With one value the repeat dimension cannot change the black. That exception is the
  Pentax K-3 III Monochrome's shipping file, the case DEC-012 was written for. It applies
  whatever `DevelopOptions` says, as DEC-058 §3 requires: an option never skips a tag's
  validation.
- **CFA set:** `CFAPlaneColor` (50710), `CFALayout` (50711). A malformed tag no longer
  bypasses PR 301's `UnsupportedCfaLayout` and `UnsupportedCfaPlaneColor`.
- **Colour set:** `ColorMatrix1/2` (50721/50722), `CameraCalibration1/2` (50723/50724),
  `AnalogBalance` (50727), `CalibrationIlluminant1/2` (50778/50779),
  `CameraCalibrationSignature` (50931), `ProfileCalibrationSignature` (50932),
  `ForwardMatrix1/2` (50964/50965).
- **Every path:** `DNGVersion` (50706) when `DNGBackwardVersion` is absent. DEC-077's
  version guard needs it then. It refuses `Error::UnsupportedTag { tag: 50706 }`, the
  same variant DEC-077 uses for a malformed 50707, with its own `Display` wording ("the
  file's reader version is unknown"). This is the SPEC-086 verify's routed follow-up.

**Not in any set:**
- Tags already refused another way keep their errors: `CFAPattern` and
  `CFARepeatPatternDim` (`NotAColorFilterArray` / `UnsupportedCfaPattern`),
  `AsShotNeutral` (`MissingColorCalibration`), the opcode lists (`UnreadableOpcodeList`),
  and DEC-077's set (`UnsupportedTag`).
- `MaskedAreas`, which no develop reads.

The whole colour set applies on the colour paths, even a tag the chosen sub-path will
not use, such as `CameraCalibration2` on a single-illuminant file. A malformed
`CalibrationIlluminant2` or `ColorMatrix2` decides whether the dual path runs at all, and
a set that depends on the decision it guards is a place for a silent default to come back.

### 3. Order and error

Inside `check_required_processing`:
1. DEC-077's checks, unchanged, with the malformed `DNGVersion` case added beside the
   malformed `DNGBackwardVersion` case;
2. then this decision's malformed-tag check for the path, in ascending tag order;
3. then the opcode lists (DEC-053), unchanged.

The error is a new variant `Error::MalformedTag { tag: u16 }`, the first offending tag,
classified `ErrorKind::Corrupt` like `UnreadableOpcodeList`. It is appended at the end of
`Error` (DEC-051). Its `Display` names the tag ("WhiteLevel (50717) is present but
malformed, so the develop cannot use it"). The name lookup is widened to every tag in §2,
with "tag" as the fallback.

### 4. A singular `ColorMatrix` or calibration that the converged weight never uses still develops

SPEC-085 asked this spec to decide. It stays as DEC-076 §3 left it: such a file develops
when the weight lands on the other illuminant, and is refused (`MissingColorCalibration`)
when the weight needs the singular product. This is Adobe DNG SDK 1.7.1's behaviour, and
the SDK is the colour develop's oracle (DEC-076). A well-formed but singular matrix is not
a malformed tag, so it is outside this decision's rule. No change.

### 5. `DEVELOP_VERSION` is unchanged

Refusals only: every develop that still succeeds gives the same bytes (DEC-053).

### 6. DEC-012 amended

DEC-012's rationale for tolerance ("not consumed yet") no longer holds for these tags.
Its read rule stands; this decision adds the develop-time half. DEC-012 gets an amendment
note pointing here.

## Context

**The gap.** The STAGE-027 audit (`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md`
§1, §4 rank 2) found that `malformed_tags` was read only for the opcode lists. A malformed
`WhiteLevel` develops about 8% dark on a 14-bit file. A malformed `ActiveArea` puts masked
photosites in the image. A malformed `CFALayout` or `CFAPlaneColor` re-opens PR 301's two
refusals. A malformed `ColorMatrix2` silently drops to single-illuminant colour. A
malformed `CameraCalibration` silently becomes the identity, the gap SPEC-085 closed for
well-formed tags.

**Re-measured at SPEC-087's design (2026-11-22, `irr ifd` on every held DNG):** every
`malformed_tags` is empty except the Pentax K-3 III Monochrome's `[50713]`
(`BlackLevelRepeatDim`, count 1 where DNG requires 2), whose `BlackLevel` has one value.
So no held file is refused and no held output changes.

**The path split.** `demosaic_into` reads only the CFA tags. `to_srgb_linear_into` reads
only the colour tags. Neither resolves levels or geometry. Before this decision both
passed `DevelopPath::Colour`, which could not tell them from the full colour develop.

## Provenance

- **Class 1:** Adobe DNG Specification 1.7.1.0, the tag definitions (types and counts)
  for each tag in §2; `DNGBackwardVersion`'s default from `DNGVersion`.
- **Class 4:** Adobe DNG SDK 1.7.1 `dng_color_spec.cpp`, already recorded under DEC-076,
  for §4's singular-matrix behaviour. Nothing new was read.

## Alternatives Considered

- **One `Colour` set for demosaic, the transform and the full develop.** Rejected. It
  would refuse a malformed `WhiteLevel` on `demosaic_into`, which never reads it, and
  surprise a composer who resolved levels themselves.
- **Refuse any malformed tag on every path.** Rejected for the same reason. It would also
  refuse the Pentax K-3 III Monochrome, which develops correctly.
- **Reuse `UnsupportedTag` for malformed tags.** Rejected. "Not supported yet" and
  "corrupt" are different `ErrorKind`s, and DEC-077's wording is about sample meaning. The
  one exception is the version pair, 50706 with 50707, where "unknown reader version" is
  an unsupported-reader question.
- **Refuse a singular unused `ColorMatrix`.** Rejected (§4). It would diverge from the SDK
  for a file the SDK develops.
- **Narrow the colour set to the tags the chosen sub-path reads.** Rejected (§2). The
  sub-path choice itself reads those tags.

## Consequences

- A malformed interpretation tag the path reads refuses that develop with
  `MalformedTag { tag }` and `dst` untouched. Metadata, previews and plane reads are
  unaffected.
- PR 301's CFA refusals and SPEC-085's calibration can no longer be bypassed through a
  malformed tag.
- `DevelopPath` gains `Demosaic` and `ColourTransform`. `Error` grows from 75 to 76
  variants (`Corrupt` 32 to 33).
- The Pentax K-3 III Monochrome still develops. `DEVELOP_VERSION` stays `"8"`.
- DEC-012 is amended (§6). DEC-076 §3's singular-matrix behaviour is confirmed (§4).
