//! SOF-3 lossless-JPEG decoder — `SPEC-036` (entropy front-end) +
//! `SPEC-037` (single-component reconstruction) + `SPEC-046` (two-component
//! `Nf=2` column-interleave decode + de-interleave; `DEC-037`); `DEC-035`.
//!
//! A DNG `Compression 7` sensor plane is a lossless-JPEG (SOF-3, ITU-T T.81
//! Annex H) bitstream. This module decodes it in two layers:
//!
//! - **The entropy front-end (`SPEC-036`)** turns the stream, down to the
//!   Huffman-coded scan, into the ordered signed **difference** values (plus
//!   the parsed frame parameters): [`parse_headers`], [`HuffTable`],
//!   [`ScanReader`].
//! - **Reconstruction (`SPEC-037`)** turns those differences into the exact
//!   caller-owned `u16` sample plane: [`reconstruct`] and the one-call
//!   [`decode_into`]. This is the lossless predictor model of ITU-T T.81 Annex
//!   H.1 — predictors 1–7 (Table H.1), the initial prediction `2^(P−Pt−1)`, the
//!   §H.2.1 line/scan/restart boundary rules, the point transform `Pt`, and the
//!   per-restart predictor reset — all in the point-transformed domain, modular
//!   `2^16` via explicit `wrapping_*` (`DEC-018`), writing into the caller's
//!   buffer with no allocation (`DEC-016`).
//!
//! What the front-end does:
//!
//! - [`parse_headers`] walks the marker/segment structure (`SOI → {DHT | DRI |
//!   SOF3 | APPn | COM} → SOS`, ITU-T T.81 Annex B; APPn / COM are skipped by
//!   length, `DEC-039`), returning the [`Frame`]
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

// As of STAGE-011 / SPEC-043, `plane::unpack_into` routes a DNG `Compression 7`
// plane through [`decode_into`], so this module finally has a real caller. The
// allow stays until SPEC-045 settles the module's public surface (DEC-036): some
// `pub(crate)` surface here is still reached only from this module's own `mod
// tests` or, under the `fuzzing` feature, the `__fuzz_ljpeg` hook in `lib.rs`, so
// `dead_code` would still fire in the default build without it. It is scoped to
// exactly this module.
#![allow(dead_code)]

use crate::Error;

// SPEC-043: `plane`'s own `mod tests` builds its tier-A Compression-7 fixtures
// with the STAGE-009 round-trip encoder, so the one `encode_lj92` helper is
// re-exported crate-wide under `cfg(test)` only. This is test-only glue — it does
// not widen the module's non-test surface (DEC-036); `decode_into` stays the
// single `pub(crate)` entry point.
#[cfg(test)]
pub(crate) use tests::encode_lj92;

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
/// First application segment; the sixteen are `0xE0..=0xEF` (`APP0..APP15`).
const MARKER_APP0: u8 = 0xE0;
/// Last application segment (`APP15`).
const MARKER_APP15: u8 = 0xEF;
/// Comment segment.
const MARKER_COM: u8 = 0xFE;

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

/// The SOF-3 frame parameters plus the scan parameters this front-end needs.
/// `SPEC-037` reads these to reconstruct a single-component plane; `SPEC-046`
/// adds the two-component (`Nf=2`) column-interleave case (`DEC-037`).
/// `SPEC-036` only populates them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frame {
    /// Sample precision `P` (bits per sample), `2..=16` for lossless.
    pub(crate) precision: u8,
    /// Number of lines `Y` (image height).
    pub(crate) lines: u16,
    /// Samples per line `X` (image width) — **per component**. For `Nf=2` the
    /// de-interleaved monochrome plane is `2·X` wide (`SPEC-046`).
    pub(crate) samples_per_line: u16,
    /// Point transform `Pt` (= `Al` from SOS, `0..=15`); the right shift
    /// `SPEC-037` applies during reconstruction.
    pub(crate) point_transform: u8,
    /// Predictor selector `Ss` (`1..=7`); which of the seven lossless
    /// predictors `SPEC-037` applies.
    pub(crate) predictor: u8,
    /// Restart interval `Ri` from DRI (`0` = no restart markers).
    pub(crate) restart_interval: u16,
    /// Number of frame components `Nf` (`1` or `2`). `SPEC-037` decodes the
    /// single-component case; `SPEC-046` decodes the two-component
    /// column-interleave case (both held Compression-7 files — `DEC-037`).
    /// `Nf` outside `{1,2}` is rejected at parse time.
    pub(crate) components: u8,
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

/// Parse a one- or two-component SOF-3 header: walk `SOI → {DHT | DRI | SOF3} →
/// SOS` and return the [`Frame`], one DC [`HuffTable`] **per component** (in
/// component order — each component selects its own table by `Td`), and the
/// byte offset into `data` where the entropy-coded scan begins. The returned
/// `Vec` has `frame.components` entries (`SPEC-046`); for `Nf=1` that is one.
///
/// # Errors
///
/// A typed [`Error`] on every malformed input, never a panic:
/// [`Error::LjpegMissingMarker`] (no SOI, a scan with no frame/table, EOI
/// before a scan, or a segment length that does not fit),
/// [`Error::LjpegUnsupportedProcess`] (a SOF that is not SOF3),
/// [`Error::LjpegUnsupportedComponentCount`] (`Nf` outside `{1,2}`, or the
/// scan's `Ns` not equal to the frame's `Nf`),
/// [`Error::LjpegUnsupportedPrecision`] (`P` outside `2..=16`),
/// [`Error::LjpegUnsupportedPredictor`] (`Ss` outside `1..=7`),
/// [`Error::LjpegInvalidHuffmanTable`] (a malformed DHT),
/// [`Error::LjpegUnexpectedMarker`] (a marker where none belongs), and
/// [`Error::Truncated`] (EOF inside a segment).
pub(crate) fn parse_headers(data: &[u8]) -> Result<(Frame, Vec<HuffTable>, usize), Error> {
    // SOI must open the stream.
    if !matches!(data.first(), Some(&0xFF)) || !matches!(data.get(1), Some(&MARKER_SOI)) {
        return Err(Error::LjpegMissingMarker { marker: MARKER_SOI });
    }
    let mut pos: usize = 2;

    let mut precision: Option<u8> = None;
    let mut components: u8 = 1;
    let mut lines: u16 = 0;
    let mut samples_per_line: u16 = 0;
    let mut restart_interval: u16 = 0;
    // DC tables keyed by their destination identifier `Th`.
    let mut dc_tables: Vec<(u8, HuffTable)> = Vec::new();

    loop {
        let marker = next_marker(data, &mut pos)?;
        match marker {
            MARKER_SOF3 => {
                let (p, y, x, nf) = parse_frame_header(data, &mut pos)?;
                precision = Some(p);
                lines = y;
                samples_per_line = x;
                components = nf;
            }
            MARKER_DHT => parse_dht(data, &mut pos, &mut dc_tables)?,
            MARKER_DRI => restart_interval = parse_dri(data, &mut pos)?,
            MARKER_SOS => {
                let (predictor, point_transform, tds) = parse_scan_header(data, &mut pos)?;
                let precision = precision.ok_or(Error::LjpegMissingMarker {
                    marker: MARKER_SOF3,
                })?;
                // The scan's component count `Ns` must equal the frame's `Nf`.
                if tds.len() != usize::from(components) {
                    return Err(Error::LjpegUnsupportedComponentCount {
                        nf: u8::try_from(tds.len()).unwrap_or(u8::MAX),
                    });
                }
                // Resolve each component's DC table by its own `Td`. A table is
                // cloned, not removed, so two components may share one
                // destination (both held Compression-7 files reference `Td=0`
                // for both components — `SPEC-046` / `DEC-037`).
                let mut component_tables: Vec<HuffTable> = Vec::with_capacity(tds.len());
                for td in tds {
                    let table = find_table(&dc_tables, td)
                        .ok_or(Error::LjpegMissingMarker { marker: MARKER_DHT })?;
                    component_tables.push(table.clone());
                }
                let frame = Frame {
                    precision,
                    lines,
                    samples_per_line,
                    point_transform,
                    predictor,
                    restart_interval,
                    components,
                };
                return Ok((frame, component_tables, pos));
            }
            // APPn / COM carry optional metadata (camera-written: the Pentax
            // K-3 III puts an APP0 between DHT and SOS). They are skipped by
            // their declared length, never parsed (`DEC-039`).
            MARKER_APP0..=MARKER_APP15 | MARKER_COM => skip_segment(data, &mut pos)?,
            MARKER_EOI => return Err(Error::LjpegMissingMarker { marker: MARKER_SOS }),
            m if is_other_sof(m) => return Err(Error::LjpegUnsupportedProcess { sof: m }),
            other => return Err(Error::LjpegUnexpectedMarker { marker: other }),
        }
    }
}

/// Return a reference to the DC table with destination `th`, if present. The
/// table is cloned by the caller rather than removed, so two components in a
/// two-component scan may both select one shared destination (`SPEC-046`).
fn find_table(tables: &[(u8, HuffTable)], th: u8) -> Option<&HuffTable> {
    tables.iter().find(|(t, _)| *t == th).map(|(_, tbl)| tbl)
}

/// Parse a SOF3 frame header (ITU-T T.81 B.2.2): `Lf P Y X Nf` then `Nf`
/// component specifications. Accepts `Nf ∈ {1, 2}` — the single-component case
/// (`SPEC-037`) and the two-component column-interleave case both held
/// Compression-7 files use (`SPEC-046` / `DEC-037`); `Nf` outside `{1,2}`
/// (Bayer/CFA 3+) is rejected with [`Error::LjpegUnsupportedComponentCount`].
/// Returns `(precision, lines, samples_per_line, nf)`.
fn parse_frame_header(data: &[u8], pos: &mut usize) -> Result<(u8, u16, u16, u8), Error> {
    let lf = read_u16_be(data, pos)?;
    let precision = read_u8(data, pos)?;
    let y = read_u16_be(data, pos)?;
    let x = read_u16_be(data, pos)?;
    let nf = read_u8(data, pos)?;

    if nf != 1 && nf != 2 {
        return Err(Error::LjpegUnsupportedComponentCount { nf });
    }
    // Lf must equal `8 + 3*Nf` (header fields + Nf 3-byte component specs).
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

    // Each component's spec: Ci, Hi/Vi, Tqi. The front-end stores none of them:
    // for lossless there is no quantisation, and both held two-component files
    // are `H=V=1` (the §12 probe / `DEC-037`), so the interleave is one comp0
    // then one comp1 sample per grid position — no subsampling to track.
    // (Subsampled `H`/`V`≠1 is out of scope — `DEC-037` — and no held file uses
    // it.) The bytes are consumed to reach the next marker.
    for _ in 0..nf {
        let _ci = read_u8(data, pos)?;
        let _hv = read_u8(data, pos)?;
        let _tq = read_u8(data, pos)?;
    }

    Ok((precision, y, x, nf))
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

/// Skip a length-delimited marker segment the decoder does not interpret —
/// APPn / COM (ITU-T T.81 B.2.4.5–B.2.4.6). The 2-byte length `L` counts
/// itself, so the payload is `L - 2` bytes; `pos` advances past it. Same
/// length-read idiom as [`parse_dht`]. A length below 2, or one that runs past
/// the buffer, is [`Error::Truncated`] — never a panic (`DEC-039`).
fn skip_segment(data: &[u8], pos: &mut usize) -> Result<(), Error> {
    let at = *pos;
    let len = usize::from(read_u16_be(data, pos)?);
    let payload = len.checked_sub(2).ok_or(Error::Truncated { at, len })?;
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
    *pos = seg_end;
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
/// transform. Accepts `Ns ∈ {1, 2}` (the caller requires `Ns == Nf`); reads
/// each component's own DC table destination `Td`. `Ss`/`Al` stay scan-level
/// (one predictor and one point transform shared by the interleaved
/// components, as both held files use — `SPEC-046`). Returns `(predictor,
/// point_transform, td_per_component)`, the `Td` list in component order.
fn parse_scan_header(data: &[u8], pos: &mut usize) -> Result<(u8, u8, Vec<u8>), Error> {
    let ls = read_u16_be(data, pos)?;
    let ns = read_u8(data, pos)?;
    if ns != 1 && ns != 2 {
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

    // One selector per component: Csj then Tdj/Taj. Each component chooses its
    // own DC table (high nibble of the Td/Ta byte); collect them in component
    // order so `parse_headers` resolves one table per component rather than
    // hard-coding a single shared table.
    let mut tds: Vec<u8> = Vec::with_capacity(usize::from(ns));
    for _ in 0..ns {
        let _cs = read_u8(data, pos)?; // scan component selector
        let td_ta = read_u8(data, pos)?;
        let td = td_ta.checked_div(16).unwrap_or(0); // DC table destination (high nibble)
        tds.push(td);
    }
    let ss = read_u8(data, pos)?; // predictor
    let _se = read_u8(data, pos)?; // 0 in lossless
    let ah_al = read_u8(data, pos)?;
    let al = ah_al.checked_rem(16).unwrap_or(0); // point transform (low nibble)

    if !(1..=7).contains(&ss) {
        return Err(Error::LjpegUnsupportedPredictor { predictor: ss });
    }

    Ok((ss, al, tds))
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

// ─────────────────────────────────────────────────────────────────────────────
// Reconstruction (ITU-T T.81 Annex H.1) — SPEC-037
//
// Turns SPEC-036's per-sample signed differences into the caller's `u16` plane:
// predictors 1–7 (Table H.1), the initial prediction `2^(P−Pt−1)`, the §H.2.1
// line/scan/restart boundary rules, the point transform `Pt`, and the
// per-restart predictor reset. All prediction arithmetic is in the
// point-transformed domain, modular `2^16` via explicit `wrapping_*` (`DEC-018`
// — intent, not a silencing cast); the output is caller-owned with no
// allocation (`DEC-016`).
// ─────────────────────────────────────────────────────────────────────────────

/// Floor division by two of a signed working value — the `>> 1` of T.81 Table
/// H.1 predictors 5/6/7, which is an arithmetic (sign-preserving) shift, i.e. a
/// floor toward negative infinity. Done without the `/` or `>>` operators the
/// crate root denies (`clippy::arithmetic_side_effects`): `wrapping_div`/
/// `wrapping_rem` by the constant `2` never panic (the divisor is neither `0`
/// nor the `i32::MIN / -1` overflow case), and the correction turns truncation
/// toward zero into a true floor for negative odd inputs.
fn floor_div2(x: i32) -> i32 {
    let q = x.wrapping_div(2);
    let r = x.wrapping_rem(2);
    if r != 0 && x < 0 {
        q.wrapping_sub(1)
    } else {
        q
    }
}

/// The T.81 Table H.1 lossless predictor `Px` for selector `sel` (`1..=7`),
/// from the point-transformed neighbours `Ra` (left), `Rb` (above) and `Rc`
/// (above-left). Working values are `i32` so predictors 4/5/6 can go negative
/// and the `>> 1` of 5/6/7 floors correctly (T.81 H.1); the final modular
/// `2^16` reduction happens in [`reconstruct`]. `sel` is validated `1..=7` by
/// [`parse_headers`]; the catch-all never panics on an unexpected value, it
/// falls back to `Ra` (predictor 1).
fn predict(sel: u8, ra: i32, rb: i32, rc: i32) -> i32 {
    match sel {
        1 => ra,
        2 => rb,
        3 => rc,
        4 => ra.wrapping_add(rb).wrapping_sub(rc),
        5 => ra.wrapping_add(floor_div2(rb.wrapping_sub(rc))),
        6 => rb.wrapping_add(floor_div2(ra.wrapping_sub(rc))),
        7 => floor_div2(ra.wrapping_add(rb)),
        _ => ra,
    }
}

/// Read a already-reconstructed neighbour `back` samples before linear index
/// `idx` as an `i32`, or `0` when it would be out of range. Callers only read a
/// neighbour where the boundary rules guarantee it exists; the bounded form
/// keeps the function total (no slice indexing, no panicking subtraction) for
/// `no-panics-on-untrusted-input`.
fn neighbour(out: &[u16], idx: usize, back: usize) -> i32 {
    match idx.checked_sub(back) {
        Some(j) => out.get(j).copied().map(i32::from).unwrap_or(0),
        None => 0,
    }
}

/// Reconstruct a single-component SOF-3 plane from `reader`'s decoded
/// difference stream into the caller-owned `out` (ITU-T T.81 Annex H.1).
///
/// `out` must hold exactly `frame.lines × frame.samples_per_line` samples
/// (else [`Error::LjpegPlaneWrongLength`]); it is then filled **row-major** by
/// decoding one difference per sample ([`ScanReader::decode_difference`]) and
/// adding the prediction. For each sample at `(r, c)`:
///
/// - the first sample of the scan **and** the first sample of each restart
///   interval use the initial prediction `2^(P−Pt−1)`;
/// - the rest of the first line (`r == 0`) uses `Ra` (the left neighbour);
/// - the first column of each later line (`c == 0`) uses `Rb` (the sample
///   above);
/// - every other sample uses the selected predictor ([`predict`]).
///
/// The reconstructed value is `(Px + difference) mod 2^16` (`wrapping_*`,
/// `DEC-018`) computed in the point-transformed domain; the stored output is
/// that value `<< Pt` (applied as a final modular multiply, so the predictor
/// neighbours above are the pre-shift values). When `frame.restart_interval >
/// 0`, [`ScanReader::restart`] is called at each interval boundary and the
/// prediction resets to the scan-start rule for the next sample.
///
/// No allocation occurs (`DEC-016`): the only buffer is the caller's `out`.
///
/// # Errors
///
/// [`Error::LjpegPlaneWrongLength`] if `out` is the wrong length, or any typed
/// [`Error`] surfaced by `reader` (a truncated scan, a bad code, a missing or
/// wrong restart marker). Never a panic on malformed input.
pub(crate) fn reconstruct(
    frame: &Frame,
    table: &HuffTable,
    reader: &mut ScanReader<'_>,
    out: &mut [u16],
) -> Result<(), Error> {
    let width = usize::from(frame.samples_per_line);
    let height = usize::from(frame.lines);

    // Length precondition (the one check a caller-owned, no-alloc path can make
    // before writing). `checked_mul` keeps the lint happy and covers the
    // (unreachable on 32/64-bit targets) overflow: a plane larger than
    // `usize::MAX` cannot match any real buffer.
    let expected = match width.checked_mul(height) {
        Some(e) => e,
        None => {
            return Err(Error::LjpegPlaneWrongLength {
                expected: usize::MAX,
                got: out.len(),
            });
        }
    };
    if out.len() != expected {
        return Err(Error::LjpegPlaneWrongLength {
            expected,
            got: out.len(),
        });
    }
    if expected == 0 {
        // A zero-sample plane (either dimension 0) has nothing to decode.
        return Ok(());
    }

    // Initial prediction `2^(P − Pt − 1)`. `P` is `2..=16` and `Pt` is `0..=15`
    // (both from the parsed header); a `Pt >= P` (malformed, since valid
    // streams have `Pt < P`) makes the exponent negative — treated as `0`
    // rather than panicking.
    let p = i32::from(frame.precision);
    let pt = frame.point_transform;
    let initial: i32 = {
        let exp = p.wrapping_sub(i32::from(pt)).wrapping_sub(1);
        if exp < 0 {
            0
        } else {
            i32::try_from(pow2(u32::try_from(exp).unwrap_or(0))).unwrap_or(0)
        }
    };

    let restart_interval = usize::from(frame.restart_interval);
    let selector = frame.predictor;

    let mut idx: usize = 0;
    // Samples decoded in the current restart interval (resets to 0 at the scan
    // start and after each restart). `== 0` is exactly the set of samples that
    // take the initial prediction: the first sample of the scan, and the first
    // sample after each restart.
    let mut count_in_interval: usize = 0;

    for r in 0..height {
        for c in 0..width {
            if restart_interval > 0 && count_in_interval == restart_interval {
                reader.restart()?;
                count_in_interval = 0;
            }

            let diff = reader.decode_difference(table)?;

            let px: i32 = if count_in_interval == 0 {
                initial
            } else if r == 0 {
                // Rest of the first line: Ra (left).
                neighbour(out, idx, 1)
            } else if c == 0 {
                // First column of a later line: Rb (above).
                neighbour(out, idx, width)
            } else {
                let ra = neighbour(out, idx, 1);
                let rb = neighbour(out, idx, width);
                let rc = neighbour(out, idx, width.wrapping_add(1));
                predict(selector, ra, rb, rc)
            };

            // Sample = (Px + difference) mod 2^16, in the point-transformed
            // domain. `& 0xFFFF` on the two's-complement sum is the modular
            // reduction (a negative sum maps to its 0..=65535 residue).
            let sample = px.wrapping_add(diff) & 0xFFFF;
            if let Some(slot) = out.get_mut(idx) {
                *slot = u16::try_from(sample).unwrap_or(0);
            }

            idx = idx.wrapping_add(1);
            count_in_interval = count_in_interval.wrapping_add(1);
        }
    }

    // Point transform: the stored output is `reconstructed << Pt`. Done as a
    // modular `2^16` multiply so a malformed over-range value wraps rather than
    // panics; for honest input `reconstructed < 2^(P−Pt)` so `<< Pt` never
    // overflows 16 bits. `Pt == 0` (every held corpus file) is a no-op.
    if pt > 0 {
        let factor = u16::try_from(pow2(u32::from(pt))).unwrap_or(1);
        for slot in out.iter_mut() {
            *slot = slot.wrapping_mul(factor);
        }
    }

    Ok(())
}

/// The loop-invariant prediction parameters of a two-component reconstruction:
/// the de-interleaved plane width `2·X`, the shared predictor selector, and the
/// initial prediction `2^(P−Pt−1)`. Grouped so [`predict_component`] stays
/// within the crate's argument-count lint.
#[derive(Clone, Copy)]
struct CompPredict {
    /// `2 · samples_per_line` — the width of the de-interleaved plane.
    plane_width: usize,
    /// The scan's predictor selector `Ss` (shared by both components).
    selector: u8,
    /// The initial prediction `2^(P−Pt−1)`.
    initial: i32,
}

/// The lossless prediction `Px` for one component of a two-component
/// column-interleave scan (`SPEC-046`), applying the §H.2.1 boundary rules
/// **independently per component**. `slot` is the component's own linear index
/// in the `2·X`-wide plane; its same-component neighbours are two output
/// columns back (`Ra`), one plane row back (`Rb`), and one of each (`Rc`). This
/// is the per-component counterpart of the single-sample selection inside
/// [`reconstruct`].
///
/// - the first sample of the scan (and of each restart interval, when
///   `is_interval_start`) uses the initial prediction `2^(P−Pt−1)`;
/// - the rest of the first line (`row == 0`) uses `Ra` (same-component left);
/// - the first column of a later line (`col == 0`) uses `Rb` (same-component
///   above);
/// - every other sample uses the selected predictor ([`predict`]).
fn predict_component(
    out: &[u16],
    cfg: CompPredict,
    slot: usize,
    is_interval_start: bool,
    row: usize,
    col: usize,
) -> i32 {
    if is_interval_start {
        cfg.initial
    } else if row == 0 {
        // Ra: same-component left neighbour, two output columns back.
        neighbour(out, slot, 2)
    } else if col == 0 {
        // Rb: same-component sample directly above, one plane row back.
        neighbour(out, slot, cfg.plane_width)
    } else {
        let ra = neighbour(out, slot, 2);
        let rb = neighbour(out, slot, cfg.plane_width);
        let rc = neighbour(out, slot, cfg.plane_width.wrapping_add(2));
        predict(cfg.selector, ra, rb, rc)
    }
}

/// Reconstruct a two-component (`Nf=2`) column-interleave SOF-3 plane from
/// `reader`'s interleaved difference stream into the caller-owned `out`
/// (`SPEC-046` / `DEC-037`, ITU-T T.81 Annex H.1).
///
/// The two SOF-3 components are the two column phases of one monochrome plane.
/// `out` must hold exactly `2 × frame.samples_per_line × frame.lines` samples
/// (else [`Error::LjpegPlaneWrongLength`]). It is filled MCU by MCU in scan
/// order: at each grid position `(x, y)` component 0 is decoded, then component
/// 1, and written column-interleaved —
/// `out[y·2X + 2x] = comp0`, `out[y·2X + 2x + 1] = comp1`.
///
/// Prediction state is held **per component**: predictor 1's `Ra` is the same
/// component's own left neighbour (two output columns back), and the §H.2.1
/// first-line / first-column / initial-prediction rules apply to each component
/// independently ([`predict_component`]). `tables[k]` is component `k`'s DC
/// Huffman table (both held files share one `Td=0` table, but each component
/// selects its own). The reconstructed value is `(Px + difference) mod 2^16`
/// (`wrapping_*`, `DEC-018`) in the point-transformed domain; the stored output
/// is that value `<< Pt` (both held files use `Pt=0`, a no-op). When
/// `frame.restart_interval > 0`, [`ScanReader::restart`] is called at each
/// interval boundary (counted in MCUs) and both components' prediction resets.
/// No allocation occurs (`DEC-016`).
///
/// # Errors
///
/// [`Error::LjpegPlaneWrongLength`] if `out` is the wrong length, or any typed
/// [`Error`] surfaced by `reader` (a truncated scan, a bad code, a missing
/// table, a wrong restart marker). Never a panic on malformed input.
fn reconstruct_two_component(
    frame: &Frame,
    tables: &[HuffTable],
    reader: &mut ScanReader<'_>,
    out: &mut [u16],
) -> Result<(), Error> {
    let width = usize::from(frame.samples_per_line); // X, per component
    let height = usize::from(frame.lines);

    // The de-interleaved plane is `2·X` wide. The length precondition (the one
    // check a caller-owned, no-alloc path makes before writing) keys on `Nf=2`:
    // `out.len() == 2 · X · Y`. `checked_mul` keeps the lint happy and treats
    // the (unreachable on real buffers) overflow as a length that cannot match.
    let plane_width = match width.checked_mul(2) {
        Some(w) => w,
        None => {
            return Err(Error::LjpegPlaneWrongLength {
                expected: usize::MAX,
                got: out.len(),
            });
        }
    };
    let expected = match plane_width.checked_mul(height) {
        Some(e) => e,
        None => {
            return Err(Error::LjpegPlaneWrongLength {
                expected: usize::MAX,
                got: out.len(),
            });
        }
    };
    if out.len() != expected {
        return Err(Error::LjpegPlaneWrongLength {
            expected,
            got: out.len(),
        });
    }
    if expected == 0 {
        return Ok(());
    }

    // One DC table per component (resolved by `Td` in `parse_headers`). A
    // malformed `tables` shorter than two components is a typed error, never an
    // index panic.
    let table0 = tables.first().ok_or(Error::LjpegInvalidHuffmanTable)?;
    let table1 = tables.get(1).ok_or(Error::LjpegInvalidHuffmanTable)?;

    // Initial prediction `2^(P − Pt − 1)`, as in the single-component path.
    let p = i32::from(frame.precision);
    let pt = frame.point_transform;
    let initial: i32 = {
        let exp = p.wrapping_sub(i32::from(pt)).wrapping_sub(1);
        if exp < 0 {
            0
        } else {
            i32::try_from(pow2(u32::try_from(exp).unwrap_or(0))).unwrap_or(0)
        }
    };

    let restart_interval = usize::from(frame.restart_interval);
    let cfg = CompPredict {
        plane_width,
        selector: frame.predictor,
        initial,
    };

    // `idx` is component 0's slot for the current MCU; component 1 is `idx + 1`.
    // `count_in_interval` counts MCUs in the current restart interval; `== 0`
    // marks the MCU whose two components both take the initial prediction (the
    // scan start and each sample after a restart).
    let mut idx: usize = 0;
    let mut count_in_interval: usize = 0;

    for y in 0..height {
        for x in 0..width {
            if restart_interval > 0 && count_in_interval == restart_interval {
                reader.restart()?;
                count_in_interval = 0;
            }
            let is_interval_start = count_in_interval == 0;

            // Component 0, then component 1 — the natural MCU order. Each
            // difference is decoded with the component's own table; each
            // prediction reads only the component's own neighbours.
            let diff0 = reader.decode_difference(table0)?;
            let px0 = predict_component(out, cfg, idx, is_interval_start, y, x);
            let sample0 = px0.wrapping_add(diff0) & 0xFFFF;
            if let Some(slot) = out.get_mut(idx) {
                *slot = u16::try_from(sample0).unwrap_or(0);
            }

            let idx1 = idx.wrapping_add(1);
            let diff1 = reader.decode_difference(table1)?;
            let px1 = predict_component(out, cfg, idx1, is_interval_start, y, x);
            let sample1 = px1.wrapping_add(diff1) & 0xFFFF;
            if let Some(slot) = out.get_mut(idx1) {
                *slot = u16::try_from(sample1).unwrap_or(0);
            }

            idx = idx.wrapping_add(2);
            count_in_interval = count_in_interval.wrapping_add(1);
        }
    }

    // Point transform: stored output is `reconstructed << Pt`, a modular `2^16`
    // multiply (both held files use `Pt == 0`, a no-op). Mirrors `reconstruct`.
    if pt > 0 {
        let factor = u16::try_from(pow2(u32::from(pt))).unwrap_or(1);
        for slot in out.iter_mut() {
            *slot = slot.wrapping_mul(factor);
        }
    }

    Ok(())
}

/// One-call SOF-3 decode: [`parse_headers`] → [`ScanReader`] over the entropy
/// bytes → reconstruction (single-component [`reconstruct`] for `Nf=1`,
/// [`reconstruct_two_component`] for `Nf=2`), returning the parsed [`Frame`].
/// This is the entry point STAGE-011 (SPEC-043) wires `Compression 7` to,
/// through `plane::unpack_into`; `out` is the caller-owned `u16` plane
/// (`DEC-016`), sized `Nf · samples_per_line · lines`. The length check lives
/// in the chosen reconstruction function.
///
/// # Errors
///
/// Any typed [`Error`] from header parsing, the length check
/// ([`Error::LjpegPlaneWrongLength`]), or reconstruction — never a panic on
/// malformed input.
pub(crate) fn decode_into(data: &[u8], out: &mut [u16]) -> Result<Frame, Error> {
    let (frame, tables, offset) = parse_headers(data)?;
    // `offset` is `<= data.len()` (parse_headers only advances through
    // bounds-checked reads), so this slice is always present; the `ok_or`
    // keeps the path total without an index.
    let entropy = data
        .get(offset..)
        .ok_or(Error::Truncated { at: offset, len: 0 })?;
    let mut reader = ScanReader::new(entropy);
    if frame.components == 2 {
        reconstruct_two_component(&frame, &tables, &mut reader, out)?;
    } else {
        // `Nf=1`: the single-component path, byte-for-byte unchanged. `tables`
        // holds exactly one entry for a well-formed single-component header.
        let table = tables
            .first()
            .ok_or(Error::LjpegMissingMarker { marker: MARKER_DHT })?;
        reconstruct(&frame, table, &mut reader, out)?;
    }
    Ok(frame)
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

        let (_frame, tables, off) = parse_headers(&data).unwrap();
        assert_eq!(off, offset);
        // One table for the single component; it is the scan-selected Td=1.
        let table = &tables[0];
        let mut r = ScanReader::new(&data[off..]);
        // Code "0" → symbol 0 → category 0 → difference 0.
        assert_eq!(r.decode_difference(table).unwrap(), 0);
    }

    // ══ SPEC-037: reconstruction ══════════════════════════════════════════
    //
    // The in-harness LJ92 **encoder** below is the stage's corpus-free oracle.
    // It is the inverse of the decoder: given the final `u16` samples it emits
    // a complete SOF-3 stream (SOI / SOF3 / DHT / optional DRI / SOS / a
    // Huffman-coded + EXTENDed + 0xFF-stuffed scan + RSTm / EOI) that
    // `decode_into` must reconstruct bit-for-bit. It is also the seed generator
    // for SPEC-038's fuzz target, so it is kept clean and parameterised.
    //
    // It uses a flat 17-symbol / 5-bit DC Huffman table (symbols = categories
    // `0..=16`): canonical code of symbol `s` is the 5-bit value `s`, so the
    // encoder emits `s` directly and `HuffTable::build` decodes it back. A
    // realistic variable-length table is SPEC-036's ground already; here a flat
    // table keeps the encoder a trivially-correct inverse.

    const SYMBOL_BITS: u32 = 5;

    /// The flat DC table's `BITS`/`HUFFVAL`: 17 codes all of length 5, symbols
    /// `0..=16` in order.
    fn flat_table_bytes() -> ([u8; 16], Vec<u8>) {
        let mut bits = [0u8; 16];
        bits[4] = 17; // 17 codes of length 5 (index 4 == length 5)
        let huffval: Vec<u8> = (0u8..=16).collect();
        (bits, huffval)
    }

    /// MSB-first bit sink with JPEG `0xFF 0x00` byte-stuffing and 1-bit pad to a
    /// byte boundary — the encoder's counterpart to `ScanReader`.
    struct BitWriter {
        bytes: Vec<u8>,
        acc: u32,
        nbits: u32,
    }

    impl BitWriter {
        fn new() -> BitWriter {
            BitWriter {
                bytes: Vec::new(),
                acc: 0,
                nbits: 0,
            }
        }

        fn put_bits(&mut self, value: u32, count: u32) {
            let mut i = count;
            while i > 0 {
                i -= 1;
                let bit = (value >> i) & 1;
                self.acc = (self.acc << 1) | bit;
                self.nbits += 1;
                if self.nbits == 8 {
                    self.emit_acc();
                }
            }
        }

        fn emit_acc(&mut self) {
            let b = (self.acc & 0xFF) as u8;
            self.bytes.push(b);
            if b == 0xFF {
                self.bytes.push(0x00); // stuff a literal 0xFF data byte
            }
            self.acc = 0;
            self.nbits = 0;
        }

        /// Pad the partial byte with 1-bits and flush, so a marker can follow.
        fn flush_to_byte(&mut self) {
            if self.nbits > 0 {
                let pad = 8 - self.nbits;
                self.acc = (self.acc << pad) | ((1u32 << pad) - 1);
                self.nbits = 8;
                self.emit_acc();
            }
        }

        /// Emit `0xFF <code>` (caller must have flushed to a byte boundary).
        fn emit_marker(&mut self, code: u8) {
            self.bytes.push(0xFF);
            self.bytes.push(code);
        }
    }

    /// Inverse of `RECEIVE`+`EXTEND` (ITU-T T.81 §F.1.2.1 / Annex H.1.2.1):
    /// map a signed difference `d` (already reduced to `[-32768, 32767]`) to its
    /// category `SSSS` and the `SSSS`-bit value to append. `d == 0` → category
    /// 0, no bits; `d == -32768` → category 16, no bits; otherwise category =
    /// bit-length of `|d|`, value = low `SSSS` bits of `d` (of `d-1` when `d`
    /// is negative).
    fn encode_diff(d: i32) -> (u32, u32) {
        if d == 0 {
            return (0, 0);
        }
        if d == -32768 {
            return (16, 0);
        }
        let mag = d.unsigned_abs(); // 1..=32767
        let ssss = 32 - mag.leading_zeros(); // bit length, 1..=15
        let mask = (1u32 << ssss) - 1;
        let raw = if d > 0 { d as u32 } else { (d - 1) as u32 };
        (ssss, raw & mask)
    }

    /// Encode an ordered difference list into a Huffman-coded, `0xFF`-stuffed
    /// entropy scan, inserting an `RSTm` marker before every `restart_interval`
    /// samples (cycling `D0..D7`), byte-aligning first — exactly where
    /// `reconstruct` calls `ScanReader::restart`.
    fn encode_scan(diffs: &[i32], restart_interval: usize) -> Vec<u8> {
        let mut w = BitWriter::new();
        let mut rst: u8 = 0;
        for (i, &d) in diffs.iter().enumerate() {
            if restart_interval > 0 && i > 0 && i % restart_interval == 0 {
                w.flush_to_byte();
                w.emit_marker(MARKER_RST0 + rst);
                rst = (rst + 1) % 8;
            }
            let (ssss, value) = encode_diff(d);
            w.put_bits(ssss, SYMBOL_BITS); // flat table: code == symbol
            if (1..=15).contains(&ssss) {
                w.put_bits(value, ssss);
            }
        }
        w.flush_to_byte();
        w.bytes
    }

    /// The in-harness LJ92 encoder — the round-trip oracle. Given the final
    /// `u16` samples (row-major, `cols * rows`) it produces a complete
    /// single-component SOF-3 stream that `decode_into` reconstructs
    /// bit-for-bit. Prediction mirrors `reconstruct` exactly (it shares the
    /// production `predict`), computing each difference in the point-transformed
    /// domain; the differences are the inverse of the decoder's.
    pub(crate) fn encode_lj92(
        samples: &[u16],
        cols: usize,
        rows: usize,
        precision: u8,
        predictor: u8,
        point_transform: u8,
        restart_interval: usize,
    ) -> Vec<u8> {
        assert_eq!(samples.len(), cols * rows, "samples must be cols*rows");
        let pt = point_transform;
        // Pre-shift reconstructed values (the point-transformed domain).
        let v: Vec<i32> = samples.iter().map(|&s| i32::from(s >> pt)).collect();
        let initial = {
            let e = i32::from(precision) - i32::from(pt) - 1;
            if e < 0 {
                0
            } else {
                1i32 << e
            }
        };
        let mut diffs: Vec<i32> = Vec::with_capacity(samples.len());
        let mut idx = 0usize;
        let mut count = 0usize;
        for r in 0..rows {
            for c in 0..cols {
                if restart_interval > 0 && count == restart_interval {
                    count = 0;
                }
                let px = if count == 0 {
                    initial
                } else if r == 0 {
                    v[idx - 1]
                } else if c == 0 {
                    v[idx - cols]
                } else {
                    predict(predictor, v[idx - 1], v[idx - cols], v[idx - cols - 1])
                };
                let raw = (v[idx] - px).rem_euclid(65536);
                let d = if raw >= 32768 { raw - 65536 } else { raw };
                diffs.push(d);
                idx += 1;
                count += 1;
            }
        }

        let entropy = encode_scan(&diffs, restart_interval);
        let (bits, huffval) = flat_table_bytes();
        let mut data = vec![0xFF, MARKER_SOI];
        data.extend(sof3_segment(precision, rows as u16, cols as u16));
        if restart_interval > 0 {
            data.extend(dri_segment(restart_interval as u16));
        }
        data.extend(dht_segment(0, &bits, &huffval));
        data.extend(sos_segment(predictor, point_transform, 0));
        data.extend(entropy);
        data.extend_from_slice(&[0xFF, MARKER_EOI]);
        data
    }

    // ══ SPEC-046: two-component (Nf=2) column-interleave encode ════════════
    //
    // The two-component counterpart of the helpers above, so the analytic
    // round-trip covers Nf=2. A two-component SOF-3 frame carries two 3-byte
    // component specs (Lf = 8 + 3·2 = 14) and the SOS two selectors
    // (Ls = 6 + 2·2 = 10), both H=V=1, both referencing their own Td. The
    // entropy scan is the single-component `encode_scan` over an MCU-interleaved
    // difference list (comp0 diff then comp1 diff per MCU), which is exactly
    // what `reconstruct_two_component` decodes.

    /// A two-component SOF3 frame segment: Nf=2, both components H=V=1, Tq=0.
    fn sof3_segment_nf2(precision: u8, lines: u16, samples: u16) -> Vec<u8> {
        let mut seg = vec![0xFF, MARKER_SOF3];
        seg.extend_from_slice(&14u16.to_be_bytes()); // Lf = 8 + 3*2
        seg.push(precision);
        seg.extend_from_slice(&lines.to_be_bytes());
        seg.extend_from_slice(&samples.to_be_bytes()); // X, per component
        seg.push(2); // Nf
        seg.extend_from_slice(&[0x00, 0x11, 0x00]); // comp0: Ci=0, Hi/Vi=1/1, Tqi=0
        seg.extend_from_slice(&[0x01, 0x11, 0x00]); // comp1: Ci=1, Hi/Vi=1/1, Tqi=0
        seg
    }

    /// A two-component SOS scan segment: Ns=2, one predictor (`ss`) and one
    /// point transform (`al`) shared by both components, each selecting its own
    /// DC table destination (`td0` / `td1`).
    fn sos_segment_nf2(ss: u8, al: u8, td0: u8, td1: u8) -> Vec<u8> {
        let mut seg = vec![0xFF, MARKER_SOS];
        seg.extend_from_slice(&10u16.to_be_bytes()); // Ls = 6 + 2*2
        seg.push(2); // Ns
        seg.push(0x00); // Cs0
        seg.push(td0 << 4); // Td0 (high nibble), Ta0 = 0
        seg.push(0x01); // Cs1
        seg.push(td1 << 4); // Td1 (high nibble), Ta1 = 0
        seg.push(ss); // Ss (predictor)
        seg.push(0x00); // Se
        seg.push(al & 0x0F); // Ah = 0 (high nibble), Al (point transform, low)
        seg
    }

    /// The two-component in-harness LJ92 encoder (`SPEC-046`). Given the final
    /// `2·X`-wide monochrome plane (`samples`, row-major, `2·x_per_comp · rows`)
    /// it produces a complete two-component SOF-3 stream that `decode_into`
    /// reconstructs bit-for-bit. Component 0 is the even columns, component 1
    /// the odd; prediction mirrors `reconstruct_two_component` exactly (shared
    /// production `predict`), computing each component's differences in the
    /// point-transformed domain from its own two-columns-back / one-row-back
    /// neighbours, interleaved one comp0 then one comp1 per MCU. Restart (if
    /// `restart_interval > 0`) is counted in MCUs: `encode_scan` with interval
    /// `restart_interval·2` inserts an `RSTm` after exactly that many MCUs, where
    /// `reconstruct_two_component` resets prediction.
    pub(crate) fn encode_lj92_two_component(
        samples: &[u16],
        x_per_comp: usize,
        rows: usize,
        precision: u8,
        predictor: u8,
        point_transform: u8,
        restart_interval: usize,
    ) -> Vec<u8> {
        let plane_width = x_per_comp * 2;
        assert_eq!(
            samples.len(),
            plane_width * rows,
            "samples must be 2*x_per_comp*rows"
        );
        let pt = point_transform;
        // Pre-shift reconstructed values (the point-transformed domain).
        let v: Vec<i32> = samples.iter().map(|&s| i32::from(s >> pt)).collect();
        let initial = {
            let e = i32::from(precision) - i32::from(pt) - 1;
            if e < 0 {
                0
            } else {
                1i32 << e
            }
        };
        // Same-component neighbour in the pre-shift domain, or 0 out of range.
        let nb = |slot: usize, back: usize| -> i32 {
            match slot.checked_sub(back) {
                Some(j) => v.get(j).copied().unwrap_or(0),
                None => 0,
            }
        };

        let mut diffs: Vec<i32> = Vec::with_capacity(samples.len());
        let mut idx = 0usize; // comp0 slot of the current MCU
        let mut count = 0usize; // MCUs since the scan start / last restart
        for y in 0..rows {
            for xcol in 0..x_per_comp {
                if restart_interval > 0 && count == restart_interval {
                    count = 0;
                }
                let is_interval_start = count == 0;
                for k in 0..2 {
                    let slot = idx + k;
                    let px = if is_interval_start {
                        initial
                    } else if y == 0 {
                        nb(slot, 2)
                    } else if xcol == 0 {
                        nb(slot, plane_width)
                    } else {
                        predict(
                            predictor,
                            nb(slot, 2),
                            nb(slot, plane_width),
                            nb(slot, plane_width + 2),
                        )
                    };
                    let raw = (v[slot] - px).rem_euclid(65536);
                    let d = if raw >= 32768 { raw - 65536 } else { raw };
                    diffs.push(d);
                }
                idx += 2;
                count += 1;
            }
        }

        // Restart interval is per-MCU; each MCU is two diffs, so an RSTm every
        // `restart_interval` MCUs is every `restart_interval*2` diffs.
        let entropy = encode_scan(&diffs, restart_interval * 2);
        let (bits, huffval) = flat_table_bytes();
        let mut data = vec![0xFF, MARKER_SOI];
        data.extend(sof3_segment_nf2(precision, rows as u16, x_per_comp as u16));
        if restart_interval > 0 {
            data.extend(dri_segment(restart_interval as u16));
        }
        data.extend(dht_segment(0, &bits, &huffval));
        data.extend(sos_segment_nf2(predictor, point_transform, 0, 0));
        data.extend(entropy);
        data.extend_from_slice(&[0xFF, MARKER_EOI]);
        data
    }

    /// An INDEPENDENT re-implementation of the lossless predictor set (using the
    /// native `>>` arithmetic shift rather than production's `floor_div2`), for
    /// the red-proof's reference decoder. A match here cross-checks production's
    /// `predict`/`floor_div2`; a deliberate mutation of it drives the RED.
    fn honest_predict(sel: u8, ra: i32, rb: i32, rc: i32) -> i32 {
        match sel {
            1 => ra,
            2 => rb,
            3 => rc,
            4 => ra + rb - rc,
            5 => ra + ((rb - rc) >> 1),
            6 => rb + ((ra - rc) >> 1),
            7 => (ra + rb) >> 1,
            _ => ra,
        }
    }

    /// An INDEPENDENT reconstruction driver (its own boundary/restart/point-
    /// transform loop, not production's) parameterised by a predictor function.
    /// With `honest_predict` it must agree with `decode_into`; with a perturbed
    /// predictor it must diverge — that divergence is the red-proof.
    fn reference_decode(
        data: &[u8],
        out: &mut [u16],
        predict_fn: &dyn Fn(u8, i32, i32, i32) -> i32,
    ) {
        let (frame, tables, offset) = parse_headers(data).unwrap();
        let table = &tables[0];
        let mut reader = ScanReader::new(&data[offset..]);
        let width = frame.samples_per_line as usize;
        let height = frame.lines as usize;
        assert_eq!(out.len(), width * height);
        let pt = frame.point_transform;
        let initial = {
            let e = frame.precision as i32 - pt as i32 - 1;
            if e < 0 {
                0
            } else {
                1i32 << e
            }
        };
        let ri = frame.restart_interval as usize;
        let sel = frame.predictor;
        let mut idx = 0usize;
        let mut count = 0usize;
        for r in 0..height {
            for c in 0..width {
                if ri > 0 && count == ri {
                    reader.restart().unwrap();
                    count = 0;
                }
                let diff = reader.decode_difference(table).unwrap();
                let px = if count == 0 {
                    initial
                } else if r == 0 {
                    out[idx - 1] as i32
                } else if c == 0 {
                    out[idx - width] as i32
                } else {
                    predict_fn(
                        sel,
                        out[idx - 1] as i32,
                        out[idx - width] as i32,
                        out[idx - width - 1] as i32,
                    )
                };
                out[idx] = ((px + diff) & 0xFFFF) as u16;
                idx += 1;
                count += 1;
            }
        }
        if pt > 0 {
            let factor = 1u16 << pt;
            for s in out.iter_mut() {
                *s = s.wrapping_mul(factor);
            }
        }
    }

    // ── Acceptance tests (SPEC-037 `## Failing Tests`) ─────────────────────

    #[test]
    fn reconstruct_applies_each_predictor_1_through_7() {
        // 2x2 grid, P=8, Pt=0 → initial = 2^7 = 128. Differences are chosen so
        // the interior sample (1,1) has neighbours Ra=120, Rb=110, Rc=100; the
        // expected value is hand-computed per T.81 Table H.1, independent of
        // `predict`.
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        // s0=(0,0)=128+(-28)=100; s1=(0,1)=Ra(100)+10=110; s2=(1,0)=Rb(100)+20=120.
        let diffs = [-28i32, 10, 20, 5];
        let hand_interior = |sel: u8| -> u16 {
            let px = match sel {
                1 => 120,
                2 => 110,
                3 => 100,
                4 => 120 + 110 - 100,
                5 => 120 + ((110 - 100) >> 1),
                6 => 110 + ((120 - 100) >> 1),
                7 => (120 + 110) >> 1,
                _ => unreachable!(),
            };
            (px + 5) as u16
        };
        for sel in 1u8..=7 {
            let frame = Frame {
                precision: 8,
                lines: 2,
                samples_per_line: 2,
                point_transform: 0,
                predictor: sel,
                restart_interval: 0,
                components: 1,
            };
            let entropy = encode_scan(&diffs, 0);
            let mut reader = ScanReader::new(&entropy);
            let mut out = [0u16; 4];
            reconstruct(&frame, &table, &mut reader, &mut out).unwrap();
            assert_eq!(out[0], 100, "sel={sel}: first sample (initial+d)");
            assert_eq!(out[1], 110, "sel={sel}: first-line Ra");
            assert_eq!(out[2], 120, "sel={sel}: first-column Rb");
            assert_eq!(out[3], hand_interior(sel), "sel={sel}: interior predictor");
        }
    }

    #[test]
    fn reconstruct_floors_negative_odd_predictor_intermediate() {
        // SPEC-037 verify punch-list (HANDOFF-088, item 1): pin `floor_div2`'s
        // floor-toward-negative-infinity on a NEGATIVE ODD `>> 1` intermediate
        // (predictor 5's `Rb - Rc`, T.81 H.1). The round-trip oracle cannot catch
        // a truncate-toward-zero regression here — it shares `predict`→`floor_div2`
        // so the error cancels in encode→decode — and the positive neighbours of
        // `reconstruct_applies_each_predictor_1_through_7` never reach the negative
        // branch. So this asserts a hand-computed value directly.
        //
        // 2x2, P=8, Pt=0, predictor 5, initial = 128.
        //   s0=(0,0)=128-28=100; s1=(0,1)=Ra(100)-3=97; s2=(1,0)=Rb(100)+20=120.
        //   interior (1,1): Ra=120, Rb=97, Rc=100 → Rb-Rc = -3 (negative, odd).
        //   px = 120 + floor_div2(-3) = 120 + (-2) = 118   (truncate-to-zero → 119)
        //   out[3] = px + 5 = 123                            (truncate-to-zero → 124)
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        let diffs = [-28i32, -3, 20, 5];
        let frame = Frame {
            precision: 8,
            lines: 2,
            samples_per_line: 2,
            point_transform: 0,
            predictor: 5,
            restart_interval: 0,
            components: 1,
        };
        let entropy = encode_scan(&diffs, 0);
        let mut reader = ScanReader::new(&entropy);
        let mut out = [0u16; 4];
        reconstruct(&frame, &table, &mut reader, &mut out).unwrap();
        assert_eq!(
            out[3], 123,
            "predictor 5: floor_div2(-3) must floor to -2, not truncate to -1"
        );
    }

    #[test]
    fn reconstruct_honours_the_h21_boundary_rules() {
        // 3x2 grid, P=8, Pt=0, selected predictor = 2 (Rb) — so the first-line
        // and first-column rules must OVERRIDE the selected predictor.
        // idx: 0=(0,0) 1=(0,1) 2=(1,0) 3=(1,1) 4=(2,0) 5=(2,1).
        // (0,0) initial 128 + 0        = 128
        // (0,1) Ra=128 + 5             = 133   (first line uses Ra, not pred 2)
        // (1,0) Rb=out[0]=128 + 10     = 138
        // (1,1) pred2 = Rb=out[1]=133  = 133
        // (2,0) Rb=out[2]=138 + (-8)   = 130   (first col uses Rb=above, not initial)
        // (2,1) pred2 = Rb=out[3]=133  = 133
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        let diffs = [0i32, 5, 10, 0, -8, 0];
        let frame = Frame {
            precision: 8,
            lines: 3,
            samples_per_line: 2,
            point_transform: 0,
            predictor: 2,
            restart_interval: 0,
            components: 1,
        };
        let entropy = encode_scan(&diffs, 0);
        let mut reader = ScanReader::new(&entropy);
        let mut out = [0u16; 6];
        reconstruct(&frame, &table, &mut reader, &mut out).unwrap();
        assert_eq!(out[0], 128, "first sample of scan = 2^(P-Pt-1)");
        assert_eq!(out[1], 133, "rest of first line uses Ra (not predictor 2)");
        assert_eq!(out[2], 138, "first column of a later line uses Rb");
        assert_eq!(
            out[4], 130,
            "first column uses Rb=above(138), not initial(128)"
        );
    }

    #[test]
    fn reconstruct_applies_the_point_transform() {
        // 2x2, P=12, Pt=2 → initial = 2^(12-2-1) = 2^9 = 512. Prediction is in
        // the pre-shift domain; the stored output is reconstructed << 2.
        // recon: (0,0)=512, (0,1)=Ra512+3=515, (1,0)=Rb512-12=500, (1,1)=Ra500+4=504.
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        let diffs = [0i32, 3, -12, 4];
        let frame = Frame {
            precision: 12,
            lines: 2,
            samples_per_line: 2,
            point_transform: 2,
            predictor: 1,
            restart_interval: 0,
            components: 1,
        };
        let entropy = encode_scan(&diffs, 0);
        let mut reader = ScanReader::new(&entropy);
        let mut out = [0u16; 4];
        reconstruct(&frame, &table, &mut reader, &mut out).unwrap();
        // reconstructed = [512, 515, 500, 504]; stored = reconstructed << 2.
        assert_eq!(out, [2048, 2060, 2000, 2016]);
        // out[1] == 2060 proves Ra was 512 (pre-shift), not 2048 (post-shift):
        // a post-shift prediction would give (2048+3)<<2 = 8204.
    }

    #[test]
    fn reconstruct_resets_prediction_at_each_restart() {
        // 2x2, P=8, Pt=0, restart_interval = 2 (one RST0 between the two rows).
        // (0,0) initial 128 + 20 = 148
        // (0,1) Ra=148 + (-10)   = 138
        //   -- RST0; prediction resets --
        // (1,0) initial 128 + 5  = 133   (NOT Rb=out[0]=148, which would give 153)
        // (1,1) pred1 Ra=133 + 0 = 133
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        let diffs = [20i32, -10, 5, 0];
        let frame = Frame {
            precision: 8,
            lines: 2,
            samples_per_line: 2,
            point_transform: 0,
            predictor: 1,
            restart_interval: 2,
            components: 1,
        };
        let entropy = encode_scan(&diffs, 2);
        let mut reader = ScanReader::new(&entropy);
        let mut out = [0u16; 4];
        reconstruct(&frame, &table, &mut reader, &mut out).unwrap();
        assert_eq!(out[0], 148);
        assert_eq!(out[1], 138);
        assert_eq!(
            out[2], 133,
            "first sample after a restart uses the initial prediction"
        );
        assert_eq!(out[3], 133);
    }

    #[test]
    fn round_trip_is_bit_identical_across_predictors_precisions_and_pt() {
        // The analytic encode→decode oracle over the full matrix: predictors
        // 1..=7 × precision {8,12,16} × Pt {0,2} × restart {off, 7}.
        let width = 5usize;
        let height = 4usize;
        for &precision in &[8u8, 12, 16] {
            for &pt in &[0u8, 2] {
                for predictor in 1u8..=7 {
                    for &ri in &[0usize, 7] {
                        let shift_bits = precision - pt; // P - Pt (6..=16)
                        let modulo: u32 = 1u32 << shift_bits; // 2^(P-Pt)
                        let mut state = 0x1357u32
                            ^ (u32::from(precision) << 1)
                            ^ (u32::from(pt) << 8)
                            ^ (u32::from(predictor) << 12)
                            ^ (ri as u32);
                        let mut samples = vec![0u16; width * height];
                        for s in samples.iter_mut() {
                            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                            let recon = (state >> 8) % modulo; // 0 .. 2^(P-Pt)-1
                            *s = (recon << pt) as u16; // stored value
                        }
                        let stream =
                            encode_lj92(&samples, width, height, precision, predictor, pt, ri);
                        let mut out = vec![0u16; width * height];
                        let frame = decode_into(&stream, &mut out).unwrap_or_else(|e| {
                            panic!("decode P={precision} Pt={pt} pred={predictor} ri={ri}: {e}")
                        });
                        assert_eq!(
                            out, samples,
                            "round-trip P={precision} Pt={pt} pred={predictor} ri={ri}"
                        );
                        assert_eq!(frame.predictor, predictor);
                        assert_eq!(frame.point_transform, pt);
                        assert_eq!(frame.restart_interval as usize, ri);
                    }
                }
            }
        }
    }

    #[test]
    fn round_trip_red_proof_a_perturbed_predictor_breaks_the_round_trip() {
        // oracle-must-be-shown-red: a decoder with ONE predictor formula altered
        // (predictor 4: Ra+Rb-Rc → Ra+Rb+Rc) must break the round-trip, with the
        // honest control green. Samples are in [1, 2^P) so every Rc is non-zero
        // and the perturbation provably moves the first interior sample.
        let cols = 4usize;
        let rows = 3usize;
        let precision = 12u8;
        let pt = 0u8;
        let predictor = 4u8;
        let max: u32 = 1u32 << precision; // Pt == 0, so stored == reconstructed
        let mut state = 0xABCDu32;
        let mut samples = vec![0u16; cols * rows];
        for s in samples.iter_mut() {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *s = (((state >> 13) % (max - 1)) + 1) as u16; // 1 ..= 2^P - 1
        }
        let stream = encode_lj92(&samples, cols, rows, precision, predictor, pt, 0);

        // Honest control: the real decoder round-trips bit-exactly.
        let mut honest = vec![0u16; cols * rows];
        let frame = decode_into(&stream, &mut honest).unwrap();
        assert_eq!(honest, samples, "honest control must round-trip");
        assert_eq!(frame.predictor, 4);

        // The independent reference with the SAME predictor agrees — this ties
        // the reference driver to production so the RED below is the mutation.
        let mut reference = vec![0u16; cols * rows];
        reference_decode(&stream, &mut reference, &honest_predict);
        assert_eq!(
            reference, samples,
            "independent honest reference must agree"
        );

        // RED: perturb predictor 4 to Ra+Rb+Rc. The round-trip now FAILS.
        let mutant = |sel: u8, ra: i32, rb: i32, rc: i32| -> i32 {
            if sel == 4 {
                ra + rb + rc
            } else {
                honest_predict(sel, ra, rb, rc)
            }
        };
        let mut mutated = vec![0u16; cols * rows];
        reference_decode(&stream, &mut mutated, &mutant);
        assert_ne!(
            mutated, samples,
            "a perturbed predictor MUST break the round-trip (oracle-must-be-shown-red)"
        );
    }

    #[test]
    fn wrong_out_length_is_a_typed_error() {
        let samples = [10u16, 20, 30, 40, 50, 60]; // 3x2
        let stream = encode_lj92(&samples, 3, 2, 8, 1, 0, 0);

        let mut too_short = [0u16; 5];
        assert!(matches!(
            decode_into(&stream, &mut too_short),
            Err(Error::LjpegPlaneWrongLength {
                expected: 6,
                got: 5
            })
        ));

        let mut too_long = [0u16; 7];
        assert!(matches!(
            decode_into(&stream, &mut too_long),
            Err(Error::LjpegPlaneWrongLength {
                expected: 6,
                got: 7
            })
        ));

        // The exact length decodes cleanly.
        let mut exact = [0u16; 6];
        assert!(decode_into(&stream, &mut exact).is_ok());
        assert_eq!(exact, samples);

        // reconstruct itself refuses a mismatched buffer without reading the
        // scan — a typed error, never a panic.
        let frame = Frame {
            precision: 8,
            lines: 2,
            samples_per_line: 3,
            point_transform: 0,
            predictor: 1,
            restart_interval: 0,
            components: 1,
        };
        let (bits, huffval) = flat_table_bytes();
        let table = HuffTable::build(&bits, huffval).unwrap();
        let entropy = encode_scan(&[0, 0, 0, 0, 0, 0], 0);
        let mut reader = ScanReader::new(&entropy);
        let mut wrong = [0u16; 1];
        assert!(matches!(
            reconstruct(&frame, &table, &mut reader, &mut wrong),
            Err(Error::LjpegPlaneWrongLength {
                expected: 6,
                got: 1
            })
        ));
    }

    // ══ SPEC-038: panic-free replay over the fuzz surface ═════════════════════
    //
    // The libFuzzer run itself is manual (`cargo +nightly fuzz run ljpeg`, see
    // DEC-036 and the target's header); `just test` carries the panic-free
    // guarantee as these two in-crate UNIT tests. They reach `pub(crate)
    // decode_into` directly — NOT through the `fuzzing`-feature `__fuzz_ljpeg`
    // entry — so the guarantee holds regardless of that feature.

    /// Decode a seed into a dimension-bounded destination, mirroring
    /// `__fuzz_ljpeg` (`src/lib.rs`): parse the header to learn the plane size,
    /// cap it, allocate, and decode. A rejected header is itself a typed error
    /// — a "return", which is all the guarantee asks.
    fn decode_seed_bounded(data: &[u8]) -> Result<(), Error> {
        const MAX_PIXELS: usize = 4_000_000;
        let (frame, _table, _offset) = parse_headers(data)?;
        // Size the plane by `Nf · X · Y` so a two-component seed reaches the
        // decode path rather than tripping the length check (SPEC-046).
        let pixels = usize::from(frame.components)
            * usize::from(frame.lines)
            * usize::from(frame.samples_per_line);
        let len = pixels.min(MAX_PIXELS);
        let mut out = vec![0u16; len];
        decode_into(data, &mut out).map(|_| ())
    }

    #[test]
    fn fuzz_seed_corpus_decodes_without_panic() {
        use std::path::PathBuf;

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fuzz")
            .join("seeds")
            .join("ljpeg");
        let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| {
            panic!(
                "read {}: {e} — run `cargo run --example fuzz-seeds`",
                dir.display()
            )
        });

        let mut count = 0usize;
        for entry in entries {
            let path = entry.unwrap().path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !path.is_file() || name.starts_with('.') {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            // The guarantee is "returns, never panics" — a panic here fails the
            // test. Valid seeds decode to `Ok`, hostile ones to a typed `Error`.
            let _ = decode_seed_bounded(&bytes);
            count += 1;
        }

        assert!(
            count > 0,
            "no seeds in {} — run `cargo run --example fuzz-seeds` (the seed-gen must have run)",
            dir.display()
        );
    }

    #[test]
    fn hostile_bitstreams_are_typed_errors_not_panics() {
        // A table of crafted inputs, each asserted to be a typed Error through
        // the full `decode_into` pipeline — never a panic (a panic fails the
        // test). Where the dimensions parse, the destination is the exact
        // length, so the error is the hostile shape's own, not a length
        // mismatch.

        // Empty input — no SOI.
        let mut out = [0u16; 1];
        assert!(matches!(
            decode_into(&[], &mut out),
            Err(Error::LjpegMissingMarker { marker: MARKER_SOI })
        ));

        // A valid header then a truncated scan — the 16-sample scan runs out
        // after one entropy byte.
        let (bits, huffval) = flat_table_bytes();
        let mut truncated = vec![0xFF, MARKER_SOI];
        truncated.extend(sof3_segment(8, 4, 4));
        truncated.extend(dht_segment(0, &bits, &huffval));
        truncated.extend(sos_segment(1, 0, 0));
        truncated.push(0x00);
        let mut out_trunc = [0u16; 16];
        assert!(matches!(
            decode_into(&truncated, &mut out_trunc),
            Err(Error::Truncated { .. })
        ));

        // A SOF3 frame with Nf = 3.
        let mut nf3 = vec![0xFF, MARKER_SOI, 0xFF, MARKER_SOF3];
        nf3.extend_from_slice(&17u16.to_be_bytes()); // Lf = 8 + 3*3
        nf3.push(0x08);
        nf3.extend_from_slice(&2u16.to_be_bytes());
        nf3.extend_from_slice(&2u16.to_be_bytes());
        nf3.push(0x03); // Nf = 3
        nf3.extend_from_slice(&[0x00, 0x11, 0x00, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00]);
        let mut out_nf3 = [0u16; 1];
        assert!(matches!(
            decode_into(&nf3, &mut out_nf3),
            Err(Error::LjpegUnsupportedComponentCount { nf: 3 })
        ));

        // A DHT whose BITS over-subscribe the code space — three 1-bit codes.
        let mut oversub = vec![0xFF, MARKER_SOI];
        oversub.extend(sof3_segment(8, 2, 2));
        let oversub_bits = [3u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        oversub.extend(dht_segment(0, &oversub_bits, &[1u8, 2, 3]));
        let mut out_ov = [0u16; 1];
        assert!(matches!(
            decode_into(&oversub, &mut out_ov),
            Err(Error::LjpegInvalidHuffmanTable)
        ));

        // A forged 65535 x 65535 frame — the dimensions parse, but reconstruct
        // refuses the length against a real buffer before reading the scan
        // (nothing allocates the forged plane).
        let mut huge = vec![0xFF, MARKER_SOI];
        huge.extend(sof3_segment(8, 65535, 65535));
        huge.extend(dht_segment(0, &bits, &huffval));
        huge.extend(sos_segment(1, 0, 0));
        huge.push(0x00);
        let mut out_small = [0u16; 4];
        assert!(matches!(
            decode_into(&huge, &mut out_small),
            Err(Error::LjpegPlaneWrongLength { .. })
        ));

        // A wrong RSTm — a valid restart stream whose RST0 marker is bumped to
        // RST1. The only `FF D0` in the stream is the restart marker (the header
        // markers are D8/C3/C4/DD/DA, EOI is FF D9; entropy `FF` bytes are always
        // stuffed `FF 00`), so this rewrite hits exactly it.
        let samples = [10u16, 20, 30, 40]; // 2x2
        let mut wrong_rst = encode_lj92(&samples, 2, 2, 8, 1, 0, 2);
        for i in 0..wrong_rst.len().saturating_sub(1) {
            if wrong_rst[i] == 0xFF && wrong_rst[i + 1] == MARKER_RST0 {
                wrong_rst[i + 1] = 0xD1;
                break;
            }
        }
        let mut out_rst = [0u16; 4];
        assert!(matches!(
            decode_into(&wrong_rst, &mut out_rst),
            Err(Error::LjpegUnexpectedMarker { .. })
        ));
    }

    // ══ SPEC-046: two-component (Nf=2) column-interleave decode ════════════
    //
    // Tier-A only (hand-built two-component SOF-3 via `encode_lj92_two_component`;
    // no corpus). The analytic round-trip proves the decode is self-consistent
    // and per-component prediction is correct, but it CANNOT confirm the
    // real-file column-interleave MAPPING — encode and decode share the
    // convention (`shared-path-oracle-tautology`). SPEC-044's `dnglab
    // --raw-checksum` oracle pins the exact component order + column phase on
    // the two real files.

    /// An INDEPENDENT two-component reconstruction driver (its own MCU loop, not
    /// production's), parameterised by a predictor function and a
    /// `swap_components` flag. With `honest_predict` and `swap=false` it must
    /// agree with `decode_into`; a perturbed predictor, or writing the two
    /// components to swapped column phases, makes it diverge — those are the
    /// red-proof. Each component predicts only from its own stored-column
    /// neighbours (two output columns back, one plane row back), so swapping the
    /// write phase is a pure relabelling that yields the per-MCU-swapped plane.
    fn reference_decode_two_component(
        data: &[u8],
        out: &mut [u16],
        predict_fn: &dyn Fn(u8, i32, i32, i32) -> i32,
        swap_components: bool,
    ) {
        let (frame, tables, offset) = parse_headers(data).unwrap();
        assert_eq!(frame.components, 2);
        let table0 = &tables[0];
        let table1 = &tables[1];
        let mut reader = ScanReader::new(&data[offset..]);
        let x = frame.samples_per_line as usize;
        let height = frame.lines as usize;
        let plane_width = x * 2;
        assert_eq!(out.len(), plane_width * height);
        let pt = frame.point_transform;
        let initial = {
            let e = frame.precision as i32 - pt as i32 - 1;
            if e < 0 {
                0
            } else {
                1i32 << e
            }
        };
        let ri = frame.restart_interval as usize;
        let sel = frame.predictor;

        let nb = |out: &[u16], slot: usize, back: usize| -> i32 {
            match slot.checked_sub(back) {
                Some(j) => out[j] as i32,
                None => 0,
            }
        };
        let px_of = |out: &[u16], slot: usize, is_start: bool, y: usize, xcol: usize| -> i32 {
            if is_start {
                initial
            } else if y == 0 {
                nb(out, slot, 2)
            } else if xcol == 0 {
                nb(out, slot, plane_width)
            } else {
                predict_fn(
                    sel,
                    nb(out, slot, 2),
                    nb(out, slot, plane_width),
                    nb(out, slot, plane_width + 2),
                )
            }
        };

        let mut idx = 0usize;
        let mut count = 0usize;
        for y in 0..height {
            for xcol in 0..x {
                if ri > 0 && count == ri {
                    reader.restart().unwrap();
                    count = 0;
                }
                let is_start = count == 0;
                // Scan order is always comp0 then comp1; only the WRITE phase
                // and the neighbour reads swap.
                let d0 = reader.decode_difference(table0).unwrap();
                let d1 = reader.decode_difference(table1).unwrap();
                let pos0 = idx + usize::from(swap_components);
                let pos1 = idx + usize::from(!swap_components);
                let px0 = px_of(out, pos0, is_start, y, xcol);
                out[pos0] = ((px0 + d0) & 0xFFFF) as u16;
                let px1 = px_of(out, pos1, is_start, y, xcol);
                out[pos1] = ((px1 + d1) & 0xFFFF) as u16;
                idx += 2;
                count += 1;
            }
        }
        if pt > 0 {
            let factor = 1u16 << pt;
            for s in out.iter_mut() {
                *s = s.wrapping_mul(factor);
            }
        }
    }

    #[test]
    fn two_component_interleaved_decodes_bit_exact() {
        // A known 2·X × Y monochrome plane (comp0 = even columns, comp1 = odd)
        // encoded as a two-component interleaved SOF-3 scan, decoded, and
        // compared bit-for-bit. At P=12 (Leica Typ246) and P=14 (Pentax K3III);
        // predictor 1 (both held files) plus two interior predictors (4, 6) to
        // exercise the per-component Ra/Rb/Rc neighbour math; with and without a
        // per-MCU restart interval.
        let x = 3usize; // per component
        let rows = 3usize;
        let plane_width = x * 2;
        for &precision in &[12u8, 14] {
            let modulo: u32 = 1u32 << precision;
            for &predictor in &[1u8, 4, 6] {
                for &ri in &[0usize, x] {
                    let mut state = 0x2468u32
                        ^ (u32::from(precision) << 3)
                        ^ (u32::from(predictor) << 10)
                        ^ (ri as u32);
                    let mut samples = vec![0u16; plane_width * rows];
                    for s in samples.iter_mut() {
                        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                        *s = ((state >> 9) % modulo) as u16;
                    }
                    let stream =
                        encode_lj92_two_component(&samples, x, rows, precision, predictor, 0, ri);
                    let mut out = vec![0u16; plane_width * rows];
                    let frame = decode_into(&stream, &mut out).unwrap_or_else(|e| {
                        panic!("decode Nf=2 P={precision} pred={predictor} ri={ri}: {e}")
                    });
                    assert_eq!(
                        frame.components, 2,
                        "P={precision} pred={predictor} ri={ri}"
                    );
                    assert_eq!(frame.precision, precision);
                    assert_eq!(frame.samples_per_line as usize, x);
                    assert_eq!(frame.lines as usize, rows);
                    assert_eq!(frame.restart_interval as usize, ri);
                    assert_eq!(
                        out, samples,
                        "two-component round-trip must be bit-exact (P={precision} pred={predictor} ri={ri})"
                    );
                }
            }
        }
    }

    #[test]
    fn two_component_round_trip_red_on_perturbed_predictor() {
        // oracle-must-be-shown-red for the two-component path. Three mutations
        // each break the round-trip while the honest control stays green:
        //   (a) a perturbed predictor, (b) swapping which column phase each
        //   component is written to, (c) a mutated DC Huffman table.
        let x = 3usize;
        let rows = 3usize;
        let plane_width = x * 2;
        let precision = 12u8;
        let max: u32 = 1u32 << precision;
        // Values in [1, 2^P) so every Rc is non-zero and the predictor
        // perturbation provably moves an interior sample.
        let mut state = 0xBEEFu32;
        let mut samples = vec![0u16; plane_width * rows];
        for s in samples.iter_mut() {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *s = (((state >> 11) % (max - 1)) + 1) as u16;
        }
        let stream = encode_lj92_two_component(&samples, x, rows, precision, 4, 0, 0);

        // Honest control: production decodes bit-exactly.
        let mut honest = vec![0u16; plane_width * rows];
        let frame = decode_into(&stream, &mut honest).unwrap();
        assert_eq!(frame.components, 2);
        assert_eq!(honest, samples, "honest control must round-trip");

        // The independent reference (honest predictor, no swap) agrees — ties
        // the reference driver to production so the REDs below are the mutations.
        let mut reference = vec![0u16; plane_width * rows];
        reference_decode_two_component(&stream, &mut reference, &honest_predict, false);
        assert_eq!(
            reference, samples,
            "independent honest reference must agree"
        );

        // (a) RED — perturb predictor 4 to Ra+Rb+Rc.
        let mutant = |sel: u8, ra: i32, rb: i32, rc: i32| -> i32 {
            if sel == 4 {
                ra + rb + rc
            } else {
                honest_predict(sel, ra, rb, rc)
            }
        };
        let mut mutated = vec![0u16; plane_width * rows];
        reference_decode_two_component(&stream, &mut mutated, &mutant, false);
        assert_ne!(
            mutated, samples,
            "a perturbed predictor MUST break the round-trip"
        );

        // (b) RED — swap the two components' column phases. The result is the
        // per-MCU-swapped plane, which differs from the original.
        let mut swapped_expected = vec![0u16; plane_width * rows];
        for mcu in 0..(x * rows) {
            swapped_expected[mcu * 2] = samples[mcu * 2 + 1];
            swapped_expected[mcu * 2 + 1] = samples[mcu * 2];
        }
        assert_ne!(
            swapped_expected, samples,
            "test premise: comp0 and comp1 differ somewhere"
        );
        let mut swapped = vec![0u16; plane_width * rows];
        reference_decode_two_component(&stream, &mut swapped, &honest_predict, true);
        assert_eq!(
            swapped, swapped_expected,
            "swapping the write phase yields the swapped plane"
        );
        assert_ne!(
            swapped, samples,
            "swapping component order MUST break the round-trip"
        );

        // (c) RED — mutate the DC Huffman table so categories 1 and 2 are
        // swapped. A deterministic X=2, Y=1 probe whose diffs are [0,0,1,2]
        // (comp0: initial 2048 then Ra+1; comp1: initial 2048 then Ra+2).
        let probe = [2048u16, 2048, 2049, 2050]; // out[0..4]: c0_0 c1_0 c0_1 c1_1
        let probe_stream = encode_lj92_two_component(&probe, 2, 1, 12, 1, 0, 0);
        let mut probe_out = [0u16; 4];
        assert_eq!(
            decode_into(&probe_stream, &mut probe_out)
                .unwrap()
                .components,
            2
        );
        assert_eq!(probe_out, probe, "probe honest control");
        let mut mutated_table = probe_stream.clone();
        let dht = mutated_table
            .windows(2)
            .position(|w| w == [0xFF, MARKER_DHT])
            .unwrap();
        // DHT: FF C4 | Lh(2) | Tc/Th(1) | BITS(16) | HUFFVAL... — symbols start
        // at dht + 2 + 2 + 1 + 16 = dht + 21.
        let huffval = dht + 21;
        mutated_table.swap(huffval + 1, huffval + 2); // symbols 1 <-> 2
        let mut mut_out = [0u16; 4];
        let diverged = match decode_into(&mutated_table, &mut mut_out) {
            Ok(_) => mut_out != probe,
            Err(_) => true,
        };
        assert!(
            diverged,
            "a mutated Huffman table MUST break the round-trip"
        );
    }

    #[test]
    fn single_component_decode_unchanged() {
        // Characterization: the Nf=1 path still decodes identically after the
        // two-component extension. A fixed plane encoded with the unchanged
        // single-component encoder decodes to exactly those samples, and the
        // frame reports one component.
        let samples = [100u16, 110, 120, 130]; // 2x2, P=8, predictor 1
        let stream = encode_lj92(&samples, 2, 2, 8, 1, 0, 0);
        let mut out = [0u16; 4];
        let frame = decode_into(&stream, &mut out).unwrap();
        assert_eq!(frame.components, 1, "Nf=1 header must report one component");
        assert_eq!(out, samples, "single-component decode must be unchanged");

        // A wider spot check through the same dispatch (P=12, predictor 2).
        let wider = [10u16, 4095, 2000, 7, 1234, 4094]; // 3x2
        let s2 = encode_lj92(&wider, 3, 2, 12, 2, 0, 0);
        let mut o2 = [0u16; 6];
        let f2 = decode_into(&s2, &mut o2).unwrap();
        assert_eq!(f2.components, 1);
        assert_eq!(o2, wider);
    }

    #[test]
    fn nf_three_still_unsupported() {
        // A SOF3 frame with Nf=3 (Bayer/CFA colour) stays rejected (DEC-037).
        let mut data = vec![0xFF, MARKER_SOI, 0xFF, MARKER_SOF3];
        data.extend_from_slice(&17u16.to_be_bytes()); // Lf = 8 + 3*3
        data.push(0x08); // P
        data.extend_from_slice(&2u16.to_be_bytes()); // Y
        data.extend_from_slice(&2u16.to_be_bytes()); // X
        data.push(0x03); // Nf = 3
        data.extend_from_slice(&[0x00, 0x11, 0x00, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00]);
        assert!(matches!(
            parse_headers(&data),
            Err(Error::LjpegUnsupportedComponentCount { nf: 3 })
        ));
        let mut out = [0u16; 4];
        assert!(matches!(
            decode_into(&data, &mut out),
            Err(Error::LjpegUnsupportedComponentCount { nf: 3 })
        ));
    }

    #[test]
    fn two_component_wrong_out_length_is_typed_error() {
        // X=2, Y=2, Nf=2 → the de-interleaved plane is 2·2·2 = 8 samples.
        let samples = [10u16, 20, 30, 40, 50, 60, 70, 80];
        let stream = encode_lj92_two_component(&samples, 2, 2, 8, 1, 0, 0);

        let mut too_short = [0u16; 7];
        assert!(matches!(
            decode_into(&stream, &mut too_short),
            Err(Error::LjpegPlaneWrongLength {
                expected: 8,
                got: 7
            })
        ));
        let mut too_long = [0u16; 9];
        assert!(matches!(
            decode_into(&stream, &mut too_long),
            Err(Error::LjpegPlaneWrongLength {
                expected: 8,
                got: 9
            })
        ));
        // The exact `Nf · X · Y` length decodes cleanly.
        let mut exact = [0u16; 8];
        let frame = decode_into(&stream, &mut exact).unwrap();
        assert_eq!(frame.components, 2);
        assert_eq!(exact, samples);
    }

    #[test]
    fn two_component_truncated_scan_is_typed_error() {
        // A single-MCU two-component scan (X=1, Y=1). comp0's difference fills
        // exactly one entropy byte; truncating the scan to that one byte leaves
        // the decoder mid-second-component (comp1), which must return a typed
        // error, not panic. P=12, Pt=0, predictor 1 → both components use the
        // initial prediction 2048. comp0 = 2053 (diff 5, category 3 → one byte);
        // comp1 = 2055 (diff 7) begins in the next byte.
        let samples = [2053u16, 2055]; // out[0]=comp0, out[1]=comp1
        let stream = encode_lj92_two_component(&samples, 1, 1, 12, 1, 0, 0);
        let (_f, _t, offset) = parse_headers(&stream).unwrap();
        assert!(
            stream.len() > offset + 1,
            "the full scan has >1 entropy byte"
        );
        let truncated = &stream[..offset + 1]; // headers + exactly one entropy byte
        let mut out = [0u16; 2];
        let result = decode_into(truncated, &mut out);
        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "a scan truncated mid-second-component must be a typed error, got {result:?}"
        );
    }

    // ══ SPEC-044 / DEC-039: APPn / COM header segments are skipped ═════════
    //
    // Tier-A (hand-built, no corpus). The real Pentax K-3 III stream is
    // `SOI → SOF3 → DHT → APP0(len 60) → SOS`; these fixtures put the same
    // shape around the SPEC-046 two-component encoder. The real-file proof is
    // the tier-B `dnglab --raw-checksum` oracle in `tests/plane_oracle.rs`.

    /// A length-delimited marker segment `0xFF code | L | payload`, with
    /// `L = payload.len() + 2` (the length counts itself).
    fn marker_segment(code: u8, payload: &[u8]) -> Vec<u8> {
        let mut seg = vec![0xFF, code];
        seg.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        seg.extend_from_slice(payload);
        seg
    }

    /// `stream` with `segment` spliced in immediately before its SOS marker —
    /// i.e. between DHT and SOS, where K3III carries its APP0. The SOS is found
    /// before the splice, so a payload containing `0xFF 0xDA` cannot confuse it.
    fn insert_before_sos(stream: &[u8], segment: &[u8]) -> Vec<u8> {
        let sos = stream
            .windows(2)
            .position(|w| w == [0xFF, MARKER_SOS])
            .expect("stream has an SOS marker");
        let mut out = stream[..sos].to_vec();
        out.extend_from_slice(segment);
        out.extend_from_slice(&stream[sos..]);
        out
    }

    /// A deterministic 2·X × Y plane at precision `p` for the APPn fixtures.
    fn appn_fixture_plane(x: usize, rows: usize, precision: u8) -> Vec<u16> {
        let modulo = 1u32 << precision;
        let mut state = 0x044u32;
        (0..x * 2 * rows)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                ((state >> 9) % modulo) as u16
            })
            .collect()
    }

    /// Decode `stream` (two-component, `2·x·rows` samples) to a fresh plane.
    fn decode_plane(stream: &[u8], len: usize) -> Result<Vec<u16>, Error> {
        let mut out = vec![0u16; len];
        decode_into(stream, &mut out)?;
        Ok(out)
    }

    #[test]
    fn appn_segment_between_dht_and_sos_is_skipped() {
        // K3III's shape: P=14, predictor 1, Pt 0, two components, an APP0
        // between DHT and SOS. The payload deliberately contains `0xFF 0xDA`
        // (an SOS lookalike) and `0xFF 0x00`: a skip by declared length must
        // step over them, never interpret them.
        let (x, rows, precision) = (4usize, 3usize, 14u8);
        let samples = appn_fixture_plane(x, rows, precision);
        let clean = encode_lj92_two_component(&samples, x, rows, precision, 1, 0, 0);
        let mut payload = b"PENTAX \0".to_vec();
        payload.extend_from_slice(&[0xFF, MARKER_SOS, 0xFF, 0x00, 0x12]);
        payload.resize(58, 0xAB); // 58 + 2 = L 60, as on the real file
        let with_app0 = insert_before_sos(&clean, &marker_segment(MARKER_APP0, &payload));

        let len = samples.len();
        let base = decode_plane(&clean, len).unwrap();
        assert_eq!(base, samples, "segment-free frame decodes bit-exact");
        let got = decode_plane(&with_app0, len).unwrap();
        assert_eq!(got, base, "an APP0 between DHT and SOS must be skipped");

        // Every APPn (APP0..APP15), stacked, and an empty-payload one (L = 2).
        let mut stacked = clean.clone();
        for code in MARKER_APP0..=MARKER_APP15 {
            stacked = insert_before_sos(&stacked, &marker_segment(code, &[code, 0xFF]));
        }
        stacked = insert_before_sos(&stacked, &marker_segment(MARKER_APP0 + 1, &[]));
        assert_eq!(decode_plane(&stacked, len).unwrap(), base);

        // The returned entropy offset moves by exactly the segment length.
        let (_, _, off_clean) = parse_headers(&clean).unwrap();
        let (_, _, off_app) = parse_headers(&with_app0).unwrap();
        assert_eq!(off_app, off_clean + 2 + 60);
    }

    #[test]
    fn com_segment_is_skipped() {
        let (x, rows, precision) = (3usize, 2usize, 12u8);
        let samples = appn_fixture_plane(x, rows, precision);
        let clean = encode_lj92_two_component(&samples, x, rows, precision, 1, 0, 0);
        let com = marker_segment(MARKER_COM, b"irradiance SPEC-044 \xFF\xD9 comment");
        let with_com = insert_before_sos(&clean, &com);
        let len = samples.len();
        assert_eq!(decode_plane(&with_com, len).unwrap(), samples);

        // A COM right after SOI (before SOF3) is skipped too: the arm is
        // position-independent within the header marker sequence.
        let mut early = vec![0xFF, MARKER_SOI];
        early.extend_from_slice(&com);
        early.extend_from_slice(&clean[2..]);
        assert_eq!(decode_plane(&early, len).unwrap(), samples);
    }

    #[test]
    fn truncated_appn_length_is_a_typed_error_not_a_panic() {
        let samples = appn_fixture_plane(2, 2, 12);
        let clean = encode_lj92_two_component(&samples, 2, 2, 12, 1, 0, 0);
        let sos = clean
            .windows(2)
            .position(|w| w == [0xFF, MARKER_SOS])
            .unwrap();
        let head = &clean[..sos]; // SOI, SOF3, DHT

        // A declared length that runs past the end of the buffer.
        let mut past_end = head.to_vec();
        past_end.extend_from_slice(&[0xFF, MARKER_APP0, 0x00, 0xC8, 1, 2, 3]);
        assert!(matches!(
            parse_headers(&past_end),
            Err(Error::Truncated { .. })
        ));
        // The same, mid-stream in a full frame (L = 0xFFFF swallows the rest).
        let mut huge = head.to_vec();
        huge.extend_from_slice(&[0xFF, MARKER_COM, 0xFF, 0xFF]);
        huge.extend_from_slice(&clean[sos..]);
        let mut out = vec![0u16; samples.len()];
        assert!(matches!(
            decode_into(&huge, &mut out),
            Err(Error::Truncated { .. })
        ));
        // A length below 2 (it must count its own two bytes).
        for bad_len in [0u8, 1] {
            let mut short = head.to_vec();
            short.extend_from_slice(&[0xFF, MARKER_APP0 + 5, 0x00, bad_len]);
            short.extend_from_slice(&clean[sos..]);
            assert!(matches!(
                parse_headers(&short),
                Err(Error::Truncated { .. })
            ));
        }
        // The length field itself cut off after one byte, and absent.
        let mut half = head.to_vec();
        half.extend_from_slice(&[0xFF, MARKER_APP0, 0x00]);
        assert!(matches!(parse_headers(&half), Err(Error::Truncated { .. })));
        let mut none = head.to_vec();
        none.extend_from_slice(&[0xFF, MARKER_COM]);
        assert!(matches!(parse_headers(&none), Err(Error::Truncated { .. })));
        // A segment that ends exactly at EOF parses, then the missing SOS is
        // the typed error.
        let mut exact = head.to_vec();
        exact.extend_from_slice(&marker_segment(MARKER_APP0, &[9, 9]));
        assert!(parse_headers(&exact).is_err());
    }

    #[test]
    fn single_component_decode_unchanged_by_appn_arm() {
        // Characterization: a segment-free Nf=1 scan decodes exactly as shipped
        // (fixed plane, fixed stream), and its entropy offset is unchanged.
        let samples = [10u16, 4095, 2000, 7, 1234, 4094]; // 3x2, P=12
        let stream = encode_lj92(&samples, 3, 2, 12, 1, 0, 0);
        let (frame, _, offset) = parse_headers(&stream).unwrap();
        assert_eq!(frame.components, 1);
        // SOI(2) + SOF3 Nf=1 (2+11) + DHT (2+2+1+16+HUFFVAL) + SOS (2+8).
        let (_bits, huffval) = flat_table_bytes();
        assert_eq!(offset, 2 + 13 + (2 + 2 + 1 + 16 + huffval.len()) + 10);
        let mut out = [0u16; 6];
        decode_into(&stream, &mut out).unwrap();
        assert_eq!(out, samples);

        // And an APP0 in an Nf=1 stream is skipped to the same plane.
        let with_app = insert_before_sos(&stream, &marker_segment(MARKER_APP0, b"JFIF\0"));
        let mut out2 = [0u16; 6];
        decode_into(&with_app, &mut out2).unwrap();
        assert_eq!(out2, samples);
    }
}
