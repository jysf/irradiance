//! `irr` — internal dev/oracle binary for `irradiance`.
//!
//! Constraint `library-not-application` (`guidance/constraints.yaml`): this
//! binary is a development tool only, never a shipped product surface, and
//! the library crate must not grow features around it. It is not part of
//! `irradiance`'s public API.
//!
//! The panic-free lints denied in `src/lib.rs` do not apply here — `irr`
//! runs on developer-controlled input, not attacker-influenced RAW bytes
//! (`guidance/toolchain-brief.md`: "`irr` is a dev/oracle binary, not a
//! product surface. `unwrap()` is fine there.").
//!
//! SPEC-001 scaffolds this as a placeholder only; it grows real subcommands
//! (container dump, tag listing, oracle diff) alongside the specs that add
//! the decoding it would inspect.

fn main() {
    println!(
        "irr: irradiance's internal dev/oracle binary — nothing to run yet (SPEC-001 scaffold)."
    );
}
