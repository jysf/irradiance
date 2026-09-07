---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-023
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-06
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - "**/Cargo.toml"
  - tests/support/ssimulacra2.rs
  - tests/perceptual_oracle.rs

tags:
  - dependencies
  - testing
  - oracle
  - licensing
---

# DEC-023: `ssimulacra2` is a dev-dependency; the library keeps zero dependencies

## Decision

**`ssimulacra2 = "0.5.1"` goes in `[dev-dependencies]`, and nowhere else.**
It scores the develop-layer oracle's tier-A synthetic fixtures and dnglab's
`--srgb` output with the SSIMULACRA2 perceptual metric (`SPEC-020`,
`DEC-005`). `[dependencies]` stays **empty** — a dev-dependency is never
compiled into the library and never reaches a consumer's graph.

Sanctioned by `no-new-top-level-deps-without-decision` as narrowed by
`AGENTS.md` §13 rule 4 / `DEC-004` rule 4: a build cycle may add a
clearly-trivial **dev-only** permissive dependency provided its DEC is
authored in the same pass. This is that DEC. This repo's extra narrowing also
holds: `ssimulacra2` is permissive and it is not a RAW decoder — it consumes
already-decoded pixel arrays, no camera-file parsing of any kind.

## Context

`SPEC-020` ships the develop-layer oracle: score a candidate render against
`dnglab analyze --srgb`'s output with SSIMULACRA2, pass at ≥ 85
(`DEC-005`). The metric itself needs an implementation; crustyimg (same
author, sibling repo) already integrates the `ssimulacra2` crate at
`src/quality/mod.rs`, pinned to `0.5.1`.

### The licence check, run before this DEC was written (per `SPEC-020/AC9`)

A probe workspace (`ssimulacra2 = "0.5.1"`, this repo's real `deny.toml`
copied in) was built and checked, per `SPEC-020`'s `## Implementation
Context` requirement to re-confirm on the pinned tree rather than trust the
design-time guess:

```
$ cargo deny check licenses
licenses ok
```

`cargo deny list` on that probe workspace's full resolved graph (38 crates):

```
Apache-2.0 (29): autocfg, bumpalo, cfg-if, crossbeam-deque, crossbeam-epoch,
  crossbeam-utils, either, log, num-bigint, num-derive, num-integer,
  num-rational, num-traits, once_cell, proc-macro2, quote, rayon, rayon-core,
  rustversion, syn (x2), thiserror, thiserror-impl, unicode-ident,
  wasm-bindgen (+macro/macro-support/shared)
BSD-2-Clause (2): ssimulacra2@0.5.1, v_frame@0.3.9
MIT (37): aligned-vec, autocfg, av-data, bumpalo, byte-slice-cast, bytes,
  cfg-if, crossbeam-deque, crossbeam-epoch, crossbeam-utils, either, equator,
  equator-macro, log, num-bigint, num-derive, num-integer, num-rational,
  num-traits, once_cell, proc-macro2, quote, rayon, rayon-core, rustversion,
  syn (x2), thiserror, thiserror-impl, unicode-ident, wasm-bindgen
  (+macro/macro-support/shared), yuvxyb, yuvxyb-math
Unicode-3.0 (1): unicode-ident
```

Every crate in the resolved graph is `MIT`, `Apache-2.0`, `BSD-2-Clause` or
`Unicode-3.0` — all four already in `deny.toml`'s `allow` list, all
permissive, no `gpl`/`lgpl`/`agpl`/`unlicense` string anywhere. **Correction
to the design-time guess:** `SPEC-020.md`'s `## Implementation Context`
expected `ssimulacra2` itself to be "BSD-3-Clause OR MIT" (rust-av's stated
convention); measurement shows it is actually **BSD-2-Clause** — still
permissive, still inside `deny.toml`'s existing `allow` list with zero
changes needed, but the exact SPDX id was a guess and the guess was wrong.
Re-confirmed under the pinned MSRV toolchain too:
`~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` compiles the
probe workspace clean.

### A surprise worth naming: the dependency graph is far larger than the
design-time reference implied

`SPEC-020.md`'s `## Implementation Context` recorded "Dependencies:
`num-traits` only" (read from crustyimg's own module doc comment, which
undercounts *its* dependency by describing only the direct one). The real
resolved graph is 38 crates, including `rayon` (parallelism), `yuvxyb` (the
`Rgb`/`LinearRgb`/`Xyb` colour-space types `ssimulacra2` re-exports and
actually returns from `Rgb::new`), and `av-data`/`v_frame` (pixel-format
plumbing `yuvxyb` depends on). None of this changes the licence answer above
— every one of the 38 is permissive — but it is a real correction to the
design-time record, made by running the probe rather than trusting the
citation (`AGENTS.md` §16's "verify a claim you are about to act on" — the
sibling repo's own doc comment was itself a self-report that undercounted).

`rayon` appearing here is **not** a violation of `DEC-002`'s proposed
`no rayon` runtime-surface rule: that rule (still `status: proposed`) is
about the *library's* dependency graph on the algorithmic decode/develop
path, not about a dev-only test tool's own internal parallelism. `ssimulacra2`
is invoked only from `tests/`, never from `src/`.

## Alternatives Considered

- **Option A: hand-roll SSIMULACRA2.**
  - Why rejected: SSIMULACRA2 is a ~2000-line project of its own (multi-scale
    XYB conversion, six-scale SSIM-family blur/compare, per-scale error
    pooling with hand-tuned weights from the reference implementation's
    published constants). Reimplementing it is a correctness liability this
    spec has no way to independently verify against — it would need its OWN
    oracle, which is the exact problem this spec exists to solve for the
    develop pipeline, one level up.

- **Option B: shell out to the `ssimulacra2` CLI (the libjxl reference
  binary), already installed on this machine as a *tool*.**
  - Why rejected: a CLI shell-out is a **runtime** dependency surface for the
    harness (subject to build-time SIMD variability between machines/runs —
    the reference binary is not guaranteed bit-identical the way the Rust
    crate's pure computation is), and it is worse than a dev-dependency for
    the same reason `dnglab` is run as a tool but never linked: the CLI's
    output shape and rounding are outside this repo's control. `SPEC-020`'s
    `AC9` rejects this explicitly.

- **Option C (chosen): `ssimulacra2` crate 0.5.1, dev-only.**
  - Why selected: it is the crate crustyimg (same author) already trusts in
    production, the version is pinned and reproducible, the whole resolved
    graph is permissive (measured above), and it is not a RAW decoder — it
    takes an already-decoded pixel array, which is squarely within what a
    dev-only test tool may do under `AGENTS.md` §13 rule 4's narrowing.

## Consequences

- **Positive.** The develop-layer oracle (`SPEC-020`) has a metric
  implementation that is deterministic, pinned, and independently maintained
  — not a mirror of anything this repo's own code does.
- **Negative.** `cargo test`/`cargo check --all-targets` now compile 38 extra
  crates on a cold build (dev-only; `cargo build`, the library's own build,
  is untouched). `rayon` in particular is a nontrivial dependency to pull in
  for a test tool, though its cost is compile time only — nothing about it
  touches the shipped library's runtime characteristics.
- **Negative, and worth naming precisely.** This is the SECOND crack in "zero
  dependencies" as a slogan (`DEC-010`'s `toml` was the first). The accurate
  claim remains **"the library has no dependencies"** — `cargo tree -e
  normal` on the real crate still shows `irradiance` alone.
- **Neutral.** `ssimulacra2` itself gets a `docs/provenance-ledger.md` row,
  class 2 (third-party permissive crate) — this spec did not implement the
  metric from Cloudinary/Alakuijala et al.'s published algorithm, and is not
  reimplementing from another implementation either.

## Validation

Right if:

- `[dependencies]` in `Cargo.toml` stays empty, and `cargo tree -e normal`
  shows `irradiance` alone.
- `cargo deny check licenses` keeps reporting `licenses ok` with no exception
  added on `ssimulacra2`'s behalf.
- No later spec needs `ssimulacra2` (or any perceptual metric) at runtime. If
  one appears to, that is a signal the library is growing an application's
  responsibility (`library-not-application`), not a signal to promote the
  dependency.

Revisit if:

- `ssimulacra2` releases a version whose resolved graph pulls in a
  copyleft-licensed crate — re-run the `cargo deny check licenses` probe
  before bumping the pin, exactly as this record did before adding it.
- A future spec needs a SECOND perceptual metric (e.g. for a different colour
  space or a video-domain check) — evaluate it on its own merits rather than
  assuming this decision's reasoning transfers.

## References

- Related specs: `SPEC-020` (`AC8`, `AC9`)
- Related decisions: `DEC-005` (the develop oracle's mechanics and ≥ 85
  tolerance this metric implements); `DEC-010` (the first dev-dep sanction,
  same shape); `DEC-002` (proposed, `no rayon` — scoped to the library's own
  graph, not touched by this decision)
- Constraints: `no-new-top-level-deps-without-decision`,
  `no-copyleft-dependencies`, `library-not-application`
- Provenance: `docs/provenance-ledger.md` (the SSIMULACRA2 row)
- Crate: [`ssimulacra2` 0.5.1](https://docs.rs/ssimulacra2/0.5.1/ssimulacra2/)
  — `BSD-2-Clause`
- Reference integration: `~/PSeven/experiments/crustimg_redo_plus/crustyimg/src/quality/mod.rs`
