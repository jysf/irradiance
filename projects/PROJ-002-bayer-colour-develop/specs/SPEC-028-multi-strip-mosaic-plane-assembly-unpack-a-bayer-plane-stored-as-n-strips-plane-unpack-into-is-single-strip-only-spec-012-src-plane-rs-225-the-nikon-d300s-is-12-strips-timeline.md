# SPEC-028 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-028-<cycle>.md`.

## Instructions

- [x] **design** — multi-strip assembly + per-strip padding; failing tests + ACs (PR #40).
- [x] **build** — HANDOFF-066. N-strip loop in `src/plane.rs` + `Error::MissingRowsPerStrip`;
      10 new `tests/plane_unpack.rs` tests (7 ACs + inverse-check + Nikon 12-strip corpus);
      Nikon assembles (was `UnsupportedStripLayout`), then trips WhiteLevel (SPEC-029).
      Gates green; `plane` fuzz 15,026,955 runs / 61s, 0 crashes. $102.25, 2026-10-29. PR #PENDING.
