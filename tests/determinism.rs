//! `SPEC-041` — cross-arch determinism evidence over tier-A synthetic Bayer.
//!
//! Each fixture below is a hand-built Bayer DNG (tier A: own work, built from
//! the `tests/support/tiff.rs` primitives, never a held corpus file —
//! `DEC-003`). The test runs the three public outputs a caller sees —
//! [`unpack_into`], [`demosaic_into`] and [`to_linear_f32_into`] — over each
//! fixture, hashes every output's raw bytes with a small in-test FNV-1a, and
//! asserts each hash against a committed constant.
//!
//! On its own a committed hash only proves "unchanged on this machine". The
//! evidence is the CI `determinism` job, which runs this file on
//! `ubuntu-latest` (x86_64) **and** `macos-latest` (arm64): both arches must
//! reproduce the same committed constants, so a green matrix is a measured
//! cross-arch agreement, not an argument from the maths.
//!
//! Why the outputs should agree (`docs/determinism.md` names every path):
//! `unpack_into` and `demosaic_into` are integer arithmetic only; the one float
//! op, `to_linear_f32_into`, is a single correctly-rounded IEEE `f32` division.
//! The `f64` colour path (`src/color.rs`) was named there as unmeasured until
//! `SPEC-091` added the colour fixture below: the sensor-geometry linear-sRGB
//! plane (`srgb_linear_plane_into` / `_f32_into`, `DEC-079`), whose `u16` pixels
//! are `develop_color_into`'s, now has committed hashes on both arches.
//!
//! If a hash ever disagrees between arches, that is a finding for a NEW spec —
//! do not edit a kernel (or a constant) to force agreement (`SPEC-041`
//! Non-Goals).

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::color::{
    mono_mix_plane_into, mono_mix_row, srgb_linear_plane_f32_into, srgb_linear_plane_into,
    MONO_MIX_VERSION,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{develop_with_into, output_dimensions_with, DevelopOptions};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::plane::unpack_into;
use irradiance::tone::to_linear_f32_into;
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// The hash — 64-bit FNV-1a, a few lines, no dependency
// ─────────────────────────────────────────────────────────────────────────────

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 64-bit FNV-1a over `bytes`.
fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash a `u16` buffer as its little-endian bytes, so the hash names the
/// values, not the host's memory layout.
fn hash_u16(samples: &[u16]) -> u64 {
    fnv1a(samples.iter().flat_map(|s| s.to_le_bytes()))
}

/// Hash an `f32` buffer as each value's little-endian IEEE bytes — a bit-exact
/// comparison, so `-0.0`/`0.0` or a one-ulp difference changes the hash.
fn hash_f32(values: &[f32]) -> u64 {
    fnv1a(values.iter().flat_map(|v| v.to_le_bytes()))
}

/// The hash itself is pinned to the published FNV-1a 64 test vectors, so a
/// broken hash cannot make every fixture "agree" trivially.
#[test]
fn fnv1a_matches_reference_vectors() {
    assert_eq!(fnv1a(*b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a(*b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fnv1a(*b"foobar"), 0x8594_4171_f739_67e8);
}

// ─────────────────────────────────────────────────────────────────────────────
// The tier-A fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// 24 x 16: large enough that `demosaic_into` runs both its interior fast path
/// (every pixel two or more from each edge) and its reflecting border path.
const WIDTH: u32 = 24;
const HEIGHT: u32 = 16;
const STRIP_OFFSET: u32 = 512;

/// One synthetic Bayer plane: a CFA phase, a bit depth and a container byte
/// order. The four fixtures cover all four phases of the RGGB family and all
/// four bit depths `DEC-008` supports, each with a different output.
#[derive(Clone, Copy)]
struct Fixture {
    name: &'static str,
    /// `CFAPattern` colour indices, row-major (0 = R, 1 = G, 2 = B).
    pattern: [u8; 4],
    bits: u16,
    order: Order,
    seed: u32,
}

const FIXTURES: [Fixture; 4] = [
    Fixture {
        name: "rggb-16bit-le",
        pattern: [0, 1, 1, 2],
        bits: 16,
        order: Order::Little,
        seed: 0x0416_0001,
    },
    Fixture {
        name: "bggr-14bit-be",
        pattern: [2, 1, 1, 0],
        bits: 14,
        order: Order::Big,
        seed: 0x0414_0002,
    },
    Fixture {
        name: "grbg-12bit-le",
        pattern: [1, 0, 2, 1],
        bits: 12,
        order: Order::Little,
        seed: 0x0412_0003,
    },
    Fixture {
        name: "gbrg-8bit-be",
        pattern: [1, 2, 0, 1],
        bits: 8,
        order: Order::Big,
        seed: 0x0408_0004,
    },
];

/// xorshift32 — an integer-only generator, so the fixture samples are the
/// same on every arch by construction.
fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

impl Fixture {
    /// The plane's samples: a horizontal ramp plus noise, masked to the bit
    /// depth, with the first two samples pinned to 0 and full scale so the
    /// extremes (and demosaic's clamping) are exercised.
    fn samples(&self) -> Vec<u16> {
        let max = (1u32 << self.bits) - 1;
        let mut state = self.seed;
        let mut out = Vec::with_capacity((WIDTH * HEIGHT) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let ramp = (x * max) / (WIDTH - 1);
                let noise = xorshift32(&mut state) & (max >> 2);
                let value = (ramp + noise + y) & max;
                out.push(value as u16);
            }
        }
        out[0] = 0;
        out[1] = max as u16;
        out
    }

    /// The strip's wire form: 8/16-bit byte-aligned (16-bit in the
    /// container's byte order), 12/14-bit packed MSB-first with the final
    /// byte zero-padded — the two paths of `DEC-008`.
    fn strip(&self, samples: &[u16]) -> Vec<u8> {
        match self.bits {
            8 => samples.iter().map(|&s| s as u8).collect(),
            16 => samples
                .iter()
                .flat_map(|&s| match self.order {
                    Order::Little => s.to_le_bytes(),
                    Order::Big => s.to_be_bytes(),
                })
                .collect(),
            bits => pack_msb(samples, u32::from(bits)),
        }
    }

    /// A minimal Bayer DNG — IFD0 is the CFA sensor plane, one strip at
    /// [`STRIP_OFFSET`] holding `strip`.
    fn dng(&self, strip: &[u8]) -> Vec<u8> {
        let order = self.order;
        let entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, WIDTH),
            tiff::long(tiff::IMAGE_LENGTH, HEIGHT),
            tiff::short(tiff::BITS_PER_SAMPLE, self.bits, order),
            tiff::short(tiff::COMPRESSION, 1, order),
            tiff::short(tiff::PHOTOMETRIC, tiff::CFA_PHOTOMETRIC, order),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
            tiff::long(tiff::STRIP_OFFSETS, STRIP_OFFSET),
            tiff::long(tiff::STRIP_BYTE_COUNTS, strip.len() as u32),
            tiff::two_shorts(tiff::CFA_REPEAT_PATTERN_DIM, 2, 2, order),
            tiff::inline_bytes(tiff::CFA_PATTERN, self.pattern, 4, order),
            tiff::inline_bytes(tiff::CFA_PLANE_COLOR, [0, 1, 2, 0], 3, order),
            tiff::short(tiff::CFA_LAYOUT, 1, order),
        ];
        let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
        let at = STRIP_OFFSET as usize;
        let end = at + strip.len();
        if data.len() < end {
            data.resize(end, 0);
        }
        data[at..end].copy_from_slice(strip);
        data
    }

    /// The fixture's container bytes.
    fn file(&self) -> Vec<u8> {
        self.dng(&self.strip(&self.samples()))
    }
}

/// Pack `samples` MSB-first at `bits` per sample, zero-padding the final byte
/// — the inverse of `irradiance::plane`'s bit reader (the same helper
/// `tests/plane_unpack.rs` round-trips against measured strip bytes).
fn pack_msb(samples: &[u16], bits: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut nbits: u32 = 0;
    let mask = (1u32 << bits) - 1;
    for &s in samples {
        acc = (acc << bits) | (u32::from(s) & mask);
        nbits += bits;
        while nbits >= 8 {
            nbits -= 8;
            out.push(((acc >> nbits) & 0xFF) as u8);
        }
    }
    if nbits > 0 {
        out.push(((acc << (8 - nbits)) & 0xFF) as u8);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// The pipeline under test
// ─────────────────────────────────────────────────────────────────────────────

/// The three output hashes of one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hashes {
    unpack: u64,
    demosaic: u64,
    linear_f32: u64,
}

/// Parse `file`, then `unpack_into → demosaic_into → to_linear_f32_into`,
/// hashing each output. `to_linear_f32_into` takes the demosaiced RGB, so the
/// float hash covers every sample the integer stages produced.
fn run(file: &[u8]) -> Hashes {
    let container = Container::parse(file).expect("tier-A fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor plane");
    let pixels = (WIDTH * HEIGHT) as usize;

    let mut plane = vec![0u16; pixels];
    unpack_into(&sensor, container.byte_order(), file, &mut plane).expect("fixture unpacks");

    let mut rgb = vec![0u16; pixels * 3];
    demosaic_into(&sensor, &plane, &mut rgb).expect("fixture demosaics");

    let mut linear = vec![0f32; pixels * 3];
    to_linear_f32_into(&rgb, &mut linear).expect("lengths match");

    Hashes {
        unpack: hash_u16(&plane),
        demosaic: hash_u16(&rgb),
        linear_f32: hash_f32(&linear),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The committed hashes — one per (fixture, output), in `FIXTURES` order.
// Recorded on macOS arm64 at build; the CI matrix asserts them on
// ubuntu-latest x86_64 too. Never edit one to make a run pass.
// ─────────────────────────────────────────────────────────────────────────────

const COMMITTED: [Hashes; 4] = [
    // rggb-16bit-le
    Hashes {
        unpack: 0x4d7c_1291_f554_eda6,
        demosaic: 0x6988_e05f_c102_a8ca,
        linear_f32: 0xa6f1_9f47_9f6b_3a23,
    },
    // bggr-14bit-be
    Hashes {
        unpack: 0x5aa5_a38e_b28e_4245,
        demosaic: 0xa577_9d99_e5e8_d63c,
        linear_f32: 0x57f4_8eb5_606c_16b2,
    },
    // grbg-12bit-le
    Hashes {
        unpack: 0x927d_55b8_3d73_9c0f,
        demosaic: 0xda70_5342_ec01_5066,
        linear_f32: 0xb257_53cb_773e_bc9a,
    },
    // gbrg-8bit-be
    Hashes {
        unpack: 0x9fbc_02c3_a3cb_1322,
        demosaic: 0xda25_03b1_b277_71d9,
        linear_f32: 0x63cb_2357_3b0b_3fff,
    },
];

/// Every fixture's three hashes, computed once per test.
fn actual() -> Vec<(Fixture, Hashes)> {
    FIXTURES.iter().map(|f| (*f, run(&f.file()))).collect()
}

/// Print every actual hash before asserting, so a failing run (on either arch)
/// shows the whole picture rather than the first mismatch only.
fn report(actual: &[(Fixture, Hashes)]) {
    for (fixture, h) in actual {
        eprintln!(
            "{:<14} unpack {:#018x} demosaic {:#018x} linear_f32 {:#018x}",
            fixture.name, h.unpack, h.demosaic, h.linear_f32
        );
    }
}

/// Independent of the committed constants: `unpack_into` reproduces exactly
/// the samples the fixture was built from, so the unpack hash pins a correct
/// plane, not merely a stable one.
#[test]
fn unpack_round_trips_the_fixture_samples() {
    for fixture in &FIXTURES {
        let file = fixture.file();
        let container = Container::parse(&file).expect("tier-A fixture parses");
        let sensor = container.sensor().expect("fixture has a sensor plane");
        let mut plane = vec![0u16; (WIDTH * HEIGHT) as usize];
        unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("fixture unpacks");
        assert_eq!(
            plane,
            fixture.samples(),
            "{}: unpack round-trip",
            fixture.name
        );
    }
}

#[test]
fn fixtures_are_distinct() {
    // Each fixture must contribute its own evidence: no two share an output.
    let actual = actual();
    for (i, (a, ha)) in actual.iter().enumerate() {
        for (b, hb) in actual.iter().skip(i + 1) {
            assert_ne!(ha.unpack, hb.unpack, "{} vs {}: unpack", a.name, b.name);
            assert_ne!(
                ha.demosaic, hb.demosaic,
                "{} vs {}: demosaic",
                a.name, b.name
            );
            assert_ne!(
                ha.linear_f32, hb.linear_f32,
                "{} vs {}: linear_f32",
                a.name, b.name
            );
        }
    }
}

#[test]
fn unpack_into_hash_matches_committed() {
    let actual = actual();
    report(&actual);
    for ((fixture, h), committed) in actual.iter().zip(COMMITTED.iter()) {
        assert_eq!(
            h.unpack, committed.unpack,
            "{}: unpack_into hash diverged from the committed constant",
            fixture.name
        );
    }
}

#[test]
fn demosaic_into_hash_matches_committed() {
    let actual = actual();
    report(&actual);
    for ((fixture, h), committed) in actual.iter().zip(COMMITTED.iter()) {
        assert_eq!(
            h.demosaic, committed.demosaic,
            "{}: demosaic_into hash diverged from the committed constant",
            fixture.name
        );
    }
}

#[test]
fn to_linear_f32_into_hash_matches_committed() {
    let actual = actual();
    report(&actual);
    for ((fixture, h), committed) in actual.iter().zip(COMMITTED.iter()) {
        assert_eq!(
            h.linear_f32, committed.linear_f32,
            "{}: to_linear_f32_into hash diverged from the committed constant",
            fixture.name
        );
    }
}

/// The oracle shown red (`oracle-must-be-shown-red`, `DEC-009`).
///
/// Control first: each unperturbed fixture reproduces its committed hashes
/// through this very code path. Then one bit of one sample is flipped — the
/// LSB of an interior pixel, the smallest possible change — and all three
/// hashes must move off the committed constant, so the assertions above
/// would fail on it. A hash that did not move would mean the oracle cannot see
/// a one-count difference, which is the whole claim.
#[test]
fn determinism_red_proof() {
    // An interior pixel (row 8, column 10): demosaic reads it on the fast path.
    let target = (8 * WIDTH + 10) as usize;
    for (fixture, committed) in FIXTURES.iter().zip(COMMITTED.iter()) {
        let samples = fixture.samples();

        // Control: the unperturbed fixture matches.
        let control = run(&fixture.dng(&fixture.strip(&samples)));
        assert_eq!(
            control, *committed,
            "{}: control run must reproduce the committed hashes before the red-proof means anything",
            fixture.name
        );

        // Perturbation: flip the sample's least-significant bit.
        let mut perturbed = samples.clone();
        perturbed[target] ^= 1;
        let red = run(&fixture.dng(&fixture.strip(&perturbed)));
        assert_ne!(
            red.unpack, committed.unpack,
            "{}: unpack hash blind to a one-bit change",
            fixture.name
        );
        assert_ne!(
            red.demosaic, committed.demosaic,
            "{}: demosaic hash blind to a one-bit change",
            fixture.name
        );
        assert_ne!(
            red.linear_f32, committed.linear_f32,
            "{}: linear_f32 hash blind to a one-bit change",
            fixture.name
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The colour fixture (`SPEC-091` AC12): the sensor-geometry linear-sRGB plane
// ─────────────────────────────────────────────────────────────────────────────

/// A separate fixture, so no byte any fixture above hashes changes. A 24 × 16
/// RGGB plane at 16 bits carrying `AsShotNeutral`, `ColorMatrix1/2`,
/// `ForwardMatrix1/2` and two `CalibrationIlluminant`s (A and D65), so the
/// matrix build runs `DEC-076`'s convergent illuminant interpolation, the
/// path with the most `f64` operations. A strong white balance and a
/// near-full-scale band give the `f32` plane negative and above-1.0 values, so
/// its hash covers the unclamped path, not only values a clamp would also give.
fn colour_samples() -> Vec<u16> {
    let mut state = 0x5091_0c01;
    (0..WIDTH * HEIGHT)
        .map(|i| {
            let n = xorshift32(&mut state);
            if i % WIDTH >= 18 {
                (65535 - (n & 0x7ff)) as u16
            } else {
                (n & 0xffff) as u16
            }
        })
        .collect()
}

/// `num / 10000` SRATIONALs, row-major.
fn matrix(rows: [[i32; 3]; 3]) -> Vec<(i32, i32)> {
    rows.iter()
        .flat_map(|row| row.iter().map(|&num| (num, 10_000)))
        .collect()
}

fn colour_description(samples: Vec<u16>) -> DngDescription {
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: WIDTH,
        height: HEIGHT,
        bits_per_sample: 16,
        kind: PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        },
        organisation: Organisation::Strips {
            rows_per_strip: HEIGHT,
        },
        payload: Payload::Samples(samples),
        meta: DngMeta {
            color_matrix_1: Some(matrix([
                [8000, -2000, -500],
                [-4000, 12000, 2000],
                [-1000, 2500, 7000],
            ])),
            color_matrix_2: Some(matrix([
                [7000, -1500, -500],
                [-4500, 12500, 2000],
                [-800, 2000, 6000],
            ])),
            forward_matrix_1: Some(matrix([
                [6000, 3000, 642],
                [2000, 7000, 1000],
                [1000, 1000, 6249],
            ])),
            forward_matrix_2: Some(matrix([
                [6500, 2500, 642],
                [2500, 6800, 700],
                [500, 1200, 6549],
            ])),
            calibration_illuminant_1: Some(17),
            calibration_illuminant_2: Some(21),
            as_shot_neutral: Some(vec![(9, 20), (1, 1), (31, 50)]),
            ..DngMeta::default()
        },
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// The two plane hashes of one colour description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlaneHashes {
    u16_plane: u64,
    f32_plane: u64,
}

/// The fixture's `u16` and `f32` planes.
fn colour_planes(d: &DngDescription) -> (Vec<u16>, Vec<f32>) {
    let file = write_dng(d).expect("the colour fixture writes");
    let container = Container::parse(&file).expect("the colour fixture parses");
    let sensor = container.sensor().expect("the colour fixture has a sensor");
    let pixels = (WIDTH * HEIGHT) as usize;
    let mut plane = vec![0u16; pixels];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpacks");
    let mut u = vec![0u16; pixels * 3];
    srgb_linear_plane_into(&sensor, &plane, &mut u).expect("the u16 plane develops");
    let mut f = vec![0f32; pixels * 3];
    srgb_linear_plane_f32_into(&sensor, &plane, &mut f).expect("the f32 plane develops");
    (u, f)
}

fn colour_hashes(d: &DngDescription) -> PlaneHashes {
    let (u, f) = colour_planes(d);
    PlaneHashes {
        u16_plane: hash_u16(&u),
        f32_plane: hash_f32(&f),
    }
}

/// Recorded on macOS arm64 at `SPEC-091`'s build; the CI matrix asserts them on
/// ubuntu-latest x86_64 too. Never edit one to make a run pass.
const COLOUR_COMMITTED: PlaneHashes = PlaneHashes {
    u16_plane: 0x03c3_5911_9395_8585,
    f32_plane: 0x419a_f4d1_3262_5320,
};

#[test]
fn colour_plane_hashes_match_committed() {
    let d = colour_description(colour_samples());
    let (_, f) = colour_planes(&d);
    let actual = colour_hashes(&d);
    eprintln!(
        "colour-rggb-16bit u16_plane {:#018x} f32_plane {:#018x}",
        actual.u16_plane, actual.f32_plane
    );
    // The hash covers the unclamped path: the plane holds both a negative and
    // an above-1.0 value.
    assert!(f.iter().any(|&v| v < 0.0), "the f32 plane holds a negative");
    assert!(
        f.iter().any(|&v| v > 1.0),
        "the f32 plane holds a value above 1.0"
    );
    // The second calibration takes part: without CalibrationIlluminant2 the
    // single-illuminant path runs and the plane differs.
    let mut single = d.clone();
    single.meta.calibration_illuminant_2 = None;
    assert_ne!(
        colour_hashes(&single).f32_plane,
        actual.f32_plane,
        "the illuminant interpolation changes the plane"
    );
    assert_eq!(
        actual.u16_plane, COLOUR_COMMITTED.u16_plane,
        "srgb_linear_plane_into hash diverged from the committed constant"
    );
    assert_eq!(
        actual.f32_plane, COLOUR_COMMITTED.f32_plane,
        "srgb_linear_plane_f32_into hash diverged from the committed constant"
    );
}

/// `determinism_red_proof` for the colour fixture: the control reproduces the
/// committed hashes, then the LSB of one interior sample is flipped and both
/// plane hashes must move.
#[test]
fn colour_determinism_red_proof() {
    let samples = colour_samples();
    let control = colour_hashes(&colour_description(samples.clone()));
    assert_eq!(
        control, COLOUR_COMMITTED,
        "control run must reproduce the committed colour hashes before the red-proof means anything"
    );
    let target = (8 * WIDTH + 10) as usize;
    let mut perturbed = samples;
    perturbed[target] ^= 1;
    let red = colour_hashes(&colour_description(perturbed));
    assert_ne!(
        red.u16_plane, COLOUR_COMMITTED.u16_plane,
        "u16 plane hash blind to a one-bit change"
    );
    assert_ne!(
        red.f32_plane, COLOUR_COMMITTED.f32_plane,
        "f32 plane hash blind to a one-bit change"
    );

    // SPEC-092 AC12: the mixer's hashes. The control reproduces them; the same
    // one-bit sample change moves both mono plane hashes, and a one-unit change
    // of one AsShotNeutral numerator moves both row hashes.
    let d = colour_description(colour_samples());
    let control = mono_hashes(&d);
    assert_eq!(
        control, MONO_COMMITTED,
        "control run must reproduce the committed mixer hashes before the red-proof means anything"
    );
    let mut flipped = colour_samples();
    flipped[target] ^= 1;
    let red = mono_hashes(&colour_description(flipped));
    for i in 0..2 {
        assert_ne!(
            red.planes[i], MONO_COMMITTED.planes[i],
            "mono plane hash {i} blind to a one-bit change"
        );
        assert_eq!(
            red.rows[i], MONO_COMMITTED.rows[i],
            "a sample does not move a row"
        );
    }
    let mut nudged = d.clone();
    nudged.meta.as_shot_neutral = Some(vec![(10, 20), (1, 1), (31, 50)]);
    let red = mono_hashes(&nudged);
    for i in 0..2 {
        assert_ne!(
            red.rows[i], MONO_COMMITTED.rows[i],
            "row hash {i} blind to a one-unit AsShotNeutral change"
        );
    }
    // SPEC-093 AC11: the developed mix's four hashes.
    developed_mix_red_proof();
}

// ─────────────────────────────────────────────────────────────────────────────
// The mono mixer on the colour fixture (`SPEC-092` AC12, `DEC-083`)
// ─────────────────────────────────────────────────────────────────────────────

const fn rational(num: i32, den: i32) -> Rational {
    Rational { num, den }
}

/// Rec. 709 luminance on linear sRGB.
const LUMINANCE: [Rational; 3] = [
    rational(2126, 10000),
    rational(7152, 10000),
    rational(722, 10000),
];

/// The one-hot weight sets, in order; the fixture uses the first whose row
/// has a negative entry (asserted below; it is the red set, index 0).
const ONE_HOT: [[Rational; 3]; 3] = [
    [rational(1, 1), rational(0, 1), rational(0, 1)],
    [rational(0, 1), rational(1, 1), rational(0, 1)],
    [rational(0, 1), rational(0, 1), rational(1, 1)],
];

/// The one-hot set the mixer hashes use (recorded at SPEC-092's build).
const NEGATIVE_ONE_HOT: usize = 0;

/// The mixer's hashes on one description: the two rows (12 little-endian
/// bytes each) and the two mono planes, for `[LUMINANCE, ONE_HOT[NEGATIVE_ONE_HOT]]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MonoHashes {
    rows: [u64; 2],
    planes: [u64; 2],
}

fn colour_sensor(d: &DngDescription) -> (Sensor, Vec<u16>) {
    let file = write_dng(d).expect("the colour fixture writes");
    let container = Container::parse(&file).expect("the colour fixture parses");
    let sensor = container.sensor().expect("the colour fixture has a sensor");
    let mut plane = vec![0u16; (WIDTH * HEIGHT) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpacks");
    (sensor, plane)
}

/// The rows and the mono planes for the two weight sets.
fn mono_outputs(d: &DngDescription) -> ([[i32; 3]; 2], [Vec<u16>; 2]) {
    let (sensor, plane) = colour_sensor(d);
    let sets = [LUMINANCE, ONE_HOT[NEGATIVE_ONE_HOT]];
    let rows = sets.map(|w| mono_mix_row(&sensor, &w).expect("the row builds"));
    let planes = sets.map(|w| {
        let mut out = vec![0u16; (WIDTH * HEIGHT) as usize];
        mono_mix_plane_into(&sensor, &plane, &w, &mut out).expect("the mono plane mixes");
        out
    });
    (rows, planes)
}

fn mono_hashes(d: &DngDescription) -> MonoHashes {
    let (rows, planes) = mono_outputs(d);
    MonoHashes {
        rows: rows.map(|q| fnv1a(q.iter().flat_map(|v| v.to_le_bytes()))),
        planes: [hash_u16(&planes[0]), hash_u16(&planes[1])],
    }
}

/// Recorded on macOS arm64 at `SPEC-092`'s build; the CI matrix asserts them on
/// ubuntu-latest x86_64 too. Never edit one to make a run pass; a new value
/// needs a `MONO_MIX_VERSION` bump (or a `DEVELOP_VERSION` one, upstream) in
/// the same change.
const MONO_COMMITTED: MonoHashes = MonoHashes {
    rows: [0xe837_0a06_b8c6_e1cb, 0x55b0_e759_ac2e_6883],
    planes: [0x7b07_6b08_b1b5_1de1, 0x0f50_77a8_76c2_fbd3],
};

#[test]
fn mono_mix_hashes_match_committed() {
    // The committed hashes are mixer version 1's.
    assert_eq!(MONO_MIX_VERSION, "1");
    let d = colour_description(colour_samples());
    let (sensor, plane) = colour_sensor(&d);
    // The first one-hot set whose row has a negative entry is the one hashed.
    let first_negative = ONE_HOT.iter().position(|w| {
        mono_mix_row(&sensor, w)
            .expect("row")
            .iter()
            .any(|&q| q < 0)
    });
    assert_eq!(first_negative, Some(NEGATIVE_ONE_HOT));

    let (rows, planes) = mono_outputs(&d);
    // The fade changes at least one pixel: the negative row's plane differs
    // from the plain fold of demosaic_into's triples (the levels here are the
    // identity), computed in i128 here.
    let mut camera = vec![0u16; (WIDTH * HEIGHT * 3) as usize];
    demosaic_into(&sensor, &plane, &mut camera).expect("demosaics");
    let q = rows[1];
    let faded = camera
        .as_chunks::<3>()
        .0
        .iter()
        .zip(&planes[1])
        .filter(|(x, &mono)| {
            let acc: i128 = (0..3).map(|i| i128::from(q[i]) * i128::from(x[i])).sum();
            let plain = (acc + (1 << 23)).div_euclid(1 << 24).clamp(0, 65535);
            plain != i128::from(mono)
        })
        .count();
    assert!(faded > 0, "the fade changes no pixel of the fixture");

    let actual = mono_hashes(&d);
    eprintln!(
        "colour-rggb-16bit mono rows {:?} hashes {:#018x} {:#018x}, planes {:#018x} {:#018x}; fade changes {faded} pixels",
        rows, actual.rows[0], actual.rows[1], actual.planes[0], actual.planes[1]
    );
    assert_eq!(
        actual.rows, MONO_COMMITTED.rows,
        "mono_mix_row hashes diverged from the committed constants"
    );
    assert_eq!(
        actual.planes, MONO_COMMITTED.planes,
        "mono_mix_plane_into hashes diverged from the committed constants"
    );
}

/// `SPEC-091` AC12, verify: the fixture's illuminant weight lies strictly
/// between its two calibrations, so the committed `f32` hash covers the
/// interpolation itself and not a weight clamped to one end. With both
/// calibrations set to the same matrices the weight cannot matter, so the
/// fixture equals one of those two controls exactly when the weight is 0 or 1.
#[test]
fn colour_fixture_interpolates_between_its_illuminants() {
    let d = colour_description(colour_samples());
    let actual = colour_hashes(&d).f32_plane;
    let mut all_first = d.clone();
    all_first.meta.color_matrix_2 = d.meta.color_matrix_1.clone();
    all_first.meta.forward_matrix_2 = d.meta.forward_matrix_1.clone();
    let mut all_second = d.clone();
    all_second.meta.color_matrix_1 = d.meta.color_matrix_2.clone();
    all_second.meta.forward_matrix_1 = d.meta.forward_matrix_2.clone();
    assert_ne!(
        colour_hashes(&all_first).f32_plane,
        actual,
        "the weight is not pinned to CalibrationIlluminant1"
    );
    assert_ne!(
        colour_hashes(&all_second).f32_plane,
        actual,
        "the weight is not pinned to CalibrationIlluminant2"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The developed mix on the colour fixture (`SPEC-093` AC11, `DEC-084`)
// ─────────────────────────────────────────────────────────────────────────────

/// The in-memory warp of the warped pair: an optional (flags 1) non-identity
/// radial `WarpRectilinear`, scale `1 − 0.08·r²`, centre (½, ½). Injected
/// into the parsed sensor's `OpcodeList3`, so the fixture's bytes are
/// unchanged (`SPEC-093` Notes for the Implementer).
const DEVELOPED_WARP_KR: [f64; 4] = [1.0, -0.08, 0.0, 0.0];

/// An `OpcodeList3` holding one optional `WarpRectilinear` with `kr`, built
/// byte by byte (DNG 1.7.0.0 §6.4.1, big-endian).
fn warp_list(kr: [f64; 4]) -> Vec<u8> {
    let mut params = 1u32.to_be_bytes().to_vec();
    for v in kr.into_iter().chain([0.0, 0.0, 0.5, 0.5]) {
        params.extend_from_slice(&v.to_be_bytes());
    }
    let mut list = 1u32.to_be_bytes().to_vec();
    for v in [1u32, 0x0104_0000, 1, params.len() as u32] {
        list.extend_from_slice(&v.to_be_bytes());
    }
    list.extend_from_slice(&params);
    list
}

/// The fixture's sensor for the warped pair: the warp with `kr` and
/// Orientation 6, both in memory.
fn warped(sensor: &Sensor, kr: [f64; 4]) -> Sensor {
    let mut s = sensor.clone();
    s.opcode_lists[2] = true;
    s.opcode_list_3 = Some(warp_list(kr));
    s.orientation = Some(6);
    s
}

/// The developed mix of `sensor` under the default options.
fn developed_mix(sensor: &Sensor, plane: &[u16], weights: [Rational; 3]) -> Vec<u16> {
    let mut o = DevelopOptions::default();
    o.mono_mix = Some(weights);
    let (w, h) = output_dimensions_with(sensor, &o).expect("dims");
    let mut dst = vec![0u16; (w * h) as usize];
    develop_with_into(sensor, plane, &mut dst, &o).expect("the developed mix");
    dst
}

/// The `SPEC-093` §2 oracle: the mixed plane developed as a mono sensor at
/// levels 0 and 65535, without `OpcodeList1` or `OpcodeList2`.
fn developed_oracle(sensor: &Sensor, plane: &[u16], weights: [Rational; 3]) -> Vec<u16> {
    let mut mixed = vec![0u16; plane.len()];
    mono_mix_plane_into(sensor, plane, &weights, &mut mixed).expect("mixed");
    let mut mono = sensor.clone();
    mono.black_level = Some(0);
    mono.black_levels = vec![];
    mono.black_level_repeat_dim = None;
    mono.white_level = Some(65535);
    mono.white_levels = vec![];
    mono.opcode_lists[0] = false;
    mono.opcode_lists[1] = false;
    mono.opcode_list_1 = None;
    mono.opcode_list_2 = None;
    let o = DevelopOptions::default();
    let (w, h) = output_dimensions_with(&mono, &o).expect("dims");
    let mut dst = vec![0u16; (w * h) as usize];
    develop_with_into(&mono, &mixed, &mut dst, &o).expect("the oracle");
    dst
}

/// The four developed-mix hashes: `[unwarped luminance, unwarped negative
/// one-hot, warped luminance, warped negative one-hot]`.
fn developed_hashes(d: &DngDescription, kr: [f64; 4]) -> [u64; 4] {
    let (sensor, plane) = colour_sensor(d);
    let w = warped(&sensor, kr);
    let sets = [LUMINANCE, ONE_HOT[NEGATIVE_ONE_HOT]];
    [
        hash_u16(&developed_mix(&sensor, &plane, sets[0])),
        hash_u16(&developed_mix(&sensor, &plane, sets[1])),
        hash_u16(&developed_mix(&w, &plane, sets[0])),
        hash_u16(&developed_mix(&w, &plane, sets[1])),
    ]
}

/// Recorded on macOS arm64 at `SPEC-093`'s build; the CI matrix asserts them
/// on ubuntu-latest x86_64 too. Never edit one to make a run pass; a new value
/// needs a `DEVELOP_VERSION` or `MONO_MIX_VERSION` bump in the same change.
const DEVELOPED_COMMITTED: [u64; 4] = [
    0x7b07_6b08_b1b5_1de1,
    0x0f50_77a8_76c2_fbd3,
    0x60c5_9f35_c60f_4cc4,
    0xe764_1465_f8f2_cd58,
];

#[test]
fn developed_mix_hashes_match_committed() {
    // Recorded at these versions.
    assert_eq!(MONO_MIX_VERSION, "1");
    assert_eq!(irradiance::DEVELOP_VERSION, "8");
    let d = colour_description(colour_samples());
    let (sensor, plane) = colour_sensor(&d);
    let w = warped(&sensor, DEVELOPED_WARP_KR);
    // Each equals the §2 oracle.
    for weights in [LUMINANCE, ONE_HOT[NEGATIVE_ONE_HOT]] {
        assert_eq!(
            developed_mix(&sensor, &plane, weights),
            developed_oracle(&sensor, &plane, weights)
        );
        assert_eq!(
            developed_mix(&w, &plane, weights),
            developed_oracle(&w, &plane, weights)
        );
    }
    let actual = developed_hashes(&d, DEVELOPED_WARP_KR);
    eprintln!(
        "colour-rggb-16bit developed mix {:#018x} {:#018x} warped {:#018x} {:#018x}",
        actual[0], actual[1], actual[2], actual[3]
    );
    // The fixture has no ActiveArea, DefaultCrop or Orientation, so the
    // unwarped developed mix is the mixed plane itself: its two hashes are
    // SPEC-092's committed plane hashes, a cross-check between the entries.
    assert_eq!(actual[..2], MONO_COMMITTED.planes);
    // The warped pair differs from the unwarped pair.
    assert_ne!(actual[0], actual[2], "the warp moves the luminance mix");
    assert_ne!(actual[1], actual[3], "the warp moves the one-hot mix");
    assert_eq!(
        actual, DEVELOPED_COMMITTED,
        "developed-mix hashes diverged from the committed constants"
    );
}

/// `SPEC-093` AC11's red-proof: the control reproduces the committed hashes,
/// a one-bit sample change moves all four, and a change of `2^-26` in a warp
/// coefficient (the smallest power of two that moves them) moves the warped
/// two and not the unwarped two. Run by `colour_determinism_red_proof`.
fn developed_mix_red_proof() {
    let samples = colour_samples();
    let control = developed_hashes(&colour_description(samples.clone()), DEVELOPED_WARP_KR);
    assert_eq!(
        control, DEVELOPED_COMMITTED,
        "control run must reproduce the committed developed-mix hashes before the red-proof means anything"
    );
    let target = (8 * WIDTH + 10) as usize;
    let mut flipped = samples.clone();
    flipped[target] ^= 1;
    let red = developed_hashes(&colour_description(flipped), DEVELOPED_WARP_KR);
    for i in 0..4 {
        assert_ne!(
            red[i], control[i],
            "developed-mix hash {i} blind to a one-bit change"
        );
    }
    // AC11 asks for a one-ulp change of a warp coefficient. Measured at
    // SPEC-093's build, one ulp of any coefficient moves no output pixel: the
    // warped samples are rounded to u16, and a shift of ~1e-16 px crosses no
    // rounding boundary on 384 pixels. The smallest power-of-two step of kr0
    // that moves both warped hashes is 2^26 ulps (kr0 = 1 + 2^-26), so that
    // is the change here (a deviation recorded in HANDOFF-204).
    let mut kr = DEVELOPED_WARP_KR;
    kr[0] = 1.0 + f64::powi(2.0, -26);
    let red = developed_hashes(&colour_description(samples), kr);
    assert_eq!(
        red[..2],
        control[..2],
        "the warp does not touch the unwarped pair"
    );
    for i in 2..4 {
        assert_ne!(
            red[i], control[i],
            "warped hash {i} blind to a one-ulp coefficient change"
        );
    }
}
