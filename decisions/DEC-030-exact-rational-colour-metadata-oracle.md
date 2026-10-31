---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-030
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-10-30
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# Extends DEC-005/DEC-026's oracle-scope reasoning to the first fractional
# metadata irradiance parses; it does not amend either — both still govern the
# integer tags unchanged.
affected_scope:
  - tests/colour_metadata_oracle.rs
  - tests/support/tools.rs

tags:
  - oracle
  - colour
  - rational
  - exiftool
  - dnglab
  - spec-026
  - dec-005
  - dec-026
---

# DEC-030: The colour-metadata oracle compares fractional DNG rationals exactly, and `dnglab` is scoped off the matrices its JSON does not carry

## Decision

The colour-metadata oracle (`tests/colour_metadata_oracle.rs`, SPEC-026) compares
the **fractional** DNG colour tags — `ColorMatrix1/2`, `ForwardMatrix1/2`,
`AsShotNeutral` — **exactly, by cross-multiplication in `i128`** against the exact
rational parsed from `exiftool -n`'s full-precision decimal. **No float epsilon
tolerance.** The integer colour tags (`CalibrationIlluminant1/2`, and the
per-channel `BlackLevel` / `WhiteLevel`) compare as integers.

`exiftool` is the **sole** oracle for the matrices / illuminants / `AsShotNeutral`.
`dnglab analyze --meta --json` is **scoped off** them — its JSON does not carry the
matrices at all — and cross-checks only the per-channel `BlackLevel` / `WhiteLevel`
it does expose. This **extends** `DEC-005` (exiftool as the tag-level metadata
oracle) and `DEC-026` (amending DEC-005's oracle scope) to the first fractional
metadata the library parses; it amends neither — both still govern the integer
tags unchanged.

## Context

SPEC-026 is the first spec whose oracle targets **signed, fractional** values: the
colour matrices are `SRATIONAL` and `AsShotNeutral` is a fractional `RATIONAL`. The
existing metadata oracle (`SPEC-005` / `SPEC-010`) is entirely `u32`-integer. Two
questions had to be settled, and a measurement pass over both corpus Bayer DNGs
(`exiftool -v3 -n`, `dnglab 0.7.2 analyze --meta --json`, 2026-10-30) answered both
from the files themselves, not from the DNG spec alone:

- **How to compare fractional values truthfully.** The DNG matrices are stored as
  exact terminating-decimal rationals (`ColorMatrix` `n/10000`; `AsShotNeutral`
  `n/1000000` on the Canon, `n/100000` on the Nikon), and `exiftool -n` prints the
  exact decimal **in full** (`606276/1000000` → `0.606276`), not a rounded view. So
  the stored rational and exiftool's printed decimal denote the *same exact
  rational* — an exact comparison is both possible and correct.
- **Which tool is the oracle for which tag.** `dnglab`'s `--meta --json` exposes
  `blacklevels.levels` (exact `N/D` strings), `whitelevels` (plain integers) and a
  derived float `wbCoeffs` — but **no colour matrices and no illuminants**. Its
  `wbCoeffs` are lossy float reciprocals of `AsShotNeutral`. So dnglab cannot be an
  oracle for the matrices.

## Alternatives Considered

- **Option A: a float epsilon tolerance** (compare `num/den` as `f64` within
  `1e-N`).
  - Why rejected: a tolerance hides real drift — a matrix element wrong in a low
    digit, or a byte-swap that happens to land near the true value, would pass.
    The measurement shows the values are exact, so a tolerance would *weaken* a
    comparison that can be exact. `oracle-must-be-shown-red`'s whole point is that
    "a green oracle that cannot fail manufactures confidence"; a tolerance on an
    exactly-comparable value is precisely that softening.

- **Option B: exact `f64` equality** (`==` of `num as f64 / den as f64` vs the
  parsed decimal).
  - Why rejected (subtle, not wrong): in practice both sides round the same real to
    the same IEEE double, so it would pass — but the correctness *argument* depends
    on both roundings landing identically, which is fragile. Cross-multiplication is
    obviously exact, uses no float at all, and needs no such argument.

- **Option C: force `dnglab` to be a second oracle for the matrices** (parse them
  from its human/text output).
  - Why rejected: dnglab's JSON does not expose the matrices, and its human text is
    not a stable contract. Manufacturing a second oracle where the tool carries no
    clean value is how a brittle or perpetually-skipped check gets shipped —
    `DEC-005`/`DEC-026` already scope dnglab to what it exposes cleanly (and exclude
    the Pentax `.PEF`, whose values are rawler-DB-derived) for exactly this reason.

- **Option D (chosen): exact `i128` cross-multiplication against exiftool's
  full-precision decimal for the fractional tags; integer equality for the integer
  tags; dnglab scoped to the per-channel levels it exposes, exiftool the sole oracle
  for the matrices / illuminants / `AsShotNeutral`.**
  - Why selected: it is exact, needs no tolerance to tune or to hide drift, cannot
    overflow for DNG-range numerators/denominators, and checks each tag with the
    tool that actually carries it — the honest continuation of the project's
    oracle-scope discipline.

## Consequences

- **Positive.** The colour-metadata oracle is exact, with no tolerance to tune and
  nothing to hide low-digit drift. Verify proved it genuinely goes red: a `num + 1`
  perturbation of src `rationals()` turns the live tier-B oracle red (Canon: 5
  fields disagree) while the tier-A fixture control stays green. `i128`
  cross-multiplication cannot overflow for DNG-range values.
- **Positive.** The tool/tag scoping is honest — each tag is checked by the tool
  that actually carries it, extending `DEC-005`/`DEC-026` rather than straining a
  tool past what it exposes.
- **Negative — the honest cost.** The comparison assumes exiftool prints the stored
  rational's **exact** decimal. If a future file stored a *non-terminating* colour
  rational (so exiftool had to round), the oracle would go **red** rather than
  silently accept — correct for an oracle, but red for a tool-precision reason, not
  a reader bug. A file that genuinely needs it is its own spec (tolerance vs.
  raw-rational extraction, decided then with the evidence). `AsShotNeutral` is also
  cross-checkable against dnglab only via a lossy reciprocal, so it is deliberately
  **not** cross-checked there — exiftool alone.
- **Neutral.** The integer colour tags keep integer equality; the mono
  `SPEC-005`/`SPEC-010` oracle and its red-proofs are byte-for-byte untouched.

## Validation

Right if: the colour oracle matches `exiftool` exactly on both corpus Bayer files,
is shown red by any single-element perturbation (demonstrated at verify), and the
per-channel levels agree with **both** `exiftool` and `dnglab`. Revisit if: a
corpus file stores a non-terminating colour rational (exiftool rounds — then decide
tolerance vs. raw-rational extraction as its own spec, with the file as evidence),
or if a tool begins exposing the colour matrices in a stable machine-readable form
worth adding as a second oracle.

## References

- Extends: `DEC-005` (exiftool as the tag-level metadata oracle), `DEC-026`
  (amending DEC-005's oracle scope). Amends neither.
- Related specs: `SPEC-026` (this change), `SPEC-005` / `SPEC-010` (the integer
  metadata oracle and its tri-state), `SPEC-024` (CFA sensor identity), `SPEC-025`
  (the bit-exact mosaic-plane oracle).
- Evidence: `CANON-EOS-5D-MARK-III/5G4A9394.DNG` (`ColorMatrix` `n/10000`,
  `AsShotNeutral` `606276/1000000 …`, per-channel black `[2047,2047,2047,2048]`;
  dnglab JSON carries no matrices) and `NIKON-D300S/_DSC4615.DNG` (`ForwardMatrix`
  absent → `None`, `AsShotNeutral` `64000/100000 …`), measured via `exiftool -v3 -n`
  + `dnglab 0.7.2 analyze --meta --json`, 2026-10-30.
- Constraints: `oracle-must-be-shown-red`, `no-panics-on-untrusted-input`.
