//! `SPEC-098` AC1 to AC5 (`DEC-086`): the colour entries apply `OpcodeList1`'s
//! first `FixBadPixelsConstant` on the raw CFA, filling each bad photosite from
//! its four nearest photosites of the same colour.
//!
//! The scene is a 24×20 RGGB mosaic, black 0 and white 65535 (16-bit, no level
//! tags), with `v(x, y) = 1000 + 37x + 53y + [0, 9000, 9000, 20000][slot]` and
//! `slot = (y & 1)·2 + (x & 1)`. Twenty photosites are set to `Constant`: an
//! isolated one of each colour, an adjacent pair, a same-colour pair, corners
//! and edges, and a red one, (18, 10), whose four same-colour neighbours are
//! all bad.
//!
//! **The oracle** is the Adobe DNG SDK 1.7.1's `dng_validate -2` stage 2 on the
//! same mosaic written as a DNG (SPEC-098's design probe, 2026-11-27), at the
//! `BayerPhase` where the SDK's parity matches the mosaic. Its values are copied
//! below from the spec's Implementation Context as literals; nothing here calls
//! a kernel to make an expected value. Each test develops the scene with list 1
//! and compares it, exactly, with the same entry on the hand-fixed mosaic (the
//! literals written in) with list 1 removed.
//!
//! **Tier A**: synthetic fixtures through the real container read, no corpus.

#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, mono_mix_plane_into, mono_mix_plane_rows_into,
    srgb_linear_plane_f32_into, srgb_linear_plane_into, srgb_linear_plane_rows_f32_into,
    srgb_linear_plane_rows_into,
};
use irradiance::develop::{
    develop_with_into, develop_with_scratch_into, output_dimensions, scratch_len, DevelopOptions,
};
use irradiance::ifd::{ActiveArea, Container, Rational, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;

use opcode_support::{opcode_list, raw_opcode, single_fix_bad_pixels_constant_list};
use tiff::Order;

const W: u32 = 24;
const H: u32 = 20;
const MANDATORY: u32 = 0;
const OPTIONAL: u32 = 1;
const ID_FBPC: u32 = 4;

/// The 20 bad photosites and the SDK's stage 2 at each, `ActiveArea` at the
/// plane origin (`BayerPhase` 1 or 2 in the SDK, where its parity matches).
/// (18, 10) has no valid same-colour neighbour and stays at `Constant`.
const FIXED_AT_ORIGIN: [(u32, u32, u16); 20] = [
    (0, 0, 1090),
    (1, 0, 10090),
    (23, 0, 10867),
    (3, 1, 21199),
    (1, 3, 21221),
    (6, 6, 1540),
    (7, 6, 10572),
    (6, 7, 10598),
    (7, 7, 21630),
    (18, 8, 2055),
    (12, 10, 1974),
    (13, 10, 11011),
    (16, 10, 2097),
    (18, 10, 0),
    (20, 10, 2295),
    (18, 12, 2337),
    (12, 14, 2161),
    (14, 14, 2285),
    (0, 19, 10991),
    (23, 19, 22768),
];

/// The same scene plus a bad (0, 5), with the `ActiveArea` at (top 0, left 1,
/// bottom 20, right 24) (`BayerPhase` 0 in the SDK, where its parity then
/// matches). Column 0 is masked: its three bad photosites are not candidates
/// and stay 0.
const FIXED_AT_LEFT_1: [(u32, u32, u16); 21] = [
    (0, 0, 0),
    (0, 5, 0),
    (0, 19, 0),
    (1, 0, 10127),
    (23, 0, 10867),
    (3, 1, 1164),
    (1, 3, 1196),
    (6, 6, 21510),
    (7, 6, 10577),
    (6, 7, 10593),
    (7, 7, 1660),
    (18, 8, 22090),
    (12, 10, 21974),
    (13, 10, 11011),
    (16, 10, 22122),
    (18, 10, 22196),
    (20, 10, 22270),
    (18, 12, 22302),
    (12, 14, 22186),
    (14, 14, 22260),
    (23, 19, 2768),
];

/// Which `ActiveArea` the scene is developed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    /// No `ActiveArea` tag: the whole plane, origin (0, 0).
    Origin,
    /// `ActiveArea` (0, 1, 20, 24), set on the parsed `Sensor`.
    Left1,
    /// As `Origin`, with a 4×4 `BlackLevelRepeatDim` of zero blacks: the same
    /// levels, read through the colour path's per-photosite black pattern
    /// (`DEC-068` §2) rather than its four 2×2 slots.
    Pattern,
}

impl Scene {
    fn literals(self) -> &'static [(u32, u32, u16)] {
        match self {
            Scene::Origin | Scene::Pattern => &FIXED_AT_ORIGIN,
            Scene::Left1 => &FIXED_AT_LEFT_1,
        }
    }
}

/// The scene's value at `(x, y)` before any photosite is made bad.
fn scene_value(x: u32, y: u32) -> u16 {
    let offset = [0u32, 9000, 9000, 20000][((y & 1) * 2 + (x & 1)) as usize];
    (1000 + 37 * x + 53 * y + offset) as u16
}

fn index(x: u32, y: u32) -> usize {
    (y * W + x) as usize
}

/// The scene with every bad photosite of `scene` set to `bad`.
fn bad_mosaic(scene: Scene, bad: u16) -> Vec<u16> {
    let mut m: Vec<u16> = (0..H)
        .flat_map(|y| (0..W).map(move |x| scene_value(x, y)))
        .collect();
    for &(x, y, _) in scene.literals() {
        m[index(x, y)] = bad;
    }
    m
}

/// The hand-fixed mosaic: the bad mosaic with the SDK's literals written in
/// (a literal 0 means "stays at `Constant`", which is `bad`).
fn fixed_mosaic(scene: Scene, bad: u16) -> Vec<u16> {
    let mut m = bad_mosaic(scene, bad);
    for &(x, y, v) in scene.literals() {
        m[index(x, y)] = if v == 0 { bad } else { v };
    }
    m
}

/// Parse the 24×20 RGGB DNG over `mosaic` with `list1`, unpack it, and give it
/// the colour calibration the colour develop needs and `scene`'s `ActiveArea`.
fn sensor_for(scene: Scene, mosaic: &[u16], list1: Option<&[u8]>) -> (Sensor, Vec<u16>) {
    let data =
        tiff::dng_16bit_with_opcode_lists(Order::Little, true, W, H, mosaic, [list1, None, None]);
    let container = Container::parse(&data).expect("fixture parses");
    let mut sensor = container.sensor().expect("fixture has a sensor IFD");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane).expect("fixture unpacks");
    assert_eq!(plane, mosaic, "the fixture's mosaic reads back as written");
    sensor.forward_matrix_1 = Some(
        [
            [60000, 30000, 6422],
            [20000, 70000, 10000],
            [10000, 10000, 62521],
        ]
        .iter()
        .flatten()
        .map(|&num| Rational { num, den: 100000 })
        .collect(),
    );
    sensor.as_shot_neutral = Some(vec![
        Rational { num: 1, den: 2 },
        Rational { num: 1, den: 1 },
        Rational { num: 1, den: 2 },
    ]);
    if scene == Scene::Pattern {
        sensor.black_level_repeat_dim = Some([4, 4]);
        sensor.black_levels = vec![0; 16];
    }
    if scene == Scene::Left1 {
        sensor.active_area = Some(ActiveArea {
            top: 0,
            left: 1,
            bottom: H,
            right: W,
        });
    }
    (sensor, plane)
}

fn fbpc_list(constant: u32, bayer_phase: u32, flags: u32) -> Vec<u8> {
    single_fix_bad_pixels_constant_list(constant, bayer_phase, flags)
}

fn fbpc_op(constant: u32, flags: u32) -> Vec<u8> {
    let mut params = Vec::new();
    params.extend_from_slice(&constant.to_be_bytes());
    params.extend_from_slice(&0u32.to_be_bytes());
    raw_opcode(ID_FBPC, 0x0103_0000, flags, &params)
}

/// Rec. 709 luma, as SPEC-092's tests use.
const MIX: [Rational; 3] = [
    Rational {
        num: 2126,
        den: 10000,
    },
    Rational {
        num: 7152,
        den: 10000,
    },
    Rational {
        num: 722,
        den: 10000,
    },
];

/// The six AC1 entries, each its own door.
#[derive(Debug, Clone, Copy)]
enum Entry {
    Plane,
    PlaneF32,
    MixPlane,
    DevelopColour,
    DevelopColourF32,
    DevelopedMix,
}

const ENTRIES: [Entry; 6] = [
    Entry::Plane,
    Entry::PlaneF32,
    Entry::MixPlane,
    Entry::DevelopColour,
    Entry::DevelopColourF32,
    Entry::DevelopedMix,
];

fn developed_len(sensor: &Sensor, channels: usize) -> usize {
    let (w, h) = output_dimensions(sensor).expect("fixture geometry is valid");
    w as usize * h as usize * channels
}

fn mix_options() -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.mono_mix = Some(MIX);
    options
}

/// Run `entry`; its output as `u32`s (an `f32` as its bits).
fn run(entry: Entry, sensor: &Sensor, src: &[u16]) -> Result<Vec<u32>, Error> {
    let plane = (W * H) as usize;
    let widen = |v: Vec<u16>| v.into_iter().map(u32::from).collect();
    let bits = |v: Vec<f32>| v.into_iter().map(f32::to_bits).collect();
    match entry {
        Entry::Plane => {
            let mut dst = vec![0u16; plane * 3];
            srgb_linear_plane_into(sensor, src, &mut dst).map(|()| widen(dst))
        }
        Entry::PlaneF32 => {
            let mut dst = vec![0f32; plane * 3];
            srgb_linear_plane_f32_into(sensor, src, &mut dst).map(|()| bits(dst))
        }
        Entry::MixPlane => {
            let mut dst = vec![0u16; plane];
            mono_mix_plane_into(sensor, src, &MIX, &mut dst).map(|()| widen(dst))
        }
        Entry::DevelopColour => {
            let mut dst = vec![0u16; developed_len(sensor, 3)];
            develop_color_into(sensor, src, &mut dst).map(|()| widen(dst))
        }
        Entry::DevelopColourF32 => {
            let mut dst = vec![0f32; developed_len(sensor, 3)];
            develop_color_f32_into(sensor, src, &mut dst).map(|()| bits(dst))
        }
        Entry::DevelopedMix => {
            let mut dst = vec![0u16; developed_len(sensor, 1)];
            develop_with_into(sensor, src, &mut dst, &mix_options()).map(|()| widen(dst))
        }
    }
}

#[track_caller]
fn ok(entry: Entry, sensor: &Sensor, src: &[u16]) -> Vec<u32> {
    run(entry, sensor, src).unwrap_or_else(|e| panic!("{entry:?}: {e:?}"))
}

/// Every entry on `list1` over the bad mosaic equals the same entry on the
/// hand-fixed mosaic with no list 1, and differs from the unfixed develop.
#[track_caller]
fn assert_fixed_like_the_sdk(scene: Scene, bad: u16, list1: &[u8]) {
    let (with_list, bad_src) = sensor_for(scene, &bad_mosaic(scene, bad), Some(list1));
    let (reference, fixed_src) = sensor_for(scene, &fixed_mosaic(scene, bad), None);
    let (unfixed, unfixed_src) = sensor_for(scene, &bad_mosaic(scene, bad), None);
    for entry in ENTRIES {
        let got = ok(entry, &with_list, &bad_src);
        let want = ok(entry, &reference, &fixed_src);
        assert!(
            got == want,
            "{scene:?} {entry:?}: the fixed develop differs from the hand-fixed one at {} samples",
            got.iter().zip(&want).filter(|(a, b)| a != b).count()
        );
        let unfixed = ok(entry, &unfixed, &unfixed_src);
        assert!(
            got != unfixed,
            "{scene:?} {entry:?}: list 1 changed nothing (the fix did not run)"
        );
    }
}

/// AC1: green from its four diagonals, red and blue from the four photosites
/// two away on the axes, unfixed reads, mean rounded half up, (18, 10) left at
/// `Constant`; on all six entries.
#[test]
fn ac1_the_kernel_fills_from_the_four_nearest_same_colour_photosites() {
    assert_fixed_like_the_sdk(Scene::Origin, 0, &fbpc_list(0, 0, MANDATORY));
}

/// AC1, the `ActiveArea` (0, 1) variant: the colour is indexed from the
/// `ActiveArea` origin (`DEC-040`), only photosites inside the `ActiveArea` are
/// candidates (column 0's stay 0), and neighbours come from the whole plane
/// (the masked column counts).
#[test]
fn ac1_colour_is_indexed_from_the_active_area_origin() {
    assert_fixed_like_the_sdk(Scene::Left1, 0, &fbpc_list(0, 0, MANDATORY));
}

/// AC1 under a black pattern larger than 2×2: the fix sits at that
/// normaliser's raw read too (`DEC-086` item 2), so the same literals hold.
#[test]
fn ac1_the_fix_runs_under_a_black_pattern_larger_than_two_by_two() {
    assert_fixed_like_the_sdk(Scene::Pattern, 0, &fbpc_list(0, 0, MANDATORY));
}

/// AC2: `BayerPhase` is read and not used. 1, 2, 3 and 7 (out of range) give
/// exactly phase 0's output on both scenes.
#[test]
fn ac2_bayer_phase_is_not_used() {
    for scene in [Scene::Origin, Scene::Left1] {
        let src = bad_mosaic(scene, 0);
        let (phase0, _) = sensor_for(scene, &src, Some(&fbpc_list(0, 0, MANDATORY)));
        for phase in [1, 2, 3, 7] {
            let (other, _) = sensor_for(scene, &src, Some(&fbpc_list(0, phase, MANDATORY)));
            for entry in ENTRIES {
                assert_eq!(
                    ok(entry, &other, &src),
                    ok(entry, &phase0, &src),
                    "{scene:?} {entry:?}: BayerPhase {phase} changed the output"
                );
            }
        }
        // And phase 0 is the fixed develop (not merely equal to itself).
        assert_fixed_like_the_sdk(scene, 0, &fbpc_list(0, 7, MANDATORY));
    }
}

#[track_caller]
fn assert_required_list_1(r: Result<Vec<u32>, Error>, entry: Entry) {
    assert!(
        matches!(
            r,
            Err(Error::UnsupportedRequiredOpcode {
                list: 1,
                id: ID_FBPC
            })
        ),
        "{entry:?}: expected UnsupportedRequiredOpcode {{ list: 1, id: 4 }}, got {:?}",
        r.map(|v| v.len())
    );
}

#[track_caller]
fn assert_malformed_fbpc(r: Result<Vec<u32>, Error>, entry: Entry) {
    assert!(
        matches!(r, Err(Error::MalformedOpcodeParams { id: ID_FBPC, .. })),
        "{entry:?}: expected MalformedOpcodeParams {{ id: 4, .. }}, got {:?}",
        r.map(|v| v.len())
    );
}

/// AC3: the constant is any `u32` (compared as one), the first instance is
/// applied whatever its flags, a second mandatory one is refused, a malformed
/// one is refused whether optional or not, and that refusal comes before the
/// CFA's.
#[test]
fn ac3_constant_flags_and_instances() {
    // Constant 4000: the same literals, (18, 10) staying 4000. No other
    // scene sample is 4000 (the largest offset-0 value is 2858).
    assert_fixed_like_the_sdk(Scene::Origin, 4000, &fbpc_list(4000, 0, MANDATORY));

    // Constant 65536 matches no u16 sample: nothing is fixed, nothing refused.
    let src = bad_mosaic(Scene::Origin, 0);
    let (wide, _) = sensor_for(Scene::Origin, &src, Some(&fbpc_list(65536, 0, MANDATORY)));
    let (none, _) = sensor_for(Scene::Origin, &src, None);
    for entry in ENTRIES {
        assert_eq!(
            ok(entry, &wide, &src),
            ok(entry, &none, &src),
            "{entry:?}: Constant 65536 fixed something (compared as u16?)"
        );
    }

    // An optional first instance is applied.
    assert_fixed_like_the_sdk(Scene::Origin, 0, &fbpc_list(0, 0, OPTIONAL));
    // ...and an optional duplicate behind the applied one is skipped.
    assert_fixed_like_the_sdk(
        Scene::Origin,
        0,
        &opcode_list(&[fbpc_op(0, MANDATORY), fbpc_op(4000, OPTIONAL)]),
    );

    // A second mandatory instance is not applied, so it is refused.
    let two = opcode_list(&[fbpc_op(0, MANDATORY), fbpc_op(0, MANDATORY)]);
    let (sensor, _) = sensor_for(Scene::Origin, &src, Some(&two));
    for entry in ENTRIES {
        assert_required_list_1(run(entry, &sensor, &src), entry);
    }

    // A malformed (7-byte) first instance, optional or not.
    for flags in [MANDATORY, OPTIONAL] {
        let short = opcode_list(&[raw_opcode(ID_FBPC, 0x0103_0000, flags, &[0; 7])]);
        let (mut sensor, _) = sensor_for(Scene::Origin, &src, Some(&short));
        for entry in ENTRIES {
            assert_malformed_fbpc(run(entry, &sensor, &src), entry);
        }
        // DEC-086 item 7: the FBPC refusal ranks before the CFA's.
        sensor.color_filter = None;
        for entry in ENTRIES {
            assert_malformed_fbpc(run(entry, &sensor, &src), entry);
        }
    }

    // With a well-formed FBPC and no CFA, no fix is built and the entry
    // refuses at its CFA check, as before.
    let (mut sensor, _) = sensor_for(Scene::Origin, &src, Some(&fbpc_list(0, 0, MANDATORY)));
    sensor.color_filter = None;
    for entry in ENTRIES {
        let r = run(entry, &sensor, &src);
        assert!(
            matches!(r, Err(Error::NotAColorFilterArray)),
            "{entry:?}: expected NotAColorFilterArray, got {:?}",
            r.map(|v| v.len())
        );
    }
}

/// AC4: the fix compares raw samples, before black subtraction. With a black
/// of 512, three non-bad photosites at 100 (below black, normalised to 0, and
/// none a same-colour neighbour of a bad one) are not touched, and the bad
/// photosites take the same raw literals.
#[test]
fn ac4_the_fix_reads_raw_samples() {
    let below_black = [(10, 2), (11, 3), (4, 17)];
    let with_black = |mosaic: &[u16], list1: Option<&[u8]>| {
        let mut m = mosaic.to_vec();
        for (x, y) in below_black {
            m[index(x, y)] = 100;
        }
        let (mut sensor, src) = sensor_for(Scene::Origin, &m, list1);
        sensor.black_level = Some(512);
        sensor.black_levels = vec![512];
        (sensor, src)
    };
    let (with_list, bad_src) = with_black(
        &bad_mosaic(Scene::Origin, 0),
        Some(&fbpc_list(0, 0, MANDATORY)),
    );
    let (reference, fixed_src) = with_black(&fixed_mosaic(Scene::Origin, 0), None);
    for entry in ENTRIES {
        assert_eq!(
            ok(entry, &with_list, &bad_src),
            ok(entry, &reference, &fixed_src),
            "{entry:?}: the fix did not run on raw values"
        );
    }
}

/// AC5: the fix sits at the one raw read, so every traversal sees the same
/// fixed value: bands tiling the plane equal the whole plane, the threaded
/// develops equal the single-thread one, and the developed mix's scratch form
/// equals its allocating form.
#[test]
fn ac5_bands_threads_and_strips_equal_the_whole() {
    let src = bad_mosaic(Scene::Origin, 0);
    let (sensor, _) = sensor_for(Scene::Origin, &src, Some(&fbpc_list(0, 0, MANDATORY)));
    let plane = (W * H) as usize;
    let bands = [0..3, 3..11, 11..20];

    let mut whole = vec![0u16; plane * 3];
    srgb_linear_plane_into(&sensor, &src, &mut whole).expect("plane");
    let mut tiled = Vec::new();
    for rows in bands.clone() {
        let mut band = vec![0u16; (rows.end - rows.start) as usize * W as usize * 3];
        srgb_linear_plane_rows_into(&sensor, &src, rows, &mut band).expect("band");
        tiled.extend(band);
    }
    assert_eq!(tiled, whole, "srgb_linear_plane_rows_into");

    let mut whole = vec![0f32; plane * 3];
    srgb_linear_plane_f32_into(&sensor, &src, &mut whole).expect("plane f32");
    let mut tiled = Vec::new();
    for rows in bands.clone() {
        let mut band = vec![0f32; (rows.end - rows.start) as usize * W as usize * 3];
        srgb_linear_plane_rows_f32_into(&sensor, &src, rows, &mut band).expect("band f32");
        tiled.extend(band);
    }
    assert_eq!(
        tiled.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        whole.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "srgb_linear_plane_rows_f32_into"
    );

    let mut whole = vec![0u16; plane];
    mono_mix_plane_into(&sensor, &src, &MIX, &mut whole).expect("mix plane");
    let mut tiled = Vec::new();
    for rows in bands {
        let mut band = vec![0u16; (rows.end - rows.start) as usize * W as usize];
        mono_mix_plane_rows_into(&sensor, &src, &MIX, rows, &mut band).expect("mix band");
        tiled.extend(band);
    }
    assert_eq!(tiled, whole, "mono_mix_plane_rows_into");

    let n = developed_len(&sensor, 3);
    let mut single = vec![0u16; n];
    develop_color_into(&sensor, &src, &mut single).expect("develop");
    let mut single_f32 = vec![0f32; n];
    develop_color_f32_into(&sensor, &src, &mut single_f32).expect("develop f32");
    for threads in [1, 2, 3] {
        let mut dst = vec![0u16; n];
        develop_color_into_threaded(&sensor, &src, &mut dst, threads).expect("threaded");
        assert_eq!(dst, single, "develop_color_into_threaded at {threads}");
        let mut dst = vec![0f32; n];
        develop_color_f32_into_threaded(&sensor, &src, &mut dst, threads).expect("threaded f32");
        assert_eq!(
            dst.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            single_f32.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "develop_color_f32_into_threaded at {threads}"
        );
    }

    let options = mix_options();
    let mut allocating = vec![0u16; developed_len(&sensor, 1)];
    develop_with_into(&sensor, &src, &mut allocating, &options).expect("mix");
    let mut scratch = vec![0u16; scratch_len(&sensor, &options).expect("scratch_len")];
    let mut with_scratch = vec![0u16; allocating.len()];
    develop_with_scratch_into(&sensor, &src, &mut with_scratch, &mut scratch, &options)
        .expect("mix scratch");
    assert_eq!(with_scratch, allocating, "the developed mix's scratch form");
    // Not vacuous: the bad photosites are in these outputs.
    let (unfixed, _) = sensor_for(Scene::Origin, &src, None);
    let mut plain = vec![0u16; allocating.len()];
    develop_with_into(&unfixed, &src, &mut plain, &options).expect("unfixed mix");
    assert_ne!(plain, allocating, "the mix's fix changed nothing");
}
