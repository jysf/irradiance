//! `SPEC-067` — the mono orientation gather walks the output in 64×64 tiles for
//! Orientations 5–8 (`DEC-062`). Only the visiting order changes, so the output
//! must be byte-identical to the row walk it replaces.
//!
//! **Two oracles, neither of them the crate:**
//!
//! - **Tier A (`AC2`).** [`expected_image`] builds the expected output with its
//!   own scatter: every crop pixel is sent to its oriented position by the
//!   TIFF 6.0 / DNG `Orientation` definitions written out below
//!   ([`oriented_position`]), and its value is [`normalize`] — the DNG affine map
//!   with `DEC-018`'s clamp and round-half-up and `SPEC-066`'s per-position
//!   black — also written out here. Nothing here calls `crop_source_coords`.
//!   The crop sizes straddle every tile edge (1×1, 63×65, 64×64, 65×63, 130×67,
//!   200×129) and every fixture has odd `ActiveArea` and `DefaultCrop` origins.
//!   The warped case compares the oriented develop against the test's own
//!   re-orientation of the `apply_orientation: false` develop, so the warp
//!   itself needs no reference.
//! - **Tier B (`AC1`).** Twenty-four MD5s of `develop_with_into`'s output on
//!   `L1021223` with `Orientation` overridden to each of 1–8, computed on main at
//!   `eaa2ae3` **before** this change (`SPEC-067` §1). If one goes red the
//!   change altered output: fix the change, never the pin.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/opcode.rs"]
mod opcode_support;

use corpus::{CorpusRoot, Manifest};
use irradiance::develop::{
    develop_with_into, develop_with_scratch_into, output_dimensions_with, scratch_len, BlackLevels,
    DevelopOptions, Framing,
};
use irradiance::ifd::{ActiveArea, Container, DefaultCropOrigin, DefaultCropSize, Sensor};
use irradiance::plane::unpack_into;
use irradiance::DEVELOP_VERSION;

const SENTINEL: u16 = 0xABCD;

/// The Q2M's measured `WarpRectilinear` radial coefficients: a real,
/// non-identity warp that stays inside the image, so it is accepted under
/// `FullActiveArea` too (`DEC-059` §3).
const Q2M_KR: [f64; 4] = [
    0.999_251_106,
    -0.061_376_512_877_263_58,
    -0.093_915_541_393_360_16,
    0.055_882_009_215_291_75,
];

// ─────────────────────────────────────────────────────────────────────────────
// The oracle, written out independently of the crate
// ─────────────────────────────────────────────────────────────────────────────

/// DNG's `[black, white] -> [0, 65535]` map: clamp (`DEC-018`), scale, round
/// half up in integers.
fn normalize(sample: u32, black: u32, white: u32) -> u16 {
    let clamped = sample.clamp(black, white);
    let numerator = u64::from(clamped - black) * 65535;
    let denominator = u64::from(white - black);
    u16::try_from((numerator + denominator / 2) / denominator).unwrap()
}

/// Where crop pixel `(cx, cy)` of a `w × h` crop lands in the oriented output,
/// from the TIFF 6.0 `Orientation` table (which corner of the displayed image
/// the stored row 0 / column 0 is). Written as the forward map (crop → output),
/// the opposite direction from the crate's gather, so a shared mistake would
/// have to be made twice in two different forms.
fn oriented_position(orientation: u32, cx: u32, cy: u32, w: u32, h: u32) -> (u32, u32) {
    match orientation {
        // Row 0 top, column 0 left: as stored.
        1 => (cx, cy),
        // Row 0 top, column 0 right: mirrored left-right.
        2 => (w - 1 - cx, cy),
        // Row 0 bottom, column 0 right: rotated 180°.
        3 => (w - 1 - cx, h - 1 - cy),
        // Row 0 bottom, column 0 left: mirrored top-bottom.
        4 => (cx, h - 1 - cy),
        // Row 0 left, column 0 top: transposed about the main diagonal.
        5 => (cy, cx),
        // Row 0 right, column 0 top: rotated 90° clockwise.
        6 => (h - 1 - cy, cx),
        // Row 0 right, column 0 bottom: transposed about the anti-diagonal.
        7 => (h - 1 - cy, w - 1 - cx),
        // Row 0 left, column 0 bottom: rotated 90° counter-clockwise.
        8 => (cy, w - 1 - cx),
        other => panic!("orientation {other} is not 1..=8"),
    }
}

/// The output size for a `w × h` crop: 5–8 swap the axes.
fn oriented_size(orientation: u32, w: u32, h: u32) -> (u32, u32) {
    if orientation >= 5 {
        (h, w)
    } else {
        (w, h)
    }
}

/// Scatter a `w × h` image (row-major) to its oriented layout.
fn reorient(orientation: u32, w: u32, h: u32, pixels: &[u16]) -> Vec<u16> {
    let (out_w, out_h) = oriented_size(orientation, w, h);
    let mut out = vec![SENTINEL; (out_w * out_h) as usize];
    let mut written = vec![false; out.len()];
    for cy in 0..h {
        for cx in 0..w {
            let (ox, oy) = oriented_position(orientation, cx, cy, w, h);
            let i = (oy * out_w + ox) as usize;
            assert!(
                !written[i],
                "orientation {orientation}: ({ox}, {oy}) hit twice"
            );
            written[i] = true;
            out[i] = pixels[(cy * w + cx) as usize];
        }
    }
    assert!(
        written.iter().all(|&w| w),
        "orientation {orientation}: a hole"
    );
    out
}

/// A fixture's geometry, kept beside the `Sensor` so the expected image comes
/// from these numbers and not from the crate.
#[derive(Clone, Copy, Debug)]
struct Shape {
    width: u32,
    height: u32,
    /// `ActiveArea` (left, top, right, bottom).
    active: (u32, u32, u32, u32),
    /// `DefaultCrop` origin within `ActiveArea`, and size.
    crop: (u32, u32, u32, u32),
}

impl Shape {
    /// A `crop_w × crop_h` crop at the odd origin `(1, 3)` inside an
    /// `ActiveArea` at the odd origin `(3, 1)`, with a border of 2 / 3 on the
    /// far sides and 1 more column/row of plane padding beyond that.
    fn around(crop_w: u32, crop_h: u32) -> Shape {
        let (left, top) = (3, 1);
        let (origin_x, origin_y) = (1, 3);
        let active_w = origin_x + crop_w + 2;
        let active_h = origin_y + crop_h + 3;
        Shape {
            width: left + active_w + 1,
            height: top + active_h + 1,
            active: (left, top, left + active_w, top + active_h),
            crop: (origin_x, origin_y, crop_w, crop_h),
        }
    }

    /// The framed rectangle within the plane: `(raw_x0, raw_y0, w, h)`.
    fn framed(&self, framing: Framing) -> (u32, u32, u32, u32) {
        match framing {
            Framing::StandardCrop => (
                self.active.0 + self.crop.0,
                self.active.1 + self.crop.1,
                self.crop.2,
                self.crop.3,
            ),
            _ => (
                self.active.0,
                self.active.1,
                self.active.2 - self.active.0,
                self.active.3 - self.active.1,
            ),
        }
    }
}

/// The expected develop: each framed photosite's `value(raw_x, raw_y)`,
/// scattered to its oriented position.
fn expected_image(
    shape: &Shape,
    framing: Framing,
    orientation: u32,
    value: impl Fn(u32, u32) -> u16,
) -> (u32, u32, Vec<u16>) {
    let (x0, y0, w, h) = shape.framed(framing);
    let mut framed = Vec::with_capacity((w * h) as usize);
    for cy in 0..h {
        for cx in 0..w {
            framed.push(value(x0 + cx, y0 + cy));
        }
    }
    let (out_w, out_h) = oriented_size(orientation, w, h);
    (out_w, out_h, reorient(orientation, w, h, &framed))
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// The crop sizes `AC2` names: below, at and above one 64-pixel tile on each
/// axis, and two that span several tiles with ragged edges.
const CROPS: [(u32, u32); 6] = [(1, 1), (63, 65), (64, 64), (65, 63), (130, 67), (200, 129)];

const BLACK: u32 = 64;
const WHITE: u32 = 65000;

fn sensor_with(shape: &Shape, orientation: u32) -> Sensor {
    let mut sensor = Sensor::test_fixture();
    sensor.width = shape.width;
    sensor.height = shape.height;
    sensor.bits_per_sample = 16;
    sensor.black_level = Some(BLACK);
    sensor.black_levels = vec![BLACK];
    sensor.white_level = Some(WHITE);
    sensor.white_levels = vec![WHITE];
    sensor.active_area = Some(ActiveArea {
        left: shape.active.0,
        top: shape.active.1,
        right: shape.active.2,
        bottom: shape.active.3,
    });
    sensor.default_crop_origin = Some(DefaultCropOrigin {
        x: shape.crop.0,
        y: shape.crop.1,
    });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: shape.crop.2,
        height: shape.crop.3,
    });
    sensor.orientation = Some(orientation);
    sensor
}

fn with_warp(mut sensor: Sensor) -> Sensor {
    sensor.opcode_list_3 = Some(opcode_support::single_warp_rectilinear_list(
        Q2M_KR,
        [0.0, 0.0],
        0.5,
        0.5,
        1,
        0,
    ));
    sensor.opcode_lists = [false, false, true];
    sensor
}

/// A plane distinct at every photosite (the largest fixture has under 30,000),
/// so any transposed, shifted or skipped read changes the output.
fn distinct_plane(shape: &Shape) -> Vec<u16> {
    (0..shape.width * shape.height)
        .map(|i| u16::try_from(100 + i).unwrap())
        .collect()
}

fn options(framing: Framing, warp_alpha: f64, apply_orientation: bool) -> DevelopOptions {
    let mut options = DevelopOptions::default();
    options.framing = framing;
    options.warp_alpha = warp_alpha;
    options.apply_orientation = apply_orientation;
    options
}

/// Develop through `develop_with_into` AND `develop_with_scratch_into` into
/// sentinel-filled buffers, assert they agree, and return `(w, h, pixels)`.
fn develop(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> (u32, u32, Vec<u16>) {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions");
    let mut dst = vec![SENTINEL; (w * h) as usize];
    develop_with_into(sensor, src, &mut dst, options).expect("develop_with_into");
    let mut scratch = vec![0xA5A5u16; scratch_len(sensor, options).expect("scratch_len")];
    let mut via_scratch = vec![SENTINEL; dst.len()];
    develop_with_scratch_into(sensor, src, &mut via_scratch, &mut scratch, options)
        .expect("develop_with_scratch_into");
    assert_eq!(dst, via_scratch, "the two entry points disagree");
    (w, h, dst)
}

/// Assert two images are equal, naming the first differing pixel.
fn assert_same(label: &str, w: u32, got: &[u16], want: &[u16]) {
    assert_eq!(got.len(), want.len(), "{label}: length");
    if let Some(i) = (0..got.len()).find(|&i| got[i] != want[i]) {
        let (x, y) = (i as u32 % w, i as u32 / w);
        panic!(
            "{label}: first difference at ({x}, {y}): got {} want {}",
            got[i], want[i]
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — tier A, against the independent gather
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn every_orientation_matches_an_independent_gather() {
    assert_eq!(DEVELOP_VERSION, "4");
    for (crop_w, crop_h) in CROPS {
        let shape = Shape::around(crop_w, crop_h);
        let src = distinct_plane(&shape);
        let value = |x: u32, y: u32| {
            normalize(u32::from(src[(y * shape.width + x) as usize]), BLACK, WHITE)
        };
        for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
            // The stored-order warped develop, for this framing: the input the
            // post-warp gather orients.
            let warped = with_warp(sensor_with(&shape, 1));
            let (stored_w, stored_h, stored) =
                develop(&warped, &src, &options(framing, 1.0, false));
            if crop_w > 1 {
                // The warp really ran: it moves samples, so the stored-order
                // warped develop is not the warp-free one.
                let (_, _, unwarped) = develop(&warped, &src, &options(framing, 0.0, false));
                assert_ne!(
                    stored, unwarped,
                    "{shape:?} {framing:?}: the warp was a no-op"
                );
            }

            for orientation in 1..=8 {
                let label = format!("{crop_w}×{crop_h} {framing:?} O{orientation}");

                // Warp-free: the whole expected image from the test's own
                // normalize and orientation.
                let sensor = sensor_with(&shape, orientation);
                let (w, h, got) = develop(&sensor, &src, &options(framing, 0.0, true));
                let (want_w, want_h, want) = expected_image(&shape, framing, orientation, value);
                assert_eq!((w, h), (want_w, want_h), "{label} α 0: dimensions");
                assert_same(&format!("{label} α 0"), w, &got, &want);

                // Warped: the oriented develop is the stored-order develop
                // re-oriented by the test's own formulas.
                let sensor = with_warp(sensor_with(&shape, orientation));
                let (w, h, got) = develop(&sensor, &src, &options(framing, 1.0, true));
                assert_eq!(
                    (w, h),
                    oriented_size(orientation, stored_w, stored_h),
                    "{label} α 1: dimensions"
                );
                let want = reorient(orientation, stored_w, stored_h, &stored);
                assert_same(&format!("{label} α 1"), w, &got, &want);
            }
        }
    }
}

#[test]
fn per_position_blacks_survive_the_tiled_walk() {
    // Four distinct blacks on a [2, 2] pattern phased from the odd `ActiveArea`
    // origin (3, 1): a pixel read from the wrong photosite, or normalized
    // with its neighbour's black, changes the value.
    const BLACKS: [u32; 4] = [100, 200, 300, 400];
    let pattern = |x: u32, y: u32| {
        let row = (y - 1) % 2;
        let col = (x + 1) % 2; // (x - 3) mod 2, without the underflow at x < 3
        BLACKS[(row * 2 + col) as usize]
    };
    for (crop_w, crop_h) in [(130, 67), (65, 63)] {
        let shape = Shape::around(crop_w, crop_h);
        let src = distinct_plane(&shape);
        for orientation in 1..=8 {
            let mut sensor = sensor_with(&shape, orientation);
            sensor.black_level = Some(BLACKS[0]);
            sensor.black_levels = BLACKS.to_vec();
            sensor.black_level_repeat_dim = Some([2, 2]);
            for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
                let mut opts = options(framing, 0.0, true);
                opts.black_levels = BlackLevels::PerPosition;
                let (w, _, got) = develop(&sensor, &src, &opts);
                let (_, _, want) = expected_image(&shape, framing, orientation, |x, y| {
                    normalize(
                        u32::from(src[(y * shape.width + x) as usize]),
                        pattern(x, y),
                        WHITE,
                    )
                });
                assert_same(
                    &format!("{crop_w}×{crop_h} {framing:?} O{orientation} PerPosition"),
                    w,
                    &got,
                    &want,
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — tier B, the 24 pre-change pins
// ─────────────────────────────────────────────────────────────────────────────

/// `SPEC-067` §1: `(orientation, warp_alpha, framing, width, height, md5)` of
/// `develop_with_into` on `L1021223` with `Orientation` overridden, from main
/// at `eaa2ae3`, before this change.
#[rustfmt::skip]
const PINNED: [(u32, f64, Framing, u32, u32, &str); 24] = [
    (1, 0.0, Framing::StandardCrop, 8368, 5584, "5987848b3a7d67f5eedcc15643424fe3"),
    (2, 0.0, Framing::StandardCrop, 8368, 5584, "48e16c555ce7902dfe69f667b5fdb5aa"),
    (3, 0.0, Framing::StandardCrop, 8368, 5584, "47a94fb041d111fd1ec933458258987f"),
    (4, 0.0, Framing::StandardCrop, 8368, 5584, "e47775dac94c816b54b246c58425f567"),
    (5, 0.0, Framing::StandardCrop, 5584, 8368, "1cc4629f4b01eee22e5b27644bb507a5"),
    (6, 0.0, Framing::StandardCrop, 5584, 8368, "c4e27ed3c4c5db88b9ce5b90e479953e"),
    (7, 0.0, Framing::StandardCrop, 5584, 8368, "7bdd2a15d81c6e038a49f6f7fe44e49c"),
    (8, 0.0, Framing::StandardCrop, 5584, 8368, "018aa23a740a9709292ed4f197972e88"),
    (1, 1.0, Framing::StandardCrop, 8368, 5584, "feb4910588d189bd5bf89620b480fe6c"),
    (2, 1.0, Framing::StandardCrop, 8368, 5584, "bfddc03dfba0a5e11d0bccd4e8bd0776"),
    (3, 1.0, Framing::StandardCrop, 8368, 5584, "c59395e798fa7e077800d263868957c9"),
    (4, 1.0, Framing::StandardCrop, 8368, 5584, "2cb1916f23c45f22c412a31529f24e92"),
    (5, 1.0, Framing::StandardCrop, 5584, 8368, "a4292458049cba8683bc9824374817ba"),
    (6, 1.0, Framing::StandardCrop, 5584, 8368, "94059540d3da4547880ae4497bffcf16"),
    (7, 1.0, Framing::StandardCrop, 5584, 8368, "3e2e9f08e1f6f9d20376c12ce5f85852"),
    (8, 1.0, Framing::StandardCrop, 5584, 8368, "6ed7ac26d186433bc3c18982f737b2b8"),
    (1, 0.0, Framing::FullActiveArea, 8392, 5632, "70dfe17181c2e382cabba831d84866fb"),
    (2, 0.0, Framing::FullActiveArea, 8392, 5632, "0a4d591a282863fd5b929ea2753b4fb5"),
    (3, 0.0, Framing::FullActiveArea, 8392, 5632, "4ee3c4b57ff4da6adae334fc163df5c3"),
    (4, 0.0, Framing::FullActiveArea, 8392, 5632, "5643a83af6a697f049b5a2ec5b1e55fc"),
    (5, 0.0, Framing::FullActiveArea, 5632, 8392, "0ef1ae6659cf9db99de7dcb8195ae594"),
    (6, 0.0, Framing::FullActiveArea, 5632, 8392, "3ffb1ff599af4add4f4ced0c4c24134c"),
    (7, 0.0, Framing::FullActiveArea, 5632, 8392, "f28a2e0384100ab4e6755633fc2df9df"),
    (8, 0.0, Framing::FullActiveArea, 5632, 8392, "43dbea0e36328116738b6c32e25834f4"),
];

const PINNED_FILE: &str = "LEICA-Q2-MONO/L1021223.DNG";

fn md5_le(samples: &[u16]) -> String {
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    md5::to_hex(&md5::hash(&bytes))
}

#[test]
fn tiled_output_matches_the_pre_change_pins() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(PINNED_FILE)
        .unwrap_or_else(|| panic!("{PINNED_FILE} must be in the manifest"));
    let Some(file_path) = entry.require(&root) else {
        eprintln!("SKIP {PINNED_FILE}: not present");
        return;
    };
    let bytes = std::fs::read(&file_path).unwrap_or_else(|e| panic!("{PINNED_FILE}: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{PINNED_FILE}: {e}"));
    let base = container
        .sensor()
        .unwrap_or_else(|e| panic!("{PINNED_FILE}: {e}"));
    let mut src = vec![0u16; (base.width as usize) * (base.height as usize)];
    unpack_into(&base, container.byte_order(), &bytes, &mut src)
        .unwrap_or_else(|e| panic!("{PINNED_FILE}: unpack: {e}"));
    assert!(base.opcode_list_3.is_some(), "{PINNED_FILE} carries a warp");

    // One scratch buffer, sized for the largest case, reused by every call.
    let mut scratch: Vec<u16> = Vec::new();
    for (orientation, alpha, framing, width, height, md5) in PINNED {
        let label = format!("{PINNED_FILE} O{orientation} α {alpha} {framing:?}");
        let mut sensor = base.clone();
        sensor.orientation = Some(orientation);
        let opts = options(framing, alpha, true);
        assert_eq!(
            output_dimensions_with(&sensor, &opts).unwrap(),
            (width, height),
            "{label}: dimensions"
        );
        let mut dst = vec![SENTINEL; (width as usize) * (height as usize)];
        develop_with_into(&sensor, &src, &mut dst, &opts)
            .unwrap_or_else(|e| panic!("{label}: {e}"));
        assert_eq!(md5_le(&dst), md5, "{label}: develop_with_into");

        let n = scratch_len(&sensor, &opts).unwrap();
        scratch.resize(n, 0);
        dst.fill(SENTINEL);
        develop_with_scratch_into(&sensor, &src, &mut dst, &mut scratch, &opts)
            .unwrap_or_else(|e| panic!("{label}: scratch: {e}"));
        assert_eq!(md5_le(&dst), md5, "{label}: develop_with_scratch_into");
    }
}
