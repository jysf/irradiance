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
// Only the tags the container walk and SPEC-003's acceptance criteria require.
// The full typed tag model is SPEC-004's job, not this module's.

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
    /// DNG `ActiveArea` (top, left, bottom, right), when present.
    pub active_area: Option<[u32; 4]>,
    /// DNG `DefaultCropOrigin`, when present.
    pub default_crop_origin: Option<[u32; 2]>,
    /// DNG `DefaultCropSize`, when present.
    pub default_crop_size: Option<[u32; 2]>,
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
    /// Handles `BYTE`, `SHORT`, `LONG`, `IFD` and `UNDEFINED`. The wider type
    /// model — `RATIONAL`, the signed types, `ASCII` — is `SPEC-004`'s, and
    /// asking for one here is a typed [`Error::UnexpectedFieldType`] rather
    /// than a silent zero.
    pub fn uints(&self, entry: &Entry) -> Result<Vec<u32>, Error> {
        // The field type is a property of the entry, so it is checked before
        // the buffer is touched: otherwise a RATIONAL that also happens to
        // point out of bounds reports `Truncated` and hides the real problem.
        match entry.field_type {
            TYPE_BYTE | TYPE_UNDEFINED | TYPE_SHORT | TYPE_LONG | TYPE_IFD => {}
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
    pub fn is_sensor_ifd(&self, ifd: &Ifd) -> Result<bool, Error> {
        let subfile_type = self.scalar(ifd, TAG_NEW_SUBFILE_TYPE)?.unwrap_or(0);
        let photometric = self.scalar(ifd, TAG_PHOTOMETRIC)?;
        let samples = self.scalar(ifd, TAG_SAMPLES_PER_PIXEL)?.unwrap_or(1);
        Ok(subfile_type == 0 && photometric == Some(PHOTOMETRIC_LINEAR_RAW) && samples == 1)
    }

    /// Indices of every IFD matching [`Container::is_sensor_ifd`].
    ///
    /// Exposed so a test can assert there is exactly **one** — a selection rule
    /// that happens to pick the right IFD because it picks the first of several
    /// is not the rule doing the work.
    pub fn sensor_candidates(&self) -> Result<Vec<usize>, Error> {
        let mut out = Vec::new();
        for (index, ifd) in self.ifds.iter().enumerate() {
            if self.is_sensor_ifd(ifd)? {
                out.push(index);
            }
        }
        Ok(out)
    }

    /// The sensor IFD, or [`Error::NoSensorIfd`].
    pub fn sensor_ifd(&self) -> Result<&Ifd, Error> {
        for ifd in &self.ifds {
            if self.is_sensor_ifd(ifd)? {
                return Ok(ifd);
            }
        }
        Err(Error::NoSensorIfd)
    }

    /// The sensor plane's tags, as [`Sensor`].
    ///
    /// Reads tags only — this never touches `StripOffsets`' target, and it
    /// succeeds on compressed files too, so their tags are readable and their
    /// rejection is a separate, explicit [`Sensor::require_uncompressed`].
    pub fn sensor(&self) -> Result<Sensor, Error> {
        let mut index = None;
        for (i, ifd) in self.ifds.iter().enumerate() {
            if self.is_sensor_ifd(ifd)? {
                index = Some(i);
                break;
            }
        }
        let ifd_index = index.ok_or(Error::NoSensorIfd)?;
        let ifd = self.ifds.get(ifd_index).ok_or(Error::NoSensorIfd)?;

        let mut malformed: Vec<u16> = Vec::new();

        // Orientation is an IFD0 tag in every DNG measured; the Pentax PEF
        // puts the plane in IFD0 itself, so the fallback finds it there.
        let orientation = match self.ifd0() {
            Some(ifd0) => match self.scalar(ifd0, TAG_ORIENTATION)? {
                Some(value) => Some(value),
                None => self.scalar(ifd, TAG_ORIENTATION)?,
            },
            None => self.scalar(ifd, TAG_ORIENTATION)?,
        };

        Ok(Sensor {
            ifd_index,
            width: self.required_scalar(ifd, TAG_IMAGE_WIDTH)?,
            height: self.required_scalar(ifd, TAG_IMAGE_LENGTH)?,
            bits_per_sample: self.required_scalar(ifd, TAG_BITS_PER_SAMPLE)?,
            samples_per_pixel: self.scalar(ifd, TAG_SAMPLES_PER_PIXEL)?.unwrap_or(1),
            photometric: self.required_scalar(ifd, TAG_PHOTOMETRIC)?,
            // TIFF 6.0: Compression defaults to 1 (uncompressed) when absent.
            compression: Compression::from_code(self.scalar(ifd, TAG_COMPRESSION)?.unwrap_or(1)),
            rows_per_strip: self.scalar(ifd, TAG_ROWS_PER_STRIP)?,
            strip_offsets: self.values(ifd, TAG_STRIP_OFFSETS)?,
            strip_byte_counts: self.values(ifd, TAG_STRIP_BYTE_COUNTS)?,
            black_level: self.scalar(ifd, TAG_BLACK_LEVEL)?,
            white_level: self.scalar(ifd, TAG_WHITE_LEVEL)?,
            black_level_repeat_dim: self.array::<2>(
                ifd,
                TAG_BLACK_LEVEL_REPEAT_DIM,
                &mut malformed,
            )?,
            active_area: self.array::<4>(ifd, TAG_ACTIVE_AREA, &mut malformed)?,
            default_crop_origin: self.array::<2>(ifd, TAG_DEFAULT_CROP_ORIGIN, &mut malformed)?,
            default_crop_size: self.array::<2>(ifd, TAG_DEFAULT_CROP_SIZE, &mut malformed)?,
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
        // RATIONAL has a known size, so `payload` works; `uints` is what
        // refuses, naming the type. The wider model is SPEC-004's.
        let data = Build::new(false, 8)
            .ifd(8, &[(TAG_BLACK_LEVEL, 5, 1, 200)], 0)
            .done();
        let c = Container::parse(&data).unwrap();
        let entry = c.ifd0().unwrap().entry(TAG_BLACK_LEVEL).unwrap();
        assert_eq!(entry.byte_len(), Some(8));
        assert!(matches!(
            c.uints(entry),
            Err(Error::UnexpectedFieldType { field_type: 5, .. })
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
        assert_eq!(c.sensor_candidates().unwrap(), vec![1]);
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
    fn a_malformed_fixed_length_tag_costs_the_tag_not_the_file() {
        // BlackLevelRepeatDim with count 1 where DNG requires 2 — the Pentax
        // K-3 III Monochrome's real defect, in miniature.
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
        assert_eq!(s.active_area, Some([0, 0, 2, 4]));
        assert!(s.malformed_tags.is_empty());
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
            Error::TagTooLarge {
                tag: 1,
                count: 2,
                limit: 3,
            },
            Error::MissingTag { tag: 256 },
            Error::OffsetOutOfRange { offset: 9 },
            Error::NoSensorIfd,
            Error::UnsupportedCompression { compression: 7 },
        ];
        for e in &errors {
            assert!(!e.to_string().is_empty(), "{e:?} has an empty Display");
        }
    }
}
