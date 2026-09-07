//! Hand-built `OpcodeList` byte fixtures and the committed-hex-fixture
//! loader — `SPEC-018`.
//!
//! Every encoder here is **own work, built byte by byte from DNG 1.7.0.0
//! Chapter 7 and §6.4.1** (`src/opcode.rs`'s own module doc carries the exact
//! layout this mirrors). Compiled into `tests/opcode.rs`, `tests/warp.rs` and
//! `examples/fuzz-seeds.rs` via `#[path]`; each uses a different subset, so
//! `dead_code` is allowed here for the same reason it is in
//! `tests/support/tiff.rs`.

#![allow(dead_code)]

/// One raw opcode entry: `id, version, flags, size, params` — DNG 1.7.0.0
/// Chapter 7, big-endian regardless of the file's own byte order.
pub fn raw_opcode(id: u32, version: u32, flags: u32, params: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&flags.to_be_bytes());
    out.extend_from_slice(
        &u32::try_from(params.len())
            .expect("test fixture params fit u32")
            .to_be_bytes(),
    );
    out.extend_from_slice(params);
    out
}

/// A complete `OpcodeList` byte stream: the `count` prefix, then every
/// opcode's own already-encoded bytes ([`raw_opcode`]) concatenated.
pub fn opcode_list(opcodes: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(
        &u32::try_from(opcodes.len())
            .expect("test fixture opcode count fits u32")
            .to_be_bytes(),
    );
    for op in opcodes {
        out.extend_from_slice(op);
    }
    out
}

/// `WarpRectilinear`'s parameter block (DNG 1.7.0.0 §6.4.1): `N` (always 1
/// here — every Q2M frame measures a single coefficient set), the six `kr`/
/// `kt` `DOUBLE`s, then `cx_hat`/`cy_hat`. 68 bytes for `N = 1`, matching
/// every real Q2M frame's measured `DataSize`.
pub fn warp_rectilinear_params(
    kr: [f64; 4],
    kt: [f64; 2],
    cx: f64,
    cy: f64,
    planes: u32,
) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&planes.to_be_bytes());
    for _ in 0..planes {
        for v in kr {
            p.extend_from_slice(&v.to_be_bytes());
        }
        for v in kt {
            p.extend_from_slice(&v.to_be_bytes());
        }
    }
    p.extend_from_slice(&cx.to_be_bytes());
    p.extend_from_slice(&cy.to_be_bytes());
    p
}

/// A complete `OpcodeList` byte stream carrying exactly one `WarpRectilinear`
/// opcode (`OpcodeID` 1) with the given `flags` — the shape every real Q2M
/// `OpcodeList3` takes (`flags = 0`: mandatory, confirmed against all three
/// decodable frames).
pub fn single_warp_rectilinear_list(
    kr: [f64; 4],
    kt: [f64; 2],
    cx: f64,
    cy: f64,
    planes: u32,
    flags: u32,
) -> Vec<u8> {
    let params = warp_rectilinear_params(kr, kt, cx, cy, planes);
    opcode_list(&[raw_opcode(1, 0x0104_0000, flags, &params)])
}

/// Load a committed hex fixture from `tests/oracle-fixtures/<name>.hex`
/// (e.g. `opcodelist3-L1021223`) — the exact `OpcodeList3` bytes read
/// straight from the real file with `exiftool -b -OpcodeList3` (AGENTS.md §12
/// "design-time probe": the real bytes, not a re-derivation). Whitespace
/// (including the trailing newline) is stripped before decoding; the file
/// itself is a single unbroken lowercase hex string.
pub fn load_hex_fixture(name: &str) -> Vec<u8> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("oracle-fixtures")
        .join(format!("{name}.hex"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("load_hex_fixture: {}: {e}", path.display()));
    let hex: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        hex.len().is_multiple_of(2),
        "load_hex_fixture: {}: odd number of hex digits ({})",
        path.display(),
        hex.len()
    );
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .unwrap_or_else(|e| panic!("load_hex_fixture: {}: {e}", path.display()))
        })
        .collect()
}

/// The three decodable Q2M frames' hex-fixture names (see
/// `tests/opcode.rs`'s own `FRAMES` for the matching `kr0..kr3` table —
/// duplicated as bare names here because this module has no reason to know
/// the coefficient VALUES, only which fixtures exist).
const FIXTURE_NAMES: [&str; 3] = [
    "opcodelist3-L1021223",
    "opcodelist3-L1026016",
    "opcodelist3-L1026192",
];

/// The `warp_opcode` fuzz target's seed corpus — real bytes from all three
/// decodable frames plus three hand-truncated/adversarial variants
/// (`## Implementation Context`'s "Fuzz seed set for `warp_opcode`"). Shared
/// by `tests/opcode.rs`'s smoke test and `examples/fuzz-seeds.rs`'s
/// `fuzz/seeds/warp_opcode/` writer, so the two can never drift apart.
pub fn fuzz_seed_corpus() -> Vec<(&'static str, Vec<u8>)> {
    let mut seeds: Vec<(&'static str, Vec<u8>)> = FIXTURE_NAMES
        .iter()
        .map(|&fixture| (fixture, load_hex_fixture(fixture)))
        .collect();

    // Truncated length prefix: declares more opcodes than bytes remain.
    let mut lying_count = load_hex_fixture("opcodelist3-L1021223");
    lying_count[0..4].copy_from_slice(&99u32.to_be_bytes());
    seeds.push(("lying-opcode-count", lying_count));

    // Zero-byte parameter block on a recognized opcode ID (WarpRectilinear
    // with DataSize = 0, its own params empty).
    seeds.push((
        "zero-byte-warp-params",
        opcode_list(&[raw_opcode(1, 0x0104_0000, 0, &[])]),
    ));

    // Unknown opcode ID with the mandatory (non-optional) flag bit set.
    seeds.push((
        "unknown-mandatory-opcode",
        opcode_list(&[raw_opcode(0xDEAD_BEEF, 0x0104_0000, 0, &[1, 2, 3, 4])]),
    ));

    seeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_hex_fixture_decodes_the_committed_l1021223_bytes() {
        let bytes = load_hex_fixture("opcodelist3-L1021223");
        assert_eq!(
            bytes.len(),
            88,
            "count(4) + id/version/flags/size(16) + params(68)"
        );
        assert_eq!(&bytes[0..4], &1u32.to_be_bytes(), "opcode count");
        assert_eq!(
            &bytes[4..8],
            &1u32.to_be_bytes(),
            "OpcodeID 1 (WarpRectilinear)"
        );
        assert_eq!(
            &bytes[8..12],
            &0x0104_0000u32.to_be_bytes(),
            "DNG version 1.4.0.0"
        );
        assert_eq!(&bytes[12..16], &0u32.to_be_bytes(), "Flags 0: mandatory");
        assert_eq!(&bytes[16..20], &68u32.to_be_bytes(), "DataSize 68");
    }

    #[test]
    fn opcode_list_round_trips_through_raw_opcode() {
        let params = warp_rectilinear_params([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5, 1);
        let bytes = opcode_list(&[raw_opcode(1, 0x0104_0000, 0, &params)]);
        // count(4) + id/version/flags/size(16) + params(68) = 88.
        assert_eq!(bytes.len(), 88);
    }
}
