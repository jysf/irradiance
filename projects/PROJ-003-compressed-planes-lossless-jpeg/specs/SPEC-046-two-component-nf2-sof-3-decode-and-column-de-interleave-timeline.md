# SPEC-046 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-046-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (§12 probe parsing both real strips' SOF3+DHT+SOS — one interleaved scan, Ns=2, both H=V=1, one Td=0 table, predictor 1, Pt=0, no DRI — + the `ljpeg` extension seam; acceptance + failing tests + HANDOFF-093 + build prompt)
- [ ] **build** — prompt: `prompts/SPEC-046-build.md` (HANDOFF-093)
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
