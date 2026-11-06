# SPEC-046 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-046-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe parsing both real strips' SOF3+DHT+SOS — one interleaved scan, Ns=2, both H=V=1, one Td=0 table, predictor 1, Pt=0, no DRI — + the `ljpeg` extension seam; acceptance + failing tests + HANDOFF-093 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #117; cost FLOOR 276,963 tokens via `ccd_session_mgmt get_usage` context-occupancy, no interactive `/cost` in the desktop Code tab); extended `ljpeg` to decode a two-component (Nf=2) column-interleave SOF-3 frame with per-component predictor state + column de-interleave (`reconstruct_two_component`), Td-keyed DC tables, Nf-keyed length check; Nf=1 path byte-for-byte unchanged; six tier-A failing tests pass (bit-exact P=12/P=14, round-trip red-proof on perturbed predictor / Huffman table / swapped component order, single-component characterization, Nf=3 rejected, wrong-length + truncated typed errors); `fmt-check`/`lint-ci`/`validate` green locally, full `just test` on PR CI (HANDOFF-093)
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
