---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-081
  type: decision
  confidence: 0.8                    # the empty-string and offset choices follow measured held files and DEC-072's words; the rest is SPEC-061's pattern extended
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

created_at: 2026-11-24
supersedes: null
superseded_by: null

status: accepted                     # jysf, 2026-11-24, at the SPEC-081 ship (verify HANDOFF-192 ruled it sound)

deciders: [claude]

affected_scope:
  - src/dng/write.rs
  - src/write_types.rs

tags:
  - writer
  - exif
  - xmp
  - validation
---

# DEC-081: The writer takes metadata as typed options, checked in `plan`, and writes an empty string as an empty tag

## Decision

SPEC-081 extends the public writer (DEC-071, DEC-072, DEC-074, DEC-075) with the EXIF
block, XMP, eight descriptive tags and four rendering hints. Where those decisions leave a
choice open, the writer takes these:

1. **Shape.** The block is an `ExifBlock { bytes, exif_ifd, gps_ifd }`,
   `#[non_exhaustive]` with `Default` like `WriteOptions` (DEC-052, DEC-058). The base is
   `EXIF_BLOCK_BASE` (8) and `exif_block_base(&options)`, both as DEC-072 asks. Every new
   field is an `Option`, and `None` writes no tag.
2. **Empty values.** An empty descriptive string writes an empty ASCII tag (count 1, the
   NUL only), because the held K-3 III stores `Artist` and `Copyright` that way and
   `mono-core` copies what the source holds. An empty XMP packet
   is refused (`EmptyXmp`): a count-0 tag is malformed to irradiance's own reader
   (SPEC-089), and no source carries one. `UniqueCameraModel` keeps DEC-071 §3a's rule.
   `DateTime` is the other exception: an empty `DateTime` fails its form check (§6), so it is
   refused (`DateTimeInvalid`); leave the field `None` instead.
3. **Hint type.** The four hints are irradiance's `Rational { num: i32, den: i32 }`, what
   `mono-core` reads with `Container::rationals` (`monorail-000/DEC-078` item 1). A zero
   denominator is refused on all four (DEC-074). The three RATIONAL tags are unsigned on
   disk, so a negative numerator or denominator there is refused too (`RationalInvalid`);
   such a value cannot come from a RATIONAL source read through `Container::rationals`.
   `BaselineExposure` (SRATIONAL) takes any sign.
4. **Checks and order.** Every new check runs in `plan`, so `written_len` returns the same
   error as `write_into` (SPEC-061 §3, the parity onyx's export sheet relies on). They run
   after `UniqueCameraModel` and before the `OpcodeList1` walk, in this order: the
   descriptive strings in tag order (`DateTime`'s form with its own), the hints in tag
   order, the EXIF block, the XMP packet. Within one string field the printable-ASCII and
   length check runs first and the `DateTime` form check second, so a `DateTime` with a
   non-ASCII byte is `InvalidAscii { field: "DateTime" }`, not `DateTimeInvalid` (ruled at
   SPEC-081 verify, HANDOFF-192). Sizes stay in SPEC-061's last step
   (`OutputTooLarge`).
5. **"Inside the block"** (DEC-072) means the IFD's 2-byte entry count fits:
   `offset + 2 <= bytes.len()`, and the offset is even. The writer reads no byte of the
   block, so it cannot check more without parsing, which DEC-072 rules out.
6. **`DateTime` form** is checked, not the calendar: 19 bytes, digits at the digit places,
   `:` and one space where TIFF puts them. No range checks.
7. **Errors.** Five new `WriteInputError` reasons (`DateTimeInvalid`, `RationalInvalid`,
   `ExifBlockWithoutIfd`, `ExifIfdOffsetInvalid`, `EmptyXmp`), all `CallerError`. No new
   `Error` variant.

## Context

DEC-071 §3a, DEC-072 and DEC-074 fix what is written and that it is checked, but not the
API shape, what an empty value means, which hint type to take, where the new checks sit
in the order, or how far "inside the block" goes. SPEC-081's design probe (2026-11-24)
measured the held files: the K-3 III stores `Artist` and `Copyright` as a lone NUL, the
Q2M as four spaces, and the Typ 246 a count-0 `Copyright`. `dng_validate` 1.7.1 gives no
warnings with the source's metadata, an XMP packet and GPS added to SPEC-061's output.

## Alternatives Considered

- **Refuse an empty string, or treat it as absent.** Rejected: refusing makes `mono-core`
  special-case values held files really carry, and treating it as absent gives two inputs
  one output and drops a tag the source had. Writing it is faithful and costs nothing.
- **Write an empty XMP packet as a count-0 tag.** Rejected: irradiance's own reader treats
  count-0 interpretation tags as malformed (SPEC-089), and a writer that emits what its
  reader calls malformed is a trap.
- **Per-tag `(u32, u32)` and `(i32, i32)` pairs for the hints.** Rejected: `mono-core`
  holds `Rational`, and two types for one concept is a conversion bug waiting to happen.
  It would allow RATIONAL values past `i32::MAX`, but no source read through irradiance can
  hold them.
- **Check `offset < len` only, or walk the IFD's entries.** The first lets the IFD's count
  fall outside the block. The second is parsing, which DEC-072 rules out.
- **Put the new checks after the `OpcodeList1` walk.** Rejected: then an `Unsupported`
  refusal would hide a `CallerError`, and onyx's export sheet treats `Unsupported` as "this
  framing is unavailable", so a caller bug would look like a framing limit.

## Consequences

- **Positive:** the API matches SPEC-061's patterns, every held file's metadata round-trips
  as stored, and preflight parity holds for every new field.
- **Negative:** RATIONAL hints above `i32::MAX` cannot be written. No consumer has one.
- **Follow-up:** SPEC-061 writes an empty opcode list (`Some(&[])`) as a count-0 tag, the
  same shape §2 refuses for XMP. Refusing it would change shipped behaviour, so it is left
  for a later patch, not changed here. **Resolved by SPEC-094:** the writer refuses it
  with `WriteInputError::EmptyOpcodeList { list }` under every framing, in `plan`, before
  the `OpcodeList1` walk.

## Validation

- SPEC-081's oracles 1, 3, 4, 5 and 8, and its rule-clause map.

## References

- Related specs: SPEC-061, SPEC-081, SPEC-089
- Related decisions: DEC-052, DEC-058, DEC-071, DEC-072, DEC-074, DEC-075, DEC-080
