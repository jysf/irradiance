# SPEC-029 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-029-<cycle>.md`.

## Instructions

- [x] **design** — relax `plane::unpack_into`'s `max > WhiteLevel` assertion (a
      sample above `WhiteLevel` is legal; preserve raw values, no clamp) + amend
      DEC-008; failing tests + ACs (PR #40, shared with SPEC-028 design).
- [x] **build** — HANDOFF-068. Removed the post-assembly `max > WhiteLevel` loop
      in `src/plane.rs` + `Error::SampleExceedsWhiteLevel` (variant + Display) in
      `src/lib.rs`; wrote DEC-029 amending DEC-008's "permanent invariant" clause
      (byte-swap guard → SPEC-025 + layer-0); `irr.rs` prints the real `>`
      relation. 2 new + reworked `tests/plane_unpack.rs` tests (AC1 above-white
      preserved, AC2 within-white unchanged, Canon + Nikon corpus unpack fully).
      Gates green; `plane` fuzz 0 crashes. $23.45, 2026-10-29. PR #44 (merged).
- [x] **verify** — HANDOFF-069. ✅ APPROVED at `1112c18` (PR #44, already merged —
      cold retrospective review). All 5 ACs reproduced; both corpus Bayer DNGs
      unpack FULLY (Canon single-strip 5920×3950 max 15464 > 15000; Nikon 12-strip
      4352×2868 max 4095 > 3808) via `--nocapture` tests + `irr unpack`; mono
      regression BYTE-FOR-BYTE (md5 == `dnglab --raw-checksum` on all 4 decodable
      frames). Gates re-run green (test 14 binaries/0 failed, corpus 7/7; pinned
      clippy `-D warnings`; fmt; validate; decisions-audit 0 structural errors);
      `plane` fuzz 25,936,891 runs / 91s, 0 crashes. DEC-029 content correct; the
      DEC-029 number is an accepted intentional deviation (reservation released in
      the build brief). One non-blocking follow-up SPEC-029/FU-1 (drop the stale
      reservation hedge in spec + 4 sibling docs, at ship). $20.83, 2026-10-30.
- [ ] **ship** — see `prompts/SPEC-029-ship.md` (maintainer's main-session step:
      reflection, cost totals, `complexity_actual`, SPEC-029/FU-1 text cleanup,
      archive, CHANGELOG, brag).
