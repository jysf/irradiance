//! A test-only EXIF block builder (`SPEC-081` §5 oracle 3): a little-endian
//! run of Exif, Interop and GPS IFDs built for a given base, every offset in
//! it absolute (`base + position in the block`), as `DEC-072` asks the
//! caller to build it.
//!
//! The stand-in for `diamond-exif/SPEC-025`, not a design for it, and never
//! used by `src/`. Written from TIFF 6.0 §2 (IFD layout) and Exif 2.32 §4.6
//! (tag numbers and types); no writer implementation was read.
#![allow(dead_code)]

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_UNDEFINED: u16 = 7;

/// The values the builder writes, so an oracle can compare a tool's reading
/// against exactly what was set.
pub const EXPOSURE_TIME: (u32, u32) = (1, 250);
pub const F_NUMBER: (u32, u32) = (28, 10);
pub const ISO: u16 = 400;
pub const DATE_TIME_ORIGINAL: &str = "2026:11:24 09:30:15";
pub const EXIF_VERSION: &[u8; 4] = b"0232";
pub const INTEROP_INDEX: &str = "R98";
pub const GPS_VERSION_ID: [u8; 4] = [2, 3, 0, 0];
pub const GPS_LATITUDE_REF: &str = "N";
pub const GPS_LATITUDE: [(u32, u32); 3] = [(47, 1), (36, 1), (1234, 100)];

/// Which IFDs the block holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    ExifOnly,
    GpsOnly,
    Both,
}

/// A built block and its IFDs' offsets inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub bytes: Vec<u8>,
    pub exif_ifd: Option<u32>,
    pub gps_ifd: Option<u32>,
}

/// One IFD field: tag, type, count, and its value's bytes (little-endian).
struct Field {
    tag: u16,
    field_type: u16,
    count: u32,
    data: Vec<u8>,
}

fn ascii(tag: u16, text: &str) -> Field {
    let mut data = text.as_bytes().to_vec();
    data.push(0);
    Field {
        tag,
        field_type: TYPE_ASCII,
        count: data.len() as u32,
        data,
    }
}

fn rationals(tag: u16, values: &[(u32, u32)]) -> Field {
    let mut data = Vec::new();
    for (n, d) in values {
        data.extend_from_slice(&n.to_le_bytes());
        data.extend_from_slice(&d.to_le_bytes());
    }
    Field {
        tag,
        field_type: TYPE_RATIONAL,
        count: values.len() as u32,
        data,
    }
}

fn long(tag: u16, value: u32) -> Field {
    Field {
        tag,
        field_type: TYPE_LONG,
        count: 1,
        data: value.to_le_bytes().to_vec(),
    }
}

/// The IFD's size in bytes: count, entries, next-IFD offset, and each
/// out-of-line value at an even offset.
fn ifd_len(fields: &[Field]) -> usize {
    let mut len = 2 + 12 * fields.len() + 4;
    for f in fields.iter().filter(|f| f.data.len() > 4) {
        len += len % 2;
        len += f.data.len();
    }
    len + len % 2
}

/// Append the IFD at `out.len()`, which is its position in the block; every
/// offset written is `base` plus a position in the block.
fn push_ifd(out: &mut Vec<u8>, base: u32, fields: &[Field]) {
    let start = out.len();
    // The out-of-line values follow the IFD; `cursor` is the next one's
    // position in the block.
    let mut cursor = start + 2 + 12 * fields.len() + 4;
    let mut data = Vec::new();
    out.extend_from_slice(&(fields.len() as u16).to_le_bytes());
    for f in fields {
        out.extend_from_slice(&f.tag.to_le_bytes());
        out.extend_from_slice(&f.field_type.to_le_bytes());
        out.extend_from_slice(&f.count.to_le_bytes());
        if f.data.len() > 4 {
            if cursor % 2 == 1 {
                data.push(0);
                cursor += 1;
            }
            out.extend_from_slice(&(base + cursor as u32).to_le_bytes());
            data.extend_from_slice(&f.data);
            cursor += f.data.len();
        } else {
            let mut word = [0u8; 4];
            word[..f.data.len()].copy_from_slice(&f.data);
            out.extend_from_slice(&word);
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&data);
    out.resize(start + ifd_len(fields), 0);
}

fn exif_fields(interop_at: u32) -> Vec<Field> {
    vec![
        rationals(33434, &[EXPOSURE_TIME]), // ExposureTime
        rationals(33437, &[F_NUMBER]),      // FNumber
        Field {
            tag: 34855, // ISO (PhotographicSensitivity)
            field_type: TYPE_SHORT,
            count: 1,
            data: ISO.to_le_bytes().to_vec(),
        },
        Field {
            tag: 36864, // ExifVersion
            field_type: TYPE_UNDEFINED,
            count: 4,
            data: EXIF_VERSION.to_vec(),
        },
        ascii(36867, DATE_TIME_ORIGINAL), // DateTimeOriginal
        long(40965, interop_at),          // InteroperabilityIFDPointer
    ]
}

fn interop_fields() -> Vec<Field> {
    vec![ascii(1, INTEROP_INDEX)] // InteroperabilityIndex
}

fn gps_fields() -> Vec<Field> {
    vec![
        Field {
            tag: 0, // GPSVersionID
            field_type: TYPE_BYTE,
            count: 4,
            data: GPS_VERSION_ID.to_vec(),
        },
        ascii(1, GPS_LATITUDE_REF),  // GPSLatitudeRef
        rationals(2, &GPS_LATITUDE), // GPSLatitude
    ]
}

/// Build a block for `base`: the Exif IFD (with its Interop IFD) first, then
/// the GPS IFD, as `shape` says. Every IFD starts at an even position.
pub fn build(base: u32, shape: Shape) -> Block {
    let mut bytes = Vec::new();
    let mut exif_ifd = None;
    let mut gps_ifd = None;
    if shape != Shape::GpsOnly {
        // The Interop IFD follows the Exif IFD, whose length does not depend
        // on the pointer's value.
        let interop_at = ifd_len(&exif_fields(0)) as u32;
        exif_ifd = Some(0);
        push_ifd(&mut bytes, base, &exif_fields(base + interop_at));
        assert_eq!(bytes.len() as u32, interop_at);
        push_ifd(&mut bytes, base, &interop_fields());
    }
    if shape != Shape::ExifOnly {
        gps_ifd = Some(bytes.len() as u32);
        push_ifd(&mut bytes, base, &gps_fields());
    }
    Block {
        bytes,
        exif_ifd,
        gps_ifd,
    }
}

/// [`build`], then one trailing non-zero byte so the block's length is odd
/// and the writer's pad byte (always 0) is observable.
pub fn build_odd(base: u32, shape: Shape) -> Block {
    let mut block = build(base, shape);
    block.bytes.push(0xAB);
    block
}
