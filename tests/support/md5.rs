//! MD5 — RFC 1321. Dev-only test support; never in the library.
//!
//! Follows `tests/support/corpus.rs::sha256`'s precedent exactly (`DEC-010`):
//! written from the published standard, not from any implementation, and
//! proven against RFC 1321's own published test suite
//! (`md5_matches_the_rfc_1321_test_vectors`, `tests/plane_oracle.rs`). This is
//! a file-integrity/oracle check against a pinned digest (`dnglab analyze
//! --raw-checksum`, `docs/oracle-contract.md`), not a security boundary, and
//! nothing in the library uses it (`SPEC-013`).
//!
//! MD5's byte/word order is **little-endian throughout** (RFC 1321 §3.4 step
//! 3, §3.5) — the opposite convention from `sha256`'s big-endian, and the
//! reason this is not a copy of that module with new constants: message
//! words are read little-endian, the bit-length suffix is little-endian, and
//! the four output words are emitted little-endian.
//!
//! This same file is also embedded verbatim (`include_str!`) into the
//! red-proof's synthesized probe binary (`tests/plane_oracle.rs`,
//! `PROBE_MD5_SOURCE`), so the mutated-crate build and this test binary run
//! the exact same hasher — one implementation, never two to keep in sync.

/// RFC 1321 §3.4 Step 4 — one 32-bit additive constant per round step,
/// `floor(abs(sin(i + 1)) * 2^32)` for `i` in `0..64`, exactly as the RFC's
/// own step tables list them.
const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// RFC 1321 §3.4 Step 4 — the per-round-step left-rotate amount.
const SHIFT: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// RFC 1321 §3.3 — the initial state.
const H0: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

const BLOCK: usize = 64;

/// Streaming MD5 state.
pub struct Md5 {
    state: [u32; 4],
    buf: [u8; BLOCK],
    buf_len: usize,
    total_bytes: u64,
}

impl Default for Md5 {
    fn default() -> Self {
        Md5::new()
    }
}

impl Md5 {
    /// A fresh hasher.
    pub fn new() -> Md5 {
        Md5 {
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

        if !data.is_empty() {
            let (blocks, rest) = data.as_chunks::<BLOCK>();
            for block in blocks {
                compress(&mut self.state, block);
            }
            self.buf[..rest.len()].copy_from_slice(rest);
            self.buf_len = rest.len();
        }
    }

    /// RFC 1321 §3.1 padding — length suffix is **little-endian**, unlike
    /// `sha256`'s big-endian — then the digest.
    pub fn finish(mut self) -> [u8; 16] {
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
        self.buf[BLOCK - 8..].copy_from_slice(&bit_len.to_le_bytes());
        let block = self.buf;
        compress(&mut self.state, &block);

        let mut out = [0u8; 16];
        for (chunk, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(self.state.iter()) {
            *chunk = word.to_le_bytes();
        }
        out
    }
}

/// RFC 1321 §3.4 Step 4 — the 64-round compression function.
fn compress(state: &mut [u32; 4], block: &[u8; BLOCK]) {
    let mut m = [0u32; 16];
    for (word, chunk) in m.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_le_bytes(*chunk);
    }

    let [mut a, mut b, mut c, mut d] = *state;

    for i in 0..64usize {
        let (f, g): (u32, usize) = if i < 16 {
            ((b & c) | (!b & d), i)
        } else if i < 32 {
            ((d & b) | (!d & c), (5 * i + 1) % 16)
        } else if i < 48 {
            (b ^ c ^ d, (3 * i + 5) % 16)
        } else {
            (c ^ (b | !d), (7 * i) % 16)
        };

        let f = f.wrapping_add(a).wrapping_add(K[i]).wrapping_add(m[g]);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(f.rotate_left(SHIFT[i]));
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

/// Digest of a byte slice.
pub fn hash(data: &[u8]) -> [u8; 16] {
    let mut h = Md5::new();
    h.update(data);
    h.finish()
}

/// Lowercase hex, the form `docs/oracle-contract.md`'s `raw_checksum` and
/// `tests/corpus/manifest.toml` record.
pub fn to_hex(digest: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
