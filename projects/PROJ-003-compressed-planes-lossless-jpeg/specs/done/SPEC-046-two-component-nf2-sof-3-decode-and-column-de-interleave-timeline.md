# SPEC-046 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-046-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe parsing both real strips' SOF3+DHT+SOS — one interleaved scan, Ns=2, both H=V=1, one Td=0 table, predictor 1, Pt=0, no DRI — + the `ljpeg` extension seam; acceptance + failing tests + HANDOFF-093 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #117; cost FLOOR 276,963 tokens via `ccd_session_mgmt get_usage` context-occupancy, no interactive `/cost` in the desktop Code tab); extended `ljpeg` to decode a two-component (Nf=2) column-interleave SOF-3 frame with per-component predictor state + column de-interleave (`reconstruct_two_component`), Td-keyed DC tables, Nf-keyed length check; Nf=1 path byte-for-byte unchanged; six tier-A failing tests pass (bit-exact P=12/P=14, round-trip red-proof on perturbed predictor / Huffman table / swapped component order, single-component characterization, Nf=3 rejected, wrong-length + truncated typed errors); `fmt-check`/`lint-ci`/`validate` green locally, full `just test` on PR CI (HANDOFF-093)
- [x] **verify** — ✅ APPROVED 2026-11-05 (HANDOFF-094; PR #118; cost FLOOR 215,327 tokens via `ccd_session_mgmt get_usage` context-occupancy, no interactive `/cost` in the desktop Code tab). Cold review at build head `54704c2`: all six ACs met, all six `## Failing Tests` present and passing (`cargo test --lib ljpeg::` 27/27), the Nf=1 `reconstruct` body diff-identical to `main`, per-component predictor state confirmed (two-output-columns-back `Ra`), the round-trip red-proof genuinely red on perturbed predictor / mutated Huffman table / swapped component order (independent reference decoder ties to production), the length check keys on `Nf`, `Nf=3` still rejected; `lint-ci`/`fmt-check`/`validate`/`decisions-audit` green locally, full `rust / test` green on PR #117 CI. Tier-A scope only — the real-file bit-exact `dnglab` mapping confirmation is SPEC-044 (`shared-path-oracle-tautology`).
- [x] **ship** — 2026-11-05 (prompt: `prompts/SPEC-046-ship.md`). Reflection + cost recovery + archive + brag. True costs recovered from both cold transcripts (floors superseded): build 15,641,899 tok / $36.89 (local_47012b20), verify 6,846,595 tok / $16.72 (local_6a7cc6b6); **spec total 22,488,494 tok / $53.61** across 4 sessions. `complexity_actual: M`. **STAGE-011 NOT complete** — SPEC-044 (dnglab oracle) + SPEC-045 (capstone) remain; no stage-ship.

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
