# SPEC-037 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-037-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-05 (reconstruction + round-trip oracle spec + HANDOFF-087 + build prompt; no new DEC — governed by DEC-035/016/018)
- [x] **build** — PR #102, 2026-11-05 (HANDOFF-087); reconstruction (predictors 1–7 / §H.2.1 boundaries / point transform / restart reset) + `decode_into` + `Error::LjpegPlaneWrongLength` + the `encode_lj92` round-trip oracle & perturbed-predictor red-proof; four gates green; no new DEC. Cost: 271,544 tok (FLOOR — context occupancy, no `/cost`; see HANDOFF-087 handback)
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked
(one-line reason, needs human or external unblock).
