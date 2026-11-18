# SPEC-025 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-025-<cycle>.md`.

## Cycles

- [x] **design** (2026-10-30, PR #47) — approach + failing tests. STAGE-006's
  capstone oracle: direct-path (not the manifest) with a byte-aligned red-proof;
  pinned `dnglab --raw-checksum` constants for both corpus Bayer DNGs.
- [x] **build** (2026-10-30, PR #48) — both corpus Bayer planes MD5-match
  `dnglab --raw-checksum`; byte-aligned red-proof added to `tests/plane_oracle.rs`
  (test-only — no `src/`, no manifest; `stage_probe_crate` generalised from a bool
  to an `Option<fn>` injector, mono path unchanged). `claude-opus-4-8` (HANDOFF-070).
- [x] **verify** (2026-10-30, PR #49) — ✅ APPROVED, cold independent review. All
  5 ACs reproduced; dnglab 0.7.2 checksums recomputed == pinned constants (no
  tautology); both red-proofs red with green controls; `plane_oracle` 15/15; `plane`
  fuzz 13,407,764 runs / 0 crashes. `claude-opus-4-8` (HANDOFF-071).
- [x] **ship** (2026-10-30) — Reflection, cost totals (build + verify = 24,006,243
  tokens / $63.88; ship un-metered), archive, brag. `verify_verdict: approved`,
  `complexity_actual: M` (as expected). Test-only — no CHANGELOG entry.
