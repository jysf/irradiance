---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-072
  type: decision
  confidence: 0.7                    # needs diamond-exif's agreement; the shape is final only once its serialiser accepts it
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

created_at: 2026-11-20
supersedes: null
superseded_by: null

status: accepted                     # jysf, 2026-11-21, in the STAGE-017 framing conversation

deciders: [jysf, claude]

affected_scope:
  - src/dng/write.rs

tags:
  - dng-writer
  - exif
  - offset-contract
  - stage-017
  - onyx
---

# DEC-072: The EXIF block is built for a fixed base offset, and the writer never rebases it

## Decision

The writer reports the base first, and the base is a constant: **the EXIF block is
always placed at file offset 8**, directly after the TIFF header, in the writer's
byte order (little-endian, DEC-071 §4). IFD0, the strips, the XMP packet and any
preview follow it.

- **What the caller hands over:** the block bytes, plus the offsets **inside the
  block** of the Exif IFD and of the GPS IFD. Each offset is optional, but a block
  must come with **at least one** of them; a block with neither is a
  `CallerError` (amended 2026-11-21, below).
  Every offset stored inside the block is already absolute, computed by `diamond-exif`
  for a block that starts at file offset 8. The Interoperability IFD pointer lives
  inside the Exif IFD and is the block's business.
- **What the writer does:** copies the block verbatim, writes `ExifIFD` (0x8769) and
  `GPSInfo` (0x8825) in IFD0 as `8 + offset`, each only when its offset is given, and checks only what it needs to stay
  well-formed: the block length fits in the file's offset range, the declared IFD
  offsets are inside the block and even (TIFF word alignment), and the block length
  is padded to even before IFD0. It does not parse, walk or rebase the block.
- **The writer exposes the base**, as a public constant and as a function of the
  options that returns it. `mono-core` reads the base from there and passes it to
  `diamond-exif`; it never hard-codes 8 (`monorail-000/DEC-076` item 1). Changing the
  base's value, or making it depend on the options, is a **breaking change**: it needs
  a DEC amending this one and a semver-major release (pre-1.0, a minor bump), and the
  release notes name it. A test pins the constant to 8 so the change cannot happen
  by accident.
- **No block** means no pointer tags; IFD0 then starts at offset 8. The file is still
  a valid DNG.

## Agreement

- **onyx: agreed**, `monorail-000/DEC-076` item 1 (2026-11-20), on the contract as
  written: base 8, little-endian, final absolute offsets including the Interop pointer,
  `diamond-exif` reporting the in-block offsets of the Exif IFD (always) and the GPS IFD
  (when present).
- **`diamond-exif`: agreed**, `diamond-exif/DEC-015` (accepted, 2026-11-21), with the
  builder in `diamond-exif/SPEC-025` (not built yet):
  - a size query, then a write at a caller-given base with final absolute offsets,
    the Interop pointer included; base 8 and little-endian are supported, and a
    big-endian source is re-serialised element by element, not copied;
  - it reports the in-block offsets of the Exif IFD and, when present, the GPS IFD;
  - the block length is always even, so the writer's padding before IFD0 is a no-op.
    The writer keeps the padding, because it does not trust a caller's length;
  - the same source, options and base give the same bytes, and `diamond-exif` reads
    its own output back before returning it;
  - the MakerNote is always left out, so in v1 no MakerNote survives an export.

## Amendment 2026-11-21: the Exif IFD offset is optional

`diamond-exif` pointed out that a source with a GPS IFD and no Exif IFD gives a
GPS-only block, which the contract as first written refused. onyx chose the same in
`monorail-000/DEC-077` item 1 (jysf). Chosen: **both offsets
are optional, and a block needs at least one.** Rejected: `diamond-exif` writing an
empty Exif IFD (a fabricated IFD in every such file, only to satisfy the writer), and
`mono-core` dropping the block (loses GPS for no reason). The writer's checks are
unchanged for each offset it is given. `diamond-exif/DEC-014` follows this choice.

## Context

`monorail-000/DEC-075` item 5 hands the offset contract to irradiance's design, with
`diamond-exif` to agree, and says the app leans to base-first. The two choices:

- **block-relative offsets, rebased by the writer**: the writer must parse every IFD
  in the block, find every offset-typed value (including the Interop pointer and any
  pointer a future Exif tag adds), and rewrite it;
- **base first**: the writer says where the block will go and `diamond-exif` writes
  final offsets.

## Alternatives Considered

- **Option A: block-relative offsets that the writer rebases.**
  - Why rejected: it makes irradiance interpret the block, which `monorail-000/DEC-075`
    says it should not. The block derives from a third-party source file, so the
    rebaser is a new untrusted-input parser with its own fuzz target and its own
    failure modes; and any offset it does not know about (a MakerNote's internal
    offsets) is silently corrupted. `diamond-exif` already owns the knowledge of which
    values are offsets; splitting it across two crates is the bug.
- **Option B: base first, base computed from the layout (block after the strips).**
  - Why rejected: the base then depends on the plane size, the preview and the XMP
    length, so the caller needs a two-phase call (ask for the base, build the block,
    write) whose two halves must be given identical options. That is a contract the
    writer cannot check and a mismatch produces a well-formed file with wrong EXIF.
- **Option C (chosen): base first, base fixed at offset 8.**
  - Why: no layout coupling, a single stateless call, nothing to rebase or parse, and
    `diamond-exif` needs one number. TIFF lets IFD0 sit anywhere, so placing data
    before it is valid; the round-trip and `dng_validate` acceptance in SPEC-081 test
    that readers agree.

## Consequences

- **Positive:** irradiance's writer gains no parser; the only new checks are bounds
  and alignment. `diamond-exif` can build the block before the plane exists.
- **Negative:** the block must be built in little-endian; a source file in big-endian
  needs `diamond-exif` to re-serialise, which it does anyway when it builds a block.
  The layout is frozen at "header, block, IFD0" until a DEC changes it.
- **Neutral:** a MakerNote survives only if `diamond-exif` can place it for offset 8;
  that remains its call (`monorail-000/DEC-075` item 5). In v1 it never does: its
  serialiser always leaves the MakerNote out.

## Validation

- SPEC-081: a hand-built block (Exif IFD, GPS IFD, Interop IFD) written through the
  writer reads back through exiftool with the same values; the red-proof shifts the
  base by 2 and exiftool's values change or fail. A GPS-only block writes only
  `GPSInfo` and reads back; a block with neither offset is refused.
- `diamond-exif` confirmed the fixed base and byte order (`diamond-exif/DEC-015`,
  accepted 2026-11-21).

## References

- Related specs: SPEC-081
- Related decisions: DEC-071
- External: `monorail-000/DEC-075` item 5; `monorail-000/DEC-076` item 1;
  `monorail-000/DEC-077` item 1; `diamond-exif/DEC-014`, `diamond-exif/DEC-015` and `diamond-exif/SPEC-025` (the block
  serialiser, not yet built)
