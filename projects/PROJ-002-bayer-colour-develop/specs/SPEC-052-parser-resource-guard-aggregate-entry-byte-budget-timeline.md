# SPEC-052 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-052-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — scope + full design written against the
  current tree; DEC-046 minted (aggregate IFD entry-table bytes ≤ file length);
  corpus baseline measured with `irr ifd`; HANDOFF-109 (build) prepared. Spec moved
  `frame → build`.
- [~] **build** — delegated to a cold session via HANDOFF-109. Branch
  `claude/spec-052-build-<suffix>` from freshly-merged `main`. Implement the DEC-046
  guard, make SPEC-052's failing tests pass (red-proof first), extend the `ifd` fuzz
  target, run the gates + tier-B corpus, open a PR, fill the handback.
- [ ] **verify** — cold review against the acceptance criteria + red-proof + the
  distinct-offsets isolation.
- [ ] **ship** — merge, recover true build/verify cost, brag, stage bookkeeping.
