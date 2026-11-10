---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-046
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/lib.rs
  - fuzz/fuzz_targets/ifd.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-052-parser-resource-guard-aggregate-entry-byte-budget.md
  - docs/provenance-ledger.md

tags:
  - ifd
  - parser
  - resource-guard
  - memory-amplification
  - no-panics-on-untrusted-input
  - stage-010
  - spec-052
---

# DEC-046: Bound the aggregate IFD entry-table bytes by the file length

## Decision

During the container walk, accumulate a running total of every IFD's entry-table
size (`count × 12` bytes, `ENTRY_SIZE = 12`) and reject the file the moment that
running total would exceed the file length. This caps the **total** number of IFD
entries parsed and allocated across the whole walk at `file_len / ENTRY_SIZE`,
which closes the memory-amplification of finding #1 (SPEC-052): a ~770 KB file can
no longer drive a ~72 MiB parse. New typed error
`Error::IfdTableBytesExceedFile { declared: u64, file_len: usize }` (additive;
`Error` is `#[non_exhaustive]`).

## Why this shape

`Entry` is a flat 12-byte struct with no heap payload (`src/ifd.rs:339`), so the
walk's memory is `total_entries × ~12 bytes`. The existing guards bound each IFD
(`count` is a `u16`, ≤ 65,535; the table must lie inside the file) and the IFD
*count* (`MAX_IFDS = 512`) and cycles (`visited` rejects an identical offset), but
nothing bounds the **aggregate**. Entry tables at *distinct* offsets may overlap
the same file bytes — every per-IFD check passes — so 512 × 65,535 ≈ 33.5M entries
(~400 MiB) is the ceiling; the reviewer measured ~6.3M (~72 MiB) from ~770 KB. The
sum-of-table-bytes rule makes the aggregate proportional to the file: overlap can
only *inflate* the running total, so any amplifying layout trips the bound almost
immediately (after ~`file_len / 12` entries), while a well-formed file — whose
tables are disjoint and therefore sum to ≤ `file_len` — is never rejected.

## Why not the alternatives

- **Per-range overlap rejection** (track each claimed `[at, entries_end)` and
  reject an overlap): equivalent protection but `O(IFDs²)` with range bookkeeping,
  and it rejects harmless small overlaps. The running sum is `O(1)` per IFD and
  rejects only what actually amplifies. It is also the **same idiom** as the
  plane-level aliasing rule the SPEC-039 design proposes
  (`StripBytesExceedFile`: "declared bytes may not exceed the file") — one
  consistent shape across the metadata and the pixel paths.
- **An absolute `MAX_TOTAL_IFD_ENTRIES` cap:** introduces a magic number; the
  file-length-proportional bound needs none, is strictly tighter for the small
  hostile file (the amplification case), and never rejects a legitimate large
  file. Rejected as a redundant second guard.

## Mechanics

- The check runs **inside `read_ifd`** right after `table_len` is computed and
  **before** `Vec::with_capacity(count)`, so an over-budget IFD allocates nothing.
- A `&mut u64` running total is threaded from `Container::parse` through
  `walk_chain` into `read_ifd` (the exact carrier — a parameter or a small walk
  state struct — is the implementer's; no allocation is added).
- `declared` is the running total at rejection; `file_len` is `data.len()`.
- This is a **third, orthogonal** guard. `MAX_IFDS`, `MAX_IFD_DEPTH`,
  `MAX_TAG_VALUES` and the `visited` cycle guard are all unchanged; the regression
  uses **distinct** offsets so it exercises this guard, not the cycle guard.
- Classification: `IfdTableBytesExceedFile` is a **Corrupt**-class error for
  SPEC-040's future `Error::kind` table (a hostile/malformed file, not caller
  misuse).

## Measured baseline (this machine, 2026-11-10)

`irr ifd` over the tier-B corpus: Canon 5D-III `5G4A9394.DNG` — 48,382,640 bytes,
4 IFDs, ~131 aggregate entries (~1.5 KiB of tables); Nikon D300S `_DSC4615.DNG` —
37,562,890 bytes, 3 IFDs, ~72 entries (~0.9 KiB). Real IFD tables are ~0.003 % of
the file, so the bound clears legitimate files by ~30,000×.

## Scope and non-goals

- **In:** the running aggregate-table-bytes guard; the one new `Error` variant and
  its `Display`; the fuzz target running it on every input; a provenance-ledger
  row.
- **Out:** any change to `MAX_IFDS`, `MAX_IFD_DEPTH`, `MAX_TAG_VALUES`, the cycle
  guard, or how value payloads are read; the pixel/plane validate
  (SPEC-039/group E); an absolute entry cap; de-duplicating distinct IFDs that
  legitimately point at disjoint tables.
