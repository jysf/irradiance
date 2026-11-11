//! Required-processing validation (`SPEC-054`, `DEC-049`): a DNG that carries a
//! **mandatory** opcode the develop path will not apply is told so with a typed
//! error, instead of developing to a plausible-looking wrong image and `Ok(())`.
//!
//! Every fixture goes through the real container read
//! (`Container::parse → sensor → unpack_into`), so `OpcodeList2`'s bytes travel
//! the same tag-read path a camera file's would. The applied set (DEC-049):
//!
//! - mono (`develop_into`): the first `FixBadPixelsConstant` (id 4) in list 1
//!   and the first `WarpRectilinear` (id 1) in list 3;
//! - colour (`develop_color_*`): nothing;
//! - `OpcodeList2`: nothing, on either path.
//!
//! A mandatory opcode outside that set is `Error::UnsupportedRequiredOpcode`
//! (or, for an id this reader does not recognize at all, the existing
//! `Error::UnsupportedMandatoryOpcode`). Optional opcodes are always skippable.
//!
//! §12 behavioural verification: each public develop entry is exercised on its
//! own (`develop_into`, `develop_color_into`, `develop_color_f32_into`, and both
//! threaded colour entries), not one representative.
//!
//! **Tier-A only** — synthetic fixtures, no corpus, no tools.

#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, to_srgb_linear_into,
};
use irradiance::demosaic::{demosaic_into, demosaic_into_threaded};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::opcode::{check_required_processing, DevelopPath};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

use opcode_support::{opcode_list, raw_opcode, warp_rectilinear_params};
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Opcode builders. Flags bit 0 set = optional (DNG 1.7.0.0 Ch.7, p.101).
// ─────────────────────────────────────────────────────────────────────────────

const MANDATORY: u32 = 0;
const OPTIONAL: u32 = 1;
const ID_WARP: u32 = 1;
const ID_FBPC: u32 = 4;
const ID_UNKNOWN: u32 = 999;

/// `FixBadPixelsConstant` with `Constant = 0` — no fixture sample is 0, so an
/// applied instance changes nothing and the output is comparable.
fn fbpc(flags: u32) -> Vec<u8> {
    let mut params = Vec::new();
    params.extend_from_slice(&0u32.to_be_bytes());
    params.extend_from_slice(&0u32.to_be_bytes());
    raw_opcode(ID_FBPC, 0x0103_0000, flags, &params)
}

/// An identity `WarpRectilinear` (`kr = [1, 0, 0, 0]`), which the mono applier
/// recognizes as a no-op.
fn warp(flags: u32) -> Vec<u8> {
    let params = warp_rectilinear_params([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5, 1);
    raw_opcode(ID_WARP, 0x0104_0000, flags, &params)
}

fn unknown(flags: u32) -> Vec<u8> {
    raw_opcode(ID_UNKNOWN, 0x0104_0000, flags, &[1, 2, 3, 4])
}

fn list(ops: &[Vec<u8>]) -> Option<Vec<u8>> {
    Some(opcode_list(ops))
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures: an 8×8 16-bit plane, mono or 2×2-RGGB, with the given opcode lists.
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 8;
const H: u32 = 8;

fn samples() -> Vec<u16> {
    (0..W * H).map(|i| 1000 + (i as u16) * 37).collect()
}

/// Parse the fixture DNG and unpack its plane. Colour fixtures get the colour
/// calibration the colour develop needs set on the parsed `Sensor` (a sealed
/// struct's public fields stay assignable).
fn fixture_bytes(cfa: bool, lists: &[Option<Vec<u8>>; 3]) -> Vec<u8> {
    let [l1, l2, l3] = lists;
    tiff::dng_16bit_with_opcode_lists(
        Order::Little,
        cfa,
        W,
        H,
        &samples(),
        [l1.as_deref(), l2.as_deref(), l3.as_deref()],
    )
}

fn fixture(cfa: bool, lists: [Option<Vec<u8>>; 3]) -> (Sensor, Vec<u16>) {
    fixture_from_bytes(cfa, &fixture_bytes(cfa, &lists))
}

/// [`fixture`] over already-built DNG bytes (e.g. one whose opcode-list entry
/// was made unreadable by [`make_list_unreadable`]).
fn fixture_from_bytes(cfa: bool, data: &[u8]) -> (Sensor, Vec<u16>) {
    let data = data.to_vec();
    let container = Container::parse(&data).expect("fixture parses");
    let mut sensor = container.sensor().expect("fixture has a sensor IFD");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane).expect("fixture unpacks");
    if cfa {
        sensor.forward_matrix_1 = Some(
            [
                [60000, 30000, 6422],
                [20000, 70000, 10000],
                [10000, 10000, 62521],
            ]
            .iter()
            .flatten()
            .map(|&num| Rational { num, den: 100000 })
            .collect(),
        );
        sensor.as_shot_neutral = Some(vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ]);
    }
    (sensor, plane)
}

fn out_len(sensor: &Sensor, channels: usize) -> usize {
    let (w, h) = output_dimensions(sensor).expect("fixture geometry is valid");
    w as usize * h as usize * channels
}

fn mono(lists: [Option<Vec<u8>>; 3]) -> Result<(), Error> {
    let (sensor, plane) = fixture(false, lists);
    let mut dst = vec![0u16; out_len(&sensor, 1)];
    develop_into(&sensor, &plane, &mut dst)
}

/// The four public colour develop entries, each its own door (§12).
#[derive(Debug, Clone, Copy)]
enum ColourEntry {
    U16,
    F32,
    U16Threaded,
    F32Threaded,
}

const COLOUR_ENTRIES: [ColourEntry; 4] = [
    ColourEntry::U16,
    ColourEntry::F32,
    ColourEntry::U16Threaded,
    ColourEntry::F32Threaded,
];

fn colour(entry: ColourEntry, lists: [Option<Vec<u8>>; 3]) -> Result<(), Error> {
    let (sensor, plane) = fixture(true, lists);
    let n = out_len(&sensor, 3);
    match entry {
        ColourEntry::U16 => develop_color_into(&sensor, &plane, &mut vec![0u16; n]),
        ColourEntry::F32 => develop_color_f32_into(&sensor, &plane, &mut vec![0f32; n]),
        ColourEntry::U16Threaded => {
            develop_color_into_threaded(&sensor, &plane, &mut vec![0u16; n], 4)
        }
        ColourEntry::F32Threaded => {
            develop_color_f32_into_threaded(&sensor, &plane, &mut vec![0f32; n], 4)
        }
    }
}

// Shown red first (DEC-009): run against the pre-fix tree with plain
// `is_err()` assertions, all 12 required-processing tests failed with
// `Ok(())` while all 5 controls passed — the silently-wrong develop the review
// named. `required_processing_red_proof` below keeps that red reproducible.

#[track_caller]
fn assert_required(r: Result<(), Error>, list: u8, id: u32) {
    assert!(
        matches!(r, Err(Error::UnsupportedRequiredOpcode { list: l, id: i }) if l == list && i == id),
        "expected UnsupportedRequiredOpcode {{ list: {list}, id: {id} }}, got {r:?}"
    );
}

#[track_caller]
fn assert_mandatory_unknown(r: Result<(), Error>, id: u32) {
    assert!(
        matches!(r, Err(Error::UnsupportedMandatoryOpcode { id: i }) if i == id),
        "expected UnsupportedMandatoryOpcode {{ id: {id} }}, got {r:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Controls — must stay Ok (DEC-009: green before any red means anything).
// ─────────────────────────────────────────────────────────────────────────────

/// The Leica Q2-Mono shape: one mandatory `FixBadPixelsConstant` in list 1 and
/// one mandatory `WarpRectilinear` in list 3 — both applied on the mono path.
#[test]
fn mono_applied_mandatory_opcodes_still_develop() {
    mono([list(&[fbpc(MANDATORY)]), None, list(&[warp(MANDATORY)])]).expect("Q2-Mono shape");
}

#[test]
fn mono_optional_only_opcodes_in_every_list_still_develop() {
    let opt = || list(&[fbpc(OPTIONAL), warp(OPTIONAL), unknown(OPTIONAL)]);
    mono([opt(), opt(), opt()]).expect("optional opcodes are skippable");
}

/// An optional duplicate beyond the applied instance is skippable.
#[test]
fn mono_optional_duplicate_after_the_applied_instance_still_develops() {
    mono([list(&[fbpc(MANDATORY), fbpc(OPTIONAL)]), None, None]).expect("optional dup");
}

#[test]
fn colour_optional_only_opcodes_still_develop_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        let opt = || list(&[fbpc(OPTIONAL), warp(OPTIONAL), unknown(OPTIONAL)]);
        colour(entry, [opt(), opt(), opt()])
            .unwrap_or_else(|e| panic!("{entry:?}: optional-only must develop, got {e:?}"));
    }
}

#[test]
fn opcode_free_files_still_develop_on_every_entry() {
    mono([None, None, None]).expect("mono, no opcode lists");
    for entry in COLOUR_ENTRIES {
        colour(entry, [None, None, None])
            .unwrap_or_else(|e| panic!("{entry:?}: no opcode lists must develop, got {e:?}"));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The mono path (`develop_into`).
// ─────────────────────────────────────────────────────────────────────────────

/// Primary oracle: `OpcodeList2` is applied by no path, so a mandatory opcode
/// there is unsupported required processing.
#[test]
fn mono_mandatory_opcode_in_list_2_is_rejected() {
    assert_required(mono([None, list(&[fbpc(MANDATORY)]), None]), 2, ID_FBPC);
}

/// The `find_map` gap: the applier consumes only the FIRST instance, so a
/// second mandatory one would be silently dropped.
#[test]
fn mono_second_mandatory_fix_bad_pixels_in_list_1_is_rejected() {
    assert_required(
        mono([list(&[fbpc(MANDATORY), fbpc(MANDATORY)]), None, None]),
        1,
        ID_FBPC,
    );
}

/// The first instance is the applied one even when it is optional, so a
/// mandatory instance behind it is not applied.
#[test]
fn mono_mandatory_fix_bad_pixels_behind_an_optional_one_is_rejected() {
    assert_required(
        mono([list(&[fbpc(OPTIONAL), fbpc(MANDATORY)]), None, None]),
        1,
        ID_FBPC,
    );
}

#[test]
fn mono_second_mandatory_warp_in_list_3_is_rejected() {
    assert_required(
        mono([None, None, list(&[warp(MANDATORY), warp(MANDATORY)])]),
        3,
        ID_WARP,
    );
}

/// A recognized opcode in a list its applier does not read.
#[test]
fn mono_mandatory_recognized_opcode_in_the_wrong_list_is_rejected() {
    assert_required(mono([None, list(&[warp(MANDATORY)]), None]), 2, ID_WARP);
    assert_required(mono([list(&[warp(MANDATORY)]), None, None]), 1, ID_WARP);
    assert_required(mono([None, None, list(&[fbpc(MANDATORY)])]), 3, ID_FBPC);
}

/// An unrecognized mandatory id keeps its existing error, now in list 2 too.
#[test]
fn mono_mandatory_unknown_opcode_in_list_2_is_rejected() {
    assert_mandatory_unknown(mono([None, list(&[unknown(MANDATORY)]), None]), ID_UNKNOWN);
}

/// Framing errors in `OpcodeList2` surface instead of being dropped with it.
#[test]
fn mono_truncated_list_2_is_rejected() {
    let mut bytes = opcode_list(&[fbpc(OPTIONAL)]);
    bytes.truncate(bytes.len() - 1);
    let r = mono([None, Some(bytes), None]);
    assert!(
        matches!(r, Err(Error::Truncated { .. })),
        "a truncated OpcodeList2 must not develop: {r:?}"
    );
}

/// The colour path reads every present list in full, too.
#[test]
fn colour_truncated_list_is_rejected_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        let mut bytes = opcode_list(&[warp(OPTIONAL)]);
        bytes.truncate(bytes.len() - 1);
        let r = colour(entry, [None, None, Some(bytes)]);
        assert!(
            matches!(r, Err(Error::Truncated { .. })),
            "{entry:?}: a truncated OpcodeList3 must not develop: {r:?}"
        );
    }
}

/// `OpcodeList2`'s bytes reach the `Sensor` through the real container read
/// (`DEC-012` parity with lists 1 and 3).
#[test]
fn opcode_list_2_bytes_are_kept_on_read() {
    let l2 = opcode_list(&[fbpc(OPTIONAL)]);
    let (sensor, _) = fixture(false, [None, Some(l2.clone()), None]);
    assert_eq!(sensor.opcode_list_2.as_deref(), Some(l2.as_slice()));
    assert_eq!(sensor.opcode_lists, [false, true, false]);
    assert!(sensor.malformed_tags.is_empty());
}

#[test]
fn unsupported_required_opcode_display_names_the_list_and_id() {
    let msg = Error::UnsupportedRequiredOpcode { list: 2, id: 4 }.to_string();
    assert!(msg.contains("OpcodeList2"), "{msg}");
    assert!(msg.contains("opcode 4"), "{msg}");
}

// ─────────────────────────────────────────────────────────────────────────────
// The colour path — applies nothing, so ANY mandatory opcode is rejected, on
// every public entry.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn colour_mandatory_opcode_in_list_1_is_rejected_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        assert_required(
            colour(entry, [list(&[fbpc(MANDATORY)]), None, None]),
            1,
            ID_FBPC,
        );
    }
}

#[test]
fn colour_mandatory_opcode_in_list_2_is_rejected_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        assert_required(
            colour(entry, [None, list(&[fbpc(MANDATORY)]), None]),
            2,
            ID_FBPC,
        );
    }
}

#[test]
fn colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        assert_required(
            colour(entry, [None, None, list(&[warp(MANDATORY)])]),
            3,
            ID_WARP,
        );
    }
}

/// A mandatory opcode behind optional ones in the same list is still found.
#[test]
fn colour_mandatory_opcode_after_optional_ones_is_rejected() {
    for entry in COLOUR_ENTRIES {
        let l1 = list(&[fbpc(OPTIONAL), unknown(OPTIONAL), warp(MANDATORY)]);
        assert_required(colour(entry, [l1, None, None]), 1, ID_WARP);
    }
}

#[test]
fn colour_mandatory_unknown_opcode_is_rejected_on_every_entry() {
    for entry in COLOUR_ENTRIES {
        assert_mandatory_unknown(
            colour(entry, [None, None, list(&[unknown(MANDATORY)])]),
            ID_UNKNOWN,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The red-proof (oracle-must-be-shown-red, DEC-009)
// ─────────────────────────────────────────────────────────────────────────────
//
// A copy of the crate is built in a temp dir with a small probe binary that
// parses a fixture DNG, develops it through one named public entry, and prints
// `ok` or `err <Debug>`. The CONTROL copy is unmutated and must reproduce the
// honest tree's typed errors; the MUTANT copy has the required-processing check
// short-circuited to `Ok(())` and must develop every fixture silently — the
// pre-fix behaviour the review named. Mirrors `tests/ifd_resource_guard.rs`'s
// `stage_probe_crate` (copied, not shared, per the repo's red-proof shape).
//
// The probe uses only API that existed before SPEC-054 (parse, sensor, unpack,
// public-field assignment, the five develop entries), so the same probe source
// also stood for the "pre-fix tree" run recorded at the top of this file.

use std::path::{Path, PathBuf};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-required-opcode-{label}-{}-{nanos}",
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
        if entry.file_type().expect("file_type").is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {}: {e}", target.display()));
        }
    }
}

/// The first statement of `check_lists` (the body of the public
/// `check_required_processing`) in `src/opcode.rs`. If it moves, the red-proof
/// fails loudly rather than silently mutating nothing. Since `SPEC-056` the
/// same loop also holds the unreadable-list refusal, so one injection disables
/// both halves of the check.
const CHECK_NEEDLE: &str =
    "    for (list, (bytes, unreadable)) in (1u8..=3).zip(lists.into_iter().zip(unreadable)) {";

/// Disable the check (`DEC-049`, `DEC-053`): it returns `Ok(())` before
/// reading any list or unreadable flag, leaving the rest of the pipeline
/// untouched.
fn inject_check_removed(opcode_rs: &Path) {
    let src = std::fs::read_to_string(opcode_rs)
        .unwrap_or_else(|e| panic!("read {}: {e}", opcode_rs.display()));
    let occurrences = src.matches(CHECK_NEEDLE).count();
    assert_eq!(
        occurrences, 1,
        "expected exactly one required-processing loop in src/opcode.rs; found {occurrences} — \
         the check moved, update CHECK_NEEDLE"
    );
    let mutated = src.replacen(
        CHECK_NEEDLE,
        &format!(
            "    if true {{ return Ok(()); }} // RED-PROOF INJECTION -- tests/required_processing.rs\n{CHECK_NEEDLE}"
        ),
        1,
    );
    assert_ne!(mutated, src, "the injection must change the source");
    std::fs::write(opcode_rs, mutated)
        .unwrap_or_else(|e| panic!("write mutated {}: {e}", opcode_rs.display()));
}

const PROBE_MAIN: &str = r#"
use irradiance::ifd::Rational;

fn main() {
    let mut args = std::env::args().skip(1);
    let entry = args.next().expect("entry name");
    let path = args.next().expect("fixture path");
    let data = std::fs::read(&path).expect("read fixture");
    let container = irradiance::ifd::Container::parse(&data).expect("parse");
    let mut sensor = container.sensor().expect("sensor");
    let n = sensor.width as usize * sensor.height as usize;
    let mut plane = vec![0u16; n];
    irradiance::plane::unpack_into(&sensor, container.byte_order(), &data, &mut plane)
        .expect("unpack");
    if sensor.photometric == 32803 {
        sensor.forward_matrix_1 = Some(
            [60000, 30000, 6422, 20000, 70000, 10000, 10000, 10000, 62521]
                .iter()
                .map(|&num| Rational { num, den: 100000 })
                .collect(),
        );
        sensor.as_shot_neutral = Some(vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ]);
    }
    let (w, h) = irradiance::develop::output_dimensions(&sensor).expect("geometry");
    let px = w as usize * h as usize;
    let r = match entry.as_str() {
        "develop_into" => irradiance::develop::develop_into(&sensor, &plane, &mut vec![0u16; px]),
        "develop_color_into" => {
            irradiance::color::develop_color_into(&sensor, &plane, &mut vec![0u16; px * 3])
        }
        "develop_color_f32_into" => {
            irradiance::color::develop_color_f32_into(&sensor, &plane, &mut vec![0f32; px * 3])
        }
        "develop_color_into_threaded" => irradiance::color::develop_color_into_threaded(
            &sensor, &plane, &mut vec![0u16; px * 3], 4,
        ),
        "develop_color_f32_into_threaded" => irradiance::color::develop_color_f32_into_threaded(
            &sensor, &plane, &mut vec![0f32; px * 3], 4,
        ),
        // SPEC-056: the colour stage primitives, on the raw plane's own size.
        "demosaic_into" => {
            let n = sensor.width as usize * sensor.height as usize * 3;
            irradiance::demosaic::demosaic_into(&sensor, &plane, &mut vec![0u16; n])
        }
        "demosaic_into_threaded" => {
            let n = sensor.width as usize * sensor.height as usize * 3;
            irradiance::demosaic::demosaic_into_threaded(&sensor, &plane, &mut vec![0u16; n], 4)
        }
        "to_srgb_linear_into" => {
            let n = sensor.width as usize * sensor.height as usize * 3;
            let camera: Vec<u16> = (0..n).map(|i| (i % 4096) as u16).collect();
            irradiance::color::to_srgb_linear_into(&sensor, &camera, &mut vec![0u16; n])
        }
        other => panic!("unknown entry {other}"),
    };
    match r {
        Ok(()) => println!("ok"),
        Err(e) => println!("err {e:?}"),
    }
}
"#;

fn stage_probe_crate(dest: &Path, mutate: bool) {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(dest.join("src/bin")).expect("mkdir src/bin");
    std::fs::copy(repo_root.join("Cargo.toml"), dest.join("Cargo.toml")).expect("copy Cargo.toml");
    let lock = repo_root.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, dest.join("Cargo.lock")).expect("copy Cargo.lock");
    }
    copy_dir_recursive(&repo_root.join("src"), &dest.join("src"));
    if mutate {
        inject_check_removed(&dest.join("src/opcode.rs"));
    }
    std::fs::write(dest.join("src/bin/required_opcode_probe.rs"), PROBE_MAIN)
        .expect("write probe binary source");
    let mut cargo_toml =
        std::fs::read_to_string(dest.join("Cargo.toml")).expect("read staged Cargo.toml");
    cargo_toml.push_str(
        "\n[[bin]]\nname = \"required_opcode_probe\"\npath = \"src/bin/required_opcode_probe.rs\"\n",
    );
    std::fs::write(dest.join("Cargo.toml"), cargo_toml)
        .expect("append [[bin]] to staged Cargo.toml");
}

fn build_probe(dir: &Path) -> PathBuf {
    let build = std::process::Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "required_opcode_probe",
            "--quiet",
        ])
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
    dir.join("target/release/required_opcode_probe")
}

fn run_probe(bin: &Path, entry: &str, fixture: &Path) -> String {
    let run = std::process::Command::new(bin)
        .args([entry.as_ref(), fixture.as_os_str()])
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()));
    assert!(
        run.status.success(),
        "required_opcode_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout)
        .expect("probe stdout is not UTF-8")
        .trim()
        .to_string()
}

#[test]
fn required_processing_red_proof() {
    let fixtures_dir = TempDir::new("fixtures");
    let write = |name: &str, cfa: bool, lists: [Option<Vec<u8>>; 3]| -> PathBuf {
        let path = fixtures_dir.0.join(name);
        std::fs::write(&path, fixture_bytes(cfa, &lists)).expect("write fixture");
        path
    };
    let control_mono = write(
        "control-mono.dng",
        false,
        [
            list(&[fbpc(MANDATORY)]),
            list(&[fbpc(OPTIONAL)]),
            list(&[warp(MANDATORY)]),
        ],
    );
    let control_colour = write(
        "control-colour.dng",
        true,
        [
            list(&[fbpc(OPTIONAL)]),
            list(&[warp(OPTIONAL)]),
            list(&[unknown(OPTIONAL)]),
        ],
    );
    let mono_list_2 = write(
        "mono-list-2.dng",
        false,
        [None, list(&[fbpc(MANDATORY)]), None],
    );
    let mono_dup = write(
        "mono-duplicate.dng",
        false,
        [list(&[fbpc(MANDATORY), fbpc(MANDATORY)]), None, None],
    );
    let colour_l1 = write(
        "colour-list-1.dng",
        true,
        [list(&[fbpc(MANDATORY)]), None, None],
    );

    // SPEC-056: a present-but-unreadable list (its lost content a mandatory
    // unknown opcode), on list 2 for the mono path and list 1 for colour.
    let write_unreadable = |name: &str, cfa: bool, unreadable: u8| -> PathBuf {
        let lists: [Option<Vec<u8>>; 3] = std::array::from_fn(|i| {
            if i + 1 == usize::from(unreadable) {
                list(&[unknown(MANDATORY)])
            } else {
                None
            }
        });
        let mut data = fixture_bytes(cfa, &lists);
        make_list_unreadable(&mut data, unreadable);
        let path = fixtures_dir.0.join(name);
        std::fs::write(&path, data).expect("write fixture");
        path
    };
    let mono_unreadable_l2 = write_unreadable("mono-unreadable-list-2.dng", false, 2);
    let colour_unreadable_l1 = write_unreadable("colour-unreadable-list-1.dng", true, 1);
    const PRIMITIVES: [&str; 3] = [
        "demosaic_into",
        "demosaic_into_threaded",
        "to_srgb_linear_into",
    ];

    const COLOUR: [&str; 4] = [
        "develop_color_into",
        "develop_color_f32_into",
        "develop_color_into_threaded",
        "develop_color_f32_into_threaded",
    ];
    // (entry, fixture, honest answer)
    let mut cases: Vec<(&str, &Path, String)> = vec![
        (
            "develop_into",
            &mono_list_2,
            "err UnsupportedRequiredOpcode { list: 2, id: 4 }".to_string(),
        ),
        (
            "develop_into",
            &mono_dup,
            "err UnsupportedRequiredOpcode { list: 1, id: 4 }".to_string(),
        ),
    ];
    for entry in COLOUR {
        cases.push((
            entry,
            &colour_l1,
            "err UnsupportedRequiredOpcode { list: 1, id: 4 }".to_string(),
        ));
    }
    cases.push((
        "develop_into",
        &mono_unreadable_l2,
        "err UnreadableOpcodeList { list: 2 }".to_string(),
    ));
    for entry in COLOUR {
        cases.push((
            entry,
            &colour_unreadable_l1,
            "err UnreadableOpcodeList { list: 1 }".to_string(),
        ));
    }
    for entry in PRIMITIVES {
        cases.push((
            entry,
            &colour_l1,
            "err UnsupportedRequiredOpcode { list: 1, id: 4 }".to_string(),
        ));
        cases.push((
            entry,
            &colour_unreadable_l1,
            "err UnreadableOpcodeList { list: 1 }".to_string(),
        ));
    }

    // CONTROL first (DEC-009): the unmutated copy must reproduce the honest
    // tree — the optional/applied-only controls develop, every required case
    // is the typed error — so a red below is attributable to the injection.
    let control_dir = TempDir::new("control");
    stage_probe_crate(&control_dir.0, false);
    let control_bin = build_probe(&control_dir.0);
    assert_eq!(run_probe(&control_bin, "develop_into", &control_mono), "ok");
    for entry in COLOUR.into_iter().chain(PRIMITIVES) {
        assert_eq!(
            run_probe(&control_bin, entry, &control_colour),
            "ok",
            "{entry}"
        );
    }
    for (entry, fixture, honest) in &cases {
        assert_eq!(
            &run_probe(&control_bin, entry, fixture),
            honest,
            "control {entry}"
        );
    }

    // MUTANT: check disabled. Every required case must now develop to a silent
    // `ok` — the review's plausible-looking wrong image.
    let mutant_dir = TempDir::new("mutant");
    stage_probe_crate(&mutant_dir.0, true);
    let mutant_bin = build_probe(&mutant_dir.0);
    for (entry, fixture, honest) in &cases {
        let mutant = run_probe(&mutant_bin, entry, fixture);
        assert_ne!(
            &mutant, honest,
            "disabling the check did NOT change {entry}'s output — the injection is a no-op and \
             this red-proof has caught NOTHING"
        );
        assert_eq!(
            mutant, "ok",
            "{entry}: with the check disabled the required opcode must be silently dropped"
        );
        eprintln!(
            "RED-PROOF ({entry}, {}): control={honest:?} mutant={mutant:?}",
            fixture.display()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-056 / DEC-053 §1 — an OpcodeList that is PRESENT but UNREADABLE refuses
// the develop with `Error::UnreadableOpcodeList { list }` (kind `Corrupt`),
// while the read itself stays tolerant (DEC-012): the file parses, the field is
// `None`, and the tag is in `malformed_tags`.
// ─────────────────────────────────────────────────────────────────────────────

const LIST_TAGS: [u16; 3] = [51008, 51009, 51022];

/// Rewrite the IFD0 entry for `OpcodeList{list}` so its payload offset runs far
/// past EOF: `payload()` then fails, so the reader records the tag in
/// `malformed_tags` and leaves the field `None`. The fixture is little-endian
/// with IFD0 at offset 8 (`tiff::dng_16bit_with_opcode_lists`).
fn make_list_unreadable(data: &mut [u8], list: u8) {
    let tag = LIST_TAGS[usize::from(list - 1)];
    let count = usize::from(u16::from_le_bytes([data[8], data[9]]));
    let mut patched = 0;
    for i in 0..count {
        let at = 10 + i * 12;
        if u16::from_le_bytes([data[at], data[at + 1]]) == tag {
            data[at + 8..at + 12].copy_from_slice(&0xFFFF_FF00u32.to_le_bytes());
            patched += 1;
        }
    }
    assert_eq!(patched, 1, "OpcodeList{list} entry must exist exactly once");
}

/// A fixture whose list `unreadable` is present but unreadable. That list's
/// (lost) content is a MANDATORY unknown opcode — the silent-wrong case: were
/// its bytes readable, every path would refuse it. Every other list is given
/// optional-only content, which every path accepts.
fn unreadable_fixture(cfa: bool, unreadable: u8) -> (Sensor, Vec<u16>) {
    let lists: [Option<Vec<u8>>; 3] = std::array::from_fn(|i| {
        if i + 1 == usize::from(unreadable) {
            list(&[unknown(MANDATORY)])
        } else {
            list(&[fbpc(OPTIONAL), warp(OPTIONAL)])
        }
    });
    let mut data = fixture_bytes(cfa, &lists);
    make_list_unreadable(&mut data, unreadable);
    let (sensor, plane) = fixture_from_bytes(cfa, &data);
    // Guard the fixture itself: it must be "present but unreadable", never a
    // quietly absent tag.
    let tag = LIST_TAGS[usize::from(unreadable - 1)];
    assert!(
        sensor.malformed_tags.contains(&tag),
        "fixture: tag {tag} must be in malformed_tags, got {:?}",
        sensor.malformed_tags
    );
    assert!(
        sensor.opcode_lists[usize::from(unreadable - 1)],
        "tag present"
    );
    let field = match unreadable {
        1 => &sensor.opcode_list_1,
        2 => &sensor.opcode_list_2,
        _ => &sensor.opcode_list_3,
    };
    assert!(field.is_none(), "an unreadable list's field is None");
    (sensor, plane)
}

#[track_caller]
fn assert_unreadable(r: Result<(), Error>, list: u8) {
    assert!(
        matches!(r, Err(Error::UnreadableOpcodeList { list: l }) if l == list),
        "expected UnreadableOpcodeList {{ list: {list} }}, got {r:?}"
    );
}

fn mono_sensor(sensor: &Sensor, plane: &[u16]) -> Result<(), Error> {
    develop_into(sensor, plane, &mut vec![0u16; out_len(sensor, 1)])
}

fn colour_sensor(entry: ColourEntry, sensor: &Sensor, plane: &[u16]) -> Result<(), Error> {
    let n = out_len(sensor, 3);
    match entry {
        ColourEntry::U16 => develop_color_into(sensor, plane, &mut vec![0u16; n]),
        ColourEntry::F32 => develop_color_f32_into(sensor, plane, &mut vec![0f32; n]),
        ColourEntry::U16Threaded => {
            develop_color_into_threaded(sensor, plane, &mut vec![0u16; n], 4)
        }
        ColourEntry::F32Threaded => {
            develop_color_f32_into_threaded(sensor, plane, &mut vec![0f32; n], 4)
        }
    }
}

/// The read stays tolerant: the file still parses and unpacks (DEC-012).
#[test]
fn unreadable_opcode_list_still_parses() {
    for list in 1..=3 {
        for cfa in [false, true] {
            let _ = unreadable_fixture(cfa, list);
        }
    }
}

/// Per-list, mono path: lists 1, 2 and 3 each refuse `develop_into`.
#[test]
fn mono_unreadable_opcode_list_is_refused_per_list() {
    for list in 1..=3 {
        let (sensor, plane) = unreadable_fixture(false, list);
        assert_unreadable(mono_sensor(&sensor, &plane), list);
    }
}

/// Every colour develop entry refuses every unreadable list.
#[test]
fn colour_unreadable_opcode_list_is_refused_on_every_entry() {
    for list in 1..=3 {
        let (sensor, plane) = unreadable_fixture(true, list);
        for entry in COLOUR_ENTRIES {
            assert_unreadable(colour_sensor(entry, &sensor, &plane), list);
        }
    }
}

/// Deterministic order: with lists 1 and 3 both unreadable, list 1 is reported.
#[test]
fn the_lowest_unreadable_list_is_reported_first() {
    let lists = [list(&[fbpc(OPTIONAL)]), None, list(&[warp(OPTIONAL)])];
    let mut data = fixture_bytes(false, &lists);
    make_list_unreadable(&mut data, 1);
    make_list_unreadable(&mut data, 3);
    let (sensor, plane) = fixture_from_bytes(false, &data);
    assert_unreadable(mono_sensor(&sensor, &plane), 1);
}

/// The unreadable check for list n comes before list n's scan, but a list
/// earlier in order is scanned first: a mandatory unsupported opcode in a
/// readable list 2 wins over an unreadable list 3.
#[test]
fn an_earlier_lists_scan_error_precedes_a_later_unreadable_list() {
    let lists = [None, list(&[fbpc(MANDATORY)]), list(&[warp(OPTIONAL)])];
    let mut data = fixture_bytes(false, &lists);
    make_list_unreadable(&mut data, 3);
    let (sensor, plane) = fixture_from_bytes(false, &data);
    assert_required(mono_sensor(&sensor, &plane), 2, ID_FBPC);
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-056 / DEC-053 §2 — the Sensor-taking stage primitives run the colour
// path's check first. One test per primitive per case (mandatory, unreadable,
// optional-only, opcode-free).
// ─────────────────────────────────────────────────────────────────────────────

/// The three primitives, each its own door (§12).
#[derive(Debug, Clone, Copy)]
enum Primitive {
    Demosaic,
    DemosaicThreaded,
    ToSrgbLinear,
}

fn primitive(p: Primitive, sensor: &Sensor, plane: &[u16]) -> Result<(), Error> {
    let n = sensor.width as usize * sensor.height as usize * 3;
    match p {
        Primitive::Demosaic => demosaic_into(sensor, plane, &mut vec![0u16; n]),
        Primitive::DemosaicThreaded => demosaic_into_threaded(sensor, plane, &mut vec![0u16; n], 4),
        Primitive::ToSrgbLinear => {
            // Camera-space RGB input of the right length; values are irrelevant.
            let camera: Vec<u16> = (0..n).map(|i| (i % 4096) as u16).collect();
            to_srgb_linear_into(sensor, &camera, &mut vec![0u16; n])
        }
    }
}

fn primitive_mandatory(p: Primitive) {
    let (sensor, plane) = fixture(true, [list(&[fbpc(MANDATORY)]), None, None]);
    assert_required(primitive(p, &sensor, &plane), 1, ID_FBPC);
}

fn primitive_unreadable(p: Primitive) {
    let (sensor, plane) = unreadable_fixture(true, 2);
    assert_unreadable(primitive(p, &sensor, &plane), 2);
}

fn primitive_optional_only(p: Primitive) {
    let opt = || list(&[fbpc(OPTIONAL), warp(OPTIONAL), unknown(OPTIONAL)]);
    let (sensor, plane) = fixture(true, [opt(), opt(), opt()]);
    primitive(p, &sensor, &plane)
        .unwrap_or_else(|e| panic!("{p:?}: optional-only must succeed, got {e:?}"));
}

fn primitive_opcode_free(p: Primitive) {
    let (sensor, plane) = fixture(true, [None, None, None]);
    primitive(p, &sensor, &plane)
        .unwrap_or_else(|e| panic!("{p:?}: opcode-free must succeed, got {e:?}"));
}

#[test]
fn demosaic_into_refuses_a_mandatory_opcode() {
    primitive_mandatory(Primitive::Demosaic);
}
#[test]
fn demosaic_into_refuses_an_unreadable_list() {
    primitive_unreadable(Primitive::Demosaic);
}
#[test]
fn demosaic_into_accepts_optional_only() {
    primitive_optional_only(Primitive::Demosaic);
}
#[test]
fn demosaic_into_accepts_opcode_free() {
    primitive_opcode_free(Primitive::Demosaic);
}

#[test]
fn demosaic_into_threaded_refuses_a_mandatory_opcode() {
    primitive_mandatory(Primitive::DemosaicThreaded);
}
#[test]
fn demosaic_into_threaded_refuses_an_unreadable_list() {
    primitive_unreadable(Primitive::DemosaicThreaded);
}
#[test]
fn demosaic_into_threaded_accepts_optional_only() {
    primitive_optional_only(Primitive::DemosaicThreaded);
}
#[test]
fn demosaic_into_threaded_accepts_opcode_free() {
    primitive_opcode_free(Primitive::DemosaicThreaded);
}

#[test]
fn to_srgb_linear_into_refuses_a_mandatory_opcode() {
    primitive_mandatory(Primitive::ToSrgbLinear);
}
#[test]
fn to_srgb_linear_into_refuses_an_unreadable_list() {
    primitive_unreadable(Primitive::ToSrgbLinear);
}
#[test]
fn to_srgb_linear_into_accepts_optional_only() {
    primitive_optional_only(Primitive::ToSrgbLinear);
}
#[test]
fn to_srgb_linear_into_accepts_opcode_free() {
    primitive_opcode_free(Primitive::ToSrgbLinear);
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-056 — the public check and the error's shape.
// ─────────────────────────────────────────────────────────────────────────────

/// `opcode::check_required_processing` is callable from outside the crate and
/// answers exactly what the matching develop answers, across the SPEC-054
/// fixture shapes plus the unreadable ones.
#[test]
fn public_check_agrees_with_every_develop_entry() {
    let shapes: Vec<[Option<Vec<u8>>; 3]> = vec![
        [None, None, None],
        [list(&[fbpc(MANDATORY)]), None, list(&[warp(MANDATORY)])],
        [
            list(&[fbpc(OPTIONAL), warp(OPTIONAL), unknown(OPTIONAL)]),
            None,
            None,
        ],
        [None, list(&[fbpc(MANDATORY)]), None],
        [list(&[fbpc(MANDATORY), fbpc(MANDATORY)]), None, None],
        [None, None, list(&[unknown(MANDATORY)])],
        [None, None, list(&[fbpc(MANDATORY)])],
    ];
    let mut cases: Vec<(bool, Sensor, Vec<u16>)> = Vec::new();
    for cfa in [false, true] {
        for lists in &shapes {
            let (sensor, plane) = fixture(cfa, lists.clone());
            cases.push((cfa, sensor, plane));
        }
        for unreadable in 1..=3 {
            let (sensor, plane) = unreadable_fixture(cfa, unreadable);
            cases.push((cfa, sensor, plane));
        }
    }
    let mut refused = 0;
    for (cfa, sensor, plane) in &cases {
        if *cfa {
            let check = format!(
                "{:?}",
                check_required_processing(sensor, DevelopPath::Colour)
            );
            for entry in COLOUR_ENTRIES {
                let dev = format!("{:?}", colour_sensor(entry, sensor, plane));
                assert_eq!(check, dev, "{entry:?}");
            }
            if check != "Ok(())" {
                refused += 1;
            }
        } else {
            let check = format!("{:?}", check_required_processing(sensor, DevelopPath::Mono));
            assert_eq!(check, format!("{:?}", mono_sensor(sensor, plane)), "mono");
            if check != "Ok(())" {
                refused += 1;
            }
        }
    }
    // Not vacuous: most shapes refuse on at least one path.
    assert!(
        refused >= 10,
        "only {refused} refusals — the agreement test is vacuous"
    );
}

/// `DevelopPath` is `#[non_exhaustive]` downstream: this match needs the
/// wildcard arm to compile (a missing one is error E0004 outside the crate).
#[test]
fn develop_path_is_public_and_non_exhaustive() {
    fn name(p: DevelopPath) -> &'static str {
        match p {
            DevelopPath::Mono => "mono",
            DevelopPath::Colour => "colour",
            _ => "other",
        }
    }
    assert_eq!(name(DevelopPath::Mono), "mono");
    assert_eq!(name(DevelopPath::Colour), "colour");
}

#[test]
fn unreadable_opcode_list_is_corrupt_and_displays_the_list() {
    for list in 1..=3u8 {
        let e = Error::UnreadableOpcodeList { list };
        assert_eq!(e.kind(), ErrorKind::Corrupt);
        let msg = e.to_string();
        assert!(msg.contains(&format!("OpcodeList{list}")), "{msg}");
        assert!(msg.contains("unreadable"), "{msg}");
    }
}
