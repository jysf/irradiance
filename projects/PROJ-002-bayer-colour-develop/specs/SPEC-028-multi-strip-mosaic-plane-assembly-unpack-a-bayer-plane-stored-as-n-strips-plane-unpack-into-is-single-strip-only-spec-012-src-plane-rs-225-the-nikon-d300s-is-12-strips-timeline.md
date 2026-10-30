# SPEC-028 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-028-<cycle>.md`.

## Instructions

- [x] **design** — multi-strip assembly + per-strip padding; failing tests + ACs (PR #40).
- [x] **build** — HANDOFF-066. N-strip loop in `src/plane.rs` + `Error::MissingRowsPerStrip`;
      10 new `tests/plane_unpack.rs` tests (7 ACs + inverse-check + Nikon 12-strip corpus);
      Nikon assembles (was `UnsupportedStripLayout`), then trips WhiteLevel (SPEC-029).
      Gates green; `plane` fuzz 15,026,955 runs / 61s, 0 crashes. $102.25, 2026-10-29. PR #41.
- [x] **verify** — HANDOFF-067. ✅ APPROVED at `233ae5c` (PR #41, already merged — cold
      retrospective review). All 7 ACs reproduced; single-strip regression held byte-for-byte
      (Q2M 14-bit + M Monochrom 16-bit green). Gates re-run green (test 14 binaries/0 failed,
      corpus 7/7; clippy 0.1.98 -D warnings; fmt; validate; lint-no-allow); `plane` fuzz
      20,465,198 runs / 91s, 0 crashes; adversarial mutation flips AC2 red / control green.
      Both build deviations ACCEPTABLE; SPEC-028/FU-1 (spec Outputs byte-ceil wording) filed,
      non-blocking. $21.10, 2026-10-29.
- [ ] **ship** — see `prompts/SPEC-028-ship.md` (maintainer's main-session step: reflection,
      cost totals, `complexity_actual`, archive, CHANGELOG, brag).
