//! Minimal reader for Adobe `dng_validate -2`'s stage-2 TIFF (`SPEC-060`,
//! `DEC-069` §6).
//!
//! It accepts exactly the shape `dng_validate` 1.7.1 writes for a one-plane
//! stage-2 image: a classic TIFF in either byte order, uncompressed
//! (`Compression` 1), one sample per pixel (`SamplesPerPixel` 1), 16 bits per
//! sample, chunky (`PlanarConfiguration` 1 or absent), stored in strips. Any
//! other shape is refused with a message naming the tag, never read as
//! something it is not: a reference that is silently misread is a reference
//! that cannot go red.
//!
//! Tier-B only: it reads SDK outputs generated locally by
//! `scripts/sdk-stage2.sh`, which are never committed.

/// A decoded stage-2 plane: `width × height` samples, row-major.
pub struct Stage2 {
    pub width: u32,
    pub height: u32,
    pub samples: Vec<u16>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    little: bool,
}

impl Reader<'_> {
    fn slice(&self, at: usize, len: usize) -> Result<&[u8], String> {
        at.checked_add(len)
            .and_then(|end| self.bytes.get(at..end))
            .ok_or_else(|| format!("{len} bytes at offset {at} run past the end of the file"))
    }
    fn u16_at(&self, at: usize) -> Result<u16, String> {
        let b: [u8; 2] = self
            .slice(at, 2)?
            .try_into()
            .map_err(|_| "u16".to_string())?;
        Ok(if self.little {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        })
    }
    fn u32_at(&self, at: usize) -> Result<u32, String> {
        let b: [u8; 4] = self
            .slice(at, 4)?
            .try_into()
            .map_err(|_| "u32".to_string())?;
        Ok(if self.little {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    }
}

/// One IFD entry's values, widened to `u32` (SHORT or LONG only).
fn entry_values(r: &Reader<'_>, entry: usize, tag: u16) -> Result<Vec<u32>, String> {
    let field_type = r.u16_at(entry + 2)?;
    let count = r.u32_at(entry + 4)? as usize;
    let size = match field_type {
        3 => 2,
        4 => 4,
        other => {
            return Err(format!(
                "tag {tag}: field type {other}, expected SHORT or LONG"
            ))
        }
    };
    let total = count
        .checked_mul(size)
        .ok_or_else(|| format!("tag {tag}: count {count} overflows"))?;
    let base = if total <= 4 {
        entry + 8
    } else {
        r.u32_at(entry + 8)? as usize
    };
    (0..count)
        .map(|i| {
            let at = base + i * size;
            if size == 2 {
                r.u16_at(at).map(u32::from)
            } else {
                r.u32_at(at)
            }
        })
        .collect()
}

/// Parse a `dng_validate -2` stage-2 TIFF, refusing every shape it was not
/// built for.
pub fn read(bytes: &[u8]) -> Result<Stage2, String> {
    let little = match bytes.get(0..2) {
        Some(b"II") => true,
        Some(b"MM") => false,
        _ => return Err("not a TIFF: no II/MM byte-order mark".to_string()),
    };
    let r = Reader { bytes, little };
    if r.u16_at(2)? != 42 {
        return Err("not a classic TIFF: magic is not 42".to_string());
    }
    let ifd = r.u32_at(4)? as usize;
    let n = usize::from(r.u16_at(ifd)?);

    let mut width = None;
    let mut height = None;
    let mut bits = None;
    let mut compression = None;
    let mut samples_per_pixel = None;
    let mut planar = None;
    let mut offsets = None;
    let mut counts = None;
    for i in 0..n {
        let entry = ifd + 2 + 12 * i;
        let tag = r.u16_at(entry)?;
        let slot = match tag {
            256 => &mut width,
            257 => &mut height,
            258 => &mut bits,
            259 => &mut compression,
            277 => &mut samples_per_pixel,
            284 => &mut planar,
            273 => &mut offsets,
            279 => &mut counts,
            322..=325 => return Err(format!("tag {tag}: tiled TIFF, expected strips")),
            _ => continue,
        };
        *slot = Some(entry_values(&r, entry, tag)?);
    }

    let single = |v: Option<Vec<u32>>, tag: u16, name: &str| -> Result<u32, String> {
        match v.as_deref() {
            Some([x]) => Ok(*x),
            Some(other) => Err(format!(
                "{name} ({tag}): {} values, expected 1",
                other.len()
            )),
            None => Err(format!("{name} ({tag}): missing")),
        }
    };
    let width = single(width, 256, "ImageWidth")?;
    let height = single(height, 257, "ImageLength")?;
    let bits = single(bits, 258, "BitsPerSample")?;
    let compression = single(compression, 259, "Compression")?;
    let spp = single(samples_per_pixel, 277, "SamplesPerPixel")?;
    if bits != 16 {
        return Err(format!("BitsPerSample {bits}, expected 16"));
    }
    if compression != 1 {
        return Err(format!(
            "Compression {compression}, expected 1 (uncompressed)"
        ));
    }
    if spp != 1 {
        return Err(format!("SamplesPerPixel {spp}, expected 1"));
    }
    if let Some(p) = planar {
        let p = single(Some(p), 284, "PlanarConfiguration")?;
        if p != 1 {
            return Err(format!("PlanarConfiguration {p}, expected 1"));
        }
    }
    let offsets = offsets.ok_or("StripOffsets (273): missing")?;
    let counts = counts.ok_or("StripByteCounts (279): missing")?;
    if offsets.len() != counts.len() || offsets.is_empty() {
        return Err(format!(
            "{} StripOffsets against {} StripByteCounts",
            offsets.len(),
            counts.len()
        ));
    }

    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(2))
        .ok_or("image size overflows")?;
    let mut data = Vec::with_capacity(expected);
    for (&off, &len) in offsets.iter().zip(&counts) {
        data.extend_from_slice(r.slice(off as usize, len as usize)?);
    }
    if data.len() != expected {
        return Err(format!(
            "strips hold {} bytes, expected {width}×{height}×2 = {expected}",
            data.len()
        ));
    }
    let samples = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&b| {
            if little {
                u16::from_le_bytes(b)
            } else {
                u16::from_be_bytes(b)
            }
        })
        .collect();
    Ok(Stage2 {
        width,
        height,
        samples,
    })
}
