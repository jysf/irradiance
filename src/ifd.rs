//! TIFF/IFD container reader — bounded, panic-free, cycle-guarded.
//!
//! `SPEC-003`. This module walks the container and nothing else: it finds the
//! IFDs, reads their entries, resolves each entry's payload **inside the
//! buffer it was handed**, and locates the sensor IFD. It does not decode a
//! single pixel — `StripOffsets`/`StripByteCounts` are read here as *tags*,
//! and reading the strip they point at is `STAGE-002`, where `DEC-008`'s
//! two-path (`bits % 8`) unpack rule lands.
//!
//! # Provenance
//!
//! Written from published specifications only — TIFF 6.0 (1992) §2 "Image File
//! Header" and §2 "Image File Directory", and the Adobe DNG Specification
//! 1.7.1.0 for the DNG-private tags. Not from any implementation, and
//! explicitly not from `dnglab`/`rawler`, which are LGPL-2.1 and are run as
//! tools here, never read (`guidance/constraints.yaml`:
//! `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`). See the
//! row in `docs/provenance-ledger.md`.
//!
//! # Panic policy, and what makes it mechanical
//!
//! Every offset, length and count in a TIFF comes from the file, which means
//! it comes from whoever made the file. So every one of them is checked:
//! `.get(range)` for every slice, `try_into()` for every fixed-width read
//! (never `.get()` followed by an index — that pattern is what the crate's
//! `#![deny(clippy::indexing_slicing)]` exists to reject), `checked_*` for
//! every product or sum that could overflow, and a typed [`Error`] on every
//! failure. There is no `unwrap`, no `expect`, no `panic!`, and no `[]` on any
//! path in this file.
//!
//! # The three guards
//!
//! A TIFF is a graph, not a tree, and nothing in the format stops it being
//! cyclic:
//!
//! - **Cycle guard.** Every IFD offset reached is recorded; reaching one twice
//!   is [`Error::CyclicIfd`], not a second visit. A `SubIFD` that points at
//!   itself therefore terminates on the first revisit.
//! - **Depth guard.** `SubIFD` recursion stops at [`MAX_IFD_DEPTH`] with
//!   [`Error::IfdDepthExceeded`]. Native stack depth is bounded by that
//!   constant, which is why the recursion below is safe to write as recursion.
//! - **Count guard.** [`MAX_IFDS`] caps the total number of IFDs walked, so an
//!   acyclic-but-enormous chain cannot be used to burn unbounded time, and
//!   [`MAX_TAG_VALUES`] caps how many values one entry can be asked to
//!   materialise, so a tag declaring 4 billion `LONG`s cannot be used to
//!   allocate a Vec the size of the machine.
//!
//! Only `core` and `alloc` shapes are used here (no `std::` paths), so
//! `DEC-002`'s open `no_std + alloc` question stays open rather than being
//! decided in a build cycle.

use crate::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Limits
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum `SubIFD` nesting depth. IFD0 is depth 0.
///
/// Real DNGs nest one level (`IFD0` → `SubIFD`); TIFF/EP and Exif chains add a
/// second. Eight is far past anything a camera writes and far short of
/// anything that could exhaust the stack.
pub const MAX_IFD_DEPTH: u32 = 8;

/// Maximum number of IFDs walked in one container.
///
/// The cycle guard already makes a walk terminate; this bounds how long
/// terminating takes on a file with a very long acyclic chain.
pub const MAX_IFDS: usize = 512;

/// Maximum number of values one entry may be asked to materialise.
///
/// Bounds allocation from a hostile `count` independently of the file's own
/// length. A tiled 8K DNG has a few thousand strip offsets; this is 1,048,576.
pub const MAX_TAG_VALUES: usize = 1 << 20;

/// TIFF 6.0 §2: the 42 that follows the byte-order mark.
const TIFF_VERSION: u16 = 42;

/// TIFF 6.0 §2: every IFD entry is exactly 12 bytes.
const ENTRY_SIZE: usize = 12;

// ─────────────────────────────────────────────────────────────────────────────
// Tags this module needs
// ─────────────────────────────────────────────────────────────────────────────
//
// The tags SPEC-003's container walk needs, plus SPEC-004's typed extraction
// of the develop pipeline's remaining DNG integer tags. RATIONAL, ASCII and
// the signed field types are still absent — no tag PROJ-001 reads needs them.

/// TIFF 6.0: `NewSubfileType`. **Default 0 when absent** — that default is
/// load-bearing, see [`Container::sensor_ifd`].
pub const TAG_NEW_SUBFILE_TYPE: u16 = 254;
/// TIFF 6.0: `ImageWidth`.
pub const TAG_IMAGE_WIDTH: u16 = 256;
/// TIFF 6.0: `ImageLength` (height).
pub const TAG_IMAGE_LENGTH: u16 = 257;
/// TIFF 6.0: `BitsPerSample`.
pub const TAG_BITS_PER_SAMPLE: u16 = 258;
/// TIFF 6.0: `Compression`.
pub const TAG_COMPRESSION: u16 = 259;
/// TIFF 6.0: `PhotometricInterpretation`.
pub const TAG_PHOTOMETRIC: u16 = 262;
/// TIFF 6.0: `StripOffsets`.
pub const TAG_STRIP_OFFSETS: u16 = 273;
/// TIFF 6.0: `Orientation`.
pub const TAG_ORIENTATION: u16 = 274;
/// TIFF 6.0: `SamplesPerPixel`. Default 1 when absent.
pub const TAG_SAMPLES_PER_PIXEL: u16 = 277;
/// TIFF 6.0: `RowsPerStrip`.
pub const TAG_ROWS_PER_STRIP: u16 = 278;
/// TIFF 6.0: `StripByteCounts`.
pub const TAG_STRIP_BYTE_COUNTS: u16 = 279;
/// TIFF/EP + DNG: `SubIFDs` — the offsets this walk recurses into.
pub const TAG_SUB_IFDS: u16 = 330;
/// DNG 1.7 §Chapter 4: `BlackLevelRepeatDim`. Two SHORTs, and one real camera
/// writes one — see [`Sensor::malformed_tags`].
pub const TAG_BLACK_LEVEL_REPEAT_DIM: u16 = 50713;
/// DNG 1.7 §Chapter 4: `BlackLevel`.
pub const TAG_BLACK_LEVEL: u16 = 50714;
/// DNG 1.7 §Chapter 4: `WhiteLevel`.
pub const TAG_WHITE_LEVEL: u16 = 50717;
/// DNG 1.7 §Chapter 4: `DefaultCropOrigin`.
pub const TAG_DEFAULT_CROP_ORIGIN: u16 = 50719;
/// DNG 1.7 §Chapter 4: `DefaultCropSize`.
pub const TAG_DEFAULT_CROP_SIZE: u16 = 50720;
/// DNG 1.7 §Chapter 4: `ActiveArea`.
pub const TAG_ACTIVE_AREA: u16 = 50829;
/// DNG 1.7 §Chapter 6: `OpcodeList1` — applied to the raw plane.
pub const TAG_OPCODE_LIST_1: u16 = 51008;
/// DNG 1.7 §Chapter 6: `OpcodeList2`.
pub const TAG_OPCODE_LIST_2: u16 = 51009;
/// DNG 1.7 §Chapter 6: `OpcodeList3` — applied after demosaic.
pub const TAG_OPCODE_LIST_3: u16 = 51022;

/// DNG 1.7: `PhotometricInterpretation = LinearRaw`. The sensor plane *is* the
/// image; there is no CFA to demosaic (`docs/measured-q2m-dng.md`).
pub const PHOTOMETRIC_LINEAR_RAW: u32 = 34892;

// TIFF 6.0 §2, "Types". Only the ones this module reads as integers are named.
const TYPE_BYTE: u16 = 1;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
/// TIFF 6.0 §2: two `LONG`s, numerator then denominator. `SPEC-007` —
/// several DNG tags this reader extracts (`BlackLevel`, `DefaultCropSize`
/// among them) legally use this type; a zero denominator or a non-integral
/// ratio is `Error::MalformedRationalValue`, read in [`Container::uints`].
const TYPE_RATIONAL: u16 = 5;
const TYPE_UNDEFINED: u16 = 7;
const TYPE_IFD: u16 = 13;

/// Size in bytes of one value of a TIFF field type (TIFF 6.0 §2, "Types";
/// type 13 `IFD` from TIFF/EP). `None` for a type this reader does not know —
/// TIFF says a reader should skip such a field rather than reject the file, so
/// the entry is still recorded and only reading its payload fails.
fn type_size(field_type: u16) -> Option<u64> {
    match field_type {
        1 | 2 | 6 | 7 => Some(1),   // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => Some(2),           // SHORT, SSHORT
        4 | 9 | 11 | 13 => Some(4), // LONG, SLONG, FLOAT, IFD
        5 | 10 | 12 => Some(8),     // RATIONAL, SRATIONAL, DOUBLE
        _ => None,
    }
}

/// `DEC-012`'s Structure class, restated as a predicate so [`Container::uints`]
/// can reject `TYPE_RATIONAL` **per-tag** instead of globally (`SPEC-007`
/// `FU-4`).
///
/// On `main`, every tag read through `uints()` rejected `RATIONAL`, because
/// none of them are legal that way. `SPEC-007` widened `uints()`'s type-gate
/// match arm to let a DNG-legal RATIONAL `DefaultCropSize` read instead of
/// erroring — but that match arm is `uints()`'s only gate, shared by every
/// tag that flows through it, so the widening silently applied to every
/// *other* tag too. Measured consequence: a `SubIFDs` (330) entry encoded as
/// `RATIONAL 400/2` (= 200, a valid-looking offset) walked the SubIFD, where
/// `main` returned `Err(UnexpectedFieldType)`; `StripByteCounts` (279) as
/// `RATIONAL 28/2` silently read `[14]`. `DEC-012` names both tags
/// structural — "what exists" must not depend on an out-of-spec encoding a
/// reader happens to tolerate.
///
/// This list is exactly `DEC-012`'s amended Structure row (the walk-phase
/// items — the header, entry tables, and chain `next` — never reach
/// `uints()` at all, so they need no entry here). Interpretation tags are
/// *not* listed, so they keep the RATIONAL widening `SPEC-007` added them
/// for (`FU-17`: `BlackLevel`, `DefaultCropOrigin`, `DefaultCropSize` and
/// friends may legally be RATIONAL in DNG).
fn is_structural_tag(tag: u16) -> bool {
    matches!(
        tag,
        TAG_NEW_SUBFILE_TYPE
            | TAG_IMAGE_WIDTH
            | TAG_IMAGE_LENGTH
            | TAG_BITS_PER_SAMPLE
            | TAG_COMPRESSION
            | TAG_PHOTOMETRIC
            | TAG_STRIP_OFFSETS
            | TAG_SAMPLES_PER_PIXEL
            | TAG_ROWS_PER_STRIP
            | TAG_STRIP_BYTE_COUNTS
            | TAG_SUB_IFDS
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Byte order
// ─────────────────────────────────────────────────────────────────────────────

/// TIFF 6.0 §2: the container's byte order, from the `II`/`MM` header mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// `II` — little-endian (Intel).
    Little,
    /// `MM` — big-endian (Motorola).
    Big,
}

impl ByteOrder {
    fn u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            ByteOrder::Little => u16::from_le_bytes(bytes),
            ByteOrder::Big => u16::from_be_bytes(bytes),
        }
    }

    fn u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            ByteOrder::Little => u32::from_le_bytes(bytes),
            ByteOrder::Big => u32::from_be_bytes(bytes),
        }
    }
}

/// Read a `u16` at `at`. Bounds-checked with `.get()` and converted with
/// `try_into()` — never `.get()` then index.
fn read_u16(data: &[u8], at: usize, order: ByteOrder) -> Result<u16, Error> {
    let end = at.checked_add(2).ok_or(Error::Truncated { at, len: 2 })?;
    let bytes: [u8; 2] = data
        .get(at..end)
        .ok_or(Error::Truncated { at, len: 2 })?
        .try_into()
        .map_err(|_| Error::Truncated { at, len: 2 })?;
    Ok(order.u16(bytes))
}

/// Read a `u32` at `at`. Same discipline as [`read_u16`].
fn read_u32(data: &[u8], at: usize, order: ByteOrder) -> Result<u32, Error> {
    let end = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;
    let bytes: [u8; 4] = data
        .get(at..end)
        .ok_or(Error::Truncated { at, len: 4 })?
        .try_into()
        .map_err(|_| Error::Truncated { at, len: 4 })?;
    Ok(order.u32(bytes))
}

/// `u32` → `usize`, typed rather than `as`. Infallible on 64-bit; on a 32-bit
/// host a file offset above `usize::MAX` is genuinely unreadable, and this is
/// what says so instead of truncating silently.
fn offset_to_usize(offset: u32) -> Result<usize, Error> {
    usize::try_from(offset).map_err(|_| Error::OffsetOutOfRange { offset })
}

// ─────────────────────────────────────────────────────────────────────────────
// Entries and IFDs
// ─────────────────────────────────────────────────────────────────────────────

/// One 12-byte IFD entry (TIFF 6.0 §2): tag, type, count, and a 4-byte field
/// that holds the value itself when it fits and an offset to it when it does
/// not.
///
/// The 4-byte field is kept **raw**. Whether it is a value or an offset
/// depends on `count × sizeof(type)`, which is exactly the product a hostile
/// file wants to overflow — so it is computed once, checked, in
/// [`Container::payload`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    tag: u16,
    field_type: u16,
    count: u32,
    value: [u8; 4],
}

impl Entry {
    /// The tag number.
    pub fn tag(&self) -> u16 {
        self.tag
    }

    /// The TIFF field type (TIFF 6.0 §2, "Types").
    pub fn field_type(&self) -> u16 {
        self.field_type
    }

    /// The declared value count. **Unvalidated** — this is a number from the
    /// file, and it is only trustworthy after [`Container::payload`] has
    /// checked it against the buffer.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Total payload size in bytes, or `None` for an unknown field type or a
    /// `count × size` that overflows.
    pub fn byte_len(&self) -> Option<u64> {
        type_size(self.field_type)?.checked_mul(u64::from(self.count))
    }

    fn parse(chunk: &[u8], order: ByteOrder) -> Result<Entry, Error> {
        let tag: [u8; 2] = chunk
            .get(0..2)
            .ok_or(Error::Truncated {
                at: 0,
                len: ENTRY_SIZE,
            })?
            .try_into()
            .map_err(|_| Error::Truncated {
                at: 0,
                len: ENTRY_SIZE,
            })?;
        let field_type: [u8; 2] = chunk
            .get(2..4)
            .ok_or(Error::Truncated {
                at: 2,
                len: ENTRY_SIZE,
            })?
            .try_into()
            .map_err(|_| Error::Truncated {
                at: 2,
                len: ENTRY_SIZE,
            })?;
        let count: [u8; 4] = chunk
            .get(4..8)
            .ok_or(Error::Truncated {
                at: 4,
                len: ENTRY_SIZE,
            })?
            .try_into()
            .map_err(|_| Error::Truncated {
                at: 4,
                len: ENTRY_SIZE,
            })?;
        let value: [u8; 4] = chunk
            .get(8..12)
            .ok_or(Error::Truncated {
                at: 8,
                len: ENTRY_SIZE,
            })?
            .try_into()
            .map_err(|_| Error::Truncated {
                at: 8,
                len: ENTRY_SIZE,
            })?;

        Ok(Entry {
            tag: order.u16(tag),
            field_type: order.u16(field_type),
            count: order.u32(count),
            value,
        })
    }
}

/// One Image File Directory: its entries, where it was found, and how it was
/// reached.
#[derive(Debug, Clone)]
pub struct Ifd {
    offset: u32,
    depth: u32,
    parent: Option<usize>,
    next: u32,
    entries: Vec<Entry>,
}

impl Ifd {
    /// Byte offset this IFD was read from.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// `SubIFD` nesting depth; IFD0 and its chain are depth 0.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Index of the IFD whose `SubIFDs` tag pointed here, if any.
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    /// Offset of the next IFD in this chain, or 0 for the last.
    pub fn next(&self) -> u32 {
        self.next
    }

    /// Every entry, in file order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The first entry with this tag, if present.
    pub fn entry(&self, tag: u16) -> Option<&Entry> {
        self.entries.iter().find(|e| e.tag == tag)
    }

    /// Whether this IFD carries a tag at all — presence without reading the
    /// payload, which is all the opcode lists need here.
    pub fn has(&self, tag: u16) -> bool {
        self.entry(tag).is_some()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compression
// ─────────────────────────────────────────────────────────────────────────────

/// TIFF 6.0 `Compression`, narrowed to what PROJ-001 can and cannot read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// 1 — no compression. The only kind PROJ-001 unpacks.
    Uncompressed,
    /// 7 — JPEG (lossless SOF-3 in a DNG). Rejected cleanly here; reading it
    /// is PROJ-003's problem, not this library's today.
    Jpeg,
    /// Anything else, including vendor-private schemes (Pentax PEF is 65535).
    ///
    /// Held as `u32` because that is what the tag reader returns; TIFF writes
    /// `Compression` as a `SHORT`, but narrowing a value that came from the
    /// file just to fit an assumption is how a hostile value gets silently
    /// rewritten into a plausible one.
    Other(u32),
}

impl Compression {
    /// The raw TIFF code, whichever variant this is.
    pub fn code(self) -> u32 {
        match self {
            Compression::Uncompressed => 1,
            Compression::Jpeg => 7,
            Compression::Other(code) => code,
        }
    }

    fn from_code(code: u32) -> Compression {
        match code {
            1 => Compression::Uncompressed,
            7 => Compression::Jpeg,
            other => Compression::Other(other),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Typed geometry — SPEC-004 acceptance criterion 1
// ─────────────────────────────────────────────────────────────────────────────
//
// A bare `[u32; N]` makes the caller remember an order the DNG spec defines
// but the type does not. These structs carry that order in their field names
// instead, so `sensor.active_area.top` cannot be confused with `.left` the
// way `active_area[0]` invites.

/// DNG `ActiveArea` (tag 50829, DNG 1.7 §Chapter 4): the rectangle of valid
/// pixels within the full sensor image, in the tag's own (top, left, bottom,
/// right) order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveArea {
    /// Top edge, in raw pixel rows.
    pub top: u32,
    /// Left edge, in raw pixel columns.
    pub left: u32,
    /// Bottom edge (exclusive), in raw pixel rows.
    pub bottom: u32,
    /// Right edge (exclusive), in raw pixel columns.
    pub right: u32,
}

/// DNG `DefaultCropOrigin` (tag 50719, DNG 1.7 §Chapter 4): the origin of the
/// final image area, in raw pixel coordinates relative to `ActiveArea`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultCropOrigin {
    /// Horizontal offset.
    pub x: u32,
    /// Vertical offset.
    pub y: u32,
}

/// DNG `DefaultCropSize` (tag 50720, DNG 1.7 §Chapter 4): the width and
/// height of the final image area, in raw pixel units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultCropSize {
    /// Width, in raw pixel columns.
    pub width: u32,
    /// Height, in raw pixel rows.
    pub height: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// The sensor summary
// ─────────────────────────────────────────────────────────────────────────────

/// What the sensor IFD says about the plane, read as tags.
///
/// This is the container's answer, not a decode: nothing here has looked at a
/// single pixel. It is exactly the field set `SPEC-003` acceptance criterion 6
/// requires be cross-checked against `exiftool`, and `src/bin/irr.rs`'s
/// `ifd` subcommand is what surfaces it (AGENTS.md §11, "ship the reader with
/// the field").
#[derive(Debug, Clone)]
pub struct Sensor {
    /// Index into [`Container::ifds`] of the IFD this came from.
    pub ifd_index: usize,
    /// `ImageWidth`.
    pub width: u32,
    /// `ImageLength`.
    pub height: u32,
    /// `BitsPerSample` — the first value; monochrome planes have exactly one.
    pub bits_per_sample: u32,
    /// `SamplesPerPixel` (1 for every file PROJ-001 targets).
    pub samples_per_pixel: u32,
    /// `PhotometricInterpretation` — 34892 by construction of the selection.
    pub photometric: u32,
    /// `Compression`.
    pub compression: Compression,
    /// `RowsPerStrip`, when present.
    pub rows_per_strip: Option<u32>,
    /// `StripOffsets` — read as a tag. Reading the strip is STAGE-002.
    pub strip_offsets: Vec<u32>,
    /// `StripByteCounts` — likewise a tag, and the layer-0 oracle's right-hand
    /// side (see [`Sensor::packed_bits`]).
    pub strip_byte_counts: Vec<u32>,
    /// DNG `BlackLevel`, when present.
    pub black_level: Option<u32>,
    /// DNG `WhiteLevel`, when present.
    pub white_level: Option<u32>,
    /// DNG `BlackLevelRepeatDim`, when present **and well-formed**.
    pub black_level_repeat_dim: Option<[u32; 2]>,
    /// DNG `ActiveArea`, when present. **Absent, not a zero rectangle** —
    /// the M Monochrom writes no `ActiveArea` at all, and this is `None`
    /// there, never `Some(ActiveArea { top: 0, .. })` (acceptance criterion
    /// 3).
    pub active_area: Option<ActiveArea>,
    /// DNG `DefaultCropOrigin`, when present.
    pub default_crop_origin: Option<DefaultCropOrigin>,
    /// DNG `DefaultCropSize`, when present.
    pub default_crop_size: Option<DefaultCropSize>,
    /// `Orientation`, read from IFD0 and falling back to the sensor IFD.
    ///
    /// Per-frame, never a camera constant: two frames from the same body
    /// differ (`docs/measured-q2m-dng.md`).
    pub orientation: Option<u32>,
    /// Presence of `OpcodeList1`/`2`/`3`. Presence only — decoding the opcode
    /// streams is STAGE-003.
    pub opcode_lists: [bool; 3],
    /// Tags that are **present but shaped wrong**, recorded rather than
    /// rejected.
    ///
    /// A shipping Pentax K-3 III Monochrome writes `BlackLevelRepeatDim` with
    /// `count = 1` where DNG requires 2 — `dnglab` itself warns about it. A
    /// malformed optional tag must not cost the file: it is dropped from its
    /// typed field, listed here, and the walk continues.
    pub malformed_tags: Vec<u16>,
}

impl Sensor {
    /// Reject a plane this library cannot unpack, with the code in the error.
    ///
    /// Three of the seven corpus files are compressed (two JPEG, one Pentax
    /// PEF) and this is what makes their rejection *clean* — a typed error
    /// naming the compression, not a decode attempt and not a panic.
    pub fn require_uncompressed(&self) -> Result<(), Error> {
        match self.compression {
            Compression::Uncompressed => Ok(()),
            other => Err(Error::UnsupportedCompression {
                compression: other.code(),
            }),
        }
    }

    /// The layer-0 oracle: `width × height × bits_per_sample`, in **bits**.
    ///
    /// For a tightly-packed plane with no row padding this must equal
    /// `StripByteCounts × 8` — the check `AGENTS.md` §12 bar 3 calls free,
    /// because it needs no oracle tooling, no corpus and no network. Returned
    /// in bits, not bytes, so that this module states the invariant without
    /// pre-empting `DEC-008`'s two-path (`bits % 8`) rule, which is STAGE-002's
    /// to make.
    pub fn packed_bits(&self) -> Result<u64, Error> {
        u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .and_then(|px| px.checked_mul(u64::from(self.bits_per_sample)))
            .ok_or(Error::ValueOverflow {
                tag: TAG_IMAGE_WIDTH,
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The container
// ─────────────────────────────────────────────────────────────────────────────

/// A parsed TIFF container: every IFD reachable from IFD0, flattened.
///
/// Borrows the input; nothing is copied out of it except the 12-byte entry
/// headers. `SPEC-003` scope: the walk and the tags, never the strip.
#[derive(Debug, Clone)]
pub struct Container<'a> {
    data: &'a [u8],
    byte_order: ByteOrder,
    ifd0_offset: u32,
    ifds: Vec<Ifd>,
}

/// The outcome of testing one IFD against the sensor-selection rule
/// (`NewSubfileType == 0 && PhotometricInterpretation == 34892 (LinearRaw) &&
/// SamplesPerPixel == 1`, see [`Container::is_sensor_ifd`]).
///
/// Not a `Result`: a malformed identifying tag is not an *error* to a caller
/// scanning every IFD for the plane, it is a **third possible answer**,
/// distinct from "no". Coercing it into [`SensorMatch::No`] would silently
/// hide a real plane behind a bad tag if the IFD in question is, in fact, the
/// sensor IFD — the obvious-looking fix `DEC-012` and `SPEC-004`'s FU-11 both
/// reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorMatch {
    /// This IFD matches the rule.
    Yes,
    /// This IFD does not match the rule — a confident negative.
    No,
    /// One of the three identifying tags could not be read. This IFD's
    /// identity is *unknown*, not "not the sensor". Carries the tag that
    /// failed.
    Unreadable(u16),
}

impl<'a> Container<'a> {
    /// Parse the header and walk every IFD reachable from IFD0.
    ///
    /// Depth-first: an IFD is recorded, then its `SubIFDs` are walked, then the
    /// chain continues. Fails with a typed [`Error`] on a bad header, a
    /// truncated read, a cycle, or a walk that exceeds [`MAX_IFD_DEPTH`] or
    /// [`MAX_IFDS`].
    pub fn parse(data: &'a [u8]) -> Result<Container<'a>, Error> {
        let mark: [u8; 2] = data
            .get(0..2)
            .ok_or(Error::Truncated { at: 0, len: 8 })?
            .try_into()
            .map_err(|_| Error::Truncated { at: 0, len: 8 })?;
        let byte_order = match &mark {
            b"II" => ByteOrder::Little,
            b"MM" => ByteOrder::Big,
            _ => return Err(Error::NotTiff { mark }),
        };

        let version = read_u16(data, 2, byte_order)?;
        if version != TIFF_VERSION {
            return Err(Error::UnsupportedTiffVersion { version });
        }
        let ifd0_offset = read_u32(data, 4, byte_order)?;

        let mut container = Container {
            data,
            byte_order,
            ifd0_offset,
            ifds: Vec::new(),
        };
        let mut visited: Vec<u32> = Vec::new();
        container.walk_chain(ifd0_offset, 0, None, &mut visited)?;
        Ok(container)
    }

    /// The container's byte order.
    pub fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }

    /// Offset of IFD0, as the header declared it.
    pub fn ifd0_offset(&self) -> u32 {
        self.ifd0_offset
    }

    /// Every IFD walked, in depth-first order. IFD0 is first.
    pub fn ifds(&self) -> &[Ifd] {
        &self.ifds
    }

    /// IFD0 — present whenever [`Container::parse`] succeeded with a non-zero
    /// IFD0 offset.
    pub fn ifd0(&self) -> Option<&Ifd> {
        self.ifds.first()
    }

    // ── the walk ────────────────────────────────────────────────────────────

    /// Walk one IFD chain, recursing into each IFD's `SubIFDs`.
    ///
    /// Recursion is bounded by [`MAX_IFD_DEPTH`], so native stack depth is
    /// bounded too; the chain itself is iterative because chains are long and
    /// their length is attacker-chosen.
    fn walk_chain(
        &mut self,
        start: u32,
        depth: u32,
        parent: Option<usize>,
        visited: &mut Vec<u32>,
    ) -> Result<(), Error> {
        if depth > MAX_IFD_DEPTH {
            return Err(Error::IfdDepthExceeded {
                limit: MAX_IFD_DEPTH,
            });
        }

        let mut offset = start;
        while offset != 0 {
            // Cycle guard. `visited` spans the whole walk, not just this
            // chain, so a SubIFD pointing back at IFD0 is caught too.
            if visited.contains(&offset) {
                return Err(Error::CyclicIfd { offset });
            }
            visited.push(offset);

            if self.ifds.len() >= MAX_IFDS {
                return Err(Error::TooManyIfds { limit: MAX_IFDS });
            }

            let (entries, next) = self.read_ifd(offset)?;
            // Index this IFD will occupy — the parent link for its SubIFDs.
            let index = self.ifds.len();
            self.ifds.push(Ifd {
                offset,
                depth,
                parent,
                next,
                entries,
            });

            // Materialise the SubIFD offsets before recursing: the Vec ends
            // the borrow of `self.ifds`, which the recursive call mutates.
            let subs = self.sub_ifd_offsets_of_last()?;
            for sub in subs {
                self.walk_chain(sub, depth.saturating_add(1), Some(index), visited)?;
            }

            offset = next;
        }
        Ok(())
    }

    /// Read the entry table at `offset` and the chain pointer after it.
    fn read_ifd(&self, offset: u32) -> Result<(Vec<Entry>, u32), Error> {
        let at = offset_to_usize(offset)?;
        let count = read_u16(self.data, at, self.byte_order)?;

        let entries_at = at.checked_add(2).ok_or(Error::Truncated { at, len: 2 })?;
        let table_len = usize::from(count)
            .checked_mul(ENTRY_SIZE)
            .ok_or(Error::Truncated {
                at: entries_at,
                len: ENTRY_SIZE,
            })?;
        let entries_end = entries_at.checked_add(table_len).ok_or(Error::Truncated {
            at: entries_at,
            len: table_len,
        })?;
        let table = self
            .data
            .get(entries_at..entries_end)
            .ok_or(Error::Truncated {
                at: entries_at,
                len: table_len,
            })?;

        let mut entries = Vec::with_capacity(usize::from(count));
        for chunk in table.chunks_exact(ENTRY_SIZE) {
            entries.push(Entry::parse(chunk, self.byte_order)?);
        }

        let next = read_u32(self.data, entries_end, self.byte_order)?;
        Ok((entries, next))
    }

    /// The `SubIFDs` offsets of the IFD just pushed, or an empty Vec.
    ///
    /// Reads `last()` rather than an index so there is no lookup that could
    /// fail and no unrelated error variant standing in for "impossible".
    ///
    /// See `DEC-012`: this is the **walk**-phase, and deliberately strict — a
    /// malformed `SubIFDs` entry is fatal to the whole container, unlike
    /// `array()`'s tag-level tolerance below.
    fn sub_ifd_offsets_of_last(&self) -> Result<Vec<u32>, Error> {
        match self.ifds.last().and_then(|ifd| ifd.entry(TAG_SUB_IFDS)) {
            None => Ok(Vec::new()),
            Some(entry) => self.uints(entry),
        }
    }

    // ── payloads ────────────────────────────────────────────────────────────

    /// The raw payload bytes of an entry.
    ///
    /// Inline when `count × sizeof(type) <= 4`, otherwise the bytes at the
    /// offset the entry carries — bounds-checked against the whole buffer.
    /// This is the one place the hostile product is computed, and it is
    /// computed in `u64` and `checked_mul`.
    pub fn payload<'s>(&'s self, entry: &'s Entry) -> Result<&'s [u8], Error> {
        let size = type_size(entry.field_type).ok_or(Error::UnexpectedFieldType {
            tag: entry.tag,
            field_type: entry.field_type,
        })?;
        let len_u64 = u64::from(entry.count)
            .checked_mul(size)
            .ok_or(Error::ValueOverflow { tag: entry.tag })?;
        let len = usize::try_from(len_u64).map_err(|_| Error::ValueOverflow { tag: entry.tag })?;

        if len <= 4 {
            // Fits in the entry itself (TIFF 6.0 §2, "Value/Offset").
            entry
                .value
                .get(..len)
                .ok_or(Error::ValueOverflow { tag: entry.tag })
        } else {
            let at = offset_to_usize(self.byte_order.u32(entry.value))?;
            let end = at.checked_add(len).ok_or(Error::Truncated { at, len })?;
            self.data.get(at..end).ok_or(Error::Truncated { at, len })
        }
    }

    /// An entry's values widened to `u32`.
    ///
    /// Handles `BYTE`, `SHORT`, `LONG`, `IFD`, `UNDEFINED` and `RATIONAL` —
    /// every DNG tag PROJ-001 reads. The signed types and `ASCII` remain
    /// unimplemented, still absent by design; asking for one here is a typed
    /// [`Error::UnexpectedFieldType`] rather than a silent zero. A `RATIONAL`
    /// whose shape is malformed (zero denominator, non-integral ratio) is
    /// [`Error::MalformedRationalValue`] — `SPEC-007`, `SPEC-004` FU-17.
    pub fn uints(&self, entry: &Entry) -> Result<Vec<u32>, Error> {
        // The field type is a property of the entry, so it is checked before
        // the buffer is touched: otherwise a RATIONAL that also happens to
        // point out of bounds reports `Truncated` and hides the real problem.
        //
        // RATIONAL's acceptance is gated PER-TAG, ahead of the general match
        // below — see [`is_structural_tag`] for why (`SPEC-007/FU-4`).
        if entry.field_type == TYPE_RATIONAL && is_structural_tag(entry.tag) {
            return Err(Error::UnexpectedFieldType {
                tag: entry.tag,
                field_type: entry.field_type,
            });
        }
        match entry.field_type {
            TYPE_BYTE | TYPE_UNDEFINED | TYPE_SHORT | TYPE_LONG | TYPE_IFD | TYPE_RATIONAL => {}
            other => {
                return Err(Error::UnexpectedFieldType {
                    tag: entry.tag,
                    field_type: other,
                })
            }
        }

        let count =
            usize::try_from(entry.count).map_err(|_| Error::ValueOverflow { tag: entry.tag })?;
        if count > MAX_TAG_VALUES {
            return Err(Error::TagTooLarge {
                tag: entry.tag,
                count: entry.count,
                limit: MAX_TAG_VALUES,
            });
        }

        let bytes = self.payload(entry)?;
        let mut out = Vec::with_capacity(count);
        match entry.field_type {
            TYPE_BYTE | TYPE_UNDEFINED => {
                for b in bytes {
                    out.push(u32::from(*b));
                }
            }
            TYPE_SHORT => {
                for chunk in bytes.chunks_exact(2) {
                    let raw: [u8; 2] = chunk
                        .try_into()
                        .map_err(|_| Error::ValueOverflow { tag: entry.tag })?;
                    out.push(u32::from(self.byte_order.u16(raw)));
                }
            }
            TYPE_LONG | TYPE_IFD => {
                for chunk in bytes.chunks_exact(4) {
                    let raw: [u8; 4] = chunk
                        .try_into()
                        .map_err(|_| Error::ValueOverflow { tag: entry.tag })?;
                    out.push(self.byte_order.u32(raw));
                }
            }
            TYPE_RATIONAL => {
                // TIFF 6.0 §2: two LONGs, numerator then denominator. Reading
                // this correctly (rather than just refusing it) is what
                // closes `SPEC-004`'s FU-17 for the well-formed case.
                for chunk in bytes.chunks_exact(8) {
                    let num_bytes: [u8; 4] = chunk
                        .get(0..4)
                        .ok_or(Error::ValueOverflow { tag: entry.tag })?
                        .try_into()
                        .map_err(|_| Error::ValueOverflow { tag: entry.tag })?;
                    let den_bytes: [u8; 4] = chunk
                        .get(4..8)
                        .ok_or(Error::ValueOverflow { tag: entry.tag })?
                        .try_into()
                        .map_err(|_| Error::ValueOverflow { tag: entry.tag })?;
                    let numerator = self.byte_order.u32(num_bytes);
                    let denominator = self.byte_order.u32(den_bytes);
                    // `checked_div`/`checked_rem` both report zero
                    // denominator as `None`, so that shape and a
                    // non-integral ratio are the same malformed-value error.
                    match (
                        numerator.checked_div(denominator),
                        numerator.checked_rem(denominator),
                    ) {
                        (Some(value), Some(0)) => out.push(value),
                        _ => {
                            return Err(Error::MalformedRationalValue {
                                tag: entry.tag,
                                numerator,
                                denominator,
                            })
                        }
                    }
                }
            }
            // Unreachable given the guard above, and kept because the match
            // must stay total if a type is ever added to one arm and not the
            // other.
            other => {
                return Err(Error::UnexpectedFieldType {
                    tag: entry.tag,
                    field_type: other,
                })
            }
        }
        Ok(out)
    }

    /// First value of `tag` in `ifd`, or `None` when the tag is absent.
    pub fn scalar(&self, ifd: &Ifd, tag: u16) -> Result<Option<u32>, Error> {
        match ifd.entry(tag) {
            None => Ok(None),
            Some(entry) => Ok(self.uints(entry)?.first().copied()),
        }
    }

    /// First value of `tag` in `ifd`, or [`Error::MissingTag`].
    pub fn required_scalar(&self, ifd: &Ifd, tag: u16) -> Result<u32, Error> {
        self.scalar(ifd, tag)?.ok_or(Error::MissingTag { tag })
    }

    /// All values of `tag` in `ifd`, or an empty Vec when absent.
    pub fn values(&self, ifd: &Ifd, tag: u16) -> Result<Vec<u32>, Error> {
        match ifd.entry(tag) {
            None => Ok(Vec::new()),
            Some(entry) => self.uints(entry),
        }
    }

    /// A tag whose DNG shape is a fixed-length array.
    ///
    /// Present-but-wrong-length is **not** an error: the tag is dropped and its
    /// number is pushed to `malformed`. That is what keeps the Pentax's
    /// one-element `BlackLevelRepeatDim` from costing the whole file.
    ///
    /// See `DEC-012`: this is the **interpret**-phase tolerance — a malformed
    /// *optional* tag costs that tag alone, never the container.
    fn array<const N: usize>(
        &self,
        ifd: &Ifd,
        tag: u16,
        malformed: &mut Vec<u16>,
    ) -> Result<Option<[u32; N]>, Error> {
        let Some(entry) = ifd.entry(tag) else {
            return Ok(None);
        };
        let values = self.uints(entry)?;
        if values.len() != N {
            malformed.push(tag);
            return Ok(None);
        }
        let mut out = [0u32; N];
        for (slot, value) in out.iter_mut().zip(values.iter()) {
            *slot = *value;
        }
        Ok(Some(out))
    }

    // ── sensor selection ────────────────────────────────────────────────────

    /// Whether this IFD is the full-resolution sensor plane.
    ///
    /// `NewSubfileType == 0 && PhotometricInterpretation == 34892 (LinearRaw)
    /// && SamplesPerPixel == 1` — **never by largest dimensions**. A Leica Q2
    /// Monochrom DNG carries `SubIFD2`, a full-resolution *JPEG preview* only
    /// 56 px narrower than the plane (8368 against 8424), so a size heuristic
    /// is one firmware revision away from silently returning the preview
    /// (`docs/measured-q2m-dng.md`).
    ///
    /// Both defaults are TIFF 6.0's and both are load-bearing: a Pentax `.PEF`
    /// puts its plane in IFD0 and writes **no** `NewSubfileType` at all, so
    /// absent-means-0 is what finds it.
    ///
    /// Returns [`SensorMatch`], not `Result<bool, Error>` — see FU-11 below
    /// for why a `?`-shaped answer here was the defect, not the fix.
    ///
    /// Each identifying tag is read **and evaluated** before the next one is
    /// even read (`SPEC-004` FU-20): a readable tag that already disqualifies
    /// the IFD returns `No` immediately, rather than reading the remaining
    /// tags and reporting `Unreadable` for an IFD that was never a candidate
    /// in the first place. Reading all three unconditionally, then combining,
    /// was the bug — an IFD with `NewSubfileType == 1` (a preview) is a
    /// confident `No` even when its `Photometric` also happens to be
    /// malformed.
    pub fn is_sensor_ifd(&self, ifd: &Ifd) -> SensorMatch {
        let subfile_type = match self.scalar(ifd, TAG_NEW_SUBFILE_TYPE) {
            Ok(v) => v.unwrap_or(0),
            Err(_) => return SensorMatch::Unreadable(TAG_NEW_SUBFILE_TYPE),
        };
        if subfile_type != 0 {
            return SensorMatch::No;
        }

        let photometric = match self.scalar(ifd, TAG_PHOTOMETRIC) {
            Ok(v) => v,
            Err(_) => return SensorMatch::Unreadable(TAG_PHOTOMETRIC),
        };
        if photometric != Some(PHOTOMETRIC_LINEAR_RAW) {
            return SensorMatch::No;
        }

        let samples = match self.scalar(ifd, TAG_SAMPLES_PER_PIXEL) {
            Ok(v) => v.unwrap_or(1),
            Err(_) => return SensorMatch::Unreadable(TAG_SAMPLES_PER_PIXEL),
        };
        if samples == 1 {
            SensorMatch::Yes
        } else {
            SensorMatch::No
        }
    }

    /// Every IFD's outcome against [`Container::is_sensor_ifd`]: the matching
    /// indices, and every `(index, tag)` whose identity could not be
    /// determined at all.
    ///
    /// **FU-11.** `is_sensor_ifd` used to be `Result<bool, Error>`, and
    /// `sensor_candidates`/`sensor_ifd`/`sensor` each `?`-propagated it over
    /// *every* IFD — so a malformed identifying tag on a thumbnail aborted
    /// the scan before it ever reached the real sensor plane elsewhere in the
    /// same file. This is the one place that scan happens now: a malformed
    /// candidate is skipped and recorded here, never allowed to abort the
    /// loop, and the record is what lets [`Container::sensor`] say *why* when
    /// nothing is found instead of returning a bare [`Error::NoSensorIfd`]
    /// indistinguishable from "this file has no raw plane at all".
    fn scan_sensor(&self) -> (Vec<usize>, Vec<(usize, u16)>) {
        let mut matches = Vec::new();
        let mut unreadable = Vec::new();
        for (index, ifd) in self.ifds.iter().enumerate() {
            match self.is_sensor_ifd(ifd) {
                SensorMatch::Yes => matches.push(index),
                SensorMatch::No => {}
                SensorMatch::Unreadable(tag) => unreadable.push((index, tag)),
            }
        }
        (matches, unreadable)
    }

    /// [`Error::NoSensorIfd`] when nothing was skipped; the more specific
    /// [`Error::NoSensorIfdCandidatesMalformed`] when at least one candidate's
    /// identity could not be determined (FU-11).
    fn no_sensor_ifd_error(unreadable: Vec<(usize, u16)>) -> Error {
        if unreadable.is_empty() {
            Error::NoSensorIfd
        } else {
            Error::NoSensorIfdCandidatesMalformed {
                candidates: unreadable,
            }
        }
    }

    /// Indices of every IFD matching [`Container::is_sensor_ifd`].
    ///
    /// Exposed so a test can assert there is exactly **one** — a selection rule
    /// that happens to pick the right IFD because it picks the first of several
    /// is not the rule doing the work. Infallible: a malformed candidate is
    /// recorded, not surfaced as an error, here (FU-11) — see
    /// [`Container::sensor`] for where that record turns into one.
    pub fn sensor_candidates(&self) -> Vec<usize> {
        self.scan_sensor().0
    }

    /// The sensor IFD, or an error explaining why none was found (FU-11).
    pub fn sensor_ifd(&self) -> Result<&Ifd, Error> {
        let (matches, unreadable) = self.scan_sensor();
        match matches.first() {
            Some(&index) => self.ifds.get(index).ok_or(Error::NoSensorIfd),
            None => Err(Self::no_sensor_ifd_error(unreadable)),
        }
    }

    /// `DEC-012`'s amendment, applied at the one place it was contradicted:
    /// an **interpretation** tag's malformed read costs that field alone, and
    /// must not be inherited by the composite accessor that invoked it.
    ///
    /// `scalar()`/`array()` (the leaf accessors) keep reporting a malformed
    /// tag honestly, as `Err` — this is what catches that `Err` for an
    /// interpretation-class call, records the tag, and continues, the same
    /// shape [`Container::scan_sensor`] already applies to `SensorMatch`
    /// per-IFD. `Ok(None)` (tag absent, or already recorded by `array()`'s own
    /// count check) passes through unchanged.
    fn cost_the_field<T>(
        result: Result<Option<T>, Error>,
        tag: u16,
        malformed: &mut Vec<u16>,
    ) -> Option<T> {
        match result {
            Ok(value) => value,
            Err(_) => {
                malformed.push(tag);
                None
            }
        }
    }

    /// The sensor plane's tags, as [`Sensor`].
    ///
    /// Reads tags only — this never touches `StripOffsets`' target, and it
    /// succeeds on compressed files too, so their tags are readable and their
    /// rejection is a separate, explicit [`Sensor::require_uncompressed`].
    ///
    /// `DEC-012`: fields below are commented **structural** (a malformed read
    /// propagates via `?` and fails the whole file — "what exists" is the
    /// plane) or **interpretation** (a malformed read costs that field alone,
    /// via [`Container::cost_the_field`], and the file still reads).
    pub fn sensor(&self) -> Result<Sensor, Error> {
        let (matches, unreadable) = self.scan_sensor();
        let ifd_index = *matches
            .first()
            .ok_or_else(|| Self::no_sensor_ifd_error(unreadable))?;
        let ifd = self.ifds.get(ifd_index).ok_or(Error::NoSensorIfd)?;

        let mut malformed: Vec<u16> = Vec::new();

        // Interpretation: Orientation. `SPEC-004` FU-16 — sensor() used to
        // read this from IFD0 with a bare `?`, so a malformed tag on a
        // NON-SENSOR IFD discarded an already-located plane. IFD0 is tried
        // first (every DNG measured carries it there); the sensor IFD is the
        // fallback when IFD0 doesn't yield a value.
        //
        // `SPEC-008/FU-1,FU-2`: the field is costed **at most once**, and
        // only when no valid value was found anywhere. The old code got two
        // cases wrong: (1) the plane IS IFD0 (the Pentax `.PEF` corpus
        // shape) — there is only ONE entry to read, not two, so a malformed
        // read must not be recorded twice; (2) IFD0's entry is malformed but
        // the sensor IFD's OWN entry is well-formed — the field was
        // recovered, not dropped, so it must not be recorded malformed at
        // all. `ifd_index == 0` is exactly "the plane is IFD0": `ifd0()` is
        // always `self.ifds[0]`, so a distinct sensor-IFD read only exists
        // when the sensor plane is some other index.
        let ifd0_read = self.ifd0().map(|ifd0| self.scalar(ifd0, TAG_ORIENTATION));
        let plane_is_ifd0 = ifd_index == 0;
        let sensor_read = if plane_is_ifd0 {
            None
        } else {
            Some(self.scalar(ifd, TAG_ORIENTATION))
        };
        let orientation = match &ifd0_read {
            Some(Ok(Some(value))) => Some(*value),
            _ => match &sensor_read {
                Some(Ok(value)) => *value,
                _ => None,
            },
        };
        if orientation.is_none()
            && (matches!(ifd0_read, Some(Err(_))) || matches!(sensor_read, Some(Err(_))))
        {
            malformed.push(TAG_ORIENTATION);
        }

        // Interpretation: BlackLevel, WhiteLevel.
        let black_level = Self::cost_the_field(
            self.scalar(ifd, TAG_BLACK_LEVEL),
            TAG_BLACK_LEVEL,
            &mut malformed,
        );
        let white_level = Self::cost_the_field(
            self.scalar(ifd, TAG_WHITE_LEVEL),
            TAG_WHITE_LEVEL,
            &mut malformed,
        );

        // Interpretation: BlackLevelRepeatDim, ActiveArea, DefaultCropOrigin,
        // DefaultCropSize. `array()` already records a wrong-length payload
        // itself; `cost_the_field` additionally catches a genuine read
        // failure (wrong field type, malformed RATIONAL shape) that `array()`
        // still reports honestly as `Err` — `SPEC-004` FU-17.
        let black_level_repeat_dim = Self::cost_the_field(
            self.array::<2>(ifd, TAG_BLACK_LEVEL_REPEAT_DIM, &mut malformed),
            TAG_BLACK_LEVEL_REPEAT_DIM,
            &mut malformed,
        );
        let active_area = Self::cost_the_field(
            self.array::<4>(ifd, TAG_ACTIVE_AREA, &mut malformed),
            TAG_ACTIVE_AREA,
            &mut malformed,
        )
        .map(|[top, left, bottom, right]| ActiveArea {
            top,
            left,
            bottom,
            right,
        });
        let default_crop_origin = Self::cost_the_field(
            self.array::<2>(ifd, TAG_DEFAULT_CROP_ORIGIN, &mut malformed),
            TAG_DEFAULT_CROP_ORIGIN,
            &mut malformed,
        )
        .map(|[x, y]| DefaultCropOrigin { x, y });
        let default_crop_size = Self::cost_the_field(
            self.array::<2>(ifd, TAG_DEFAULT_CROP_SIZE, &mut malformed),
            TAG_DEFAULT_CROP_SIZE,
            &mut malformed,
        )
        .map(|[width, height]| DefaultCropSize { width, height });

        Ok(Sensor {
            ifd_index,
            // Structural: ImageWidth, ImageLength, BitsPerSample — "what
            // exists" is the plane's dimensions.
            width: self.required_scalar(ifd, TAG_IMAGE_WIDTH)?,
            height: self.required_scalar(ifd, TAG_IMAGE_LENGTH)?,
            bits_per_sample: self.required_scalar(ifd, TAG_BITS_PER_SAMPLE)?,
            // Structural: SamplesPerPixel — part of the identifying trio.
            //
            // `SPEC-008`: no softening test is written for this read. It is
            // a RE-READ of a tag `is_sensor_ifd` already read successfully
            // to select THIS `ifd` as the candidate in the first place — a
            // mutant that tolerates a bad read here can never fire, because
            // the value it would tolerate was already read once, without
            // error, by the scan that chose this IFD. Equivalent mutant,
            // unkillable by construction; do not manufacture coverage for it.
            samples_per_pixel: self.scalar(ifd, TAG_SAMPLES_PER_PIXEL)?.unwrap_or(1),
            // Structural: Photometric — part of the identifying trio. Same
            // equivalent-mutant reasoning as `samples_per_pixel` above.
            photometric: self.required_scalar(ifd, TAG_PHOTOMETRIC)?,
            // Structural: Compression. TIFF 6.0 defaults to 1 (uncompressed)
            // when absent.
            compression: Compression::from_code(self.scalar(ifd, TAG_COMPRESSION)?.unwrap_or(1)),
            // Structural: RowsPerStrip — maps strips to rows; without it a
            // multi-strip plane cannot be assembled honestly. Inferable on a
            // single-strip file, and every corpus file is single-strip, so
            // this classification is untested by real data (`DEC-012`'s
            // amendment; do not soften this without re-reading it).
            rows_per_strip: self.scalar(ifd, TAG_ROWS_PER_STRIP)?,
            // Structural: StripOffsets, StripByteCounts.
            strip_offsets: self.values(ifd, TAG_STRIP_OFFSETS)?,
            strip_byte_counts: self.values(ifd, TAG_STRIP_BYTE_COUNTS)?,
            black_level,
            white_level,
            black_level_repeat_dim,
            active_area,
            default_crop_origin,
            default_crop_size,
            orientation,
            opcode_lists: [
                ifd.has(TAG_OPCODE_LIST_1),
                ifd.has(TAG_OPCODE_LIST_2),
                ifd.has(TAG_OPCODE_LIST_3),
            ],
            malformed_tags: malformed,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    // Test code is not a parse/decode path exercised on untrusted input
    // (AGENTS.md §11, guidance/toolchain-brief.md). Same five lints, same
    // sanctioned exception as `src/lib.rs`'s own test module. `just
    // lint-no-allow` runs `--lib`, where this module does not exist.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    /// A minimal byte-level TIFF builder, deliberately independent of
    /// `tests/support/tiff.rs`: these are the unit tests of the reader, and a
    /// shared builder would let one bug hide the other.
    struct Build {
        big: bool,
        out: Vec<u8>,
    }

    impl Build {
        fn new(big: bool, ifd0: u32) -> Build {
            let mut b = Build {
                big,
                out: Vec::new(),
            };
            b.out.extend_from_slice(if big { b"MM" } else { b"II" });
            let v = b.u16_bytes(42);
            b.out.extend_from_slice(&v);
            let o = b.u32_bytes(ifd0);
            b.out.extend_from_slice(&o);
            b
        }
        fn u16_bytes(&self, v: u16) -> [u8; 2] {
            if self.big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        }
        fn u32_bytes(&self, v: u32) -> [u8; 4] {
            if self.big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        }
        /// Write an IFD at `at`. Entries are `(tag, type, count, value word)`.
        fn ifd(mut self, at: u32, entries: &[(u16, u16, u32, u32)], next: u32) -> Build {
            let mut block = Vec::new();
            block.extend_from_slice(&self.u16_bytes(entries.len() as u16));
            for (tag, ty, count, value) in entries {
                block.extend_from_slice(&self.u16_bytes(*tag));
                block.extend_from_slice(&self.u16_bytes(*ty));
                block.extend_from_slice(&self.u32_bytes(*count));
                block.extend_from_slice(&self.u32_bytes(*value));
            }
            block.extend_from_slice(&self.u32_bytes(next));
            let at = at as usize;
            if self.out.len() < at + block.len() {
                self.out.resize(at + block.len(), 0);
            }
            self.out[at..at + block.len()].copy_from_slice(&block);
            self
        }
        fn done(self) -> Vec<u8> {
            self.out
        }
        /// A SHORT value packs at the START of the 4-byte field, so it sits in
        /// the high half on a big-endian file.
        fn short_word(&self, v: u16) -> u32 {
            if self.big {
                u32::from(v) << 16
            } else {
                u32::from(v)
            }
        }
    }

    /// The entries of a valid 4x2, 14-bit, single-sample LinearRaw plane.
    fn sensor(b: &Build) -> Vec<(u16, u16, u32, u32)> {
        vec![
            (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 0),
            (TAG_IMAGE_WIDTH, TYPE_LONG, 1, 4),
            (TAG_IMAGE_LENGTH, TYPE_LONG, 1, 2),
            (TAG_BITS_PER_SAMPLE, TYPE_SHORT, 1, b.short_word(14)),
            (TAG_COMPRESSION, TYPE_SHORT, 1, b.short_word(1)),
            (
                TAG_PHOTOMETRIC,
                TYPE_SHORT,
                1,
                b.short_word(PHOTOMETRIC_LINEAR_RAW as u16),
            ),
            (TAG_SAMPLES_PER_PIXEL, TYPE_SHORT, 1, b.short_word(1)),
            (TAG_STRIP_OFFSETS, TYPE_LONG, 1, 512),
            (TAG_STRIP_BYTE_COUNTS, TYPE_LONG, 1, 14),
        ]
    }

    // ── header ──────────────────────────────────────────────────────────────

    #[test]
    fn header_reads_both_byte_orders() {
        for big in [false, true] {
            let b = Build::new(big, 8);
            let entries = sensor(&b);
            let data = b.ifd(8, &entries, 0).done();
            let c = Container::parse(&data).unwrap();
            assert_eq!(
                c.byte_order(),
                if big {
                    ByteOrder::Big
                } else {
                    ByteOrder::Little
                }
            );
            assert_eq!(c.ifd0_offset(), 8);
            let s = c.sensor().unwrap();
            // The same plane must come out of MM and II alike; a byte-order
            // bug shows up here as 1024 instead of 4.
            assert_eq!((s.width, s.height, s.bits_per_sample), (4, 2, 14));
        }
    }

    #[test]
    fn empty_input_is_truncated_not_a_panic() {
        assert!(matches!(
            Container::parse(&[]),
            Err(Error::Truncated { .. })
        ));
    }

    #[test]
    fn a_non_tiff_mark_is_rejected() {
        assert!(matches!(
            Container::parse(b"XX\x2a\x00\x08\x00\x00\x00"),
            Err(Error::NotTiff { .. })
        ));
    }

    #[test]
    fn a_wrong_version_word_is_rejected() {
        assert!(matches!(
            Container::parse(b"II\x2b\x00\x08\x00\x00\x00"),
            Err(Error::UnsupportedTiffVersion { version: 43 })
        ));
    }

    #[test]
    fn an_ifd0_offset_of_zero_yields_no_ifds() {
        let data = Build::new(false, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(c.ifds().is_empty());
        assert!(c.ifd0().is_none());
        assert!(matches!(c.sensor(), Err(Error::NoSensorIfd)));
    }

    // ── bounds ──────────────────────────────────────────────────────────────

    #[test]
    fn an_ifd_offset_past_eof_is_truncated() {
        let data = Build::new(false, 0xFFFF_0000).done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::Truncated { .. })
        ));
    }

    #[test]
    fn an_entry_count_past_eof_is_truncated() {
        let mut data = Build::new(false, 8)
            .ifd(8, &[(TAG_IMAGE_WIDTH, TYPE_LONG, 1, 4)], 0)
            .done();
        data[8..10].copy_from_slice(&1000u16.to_le_bytes());
        assert!(matches!(
            Container::parse(&data),
            Err(Error::Truncated { .. })
        ));
    }

    #[test]
    fn a_payload_offset_past_eof_is_truncated() {
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_STRIP_OFFSETS, TYPE_LONG, 64, 0xFFFF_FF00)], 0)
            .done();
        // The walk itself succeeds — the entry table is intact.
        let c = Container::parse(&data).unwrap();
        let ifd = c.ifd0().unwrap();
        let entry = ifd.entry(TAG_STRIP_OFFSETS).unwrap();
        assert!(matches!(c.payload(entry), Err(Error::Truncated { .. })));
    }

    #[test]
    fn a_count_that_overflows_the_size_product_is_typed() {
        // u32::MAX LONGs is 16 GiB; the product must be computed in a width
        // that cannot wrap, and rejected rather than truncated.
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_STRIP_OFFSETS, TYPE_LONG, u32::MAX, 64)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        let entry = c.ifd0().unwrap().entry(TAG_STRIP_OFFSETS).unwrap();
        assert!(matches!(c.payload(entry), Err(Error::Truncated { .. })));
        assert!(matches!(c.uints(entry), Err(Error::TagTooLarge { .. })));
    }

    #[test]
    fn an_inline_value_is_read_from_the_entry_not_the_file() {
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_IMAGE_WIDTH, TYPE_LONG, 1, 0xDEAD_BEEF)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        let entry = c.ifd0().unwrap().entry(TAG_IMAGE_WIDTH).unwrap();
        assert_eq!(c.payload(entry).unwrap().len(), 4);
        assert_eq!(c.uints(entry).unwrap(), vec![0xDEAD_BEEFu32]);
    }

    #[test]
    fn an_unknown_field_type_costs_the_entry_not_the_file() {
        let data = Build::new(false, 8)
            .ifd(8, &[(60000, 250, 4, 400)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        let entry = c.ifd0().unwrap().entry(60000).unwrap();
        assert_eq!(entry.byte_len(), None);
        assert!(matches!(
            c.payload(entry),
            Err(Error::UnexpectedFieldType {
                field_type: 250,
                ..
            })
        ));
    }

    #[test]
    fn a_readable_type_this_module_does_not_widen_is_typed() {
        // SRATIONAL (10) has a known size, so `payload` works; `uints` is
        // what refuses, naming the type. `SPEC-007` reads plain RATIONAL (5)
        // — see `rational_default_crop_size_reads_or_costs_the_field` — but
        // the signed types and ASCII remain unimplemented.
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_BLACK_LEVEL, 10, 1, 200)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        let entry = c.ifd0().unwrap().entry(TAG_BLACK_LEVEL).unwrap();
        assert_eq!(entry.byte_len(), Some(8));
        assert!(matches!(
            c.uints(entry),
            Err(Error::UnexpectedFieldType { field_type: 10, .. })
        ));
    }

    // ── the guards ──────────────────────────────────────────────────────────

    #[test]
    fn a_self_referential_subifd_terminates() {
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_SUB_IFDS, TYPE_LONG, 1, 8)], 0)
            .done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::CyclicIfd { offset: 8 })
        ));
    }

    #[test]
    fn a_two_hop_subifd_cycle_terminates() {
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_SUB_IFDS, TYPE_LONG, 1, 200)], 0)
            .ifd(200, &[(TAG_SUB_IFDS, TYPE_LONG, 1, 8)], 0)
            .done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::CyclicIfd { offset: 8 })
        ));
    }

    #[test]
    fn a_chain_that_points_at_itself_terminates() {
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_IMAGE_WIDTH, TYPE_LONG, 1, 4)], 8)
            .done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::CyclicIfd { offset: 8 })
        ));
    }

    #[test]
    fn nesting_past_the_depth_limit_terminates() {
        let mut b = Build::new(false, 8);
        // One more level than MAX_IFD_DEPTH allows, each at a distinct offset
        // so the CYCLE guard cannot be what stops it — the depth guard must.
        for i in 0..=(MAX_IFD_DEPTH + 1) {
            let at = 8 + i * 40;
            b = b.ifd(at, &[(TAG_SUB_IFDS, TYPE_LONG, 1, at + 40)], 0);
        }
        let data = b.done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::IfdDepthExceeded {
                limit: MAX_IFD_DEPTH
            })
        ));
    }

    #[test]
    fn a_long_chain_stops_at_the_ifd_limit() {
        let mut b = Build::new(false, 8);
        let n = MAX_IFDS as u32 + 2;
        for i in 0..n {
            let at = 8 + i * 20;
            b = b.ifd(at, &[], at + 20);
        }
        let data = b.done();
        assert!(matches!(
            Container::parse(&data),
            Err(Error::TooManyIfds { limit: MAX_IFDS })
        ));
    }

    // ── selection ───────────────────────────────────────────────────────────

    #[test]
    fn the_sensor_is_chosen_by_rule_not_by_size() {
        // IFD0 is a 9999-wide reduced-resolution preview; the SubIFD is a
        // 4-wide LinearRaw plane. Largest-dimensions picks the preview, which
        // is the bug this rule exists to prevent.
        let b = Build::new(false, 8);
        let plane = sensor(&b);
        let data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_IMAGE_WIDTH, TYPE_LONG, 1, 9999),
                    (TAG_IMAGE_LENGTH, TYPE_LONG, 1, 9999),
                    (TAG_PHOTOMETRIC, TYPE_SHORT, 1, 6),
                    (TAG_SAMPLES_PER_PIXEL, TYPE_SHORT, 1, 3),
                    (TAG_SUB_IFDS, TYPE_LONG, 1, 200),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .done();
        let c = Container::parse(&data).unwrap();
        assert_eq!(c.sensor_candidates(), vec![1]);
        let s = c.sensor().unwrap();
        assert_eq!(s.width, 4);
        assert_eq!(s.ifd_index, 1);
    }

    #[test]
    fn an_absent_new_subfile_type_defaults_to_zero() {
        // A Pentax .PEF writes no NewSubfileType and puts the plane in IFD0.
        // TIFF 6.0's default of 0 is what finds it.
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_NEW_SUBFILE_TYPE);
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert_eq!(c.sensor().unwrap().ifd_index, 0);
    }

    #[test]
    fn a_reduced_resolution_linear_raw_ifd_is_not_the_sensor() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        for e in entries.iter_mut() {
            if e.0 == TAG_NEW_SUBFILE_TYPE {
                e.3 = 1;
            }
        }
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(c.sensor(), Err(Error::NoSensorIfd)));
    }

    #[test]
    fn candidates_malformed_names_only_candidates() {
        // SPEC-004/FU-20: `is_sensor_ifd` used to read all three identifying
        // tags before combining them, so an IFD with a READABLE, disqualifying
        // `NewSubfileType == 1` (a preview) still reported `Unreadable` if its
        // `Photometric` also happened to be malformed — even though it was
        // never a real candidate. Two IFDs: index 0 is disqualified by a
        // readable tag and must never be named; index 1 has no disqualifying
        // tag and a genuinely unreadable `Photometric`, so it must be the
        // only entry in `candidates`.
        let b = Build::new(false, 8);
        let data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_PHOTOMETRIC, 250, 1, 0),
                ],
                200,
            )
            .ifd(200, &[(TAG_PHOTOMETRIC, 250, 1, 0)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        match c.sensor() {
            Err(Error::NoSensorIfdCandidatesMalformed { candidates }) => {
                assert_eq!(candidates, vec![(1, TAG_PHOTOMETRIC)]);
            }
            other => panic!("expected NoSensorIfdCandidatesMalformed naming only the real candidate, got {other:?}"),
        }
    }

    // ── the sensor summary ──────────────────────────────────────────────────

    #[test]
    fn compression_is_reported_and_the_unsupported_ones_are_refused() {
        for (code, expect) in [
            (1u32, Compression::Uncompressed),
            (7, Compression::Jpeg),
            (65535, Compression::Other(65535)),
        ] {
            let b = Build::new(false, 8);
            let mut entries = sensor(&b);
            for e in entries.iter_mut() {
                if e.0 == TAG_COMPRESSION {
                    e.3 = code;
                }
            }
            let data = b.ifd(8, &entries, 0).done();
            let c = Container::parse(&data).unwrap();
            let s = c.sensor().unwrap();
            assert_eq!(s.compression, expect);
            assert_eq!(s.compression.code(), code);
            if code == 1 {
                assert!(s.require_uncompressed().is_ok());
            } else {
                assert!(matches!(
                    s.require_uncompressed(),
                    Err(Error::UnsupportedCompression { compression }) if compression == code
                ));
            }
        }
    }

    #[test]
    fn the_layer0_packing_arithmetic_closes() {
        let b = Build::new(false, 8);
        let entries = sensor(&b);
        let data = b.ifd(8, &entries, 0).done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        // 4 x 2 x 14 bits = 112 bits = 14 bytes == StripByteCounts.
        assert_eq!(s.packed_bits().unwrap(), 112);
        assert_eq!(s.strip_byte_counts, vec![14]);
        assert_eq!(
            s.packed_bits().unwrap(),
            u64::from(s.strip_byte_counts[0]) * 8
        );
    }

    #[test]
    fn a_missing_required_tag_names_itself() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_IMAGE_WIDTH);
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::MissingTag {
                tag: TAG_IMAGE_WIDTH
            })
        ));
    }

    #[test]
    fn malformed_tag_costs_only_that_tag() {
        // BlackLevelRepeatDim with count 1 where DNG requires 2 — the Pentax
        // K-3 III Monochrome's real defect, in miniature. DEC-012.
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        let word = b.short_word(1);
        entries.push((TAG_BLACK_LEVEL_REPEAT_DIM, TYPE_SHORT, 1, word));
        let data = b.ifd(8, &entries, 0).done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.black_level_repeat_dim, None);
        assert_eq!(s.malformed_tags, vec![TAG_BLACK_LEVEL_REPEAT_DIM]);
        // and the rest of the plane still read
        assert_eq!(s.width, 4);
    }

    #[test]
    fn malformed_interpretation_tag_costs_only_the_field() {
        // BlackLevel with a field type TIFF does not define at all — not a
        // count mismatch, the broader class `DEC-012`'s amendment and
        // `SPEC-004`/FU-17 are about. An interpretation tag's read failure of
        // ANY kind costs that field, never the file.
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_BLACK_LEVEL, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.black_level, None);
        assert_eq!(s.malformed_tags, vec![TAG_BLACK_LEVEL]);
        // and the rest of the plane still read
        assert_eq!(s.width, 4);
    }

    #[test]
    fn malformed_structural_tag_is_still_fatal() {
        // The other direction of the same boundary: RowsPerStrip stays
        // structural under `DEC-012`'s amendment, so the SAME kind of
        // malformed-field-type read that costs BlackLevel above must fail
        // the whole file here. Every corpus file is single-strip, so this
        // classification is untested by real data — this fixture is the
        // only evidence for it; do not soften it on the strength of a green
        // corpus (HANDOFF-017).
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_ROWS_PER_STRIP, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::UnexpectedFieldType {
                tag: TAG_ROWS_PER_STRIP,
                field_type: 250,
            })
        ));
    }

    // `SPEC-008`: `RowsPerStrip` was the ONLY structural tag with a
    // softening-fails test (`malformed_structural_tag_is_still_fatal` above).
    // Measured before this spec: softening `Compression`, `StripOffsets`,
    // `StripByteCounts` or `BitsPerSample` to tolerant left the full 58-test
    // suite green. `Compression` is the dangerous one — softened, it
    // defaults to 1 (uncompressed), `require_uncompressed()` passes, and
    // STAGE-002 would read JPEG bytes as raw samples. All four go through
    // `uints()` and propagate with `?` the same as `RowsPerStrip`
    // (`required_scalar()` for `BitsPerSample`, `scalar()?` for
    // `Compression`, `values()` for `StripOffsets`/`StripByteCounts`), so
    // the same fixture shape pins all four the same way.
    //
    // `SamplesPerPixel` and `Photometric` are deliberately NOT here — see the
    // comment on `Sensor::samples_per_pixel`'s construction in `sensor()`:
    // they are equivalent mutants, unkillable by construction.

    #[test]
    fn structural_compression_bad_type_is_fatal() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_COMPRESSION);
        entries.push((TAG_COMPRESSION, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::UnexpectedFieldType {
                tag: TAG_COMPRESSION,
                field_type: 250,
            })
        ));
    }

    #[test]
    fn structural_strip_offsets_bad_type_is_fatal() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_STRIP_OFFSETS);
        entries.push((TAG_STRIP_OFFSETS, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::UnexpectedFieldType {
                tag: TAG_STRIP_OFFSETS,
                field_type: 250,
            })
        ));
    }

    #[test]
    fn structural_strip_byte_counts_bad_type_is_fatal() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_STRIP_BYTE_COUNTS);
        entries.push((TAG_STRIP_BYTE_COUNTS, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::UnexpectedFieldType {
                tag: TAG_STRIP_BYTE_COUNTS,
                field_type: 250,
            })
        ));
    }

    #[test]
    fn structural_bits_per_sample_bad_type_is_fatal() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.retain(|(tag, ..)| *tag != TAG_BITS_PER_SAMPLE);
        entries.push((TAG_BITS_PER_SAMPLE, 250, 1, 0));
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert!(matches!(
            c.sensor(),
            Err(Error::UnexpectedFieldType {
                tag: TAG_BITS_PER_SAMPLE,
                field_type: 250,
            })
        ));
    }

    #[test]
    fn a_well_formed_fixed_length_tag_is_reported() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_ACTIVE_AREA, TYPE_LONG, 4, 600));
        let mut data = b.ifd(8, &entries, 0).done();
        data.resize(616, 0);
        for (i, v) in [0u32, 0, 2, 4].iter().enumerate() {
            let at = 600 + i * 4;
            data[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(
            s.active_area,
            Some(ActiveArea {
                top: 0,
                left: 0,
                bottom: 2,
                right: 4
            })
        );
        assert!(s.malformed_tags.is_empty());
    }

    /// Write `values` as consecutive RATIONAL (numerator, denominator) pairs
    /// starting at byte offset `at`, resizing `data` to fit.
    fn write_rationals(data: &mut Vec<u8>, at: usize, values: &[(u32, u32)]) {
        let end = at + values.len() * 8;
        if data.len() < end {
            data.resize(end, 0);
        }
        for (i, (num, den)) in values.iter().enumerate() {
            let pair_at = at + i * 8;
            data[pair_at..pair_at + 4].copy_from_slice(&num.to_le_bytes());
            data[pair_at + 4..pair_at + 8].copy_from_slice(&den.to_le_bytes());
        }
    }

    #[test]
    fn rational_default_crop_size_reads_or_costs_the_field() {
        // SPEC-004/FU-17: DNG permits DefaultCropSize as RATIONAL, and
        // TYPE_RATIONAL was not read at all before SPEC-007. Both directions
        // of the boundary, one fixture family: an exact integral RATIONAL
        // reads correctly; a zero denominator costs the field alone — the
        // file still reads.
        let good = {
            let b = Build::new(false, 8);
            let mut entries = sensor(&b);
            entries.push((TAG_DEFAULT_CROP_SIZE, TYPE_RATIONAL, 2, 600));
            let mut data = b.ifd(8, &entries, 0).done();
            // The Q2 Monochrom's real crop size (docs/measured-q2m-dng.md),
            // read as RATIONAL rather than the LONG every corpus file uses.
            write_rationals(&mut data, 600, &[(8368, 1), (5584, 1)]);
            data
        };
        let s = Container::parse(&good).unwrap().sensor().unwrap();
        assert_eq!(
            s.default_crop_size,
            Some(DefaultCropSize {
                width: 8368,
                height: 5584
            })
        );
        assert!(s.malformed_tags.is_empty());

        let malformed = {
            let b = Build::new(false, 8);
            let mut entries = sensor(&b);
            entries.push((TAG_DEFAULT_CROP_SIZE, TYPE_RATIONAL, 2, 600));
            let mut data = b.ifd(8, &entries, 0).done();
            // Zero denominator — a malformed SHAPE, not a missing structure.
            write_rationals(&mut data, 600, &[(5, 0), (5, 1)]);
            data
        };
        let s = Container::parse(&malformed).unwrap().sensor().unwrap();
        assert_eq!(s.default_crop_size, None);
        assert_eq!(s.malformed_tags, vec![TAG_DEFAULT_CROP_SIZE]);
        // and the rest of the plane still read
        assert_eq!(s.width, 4);
    }

    #[test]
    fn a_non_integral_rational_also_costs_the_field() {
        // The other malformed shape criterion 3 names: denominator nonzero
        // but the ratio is not a whole number (5/2, not 2 or 3).
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_DEFAULT_CROP_SIZE, TYPE_RATIONAL, 2, 600));
        let mut data = b.ifd(8, &entries, 0).done();
        write_rationals(&mut data, 600, &[(5, 2), (5, 1)]);
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.default_crop_size, None);
        assert_eq!(s.malformed_tags, vec![TAG_DEFAULT_CROP_SIZE]);
    }

    #[test]
    fn rational_denominator_is_actually_divided() {
        // `SPEC-008`/FU-5: every well-formed RATIONAL fixture before this one
        // used denominator 1 (see `rational_default_crop_size_reads_or_costs_the_field`
        // above: `(8368, 1), (5584, 1)`), so a mutant that pushed the
        // numerator and never divided would still pass all 58 tests.
        // 16736/2 = 8368 pins the division: if a mutant returned the
        // numerator unchanged, this asserts 16736, not 8368, and fails.
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_DEFAULT_CROP_SIZE, TYPE_RATIONAL, 2, 600));
        let mut data = b.ifd(8, &entries, 0).done();
        write_rationals(&mut data, 600, &[(16736, 2), (5584, 1)]);
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(
            s.default_crop_size,
            Some(DefaultCropSize {
                width: 8368,
                height: 5584
            })
        );
        assert!(s.malformed_tags.is_empty());
    }

    #[test]
    fn subifds_rational_is_rejected() {
        // `SPEC-007`/FU-4: `uints()`'s RATIONAL widening was GLOBAL, so a
        // `SubIFDs` entry encoded as RATIONAL 400/2 (= 200, a valid-looking —
        // and here, actually correct — offset) walked the SubIFD instead of
        // failing the container, where `main` returned
        // `Err(UnexpectedFieldType)`. `DEC-012` names `SubIFDs` structural;
        // the acceptance must be per-tag, not global — `is_structural_tag`.
        let b = Build::new(false, 8);
        let plane = sensor(&b);
        let mut data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_SUB_IFDS, TYPE_RATIONAL, 1, 600),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .done();
        write_rationals(&mut data, 600, &[(400, 2)]);
        assert!(matches!(
            Container::parse(&data),
            Err(Error::UnexpectedFieldType {
                tag: TAG_SUB_IFDS,
                field_type: TYPE_RATIONAL,
            })
        ));
    }

    #[test]
    fn opcode_list_presence_is_reported_without_decoding_them() {
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_OPCODE_LIST_1, TYPE_UNDEFINED, 4, 0));
        entries.push((TAG_OPCODE_LIST_3, TYPE_UNDEFINED, 4, 0));
        let data = b.ifd(8, &entries, 0).done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.opcode_lists, [true, false, true]);
    }

    #[test]
    fn orientation_comes_from_ifd0_when_the_plane_is_a_subifd() {
        let b = Build::new(false, 8);
        let plane = sensor(&b);
        let word = b.short_word(6);
        let data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_ORIENTATION, TYPE_SHORT, 1, word),
                    (TAG_SUB_IFDS, TYPE_LONG, 1, 200),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.orientation, Some(6));
    }

    #[test]
    fn malformed_orientation_on_ifd0_keeps_the_plane() {
        // SPEC-004/FU-16: sensor() used to read Orientation from IFD0 with a
        // bare `?`, so a malformed tag on a NON-SENSOR IFD discarded an
        // already-located plane. DEC-012: Orientation is interpretation-class
        // — malformed costs the field, never the plane.
        let b = Build::new(false, 8);
        let plane = sensor(&b);
        let data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_ORIENTATION, 250, 1, 0), // unreadable field type
                    (TAG_SUB_IFDS, TYPE_LONG, 1, 200),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .done();
        let c = Container::parse(&data).unwrap();
        assert_eq!(c.sensor_candidates(), vec![1]);
        let s = c.sensor().unwrap();
        assert_eq!(s.orientation, None);
        assert_eq!(s.malformed_tags, vec![TAG_ORIENTATION]);
        // and the plane is still located and read
        assert_eq!(s.width, 4);
    }

    #[test]
    fn orientation_costed_once_when_plane_is_ifd0() {
        // `SPEC-008`/FU-1: the Pentax `.PEF` corpus shape — the plane IS
        // IFD0, so there is only ONE `Orientation` entry to read, not two
        // (an "IFD0 first" try followed by a "sensor IFD" fallback that
        // happens to be the SAME entry). Before this fix the fallback
        // re-read the identical malformed entry and pushed `TAG_ORIENTATION`
        // a second time: `malformed_tags == [274, 274]`.
        let b = Build::new(false, 8);
        let mut entries = sensor(&b);
        entries.push((TAG_ORIENTATION, 250, 1, 0)); // unreadable field type
        let data = b.ifd(8, &entries, 0).done();
        let c = Container::parse(&data).unwrap();
        assert_eq!(c.sensor_candidates(), vec![0]);
        let s = c.sensor().unwrap();
        assert_eq!(s.orientation, None);
        assert_eq!(s.malformed_tags, vec![TAG_ORIENTATION]);
        // and the plane is still located and read
        assert_eq!(s.width, 4);
    }

    #[test]
    fn wellformed_orientation_is_not_recorded_malformed() {
        // `SPEC-008`/FU-2: IFD0's `Orientation` is malformed, but the sensor
        // IFD (a DIFFERENT ifd — a SubIFD, unlike the fixture above) carries
        // its OWN well-formed `Orientation`. The field is recovered, not
        // dropped: `orientation` must read `Some(6)` and `malformed_tags`
        // must NOT carry `TAG_ORIENTATION` — before this fix it did, even
        // though a good value was found and used.
        let b = Build::new(false, 8);
        let mut plane = sensor(&b);
        let word = b.short_word(6);
        plane.push((TAG_ORIENTATION, TYPE_SHORT, 1, word));
        let data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_ORIENTATION, 250, 1, 0), // malformed, on IFD0
                    (TAG_SUB_IFDS, TYPE_LONG, 1, 200),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .done();
        let s = Container::parse(&data).unwrap().sensor().unwrap();
        assert_eq!(s.orientation, Some(6));
        assert!(s.malformed_tags.is_empty());
    }

    #[test]
    fn subifd_offsets_are_walked_in_order_and_recorded_with_their_parent() {
        let b = Build::new(false, 8);
        let plane = sensor(&b);
        let mut data = b
            .ifd(
                8,
                &[
                    (TAG_NEW_SUBFILE_TYPE, TYPE_LONG, 1, 1),
                    (TAG_SUB_IFDS, TYPE_LONG, 2, 600),
                ],
                0,
            )
            .ifd(200, &plane, 0)
            .ifd(400, &[(TAG_IMAGE_WIDTH, TYPE_LONG, 1, 7)], 0)
            .done();
        data.resize(608, 0);
        data[600..604].copy_from_slice(&200u32.to_le_bytes());
        data[604..608].copy_from_slice(&400u32.to_le_bytes());
        let c = Container::parse(&data).unwrap();
        assert_eq!(c.ifds().len(), 3);
        assert_eq!(c.ifds()[1].offset(), 200);
        assert_eq!(c.ifds()[1].depth(), 1);
        assert_eq!(c.ifds()[1].parent(), Some(0));
        assert_eq!(c.ifds()[2].offset(), 400);
    }

    #[test]
    fn every_error_variant_this_module_returns_displays_something() {
        // Display is the only surface a consumer has for these; an empty one
        // is a silent error (constraint: errors carry their context).
        let errors = [
            Error::Truncated { at: 1, len: 2 },
            Error::NotTiff { mark: *b"XX" },
            Error::UnsupportedTiffVersion { version: 43 },
            Error::CyclicIfd { offset: 8 },
            Error::IfdDepthExceeded { limit: 8 },
            Error::TooManyIfds { limit: 512 },
            Error::UnexpectedFieldType {
                tag: 1,
                field_type: 250,
            },
            Error::ValueOverflow { tag: 1 },
            Error::MalformedRationalValue {
                tag: 1,
                numerator: 5,
                denominator: 0,
            },
            Error::TagTooLarge {
                tag: 1,
                count: 2,
                limit: 3,
            },
            Error::MissingTag { tag: 256 },
            Error::OffsetOutOfRange { offset: 9 },
            Error::NoSensorIfd,
            Error::NoSensorIfdCandidatesMalformed {
                candidates: vec![(0, TAG_PHOTOMETRIC)],
            },
            Error::UnsupportedCompression { compression: 7 },
        ];
        for e in &errors {
            assert!(!e.to_string().is_empty(), "{e:?} has an empty Display");
        }
    }
}
