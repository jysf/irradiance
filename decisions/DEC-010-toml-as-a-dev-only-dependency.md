---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-010
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-20
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - "**/Cargo.toml"
  - tests/support/**
  - examples/corpus-status.rs

tags:
  - dependencies
  - testing
  - corpus
  - licensing
---

# DEC-010: `toml` is a dev-dependency; the library keeps zero dependencies

## Decision

**`toml = { version = "0.8", default-features = false, features = ["parse"] }`
goes in `[dev-dependencies]`, and nowhere else.** It parses
`tests/corpus/manifest.toml` for the corpus tests and the `corpus-status`
example. `[dependencies]` stays **empty**, so the claim *"irradiance has no
dependencies"* remains true as written — a dev-dependency is never compiled
into the library and never reaches a consumer's dependency graph.

Sanctioned by `no-new-top-level-deps-without-decision` as narrowed by DEC-004
rule 4: a build cycle may add a clearly-trivial **dev-only** permissive
dependency provided its DEC is authored in the same pass. This is that DEC.
This repo's extra narrowing also holds: `toml` is permissive and it is not a
RAW decoder.

## Context

DEC-003 made `tests/corpus/manifest.toml` the committed index of a corpus that
is deliberately *not* committed. The manifest shipped seeded with 7 entries and
its own header recorded the reader as scheduled debt:

> ⚠ NO READER YET, and that is a scheduled debt, not an oversight. Nothing on
> this machine parses TOML […] The real reader is Rust's `toml` crate and it
> arrives with the first Cargo.toml, in STAGE-001's corpus spec.

SPEC-002 is that spec. An unread manifest is precisely the defect AGENTS.md §11
names ("ship the reader with the field"), and the fix needs a TOML parser.

### The feature set is measured, not guessed

Probed at design against this crate, on `cargo +1.90.0`:

| config | crates in graph | parses? |
|---|---|---|
| `toml = "0.8"` | 12 | yes |
| `default-features = false, features = ["parse"]` | **11** | **yes** |
| `default-features = false` | 6 | **NO** — `Value: FromStr` unsatisfied |

The third row is the interesting one and the reason this table is in the record:
it **passes `cargo check`**, because nothing in a crate that merely declares a
dependency calls its API. That is a shape-check standing in for a
behaviour-check — the substitution AGENTS.md §12 warns about — and it would have
shipped a dependency that cannot do the one job it was added for.

Re-measured at build: the resolved graph is **11 crates** (confirmed by
`cargo tree -e normal,build,dev`), MSRV 1.90 holds
(`cargo +1.90.0 check --all-targets --all-features` → 0), and
`cargo deny check licenses` → **licenses ok**. Every crate pulled in is
MIT/Apache-2.0 or MIT, so `no-copyleft-dependencies` is satisfied with room to
spare:

```
equivalent 1.0.2       Apache-2.0 OR MIT      serde_core 1.0.229     MIT OR Apache-2.0
hashbrown 0.17.1       MIT OR Apache-2.0      serde_spanned 0.6.9    MIT OR Apache-2.0
indexmap 2.14.0        Apache-2.0 OR MIT      toml 0.8.23            MIT OR Apache-2.0
serde 1.0.229          MIT OR Apache-2.0      toml_datetime 0.6.11   MIT OR Apache-2.0
toml_edit 0.22.27      MIT OR Apache-2.0      winnow 0.7.15          MIT
```

`features = ["parse"]` buys `toml::Value: FromStr`, which is the whole API the
reader uses — it walks the `Value` tree by hand. Taking the `serde` derive route
instead would have cost a *second* dev-dependency (`serde` with `derive`, and
its proc-macro chain) to save perhaps forty lines of field extraction. It also
happens to be why the reader's error messages can name the offending entry and
key, which a derive-based one would not do for free.

### A consequence for where the reader can live

Cargo makes dev-dependencies available to tests, examples and benches — **not
to `[lib]` or `[[bin]]`**. So the manifest reader physically cannot live in
`src/`, and `irr` cannot grow a `corpus status` subcommand without either
promoting `toml` to a runtime dependency (refused, below) or hand-rolling a TOML
parser (absurd). That is why the reader is `tests/support/corpus.rs`, shared by
`tests/corpus_manifest.rs` and `examples/corpus-status.rs` via `#[path]`, and
why the visible corpus-status surface is an **example** rather than a subcommand
of `irr`. This is a constraint of the choice, not an accident of it.

## Alternatives Considered

- **Option A: `toml` as a normal (runtime) dependency.**
  - Why rejected: it would falsify a public claim for zero benefit. Nothing in
    the library reads TOML; `library-not-application` says `irradiance` takes
    bytes and returns pixels. It would also put a parser chain into every
    consumer's graph, and DEC-004 rule 4 sanctions **dev-only** deps precisely
    because runtime deps are the ones that deserve a stop-and-ask.

- **Option B: hand-roll a minimal TOML subset parser, keeping zero deps of any
  kind.**
  - Why rejected: the manifest already uses multi-line basic strings, nested
    `[file.oracle]` tables and arrays of tables. A "minimal subset" parser is
    either wrong on those or is a TOML parser. It would be library-quality
    effort spent on test scaffolding, and every hour of it is an hour not spent
    on the decoder. The zero-dependency promise is about what consumers link,
    and a dev-dependency does not touch that.

- **Option C: don't parse the manifest — keep a hardcoded list of paths in the
  tests.**
  - Why rejected: this is the status quo the spec exists to end. It is the
    unread-field defect (AGENTS.md §11) with an extra copy of the data beside
    it, and it guarantees the two drift. SPEC-002 acceptance criterion 1 forbids
    it outright: *no test in this repo hardcodes a corpus path.*

- **Option D (chosen): `toml`, dev-only, `features = ["parse"]`.**
  - Why selected: it is the only option that reads the real manifest, costs
    consumers nothing, keeps the licence surface permissive, and is one crate
    cheaper than the default feature set while still actually parsing.

## Consequences

- **Positive.** The manifest has a reader, so DEC-003's provenance rule
  ("EVERY entry MUST carry `licence` and `source`") is now enforced *mechanically*
  at parse time rather than by remembering — an entry missing either is rejected
  by name. The pinned `sha256` and `oracle.raw_checksum` are readable by every
  spec downstream, which is what SPEC-003/004/005 need.

- **Negative.** `cargo test` now compiles 10 extra crates on a cold build, and
  `Cargo.lock`-less builds are exposed to those crates' future breakage. The
  library's own build (`cargo build`) is untouched.

- **Negative, and worth naming.** This is the first crack in "zero dependencies"
  as a *slogan*, even though the technical claim is intact. Anyone repeating the
  claim must say **"the library has no dependencies"**, not "the repo has none".
  README wording should be checked when a README lands.

- **Neutral.** `sha256` is deliberately **not** a second dependency. It is
  implemented in `tests/support/corpus.rs` from FIPS PUB 180-4 (provenance class
  1 — published specification) and proven against the published NIST vectors on
  every run. A hashing crate would have been exercised only on machines that
  hold the corpus, so a broken integration would have been invisible in CI —
  the same invisibility this spec exists to remove. See
  `docs/provenance-ledger.md`.

## Validation

Right if:

- `[dependencies]` in `Cargo.toml` is still empty at PROJ-001's close, and
  `cargo tree -e normal` shows `irradiance` alone.
- `cargo deny check licenses` keeps reporting `licenses ok` with no exceptions
  added on `toml`'s behalf.
- No later spec needs `toml` at runtime. If one appears to, that is a signal the
  library is growing an application's responsibilities
  (`library-not-application`), not a signal to promote the dependency.

Revisit if:

- The manifest schema outgrows what hand-walking a `toml::Value` can express
  cleanly — the answer then is `serde` with `derive` as a second dev-dependency,
  re-measured, not a rewrite.
- `toml` 0.9+ changes the `parse` feature's contents; re-run the three-row table
  above rather than trusting this one.

## References

- Related specs: SPEC-002 (`projects/PROJ-001-monochrome-dng-develop/specs/`)
- Related handoff: HANDOFF-008
- Related decisions: DEC-003 (corpus storage and the manifest this parses);
  `docs/decisions/DEC-004` rule 4 (the dev-dep exception being used) — note the
  namespace, per AGENTS.md §10
- Constraints: `no-new-top-level-deps-without-decision`,
  `no-copyleft-dependencies`, `library-not-application`
- Provenance: `docs/provenance-ledger.md` (the SHA-256 row)
- Crate: [`toml` 0.8.23](https://docs.rs/toml/0.8.23/toml/) — `MIT OR Apache-2.0`
