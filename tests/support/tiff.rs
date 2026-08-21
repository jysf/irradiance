//! Hand-built TIFF byte fixtures — the hostile-input lane, and the fuzz seeds.
//!
//! `SPEC-003`. Every buffer here is **own work, built byte by byte from TIFF
//! 6.0 §2**, so it is tier A: committable, licence-clean, and small enough to
//! read. No corpus file is copied, truncated or derived from — tier B is never
//! committed (`DEC-003`), and a truncated 86 MB Leica frame would still be a
//! Leica frame.
//!
//! Compiled into two crates via `#[path]` — `tests/ifd_reader.rs` (which
//! asserts the reader rejects each of these with a typed error) and
//! `examples/fuzz-seeds.rs` (which writes them into `fuzz/corpus/ifd/`). Each
//! uses a different subset, and neither can see the other's use, so
//! `dead_code` is allowed here for the same reason it is in
//! `tests/support/corpus.rs`. This is NOT the `#[allow]` bypass `SPEC-006`
//! closes: that is about the five panic-free lints escaping `src/lib.rs`'s
//! crate-root policy, and this file is never compiled into the library.
#![allow(dead_code)]

/// Byte order of a fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Little,
    Big,
}

impl Order {
    fn u16(self, v: u16) -> [u8; 2] {
        match self {
            Order::Little => v.to_le_bytes(),
            Order::Big => v.to_be_bytes(),
        }
    }
    fn u32(self, v: u32) -> [u8; 4] {
        match self {
            Order::Little => v.to_le_bytes(),
            Order::Big => v.to_be_bytes(),
        }
    }
    fn mark(self) -> [u8; 2] {
        match self {
            Order::Little => *b"II",
            Order::Big => *b"MM",
        }
    }
}

/// One 12-byte IFD entry, with its value/offset word kept raw so a fixture can
/// put a deliberately absurd offset there.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub tag: u16,
    pub field_type: u16,
    pub count: u32,
    pub value: u32,
}

/// A SHORT entry holding one value inline.
///
/// TIFF packs inline values at the START of the 4-byte field, which means a
/// single SHORT sits in the low half little-endian and the HIGH half
/// big-endian. Getting this wrong makes every big-endian fixture silently read
/// as zero, so it is done once, here.
pub fn short(tag: u16, value: u16, order: Order) -> Entry {
    let raw = match order {
        Order::Little => u32::from(value),
        Order::Big => u32::from(value) << 16,
    };
    Entry {
        tag,
        field_type: 3,
        count: 1,
        value: raw,
    }
}

/// A LONG entry holding one value inline.
pub fn long(tag: u16, value: u32) -> Entry {
    Entry {
        tag,
        field_type: 4,
        count: 1,
        value,
    }
}

/// An entry whose value word is an OFFSET — `count` values of `field_type`
/// living at `offset`. The fixture chooses both, including impossible ones.
pub fn at_offset(tag: u16, field_type: u16, count: u32, offset: u32) -> Entry {
    Entry {
        tag,
        field_type,
        count,
        value: offset,
    }
}

/// One IFD placed at an explicit offset.
#[derive(Debug, Clone)]
pub struct Ifd {
    pub offset: u32,
    pub entries: Vec<Entry>,
    pub next: u32,
}

impl Ifd {
    pub fn new(offset: u32, entries: Vec<Entry>, next: u32) -> Ifd {
        Ifd {
            offset,
            entries,
            next,
        }
    }
}

/// Assemble a TIFF: an 8-byte header, then each IFD written at its own offset.
///
/// The buffer grows to fit whatever offsets the fixture chose, so an IFD placed
/// beyond the data is simply a hole — which is the point for the fixtures that
/// want one.
pub fn tiff(order: Order, ifd0_offset: u32, ifds: &[Ifd]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(&order.mark());
    out.extend_from_slice(&order.u16(42));
    out.extend_from_slice(&order.u32(ifd0_offset));

    for ifd in ifds {
        let at = ifd.offset as usize;
        let mut block: Vec<u8> = Vec::new();
        let count = u16::try_from(ifd.entries.len()).unwrap_or(u16::MAX);
        block.extend_from_slice(&order.u16(count));
        for e in &ifd.entries {
            block.extend_from_slice(&order.u16(e.tag));
            block.extend_from_slice(&order.u16(e.field_type));
            block.extend_from_slice(&order.u32(e.count));
            block.extend_from_slice(&order.u32(e.value));
        }
        block.extend_from_slice(&order.u32(ifd.next));

        if out.len() < at {
            out.resize(at, 0);
        }
        let end = at + block.len();
        if out.len() < end {
            out.resize(end, 0);
        }
        out[at..end].copy_from_slice(&block);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tags, so the fixtures read like the spec they came from
// ─────────────────────────────────────────────────────────────────────────────

pub const NEW_SUBFILE_TYPE: u16 = 254;
pub const IMAGE_WIDTH: u16 = 256;
pub const IMAGE_LENGTH: u16 = 257;
pub const BITS_PER_SAMPLE: u16 = 258;
pub const COMPRESSION: u16 = 259;
pub const PHOTOMETRIC: u16 = 262;
pub const STRIP_OFFSETS: u16 = 273;
pub const SAMPLES_PER_PIXEL: u16 = 277;
pub const STRIP_BYTE_COUNTS: u16 = 279;
pub const SUB_IFDS: u16 = 330;
pub const BLACK_LEVEL_REPEAT_DIM: u16 = 50713;
pub const LINEAR_RAW: u16 = 34892;

/// The entry list of a minimal but *valid* sensor IFD: 4 x 2, 14-bit, one
/// sample, uncompressed, LinearRaw. `4 * 2 * 14 = 112` bits = 14 bytes, so the
/// layer-0 arithmetic closes against `StripByteCounts` too.
pub fn sensor_entries(order: Order) -> Vec<Entry> {
    vec![
        long(NEW_SUBFILE_TYPE, 0),
        long(IMAGE_WIDTH, 4),
        long(IMAGE_LENGTH, 2),
        short(BITS_PER_SAMPLE, 14, order),
        short(COMPRESSION, 1, order),
        short(PHOTOMETRIC, LINEAR_RAW, order),
        short(SAMPLES_PER_PIXEL, 1, order),
        long(STRIP_OFFSETS, 512),
        long(STRIP_BYTE_COUNTS, 14),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// The named fixtures
// ─────────────────────────────────────────────────────────────────────────────
//
// Each is one hostile shape, named for the guard it exercises. `hostile()`
// returns all of them, so adding one adds it to the test AND to the fuzz seeds
// with no second edit.

/// A valid container whose IFD0 *is* the sensor plane. The good seed: a fuzzer
/// with only malformed inputs never gets past the header.
pub fn valid_sensor(order: Order) -> Vec<u8> {
    tiff(order, 8, &[Ifd::new(8, sensor_entries(order), 0)])
}

/// A valid container with the plane one `SubIFD` down, as every DNG has it.
pub fn valid_subifd(order: Order) -> Vec<u8> {
    tiff(
        order,
        8,
        &[
            Ifd::new(8, vec![long(NEW_SUBFILE_TYPE, 1), long(SUB_IFDS, 200)], 0),
            Ifd::new(200, sensor_entries(order), 0),
        ],
    )
}

/// Four bytes: not even a header.
pub fn truncated_header() -> Vec<u8> {
    vec![b'I', b'I', 42, 0]
}

/// A header cut off mid-IFD0-offset.
pub fn truncated_ifd0_offset() -> Vec<u8> {
    vec![b'I', b'I', 42, 0, 8, 0]
}

/// Byte-order mark that is neither `II` nor `MM`.
pub fn bad_magic() -> Vec<u8> {
    let mut v = valid_sensor(Order::Little);
    if let Some(slot) = v.get_mut(0..2) {
        slot.copy_from_slice(b"XX");
    }
    v
}

/// Version word 43 instead of 42.
pub fn bad_version() -> Vec<u8> {
    let mut v = valid_sensor(Order::Little);
    if let Some(slot) = v.get_mut(2..4) {
        slot.copy_from_slice(&43u16.to_le_bytes());
    }
    v
}

/// IFD0 offset points past the end of the file.
pub fn ifd0_past_eof() -> Vec<u8> {
    tiff(Order::Little, 0xFFFF_0000, &[])
}

/// IFD0 offset lands inside the header — legal to *read*, and it decodes to a
/// nonsense entry count. The bounds checks, not a sanity rule, are what must
/// hold.
pub fn ifd0_inside_header() -> Vec<u8> {
    tiff(Order::Little, 2, &[])
}

/// The IFD declares 1000 entries; the file holds room for none of them.
pub fn entry_count_past_eof() -> Vec<u8> {
    let mut v = tiff(
        Order::Little,
        8,
        &[Ifd::new(8, vec![long(IMAGE_WIDTH, 4)], 0)],
    );
    if let Some(slot) = v.get_mut(8..10) {
        slot.copy_from_slice(&1000u16.to_le_bytes());
    }
    v
}

/// An entry whose payload offset is near `u32::MAX`.
pub fn payload_offset_past_eof() -> Vec<u8> {
    tiff(
        Order::Little,
        8,
        &[Ifd::new(
            8,
            vec![at_offset(STRIP_OFFSETS, 4, 64, 0xFFFF_FF00)],
            0,
        )],
    )
}

/// An entry declaring `u32::MAX` LONGs: `count * 4` overflows 32 bits, and the
/// product must be computed in a width that cannot wrap.
pub fn count_overflow() -> Vec<u8> {
    tiff(
        Order::Little,
        8,
        &[Ifd::new(
            8,
            vec![at_offset(STRIP_OFFSETS, 4, u32::MAX, 64)],
            0,
        )],
    )
}

/// `SubIFDs` points at the IFD that contains it. The self-referential case
/// named in `SPEC-003` acceptance criterion 3.
pub fn self_referential_subifd() -> Vec<u8> {
    tiff(Order::Little, 8, &[Ifd::new(8, vec![long(SUB_IFDS, 8)], 0)])
}

/// A two-IFD cycle: A's `SubIFDs` points at B, B's points back at A.
pub fn subifd_cycle_two() -> Vec<u8> {
    tiff(
        Order::Little,
        8,
        &[
            Ifd::new(8, vec![long(SUB_IFDS, 200)], 0),
            Ifd::new(200, vec![long(SUB_IFDS, 8)], 0),
        ],
    )
}

/// A chain cycle rather than a SubIFD one: IFD0's `next` points at IFD0.
pub fn chain_cycle() -> Vec<u8> {
    tiff(
        Order::Little,
        8,
        &[Ifd::new(8, vec![long(IMAGE_WIDTH, 4)], 8)],
    )
}

/// A cycle that closes only after several distinct hops — a visited-set guard
/// catches it, a "did I just come from here" guard does not.
pub fn long_cycle() -> Vec<u8> {
    let mut ifds = Vec::new();
    for i in 0..6u32 {
        let at = 8 + i * 40;
        let next = if i == 5 { 8 } else { at + 40 };
        ifds.push(Ifd::new(at, vec![long(IMAGE_WIDTH, i)], next));
    }
    tiff(Order::Little, 8, &ifds)
}

/// Twelve levels of `SubIFD` nesting, past the depth limit.
pub fn deep_subifd_nest() -> Vec<u8> {
    let mut ifds = Vec::new();
    for i in 0..12u32 {
        let at = 8 + i * 40;
        let child = at + 40;
        ifds.push(Ifd::new(at, vec![long(SUB_IFDS, child)], 0));
    }
    tiff(Order::Little, 8, &ifds)
}

/// An entry with a field type TIFF does not define. The file stays readable;
/// only that entry's payload does not.
pub fn unknown_field_type() -> Vec<u8> {
    let mut entries = sensor_entries(Order::Little);
    entries.push(at_offset(60000, 250, 4, 400));
    tiff(Order::Little, 8, &[Ifd::new(8, entries, 0)])
}

/// `BlackLevelRepeatDim` with one value where DNG requires two — the shape a
/// shipping Pentax K-3 III Monochrome actually writes, and `dnglab` warns
/// about. Must cost the tag, not the file.
pub fn malformed_black_level_repeat_dim() -> Vec<u8> {
    let mut entries = sensor_entries(Order::Little);
    entries.push(short(BLACK_LEVEL_REPEAT_DIM, 1, Order::Little));
    tiff(Order::Little, 8, &[Ifd::new(8, entries, 0)])
}

/// An IFD with zero entries, chaining to nothing.
pub fn zero_entries() -> Vec<u8> {
    tiff(Order::Little, 8, &[Ifd::new(8, vec![], 0)])
}

/// A container with no IFD at all — IFD0 offset 0.
pub fn no_ifd0() -> Vec<u8> {
    tiff(Order::Little, 0, &[])
}

/// The empty input.
pub fn empty() -> Vec<u8> {
    Vec::new()
}

/// Every fixture, with the name it is stored and reported under.
///
/// One list, consumed by both the hostile-input test and the fuzz-seed writer,
/// so a new fixture cannot land in one and be forgotten by the other.
pub fn all() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("valid-sensor-ii", valid_sensor(Order::Little)),
        ("valid-sensor-mm", valid_sensor(Order::Big)),
        ("valid-subifd-ii", valid_subifd(Order::Little)),
        ("valid-subifd-mm", valid_subifd(Order::Big)),
        ("empty", empty()),
        ("truncated-header", truncated_header()),
        ("truncated-ifd0-offset", truncated_ifd0_offset()),
        ("bad-magic", bad_magic()),
        ("bad-version", bad_version()),
        ("ifd0-past-eof", ifd0_past_eof()),
        ("ifd0-inside-header", ifd0_inside_header()),
        ("entry-count-past-eof", entry_count_past_eof()),
        ("payload-offset-past-eof", payload_offset_past_eof()),
        ("count-overflow", count_overflow()),
        ("self-referential-subifd", self_referential_subifd()),
        ("subifd-cycle-two", subifd_cycle_two()),
        ("chain-cycle", chain_cycle()),
        ("long-cycle", long_cycle()),
        ("deep-subifd-nest", deep_subifd_nest()),
        ("unknown-field-type", unknown_field_type()),
        (
            "malformed-black-level-repeat-dim",
            malformed_black_level_repeat_dim(),
        ),
        ("zero-entries", zero_entries()),
        ("no-ifd0", no_ifd0()),
    ]
}
