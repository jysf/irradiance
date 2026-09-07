//! Reader for `dnglab analyze --srgb`'s malformed PNM output (`SPEC-020`,
//! `DEC-005`).
//!
//! `dnglab --help` documents this as "16-bit sRGB TIFF". It is not a TIFF —
//! it writes a PNM — and on a *monochrome* file the header claims `P6` (RGB,
//! 3 samples/pixel) over a payload that is actually `P5`-shaped (1
//! sample/pixel): exactly `w*h*2` bytes, where a real `P6` at 16-bit depth
//! needs `w*h*3*2` (`docs/oracle-contract.md` § "Layer 3 is not what this
//! document originally said"). The workaround: assert the payload length
//! matches the malformed shape BEFORE trusting the header at all, then read
//! it as grayscale (`AC1`). If a future dnglab release emits a genuinely
//! well-formed `P6`, this reader must say so loudly rather than silently
//! read one third of a real RGB image as grayscale (`AC2` — `DEC-005`'s
//! pre-registered revisit condition).
//!
//! Precedent: `tests/plane_oracle.rs::parse_raw_pixel_pgm` reads dnglab's
//! (well-formed) `--raw-pixel` PGM the same big-endian way; this reader is a
//! straight port of that shape with the header check changed from
//! "`P5`-sized-for-plane" to "`P6`-header-over-`P5`-payload"
//! (`SPEC-020/## Notes for the Implementer`).

use std::fmt;

/// A decoded plane: dimensions plus big-endian 16-bit grayscale samples,
/// exactly `width * height` long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrayscalePlane {
    pub width: u32,
    pub height: u32,
    pub samples: Vec<u16>,
}

/// Everything that can go wrong reading `dnglab --srgb`'s output. Every
/// variant names the figures involved — no `unwrap()`/`expect()`/panicking
/// index on this path, matching this repo's other test-support readers
/// (`tests/plane_oracle.rs::parse_raw_pixel_pgm`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PnmError {
    /// No `\n` was found — there is no header to read at all.
    NoHeaderNewline,
    /// The header bytes are not UTF-8.
    HeaderNotUtf8,
    /// The magic was not `P6` — this reader recognises exactly the one
    /// shape `dnglab --srgb` emits (`DEC-005`); anything else is
    /// unrecognised, not merely different.
    UnexpectedMagic { got: String },
    /// A required header field is missing.
    MissingField(&'static str),
    /// A header field did not parse as an integer.
    FieldNotInteger { field: &'static str },
    /// `maxval` was not `65535` — the only depth `DEC-005`'s workaround
    /// covers.
    UnsupportedMaxval { got: u32 },
    /// `width * height` (or a derived byte count) does not fit this host's
    /// arithmetic types.
    DimensionsOverflow,
    /// The payload is neither the malformed grayscale length nor a
    /// well-formed `P6` length — forcing either interpretation would be a
    /// guess, so both compared figures are named (`AC1`).
    PayloadLengthMismatch {
        actual: u64,
        expected_grayscale: u64,
    },
    /// The payload IS `w*h*3*2` bytes: dnglab appears to have started
    /// emitting a genuinely well-formed `P6`. Forcing the `P5` workaround
    /// here would silently read one third of a real RGB image as grayscale
    /// (`AC2` — `DEC-005`'s pre-registered revisit condition).
    WellFormedP6Dec005RevisitConditionMet {
        actual: u64,
        malformed_grayscale_length: u64,
    },
}

impl fmt::Display for PnmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PnmError::NoHeaderNewline => write!(f, "no newline in output — not a PNM header"),
            PnmError::HeaderNotUtf8 => write!(f, "PNM header is not UTF-8"),
            PnmError::UnexpectedMagic { got } => write!(
                f,
                "expected P6 magic (dnglab's malformed --srgb shape, DEC-005), got {got:?}"
            ),
            PnmError::MissingField(field) => write!(f, "PNM header missing `{field}`"),
            PnmError::FieldNotInteger { field } => {
                write!(f, "PNM header field `{field}` does not parse as an integer")
            }
            PnmError::UnsupportedMaxval { got } => write!(
                f,
                "PNM maxval {got} — only 65535 (16-bit) is the shape DEC-005's workaround covers"
            ),
            PnmError::DimensionsOverflow => {
                write!(f, "width * height overflows this host's arithmetic")
            }
            PnmError::PayloadLengthMismatch {
                actual,
                expected_grayscale,
            } => write!(
                f,
                "payload is {actual} bytes, expected {expected_grayscale} (w*h*2 — dnglab's \
                 known P6-header-over-P5-payload defect, DEC-005)"
            ),
            PnmError::WellFormedP6Dec005RevisitConditionMet {
                actual,
                malformed_grayscale_length,
            } => write!(
                f,
                "payload is {actual} bytes — a WELL-FORMED P6 (w*h*3*2), not the \
                 {malformed_grayscale_length}-byte (w*h*2) malformed shape DEC-005 documented. \
                 dnglab's --srgb output shape appears to have changed; DEC-005's revisit \
                 condition is met. Refusing to force the P5 workaround — it would silently read \
                 one third of a real RGB image as grayscale."
            ),
        }
    }
}

impl std::error::Error for PnmError {}

/// Read `dnglab analyze --srgb`'s output: a `P6 <w> <h> 65535\n` header over
/// a payload that is `w*h*2` bytes (grayscale) rather than the `w*h*3*2`
/// bytes the header claims (`DEC-005`). The payload length is checked
/// BEFORE any interpretation (`AC1`); a well-formed `P6` payload is refused
/// loudly, never silently truncated (`AC2`).
pub fn read_dnglab_srgb(bytes: &[u8]) -> Result<GrayscalePlane, PnmError> {
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .ok_or(PnmError::NoHeaderNewline)?;
    let header = std::str::from_utf8(&bytes[..header_end]).map_err(|_| PnmError::HeaderNotUtf8)?;

    let mut fields = header.split_whitespace();
    let magic = fields.next().ok_or(PnmError::MissingField("magic"))?;
    if magic != "P6" {
        return Err(PnmError::UnexpectedMagic {
            got: magic.to_string(),
        });
    }
    let width: u32 = fields
        .next()
        .ok_or(PnmError::MissingField("width"))?
        .parse()
        .map_err(|_| PnmError::FieldNotInteger { field: "width" })?;
    let height: u32 = fields
        .next()
        .ok_or(PnmError::MissingField("height"))?
        .parse()
        .map_err(|_| PnmError::FieldNotInteger { field: "height" })?;
    let maxval: u32 = fields
        .next()
        .ok_or(PnmError::MissingField("maxval"))?
        .parse()
        .map_err(|_| PnmError::FieldNotInteger { field: "maxval" })?;
    if maxval != 65535 {
        return Err(PnmError::UnsupportedMaxval { got: maxval });
    }

    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or(PnmError::DimensionsOverflow)?;
    let grayscale_len = pixel_count
        .checked_mul(2)
        .ok_or(PnmError::DimensionsOverflow)?;
    let rgb_len = pixel_count
        .checked_mul(6)
        .ok_or(PnmError::DimensionsOverflow)?; // w*h*3*2

    let payload = &bytes[header_end + 1..];
    let actual = payload.len() as u64;

    // The length is compared before any conversion (AC1). A well-formed P6
    // is checked FIRST and named specially (AC2) — the two shapes coincide
    // only when pixel_count == 0, which is not a plane worth reading either
    // way.
    if actual == rgb_len && actual != grayscale_len {
        return Err(PnmError::WellFormedP6Dec005RevisitConditionMet {
            actual,
            malformed_grayscale_length: grayscale_len,
        });
    }
    if actual != grayscale_len {
        return Err(PnmError::PayloadLengthMismatch {
            actual,
            expected_grayscale: grayscale_len,
        });
    }

    let sample_count = usize::try_from(pixel_count).map_err(|_| PnmError::DimensionsOverflow)?;
    let (chunks, _remainder) = payload.as_chunks::<2>();
    let mut samples = Vec::with_capacity(sample_count);
    for c in chunks {
        samples.push(u16::from_be_bytes(*c));
    }

    Ok(GrayscalePlane {
        width,
        height,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_srgb_bytes(width: u32, height: u32, payload: &[u8]) -> Vec<u8> {
        let mut bytes = format!("P6 {width} {height} 65535\n").into_bytes();
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn pnm_reader_accepts_dnglab_p6_with_p5_payload_length() {
        // 2x1 grayscale plane, big-endian samples 746 (0x02EA) and 100
        // (0x0064) — the exact endianness proof `tests/plane_oracle.rs`
        // established for `--raw-pixel`, reused here for the same
        // big-endian assumption on this different oracle layer.
        let payload = [0x02, 0xEA, 0x00, 0x64];
        let bytes = build_srgb_bytes(2, 1, &payload);
        let plane = read_dnglab_srgb(&bytes).expect("matches the documented w*h*2 defect");
        assert_eq!((plane.width, plane.height), (2, 1));
        assert_eq!(plane.samples, vec![746, 100]);
    }

    #[test]
    fn pnm_reader_rejects_mismatched_payload_length() {
        // Neither w*h*2 (4 bytes) nor w*h*3*2 (12 bytes) for a 2x1 plane —
        // a genuinely truncated/corrupt capture, not either recognised shape.
        let bytes = build_srgb_bytes(2, 1, &[0x00, 0x01, 0x02]);
        let err = read_dnglab_srgb(&bytes).expect_err("3 bytes matches neither shape");
        assert_eq!(
            err,
            PnmError::PayloadLengthMismatch {
                actual: 3,
                expected_grayscale: 4
            }
        );
    }

    #[test]
    fn pnm_reader_errors_on_wellformed_p6_dec_005_revisit() {
        // A hand-built, genuinely well-formed P6: w*h*3*2 = 2*1*3*2 = 12
        // bytes of real RGB payload. Must NOT be silently forced to P5.
        let payload = [0u8; 12];
        let bytes = build_srgb_bytes(2, 1, &payload);
        let err =
            read_dnglab_srgb(&bytes).expect_err("a real P6 must not be silently forced to P5");
        assert_eq!(
            err,
            PnmError::WellFormedP6Dec005RevisitConditionMet {
                actual: 12,
                malformed_grayscale_length: 4
            }
        );
    }

    #[test]
    fn pnm_reader_rejects_missing_newline() {
        assert_eq!(
            read_dnglab_srgb(b"P6 2 1 65535 no newline here"),
            Err(PnmError::NoHeaderNewline)
        );
    }

    #[test]
    fn pnm_reader_rejects_wrong_magic() {
        let mut bytes = b"P5 2 1 65535\n".to_vec();
        bytes.extend_from_slice(&[0u8; 4]);
        assert_eq!(
            read_dnglab_srgb(&bytes),
            Err(PnmError::UnexpectedMagic {
                got: "P5".to_string()
            })
        );
    }

    #[test]
    fn pnm_reader_rejects_unsupported_maxval() {
        let mut bytes = b"P6 2 1 255\n".to_vec();
        bytes.extend_from_slice(&[0u8; 4]);
        assert_eq!(
            read_dnglab_srgb(&bytes),
            Err(PnmError::UnsupportedMaxval { got: 255 })
        );
    }

    #[test]
    fn pnm_reader_rejects_missing_header_fields() {
        assert_eq!(
            read_dnglab_srgb(b"P6 2\n"),
            Err(PnmError::MissingField("height"))
        );
        assert_eq!(
            read_dnglab_srgb(b"\n"),
            Err(PnmError::MissingField("magic"))
        );
    }
}
