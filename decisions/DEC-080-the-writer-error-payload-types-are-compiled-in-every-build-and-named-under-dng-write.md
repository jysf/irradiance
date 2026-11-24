---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-080
  type: decision
  confidence: 0.75                   # compiles, documents and semver-checks cleanly; the alternative paths trade a public name for the default build
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-005
repo:
  id: irradiance

created_at: 2026-11-24
supersedes: null
superseded_by: null

status: proposed                     # emitted by the SPEC-061 build; the architect accepts or redirects at review

deciders: [claude]

affected_scope:
  - src/write_types.rs

tags:
  - writer
  - errors
  - features
  - semver
---

# DEC-080: The writer's error payload types are compiled in every build and named under `dng::write`

## Decision

`Framing` and `WriteInputError`, the payloads of `Error::UnsupportedOpcodeForFraming` and
`Error::InvalidWriteInput`, are defined in a private, always-compiled module
(`src/write_types.rs`) and re-exported with `pub use` from `irradiance::dng::write`, the
path SPEC-061 §2 names for them.

- With the `write` feature on, they are `irradiance::dng::write::Framing` and
  `irradiance::dng::write::WriteInputError`, exactly as specified.
- Without it, the two `Error` variants still exist (SPEC-061 Outputs: "so `Error`'s shape
  does not depend on features"), and their payload types exist too, but have no public
  path. No function in that build returns either variant.
- `tests/error_kind.rs`'s source parser now also reads a tuple variant
  (`InvalidWriteInput(..)`), so the wildcard-free table still covers every variant by name.

## Context

SPEC-061 makes two requirements that meet at the payload types. The two `Error` variants
must exist in every build, and the types they carry are named under `dng::write`, a
module AC1 compiles only with the feature. A variant's field type must exist whenever the
variant does, so the types cannot live only behind the feature. The spec does not say
where they live. The build had to choose.

## Alternatives Considered

- **Compile `dng::write`'s types in every build and gate only the functions.** Rejected:
  AC1 says the default build compiles no `dng` module.
- **Define the types at the crate root (`irradiance::WriteInputError`) and re-export
  them.** Rejected: it adds two public names to the default build for a feature it does not
  have. It also gives the same type two public paths.
- **Gate the two variants on the feature.** Rejected by the spec. A crate whose `Error`
  changes shape with a feature breaks every downstream exhaustive-by-name table, such as
  `tests/error_kind.rs`, depending on which features a dependency graph unifies.
- **Make the payloads feature-independent primitives** (a `u8` for the framing, a struct
  variant per reason). Rejected: it contradicts SPEC-061 §2 and DEC-075 §7, which chose
  one reason enum.

## Consequences

- **Positive:** every SPEC-061 name is honoured, `Error`'s shape is the same in every
  build, and the default build gains no public name.
- **Negative:** in a default-features build, a caller can match
  `Error::InvalidWriteInput(reason)` and print or compare `reason`, but cannot name its
  type. Rustdoc for that build shows the field type without a link. Neither matters in
  practice, because such a build never produces the variant.
- **Neutral:** if a future spec wants the reason type usable without the writer, moving
  it to a public path is additive.

## Validation

- `cargo check --no-default-features`, the default build and `--features write` all
  compile; `just doc-check` and `cargo semver-checks` pass. See the SPEC-061 build
  handback.

## References

- Related specs: SPEC-061
- Related decisions: DEC-051, DEC-052, DEC-055, DEC-075
