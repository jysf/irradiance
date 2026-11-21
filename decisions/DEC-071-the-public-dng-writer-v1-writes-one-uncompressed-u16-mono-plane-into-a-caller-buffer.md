---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-071
  type: decision
  confidence: 0.75                   # requirements are settled downstream; the API's details move at SPEC-061 design
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

status: proposed                     # drafted at STAGE-017 re-frame; onyx agreed (monorail-000/DEC-076) with two asks folded in as §2 and §3a; jysf accepts at PR review

deciders: [claude]                   # proposal; jysf accepts or amends at review

affected_scope:
  - src/dng/write.rs
  - projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md

tags:
  - dng-writer
  - write-feature
  - stage-017
  - onyx
  - api
---

# DEC-071: The public DNG writer v1 writes one uncompressed, u16, monochrome plane into a caller buffer, and nothing else

## Decision

STAGE-017's public writer (behind the `write` feature, `src/dng/write.rs`, DEC-055)
is sized to what onyx's Darkroom v1 asked for in `monorail-000/DEC-075`, and no
larger:

1. **What it writes.** One monochrome (`LinearRaw`, one sample per pixel) DNG of a
   u16 sensor plane. Samples are written **unchanged**: no black subtraction, no
   rescale, no warp. The caller's `BlackLevel` (with `BlackLevelRepeatDim`) and
   `WhiteLevel` are written as given, so re-parsing the output through irradiance
   returns the input samples bit for bit.
2. **Pass-through bytes.** `OpcodeList3` (and, if the caller supplies them,
   `OpcodeList1`/`OpcodeList2`) is written verbatim. The writer never applies or
   rewrites an opcode list. Two more opaque caller payloads are in v1 (SPEC-081):
   - **XMP:** an optional packet written verbatim as IFD0 tag 700 (`XMLPacket`,
     type BYTE). The writer does not parse or validate it beyond its length. This is
     where onyx records each export's provenance (`monorail-000/DEC-075` item 5,
     confirmed in `monorail-000/DEC-076` item 5).
   - **EXIF block:** an optional Exif/GPS/Interop block, placed per DEC-072.
3. **Structural tags are the writer's alone.** Every IFD0 structural tag
   (dimensions, `BitsPerSample`, `PhotometricInterpretation`, strip layout,
   `DNGVersion`, levels, `ActiveArea`, `DefaultCrop*`, `Orientation`, `MaskedAreas`,
   the EXIF/GPS pointers, `XMLPacket`'s placement) is computed from the plane and the
   options. No caller byte blob can supply or override one.
3a. **Descriptive tags are typed options** (`monorail-000/DEC-076` item 4, as widened
   on 2026-11-20 after `diamond-exif` raised the same gap). `Make`, `Model`,
   `UniqueCameraModel`, `LocalizedCameraModel`, `Artist`, `Copyright`, `DateTime`,
   `Software` and `CameraSerialNumber` live in IFD0, not in the Exif block, so the
   caller supplies them as typed fields, never as bytes. `diamond-exif` does not hand
   the writer IFD0 entries to merge (that option was rejected in `monorail-000/DEC-076`):
   - each is an ASCII string field, validated before any write: printable ASCII only
     (0x20 to 0x7E), no NUL (the writer appends the terminator), and a length bound so
     the size query cannot overflow. Anything else is a typed error;
   - `DateTime` must also have TIFF's form, `YYYY:MM:DD HH:MM:SS`. The writer never
     stamps the time itself, because output must be deterministic;
   - `UniqueCameraModel` is **required**: DNG requires it in every file. SPEC-061's
     design probe (2026-11-21) found that `dng_validate` 1.7.1 accepts a file without it,
     or with it empty, but prints `Missing or invalid UniqueCameraModel`; the writer's
     acceptance bar is no warnings, so this is a refusal here, not a warning. A
     write without it, or with it empty, fails with an error whose `Error::kind()` is
     `ErrorKind::CallerError` (DEC-051), checked before any output byte is written. The
     writer never invents one; `mono-core` always supplies one (the source's value, or
     Make + Model). The other eight are optional, and an absent field writes no tag, so
     a caller's strip policy (for example, no serial number) is simply a field left
     empty;
   - `LocalizedCameraModel` is ASCII in v1 like the rest. DNG also allows it as UTF-8
     bytes; a non-ASCII value is a `CallerError`, so the caller leaves it out rather
     than the export failing. Widening it to UTF-8 is a later, additive change;
   - these fields map to fixed tag numbers inside the writer. No field can name a tag,
     so none can set or override a structural tag.
4. **Encoding.** Compression 1 (uncompressed), strips, one fixed byte order
   (little-endian), `BitsPerSample` 16. No float, no lossy, no tiles, no CFA plane
   in v1. Lossless compression (Compression 7 or deflate) is a later spec on request.
5. **Previews.** Zero or one caller-encoded JPEG, stored as a reduced-resolution
   subfile (`NewSubFileType` 1, the form DEC-057's reader already returns). The
   writer never encodes a preview.
6. **`MaskedAreas` only from the caller.** irradiance has no "is this margin really
   masked" check; that judgement is the caller's (`monorail-000/DEC-075` O3). The
   options carry an explicit, possibly empty, list of masked rectangles. The writer
   validates their geometry (inside the plane, outside `ActiveArea`) and writes the
   tag only when the list is non-empty. onyx agreed in `monorail-000/DEC-076` item 3:
   `mono-core` runs its own masked check and passes the list, which is empty for the
   Q2M.
7. **API shape.**
   - **Memory in, memory out.** No paths, no `std::io` in the core.
   - **Size query, then write.** A function returns the exact output length for a
     plane and options; a second writes into a caller-owned `&mut [u8]` of exactly
     that length (the `unpack_into` shape, DEC-016). The core does not allocate the
     output. A `Vec<u8>` convenience may wrap the pair.
   - **One plain options struct**, `#[non_exhaustive]` with a `Default` (DEC-052,
     DEC-058 pattern): framing, orientation, masked areas, opcode-list bytes,
     preview bytes, EXIF block, XMP bytes, the levels and the descriptive tags (§3a).
   - **Errors** are new `Error` variants classified by `Error::kind()` (DEC-051),
     wildcard-free.
   - **Pixel budget.** The caller's `max_pixels` (DEC-050) is checked before any
     size arithmetic that could overflow and before any allocation. SPEC-061's design
     passes it as an argument, as `plane::validate` does, rather than as an options
     field (DEC-075 §8).
   - **Deterministic and stateless.** The same plane and options give the same bytes
     on every run and machine: no timestamps, no unordered maps, no global state, no
     padding left uninitialised. No panics on any caller input.

## Context

STAGE-017 was framed on 2026-11-11 as a general public writer (mono and CFA, strips
and tiles, both byte orders) waiting for a named consumer. onyx (`monorail-000`,
the same app suite as "monorail") became that consumer on 2026-11-20 and answered the
six questions in PROJ-005's brief with `monorail-000/DEC-075`. onyx then answered
this framing in `monorail-000/DEC-076` (which amends DEC-075): it agreed to §6 and to
DEC-072 and DEC-073, and asked for §3a (descriptive tags) and for XMP to be explicit
(§2). Both records are external inputs, not dependencies: they are referenced, never
put in `depends_on`.

## Alternatives Considered

- **Option A: keep STAGE-017's original scope (mono + CFA, strips + tiles, both
  byte orders).**
  - Why rejected: no consumer needs CFA, tiles or big-endian. Each is a public API
    and test-matrix cost paid ahead of its caller, which PROJ-005's brief lists as a
    risk. The test-only writer (`tests/support/dng_writer.rs`) keeps them for
    fixtures.
- **Option B: the writer copies the source's EXIF and levels itself from a parsed
  `Dng`.**
  - Why rejected: `monorail-000/DEC-075` O4 gives EXIF one owner (`diamond-exif`),
    and taking a parsed container as input would tie the public writer to the reader's
    types (DEC-055 keeps it on container types only).
- **Option C: return an owned `Vec<u8>` only.**
  - Why rejected: the FFI wrap wants a caller-owned buffer, and the size-query shape
    is already this crate's convention.

## Consequences

- **Positive:** a small public surface, testable in one spec, that onyx can wrap
  without new boundaries.
- **Negative:** CFA, tiles, compression and big-endian are each a later spec. A
  consumer wanting them waits. A `Default` options struct cannot write a file on its
  own, because `UniqueCameraModel` has no default.
- **Neutral:** the test-only writer stays in `tests/support` for fixtures; whether the
  public writer shares its code is SPEC-061's design call.

## Validation

- SPEC-061's round-trip: a written file re-parses through irradiance to the input
  samples, levels, `ActiveArea`, `DefaultCrop`, `Orientation` and opcode bytes.
- dnglab, exiftool and Adobe `dng_validate` (local only, DEC-069 §6) accept the output.
- Two writes of the same input are byte-identical.
- Revisit if a consumer asks for CFA, compression or tiles.

## References

- Related specs: SPEC-061, SPEC-081, SPEC-082
- Related decisions: DEC-016, DEC-050, DEC-051, DEC-052, DEC-055, DEC-057, DEC-058,
  DEC-072, DEC-073
- External: `monorail-000/DEC-075` (onyx Darkroom v1's writer requirements, accepted
  2026-11-20); `monorail-000/DEC-076` (onyx's answer to this framing, 2026-11-20)
