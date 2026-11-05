# SPEC-036 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-036-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (DEC-035 patent clearance + full spec + HANDOFF-085 + build prompt)
- [x] **build** — completed 2026-11-05 (PR #97, HANDOFF-085; cost ≈299,209 tokens — context-occupancy lower bound, no `/cost` in this interface, see the handback). `src/ljpeg.rs`: SOF-3 marker/frame parse + `0xFF`-destuffing bit reader + DC Huffman → per-sample differences; 7 additive `Ljpeg*` errors; all failing tests pass; gates green.
- [x] **verify** — ✅ APPROVED 2026-11-05 at commit `ef16fa1` (HANDOFF-086; cold review of build PR #97). All 8 ACs met + tested; gates green (`just test` 132 lib + integration, `just lint-ci -D warnings`, `cargo fmt --check`, `just validate` 42, `just lint-no-allow`, `decisions-audit --changed` no drift); 2 adversarial mutations confirmed the over-sub guard + `EXTEND` sign-extension have real teeth; clean-room + provenance row held; scope stops at the difference stream. Cost ≈199,216 tokens (context-occupancy floor, no `/cost` in this interface, see the handback).
- [ ] **ship** — prompt: `prompts/SPEC-036-ship.md` (ready; runs after merge of build PR #97 + the verify PR)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
