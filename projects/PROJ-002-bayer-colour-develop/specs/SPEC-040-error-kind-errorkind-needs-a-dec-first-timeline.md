# SPEC-040 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-040-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-03) — ID reserved; scope captured.
- [x] **design** (orchestrator, 2026-11-10) — DEC-051 minted: three-kind `ErrorKind {
  Unsupported, Corrupt, CallerError }` (`#[non_exhaustive]`) + `Error::kind()` as a
  **wildcard-free** match (compiler-enforced "a new variant cannot land unclassified"),
  with the full per-variant table (12 CallerError / 18 Unsupported / 25 Corrupt) and the
  four flagged judgment calls (`PlaneTooLarge`→CallerError; `NotTiff`/`NoSensorIfd`/
  `MissingColorCalibration`→Unsupported; the `Invalid*`→Corrupt). Spec moved `frame →
  build`; HANDOFF-119 prepared.
- [x] **build** (claude-opus-5-5 cold, PR #182 merged) — delegated via HANDOFF-119. Branch `claude/spec-040-build-ed86ac` from
  freshly-merged `main`. Add `ErrorKind` + `kind()` (no `_ =>` arm) + the exhaustive
  per-variant test; confirm the compile-time failure on a throwaway dummy variant; no new
  variant / no `Display` change; gates; PR; fill the handback.
- [x] **verify** (claude-opus-5-5 cold, 2026-11-10) — ✅ APPROVED at de30009; 512 tests, all 55 variants match DEC-051, a dummy variant fails E0004 in kind() (no-wildcard holds), Display/production untouched; no PR, no change. Confirmed: no wildcard arm (dummy variant fails to compile), every
  variant matches DEC-051's table, the four judgment calls as decided, `Display`/production
  untouched.
- [x] **ship** (2026-11-10) — code merged via PR #182; this bookkeeping PR records it. Cost 7,407,456 tok / $5.64 (build 5.76M/$4.20 + verify 1.65M/$1.44). brag logged; STAGE-010 → 7 shipped / 1 active. Non-blocking nit: the `tests/error_kind.rs` variant-name parser would skip a future tuple-style variant (the compiler still gates classification).
