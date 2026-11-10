# SPEC-052 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-052-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — scope + full design written against the
  current tree; DEC-046 minted (aggregate IFD entry-table bytes ≤ file length);
  corpus baseline measured with `irr ifd`; HANDOFF-109 (build) prepared. Spec moved
  `frame → build`.
- [x] **build** — delegated to a cold session via HANDOFF-109. Branch
  `claude/spec-052-build-<suffix>` from freshly-merged `main`. Implement the DEC-046
  guard, make SPEC-052's failing tests pass (red-proof first), extend the `ifd` fuzz
  target, run the gates + tier-B corpus, open a PR, fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ✅ APPROVED at 75f681a; all gates green, red-proof reproduced, both corpus DNGs parse, a counting-allocator probe showed the guard cuts a 790 KB hostile file from 402 MB to 787 KB; no code change needed. — cold review against the acceptance criteria + red-proof + the
  distinct-offsets isolation.
- [x] **ship** (2026-11-10) — group A merged via PR #167 (verify APPROVED, no PR needed). Cost 11,402,194 tok / $8.44 (build 7.03M + verify 4.38M, handback-sync). Brag logged; STAGE-010 backlog updated.
