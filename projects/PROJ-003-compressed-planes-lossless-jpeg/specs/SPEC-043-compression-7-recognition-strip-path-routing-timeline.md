# SPEC-043 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-043-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe of both held files — both single-strip, 12/14-bit, MM/II — + the `unpack_into` routing seam + the `Frame`/`decode_into` contract; acceptance + failing tests + HANDOFF-091 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #112); `Compression` dispatch in `plane::unpack_into` (`Uncompressed` unchanged, `Jpeg`→`ljpeg::decode_into` per strip, `other`→`UnsupportedCompression`) + 6 tier-A SOF-3 tests; all four gates green; cost FLOOR 306,881 tokens (context-occupancy, HANDOFF-091). ⚠ both held Compression-7 files are `Nf=2` multi-component — routes cleanly to a typed error, but SPEC-044's bit-exact premise is affected (see `guidance/questions.yaml`).
- [x] **verify** — completed 2026-11-05 (✅ APPROVED at `594dcc8`; HANDOFF-092). Six acceptance criteria confirmed cold: the six tier-A tests run individually ok (single-strip bit-exact both byte orders, uncompressed characterization, 2-strip multi-band, dimension-mismatch → `LjpegPlaneWrongLength`, truncated → typed error, non-1/7 → `UnsupportedCompression`) + the tier-B real-file routing test (both held C7 files reach the decoder). Gates green: `lint-ci` / `fmt-check` / `validate` / `decisions-audit` local, `rust / test` + full CI green on the exact commit. No decision drift. The `Nf=2` two-component finding is a STAGE-011 re-plan matter (orchestrator-owned, `guidance/questions.yaml`), NOT a SPEC-043 defect — routing-only scope upheld. Cost FLOOR 205,139 tokens (context-occupancy, HANDOFF-092).
- [ ] **ship** — prompt: `prompts/SPEC-043-ship.md` (waiting on merge)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
