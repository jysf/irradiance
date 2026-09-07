//! `SPEC-018` — `WarpRectilinear` application (`AC4`, `AC6`, `AC8`-`AC10`,
//! `AC12`). Byte-level opcode PARSING is `tests/opcode.rs` — see that file's
//! header for the split.
//!
//! `AC5`/`AC12`/(part of `AC7`) are unit tests inside `src/warp.rs` itself
//! (`identity_warp_is_the_identity_transform`,
//! `warp_output_is_bit_identical_across_two_runs`,
//! `outside_pixels_follow_the_dng_spec_rule`) — `cargo test <name>` finds
//! them there; AGENTS.md §12's "unit test in the module it tests" applies
//! the same way `tests/develop.rs`'s header already explains for
//! `crop_origin_is_relative_to_active_area`.
//!
//! Two lanes:
//!
//! - **Tier A** (`warp_moves_corner_impulse_to_spike_001s_source_coord`,
//!   `output_dimensions_are_unchanged_by_warp`,
//!   `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more`) run
//!   everywhere: a synthetic corner-impulse fixture at the Q2M crop size
//!   (8368x5584), the same size AS the real frames but built in-test, no
//!   corpus needed.
//! - **Tier B** (`warp_scores_at_least_eightyfive_via_spec_020_oracle`,
//!   `warp_oracle_is_red_on_a_zeroed_kr1_coefficient`) need the real corpus
//!   and `dnglab` on `PATH`, and skip loudly, per-entry, when absent.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/ssimulacra2.rs"]
mod metric;
#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/pnm.rs"]
mod pnm;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{
    ActiveArea, Compression, Container, DefaultCropOrigin, DefaultCropSize, Sensor,
};
use irradiance::opcode::{parse_warp_rectilinear, WarpRect};
use irradiance::plane::unpack_into;
use irradiance::warp::apply_warp_into;

/// Q2M's own measured crop size (`SPEC-014`'s `## Implementation Context`,
/// `docs/measured-q2m-dng.md`) — the size every AC4/AC10 fixture uses so the
/// corner displacement is directly comparable to `SPIKE-001`'s figure.
const WIDTH: u32 = 8368;
const HEIGHT: u32 = 5584;

/// SPEC-020/FU-1's per-frame `kr0..kr3` table (see `tests/opcode.rs`'s own
/// copy for provenance — measured independently by this build via `exiftool
/// -b -OpcodeList3` on all three decodable frames).
const FRAME_COEFFICIENTS: [(&str, [f64; 4]); 3] = [
    (
        "L1021223.DNG",
        [
            0.999_251_106,
            -0.061_376_512_877_263_58,
            -0.093_915_541_393_360_16,
            0.055_882_009_215_291_75,
        ],
    ),
    (
        "L1026016.DNG",
        [
            0.999_251_106,
            -0.041_845_169_276_595_75,
            -0.102_311_478_808_510_63,
            0.057_876_719_170_212_77,
        ],
    ),
    (
        "L1026192.DNG",
        [
            0.999_251_106,
            -0.032_331_453_292_576_424,
            -0.104_204_110_174_672_49,
            0.056_364_293_537_117_91,
        ],
    ),
];

fn warp_rect(kr: [f64; 4]) -> WarpRect {
    WarpRect {
        kr,
        kt: [0.0, 0.0],
        cx: 0.5,
        cy: 0.5,
        planes: 1,
    }
}

/// The optical center and `m` (half-diagonal, DNG 1.7.0.0 §6.4.1) for a
/// `WIDTH x HEIGHT` plane with `cx_hat = cy_hat = 0.5` — mirrors
/// `src/warp.rs::source_coord`'s own arithmetic (kept independent, not
/// imported, so this test does not verify the implementation against
/// itself).
fn center_and_half_diagonal() -> (f64, f64, f64) {
    let cx = f64::from(WIDTH - 1) / 2.0;
    let cy = f64::from(HEIGHT - 1) / 2.0;
    let m = cx.hypot(cy);
    (cx, cy, m)
}

/// The source coordinate the polynomial predicts for the bottom-right
/// DESTINATION corner `(WIDTH-1, HEIGHT-1)`, per DNG 1.7.0.0 §6.4.1: at the
/// corner, `r = 1` exactly (the corner IS the farthest pixel from a centered
/// optical center), so `f(1) = kr0 + kr1 + kr2 + kr3` and the source
/// coordinate is `cx + f(1)*(corner - cx)`.
fn corner_source_coordinate(kr: [f64; 4]) -> (f64, f64) {
    let (cx, cy, _m) = center_and_half_diagonal();
    let f1 = kr[0] + kr[1] + kr[2] + kr[3];
    (
        cx + f1 * (f64::from(WIDTH - 1) - cx),
        cy + f1 * (f64::from(HEIGHT - 1) - cy),
    )
}

/// A `WIDTH x HEIGHT` all-zero plane with a single bright impulse at
/// `(x, y)` (rounded to the nearest pixel).
fn impulse_at(x: f64, y: f64) -> Vec<u16> {
    let mut plane = vec![0u16; (WIDTH as usize) * (HEIGHT as usize)];
    let px = (x.round().clamp(0.0, f64::from(WIDTH - 1))) as u32;
    let py = (y.round().clamp(0.0, f64::from(HEIGHT - 1))) as u32;
    let index = (py as usize) * (WIDTH as usize) + (px as usize);
    if let Some(slot) = plane.get_mut(index) {
        *slot = u16::MAX;
    }
    plane
}

/// The `(x, y)` of the brightest pixel in a `WIDTH x HEIGHT` plane.
fn argmax(plane: &[u16]) -> (u32, u32) {
    let (index, _value) = plane
        .iter()
        .enumerate()
        .max_by_key(|&(_, &v)| v)
        .expect("plane is non-empty");
    let x = u32::try_from(index % WIDTH as usize).expect("fits u32");
    let y = u32::try_from(index / WIDTH as usize).expect("fits u32");
    (x, y)
}

/// `AC4` — on all THREE decodable frames' coefficients
/// (`measurement-over-generalised`: one frame is one measurement, not a
/// boundary), an impulse placed at the polynomial's own predicted source
/// coordinate for the bottom-right corner reappears AT that corner after
/// warping, within +/-1 pixel — `SPIKE-001`'s ~504 px displacement is the
/// anchor for `L1021223.DNG` specifically; the other two frames' corner
/// displacements are smaller (`kr1` is smaller in magnitude) and computed
/// here from each frame's own coefficients, never hardcoded as 0.899841.
#[test]
fn warp_moves_corner_impulse_to_spike_001s_source_coord() {
    for (frame, kr) in FRAME_COEFFICIENTS {
        let (source_x, source_y) = corner_source_coordinate(kr);
        let (cx, cy, m) = center_and_half_diagonal();
        let f1 = kr[0] + kr[1] + kr[2] + kr[3];
        let displacement = m * (1.0 - f1);
        eprintln!(
            "{frame}: f(1) = {f1:.6}, corner displacement = {displacement:.1} px, \
             source = ({source_x:.2}, {source_y:.2}), center = ({cx:.1}, {cy:.1})"
        );

        let src = impulse_at(source_x, source_y);
        let mut dst = vec![0u16; src.len()];
        apply_warp_into(&warp_rect(kr), WIDTH, HEIGHT, &src, &mut dst)
            .unwrap_or_else(|e| panic!("{frame}: apply_warp_into: {e}"));

        let (peak_x, peak_y) = argmax(&dst);
        let expected_x = WIDTH - 1;
        let expected_y = HEIGHT - 1;
        assert!(
            peak_x.abs_diff(expected_x) <= 1 && peak_y.abs_diff(expected_y) <= 1,
            "{frame}: warped peak at ({peak_x}, {peak_y}), expected within 1px of the corner \
             ({expected_x}, {expected_y})"
        );
    }
}

/// `AC6` — the warp does not change `output_dimensions`. A `Sensor` matching
/// Q2M's documented geometry (`SPEC-014`'s measured shape) WITH a real,
/// non-identity `WarpRectilinear` opcode attached still reports the same
/// `(8368, 5584)` `output_dimensions` a warp-free sensor would — DNG
/// 1.7.0.0 §6.4.1: the warp maps the input extent to itself, and
/// `output_dimensions` does not (and must not) look at `opcode_list_3` at
/// all.
#[test]
fn output_dimensions_are_unchanged_by_warp() {
    let bytes = opcode_support::load_hex_fixture("opcodelist3-L1021223");
    let sensor = Sensor {
        ifd_index: 0,
        width: 8424,
        height: 5632,
        bits_per_sample: 14,
        samples_per_pixel: 1,
        photometric: 34892,
        compression: Compression::Uncompressed,
        rows_per_strip: None,
        strip_offsets: vec![],
        strip_byte_counts: vec![],
        black_level: Some(512),
        white_level: Some(16383),
        black_level_repeat_dim: None,
        active_area: Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        }),
        default_crop_origin: Some(DefaultCropOrigin { x: 12, y: 24 }),
        default_crop_size: Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        }),
        orientation: Some(1),
        opcode_lists: [false, false, true],
        opcode_list_3: Some(bytes),
        malformed_tags: vec![],
    };
    assert_eq!(
        output_dimensions(&sensor).expect("fits"),
        (8368, 5584),
        "a real WarpRectilinear opcode must not change output_dimensions"
    );
}

/// `AC10` — the tier-A red-proof: mutating `kr1` to `0.0` moves the impulse's
/// warped peak by at least 20 pixels versus the honest polynomial, on the
/// SAME synthetic corner-impulse fixture `AC4` uses, with no corpus
/// (`SPEC-015/FU-10`'s lesson, made an AC: `AC9` needs the real corpus and CI
/// runs 0/7 corpus files, so this is the only proof CI can see). Uses
/// `L1021223.DNG`'s coefficients (`kr1` is largest in magnitude there, the
/// clearest separation).
#[test]
fn warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more() {
    let honest_kr = FRAME_COEFFICIENTS[0].1;
    let (source_x, source_y) = corner_source_coordinate(honest_kr);
    let src = impulse_at(source_x, source_y);

    let mut honest_dst = vec![0u16; src.len()];
    apply_warp_into(&warp_rect(honest_kr), WIDTH, HEIGHT, &src, &mut honest_dst)
        .expect("honest warp applies");
    let honest_peak = argmax(&honest_dst);

    let mut mutated_kr = honest_kr;
    mutated_kr[1] = 0.0;
    let mut mutated_dst = vec![0u16; src.len()];
    apply_warp_into(
        &warp_rect(mutated_kr),
        WIDTH,
        HEIGHT,
        &src,
        &mut mutated_dst,
    )
    .expect("mutated warp applies");
    let mutated_peak = argmax(&mutated_dst);

    let dx = f64::from(honest_peak.0) - f64::from(mutated_peak.0);
    let dy = f64::from(honest_peak.1) - f64::from(mutated_peak.1);
    let distance = dx.hypot(dy);
    eprintln!(
        "L1021223.DNG: honest peak {honest_peak:?}, kr1=0 mutated peak {mutated_peak:?}, \
         distance {distance:.1} px"
    );
    assert!(
        distance >= 20.0,
        "zeroing kr1 must move the peak by >= 20px versus the honest polynomial; got {distance:.1}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 + AC9 — the develop-oracle score, and its red-proof (tier B)
// ─────────────────────────────────────────────────────────────────────────────

/// A full decode-and-develop-and-score against `dnglab analyze --srgb`
/// (`SPEC-020`'s oracle) for one corpus file, using the given `Sensor`
/// (already loaded, possibly mutated) as the develop input. `None` with the
/// skip already announced when the corpus or `dnglab` is absent.
fn develop_and_score(
    path: &str,
    sensor: &Sensor,
    plane: &[u16],
    data_path: &std::path::Path,
) -> Option<f64> {
    let (out_width, out_height) =
        output_dimensions(sensor).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut developed = vec![0u16; out_width as usize * out_height as usize];
    develop_into(sensor, plane, &mut developed)
        .unwrap_or_else(|e| panic!("{path}: develop_into: {e}"));

    let dnglab = match std::process::Command::new("dnglab")
        .args(["analyze", "--srgb"])
        .arg(data_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("SKIP {path} — dnglab: could not run ({e}) — is it on PATH?");
            return None;
        }
    };
    if !dnglab.status.success() {
        eprintln!(
            "SKIP {path} — dnglab exited {:?}: {}",
            dnglab.status.code(),
            String::from_utf8_lossy(&dnglab.stderr)
        );
        return None;
    }
    let reference = pnm::read_dnglab_srgb(&dnglab.stdout)
        .unwrap_or_else(|e| panic!("{path}: dnglab --srgb output must parse: {e}"));
    assert_eq!(
        (reference.width, reference.height),
        (out_width, out_height),
        "{path}: dnglab's own output and our developed image must be the same size"
    );

    Some(
        metric::score(out_width, out_height, &reference.samples, &developed)
            .unwrap_or_else(|e| panic!("{path}: scoring: {e}")),
    )
}

/// Decode `path`, unpack its plane, and return `(Sensor, raw plane, real
/// file path)` — `None` with the skip already announced when absent.
fn load_corpus_frame(
    manifest: &Manifest,
    root: &CorpusRoot,
    path: &str,
) -> Option<(Sensor, Vec<u16>, std::path::PathBuf)> {
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(root)?;
    let data = std::fs::read(&file_path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let container = Container::parse(&data).unwrap_or_else(|e| panic!("parse {path}: {e}"));
    let sensor = container
        .sensor()
        .unwrap_or_else(|e| panic!("sensor {path}: {e}"));
    let mut plane = vec![0u16; sensor.width as usize * sensor.height as usize];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane)
        .unwrap_or_else(|e| panic!("unpack {path}: {e}"));
    Some((sensor, plane, file_path))
}

const DECODABLE_Q2M: [&str; 3] = [
    "LEICA-Q2-MONO/L1021223.DNG",
    "LEICA-Q2-MONO/L1026016.DNG",
    "LEICA-Q2-MONO/L1026192.DNG",
];

/// `AC8` — the primary acceptance: `develop_into`'s real output on ALL THREE
/// decodable Q2M frames scores >= 85 through `SPEC-020`'s oracle. Reports
/// the measured score per frame (handback data — `measurement-over-
/// generalised`).
#[test]
#[ignore = "DEC-024: dnglab/rawler does not implement DNG OpcodeList processing at all \
            (verified against the dnglab/dnglab source), so SPEC-020's oracle cannot score \
            WarpRectilinear in either direction. Measured -60.169 on L1021223.DNG, worse than \
            a synthetic no-warp baseline (83.145 with an approximate gamma stand-in) -- applying \
            the correct warp diverges FURTHER from dnglab's uncorrected reference, not less. \
            Run explicitly with `cargo test -- --ignored` to re-measure. AC4/AC10 in this same \
            file are this spec's real, oracle-free correctness proof."]
fn warp_scores_at_least_eightyfive_via_spec_020_oracle() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    for path in DECODABLE_Q2M {
        let Some((sensor, plane, file_path)) = load_corpus_frame(&manifest, &root, path) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        let Some(score) = develop_and_score(path, &sensor, &plane, &file_path) else {
            continue; // SKIP already announced by develop_and_score
        };
        eprintln!("AC8 {path}: SSIMULACRA2 score = {score:.3}");
        assert!(
            score >= 85.0,
            "{path}: AC8 requires >= 85 via SPEC-020's oracle; got {score:.3} — this is a \
             FINDING, not a threshold to relax"
        );
    }
}

/// `AC9` — the red-proof: replacing `kr1` with `0.0` in the PARSED
/// `WarpRect` (code-side mutation of a TEST COPY, `SPEC-013`'s precedent,
/// `DEC-017`'s mechanism — not a mutation of the opcode bytes) must drop the
/// score below 85 on a real frame. Isolates the "pre-warp" developed image
/// via a sensor clone with `opcode_list_3` cleared (same technique
/// `tests/develop_oracle.rs` uses post-`SPEC-018`), applies the honest and
/// the mutated warp with [`apply_warp_into`] directly, and scores both.
///
/// ⚠ **Honest limit, recorded in `DEC-024`.** The HONEST score never reaches
/// 85 either (measured -60.193 on `L1021223.DNG`, `AC8`'s own finding:
/// `dnglab`/`rawler` do not implement DNG `OpcodeList` processing at all, so
/// `SPEC-020`'s oracle cannot see this feature in either direction). This
/// test's `< 85` assertion on the mutated score is therefore satisfied
/// VACUOUSLY — not because the mutation was caught against a passing
/// baseline, but because neither score ever passes. It is NOT the
/// "green-then-red" shape `oracle-must-be-shown-red` intends. `AC10`
/// (`tests/warp.rs`, tier A, no corpus) is this spec's REAL red-proof: it
/// measures a 339.5 px peak displacement from the same mutation against an
/// independent, oracle-free ground truth (the DNG spec's own formula).
#[test]
#[ignore = "DEC-024: this assertion is satisfied VACUOUSLY -- the honest score never reaches 85 \
            either (measured -60.193 on L1021223.DNG, same root cause as AC8's ignore reason), \
            so the mutated score being < 85 proves nothing about whether the mutation was \
            caught. Measured: honest -60.193, kr1=0 mutated -55.075 (LESS negative -- zeroing \
            kr1 moves the applied correction CLOSER to dnglab's always-uncorrected reference, \
            not further). AC10 in this same file is this spec's real red-proof. Run explicitly \
            with `cargo test -- --ignored` to re-measure."]
fn warp_oracle_is_red_on_a_zeroed_kr1_coefficient() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let path = "LEICA-Q2-MONO/L1021223.DNG";

    let Some((sensor, plane, file_path)) = load_corpus_frame(&manifest, &root, path) else {
        return; // SKIP already announced by CorpusFile::require
    };

    let bytes = sensor
        .opcode_list_3
        .as_ref()
        .unwrap_or_else(|| panic!("{path}: must carry a real OpcodeList3"));
    let honest_warp = parse_warp_rectilinear(bytes)
        .unwrap_or_else(|e| panic!("{path}: parse: {e}"))
        .unwrap_or_else(|| panic!("{path}: expected a WarpRectilinear opcode"));

    let sensor_no_warp = Sensor {
        opcode_list_3: None,
        ..sensor.clone()
    };
    let (out_width, out_height) =
        output_dimensions(&sensor_no_warp).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut unwarped = vec![0u16; out_width as usize * out_height as usize];
    develop_into(&sensor_no_warp, &plane, &mut unwarped)
        .unwrap_or_else(|e| panic!("{path}: develop_into (no warp): {e}"));

    let mut mutated_warp = honest_warp;
    mutated_warp.kr[1] = 0.0;

    let mut honest_developed = vec![0u16; unwarped.len()];
    apply_warp_into(
        &honest_warp,
        out_width,
        out_height,
        &unwarped,
        &mut honest_developed,
    )
    .expect("honest warp applies");
    let mut mutated_developed = vec![0u16; unwarped.len()];
    apply_warp_into(
        &mutated_warp,
        out_width,
        out_height,
        &unwarped,
        &mut mutated_developed,
    )
    .expect("mutated warp applies");

    let dnglab = match std::process::Command::new("dnglab")
        .args(["analyze", "--srgb"])
        .arg(&file_path)
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "SKIP warp_oracle_is_red_on_a_zeroed_kr1_coefficient — dnglab exited {:?}: {}",
                o.status.code(),
                String::from_utf8_lossy(&o.stderr)
            );
            return;
        }
        Err(e) => {
            eprintln!(
                "SKIP warp_oracle_is_red_on_a_zeroed_kr1_coefficient — dnglab: could not run \
                 ({e}) — is it on PATH?"
            );
            return;
        }
    };
    let reference = pnm::read_dnglab_srgb(&dnglab.stdout)
        .unwrap_or_else(|e| panic!("{path}: dnglab --srgb output must parse: {e}"));

    let honest_score = metric::score(out_width, out_height, &reference.samples, &honest_developed)
        .unwrap_or_else(|e| panic!("{path}: scoring honest: {e}"));
    let mutated_score = metric::score(
        out_width,
        out_height,
        &reference.samples,
        &mutated_developed,
    )
    .unwrap_or_else(|e| panic!("{path}: scoring mutated: {e}"));

    eprintln!(
        "AC9 {path}: honest score = {honest_score:.3}, kr1=0 mutated score = {mutated_score:.3}, \
         delta = {:.3}",
        honest_score - mutated_score
    );
    assert!(
        mutated_score < 85.0,
        "{path}: zeroing kr1 must drop the score below 85; got {mutated_score:.3} (honest {honest_score:.3})"
    );
}
