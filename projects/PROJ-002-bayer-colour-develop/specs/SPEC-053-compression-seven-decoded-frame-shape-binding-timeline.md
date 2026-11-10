# SPEC-053 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-053-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — full design against the current tree;
  DEC-047 minted (shape binding in `unpack_jpeg` + `H/V != 1` rejection in
  `parse_frame_header`); the equal-area-transpose and discarded-`Hi/Vi` paths read
  and confirmed; HANDOFF-110 (build) prepared. Spec moved `frame → build`.
- [~] **build** — delegated via HANDOFF-110. Branch `claude/spec-053-build-<suffix>`
  from freshly-merged `main`. Implement the two guards, make the failing tests pass
  (two red-proofs first), extend the `ljpeg` fuzz target, run the gates + tier-B
  bit-exact corpus check, open a PR, fill the handback.
- [ ] **verify** — cold review against the acceptance criteria + both red-proofs + the
  Nf=1/Nf=2 cases.
- [ ] **ship** — merge, recover true build/verify cost, brag, stage bookkeeping.
