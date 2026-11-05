//! SOF-3 lossless-JPEG entropy front-end — `SPEC-036`, `DEC-035`.
//!
//! A DNG `Compression 7` sensor plane is a lossless-JPEG (SOF-3, ITU-T T.81
//! Annex H) bitstream. This module turns that stream, down to the Huffman-coded
//! scan, into the ordered **difference** values (plus the parsed frame
//! parameters) that `SPEC-037` reconstructs into a `u16` plane. It stops at the
//! difference stream: predictors, the initial prediction `2^(P−Pt−1)`, the
//! point transform, the per-restart predictor reset and the output plane are
//! all `SPEC-037`.
//!
//! What it does:
//!
//! - [`parse_headers`] walks the marker/segment structure (`SOI → {DHT | DRI |
//!   SOF3} → SOS`, ITU-T T.81 Annex B), returning the single-component [`Frame`]
//!   parameters, the DC [`HuffTable`], and the byte offset where entropy-coded
//!   data begins.
//! - [`HuffTable::build`] turns a DHT `BITS`/`HUFFVAL` segment into a canonical
//!   decode table (Annex C; the `MINCODE`/`MAXCODE`/`VALPTR` form of Annex F
//!   Figure F.15).
//! - [`ScanReader`] reads the entropy-coded bytes MSB-first with `0xFF`
//!   destuffing and marker detection (Annex B), `DECODE`s a Huffman symbol
//!   (Figure F.16), and `RECEIVE`+`EXTEND`s it to a signed difference
//!   (Figure F.17 / §F.2.2.1). [`ScanReader::restart`] byte-aligns and consumes
//!   an `RSTm` marker (`SPEC-037` calls it per restart interval).
//!
//! # Panic policy
//!
//! RAW is attacker-influenced binary (`no-panics-on-untrusted-input`). Every
//! marker, length, table and bit read is bounded and returns a typed
//! [`Error`] on malformed input — never a panic, no slice indexing, no
//! unchecked arithmetic. `SPEC-038` fuzzes this surface; it is designed for it
//! now. EOF reuses [`Error::Truncated`].
//!
//! # BitReader relationship
//!
//! [`ScanReader`] **mirrors** `src/plane.rs`'s private `BitReader` — MSB-first,
//! `checked_*`-only arithmetic, the `pow2` helper, [`Error::Truncated`] on EOF
//! — but is a **separate** reader, not a reuse: JPEG entropy coding adds
//! `0xFF`-byte stuffing (`0xFF 0x00` → a literal `0xFF` data byte; `0xFF` +
//! anything else = a marker) that the TIFF packed-bits reader has no concept
//! of.
//!
//! # Provenance
//!
//! ITU-T T.81 (1992): Annex B (marker/segment structure, `0xFF` stuffing),
//! Annex C (`BITS`/`HUFFVAL` → canonical Huffman codes), Annex F §F.2.2 +
//! Figures F.15/F.16/F.17 (`DECODE`/`RECEIVE`/`EXTEND`), Annex H (lossless
//! mode). Implemented **clean-room from the published standard** (`DEC-035`):
//! the permissive cross-reference crates (`oxideav-mjpeg`, `photocraft-raw`)
//! were read for structure only, never copied or linked. SOF-3 lossless Huffman
//! coding is patent-clean (`DEC-035`: the lossy-DCT and arithmetic-coding
//! patents expired and never read on it). See `docs/provenance-ledger.md`.

// Nothing in the crate calls this module yet — STAGE-010 wires `Compression 7`
// to it and SPEC-037 adds reconstruction in-place. The front-end's items are
// `pub(crate)` per SPEC-036 and are exercised by this module's own `mod tests`
// (every function and every `Frame` field is read there), but the non-test
// library build sees no caller, so `dead_code` would fire. This allow is
// scoped to exactly this pre-consumer module and comes off when STAGE-010
// wires it in.
#![allow(dead_code)]

use crate::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Markers (ITU-T T.81 Annex B.1) — the subset this front-end recognises.
// ─────────────────────────────────────────────────────────────────────────────

/// Start of image.
const MARKER_SOI: u8 = 0xD8;
/// End of image.
const MARKER_EOI: u8 = 0xD9;
/// Start of frame, lossless (sequential), Huffman coding — the only process
/// this decoder implements.
const MARKER_SOF3: u8 = 0xC3;
/// Define Huffman table(s).
const MARKER_DHT: u8 = 0xC4;
/// Start of scan.
const MARKER_SOS: u8 = 0xDA;
/// Define restart interval.
const MARKER_DRI: u8 = 0xDD;
/// First restart marker; the eight are `0xD0..=0xD7`.
const MARKER_RST0: u8 = 0xD0;

/// `2^n` for `n` in `0..=16`, without the shift/`pow` operators the crate root
/// denies (`clippy::arithmetic_side_effects`). The bit reader needs `n <= 7`
/// (a byte holds at most 8 unconsumed bits), `EXTEND` needs `n <= 15`, and the
/// Huffman over-subscription check needs `n <= 16`; the catch-all is never
/// reached and returns `0` so the function has no panicking path at all
/// (mirrors `plane::pow2`'s total-function shape).
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
        8 => 256,
        9 => 512,
        10 => 1024,
        11 => 2048,
        12 => 4096,
        13 => 8192,
        14 => 16384,
        15 => 32768,
        16 => 65536,
        _ => 0,
    }
}

/// A SOF marker (`0xC0..=0xCF`) that is **not** SOF3 and not one of the
/// non-SOF markers that share the range (`0xC4` DHT, `0xC8` JPG, `0xCC` DAC).
/// Each such marker selects a JPEG process this decoder does not implement.
fn is_other_sof(marker: u8) -> bool {
    matches!(
        marker,
        0xC0 | 0xC1 | 0xC2 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE | 0xCF
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Header-byte readers (big-endian, as JPEG segments always are).
// ─────────────────────────────────────────────────────────────────────────────

fn read_u8(data: &[u8], pos: &mut usize) -> Result<u8, Error> {
    let b = *data
        .get(*pos)
        .ok_or(Error::Truncated { at: *pos, len: 1 })?;
    *pos = pos
        .checked_add(1)
        .ok_or(Error::Truncated { at: *pos, len: 1 })?;
    Ok(b)
}

fn read_u16_be(data: &[u8], pos: &mut usize) -> Result<u16, Error> {
    let hi = read_u8(data, pos)?;
    let lo = read_u8(data, pos)?;
    Ok(u16::from_be_bytes([hi, lo]))
}

/// Read the next marker: one or more `0xFF` fill bytes followed by a non-zero
/// marker code (ITU-T T.81 B.1.1.3). Advances `pos` past the code.
fn next_marker(data: &[u8], pos: &mut usize) -> Result<u8, Error> {
    if !matches!(data.get(*pos), Some(&0xFF)) {
        // A marker was expected here but the byte is not `0xFF`.
        return Err(Error::LjpegMissingMarker { marker: 0 });
    }
    let mut p = *pos;
    while matches!(data.get(p), Some(&0xFF)) {
        p = p.checked_add(1).ok_or(Error::Truncated { at: p, len: 1 })?;
    }
    let code = *data.get(p).ok_or(Error::Truncated { at: p, len: 1 })?;
    if code == 0x00 {
        // `0xFF 0x00` is entropy byte-stuffing, not a marker — illegal between
        // header segments.
        return Err(Error::LjpegUnexpectedMarker { marker: 0 });
    }
    *pos = p.checked_add(1).ok_or(Error::Truncated { at: p, len: 1 })?;
    Ok(code)
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsed frame/scan parameters
// ─────────────────────────────────────────────────────────────────────────────

/// The single-component SOF-3 frame parameters plus the scan parameters this
/// front-end needs. `SPEC-037` reads these to reconstruct samples; `SPEC-036`
/// only populates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frame {
    /// Sample precision `P` (bits per sample), `2..=16` for lossless.
    pub(crate) precision: u8,
    /// Number of lines `Y` (image height).
    pub(crate) lines: u16,
    /// Samples per line `X` (image width).
    pub(crate) samples_per_line: u16,
    /// Point transform `Pt` (= `Al` from SOS, `0..=15`); the right shift
    /// `SPEC-037` applies during reconstruction.
    pub(crate) point_transform: u8,
    /// Predictor selector `Ss` (`1..=7`); which of the seven lossless
    /// predictors `SPEC-037` applies.
    pub(crate) predictor: u8,
    /// Restart interval `Ri` from DRI (`0` = no restart markers).
    pub(crate) restart_interval: u16,
}

// ─────────────────────────────────────────────────────────────────────────────
// DC Huffman decode table (Annex C + Annex F Figure F.15)
// ─────────────────────────────────────────────────────────────────────────────

/// A DC Huffman decode table built from a DHT `BITS`/`HUFFVAL` segment.
///
/// `mincode`/`maxcode`/`valptr` are indexed by `code length − 1` (so index `0`
/// is length 1). `maxcode[i] == -1` marks a length with no codes. This is the
/// `MINCODE`/`MAXCODE`/`VALPTR` decoder-table form of ITU-T T.81 Annex F
/// Figure F.15, built from the canonical codes of Annex C.
#[derive(Debug, Clone)]
pub(crate) struct HuffTable {
    /// Smallest canonical code of each length (index `l-1`).
    mincode: [i32; 16],
    /// Largest canonical code of each length, or `-1` if that length is empty.
    maxcode: [i32; 16],
    /// Index into `huffval` of the first symbol of each length.
    valptr: [usize; 16],
    /// The symbol values in canonical order (the `HUFFVAL` list).
    huffval: Vec<u8>,
}

impl HuffTable {
    /// Build a decode table from a DHT segment's 16 `BITS` counts and its
    /// `HUFFVAL` symbol list (ITU-T T.81 Annex C).
    ///
    /// # Errors
    ///
    /// [`Error::LjpegInvalidHuffmanTable`] if `sum(BITS) != HUFFVAL.len()` or
    /// the lengths over-subscribe the 16-bit code space (a canonical code would
    /// need more than its length's bits).
    pub(crate) fn build(bits: &[u8; 16], huffval: Vec<u8>) -> Result<HuffTable, Error> {
        // sum(BITS) must equal the number of symbols.
        let mut total: usize = 0;
        for &count in bits.iter() {
            total = total
                .checked_add(usize::from(count))
                .ok_or(Error::LjpegInvalidHuffmanTable)?;
        }
        if total != huffval.len() {
            return Err(Error::LjpegInvalidHuffmanTable);
        }

        // Generate the canonical MINCODE/MAXCODE/VALPTR tables. `code` is the
        // running canonical code; it is shifted left by one at each length
        // boundary (Annex C). All arithmetic is bounded — `code < 2^16` for a
        // well-formed table and the over-subscription guard below rejects
        // anything larger — but every step is still `checked_*`.
        let mut mincode = [0i32; 16];
        let mut maxcode = [-1i32; 16];
        let mut valptr = [0usize; 16];
        let mut code: i32 = 0;
        let mut k: usize = 0;
        for (i, &count) in bits.iter().enumerate() {
            if count > 0 {
                let count_i = i32::from(count);
                if let Some(slot) = valptr.get_mut(i) {
                    *slot = k;
                }
                if let Some(slot) = mincode.get_mut(i) {
                    *slot = code;
                }
                let next = code
                    .checked_add(count_i)
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
                let maxc = next.checked_sub(1).ok_or(Error::LjpegInvalidHuffmanTable)?;
                if let Some(slot) = maxcode.get_mut(i) {
                    *slot = maxc;
                }
                // Over-subscription: codes of length `l = i + 1` must fit in
                // `l` bits, i.e. the next free code is `<= 2^l`.
                let l = i.checked_add(1).unwrap_or(16);
                let limit = i32::try_from(pow2(u32::try_from(l).unwrap_or(16))).unwrap_or(i32::MAX);
                if next > limit {
                    return Err(Error::LjpegInvalidHuffmanTable);
                }
                code = next;
                k = k
                    .checked_add(usize::from(count))
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
            }
            // Shift left for the next code length.
            code = code.checked_mul(2).ok_or(Error::LjpegInvalidHuffmanTable)?;
        }

        Ok(HuffTable {
            mincode,
            maxcode,
            valptr,
            huffval,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// parse_headers — walk the marker/segment structure (Annex B)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a single-component SOF-3 header: walk `SOI → {DHT | DRI | SOF3} → SOS`
/// and return the [`Frame`], the DC [`HuffTable`] the scan selects, and the
/// byte offset into `data` where the entropy-coded scan begins.
///
/// # Errors
///
/// A typed [`Error`] on every malformed input, never a panic:
/// [`Error::LjpegMissingMarker`] (no SOI, a scan with no frame/table, EOI
/// before a scan, or a segment length that does not fit),
/// [`Error::LjpegUnsupportedProcess`] (a SOF that is not SOF3),
/// [`Error::LjpegUnsupportedComponentCount`] (`Nf != 1`),
/// [`Error::LjpegUnsupportedPrecision`] (`P` outside `2..=16`),
/// [`Error::LjpegUnsupportedPredictor`] (`Ss` outside `1..=7`),
/// [`Error::LjpegInvalidHuffmanTable`] (a malformed DHT),
/// [`Error::LjpegUnexpectedMarker`] (a marker where none belongs), and
/// [`Error::Truncated`] (EOF inside a segment).
pub(crate) fn parse_headers(data: &[u8]) -> Result<(Frame, HuffTable, usize), Error> {
    // SOI must open the stream.
    if !matches!(data.first(), Some(&0xFF)) || !matches!(data.get(1), Some(&MARKER_SOI)) {
        return Err(Error::LjpegMissingMarker { marker: MARKER_SOI });
    }
    let mut pos: usize = 2;

    let mut precision: Option<u8> = None;
    let mut lines: u16 = 0;
    let mut samples_per_line: u16 = 0;
    let mut restart_interval: u16 = 0;
    // DC tables keyed by their destination identifier `Th`.
    let mut dc_tables: Vec<(u8, HuffTable)> = Vec::new();

    loop {
        let marker = next_marker(data, &mut pos)?;
        match marker {
            MARKER_SOF3 => {
                let (p, y, x) = parse_frame_header(data, &mut pos)?;
                precision = Some(p);
                lines = y;
                samples_per_line = x;
            }
            MARKER_DHT => parse_dht(data, &mut pos, &mut dc_tables)?,
            MARKER_DRI => restart_interval = parse_dri(data, &mut pos)?,
            MARKER_SOS => {
                let (predictor, point_transform, td) = parse_scan_header(data, &mut pos)?;
                let precision = precision.ok_or(Error::LjpegMissingMarker {
                    marker: MARKER_SOF3,
                })?;
                let table = take_table(&mut dc_tables, td)
                    .ok_or(Error::LjpegMissingMarker { marker: MARKER_DHT })?;
                let frame = Frame {
                    precision,
                    lines,
                    samples_per_line,
                    point_transform,
                    predictor,
                    restart_interval,
                };
                return Ok((frame, table, pos));
            }
            MARKER_EOI => return Err(Error::LjpegMissingMarker { marker: MARKER_SOS }),
            m if is_other_sof(m) => return Err(Error::LjpegUnsupportedProcess { sof: m }),
            other => return Err(Error::LjpegUnexpectedMarker { marker: other }),
        }
    }
}

/// Remove and return the DC table with destination `th`, if present.
fn take_table(tables: &mut Vec<(u8, HuffTable)>, th: u8) -> Option<HuffTable> {
    let idx = tables.iter().position(|(t, _)| *t == th)?;
    Some(tables.swap_remove(idx).1)
}

/// Parse a SOF3 frame header (ITU-T T.81 B.2.2): `Lf P Y X Nf` then `Nf`
/// component specifications. Returns `(precision, lines, samples_per_line)`.
fn parse_frame_header(data: &[u8], pos: &mut usize) -> Result<(u8, u16, u16), Error> {
    let lf = read_u16_be(data, pos)?;
    let precision = read_u8(data, pos)?;
    let y = read_u16_be(data, pos)?;
    let x = read_u16_be(data, pos)?;
    let nf = read_u8(data, pos)?;

    if nf != 1 {
        return Err(Error::LjpegUnsupportedComponentCount { nf });
    }
    // Lf must equal `8 + 3*Nf` (header fields + one 3-byte component spec).
    let expected_lf = 3usize
        .checked_mul(usize::from(nf))
        .and_then(|c| c.checked_add(8))
        .ok_or(Error::LjpegMissingMarker {
            marker: MARKER_SOF3,
        })?;
    if usize::from(lf) != expected_lf {
        return Err(Error::LjpegMissingMarker {
            marker: MARKER_SOF3,
        });
    }
    if !(2..=16).contains(&precision) {
        return Err(Error::LjpegUnsupportedPrecision { precision });
    }

    // The single component's spec: Ci, Hi/Vi, Tqi. The front-end stores none of
    // them (a single component has no sampling-factor or quantisation meaning
    // for lossless), but it must consume the bytes to reach the next marker.
    let _ci = read_u8(data, pos)?;
    let _hv = read_u8(data, pos)?;
    let _tq = read_u8(data, pos)?;

    Ok((precision, y, x))
}

/// Parse a DHT segment (ITU-T T.81 B.2.4.2): one or more `Tc/Th | BITS[16] |
/// HUFFVAL` tables within the declared length. DC (class 0) tables are built
/// and stored by destination; any AC table (class 1), which lossless does not
/// use, is consumed and ignored.
fn parse_dht(
    data: &[u8],
    pos: &mut usize,
    dc_tables: &mut Vec<(u8, HuffTable)>,
) -> Result<(), Error> {
    let lh = usize::from(read_u16_be(data, pos)?);
    // Payload is `Lh - 2` bytes, beginning at the current position.
    let payload = lh
        .checked_sub(2)
        .ok_or(Error::LjpegMissingMarker { marker: MARKER_DHT })?;
    let seg_end = pos.checked_add(payload).ok_or(Error::Truncated {
        at: *pos,
        len: payload,
    })?;
    if seg_end > data.len() {
        return Err(Error::Truncated {
            at: *pos,
            len: payload,
        });
    }

    while *pos < seg_end {
        let tc_th = read_u8(data, pos)?;
        let tc = tc_th.checked_div(16).unwrap_or(0); // high nibble: table class
        let th = tc_th.checked_rem(16).unwrap_or(0); // low nibble: destination

        let mut bits = [0u8; 16];
        let mut total: usize = 0;
        for slot in bits.iter_mut() {
            let c = read_u8(data, pos)?;
            *slot = c;
            total = total
                .checked_add(usize::from(c))
                .ok_or(Error::LjpegInvalidHuffmanTable)?;
        }

        // The HUFFVAL list must fit within the declared segment — a sum(BITS)
        // that would read past `seg_end` is a length-inconsistent table.
        let huffval_end = pos
            .checked_add(total)
            .ok_or(Error::LjpegInvalidHuffmanTable)?;
        if huffval_end > seg_end {
            return Err(Error::LjpegInvalidHuffmanTable);
        }

        let mut huffval: Vec<u8> = Vec::with_capacity(total);
        for _ in 0..total {
            huffval.push(read_u8(data, pos)?);
        }

        if tc == 0 {
            let table = HuffTable::build(&bits, huffval)?;
            dc_tables.push((th, table));
        }
        // tc == 1 (AC) is unused by lossless; its bytes are consumed above and
        // the table is dropped rather than built.
    }

    // The tables must consume the segment exactly.
    if *pos != seg_end {
        return Err(Error::LjpegInvalidHuffmanTable);
    }
    Ok(())
}

/// Parse a DRI segment (ITU-T T.81 B.2.4.4): `Lr` (= 4) then the restart
/// interval `Ri`.
fn parse_dri(data: &[u8], pos: &mut usize) -> Result<u16, Error> {
    let lr = read_u16_be(data, pos)?;
    if lr != 4 {
        return Err(Error::LjpegMissingMarker { marker: MARKER_DRI });
    }
    read_u16_be(data, pos)
}

/// Parse a SOS scan header (ITU-T T.81 B.2.3): `Ls Ns {Csj Tdj/Taj} Ss Se
/// Ah/Al`. For lossless, `Ss` is the predictor selector and `Al` is the point
/// transform. Returns `(predictor, point_transform, dc_table_destination)`.
fn parse_scan_header(data: &[u8], pos: &mut usize) -> Result<(u8, u8, u8), Error> {
    let ls = read_u16_be(data, pos)?;
    let ns = read_u8(data, pos)?;
    if ns != 1 {
        return Err(Error::LjpegUnsupportedComponentCount { nf: ns });
    }
    // Ls must equal `6 + 2*Ns`.
    let expected_ls = 2usize
        .checked_mul(usize::from(ns))
        .and_then(|c| c.checked_add(6))
        .ok_or(Error::LjpegMissingMarker { marker: MARKER_SOS })?;
    if usize::from(ls) != expected_ls {
        return Err(Error::LjpegMissingMarker { marker: MARKER_SOS });
    }

    let _cs = read_u8(data, pos)?; // scan component selector
    let td_ta = read_u8(data, pos)?;
    let td = td_ta.checked_div(16).unwrap_or(0); // DC table destination (high nibble)
    let ss = read_u8(data, pos)?; // predictor
    let _se = read_u8(data, pos)?; // 0 in lossless
    let ah_al = read_u8(data, pos)?;
    let al = ah_al.checked_rem(16).unwrap_or(0); // point transform (low nibble)

    if !(1..=7).contains(&ss) {
        return Err(Error::LjpegUnsupportedPredictor { predictor: ss });
    }

    Ok((ss, al, td))
}

// ─────────────────────────────────────────────────────────────────────────────
// ScanReader — the 0xFF-destuffing, MSB-first entropy reader
// ─────────────────────────────────────────────────────────────────────────────

/// Reads the entropy-coded scan: MSB-first bits with `0xFF` destuffing and
/// marker detection, Huffman `DECODE`, and `RECEIVE`+`EXTEND` to a signed
/// difference.
///
/// Mirrors `plane::BitReader` (checked arithmetic, `pow2`, [`Error::Truncated`]
/// on EOF) and adds JPEG byte-stuffing: `0xFF 0x00` is a literal `0xFF` data
/// byte, and any other `0xFF xx` is a marker that ends the entropy segment.
pub(crate) struct ScanReader<'a> {
    data: &'a [u8],
    /// Next byte to fetch from `data`.
    byte_pos: usize,
    /// The current data byte (post-destuffing) being consumed.
    bit_buffer: u32,
    /// Bits still unconsumed in `bit_buffer`, from the MSB side (`0..=8`).
    bits_left: u32,
    /// The `m` of the next expected `RSTm` marker (`0..=7`, cycling).
    expected_rst: u8,
}

impl<'a> ScanReader<'a> {
    /// Create a reader over the entropy-coded bytes (the slice beginning at the
    /// offset [`parse_headers`] returns).
    pub(crate) fn new(entropy: &'a [u8]) -> ScanReader<'a> {
        ScanReader {
            data: entropy,
            byte_pos: 0,
            bit_buffer: 0,
            bits_left: 0,
            expected_rst: 0,
        }
    }

    /// Fetch the next entropy **data** byte, applying `0xFF 0x00` destuffing.
    ///
    /// # Errors
    ///
    /// [`Error::LjpegUnexpectedMarker`] if a real marker (`0xFF` + non-zero) is
    /// reached where a data byte was expected; [`Error::Truncated`] at EOF.
    fn fetch_byte(&mut self) -> Result<u8, Error> {
        let b = *self.data.get(self.byte_pos).ok_or(Error::Truncated {
            at: self.byte_pos,
            len: 1,
        })?;
        if b != 0xFF {
            self.byte_pos = self.byte_pos.checked_add(1).ok_or(Error::Truncated {
                at: self.byte_pos,
                len: 1,
            })?;
            return Ok(b);
        }
        // `0xFF`: inspect the following byte.
        let next_pos = self.byte_pos.checked_add(1).ok_or(Error::Truncated {
            at: self.byte_pos,
            len: 1,
        })?;
        let next = *self.data.get(next_pos).ok_or(Error::Truncated {
            at: next_pos,
            len: 1,
        })?;
        if next == 0x00 {
            // Stuffed: `0xFF 0x00` is a literal `0xFF` data byte.
            self.byte_pos = self.byte_pos.checked_add(2).ok_or(Error::Truncated {
                at: self.byte_pos,
                len: 2,
            })?;
            Ok(0xFF)
        } else {
            // A marker where entropy data was expected. `byte_pos` is left at
            // the `0xFF` so `restart` can find the marker.
            Err(Error::LjpegUnexpectedMarker { marker: next })
        }
    }

    /// Read one bit, MSB-first.
    fn read_bit(&mut self) -> Result<u32, Error> {
        if self.bits_left == 0 {
            self.bit_buffer = u32::from(self.fetch_byte()?);
            self.bits_left = 8;
        }
        // The next unconsumed bit is bit `bits_left - 1` (MSB-first).
        let shift = self.bits_left.checked_sub(1).ok_or(Error::Truncated {
            at: self.byte_pos,
            len: 1,
        })?;
        let bit = self
            .bit_buffer
            .checked_div(pow2(shift))
            .unwrap_or(0)
            .checked_rem(2)
            .unwrap_or(0);
        self.bits_left = shift;
        Ok(bit)
    }

    /// `RECEIVE(count)` (ITU-T T.81 Figure F.17): read `count` bits MSB-first
    /// into an unsigned magnitude. `count` is `0..=15` in use; the accumulator
    /// cannot overflow, and `unwrap_or` covers that structurally-unreachable
    /// case without a dead error arm (as `plane::BitReader` does).
    fn receive(&mut self, count: u32) -> Result<u32, Error> {
        let mut value: u32 = 0;
        let mut remaining = count;
        while remaining > 0 {
            let bit = self.read_bit()?;
            value = value
                .checked_mul(2)
                .and_then(|v| v.checked_add(bit))
                .unwrap_or(value);
            remaining = remaining.saturating_sub(1);
        }
        Ok(value)
    }

    /// `DECODE` one Huffman symbol (ITU-T T.81 Figure F.16): read bits until the
    /// accumulated code lies within a code length's `[MINCODE, MAXCODE]`, then
    /// look the symbol up in `HUFFVAL`.
    ///
    /// # Errors
    ///
    /// [`Error::LjpegInvalidHuffmanTable`] if the code does not resolve within
    /// 16 bits; [`Error::LjpegUnexpectedMarker`]/[`Error::Truncated`] from the
    /// underlying bit reads.
    fn decode_symbol(&mut self, table: &HuffTable) -> Result<u8, Error> {
        // `code` accumulates at most 16 bits, so `code < 2^16` and never
        // overflows `i32`; `unwrap_or` covers the unreachable overflow.
        let mut code: i32 = i32::try_from(self.read_bit()?).unwrap_or(0);
        let mut l: usize = 1;
        loop {
            let idx = l.checked_sub(1).ok_or(Error::LjpegInvalidHuffmanTable)?;
            let maxc = *table
                .maxcode
                .get(idx)
                .ok_or(Error::LjpegInvalidHuffmanTable)?;
            if maxc >= 0 && code <= maxc {
                let minc = *table
                    .mincode
                    .get(idx)
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
                let vp = *table
                    .valptr
                    .get(idx)
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
                let offset = code
                    .checked_sub(minc)
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
                let offset =
                    usize::try_from(offset).map_err(|_| Error::LjpegInvalidHuffmanTable)?;
                let hidx = vp
                    .checked_add(offset)
                    .ok_or(Error::LjpegInvalidHuffmanTable)?;
                return table
                    .huffval
                    .get(hidx)
                    .copied()
                    .ok_or(Error::LjpegInvalidHuffmanTable);
            }
            if l >= 16 {
                // No code of any length `1..=16` matched — a corrupt table or
                // bitstream (a code that overruns 16 bits).
                return Err(Error::LjpegInvalidHuffmanTable);
            }
            let bit = i32::try_from(self.read_bit()?).unwrap_or(0);
            code = code
                .checked_mul(2)
                .and_then(|c| c.checked_add(bit))
                .unwrap_or(code);
            l = l.checked_add(1).ok_or(Error::LjpegInvalidHuffmanTable)?;
        }
    }

    /// Given a magnitude category `ssss` (`0..=16`), read and sign-extend the
    /// difference (ITU-T T.81 Annex H.1.2.1): `0 → 0`; `16 → 32768` (no bits
    /// read); otherwise `RECEIVE(ssss)` then `EXTEND`.
    ///
    /// # Errors
    ///
    /// [`Error::LjpegInvalidHuffmanTable`] if `ssss > 16`; bit-read errors
    /// otherwise.
    fn receive_and_extend(&mut self, ssss: u32) -> Result<i32, Error> {
        match ssss {
            0 => Ok(0),
            16 => Ok(32768),
            1..=15 => {
                let v = self.receive(ssss)?;
                Ok(extend(v, ssss))
            }
            _ => Err(Error::LjpegInvalidHuffmanTable),
        }
    }

    /// Decode one sample's signed difference: `DECODE` the category, then
    /// `RECEIVE`+`EXTEND` it (ITU-T T.81 §F.2.2.1 / Annex H.1.2.1).
    ///
    /// This is the end of `SPEC-036`'s scope: the *difference*, not the
    /// reconstructed sample. Applying predictors, the initial prediction and
    /// the point transform to turn these into `u16` samples is `SPEC-037`.
    ///
    /// # Errors
    ///
    /// A typed [`Error`] on any malformed table, code, or truncated scan; never
    /// a panic.
    pub(crate) fn decode_difference(&mut self, table: &HuffTable) -> Result<i32, Error> {
        let ssss = self.decode_symbol(table)?;
        self.receive_and_extend(u32::from(ssss))
    }

    /// Byte-align and consume the next expected `RSTm` marker (ITU-T T.81
    /// B.2.1 / E.1.1). `SPEC-037` calls this at each restart interval; resetting
    /// the predictor afterwards is `SPEC-037`'s, not this method's.
    ///
    /// # Errors
    ///
    /// [`Error::LjpegUnexpectedMarker`] if the marker is absent or is not the
    /// next `RSTm` in the `D0..D7` cycle; [`Error::Truncated`] at EOF.
    pub(crate) fn restart(&mut self) -> Result<(), Error> {
        // Byte-align: discard any unconsumed bits of the current byte.
        self.bits_left = 0;

        let mut p = self.byte_pos;
        if !matches!(self.data.get(p), Some(&0xFF)) {
            return match self.data.get(p).copied() {
                Some(byte) => Err(Error::LjpegUnexpectedMarker { marker: byte }),
                None => Err(Error::Truncated { at: p, len: 1 }),
            };
        }
        while matches!(self.data.get(p), Some(&0xFF)) {
            p = p.checked_add(1).ok_or(Error::Truncated { at: p, len: 1 })?;
        }
        let code = *self.data.get(p).ok_or(Error::Truncated { at: p, len: 1 })?;
        let expected = MARKER_RST0
            .checked_add(self.expected_rst)
            .ok_or(Error::LjpegUnexpectedMarker { marker: code })?;
        if code != expected {
            return Err(Error::LjpegUnexpectedMarker { marker: code });
        }
        self.byte_pos = p.checked_add(1).ok_or(Error::Truncated { at: p, len: 1 })?;
        // Advance the restart counter, cycling 0..=7.
        self.expected_rst = self
            .expected_rst
            .checked_add(1)
            .unwrap_or(0)
            .checked_rem(8)
            .unwrap_or(0);
        Ok(())
    }
}

/// `EXTEND(V, T)` (ITU-T T.81 Figure F.12 / §F.2.2.1): sign-extend a `T`-bit
/// (`ssss`) magnitude `v` to a signed difference. `ssss` is `1..=15` here
/// (`0` and `16` are handled by the caller). If `v` is below `2^(ssss-1)` the
/// value is negative and `v - 2^ssss + 1` is its signed form.
fn extend(v: u32, ssss: u32) -> i32 {
    let vt = pow2(ssss.saturating_sub(1)); // 2^(ssss-1)
    let v_signed = i32::try_from(v).unwrap_or(0);
    if v < vt {
        let two_pow = i32::try_from(pow2(ssss)).unwrap_or(0); // 2^ssss, fits i32 for ssss<=15
        v_signed.wrapping_sub(two_pow).wrapping_add(1)
    } else {
        v_signed
    }
}

#[cfg(test)]
#[allow(
    // Test code is not a parse/decode path on untrusted input — the same five
    // lints are relaxed here as in src/plane.rs's own `mod tests`.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    // ── Hand-built header assembly helpers ────────────────────────────────
    //
    // These are the seed of SPEC-037's encoder: they assemble a SOF-3 header +
    // scan from parameters so the tests read as data, not byte soup.

    fn sof3_segment(precision: u8, lines: u16, samples: u16) -> Vec<u8> {
        let mut seg = vec![0xFF, MARKER_SOF3];
        seg.extend_from_slice(&11u16.to_be_bytes()); // Lf = 8 + 3*1
        seg.push(precision);
        seg.extend_from_slice(&lines.to_be_bytes());
        seg.extend_from_slice(&samples.to_be_bytes());
        seg.push(1); // Nf
        seg.extend_from_slice(&[0x00, 0x11, 0x00]); // Ci, Hi/Vi, Tqi
        seg
    }

    fn dht_segment(th: u8, bits: &[u8; 16], huffval: &[u8]) -> Vec<u8> {
        let lh = (2 + 1 + 16 + huffval.len()) as u16;
        let mut seg = vec![0xFF, MARKER_DHT];
        seg.extend_from_slice(&lh.to_be_bytes());
        seg.push(th & 0x0F); // Tc = 0 (DC/lossless) in the high nibble, Th = th
        seg.extend_from_slice(bits);
        seg.extend_from_slice(huffval);
        seg
    }

    fn dri_segment(ri: u16) -> Vec<u8> {
        let mut seg = vec![0xFF, MARKER_DRI];
        seg.extend_from_slice(&4u16.to_be_bytes()); // Lr
        seg.extend_from_slice(&ri.to_be_bytes());
        seg
    }

    fn sos_segment(ss: u8, al: u8, td: u8) -> Vec<u8> {
        let mut seg = vec![0xFF, MARKER_SOS];
        seg.extend_from_slice(&8u16.to_be_bytes()); // Ls = 6 + 2*1
        seg.push(1); // Ns
        seg.push(0x00); // Cs
        seg.push(td << 4); // Td (DC table) in the high nibble, Ta = 0
        seg.push(ss); // Ss (predictor)
        seg.push(0x00); // Se
        seg.push(al & 0x0F); // Ah = 0 in the high nibble, Al (point transform) in the low
        seg
    }

    /// Pack `nbits` low bits of `value` MSB-first into bytes, then apply JPEG
    /// `0xFF 0x00` stuffing so the bytes are a legal entropy stream.
    fn stuffed_bits(value: u32, nbits: u32) -> Vec<u8> {
        let mut flat: Vec<u8> = Vec::new();
        for i in (0..nbits).rev() {
            flat.push(((value >> i) & 1) as u8);
        }
        let mut raw: Vec<u8> = Vec::new();
        for chunk in flat.chunks(8) {
            let mut b = 0u8;
            for (j, &bit) in chunk.iter().enumerate() {
                b |= bit << (7 - j);
            }
            raw.push(b);
        }
        let mut out: Vec<u8> = Vec::new();
        for b in raw {
            out.push(b);
            if b == 0xFF {
                out.push(0x00);
            }
        }
        out
    }

    /// A tiny valid DC table: one 1-bit code → symbol 0.
    fn minimal_table_bytes() -> ([u8; 16], Vec<u8>) {
        ([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], vec![0x00])
    }

    // ── Acceptance tests ──────────────────────────────────────────────────

    #[test]
    fn parse_headers_reads_a_minimal_sof3_frame() {
        let (bits, huffval) = minimal_table_bytes();
        let mut data = vec![0xFF, MARKER_SOI];
        data.extend(sof3_segment(8, 2, 2));
        data.extend(dht_segment(0, &bits, &huffval));
        data.extend(sos_segment(1, 0, 0)); // predictor 1, Pt 0
        let entropy_offset = data.len();
        data.extend_from_slice(&[0x00, 0xFF, MARKER_EOI]);

        let (frame, _table, offset) = parse_headers(&data).unwrap();
        assert_eq!(frame.precision, 8);
        assert_eq!(frame.lines, 2);
        assert_eq!(frame.samples_per_line, 2);
        assert_eq!(frame.predictor, 1);
        assert_eq!(frame.point_transform, 0);
        assert_eq!(frame.restart_interval, 0);
        assert_eq!(offset, entropy_offset);
    }

    #[test]
    fn parse_headers_reads_precision_dimensions_pt_and_ri() {
        let (bits, huffval) = minimal_table_bytes();
        let mut data = vec![0xFF, MARKER_SOI];
        data.extend(sof3_segment(16, 768, 1024));
        data.extend(dri_segment(8)); // restart interval
        data.extend(dht_segment(0, &bits, &huffval));
        data.extend(sos_segment(2, 3, 0)); // predictor 2, Pt 3
        let entropy_offset = data.len();
        data.extend_from_slice(&[0x00, 0xFF, MARKER_EOI]);

        let (frame, _table, offset) = parse_headers(&data).unwrap();
        assert_eq!(frame.precision, 16);
        assert_eq!(frame.lines, 768);
        assert_eq!(frame.samples_per_line, 1024);
        assert_eq!(frame.predictor, 2);
        assert_eq!(frame.point_transform, 3);
        assert_eq!(frame.restart_interval, 8);
        assert_eq!(offset, entropy_offset);
    }

    #[test]
    fn huffman_table_decodes_canonical_codes() {
        // One 1-bit code (the shortest) and one 16-bit code (the longest),
        // symbols 5 and 10.
        let bits = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let table = HuffTable::build(&bits, vec![5u8, 10u8]).unwrap();

        // Canonical codes: "0" → 5, "1000000000000000" (16 bits) → 10.
        // Bit stream: 0 | 1 0...(15 zeros) = 0x40 0x00 0x00.
        let entropy = [0x40, 0x00, 0x00];
        let mut r = ScanReader::new(&entropy);
        assert_eq!(r.decode_symbol(&table).unwrap(), 5);
        assert_eq!(r.decode_symbol(&table).unwrap(), 10);
    }

    #[test]
    fn receive_extend_sign_extends_per_annex_f() {
        // (ssss, raw magnitude, expected signed difference).
        let cases: &[(u32, u32, i32)] = &[
            (1, 0b0, -1),
            (1, 0b1, 1),
            (2, 0b00, -3),
            (2, 0b01, -2),
            (2, 0b10, 2),
            (2, 0b11, 3),
            (8, 0, -255),
            (8, 255, 255),
            (15, 0, -32767),
            (15, 0x7FFF, 32767),
        ];
        for &(ssss, raw, expected) in cases {
            let entropy = stuffed_bits(raw, ssss);
            let mut r = ScanReader::new(&entropy);
            let diff = r.receive_and_extend(ssss).unwrap();
            assert_eq!(diff, expected, "ssss={ssss} raw={raw}");
        }
        // The two no-RECEIVE categories.
        assert_eq!(ScanReader::new(&[]).receive_and_extend(0).unwrap(), 0);
        assert_eq!(ScanReader::new(&[]).receive_and_extend(16).unwrap(), 32768);
    }

    #[test]
    fn scan_reader_destuffs_ff00_and_reads_msb_first_across_bytes() {
        // 0xFF 0x00 is a literal 0xFF data byte; 0x3C follows.
        let entropy = [0xFF, 0x00, 0x3C];
        let mut r = ScanReader::new(&entropy);
        assert_eq!(r.receive(4).unwrap(), 0b1111); // top 4 of the literal 0xFF
        assert_eq!(r.receive(8).unwrap(), 0b1111_0011); // low 4 of 0xFF + top 4 of 0x3C
        assert_eq!(r.receive(4).unwrap(), 0b1100); // low 4 of 0x3C
    }

    #[test]
    fn restart_consumes_rstm_and_byte_aligns() {
        // 3 bits of 0x55, then RST0 (0xFF 0xD0), then 0xAA.
        let entropy = [0x55, 0xFF, MARKER_RST0, 0xAA];
        let mut r = ScanReader::new(&entropy);
        let _ = r.receive(3).unwrap();
        r.restart().unwrap();
        assert_eq!(r.byte_pos, 3); // past 0x55, 0xFF, 0xD0
        assert_eq!(r.receive(8).unwrap(), 0xAA);

        // A wrong (non-RST) marker is a typed error.
        let wrong = [0x55, 0xFF, MARKER_DHT, 0xAA];
        let mut r2 = ScanReader::new(&wrong);
        let _ = r2.receive(3).unwrap();
        assert!(matches!(
            r2.restart(),
            Err(Error::LjpegUnexpectedMarker { marker: MARKER_DHT })
        ));

        // The wrong RSTm number (RST1 where RST0 is expected) is also typed.
        let wrong_rst = [0xFF, 0xD1, 0x00];
        let mut r3 = ScanReader::new(&wrong_rst);
        assert!(matches!(
            r3.restart(),
            Err(Error::LjpegUnexpectedMarker { marker: 0xD1 })
        ));
    }

    #[test]
    fn malformed_inputs_are_typed_errors_never_panics() {
        // A non-SOF3 SOF process.
        let mut sof0 = vec![0xFF, MARKER_SOI];
        sof0.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x02, 0x00, 0x02, 0x01]);
        assert!(matches!(
            parse_headers(&sof0),
            Err(Error::LjpegUnsupportedProcess { sof: 0xC0 })
        ));

        // Nf = 3.
        let mut nf3 = vec![0xFF, MARKER_SOI];
        nf3.extend_from_slice(&[
            0xFF,
            MARKER_SOF3,
            0x00,
            0x11,
            0x08,
            0x00,
            0x02,
            0x00,
            0x02,
            0x03,
            0x00,
            0x11,
            0x00,
            0x01,
            0x11,
            0x00,
            0x02,
            0x11,
            0x00,
        ]);
        assert!(matches!(
            parse_headers(&nf3),
            Err(Error::LjpegUnsupportedComponentCount { nf: 3 })
        ));

        // Ss = 0 and Ss = 8 (predictor out of range).
        let predictor_header = |ss: u8| -> Vec<u8> {
            let (bits, huffval) = minimal_table_bytes();
            let mut d = vec![0xFF, MARKER_SOI];
            d.extend(sof3_segment(8, 2, 2));
            d.extend(dht_segment(0, &bits, &huffval));
            d.extend(sos_segment(ss, 0, 0));
            d.extend_from_slice(&[0x00, 0xFF, MARKER_EOI]);
            d
        };
        assert!(matches!(
            parse_headers(&predictor_header(0)),
            Err(Error::LjpegUnsupportedPredictor { predictor: 0 })
        ));
        assert!(matches!(
            parse_headers(&predictor_header(8)),
            Err(Error::LjpegUnsupportedPredictor { predictor: 8 })
        ));

        // HUFFVAL length ≠ sum(BITS): Lh leaves room for one HUFFVAL byte but
        // BITS sums to two.
        let mut bad_dht = vec![0xFF, MARKER_SOI];
        bad_dht.extend(sof3_segment(8, 2, 2));
        let mut dht = vec![0xFF, MARKER_DHT, 0x00, 20, 0x00]; // Lh = 20, Tc/Th = 0
        dht.extend_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // sum(BITS) = 2
        dht.push(0x05); // only one HUFFVAL byte fits the declared length
        bad_dht.extend(dht);
        bad_dht.extend(sos_segment(1, 0, 0));
        bad_dht.extend_from_slice(&[0x00, 0xFF, MARKER_EOI]);
        assert!(matches!(
            parse_headers(&bad_dht),
            Err(Error::LjpegInvalidHuffmanTable)
        ));

        // A header cut mid-segment (SOF3 ends after the Y field).
        let cut = vec![
            0xFF,
            MARKER_SOI,
            0xFF,
            MARKER_SOF3,
            0x00,
            0x0B,
            0x08,
            0x00,
            0x02,
        ];
        assert!(matches!(parse_headers(&cut), Err(Error::Truncated { .. })));

        // A scan that ends mid-code: the 16-bit code is started but the stream
        // ends before it resolves.
        let bits = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let table = HuffTable::build(&bits, vec![5u8, 10u8]).unwrap();
        let mut truncated = ScanReader::new(&[0x80]); // starts "1000...", then EOF
        assert!(matches!(
            truncated.decode_difference(&table),
            Err(Error::Truncated { .. })
        ));

        // A marker (0xFF 0xC4) where a code was expected.
        let mut marker_stream = ScanReader::new(&[0xFF, MARKER_DHT, 0x00]);
        assert!(matches!(
            marker_stream.decode_difference(&table),
            Err(Error::LjpegUnexpectedMarker { marker: MARKER_DHT })
        ));
    }

    // ── Unit coverage for the helpers ─────────────────────────────────────

    #[test]
    fn pow2_matches_the_naive_definition() {
        for n in 0..=16u32 {
            assert_eq!(pow2(n), 2u32.pow(n));
        }
    }

    #[test]
    fn missing_soi_is_a_typed_error() {
        assert!(matches!(
            parse_headers(&[0x00, 0x01, 0x02]),
            Err(Error::LjpegMissingMarker { marker: MARKER_SOI })
        ));
        assert!(matches!(
            parse_headers(&[]),
            Err(Error::LjpegMissingMarker { marker: MARKER_SOI })
        ));
    }

    #[test]
    fn oversubscribed_huffman_table_is_rejected() {
        // Three 1-bit codes cannot exist (only two 1-bit codes fit): the code
        // space over-subscribes at length 1.
        let bits = [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!(matches!(
            HuffTable::build(&bits, vec![1, 2, 3]),
            Err(Error::LjpegInvalidHuffmanTable)
        ));
    }

    #[test]
    fn selects_the_dc_table_the_scan_names() {
        // Two DC tables (destinations 0 and 1); the scan selects destination 1,
        // whose single 1-bit code maps to symbol 0, so decoding "0" yields a
        // zero difference.
        let (bits, _huffval) = minimal_table_bytes();
        let mut data = vec![0xFF, MARKER_SOI];
        data.extend(sof3_segment(8, 1, 1));
        data.extend(dht_segment(0, &bits, &[0x07])); // dest 0 → symbol 7
        data.extend(dht_segment(1, &bits, &[0x00])); // dest 1 → symbol 0
        data.extend(sos_segment(1, 0, 1)); // Td = 1
        let offset = data.len();
        data.extend_from_slice(&[0x00, 0xFF, MARKER_EOI]);

        let (_frame, table, off) = parse_headers(&data).unwrap();
        assert_eq!(off, offset);
        let mut r = ScanReader::new(&data[off..]);
        // Code "0" → symbol 0 → category 0 → difference 0.
        assert_eq!(r.decode_difference(&table).unwrap(), 0);
    }
}
