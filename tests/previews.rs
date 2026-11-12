//! `SPEC-062` (`DEC-057`) — `Container::previews()`, the zero-copy preview
//! accessor.
//!
//! Tier A: hand-built TIFF fixtures from `tests/support/tiff.rs` cover the
//! selection rule (AC1, AC2), the zero-copy slice (AC3), strip joining (AC4)
//! and every hostile layout (AC5). Tier B: every held DNG's previews are
//! checked against the `[[file.preview]]` pins in `tests/corpus/manifest.toml`,
//! which were taken from exiftool 13.55, never from irradiance (AC7). The
//! red-proofs (AC8) mutate a staged copy of `src/` and show each mutation
//! changes the answer the named test asserts on.
//!
//! The allocation check (AC6) is its own test binary,
//! `tests/previews_alloc.rs`, so its counting allocator sees no other test.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tiff.rs"]
mod tiff;

use std::path::{Path, PathBuf};

use irradiance::ifd::{Compression, Container, Preview, PreviewKind};
use irradiance::{Error, ErrorKind, UnsupportedPreview};

use corpus::{CorpusRoot, Manifest};
use tiff::{
    at_offset, long, preview_chain, preview_contiguous_strips, preview_entries,
    preview_gapped_strips, preview_selection, preview_strips, preview_two, sensor_entries, short,
    with_entry, without_tag, Order, IMAGE_LENGTH, IMAGE_WIDTH, JPEG_INTERCHANGE_FORMAT,
    NEW_SUBFILE_TYPE, PREVIEW_DATA, STRIP_BYTE_COUNTS, STRIP_OFFSETS, TILE_OFFSETS,
};

const ORDERS: [Order; 2] = [Order::Little, Order::Big];

/// Drain `previews()`. Asserts the iterator's own bound on the way: it never
/// yields more items than the container has IFDs.
fn drain<'a>(c: &Container<'a>) -> Vec<Result<Preview<'a>, Error>> {
    let items: Vec<_> = c.previews().collect();
    assert!(
        items.len() <= c.ifds().len(),
        "previews() yielded {} items from {} IFDs",
        items.len(),
        c.ifds().len()
    );
    items
}

fn oks<'a>(c: &Container<'a>) -> Vec<Preview<'a>> {
    drain(c)
        .into_iter()
        .map(|r| r.unwrap_or_else(|e| panic!("unexpected Err item: {e}")))
        .collect()
}

/// AC3's invariant, for one preview: the slice IS the input at `offset`.
fn assert_subslice(input: &[u8], p: &Preview<'_>, what: &str) {
    let off = p.offset as usize;
    assert!(
        off + p.bytes.len() <= input.len(),
        "{what}: preview at {off}+{} runs past the {}-byte input",
        p.bytes.len(),
        input.len()
    );
    assert_eq!(
        p.bytes.as_ptr(),
        input.as_ptr().wrapping_add(off),
        "{what}: preview bytes are not the input at offset {off} (copied or shifted)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 / AC2 — selection
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn selection_is_exactly_nst_1_and_65537() {
    for order in ORDERS {
        let data = preview_selection(order);
        let c = Container::parse(&data).expect("selection fixture parses");
        assert_eq!(c.ifds().len(), 10);
        let got: Vec<(usize, PreviewKind)> =
            oks(&c).iter().map(|p| (p.ifd_index, p.kind)).collect();
        assert_eq!(
            got,
            vec![(2, PreviewKind::Primary), (8, PreviewKind::Alternate)],
            "{order:?}: only NewSubfileType 1 and 65537 are previews (DEC-057); absent, 0, 4, 5, \
             8, 9, 16 and 65540 are not"
        );
        for p in oks(&c) {
            assert_eq!((p.width, p.height), (2, 2));
            assert_eq!(p.compression, Compression::Uncompressed);
            assert_eq!(p.photometric, Some(2));
            assert_eq!(p.samples_per_pixel, 3);
            assert_eq!(p.bits_per_sample, Some(8));
            assert_eq!(p.bytes.len(), 12);
            assert_subslice(&data, &p, "selection");
        }
    }
}

#[test]
fn larger_preview_than_sensor_is_still_a_preview() {
    for order in ORDERS {
        // IFD0: a 64×64 RGB preview (12288 bytes). IFD1: the 4×2 14-bit
        // LinearRaw sensor plane (14 bytes, strip at 512). Size must not decide.
        let preview = preview_entries(order, Some(1), 64, 64, PREVIEW_DATA, 64 * 64 * 3);
        let data = preview_chain(
            order,
            &[preview, sensor_entries(order)],
            &[],
            PREVIEW_DATA + 64 * 64 * 3,
        );
        let c = Container::parse(&data).expect("fixture parses");
        let got = oks(&c);
        assert_eq!(
            got.len(),
            1,
            "{order:?}: exactly the preview, never the sensor"
        );
        assert_eq!(got[0].ifd_index, 0);
        assert_eq!((got[0].width, got[0].height), (64, 64));
        assert_eq!(c.sensor_candidates(), vec![1]);
        let sensor = c.sensor().expect("the sensor is still selected");
        assert_eq!((sensor.width, sensor.height), (4, 2));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 / AC4 — the slice
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn preview_bytes_are_a_subslice_of_the_input() {
    for order in ORDERS {
        for (name, data) in [
            ("selection", preview_selection(order)),
            ("two", preview_two(order)),
            ("contiguous", preview_contiguous_strips(order)),
        ] {
            let c = Container::parse(&data).expect("fixture parses");
            let got = oks(&c);
            assert!(!got.is_empty(), "{name}: no preview found");
            for p in &got {
                assert_subslice(&data, p, name);
            }
        }
        // Pin the two-preview fixture's exact ranges too, so a consistent
        // shift of offset AND slice would still be caught.
        let data = preview_two(order);
        let c = Container::parse(&data).expect("fixture parses");
        let ranges: Vec<(u32, usize)> = oks(&c).iter().map(|p| (p.offset, p.bytes.len())).collect();
        assert_eq!(ranges, vec![(PREVIEW_DATA, 12), (PREVIEW_DATA + 12, 48)]);
    }
}

#[test]
fn contiguous_strips_join_into_one_slice() {
    for order in ORDERS {
        let data = preview_contiguous_strips(order);
        let c = Container::parse(&data).expect("fixture parses");
        let got = oks(&c);
        assert_eq!(got.len(), 1);
        let p = &got[0];
        assert_eq!(p.offset, PREVIEW_DATA);
        assert_eq!(
            p.bytes.len(),
            10 + 15 + 20,
            "{order:?}: three contiguous strips are ONE slice spanning all three"
        );
        assert_subslice(&data, p, "contiguous");
        let start = PREVIEW_DATA as usize;
        assert_eq!(p.bytes, &data[start..start + 45]);
    }
}

#[test]
fn non_contiguous_strips_are_unsupported() {
    let d = PREVIEW_DATA;
    for order in ORDERS {
        let cases: [(&str, Vec<u8>); 3] = [
            ("gap", preview_gapped_strips(order)),
            (
                "overlap",
                preview_strips(order, &[d, d + 9, d + 24], &[10, 15, 20], d + 64),
            ),
            (
                "reversed",
                preview_strips(order, &[d + 35, d + 20, d], &[10, 15, 20], d + 64),
            ),
        ];
        for (name, data) in cases {
            let c = Container::parse(&data).expect("fixture parses");
            let items = drain(&c);
            assert_eq!(items.len(), 1, "{name}");
            match &items[0] {
                Err(Error::UnsupportedPreviewLayout {
                    ifd_index: 0,
                    layout: UnsupportedPreview::NonContiguousStrips,
                }) => {}
                other => panic!("{order:?} {name}: expected NonContiguousStrips, got {other:?}"),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — hostile layouts are typed, and iteration continues
// ─────────────────────────────────────────────────────────────────────────────

/// One single-IFD fixture whose only preview yields `Err`, and a check on it.
type Hostile = (&'static str, Vec<u8>, fn(&Error) -> bool);

fn hostile_cases(order: Order) -> Vec<Hostile> {
    let d = PREVIEW_DATA;
    let good = || preview_entries(order, Some(1), 2, 2, d, 12);
    let one = |entries: Vec<tiff::Entry>, arrays: &[(u32, Vec<u32>)], end: u32| {
        preview_chain(order, &[entries], arrays, end)
    };
    vec![
        (
            "tiled",
            one(with_entry(good(), long(TILE_OFFSETS, d)), &[], d + 16),
            |e| {
                matches!(
                    e,
                    Error::UnsupportedPreviewLayout {
                        ifd_index: 0,
                        layout: UnsupportedPreview::Tiled
                    }
                )
            },
        ),
        (
            "jpeg-interchange-format-only",
            one(
                with_entry(
                    without_tag(without_tag(good(), STRIP_OFFSETS), STRIP_BYTE_COUNTS),
                    long(JPEG_INTERCHANGE_FORMAT, d),
                ),
                &[],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::UnsupportedPreviewLayout {
                        ifd_index: 0,
                        layout: UnsupportedPreview::JpegInterchangeFormat
                    }
                )
            },
        ),
        (
            "count-mismatch",
            one(
                with_entry(good(), at_offset(STRIP_OFFSETS, 4, 2, 0x3000)),
                &[(0x3000, vec![d, d + 6])],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::InvalidPreviewLayout {
                        ifd_index: 0,
                        strip_offsets: 2,
                        strip_byte_counts: 1
                    }
                )
            },
        ),
        (
            "zero-strips",
            one(
                with_entry(
                    with_entry(good(), at_offset(STRIP_OFFSETS, 4, 0, 0)),
                    at_offset(STRIP_BYTE_COUNTS, 4, 0, 0),
                ),
                &[],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::InvalidPreviewLayout {
                        ifd_index: 0,
                        strip_offsets: 0,
                        strip_byte_counts: 0
                    }
                )
            },
        ),
        (
            "no-strip-byte-counts",
            one(without_tag(good(), STRIP_BYTE_COUNTS), &[], d + 16),
            |e| {
                matches!(
                    e,
                    Error::InvalidPreviewLayout {
                        ifd_index: 0,
                        strip_offsets: 1,
                        strip_byte_counts: 0
                    }
                )
            },
        ),
        (
            "offset-past-end",
            one(
                with_entry(good(), long(STRIP_OFFSETS, 0xFFFF_0000)),
                &[],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::Truncated {
                        at: 0xFFFF_0000,
                        len: 12
                    }
                )
            },
        ),
        (
            "offset-plus-length-past-end",
            one(
                with_entry(good(), long(STRIP_BYTE_COUNTS, 0x1_0000)),
                &[],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::Truncated {
                        at: 0x4000,
                        len: 0x1_0000
                    }
                )
            },
        ),
        (
            // One strip whose length alone is u32::MAX: the range is checked in
            // u64, so it is out of the file, not wrapped into a small slice.
            "length-at-u32-max",
            one(
                with_entry(good(), long(STRIP_BYTE_COUNTS, u32::MAX)),
                &[],
                d + 16,
            ),
            |e| matches!(e, Error::Truncated { at: 0x4000, len } if *len == u32::MAX as usize),
        ),
        (
            // Two strips whose lengths sum past u32::MAX. In wrapping u32
            // arithmetic the second strip "starts where the first ends"
            // (0x4000 + 0xFFFF_FFF0 wraps to 0x3FF0) and the total wraps to
            // 0x10, an in-bounds 16-byte lie. Summed in u64, the second strip
            // does not start at the first one's true end.
            "length-sum-overflows-u32",
            preview_strips(order, &[d, d - 0x10], &[0xFFFF_FFF0, 0x20], d + 64),
            |e| {
                matches!(
                    e,
                    Error::UnsupportedPreviewLayout {
                        ifd_index: 0,
                        layout: UnsupportedPreview::NonContiguousStrips
                    }
                )
            },
        ),
        (
            "missing-width",
            one(without_tag(good(), IMAGE_WIDTH), &[], d + 16),
            |e| matches!(e, Error::MissingTag { tag: 256 }),
        ),
        (
            "missing-height",
            one(without_tag(good(), IMAGE_LENGTH), &[], d + 16),
            |e| matches!(e, Error::MissingTag { tag: 257 }),
        ),
        (
            // NewSubfileType as ASCII: unknown is not no (DEC-012).
            "unreadable-new-subfile-type",
            one(
                with_entry(good(), at_offset(NEW_SUBFILE_TYPE, 2, 4, 0)),
                &[],
                d + 16,
            ),
            |e| {
                matches!(
                    e,
                    Error::UnexpectedFieldType {
                        tag: 254,
                        field_type: 2
                    }
                )
            },
        ),
        (
            // NewSubfileType as 2 LONGs pointing past the end of the file.
            "new-subfile-type-past-end",
            one(
                with_entry(good(), at_offset(NEW_SUBFILE_TYPE, 4, 2, 0xFFFF_FF00)),
                &[],
                d + 16,
            ),
            |e| matches!(e, Error::Truncated { .. }),
        ),
    ]
}

#[test]
fn hostile_preview_layouts_are_typed() {
    for order in ORDERS {
        for (name, data, check) in hostile_cases(order) {
            let c = Container::parse(&data)
                .unwrap_or_else(|e| panic!("{order:?} {name}: fixture must parse, got {e}"));
            let items = drain(&c);
            assert_eq!(items.len(), 1, "{order:?} {name}: one IFD, one item");
            match &items[0] {
                Err(e) => {
                    assert!(check(e), "{order:?} {name}: wrong error {e:?}");
                    assert!(!e.to_string().is_empty(), "{name}: empty Display");
                }
                Ok(p) => panic!("{order:?} {name}: hostile layout yielded Ok({p:?})"),
            }
        }
    }
}

#[test]
fn bad_preview_does_not_hide_a_later_good_one() {
    let d = PREVIEW_DATA;
    for order in ORDERS {
        let ifds = [
            // 0: tiled — refused.
            with_entry(
                preview_entries(order, Some(1), 2, 2, d, 12),
                long(TILE_OFFSETS, d),
            ),
            // 1: NewSubfileType unreadable — an Err, not a skip.
            with_entry(
                preview_entries(order, Some(1), 2, 2, d, 12),
                at_offset(NEW_SUBFILE_TYPE, 2, 4, 0),
            ),
            // 2: not a preview at all.
            preview_entries(order, Some(0), 2, 2, d, 12),
            // 3: the good full-size preview.
            preview_entries(order, Some(1), 4, 4, d + 16, 48),
        ];
        let data = preview_chain(order, &ifds, &[], d + 64);
        let c = Container::parse(&data).expect("fixture parses");
        let items = drain(&c);
        assert_eq!(items.len(), 3, "{order:?}: Err, Err, then the good preview");
        assert!(matches!(
            items[0],
            Err(Error::UnsupportedPreviewLayout {
                ifd_index: 0,
                layout: UnsupportedPreview::Tiled
            })
        ));
        assert!(matches!(
            items[1],
            Err(Error::UnexpectedFieldType { tag: 254, .. })
        ));
        let good = items[2]
            .as_ref()
            .expect("the later good preview is yielded");
        assert_eq!(good.ifd_index, 3);
        assert_eq!((good.width, good.height), (4, 4));
        assert_subslice(&data, good, "after-errors");
        // And the iterator really is finished afterwards.
        let mut it = c.previews();
        assert_eq!(it.by_ref().count(), 3);
        assert!(it.next().is_none());
    }
}

#[test]
fn preview_error_kinds_and_display() {
    let tiled = Error::UnsupportedPreviewLayout {
        ifd_index: 3,
        layout: UnsupportedPreview::Tiled,
    };
    assert_eq!(tiled.kind(), ErrorKind::Unsupported);
    let text = tiled.to_string();
    assert!(text.contains("IFD 3") && text.contains("tile"), "{text}");

    let jif = Error::UnsupportedPreviewLayout {
        ifd_index: 0,
        layout: UnsupportedPreview::JpegInterchangeFormat,
    }
    .to_string();
    assert!(jif.contains("JPEGInterchangeFormat"), "{jif}");

    let nc = Error::UnsupportedPreviewLayout {
        ifd_index: 0,
        layout: UnsupportedPreview::NonContiguousStrips,
    }
    .to_string();
    assert!(nc.contains("contiguous"), "{nc}");

    let invalid = Error::InvalidPreviewLayout {
        ifd_index: 2,
        strip_offsets: 2,
        strip_byte_counts: 1,
    };
    assert_eq!(invalid.kind(), ErrorKind::Corrupt);
    let text = invalid.to_string();
    assert!(
        text.contains("IFD 2")
            && text.contains("2 StripOffsets")
            && text.contains("1 StripByteCounts"),
        "{text}"
    );
}

/// Optional tags take their TIFF 6.0 defaults; present ones are reported.
#[test]
fn preview_fields_defaults_and_values() {
    let d = PREVIEW_DATA;
    for order in ORDERS {
        let bare = vec![
            long(NEW_SUBFILE_TYPE, 1),
            long(IMAGE_WIDTH, 3),
            long(IMAGE_LENGTH, 1),
            long(STRIP_OFFSETS, d),
            long(STRIP_BYTE_COUNTS, 3),
        ];
        let jpeg = with_entry(
            with_entry(
                preview_entries(order, Some(65537), 8, 8, d + 8, 20),
                short(tiff::COMPRESSION, 7, order),
            ),
            short(tiff::PHOTOMETRIC, 6, order),
        );
        let data = preview_chain(order, &[bare, jpeg], &[], d + 64);
        let c = Container::parse(&data).expect("fixture parses");
        let got = oks(&c);
        assert_eq!(got.len(), 2);
        let b = &got[0];
        assert_eq!(b.kind, PreviewKind::Primary);
        assert_eq!(b.compression, Compression::Uncompressed);
        assert_eq!(b.photometric, None);
        assert_eq!(b.samples_per_pixel, 1);
        assert_eq!(b.bits_per_sample, None);
        let j = &got[1];
        assert_eq!(j.kind, PreviewKind::Alternate);
        assert_eq!(j.compression, Compression::Jpeg);
        assert_eq!(j.photometric, Some(6));
        assert_eq!((j.offset, j.bytes.len()), (d + 8, 20));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 / AC7 — tier B, against the exiftool pins
// ─────────────────────────────────────────────────────────────────────────────

/// The 21 previews the SPEC-062 probe found across the 11 held DNGs.
const PINNED_PREVIEWS: usize = 21;

#[test]
fn corpus_previews_match_exiftool_pins() {
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();

    let pinned: usize = manifest
        .files
        .iter()
        .filter_map(|f| f.previews.as_ref())
        .map(Vec::len)
        .sum();
    assert_eq!(
        pinned, PINNED_PREVIEWS,
        "the manifest pins {pinned} previews; SPEC-062's probe found {PINNED_PREVIEWS}"
    );

    let mut matched = 0usize;
    for file in &manifest.files {
        let Some(pins) = file.previews.as_ref() else {
            eprintln!("NOT PINNED {} — no [[file.preview]] entries", file.path);
            continue;
        };
        let Some(path) = file.require(&root) else {
            continue;
        };
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let c = Container::parse(&data).unwrap_or_else(|e| panic!("{}: parse: {e}", file.path));
        let items = drain(&c);
        let got: Vec<Preview<'_>> = items
            .into_iter()
            .map(|r| r.unwrap_or_else(|e| panic!("{}: Err preview item: {e}", file.path)))
            .collect();
        assert_eq!(
            got.len(),
            pins.len(),
            "{}: irradiance reports {} previews, exiftool {}",
            file.path,
            got.len(),
            pins.len()
        );
        for (p, pin) in got.iter().zip(pins) {
            let kind = match p.kind {
                PreviewKind::Primary => "primary",
                PreviewKind::Alternate => "alternate",
                other => panic!("{}: unknown kind {other:?}", file.path),
            };
            let ours = (
                p.ifd_index,
                kind,
                p.width,
                p.height,
                p.compression.code(),
                p.photometric,
                u64::from(p.offset),
                p.bytes.len() as u64,
            );
            let theirs = (
                pin.ifd_index,
                pin.kind.as_str(),
                pin.width,
                pin.height,
                pin.compression,
                Some(pin.photometric),
                pin.offset,
                pin.length,
            );
            assert_eq!(
                ours, theirs,
                "{}: irradiance disagrees with the exiftool pin (ours, theirs)",
                file.path
            );
            assert_subslice(&data, p, &file.path);
            matched += 1;
        }
        eprintln!("PREVIEWS {} — {} match exiftool", file.path, pins.len());
    }
    eprintln!("corpus_previews_match_exiftool_pins: {matched}/{PINNED_PREVIEWS} previews matched");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8 — red-proofs: mutate a staged copy of src/, show the answer changes
// ─────────────────────────────────────────────────────────────────────────────

/// One source mutation: `needle` must occur exactly once in `file`.
struct Mutation {
    label: &'static str,
    file: &'static str,
    needle: &'static str,
    replacement: &'static str,
}

/// Selection widened to "bit 0 set" — DEC-057's rejected alternative.
const SELECTION_WIDENED: Mutation = Mutation {
    label: "selection-widened",
    file: "src/ifd.rs",
    needle: "let kind = match nst {",
    replacement: "let nst = if nst & 1 != 0 { 1 } else { nst }; // RED-PROOF INJECTION\n        let kind = match nst {",
};

/// Only the first strip's length counts.
const ONE_STRIP_ONLY: Mutation = Mutation {
    label: "one-strip-only",
    file: "src/ifd.rs",
    needle: "let strips_end = end;",
    replacement: "let strips_end = first_end; // RED-PROOF INJECTION",
};

/// The returned slice starts one byte late.
const RANGE_OFF_BY_ONE: Mutation = Mutation {
    label: "range-off-by-one",
    file: "src/ifd.rs",
    needle: "let slice_start = start;",
    replacement: "let slice_start = start.saturating_add(1); // RED-PROOF INJECTION",
};

/// The probe prints one line per `previews()` item, in the same format
/// [`describe`] produces in-process.
const PROBE_MAIN: &str = r#"
fn main() {
    for path in std::env::args().skip(1) {
        let data = std::fs::read(&path).expect("read fixture");
        let c = irradiance::ifd::Container::parse(&data).expect("fixture parses");
        for item in c.previews() {
            match item {
                Ok(p) => {
                    let ptr_ok = p.bytes.as_ptr() == data.as_ptr().wrapping_add(p.offset as usize);
                    println!("ok {} {:?} {}x{} {} {} ptr_ok={}", p.ifd_index, p.kind, p.width, p.height, p.offset, p.bytes.len(), ptr_ok);
                }
                Err(e) => println!("err {e}"),
            }
        }
        println!("--");
    }
}
"#;

/// The probe's output for `data`, computed in-process from the honest tree.
fn describe(data: &[u8]) -> String {
    let c = Container::parse(data).expect("fixture parses");
    let mut out = String::new();
    for item in c.previews() {
        match item {
            Ok(p) => {
                let ptr_ok = p.bytes.as_ptr() == data.as_ptr().wrapping_add(p.offset as usize);
                out.push_str(&format!(
                    "ok {} {:?} {}x{} {} {} ptr_ok={}\n",
                    p.ifd_index,
                    p.kind,
                    p.width,
                    p.height,
                    p.offset,
                    p.bytes.len(),
                    ptr_ok
                ));
            }
            Err(e) => out.push_str(&format!("err {e}\n")),
        }
    }
    out.push_str("--\n");
    out
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "irradiance-previews-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
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

/// Stage `Cargo.toml` + `src/` + the probe bin into `dest`.
fn stage_probe_crate(dest: &Path) {
    let repo = corpus::crate_root();
    std::fs::copy(repo.join("Cargo.toml"), dest.join("Cargo.toml")).expect("copy Cargo.toml");
    let lock = repo.join("Cargo.lock");
    if lock.is_file() {
        std::fs::copy(&lock, dest.join("Cargo.lock")).expect("copy Cargo.lock");
    }
    copy_dir_recursive(&repo.join("src"), &dest.join("src"));
    std::fs::write(dest.join("src/bin/preview_probe.rs"), PROBE_MAIN).expect("write probe");
    let mut toml = std::fs::read_to_string(dest.join("Cargo.toml")).expect("read Cargo.toml");
    toml.push_str("\n[[bin]]\nname = \"preview_probe\"\npath = \"src/bin/preview_probe.rs\"\n");
    std::fs::write(dest.join("Cargo.toml"), toml).expect("write Cargo.toml");
}

fn build_and_run_probe(dir: &Path, fixtures: &[PathBuf]) -> String {
    let build = std::process::Command::new("cargo")
        .args(["build", "--bin", "preview_probe", "--quiet"])
        // The staged crate builds into its own target/ (SPEC-056): never into
        // the caller's shared CARGO_TARGET_DIR.
        .env_remove("CARGO_TARGET_DIR")
        .current_dir(dir)
        .output()
        .expect("spawn cargo build");
    assert!(
        build.status.success(),
        "cargo build failed in {}:\n{}",
        dir.display(),
        String::from_utf8_lossy(&build.stderr)
    );
    let run = std::process::Command::new(dir.join("target/debug/preview_probe"))
        .args(fixtures)
        .output()
        .expect("spawn preview_probe");
    assert!(
        run.status.success(),
        "preview_probe failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("probe stdout is UTF-8")
}

/// Apply `m` to the staged copy, returning the original text to restore.
fn inject(dir: &Path, m: &Mutation) -> String {
    let path = dir.join(m.file);
    let src = std::fs::read_to_string(&path).expect("read staged source");
    let n = src.matches(m.needle).count();
    assert_eq!(
        n, 1,
        "{}: expected exactly one {:?} in {}; found {n} — the code moved, update the needle",
        m.label, m.needle, m.file
    );
    std::fs::write(&path, src.replacen(m.needle, m.replacement, 1)).expect("write mutant");
    src
}

#[test]
fn red_proof_preview_mutations() {
    let order = Order::Little;
    let fixtures: Vec<(&str, Vec<u8>)> = vec![
        ("selection", preview_selection(order)),
        ("contiguous", preview_contiguous_strips(order)),
        ("two", preview_two(order)),
    ];
    let fixture_dir = TempDir::new("fixtures");
    let paths: Vec<PathBuf> = fixtures
        .iter()
        .map(|(name, data)| {
            let p = fixture_dir.0.join(format!("{name}.tif"));
            std::fs::write(&p, data).expect("write fixture");
            p
        })
        .collect();
    let honest: String = fixtures.iter().map(|(_, d)| describe(d)).collect();

    let stage = TempDir::new("stage");
    stage_probe_crate(&stage.0);

    // CONTROL first (DEC-009): the unmutated staged copy must reproduce the
    // honest in-process answer, or a red below proves nothing.
    let control = build_and_run_probe(&stage.0, &paths);
    assert_eq!(
        control, honest,
        "the UNMUTATED staged copy disagrees with the in-process tree"
    );

    // selection widened: NST 5 (index 4) and 9 (index 6) now count, so
    // `selection_is_exactly_nst_1_and_65537` would see extra previews.
    let original = inject(&stage.0, &SELECTION_WIDENED);
    let out = build_and_run_probe(&stage.0, &paths);
    std::fs::write(stage.0.join(SELECTION_WIDENED.file), &original).expect("restore");
    assert_ne!(out, control, "selection-widened changed nothing");
    assert!(
        out.contains("ok 4 Primary") && out.contains("ok 6 Primary"),
        "selection-widened: expected NST 5 and 9 reported as previews:\n{out}"
    );
    eprintln!("RED-PROOF (selection-widened): red as expected");

    // one strip only: the 45-byte three-strip preview shrinks to 10, so
    // `contiguous_strips_join_into_one_slice` fails.
    let original = inject(&stage.0, &ONE_STRIP_ONLY);
    let out = build_and_run_probe(&stage.0, &paths);
    std::fs::write(stage.0.join(ONE_STRIP_ONLY.file), &original).expect("restore");
    assert_ne!(out, control, "one-strip-only changed nothing");
    let contiguous = format!("ok 0 Primary 4x4 {PREVIEW_DATA} 10 ptr_ok=true");
    assert!(
        out.contains(&contiguous),
        "one-strip-only: expected the contiguous preview cut to 10 bytes:\n{out}"
    );
    eprintln!("RED-PROOF (one-strip-only): red as expected");

    // range off by one: the slice no longer starts at `offset`, so
    // `preview_bytes_are_a_subslice_of_the_input` (and AC7's tier-B
    // assert_subslice) fail.
    let original = inject(&stage.0, &RANGE_OFF_BY_ONE);
    let out = build_and_run_probe(&stage.0, &paths);
    std::fs::write(stage.0.join(RANGE_OFF_BY_ONE.file), &original).expect("restore");
    assert_ne!(out, control, "range-off-by-one changed nothing");
    assert!(
        out.contains("ptr_ok=false") && !out.contains("ptr_ok=true"),
        "range-off-by-one: every preview must now miss its offset:\n{out}"
    );
    eprintln!("RED-PROOF (range-off-by-one): red as expected");
}
