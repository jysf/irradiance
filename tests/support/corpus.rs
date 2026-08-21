//! Corpus manifest reader — the single place that knows where tier-B files
//! live and what they are supposed to hash to.
//!
//! `SPEC-002`. `DEC-003` settled the storage model: **real camera files are
//! never committed**; they live outside the repo, located at run time by
//! `$IRRADIANCE_CORPUS_DIR`, and `tests/corpus/manifest.toml` is the committed
//! index that pins each file's `sha256` and its oracle answers. This module is
//! the reader for that index. Before it existed the manifest was the "unread
//! field" AGENTS.md §11 warns about — its own header recorded that as a debt.
//!
//! **No test in this repo may hardcode a corpus path.** Go through
//! [`CorpusRoot::resolve`] and [`CorpusFile::resolve`].
//!
//! This is test-support code, shared by `tests/corpus_manifest.rs` and
//! `examples/corpus-status.rs` via `#[path]`. It is not part of the library
//! and is never compiled into it — the `toml` dependency it uses is a
//! dev-dependency (`DEC-010`), which is why the reader cannot live in `src/`.
//!
//! ## Why the loudness is not in here
//!
//! Measured at design: `eprintln!` inside a *passing* test is captured —
//! `cargo test` shows 0 SKIP lines, `cargo test -- --nocapture` shows them.
//! So a "loud skip" written inside the harness is silent in exactly the
//! situation it exists for. The visible surface is therefore
//! `examples/corpus-status.rs`, which `just test` runs **before** the suite.
//! In-harness, [`CorpusFile::require`] just returns `None` and the test
//! returns early.

// `dead_code` only, and only here. This file is compiled into TWO crates via
// `#[path]` — the `corpus_manifest` test binary and the `corpus-status`
// example — and each uses a different subset of it: the example calls
// `origin()`, the tests call the whole `sha256` module and `verify()`. Neither
// crate can see the other's use, so `dead_code` reports items that ARE used.
//
// ⚠ This is NOT the `#[allow]` bypass SPEC-006 closes. That one is about the
// five panic-free lints (`unwrap_used`, `expect_used`, `indexing_slicing`,
// `panic`, `arithmetic_side_effects`) escaping `src/lib.rs`'s crate-root
// policy. This is `dead_code`, in `tests/`, on code that is never compiled
// into the library at all.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Environment variable that relocates the tier-B corpus (`DEC-003`).
pub const CORPUS_DIR_ENV: &str = "IRRADIANCE_CORPUS_DIR";

/// Repo-relative default root, `.gitignore`d so a stray `git add -A` after a
/// camera session cannot put an 86 MB blob in history (`DEC-003`).
pub const DEFAULT_CORPUS_SUBDIR: &str = "tests/corpus/tier-b";

/// Path of the committed manifest, relative to the crate root.
pub const MANIFEST_SUBPATH: &str = "tests/corpus/manifest.toml";

// ─────────────────────────────────────────────────────────────────────────────
// Corpus root
// ─────────────────────────────────────────────────────────────────────────────

/// Where tier-B files are looked for, and whether that came from the
/// environment or the default.
#[derive(Debug, Clone)]
pub struct CorpusRoot {
    /// The directory tier-B paths are resolved against.
    pub path: PathBuf,
    /// `true` when `$IRRADIANCE_CORPUS_DIR` supplied it.
    pub from_env: bool,
}

impl CorpusRoot {
    /// `$IRRADIANCE_CORPUS_DIR` if set and non-empty, else the gitignored
    /// `tests/corpus/tier-b/` under the crate root.
    pub fn resolve() -> CorpusRoot {
        match std::env::var(CORPUS_DIR_ENV) {
            Ok(v) if !v.trim().is_empty() => CorpusRoot {
                path: PathBuf::from(v),
                from_env: true,
            },
            _ => CorpusRoot {
                path: crate_root().join(DEFAULT_CORPUS_SUBDIR),
                from_env: false,
            },
        }
    }

    /// A root at an explicit directory — used by tests that plant fixtures.
    pub fn at<P: AsRef<Path>>(path: P) -> CorpusRoot {
        CorpusRoot {
            path: path.as_ref().to_path_buf(),
            from_env: false,
        }
    }

    /// How the root was chosen, for the status line.
    pub fn origin(&self) -> &'static str {
        if self.from_env {
            "from $IRRADIANCE_CORPUS_DIR"
        } else {
            "default; set $IRRADIANCE_CORPUS_DIR to override"
        }
    }
}

/// The crate root, resolved at compile time so the cwd never matters.
pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Manifest model
// ─────────────────────────────────────────────────────────────────────────────

/// The pinned oracle answers for one file (`docs/oracle-contract.md`).
///
/// `raw_checksum` is **required**: it pins both the file and the oracle, so a
/// dnglab version bump that changes the answer is caught rather than silently
/// redefining ground truth (`DEC-003`). `pgm_bytes` and `strip_bytes` are
/// optional — the JPEG-compressed entries have no layer-0 arithmetic to pin.
#[derive(Debug, Clone)]
pub struct Oracle {
    /// `dnglab analyze --raw-checksum <file>`.
    pub raw_checksum: String,
    /// Byte count of the PGM dnglab writes, when one is pinned.
    pub pgm_bytes: Option<u64>,
    /// `StripByteCounts` — the layer-0 oracle, when one is pinned.
    pub strip_bytes: Option<u64>,
}

/// One `[[file]]` entry.
#[derive(Debug, Clone)]
pub struct CorpusFile {
    /// Path **relative to the corpus root**. Never absolute (`DEC-003`).
    pub path: String,
    /// `"b"` today; tier A is a separate, CC0-only lane.
    pub tier: String,
    /// Expected size in bytes.
    pub bytes: u64,
    /// Expected SHA-256, lowercase hex.
    pub sha256: String,
    /// SPDX id, a CC id, `own-work`, or `personal-evaluation-only`.
    pub licence: String,
    /// Where the file came from.
    pub source: String,
    /// Pinned oracle answers.
    pub oracle: Oracle,
}

/// The parsed `tests/corpus/manifest.toml`.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// Every `[[file]]` entry, in manifest order.
    pub files: Vec<CorpusFile>,
}

impl Manifest {
    /// Read and parse the committed manifest.
    pub fn load() -> Result<Manifest, String> {
        let path = crate_root().join(MANIFEST_SUBPATH);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read manifest {}: {e}", path.display()))?;
        Manifest::parse(&text)
    }

    /// Parse manifest text. Separated from [`Manifest::load`] so the
    /// hash-mismatch red-proof can build a synthetic manifest without
    /// touching the committed one.
    pub fn parse(text: &str) -> Result<Manifest, String> {
        // `toml::Value: FromStr` is what `features = ["parse"]` buys; the
        // serde-derive route would cost a second dev-dependency.
        let doc: toml::Value = text
            .parse()
            .map_err(|e| format!("manifest is not valid TOML: {e}"))?;

        let entries = match doc.get("file") {
            Some(toml::Value::Array(a)) => a,
            Some(_) => return Err("manifest key `file` is not an array of tables".to_string()),
            None => return Err("manifest has no [[file]] entries".to_string()),
        };

        let mut files = Vec::with_capacity(entries.len());
        for (i, entry) in entries.iter().enumerate() {
            files.push(parse_file(entry, i)?);
        }
        Ok(Manifest { files })
    }

    /// Look up one entry by its manifest path.
    pub fn get(&self, path: &str) -> Option<&CorpusFile> {
        self.files.iter().find(|f| f.path == path)
    }
}

/// Every required field is required *loudly*: `licence` and `source` have no
/// second chance (`DEC-003` — provenance cannot be reconstructed later), and
/// an entry with no pinned `raw_checksum` would be a file nothing can check.
fn parse_file(entry: &toml::Value, index: usize) -> Result<CorpusFile, String> {
    let at = format!("[[file]] #{}", index + 1);

    let path = req_str(entry, "path", &at)?;
    // Now that we know the path, name it in every subsequent error.
    let at = format!("[[file]] #{} ({path})", index + 1);

    if path.starts_with('/') {
        return Err(format!(
            "{at}: `path` is absolute; manifest paths are relative to \
             ${CORPUS_DIR_ENV} (DEC-003)"
        ));
    }

    let sha256 = req_str(entry, "sha256", &at)?;
    if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "{at}: `sha256` is not 64 hex characters (got {} chars)",
            sha256.len()
        ));
    }

    let oracle = entry
        .get("oracle")
        .ok_or_else(|| format!("{at}: missing [file.oracle] table"))?;

    Ok(CorpusFile {
        path,
        tier: req_str(entry, "tier", &at)?,
        bytes: req_int(entry, "bytes", &at)?,
        sha256: sha256.to_ascii_lowercase(),
        licence: req_str(entry, "licence", &at)?,
        source: req_str(entry, "source", &at)?,
        oracle: Oracle {
            raw_checksum: req_str(oracle, "raw_checksum", &at)?,
            pgm_bytes: opt_int(oracle, "pgm_bytes", &at)?,
            strip_bytes: opt_int(oracle, "strip_bytes", &at)?,
        },
    })
}

fn req_str(table: &toml::Value, key: &str, at: &str) -> Result<String, String> {
    match table.get(key) {
        Some(toml::Value::String(s)) if !s.trim().is_empty() => Ok(s.clone()),
        Some(toml::Value::String(_)) => Err(format!("{at}: `{key}` is empty")),
        Some(_) => Err(format!("{at}: `{key}` is not a string")),
        None => Err(format!("{at}: missing required key `{key}`")),
    }
}

fn req_int(table: &toml::Value, key: &str, at: &str) -> Result<u64, String> {
    match opt_int(table, key, at)? {
        Some(v) => Ok(v),
        None => Err(format!("{at}: missing required key `{key}`")),
    }
}

fn opt_int(table: &toml::Value, key: &str, at: &str) -> Result<Option<u64>, String> {
    match table.get(key) {
        None => Ok(None),
        Some(toml::Value::Integer(i)) if *i >= 0 => Ok(Some(*i as u64)),
        Some(toml::Value::Integer(i)) => Err(format!("{at}: `{key}` is negative ({i})")),
        Some(_) => Err(format!("{at}: `{key}` is not an integer")),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Presence and verification
// ─────────────────────────────────────────────────────────────────────────────

/// Whether a tier-B file is on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// The file is there.
    Present,
    /// The file is absent — tests over it skip, and `corpus-status` says so.
    Missing,
}

impl CorpusFile {
    /// Absolute path of this entry under `root`. Manifest paths use `/`;
    /// join component-wise so they stay portable.
    pub fn resolve(&self, root: &CorpusRoot) -> PathBuf {
        let mut p = root.path.clone();
        for part in self.path.split('/').filter(|s| !s.is_empty()) {
            p.push(part);
        }
        p
    }

    /// Presence of this entry under `root`.
    pub fn status(&self, root: &CorpusRoot) -> Status {
        if self.resolve(root).is_file() {
            Status::Present
        } else {
            Status::Missing
        }
    }

    /// The path a tier-B test should use, or `None` when the file is absent.
    ///
    /// `None` means **skip**: the caller returns early. The skip is made
    /// visible by `examples/corpus-status.rs`, which `just test` runs before
    /// the suite — not by printing here, which `cargo test` would swallow.
    pub fn require(&self, root: &CorpusRoot) -> Option<PathBuf> {
        let p = self.resolve(root);
        if p.is_file() {
            Some(p)
        } else {
            // Visible only under `--nocapture`; kept because it costs nothing
            // and helps whoever is already running with the flag. It is NOT
            // the surface that satisfies the spec's criterion 4.
            eprintln!(
                "SKIP {} — MISSING at {} (see the corpus-status lines `just test` prints)",
                self.path,
                p.display()
            );
            None
        }
    }

    /// Verify a **present** file against the manifest: size first (a truncated
    /// or half-downloaded copy gets an actionable message), then SHA-256.
    ///
    /// The error names the file, per `SPEC-002` acceptance criterion 3 — a
    /// re-download or a truncation must not pass silently.
    pub fn verify(&self, root: &CorpusRoot) -> Result<(), String> {
        let path = self.resolve(root);

        let meta = std::fs::metadata(&path).map_err(|e| {
            format!(
                "CORRUPT CORPUS: {} — cannot stat {}: {e}",
                self.path,
                path.display()
            )
        })?;
        if meta.len() != self.bytes {
            return Err(format!(
                "CORRUPT CORPUS: {} — size mismatch at {}\n  manifest: {} bytes\n  on disk:  {} bytes\n\
                 A truncated or partial download. Re-fetch it, or fix the manifest if the file changed.",
                self.path,
                path.display(),
                self.bytes,
                meta.len()
            ));
        }

        let actual = sha256::hash_file(&path).map_err(|e| {
            format!(
                "CORRUPT CORPUS: {} — cannot read {}: {e}",
                self.path,
                path.display()
            )
        })?;
        let actual = sha256::to_hex(&actual);
        if actual != self.sha256 {
            return Err(format!(
                "CORRUPT CORPUS: {} — sha256 mismatch at {}\n  manifest: {}\n  on disk:  {}\n\
                 The bytes are not the bytes this manifest pins. Every oracle answer \
                 recorded for this entry is about a different file.",
                self.path,
                path.display(),
                self.sha256,
                actual
            ));
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SHA-256
// ─────────────────────────────────────────────────────────────────────────────

/// SHA-256, implemented from **FIPS PUB 180-4 §6.2** (published specification —
/// provenance class 1, `docs/provenance-ledger.md`).
///
/// Written rather than taken as a dependency for two reasons. It keeps this
/// spec to the one dev-dependency its design measured (`DEC-010`), and — the
/// real reason — a hashing crate would be exercised *only* on a machine that
/// holds the corpus, so a broken integration would be invisible in CI. The
/// NIST vectors in `tests/corpus_manifest.rs` run everywhere, every time.
///
/// This is a file-integrity check against a pinned digest in dev-only test
/// support. It is not a security boundary and nothing in the library uses it.
pub mod sha256 {
    use std::io::Read;
    use std::path::Path;

    /// FIPS 180-4 §4.2.2 — first 32 bits of the fractional parts of the cube
    /// roots of the first 64 primes.
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    /// FIPS 180-4 §5.3.3 — first 32 bits of the fractional parts of the square
    /// roots of the first 8 primes.
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    const BLOCK: usize = 64;

    /// Streaming SHA-256 state.
    pub struct Sha256 {
        state: [u32; 8],
        buf: [u8; BLOCK],
        buf_len: usize,
        total_bytes: u64,
    }

    impl Default for Sha256 {
        fn default() -> Self {
            Sha256::new()
        }
    }

    impl Sha256 {
        /// A fresh hasher.
        pub fn new() -> Sha256 {
            Sha256 {
                state: H0,
                buf: [0u8; BLOCK],
                buf_len: 0,
                total_bytes: 0,
            }
        }

        /// Absorb more input. Any split of the same byte stream is equivalent.
        pub fn update(&mut self, mut data: &[u8]) {
            self.total_bytes = self.total_bytes.wrapping_add(data.len() as u64);

            if self.buf_len > 0 {
                let want = BLOCK - self.buf_len;
                let take = want.min(data.len());
                self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
                self.buf_len += take;
                data = &data[take..];
                if self.buf_len == BLOCK {
                    let block = self.buf;
                    compress(&mut self.state, &block);
                    self.buf_len = 0;
                }
            }

            // Guard: when the call above consumed all of `data` into a
            // partial buffer, there is no tail to store and `buf_len` must
            // NOT be overwritten. Without this the buffered bytes are dropped
            // and the digest silently changes — caught by
            // `sha256_streaming_matches_one_shot`, split at 4999.
            if !data.is_empty() {
                let mut chunks = data.chunks_exact(BLOCK);
                for block in &mut chunks {
                    let mut b = [0u8; BLOCK];
                    b.copy_from_slice(block);
                    compress(&mut self.state, &b);
                }

                let rest = chunks.remainder();
                self.buf[..rest.len()].copy_from_slice(rest);
                self.buf_len = rest.len();
            }
        }

        /// FIPS 180-4 §5.1.1 padding, then the final digest.
        pub fn finish(mut self) -> [u8; 32] {
            let bit_len = self.total_bytes.wrapping_mul(8);

            self.buf[self.buf_len] = 0x80;
            self.buf_len += 1;
            if self.buf_len > BLOCK - 8 {
                self.buf[self.buf_len..].fill(0);
                let block = self.buf;
                compress(&mut self.state, &block);
                self.buf_len = 0;
            }
            self.buf[self.buf_len..BLOCK - 8].fill(0);
            self.buf[BLOCK - 8..].copy_from_slice(&bit_len.to_be_bytes());
            let block = self.buf;
            compress(&mut self.state, &block);

            let mut out = [0u8; 32];
            for (chunk, word) in out.chunks_exact_mut(4).zip(self.state.iter()) {
                chunk.copy_from_slice(&word.to_be_bytes());
            }
            out
        }
    }

    /// FIPS 180-4 §6.2.2 — the 64-round compression function.
    fn compress(state: &mut [u32; 8], block: &[u8; BLOCK]) {
        let mut w = [0u32; 64];
        for (word, chunk) in w.iter_mut().zip(block.chunks_exact(4)) {
            let mut b = [0u8; 4];
            b.copy_from_slice(chunk);
            *word = u32::from_be_bytes(b);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

        for (k, wi) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(*k)
                .wrapping_add(*wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        for (s, v) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(v);
        }
    }

    /// Digest of a byte slice.
    pub fn hash(data: &[u8]) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(data);
        h.finish()
    }

    /// Digest of a file, streamed — corpus frames are 20–86 MB each.
    pub fn hash_file(path: &Path) -> std::io::Result<[u8; 32]> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finish())
    }

    /// Lowercase hex, the form the manifest records.
    pub fn to_hex(digest: &[u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for b in digest {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}
