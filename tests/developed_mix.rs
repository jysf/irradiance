//! `SPEC-093` — the developed mix (`DEC-084`): `DevelopOptions::mono_mix` on
//! `develop_with_into`, `develop_with_scratch_into`, `output_dimensions_with`
//! and `scratch_len`.
//!
//! - **Tier A** (always runs): fixtures (i) to (v) of `tests/support/developed_mix.rs`.
//! - **Tier B** (`$IRRADIANCE_CORPUS_DIR`, skips per file by name): the five
//!   held colour files (AC4).
//!
//! The oracle (Design §2) is `mono_mix_plane_into`'s whole plane developed by
//! the mono develop, without the mix, on `mono_of(sensor)`: the mixed plane
//! read at black `MONO_MIX_BLACK` and white `MONO_MIX_WHITE`, no black pattern,
//! no `OpcodeList1` or `OpcodeList2`. It shares with the implementation only the
//! mixer's kernel, the warp and the mono gather, none of which this spec changes.

#[path = "support/ca_reference.rs"]
mod ca_reference;
#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/developed_mix.rs"]
mod developed_mix;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use corpus::{CorpusRoot, Manifest};
use developed_mix::*;
use irradiance::color::{mono_mix_plane_into, mono_mix_row, MONO_MIX_VERSION, MONO_MIX_WHITE};
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions_with,
    scratch_len, BlackLevels, DevelopOptions, Framing,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use irradiance::{Error, DEVELOP_VERSION};
use opcode_support::single_warp_rectilinear_list;

/// Every options set of AC1: Orientation 1 to 8 (set in the sensor) × both
/// framings × `apply_orientation`, at `warp_alpha` 1; then `warp_alpha` 0 and
/// ½ on Orientations 6 and 3.
fn ac1_grid() -> Vec<(u32, DevelopOptions)> {
    let mut grid = Vec::new();
    for orientation in 1..=8 {
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            for apply in [true, false] {
                grid.push((orientation, options(framing, apply, 1.0)));
            }
        }
    }
    for orientation in [6, 3] {
        for alpha in [0.0, 0.5] {
            for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
                grid.push((orientation, options(framing, true, alpha)));
            }
        }
    }
    grid
}

/// Assert the developed mix equals the oracle on `(sensor, src)` for every
/// AC1 options set and both weight sets. Returns how many cases ran.
fn assert_rule(name: &str, sensor: &Sensor, src: &[u16]) -> usize {
    let negative = negative_set(sensor);
    assert!(
        mono_mix_row(sensor, &negative)
            .unwrap()
            .iter()
            .any(|&q| q < 0),
        "{name}: the second weight set has a negative row entry"
    );
    let mut ran = 0;
    for weights in [LUMINANCE, negative] {
        let mixed = mixed_plane(sensor, src, &weights);
        for (orientation, o) in ac1_grid() {
            let mut s = sensor.clone();
            s.orientation = Some(orientation);
            let expected = oracle_of_mixed(&s, &mixed, &o);
            let actual = developed(&s, src, weights, &o);
            assert_same!(
                actual,
                expected,
                "{name}: orientation {orientation} {o:?} weights {weights:?}"
            );
            assert!(actual.is_ok(), "{name}: orientation {orientation} {o:?}");
            ran += 1;
        }
    }
    ran
}

#[test]
fn ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane() {
    let fixtures: [(&str, FixtureFn); 5] = [
        ("(i) odd", odd),
        ("(ii) odd, warped", odd_warped),
        ("(iii) tall", tall),
        ("(iv) tall, warped", tall_warped),
        ("(v) gain maps", odd_gained),
    ];
    for (name, fixture) in fixtures {
        let (sensor, src) = fixture();
        let ran = assert_rule(name, &sensor, &src);
        eprintln!("ac1 {name}: {ran} cases equal the oracle");
    }
    // The warp moves pixels: on (ii) the developed mix with and without it
    // differ, so (ii) and (iv) test the warp branch, not the no-warp one.
    let (warped, src) = odd_warped();
    let (plain, _) = odd();
    let o = DevelopOptions::default();
    assert_ne!(
        developed(&warped, &src, LUMINANCE, &o).unwrap(),
        developed(&plain, &src, LUMINANCE, &o).unwrap(),
        "fixture (ii)'s warp changes the developed mix"
    );
}

/// `SPEC-093` verify (HANDOFF-205), onyx's parity case (its DEC-073): the mix
/// composes with every other option at once. On the warped fixtures (ii) and
/// (iv), the full product of both framings, `apply_orientation` true and
/// false, and `warp_alpha` 0, ½ and 1 (AC1 crosses α 0 and ½ with
/// `apply_orientation` true only), each equals the oracle, and α ½ differs
/// from both ends, so the lerp is really exercised.
#[test]
fn ac1_parity_grid_composes_every_option() {
    for (name, fixture) in [("(ii)", odd_warped as FixtureFn), ("(iv)", tall_warped)] {
        let (sensor, src) = fixture();
        let mixed = mixed_plane(&sensor, &src, &LUMINANCE);
        for orientation in [1, 6, 7] {
            let mut s = sensor.clone();
            s.orientation = Some(orientation);
            for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
                for apply in [true, false] {
                    let mut outs = Vec::new();
                    for alpha in [0.0, 0.5, 1.0] {
                        let o = options(framing, apply, alpha);
                        let label = format!("{name} orientation {orientation} {o:?}");
                        let actual = developed(&s, &src, LUMINANCE, &o);
                        assert_same!(actual, oracle_of_mixed(&s, &mixed, &o), "{label}");
                        outs.push(actual.expect(&label));
                    }
                    assert_ne!(outs[1], outs[0], "{name}: α ½ is not α 0");
                    assert_ne!(outs[1], outs[2], "{name}: α ½ is not α 1");
                }
            }
        }
    }
}

#[test]
fn ac1_hand_derived_pixels_under_orientation_6() {
    // Fixture (i): ActiveArea left 1, top 3; DefaultCrop origin (1, 3), size
    // 17 × 11. Orientation 6 ("rotate 90 CW") makes the output 11 × 17, and
    // output (ox, oy) shows crop (oy, 10 − ox), i.e. plane (2 + oy, 16 − ox).
    let (mut sensor, src) = odd();
    sensor.orientation = Some(6);
    let mixed = mixed_plane(&sensor, &src, &LUMINANCE);
    let out = developed(&sensor, &src, LUMINANCE, &DevelopOptions::default()).unwrap();
    assert_eq!(out.len(), 11 * 17);
    let plane = |x: usize, y: usize| mixed[y * 22 + x];
    let output = |x: usize, y: usize| out[y * 11 + x];
    // Output (0, 0): crop (0, 10), plane (2, 16).
    assert_eq!(output(0, 0), plane(2, 16));
    // Output (10, 0): crop (0, 0), plane (2, 6).
    assert_eq!(output(10, 0), plane(2, 6));
    // Output (0, 16): crop (16, 10), plane (18, 16).
    assert_eq!(output(0, 16), plane(18, 16));
}

#[test]
fn ac2_the_option_absent_is_develop_into() {
    assert_eq!(DevelopOptions::default().mono_mix, None);
    for (name, fixture) in [("(i)", odd as FixtureFn), ("(ii)", odd_warped)] {
        let (sensor, src) = fixture();
        let o = DevelopOptions::default();
        let (w, h) = output_dimensions_with(&sensor, &o).unwrap();
        let mut a = vec![SENTINEL; (w * h) as usize];
        develop_into(&sensor, &src, &mut a).unwrap();
        let mut b = vec![0u16; (w * h) as usize];
        develop_with_into(&sensor, &src, &mut b, &o).unwrap();
        assert_eq!(a, b, "{name}: mono_mix None is develop_into");
        // And it is not the developed mix (the option is read).
        assert_ne!(
            a,
            developed(&sensor, &src, LUMINANCE, &o).unwrap(),
            "{name}: the mix changes the output"
        );
    }
}

/// A width-8 colour fixture whose ActiveArea is the whole plane.
fn narrow(height: u32) -> (Sensor, Vec<u16>) {
    let mut d = colour_description(8, height, [0, 0, height, 8], [0, 0], [8, height]);
    d.meta.active_area = None;
    read(&d)
}

#[test]
fn ac3_strip_count_and_the_cap_of_t() {
    let o = options(Framing::FullActiveArea, true, 1.0);
    for (height, expected_len) in [
        (512u32, 4_160usize),
        (513, 4_168),
        (519, 4_216),
        (520, 4_216),
    ] {
        let (sensor, src) = narrow(height);
        assert_same!(
            developed(&sensor, &src, LUMINANCE, &o),
            oracle(&sensor, &src, &LUMINANCE, &o),
            "height {height}"
        );
        assert_same!(
            scratch_len(&sensor, &with_mix(&o, LUMINANCE)),
            Ok::<usize, Error>(expected_len),
            "height {height}: scratch_len with the mix"
        );
    }
}

#[test]
fn ac5_black_levels_is_not_read_under_the_mix() {
    let (sensor, src) = odd();
    let mut first = DevelopOptions::default();
    first.black_levels = BlackLevels::FirstValue;
    let mut per = first;
    per.black_levels = BlackLevels::PerPosition;
    // The fixture can tell: developed as mono, the two differ.
    let mono = |o: &DevelopOptions| {
        let (w, h) = output_dimensions_with(&sensor, o).unwrap();
        let mut dst = vec![0u16; (w * h) as usize];
        develop_with_into(&sensor, &src, &mut dst, o).unwrap();
        dst
    };
    assert_ne!(
        mono(&first),
        mono(&per),
        "the black pattern matters as mono"
    );
    let a = developed(&sensor, &src, LUMINANCE, &first).unwrap();
    let b = developed(&sensor, &src, LUMINANCE, &per).unwrap();
    assert_eq!(a, b, "black_levels is not read under the mix");
    assert_same!(
        Ok::<Vec<u16>, Error>(a),
        oracle(&sensor, &src, &LUMINANCE, &first)
    );
}

/// `SPEC-098` (`DEC-086` item 1): the mix applies the first mandatory
/// `FixBadPixelsConstant` on the raw CFA (the kernel's values are
/// `tests/colour_bad_pixels.rs`'s) and refuses a second one.
#[test]
fn ac6_the_mix_applies_the_first_fix_bad_pixels() {
    let (plain, src) = odd();
    assert!(
        !src.contains(&0),
        "no photosite equals the Constant, so the fix is inert here"
    );
    let mut sensor = plain.clone();
    set_list(&mut sensor, 1, Some(mandatory_fix_bad_pixels()));
    let o = with_mix(&DevelopOptions::default(), LUMINANCE);
    assert_same!(
        developed(&sensor, &src, LUMINANCE, &o),
        developed(&plain, &src, LUMINANCE, &o)
    );
    assert!(developed(&sensor, &src, LUMINANCE, &o).is_ok());
    // The fix needs no scratch.
    assert_same!(scratch_len(&sensor, &o), scratch_len(&plain, &o));

    // A second mandatory one is refused, before any write, on both entries.
    set_list(&mut sensor, 1, Some(two_mandatory_fix_bad_pixels()));
    let (w, h) = output_dimensions_with(&sensor, &o).unwrap();
    let mut dst = vec![SENTINEL; (w * h) as usize];
    assert_same!(
        develop_with_into(&sensor, &src, &mut dst, &o),
        Err::<(), Error>(Error::UnsupportedRequiredOpcode { list: 1, id: 4 })
    );
    let mut scratch = vec![0u16; scratch_len(&sensor, &o).unwrap()];
    assert_same!(
        develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &o),
        Err::<(), Error>(Error::UnsupportedRequiredOpcode { list: 1, id: 4 })
    );
    assert!(dst.iter().all(|&v| v == SENTINEL));
}

#[test]
fn ac6_the_mix_applies_a_mandatory_warp() {
    let (optional, src) = odd_warped();
    let mut mandatory = optional.clone();
    set_list(&mut mandatory, 3, Some(radial_warp(0)));
    // The sensor-geometry plane passes the mandatory warp through unapplied
    // (`SPEC-098`, `DEC-086` item 5): the same plane as the optional copy's...
    let mut plane = vec![SENTINEL; src.len()];
    mono_mix_plane_into(&mandatory, &src, &LUMINANCE, &mut plane)
        .expect("the plane passes the warp through");
    let mut unwarped = vec![SENTINEL; src.len()];
    mono_mix_plane_into(&optional, &src, &LUMINANCE, &mut unwarped).unwrap();
    assert_eq!(plane, unwarped, "the sensor-geometry plane is not warped");
    // ...the developed mix applies it. The oracle's mixed plane comes from the
    // optional-flag copy (the flag does not touch the plane).
    for o in [
        DevelopOptions::default(),
        options(Framing::FullActiveArea, true, 0.5),
    ] {
        let mixed = mixed_plane(&optional, &src, &LUMINANCE);
        let expected = oracle_of_mixed(&mandatory, &mixed, &o);
        assert!(expected.is_ok());
        assert_same!(
            developed(&mandatory, &src, LUMINANCE, &o),
            expected,
            "{o:?}"
        );
    }
}

/// `SPEC-098` (`DEC-086` item 7): the mix now parses `OpcodeList1` with its
/// stage 2, so a malformed optional `FixBadPixelsConstant` is refused by the
/// develop as by the mono one; `scratch_len` still does not parse it under the
/// mix (the fix needs no scratch).
#[test]
fn ac6_the_mix_refuses_a_malformed_fix_bad_pixels_and_scratch_len_ignores_it() {
    let (mut sensor, src) = odd();
    set_list(&mut sensor, 1, Some(malformed_optional_fix_bad_pixels()));
    let mono = DevelopOptions::default();
    let mix = with_mix(&mono, LUMINANCE);
    // The mono develop parses it and refuses it.
    assert!(matches!(
        scratch_len(&sensor, &mono),
        Err(Error::MalformedOpcodeParams { id: 4, .. })
    ));
    let (w, h) = output_dimensions_with(&sensor, &mono).unwrap();
    let mut dst = vec![0u16; (w * h) as usize];
    assert!(matches!(
        develop_with_into(&sensor, &src, &mut dst, &mono),
        Err(Error::MalformedOpcodeParams { id: 4, .. })
    ));
    // The mix refuses it too, on both entries, before any write...
    let mut dst = vec![SENTINEL; (w * h) as usize];
    assert!(matches!(
        develop_with_into(&sensor, &src, &mut dst, &mix),
        Err(Error::MalformedOpcodeParams { id: 4, .. })
    ));
    let mut scratch = vec![0u16; scratch_len(&sensor, &mix).unwrap()];
    assert!(matches!(
        develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &mix),
        Err(Error::MalformedOpcodeParams { id: 4, .. })
    ));
    assert!(dst.iter().all(|&v| v == SENTINEL));
    // ...and its scratch length does not read it.
    assert_same!(scratch_len(&sensor, &mix), scratch_len(&odd().0, &mix));
}

/// `SPEC-098` verify: the developed mix's strips (`MIX_STRIP_ROWS`, 512 rows)
/// see the bad-pixel fix across a strip edge. Fixture (iii) is 1100 rows tall,
/// so its strips end near rows 512 and 1024 (framed rows from row 8, active
/// rows from row 3). A column of bad photosites crosses each edge, so every
/// candidate there reads neighbours one and two rows into the next strip. The
/// whole-plane `mono_mix_plane_into` inside the oracle has no strips; AC5's
/// 20-row scene fits in one strip and could not show this.
#[test]
fn ac5_the_mix_strips_see_the_fix_across_a_strip_edge() {
    for fixture in [tall as FixtureFn, tall_warped] {
        let (plain, mut src) = fixture();
        assert!(!src.contains(&0), "no photosite equals the Constant yet");
        let width = plain.width as usize;
        for y in (505..=530).chain(1015..=1040) {
            src[y * width + 6] = 0;
        }
        let mut sensor = plain.clone();
        set_list(&mut sensor, 1, Some(mandatory_fix_bad_pixels()));
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let o = options(framing, true, 1.0);
            let fixed = developed(&sensor, &src, LUMINANCE, &o);
            assert_same!(fixed, oracle(&sensor, &src, &LUMINANCE, &o));
            assert_ne!(
                fixed.unwrap(),
                developed(&plain, &src, LUMINANCE, &o).unwrap(),
                "{framing:?}: the fix changed nothing"
            );
        }
    }
}

/// `SPEC-100` (`DEC-088` item 3, replacing `DEC-086` item 6): the developed
/// mix applies the first `WarpRectilinear` whatever its flags, with `N` 1 or
/// 3. An optional `N = 3` warp with identical sets is applied as `W` (the one
/// set) and sized as a warp, so it equals the `N = 1` warp with that set on
/// both mix entries; a mandatory one too (no longer refused). Without the mix
/// (the mono develop) an `N = 3` warp is applied as `K1`, as before.
#[test]
fn ac6_the_mix_sizes_an_optional_multi_plane_warp_as_it_develops() {
    let (plain, src) = odd();
    let warped = |n: u32, flags: u32| {
        let mut s = plain.clone();
        let list = single_warp_rectilinear_list(RADIAL, [0.0, 0.0], 0.5, 0.5, n, flags);
        set_list(&mut s, 3, Some(list));
        s
    };
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        let mono = options(framing, true, 1.0);
        let mix = with_mix(&mono, LUMINANCE);

        // The mix: N = 1 is applied, and needs more scratch than no warp.
        let n1 = warped(1, 1);
        assert!(scratch_len(&n1, &mix).unwrap() > scratch_len(&plain, &mix).unwrap());
        let applied = developed(&n1, &src, LUMINANCE, &mono).unwrap();
        assert_ne!(
            applied,
            developed(&plain, &src, LUMINANCE, &mono).unwrap(),
            "{framing:?}: the N = 1 warp is applied by the mix"
        );
        // An identical-set N = 3 warp, optional or mandatory: applied as W and
        // sized as a warp.
        for flags in [1, 0] {
            let n3 = warped(3, flags);
            assert_same!(scratch_len(&n3, &mix), scratch_len(&n1, &mix));
            assert_same!(
                developed(&n3, &src, LUMINANCE, &mono),
                Ok::<&Vec<u16>, Error>(&applied),
                "{framing:?}: an N = 3 warp (flags {flags}) is applied as W"
            );
        }

        // The mono develop: N = 3 sized as N = 1, applied, scratch form equal.
        let n3 = warped(3, 1);
        assert_same!(scratch_len(&n3, &mono), scratch_len(&n1, &mono));
        let (w, h) = output_dimensions_with(&n3, &mono).unwrap();
        let mut allocating = vec![SENTINEL; (w * h) as usize];
        develop_with_into(&n3, &src, &mut allocating, &mono).expect("mono develops");
        let mut scratch = vec![0u16; scratch_len(&n3, &mono).unwrap()];
        let mut with_scratch = vec![SENTINEL; allocating.len()];
        develop_with_scratch_into(&n3, &src, &mut with_scratch, &mut scratch, &mono)
            .expect("mono scratch develops");
        assert_eq!(with_scratch, allocating, "{framing:?}: mono scratch form");
        let mut unwarped = vec![SENTINEL; allocating.len()];
        develop_with_into(&plain, &src, &mut unwarped, &mono).expect("mono develops");
        assert_ne!(
            allocating, unwarped,
            "{framing:?}: the mono develop warps N = 3"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-100 (DEC-088 items 3 to 5): the per-plane warp on the developed mix
// ─────────────────────────────────────────────────────────────────────────────

/// `DEC-087` item 3's halo, from the decision's text with the tests' own
/// table: `H = ⌈B⌉ + 1` capped at the `ActiveArea` height, `B = m · max_j
/// max(|t[j] − 1|, |t[j+1] − 1|) · √((j+1)/T)` over the resampled planes.
fn halo_of(planes: &[[f64; 4]], cx: f64, cy: f64, aw: u32, ah: u32) -> u64 {
    let (x1, y1) = (f64::from(aw - 1), f64::from(ah - 1));
    let (cx, cy) = (cx * x1, cy * y1);
    let m = cx
        .abs()
        .max((x1 - cx).abs())
        .hypot(cy.abs().max((y1 - cy).abs()));
    let t_len = ca_reference::T as f64;
    let mut b = 0.0f64;
    for &set in planes {
        let t = ca_reference::table(set, CA_GREEN);
        for j in 0..ca_reference::T {
            let reach = (t[j] - 1.0).abs().max((t[j + 1] - 1.0).abs());
            b = b.max(m * reach * ((j as f64 + 1.0) / t_len).sqrt());
        }
    }
    (b.ceil() as u64 + 1).min(u64::from(ah))
}

/// `T_H` and `R` (`DEC-088` item 4) for a `w × h` plane and halo `hh`.
fn strip_and_ring(w: u32, h: u32, hh: u64) -> (u64, u64) {
    let t = u64::from(w) * (u64::from(h).min(STRIP_ROWS + 2 * hh + 7) + RING_ROWS);
    (t, 3 * (2 * hh + 2) * u64::from(w))
}

/// AC4: on the three scene variants with the differing `N = 3` list, mandatory
/// and optional, the developed mix is the mono develop of
/// `mono_mix_plane_into`'s CA-corrected plane with `W` as one `N = 1` set,
/// over SPEC-093's grid and both weight sets, on both develop entries.
#[test]
fn ac4_the_mix_is_the_mono_develop_of_the_ca_plane() {
    let mut ran = 0;
    for (name, sensor, src, cx, cy) in ca_scenes() {
        for flags in [0u32, 1] {
            let mut s0 = sensor.clone();
            set_list(&mut s0, 3, Some(ca_list(cx, cy, flags)));
            let negative = negative_set(&s0);
            for weights in [LUMINANCE, negative] {
                let mixed = mixed_plane(&s0, &src, &weights);
                // The CA step moves the plane, so the oracle's input is not
                // the uncorrected one.
                let mut no_ca = s0.clone();
                set_list(&mut no_ca, 3, Some(w_list(cx, cy)));
                assert_ne!(
                    mixed,
                    mixed_plane(&no_ca, &src, &weights),
                    "{name}: the CA step changes the mixed plane"
                );
                for (orientation, o) in ac1_grid() {
                    let mut s = s0.clone();
                    s.orientation = Some(orientation);
                    let expected = oracle_with_list3(&s, &mixed, &o, Some(w_list(cx, cy)));
                    assert!(
                        expected.is_ok(),
                        "{name}: the oracle develops: {expected:?}"
                    );
                    assert_same!(
                        developed(&s, &src, weights, &o),
                        expected,
                        "{name} flags {flags} orientation {orientation} {o:?}"
                    );
                    ran += 1;
                }
                // W is green's set, not red's (K1, what the mono develop of the
                // file would apply).
                let o = DevelopOptions::default();
                let k1 = single_warp_rectilinear_list(CA_RED, [0.0, 0.0], cx, cy, 1, 0);
                assert_ne!(
                    developed(&s0, &src, weights, &o).unwrap(),
                    oracle_with_list3(&s0, &mixed, &o, Some(k1)).unwrap(),
                    "{name}: red's set as the warp"
                );
            }
        }
    }
    eprintln!("ac4: {ran} cases equal the oracle");
}

/// AC4's strips: an 8 × 1100 plane (taller than two strips) with the CA step,
/// whose halo reaches across the strip edges at rows 512 and 1024; every
/// option equals the oracle.
#[test]
fn ac4_ca_strips_tile_across_a_strip_edge() {
    let (mut sensor, src) = narrow(1100);
    set_list(&mut sensor, 3, Some(ca_list(0.5, 0.5, 0)));
    let halo = halo_of(&[CA_RED, CA_BLUE], 0.5, 0.5, 8, 1100);
    eprintln!("ac4 strips: H = {halo}");
    assert!(halo > 8, "the halo reaches well across a strip edge");
    let mixed = mixed_plane(&sensor, &src, &LUMINANCE);
    for o in [
        options(Framing::StandardCrop, true, 1.0),
        options(Framing::StandardCrop, true, 0.0),
        options(Framing::FullActiveArea, true, 0.5),
        options(Framing::FullActiveArea, false, 0.0),
    ] {
        assert_same!(
            developed(&sensor, &src, LUMINANCE, &o),
            oracle_with_list3(&sensor, &mixed, &o, Some(w_list(0.5, 0.5))),
            "{o:?}"
        );
    }
}

/// AC5: `scratch_len` under the mix is `DEC-088` item 4's formula: no CA step
/// (`SPEC-093`'s numbers), the CA step without an applied warp (`T_H + R`),
/// with one (`A + max(A, T_H + R)`), on a scene where `T_H + R > A` and one
/// where `A` is larger; one sample more or less is refused.
#[test]
fn ac5_scratch_len_is_the_formula() {
    let mix = |alpha| with_mix(&options(Framing::StandardCrop, true, alpha), LUMINANCE);
    // No CA step: today's numbers.
    let (plain, _) = odd_warped();
    assert_same!(
        scratch_len(&plain, &mix(1.0)),
        Ok::<usize, Error>(warp_scratch(22, 18, 280) as usize)
    );
    assert_same!(
        scratch_len(&plain, &mix(0.0)),
        Ok::<usize, Error>(strip_t(22, 18) as usize)
    );
    // (i) with the CA step: A = 280 is below T_H + R.
    let (mut small, src) = odd();
    set_list(&mut small, 3, Some(ca_list(0.5, 0.5, 0)));
    let hh = halo_of(&[CA_RED, CA_BLUE], 0.5, 0.5, 20, 14);
    let (t, r) = strip_and_ring(22, 18, hh);
    assert!(t + r > 280, "the small scene: T_H + R > A");
    assert_same!(
        scratch_len(&small, &mix(0.0)),
        Ok::<usize, Error>((t + r) as usize),
        "CA, no warp"
    );
    assert_same!(
        scratch_len(&small, &mix(1.0)),
        Ok::<usize, Error>((280 + t + r) as usize),
        "CA and a warp, T_H + R > A"
    );
    // The tall narrow plane: A = 8800 is above T_H + R.
    let (mut tall, _) = narrow(1100);
    set_list(&mut tall, 3, Some(ca_list(0.5, 0.5, 0)));
    let hh = halo_of(&[CA_RED, CA_BLUE], 0.5, 0.5, 8, 1100);
    let (t, r) = strip_and_ring(8, 1100, hh);
    assert!(t + r < 8800, "the tall scene: A > T_H + R");
    assert_same!(
        scratch_len(&tall, &mix(1.0)),
        Ok::<usize, Error>(2 * 8800),
        "CA and a warp, A > T_H + R"
    );
    assert_same!(
        scratch_len(&tall, &mix(0.0)),
        Ok::<usize, Error>((t + r) as usize),
        "tall, CA, no warp"
    );
    // One sample more or less is refused, before any write.
    for alpha in [0.0, 1.0] {
        let o = mix(alpha);
        let len = scratch_len(&small, &o).unwrap();
        for wrong in [len - 1, len + 1] {
            let mut dst = vec![SENTINEL; output_len(&small, &o)];
            let mut scratch = vec![SENTINEL; wrong];
            let r = develop_with_scratch_into(&small, &src, &mut dst, &mut scratch, &o);
            assert!(
                matches!(r, Err(Error::DevelopScratchWrongLength { .. })),
                "{wrong}: {r:?}"
            );
            assert!(dst.iter().all(|&v| v == SENTINEL));
            assert!(scratch.iter().all(|&v| v == SENTINEL));
        }
    }
}

/// Both develop entries' results on `sensor`, from sentinel buffers sized by
/// `scratch_len` (or the plain fixture's when it refuses), with whether `dst`
/// and the scratch stayed untouched.
fn mix_results(sensor: &Sensor, src: &[u16], o: &DevelopOptions) -> [(Result<(), Error>, bool); 2] {
    let (plain, _) = odd();
    let valid = with_mix(&without_mix(o), LUMINANCE);
    let dims = output_dimensions_with(sensor, o)
        .or_else(|_| output_dimensions_with(&plain, &valid))
        .unwrap();
    let n = dims.0 as usize * dims.1 as usize;
    let len = scratch_len(sensor, o)
        .or_else(|_| scratch_len(&plain, &valid))
        .unwrap();
    let mut dst = vec![SENTINEL; n];
    let a = develop_with_into(sensor, src, &mut dst, o);
    let a_untouched = dst.iter().all(|&v| v == SENTINEL);
    let mut dst = vec![SENTINEL; n];
    let mut scratch = vec![SENTINEL; len];
    let b = develop_with_scratch_into(sensor, src, &mut dst, &mut scratch, o);
    let b_untouched = dst.iter().chain(&scratch).all(|&v| v == SENTINEL);
    [(a, a_untouched), (b, b_untouched)]
}

#[track_caller]
fn assert_mix_refused(
    sensor: &Sensor,
    src: &[u16],
    o: &DevelopOptions,
    case: &str,
    pred: fn(&Error) -> bool,
) {
    for (i, (r, untouched)) in mix_results(sensor, src, o).into_iter().enumerate() {
        match r {
            Err(e) if pred(&e) => assert!(untouched, "{case}: entry {i} wrote before {e:?}"),
            other => panic!("{case}: entry {i}: unexpected {other:?}"),
        }
    }
}

/// AC6 (the mix): `N = 2` is malformed, optional or not; differing sets with a
/// tangential term are refused after the CFA and before the calibration;
/// `InvalidMixWeights` stays first; under `FullActiveArea` the bounds rule
/// reads `W`, not `K1`; `scratch_len` reads the per-plane parse, the plane
/// colours and the fold, and sizes a sensor with no CFA with `H = 0`.
#[test]
fn ac6_the_mix_per_plane_refusals() {
    let (base, src) = odd();
    let z = [0.0, 0.0];
    let with = |list: Vec<u8>| {
        let mut s = base.clone();
        set_list(&mut s, 3, Some(list));
        s
    };
    let o = with_mix(&DevelopOptions::default(), LUMINANCE);
    let malformed: fn(&Error) -> bool = |e| matches!(e, Error::MalformedOpcodeParams { id: 1, .. });
    for flags in [0, 1] {
        let s = with(sets_list(&[(CA_RED, z), (CA_GREEN, z)], 0.5, 0.5, flags));
        assert_mix_refused(&s, &src, &o, &format!("N = 2, flags {flags}"), malformed);
        assert!(
            malformed(&scratch_len(&s, &o).unwrap_err()),
            "scratch_len N = 2"
        );
    }

    // Differing sets with blue's kt: after the CFA, before the calibration.
    let kt: fn(&Error) -> bool = |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if *kt0 == 0.002 && *kt1 == 0.0);
    let kt_list = sets_list(
        &[(CA_RED, z), (CA_GREEN, z), (CA_BLUE, [0.002, 0.0])],
        0.5,
        0.5,
        0,
    );
    let s = with(kt_list.clone());
    assert_mix_refused(&s, &src, &o, "blue kt", kt);
    let mut cfa = s.clone();
    cfa.color_filter.as_mut().unwrap().pattern = vec![0, 0, 1, 2];
    assert_mix_refused(&cfa, &src, &o, "CFA first", |e| {
        matches!(e, Error::UnsupportedCfaPattern { .. })
    });
    let mut uncalibrated = s.clone();
    uncalibrated.color_matrix_1 = None;
    uncalibrated.forward_matrix_1 = None;
    assert_mix_refused(&uncalibrated, &src, &o, "before the calibration", kt);
    let mut weights = o;
    weights.mono_mix = Some([r(1, 3), r(1, 3), r(1, 4)]);
    assert_mix_refused(&s, &src, &weights, "weights first", |e| {
        matches!(e, Error::InvalidMixWeights { .. })
    });
    // scratch_len reads no tangential term.
    assert_same!(
        scratch_len(&s, &o),
        scratch_len(&with(ca_list(0.5, 0.5, 0)), &o),
        "scratch_len sizes the kt list"
    );
    // A folding green with differing sets.
    let fold = [1.0, -2.0, 0.0, 0.0];
    let s = with(sets_list(
        &[(CA_RED, z), (fold, z), (CA_BLUE, z)],
        0.5,
        0.5,
        0,
    ));
    let folds: fn(&Error) -> bool = |e| matches!(e, Error::UnsupportedWarpFold { .. });
    assert_mix_refused(&s, &src, &o, "fold", folds);
    assert!(
        folds(&scratch_len(&s, &o).unwrap_err()),
        "scratch_len reads the fold"
    );
    // CFAPlaneColor [0, 1, 1] with N = 3.
    let mut s = with(ca_list(0.5, 0.5, 0));
    s.color_filter.as_mut().unwrap().plane_color = vec![0, 1, 1];
    assert!(
        matches!(
            scratch_len(&s, &o),
            Err(Error::UnsupportedCfaPlaneColor { .. })
        ),
        "scratch_len reads CFAPlaneColor"
    );
    // A sensor with no CFA sizes N = 3 with H = 0 (W is K1, red's set), and
    // the develop refuses it at the CFA.
    let mut s = with(ca_list(0.5, 0.5, 0));
    s.color_filter = None;
    assert_same!(
        scratch_len(&s, &o),
        Ok::<usize, Error>(warp_scratch(22, 18, 280) as usize),
        "no CFA: H = 0"
    );
    assert_mix_refused(&s, &src, &o, "no CFA", |e| {
        matches!(e, Error::NotAColorFilterArray)
    });

    // FullActiveArea: the bounds rule reads W (green), not K1 (red).
    let full = with_mix(&options(Framing::FullActiveArea, true, 1.0), LUMINANCE);
    let out = [1.0, 0.08, 0.0, 0.0];
    let s = with(sets_list(
        &[(CA_RED, z), (out, z), (CA_BLUE, z)],
        0.5,
        0.5,
        0,
    ));
    assert_mix_refused(&s, &src, &full, "W fails the bounds rule", |e| {
        matches!(e, Error::UnsupportedWarpedFullActiveArea { .. })
    });
    let s = with(sets_list(
        &[(out, z), (CA_GREEN, z), (CA_BLUE, z)],
        0.5,
        0.5,
        0,
    ));
    for (r, _) in mix_results(&s, &src, &full) {
        r.expect("red fails the bounds rule, W passes: develops");
    }
}

fn output_len(sensor: &Sensor, options: &DevelopOptions) -> usize {
    let (w, h) = output_dimensions_with(sensor, options).unwrap();
    (w * h) as usize
}

#[test]
fn ac6_gain_maps_apply_once() {
    let (sensor, src) = odd_gained();
    let (plain, _) = odd();
    let o = DevelopOptions::default();
    // The maps change the mix (so applying them zero times would show)...
    assert_ne!(
        developed(&sensor, &src, LUMINANCE, &o).unwrap(),
        developed(&plain, &src, LUMINANCE, &o).unwrap()
    );
    // ...and equal the oracle, whose mono_of drops OpcodeList2 (so applying
    // them twice would show).
    assert_same!(
        developed(&sensor, &src, LUMINANCE, &o),
        oracle(&sensor, &src, &LUMINANCE, &o)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8: the ranked refusals (FU-6)
// ─────────────────────────────────────────────────────────────────────────────

/// F13 (scratch) applies to `develop_with_scratch_into` only.
fn entry_faults(with_scratch: bool) -> Vec<usize> {
    (0..FAULTS).filter(|&f| with_scratch || f != 13).collect()
}

fn assert_refused(label: &str, set: &[usize], expected: usize, with_scratch: bool) {
    let c = case(set);
    let (result, dst_untouched, scratch_untouched) = run_case(&c, with_scratch);
    let err = result.expect_err(label);
    assert!(
        expected_error(expected)(&err),
        "{label}: expected F{expected}'s error, got {err:?}"
    );
    assert!(dst_untouched, "{label}: dst was written ({err:?})");
    if expected != 14 {
        assert!(scratch_untouched, "{label}: scratch was written ({err:?})");
    }
}

#[test]
fn ac8_each_fault_alone() {
    // The valid fixture develops, so each fault is the only fault.
    for with_scratch in [false, true] {
        let (result, _, _) = run_case(&case(&[]), with_scratch);
        assert_same!(result, Ok::<(), Error>(()), "the valid fixture develops");
        for f in entry_faults(with_scratch) {
            let label = format!("F{f} alone, scratch form {with_scratch}");
            assert_refused(&label, &[f], f, with_scratch);
        }
    }
}

#[test]
fn ac8_every_ranked_pair_on_both_develop_entries() {
    let mut skipped = Vec::new();
    for with_scratch in [false, true] {
        let faults = entry_faults(with_scratch);
        let mut pairs = 0;
        for (a, &i) in faults.iter().enumerate() {
            for &j in &faults[a + 1..] {
                if let Some(why) = skipped_pair(i, j) {
                    if with_scratch {
                        skipped.push(format!("(F{i}, F{j}): {why}"));
                    }
                    continue;
                }
                let label = format!("(F{i}, F{j}), scratch form {with_scratch}");
                assert_refused(&label, &[i, j], i, with_scratch);
                pairs += 1;
            }
        }
        eprintln!(
            "ac8: {pairs} pairs on {}",
            if with_scratch {
                "develop_with_scratch_into"
            } else {
                "develop_with_into"
            }
        );
        assert_eq!(pairs, if with_scratch { 102 } else { 88 });
    }
    // F11 with F12 and F7 with F14 run with one value carrying both faults.
    eprintln!("ac8 skipped: {skipped:?}");
}

/// `SPEC-093` verify (HANDOFF-205): the pair (F8, F14; F13 before SPEC-100), which the build
/// skipped as impossible. One `WarpRectilinear` block can carry both faults:
/// it is malformed (a NaN optical centre, or one trailing byte past its exact
/// `4 + 48·N + 16` length) and its tangential term is non-zero. Under
/// `StandardCrop` the parse (rank 8) must refuse it before the applier
/// (rank 14) could see the term, on both develop entries and `scratch_len`,
/// with `dst` and the scratch untouched.
#[test]
fn ac8_one_warp_block_both_malformed_and_tangential_is_f8() {
    let nan_centre =
        opcode_support::single_warp_rectilinear_list(RADIAL, [0.01, 0.0], f64::NAN, 0.5, 1, 1);
    let mut params = 1u32.to_be_bytes().to_vec();
    for v in RADIAL.into_iter().chain([0.01, 0.0, 0.5, 0.5]) {
        params.extend_from_slice(&v.to_be_bytes());
    }
    params.push(0);
    let trailing =
        opcode_support::opcode_list(&[opcode_support::raw_opcode(1, 0x0104_0000, 1, &params)]);
    for (name, list) in [("NaN centre", nan_centre), ("trailing byte", trailing)] {
        let mut c = case(&[]);
        set_list(&mut c.sensor, 3, Some(list));
        for with_scratch in [false, true] {
            let label = format!("(F8, F14) as one block, {name}, scratch form {with_scratch}");
            let (result, dst_untouched, scratch_untouched) = run_case(&c, with_scratch);
            let err = result.expect_err(&label);
            assert!(expected_error(8)(&err), "{label}: got {err:?}");
            assert!(dst_untouched, "{label}: dst was written");
            assert!(scratch_untouched, "{label}: scratch was written");
        }
        let err = scratch_len(&c.sensor, &c.options).unwrap_err();
        assert!(expected_error(8)(&err), "scratch_len, {name}: got {err:?}");
    }
}

#[test]
fn ac8_the_sizing_entries_check_only_what_they_need() {
    let base = case(&[]);
    let base_dims = output_dimensions_with(&base.sensor, &base.options).unwrap();
    let (w, h) = (base.sensor.width, base.sensor.height);
    let active = 20u64 * 14;
    for f in 0..FAULTS {
        let c = case(&[f]);
        let dims = output_dimensions_with(&c.sensor, &c.options);
        let len = scratch_len(&c.sensor, &c.options);
        match f {
            0 | 1 | 4 => {
                assert!(expected_error(f)(&dims.unwrap_err()), "dims F{f}");
                assert!(expected_error(f)(&len.unwrap_err()), "scratch_len F{f}");
            }
            8 => {
                assert_same!(dims, Ok::<(u32, u32), Error>(base_dims), "dims F8");
                assert!(expected_error(8)(&len.unwrap_err()), "scratch_len F8");
            }
            _ => {
                // F7 under FullActiveArea is the active area's size.
                let want = if f == 7 { (20, 14) } else { base_dims };
                assert_same!(dims, Ok::<(u32, u32), Error>(want), "dims F{f}");
                let want_len = if f == 7 || f == 14 {
                    warp_scratch(w, h, active)
                } else if f == 10 {
                    // `SPEC-100`: W and the CA halo (`DEC-088` item 4).
                    let hh = halo_of(&[CA_RED, CA_BLUE], 0.5, 0.5, 20, 14);
                    let (t, r) = strip_and_ring(w, h, hh);
                    active + active.max(t + r)
                } else {
                    strip_t(w, h)
                };
                assert_same!(
                    len,
                    Ok::<usize, Error>(want_len as usize),
                    "scratch_len F{f}"
                );
            }
        }
    }
    // An optional malformed FixBadPixelsConstant sizes too.
    let mut c = case(&[]);
    set_list(&mut c.sensor, 1, Some(malformed_optional_fix_bad_pixels()));
    assert_same!(
        scratch_len(&c.sensor, &c.options),
        Ok::<usize, Error>(strip_t(w, h) as usize)
    );
    // Pairs among F0, F1, F4 (and F8 for scratch_len): the lower rank's.
    for (i, j) in [(0, 1), (0, 4), (1, 4), (0, 8), (1, 8), (4, 8)] {
        let c = case(&[i, j]);
        if j != 8 {
            let e = output_dimensions_with(&c.sensor, &c.options).unwrap_err();
            assert!(expected_error(i)(&e), "dims (F{i}, F{j}): {e:?}");
        }
        let e = scratch_len(&c.sensor, &c.options).unwrap_err();
        assert!(expected_error(i)(&e), "scratch_len (F{i}, F{j}): {e:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC9: the contract constants (FU-5)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn developed_mix_contract_constants_are_the_computation() {
    // (a) The field's default is None, and the default develop is develop_into.
    assert_eq!(DevelopOptions::default().mono_mix, None);
    let (sensor, src) = odd();
    let (w, h) = output_dimensions_with(&sensor, &DevelopOptions::default()).unwrap();
    let mut a = vec![0u16; (w * h) as usize];
    let mut b = vec![1u16; (w * h) as usize];
    develop_into(&sensor, &src, &mut a).unwrap();
    develop_with_into(&sensor, &src, &mut b, &DevelopOptions::default()).unwrap();
    assert_eq!(a, b);

    // (b) MONO_MIX_BLACK and MONO_MIX_WHITE: mono_of reads its levels from the
    // two constants, and the developed mix equals that oracle where the output
    // has a pixel at white and one in (32768, 65535), so a changed constant
    // alone moves the oracle.
    let mono = mono_of(&sensor);
    assert_eq!(mono.white_level, Some(u32::from(MONO_MIX_WHITE)));
    let out = developed(&sensor, &src, LUMINANCE, &DevelopOptions::default()).unwrap();
    assert!(out.contains(&65535), "a developed pixel at white");
    assert!(
        out.iter().any(|&v| v > 32768 && v < 65535),
        "a developed pixel in (32768, 65535)"
    );
    assert_same!(
        Ok::<Vec<u16>, Error>(out),
        oracle(&sensor, &src, &LUMINANCE, &DevelopOptions::default())
    );

    // (c) The versions the pinned values below and in tests/determinism.rs
    // were recorded at. A new value of either needs a version bump.
    assert_eq!(MONO_MIX_VERSION, "1");
    assert_eq!(DEVELOP_VERSION, "8");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4: the held files (Tier B)
// ─────────────────────────────────────────────────────────────────────────────

fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn digest(s: &[u16]) -> u64 {
    fnv1a(s.iter().flat_map(|v| v.to_le_bytes()))
}

/// What AC4 expects of one options set: a digest, or the Pixel 3a's refusal.
#[derive(Clone, Copy)]
enum Outcome {
    Digest(u64),
    Refused,
}

struct Held {
    file: &'static str,
    /// default, `warp_alpha` 0, `FullActiveArea`, `apply_orientation` false.
    outcomes: [Outcome; 4],
    /// `scratch_len` with the mix: default options, then `warp_alpha` 0.
    scratch: [usize; 2],
}

use Outcome::{Digest, Refused};

/// Pre-registered (SPEC-093 AC4, the design probe's oracle, 2026-11-26).
/// A mismatch is reported, never edited.
const HELD: [Held; 5] = [
    Held {
        file: "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
        outcomes: [
            Digest(0x7fa38a4df1ef70fa),
            Digest(0x7fa38a4df1ef70fa),
            Digest(0xc2f1f457c0b0ff19),
            Digest(0x7fa38a4df1ef70fa),
        ],
        scratch: [3_119_840, 3_119_840],
    },
    Held {
        file: "GOOGLE-PIXEL-3A/IMG_20190918_164153.dng",
        outcomes: [
            Digest(0x4139c125eb0480cc),
            Digest(0x8d0f238b1140d37c),
            Refused,
            Digest(0x4139c125eb0480cc),
        ],
        scratch: [24_385_536, 2_124_864],
    },
    Held {
        file: "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng",
        outcomes: [
            Digest(0x3a0b22d387e1fa1b),
            Digest(0x3a0b22d387e1fa1b),
            Digest(0x46fa545dcdd1fdcc),
            Digest(0x3a0b22d387e1fa1b),
        ],
        scratch: [2_124_864, 2_124_864],
    },
    Held {
        file: "GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng",
        outcomes: [
            Digest(0xaa092016652f3fa7),
            Digest(0x8071d9cb873f04b4),
            Digest(0x6e7c387db9c1b748),
            Digest(0xa41a139d6b54ae07),
        ],
        scratch: [22_701_936, 2_045_814],
    },
    Held {
        file: "NIKON-D300S/_DSC4615.DNG",
        outcomes: [
            Digest(0xc865b3ed76b53366),
            Digest(0xc865b3ed76b53366),
            Digest(0xc258fdc0f7af9434),
            Digest(0xc865b3ed76b53366),
        ],
        scratch: [2_293_504, 2_293_504],
    },
];

#[test]
fn ac4_held_files_match_the_oracle_and_the_digests() {
    // The digests below were measured at these versions; a new value needs a
    // bump of one of them.
    assert_eq!(DEVELOP_VERSION, "8");
    assert_eq!(MONO_MIX_VERSION, "1");
    let manifest = Manifest::load().expect("manifest");
    let root = CorpusRoot::resolve();
    let sets = [
        options(Framing::StandardCrop, true, 1.0),
        options(Framing::StandardCrop, true, 0.0),
        options(Framing::FullActiveArea, true, 1.0),
        options(Framing::StandardCrop, false, 1.0),
    ];
    let mut mismatches = Vec::new();
    for held in HELD {
        let Some(path) = manifest.get(held.file).and_then(|e| e.require(&root)) else {
            continue;
        };
        let bytes = std::fs::read(&path).expect("read");
        let container = Container::parse(&bytes).expect("parse");
        let sensor = container.sensor().expect("sensor");
        let mut src = vec![0u16; (sensor.width * sensor.height) as usize];
        unpack_into(&sensor, container.byte_order(), &bytes, &mut src).expect("unpack");
        let mixed = mixed_plane(&sensor, &src, &LUMINANCE);
        for (k, (o, outcome)) in sets.iter().zip(held.outcomes).enumerate() {
            let mix = with_mix(o, LUMINANCE);
            let expected = oracle_of_mixed(&sensor, &mixed, o);
            let (w, h) = output_dimensions_with(&sensor, &mix).expect("dims");
            let mut dst = vec![SENTINEL; w as usize * h as usize];
            let actual = develop_with_into(&sensor, &src, &mut dst, &mix);
            match outcome {
                Refused => {
                    match &actual {
                        Err(Error::UnsupportedWarpedFullActiveArea { scale_max, .. }) => {
                            assert_eq!(*scale_max, 1.0015292540192604, "{}", held.file);
                        }
                        other => panic!("{}: expected the refusal, got {other:?}", held.file),
                    }
                    assert_same!(
                        actual,
                        expected.map(|_| ()),
                        "{}: the mono develop refuses too",
                        held.file
                    );
                    assert!(dst.iter().all(|&v| v == SENTINEL));
                }
                Digest(want) => {
                    assert_same!(actual, Ok::<(), Error>(()), "{} set {k}", held.file);
                    assert_same!(
                        Ok::<&Vec<u16>, &Error>(&dst),
                        expected.as_ref(),
                        "{} set {k}: oracle",
                        held.file
                    );
                    let got = digest(&dst);
                    eprintln!("ac4 {} set {k}: {w}x{h} digest {got:#018x}", held.file);
                    if got != want {
                        mismatches.push(format!(
                            "{} set {k}: {got:#018x}, registered {want:#018x}",
                            held.file
                        ));
                    }
                }
            }
            // scratch_len: §7's numbers, by warp_alpha (the framing and the
            // orientation do not change it).
            let want_len = held.scratch[usize::from(o.warp_alpha == 0.0)];
            assert_same!(
                scratch_len(&sensor, &mix),
                Ok::<usize, Error>(want_len),
                "{} set {k}",
                held.file
            );
        }
        // Pixel 3a and 7 Pro: the mono develop's own warp scratch for mono_of.
        if held.file.starts_with("GOOGLE-PIXEL-3A") || held.file.starts_with("GOOGLE-PIXEL-7") {
            let o = DevelopOptions::default();
            assert_same!(
                scratch_len(&sensor, &with_mix(&o, LUMINANCE)),
                scratch_len(&mono_of(&sensor), &o),
                "{}",
                held.file
            );
        }
    }
    assert!(mismatches.is_empty(), "AC4 digests: {mismatches:#?}");
}
