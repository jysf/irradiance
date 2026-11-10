# SPEC-053 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-053-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator, 2026-11-10) — full design against the current tree;
  DEC-047 minted (shape binding in `unpack_jpeg` + `H/V != 1` rejection in
  `parse_frame_header`); the equal-area-transpose and discarded-`Hi/Vi` paths read
  and confirmed; HANDOFF-110 (build) prepared. Spec moved `frame → build`.
- [x] **build** — delegated via HANDOFF-110. Branch `claude/spec-053-build-<suffix>`
  from freshly-merged `main`. Implement the two guards, make the failing tests pass
  (two red-proofs first), extend the `ljpeg` fuzz target, run the gates + tier-B
  bit-exact corpus check, open a PR, fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ✅ APPROVED at 90f4898; tests, lint, both red-proofs and the bit-exact corpus check pass, DEVELOP_VERSION unchanged, fuzz clean (ljpeg 3.35M, plane 3.10M); no code change needed. — cold review against the acceptance criteria + both red-proofs + the
  Nf=1/Nf=2 cases.
- [x] **ship** (2026-11-10) — group B merged via PR #169 (verify APPROVED, no PR needed). Cost 16,460,765 tok / $11.51 (build 12.63M + verify 3.83M, handback-sync). Brag logged; STAGE-010 backlog updated.
