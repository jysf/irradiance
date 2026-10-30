# SPEC-026 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-026-<cycle>.md`.

## Instructions

- [x] **build** — delegated to a cold Claude Code session. Added the signed/
  fractional `Rational` type + `Container::rationals()` reader (SRATIONAL +
  fractional RATIONAL, typed overflow on the RATIONAL→i32 narrow), the additive
  `Sensor` colour fields parsed from IFD0 (matrices/illuminants/AsShotNeutral)
  and the per-channel levels from the sensor IFD, the self-contained
  `tests/colour_metadata_oracle.rs` (exact i128 cross-multiply comparator +
  tier-A red-proof with control), `irr ifd` surfacing, and the
  `CFA_DEFERRED_TO_SPEC_026` → `CFA_COVERED_BY_COLOUR_ORACLE` rename. All
  SPEC-026 tests green; gates (test / lint-ci / lint-fuzz / validate / fuzz)
  observed green. See the build PR.
