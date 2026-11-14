//! Strip and tile location and sample unpack — `SPEC-012`, `DEC-008`,
//! `SPEC-058`.
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
//! # Tiles — `SPEC-058`, `DEC-056`
//!
//! A plane carrying TIFF 6.0 §15 tile tags (`TileWidth`, `TileLength`,
//! `TileOffsets`, `TileByteCounts`) instead of strip tags is read tile by tile,
//! row-major, each tile written straight into the rectangle it covers in `dst`.
//! Edge tiles are coded at full tile size; the part past the plane's right or
//! bottom edge is never written:
//!
//! - **Uncompressed tiles** unpack row by row, each tile row from its own byte
//!   slice (a tile row must be a whole number of bytes, else
//!   [`Error::UnsupportedTileRowBits`]), keeping only the visible samples.
//!   `DEC-008`'s two paths apply per tile row.
//! - **Compression-7 tiles** reconstruct through a clipped, strided window
//!   (`ljpeg::Window`): visible samples are written in place, samples right of
//!   the plane have their entropy-coded difference read and dropped, and the
//!   decode stops after the tile's last visible row. Each tile's SOF-3 frame
//!   shape is bound to the tile, not the plane (`DEC-047`).
//!
//! Nothing is allocated for either (`DEC-016` holds). The tile table is checked
//! by the same shared pass [`validate`] runs (`DEC-050`): a plane carrying both
//! layouts is [`Error::StripsAndTiles`], a table that does not match the tile
//! grid is [`Error::InvalidTileLayout`].
//!
//! # Units — `SPEC-068`, `DEC-063`
//!
//! [`units`] cuts the plane into **full-width bands of whole rows** that decode
//! independently, and [`unpack_unit_into`] decodes one into its own band. A
//! band is a contiguous run of the plane, so a caller splits its buffer with
//! `split_at_mut` and decodes the bands on its own threads, in any order, with
//! no `unsafe`; the result equals [`unpack_into`] byte for byte, because
//! [`unpack_into`] iterates the same units through the same kernel. irradiance
//! itself starts no thread.
//!
//! - **An uncompressed strip whose rows are whole bytes** splits into bands of
//!   at most `max_band_rows` rows. A band never crosses a strip: each strip is
//!   its own byte-padded bitstream.
//! - **An uncompressed strip whose rows are not whole bytes**, and **a
//!   Compression-7 strip**, are one unit each: neither can be entered mid-way.
//! - **A tiled plane** is one unit per tile row: two tiles of one tile row
//!   share plane rows, so they cannot be two disjoint bands.
//!
//! ```
//! # fn tiny_dng() -> Vec<u8> {
//! #     // A 4 × 6, 16-bit, one-strip LinearRaw plane, little-endian.
//! #     let entries: [(u16, u16, u32); 9] = [
//! #         (254, 4, 0), (256, 4, 4), (257, 4, 6), (258, 3, 16), (259, 3, 1),
//! #         (262, 3, 34892), (277, 3, 1), (273, 4, 512), (279, 4, 48),
//! #     ];
//! #     let mut f = b"II*\0\x08\0\0\0".to_vec();
//! #     f.extend_from_slice(&9u16.to_le_bytes());
//! #     for (tag, ty, value) in entries {
//! #         f.extend_from_slice(&tag.to_le_bytes());
//! #         f.extend_from_slice(&ty.to_le_bytes());
//! #         f.extend_from_slice(&1u32.to_le_bytes());
//! #         f.extend_from_slice(&value.to_le_bytes());
//! #     }
//! #     f.extend_from_slice(&0u32.to_le_bytes());
//! #     f.resize(512, 0);
//! #     f.extend((0u16..24).flat_map(|s| s.to_le_bytes()));
//! #     f
//! # }
//! # fn main() -> Result<(), irradiance::Error> {
//! use std::num::NonZeroU32;
//!
//! use irradiance::ifd::Container;
//! use irradiance::plane::{unpack_into, unpack_unit_into, units, validate};
//!
//! let file = tiny_dng();
//! let container = Container::parse(&file)?;
//! let sensor = container.sensor()?;
//! let order = container.byte_order();
//! let n = validate(&sensor, order, &file, usize::MAX)?;
//! let mut plane = vec![0u16; n];
//!
//! // Carve the plane into each unit's band; decode the bands on scoped threads.
//! let max_band_rows = NonZeroU32::new(2).expect("non-zero");
//! std::thread::scope(|scope| -> Result<(), irradiance::Error> {
//!     let mut rest: &mut [u16] = &mut plane;
//!     let mut workers = Vec::new();
//!     for unit in units(&sensor, &file, max_band_rows)? {
//!         let len = unit.samples(sensor.width).expect("the band fits usize");
//!         let (band, tail) = rest.split_at_mut(len);
//!         rest = tail;
//!         let (sensor, file) = (&sensor, &file);
//!         workers.push(scope.spawn(move || unpack_unit_into(sensor, order, file, &unit, band)));
//!     }
//!     for worker in workers {
//!         worker.join().expect("a worker panicked")?;
//!     }
//!     Ok(())
//! })?;
//!
//! let mut whole = vec![0u16; n];
//! unpack_into(&sensor, order, &file, &mut whole)?;
//! assert_eq!(plane, whole);
//! # Ok(())
//! # }
//! ```
//!
//! # Provenance
//!
//! Tiles: TIFF 6.0 §15, the Adobe DNG specification 1.7, and ITU-T T.81 Annex
//! H (the lossless difference coding is independent of the prediction, which is
//! what lets a clipped column be skipped) — `DEC-056`; cross-checked bit-exact
//! against `dnglab --raw-checksum` on three tiled Pixel phone DNGs.
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
//!   this corpus): an MSB-first bit stream, read with `BitReader`. The TIFF
//!   byte-order tag does not apply; the packing is defined in bits, not
//!   bytes.
//! - **Byte-aligned samples** (a multiple of 8 — 8 or 16 here): plain
//!   integers in the container's own byte order. No bit cursor.
//!
//! Treating one as the other produces a plane that is wrong in a way that
//! still decodes, still has the right length, and still passes the layer-0
//! arithmetic. `DEC-008`'s `Context` records that a `max <= WhiteLevel`
//! assertion caught that byte-swap in `SPIKE-002`; `DEC-029` retired it (a
//! sample above `WhiteLevel` is legal — the Canon 5D-III clips one), so that
//! byte-swap now rests on the bit-exact oracle (`SPEC-025`) plus the per-strip
//! layer-0 packed-bits check.
//!
//! # Allocation — `DEC-016`
//!
//! [`unpack_into`] and [`unpack_unit_into`] take the destination buffer as a
//! caller-owned `&mut [u16]`, and [`units`] holds no buffer, so this module
//! allocates nothing itself and needs no allocator regardless of how
//! `DEC-002` (`no_std` + `alloc`, still `proposed`) is eventually decided. See
//! `DEC-016` for the alternative considered and rejected.

use core::num::NonZeroU32;

use crate::ifd::{ByteOrder, Compression, Sensor};
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
// The preflight — `SPEC-039`, `DEC-050`
// ─────────────────────────────────────────────────────────────────────────────

/// Check, **before the caller allocates anything**, that the sensor plane can
/// be unpacked into a buffer of the returned length, and that the length is
/// within the caller's own pixel budget (`SPEC-039`, `DEC-050`).
///
/// Returns `Ok(n)` with `n = width · height`: the exact `dst.len()`
/// [`unpack_into`] wants. Allocation-free, panic-free, reads **no** strip
/// bytes and writes nothing; it runs the same check helpers [`unpack_into`]
/// runs before it touches `dst`, so the two cannot drift. Call it, allocate
/// `n` samples, then call [`unpack_into`].
///
/// # The pixel budget
///
/// `max_pixels` is **the caller's** number, not a library cap; pass
/// `usize::MAX` for no limit. It is checked **first**, on both paths, from
/// `width · height` computed as a `u64` — before any per-strip work and before
/// any conversion to `usize` — so a forged `ImageWidth`/`ImageLength` is
/// refused with [`Error::PlaneTooLarge`] and nothing proportional to the claim
/// is ever allocated. The budget is `validate`-only: [`unpack_into`] is handed
/// a buffer the caller already sized.
///
/// # What `Ok(n)` guarantees
///
/// - **Uncompressed (`Compression = 1`):** [`unpack_into`] with
///   `dst.len() == n` returns `Ok(())` — it cannot fail on layout, size,
///   truncation, precision or compression support. The sum of
///   `StripByteCounts` is also bounded by `file.len()`
///   ([`Error::StripBytesExceedFile`]), so `n · bits_per_sample ≤ 8 ·
///   file.len()`: the plane is bounded by the bytes actually supplied. A tiled
///   plane gets the same bound from its `TileByteCounts` (padding included).
/// - **Compression-7 (SOF-3 lossless JPEG):** the pre-decode structural checks
///   pass — strip layout, `RowsPerStrip`, full row coverage and each strip's
///   in-file offset/count bounds (for a tiled plane: the tile grid and each
///   tile's in-file bounds). `n` is bounded **only by `max_pixels`**, not
///   by the file length (a few hundred compressed bytes legitimately expand to
///   a multi-megapixel plane). `validate` does not decode, so the strip's SOF-3
///   frame-shape binding ([`Error::LjpegPlaneShapeMismatch`], `DEC-047`) and
///   the sample decode itself (the `Ljpeg*` / [`Error::Truncated`] family)
///   are still enforced by [`unpack_into`] at decode time.
///
/// The develop-side buffers need no separate preflight: `demosaic_into` and
/// `to_linear_f32_into` write a fixed `3 · n` samples (`3 × 2` bytes per pixel
/// for the `u16` RGB plane, `3 × 4` bytes for the `f32` one), so `n` is the only
/// pre-allocation number a caller needs.
///
/// `byte_order` is accepted for signature parity with [`unpack_into`]; it
/// governs only the byte-aligned sample decode (`DEC-008`) and does not affect
/// validation.
///
/// # Errors
///
/// - [`Error::PlaneTooLarge`] if `width · height > max_pixels` (checked first).
/// - [`Error::StripBytesExceedFile`] if an uncompressed plane's
///   `StripByteCounts` sum to more than `file.len()` — strips that must alias
///   the same bytes. Non-overlapping strips always satisfy it.
/// - Every pre-decode error [`unpack_into`] documents, as the **same variant**:
///   [`Error::UnsupportedCompression`], [`Error::UnsupportedBitDepth`],
///   [`Error::UnsupportedStripLayout`], [`Error::MissingRowsPerStrip`],
///   [`Error::PackedSizeMismatch`], [`Error::Truncated`] (a strip or tile past
///   the end of `file`), and for a tiled plane [`Error::StripsAndTiles`],
///   [`Error::InvalidTileLayout`], [`Error::UnsupportedTileRowBits`].
pub fn validate(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    max_pixels: usize,
) -> Result<usize, Error> {
    // Parity only (see the rustdoc): validation never depends on byte order.
    let _ = byte_order;

    // 1. The caller's budget, FIRST (DEC-050 §2): the u64 product of two u32s
    //    cannot overflow, and nothing below runs on an over-budget claim.
    let pixels = pixel_product(sensor);
    let budget = u64::try_from(max_pixels).unwrap_or(u64::MAX);
    if pixels > budget {
        return Err(Error::PlaneTooLarge {
            pixels,
            limit: max_pixels,
        });
    }

    // 2. Compression recognised, and (uncompressed) a supported bit depth.
    let kind = plane_kind(sensor)?;

    // 3. `pixels <= max_pixels`, so it fits `usize`; the error arm is the
    //    same budget refusal, never a new failure mode.
    let plane_samples = usize::try_from(pixels).map_err(|_| Error::PlaneTooLarge {
        pixels,
        limit: max_pixels,
    })?;

    // 4. The full layout pass shared with `unpack_into`: strips, or (SPEC-058)
    //    the tile table. Reads no strip or tile byte.
    check_layout(sensor, kind, plane_samples, file)?;
    Ok(plane_samples)
}

// ─────────────────────────────────────────────────────────────────────────────
// The entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Unpack the sensor plane's `N` strips — or its tiles (`SPEC-058`) — into
/// `dst`, one sample per pixel, row-major, uncropped and un-normalised
/// (`SPEC-012` + `SPEC-028`).
///
/// Dispatches on `sensor.compression` (`SPEC-043`, `DEC-012`):
/// `Compression::Uncompressed` takes `DEC-008`'s sub-byte / byte-aligned path
/// unchanged; `Compression::Jpeg` (DNG `Compression 7`, SOF-3 lossless JPEG)
/// routes each strip through the crate's SOF-3 decoder; any other code is the
/// clean typed [`Error::UnsupportedCompression`] rejection it has always been.
///
/// `dst` must hold exactly `sensor.width * sensor.height` samples — the
/// caller owns the buffer (`library-not-application`; `DEC-016`). Size it with
/// [`validate`] first: that is the pre-allocation check, with the caller's
/// pixel budget, that this function cannot perform (it is handed a buffer
/// already allocated). `byte_order` is the container's own
/// (`Container::byte_order`); it governs the byte-aligned uncompressed path
/// only (`DEC-008`) and has no meaning for the sub-byte path or the
/// Compression-7 path (an SOF-3 datastream is self-describing big-endian).
/// `file` is the whole input the sensor's tags were read from.
///
/// Strip `i` holds `rows_i = min(RowsPerStrip, height - i*RowsPerStrip)` rows
/// (the last strip is the remainder) and is unpacked independently into
/// `dst[i*RowsPerStrip*width ..][.. rows_i*width]`. `RowsPerStrip` is required
/// once `N > 1`; a single strip is the whole plane and consults none, so that
/// case is byte-for-byte the `SPEC-012` path.
///
/// # Errors
///
/// - [`Error::UnsupportedCompression`] if the plane is neither `Compression = 1`
///   (uncompressed) nor `Compression = 7` (SOF-3 lossless JPEG).
/// - [`Error::UnsupportedBitDepth`] if `bits_per_sample` is not 8, 12, 14 or
///   16 (uncompressed path only — a Compression-7 plane's precision is the
///   SOF-3 frame's own and is read by the decoder).
/// - [`Error::PlaneBufferWrongLength`] if `dst.len() != width * height`. This
///   is checked before the strip layout, so a wrong-length `dst` wins over a
///   bad layout.
/// - [`Error::UnsupportedStripLayout`] if `StripOffsets` and `StripByteCounts`
///   disagree in count, or there are zero strips and no tile tags; for a
///   Compression-7 plane, also if the strips do not cover the whole plane.
/// - Tiled planes (`SPEC-058`, `DEC-056`): [`Error::StripsAndTiles`] if the
///   plane carries both layouts; [`Error::InvalidTileLayout`] if a tile
///   dimension is absent or zero, or `TileOffsets`/`TileByteCounts` do not
///   both hold `ceil(width/TileWidth) × ceil(height/TileLength)` entries;
///   [`Error::UnsupportedTileRowBits`] (uncompressed) if a tile row is not a
///   whole number of bytes; [`Error::PackedSizeMismatch`] (uncompressed) if a
///   tile's byte count is not its full packed size; [`Error::Truncated`] if a
///   tile runs past `file`; [`Error::StripBytesExceedFile`] (uncompressed) if
///   the tile byte counts sum past `file.len()`; and for a Compression-7 tile,
///   [`Error::LjpegPlaneShapeMismatch`] if its frame is not `TileWidth ×
///   TileLength`.
/// - [`Error::MissingRowsPerStrip`] if a multi-strip plane (`N > 1`) carries no
///   usable `RowsPerStrip`.
/// - [`Error::PackedSizeMismatch`] if any strip's `StripByteCounts` is not its
///   own `rows_i * width * bits_per_sample` rounded up to a byte, or the strips
///   do not cover the whole plane (uncompressed only).
/// - [`Error::Truncated`] if a strip's declared offset/length runs past
///   `file`'s end.
/// - [`Error::StripBytesExceedFile`] if an uncompressed plane's
///   `StripByteCounts` sum to more than `file.len()` — strips that alias the
///   same bytes (`SPEC-039`; the one tightening this function gained, and only
///   a forged file trips it).
/// - For a `Compression = 7` plane, after the structural checks above pass for
///   **every** strip: [`Error::LjpegPlaneShapeMismatch`] when a strip's SOF-3
///   frame shape disagrees with the strip's declared geometry (`DEC-047`), and
///   any typed error from the SOF-3 decoder — the `Ljpeg*` /
///   [`Error::Truncated`] family on a malformed, subsampled, or truncated strip
///   — never a panic.
///
/// A decoded sample above `WhiteLevel` is **not** an error (`DEC-029`, amending
/// `DEC-008`): specular highlights legitimately exceed the nominal white point,
/// and the plane preserves raw values unchanged — clamping to `WhiteLevel` is
/// develop's saturation step, not unpack's.
pub fn unpack_into(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    dst: &mut [u16],
) -> Result<(), Error> {
    // The precedence `SPEC-039` must preserve (Design §5): compression → bit
    // depth → pixel count → `dst.len()` → layout (strips, or SPEC-058's tiles). `validate` shares every
    // helper below except the `dst` check (it has no `dst`) and adds the
    // caller budget ahead of them all.
    let kind = plane_kind(sensor)?;

    let pixel_count = pixel_product(sensor);
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

    // The same full layout pass `validate` runs. After it, nothing structural
    // can fail below: the uncompressed write cannot fail at all, and the
    // Compression-7 loop can fail only in the decode itself.
    let layout = check_layout(sensor, kind, expected_len, file)?;

    // SPEC-068 / DEC-063 §3: the plane is the units `units` yields at the
    // largest band — one per strip or tile row, so today's traversal exactly:
    // strips in order, tiles in table (tile-row by tile-row) order — each
    // carved off the head of `dst` in turn and decoded by the same private
    // kernel `unpack_unit_into` calls. One decode path, so the whole-plane
    // call and the units cannot drift. `check_layout` is not re-run per unit.
    let plan = Plan::new(sensor, kind, layout)?;
    let mut rest: &mut [u16] = dst;
    for unit in Units::new(sensor, plan, u32::MAX) {
        let samples = unit
            .samples(sensor.width)
            .ok_or(Error::PlaneBufferWrongLength {
                expected: pixel_count,
                actual: expected_len,
            })?;
        let (band, tail) = carve(rest, samples)?;
        rest = tail;
        decode_unit(sensor, plan, kind, byte_order, file, &unit, band)?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Units — `SPEC-068`, `DEC-063`
// ─────────────────────────────────────────────────────────────────────────────

/// One independently decodable piece of the sensor plane: a **full-width band
/// of whole rows**, `row_start .. row_start + row_count` (`DEC-063`).
///
/// Its output is the contiguous run `dst[row_start · width ..][.. row_count ·
/// width]` of the plane, so units split a caller's buffer with `split_at_mut`
/// and decode on the caller's own threads with no `unsafe`. Only
/// [`units`] makes one: the struct is `#[non_exhaustive]` and carries a
/// private source (which strip or tile row, and the band's offset inside a
/// strip), so it cannot be built outside the crate:
///
/// ```compile_fail
/// let unit = irradiance::plane::Unit { row_start: 0, row_count: 1 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    /// The band's first plane row.
    pub row_start: u32,
    /// The band's row count.
    pub row_count: u32,
    /// Where the band's bytes come from. Private, so a unit can only come
    /// from [`units`], and so new kinds can be added later.
    source: UnitSource,
}

/// Which part of the plane's layout a [`Unit`] decodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitSource {
    /// Rows `row_in_strip ..` of one uncompressed strip whose rows are whole
    /// bytes.
    StripBand {
        /// Index into `StripOffsets`.
        strip: usize,
        /// The band's first row, counted from the strip's first row.
        row_in_strip: u32,
    },
    /// A whole strip: a Compression-7 strip, or an uncompressed one whose rows
    /// are not whole bytes.
    Strip {
        /// Index into `StripOffsets`.
        strip: usize,
    },
    /// Every tile of one tile row, left to right.
    TileRow {
        /// The tile row, `0 .. ceil(height / TileLength)`.
        tile_row: u32,
    },
}

impl Unit {
    /// `row_count · width`: the length of the band [`unpack_unit_into`]
    /// writes, or `None` if it does not fit `usize`.
    #[must_use]
    pub fn samples(&self, width: u32) -> Option<usize> {
        u64::from(self.row_count)
            .checked_mul(u64::from(width))
            .and_then(|n| usize::try_from(n).ok())
    }
}

/// The unit layout of a plane whose [`check_layout`] has passed — the numbers
/// [`Units`] walks and [`decode_unit`] reads, nothing proportional to the
/// plane.
#[derive(Debug, Clone, Copy)]
enum Plan {
    /// A strip plane.
    Strips {
        /// Number of strips.
        strips: usize,
        /// Rows per strip (the last strip holds the remainder).
        rows_per_strip: u32,
        /// Uncompressed with `width · bits` a whole number of bytes: the strip
        /// splits into bands. Otherwise each strip is one unit.
        whole_byte_rows: bool,
    },
    /// A tiled plane: one unit per tile row.
    Tiles {
        /// The checked tile grid.
        grid: TileGrid,
        /// `ceil(height / TileLength)`.
        tile_rows: u32,
    },
}

impl Plan {
    /// The plan for a `layout` that [`check_layout`] has accepted.
    fn new(sensor: &Sensor, kind: PlaneKind, layout: Layout) -> Result<Plan, Error> {
        match layout {
            Layout::Strips => {
                let walk = StripWalk::new(sensor, kind, 0)?;
                let whole_byte_rows = match kind {
                    PlaneKind::Uncompressed { bits } => {
                        u64::from(sensor.width)
                            .saturating_mul(u64::from(bits))
                            .checked_rem(8)
                            == Some(0)
                    }
                    PlaneKind::Jpeg => false,
                };
                Ok(Plan::Strips {
                    strips: walk.strips,
                    rows_per_strip: walk.rows_per_strip,
                    whole_byte_rows,
                })
            }
            // A zero-width tiled plane has no tile columns, so its tile table
            // is empty (`tile_grid` checked `across · down` entries) and it
            // has no tile rows to decode. Yielding its `ceil(height / TL)`
            // empty tile rows anyway would make `unpack_into` walk up to a
            // billion of them where it used to walk zero tiles.
            Layout::Tiles(grid) => Ok(Plan::Tiles {
                grid,
                tile_rows: if grid.across == 0 {
                    0
                } else {
                    ceil_div(sensor.height, grid.tile_length).unwrap_or(0)
                },
            }),
        }
    }

    /// Whether `unit` is a unit this plan can yield at SOME band size: its
    /// source exists, and its rows are rows that source holds — any non-empty
    /// run of rows inside a whole-byte strip, a strip's exact rows, or a tile
    /// row's exact visible rows (`DEC-063` §2).
    fn yields(self, height: u32, unit: &Unit) -> bool {
        match (self, unit.source) {
            (
                Plan::Strips {
                    strips,
                    rows_per_strip,
                    whole_byte_rows: true,
                },
                UnitSource::StripBand {
                    strip,
                    row_in_strip,
                },
            ) => {
                let (first, rows) = strip_rows(height, rows_per_strip, strip);
                strip < strips
                    && unit.row_count >= 1
                    && row_in_strip
                        .checked_add(unit.row_count)
                        .is_some_and(|end| end <= rows)
                    && first.checked_add(row_in_strip) == Some(unit.row_start)
            }
            (
                Plan::Strips {
                    strips,
                    rows_per_strip,
                    whole_byte_rows: false,
                },
                UnitSource::Strip { strip },
            ) => {
                strip < strips
                    && strip_rows(height, rows_per_strip, strip) == (unit.row_start, unit.row_count)
            }
            (Plan::Tiles { grid, tile_rows }, UnitSource::TileRow { tile_row }) => {
                tile_row < tile_rows
                    && strip_rows(
                        height,
                        grid.tile_length,
                        usize::try_from(tile_row).unwrap_or(usize::MAX),
                    ) == (unit.row_start, unit.row_count)
            }
            _ => false,
        }
    }
}

/// `(first_row, rows)` of row band `index` when the plane is cut into bands of
/// `band` rows (a strip of `RowsPerStrip`, or a tile row of `TileLength`): the
/// last band holds the remainder, and bands past the plane hold zero rows at
/// `height`. Exactly the rows [`StripWalk`] assigns strip `index`.
fn strip_rows(height: u32, band: u32, index: usize) -> (u32, u32) {
    let first = u64::try_from(index)
        .unwrap_or(u64::MAX)
        .saturating_mul(u64::from(band))
        .min(u64::from(height));
    let first = u32::try_from(first).unwrap_or(height);
    (first, band.min(height.saturating_sub(first)))
}

/// `ceil(rows / band)` for `band > 0`, as a `u64`.
fn bands_in(rows: u32, band: u32) -> u64 {
    u64::from(ceil_div(rows, band).unwrap_or(0))
}

/// The units of one plane, in plane order — see [`units`]. An
/// [`ExactSizeIterator`] that walks the strip or tile tables lazily and holds
/// no buffer.
#[derive(Debug, Clone)]
pub struct Units<'a> {
    /// The plane's geometry (`height` bounds every band).
    sensor: &'a Sensor,
    /// The checked layout.
    plan: Plan,
    /// `max_band_rows`.
    band_rows: u32,
    /// The next strip or tile row.
    index: usize,
    /// The next band's first row inside strip `index`.
    row_in_strip: u32,
    /// Units not yet yielded.
    remaining: usize,
}

impl<'a> Units<'a> {
    /// The units of `plan`, `band_rows >= 1`. `len()` is computed here, from
    /// the plan's numbers alone.
    fn new(sensor: &'a Sensor, plan: Plan, band_rows: u32) -> Units<'a> {
        let height = sensor.height;
        let count: u64 = match plan {
            Plan::Strips { strips: 0, .. } => 0,
            Plan::Strips {
                strips,
                whole_byte_rows: false,
                ..
            } => u64::try_from(strips).unwrap_or(u64::MAX),
            Plan::Strips {
                strips,
                rows_per_strip,
                whole_byte_rows: true,
            } => {
                // Strips holding a full `rows_per_strip`, then at most one
                // holding the remainder; any strip after that holds no rows
                // and yields no band.
                let strips = u64::try_from(strips).unwrap_or(u64::MAX);
                let full = u64::from(height.checked_div(rows_per_strip).unwrap_or(0)).min(strips);
                let rest_rows = height.checked_rem(rows_per_strip).unwrap_or(0);
                let rest = if full < strips {
                    bands_in(rest_rows, band_rows)
                } else {
                    0
                };
                full.saturating_mul(bands_in(rows_per_strip, band_rows))
                    .saturating_add(rest)
            }
            Plan::Tiles { tile_rows, .. } => u64::from(tile_rows),
        };
        Units {
            sensor,
            plan,
            band_rows,
            index: 0,
            row_in_strip: 0,
            remaining: usize::try_from(count).unwrap_or(usize::MAX),
        }
    }
}

impl Iterator for Units<'_> {
    type Item = Unit;

    fn next(&mut self) -> Option<Unit> {
        let height = self.sensor.height;
        let unit = match self.plan {
            Plan::Strips {
                strips,
                rows_per_strip,
                whole_byte_rows,
            } => loop {
                if self.index >= strips {
                    return None;
                }
                let strip = self.index;
                let (first, rows) = strip_rows(height, rows_per_strip, strip);
                if !whole_byte_rows {
                    self.index = strip.saturating_add(1);
                    break Unit {
                        row_start: first,
                        row_count: rows,
                        source: UnitSource::Strip { strip },
                    };
                }
                let row_in_strip = self.row_in_strip;
                if row_in_strip < rows {
                    // A band never crosses a strip: it ends at the band size
                    // or the strip's last row, whichever comes first.
                    let row_count = rows.saturating_sub(row_in_strip).min(self.band_rows);
                    self.row_in_strip = row_in_strip.saturating_add(row_count);
                    break Unit {
                        row_start: first.saturating_add(row_in_strip),
                        row_count,
                        source: UnitSource::StripBand {
                            strip,
                            row_in_strip,
                        },
                    };
                }
                self.index = strip.saturating_add(1);
                self.row_in_strip = 0;
            },
            Plan::Tiles { grid, tile_rows } => {
                let index = self.index;
                let tile_row = u32::try_from(index).ok().filter(|r| *r < tile_rows)?;
                self.index = index.saturating_add(1);
                let (row_start, row_count) = strip_rows(height, grid.tile_length, index);
                Unit {
                    row_start,
                    row_count,
                    source: UnitSource::TileRow { tile_row },
                }
            }
        };
        self.remaining = self.remaining.saturating_sub(1);
        Some(unit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for Units<'_> {}

/// The plane's kind and unit plan, after the checks [`unpack_into`] and
/// [`validate`] share, in their order: compression and depth, the pixel count
/// fits `usize`, then the full layout pass.
fn plan_for(sensor: &Sensor, file: &[u8]) -> Result<(PlaneKind, Plan), Error> {
    let kind = plane_kind(sensor)?;
    // Unreachable on a 64-bit target (two `u32`s); on a narrower one, the
    // same refusal `validate` gives with no budget.
    let pixels = pixel_product(sensor);
    let plane_samples = usize::try_from(pixels).map_err(|_| Error::PlaneTooLarge {
        pixels,
        limit: usize::MAX,
    })?;
    let layout = check_layout(sensor, kind, plane_samples, file)?;
    Ok((kind, Plan::new(sensor, kind, layout)?))
}

/// The plane's units: full-width bands of whole rows that decode
/// independently, in plane order, together covering every row exactly once
/// (`SPEC-068`, `DEC-063`).
///
/// | Layout | Unit |
/// |---|---|
/// | Uncompressed strip, rows of whole bytes (`width · bits % 8 == 0`) | a band of at most `max_band_rows` rows **inside one strip** |
/// | Uncompressed strip, rows not whole bytes | the whole strip |
/// | Uncompressed or Compression-7 tiles | one tile row (the last one clipped) |
/// | Compression-7 strip | the whole strip (a scan without restart markers cannot be entered mid-way) |
///
/// Only the bands depend on `max_band_rows`; with `max_band_rows >= height`
/// the units are exactly one per strip or per tile row. A zero-width tiled
/// plane has no tiles, so it has no units (there is nothing to write). Decode each with
/// [`unpack_unit_into`] into its own `row_count · width` band of the plane,
/// in any order, on any thread: the assembled plane equals [`unpack_into`]'s
/// byte for byte (which iterates these same units).
///
/// Runs the same full layout check [`validate`] and [`unpack_into`] share
/// before yielding anything. Allocation-free; reads no strip or tile byte.
///
/// # Errors
///
/// The pre-decode errors [`unpack_into`] documents, as the **same variant**
/// and in the same order — except [`Error::PlaneBufferWrongLength`], since
/// there is no buffer here.
pub fn units<'a>(
    sensor: &'a Sensor,
    file: &[u8],
    max_band_rows: NonZeroU32,
) -> Result<Units<'a>, Error> {
    let (_, plan) = plan_for(sensor, file)?;
    Ok(Units::new(sensor, plan, max_band_rows.get()))
}

/// Decode one [`Unit`] of the plane into `dst`, which holds exactly that
/// unit's band: `unit.row_count · width` samples, row-major (`SPEC-068`,
/// `DEC-063`). Plane row `unit.row_start + r` lands in `dst[r · width ..]`.
///
/// Every sample comes from the same decode, from the same bytes, as
/// [`unpack_into`]'s, so units decoded in any order into their bands of one
/// plane equal [`unpack_into`] byte for byte. `byte_order` and `file` are as
/// for [`unpack_into`]. Allocation-free, except the small Huffman-table parse
/// a Compression-7 strip or tile has always made.
///
/// # Errors
///
/// In this order, so nothing is written on any refusal:
///
/// - the pre-decode errors [`units`] returns for this sensor;
/// - [`Error::PlaneUnitMismatch`] if `unit` is not a unit of this sensor's
///   plane — say, one taken from another sensor's [`units`];
/// - [`Error::PlaneBufferWrongLength`] with `expected = row_count · width` if
///   `dst` is not exactly the band;
/// - for a Compression-7 strip or tile row, the decode-time errors
///   [`unpack_into`] documents for it ([`Error::LjpegPlaneShapeMismatch`] and
///   the `Ljpeg*` / [`Error::Truncated`] family) — the same error
///   [`unpack_into`] stops at when it reaches this unit.
pub fn unpack_unit_into(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    unit: &Unit,
    dst: &mut [u16],
) -> Result<(), Error> {
    let (kind, plan) = plan_for(sensor, file)?;
    // The unit must be one this layout yields: checked BEFORE `dst`, so a
    // foreign unit is reported as such whatever buffer came with it.
    if !plan.yields(sensor.height, unit) {
        return Err(Error::PlaneUnitMismatch {
            row_start: unit.row_start,
            row_count: unit.row_count,
        });
    }
    let expected = u64::from(unit.row_count).saturating_mul(u64::from(sensor.width));
    if u64::try_from(dst.len()).ok() != Some(expected) {
        return Err(Error::PlaneBufferWrongLength {
            expected,
            actual: dst.len(),
        });
    }
    decode_unit(sensor, plan, kind, byte_order, file, unit, dst)
}

// ─────────────────────────────────────────────────────────────────────────────
// The shared checks — one implementation each, used by BOTH `validate` and
// `unpack_into` (`SPEC-039` Design §5)
// ─────────────────────────────────────────────────────────────────────────────

/// Which strip format the plane declares — selects the per-path checks and
/// error variants the shared walk reports.
#[derive(Clone, Copy)]
enum PlaneKind {
    /// `Compression = 1`, at one of the [`SUPPORTED_BITS`] depths.
    Uncompressed {
        /// `BitsPerSample`, already checked against [`SUPPORTED_BITS`].
        bits: u32,
    },
    /// `Compression = 7`, SOF-3 lossless JPEG; precision is the frame's own.
    Jpeg,
}

/// Compression recognised, and for an uncompressed plane a bit depth
/// `DEC-008`'s two paths are defined for.
fn plane_kind(sensor: &Sensor) -> Result<PlaneKind, Error> {
    match sensor.compression {
        Compression::Uncompressed => {
            let bits = sensor.bits_per_sample;
            if !SUPPORTED_BITS.contains(&bits) {
                return Err(Error::UnsupportedBitDepth { bits });
            }
            Ok(PlaneKind::Uncompressed { bits })
        }
        Compression::Jpeg => Ok(PlaneKind::Jpeg),
        other => Err(Error::UnsupportedCompression {
            compression: other.code(),
        }),
    }
}

/// `width · height` as a `u64`. The product of two `u32`s is below `2^64`, so
/// `saturating_mul` is exact here; it only keeps the arithmetic lint satisfied.
fn pixel_product(sensor: &Sensor) -> u64 {
    u64::from(sensor.width).saturating_mul(u64::from(sensor.height))
}

/// One strip's checked location and destination band.
struct StripSpan {
    /// Byte offset of the strip in `file`.
    at: usize,
    /// One past the strip's last byte; `at <= end <= file.len()`.
    end: usize,
    /// Rows this strip holds.
    rows: u32,
}

/// Walks the strip tables in order, yielding each strip's checked
/// [`StripSpan`] — the per-strip checks of `SPEC-028`, factored out so that
/// [`check_strips`] (the full pass) and the write loops run the identical code.
/// Allocation-free; reads only the `Sensor`'s tag values and `file.len()`.
struct StripWalk<'s> {
    sensor: &'s Sensor,
    kind: PlaneKind,
    /// Number of strips (`StripOffsets.len() == StripByteCounts.len()`).
    strips: usize,
    rows_per_strip: u32,
    /// Next strip to visit.
    index: usize,
    /// First row of the next strip.
    row_start: u32,
    /// Plane samples not yet claimed by a strip — an overrun is a strip
    /// claiming more than this; an undershoot leaves it non-zero at the end.
    remaining_samples: usize,
}

impl<'s> StripWalk<'s> {
    /// The plane-level strip layout: the two location tags agree in count,
    /// there is at least one strip, and a multi-strip plane has a usable
    /// `RowsPerStrip`.
    fn new(sensor: &'s Sensor, kind: PlaneKind, plane_samples: usize) -> Result<Self, Error> {
        // N strips (SPEC-028), N >= 1. The two location tags must agree in
        // count, and a plane with no strips cannot be assembled — both are the
        // genuine UnsupportedStripLayout cases (tiles aside).
        let strips = sensor.strip_offsets.len();
        if strips == 0 || strips != sensor.strip_byte_counts.len() {
            return Err(Error::UnsupportedStripLayout {
                strip_offsets: sensor.strip_offsets.len(),
                strip_byte_counts: sensor.strip_byte_counts.len(),
            });
        }

        // RowsPerStrip partitions a multi-strip plane into row bands. A single
        // strip IS the whole plane (SPEC-012's path) and consults no
        // RowsPerStrip; N > 1 requires a usable value, and its absence (or a
        // zero) is a typed error, not a guess.
        let rows_per_strip = if strips == 1 {
            sensor.height
        } else {
            match sensor.rows_per_strip {
                Some(rows) if rows > 0 => rows,
                _ => return Err(Error::MissingRowsPerStrip { strips }),
            }
        };

        Ok(StripWalk {
            sensor,
            kind,
            strips,
            rows_per_strip,
            index: 0,
            row_start: 0,
            remaining_samples: plane_samples,
        })
    }

    /// The next strip's checked span, or `None` once every strip is visited.
    fn next_span(&mut self, file_len: usize) -> Option<Result<StripSpan, Error>> {
        let offset = *self.sensor.strip_offsets.get(self.index)?;
        let count = *self.sensor.strip_byte_counts.get(self.index)?;
        self.index = self.index.saturating_add(1);
        Some(self.span(offset, count, file_len))
    }

    /// The per-strip checks, in `SPEC-028`'s order: the strip's packed size
    /// (uncompressed only), its band fits the plane, its bytes lie in `file`.
    fn span(&mut self, offset: u32, count: u32, file_len: usize) -> Result<StripSpan, Error> {
        let sensor = self.sensor;
        // Rows in this strip: a full band, or the remainder for the last one.
        let remaining_rows = sensor.height.saturating_sub(self.row_start);
        let rows = self.rows_per_strip.min(remaining_rows);
        let samples_u64 = u64::from(rows).checked_mul(u64::from(sensor.width));

        // The error an overrun (or an unrepresentable band) reports: the
        // per-path variant each route has always used.
        let overrun = match self.kind {
            PlaneKind::Uncompressed { bits } => {
                let declared_bits = u64::from(count).saturating_mul(8);
                // Per-strip layer-0 (AGENTS.md §12 bar 3, free): this strip's
                // declared byte count is exactly its own samples' bits rounded
                // up to the next byte. For 8/16-bit there is never padding; for
                // 12/14-bit the final byte of each strip independently carries
                // up to 7 unused bits. This is what caught SPIKE-002's
                // byte-swap (which preserved total length). Skipped for a
                // compressed strip, whose byte count is far below this.
                let strip_sample_bits = samples_u64
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
                if u64::from(count) != expected_bytes {
                    return Err(Error::PackedSizeMismatch {
                        expected_bits: strip_sample_bits,
                        strip_bits: declared_bits,
                    });
                }
                Error::PackedSizeMismatch {
                    expected_bits: strip_sample_bits,
                    strip_bits: declared_bits,
                }
            }
            PlaneKind::Jpeg => Error::UnsupportedStripLayout {
                strip_offsets: self.strips,
                strip_byte_counts: sensor.strip_byte_counts.len(),
            },
        };

        // This strip's band must fit in what is left of the plane — the
        // dst-free form of the `split_at_mut_checked` carve the write loops
        // perform, so the strips provably never overrun the plane.
        let samples = samples_u64
            .and_then(|s| usize::try_from(s).ok())
            .filter(|s| *s <= self.remaining_samples)
            .ok_or(overrun)?;
        self.remaining_samples = self.remaining_samples.saturating_sub(samples);

        // Locate this strip's bytes, bounds-checked into Truncated. Only the
        // bounds are checked here; no strip byte is read.
        let at = offset_to_usize(offset)?;
        let len = usize::try_from(count).map_err(|_| Error::Truncated { at, len: 0 })?;
        let end = at.checked_add(len).ok_or(Error::Truncated { at, len })?;
        if end > file_len {
            return Err(Error::Truncated { at, len });
        }

        self.row_start = self.row_start.saturating_add(rows);
        Ok(StripSpan { at, end, rows })
    }
}

/// The full strip pass, shared by [`validate`] and [`unpack_into`]: the layout,
/// every strip's span, full row coverage, and (uncompressed) the
/// sum-of-`StripByteCounts` bound. Allocation-free; reads no strip bytes.
fn check_strips(
    sensor: &Sensor,
    kind: PlaneKind,
    plane_samples: usize,
    file: &[u8],
) -> Result<(), Error> {
    let mut walk = StripWalk::new(sensor, kind, plane_samples)?;
    while let Some(span) = walk.next_span(file.len()) {
        span?;
    }

    // Full coverage: the strips together claim every row. Strips that do not
    // sum to the plane height leave samples unclaimed — a strip count that
    // disagrees with RowsPerStrip/height, reported here rather than returning
    // a partially-filled plane. (The per-strip fit check rules out an
    // overrun, so this only catches an undershoot.)
    if walk.remaining_samples != 0 {
        return Err(match kind {
            PlaneKind::Uncompressed { bits } => {
                let covered_bits = u64::from(walk.row_start)
                    .saturating_mul(u64::from(sensor.width))
                    .saturating_mul(u64::from(bits));
                Error::PackedSizeMismatch {
                    expected_bits: sensor.packed_bits()?,
                    strip_bits: covered_bits,
                }
            }
            PlaneKind::Jpeg => Error::UnsupportedStripLayout {
                strip_offsets: walk.strips,
                strip_byte_counts: sensor.strip_byte_counts.len(),
            },
        });
    }

    // Uncompressed only (DEC-050 §2): the declared strip bytes may not exceed
    // the file. Each strip is in bounds by now, so a sum past `file.len()`
    // means strips alias the same bytes — the forged shape that would
    // otherwise let a tiny file claim an arbitrarily large plane. With it,
    // `Ok` implies `n · bits <= 8 · file.len()`. A compressed plane is exempt:
    // its bytes legitimately expand, and only the caller's budget bounds it.
    if let PlaneKind::Uncompressed { .. } = kind {
        let declared = sensor
            .strip_byte_counts
            .iter()
            .fold(0u64, |sum, count| sum.saturating_add(u64::from(*count)));
        let file_len = u64::try_from(file.len()).unwrap_or(u64::MAX);
        if declared > file_len {
            return Err(Error::StripBytesExceedFile {
                declared,
                file_len: file.len(),
            });
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// The layout dispatch and the tile table — `SPEC-058`, `DEC-056`
// ─────────────────────────────────────────────────────────────────────────────

/// Which of TIFF 6.0's two plane layouts the sensor IFD declares.
#[derive(Clone, Copy)]
enum Layout {
    /// `StripOffsets`/`StripByteCounts` (or neither layout — the strip checks
    /// then report today's [`Error::UnsupportedStripLayout`]).
    Strips,
    /// `TileWidth`/`TileLength`/`TileOffsets`/`TileByteCounts` (TIFF 6.0 §15).
    Tiles(TileGrid),
}

/// A tiled plane's checked tile grid: tile dimensions and how many tiles span
/// the plane. Built only by [`tile_grid`], so every value is non-zero and the
/// tile count equals both table lengths.
#[derive(Debug, Clone, Copy)]
struct TileGrid {
    /// `TileWidth`, `> 0`.
    tile_width: u32,
    /// `TileLength`, `> 0`.
    tile_length: u32,
    /// `ceil(width / TileWidth)` — tiles per tile row.
    across: u32,
}

/// Choose the layout. A plane with tile tags and no strip tags is tiled; one
/// with both is refused ([`Error::StripsAndTiles`]); anything else takes the
/// strip path, byte-for-byte unchanged.
fn select_layout(sensor: &Sensor) -> Result<Layout, Error> {
    let has_strips = !sensor.strip_offsets.is_empty() || !sensor.strip_byte_counts.is_empty();
    let has_tiles = sensor.tile_width.is_some()
        || sensor.tile_length.is_some()
        || !sensor.tile_offsets.is_empty()
        || !sensor.tile_byte_counts.is_empty();
    match (has_strips, has_tiles) {
        (true, true) => Err(Error::StripsAndTiles {
            strip_offsets: sensor.strip_offsets.len(),
            tile_offsets: sensor.tile_offsets.len(),
        }),
        (false, true) => Ok(Layout::Tiles(tile_grid(sensor)?)),
        _ => Ok(Layout::Strips),
    }
}

/// `ceil(n / d)` for `d > 0`, without the `/` operator the crate root denies.
fn ceil_div(n: u32, d: u32) -> Option<u32> {
    let q = n.checked_div(d)?;
    if n.checked_rem(d)? == 0 {
        Some(q)
    } else {
        q.checked_add(1)
    }
}

/// The tile grid: both dimensions present and non-zero, and both tables
/// exactly `ceil(W/TW) × ceil(H/TL)` long, the product computed in `u64` so a
/// count past `u32` cannot wrap into a false match.
fn tile_grid(sensor: &Sensor) -> Result<TileGrid, Error> {
    let tile_width = sensor.tile_width.unwrap_or(0);
    let tile_length = sensor.tile_length.unwrap_or(0);
    let refuse = |expected_tiles: u64| Error::InvalidTileLayout {
        tile_width,
        tile_length,
        tile_offsets: sensor.tile_offsets.len(),
        tile_byte_counts: sensor.tile_byte_counts.len(),
        expected_tiles,
    };
    let (Some(across), Some(down)) = (
        ceil_div(sensor.width, tile_width),
        ceil_div(sensor.height, tile_length),
    ) else {
        // A zero (or absent) tile dimension: no grid exists.
        return Err(refuse(0));
    };
    // Two u32s: the u64 product is exact.
    let expected = u64::from(across).saturating_mul(u64::from(down));
    let offsets = u64::try_from(sensor.tile_offsets.len()).unwrap_or(u64::MAX);
    let counts = u64::try_from(sensor.tile_byte_counts.len()).unwrap_or(u64::MAX);
    if offsets != expected || counts != expected {
        return Err(refuse(expected));
    }
    Ok(TileGrid {
        tile_width,
        tile_length,
        across,
    })
}

/// One tile's checked location and its clipped rectangle in the plane.
struct TileSpan {
    /// Byte offset of the tile in `file`.
    at: usize,
    /// One past the tile's last byte; `at <= end <= file.len()`.
    end: usize,
    /// Plane column of the tile's first column.
    x0: usize,
    /// Plane row of the tile's first row.
    y0: usize,
    /// Tile columns inside the plane: `min(TileWidth, width − x0)`, `>= 1`.
    visible_width: usize,
    /// Tile rows inside the plane: `min(TileLength, height − y0)`, `>= 1`.
    visible_height: usize,
}

/// Walks a tiled plane's tiles in table (row-major) order, yielding each
/// tile's checked [`TileSpan`] — so [`check_tiles`] (the full pass) and the
/// write loops run the identical code, as [`StripWalk`] does for strips.
/// Allocation-free; reads only the `Sensor`'s tag values and `file.len()`.
struct TileWalk<'s> {
    sensor: &'s Sensor,
    kind: PlaneKind,
    grid: TileGrid,
    /// Next tile to visit.
    index: usize,
}

impl<'s> TileWalk<'s> {
    fn new(sensor: &'s Sensor, kind: PlaneKind, grid: TileGrid) -> TileWalk<'s> {
        TileWalk {
            sensor,
            kind,
            grid,
            index: 0,
        }
    }

    /// The next tile's checked span, or `None` once every tile is visited.
    fn next_span(&mut self, file_len: usize) -> Option<Result<TileSpan, Error>> {
        let offset = *self.sensor.tile_offsets.get(self.index)?;
        let count = *self.sensor.tile_byte_counts.get(self.index)?;
        let index = self.index;
        self.index = self.index.saturating_add(1);
        Some(self.span(index, offset, count, file_len))
    }

    /// The per-tile checks: (uncompressed) the packed tile size, then the
    /// tile's bytes lie in `file`; and its clipped rectangle in the plane.
    fn span(
        &self,
        index: usize,
        offset: u32,
        count: u32,
        file_len: usize,
    ) -> Result<TileSpan, Error> {
        let sensor = self.sensor;
        let grid = self.grid;

        if let PlaneKind::Uncompressed { bits } = self.kind {
            // Every tile is coded at full size, padding included (TIFF 6.0
            // §15), so its byte count is exactly `TW · TL · bits / 8` — the
            // per-tile layer-0 check. `check_tile_rows` already proved a tile
            // row is whole bytes.
            let tile_bits = u64::from(grid.tile_width)
                .saturating_mul(u64::from(grid.tile_length))
                .saturating_mul(u64::from(bits));
            let declared_bits = u64::from(count).saturating_mul(8);
            if declared_bits != tile_bits {
                return Err(Error::PackedSizeMismatch {
                    expected_bits: tile_bits,
                    strip_bits: declared_bits,
                });
            }
        }

        // Locate this tile's bytes, bounds-checked into Truncated. Only the
        // bounds are checked here; no tile byte is read.
        let at = offset_to_usize(offset)?;
        let len = usize::try_from(count).map_err(|_| Error::Truncated { at, len: 0 })?;
        let end = at.checked_add(len).ok_or(Error::Truncated { at, len })?;
        if end > file_len {
            return Err(Error::Truncated { at, len });
        }

        // The tile's origin and visible extent. `index < across · down`
        // (`tile_grid`), so `tx < across` and `ty < down`, which puts the
        // origin strictly inside the plane: `x0 < width`, `y0 < height`.
        let across = usize::try_from(grid.across).unwrap_or(usize::MAX);
        let tx = index.checked_rem(across).unwrap_or(0);
        let ty = index.checked_div(across).unwrap_or(0);
        let x0 = u64::try_from(tx)
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::from(grid.tile_width));
        let y0 = u64::try_from(ty)
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::from(grid.tile_length));
        let visible_width = u64::from(sensor.width)
            .saturating_sub(x0)
            .min(u64::from(grid.tile_width));
        let visible_height = u64::from(sensor.height)
            .saturating_sub(y0)
            .min(u64::from(grid.tile_length));
        // All four are below the plane dimensions, and the plane fits `usize`
        // (its sample count already did); the error arm is unreachable.
        let to_usize = |v: u64| usize::try_from(v).map_err(|_| Error::Truncated { at, len });
        Ok(TileSpan {
            at,
            end,
            x0: to_usize(x0)?,
            y0: to_usize(y0)?,
            visible_width: to_usize(visible_width)?,
            visible_height: to_usize(visible_height)?,
        })
    }
}

/// Uncompressed tiles only: a tile row, `TileWidth × bits`, must be a whole
/// number of bytes (`DEC-056` §1; the `SPEC-057` probe found TIFF 6.0, this
/// reader and dnglab disagree on rows that are not).
fn check_tile_rows(grid: TileGrid, kind: PlaneKind) -> Result<(), Error> {
    if let PlaneKind::Uncompressed { bits } = kind {
        let row_bits = u64::from(grid.tile_width).saturating_mul(u64::from(bits));
        if row_bits.checked_rem(8) != Some(0) {
            return Err(Error::UnsupportedTileRowBits {
                tile_width: grid.tile_width,
                bits,
            });
        }
    }
    Ok(())
}

/// The full tile pass, shared by [`validate`] and [`unpack_into`] through
/// [`check_layout`]: whole-byte tile rows (uncompressed), every tile's span,
/// and (uncompressed) the sum-of-`TileByteCounts` bound. Allocation-free;
/// reads no tile bytes. The grid itself was checked by [`tile_grid`].
fn check_tiles(sensor: &Sensor, kind: PlaneKind, grid: TileGrid, file: &[u8]) -> Result<(), Error> {
    check_tile_rows(grid, kind)?;
    let mut walk = TileWalk::new(sensor, kind, grid);
    while let Some(span) = walk.next_span(file.len()) {
        span?;
    }

    // Uncompressed only (DEC-050 §2, DEC-056 §4): the declared tile bytes may
    // not exceed the file — the strip alias bound, applied to tiles, so `Ok`
    // still implies the plane is bounded by the bytes supplied.
    if let PlaneKind::Uncompressed { .. } = kind {
        let declared_tile_bytes = sensor
            .tile_byte_counts
            .iter()
            .fold(0u64, |sum, count| sum.saturating_add(u64::from(*count)));
        let tile_file_len = u64::try_from(file.len()).unwrap_or(u64::MAX);
        if declared_tile_bytes > tile_file_len {
            return Err(Error::StripBytesExceedFile {
                declared: declared_tile_bytes,
                file_len: file.len(),
            });
        }
    }
    Ok(())
}

/// The layout pass [`validate`] and [`unpack_into`] share: choose strips or
/// tiles, then run that layout's full structural check. Returns the layout so
/// [`unpack_into`] dispatches on exactly what was checked.
fn check_layout(
    sensor: &Sensor,
    kind: PlaneKind,
    plane_samples: usize,
    file: &[u8],
) -> Result<Layout, Error> {
    let layout = select_layout(sensor)?;
    match layout {
        Layout::Strips => check_strips(sensor, kind, plane_samples, file)?,
        Layout::Tiles(grid) => check_tiles(sensor, kind, grid, file)?,
    }
    Ok(layout)
}

// ─────────────────────────────────────────────────────────────────────────────
// The unit kernel — run only after `check_layout` has passed (`SPEC-068`)
// ─────────────────────────────────────────────────────────────────────────────

/// Carve the next `samples` samples off the head of `rest`. After
/// [`check_layout`] the carve always fits; the error arm keeps the write
/// bounds-checked and panic-free regardless.
fn carve(rest: &mut [u16], samples: usize) -> Result<(&mut [u16], &mut [u16]), Error> {
    let available = rest.len();
    rest.split_at_mut_checked(samples)
        .ok_or(Error::PlaneBufferWrongLength {
            expected: u64::try_from(samples).unwrap_or(u64::MAX),
            actual: available,
        })
}

/// This strip's bytes. After [`check_strips`] they are always in `file`; the
/// error arm keeps the slice bounds-checked regardless.
fn strip_bytes<'f>(file: &'f [u8], span: &StripSpan) -> Result<&'f [u8], Error> {
    file.get(span.at..span.end).ok_or(Error::Truncated {
        at: span.at,
        len: span.end.saturating_sub(span.at),
    })
}

/// `DEC-008`'s two uncompressed paths, chosen by depth: plain integers in the
/// container's byte order for 8/16 bits, an MSB-first bitstream otherwise.
/// `bytes` must start on the first sample's first bit.
fn unpack_packed(
    bytes: &[u8],
    bits: u32,
    byte_order: ByteOrder,
    dst: &mut [u16],
) -> Result<(), Error> {
    if bits.checked_rem(8) == Some(0) {
        unpack_byte_aligned(bytes, bits, byte_order, dst)
    } else {
        unpack_bitstream(bytes, bits, dst)
    }
}

/// Decode one unit into `band` (exactly `row_count · width` samples) — the
/// one private kernel both [`unpack_into`] (every unit, in order) and
/// [`unpack_unit_into`] (one unit) call (`DEC-063` §3). `plan` and `kind` come
/// from a [`check_layout`] that has already passed, and `unit` from that same
/// plan, so nothing structural can fail below; the error arms keep every
/// slice checked regardless.
///
/// No `max <= WhiteLevel` assertion here (`DEC-029`, amending `DEC-008`). A
/// sample above `WhiteLevel` is LEGAL: a real sensor records specular
/// highlights above the nominal white point — the Canon 5D-III Bayer plane
/// carries one at 15464 > WhiteLevel 15000 — and the bit-exact plane must
/// PRESERVE the raw value, never clamp it. Clamping is develop's saturation
/// step, not unpack's; doing it here would change bytes and break
/// `SPEC-025`'s `dnglab --raw-checksum`.
fn decode_unit(
    sensor: &Sensor,
    plan: Plan,
    kind: PlaneKind,
    byte_order: ByteOrder,
    file: &[u8],
    unit: &Unit,
    band: &mut [u16],
) -> Result<(), Error> {
    let mismatch = Error::PlaneUnitMismatch {
        row_start: unit.row_start,
        row_count: unit.row_count,
    };
    match (unit.source, plan, kind) {
        // A band of whole-byte rows inside one uncompressed strip. Each strip
        // is an INDEPENDENT bitstream padded to a byte boundary (SPEC-028), so
        // a band never crosses one; inside it, row `r` starts at exactly
        // `r · row_bytes` because every row is whole bytes (DEC-063 §1).
        (
            UnitSource::StripBand {
                strip,
                row_in_strip,
            },
            Plan::Strips { .. },
            PlaneKind::Uncompressed { bits },
        ) => {
            let span = strip_span(sensor, kind, strip, file)?;
            let strip = strip_bytes(file, &span)?;
            let row_bytes = u64::from(sensor.width)
                .saturating_mul(u64::from(bits))
                .checked_div(8)
                .unwrap_or(0);
            let truncated = || Error::Truncated {
                at: span.at,
                len: span.end.saturating_sub(span.at),
            };
            let to_usize = |v: u64| usize::try_from(v).map_err(|_| truncated());
            let at = to_usize(u64::from(row_in_strip).saturating_mul(row_bytes))?;
            let len = to_usize(u64::from(unit.row_count).saturating_mul(row_bytes))?;
            let end = at.checked_add(len).ok_or_else(truncated)?;
            let bytes = strip.get(at..end).ok_or_else(truncated)?;
            unpack_packed(bytes, bits, byte_order, band)
        }
        // A whole uncompressed strip whose rows are not whole bytes: one
        // bitstream from the strip's first byte (SPEC-028).
        (UnitSource::Strip { strip }, Plan::Strips { .. }, PlaneKind::Uncompressed { bits }) => {
            let span = strip_span(sensor, kind, strip, file)?;
            unpack_packed(strip_bytes(file, &span)?, bits, byte_order, band)
        }
        // A whole Compression-7 strip: an INDEPENDENT SOF-3 datastream, decoded
        // from its own bytes into its own row band (SPEC-043).
        (UnitSource::Strip { strip }, Plan::Strips { .. }, PlaneKind::Jpeg) => {
            let span = strip_span(sensor, kind, strip, file)?;
            let strip = strip_bytes(file, &span)?;

            // Bind the frame's SHAPE to this strip before decoding (`DEC-047`).
            // `decode_into` fills `band` row-major by the frame's own width and
            // checks only its AREA, so an equal-area transpose (a 4×6 frame in
            // a 6×4 band) would decode to a silently mis-strided plane. The
            // frame's effective width is `samples_per_line × components` (the
            // `Nf=2` column interleave is `2·X` wide — SPEC-046); it must equal
            // the sensor width, and its `lines` this strip's row count. The
            // extra header parse is tens of bytes.
            let (frame, _tables, _entropy_at) = crate::ljpeg::parse_headers(strip)?;
            check_strip_shape(&frame, sensor.width, span.rows)?;

            // Every malformed / truncated strip returns a typed `Error`, never
            // a panic (inherited from SPEC-037/038). The scan completes on
            // sample count; a missing trailing EOI is not an error (`DEC-050`
            // §4).
            crate::ljpeg::decode_into(strip, band)?;
            Ok(())
        }
        (UnitSource::TileRow { tile_row }, Plan::Tiles { grid, .. }, _) => {
            decode_tile_row(sensor, grid, kind, byte_order, file, tile_row, unit, band)
        }
        _ => Err(mismatch),
    }
}

/// Strip `strip`'s checked span, by the same [`StripWalk`] checks the full
/// pass ran, positioned at that strip.
fn strip_span(
    sensor: &Sensor,
    kind: PlaneKind,
    strip: usize,
    file: &[u8],
) -> Result<StripSpan, Error> {
    let mut walk = StripWalk::new(sensor, kind, 0)?;
    let (first_row, _) = strip_rows(sensor.height, walk.rows_per_strip, strip);
    // The samples this strip and every later one may still claim.
    let below =
        u64::from(sensor.height.saturating_sub(first_row)).saturating_mul(u64::from(sensor.width));
    walk.index = strip;
    walk.row_start = first_row;
    walk.remaining_samples = usize::try_from(below).unwrap_or(usize::MAX);
    walk.next_span(file.len())
        .unwrap_or(Err(Error::UnsupportedStripLayout {
            strip_offsets: sensor.strip_offsets.len(),
            strip_byte_counts: sensor.strip_byte_counts.len(),
        }))
}

/// This tile's bytes. After [`check_tiles`] they are always in `file`.
fn tile_bytes<'f>(file: &'f [u8], span: &TileSpan) -> Result<&'f [u8], Error> {
    file.get(span.at..span.end).ok_or(Error::Truncated {
        at: span.at,
        len: span.end.saturating_sub(span.at),
    })
}

/// One tile row (`DEC-063` §1): every tile in it, left to right, each written
/// into its rectangle of `band` — the tile row's own rows, so each tile's
/// window origin is made band-relative (`y0 − row_start`).
///
/// - **Uncompressed tiles** (`DEC-056` §1): each tile row is unpacked from its
///   own byte slice — a tile row starts on a byte boundary
///   ([`check_tile_rows`]) — and only its first `visible_width` samples are
///   written. Rows below the plane are skipped.
/// - **Compression-7 tiles** (`DEC-056` §2–3): each tile is an independent
///   SOF-3 datastream, its frame shape bound to the TILE (`DEC-047`) before any
///   sample is decoded, then reconstructed in place through a window onto its
///   clipped rectangle — no scratch buffer (`DEC-016`).
#[allow(clippy::too_many_arguments)]
fn decode_tile_row(
    sensor: &Sensor,
    grid: TileGrid,
    kind: PlaneKind,
    byte_order: ByteOrder,
    file: &[u8],
    tile_row: u32,
    unit: &Unit,
    band: &mut [u16],
) -> Result<(), Error> {
    let band_len = band.len();
    let wrong_len = || Error::PlaneBufferWrongLength {
        expected: u64::try_from(band_len).unwrap_or(u64::MAX),
        actual: band_len,
    };
    let width = usize::try_from(sensor.width).map_err(|_| wrong_len())?;
    let band_top = usize::try_from(unit.row_start).map_err(|_| wrong_len())?;
    let across = usize::try_from(grid.across).map_err(|_| wrong_len())?;
    let first_tile = usize::try_from(tile_row)
        .ok()
        .and_then(|row| row.checked_mul(across))
        .ok_or_else(wrong_len)?;

    let mut walk = TileWalk::new(sensor, kind, grid);
    walk.index = first_tile;
    for _ in 0..grid.across {
        let span = walk.next_span(file.len()).ok_or_else(wrong_len)??;
        let tile = tile_bytes(file, &span)?;
        // The tile's first row, relative to the band (every tile in a tile row
        // starts on the band's first row).
        let band_y0 = span.y0.checked_sub(band_top).ok_or_else(wrong_len)?;

        match kind {
            PlaneKind::Uncompressed { bits } => {
                // `TileWidth · bits` is a multiple of 8 (`check_tile_rows`), so
                // this is the exact byte length of one tile row.
                let row_bytes = usize::try_from(
                    u64::from(grid.tile_width)
                        .saturating_mul(u64::from(bits))
                        .checked_div(8)
                        .unwrap_or(0),
                )
                .unwrap_or(usize::MAX);
                for r in 0..span.visible_height {
                    // After `check_tiles` every index below is in bounds; the
                    // error arms keep the slicing checked and panic-free.
                    let truncated = || Error::Truncated {
                        at: span.at,
                        len: span.end.saturating_sub(span.at),
                    };
                    let row_at = r.checked_mul(row_bytes).ok_or_else(truncated)?;
                    let row_end = row_at.checked_add(row_bytes).ok_or_else(truncated)?;
                    let row = tile.get(row_at..row_end).ok_or_else(truncated)?;

                    let out_at = band_y0
                        .checked_add(r)
                        .and_then(|y| y.checked_mul(width))
                        .and_then(|i| i.checked_add(span.x0))
                        .ok_or_else(wrong_len)?;
                    let out_end = out_at
                        .checked_add(span.visible_width)
                        .ok_or_else(wrong_len)?;
                    let out = band.get_mut(out_at..out_end).ok_or_else(wrong_len)?;
                    unpack_packed(row, bits, byte_order, out)?;
                }
            }
            PlaneKind::Jpeg => {
                // Bind the frame's SHAPE to the tile before decoding (DEC-047
                // for tiles): an equal-area transpose would otherwise decode to
                // a silently mis-strided tile. Real edge tiles are coded at
                // full tile size (SPEC-058 probe), so a clipped-size edge frame
                // is refused here too.
                let (frame, _tables, _entropy_at) = crate::ljpeg::parse_headers(tile)?;
                check_strip_shape(&frame, grid.tile_width, grid.tile_length)?;
                let window = crate::ljpeg::Window {
                    stride: width,
                    x0: span.x0,
                    y0: band_y0,
                    visible_width: span.visible_width,
                    visible_height: span.visible_height,
                };
                crate::ljpeg::decode_window_into(tile, band, window)?;
            }
        }
    }
    Ok(())
}

/// Bind a parsed SOF-3 frame's shape to the geometry its strip declares
/// (`DEC-047`): the frame's effective plane width `samples_per_line ×
/// components` (the `Nf=2` column interleave is `2·X` wide — `SPEC-046`) must
/// equal `expected_width`, and its `lines` must equal `expected_height`.
///
/// `ljpeg::decode_into` checks only the destination's AREA, so without this an
/// equal-area transpose decodes to a silently mis-strided plane. Called by
/// the unit kernel before each strip or tile is decoded, and by the `ljpeg`
/// fuzz hook on every parsed header.
///
/// # Errors
///
/// [`Error::LjpegPlaneShapeMismatch`] on any disagreement.
pub(crate) fn check_strip_shape(
    frame: &crate::ljpeg::Frame,
    expected_width: u32,
    expected_height: u32,
) -> Result<(), Error> {
    let decoded_height = u32::from(frame.lines);
    // `u16 × u8` cannot overflow a `u32`; `saturating_mul` keeps the
    // arithmetic lint satisfied, and an impossible overflow would still report
    // as a mismatch rather than wrap into a false match.
    let decoded_width =
        u32::from(frame.samples_per_line).saturating_mul(u32::from(frame.components));
    if decoded_width != expected_width || decoded_height != expected_height {
        return Err(Error::LjpegPlaneShapeMismatch {
            expected_width,
            expected_height,
            decoded_width,
            decoded_height,
        });
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

    // ── SPEC-043: Compression-7 strip-path routing (tier-A) ────────────────
    //
    // Tier-A only (DEC-003): every fixture is a hand-built SOF-3 strip emitted by
    // the STAGE-009 round-trip encoder `ljpeg::encode_lj92`, never a corpus file.
    // The bit-exact `dnglab --raw-checksum` proof on the two real held files is
    // SPEC-044, not here.

    use crate::ifd::Compression;
    use crate::ljpeg::encode_lj92;

    /// A minimal `Sensor` whose only meaningful fields for `unpack_into` are the
    /// geometry, the compression, `rows_per_strip`, and the strip location tags
    /// (`strips` = `[(offset, byte_count)]`). The rest take `base_sensor`'s
    /// neutral defaults — a Compression-7 plane reads none of them.
    fn make_sensor(
        compression: Compression,
        width: u32,
        height: u32,
        bits: u32,
        rows_per_strip: Option<u32>,
        strips: &[(u32, u32)],
    ) -> Sensor {
        Sensor {
            ifd_index: 0,
            width,
            height,
            bits_per_sample: bits,
            samples_per_pixel: 1,
            photometric: 34892,
            compression,
            rows_per_strip,
            strip_offsets: strips.iter().map(|&(o, _)| o).collect(),
            strip_byte_counts: strips.iter().map(|&(_, c)| c).collect(),
            tile_width: None,
            tile_length: None,
            tile_offsets: vec![],
            tile_byte_counts: vec![],
            black_level: None,
            white_level: None,
            black_level_repeat_dim: None,
            active_area: None,
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            opcode_lists: [false, false, false],
            opcode_list_1: None,
            opcode_list_2: None,
            opcode_list_3: None,
            color_filter: None,
            color_matrix_1: None,
            color_matrix_2: None,
            forward_matrix_1: None,
            forward_matrix_2: None,
            as_shot_neutral: None,
            calibration_illuminant_1: None,
            calibration_illuminant_2: None,
            black_levels: vec![],
            white_levels: vec![],
            malformed_tags: vec![],
        }
    }

    /// Lay `strips` out back-to-back in a fresh `file` after an 8-byte pad
    /// (standing in for a container header, so no strip sits at offset 0),
    /// returning the file and each strip's `(offset, byte_count)` — exactly the
    /// descriptor `make_sensor` wants.
    fn plant_strips(strips: &[&[u8]]) -> (Vec<u8>, Vec<(u32, u32)>) {
        let mut file = vec![0u8; 8];
        let mut locs = Vec::with_capacity(strips.len());
        for s in strips {
            let at = u32::try_from(file.len()).unwrap();
            file.extend_from_slice(s);
            locs.push((at, u32::try_from(s.len()).unwrap()));
        }
        (file, locs)
    }

    #[test]
    fn compression7_single_strip_decodes_bit_exact() {
        // A small known plane, encoded as ONE SOF-3 strip, decoded straight
        // through unpack_into's Compression-7 route — bit-exact, and identical
        // for BOTH container byte orders: the SOF-3 datastream is big-endian
        // regardless, so `byte_order` must not touch this path.
        let width = 4u32;
        let height = 3u32;
        let precision = 12u8;
        let samples: Vec<u16> = vec![100, 110, 120, 130, 90, 140, 200, 210, 55, 60, 65, 70];
        assert_eq!(samples.len(), (width * height) as usize);
        let strip = encode_lj92(
            &samples,
            width as usize,
            height as usize,
            precision,
            1,
            0,
            0,
        );
        let (file, locs) = plant_strips(&[&strip]);
        let sensor = make_sensor(
            Compression::Jpeg,
            width,
            height,
            u32::from(precision),
            None,
            &locs,
        );

        for order in [ByteOrder::Big, ByteOrder::Little] {
            let mut dst = vec![0u16; (width * height) as usize];
            unpack_into(&sensor, order, &file, &mut dst).unwrap();
            assert_eq!(dst, samples, "byte order {order:?} must decode bit-exact");
        }
    }

    #[test]
    fn uncompressed_path_unchanged_after_routing() {
        // Characterization: an uncompressed `Sensor` still unpacks identically
        // after SPEC-043's compression dispatch — exercised on the 14-bit
        // MSB-first sub-byte path, the one most at risk from the refactor. The
        // strip head and its expected samples are DEC-008 / SPEC-012's measured
        // Q2M values, the same `bit_reader_reads_the_measured_q2m_samples` pins:
        // 8 samples × 14 bits = 112 bits = exactly these 14 bytes.
        let strip: [u8; 14] = [
            0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
        ];
        let expected = [746u16, 725, 711, 752, 646, 705, 772, 686];
        let (file, locs) = plant_strips(&[&strip]);
        let sensor = make_sensor(Compression::Uncompressed, 4, 2, 14, None, &locs);
        let mut dst = vec![0u16; 8];
        unpack_into(&sensor, ByteOrder::Big, &file, &mut dst).unwrap();
        assert_eq!(dst, expected);
    }

    #[test]
    fn compression7_multi_strip_decodes_each_band() {
        // Two INDEPENDENT SOF-3 datastreams as two strips decode into their
        // respective row bands (RowsPerStrip = 2 over a height-4 plane), bit-exact.
        // No held file is multi-strip (§12 probe), so this is tier-A only.
        let width = 3u32;
        let height = 4u32;
        let precision = 12u8;
        let rows_per_strip = 2u32;
        let band0: Vec<u16> = vec![100, 110, 120, 130, 140, 150]; // rows 0..2
        let band1: Vec<u16> = vec![200, 210, 220, 230, 240, 250]; // rows 2..4
        let s0 = encode_lj92(&band0, width as usize, 2, precision, 1, 0, 0);
        let s1 = encode_lj92(&band1, width as usize, 2, precision, 1, 0, 0);
        let (file, locs) = plant_strips(&[&s0, &s1]);
        let sensor = make_sensor(
            Compression::Jpeg,
            width,
            height,
            u32::from(precision),
            Some(rows_per_strip),
            &locs,
        );

        let mut dst = vec![0u16; (width * height) as usize];
        unpack_into(&sensor, ByteOrder::Little, &file, &mut dst).unwrap();

        let mut expected = band0.clone();
        expected.extend_from_slice(&band1);
        assert_eq!(dst, expected);
    }

    #[test]
    fn compression7_frame_dimension_mismatch_is_typed_error() {
        // The strip encodes a 3×2 = 6-sample SOF-3 frame, but the sensor sizes the
        // single row band at 2×2 = 4. The pre-decode shape binding (`DEC-047`)
        // rejects the disagreement as a typed error before decode_into's own
        // length check is reached — never a panic.
        let samples = [10u16, 20, 30, 40, 50, 60]; // 3x2
        let strip = encode_lj92(&samples, 3, 2, 12, 1, 0, 0);
        let (file, locs) = plant_strips(&[&strip]);
        let sensor = make_sensor(Compression::Jpeg, 2, 2, 12, None, &locs);
        let mut dst = vec![0u16; 4];
        let err = unpack_into(&sensor, ByteOrder::Big, &file, &mut dst).unwrap_err();
        assert!(
            matches!(
                err,
                Error::LjpegPlaneShapeMismatch {
                    expected_width: 2,
                    expected_height: 2,
                    decoded_width: 3,
                    decoded_height: 2,
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn compression7_truncated_strip_is_typed_error() {
        // A valid SOF-3 strip cut in half (well short of its header+scan) returns
        // a typed Error — the Ljpeg* / Truncated family, owned by the decoder —
        // never a panic. The binding inherits decode_into's panic-freedom.
        let samples = [10u16, 20, 30, 40, 50, 60]; // 3x2
        let full = encode_lj92(&samples, 3, 2, 12, 1, 0, 0);
        let truncated = &full[..full.len() / 2];
        let (file, locs) = plant_strips(&[truncated]);
        let sensor = make_sensor(Compression::Jpeg, 3, 2, 12, None, &locs);
        let mut dst = vec![0u16; 6];
        assert!(unpack_into(&sensor, ByteOrder::Big, &file, &mut dst).is_err());
    }

    #[test]
    fn non_jpeg_compression_still_rejected() {
        // A non-1/7 Compression code is still the clean typed rejection it was —
        // the dispatch's `other =>` arm fires before any strip work, so the file
        // and buffer contents are immaterial.
        let sensor = make_sensor(Compression::Other(65535), 2, 2, 14, None, &[(8, 4)]);
        let file = vec![0u8; 64];
        let mut dst = vec![0u16; 4];
        let err = unpack_into(&sensor, ByteOrder::Little, &file, &mut dst).unwrap_err();
        assert!(matches!(
            err,
            Error::UnsupportedCompression { compression: 65535 }
        ));
    }
}
