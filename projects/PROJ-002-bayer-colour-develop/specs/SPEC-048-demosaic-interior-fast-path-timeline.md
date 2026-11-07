# SPEC-048 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-048-<cycle>.md`.

## Instructions

- [x] **design** (2026-11-06) — `DEC-041` written (interior fast path + the STAGE-012 perf discipline); Acceptance + Failing Tests (an equivalence / bit-identity proof) filled; `HANDOFF-101` (build) and `prompts/SPEC-048-build.md` ready.
- [ ] **build** — `HANDOFF-101` → implementer: add the row-wise interior fast path (direct neighbour indexing over per-row slices, border kept reflecting), kernel arithmetic unchanged; prove bit-identity (equivalence test + golden hashes) and record a before/after demosaic time. Dispatch with `prompts/SPEC-048-build.md`.
- [ ] **verify** — cold review: bit-identity holds, the equivalence proof is real, the speedup is measured.
- [ ] **ship** — archive, record the provenance-ledger timing row.
