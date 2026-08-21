//! Write the fuzz seed corpus for `fuzz/fuzz_targets/ifd.rs`.
//!
//! `SPEC-003`. The seeds are the hand-built tier-A fixtures in
//! `tests/support/tiff.rs` — the same list `tests/ifd_reader.rs` asserts
//! against, so a fixture cannot be added to the test lane and forgotten by the
//! fuzz lane. They are **own work built from TIFF 6.0 §2**, not derived from
//! any corpus file: tier B is never committed (`DEC-003`), and a truncated
//! 86 MB Leica frame would still be a Leica frame.
//!
//! ```text
//! cargo run --example fuzz-seeds          # rewrite fuzz/seeds/ifd/
//! ```
//!
//! The output directory is committed, and `fuzz/corpus/ifd/` — where libFuzzer
//! writes what it discovers — is not. Run the target against both:
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
//!     fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60
//! ```

#[path = "../tests/support/tiff.rs"]
mod tiff;

use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("seeds")
        .join("ifd");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("fuzz-seeds: cannot create {}: {e}", dir.display());
        std::process::exit(1);
    }

    let mut written = 0usize;
    for (name, bytes) in tiff::all() {
        let path = dir.join(format!("{name}.tiff"));
        match std::fs::write(&path, &bytes) {
            Ok(()) => {
                println!("{:>6} bytes  {}", bytes.len(), path.display());
                written += 1;
            }
            Err(e) => {
                eprintln!("fuzz-seeds: cannot write {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
    println!("fuzz-seeds: {written} seed(s) in {}", dir.display());
}
