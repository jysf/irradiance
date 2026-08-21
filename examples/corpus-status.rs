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
    if missing == 0 {
        println!("corpus: {present}/{total} present — no tier-B test will skip");
    } else {
        println!(
            "corpus: {present}/{total} present, {missing} MISSING — tier-B tests over \
             those file(s) will SKIP. Set ${} to point at your corpus.",
            corpus::CORPUS_DIR_ENV
        );
    }

    std::process::ExitCode::SUCCESS
}
