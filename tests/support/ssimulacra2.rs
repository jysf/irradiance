//! The SSIMULACRA2 metric wrapper (`SPEC-020`, `AC3`).
//!
//! The 16-bit grayscale -> `ssimulacra2::Rgb` conversion is pinned:
//! `sample as f32 / 65535.0`, replicated into the three RGB channels, tagged
//! sRGB transfer + BT.709 primaries. This is the same route crustyimg's
//! `src/quality/mod.rs::to_ss_rgb` takes for an already-decoded 8-bit image
//! (`img.to_rgb8()`, `p[0] as f32 / 255.0`), adapted to a 16-bit source and
//! with the `::image`-crate decode step removed — `library-not-application`
//! forbids that dependency even test-side, and a PNM's pixels need no
//! decoder to reach a `Vec<u16>` (`tests/support/pnm.rs`).

use ssimulacra2::{compute_frame_ssimulacra2, ColorPrimaries, Rgb, TransferCharacteristic};

/// Everything that can go wrong scoring two grayscale planes.
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreError {
    /// `Rgb::new` rejected the conversion (in practice: `samples.len() !=
    /// width * height`).
    Conversion(String),
    /// The metric itself rejected the (already shape-valid) inputs — e.g.
    /// mismatched dimensions between reference and candidate, or an image
    /// smaller than the metric's 8x8 floor.
    Metric(String),
}

impl std::fmt::Display for ScoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoreError::Conversion(reason) => write!(f, "converting to Rgb failed: {reason}"),
            ScoreError::Metric(reason) => write!(f, "SSIMULACRA2 scoring failed: {reason}"),
        }
    }
}

impl std::error::Error for ScoreError {}

/// The pinned 16-bit -> `Rgb` conversion (`AC3`): normalise to `[0.0, 1.0]`
/// as `sample as f32 / 65535.0`, replicate into three channels, tag sRGB
/// transfer + BT.709 primaries.
pub fn to_rgb(width: u32, height: u32, samples: &[u16]) -> Result<Rgb, ScoreError> {
    let data: Vec<[f32; 3]> = samples
        .iter()
        .map(|&s| {
            let v = f32::from(s) / 65535.0;
            [v, v, v]
        })
        .collect();
    Rgb::new(
        data,
        width as usize,
        height as usize,
        TransferCharacteristic::SRGB,
        ColorPrimaries::BT709,
    )
    .map_err(|e| ScoreError::Conversion(e.to_string()))
}

/// Score `candidate` against `reference` — both `width x height` 16-bit
/// grayscale planes. Higher is better; ~100 is visually identical
/// (`DEC-005`).
pub fn score(
    width: u32,
    height: u32,
    reference: &[u16],
    candidate: &[u16],
) -> Result<f64, ScoreError> {
    let reference_rgb = to_rgb(width, height, reference)?;
    let candidate_rgb = to_rgb(width, height, candidate)?;
    compute_frame_ssimulacra2(reference_rgb, candidate_rgb)
        .map_err(|e| ScoreError::Metric(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `size x size` checkerboard — `ssimulacra2` requires at least 8x8
    /// (`Ssimulacra2Error::InvalidImageSize`), so this is never degenerate.
    fn checkerboard(size: usize) -> Vec<u16> {
        (0..size * size)
            .map(|i| {
                let (x, y) = (i % size, i / size);
                if (x + y) % 2 == 0 {
                    60_000
                } else {
                    5_000
                }
            })
            .collect()
    }

    #[test]
    fn identical_inputs_score_at_least_ninetynine_point_nine() {
        let size = 16;
        let samples = checkerboard(size);
        let s = score(size as u32, size as u32, &samples, &samples).expect("scores");
        assert!(s >= 99.9, "identical inputs scored {s}, expected >= 99.9");
    }

    #[test]
    fn sixteen_to_eight_conversion_is_deterministic() {
        // Known-value probe pinning the normalisation formula itself
        // (AC3(iii)): sample / 65535.0, replicated into all three channels.
        let rgb = to_rgb(3, 1, &[0, 32_768, 65_535]).expect("valid 3x1 plane");
        let data = rgb.data();
        assert_eq!(data[0], [0.0, 0.0, 0.0]);
        assert_eq!(data[1], [32_768.0 / 65_535.0; 3]);
        assert_eq!(data[2], [1.0, 1.0, 1.0]);

        // Two independent runs of the full scoring pipeline on the same
        // inputs must be bit-equal — no hidden nondeterminism from the
        // metric's internal parallelism (AC3's own requirement).
        let size = 16;
        let reference = checkerboard(size);
        let candidate: Vec<u16> = reference.iter().map(|&v| 65_535 - v).collect();
        let first = score(size as u32, size as u32, &reference, &candidate).expect("scores");
        let second = score(size as u32, size as u32, &reference, &candidate).expect("scores");
        assert_eq!(
            first.to_bits(),
            second.to_bits(),
            "score must be bit-equal across runs: {first} vs {second}"
        );
    }
}
