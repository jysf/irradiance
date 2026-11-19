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
- [x] **verify** — ✅ APPROVED at `cc67ad6` (cold Claude Code verify session).
  Full suite green with the corpus present (`just test` exit 0, 0 failed;
  colour oracle 23/23 with 2/2 Bayer files present — proven exercised via
  `--nocapture`, not skipped); AC1–AC8 confirmed independently against
  `exiftool`/`dnglab`/`irr ifd`. `oracle-must-be-shown-red` re-proven with two
  adversarial mutations of my own: neutering `rationals_equal` fails exactly the
  two red-proofs, and perturbing src `rationals()` (`num+1`) turns the live
  tier-B oracle red — both reverted, tree clean. Mono SPEC-005/010 oracle
  byte-for-byte behaviourally unchanged (pure rename + additive fixture fields);
  `ifd` fuzz 11,876,467 runs/61s/0 crashes; CI observed green on the merged SHA.
  The 3 build-flagged items adjudicated: Deviation 1 (dnglab whitelevels plain
  ints) and Deviation 2 (AC7 Failing-Tests bullet self-contradictory) are both
  HONEST with ACs genuinely met; the exact-rational-oracle candidate decision is
  recommended for minting as `DEC-030` at ship (cross-cutting; DEC-026 is the
  precedent). Verify cost recorded in `cost.sessions[]`. Ship is the maintainer's.
  See the verify PR.
- [x] **ship** — main session. Minted **DEC-030** (the colour-metadata oracle
  compares fractional DNG rationals exactly by i128 cross-multiplication against
  exiftool's full-precision decimal, no tolerance; `dnglab` scoped off the matrices
  its JSON omits — extending DEC-005/DEC-026) + INDEX row; filled the Reflection;
  recorded cost totals (build 29,850,028/$59.66 + verify 10,300,008/$23.63 =
  40,150,036 tokens / $83.29, 3 sessions) and `complexity_actual: M`; `advance-cycle
  verify→ship` (verify_verdict → approved); CHANGELOG `[Unreleased]→Added` (public
  API: `Rational`, `Container::rationals()`, `Error::RationalValueOutOfRange`, the 9
  additive `Sensor` colour fields, `irr ifd` surfacing); brag logged; archived to
  `specs/done/`. `just cost-audit` + `validate` green. **This ships SPEC-026 and
  COMPLETES STAGE-006 (6 shipped / 0 open)** — STAGE-007 demosaic (the first actual
  picture) is next.
