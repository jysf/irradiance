//! The analytic tone oracle (`SPEC-019`, `AC5`/`AC6`/`AC7`) and its red-proof.
//!
//! This oracle is **independent of `dnglab`/`rawler`** (`DEC-004`'s circularity
//! warning): it derives its expected values arithmetically, by recomputing the
//! sRGB OETF (IEC 61966-2-1) in `f64` for every index and comparing each
//! committed table entry against it. Nothing here reads a decoder's output.
//!
//! # Why ±1, and why in the *test*
//!
//! Cross-platform `powf` can differ in the last bit, so an *exact* re-derivation
//! would be a platform-dependent test (`SPEC-019`'s analytic-oracle note). The
//! committed table (`src/tone_table.rs`, generated once on the maintainer's
//! machine) is the single byte-identical source of truth (`AC7`); this oracle
//! proves it is *correct* (within ±1 of the spec, recomputed in `f64` here) and
//! the red-proof proves a *wrong* table is caught. Same split
//! `DEC-023`/`SPEC-020` used for the perceptual metric.
//!
//! # The table, read through the public API
//!
//! `encode_srgb16_into(&[i], dst)` writes `SRGB_ENCODE_16[i]`, so encoding the
//! whole domain `0..=65535` reconstructs the committed table exactly, through
//! the shipped public path — no `pub(crate)` internal is exposed to do it.
//!
//! # The red-proof (`AC6`) — in-process, on a copy
//!
//! Unlike the plane oracle (`DEC-017`, which must rebuild a mutated *copy of
//! the crate* because its subject is `unpack_into`'s compiled behaviour), this
//! oracle's subject is pure table *data* that already exists in memory. So the
//! red-proof is `DEC-021`'s in-process shape: clone the reconstructed table,
//! perturb one entry by a delta ≫ the ±1 tolerance, and assert the comparison
//! rejects the copy (red) while the unmutated table passes (the negative
//! control, green). The shipped table carries no fault switch and the copy
//! reverts byte-identical.

/// The sRGB OETF (IEC 61966-2-1): scene-linear `L ∈ [0, 1]` → display `V`.
fn srgb_oetf(l: f64) -> f64 {
    if l <= 0.0031308 {
        12.92 * l
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

/// The analytic expected code value for table index `i`, scaled to
/// `full_scale`, round-to-nearest, clamped — recomputed independently in `f64`.
fn analytic_code(i: usize, full_scale: f64) -> i64 {
    let l = i as f64 / 65535.0;
    let v = srgb_oetf(l).clamp(0.0, 1.0);
    (v * full_scale).round() as i64
}

/// Reconstruct `SRGB_ENCODE_16` through the public encoder: `dst[i]` is the
/// committed table entry at index `i`.
fn reconstruct_srgb16() -> Vec<u16> {
    let linear: Vec<u16> = (0..=u16::MAX).collect();
    let mut dst = vec![0u16; linear.len()];
    irradiance::tone::encode_srgb16_into(&linear, &mut dst).unwrap();
    dst
}

/// Reconstruct `SRGB_ENCODE_8` through the public encoder.
fn reconstruct_srgb8() -> Vec<u8> {
    let linear: Vec<u16> = (0..=u16::MAX).collect();
    let mut dst = vec![0u8; linear.len()];
    irradiance::tone::encode_srgb8_into(&linear, &mut dst).unwrap();
    dst
}

/// Count entries that fall outside ±1 of the analytic OETF. This is the oracle;
/// the red-proof runs it against a mutated copy.
fn violations_against_oetf_i64(table: &[i64], full_scale: f64) -> usize {
    table
        .iter()
        .enumerate()
        .filter(|(i, &entry)| (entry - analytic_code(*i, full_scale)).abs() > 1)
        .count()
}

#[test]
fn srgb16_table_matches_the_analytic_oetf_within_one_code_value() {
    // AC5: every one of the 65536 entries is within ±1 of the OETF in f64.
    let table = reconstruct_srgb16();
    assert_eq!(table.len(), 65536);
    let mut checked = 0usize;
    for (i, &entry) in table.iter().enumerate() {
        let expected = analytic_code(i, 65535.0);
        let diff = (i64::from(entry) - expected).abs();
        assert!(
            diff <= 1,
            "SRGB_ENCODE_16[{i}] = {entry}, analytic OETF = {expected} (diff {diff} > 1)"
        );
        checked += 1;
    }
    // Guard against a vacuous zero-iteration pass (AGENTS.md §16 rule 2).
    assert_eq!(checked, 65536, "the oracle must check every table entry");
}

#[test]
fn srgb8_table_matches_the_analytic_oetf_within_one_code_value() {
    // AC5: same, for the 8-bit table.
    let table = reconstruct_srgb8();
    assert_eq!(table.len(), 65536);
    let mut checked = 0usize;
    for (i, &entry) in table.iter().enumerate() {
        let expected = analytic_code(i, 255.0);
        let diff = (i64::from(entry) - expected).abs();
        assert!(
            diff <= 1,
            "SRGB_ENCODE_8[{i}] = {entry}, analytic OETF = {expected} (diff {diff} > 1)"
        );
        checked += 1;
    }
    assert_eq!(checked, 65536, "the oracle must check every table entry");
}

#[test]
fn srgb_tables_pin_the_exact_anchors() {
    // AC2/AC3/AC7: anchors that are byte-identical across platforms by
    // construction — 0 and 65535 are exact, and the breakpoint sits on the
    // linear segment (12.92·L, no powf), so CI (Linux) and a local macOS run
    // assert the same committed bytes.
    let t16 = reconstruct_srgb16();
    let t8 = reconstruct_srgb8();

    assert_eq!(t16[0], 0, "SRGB_ENCODE_16[0]");
    assert_eq!(t16[65535], 65535, "SRGB_ENCODE_16[65535]");
    assert_eq!(t8[0], 0, "SRGB_ENCODE_8[0]");
    assert_eq!(t8[65535], 255, "SRGB_ENCODE_8[65535]");

    // The largest index still on the linear segment (L ≤ 0.0031308) must equal
    // the analytic linear-segment value exactly, scaled and rounded.
    let breakpoint = (0..=65535usize)
        .rfind(|&i| i as f64 / 65535.0 <= 0.0031308)
        .expect("some index is on the linear segment");
    let l = breakpoint as f64 / 65535.0;
    let expected = (12.92 * l * 65535.0).round() as u16;
    assert_eq!(
        t16[breakpoint], expected,
        "breakpoint index {breakpoint} must equal the linear-segment value {expected}"
    );
}

#[test]
fn srgb_tables_are_monotonic_non_decreasing() {
    // AC2/AC3: neither table ever decreases across the domain.
    let t16 = reconstruct_srgb16();
    let t8 = reconstruct_srgb8();
    assert!(
        t16.windows(2).all(|w| w[0] <= w[1]),
        "SRGB_ENCODE_16 must be monotonic non-decreasing"
    );
    assert!(
        t8.windows(2).all(|w| w[0] <= w[1]),
        "SRGB_ENCODE_8 must be monotonic non-decreasing"
    );
}

#[test]
fn analytic_tone_oracle_goes_red_on_a_mutated_table() {
    // AC6 (oracle-must-be-shown-red). Operate on a COPY, DEC-021-style — the
    // shipped table carries no fault switch.
    let honest: Vec<i64> = reconstruct_srgb16().iter().map(|&v| i64::from(v)).collect();

    // Negative control: the honest table passes (green).
    assert_eq!(
        violations_against_oetf_i64(&honest, 65535.0),
        0,
        "negative control: the unmutated table must pass the ±1 oracle"
    );

    // Perturb one entry by +40 (≫ the ±1 tolerance) in a clone.
    let mut mutated = honest.clone();
    let idx = 30_000usize;
    mutated[idx] += 40;
    assert_ne!(
        mutated[idx], honest[idx],
        "the mutation must actually change a value"
    );

    // The oracle must now reject the copy (red).
    let caught = violations_against_oetf_i64(&mutated, 65535.0);
    assert!(
        caught >= 1,
        "the ±1 oracle must reject the mutated table (expected ≥1 violation, got {caught})"
    );

    // Reverts byte-identical: no fault ships.
    mutated[idx] = honest[idx];
    assert_eq!(
        mutated, honest,
        "the copy must revert byte-identical to the committed table"
    );
}
