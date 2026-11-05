# SPEC-036 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-036-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (DEC-035 patent clearance + full spec + HANDOFF-085 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #97, HANDOFF-085; cost ≈299,209 tokens — context-occupancy lower bound, no `/cost` in this interface, see the handback). `src/ljpeg.rs`: SOF-3 marker/frame parse + `0xFF`-destuffing bit reader + DC Huffman → per-sample differences; 7 additive `Ljpeg*` errors; all failing tests pass; gates green.
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
