# SPEC-049 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-049-<cycle>.md`.

## Instructions

- [x] **design** (2026-11-07) — `DEC-042` written (fuse the two scratch passes into one `w*h` green plane, matrix-into-`dst` in pass 2; add `develop_color_f32_into` that quantises then divides); Acceptance + Failing Tests (byte-identity + f32-equality) filled; `HANDOFF-103` (build) and `prompts/SPEC-049-build.md` ready.
- [ ] **build** — `HANDOFF-103` → implementer: fuse `develop_color_into` (normalise-on-read, single green plane, matrix per pixel into `dst`) and add `develop_color_f32_into`; prove `u16` bit-identity (goldens + equivalence test) and f32-equality, record peak-RSS before/after. Dispatch with `prompts/SPEC-049-build.md`.
- [ ] **verify** — cold review: `u16` bit-identical, f32 equals the reference exactly, both scratch buffers gone, peak-RSS measured, time not regressed materially.
- [ ] **ship** — archive, record the provenance-ledger peak-RSS row.
