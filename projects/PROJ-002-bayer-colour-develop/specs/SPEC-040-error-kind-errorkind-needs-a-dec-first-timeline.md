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
- [~] **build** — delegated via HANDOFF-119. Branch `claude/spec-040-build-<suffix>` from
  freshly-merged `main`. Add `ErrorKind` + `kind()` (no `_ =>` arm) + the exhaustive
  per-variant test; confirm the compile-time failure on a throwaway dummy variant; no new
  variant / no `Display` change; gates; PR; fill the handback.
- [ ] **verify** — cold review: no wildcard arm (dummy variant fails to compile), every
  variant matches DEC-051's table, the four judgment calls as decided, `Display`/production
  untouched.
- [ ] **ship** — merge, recover true build/verify cost, brag, stage bookkeeping.
