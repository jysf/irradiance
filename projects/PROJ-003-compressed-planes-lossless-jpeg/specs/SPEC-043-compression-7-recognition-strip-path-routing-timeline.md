# SPEC-043 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-043-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe of both held files — both single-strip, 12/14-bit, MM/II — + the `unpack_into` routing seam + the `Frame`/`decode_into` contract; acceptance + failing tests + HANDOFF-091 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #112); `Compression` dispatch in `plane::unpack_into` (`Uncompressed` unchanged, `Jpeg`→`ljpeg::decode_into` per strip, `other`→`UnsupportedCompression`) + 6 tier-A SOF-3 tests; all four gates green; cost FLOOR 306,881 tokens (context-occupancy, HANDOFF-091). ⚠ both held Compression-7 files are `Nf=2` multi-component — routes cleanly to a typed error, but SPEC-044's bit-exact premise is affected (see `guidance/questions.yaml`).
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
