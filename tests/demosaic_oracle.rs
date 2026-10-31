//! The demosaic oracle — bit-exact on analytic fixtures (`SPEC-030`).
//!
//! Bilinear is a fixed linear filter, so on a synthetic mosaic the output RGB
//! is **exactly computable**; this oracle asserts equality against hand-derived
//! expected tables (no tolerance), with a perturbed-kernel red-proof. There is
//! no external reference: `dnglab` emits only a full colour render and
//! implements no DNG opcodes, so it cannot isolate demosaic (`DEC-024`) — the
//! real-image perceptual check belongs to STAGE-008's ColorChecker ΔE.
//!
//! **Tier-A only** — every fixture is synthetic, no corpus and no tools, so CI
//! runs the whole file. The one test that drives the `irr demosaic` binary
//! builds its own synthetic CFA DNG (`tests/support/tiff.rs::cfa_dng_16bit`),
//! so it needs no corpus either.
//!
//! The expected tables were derived from the spec's own bilinear formulas
//! (round-half-up, phase-preserving reflect) and cross-checked with an
//! independent clean-room reference; a few representative pixels are also pinned
//! inline with their arithmetic, so the tables are human-verifiable without
//! rerunning anything.

#[path = "support/pnm.rs"]
#[allow(dead_code)] // only read_p6_rgb/RgbImage are used here; the rest is other tests'
mod pnm;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::demosaic::demosaic_into;
use irradiance::ifd::{Cfa, Compression, Sensor};
use irradiance::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Fixture builders
// ─────────────────────────────────────────────────────────────────────────────

/// A minimal `Sensor` for a synthetic plane: everything the demosaic ignores is
/// absent/empty, `color_filter` is the one field it reads. Mirrors
/// `tests/develop_oracle.rs::minimal_sensor`'s field set.
fn fixture_sensor(width: u32, height: u32, color_filter: Option<Cfa>) -> Sensor {
    Sensor {
        ifd_index: 0,
        width,
        height,
        bits_per_sample: 16,
        samples_per_pixel: 1,
        photometric: 32803,
        compression: Compression::Uncompressed,
        rows_per_strip: None,
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: None,
        white_level: None,
        black_level_repeat_dim: None,
        active_area: None,
        default_crop_origin: None,
        default_crop_size: None,
        orientation: None,
        opcode_lists: [false, false, false],
        opcode_list_1: None,
        opcode_list_3: None,
        color_filter,
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

/// A 2×2 Bayer CFA with the DNG-default plane colour, for the given phase
/// (e.g. `[0, 1, 1, 2]` = RGGB, `[2, 1, 1, 0]` = BGGR).
fn bayer(pattern: [u8; 4]) -> Cfa {
    Cfa {
        repeat_dim: [2, 2],
        pattern: pattern.to_vec(),
        plane_color: vec![0, 1, 2],
        layout: 1,
    }
}

const RGGB: [u8; 4] = [0, 1, 1, 2];
const BGGR: [u8; 4] = [2, 1, 1, 0];

/// Demosaic a mosaic through the real `demosaic_into`, returning the interleaved
/// RGB buffer. Panics (fine in a test) on any error — the error paths are their
/// own tests.
fn demosaic(sensor: &Sensor, mosaic: &[u16]) -> Vec<u16> {
    let mut dst = vec![0u16; mosaic.len() * 3];
    demosaic_into(sensor, mosaic, &mut dst).expect("demosaic_into on a valid fixture");
    dst
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture A — RGGB 4×4, every sample a multiple of 4 so EVERY average is exact
// (no half). The interior-site oracle. Expected table derived from the spec's
// bilinear formulas; the four interior site types are pinned inline below.
// ─────────────────────────────────────────────────────────────────────────────

#[rustfmt::skip]
const MOSAIC_A: [u16; 16] = [
    8,  40,  16,  48,
    80, 120, 88,  128,
    24, 56,  32,  64,
    96, 136, 104, 144,
];

#[rustfmt::skip]
const EXPECTED_A: [u16; 48] = [
    8,60,120,   12,40,120,  16,66,124,  16,48,128,
    16,80,120,  20,66,120,  24,88,124,  24,72,128,
    24,72,128,  28,56,128,  32,78,132,  32,64,136,
    24,96,136,  28,78,136,  32,104,140, 32,84,144,
];

#[test]
fn rggb_interior_matches_hand_computed_bilinear() {
    let sensor = fixture_sensor(4, 4, Some(bayer(RGGB)));
    let out = demosaic(&sensor, &MOSAIC_A);
    assert_eq!(
        out, EXPECTED_A,
        "full interleaved RGB must equal the hand table"
    );

    // Anchor the table at one interior pixel of each site type, with the
    // arithmetic shown — so a reader can confirm the table is right by hand.
    // Index into the interleaved output: pixel (x, y) is out[(y*4 + x)*3 ..].
    let px = |x: usize, y: usize| {
        let i = (y * 4 + x) * 3;
        (out[i], out[i + 1], out[i + 2])
    };

    // (2,2)=32 is a RED site: R=self=32; G=(up88+down104+left56+right64+2)/4=78;
    // B=(ul120+ur128+dl136+dr144+2)/4=132.
    assert_eq!(px(2, 2), (32, 78, 132));
    // (1,1)=120 is a BLUE site: B=self=120; G=(40+56+80+88+2)/4=66;
    // R=(ul8+ur16+dl24+dr32+2)/4=20.
    assert_eq!(px(1, 1), (20, 66, 120));
    // (1,2)=56 is GREEN on an R row: G=self=56; R=(left24+right32+1)/2=28;
    // B=(up120+down136+1)/2=128.
    assert_eq!(px(1, 2), (28, 56, 128));
    // (2,1)=88 is GREEN on a B row: G=self=88; R=(up16+down32+1)/2=24;
    // B=(left120+right128+1)/2=124.
    assert_eq!(px(2, 1), (24, 88, 124));
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture B — RGGB 4×4 holding 0..15. Many odd sums, so averages hit halves and
// pin round-half-up; the corner/edge pixels exercise the reflect border. Also
// the subject of the red-proof below.
// ─────────────────────────────────────────────────────────────────────────────

#[rustfmt::skip]
const MOSAIC_B: [u16; 16] = [
    0,  1,  2,  3,
    4,  5,  6,  7,
    8,  9,  10, 11,
    12, 13, 14, 15,
];

#[rustfmt::skip]
const EXPECTED_B: [u16; 48] = [
    0,3,5,    1,1,5,    2,4,6,    2,3,7,
    4,4,5,    5,5,5,    6,6,6,    6,7,7,
    8,9,9,    9,9,9,    10,10,10, 10,11,11,
    8,12,13,  9,11,13,  10,14,14, 10,13,15,
];

#[test]
fn edges_and_corners_follow_the_reflect_policy() {
    let sensor = fixture_sensor(4, 4, Some(bayer(RGGB)));
    let out = demosaic(&sensor, &MOSAIC_B);
    assert_eq!(
        out, EXPECTED_B,
        "full interleaved RGB (edges + corners) must equal the hand table"
    );

    let px = |x: usize, y: usize| {
        let i = (y * 4 + x) * 3;
        (out[i], out[i + 1], out[i + 2])
    };

    // Corner (0,0)=0 is a RED site. Reflect mirrors: up→(0,1)=4, left→(1,0)=1,
    // and all four diagonals→(1,1)=5. R=self=0;
    // G=(up4+down4+left1+right1+2)/4 = 12/4 = 3 (true 2.5, ROUNDED HALF UP);
    // B=(5+5+5+5+2)/4 = 22/4 = 5.
    assert_eq!(px(0, 0), (0, 3, 5));
    // Top edge (1,0)=1 is GREEN on an R row. up reflects to (1,1)=5.
    // R=(left0+right2+1)/2 = 3/2 = 1; B=(up5+down5+1)/2 = 11/2 = 5.
    assert_eq!(px(1, 0), (1, 1, 5));
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture C — BGGR 4×4: the SAME code must demosaic it correctly, proving the
// phase is read from the pattern, not hard-coded to RGGB.
// ─────────────────────────────────────────────────────────────────────────────

#[rustfmt::skip]
const MOSAIC_C: [u16; 16] = [
    10,  20,  30,  40,
    50,  60,  70,  80,
    90,  100, 110, 120,
    130, 140, 150, 160,
];

#[rustfmt::skip]
const EXPECTED_C: [u16; 48] = [
    60,35,10,    60,20,20,    70,50,30,    80,40,30,
    60,50,50,    60,60,60,    70,70,70,    80,75,70,
    100,95,90,   100,100,100, 110,110,110, 120,120,110,
    140,130,90,  140,120,100, 150,150,110, 160,135,110,
];

#[test]
fn phase_is_driven_by_the_cfa_pattern() {
    let sensor = fixture_sensor(4, 4, Some(bayer(BGGR)));
    let out = demosaic(&sensor, &MOSAIC_C);
    assert_eq!(
        out, EXPECTED_C,
        "BGGR must demosaic correctly with the same code"
    );

    // The telling pixel: corner (0,0)=10 is a BLUE site under BGGR (it would be
    // RED under RGGB). B=self=10, and R comes from the diagonals (all reflect to
    // (1,1)=60): R=(60*4+2)/4=60 — a hard-coded-RGGB kernel would put 10 in R.
    let (r, _g, b) = (out[0], out[1], out[2]);
    assert_eq!((r, b), (60, 10), "corner is a BLUE site, not a RED one");
}

// ─────────────────────────────────────────────────────────────────────────────
// The red-proof: a perturbed kernel must change the OUTPUT (DEC-017's lesson,
// per tests/plane_oracle.rs — assert mutant ≠ honest, not merely that it
// compiled), with the honest control green. SPEC-030's spec sanctions this
// in-test "injected neighbour swap" as the Tier-A alternative to a crate-copy
// rebuild.
// ─────────────────────────────────────────────────────────────────────────────

/// A local bilinear reimplementation, deliberately faultable. With
/// `drop_down == false` it is an INDEPENDENT correct kernel (cross-checked
/// against `demosaic_into` in the control test). With `drop_down == true` it
/// injects one fault: at a red/blue site the green channel drops its `down`
/// neighbour from the 4-average (sums `up + left + right`, still divides by 4) —
/// a genuine dropped neighbour, not a semantic no-op.
fn local_demosaic(sensor: &Sensor, mosaic: &[u16], drop_down: bool) -> Vec<u16> {
    let cfa = sensor.color_filter.as_ref().expect("fixture has a CFA");
    let w = i64::from(sensor.width);
    let h = i64::from(sensor.height);
    let colour = |x: i64, y: i64| -> u8 {
        let idx = (y.rem_euclid(2) * 2 + x.rem_euclid(2)) as usize;
        cfa.pattern[idx]
    };
    let reflect = |i: i64, n: i64| -> i64 {
        let r = if i < 0 {
            -i
        } else if i >= n {
            2 * n - 2 - i
        } else {
            i
        };
        r.clamp(0, (n - 1).max(0))
    };
    let at = |x: i64, y: i64| -> u32 {
        let index = (reflect(y, h) * w + reflect(x, w)) as usize;
        u32::from(mosaic[index])
    };
    // Round-half-up for two (`(a+b+1)/2`, which is exactly `div_ceil(2)`) and
    // four (`(a+b+c+d+2)/4`) — the spec's own bilinear averages.
    let avg2 = |a: u32, b: u32| (a + b).div_ceil(2) as u16;
    let avg4 = |a: u32, b: u32, c: u32, d: u32| ((a + b + c + d + 2) / 4) as u16;

    let mut out = Vec::with_capacity(mosaic.len() * 3);
    for y in 0..h {
        for x in 0..w {
            let centre = at(x, y);
            let left = at(x - 1, y);
            let right = at(x + 1, y);
            let up = at(x, y - 1);
            let real_down = at(x, y + 1);
            // The injected fault: `down` is zeroed for the R/B green average.
            let down = if drop_down { 0 } else { real_down };
            let ul = at(x - 1, y - 1);
            let ur = at(x + 1, y - 1);
            let dl = at(x - 1, y + 1);
            let dr = at(x + 1, y + 1);
            let (r, g, b) = match colour(x, y) {
                0 => (
                    centre as u16,
                    avg4(up, down, left, right),
                    avg4(ul, ur, dl, dr),
                ),
                2 => (
                    avg4(ul, ur, dl, dr),
                    avg4(up, down, left, right),
                    centre as u16,
                ),
                _ => {
                    // Green uses its own (undropped) vertical neighbour.
                    let green = centre as u16;
                    if colour(x + 1, y) == 0 {
                        (avg2(left, right), green, avg2(up, real_down))
                    } else {
                        (avg2(up, real_down), green, avg2(left, right))
                    }
                }
            };
            out.push(r);
            out.push(g);
            out.push(b);
        }
    }
    out
}

#[test]
fn the_honest_control_is_green() {
    let sensor = fixture_sensor(4, 4, Some(bayer(RGGB)));
    // The real kernel reproduces the hand table...
    assert_eq!(demosaic(&sensor, &MOSAIC_B), EXPECTED_B);
    // ...and the local reference (no fault) agrees with it, so the mutant below
    // is a one-change perturbation of a kernel known to be correct.
    assert_eq!(
        local_demosaic(&sensor, &MOSAIC_B, false),
        EXPECTED_B.to_vec(),
        "the unmutated local reference must match the real kernel + the hand table"
    );
}

#[test]
fn a_perturbed_kernel_turns_the_oracle_red() {
    let sensor = fixture_sensor(4, 4, Some(bayer(RGGB)));
    let honest = demosaic(&sensor, &MOSAIC_B);
    let mutant = local_demosaic(&sensor, &MOSAIC_B, true);

    // The load-bearing clause (plane_oracle.rs's lesson): assert the OUTPUT
    // changed. A mutation that produced the same bytes would have caught
    // nothing. e.g. red site (2,2): honest G=(6+14+9+11+2)/4=10, mutant drops
    // `down`=14 → (6+9+11+2)/4=7.
    assert_ne!(
        mutant, honest,
        "the dropped-neighbour fault did NOT change the output — it is a no-op and this \
         red-proof has caught nothing"
    );
    // And the oracle's expected table rejects the mutant — which is exactly
    // what 'the oracle goes red on a perturbed kernel' means.
    assert_ne!(mutant, EXPECTED_B.to_vec());
}

// ─────────────────────────────────────────────────────────────────────────────
// The no-panic / typed-error contract
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mono_or_absent_cfa_is_a_typed_error() {
    // A monochrome plane (color_filter == None) has nothing to demosaic.
    let sensor = fixture_sensor(4, 4, None);
    let mosaic = [0u16; 16];
    let mut dst = [0u16; 48];
    let err =
        demosaic_into(&sensor, &mosaic, &mut dst).expect_err("a mono plane must be a typed error");
    assert!(
        matches!(err, Error::NotAColorFilterArray),
        "expected NotAColorFilterArray, got {err:?}"
    );
}

#[test]
fn an_unsupported_cfa_is_a_typed_error() {
    let mosaic = [0u16; 16];
    let mut dst = [0u16; 48];

    // A non-RGGB-family 2×2 pattern (two reds, no blue) is not a Bayer mosaic.
    let not_bayer = Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 0, 1, 2],
        plane_color: vec![0, 1, 2],
        layout: 1,
    };
    let sensor = fixture_sensor(4, 4, Some(not_bayer));
    let err = demosaic_into(&sensor, &mosaic, &mut dst).expect_err("not a Bayer mosaic");
    assert!(
        matches!(err, Error::UnsupportedCfaPattern { .. }),
        "expected UnsupportedCfaPattern, got {err:?}"
    );

    // A non-2×2 repeat dim (an X-Trans-shaped 6×6) is also unsupported here.
    let xtrans_shaped = Cfa {
        repeat_dim: [6, 6],
        pattern: vec![1; 36],
        plane_color: vec![0, 1, 2],
        layout: 1,
    };
    let sensor = fixture_sensor(4, 4, Some(xtrans_shaped));
    let err = demosaic_into(&sensor, &mosaic, &mut dst).expect_err("repeat_dim != [2,2]");
    assert!(
        matches!(err, Error::UnsupportedCfaPattern { .. }),
        "expected UnsupportedCfaPattern, got {err:?}"
    );
}

#[test]
fn wrong_buffer_lengths_are_typed_errors() {
    let sensor = fixture_sensor(4, 4, Some(bayer(RGGB)));

    // src too short.
    let short_src = [0u16; 15];
    let mut dst = [0u16; 48];
    let err = demosaic_into(&sensor, &short_src, &mut dst).expect_err("src is one short");
    assert!(
        matches!(
            err,
            Error::DemosaicSourceWrongLength {
                expected: 16,
                actual: 15
            }
        ),
        "expected DemosaicSourceWrongLength{{16,15}}, got {err:?}"
    );

    // dst wrong (not width*height*3).
    let src = [0u16; 16];
    let mut wrong_dst = [0u16; 47];
    let err = demosaic_into(&sensor, &src, &mut wrong_dst).expect_err("dst is one short");
    assert!(
        matches!(
            err,
            Error::RgbBufferWrongLength {
                expected: 48,
                actual: 47
            }
        ),
        "expected RgbBufferWrongLength{{48,47}}, got {err:?}"
    );
}

#[test]
fn degenerate_dims_do_not_panic() {
    // 1×1: every neighbour reflects to the single pixel; all channels = it.
    let sensor = fixture_sensor(1, 1, Some(bayer(RGGB)));
    assert_eq!(demosaic(&sensor, &[777]), vec![777, 777, 777]);

    // 2×2: the full Bayer quad, every neighbour one reflect away.
    let sensor = fixture_sensor(2, 2, Some(bayer(RGGB)));
    #[rustfmt::skip]
    let expected_2x2 = vec![
        10,25,40,  10,20,40,
        10,30,40,  10,25,40,
    ];
    assert_eq!(demosaic(&sensor, &[10, 20, 30, 40]), expected_2x2);

    // 3×5: odd width AND odd height — the phase at the far edge differs from the
    // near edge, so a wrong parity at the border would show here.
    let sensor = fixture_sensor(3, 5, Some(bayer(RGGB)));
    let mosaic: Vec<u16> = (0..15u16).map(|i| i * 7).collect();
    #[rustfmt::skip]
    let expected_3x5 = vec![
        0,14,28,   7,7,28,    14,21,28,
        21,21,28,  28,28,28,  35,35,28,
        42,46,49,  49,49,49,  56,53,49,
        63,63,70,  70,70,70,  77,77,70,
        84,77,70,  91,91,70,  98,84,70,
    ];
    assert_eq!(demosaic(&sensor, &mosaic), expected_3x5);
}

// ─────────────────────────────────────────────────────────────────────────────
// The `irr demosaic` surfacing — the first multi-channel irr output, a P6 PPM.
// Tier-A: a synthetic 16-bit CFA DNG, no corpus.
// ─────────────────────────────────────────────────────────────────────────────

/// Write `bytes` to a uniquely-named temp file, run `irr demosaic` on it, and
/// return the process output. The temp file is removed before returning.
fn run_irr_demosaic(dng: &[u8]) -> std::process::Output {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "irradiance-demosaic-{}-{nanos}.dng",
        std::process::id()
    ));
    std::fs::write(&path, dng).expect("write temp DNG");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_irr"))
        .arg("demosaic")
        .arg(&path)
        .output()
        .expect("spawn irr demosaic");
    let _ = std::fs::remove_file(&path);
    output
}

#[test]
fn irr_demosaic_emits_a_valid_p6_ppm() {
    // A 4×3 RGGB 16-bit mosaic with a real strip, so unpack → demosaic produce
    // an honest image (the values themselves are the oracle tests' job).
    let samples: Vec<u16> = (0..12u16).map(|i| i * 1000).collect();
    let dng = tiff::cfa_dng_16bit(tiff::Order::Little, 4, 3, &samples);

    let output = run_irr_demosaic(&dng);
    assert!(
        output.status.success(),
        "irr demosaic failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // stdout is the binary P6 image; a standard PNM reader must open it.
    let image = pnm::read_p6_rgb(&output.stdout).expect("irr demosaic must emit a valid P6 PPM");
    assert_eq!((image.width, image.height), (4, 3));
    assert_eq!(
        image.samples.len(),
        4 * 3 * 3,
        "a P6 RGB image of 4×3 has width*height*3 samples"
    );
}
