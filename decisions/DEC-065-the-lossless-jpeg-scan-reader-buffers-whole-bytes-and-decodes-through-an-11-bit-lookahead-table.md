---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-065
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-14
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ljpeg.rs
  - examples/ljpeg-timing.rs
  - app.just
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-070-faster-single-stream-compression-7-decode-profile-gated-k-3.md

tags:
  - ljpeg
  - compression-7
  - huffman
  - performance
  - stage-020
---

# DEC-065: the lossless-JPEG scan reader buffers whole bytes and decodes through an 11-bit lookahead table

## Decision

### 1. `ScanReader` buffers whole data bytes in a `u64`

Today `ScanReader` holds one data byte and reads it one bit at a time. It becomes a
buffered reader:

- **The buffer.** A `u64` of bits plus a count, with the next bit at the top of the valid
  bits. It is filled with **whole data bytes**, MSB-first, while at least 8 bits are free.
- **Filling stops silently, never fails.** A fill takes the byte at `byte_pos` when:
  - it is not `0xFF`: one source byte becomes one data byte;
  - it is `0xFF 0x00`: two source bytes become one `0xFF` data byte (destuffing).

  It **stops, without an error and without moving `byte_pos`**, at:
  - the end of the slice;
  - `0xFF` followed by any other byte (a marker);
  - `0xFF` as the last byte.

  So the buffer only ever holds bytes that today's `fetch_byte` would have fetched
  successfully.
- **An error happens only when a bit is needed and none is buffered.** The reader then
  calls today's `fetch_byte` at `byte_pos`, which returns today's error:
  - `Error::Truncated { at, len }`;
  - or `Error::LjpegUnexpectedMarker { marker }`.

  The bit-at-a-time reader fails at exactly that point, at that `byte_pos`, after
  consuming exactly the same bits. **Errors and their fields are therefore identical by
  construction.**
- **`restart` gives back the unread whole bytes.** At a restart, today's reader discards
  the rest of its current byte, and its `byte_pos` points just after that byte. The
  buffered reader may hold more:
  - the rest of the current byte (`nbits mod 8` bits), which it discards;
  - `nbits / 8` whole bytes that today's reader has not fetched yet.

  It moves `byte_pos` back over those whole bytes, each by its source size (1 byte, or
  2 for a stuffed `0xFF 0x00`). To do that it records one "was stuffed" flag per
  buffered byte. Then it empties the buffer and runs today's marker check unchanged.
  `byte_pos` after a restart equals today's (the unit test
  `restart_consumes_rstm_and_byte_aligns` pins it).

### 2. Huffman codes decode through an 11-bit lookahead table

`HuffTable::build` also builds two **fixed-size arrays inside `HuffTable`**, from the
same `MINCODE` / `MAXCODE` / `VALPTR` / `HUFFVAL` it builds today. There is no heap
allocation.

- **`lookup` (2,048 entries):** for every code of length ≤ 11, the code length and the
  symbol (`SSSS`), replicated over every 11-bit index that starts with that code.
- **`fast` (2,048 entries):** for every code whose length plus its extra bits fits in 11
  bits, the total bits to consume and the **final signed difference**:
  - `SSSS` 0 contributes no extra bits, and its difference is 0;
  - `SSSS` 16 contributes no extra bits, and its difference is +32768;
  - `SSSS` 1–15 contributes `SSSS` bits, and the difference is `EXTEND` of them.

  The encoding is the builder's choice, as long as it represents every difference from
  −32767 to +32768, and every total from 1 to 11, without ambiguity, and "no entry" is
  distinct.

`decode_difference` then runs in this order:
1. **Top up the buffer** when it holds fewer than 32 bits.
2. **Peek 11 bits.** If fewer are buffered, pad with zeros on the right.
3. **`fast` hit, and its total ≤ the buffered bits:** consume the total and return the
   difference.
4. **`lookup` hit, its length ≤ the buffered bits, and the extra bits also buffered** (or
   `SSSS` is 0 or 16): consume the code and the extra bits, and return the `EXTEND`ed
   difference.
5. **Otherwise, today's path:** `decode_symbol` bit by bit, then `receive_and_extend`.
   This covers:
   - codes longer than 11 bits;
   - prefixes that match no code;
   - `SSSS` above 16;
   - a buffer that cannot hold the whole code at the end of the scan.

   Nothing is consumed before this step is chosen, so it starts from the same bit as
   today, and its result and error are today's.

**Why steps 3 and 4 are exact.** A table entry is determined by the code's own bits
only, because every index that starts with that code holds the same entry. The guard
"length ≤ buffered bits" makes sure every one of those bits is real data, not padding.
Canonical Huffman codes are prefix-free, so the first matching length is the only match,
which is what `decode_symbol`'s length-by-length search finds.

### 3. Shape constraints

- **No `unsafe`, no allocation, no new dependency, no SIMD intrinsics.** The two tables
  are arrays inside `HuffTable`, which stays `Clone`. `HuffTable` grows to about 12 KiB.
  `parse_headers` already clones tables into a `Vec`, so the allocation **count** is
  unchanged; only their size grows.
- **Shifts and casts follow the crate lints:** `checked_shl` / `checked_shr`,
  `try_from`, `get`, and wrapping or checked arithmetic. The probe measured the hot path
  with checked shifts.
- **Unchanged:**
  - `fetch_byte`, `decode_symbol`, `receive`, `receive_and_extend`, `extend` and `pow2`
    (now the slow path);
  - every reconstruction function, the predictors, `Window`, and `predict_component`.
    `tests/plane_oracle.rs`'s Compression 7 injector needs `neighbour(out, slot, 2)` to
    occur exactly twice;
  - `parse_headers` and every error variant;
  - every public item.
- **The table width is a private `const` of 11.** The probe measured 11 and 12 bits
  within 2% of each other, and 10, 13 and 14 slower. Eleven bits keeps the two tables at
  12 KiB, inside L1 on smaller cores too.

### 4. How the speed is measured

This follows SPEC-067's and SPEC-069's practice:
- a committed release example, `examples/ljpeg-timing.rs`, run by `just ljpeg-timing`;
- it times `plane::unpack_into` on the K-3 III Mono, the M Typ 246 and the Pixel 4a;
- one warm-up, then the median and minimum of 7 runs;
- a SKIP line for each absent file, and exit 0;
- no CI timing gate.

**The targets are absolute, on a fixed baseline.** On the build machine, the medians must
be:
- **K-3 III ≤ 150 ms**, against **704–708 ms at `4e48521`**;
- **Typ 246 ≤ 140 ms**, against **601–606 ms**.

The Pixel 4a is recorded with no target. These are targets, not gates. Byte identity and
error identity are the hard criteria.

## Context

monorail-000 / onyx-000 triage item 1b asked for a faster single-threaded open. STAGE-020
framed this spec as profile-gated. The two held single-strip Compression 7 files have no
restart markers, so SPEC-068's units cannot split them, and only a faster decoder helps.

The SPEC-070 design probe ran on main at `4e48521` (Apple M4 Pro, release build, a
throwaway example over `plane::unpack_into`, median of 7):

| Kernel | K-3 III (6304×4224, Nf 2) | Typ 246 (5984×4000, Nf 2) | Pixel 3a | Pixel 4a | Pixel 7 Pro |
|---|---|---|---|---|---|
| crate today (bit at a time) | 708.1 ms | 605.7 | 319.3 | 331.3 | 306.1 |
| `u64` buffer + 11-bit `lookup` | 241.6 | 220.3 | 112.4 | 112.5 | 107.8 |
| + rare paths moved out of the hot function | 217.9 | 203.7 | 102.0 | 100.2 | 96.6 |
| **+ 11-bit `fast` table (code and difference together)** | **108.1** | **95.4** | about 60 | 66.3 | about 58 |

The three Pixel files are 192 tiles each.

- **Every candidate was byte-identical to `unpack_into` on all five files.**
- **The profile names the entropy decode.** macOS `sample` over 40 `unpack_into` calls on
  the K-3 III put **7,676 of 8,395** samples in `ScanReader::decode_difference` and
  `read_bit`, about 91%. Prediction (`predict_component` and the reconstruct loop) took
  the other 9%.
- **Errors were identical, not just values.** The probe linked a copy of today's
  `src/ljpeg.rs` beside the candidate and compared `decode_into` and
  `decode_window_into`. It compared the whole `Result`, including every error field, and
  the destination buffer, over **28,880** corrupted streams. Those were 72 real Pixel
  tiles and the two held strips, mutated by truncation, bit flips, injected `0xFF`,
  injected markers, a trailing lone `0xFF`, and random bytes. Every one matched. It also
  compared **3,000** clean and **18,000** corrupted synthetic streams from the test
  encoder, covering:
  - restart intervals 0, 1, 2, 3, 5, 7 and 16;
  - precisions 8, 12, 14 and 16;
  - predictors 1–7;
  - point transforms 0–2.

  Every one matched.
- **Table width:** 10 bits measured 123 / 112 / 71 ms (K-3 III / Typ 246 / Pixel 4a),
  11 bits 108 / 95 / 66, 12 bits 110 / 96 / 67, 13 bits 106 / 95 / 70, and 14 bits
  120 / 112 / 91. Building the larger tables costs more per Pixel tile.
- **Refilling eight bytes at once when none is `0xFF` bought about 3%.** It is not
  specified; the builder may add it if it stays exact.

## Alternatives Considered

- **The buffered reader and the `lookup` table only.** Rejected. It is about 3× faster;
  adding `fast` makes it about 6.5× for one more table built from the same data.
- **A wider table (13–14 bits).** Rejected. It is no faster on the strips and slower on
  the tiled Pixels, where 192 tables are built per file.
- **Replace the bit-at-a-time path entirely.** Rejected. It still has to handle codes
  longer than 11 bits and the end of the scan, and keeping it unchanged keeps every
  error on today's code.
- **Pre-scan the scan for markers and decode the clean prefix without checks.** Rejected
  as unnecessary. Stopping the fill at a marker gives the same speed with no second pass.
- **Speed up prediction.** Not now. It is about 9% before the change; after it, it is a
  larger share of a much smaller total. Any later change is profile-gated like this one.
- **SIMD intrinsics or `unsafe` loads.** Rejected. The crate forbids `unsafe`, and the
  Huffman decode is serial by nature.

## Consequences

- **Single-threaded Compression 7 decode should be about 6.5× faster:**
  - the K-3 III Mono, about 704 to about 110 ms;
  - the Typ 246, about 601 to about 100 ms;
  - the tiled Pixels, about 320 to about 65 ms.

  The measured result is recorded at ship.
- **There is no public API change.** Output, errors and their fields are unchanged.
- **The bit-at-a-time path stays as the slow path.** A future change to the fast steps
  must keep the rule "consume nothing before choosing the slow path".
- **A new path needs its own oracle.** No held file has a restart, and the held files
  cannot be relied on to produce codes longer than 11 bits or `SSSS` 16. SPEC-070's
  differential test therefore drives synthetic tables and streams that do.
