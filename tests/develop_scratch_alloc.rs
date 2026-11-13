//! `SPEC-065` `AC3`, `AC4`, `AC8`, `AC9` — the allocation contract of
//! `DEC-060` §2, counted.
//!
//! Its own test binary on purpose (the `tests/previews_alloc.rs` pattern): the
//! counting `#[global_allocator]` below sees every allocation in this process.
//! The switch and both counters are thread-local, so only allocations made BY
//! THE TEST THREAD between the switches count, and tests running in parallel
//! on other threads cannot disturb each other. Every `Sensor`, `src`, `dst`
//! and `scratch` is built before counting starts; tier-B files are read and
//! unpacked before it too (parsing and unpacking allocate, legitimately).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/opcode.rs"]
mod opcode_support;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{
    apply_fix_bad_pixels_constant, develop_with_into, develop_with_scratch_into,
    output_dimensions_with, scratch_len, BlackLevels, DevelopOptions, Framing,
};
use irradiance::ifd::{ActiveArea, Container, DefaultCropOrigin, DefaultCropSize, Sensor};
use irradiance::opcode::{parse_fix_bad_pixels_constant, parse_warp_rectilinear};
use irradiance::plane::unpack_into;

struct Counting;

// `const` thread-locals need no lazy initialisation, so touching them inside
// the allocator never allocates.
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

fn record(size: usize) {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|c| c.set(c.get() + 1));
        let _ = BYTES.try_with(|c| c.set(c.get() + size));
    }
}

// The library's `forbid(unsafe_code)` binds the library crate, not this test
// crate. `GlobalAlloc` is an unsafe trait, so implementing it needs
// `unsafe impl`; every method forwards to `System` unchanged.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Run `f` with counting on and return `(its result, allocations, bytes)`.
fn counted<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    ALLOCATIONS.with(|c| c.set(0));
    BYTES.with(|c| c.set(0));
    COUNTING.with(|c| c.set(true));
    let out = f();
    COUNTING.with(|c| c.set(false));
    (out, ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get))
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures (as tests/develop_scratch.rs)
// ─────────────────────────────────────────────────────────────────────────────

const Q2M_KR: [f64; 4] = [
    0.999_251_106,
    -0.061_376_512_877_263_58,
    -0.093_915_541_393_360_16,
    0.055_882_009_215_291_75,
];

const PIXEL_3A_KR: [f64; 4] = [
    0.980_158_686_637_878_4,
    0.091_988_243_162_631_99,
    -0.177_878_528_833_389_28,
    0.107_260_853_052_139_28,
];

fn bordered_sensor(orientation: u32) -> Sensor {
    let mut sensor = Sensor::test_fixture();
    sensor.width = 10;
    sensor.height = 8;
    sensor.bits_per_sample = 16;
    sensor.black_level = Some(0);
    sensor.white_level = Some(u32::from(u16::MAX));
    sensor.active_area = Some(ActiveArea {
        top: 1,
        left: 2,
        bottom: 7,
        right: 9,
    });
    sensor.default_crop_origin = Some(DefaultCropOrigin { x: 1, y: 1 });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: 4,
        height: 3,
    });
    sensor.orientation = Some(orientation);
    sensor
}

fn case_sensor(fbp: bool, warp: bool, orientation: u32) -> Sensor {
    let mut sensor = bordered_sensor(orientation);
    if fbp {
        sensor.opcode_list_1 = Some(opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0));
    }
    if warp {
        sensor.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
            Q2M_KR,
            [0.0, 0.0],
            0.5,
            0.5,
            1,
            0,
        ));
    }
    sensor
}

fn plane(sensor: &Sensor) -> Vec<u16> {
    let len = (sensor.width * sensor.height) as usize;
    (0..len)
        .map(|i| if i % 5 == 0 { 0 } else { 100 + 7 * i as u16 })
        .collect()
}

fn opts(framing: Framing, warp_alpha: f64, apply_orientation: bool) -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = framing;
    options.warp_alpha = warp_alpha;
    options.apply_orientation = apply_orientation;
    options
}

fn dst_len(sensor: &Sensor, options: &DevelopOptions) -> usize {
    output_dimensions_with(sensor, options)
        .map(|(w, h)| (w as usize) * (h as usize))
        .unwrap_or(12)
}

/// One prepared call: everything the develop reads or writes, built before
/// counting starts.
struct Prepared {
    label: String,
    sensor: Sensor,
    src: Vec<u16>,
    dst: Vec<u16>,
    scratch: Vec<u16>,
    options: DevelopOptions,
    expect_ok: bool,
}

fn prepare(
    label: String,
    sensor: Sensor,
    options: DevelopOptions,
    scratch_delta: isize,
) -> Prepared {
    let src = plane(&sensor);
    let dst = vec![0u16; dst_len(&sensor, &options)];
    let n = scratch_len(&sensor, &options).unwrap_or(0);
    let scratch = vec![0xA5A5u16; n.checked_add_signed(scratch_delta).unwrap()];
    Prepared {
        label,
        sensor,
        src,
        dst,
        scratch,
        options,
        expect_ok: scratch_delta == 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Negative control: the counter sees an allocation made between the switches.
#[test]
fn counter_counts() {
    let (v, allocations, bytes) = counted(|| vec![0u16; 1000]);
    assert_eq!(v.len(), 1000);
    assert_eq!(allocations, 1, "the counter must see a Vec");
    assert_eq!(bytes, 2000);
    let ((), allocations, _) = counted(|| {});
    assert_eq!(allocations, 0);
}

#[test]
fn scratch_develop_allocates_nothing() {
    let mut calls: Vec<Prepared> = Vec::new();
    for (fbp, warp) in [(false, false), (true, false), (false, true), (true, true)] {
        for orientation in [1, 6] {
            for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
                for alpha in [0.0, 0.5, 1.0] {
                    for apply_orientation in [true, false] {
                        calls.push(prepare(
                            format!(
                                "fbp {fbp} warp {warp} orientation {orientation} {framing:?} α \
                                 {alpha} oriented {apply_orientation}"
                            ),
                            case_sensor(fbp, warp, orientation),
                            opts(framing, alpha, apply_orientation),
                            0,
                        ));
                    }
                }
            }
        }
    }
    // Error paths: wrong-length scratch, a refused over-unity warp under
    // FullActiveArea, and an out-of-domain α.
    calls.push(prepare(
        "scratch one long".into(),
        case_sensor(true, true, 1),
        DevelopOptions::default(),
        1,
    ));
    calls.push(prepare(
        "scratch one short".into(),
        case_sensor(true, true, 1),
        DevelopOptions::default(),
        -1,
    ));
    let mut over_unity = case_sensor(true, false, 1);
    over_unity.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        PIXEL_3A_KR,
        [0.0, 0.0],
        0.501_746_677_217_029_4,
        0.502_773_708_767_361_2,
        1,
        0,
    ));
    let mut refused = prepare(
        "over-unity warp under FullActiveArea".into(),
        over_unity,
        opts(Framing::FullActiveArea, 1.0, true),
        0,
    );
    refused.expect_ok = false;
    calls.push(refused);
    for alpha in [f64::NAN, 1.5] {
        let mut bad = prepare(
            format!("warp_alpha {alpha}"),
            case_sensor(true, true, 1),
            DevelopOptions::default(),
            0,
        );
        bad.options.warp_alpha = alpha;
        bad.expect_ok = false;
        calls.push(bad);
    }

    let mut counts = vec![(false, 0usize); calls.len()];
    for (call, slot) in calls.iter_mut().zip(counts.iter_mut()) {
        let (result, allocations, _) = counted(|| {
            develop_with_scratch_into(
                &call.sensor,
                &call.src,
                &mut call.dst,
                &mut call.scratch,
                &call.options,
            )
            .is_ok()
        });
        *slot = (result, allocations);
    }
    for (call, (ok, allocations)) in calls.iter().zip(counts) {
        assert_eq!(ok, call.expect_ok, "{}: result", call.label);
        assert_eq!(allocations, 0, "{}: allocated", call.label);
    }
}

/// `SPEC-066` `AC9`: the per-position black pattern (`DEC-061` §4) reads
/// `sensor.black_levels` in place. A `[2, 2]` pattern of four distinct blacks
/// on the bordered fixture (`ActiveArea` origin (2, 1)), with and without a
/// warp and `FixBadPixelsConstant`, and the pattern refusals.
#[test]
fn per_position_allocates_nothing() {
    let per = |framing: Framing, alpha: f64| {
        let mut options = opts(framing, alpha, true);
        options.black_levels = BlackLevels::PerPosition;
        options
    };
    let patterned = |fbp: bool, warp: bool, blacks: &[u32], repeat: Option<[u32; 2]>| {
        let mut sensor = case_sensor(fbp, warp, 1);
        sensor.black_level = blacks.first().copied();
        sensor.black_levels = blacks.to_vec();
        sensor.black_level_repeat_dim = repeat;
        sensor.white_level = Some(4095);
        sensor
    };
    let mut calls: Vec<Prepared> = Vec::new();
    for (fbp, warp) in [(false, false), (true, false), (false, true), (true, true)] {
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            let sensor = patterned(fbp, warp, &[100, 200, 300, 400], Some([2, 2]));
            let options = per(framing, 1.0);
            // scratch_len does not depend on the option.
            assert_eq!(
                scratch_len(&sensor, &options).unwrap(),
                scratch_len(&sensor, &opts(framing, 1.0, true)).unwrap(),
                "fbp {fbp} warp {warp} {framing:?}: scratch_len"
            );
            calls.push(prepare(
                format!("fbp {fbp} warp {warp} {framing:?}"),
                sensor,
                options,
                0,
            ));
        }
    }
    for (label, blacks, repeat) in [
        ("4 values, [1, 1]", vec![100, 200, 300, 400], Some([1, 1])),
        ("3 values, [2, 2]", vec![100, 200, 300], Some([2, 2])),
        ("[0, 2]", vec![100, 200], Some([0, 2])),
        ("overflow", vec![100, 200], Some([u32::MAX, u32::MAX])),
        ("value at white", vec![100, 4095, 300, 400], Some([2, 2])),
    ] {
        let mut bad = prepare(
            label.into(),
            patterned(true, true, &blacks, repeat),
            per(Framing::StandardCrop, 1.0),
            0,
        );
        bad.expect_ok = false;
        calls.push(bad);
    }

    let mut counts = vec![(false, 0usize); calls.len()];
    for (call, slot) in calls.iter_mut().zip(counts.iter_mut()) {
        let (result, allocations, _) = counted(|| {
            develop_with_scratch_into(
                &call.sensor,
                &call.src,
                &mut call.dst,
                &mut call.scratch,
                &call.options,
            )
            .is_ok()
        });
        *slot = (result, allocations);
    }
    for (call, (ok, allocations)) in calls.iter().zip(counts) {
        assert_eq!(ok, call.expect_ok, "{}: result", call.label);
        assert_eq!(allocations, 0, "{}: allocated", call.label);
    }

    // develop_with_into: at most the one scratch allocation.
    for call in calls.iter_mut() {
        let (_, allocations, _) = counted(|| {
            let _ = develop_with_into(&call.sensor, &call.src, &mut call.dst, &call.options);
        });
        assert!(
            allocations <= 1,
            "{}: develop_with_into allocated {allocations}",
            call.label
        );
    }
}

#[test]
fn scratch_develop_allocates_nothing_on_q2m() {
    let path = "LEICA-Q2-MONO/L1021223.DNG";
    let Some((sensor, src)) = corpus_plane(path) else {
        return;
    };
    let options = DevelopOptions::default();
    let mut scratch = vec![0u16; scratch_len(&sensor, &options).unwrap()];
    let mut dst = vec![0u16; dst_len(&sensor, &options)];
    let (result, allocations, bytes) = counted(|| {
        develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &options).is_ok()
    });
    assert!(result, "{path}: develops");
    assert_eq!((allocations, bytes), (0, 0), "{path}: allocated");
}

#[test]
fn develop_with_into_allocates_once() {
    let options = DevelopOptions::default();

    let both = case_sensor(true, true, 1);
    let src = plane(&both);
    let mut dst = vec![0u16; dst_len(&both, &options)];
    let n = scratch_len(&both, &options).unwrap();
    let (result, allocations, bytes) =
        counted(|| develop_with_into(&both, &src, &mut dst, &options).is_ok());
    assert!(result);
    assert_eq!(
        (allocations, bytes),
        (1, n * 2),
        "both: one allocation of scratch_len samples"
    );

    let neither = case_sensor(false, false, 1);
    let mut dst = vec![0u16; dst_len(&neither, &options)];
    let (result, allocations, _) =
        counted(|| develop_with_into(&neither, &src, &mut dst, &options).is_ok());
    assert!(result);
    assert_eq!(allocations, 0, "neither: no allocation");

    let path = "LEICA-Q2-MONO/L1021223.DNG";
    if let Some((sensor, src)) = corpus_plane(path) {
        let mut dst = vec![0u16; dst_len(&sensor, &options)];
        let (result, allocations, bytes) =
            counted(|| develop_with_into(&sensor, &src, &mut dst, &options).is_ok());
        assert!(result, "{path}: develops");
        assert_eq!(
            (allocations, bytes),
            (1, 189_415_424),
            "{path}: one allocation of P + A samples (6 / 283,943,504 before SPEC-065)"
        );
    }
}

#[test]
fn opcode_lookups_allocate_nothing() {
    let warp = opcode_support::raw_opcode(
        1,
        0x0104_0000,
        0,
        &opcode_support::warp_rectilinear_params(Q2M_KR, [0.0, 0.0], 0.5, 0.5, 1),
    );
    let fbp = opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0);
    let optional_unknown = opcode_support::raw_opcode(0xDEAD_BEEF, 0x0104_0000, 1, &[1, 2, 3, 4]);
    let mandatory_unknown = opcode_support::raw_opcode(0xDEAD_BEEF, 0x0104_0000, 0, &[1, 2, 3, 4]);
    let lists: Vec<Vec<u8>> = vec![
        opcode_support::load_hex_fixture("opcodelist3-L1021223"),
        opcode_support::load_hex_fixture("opcodelist1-q2m"),
        fbp,
        opcode_support::opcode_list(&[optional_unknown.clone(), warp.clone(), optional_unknown]),
        opcode_support::opcode_list(&[warp.clone(), mandatory_unknown]),
        opcode_support::opcode_list(&[warp, opcode_support::raw_opcode(4, 0x0103_0000, 0, &[0])]),
        vec![0, 0],
    ];
    let mut counts = vec![0usize; lists.len()];
    for (bytes, slot) in lists.iter().zip(counts.iter_mut()) {
        let ((), allocations, _) = counted(|| {
            let _ = parse_warp_rectilinear(bytes);
            let _ = parse_fix_bad_pixels_constant(bytes);
        });
        *slot = allocations;
    }
    assert_eq!(counts, vec![0; lists.len()], "per-list allocation counts");
}

#[test]
fn fix_bad_pixels_allocates_nothing() {
    let sensor = bordered_sensor(1);
    let mut plane = plane(&sensor);
    let area = ActiveArea {
        top: 0,
        left: 0,
        bottom: 8,
        right: 10,
    };
    let (result, allocations, _) =
        counted(|| apply_fix_bad_pixels_constant(&mut plane, 10, 8, area, 0));
    let replaced = result.expect("plane is the right length");
    assert!(replaced > 0, "the plane has bad pixels to fix");
    assert_eq!(allocations, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B
// ─────────────────────────────────────────────────────────────────────────────

fn corpus_plane(path: &str) -> Option<(Sensor, Vec<u16>)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(&root)?;
    let bytes = std::fs::read(&file_path).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    Some((sensor, plane))
}
