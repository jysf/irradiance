//! Writing DNG files (`DEC-055`), behind the `write` feature.
//!
//! [`write`] holds the mono `u16` writer (`SPEC-061`, `DEC-071` to `DEC-075`).
//! It uses only the container types in [`crate::ifd`] and [`crate::Error`],
//! never develop, colour, warp or plane code, and performs no I/O: the caller
//! owns the buffer and the file.

pub mod write;
