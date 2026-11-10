---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-048
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
  - deny.toml
  - Cargo.toml
  - .github/workflows/ci.yml
  - app.just
  - docs/license-policy.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-055-license-gate-dev-graph-and-package-surface.md
  - scripts/

tags:
  - licenses
  - cargo-deny
  - dev-dependencies
  - packaging
  - irr-binary
  - stage-010
  - spec-055
---

# DEC-048: Evaluate the licence gate over the dev-graph, and gate the `irr` binary

## Decision

Two additive release-hygiene fixes (review findings #4 and #6):

1. **Licence gate over the dev-dependency graph.** Make `cargo deny check
   licenses` evaluate the crate's **dev-dependencies** (root graph and the `fuzz/`
   graph), closing the gap between `docs/license-policy.md` (which says dev-deps are
   covered) and the gate (which today checks essentially nothing). The binding
   acceptance test is a **mutation red-proof**: a synthetic GPL dev-dependency makes
   the gate go **red**, with the unmutated control green — for both `just deny` and
   `just deny-fuzz`.
2. **Gate the internal `irr` binary out of the default build/package/install.** Add
   an empty `irr` cargo feature and declare the bin with
   `required-features = ["irr"]`, so a default `cargo build` / `cargo install` /
   `cargo package` does not produce the `irr` executable, while `--all-features`
   (dev, oracle, CI) still builds it. Add a **package-target-list check** that fails
   if the default-feature build regains an unexpected binary.

## Why — #4 (the gate is weaker than the policy)

Probed 2026-11-10 on cargo-deny 0.19.9: `cargo deny list` and `cargo deny check
licenses` evaluate a graph containing **only** `irradiance@0.1.1` (Apache-2.0 /
MIT). The dev-dependencies `toml` and `ssimulacra2` **do** resolve
(`cargo tree -e dev` lists both) but are **absent** from cargo-deny's graph, so a
GPL dev-dependency would pass the gate. `cargo deny check licenses --help` documents
`--exclude-dev` as "excludes all dev-dependencies, **not just ones for non-workspace
crates**" — i.e. cargo-deny already drops the dev-deps of non-workspace crates by
default, and this is a single-package repo with no `[workspace]`.

**Two obvious knobs did NOT fix it in the probe** (recorded so the build cycle does
not repeat them): adding `exclude-dev = false` under `[graph]`, and adding an empty
`[workspace]` table to `Cargo.toml`, each still listed only `irradiance@0.1.1`. So
the exact cargo-deny configuration is a **build-cycle investigation**; the mutation
red-proof is what guarantees correctness regardless of the mechanism, and it must be
wired into CI beside the existing `licenses` job (`.github/workflows/ci.yml:142-149`)
and given a `just` recipe, mirroring the lint / gate red-proofs already in the repo.
New signal `gate-weaker-than-policy` (N=1).

## Why — #6 (the package ships the internal tool)

`Cargo.toml` declares `[[bin]] name = "irr"` unconditionally, and the `include`
allow-list carries `src/**/*.rs`, so `cargo package` ships the oracle tool's source
and `cargo install irradiance` builds and installs `irr` — contradicting
`library-not-application`. The tests **invoke** `irr` via `CARGO_BIN_EXE_irr`
(`tests/demosaic_oracle.rs:824`), so it must stay buildable for the dev/oracle
build. Disposition options and the choice:

- **`required-features = ["irr"]` on an empty `irr` feature — CHOSEN.** Default
  builds/install/package skip the bin (a target with unmet required-features is not
  built); `just test` runs `cargo test --all-features`, which enables `irr` so
  `CARGO_BIN_EXE_irr` still resolves; the published manifest stays self-consistent
  (the bin's source still ships, harmlessly). Mirrors the existing `fuzzing` feature
  precedent.
- *Move to `examples/`* — rejected: `env!("CARGO_BIN_EXE_irr")` is a `[[bin]]`-only
  mechanism, so this breaks the demosaic oracle test's invocation.
- *`exclude` from the package* — rejected: it leaves a `[[bin]]` pointing at an
  excluded source file in the published manifest, so `cargo install` fails to build.

This resolves **open maintainer decision #2** from the triage; confirm at PR review.

## Mechanics

- `Cargo.toml`: add `irr = []` under `[features]`; add `required-features = ["irr"]`
  to the `[[bin]]`. Reconfirm `just test` (`--all-features`) still builds `irr` and
  the oracle test passes.
- `deny.toml` / `app.just`: whatever cargo-deny 0.19.9 needs so the dev-graph is
  evaluated for both the root and `fuzz/`; a `just deny-red-proof` (+ a `deny-fuzz`
  variant or one recipe covering both) that injects a synthetic GPL dev-dep and
  asserts the gate fails, control first (`oracle-must-be-shown-red`, DEC-009).
- CI: wire the red-proof beside the `licenses` / `licenses — fuzz graph` jobs.
- Package surface: a check (a test, or a script over `cargo metadata` / a default
  `cargo build --message-format=json`) asserting the default-feature target list has
  no `irr` binary; run it in CI.
- `docs/license-policy.md`: reconcile its dev-dep claim with the now-real behaviour.

## Scope and non-goals

- **In:** the dev-graph licence evaluation + its mutation red-proof (root + fuzz);
  the `irr` feature-gate + package-target-list check; the doc reconcile; a CHANGELOG
  entry.
- **Out:** any change to the licence **policy** allow-list or `exceptions`; any new
  runtime or dev dependency; the `fuzzing` feature; the parser guard (SPEC-052) and
  the Compression-7 shape binding (SPEC-053).
