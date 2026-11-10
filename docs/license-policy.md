# License policy — an optional dependency gate

This is the sibling of the cost-capture gate (`docs/cost-tracking.md`): a
discipline that documentation can't keep, made mechanical with a check + a CI
job. It is **opt-in and per-language** — the template core ships no license tool,
because the right tool depends on your ecosystem.

Wire it up when your project pulls in third-party dependencies and you care about
what licenses end up in your shipped artifact (constraint `license-policy` in
`guidance/constraints.yaml`).

## Why

A statically-linked binary inherits the licenses of everything compiled into it.
A single copyleft (GPL/AGPL) dependency can force you to relicense the *whole*
artifact — which silently breaks an otherwise permissive (MIT / Apache-2.0)
project, including any closed-source or commercial use. License drift is invisible
until someone audits it. The fix is the same shape as every other gate here:
declare the policy, enforce it in CI, fail the build on a violation.

## The pattern (any language)

1. **Pick the allowed set.** Usually permissive-only: `MIT`, `Apache-2.0`,
   `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `0BSD`, `Unicode-3.0`.
2. **Choose the tool for your ecosystem** and run it locally (a `just` recipe)
   and in CI (a job that fails on a disallowed license):

   | Ecosystem | Tool | Check command |
   |---|---|---|
   | Rust | [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | `cargo deny check licenses` |
   | Python | [pip-licenses](https://pypi.org/project/pip-licenses/) / pip-audit | `pip-licenses --fail-on 'GPL'` |
   | Node | [license-checker](https://www.npmjs.com/package/license-checker) | `license-checker --onlyAllow 'MIT;Apache-2.0;BSD;ISC'` |
   | Go | [go-licenses](https://github.com/google/go-licenses) | `go-licenses check ./...` |

3. **Check the tool sees what the policy covers.** If the policy includes
   dev-dependencies, confirm the tool actually evaluates them — cargo-deny does
   not by default (see the Rust example below) — and prove it with a mutation: add
   a disallowed dependency in a temp copy and watch the gate go red.
4. **Allow narrow, named exceptions** — never widen the global allow-list for one
   transitive dependency. Record *why* the exception is safe (e.g. weak copyleft
   like LGPL does not relicense a dynamically-used library; a fuzz-only transitive
   isn't in the shipped binary).

## Rust worked example (cargo-deny)

This is the concrete version of the pattern. It is **not** part of the template
core — copy it into your repo only if the project is Rust.

`deny.toml` at the repo root:

```toml
# cargo-deny — license policy enforcement.
#
# This crate is distributed under a permissive license, so the binary stays
# widely embeddable, including in closed-source/commercial products. To keep that
# promise, every dependency must carry a permissive (or an explicitly excepted)
# license. Any GPL/AGPL — or any license not in `allow` and not excepted — fails
# `cargo deny check licenses` (and the CI `licenses` job). Run locally: `just deny`.

[graph]
# Evaluate the FULL feature graph so a copyleft dependency cannot slip in behind
# an off-by-default feature flag unnoticed.
all-features = true

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Zlib",
    "0BSD",
    "Unicode-3.0",
]

# Per-crate exceptions: a license OUTSIDE `allow`, knowingly accepted for ONE
# named crate. Everything else still fails — including any NEW GPL/AGPL/LGPL dep.
# Document why each exception is safe.
exceptions = [
    # { name = "some-crate", allow = ["LGPL-3.0-or-later"] },
]

# Require high confidence when matching a license file's text to an SPDX id.
confidence-threshold = 0.9

# Evaluate DEV-dependencies too. cargo-deny's licence check skips them by
# default; without this line a GPL dev-dependency passes. See below.
include-dev = true
```

### Dev-dependencies are NOT checked by default — this repo turns them on

In this repo the policy covers dev-dependencies (constraint
`no-copyleft-dependencies`: "including dev-dependencies"). cargo-deny's licence
check does **not**, by default: since cargo-deny PR#557 it skips any crate reached
only through dev-dependency edges, unless `[licenses] include-dev = true` is set.
Measured here on cargo-deny 0.19.9 (SPEC-055, review finding #4, DEC-048): `cargo
deny check licenses` gathered all 48 crates of the root graph — `toml`,
`ssimulacra2` and their transitives included — then evaluated the licence of
exactly one, `irradiance` itself. A GPL-3.0-only dev-dependency of the root or of
`fuzz/` passed both gates. The gate was weaker than the policy and nothing said so.

Two things that look like the knob and are not (both probed, both changed
nothing): `exclude-dev = false` under `[graph]`, and an empty `[workspace]` table
in `Cargo.toml`. The knob is in `[licenses]`. Also, `cargo deny list` ignores it
and still prints only `irradiance` — read `cargo deny -L debug check licenses`,
whose accepted list names every crate it evaluated.

What the gate covers now, exactly:

| Graph | Invocation | Normal deps | Build deps | Dev deps |
|---|---|---|---|---|
| library (root) | `just deny` / CI `licenses` | yes | yes | **yes** (`include-dev`) |
| `irradiance-fuzz` (DEC-011) | `just deny-fuzz` / CI `licenses-fuzz` | yes | yes | **yes** (`include-dev`) |

Proof, not assertion: `scripts/deny-red-proof.sh` (`just deny-red-proof`, CI
`licenses-red-proof`) runs the control green on both graphs with `toml` /
`ssimulacra2` named in the accepted list, then injects a synthetic GPL-3.0-only
**dev**-dependency (a local path crate in a temp dir) into the root graph and into
the fuzz graph and asserts each is rejected by name, then asserts the same
mutation **passes** with `include-dev` removed — so the red is pinned to that one
line. Delete it from `deny.toml` and CI goes red.

Scope is unchanged by this: cargo-deny reads the **declared** licence expressions
of cargo packages. It still does not see provenance or vendored non-Rust source
(see the `no-copyleft-dependencies` enforcement text).

A `just deny` recipe (in your project's `justfile`):

```just
# Enforce the permissive license policy over the dependency tree
# (install: cargo install cargo-deny). Same check the CI `licenses` job runs.
deny:
    cargo deny check licenses
```

A CI job (in `.github/workflows/ci.yml`):

```yaml
  licenses:
    name: license policy (cargo-deny)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check licenses
```

Then point the `license-policy` constraint's `rationale` at this doc and bump its
severity to `blocking` once the gate is live.
