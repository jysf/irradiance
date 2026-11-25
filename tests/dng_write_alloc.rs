//! `SPEC-061` AC10, oracle 10 (and `SPEC-081` AC8, every metadata option set;
//! `SPEC-082` AC9, a preview too) — `written_len` and `write_into` perform no heap
//! allocation (`DEC-075` §9, the `DEC-016` promise applied to the writer).
//!
//! Its own test binary on purpose: the counting `#[global_allocator]` below
//! sees every allocation in this process, so no other test may share it. The
//! inputs and the destination are built BEFORE counting starts; only the two
//! calls are counted. `just test` also runs this binary in the debug profile,
//! where LLVM cannot delete a dead allocation (SPEC-072).

#[path = "support/opcode.rs"]
mod opcode;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use irradiance::dng::write::{
    write_into, written_len, ExifBlock, Framing, JpegPreview, JpegPreviewColor, WriteOptions,
};
use irradiance::ifd::{ActiveArea, DefaultCropOrigin, DefaultCropSize, Rational};
use opcode::{opcode_list, raw_opcode, single_fix_bad_pixels_constant_list};

struct Counting;

// Thread-local, so only allocations made BY THE TEST THREAD between the two
// switches count; the harness's own threads cannot make this flaky. A `const`
// thread-local needs no lazy initialisation, so reading it never allocates.
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

fn counting() -> bool {
    COUNTING.try_with(Cell::get).unwrap_or(false)
}

fn set_counting(on: bool) {
    COUNTING.with(|c| c.set(on));
}
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// The library's `forbid(unsafe_code)` binds the library crate, not this test
// crate. `GlobalAlloc` is an unsafe trait, so implementing it needs
// `unsafe impl`; every method forwards to `System` unchanged, which upholds
// `GlobalAlloc`'s contract exactly as `System` does.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if counting() {
            ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn written_len_and_write_into_allocate_nothing() {
    let (w, h) = (256u32, 256u32);
    let samples: Vec<u16> = (0..w * h).map(|i| (i * 7) as u16).collect();
    let list_1 = single_fix_bad_pixels_constant_list(0, 0, 0);
    let list_2 = opcode_list(&[raw_opcode(99, 0x0103_0000, 1, &[1, 2, 3])]);
    let list_3 = opcode_list(&[raw_opcode(1, 0x0104_0000, 0, &[0; 68])]);
    let active = ActiveArea {
        top: 4,
        left: 6,
        bottom: 250,
        right: 252,
    };
    let masked = [
        ActiveArea {
            top: 0,
            left: 0,
            bottom: 4,
            right: 256,
        },
        ActiveArea {
            top: 4,
            left: 0,
            bottom: 250,
            right: 6,
        },
    ];
    let blacks = [60, 61, 62, 63];
    // SPEC-081: an odd-length block (empty Exif IFD at 0, GPS IFD at 6) and
    // an XMP packet.
    let block = [0u8; 13];
    let xmp = b"<x:xmpmeta xmlns:x='adobe:ns:meta/'/>";
    // SPEC-082: an odd-length synthetic JPEG (the writer reads two bytes).
    let jpeg = [0xFF, 0xD8, 1, 2, 3, 0xFF, 0xD9];

    let mut total_calls = 0;
    for framing in [
        Framing::FullSensor,
        Framing::FullActiveArea,
        Framing::StandardCrop,
    ] {
        // Every option set.
        let mut o = WriteOptions::default();
        o.framing = framing;
        o.active_area = Some(active);
        o.default_crop = Some((
            DefaultCropOrigin { x: 2, y: 3 },
            DefaultCropSize {
                width: 200,
                height: 180,
            },
        ));
        o.orientation = 8;
        o.black_level = &blacks;
        o.black_level_repeat_dim = Some([2, 2]);
        o.white_level = 16383;
        if framing == Framing::FullSensor {
            o.masked_areas = &masked;
        }
        o.opcode_list_1 = Some(&list_1);
        o.opcode_list_2 = Some(&list_2);
        o.opcode_list_3 = Some(&list_3);
        o.unique_camera_model = Some("Allocation Monochrome");
        // SPEC-081 AC8: every metadata option set.
        let mut exif = ExifBlock::default();
        exif.bytes = &block;
        exif.exif_ifd = Some(0);
        exif.gps_ifd = Some(6);
        o.exif_block = Some(exif);
        o.xmp = Some(xmp);
        o.make = Some("Allocation");
        o.model = Some("Monochrome");
        o.software = Some("irradiance");
        o.date_time = Some("2026:11:24 10:00:00");
        o.artist = Some("");
        o.copyright = Some("    ");
        o.localized_camera_model = Some("Allocation Monochrome");
        o.camera_serial_number = Some("0001");
        o.baseline_exposure = Some(Rational { num: -1, den: 2 });
        o.baseline_noise = Some(Rational { num: 1, den: 1 });
        o.baseline_sharpness = Some(Rational { num: 3, den: 2 });
        o.linear_response_limit = Some(Rational { num: 1, den: 1 });
        // SPEC-082 AC9: a preview, YCbCr so its BitsPerSample is out of line.
        let mut preview = JpegPreview::default();
        preview.jpeg = &jpeg;
        preview.width = 64;
        preview.height = 48;
        preview.color = JpegPreviewColor::YCbCr;
        o.preview = Some(preview);

        // Sized outside the count, by a first uncounted query.
        let len = written_len(&samples, w, h, &o, 1 << 20).expect("valid input");
        let mut dst = vec![0u8; len];

        set_counting(true);
        let counted_len = written_len(&samples, w, h, &o, 1 << 20);
        let written = write_into(&samples, w, h, &o, 1 << 20, &mut dst);
        // A refusal is part of the contract too: no allocation on Err.
        let refused = written_len(&samples, w, h, &WriteOptions::default(), 1 << 20);
        set_counting(false);
        total_calls += 3;

        assert_eq!(counted_len.ok(), Some(len));
        assert!(written.is_ok(), "{framing:?}: {written:?}");
        assert!(refused.is_err());
        std::hint::black_box(&dst);
    }
    let allocations = ALLOCATIONS.load(Ordering::SeqCst);
    assert_eq!(total_calls, 9);
    assert_eq!(
        allocations, 0,
        "written_len/write_into allocated {allocations} time(s); DEC-075 §9 requires none"
    );

    // Negative control: the counter does count. Without this, a broken
    // allocator hook would make the zero above vacuous.
    set_counting(true);
    let v: Vec<u8> = std::hint::black_box(Vec::with_capacity(64));
    set_counting(false);
    drop(v);
    assert!(
        ALLOCATIONS.load(Ordering::SeqCst) > allocations,
        "the counting allocator missed a Vec allocation"
    );
}
