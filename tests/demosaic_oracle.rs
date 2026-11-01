//! The demosaic oracle — bit-exact on analytic fixtures (`SPEC-031`).
//!
//! Hamilton-Adams (Adams-Hamilton) is deterministic integer arithmetic, so on a
//! synthetic mosaic the output RGB is **exactly computable**; this oracle asserts
//! equality against hand-derived expected tables (no tolerance), with a
//! perturbed-kernel red-proof. There is no external reference: `dnglab` emits
//! only a full colour render and implements no DNG opcodes, so it cannot isolate
//! demosaic (`DEC-024`) — the real-image perceptual check belongs to STAGE-008's
//! ColorChecker ΔE.
//!
//! Per **`DEC-031`** the quality kernel is Hamilton-Adams (US 5,629,734,
//! expired), NOT Malvar-He-Cutler (US 7,502,505, still live) — a patented method
//! in a permissive crate defeats the thesis (clean-room beats copyright, not
//! patents). `SPEC-030`'s bilinear baseline + its oracle harness are the base
//! this extends.
//!
//! **Tier-A only** — every fixture is synthetic, no corpus and no tools, so CI
//! runs the whole file. The one test that drives the `irr demosaic` binary
//! builds its own synthetic CFA DNG (`tests/support/tiff.rs::cfa_dng_16bit`),
//! so it needs no corpus either.
//!
//! The expected tables were derived from `SPEC-031`'s own HA formulas
//! (classifier-chosen axis, second-difference correction, round-half-up floor
//! division, phase-preserving reflect) and cross-checked with an independent
//! clean-room reference (`local_hamilton_adams` below, also the red-proof's
//! subject); representative pixels are pinned inline with their arithmetic, so
//! the tables are human-verifiable without rerunning anything.

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
// Fixture HA — RGGB 6×6, the primary bit-exact table. 6×6 is wide/tall enough
// that the interior pixels (x,y ∈ {2,3}) have GENUINE offset-2 neighbours (no
// reflection), while the border pixels exercise the reflect policy. Expected
// table derived from SPEC-031's HA formulas; four representative pixels are
// pinned inline below with their full arithmetic.
// ─────────────────────────────────────────────────────────────────────────────

#[rustfmt::skip]
const MOSAIC_HA: [u16; 36] = [
     12,  64,  28,  96,  44, 128,
     80,  40, 112,  60, 144,  72,
     36, 100,  52, 132,  68, 160,
    120,  56, 152,  76, 184,  88,
     60, 136,  72, 168,  84, 200,
    160,  84, 192, 100, 224, 108,
];

#[rustfmt::skip]
const EXPECTED_HA: [u16; 108] = [
    12,56,26,   18,64,30,   28,80,36,   33,96,43,   44,124,58,  46,128,61,
    27,80,38,   32,85,40,   47,112,52,  53,130,60,  60,144,68,  63,150,72,
    36,92,44,   42,100,48,  52,116,55,  60,132,59,  68,150,69,  73,160,75,
    53,120,59,  52,115,56,  71,152,71,  80,170,76,  84,184,84,  87,190,88,
    60,130,63,  64,136,66,  72,152,71,  77,168,77,  84,187,88,  91,200,94,
    75,160,78,  82,172,84,  92,192,93,  98,210,100, 103,224,107, 105,228,108,
];

#[test]
fn adams_hamilton_rggb_matches_hand_computed() {
    let sensor = fixture_sensor(6, 6, Some(bayer(RGGB)));
    let out = demosaic(&sensor, &MOSAIC_HA);
    assert_eq!(
        out, EXPECTED_HA,
        "full interleaved RGB must equal the hand-computed Hamilton-Adams table"
    );

    // Anchor the table with the arithmetic shown, so a reader can verify it by
    // hand. Interleaved index: pixel (x, y) is out[(y*6 + x)*3 ..]. All four
    // interior pixels below have in-image offset-2 neighbours (no reflection).
    let px = |x: usize, y: usize| {
        let i = (y * 6 + x) * 3;
        (out[i], out[i + 1], out[i + 2])
    };

    // (2,2)=52 RED site. Neighbours W=100 E=132 N=112 S=152 WW=36 EE=68 NN=28 SS=72.
    //   lap_h = 36-104+68 = 0;  lap_v = 28-104+72 = -4.
    //   CLh = |100-132|+0 = 32;  CLv = |112-152|+4 = 44  → CLh<CLv, green axis = horizontal.
    //   gh = 2*(100+132)-0 = 464 → rnd4 = (464+2)/4 = 116. G=116, R=self=52.
    //   B from diagonals NW=40 NE=60 SW=56 SE=76, greens gNW=85 gNE=130 gSW=115 gSE=170:
    //     dp=2*(40+76)-(85-232+170)=232-23=209; dn=2*(60+56)-(130-232+115)=232-13=219;
    //     CLp=|76-40|+|23|=59; CLn=|56-60|+|13|=17 → CLn<CLp, B=rnd4(dn)=(219+2)/4=55.
    assert_eq!(px(2, 2), (52, 116, 55));
    // (3,3)=76 BLUE site. green = 170 (CLh=40<CLv=44, horizontal); B=self=76;
    //   R=80 (diagonals, lower-gradient diagonal picked).
    assert_eq!(px(3, 3), (80, 170, 76));
    // (2,3)=152 GREEN site on a B row. G=self=152; horizontal neighbours are RED,
    //   vertical BLUE — both colour-difference axes evaluate to 71 here.
    assert_eq!(px(2, 3), (71, 152, 71));
    // (0,0)=12 RED corner. Reflect brings offset-1/2 back in range; G=56, B=26.
    assert_eq!(px(0, 0), (12, 56, 26));
}

// ─────────────────────────────────────────────────────────────────────────────
// Directionality — green follows the LOW-gradient axis. The mosaic depends only
// on x (flat vertically, a curved horizontal profile), so at an interior R/B
// site the vertical gradient is 0 and the horizontal gradient is large: HA must
// interpolate green VERTICALLY, which on a vertically-flat column returns the
// centre's own value — distinct from bilinear's 4-neighbour average.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn green_follows_the_low_gradient_axis() {
    // Each row identical: a symmetric horizontal "peak" at x=2.
    let col = [500u16, 1500, 4000, 1500, 500];
    let mosaic: Vec<u16> = (0..5).flat_map(|_| col.iter().copied()).collect();
    let sensor = fixture_sensor(5, 5, Some(bayer(RGGB)));
    let out = demosaic(&sensor, &mosaic);

    // (2,2) is a RED site. Vertically the column is flat (N=S=NN=SS=C=4000) so
    //   CLv = 0; horizontally CLh = |1500-1500| + |500-8000+500| = 7000. HA picks
    //   the vertical axis → green = 2*(4000+4000)/4 = 4000 (the centre itself).
    let green_22 = out[(2 * 5 + 2) * 3 + 1];
    assert_eq!(green_22, 4000, "green must follow the flat (vertical) axis");

    // Bilinear's 4-neighbour average would be (4000+4000+1500+1500+2)/4 = 2750 —
    // the directional kernel is demonstrably NOT doing that here.
    let bilinear_4avg = (4000 + 4000 + 1500 + 1500 + 2) / 4;
    assert_eq!(bilinear_4avg, 2750);
    assert_ne!(
        green_22, bilinear_4avg,
        "a bilinear 4-average would give 2750; HA's directional green is 4000"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Classifier tie — when CLh == CLv the green is rnd8(gh + gv), the AVERAGE of
// both axis predictors, which differs from either single-axis rnd4. The fixture
// is built so the horizontal and vertical gradients at (2,2) are equal (400)
// while the two predictors differ (gh=4000, gv=3800).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn classifier_tie_averages_both_predictors() {
    #[rustfmt::skip]
    let mosaic: [u16; 25] = [
        1000, 1000, 1000, 1000, 1000,
        1000, 1000,  900, 1000, 1000,
         600,  800, 1000, 1200, 1400,
        1000, 1000, 1100, 1000, 1000,
        1000, 1000, 1200, 1000, 1000,
    ];
    let sensor = fixture_sensor(5, 5, Some(bayer(RGGB)));
    let out = demosaic(&sensor, &mosaic);

    // (2,2)=1000 RED site. W=800 E=1200 WW=600 EE=1400 → lap_h=0, CLh=|800-1200|=400.
    //   N=900 S=1100 NN=1000 SS=1200 → lap_v=1000-2000+1200=200, CLv=|900-1100|+200=400.
    //   TIE. gh=2*(800+1200)-0=4000; gv=2*(900+1100)-200=3800.
    //   green = rnd8(gh+gv) = (4000+3800+4)/8 = 975 — NOT rnd4(gh)=1000, NOT rnd4(gv)=950.
    let green_22 = out[(2 * 5 + 2) * 3 + 1];
    assert_eq!(green_22, 975, "the tie must average both predictors (rnd8)");
    assert_ne!(green_22, 1000, "not the horizontal-only predictor rnd4(gh)");
    assert_ne!(green_22, 950, "not the vertical-only predictor rnd4(gv)");
}

// ─────────────────────────────────────────────────────────────────────────────
// DC preservation — every HA predictor sums to 1 (an average minus a zero-sum
// second difference), so a CONSTANT mosaic must come out constant on all three
// channels, independent of phase. The invariant the oracle can assert without
// knowing the exact weights.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn constant_mosaic_is_preserved_on_all_channels() {
    for pattern in [RGGB, BGGR] {
        let sensor = fixture_sensor(5, 5, Some(bayer(pattern)));
        let out = demosaic(&sensor, &[4242u16; 25]);
        assert!(
            out.iter().all(|&v| v == 4242),
            "a constant mosaic must demosaic to that constant on R, G and B (phase {pattern:?})"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase — the SAME code must demosaic a BGGR mosaic correctly, proving the phase
// is read from the pattern, not hard-coded to RGGB.
// ─────────────────────────────────────────────────────────────────────────────

#[rustfmt::skip]
const MOSAIC_BGGR: [u16; 25] = [
     10,  20,  30,  40,  50,
     60,  70,  80,  90, 100,
    110, 120, 130, 140, 150,
    160, 170, 180, 190, 200,
    210, 220, 230, 240, 250,
];

#[rustfmt::skip]
const EXPECTED_BGGR: [u16; 75] = [
    43,10,10,    48,20,20,    55,30,30,    63,40,40,    68,50,50,
    68,60,60,    70,65,68,    80,80,80,    90,95,93,    93,100,100,
    118,110,110, 123,120,120, 130,130,130, 138,140,140, 143,150,150,
    168,160,160, 170,165,168, 180,180,180, 190,195,193, 193,200,200,
    193,210,210, 198,220,220, 205,230,230, 213,240,240, 218,250,250,
];

#[test]
fn phase_is_driven_by_the_pattern_bggr() {
    let sensor = fixture_sensor(5, 5, Some(bayer(BGGR)));
    let out = demosaic(&sensor, &MOSAIC_BGGR);
    assert_eq!(
        out, EXPECTED_BGGR,
        "BGGR must demosaic correctly with the same code path"
    );

    // Corner (0,0)=10 is a BLUE site under BGGR (it would be RED under RGGB).
    // B=self=10, and R comes from the diagonals (all reflect to (1,1)=70, with
    // its green) → R=43, not the centre 10 a hard-coded-RGGB kernel would store.
    let (r, _g, b) = (out[0], out[1], out[2]);
    assert_eq!((r, b), (43, 10), "corner is a BLUE site, not a RED one");
}

// ─────────────────────────────────────────────────────────────────────────────
// The red-proof: a perturbed Hamilton-Adams step must change the OUTPUT (DEC-017
// / tests/plane_oracle.rs — assert mutant ≠ honest, not merely that it
// compiled), with the honest control green. `local_hamilton_adams` is an
// INDEPENDENT reimplementation of the full two-pass kernel; with
// `flip_green_axis == false` it is the control (cross-checked against
// `demosaic_into` and the hand table), and with `flip_green_axis == true` it
// injects one real fault: pass-1 green picks the HIGHER-gradient axis instead of
// the lower one — a genuine directional inversion, not a semantic no-op.
// ─────────────────────────────────────────────────────────────────────────────

fn local_hamilton_adams(sensor: &Sensor, mosaic: &[u16], flip_green_axis: bool) -> Vec<u16> {
    let cfa = sensor.color_filter.as_ref().expect("fixture has a CFA");
    let pattern = &cfa.pattern;
    let w = i64::from(sensor.width);
    let h = i64::from(sensor.height);
    let colour = |x: i64, y: i64| -> u8 {
        let idx = (y.rem_euclid(2) * 2 + x.rem_euclid(2)) as usize;
        pattern[idx]
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
    let s =
        |x: i64, y: i64| -> i64 { i64::from(mosaic[(reflect(y, h) * w + reflect(x, w)) as usize]) };
    let rnd4 = |n: i64| (n + 2).div_euclid(4);
    let rnd8 = |n: i64| (n + 4).div_euclid(8);
    let clamp = |v: i64| v.clamp(0, 65535) as u16;

    // ---- pass 1: green everywhere ----
    let mut g = vec![0i64; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if colour(x, y) == 1 {
                g[idx] = s(x, y);
                continue;
            }
            let c = s(x, y);
            let lap_h = s(x - 2, y) - 2 * c + s(x + 2, y);
            let lap_v = s(x, y - 2) - 2 * c + s(x, y + 2);
            let cl_h = (s(x - 1, y) - s(x + 1, y)).abs() + lap_h.abs();
            let cl_v = (s(x, y - 1) - s(x, y + 1)).abs() + lap_v.abs();
            let gh = 2 * (s(x - 1, y) + s(x + 1, y)) - lap_h;
            let gv = 2 * (s(x, y - 1) + s(x, y + 1)) - lap_v;
            // The injected fault inverts the axis comparison.
            let (pick_h, pick_v) = if flip_green_axis {
                (cl_h > cl_v, cl_v > cl_h)
            } else {
                (cl_h < cl_v, cl_v < cl_h)
            };
            let val = if pick_h {
                rnd4(gh)
            } else if pick_v {
                rnd4(gv)
            } else {
                rnd8(gh + gv)
            };
            g[idx] = i64::from(clamp(val));
        }
    }
    let gg = |x: i64, y: i64| -> i64 { g[(reflect(y, h) * w + reflect(x, w)) as usize] };

    // ---- pass 2: red + blue, reading pass-1 green ----
    let cdiff = |p1: i64, p2: i64, g1: i64, gc: i64, g2: i64| -> u16 {
        clamp(rnd4(2 * (p1 + p2) - (g1 - 2 * gc + g2)))
    };
    let diag = |x: i64, y: i64, gc: i64| -> u16 {
        let (nw, ne, sw, se) = (
            s(x - 1, y - 1),
            s(x + 1, y - 1),
            s(x - 1, y + 1),
            s(x + 1, y + 1),
        );
        let (g_nw, g_ne, g_sw, g_se) = (
            gg(x - 1, y - 1),
            gg(x + 1, y - 1),
            gg(x - 1, y + 1),
            gg(x + 1, y + 1),
        );
        let lap_p = g_nw - 2 * gc + g_se;
        let lap_n = g_ne - 2 * gc + g_sw;
        let dp = 2 * (nw + se) - lap_p;
        let dn = 2 * (ne + sw) - lap_n;
        let cl_p = (se - nw).abs() + lap_p.abs();
        let cl_n = (sw - ne).abs() + lap_n.abs();
        if cl_p < cl_n {
            clamp(rnd4(dp))
        } else if cl_n < cl_p {
            clamp(rnd4(dn))
        } else {
            clamp(rnd8(dp + dn))
        }
    };

    let mut out = vec![0u16; (w * h * 3) as usize];
    for y in 0..h {
        for x in 0..w {
            let base = ((y * w + x) * 3) as usize;
            let gc = gg(x, y);
            let c = s(x, y);
            let (r, b) = match colour(x, y) {
                1 => {
                    // green site: horizontal neighbours one primary, vertical the other
                    let val_h = cdiff(s(x - 1, y), s(x + 1, y), gg(x - 1, y), gc, gg(x + 1, y));
                    let val_v = cdiff(s(x, y - 1), s(x, y + 1), gg(x, y - 1), gc, gg(x, y + 1));
                    if colour(x + 1, y) == 0 {
                        (val_h, val_v)
                    } else {
                        (val_v, val_h)
                    }
                }
                0 => (clamp(c), diag(x, y, gc)), // red site
                _ => (diag(x, y, gc), clamp(c)), // blue site
            };
            out[base] = r;
            out[base + 1] = clamp(gc);
            out[base + 2] = b;
        }
    }
    out
}

#[test]
fn the_honest_control_is_green() {
    let sensor = fixture_sensor(6, 6, Some(bayer(RGGB)));
    // The real kernel reproduces the hand table...
    assert_eq!(demosaic(&sensor, &MOSAIC_HA), EXPECTED_HA);
    // ...and the independent local reference (no fault) agrees with it, so the
    // mutant below is a one-change perturbation of a kernel known to be correct.
    assert_eq!(
        local_hamilton_adams(&sensor, &MOSAIC_HA, false),
        EXPECTED_HA.to_vec(),
        "the unmutated local reference must match the real kernel + the hand table"
    );
}

#[test]
fn a_perturbed_hamilton_adams_step_turns_the_oracle_red() {
    let sensor = fixture_sensor(6, 6, Some(bayer(RGGB)));
    let honest = demosaic(&sensor, &MOSAIC_HA);
    let mutant = local_hamilton_adams(&sensor, &MOSAIC_HA, true);

    // The load-bearing clause (plane_oracle.rs's lesson): assert the OUTPUT
    // changed. At (2,2) the honest axis is horizontal (CLh=32<CLv=44) → G=116;
    // the inverted classifier picks vertical → G=rnd4(gv)=rnd4(532)=133, which
    // then cascades into the red/blue of neighbouring pixels too.
    assert_ne!(
        mutant, honest,
        "the inverted-axis fault did NOT change the output — it is a no-op and this \
         red-proof has caught nothing"
    );
    // And the oracle's expected table rejects the mutant — which is exactly what
    // 'the oracle goes red on a perturbed kernel' means.
    assert_ne!(mutant, EXPECTED_HA.to_vec());
}

// ─────────────────────────────────────────────────────────────────────────────
// The no-panic / typed-error contract (carried over from SPEC-030).
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
    // 1×1: every neighbour reflects to the single pixel; all channels = it (DC).
    let sensor = fixture_sensor(1, 1, Some(bayer(RGGB)));
    assert_eq!(demosaic(&sensor, &[777]), vec![777, 777, 777]);

    // 2×2: the full Bayer quad, every neighbour one or two reflects away (the
    // offset-2 samples clamp safely into [0, 1] on a 2-wide axis).
    let sensor = fixture_sensor(2, 2, Some(bayer(RGGB)));
    #[rustfmt::skip]
    let expected_2x2 = vec![
        10,18,33,  11,20,34,
        16,30,39,  18,33,40,
    ];
    assert_eq!(demosaic(&sensor, &[10, 20, 30, 40]), expected_2x2);

    // 3×5: odd width AND odd height — the phase at the far edge differs from the
    // near edge, so a wrong parity at the border would show here.
    let sensor = fixture_sensor(3, 5, Some(bayer(RGGB)));
    let mosaic: Vec<u16> = (0..15u16).map(|i| i * 7).collect();
    #[rustfmt::skip]
    let expected_3x5 = vec![
         0,0,14,    7,7,18,    14,14,21,
        21,21,25,  28,28,28,  35,35,32,
        42,42,46,  49,49,49,  56,56,53,
        63,63,67,  70,70,70,  77,77,74,
        84,84,77,  91,91,81,  98,98,84,
    ];
    assert_eq!(demosaic(&sensor, &mosaic), expected_3x5);
}

// ─────────────────────────────────────────────────────────────────────────────
// The `irr demosaic` surfacing — the first multi-channel irr output, a P6 PPM.
// Tier-A: a synthetic 16-bit CFA DNG, no corpus. Structural only (the exact
// values are the oracle tests' job), so it is kernel-agnostic.
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
