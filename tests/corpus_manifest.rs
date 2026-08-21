//! `SPEC-002` — the corpus manifest reader, and the skip that has to be seen.
//!
//! Three of these tests run anywhere. The fourth
//! (`corpus_files_match_their_pinned_sha256`) needs tier-B files, which are
//! never committed (`DEC-003`), so it skips per-entry when they are absent —
//! and `just test` prints the corpus-status lines that make the skip visible
//! before this suite runs. See `examples/corpus-status.rs` for why the
//! loudness cannot live in here.

#[path = "support/corpus.rs"]
mod corpus;

use corpus::{sha256, CorpusFile, CorpusRoot, Manifest, Status};
use std::path::{Path, PathBuf};

/// Entry count is asserted against a literal so that adding a manifest entry
/// without looking at this file is a *failure*, not a silent no-op.
const MANIFEST_ENTRIES: usize = 7;

// ─────────────────────────────────────────────────────────────────────────────
// 1. the reader exists and reads all 7 manifest entries
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn corpus_manifest_parses() {
    let m = Manifest::load().expect("tests/corpus/manifest.toml must parse");

    assert_eq!(
        m.files.len(),
        MANIFEST_ENTRIES,
        "manifest entry count changed — update MANIFEST_ENTRIES and check whoever \
         consumes the new entry"
    );

    for f in &m.files {
        // Every field this spec promises to expose, on every entry.
        assert!(!f.path.is_empty(), "entry has an empty path");
        assert!(
            !f.path.starts_with('/'),
            "{}: manifest paths are relative to $IRRADIANCE_CORPUS_DIR (DEC-003)",
            f.path
        );
        assert_eq!(
            f.sha256.len(),
            64,
            "{}: sha256 must be 64 hex chars",
            f.path
        );
        assert!(
            f.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
            "{}: sha256 is not hex",
            f.path
        );
        assert!(f.bytes > 0, "{}: bytes must be positive", f.path);
        assert_eq!(
            f.tier, "b",
            "{}: only tier B lives in this manifest",
            f.path
        );
        // DEC-003: licence provenance has no second chance. The reader is what
        // makes "EVERY entry MUST carry licence and source" mechanical.
        assert!(!f.licence.is_empty(), "{}: missing licence", f.path);
        assert!(!f.source.is_empty(), "{}: missing source", f.path);
        assert!(
            !f.oracle.raw_checksum.is_empty(),
            "{}: missing oracle.raw_checksum — nothing pins this file's ground truth",
            f.path
        );
    }

    // The reference frame, spot-checked against DEC-003 / docs/measured-q2m-dng.md.
    let reference = m
        .get("LEICA-Q2-MONO/L1021223.DNG")
        .expect("the reference frame must be in the manifest");
    assert_eq!(
        reference.sha256,
        "5957a4ca64b87b81309a4b77cb86a595e150f169125016f3803f08b9f893b14d"
    );
    assert_eq!(reference.bytes, 85_796_864);
    assert_eq!(
        reference.oracle.raw_checksum,
        "cb653b5bec24d166eef2fd258ee61ac4"
    );
    assert_eq!(reference.oracle.strip_bytes, Some(83_026_944));
    assert_eq!(reference.oracle.pgm_bytes, Some(94_887_955));

    // Layer-0 arithmetic, free and corpus-independent (AGENTS.md §12 bar 3):
    // 8424 x 5632 x 14 bits must be exactly StripByteCounts.
    assert_eq!(8424u64 * 5632 * 14 / 8, 83_026_944);

    // The 16-bit M Monochrom closes the same way: 5216 x 3472 x 16 bits.
    let m_mono = m
        .get("LEICA-M-MONOCHROM/L1000622.DNG")
        .expect("the CC0 M Monochrom frame must be in the manifest");
    assert_eq!(m_mono.licence, "CC0-1.0");
    assert_eq!(m_mono.oracle.strip_bytes, Some(36_219_904));
    assert_eq!(5216u64 * 3472 * 16 / 8, 36_219_904);

    // The two JPEG-compressed entries pin an oracle checksum but no layer-0
    // arithmetic — optional-by-design, not accidentally absent.
    let pef = m
        .get("PENTAX-K3III-MONO/K3III.PEF")
        .expect("the Pentax PEF must be in the manifest");
    assert_eq!(pef.oracle.strip_bytes, None);
    assert!(!pef.oracle.raw_checksum.is_empty());
}

#[test]
fn corpus_root_defaults_and_is_overridable() {
    // Default: the gitignored repo-local directory, resolved from the crate
    // root so the cwd cannot change the answer.
    let explicit = CorpusRoot::at("/nowhere/in/particular");
    assert_eq!(explicit.path, PathBuf::from("/nowhere/in/particular"));

    let m = Manifest::load().expect("manifest must parse");
    let f = &m.files[0];
    assert_eq!(
        f.resolve(&explicit),
        Path::new("/nowhere/in/particular").join(&f.path),
        "entry paths resolve under the root, component-wise"
    );

    // Whatever the environment says, the resolved root is absolute and the
    // default lands on tests/corpus/tier-b (DEC-003).
    let resolved = CorpusRoot::resolve();
    if !resolved.from_env {
        assert!(
            resolved.path.ends_with("tests/corpus/tier-b"),
            "default root must be tests/corpus/tier-b, got {}",
            resolved.path.display()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. sha256 mismatch is caught — the red-proof, with its negative control
// ─────────────────────────────────────────────────────────────────────────────

/// A synthetic one-entry manifest over `bytes` of content, pinning `sha`.
fn synthetic_manifest(sha: &str, bytes: u64) -> String {
    format!(
        r#"
[[file]]
path        = "SYNTHETIC/planted.bin"
tier        = "b"
bytes       = {bytes}
sha256      = "{sha}"
licence     = "own-work"
source      = "planted by corpus_hash_mismatch_fails"

  [file.oracle]
  raw_checksum = "00000000000000000000000000000000"
"#
    )
}

/// A temp directory that removes itself, so a failing test cannot leave
/// fixtures behind.
struct TempRoot(PathBuf);

impl TempRoot {
    fn new(tag: &str) -> TempRoot {
        let dir =
            std::env::temp_dir().join(format!("irradiance-spec002-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("SYNTHETIC")).expect("create temp corpus root");
        TempRoot(dir)
    }

    fn plant(&self, content: &[u8]) {
        std::fs::write(self.0.join("SYNTHETIC").join("planted.bin"), content)
            .expect("plant fixture");
    }

    fn root(&self) -> CorpusRoot {
        CorpusRoot::at(&self.0)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn only_entry(text: &str) -> CorpusFile {
    let m = Manifest::parse(text).expect("synthetic manifest must parse");
    assert_eq!(m.files.len(), 1);
    m.files.into_iter().next().expect("one entry")
}

#[test]
fn corpus_hash_mismatch_fails() {
    let good = b"irradiance corpus fixture: the bytes the manifest pins\n";
    let good_sha = sha256::to_hex(&sha256::hash(good));

    // A single flipped byte — SAME LENGTH, so the size check cannot be what
    // catches it. This is a red-proof of the sha256 comparison itself.
    let mut corrupt = good.to_vec();
    corrupt[0] ^= 0x01;
    assert_eq!(corrupt.len(), good.len());

    let tmp = TempRoot::new("hash");
    let entry = only_entry(&synthetic_manifest(&good_sha, good.len() as u64));

    // ── negative control: the honest file must PASS. Without this, a red
    //    proves only that verify() returns Err, not that it discriminates
    //    (the DEC-009 lesson, applied to a second oracle).
    tmp.plant(good);
    entry
        .verify(&tmp.root())
        .expect("an intact file must verify clean — otherwise the red below proves nothing");

    // ── the red: corrupt bytes must be rejected, naming the file.
    tmp.plant(&corrupt);
    let err = entry
        .verify(&tmp.root())
        .expect_err("a corrupted file must NOT verify");

    assert!(
        err.contains("SYNTHETIC/planted.bin"),
        "the failure must name the file; got: {err}"
    );
    assert!(
        err.contains("sha256 mismatch"),
        "the failure must say what mismatched; got: {err}"
    );
    assert!(
        err.contains(&good_sha),
        "the failure must quote the expected digest; got: {err}"
    );
}

#[test]
fn corpus_truncation_fails_by_size() {
    let good = b"irradiance corpus fixture: the bytes the manifest pins\n";
    let good_sha = sha256::to_hex(&sha256::hash(good));

    let tmp = TempRoot::new("trunc");
    let entry = only_entry(&synthetic_manifest(&good_sha, good.len() as u64));

    tmp.plant(&good[..good.len() - 5]);
    let err = entry
        .verify(&tmp.root())
        .expect_err("a truncated file must NOT verify");
    assert!(err.contains("SYNTHETIC/planted.bin"), "got: {err}");
    assert!(err.contains("size mismatch"), "got: {err}");
}

#[test]
fn corpus_absent_file_is_missing_not_an_error() {
    let tmp = TempRoot::new("absent");
    let entry = only_entry(&synthetic_manifest(&"0".repeat(64), 1));

    // Nothing planted: absence is a SKIP, not a failure.
    assert_eq!(entry.status(&tmp.root()), Status::Missing);
    assert!(entry.require(&tmp.root()).is_none());
}

#[test]
fn manifest_rejects_entries_missing_provenance() {
    // DEC-003's "EVERY entry MUST carry licence and source" is enforced by the
    // reader, not by remembering.
    let no_licence = r#"
[[file]]
path   = "SYNTHETIC/x.bin"
tier   = "b"
bytes  = 1
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
source = "somewhere"

  [file.oracle]
  raw_checksum = "00"
"#;
    let err = Manifest::parse(no_licence).expect_err("an entry with no licence must be rejected");
    assert!(err.contains("licence"), "got: {err}");
    assert!(
        err.contains("SYNTHETIC/x.bin"),
        "the error must name the entry; got: {err}"
    );

    let bad_sha = r#"
[[file]]
path    = "SYNTHETIC/x.bin"
tier    = "b"
bytes   = 1
sha256  = "deadbeef"
licence = "own-work"
source  = "somewhere"

  [file.oracle]
  raw_checksum = "00"
"#;
    let err = Manifest::parse(bad_sha).expect_err("a short sha256 must be rejected");
    assert!(err.contains("sha256"), "got: {err}");
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. the hash function itself, proven against NIST vectors
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sha256_matches_published_vectors() {
    // FIPS 180-4 / NIST CSRC example vectors.
    let cases: [(&[u8], &str); 3] = [
        (
            b"",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(
            sha256::to_hex(&sha256::hash(input)),
            expected,
            "SHA-256 of {:?} is wrong",
            String::from_utf8_lossy(input)
        );
    }

    // One million 'a' — exercises the multi-block path the corpus files use.
    let million = vec![b'a'; 1_000_000];
    assert_eq!(
        sha256::to_hex(&sha256::hash(&million)),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn sha256_streaming_matches_one_shot() {
    // The corpus path hashes in 1 MiB chunks; any split must agree with the
    // one-shot digest, including splits that straddle the 64-byte block.
    let data: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
    let one_shot = sha256::hash(&data);

    for split in [1usize, 63, 64, 65, 127, 128, 1000, 4999] {
        let mut h = sha256::Sha256::new();
        let (a, b) = data.split_at(split);
        h.update(a);
        h.update(b);
        assert_eq!(h.finish(), one_shot, "split at {split} changed the digest");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. the tier-B test: verify what is present, skip what is not
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn corpus_files_match_their_pinned_sha256() {
    let m = Manifest::load().expect("manifest must parse");
    let root = CorpusRoot::resolve();

    let mut verified = 0usize;
    for f in &m.files {
        // `require` returns None for an absent file: that is the skip. It is
        // silent here on purpose — `cargo test` captures stdout/stderr of a
        // passing test. `just test` printed the corpus-status lines already.
        if f.require(&root).is_none() {
            continue;
        }
        if let Err(e) = f.verify(&root) {
            panic!("{e}");
        }
        verified += 1;
    }

    // Not an assertion on `verified > 0`: zero present files is the normal,
    // correct state on CI (DEC-003). The visible record of which files were
    // skipped is corpus-status, not this test.
    eprintln!("verified {verified}/{} corpus file(s)", m.files.len());
}
