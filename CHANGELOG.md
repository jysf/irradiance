# Changelog

All notable changes to this app. Newest at top.

Keep an `## [Unreleased]` section on top and add entries under it as you work —
the patch lane files fixes here (`[Unreleased] → Fixed`), and a release spec
(DEC-006) promotes the section to a version heading when it cuts the tag.

## [Unreleased]

### Added

- The crate exists: `irradiance` library + `irr` dev binary, edition 2021,
  measured MSRV 1.90, **zero dependencies**, `MIT OR Apache-2.0` (SPEC-001).
- Panic-free lint policy (`unwrap_used`, `expect_used`, `indexing_slicing`,
  `panic`, `arithmetic_side_effects` at `deny`; `#![forbid(unsafe_code)]`),
  enforced by a **red-proof that is itself proven red** — it injects violations
  into a copy of the real library and requires a clean negative control, so it
  fails if the policy is deleted, weakened to `allow`, downgraded to `warn`, or
  if clippy never ran (DEC-009).
- CI: `fmt --check`, `clippy -D warnings`, `test`, `cargo deny check licenses`,
  an MSRV check, and the lint red-proof.
  ⚠ CI **cannot** verify decode correctness — tier-B corpus files are never
  committed (DEC-003), so a green badge does not mean the decoder is bit-exact.

### Known gaps

### Changed

### Fixed
