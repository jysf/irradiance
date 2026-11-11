---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-052
  type: decision
  confidence: 0.8
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

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/opcode.rs
  - Cargo.toml
  - tests/
  - CHANGELOG.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md

tags:
  - release
  - semver
  - non-exhaustive
  - public-api
  - cfa
  - warprect
  - opcode
  - stage-010
  - spec-042
  - 0-2-0
---

# DEC-052: Seal the remaining public parser-output types, and cut 0.2.0 as semver

## Decision

The four decisions the 0.2.0 release (SPEC-042, group F) rests on.

### 1. Seal `Cfa`, `WarpRect` and `Opcode` as `#[non_exhaustive]`

Complete the sweep DEC-045 opened and DEC-049 began with `Sensor`: add
`#[non_exhaustive]` to `Cfa` (`src/ifd.rs:616`), `WarpRect` (`src/opcode.rs:155`) and
the `Opcode` enum (`src/opcode.rs:105`) — the staff review's remaining semver-urgent
public parser-output types (reconciling `src/opcode.rs`'s "`#[non_exhaustive]`-free by
choice" note, now overridden for 0.2.0). They are produced by the parser and **read** by
consumers, not constructed by them, so sealing costs real callers nothing and makes every
future field/variant non-breaking. 0.2.0 is the one release where sealing them is free of
a deprecation cycle, because the pre-0.2.0 API has no known external constructor.

Mechanics: a sealed enum (`Opcode`) forces every **external** match to carry a wildcard;
confirm the in-crate matches compile (they may match exhaustively in-crate, which stays
legal) and add the `#[non_exhaustive]` attribute only. The `tests/` crate constructs
these by struct-literal / enum-literal fixtures (the same blast radius `Sensor` had), so
migrate those fixtures onto the existing dev-only `test-fixtures` feature (SPEC-054,
mirroring the `fuzzing` precedent, DEC-036) — extend it with `Cfa`/`WarpRect`/`Opcode`
constructors rather than adding a second feature.

### 2. Version `0.2.0`, scheme **semver** (not the template default calver)

`irradiance` is a public-API library, so set `spec.version.scheme: semver` in
`.repo-context.yaml` (the frame's rule: "semver for a library/public API; calver
otherwise") and cut **`0.2.0`**. It is a minor bump over the live `0.1.1`: additive
modules (`demosaic`, `plane::validate`, `tone::to_linear_f32_into`, `Error::kind`,
`DEVELOP_VERSION`, the determinism evidence) plus the `#[non_exhaustive]` seals, which are
non-breaking for the readers who are these types' only real consumers. The top-level
`VERSION` file is template provenance and is left alone (`docs/versioning.md`).

### 3. Defer the three group-C verify follow-ups; fold only the trivial one into F

DEC-050 recorded three follow-ups from the SPEC-054 verify. For 0.2.0:
- **(3) the dev-dependency so plain `cargo test` compiles — FOLD INTO F.** Sealing the
  fixtures behind `test-fixtures` means a bare `cargo test` (no `--all-features`) no
  longer builds the integration tests. Add a dev-dependency on this crate with the
  dev features enabled (`irradiance = { path = ".", features = ["test-fixtures", "irr"] }`)
  so `cargo test` just works. Release hygiene, mechanical, in F.
- **(1) refuse a malformed `OpcodeList` tag** (today DEC-012 reads it as absent, so a file
  that declares required processing we cannot even parse develops silently wrong) and
  **(2) the stage functions (`demosaic_into*`, `to_srgb_linear_into`, `apply_warp_into`)
  that skip the required-processing check** — **DEFER to a post-0.2.0 spec (SPEC-056).**
  Both are low-severity: (1) needs a file carrying *both* a malformed opcode-list tag and
  mandatory processing (no corpus file does), and (2) only affects a consumer who calls the
  primitives below `develop_*` directly. They are recorded as **known limitations in the
  0.2.0 release notes**, not gates. 0.2.0's purpose is to unblock pyirradiance's move to a
  versioned dependency; holding it for two narrow opcode-path items would be the wrong
  trade. They are a focused 0.2.1 spec.

### 4. 0.2.0 does not wait for STAGE-008 colour

The colour develop path already shipped (STAGE-012), so 0.2.0 ships it. The release notes
**make no colour-accuracy claim** — SPEC-035 (ColorChecker ΔE) remains blocked on the
physical chart, and the pre-0.2.0 triage said the notes must not claim accuracy until it
lands. The frame's "waits for STAGE-008" note is stale and is resolved here as "no".

## Why

The staff review flagged `Cfa`/`WarpRect`/`Opcode` as the public types still unsealed
after `Sensor`; sealing them now is the last additive-safety move before the API
stabilizes, and it is free only at a minor-version boundary with no external constructor.
pyirradiance cannot move off a path dependency until a crates.io version carries the
demosaic/validate/kind work, so cutting 0.2.0 is the whole point of the wave — it should
ship the safety and capability that landed, name its known limitations honestly, and not
wait on narrow follow-ups or on a colour-accuracy oracle that is gated on hardware.

## Rejected alternatives

- **calver (`vYYYY.MM.PATCH`)** — rejected: wrong for a public-API crate whose consumers
  pin semver ranges; the frame itself says semver for a library.
- **Hold 0.2.0 for the two opcode follow-ups** — rejected: low severity, no corpus file
  hits them, and they delay the dependency unblock; a 0.2.1 spec (SPEC-056) is the right
  home.
- **Seal with a crate-wide wildcard match on `Opcode`** — rejected: the seal goes on the
  type; in-crate matches stay explicit so a future variant is still a visible diff.
- **Wait for STAGE-008 / ColorChecker** — rejected: colour develop already shipped;
  accuracy is a separate, hardware-gated claim the notes simply will not make.

## Mechanics

- `src/ifd.rs` / `src/opcode.rs`: add `#[non_exhaustive]` to `Cfa`, `WarpRect`, `Opcode`.
- `Cargo.toml`: extend the `test-fixtures` feature's constructors to `Cfa`/`WarpRect`/
  `Opcode`; add the self dev-dependency with `test-fixtures` + `irr` enabled; set
  `version = "0.2.0"`; confirm `keywords`/`categories`/`license`/`description` (DEC-028).
- `.repo-context.yaml`: `spec.version.scheme: semver`.
- `tests/`: migrate the `Cfa`/`WarpRect`/`Opcode` literal fixtures onto the constructors.
- `CHANGELOG.md`: promote `[Unreleased]` to `## [0.2.0] - <date>` (DEC-006); a provenance
  ledger row is not needed (no new algorithm — a seal + release).
- Release notes name the two deferred opcode follow-ups as known limitations.

## Scope and non-goals

- **In (SPEC-042 build):** the three seals + fixture migration; the semver/0.2.0 version
  bump and metadata; follow-up (3); CHANGELOG promotion; the tier-B corpus evidence run on
  the candidate SHA; the release notes; the Phase-1 reversible PR.
- **Out:** the tag + `cargo publish` (Phase 2, human-gated — "prep-complete, cut-deferred");
  the two deferred opcode follow-ups (SPEC-056, 0.2.1); any colour-accuracy claim; any new
  capability.
