//! A declarative, test-only DNG writer — `SPEC-057`, `DEC-055`.
//!
//! Describe a plane and its metadata ([`DngDescription`]); [`write_dng`] lays
//! out a standards-shaped DNG. It produces mono (`LinearRaw`) and CFA planes at
//! 8/12/14/16 bits, in strips or tiles, in either byte order, with the sensor in
//! IFD0 or in a SubIFD under a tiny preview IFD0, carrying the metadata
//! irradiance's reader models plus arbitrary opcode-list and raw-tag payloads.
//!
//! **Clean-room provenance** (`docs/provenance-ledger.md`): written from TIFF
//! 6.0 §2 (header, IFD, field types, the start-of-field inline rule), §8
//! (Compression 1) and §15 (tiles); the Adobe DNG specification 1.7 (tag
//! placement, `DNGVersion`); and `DEC-008`'s packing rule, applied in the
//! writing direction. No writer implementation was read — not dnglab/rawler,
//! not the AGPL `dng` crate, not the DNG SDK, not LibTIFF.
//!
//! **Not the hostile lane.** `tests/support/tiff.rs` stays the byte-placed
//! builder for aliased IFDs and absurd offsets. This writer is for *valid*
//! fixtures; its one hostility hatch is [`DngDescription::extra_tags`], which
//! replaces or appends raw entries without a second API.
//!
//! **No encoder.** [`Payload::Samples`] is packed as Compression 1;
//! [`Payload::Encoded`] writes caller-supplied chunks verbatim (e.g. from
//! `tiff.rs`'s test `lj92_strip`). Promotion into `src/` behind the `write`
//! feature (`SPEC-061`) therefore needs no codec decision, and the core is
//! already `Result`-returning with checked size arithmetic so the promotion is
//! a move, not a rewrite.
//!
//! Mounting: the including test crate must also mount `tests/support/tiff.rs`
//! as `mod tiff`, whose [`Order`] this reuses.
#![allow(dead_code)]

use crate::tiff::Order;
use irradiance::ifd::{
    TAG_ACTIVE_AREA, TAG_AS_SHOT_NEUTRAL, TAG_BITS_PER_SAMPLE, TAG_BLACK_LEVEL,
    TAG_BLACK_LEVEL_REPEAT_DIM, TAG_CALIBRATION_ILLUMINANT_1, TAG_CALIBRATION_ILLUMINANT_2,
    TAG_CFA_LAYOUT, TAG_CFA_PATTERN, TAG_CFA_PLANE_COLOR, TAG_CFA_REPEAT_PATTERN_DIM,
    TAG_COLOR_MATRIX_1, TAG_COLOR_MATRIX_2, TAG_COMPRESSION, TAG_DEFAULT_CROP_ORIGIN,
    TAG_DEFAULT_CROP_SIZE, TAG_FORWARD_MATRIX_1, TAG_FORWARD_MATRIX_2, TAG_IMAGE_LENGTH,
    TAG_IMAGE_WIDTH, TAG_NEW_SUBFILE_TYPE, TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2, TAG_OPCODE_LIST_3,
    TAG_ORIENTATION, TAG_PHOTOMETRIC, TAG_ROWS_PER_STRIP, TAG_SAMPLES_PER_PIXEL,
    TAG_STRIP_BYTE_COUNTS, TAG_STRIP_OFFSETS, TAG_SUB_IFDS, TAG_WHITE_LEVEL,
};

// ─────────────────────────────────────────────────────────────────────────────
// Tags and field types the reader has no constant for
// ─────────────────────────────────────────────────────────────────────────────

/// TIFF 6.0 §15 `TileWidth`.
pub const TAG_TILE_WIDTH: u16 = 322;
/// TIFF 6.0 §15 `TileLength`.
pub const TAG_TILE_LENGTH: u16 = 323;
/// TIFF 6.0 §15 `TileOffsets`.
pub const TAG_TILE_OFFSETS: u16 = 324;
/// TIFF 6.0 §15 `TileByteCounts`.
pub const TAG_TILE_BYTE_COUNTS: u16 = 325;
/// DNG 1.7 `DNGVersion` (IFD0).
pub const TAG_DNG_VERSION: u16 = 50706;
/// DNG 1.7 `DNGBackwardVersion` (IFD0).
pub const TAG_DNG_BACKWARD_VERSION: u16 = 50707;
/// DNG 1.7 `UniqueCameraModel` (IFD0).
pub const TAG_UNIQUE_CAMERA_MODEL: u16 = 50708;

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_UNDEFINED: u16 = 7;
const TYPE_SRATIONAL: u16 = 10;

const PHOTOMETRIC_BLACK_IS_ZERO: u16 = 1;
const PHOTOMETRIC_CFA: u16 = 32803;
const PHOTOMETRIC_LINEAR_RAW: u16 = 34892;

// ─────────────────────────────────────────────────────────────────────────────
// The description
// ─────────────────────────────────────────────────────────────────────────────

/// Where the sensor plane sits in the IFD tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfdLayout {
    /// IFD0 is the raw plane; every tag goes there.
    SensorInIfd0,
    /// IFD0 is a tiny valid 1×1 8-bit image (`NewSubfileType = 1`) whose
    /// `SubIFDs` points at the raw plane — the shape real cameras write. Not a
    /// preview feature: it exists only so the sensor sits in a SubIFD.
    PreviewIfd0WithSensorSubIfd,
}

/// What the single-sample plane encodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaneKind {
    /// `PhotometricInterpretation = 34892`, monochrome.
    LinearRaw,
    /// `PhotometricInterpretation = 32803`, a colour filter array: `pattern` is
    /// `repeat_dim[0] × repeat_dim[1]` colour indices, row-major.
    Cfa {
        repeat_dim: [u16; 2],
        pattern: Vec<u8>,
        plane_color: Vec<u8>,
    },
}

/// How the plane is cut into chunks.
///
/// TIFF 6.0 §15 says tile width and length "must be a multiple of 16"; that is
/// deliberately **not** enforced, so a fixture can break it on purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Organisation {
    /// `RowsPerStrip` rows per strip; the last strip holds the remainder.
    Strips { rows_per_strip: u32 },
    /// Row-major tiles; samples beyond the plane's right and bottom edges are
    /// `edge_fill`, so a reader that wrongly reads padding shows a sentinel
    /// rather than a plausible zero.
    Tiles {
        tile_width: u32,
        tile_length: u32,
        edge_fill: u16,
    },
}

/// The plane's bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// `width × height` samples, row-major, packed by the writer as
    /// Compression 1 (`DEC-008`): 8- and 16-bit as container-order integers,
    /// 12- and 14-bit as an MSB-first bitstream (byte order does not apply).
    Samples(Vec<u16>),
    /// Pre-encoded chunks written verbatim, one per strip or tile, with
    /// `Compression = compression`. The writer adds no bytes to a chunk.
    Encoded {
        compression: u16,
        chunks: Vec<Vec<u8>>,
    },
}

/// The metadata the reader models. `None` means the tag is not written. The
/// three DNG identity fields default to `Some` (`DNGVersion` 1.4.0.0,
/// `DNGBackwardVersion` 1.1.0.0, `UniqueCameraModel` `"irradiance-fixture"`).
///
/// Placement follows `Container::sensor` (`src/ifd.rs`): the identity tags, the
/// colour matrices, illuminants, `AsShotNeutral` and `Orientation` go in IFD0;
/// the level, crop and opcode tags go in the sensor IFD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DngMeta {
    /// `BlackLevel`, LONG per channel.
    pub black_level: Option<Vec<u32>>,
    /// `BlackLevelRepeatDim` `[rows, cols]`.
    pub black_level_repeat_dim: Option<[u16; 2]>,
    /// `WhiteLevel`, LONG per channel.
    pub white_level: Option<Vec<u32>>,
    /// `ActiveArea` `[top, left, bottom, right]`.
    pub active_area: Option<[u32; 4]>,
    /// `DefaultCropOrigin` `[x, y]`.
    pub default_crop_origin: Option<[u32; 2]>,
    /// `DefaultCropSize` `[width, height]`.
    pub default_crop_size: Option<[u32; 2]>,
    /// `Orientation`.
    pub orientation: Option<u16>,
    /// `ColorMatrix1`, SRATIONAL `(num, den)` verbatim.
    pub color_matrix_1: Option<Vec<(i32, i32)>>,
    /// `ColorMatrix2`.
    pub color_matrix_2: Option<Vec<(i32, i32)>>,
    /// `ForwardMatrix1`.
    pub forward_matrix_1: Option<Vec<(i32, i32)>>,
    /// `ForwardMatrix2`.
    pub forward_matrix_2: Option<Vec<(i32, i32)>>,
    /// `AsShotNeutral`, RATIONAL `(num, den)` verbatim.
    pub as_shot_neutral: Option<Vec<(u32, u32)>>,
    /// `CalibrationIlluminant1`.
    pub calibration_illuminant_1: Option<u16>,
    /// `CalibrationIlluminant2`.
    pub calibration_illuminant_2: Option<u16>,
    /// `DNGVersion`.
    pub dng_version: Option<[u8; 4]>,
    /// `DNGBackwardVersion`.
    pub dng_backward_version: Option<[u8; 4]>,
    /// `UniqueCameraModel`, written as NUL-terminated ASCII.
    pub unique_camera_model: Option<String>,
}

impl Default for DngMeta {
    fn default() -> DngMeta {
        DngMeta {
            black_level: None,
            black_level_repeat_dim: None,
            white_level: None,
            active_area: None,
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            color_matrix_1: None,
            color_matrix_2: None,
            forward_matrix_1: None,
            forward_matrix_2: None,
            as_shot_neutral: None,
            calibration_illuminant_1: None,
            calibration_illuminant_2: None,
            dng_version: Some([1, 4, 0, 0]),
            dng_backward_version: Some([1, 1, 0, 0]),
            unique_camera_model: Some("irradiance-fixture".to_string()),
        }
    }
}

/// Which IFD a [`RawTag`] goes in. Under [`IfdLayout::SensorInIfd0`] both name
/// the one IFD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagIfd {
    Ifd0,
    Sensor,
}

/// A raw IFD entry, written as given: `count` verbatim (it need not agree with
/// `bytes`), `bytes` already in file order. Inline when `bytes.len() <= 4`
/// (left-justified in the value word, TIFF 6.0 §2), otherwise out of line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTag {
    pub ifd: TagIfd,
    pub tag: u16,
    pub field_type: u16,
    pub count: u32,
    pub bytes: Vec<u8>,
}

/// Deliberate writer faults for the red-proof (`oracle-must-be-shown-red`).
/// Test-only; never set outside a red-proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriterFault {
    /// Write 8/16-bit samples in the opposite byte order to the container's.
    SwapByteAlignedOrder,
    /// Pack 12/14-bit samples LSB-first instead of MSB-first.
    LsbFirstPacking,
    /// Point every `TileOffsets` entry one sample late (the file is padded so
    /// the last tile still lies inside it).
    TileOffsetOffByOne,
}

/// A DNG to write.
#[derive(Debug, Clone)]
pub struct DngDescription {
    pub order: Order,
    pub layout: IfdLayout,
    pub width: u32,
    pub height: u32,
    /// 8, 12, 14 or 16 for [`Payload::Samples`]; anything that fits a SHORT
    /// for [`Payload::Encoded`] (e.g. the lossless-JPEG precision).
    pub bits_per_sample: u32,
    pub kind: PlaneKind,
    pub organisation: Organisation,
    pub payload: Payload,
    pub meta: DngMeta,
    /// `OpcodeList1`/`2`/`3`, UNDEFINED, in the sensor IFD.
    pub opcode_lists: [Option<Vec<u8>>; 3],
    /// Replaces any generated tag with the same `(ifd, tag)`, otherwise
    /// appended. The hostile escape hatch.
    pub extra_tags: Vec<RawTag>,
    /// `None` except in a red-proof.
    pub fault: Option<WriterFault>,
}

/// Why a description cannot be written. Returned, never panicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// `Payload::Samples` does not hold `width × height` samples.
    SampleCountMismatch { expected: u64, actual: usize },
    /// A sample exceeds `2^bits − 1`.
    SampleOutOfRange { index: usize, value: u16, bits: u32 },
    /// A tile's `edge_fill` exceeds `2^bits − 1`.
    EdgeFillOutOfRange { value: u16, bits: u32 },
    /// `bits_per_sample` is not 8/12/14/16 under `Payload::Samples`.
    UnsupportedBitDepth { bits: u32 },
    /// `pattern.len() != repeat_dim[0] × repeat_dim[1]`.
    CfaPatternLength { expected: u32, actual: usize },
    /// `Payload::Encoded` has a chunk count other than the strip/tile count.
    ChunkCountMismatch { expected: u32, actual: usize },
    /// A row (`width` for strips, `tile_width` for tiles) is not a whole
    /// number of bytes. TIFF 6.0 (row padding), irradiance's reader
    /// (`DEC-008`, one continuous stream per strip) and dnglab all disagree on
    /// such rows (SPEC-057 design probe), so the writer refuses rather than
    /// encode an unsettled rule.
    RowNotWholeBytes { row_samples: u32, bits: u32 },
    /// A zero dimension, `RowsPerStrip` or tile size.
    InvalidGeometry { what: &'static str },
    /// A count, offset or size does not fit the `u32` TIFF stores it in.
    Overflow { what: &'static str },
}

// ─────────────────────────────────────────────────────────────────────────────
// The entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Lay out `d` as a DNG. Deterministic: the same description always gives the
/// same bytes.
pub fn write_dng(d: &DngDescription) -> Result<Vec<u8>, WriteError> {
    let grid = validate(d)?;
    let chunks: Vec<Vec<u8>> = match &d.payload {
        Payload::Samples(samples) => pack_chunks(d, &grid, samples)?,
        Payload::Encoded { chunks, .. } => chunks.clone(),
    };
    let chunk_lens = chunks
        .iter()
        .map(|c| u32::try_from(c.len()).map_err(|_| overflow("chunk length")))
        .collect::<Result<Vec<u32>, WriteError>>()?;

    // Two passes. Every field's SIZE is independent of the offsets it holds,
    // so pass 1 (zero offsets) fixes the layout and pass 2 fills it in.
    let zero = Placement::zero(chunks.len());
    let draft = build_ifds(d, &chunk_lens, &zero)?;
    let placement = lay_out(&draft, &chunk_lens)?;
    let ifds = build_ifds(d, &chunk_lens, &placement)?;
    if lay_out(&ifds, &chunk_lens)? != placement {
        return Err(WriteError::InvalidGeometry {
            what: "layout changed between passes",
        });
    }
    serialize(d, &ifds, &placement, &chunks)
}

fn overflow(what: &'static str) -> WriteError {
    WriteError::Overflow { what }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────────────────────────

/// The chunk grid a valid description cuts into.
struct Grid {
    /// Number of strips or tiles.
    count: u32,
}

fn validate(d: &DngDescription) -> Result<Grid, WriteError> {
    if d.width == 0 || d.height == 0 {
        return Err(WriteError::InvalidGeometry {
            what: "width or height is zero",
        });
    }
    let (count, row_samples): (u64, u32) = match d.organisation {
        Organisation::Strips { rows_per_strip } => {
            if rows_per_strip == 0 {
                return Err(WriteError::InvalidGeometry {
                    what: "rows_per_strip is zero",
                });
            }
            (u64::from(d.height.div_ceil(rows_per_strip)), d.width)
        }
        Organisation::Tiles {
            tile_width,
            tile_length,
            ..
        } => {
            if tile_width == 0 || tile_length == 0 {
                return Err(WriteError::InvalidGeometry {
                    what: "tile width or length is zero",
                });
            }
            let across = u64::from(d.width.div_ceil(tile_width));
            let down = u64::from(d.height.div_ceil(tile_length));
            (
                across.checked_mul(down).ok_or(overflow("tile count"))?,
                tile_width,
            )
        }
    };
    let count = u32::try_from(count).map_err(|_| overflow("chunk count"))?;
    // The offset and byte-count arrays are LONG × count each.
    count.checked_mul(4).ok_or(overflow("chunk offset array"))?;

    if let PlaneKind::Cfa {
        repeat_dim,
        pattern,
        ..
    } = &d.kind
    {
        // Two u16s: the product always fits u32.
        let expected = u32::from(repeat_dim[0]) * u32::from(repeat_dim[1]);
        if usize::try_from(expected).ok() != Some(pattern.len()) {
            return Err(WriteError::CfaPatternLength {
                expected,
                actual: pattern.len(),
            });
        }
    }

    match &d.payload {
        Payload::Samples(samples) => {
            let bits = d.bits_per_sample;
            if ![8, 12, 14, 16].contains(&bits) {
                return Err(WriteError::UnsupportedBitDepth { bits });
            }
            let row_bits = u64::from(row_samples) * u64::from(bits);
            if row_bits % 8 != 0 {
                return Err(WriteError::RowNotWholeBytes { row_samples, bits });
            }
            let expected = u64::from(d.width) * u64::from(d.height);
            if u64::try_from(samples.len()).ok() != Some(expected) {
                return Err(WriteError::SampleCountMismatch {
                    expected,
                    actual: samples.len(),
                });
            }
            let max = max_sample(bits);
            if let Some((index, &value)) = samples.iter().enumerate().find(|(_, &s)| s > max) {
                return Err(WriteError::SampleOutOfRange { index, value, bits });
            }
            if let Organisation::Tiles { edge_fill, .. } = d.organisation {
                if edge_fill > max {
                    return Err(WriteError::EdgeFillOutOfRange {
                        value: edge_fill,
                        bits,
                    });
                }
            }
        }
        Payload::Encoded { chunks, .. } => {
            if usize::try_from(count).ok() != Some(chunks.len()) {
                return Err(WriteError::ChunkCountMismatch {
                    expected: count,
                    actual: chunks.len(),
                });
            }
            u16::try_from(d.bits_per_sample).map_err(|_| overflow("bits_per_sample"))?;
        }
    }
    Ok(Grid { count })
}

/// `2^bits − 1` for a supported depth (8..=16).
fn max_sample(bits: u32) -> u16 {
    u16::MAX >> (16 - bits)
}

// ─────────────────────────────────────────────────────────────────────────────
// Packing (Compression 1, DEC-008 in the writing direction)
// ─────────────────────────────────────────────────────────────────────────────

fn pack_chunks(
    d: &DngDescription,
    grid: &Grid,
    samples: &[u16],
) -> Result<Vec<Vec<u8>>, WriteError> {
    let width = usize::try_from(d.width).map_err(|_| overflow("width"))?;
    let rows: Vec<&[u16]> = samples.chunks_exact(width).collect();
    let mut chunks = Vec::with_capacity(usize::try_from(grid.count).unwrap_or(0));
    match d.organisation {
        Organisation::Strips { rows_per_strip } => {
            let rps = usize::try_from(rows_per_strip).map_err(|_| overflow("rows_per_strip"))?;
            for strip_rows in rows.chunks(rps) {
                let mut out = Vec::new();
                for row in strip_rows {
                    pack_row(row.iter().copied(), d, &mut out);
                }
                chunks.push(out);
            }
        }
        Organisation::Tiles {
            tile_width,
            tile_length,
            edge_fill,
        } => {
            let tw = usize::try_from(tile_width).map_err(|_| overflow("tile_width"))?;
            let tl = usize::try_from(tile_length).map_err(|_| overflow("tile_length"))?;
            let height = rows.len();
            let across = width.div_ceil(tw);
            let down = height.div_ceil(tl);
            // TIFF 6.0 §15: tiles are stored left-to-right, then top-to-bottom.
            for ty in 0..down {
                for tx in 0..across {
                    let mut out = Vec::new();
                    for r in 0..tl {
                        let row = ty.checked_mul(tl).and_then(|y| y.checked_add(r));
                        let row = row.and_then(|y| rows.get(y));
                        let x0 = tx.checked_mul(tw).ok_or(overflow("tile x"))?;
                        // Beyond the plane (a missing row, or a column past
                        // its end) is the edge fill.
                        let line = (0..tw).map(|c| {
                            row.and_then(|row| x0.checked_add(c).and_then(|x| row.get(x)))
                                .copied()
                                .unwrap_or(edge_fill)
                        });
                        pack_row(line, d, &mut out);
                    }
                    chunks.push(out);
                }
            }
        }
    }
    Ok(chunks)
}

/// Pack one row. Rows are whole bytes (validated), so packing row by row is
/// the same bitstream as packing the strip continuously.
fn pack_row(row: impl Iterator<Item = u16>, d: &DngDescription, out: &mut Vec<u8>) {
    let bits = d.bits_per_sample;
    match bits {
        8 => out.extend(row.map(|s| s.to_le_bytes()[0])),
        16 => {
            let order = if d.fault == Some(WriterFault::SwapByteAlignedOrder) {
                flip(d.order)
            } else {
                d.order
            };
            for s in row {
                out.extend_from_slice(&u16_bytes(order, s));
            }
        }
        // 12 and 14: a bitstream, byte order does not apply. `acc` never holds
        // more than 7 + 16 bits.
        _ => {
            let lsb_first = d.fault == Some(WriterFault::LsbFirstPacking);
            let mut acc: u64 = 0;
            let mut nbits: u32 = 0;
            for s in row {
                if lsb_first {
                    acc |= u64::from(s) << nbits;
                    nbits += bits;
                    while nbits >= 8 {
                        out.push(acc.to_le_bytes()[0]);
                        acc >>= 8;
                        nbits -= 8;
                    }
                } else {
                    acc = (acc << bits) | u64::from(s);
                    nbits += bits;
                    while nbits >= 8 {
                        nbits -= 8;
                        out.push((acc >> nbits).to_le_bytes()[0]);
                    }
                    acc &= (1u64 << nbits) - 1;
                }
            }
        }
    }
}

fn flip(order: Order) -> Order {
    match order {
        Order::Little => Order::Big,
        Order::Big => Order::Little,
    }
}

fn u16_bytes(order: Order, v: u16) -> [u8; 2] {
    match order {
        Order::Little => v.to_le_bytes(),
        Order::Big => v.to_be_bytes(),
    }
}

fn u32_bytes(order: Order, v: u32) -> [u8; 4] {
    match order {
        Order::Little => v.to_le_bytes(),
        Order::Big => v.to_be_bytes(),
    }
}

fn i32_bytes(order: Order, v: i32) -> [u8; 4] {
    match order {
        Order::Little => v.to_le_bytes(),
        Order::Big => v.to_be_bytes(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fields
// ─────────────────────────────────────────────────────────────────────────────

/// One IFD entry with its value bytes in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    tag: u16,
    field_type: u16,
    count: u32,
    bytes: Vec<u8>,
}

/// Builds [`Field`]s in one byte order.
struct F(Order);

impl F {
    fn field(
        &self,
        tag: u16,
        field_type: u16,
        count: usize,
        bytes: Vec<u8>,
    ) -> Result<Field, WriteError> {
        Ok(Field {
            tag,
            field_type,
            count: u32::try_from(count).map_err(|_| overflow("tag count"))?,
            bytes,
        })
    }
    fn shorts(&self, tag: u16, v: &[u16]) -> Result<Field, WriteError> {
        let bytes = v.iter().flat_map(|&x| u16_bytes(self.0, x)).collect();
        self.field(tag, TYPE_SHORT, v.len(), bytes)
    }
    fn short(&self, tag: u16, v: u16) -> Result<Field, WriteError> {
        self.shorts(tag, &[v])
    }
    fn longs(&self, tag: u16, v: &[u32]) -> Result<Field, WriteError> {
        let bytes = v.iter().flat_map(|&x| u32_bytes(self.0, x)).collect();
        self.field(tag, TYPE_LONG, v.len(), bytes)
    }
    fn long(&self, tag: u16, v: u32) -> Result<Field, WriteError> {
        self.longs(tag, &[v])
    }
    fn bytes(&self, tag: u16, field_type: u16, v: &[u8]) -> Result<Field, WriteError> {
        self.field(tag, field_type, v.len(), v.to_vec())
    }
    fn ascii(&self, tag: u16, s: &str) -> Result<Field, WriteError> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0); // TIFF 6.0 §2: ASCII is NUL-terminated, count includes it.
        self.field(tag, TYPE_ASCII, bytes.len(), bytes)
    }
    fn srationals(&self, tag: u16, v: &[(i32, i32)]) -> Result<Field, WriteError> {
        let bytes = v
            .iter()
            .flat_map(|&(n, d)| [i32_bytes(self.0, n), i32_bytes(self.0, d)])
            .flatten()
            .collect();
        self.field(tag, TYPE_SRATIONAL, v.len(), bytes)
    }
    fn rationals(&self, tag: u16, v: &[(u32, u32)]) -> Result<Field, WriteError> {
        let bytes = v
            .iter()
            .flat_map(|&(n, d)| [u32_bytes(self.0, n), u32_bytes(self.0, d)])
            .flatten()
            .collect();
        self.field(tag, TYPE_RATIONAL, v.len(), bytes)
    }
}

/// Where everything the IFDs point at lands.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Placement {
    /// Offset of the sensor IFD under the SubIFD layout; 0 otherwise.
    sensor_ifd: u32,
    /// Offset of the 1-byte preview strip under the SubIFD layout; 0 otherwise.
    preview_strip: u32,
    /// Offset of each data chunk.
    chunks: Vec<u32>,
    /// Offset of each out-of-line value, per IFD, in serialized entry order.
    values: Vec<Vec<Option<u32>>>,
    /// Total file length.
    len: u32,
}

impl Placement {
    fn zero(chunks: usize) -> Placement {
        Placement {
            sensor_ifd: 0,
            preview_strip: 0,
            chunks: vec![0; chunks],
            values: Vec::new(),
            len: 0,
        }
    }
}

/// The IFDs, each a tag-sorted entry list: `[ifd0]` or `[ifd0, sensor]`.
fn build_ifds(
    d: &DngDescription,
    chunk_lens: &[u32],
    at: &Placement,
) -> Result<Vec<Vec<Field>>, WriteError> {
    let f = F(d.order);
    let m = &d.meta;

    // IFD0-resident: identity, colour, orientation (Container::sensor reads
    // these from IFD0).
    let mut ifd0 = Vec::new();
    if let Some(v) = m.dng_version {
        ifd0.push(f.bytes(TAG_DNG_VERSION, TYPE_BYTE, &v)?);
    }
    if let Some(v) = m.dng_backward_version {
        ifd0.push(f.bytes(TAG_DNG_BACKWARD_VERSION, TYPE_BYTE, &v)?);
    }
    if let Some(v) = &m.unique_camera_model {
        ifd0.push(f.ascii(TAG_UNIQUE_CAMERA_MODEL, v)?);
    }
    if let Some(v) = m.orientation {
        ifd0.push(f.short(TAG_ORIENTATION, v)?);
    }
    for (tag, v) in [
        (TAG_COLOR_MATRIX_1, &m.color_matrix_1),
        (TAG_COLOR_MATRIX_2, &m.color_matrix_2),
        (TAG_FORWARD_MATRIX_1, &m.forward_matrix_1),
        (TAG_FORWARD_MATRIX_2, &m.forward_matrix_2),
    ] {
        if let Some(v) = v {
            ifd0.push(f.srationals(tag, v)?);
        }
    }
    if let Some(v) = &m.as_shot_neutral {
        ifd0.push(f.rationals(TAG_AS_SHOT_NEUTRAL, v)?);
    }
    for (tag, v) in [
        (TAG_CALIBRATION_ILLUMINANT_1, m.calibration_illuminant_1),
        (TAG_CALIBRATION_ILLUMINANT_2, m.calibration_illuminant_2),
    ] {
        if let Some(v) = v {
            ifd0.push(f.short(tag, v)?);
        }
    }

    // Sensor-resident: the plane, CFA, levels, crop, opcodes.
    let mut sensor = Vec::new();
    let compression = match &d.payload {
        Payload::Samples(_) => 1,
        Payload::Encoded { compression, .. } => *compression,
    };
    let bits = u16::try_from(d.bits_per_sample).map_err(|_| overflow("bits_per_sample"))?;
    sensor.push(f.long(TAG_NEW_SUBFILE_TYPE, 0)?);
    sensor.push(f.long(TAG_IMAGE_WIDTH, d.width)?);
    sensor.push(f.long(TAG_IMAGE_LENGTH, d.height)?);
    sensor.push(f.short(TAG_BITS_PER_SAMPLE, bits)?);
    sensor.push(f.short(TAG_COMPRESSION, compression)?);
    let photometric = match d.kind {
        PlaneKind::LinearRaw => PHOTOMETRIC_LINEAR_RAW,
        PlaneKind::Cfa { .. } => PHOTOMETRIC_CFA,
    };
    sensor.push(f.short(TAG_PHOTOMETRIC, photometric)?);
    sensor.push(f.short(TAG_SAMPLES_PER_PIXEL, 1)?);
    match d.organisation {
        Organisation::Strips { rows_per_strip } => {
            sensor.push(f.longs(TAG_STRIP_OFFSETS, &at.chunks)?);
            sensor.push(f.long(TAG_ROWS_PER_STRIP, rows_per_strip)?);
            sensor.push(f.longs(TAG_STRIP_BYTE_COUNTS, chunk_lens)?);
        }
        Organisation::Tiles {
            tile_width,
            tile_length,
            ..
        } => {
            sensor.push(f.long(TAG_TILE_WIDTH, tile_width)?);
            sensor.push(f.long(TAG_TILE_LENGTH, tile_length)?);
            let offsets = if d.fault == Some(WriterFault::TileOffsetOffByOne) {
                let late = sample_bytes(d.bits_per_sample);
                at.chunks
                    .iter()
                    .map(|&o| o.checked_add(late).ok_or(overflow("tile offset")))
                    .collect::<Result<Vec<u32>, WriteError>>()?
            } else {
                at.chunks.clone()
            };
            sensor.push(f.longs(TAG_TILE_OFFSETS, &offsets)?);
            sensor.push(f.longs(TAG_TILE_BYTE_COUNTS, chunk_lens)?);
        }
    }
    if let PlaneKind::Cfa {
        repeat_dim,
        pattern,
        plane_color,
    } = &d.kind
    {
        sensor.push(f.shorts(TAG_CFA_REPEAT_PATTERN_DIM, repeat_dim)?);
        sensor.push(f.bytes(TAG_CFA_PATTERN, TYPE_BYTE, pattern)?);
        sensor.push(f.bytes(TAG_CFA_PLANE_COLOR, TYPE_BYTE, plane_color)?);
        sensor.push(f.short(TAG_CFA_LAYOUT, 1)?);
    }
    if let Some(v) = m.black_level_repeat_dim {
        sensor.push(f.shorts(TAG_BLACK_LEVEL_REPEAT_DIM, &v)?);
    }
    if let Some(v) = &m.black_level {
        sensor.push(f.longs(TAG_BLACK_LEVEL, v)?);
    }
    if let Some(v) = &m.white_level {
        sensor.push(f.longs(TAG_WHITE_LEVEL, v)?);
    }
    if let Some(v) = m.default_crop_origin {
        sensor.push(f.longs(TAG_DEFAULT_CROP_ORIGIN, &v)?);
    }
    if let Some(v) = m.default_crop_size {
        sensor.push(f.longs(TAG_DEFAULT_CROP_SIZE, &v)?);
    }
    if let Some(v) = m.active_area {
        sensor.push(f.longs(TAG_ACTIVE_AREA, &v)?);
    }
    for (tag, list) in [TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2, TAG_OPCODE_LIST_3]
        .into_iter()
        .zip(&d.opcode_lists)
    {
        if let Some(bytes) = list {
            sensor.push(f.bytes(tag, TYPE_UNDEFINED, bytes)?);
        }
    }

    let mut ifds = match d.layout {
        IfdLayout::SensorInIfd0 => {
            ifd0.extend(sensor);
            vec![ifd0]
        }
        IfdLayout::PreviewIfd0WithSensorSubIfd => {
            // A tiny valid image: 1×1, 8-bit, one sample, uncompressed.
            ifd0.extend([
                f.long(TAG_NEW_SUBFILE_TYPE, 1)?,
                f.long(TAG_IMAGE_WIDTH, 1)?,
                f.long(TAG_IMAGE_LENGTH, 1)?,
                f.short(TAG_BITS_PER_SAMPLE, 8)?,
                f.short(TAG_COMPRESSION, 1)?,
                f.short(TAG_PHOTOMETRIC, PHOTOMETRIC_BLACK_IS_ZERO)?,
                f.short(TAG_SAMPLES_PER_PIXEL, 1)?,
                f.long(TAG_STRIP_OFFSETS, at.preview_strip)?,
                f.long(TAG_ROWS_PER_STRIP, 1)?,
                f.long(TAG_STRIP_BYTE_COUNTS, 1)?,
                f.long(TAG_SUB_IFDS, at.sensor_ifd)?,
            ]);
            vec![ifd0, sensor]
        }
    };

    // The escape hatch: replace a same-(ifd, tag) entry, else append.
    for raw in &d.extra_tags {
        let index = match (d.layout, raw.ifd) {
            (IfdLayout::SensorInIfd0, _) | (_, TagIfd::Ifd0) => 0,
            (_, TagIfd::Sensor) => 1,
        };
        let field = Field {
            tag: raw.tag,
            field_type: raw.field_type,
            count: raw.count,
            bytes: raw.bytes.clone(),
        };
        if let Some(ifd) = ifds.get_mut(index) {
            match ifd.iter_mut().find(|e| e.tag == raw.tag) {
                Some(existing) => *existing = field,
                None => ifd.push(field),
            }
        }
    }

    // TIFF 6.0 §2: entries in ascending tag order. Stable, so a duplicate
    // keeps its relative position (deterministic either way).
    for ifd in &mut ifds {
        ifd.sort_by_key(|e| e.tag);
    }
    Ok(ifds)
}

/// Bytes in one sample, rounded up: the "one sample late" of the tile-offset
/// fault.
fn sample_bytes(bits: u32) -> u32 {
    bits.div_ceil(8)
}

// ─────────────────────────────────────────────────────────────────────────────
// Layout and serialisation
// ─────────────────────────────────────────────────────────────────────────────

/// `n` rounded up to a word (TIFF 6.0 §2: values begin on a word boundary).
fn align2(n: u32) -> Result<u32, WriteError> {
    n.checked_add(n % 2).ok_or(overflow("file size"))
}

fn ifd_size(entries: usize) -> Result<u32, WriteError> {
    let n = u32::try_from(entries).map_err(|_| overflow("entry count"))?;
    n.checked_mul(12)
        .and_then(|b| b.checked_add(6)) // 2-byte count + 4-byte next offset
        .ok_or(overflow("IFD size"))
}

/// Header, then the IFDs back to back, then every out-of-line value (IFD by
/// IFD, entry order), then the preview strip, then the data chunks — each on a
/// word boundary.
fn lay_out(ifds: &[Vec<Field>], chunk_lens: &[u32]) -> Result<Placement, WriteError> {
    let mut pos: u32 = 8;
    let mut ifd_offsets = Vec::new();
    for ifd in ifds {
        ifd_offsets.push(pos);
        pos = pos
            .checked_add(ifd_size(ifd.len())?)
            .ok_or(overflow("file size"))?;
    }
    let mut values = Vec::new();
    for ifd in ifds {
        let mut at = Vec::new();
        for field in ifd {
            if field.bytes.len() > 4 {
                pos = align2(pos)?;
                at.push(Some(pos));
                let len = u32::try_from(field.bytes.len()).map_err(|_| overflow("value size"))?;
                pos = pos.checked_add(len).ok_or(overflow("file size"))?;
            } else {
                at.push(None);
            }
        }
        values.push(at);
    }
    let mut preview_strip = 0;
    if ifds.len() > 1 {
        pos = align2(pos)?;
        preview_strip = pos;
        pos = pos.checked_add(1).ok_or(overflow("file size"))?;
    }
    let mut chunks = Vec::with_capacity(chunk_lens.len());
    for &len in chunk_lens {
        pos = align2(pos)?;
        chunks.push(pos);
        pos = pos.checked_add(len).ok_or(overflow("file size"))?;
    }
    Ok(Placement {
        sensor_ifd: ifd_offsets.get(1).copied().unwrap_or(0),
        preview_strip,
        chunks,
        values,
        len: pos,
    })
}

fn serialize(
    d: &DngDescription,
    ifds: &[Vec<Field>],
    at: &Placement,
    chunks: &[Vec<u8>],
) -> Result<Vec<u8>, WriteError> {
    let order = d.order;
    let total = usize::try_from(at.len).map_err(|_| overflow("file size"))?;
    let mut out: Vec<u8> = Vec::with_capacity(total);
    out.extend_from_slice(match order {
        Order::Little => b"II",
        Order::Big => b"MM",
    });
    out.extend_from_slice(&u16_bytes(order, 42));
    out.extend_from_slice(&u32_bytes(order, 8));

    let mut tail: Vec<(u32, &[u8])> = Vec::new();
    for (ifd, offsets) in ifds.iter().zip(&at.values) {
        let n = u16::try_from(ifd.len()).map_err(|_| overflow("entry count"))?;
        out.extend_from_slice(&u16_bytes(order, n));
        for (field, offset) in ifd.iter().zip(offsets) {
            out.extend_from_slice(&u16_bytes(order, field.tag));
            out.extend_from_slice(&u16_bytes(order, field.field_type));
            out.extend_from_slice(&u32_bytes(order, field.count));
            match offset {
                // Left-justified in the value word, whatever the byte order
                // (TIFF 6.0 §2) — `tiff.rs`'s `short()` trap, handled by
                // keeping bytes in file order.
                None => {
                    let mut word = [0u8; 4];
                    for (w, b) in word.iter_mut().zip(&field.bytes) {
                        *w = *b;
                    }
                    out.extend_from_slice(&word);
                }
                Some(o) => {
                    out.extend_from_slice(&u32_bytes(order, *o));
                    tail.push((*o, &field.bytes));
                }
            }
        }
        out.extend_from_slice(&u32_bytes(order, 0)); // no next IFD
    }
    let preview: [u8; 1] = [0x80];
    if ifds.len() > 1 {
        tail.push((at.preview_strip, &preview));
    }
    for (o, chunk) in at.chunks.iter().zip(chunks) {
        tail.push((*o, chunk));
    }
    // `tail` is already in ascending offset order; the gaps are alignment pads.
    for (o, bytes) in tail {
        let o = usize::try_from(o).map_err(|_| overflow("offset"))?;
        out.resize(o, 0);
        out.extend_from_slice(bytes);
    }
    if d.fault == Some(WriterFault::TileOffsetOffByOne) {
        let late = usize::try_from(sample_bytes(d.bits_per_sample)).unwrap_or(0);
        out.resize(out.len().saturating_add(late), 0);
    }
    Ok(out)
}
