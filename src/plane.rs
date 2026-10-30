//! Strip location and sample unpack — `SPEC-012`, `DEC-008`.
//!
//! The sensor IFD's `StripOffsets`/`StripByteCounts` locate `N` contiguous
//! runs (strips) of packed samples — `N == 1` for every PROJ-001 corpus file,
//! `N == 12` for the Nikon D300S Bayer plane; this module assembles those
//! strips into a single linear `u16` plane, one sample per pixel, row-major,
//! **uncropped and un-normalised** — exactly the representation `dnglab analyze
//! --raw-pixel` compares against (`docs/oracle-contract.md`). Levels, crop and
//! orientation are `SPEC-014`.
//!
//! # Strips — `SPEC-028`
//!
//! `SPEC-012` scoped the unpacker single-strip-only; `SPEC-028` generalised it
//! to `N` strips, each `RowsPerStrip` rows (the last the remainder). The single
//! strip stays a degenerate one-iteration case, byte-for-byte unchanged. Each
//! strip is an **independent** bitstream padded to a byte boundary, so strips
//! are unpacked one at a time into their own row slice — never by concatenating
//! strip bytes, which for a sub-byte depth would read a strip's first sample
//! across the previous strip's tail padding.
//!
//! # Provenance
//!
//! TIFF 6.0 (1992) §2 "Compression" (`= 1`, uncompressed) plus `DEC-008`,
//! which is itself derived from TIFF 6.0's packing rule, not from any
//! implementation. `SPIKE-001`'s decoder is discarded and was not consulted
//! (`provenance-recorded-per-algorithm`); this was re-derived from the byte
//! evidence in `DEC-008` and the spec's own `## Implementation Context`,
//! cross-checked against `dnglab --raw-pixel`'s own plane on two independent
//! corpus frames (`docs/provenance-ledger.md`).
//!
//! # `DEC-008`'s two paths
//!
//! - **Sub-byte samples** (`bits_per_sample` not a multiple of 8 — 12 or 14 in
//!   this corpus): an MSB-first bit stream, read with [`BitReader`]. The TIFF
//!   byte-order tag does not apply; the packing is defined in bits, not
//!   bytes.
//! - **Byte-aligned samples** (a multiple of 8 — 8 or 16 here): plain
//!   integers in the container's own byte order. No bit cursor.
//!
//! Treating one as the other produces a plane that is wrong in a way that
//! still decodes, still has the right length, and still passes the layer-0
//! arithmetic — [`Error::SampleExceedsWhiteLevel`] is the free assertion that
//! caught it (`DEC-008`'s `Context`).
//!
//! # Allocation — `DEC-016`
//!
//! [`unpack_into`] takes the destination buffer as a caller-owned `&mut
//! [u16]` and allocates nothing itself, so it needs no allocator regardless
//! of how `DEC-002` (`no_std` + `alloc`, still `proposed`) is eventually
//! decided. See `DEC-016` for the alternative considered and rejected.

use crate::ifd::{ByteOrder, Sensor};
use crate::Error;

/// Bit widths `DEC-008`'s two paths are defined for. Anything else is
/// [`Error::UnsupportedBitDepth`] rather than a guess.
const SUPPORTED_BITS: [u32; 4] = [8, 12, 14, 16];

/// `u32::from(2).pow(n)` without the `pow`/shift operators
/// `clippy::arithmetic_side_effects` denies at the crate root — a small
/// match is exact, total, and needs no fallible arithmetic. `n` is always
/// `0..=8` in this module (a byte holds at most 8 unconsumed bits), so the
/// catch-all arm is never actually reached; it exists so the function has no
/// panicking path at all, matching `no-panics-on-untrusted-input`.
fn pow2(n: u32) -> u32 {
    match n {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        5 => 32,
        6 => 64,
        7 => 128,
        _ => 256,
    }
}

/// Resolve a TIFF-style `u32` offset into a `usize`, without `as`.
///
/// Mirrors `ifd::offset_to_usize`; kept as its own copy rather than exported
/// from `ifd` so this module's only dependency on `ifd` is the public
/// [`Sensor`]/[`ByteOrder`] types it already needs.
fn offset_to_usize(offset: u32) -> Result<usize, Error> {
    usize::try_from(offset).map_err(|_| Error::OffsetOutOfRange { offset })
}

// ─────────────────────────────────────────────────────────────────────────────
// The sub-byte path: an MSB-first bit stream
// ─────────────────────────────────────────────────────────────────────────────

/// Reads fixed-width, MSB-first, unscaled bit fields from a byte slice.
///
/// TIFF's packed sub-byte sample format: the strip is one continuous bit
/// stream, most-significant bit first, with no padding between samples and no
/// concept of byte order (`DEC-008`). Every read is bounds-checked and every
/// arithmetic step is `checked_*`/`saturating_*` — there is no path here that
/// can panic on a truncated or adversarial strip.
struct BitReader<'a> {
    data: &'a [u8],
    /// Byte currently being read from.
    byte_pos: usize,
    /// Bits already consumed from `data[byte_pos]`, counted from the MSB
    /// side. Always `0..8`.
    bit_in_byte: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> BitReader<'a> {
        BitReader {
            data,
            byte_pos: 0,
            bit_in_byte: 0,
        }
    }

    /// Read `bits` (1..=16 in practice; correct for any width `pow2` covers)
    /// MSB-first, as an unscaled integer.
    fn read(&mut self, bits: u32) -> Result<u32, Error> {
        let mut value: u32 = 0;
        let mut remaining = bits;

        while remaining > 0 {
            let byte = *self.data.get(self.byte_pos).ok_or(Error::Truncated {
                at: self.byte_pos,
                len: 1,
            })?;

            // Unconsumed bits in this byte occupy its low `bits_left` bits —
            // consuming from the MSB side is exactly `value mod 2^bits_left`.
            let bits_left = 8u32.saturating_sub(self.bit_in_byte);
            let take = remaining.min(bits_left);
            let in_byte = u32::from(byte).checked_rem(pow2(bits_left)).unwrap_or(0);
            // The top `take` bits of those `bits_left` remaining bits is
            // integer division by `2^(bits_left - take)`.
            let chunk = in_byte
                .checked_div(pow2(bits_left.saturating_sub(take)))
                .unwrap_or(0);

            // `checked_*` rather than `<<`/`+` (denied at the crate root).
            // Neither call can actually fail: `value` never exceeds 16 bits
            // and `bits` is validated against `SUPPORTED_BITS` before any
            // `BitReader` is built, so the accumulator never approaches
            // `u32::MAX`. `unwrap_or` (not `unwrap`) is the fallback for that
            // structurally-unreachable case.
            value = value
                .checked_mul(pow2(take))
                .and_then(|v| v.checked_add(chunk))
                .unwrap_or(0);

            let advanced = self.bit_in_byte.saturating_add(take);
            if advanced >= 8 {
                self.bit_in_byte = 0;
                self.byte_pos = self.byte_pos.checked_add(1).ok_or(Error::Truncated {
                    at: self.byte_pos,
                    len: 1,
                })?;
            } else {
                self.bit_in_byte = advanced;
            }
            remaining = remaining.saturating_sub(take);
        }

        Ok(value)
    }
}

fn unpack_bitstream(strip: &[u8], bits: u32, dst: &mut [u16]) -> Result<(), Error> {
    let mut reader = BitReader::new(strip);
    for out in dst.iter_mut() {
        let sample = reader.read(bits)?;
        // `bits <= 16` (SUPPORTED_BITS), so `sample` always fits `u16`;
        // `unwrap_or` rather than a fallible conversion keeps this path free
        // of a dead error arm.
        *out = u16::try_from(sample).unwrap_or(u16::MAX);
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// The byte-aligned path: plain integers in the container's byte order
// ─────────────────────────────────────────────────────────────────────────────

fn unpack_byte_aligned(
    strip: &[u8],
    bits: u32,
    byte_order: ByteOrder,
    dst: &mut [u16],
) -> Result<(), Error> {
    // `as_chunks` (not `chunks_exact`, which a newer clippy than this repo's
    // pinned Homebrew build flags as `chunks_exact_to_as_chunks` under
    // `-D warnings` — exactly the local/CI clippy-version gap `just lint-ci`
    // exists to catch, AGENTS.md §6) hands back `&[u8; N]` chunks directly,
    // so the byte-order conversion below takes no fallible `try_into` at all.
    match bits {
        8 => {
            let (chunks, _remainder) = strip.as_chunks::<1>();
            for (chunk, out) in chunks.iter().zip(dst.iter_mut()) {
                let [byte] = *chunk;
                *out = u16::from(byte);
            }
            Ok(())
        }
        16 => {
            let (chunks, _remainder) = strip.as_chunks::<2>();
            for (chunk, out) in chunks.iter().zip(dst.iter_mut()) {
                *out = match byte_order {
                    ByteOrder::Little => u16::from_le_bytes(*chunk),
                    ByteOrder::Big => u16::from_be_bytes(*chunk),
                };
            }
            Ok(())
        }
        other => Err(Error::UnsupportedBitDepth { bits: other }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Unpack the sensor plane's `N` strips into `dst`, one sample per pixel,
/// row-major, uncropped and un-normalised (`SPEC-012` + `SPEC-028`).
///
/// `dst` must hold exactly `sensor.width * sensor.height` samples — the
/// caller owns the buffer (`library-not-application`; `DEC-016`). `byte_order`
/// is the container's own (`Container::byte_order`); it governs the
/// byte-aligned path only (`DEC-008`) and has no meaning for the sub-byte
/// path. `file` is the whole input the sensor's tags were read from.
///
/// Strip `i` holds `rows_i = min(RowsPerStrip, height - i*RowsPerStrip)` rows
/// (the last strip is the remainder) and is unpacked independently into
/// `dst[i*RowsPerStrip*width ..][.. rows_i*width]`. `RowsPerStrip` is required
/// once `N > 1`; a single strip is the whole plane and consults none, so that
/// case is byte-for-byte the `SPEC-012` path.
///
/// # Errors
///
/// - [`Error::UnsupportedCompression`] if the plane is not `Compression = 1`.
/// - [`Error::UnsupportedBitDepth`] if `bits_per_sample` is not 8, 12, 14 or
///   16.
/// - [`Error::UnsupportedStripLayout`] if `StripOffsets` and `StripByteCounts`
///   disagree in count, or there are zero strips (tiled planes remain a
///   non-goal).
/// - [`Error::MissingRowsPerStrip`] if a multi-strip plane (`N > 1`) carries no
///   usable `RowsPerStrip`.
/// - [`Error::PackedSizeMismatch`] if any strip's `StripByteCounts` is not its
///   own `rows_i * width * bits_per_sample` rounded up to a byte, or the strips
///   do not cover the whole plane.
/// - [`Error::Truncated`] if a strip's declared offset/length runs past
///   `file`'s end.
/// - [`Error::PlaneBufferWrongLength`] if `dst.len() != width * height`.
/// - [`Error::SampleExceedsWhiteLevel`] if any decoded sample exceeds
///   `WhiteLevel` (only asserted when the sensor carries one) — the check
///   `DEC-008` names as the one that catches a mis-selected path.
pub fn unpack_into(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    dst: &mut [u16],
) -> Result<(), Error> {
    sensor.require_uncompressed()?;

    let bits = sensor.bits_per_sample;
    if !SUPPORTED_BITS.contains(&bits) {
        return Err(Error::UnsupportedBitDepth { bits });
    }

    let pixel_count = u64::from(sensor.width)
        .checked_mul(u64::from(sensor.height))
        .ok_or(Error::PlaneBufferWrongLength {
            expected: u64::MAX,
            actual: dst.len(),
        })?;
    let expected_len = usize::try_from(pixel_count).map_err(|_| Error::PlaneBufferWrongLength {
        expected: pixel_count,
        actual: dst.len(),
    })?;
    if dst.len() != expected_len {
        return Err(Error::PlaneBufferWrongLength {
            expected: pixel_count,
            actual: dst.len(),
        });
    }

    // N strips (SPEC-028), N >= 1. The two location tags must agree in count,
    // and a plane with no strips cannot be assembled — both are the genuine
    // UnsupportedStripLayout cases now that N > 1 is supported (tiles aside).
    let n = sensor.strip_offsets.len();
    if n == 0 || n != sensor.strip_byte_counts.len() {
        return Err(Error::UnsupportedStripLayout {
            strip_offsets: sensor.strip_offsets.len(),
            strip_byte_counts: sensor.strip_byte_counts.len(),
        });
    }

    // RowsPerStrip partitions a multi-strip plane into row bands. A single
    // strip IS the whole plane (SPEC-012's path) and consults no RowsPerStrip,
    // so N == 1 stays byte-for-byte the old behaviour; N > 1 requires a usable
    // value, and its absence (or a zero) is a typed error, not a guess.
    let rows_per_strip = if n == 1 {
        sensor.height
    } else {
        match sensor.rows_per_strip {
            Some(rows) if rows > 0 => rows,
            _ => return Err(Error::MissingRowsPerStrip { strips: n }),
        }
    };

    // Assemble strip by strip into disjoint, in-order row slices of `dst`. Each
    // strip is an INDEPENDENT bitstream padded to a byte boundary (DEC-008), so
    // it is unpacked from a fresh cursor into its own slice — never by
    // concatenating strip bytes, which for a sub-byte depth would read a
    // strip's first sample across the previous strip's tail padding (SPEC-028).
    let byte_aligned = bits.checked_rem(8) == Some(0);
    let mut rest: &mut [u16] = dst;
    let mut row_start: u32 = 0;
    for (strip_offset, strip_byte_count) in sensor
        .strip_offsets
        .iter()
        .zip(sensor.strip_byte_counts.iter())
    {
        // Rows in this strip: a full band, or the remainder for the last one.
        let remaining_rows = sensor.height.saturating_sub(row_start);
        let rows_in_strip = rows_per_strip.min(remaining_rows);
        let declared_bits = u64::from(*strip_byte_count).saturating_mul(8);

        // Per-strip layer-0 (AGENTS.md §12 bar 3, free): this strip's declared
        // byte count is exactly its own samples' bits rounded up to the next
        // byte. For 8/16-bit there is never padding; for 12/14-bit the final
        // byte of each strip independently carries up to 7 unused bits. This is
        // what caught SPIKE-002's byte-swap (which preserved total length).
        let strip_sample_bits = u64::from(rows_in_strip)
            .checked_mul(u64::from(sensor.width))
            .and_then(|samples| samples.checked_mul(u64::from(bits)))
            .ok_or(Error::PackedSizeMismatch {
                expected_bits: u64::MAX,
                strip_bits: declared_bits,
            })?;
        let expected_bytes = strip_sample_bits
            .checked_add(7)
            .and_then(|padded| padded.checked_div(8))
            .ok_or(Error::PackedSizeMismatch {
                expected_bits: strip_sample_bits,
                strip_bits: declared_bits,
            })?;
        if u64::from(*strip_byte_count) != expected_bytes {
            return Err(Error::PackedSizeMismatch {
                expected_bits: strip_sample_bits,
                strip_bits: declared_bits,
            });
        }

        // This strip's destination: `rows_in_strip * width` samples at the head
        // of the remaining buffer. split_at_mut_checked keeps the write
        // bounds-checked and panic-free, and proves the strips never overrun
        // the plane (an undershoot is caught after the loop).
        let samples_in_strip = u64::from(rows_in_strip)
            .checked_mul(u64::from(sensor.width))
            .and_then(|s| usize::try_from(s).ok())
            .ok_or(Error::PackedSizeMismatch {
                expected_bits: strip_sample_bits,
                strip_bits: declared_bits,
            })?;
        let (chunk, tail) =
            rest.split_at_mut_checked(samples_in_strip)
                .ok_or(Error::PackedSizeMismatch {
                    expected_bits: strip_sample_bits,
                    strip_bits: declared_bits,
                })?;
        rest = tail;

        // Locate this strip's bytes, bounds-checked into Truncated.
        let at = offset_to_usize(*strip_offset)?;
        let len =
            usize::try_from(*strip_byte_count).map_err(|_| Error::Truncated { at, len: 0 })?;
        let end = at.checked_add(len).ok_or(Error::Truncated { at, len })?;
        let strip = file.get(at..end).ok_or(Error::Truncated { at, len })?;

        if byte_aligned {
            unpack_byte_aligned(strip, bits, byte_order, chunk)?;
        } else {
            unpack_bitstream(strip, bits, chunk)?;
        }

        row_start = row_start.saturating_add(rows_in_strip);
    }

    // Full coverage: the strips together wrote every row. A plane whose strips
    // do not sum to its height leaves `rest` non-empty — a strip count that
    // disagrees with RowsPerStrip/height, reported here rather than returning a
    // partially-filled plane. (The per-strip split above rules out an overrun,
    // so this only catches an undershoot.)
    if !rest.is_empty() {
        let covered_bits = u64::from(row_start)
            .saturating_mul(u64::from(sensor.width))
            .saturating_mul(u64::from(bits));
        return Err(Error::PackedSizeMismatch {
            expected_bits: sensor.packed_bits()?,
            strip_bits: covered_bits,
        });
    }

    // The free assertion DEC-008 names: max > WhiteLevel is impossible for a
    // correctly-unpacked linear RAW plane, and it is unconditional, not a
    // debug assertion — it is what caught SPIKE-002's byte-swap, which had
    // already passed the layer-0 check above.
    if let Some(white_level) = sensor.white_level {
        for (index, sample) in dst.iter().enumerate() {
            if u32::from(*sample) > white_level {
                return Err(Error::SampleExceedsWhiteLevel {
                    index,
                    sample: *sample,
                    white_level,
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    #[test]
    fn pow2_matches_the_naive_definition() {
        for n in 0..=8u32 {
            assert_eq!(pow2(n), 2u32.pow(n));
        }
    }

    #[test]
    fn bit_reader_reads_the_measured_q2m_samples() {
        // DEC-008 / SPEC-012 Implementation Context, L1021223.DNG's strip head.
        let strip: [u8; 16] = [
            0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
            0x0b, 0xdc,
        ];
        let mut reader = BitReader::new(&strip);
        let expected = [746u32, 725, 711, 752, 646, 705, 772, 686];
        for want in expected {
            assert_eq!(reader.read(14).unwrap(), want);
        }
    }

    #[test]
    fn bit_reader_reads_a_byte_aligned_width_trivially() {
        let strip: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let mut reader = BitReader::new(&strip);
        for want in [1u32, 2, 3, 4] {
            assert_eq!(reader.read(8).unwrap(), want);
        }
    }
}
