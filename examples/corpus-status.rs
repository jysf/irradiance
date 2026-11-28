//! `corpus-status` — prints one line per manifest entry, present or MISSING.
//!
//! **This is the visible surface of SPEC-002's skip.** `just test` runs it
//! before `cargo test`, so an absent tier-B file is named on the terminal and
//! in CI logs with no extra flags.
//!
//! It has to live outside the test harness. Measured at design: `eprintln!`
//! inside a *passing* test is captured — `cargo test` prints 0 SKIP lines and
//! `cargo test -- --nocapture` prints them. A skip nobody can see reports
//! green for work it never did, which is the same defect class as an oracle
//! that cannot go red (`DEC-003`, AGENTS.md §12 bar 4). Making `just test`
//! pass `--nocapture` globally is not the fix: that buries the signal in full
//! test output instead of surfacing it.
//!
//! Exit status is 0 whether or not files are present — absent tier-B is the
//! normal state on CI (`DEC-003`). It exits 1 only if the manifest itself
//! cannot be read, which is a real breakage.
//!
//! Presence only: hashing 600 MB to draw a status line would make `just test`
//! unusable. The `sha256` verification is `corpus_files_match_their_pinned_sha256`.

#[path = "../tests/support/corpus.rs"]
mod corpus;
// SPEC-084: the same two probes the tier-B tests skip on, so the summary
// states the tool status it actually checked.
#[path = "../tests/support/tools.rs"]
mod tools;

use corpus::{CorpusRoot, Manifest, Status};

fn main() -> std::process::ExitCode {
    let manifest = match Manifest::load() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("corpus: MANIFEST UNREADABLE — {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    let root = CorpusRoot::resolve();
    println!("corpus: root {} ({})", root.path.display(), root.origin());

    let mut missing = 0usize;
    for file in &manifest.files {
        match file.status(&root) {
            Status::Present => println!("corpus: present  {}", file.path),
            Status::Missing => {
                missing += 1;
                println!(
                    "corpus: SKIP     {} — MISSING at {}",
                    file.path,
                    file.resolve(&root).display()
                );
            }
        }
    }

    let total = manifest.files.len();
    let present = total - missing;
    for line in corpus::preflight_summary(
        present,
        total,
        tools::exiftool_available(),
        tools::dnglab_available(),
        corpus::CORPUS_DIR_ENV,
    ) {
        println!("{line}");
    }

    // SPEC-060: the optional Adobe dng_validate cross-check
    // (tests/gain_map_corpus.rs) is gated on its own variable, never set in
    // CI (DEC-069 §6). Its skip is surfaced here for the same reason the
    // corpus's is: cargo test captures a passing test's eprintln.
    const SDK_STAGE2_ENV: &str = "IRRADIANCE_SDK_STAGE2_DIR";
    match std::env::var_os(SDK_STAGE2_ENV) {
        Some(dir) if !dir.is_empty() => println!(
            "sdk-stage2: present  {} — the dng_validate cross-check will run",
            std::path::Path::new(&dir).display()
        ),
        _ => println!(
            "sdk-stage2: SKIP     ${SDK_STAGE2_ENV} unset — the dng_validate cross-check \
             (SPEC-060) will skip. Generate it with scripts/sdk-stage2.sh."
        ),
    }

    // SPEC-085: the optional SDK colour cross-check (tests/colour_sdk.rs), on
    // its own variable, never set in CI.
    const SDK_COLOUR_ENV: &str = "IRRADIANCE_SDK_COLOUR_DIR";
    match std::env::var_os(SDK_COLOUR_ENV) {
        Some(dir) if !dir.is_empty() => println!(
            "sdk-colour: present  {} — the SDK colour cross-check will run",
            std::path::Path::new(&dir).display()
        ),
        _ => println!(
            "sdk-colour: SKIP     ${SDK_COLOUR_ENV} unset — the SDK colour cross-check \
             (SPEC-085) will skip. Generate it with scripts/sdk-colour.sh."
        ),
    }

    // SPEC-099: the optional stage-3 per-plane warp cross-check
    // (tests/colour_sdk_stage3.rs), on its own variable, never set in CI.
    const SDK_STAGE3_ENV: &str = "IRRADIANCE_SDK_STAGE3_DIR";
    match std::env::var_os(SDK_STAGE3_ENV) {
        Some(dir) if !dir.is_empty() => println!(
            "sdk-stage3: present  {} — the dng_validate stage-3 cross-check will run",
            std::path::Path::new(&dir).display()
        ),
        _ => println!(
            "sdk-stage3: SKIP     ${SDK_STAGE3_ENV} unset — the dng_validate stage-3 \
             cross-check (SPEC-099) will skip. Generate it with scripts/sdk-stage3.sh."
        ),
    }

    // SPEC-061: the DNG writer's Adobe dng_validate acceptance
    // (tests/dng_write_corpus.rs, oracle 8) runs the binary this variable
    // names. Local only, never set in CI (DEC-069 §6).
    const DNG_VALIDATE_ENV: &str = "IRRADIANCE_DNG_VALIDATE";
    match std::env::var_os(DNG_VALIDATE_ENV) {
        Some(bin) if !bin.is_empty() => println!(
            "dng-validate: present  {} — the DNG writer's dng_validate acceptance will run",
            std::path::Path::new(&bin).display()
        ),
        _ => println!(
            "dng-validate: SKIP     ${DNG_VALIDATE_ENV} unset — the DNG writer's dng_validate \
             acceptance (SPEC-061) will skip."
        ),
    }

    std::process::ExitCode::SUCCESS
}
