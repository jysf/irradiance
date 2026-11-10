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
//! arithmetic. `DEC-008`'s `Context` records that a `max <= WhiteLevel`
//! assertion caught that byte-swap in `SPIKE-002`; `DEC-029` retired it (a
//! sample above `WhiteLevel` is legal — the Canon 5D-III clips one), so that
//! byte-swap now rests on the bit-exact oracle (`SPEC-025`) plus the per-strip
//! layer-0 packed-bits check.
//!
//! # Allocation — `DEC-016`
//!
//! [`unpack_into`] takes the destination buffer as a caller-owned `&mut
//! [u16]` and allocates nothing itself, so it needs no allocator regardless
//! of how `DEC-002` (`no_std` + `alloc`, still `proposed`) is eventually
//! decided. See `DEC-016` for the alternative considered and rejected.

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
///   file.len()`: the plane is bounded by the bytes actually supplied.
/// - **Compression-7 (SOF-3 lossless JPEG):** the pre-decode structural checks
///   pass — strip layout, `RowsPerStrip`, full row coverage and each strip's
///   in-file offset/count bounds. `n` is bounded **only by `max_pixels`**, not
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
///   [`Error::PackedSizeMismatch`], [`Error::Truncated`] (a strip past the end
///   of `file`).
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

    // 4. The full strip pass shared with `unpack_into`.
    check_strips(sensor, kind, plane_samples, file)?;
    Ok(plane_samples)
}

// ─────────────────────────────────────────────────────────────────────────────
// The entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Unpack the sensor plane's `N` strips into `dst`, one sample per pixel,
/// row-major, uncropped and un-normalised (`SPEC-012` + `SPEC-028`).
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
///   disagree in count, or there are zero strips (tiled planes remain a
///   non-goal); for a Compression-7 plane, also if the strips do not cover the
///   whole plane.
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
    // depth → pixel count → `dst.len()` → strip layout. `validate` shares every
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

    // The same full strip pass `validate` runs. After it, nothing structural
    // can fail below: the uncompressed write cannot fail at all, and the
    // Compression-7 loop can fail only in the decode itself.
    check_strips(sensor, kind, expected_len, file)?;

    // SPEC-043 / STAGE-011: dispatch on the sensor's `Compression` (DEC-012).
    match kind {
        PlaneKind::Uncompressed { bits } => {
            unpack_uncompressed(sensor, bits, byte_order, file, dst)
        }
        PlaneKind::Jpeg => unpack_jpeg(sensor, file, dst),
    }
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
    /// Samples (`rows · width`) this strip fills in the plane.
    samples: usize,
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
        Ok(StripSpan {
            at,
            end,
            rows,
            samples,
        })
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
// The write loops — run only after `check_strips` has passed
// ─────────────────────────────────────────────────────────────────────────────

/// Carve the next `span.samples` samples off the head of `rest`. After
/// [`check_strips`] the carve always fits; the error arm keeps the write
/// bounds-checked and panic-free regardless.
fn carve<'d>(
    rest: &'d mut [u16],
    span: &StripSpan,
) -> Result<(&'d mut [u16], &'d mut [u16]), Error> {
    let available = rest.len();
    rest.split_at_mut_checked(span.samples)
        .ok_or(Error::PlaneBufferWrongLength {
            expected: u64::try_from(span.samples).unwrap_or(u64::MAX),
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

/// The uncompressed sub-byte / byte-aligned strip assembly — `DEC-008`'s two
/// paths, one sample per pixel, row-major (`SPEC-012` + `SPEC-028`). Called by
/// [`unpack_into`] after [`check_strips`]; see [`unpack_into`] for the full
/// contract.
fn unpack_uncompressed(
    sensor: &Sensor,
    bits: u32,
    byte_order: ByteOrder,
    file: &[u8],
    dst: &mut [u16],
) -> Result<(), Error> {
    // Assemble strip by strip into disjoint, in-order row slices of `dst`. Each
    // strip is an INDEPENDENT bitstream padded to a byte boundary (DEC-008), so
    // it is unpacked from a fresh cursor into its own slice — never by
    // concatenating strip bytes, which for a sub-byte depth would read a
    // strip's first sample across the previous strip's tail padding (SPEC-028).
    let byte_aligned = bits.checked_rem(8) == Some(0);
    let mut walk = StripWalk::new(sensor, PlaneKind::Uncompressed { bits }, dst.len())?;
    let mut rest: &mut [u16] = dst;
    while let Some(span) = walk.next_span(file.len()) {
        let span = span?;
        let (chunk, tail) = carve(rest, &span)?;
        rest = tail;
        let strip = strip_bytes(file, &span)?;
        if byte_aligned {
            unpack_byte_aligned(strip, bits, byte_order, chunk)?;
        } else {
            unpack_bitstream(strip, bits, chunk)?;
        }
    }

    // No `max <= WhiteLevel` assertion here (DEC-029, amending DEC-008). A
    // sample above WhiteLevel is LEGAL: a real sensor records specular
    // highlights above the nominal white point — the Canon 5D-III Bayer plane
    // carries one at 15464 > WhiteLevel 15000 — and the bit-exact plane must
    // PRESERVE the raw value, never clamp it. Clamping is develop's saturation
    // step (STAGE-007+), not unpack's; doing it here would change bytes and
    // break SPEC-025's `dnglab --raw-checksum`. WhiteLevel is a develop-time
    // parameter, not an unpack invariant. The per-sample byte-swap this
    // assertion caught in SPIKE-002 is now guarded by the bit-exact oracle
    // (SPEC-025) plus the per-strip layer-0 packed-bits check.
    Ok(())
}

/// Bind a parsed SOF-3 frame's shape to the geometry its strip declares
/// (`DEC-047`): the frame's effective plane width `samples_per_line ×
/// components` (the `Nf=2` column interleave is `2·X` wide — `SPEC-046`) must
/// equal `expected_width`, and its `lines` must equal `expected_height`.
///
/// `ljpeg::decode_into` checks only the destination's AREA, so without this an
/// equal-area transpose decodes to a silently mis-strided plane. Called by
/// [`unpack_jpeg`] before each strip is decoded, and by the `ljpeg` fuzz hook
/// on every parsed header.
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

/// Route a DNG `Compression 7` (SOF-3 lossless JPEG) plane: each strip is an
/// independent SOF-3 datastream, decoded by the crate's SOF-3 decoder straight
/// into its own `rows_in_strip × width` row band of `dst` (`SPEC-043`, the
/// binding half of `STAGE-011`).
///
/// Called by [`unpack_into`] after [`check_strips`] has passed for every strip
/// (layout, `RowsPerStrip`, full coverage, in-file bounds — `SPEC-039`), so this
/// loop does only the decode-time work: the frame-shape binding and the decode.
/// It skips the sub-byte unpack and the layer-0 packed-byte-count check — both
/// meaningless for a compressed strip, whose declared byte count is far below
/// `width × rows × bits / 8`. The sample precision is the SOF-3 frame's own `P`,
/// read by the decoder, so this path never consults `SUPPORTED_BITS`.
///
/// # Errors
///
/// - [`Error::LjpegPlaneShapeMismatch`] if a strip's SOF-3 frame shape
///   (`samples_per_line × components` by `lines`) is not the strip's declared
///   `width × rows_in_strip` — checked before any sample is decoded, so an
///   equal-area transpose is refused rather than mis-strided (`DEC-047`).
/// - any typed error from the decoder — the `Ljpeg*` / [`Error::Truncated`]
///   family on a malformed or truncated strip (including
///   [`Error::LjpegUnsupportedSampling`] for a subsampled component) — never a
///   panic.
fn unpack_jpeg(sensor: &Sensor, file: &[u8], dst: &mut [u16]) -> Result<(), Error> {
    // Decode strip by strip into disjoint, in-order row bands of `dst`. Each
    // strip is an INDEPENDENT SOF-3 datastream (DNG/TIFF JPEG), decoded from its
    // own bytes into its own slice — the same independent-per-strip rule as
    // SPEC-028's sub-byte assembly, with `decode_into` in place of the unpack.
    let mut walk = StripWalk::new(sensor, PlaneKind::Jpeg, dst.len())?;
    let mut rest: &mut [u16] = dst;
    while let Some(span) = walk.next_span(file.len()) {
        let span = span?;
        // This strip's row band: `rows_in_strip * width` samples at the head of
        // the remaining buffer. Sizing the band to `rows_in_strip * width` keeps
        // `decode_into`'s own `out.len() == X·Y` area check meaningful, but the
        // frame's SHAPE is bound explicitly below (`DEC-047`), since area alone
        // admits a transpose.
        let (chunk, tail) = carve(rest, &span)?;
        rest = tail;
        let strip = strip_bytes(file, &span)?;

        // Bind the frame's SHAPE to this strip before decoding (`DEC-047`).
        // `decode_into` fills `chunk` row-major by the frame's own width and
        // checks only its AREA, so an equal-area transpose (a 4×6 frame in a
        // 6×4 band) would decode to a silently mis-strided plane. The frame's
        // effective width is `samples_per_line × components` (the `Nf=2`
        // column interleave is `2·X` wide — SPEC-046); it must equal the
        // sensor width, and its `lines` this strip's row count. The extra
        // header parse is tens of bytes.
        let (frame, _tables, _entropy_at) = crate::ljpeg::parse_headers(strip)?;
        check_strip_shape(&frame, sensor.width, span.rows)?;

        // Decode this strip's SOF-3 datastream into its row band. The returned
        // `Frame` is not needed — the shape check above already bound it — so
        // it is dropped; `decode_into`'s own length check stays as a second,
        // now-redundant guard. Every malformed / truncated strip returns a
        // typed `Error`, never a panic (inherited from SPEC-037/038). The scan
        // completes on sample count; a missing trailing EOI is not an error
        // (`DEC-050` §4).
        crate::ljpeg::decode_into(strip, chunk)?;
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
