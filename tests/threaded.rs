//! The opt-in row-band threading oracle (`SPEC-051`, `DEC-044`).
//!
//! The three `*_threaded` entry points split the **output** into contiguous row
//! bands over `std::thread::scope`. Threading must be output-invariant, so every
//! test here is a `dst_threaded == dst_single` byte-equality (the `DEC-041`
//! proof) against the existing single-thread function — never a re-derived
//! reference. Each was shown red before the threaded path existed (stubs that
//! wrote nothing).
//!
//! **Tier-A only** — synthetic fixtures, no corpus, no tools.

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded,
};
use irradiance::demosaic::{demosaic_into, demosaic_into_threaded};
use irradiance::develop::output_dimensions;
use irradiance::ifd::{ActiveArea, Cfa, DefaultCropOrigin, DefaultCropSize, Rational, Sensor};

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures.
// ─────────────────────────────────────────────────────────────────────────────

/// All four 2×2 Bayer phases (CFA colour indices: 0 = R, 1 = G, 2 = B).
const BAYER_PHASES: [[u8; 4]; 4] = [
    [0, 1, 1, 2], // RGGB
    [2, 1, 1, 0], // BGGR
    [1, 0, 2, 1], // GRBG
    [1, 2, 0, 1], // GBRG
];

/// A bare `width × height` sensor with the given Bayer phase and the colour
/// calibration the colour develop needs; geometry tags absent.
fn sensor(width: u32, height: u32, pattern: &[u8]) -> Sensor {
    let forward_matrix = [
        [60000, 30000, 6422],
        [20000, 70000, 10000],
        [10000, 10000, 62521],
    ]
    .iter()
    .flatten()
    .map(|&num| Rational { num, den: 100000 })
    .collect();
    {
        let mut built = Sensor::test_fixture();
        built.width = width;
        built.height = height;
        built.photometric = 32803;
        built.color_filter = Some(Cfa {
            repeat_dim: [2, 2],
            pattern: pattern.to_vec(),
            plane_color: vec![0, 1, 2],
            layout: 1,
        });
        built.forward_matrix_1 = Some(forward_matrix);
        built.as_shot_neutral = Some(vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ]);
        // Distinct per-slot blacks + a white below some samples, so the level
        // slot and the clamp both matter.
        built.black_levels = vec![100, 140, 160, 220];
        built.white_levels = vec![4000];
        built
    }
}

/// A pseudo-random (fixed-seed LCG) mosaic: real edges in every direction so the
/// Hamilton-Adams gradient classifiers take both branches, and values above the
/// white level so the clamp is exercised. Deterministic across runs.
fn noisy_mosaic(width: u32, height: u32) -> Vec<u16> {
    let mut state: u32 = 0x1234_5678;
    (0..(width as usize * height as usize))
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 16) % 4600) as u16
        })
        .collect()
}

/// Plane size of the colour geometry fixture: tall enough that four bands each
/// span several rows plus the halo.
const GEO_W: u32 = 22;
const GEO_H: u32 = 41;

/// A colour sensor with a non-trivial geometry: an **odd-top**, odd-left
/// ActiveArea (so the `SPEC-047` phase offset is load-bearing across every band
/// boundary), a `DefaultCrop` inset inside it, and the given orientation.
fn geometry_sensor(pattern: &[u8], orientation: Option<u32>) -> Sensor {
    let mut s = sensor(GEO_W, GEO_H, pattern);
    s.active_area = Some(ActiveArea {
        top: 3,
        left: 1,
        bottom: 39,
        right: 20,
    });
    s.default_crop_origin = Some(DefaultCropOrigin { x: 2, y: 1 });
    s.default_crop_size = Some(DefaultCropSize {
        width: 15,
        height: 31,
    });
    s.orientation = orientation;
    s
}

fn rgb_len(s: &Sensor) -> usize {
    let (w, h) = output_dimensions(s).expect("fixture geometry is valid");
    w as usize * h as usize * 3
}

fn single_demosaic(s: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let mut dst = vec![0u16; mosaic.len() * 3];
    demosaic_into(s, mosaic, &mut dst).expect("single-thread demosaic");
    dst
}

fn threaded_demosaic(s: &Sensor, mosaic: &[u16], threads: usize) -> Vec<u16> {
    // Pre-fill with a sentinel so a band the threaded path forgets to write
    // cannot pass by accident against a zero-filled single-thread buffer.
    let mut dst = vec![0xA5A5u16; mosaic.len() * 3];
    demosaic_into_threaded(s, mosaic, &mut dst, threads).expect("threaded demosaic");
    dst
}

fn single_colour(s: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let mut dst = vec![0u16; rgb_len(s)];
    develop_color_into(s, mosaic, &mut dst).expect("single-thread colour develop");
    dst
}

fn threaded_colour(s: &Sensor, mosaic: &[u16], threads: usize) -> Vec<u16> {
    let mut dst = vec![0xA5A5u16; rgb_len(s)];
    develop_color_into_threaded(s, mosaic, &mut dst, threads).expect("threaded colour develop");
    dst
}

fn single_f32(s: &Sensor, mosaic: &[u16]) -> Vec<u32> {
    let mut dst = vec![0f32; rgb_len(s)];
    develop_color_f32_into(s, mosaic, &mut dst).expect("single-thread f32 develop");
    dst.iter().map(|v| v.to_bits()).collect()
}

fn threaded_f32(s: &Sensor, mosaic: &[u16], threads: usize) -> Vec<u32> {
    let mut dst = vec![-1.0f32; rgb_len(s)];
    develop_color_f32_into_threaded(s, mosaic, &mut dst, threads).expect("threaded f32 develop");
    // Compare the bit patterns, so a NaN or a signed zero cannot hide a diff.
    dst.iter().map(|v| v.to_bits()).collect()
}

/// The mono demosaic sizes: tall images (several halo-sized bands), odd in both
/// dimensions, one with an axis below the five-pixel interior threshold, and the
/// degenerate shapes (fewer rows than threads).
const DEMOSAIC_SIZES: [(u32, u32); 7] =
    [(9, 37), (20, 24), (13, 61), (3, 17), (6, 2), (2, 2), (1, 1)];

// ─────────────────────────────────────────────────────────────────────────────
// The SPEC-051 failing tests.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn threads_one_equals_the_single_thread_entry() {
    // threads 0 and 1 are the existing single-thread path, byte for byte.
    for pattern in BAYER_PHASES {
        let s = sensor(20, 24, &pattern);
        let mosaic = noisy_mosaic(20, 24);
        let g = geometry_sensor(&pattern, Some(3));
        let geo_mosaic = noisy_mosaic(GEO_W, GEO_H);
        for threads in [0, 1] {
            assert_eq!(
                threaded_demosaic(&s, &mosaic, threads),
                single_demosaic(&s, &mosaic),
                "demosaic threads={threads} phase {pattern:?}"
            );
            assert_eq!(
                threaded_colour(&g, &geo_mosaic, threads),
                single_colour(&g, &geo_mosaic),
                "colour threads={threads} phase {pattern:?}"
            );
            assert_eq!(
                threaded_f32(&g, &geo_mosaic, threads),
                single_f32(&g, &geo_mosaic),
                "f32 threads={threads} phase {pattern:?}"
            );
        }
    }
}

#[test]
fn threaded_demosaic_equals_single_threaded() {
    for (w, h) in DEMOSAIC_SIZES {
        let mosaic = noisy_mosaic(w, h);
        for pattern in BAYER_PHASES {
            for odd_origin in [false, true] {
                let mut s = sensor(w, h, &pattern);
                if odd_origin {
                    // An odd ActiveArea origin shifts the CFA phase (SPEC-047);
                    // the band halo must not disturb it.
                    s.active_area = Some(ActiveArea {
                        top: 1,
                        left: 1,
                        bottom: h,
                        right: w,
                    });
                }
                let single = single_demosaic(&s, &mosaic);
                for threads in [2, 3, 4] {
                    assert_eq!(
                        threaded_demosaic(&s, &mosaic, threads),
                        single,
                        "{w}×{h} phase {pattern:?} odd_origin={odd_origin} threads={threads}"
                    );
                }
            }
        }
    }
}

#[test]
fn threaded_colour_develop_non_transposing_equals_single_threaded() {
    let mosaic = noisy_mosaic(GEO_W, GEO_H);
    for orientation in [None, Some(1), Some(2), Some(3), Some(4)] {
        for pattern in BAYER_PHASES {
            let s = geometry_sensor(&pattern, orientation);
            let single = single_colour(&s, &mosaic);
            for threads in [2, 4] {
                assert_eq!(
                    threaded_colour(&s, &mosaic, threads),
                    single,
                    "orientation {orientation:?} phase {pattern:?} threads={threads}"
                );
            }
        }
    }
    // And with no geometry tags at all (whole plane, identity orientation).
    for pattern in BAYER_PHASES {
        let s = sensor(GEO_W, GEO_H, &pattern);
        let single = single_colour(&s, &mosaic);
        for threads in [2, 4] {
            assert_eq!(
                threaded_colour(&s, &mosaic, threads),
                single,
                "whole plane phase {pattern:?} threads={threads}"
            );
        }
    }
}

#[test]
fn threaded_colour_develop_transposing_falls_back_and_still_matches() {
    // The 90° rotations fall back to the single-thread develop — still
    // byte-identical for any requested thread count.
    let mosaic = noisy_mosaic(GEO_W, GEO_H);
    for orientation in 5..=8 {
        for pattern in BAYER_PHASES {
            let s = geometry_sensor(&pattern, Some(orientation));
            let single = single_colour(&s, &mosaic);
            for threads in [2, 3, 4] {
                assert_eq!(
                    threaded_colour(&s, &mosaic, threads),
                    single,
                    "orientation {orientation} phase {pattern:?} threads={threads}"
                );
            }
        }
    }
}

#[test]
fn threaded_f32_equals_single_threaded() {
    let mosaic = noisy_mosaic(GEO_W, GEO_H);
    for orientation in [Some(1), Some(2), Some(3), Some(4), Some(6)] {
        for pattern in BAYER_PHASES {
            let s = geometry_sensor(&pattern, orientation);
            let single = single_f32(&s, &mosaic);
            for threads in [2, 4] {
                assert_eq!(
                    threaded_f32(&s, &mosaic, threads),
                    single,
                    "orientation {orientation:?} phase {pattern:?} threads={threads}"
                );
            }
        }
    }
}

#[test]
fn output_is_independent_of_thread_count() {
    let mosaic = noisy_mosaic(13, 61);
    let geo_mosaic = noisy_mosaic(GEO_W, GEO_H);
    for pattern in BAYER_PHASES {
        let s = sensor(13, 61, &pattern);
        let g = geometry_sensor(&pattern, Some(4));
        // The baseline is the single-thread render itself, so a threaded path
        // that is merely self-consistent (e.g. writes nothing) cannot pass.
        let demosaic_1 = single_demosaic(&s, &mosaic);
        let colour_1 = single_colour(&g, &geo_mosaic);
        let f32_1 = single_f32(&g, &geo_mosaic);
        for threads in [1, 2, 3, 5, 8] {
            // Run each count twice: scheduling must not matter either.
            for _ in 0..2 {
                assert_eq!(threaded_demosaic(&s, &mosaic, threads), demosaic_1);
                assert_eq!(threaded_colour(&g, &geo_mosaic, threads), colour_1);
                assert_eq!(threaded_f32(&g, &geo_mosaic, threads), f32_1);
            }
        }
    }
}

#[test]
fn band_boundary_reflects_at_the_true_edge() {
    // 15 rows over 2 bands puts the seam on an ODD interior row (7); over 15
    // bands every band is one row, so every row is a seam and both halos reach
    // a true edge; 50 bands is more than there are rows. In every case the
    // seam rows (and the true-edge rows, the only place reflection belongs)
    // must equal the single-thread render pixel for pixel.
    let (w, h) = (11u32, 15u32);
    let row = w as usize * 3;
    let mosaic = noisy_mosaic(w, h);
    for pattern in BAYER_PHASES {
        let s = sensor(w, h, &pattern);
        let single = single_demosaic(&s, &mosaic);
        for threads in [2, 15, 50] {
            let threaded = threaded_demosaic(&s, &mosaic, threads);
            for y in [0usize, 1, 6, 7, 8, 13, 14] {
                assert_eq!(
                    threaded[y * row..(y + 1) * row],
                    single[y * row..(y + 1) * row],
                    "phase {pattern:?} threads={threads} row {y}"
                );
            }
            assert_eq!(threaded, single, "phase {pattern:?} threads={threads}");
        }

        // Same on the colour path, through a vertical flip (orientation 3),
        // where output-band seams map to reversed source rows.
        let mut g = sensor(w, h, &pattern);
        g.orientation = Some(3);
        let single = single_colour(&g, &mosaic);
        for threads in [2, 15, 50] {
            assert_eq!(
                threaded_colour(&g, &mosaic, threads),
                single,
                "colour phase {pattern:?} threads={threads}"
            );
        }
    }
}

#[test]
fn threaded_errors_match_the_single_thread_errors() {
    // Validation happens once, before spawning, and yields the single-thread
    // path's typed error — never a panic.
    let s = sensor(10, 12, &BAYER_PHASES[0]);
    let mosaic = noisy_mosaic(10, 12);
    let mut short = vec![0u16; 10 * 12 * 3 - 1];
    let single = demosaic_into(&s, &mosaic, &mut short.clone());
    let threaded = demosaic_into_threaded(&s, &mosaic, &mut short, 4);
    assert_eq!(format!("{single:?}"), format!("{threaded:?}"));

    let mut no_cfa = s.clone();
    no_cfa.color_filter = None;
    let mut dst = vec![0u16; 10 * 12 * 3];
    let single = develop_color_into(&no_cfa, &mosaic, &mut dst.clone());
    let threaded = develop_color_into_threaded(&no_cfa, &mosaic, &mut dst, 4);
    assert!(threaded.is_err());
    assert_eq!(format!("{single:?}"), format!("{threaded:?}"));

    let mut no_matrix = s.clone();
    no_matrix.forward_matrix_1 = None;
    let mut dst = vec![0f32; 10 * 12 * 3];
    let single = develop_color_f32_into(&no_matrix, &mosaic, &mut dst.clone());
    let threaded = develop_color_f32_into_threaded(&no_matrix, &mosaic, &mut dst, 4);
    assert!(threaded.is_err());
    assert_eq!(format!("{single:?}"), format!("{threaded:?}"));

    let short_src = &mosaic[..mosaic.len() - 1];
    let mut dst = vec![0u16; 10 * 12 * 3];
    let single = develop_color_into(&s, short_src, &mut dst.clone());
    let threaded = develop_color_into_threaded(&s, short_src, &mut dst, 4);
    assert!(threaded.is_err());
    assert_eq!(format!("{single:?}"), format!("{threaded:?}"));
}
