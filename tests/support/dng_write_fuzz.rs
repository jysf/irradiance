//! The `dng_write` fuzz target's input format (`SPEC-061` §8): one byte string
//! decodes to a plane of at most 64×64 and every `WriteOptions` field, plus the
//! pixel budget. Shared by `fuzz/fuzz_targets/dng_write.rs` (which decodes)
//! and `examples/fuzz-seeds.rs` (which encodes the committed seeds), so the
//! two cannot drift.
//!
//! The decoder is total: every byte string, including the empty one, is a
//! case. Short input reads as zeros. Values are deliberately NOT clamped to
//! valid ranges (an active area may lie outside the plane, an orientation may
//! be 200), so the writer's refusals are reached as well as its successes.
//!
//! Layout, in order:
//!
//! | bytes | field |
//! |---|---|
//! | 1 | framing (`% 3`: full sensor, full active area, standard crop) |
//! | 1, 1 | width, height (`% 65`, so 0 reaches `EmptyPlane`) |
//! | 1 | flags: bit 0 active area, 1 crop, 2 repeat dim, 3/4/5 opcode list 1/2/3, 6 model, 7 drop one sample |
//! | 4 | active area top, left, bottom, right (if flagged) |
//! | 4 | crop x, y, width, height (if flagged) |
//! | 1 | orientation |
//! | 2 | repeat dim rows, cols (`% 4`; if flagged) |
//! | 1 + 2n | black level count (`% 5`), then each as `u16` LE |
//! | 2 | white level, `u16` LE |
//! | 1 + 4n | masked-area count (`% 3`), then each as four bytes |
//! | 2 | `max_pixels`, `u16` LE |
//! | 1 + n | each flagged opcode list: length, bytes |
//! | 1 + n | the model (if flagged): length, bytes |
//! | 2 | metadata flags (`SPEC-081`), `u16` LE: bit 0 EXIF block, 1 Exif IFD offset, 2 GPS IFD offset, 3 XMP, 4 to 11 the eight descriptive strings in tag order, 12 to 15 the four hints in tag order |
//! | 1 + n | the EXIF block (if flagged): length, bytes |
//! | 1, 1 | the Exif and GPS IFD offsets inside the block (each if flagged) |
//! | 1 + n | the XMP packet (if flagged): length, bytes |
//! | 1 + n | each flagged descriptive string: length, bytes |
//! | 2 | each flagged hint: numerator, denominator, as `i8` |
//! | rest | samples, `u16` LE, repeated to fill the plane |
#![allow(dead_code)]

use irradiance::dng::write::ExifBlock;
use irradiance::dng::write::{Framing, WriteOptions};
use irradiance::ifd::{ActiveArea, DefaultCropOrigin, DefaultCropSize, Rational};

/// One decoded case, owning everything [`Case::options`] borrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub framing: u8,
    pub width: u32,
    pub height: u32,
    pub samples: Vec<u16>,
    pub active_area: Option<[u8; 4]>,
    pub crop: Option<[u8; 4]>,
    pub orientation: u8,
    pub repeat_dim: Option<[u8; 2]>,
    pub black_level: Vec<u16>,
    pub white_level: u16,
    pub masked: Vec<[u8; 4]>,
    pub max_pixels: u16,
    pub lists: [Option<Vec<u8>>; 3],
    pub model: Option<Vec<u8>>,
    pub drop_one_sample: bool,
    /// The EXIF block's bytes (`SPEC-081`).
    pub exif_block: Option<Vec<u8>>,
    pub exif_ifd: Option<u8>,
    pub gps_ifd: Option<u8>,
    pub xmp: Option<Vec<u8>>,
    /// `Make`, `Model`, `Software`, `DateTime`, `Artist`, `Copyright`,
    /// `LocalizedCameraModel`, `CameraSerialNumber`.
    pub strings: [Option<Vec<u8>>; 8],
    /// `BaselineExposure`, `BaselineNoise`, `BaselineSharpness`,
    /// `LinearResponseLimit`, each as numerator and denominator.
    pub hints: [Option<[i8; 2]>; 4],
}

/// What [`Case::options`] borrows from: the widened values.
pub struct Owned {
    pub black_level: Vec<u32>,
    pub masked: Vec<ActiveArea>,
    pub model: Option<String>,
    pub strings: [Option<String>; 8],
}

fn rect(r: [u8; 4]) -> ActiveArea {
    ActiveArea {
        top: u32::from(r[0]),
        left: u32::from(r[1]),
        bottom: u32::from(r[2]),
        right: u32::from(r[3]),
    }
}

impl Case {
    pub fn framing(&self) -> Framing {
        match self.framing % 3 {
            0 => Framing::FullSensor,
            1 => Framing::FullActiveArea,
            _ => Framing::StandardCrop,
        }
    }

    pub fn owned(&self) -> Owned {
        Owned {
            black_level: self.black_level.iter().map(|&b| u32::from(b)).collect(),
            masked: self.masked.iter().copied().map(rect).collect(),
            model: self
                .model
                .as_ref()
                .map(|m| String::from_utf8_lossy(m).into_owned()),
            strings: self
                .strings
                .clone()
                .map(|s| s.as_ref().map(|b| String::from_utf8_lossy(b).into_owned())),
        }
    }

    pub fn options<'a>(&'a self, owned: &'a Owned) -> WriteOptions<'a> {
        let mut o = WriteOptions::default();
        o.framing = self.framing();
        o.active_area = self.active_area.map(rect);
        o.default_crop = self.crop.map(|c| {
            (
                DefaultCropOrigin {
                    x: u32::from(c[0]),
                    y: u32::from(c[1]),
                },
                DefaultCropSize {
                    width: u32::from(c[2]),
                    height: u32::from(c[3]),
                },
            )
        });
        o.orientation = u16::from(self.orientation);
        o.black_level_repeat_dim = self
            .repeat_dim
            .map(|[r, c]| [u16::from(r % 4), u16::from(c % 4)]);
        o.black_level = &owned.black_level;
        o.white_level = u32::from(self.white_level);
        o.masked_areas = &owned.masked;
        o.opcode_list_1 = self.lists[0].as_deref();
        o.opcode_list_2 = self.lists[1].as_deref();
        o.opcode_list_3 = self.lists[2].as_deref();
        o.unique_camera_model = owned.model.as_deref();
        o.exif_block = self.exif_block.as_deref().map(|bytes| {
            let mut b = ExifBlock::default();
            b.bytes = bytes;
            b.exif_ifd = self.exif_ifd.map(u32::from);
            b.gps_ifd = self.gps_ifd.map(u32::from);
            b
        });
        o.xmp = self.xmp.as_deref();
        let [make, model, software, date_time, artist, copyright, localized, serial] =
            &owned.strings;
        o.make = make.as_deref();
        o.model = model.as_deref();
        o.software = software.as_deref();
        o.date_time = date_time.as_deref();
        o.artist = artist.as_deref();
        o.copyright = copyright.as_deref();
        o.localized_camera_model = localized.as_deref();
        o.camera_serial_number = serial.as_deref();
        let hint = |h: Option<[i8; 2]>| {
            h.map(|[n, d]| Rational {
                num: i32::from(n),
                den: i32::from(d),
            })
        };
        o.baseline_exposure = hint(self.hints[0]);
        o.baseline_noise = hint(self.hints[1]);
        o.baseline_sharpness = hint(self.hints[2]);
        o.linear_response_limit = hint(self.hints[3]);
        o
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> u8 {
        let b = self.data.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        b
    }
    fn u16(&mut self) -> u16 {
        u16::from_le_bytes([self.u8(), self.u8()])
    }
    fn four(&mut self) -> [u8; 4] {
        [self.u8(), self.u8(), self.u8(), self.u8()]
    }
    fn bytes(&mut self) -> Vec<u8> {
        let n = usize::from(self.u8());
        (0..n).map(|_| self.u8()).collect()
    }
    fn rest(&self) -> &[u8] {
        self.data.get(self.at..).unwrap_or(&[])
    }
}

/// Decode any byte string into a case.
pub fn decode(data: &[u8]) -> Case {
    let mut r = Reader { data, at: 0 };
    let framing = r.u8() % 3;
    let width = u32::from(r.u8() % 65);
    let height = u32::from(r.u8() % 65);
    let flags = r.u8();
    let bit = |n: u8| flags & (1 << n) != 0;
    let active_area = bit(0).then(|| r.four());
    let crop = bit(1).then(|| r.four());
    let orientation = r.u8();
    let repeat_dim = bit(2).then(|| [r.u8() % 4, r.u8() % 4]);
    let blacks = r.u8() % 5;
    let black_level = (0..blacks).map(|_| r.u16()).collect();
    let white_level = r.u16();
    let masks = r.u8() % 3;
    let masked = (0..masks).map(|_| r.four()).collect();
    let max_pixels = r.u16();
    let lists = [
        bit(3).then(|| r.bytes()),
        bit(4).then(|| r.bytes()),
        bit(5).then(|| r.bytes()),
    ];
    let model = bit(6).then(|| r.bytes());
    let drop_one_sample = bit(7);
    let meta = r.u16();
    let on = |n: u16| meta & (1 << n) != 0;
    let exif_block = on(0).then(|| r.bytes());
    let exif_ifd = on(1).then(|| r.u8());
    let gps_ifd = on(2).then(|| r.u8());
    let xmp = on(3).then(|| r.bytes());
    let strings = core::array::from_fn(|i| on(4 + i as u16).then(|| r.bytes()));
    let hints = core::array::from_fn(|i| on(12 + i as u16).then(|| [r.u8() as i8, r.u8() as i8]));

    let pixels = (width * height) as usize;
    let raw: Vec<u16> = r
        .rest()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_le_bytes(c))
        .collect();
    let mut samples: Vec<u16> = (0..pixels)
        .map(|i| {
            if raw.is_empty() {
                (i as u16).wrapping_mul(257)
            } else {
                raw[i % raw.len()]
            }
        })
        .collect();
    if drop_one_sample {
        samples.pop();
    }
    Case {
        framing,
        width,
        height,
        samples,
        active_area,
        crop,
        orientation,
        repeat_dim,
        black_level,
        white_level,
        masked,
        max_pixels,
        lists,
        model,
        drop_one_sample,
        exif_block,
        exif_ifd,
        gps_ifd,
        xmp,
        strings,
        hints,
    }
}

/// Encode a case so that [`decode`] gives it back. `samples` are written once;
/// the decoder repeats them to fill the plane, so a seed can carry a short
/// pattern. Lists and the model are capped at 255 bytes.
pub fn encode(c: &Case) -> Vec<u8> {
    let mut out = vec![c.framing % 3, (c.width % 65) as u8, (c.height % 65) as u8];
    let mut flags = 0u8;
    for (n, on) in [
        c.active_area.is_some(),
        c.crop.is_some(),
        c.repeat_dim.is_some(),
        c.lists[0].is_some(),
        c.lists[1].is_some(),
        c.lists[2].is_some(),
        c.model.is_some(),
        c.drop_one_sample,
    ]
    .into_iter()
    .enumerate()
    {
        if on {
            flags |= 1 << n;
        }
    }
    out.push(flags);
    if let Some(a) = c.active_area {
        out.extend_from_slice(&a);
    }
    if let Some(a) = c.crop {
        out.extend_from_slice(&a);
    }
    out.push(c.orientation);
    if let Some(d) = c.repeat_dim {
        out.extend_from_slice(&d);
    }
    out.push(c.black_level.len() as u8);
    for b in &c.black_level {
        out.extend_from_slice(&b.to_le_bytes());
    }
    out.extend_from_slice(&c.white_level.to_le_bytes());
    out.push(c.masked.len() as u8);
    for m in &c.masked {
        out.extend_from_slice(m);
    }
    out.extend_from_slice(&c.max_pixels.to_le_bytes());
    for list in c.lists.iter().flatten() {
        out.push(list.len().min(255) as u8);
        out.extend_from_slice(&list[..list.len().min(255)]);
    }
    if let Some(m) = &c.model {
        out.push(m.len().min(255) as u8);
        out.extend_from_slice(&m[..m.len().min(255)]);
    }
    let mut meta = 0u16;
    let present = [
        c.exif_block.is_some(),
        c.exif_ifd.is_some(),
        c.gps_ifd.is_some(),
        c.xmp.is_some(),
    ]
    .into_iter()
    .chain(c.strings.iter().map(Option::is_some))
    .chain(c.hints.iter().map(Option::is_some));
    for (n, on) in present.enumerate() {
        if on {
            meta |= 1 << n;
        }
    }
    out.extend_from_slice(&meta.to_le_bytes());
    if let Some(b) = &c.exif_block {
        push_bytes(&mut out, b);
    }
    if let Some(o) = c.exif_ifd {
        out.push(o);
    }
    if let Some(o) = c.gps_ifd {
        out.push(o);
    }
    if let Some(x) = &c.xmp {
        push_bytes(&mut out, x);
    }
    for s in c.strings.iter().flatten() {
        push_bytes(&mut out, s);
    }
    for [n, d] in c.hints.iter().flatten() {
        out.push(*n as u8);
        out.push(*d as u8);
    }
    for s in &c.samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// A length byte (capped at 255) then that many bytes.
fn push_bytes(out: &mut Vec<u8>, b: &[u8]) {
    out.push(b.len().min(255) as u8);
    out.extend_from_slice(&b[..b.len().min(255)]);
}

/// A valid baseline: a 16×12 plane with an active area and a crop, standard
/// crop, every level set, a model, a generous budget.
pub fn baseline() -> Case {
    Case {
        framing: 2,
        width: 16,
        height: 12,
        samples: (0..16u16).map(|i| i * 4099 + 7).collect(),
        active_area: Some([1, 2, 11, 14]),
        crop: Some([1, 1, 8, 6]),
        orientation: 1,
        repeat_dim: None,
        black_level: vec![64],
        white_level: 60000,
        masked: Vec::new(),
        max_pixels: 4096,
        lists: [None, None, None],
        model: Some(b"fuzz mono".to_vec()),
        drop_one_sample: false,
        exif_block: None,
        exif_ifd: None,
        gps_ifd: None,
        xmp: None,
        strings: Default::default(),
        hints: [None; 4],
    }
}

/// [`baseline`] with every `SPEC-081` field set and valid: a 12-byte block
/// with an empty Exif IFD at 0 and an empty GPS IFD at 6 (count 0, next 0),
/// an XMP packet, the eight strings, the four hints.
pub fn with_all_metadata() -> Case {
    let mut c = baseline();
    c.exif_block = Some(vec![0; 12]);
    c.exif_ifd = Some(0);
    c.gps_ifd = Some(6);
    c.xmp = Some(b"<x:xmpmeta xmlns:x='adobe:ns:meta/'/>".to_vec());
    c.strings = [
        Some(b"Fuzz".to_vec()),
        Some(b"Mono".to_vec()),
        Some(b"irradiance".to_vec()),
        Some(b"2026:11:24 10:00:00".to_vec()),
        Some(Vec::new()),
        Some(b"    ".to_vec()),
        Some(b"Fuzz Mono".to_vec()),
        Some(b"0042".to_vec()),
    ];
    c.hints = [Some([-1, 2]), Some([3, 4]), Some([1, 1]), Some([9, 10])];
    c
}
