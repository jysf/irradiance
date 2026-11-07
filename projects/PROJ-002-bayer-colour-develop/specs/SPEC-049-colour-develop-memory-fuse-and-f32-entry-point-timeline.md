# SPEC-049 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-049-<cycle>.md`.

## Instructions

- [x] **design** (2026-11-07) — `DEC-042` written (fuse the two scratch passes into one `w*h` green plane, matrix-into-`dst` in pass 2; add `develop_color_f32_into` that quantises then divides); Acceptance + Failing Tests (byte-identity + f32-equality) filled; `HANDOFF-103` (build) and `prompts/SPEC-049-build.md` ready.
- [x] **build** (2026-11-07, PR #147, ~358k tok) — `HANDOFF-103` → implementer: fused `develop_color_into` (normalise-on-read, single `w·h` green plane, matrix per pixel into `dst`) and added `develop_color_f32_into`. `u16` byte-identical to the pre-fuse composition (oracle + goldens green, checksum `339852` unchanged), f32 equals `to_linear_f32_into(develop_color_into(..))` exactly; peak RSS 358.7 → 224.8 MiB (−133.9 MiB) at 23.4 MP; develop time ≈2.76× slower (normalise-on-read, accepted per `DEC-042`; one-row-cache follow-up noted).
- [ ] **verify** — cold review: `u16` bit-identical, f32 equals the reference exactly, both scratch buffers gone, peak-RSS measured, time not regressed materially.
- [ ] **ship** — archive, record the provenance-ledger peak-RSS row.
