---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-036
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-003
repo:
  id: irradiance

created_at: 2026-11-05
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - Cargo.toml
  - src/lib.rs
  - src/ljpeg.rs
  - fuzz/Cargo.toml
  - fuzz/fuzz_targets/ljpeg.rs

tags:
  - fuzz
  - ljpeg
  - sof-3
  - visibility
  - fuzzing-feature
  - spec-038
  - stage-009
---

# DEC-036: Fuzz the internal SOF-3 decoder through a `fuzzing` cargo feature, keeping `ljpeg` `pub(crate)`

## Decision

Expose the SOF-3 decoder to the fuzz crate through a single `#[cfg(feature =
"fuzzing")] #[doc(hidden)] pub` entry in `src/lib.rs` (e.g. `__fuzz_ljpeg(data:
&[u8])`), enabled **only** by `irradiance-fuzz` (`irradiance = { path = "..",
features = ["fuzzing"] }`). `decode_into` / `reconstruct` and the rest of `ljpeg`
stay `pub(crate)` — the published crate (default features) exposes no `ljpeg`
symbol, so STAGE-010 still owns the real public surface. The **panic-free
seed-replay check is an in-crate unit test** in `src/ljpeg.rs` (which reaches
`decode_into` directly, needing no exposure); the libFuzzer target calls the
feature-gated entry.

## Context

SPEC-036/037 deliberately kept the SOF-3 decoder `pub(crate)` (STAGE-010 decides
the public API). SPEC-038 must fuzz it **now**, before the container binding — so
the decoder has to be reachable from (a) the separate `irradiance-fuzz` crate and
(b) a panic-free replay check, without widening the published API.

Every existing fuzz target (`plane`, `ifd`, `develop`, `opcode`, `demosaic`,
`warp_opcode`) drives genuinely-`pub` API (`plane::unpack_into`, `Container`,
`develop`, …), so no target has yet needed to reach an internal decoder. The
mechanism matters because of how the gates build the fuzz crate (probed, not
assumed):

- `rust / clippy — fuzz graph (-D warnings)` runs **plain** `cargo clippy
  --manifest-path fuzz/Cargo.toml --all-targets` — **no `--cfg fuzzing`, no
  nightly** (the comment in `ci.yml` is explicit: only the `cargo-fuzz` smoke
  jobs use nightly). So any symbol the target references must exist in an
  ordinary (non-`cfg(fuzzing)`) build of `irradiance` as compiled for the fuzz
  crate.
- The library gates (`cargo clippy --all-targets --all-features`, `just test`
  which bakes `--all-features`) build with the `fuzzing` feature **on**, so the
  entry is compiled and linted under the five-lint panic-free policy there — it
  must satisfy it (bound dimensions with checked arithmetic, call `decode_into`
  into a capped buffer, drop the typed `Result`).
- A normal `cargo build` and the published crate (default features) compile it
  **out** — `ljpeg` stays fully internal.

## Alternatives Considered

- **A: make `decode_into` (or `pub mod ljpeg`) genuinely `pub`.**
  - Rejected: pre-empts STAGE-010's public-surface decision and ships an unstable
    internal decoder in the public API.
- **B: a `#[cfg(fuzzing)]` entry (the cargo-fuzz `--cfg fuzzing` convention).**
  - Rejected: the `clippy — fuzz graph` gate builds the fuzz targets with **plain
    `cargo clippy`** (no `--cfg fuzzing`), so a `cfg(fuzzing)`-gated entry would
    be absent there and `fuzz_targets/ljpeg.rs` would fail to compile → the gate
    goes red. (This is why the mechanism is a *feature*, not `cfg(fuzzing)`.)
- **C (chosen): a `fuzzing` cargo feature enabled by `irradiance-fuzz`.**
  - Present under the fuzz-graph gate (the fuzz crate turns it on) and the
    library's `--all-features` gates; absent in normal/published builds; the
    replay test reaches `decode_into` directly as an in-crate unit test and needs
    none of it.

## Consequences

- **Positive.** `ljpeg` stays `pub(crate)`; the published crate exposes no SOF-3
  symbol (STAGE-010 unconstrained); the fuzz hook is doc-hidden and absent from
  default builds. Establishes the reusable pattern for fuzzing future internal
  decoders.
- **Neutral.** The feature-gated entry is lib code held to the five-lint
  panic-free policy (it is compiled under `--all-features`); that is appropriate.
  Two reach-paths — the unit test for replay, the feature entry for the target —
  rather than one.
- **Negative — the honest cost.** A `fuzzing` feature that exists only to expose
  a fuzz hook is mild API-surface machinery; it is justified by keeping the real
  decoder internal until STAGE-010.

## Validation

Right if SPEC-038 ships a `ljpeg` fuzz target that drives the decoder panic-free,
the `clippy — fuzz graph` + `license policy — fuzz graph` gates stay green, and
`cargo doc` / the published crate show no `ljpeg` symbol. Revisit at STAGE-010:
when the container binding adds a real public entry (recognising `Compression 7`),
the target can switch to it and the `fuzzing` feature + hook may be retired.

## References

- Governs `SPEC-038`; builds on `SPEC-036`/`SPEC-037` (the `pub(crate)` decoder)
  and `DEC-035`. First internal-decoder fuzz target (the others fuzz public API).
  Constraints: `no-panics-on-untrusted-input`, `no-new-top-level-deps-without-decision`
  (a cargo *feature* is not a dependency; recorded here for the audit trail).
- Probed from `.github/workflows/ci.yml` (`clippy-fuzz` / `licenses-fuzz` jobs)
  and `fuzz/Cargo.toml` on 2026-11-05.
