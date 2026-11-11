//! `SPEC-052` — the aggregate IFD entry-table byte guard (`DEC-046`), the fix
//! for pre-0.2.0 code-review finding #1.
//!
//! Before this guard, `Container::parse` bounded each IFD (its table must lie
//! inside the file), the IFD *count* (`MAX_IFDS`) and cycles (a repeated
//! offset), but not the **aggregate**: entry tables at *distinct* offsets may
//! overlap the same bytes, so a ~770-byte file could make the walk parse and
//! allocate 32 × 32 entries, and a ~770 KB file millions. `DEC-046` keeps a
//! running total of every walked table's `count × 12` bytes and rejects the
//! file the moment it exceeds the file length.
//!
//! Every fixture is hand-built tier A (`tests/support/tiff.rs`); the one tier-B
//! check (`corpus_dngs_not_rejected`) skips loud when the corpus is absent.
//! The red-proof replays a mutated copy of the crate with the guard disabled
//! and shows the amplification test go red — control run first (`DEC-009`).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tiff.rs"]
mod tiff;

use std::path::{Path, PathBuf};

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::Container;
use irradiance::Error;
use tiff::Order;

/// `TIFF 6.0 §2`: every IFD entry is 12 bytes.
const ENTRY_SIZE: u64 = 12;

/// The amplification fixture's shape: 32 IFDs of 32 entries, 770 bytes.
const ALIASED_N: u32 = 32;
const ALIASED_COUNT: u16 = 32;

/// Sum of `entries().len() × 12` over every IFD a successful parse walked.
fn walked_table_bytes(container: &Container<'_>) -> u64 {
    container
        .ifds()
        .iter()
        .map(|ifd| u64::try_from(ifd.entries().len()).expect("fits u64") * ENTRY_SIZE)
        .sum()
}

fn file_len_u64(data: &[u8]) -> u64 {
    u64::try_from(data.len()).expect("fits u64")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — amplification rejected
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn aliased_ifd_tables_exceeding_file_are_rejected() {
    for order in [Order::Little, Order::Big] {
        let (data, _) = tiff::aliased_ifd_chain(order, ALIASED_N, ALIASED_COUNT);
        let file_len = data.len();
        let declared_total = u64::from(ALIASED_N) * u64::from(ALIASED_COUNT) * ENTRY_SIZE;
        assert!(
            declared_total > file_len_u64(&data),
            "{order:?}: the fixture must declare more table bytes ({declared_total}) than the \
             {file_len}-byte file, or it proves nothing"
        );

        match Container::parse(&data) {
            Err(Error::IfdTableBytesExceedFile {
                declared,
                file_len: reported,
            }) => {
                assert_eq!(reported, file_len, "{order:?}: file_len must be data.len()");
                assert!(
                    declared > file_len_u64(&data),
                    "{order:?}: declared {declared} must exceed the file it was rejected against"
                );
                // The guard fires at the FIRST IFD that tips the total over, so
                // at most one table's worth past the file length was ever
                // counted — i.e. the walk allocated no more than ~file_len / 12
                // entries before it said no. The 33.5M ceiling is irrelevant.
                let one_table = u64::from(ALIASED_COUNT) * ENTRY_SIZE;
                assert!(
                    declared <= file_len_u64(&data) + one_table,
                    "{order:?}: rejected late — declared {declared} is more than one \
                     {one_table}-byte table past the {file_len}-byte file"
                );
            }
            other => panic!(
                "{order:?}: expected Err(IfdTableBytesExceedFile), got {:?}",
                other.map(|c| c.ifds().len())
            ),
        }
    }
}

/// The new variant's message names both numbers, like its `TooManyIfds`
/// sibling names its limit.
#[test]
fn table_bytes_error_display_names_both_numbers() {
    let msg = Error::IfdTableBytesExceedFile {
        declared: 1152,
        file_len: 770,
    }
    .to_string();
    assert!(msg.contains("1152"), "{msg}");
    assert!(msg.contains("770"), "{msg}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — distinct from the cycle guard
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn distinct_offsets_isolate_the_new_guard() {
    let (data, offsets) = tiff::aliased_ifd_chain(Order::Little, ALIASED_N, ALIASED_COUNT);
    assert_eq!(offsets.len(), ALIASED_N as usize);

    let mut sorted = offsets.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        offsets.len(),
        "every IFD offset must be distinct, so the visited-offset cycle guard cannot be what \
         rejects this fixture"
    );
    // And the chain is under MAX_IFDS, so TooManyIfds cannot be it either.
    assert!(offsets.len() < irradiance::ifd::MAX_IFDS);
    // The tables really do overlap: consecutive IFDs are closer together than
    // one table is long.
    let table = u32::from(ALIASED_COUNT) * 12;
    assert!(offsets.windows(2).all(|w| w[1] - w[0] < table));

    assert!(
        matches!(
            Container::parse(&data),
            Err(Error::IfdTableBytesExceedFile { .. })
        ),
        "the aggregate guard, not another guard, must be what rejects the aliased chain"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — no false positive: the rule is SUM vs FILE LENGTH, not overlap
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn aggregate_table_bytes_within_file_still_parse() {
    // Overlapping tables whose sum still fits: 2 IFDs × 2 entries = 48 table
    // bytes in a 50-byte file. Overlap alone is NOT rejected.
    for order in [Order::Little, Order::Big] {
        let (data, offsets) = tiff::aliased_ifd_chain(order, 2, 2);
        assert!(
            offsets[1] - offsets[0] < 2 * 12,
            "the two tables must overlap for this to pin sum-not-overlap"
        );
        let container = Container::parse(&data).unwrap_or_else(|e| {
            panic!("{order:?}: overlapping tables summing within the file must parse: {e}")
        });
        assert_eq!(container.ifds().len(), 2);
        assert_eq!(walked_table_bytes(&container), 48);
        assert_eq!(data.len(), 50);
    }

    // Disjoint multi-IFD containers (IFD0 + SubIFD) parse unchanged.
    for order in [Order::Little, Order::Big] {
        let data = tiff::valid_subifd(order);
        let container = Container::parse(&data)
            .unwrap_or_else(|e| panic!("{order:?}: valid SubIFD fixture must still parse: {e}"));
        assert!(container.ifds().len() >= 2);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — the bound holds whenever parse is Ok
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ok_implies_table_bytes_bounded() {
    let mut fixtures: Vec<(String, Vec<u8>)> = tiff::all()
        .into_iter()
        .map(|(name, data)| (name.to_string(), data))
        .collect();
    for order in [Order::Little, Order::Big] {
        fixtures.push((
            format!("aliased-2x2-{order:?}"),
            tiff::aliased_ifd_chain(order, 2, 2).0,
        ));
    }

    let mut ok_count = 0;
    for (name, data) in &fixtures {
        if let Ok(container) = Container::parse(data) {
            ok_count += 1;
            let walked = walked_table_bytes(&container);
            assert!(
                walked <= file_len_u64(data),
                "{name}: parse returned Ok but its tables declare {walked} bytes, more than the \
                 {}-byte file",
                data.len()
            );
        }
    }
    assert!(
        ok_count >= 5,
        "only {ok_count} fixtures parsed — the property was checked against too few Ok cases"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 (tier B) — the real DNGs are not rejected
// ─────────────────────────────────────────────────────────────────────────────

const CORPUS_DNGS: &[&str] = &[
    "CANON-EOS-5D-MARK-III/5G4A9394.DNG",
    "NIKON-D300S/_DSC4615.DNG",
];

#[test]
fn corpus_dngs_not_rejected() {
    let root = CorpusRoot::resolve();
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    for manifest_path in CORPUS_DNGS {
        let entry = manifest
            .get(manifest_path)
            .unwrap_or_else(|| panic!("{manifest_path} must be in the manifest"));
        let Some(path) = entry.require(&root) else {
            eprintln!(
                "SKIP corpus_dngs_not_rejected for {manifest_path} — not present under {} ({})",
                root.path.display(),
                root.origin()
            );
            continue;
        };
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let container = Container::parse(&data).unwrap_or_else(|e| {
            panic!("{manifest_path}: the aggregate guard rejected a real DNG: {e}")
        });
        let walked = walked_table_bytes(&container);
        assert!(walked <= file_len_u64(&data));
        eprintln!(
            "{manifest_path}: {} IFDs, {walked} table bytes of {} file bytes — parses Ok",
            container.ifds().len(),
            data.len()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the red-proof (oracle-must-be-shown-red, DEC-009)
// ─────────────────────────────────────────────────────────────────────────────
//
// A copy of the crate is built in a temp dir with a small probe binary that
// parses a fixture file and prints `ok <ifds>` or `err <message>`. The CONTROL
// copy is unmutated and must reproduce the honest tree's rejection; the MUTANT
// copy has the guard's condition forced false and must accept the aliased
// chain. Mirrors `tests/develop.rs`'s `stage_probe_crate` (copied, not shared,
// as the spec asks).

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-ifd-guard-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("create temp dir {}: {e}", dir.display()));
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {}: {e}", dst.display()));
    for entry in
        std::fs::read_dir(src).unwrap_or_else(|e| panic!("read_dir {}: {e}", src.display()))
    {
        let entry = entry.expect("dir entry");
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type().expect("file_type");
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {}: {e}", target.display()));
        }
    }
}

/// The exact guard condition in `src/ifd.rs::read_ifd`. If this moves, the
/// red-proof fails loudly rather than silently mutating nothing.
const GUARD_NEEDLE: &str = "if *table_bytes > file_len {";

/// Disable `DEC-046`'s guard: the running total is still kept, but the
/// comparison can never reject — the guard "removed" without touching
/// anything else in the walk.
fn inject_guard_removed(ifd_rs: &Path) {
    let src = std::fs::read_to_string(ifd_rs)
        .unwrap_or_else(|e| panic!("read {}: {e}", ifd_rs.display()));
    let occurrences = src.matches(GUARD_NEEDLE).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one aggregate-guard condition in src/ifd.rs; found {occurrences} — the \
         guard moved, update GUARD_NEEDLE"
    );
    let mutated = src.replacen(
        GUARD_NEEDLE,
        "if false && *table_bytes > file_len { // RED-PROOF INJECTION -- tests/ifd_resource_guard.rs",
        1,
    );
    assert_ne!(mutated, src, "the injection must change the source");
    std::fs::write(ifd_rs, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", ifd_rs.display()));
}

const PROBE_MAIN: &str = r#"
fn main() {
    let path = std::env::args().nth(1).expect("fixture path");
    let data = std::fs::read(&path).expect("read fixture");
    match irradiance::ifd::Container::parse(&data) {
        Ok(c) => println!("ok {}", c.ifds().len()),
        Err(e) => println!("err {e}"),
    }
}
"#;

fn stage_probe_crate(dest: &Path, mutate: bool) {
    let repo_root = corpus::crate_root();

    std::fs::create_dir_all(dest.join("src/bin")).expect("mkdir src/bin");
    std::fs::copy(repo_root.join("Cargo.toml"), dest.join("Cargo.toml")).expect("copy Cargo.toml");
    let lock = repo_root.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, dest.join("Cargo.lock")).expect("copy Cargo.lock");
    }
    copy_dir_recursive(&repo_root.join("src"), &dest.join("src"));

    if mutate {
        inject_guard_removed(&dest.join("src/ifd.rs"));
    }

    std::fs::write(dest.join("src/bin/ifd_guard_probe.rs"), PROBE_MAIN)
        .expect("write probe binary source");
    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml
        .push_str("\n[[bin]]\nname = \"ifd_guard_probe\"\npath = \"src/bin/ifd_guard_probe.rs\"\n");
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

/// Build the staged crate and run the probe on `fixture`, returning its line.
fn build_and_run_probe(dir: &Path, fixture: &Path) -> String {
    let build = std::process::Command::new("cargo")
        .args(["build", "--release", "--bin", "ifd_guard_probe", "--quiet"])
        // Isolate the staged crate's build in its own `target/` (SPEC-056): an
        // inherited CARGO_TARGET_DIR would put the probe elsewhere, and would
        // build a mutated staged crate into the caller's shared target dir.
        .env_remove("CARGO_TARGET_DIR")
        .current_dir(dir)
        .output()
        .expect("spawn cargo build");
    assert!(
        build.status.success(),
        "cargo build --release failed in {}:\n{}",
        dir.display(),
        String::from_utf8_lossy(&build.stderr)
    );
    let bin = dir.join("target/release/ifd_guard_probe");
    let run = std::process::Command::new(&bin)
        .arg(fixture)
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "ifd_guard_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .to_string()
}

#[test]
fn ifd_guard_red_proof() {
    let (data, _) = tiff::aliased_ifd_chain(Order::Little, ALIASED_N, ALIASED_COUNT);

    // The honest tree, in-process: the amplification test's own expectation.
    assert!(
        matches!(
            Container::parse(&data),
            Err(Error::IfdTableBytesExceedFile { .. })
        ),
        "the honest tree must reject the aliased chain, or the red below proves nothing"
    );

    let fixture_dir = TempDir::new("fixture");
    let fixture = fixture_dir.0.join("aliased-ifd-tables.tiff");
    std::fs::write(&fixture, &data).expect("write fixture");

    // CONTROL first (DEC-009): the unmutated copy-and-rebuild apparatus must
    // reproduce the honest rejection, so a red below is attributable to the
    // injection and not to a broken copy or build.
    let control_dir = TempDir::new("control");
    stage_probe_crate(&control_dir.0, false);
    let control = build_and_run_probe(&control_dir.0, &fixture);
    assert!(
        control.starts_with("err IFD entry tables declare"),
        "the UNMUTATED copy must reject the aliased chain with the new error; got {control:?}"
    );

    // MUTANT: guard disabled. The amplification test's assertion — "parse
    // returns Err(IfdTableBytesExceedFile)" — must now FAIL: the walk accepts
    // all 32 aliased IFDs.
    let mutant_dir = TempDir::new("mutant");
    stage_probe_crate(&mutant_dir.0, true);
    let mutant = build_and_run_probe(&mutant_dir.0, &fixture);
    assert_ne!(
        mutant, control,
        "disabling the guard did NOT change the output — the injection is a semantic no-op and \
         this red-proof has caught NOTHING"
    );
    assert_eq!(
        mutant,
        format!("ok {ALIASED_N}"),
        "with the aggregate guard disabled the aliased chain must parse (the red); got {mutant:?}"
    );

    eprintln!("RED-PROOF (aggregate IFD table-bytes guard): control={control:?} mutant={mutant:?}");
}
