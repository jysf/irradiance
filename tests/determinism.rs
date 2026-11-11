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
//! The `f64` colour path (`src/color.rs`) is named there as unmeasured.
//!
//! If a hash ever disagrees between arches, that is a finding for a NEW spec —
//! do not edit a kernel (or a constant) to force agreement (`SPEC-041`
//! Non-Goals).

#[path = "support/tiff.rs"]
mod tiff;

use irradiance::demosaic::demosaic_into;
use irradiance::ifd::Container;
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
