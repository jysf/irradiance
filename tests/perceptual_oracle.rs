//! `SPEC-020` — the develop-layer oracle vs `dnglab analyze --srgb`, scored
//! with SSIMULACRA2 (`DEC-005`).
//!
//! ## ⚠ The one idea this file turns on
//!
//! **The oracle lands before the develop pipeline exists.** `SPEC-018`
//! (`WarpRectilinear`) and `SPEC-019` (tone curve) depend on this spec
//! shipping first, precisely so their own tests cannot be written by the
//! session that also writes the fault (`## The design decision this spec
//! rests on`, `SPEC-020.md`). That forces the central constraint on this
//! file: **the tier-A red-proof cannot score a real render**, because there
//! is no develop pipeline yet. Instead it perturbs a synthetic reference and
//! scores the perturbed copy against the original, exercising the exact
//! metric wiring (`tests/support/ssimulacra2.rs`) a real render will later
//! go through. The fault set is the one `DEC-005` calibrated:
//!
//! - identical scores ≥ 99.9 (sanity anchor; the dedicated conversion-chain
//!   test lives in `tests/support/ssimulacra2.rs`, `AC3`)
//! - a 1-pixel horizontal shift scores < 85 (`AC4`)
//! - `SPIKE-001`'s missing radial warp scores < 85 (`AC5`)
//! - gamma 1.05 (a legitimate tone-curve difference) scores ≥ 85 — the
//!   falsifier's mirror: a bar so tight this passes would train us to ignore
//!   the oracle (`AC6`)
//!
//! ## The fixture — synthetic, not a licensed photograph
//!
//! `SPEC-020`'s `## Implementation Context` pre-registers two candidate
//! fixture shapes and prefers a real photograph IF a licence-clean source is
//! available. None was available inside this environment without either
//! sourcing a third-party image (a provenance/licence question this spec
//! should not have to answer) or decoding one via the forbidden `::image`
//! crate (`library-not-application`) — so this build uses candidate 2: a
//! synthetic gradient-plus-texture field, generated in-code so the exact
//! bytes are reviewable as source rather than trusted as an opaque binary
//! blob, matching the precedent `tests/develop_oracle.rs`'s
//! production-scale fixture (`FU-10`) already set for THIS repo (an in-test
//! synthetic image, not a committed one).
//!
//! The field is fractional Brownian motion (multi-octave hash-based value
//! noise) at `BASE_FREQ = 0.02`, `OCTAVES = 5`, `CONTRAST = 1.6`, seed
//! `1234`, over a `1024x768` plane — parameters probed empirically (a
//! standalone scratch harness against the real `ssimulacra2` crate, see the
//! handback) against the pre-registered rule in `SPEC-020`: the fixture must
//! satisfy ALL FOUR invariants above, or the fixture is wrong, not the
//! threshold. The chosen parameters, generation code and measured scores are
//! recorded together in `tests/oracle-fixtures/perceptual-oracle-fixture.md`
//! so a future reviewer can reproduce the exact numbers below.
//!
//! ## Tiers
//!
//! - **Tier A** (every `#[test]` below except the one named `_smoke`) needs
//!   neither the corpus nor `dnglab` on `PATH` — the whole point, since this
//!   oracle's proof-of-teeth cannot depend on a develop pipeline that does
//!   not exist yet (`AC7`).
//! - **Tier B** (`dnglab_srgb_reader_parses_a_real_corpus_file`) is
//!   informational only and skips loudly when the corpus or `dnglab` is
//!   absent. It deliberately does NOT run a full-resolution SSIMULACRA2
//!   self-score: measured at 8368x5584 (a real Q2M frame) that call alone
//!   costs ~9.3s in `--release` (`examples/real_timing.rs`-shaped probe,
//!   recorded in the handback) — multiple times that in the `cargo test`
//!   debug profile `just test` actually runs under. A smoke test that makes
//!   every local `just test` run pay nearly a minute, only when the corpus
//!   happens to be present, was judged not worth it for an informational,
//!   non-gating check (`SPEC-020`'s `## Failing Tests`, "does NOT gate
//!   ship"). What this test DOES check — the reader parses dnglab's real
//!   defect shape, not just a hand-built one — costs milliseconds.
//!
//! No fuzz target: this file adds no library-side input surface. The PNM
//! reader parses test-side synthetic fixtures and dnglab's own output in
//! tests only (`SPEC-020/## Non-Goals`; `AGENTS.md` §12 bar 2 does not fire).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/ssimulacra2.rs"]
mod metric;
#[path = "support/perturb.rs"]
mod perturb;
#[path = "support/pnm.rs"]
mod pnm;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const BASE_FREQ: f32 = 0.02;
const OCTAVES: u32 = 5;
const CONTRAST: f32 = 1.6;
const NOISE_SEED: u32 = 1234;

/// `DEC-005`'s pre-registered develop-oracle tolerance.
const THRESHOLD: f64 = 85.0;

// ─────────────────────────────────────────────────────────────────────────────
// The synthetic fixture: hash-based multi-octave value noise (fBm)
// ─────────────────────────────────────────────────────────────────────────────

/// A cheap, deterministic hash of `(x, y, seed)` into `[0.0, 1.0)` —
/// splitmix64-shaped mixing, chosen only for speed and a good avalanche
/// property; this is original scaffolding code for generating a test input,
/// not a decoder or an algorithm this project ships (`docs/provenance-
/// ledger.md` tracks decoders and metrics, not synthetic test fixtures).
fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h: u64 = (x as i64 as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((y as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
        .wrapping_add(u64::from(seed).wrapping_mul(0x1656_67B1_9E37_79F9));
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    ((h >> 40) as f32) / ((1u64 << 24) as f32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Bilinearly-interpolated value noise on the integer lattice, smoothed with
/// `smoothstep` at the cell boundaries.
fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (xi, yi) = (x0 as i32, y0 as i32);
    let tx = smoothstep(x - x0);
    let ty = smoothstep(y - y0);
    let n00 = hash2(xi, yi, seed);
    let n10 = hash2(xi + 1, yi, seed);
    let n01 = hash2(xi, yi + 1, seed);
    let n11 = hash2(xi + 1, yi + 1, seed);
    let nx0 = n00 + (n10 - n00) * tx;
    let nx1 = n01 + (n11 - n01) * tx;
    nx0 + (nx1 - nx0) * ty
}

/// Fractional Brownian motion: `octaves` layers of [`value_noise`] at
/// doubling frequency and halving amplitude, normalised back to `[0, 1]`.
fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let (mut amp, mut freq, mut total, mut max) = (0.5f32, 1.0f32, 0.0f32, 0.0f32);
    for o in 0..octaves {
        total += value_noise(x * freq, y * freq, seed.wrapping_add(o * 977)) * amp;
        max += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    total / max
}

/// The pre-registered tier-A synthetic reference: `WIDTH x HEIGHT`, fBm
/// texture at `BASE_FREQ`/`OCTAVES`, contrast-stretched around the midpoint
/// and quantised to 16-bit. Deterministic — same seed, same output, every
/// run (verified in the handback's determinism check).
fn synthetic_reference() -> Vec<u16> {
    let mut out = vec![0u16; WIDTH as usize * HEIGHT as usize];
    for y in 0..HEIGHT as usize {
        for x in 0..WIDTH as usize {
            let n = fbm(
                x as f32 * BASE_FREQ,
                y as f32 * BASE_FREQ,
                NOISE_SEED,
                OCTAVES,
            );
            let centered = (n - 0.5) * CONTRAST + 0.5;
            let v = centered.clamp(0.0, 1.0);
            out[y * WIDTH as usize + x] = (v * 65_535.0).round() as u16;
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 (this fixture specifically) — identical scores >= 99.9
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn synthetic_reference_scores_at_least_ninetynine_point_nine_against_itself() {
    let reference = synthetic_reference();
    let s = metric::score(WIDTH, HEIGHT, &reference, &reference).expect("scores");
    eprintln!("AC3 identical (this fixture): {s:.3}");
    assert!(s >= 99.9, "identical fixture scored {s}, expected >= 99.9");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 + AC7 — a 1-pixel shift must score below the DEC-005 threshold
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn one_pixel_shift_scores_below_the_threshold() {
    let reference = synthetic_reference();
    let shifted = perturb::shift_horizontal(WIDTH, HEIGHT, &reference, 1);
    let s = metric::score(WIDTH, HEIGHT, &reference, &shifted).expect("scores");
    eprintln!("AC4 one-pixel shift: {s:.3}");
    assert!(
        s < THRESHOLD,
        "1-pixel shift scored {s}, expected < {THRESHOLD}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 + AC7 — a missing radial warp must score below the DEC-005 threshold
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn missing_warp_scores_below_the_threshold() {
    let reference = synthetic_reference();
    let warped = perturb::apply_missing_radial_warp(WIDTH, HEIGHT, &reference);
    let s = metric::score(WIDTH, HEIGHT, &reference, &warped).expect("scores");
    eprintln!("AC5 missing warp: {s:.3}");
    assert!(
        s < THRESHOLD,
        "missing warp scored {s}, expected < {THRESHOLD}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 + AC7 — gamma 1.05 must be ADMITTED (score at/above the threshold)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn gamma_one_point_zero_five_scores_at_or_above_the_threshold() {
    let reference = synthetic_reference();
    let gamma = perturb::apply_linear_gamma(&reference, 1.05);
    let s = metric::score(WIDTH, HEIGHT, &reference, &gamma).expect("scores");
    eprintln!("AC6 gamma 1.05: {s:.3}");
    assert!(
        s >= THRESHOLD,
        "gamma 1.05 scored {s}, expected >= {THRESHOLD}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B (informational, does NOT gate ship) — the reader against a real file
// ─────────────────────────────────────────────────────────────────────────────

/// One decodable Q2M file, reused from `tests/develop_oracle.rs`'s
/// `DECODABLE` list.
const SMOKE_FILE: &str = "LEICA-Q2-MONO/L1021223.DNG";

#[test]
fn dnglab_srgb_reader_parses_a_real_corpus_file() {
    let manifest = corpus::Manifest::load().expect("manifest parses");
    let root = corpus::CorpusRoot::resolve();
    let entry = manifest
        .get(SMOKE_FILE)
        .unwrap_or_else(|| panic!("{SMOKE_FILE} must be in the manifest"));
    let Some(path) = entry.require(&root) else {
        // `require()` already announced the skip, naming the missing file
        // (AGENTS.md §12 bar 4).
        return;
    };

    let dnglab = match std::process::Command::new("dnglab")
        .args(["analyze", "--srgb"])
        .arg(&path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "SKIP dnglab_srgb_reader_parses_a_real_corpus_file — dnglab: could not run ({e}) \
                 — is it on PATH?"
            );
            return;
        }
    };
    if !dnglab.status.success() {
        eprintln!(
            "SKIP dnglab_srgb_reader_parses_a_real_corpus_file — dnglab exited {:?}: {}",
            dnglab.status.code(),
            String::from_utf8_lossy(&dnglab.stderr)
        );
        return;
    }

    let plane = pnm::read_dnglab_srgb(&dnglab.stdout).unwrap_or_else(|e| {
        panic!("dnglab's real --srgb output on {SMOKE_FILE} must parse under DEC-005's workaround: {e}")
    });
    assert_eq!(
        plane.samples.len(),
        plane.width as usize * plane.height as usize,
        "decoded sample count must equal width * height"
    );
}
