//! `SPEC-062` AC6 — draining `Container::previews()` performs no heap
//! allocation (`DEC-057`).
//!
//! Its own test binary on purpose: the counting `#[global_allocator]` below
//! sees every allocation in this process, so no other test may share it. The
//! container is parsed BEFORE counting starts (`parse` collects the IFDs into
//! a `Vec`, which is allowed); only the drain is counted.

#[path = "support/tiff.rs"]
mod tiff;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use irradiance::ifd::Container;
use tiff::{
    at_offset, long, preview_chain, preview_entries, with_entry, Order, NEW_SUBFILE_TYPE,
    PREVIEW_DATA, STRIP_BYTE_COUNTS, STRIP_OFFSETS, TILE_OFFSETS,
};

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
fn draining_previews_allocates_nothing() {
    let d = PREVIEW_DATA;
    let order = Order::Big;
    // Three previews plus three Err items, so both item shapes are counted:
    // 0: one-strip preview (Ok)
    // 1: tiled (Err)
    // 2: three contiguous out-of-line strips (Ok — the multi-value read path)
    // 3: NewSubfileType unreadable (Err)
    // 4: NewSubfileType 0 (skipped)
    // 5: alternate preview (Ok)
    // 6: offsets past the end (Err)
    let ifds = [
        preview_entries(order, Some(1), 2, 2, d, 12),
        with_entry(
            preview_entries(order, Some(1), 2, 2, d, 12),
            long(TILE_OFFSETS, d),
        ),
        with_entry(
            with_entry(
                preview_entries(order, Some(1), 4, 4, 0, 0),
                at_offset(STRIP_OFFSETS, 4, 3, 0x3000),
            ),
            at_offset(STRIP_BYTE_COUNTS, 4, 3, 0x3800),
        ),
        with_entry(
            preview_entries(order, Some(1), 2, 2, d, 12),
            at_offset(NEW_SUBFILE_TYPE, 2, 4, 0),
        ),
        preview_entries(order, Some(0), 2, 2, d, 12),
        preview_entries(order, Some(65537), 2, 2, d + 48, 12),
        preview_entries(order, Some(1), 2, 2, 0xFFFF_0000, 12),
    ];
    let data = preview_chain(
        order,
        &ifds,
        &[
            (0x3000, vec![d, d + 10, d + 25]),
            (0x3800, vec![10, 15, 20]),
        ],
        d + 64,
    );
    let container = Container::parse(&data).expect("fixture parses");

    set_counting(true);
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut bytes = 0usize;
    for item in container.previews() {
        match item {
            Ok(p) => {
                ok += 1;
                bytes += p.bytes.len();
            }
            Err(e) => {
                err += 1;
                // Classifying is part of handling an Err item; it must not
                // allocate either.
                std::hint::black_box(e.kind());
            }
        }
    }
    set_counting(false);
    let allocations = ALLOCATIONS.load(Ordering::SeqCst);

    assert_eq!(
        (ok, err),
        (3, 3),
        "the fixture yields three Ok and three Err"
    );
    assert_eq!(bytes, 12 + 45 + 12);
    assert_eq!(
        allocations, 0,
        "draining previews() allocated {allocations} time(s); DEC-057 requires none"
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
