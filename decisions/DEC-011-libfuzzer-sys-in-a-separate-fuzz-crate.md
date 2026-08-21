---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-011
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
  - fuzz/**
  - "**/Cargo.toml"
  - "**/deny.toml"

tags:
  - dependencies
  - testing
  - fuzzing
  - licensing
---

# DEC-011: `libfuzzer-sys` lives in a separate `fuzz/` crate, outside the library's graph

## Decision

**`libfuzzer-sys 0.4` and its transitive `arbitrary`, `cc`, `cfg-if`,
`find-msvc-tools`, `getrandom`, `jobserver`, `libc`, `r-efi` and `shlex` are
dependencies of `irradiance-fuzz`, a separate package under `fuzz/` that
`cargo-fuzz` generates and manages.** The library's own `Cargo.toml` is not
touched: `[dependencies]` stays empty and `[dev-dependencies]` still holds
exactly one entry (`toml`, DEC-010).

Sanctioned by `no-new-top-level-deps-without-decision` as narrowed by DEC-004
rule 4 — a build cycle may add a clearly-trivial **dev-only** permissive
dependency provided its DEC is authored in the same pass. This is that DEC.
This repo's extra narrowing also holds: every crate above is permissive, and
none of them is a RAW decoder.

## Context

AGENTS.md §12 bar 2 requires the fuzz target to ship **in the same change** as
the first parser spec, not retrofitted. SPEC-003 is that spec. `cargo-fuzz`
(0.13.2, already installed — `guidance/toolchain-brief.md`) is the only route to
a libFuzzer target on this toolchain, and it requires the `libfuzzer-sys` macro
crate to write one.

## Why this is not a dependency of the library

`fuzz/Cargo.toml` is its own package. It depends on `irradiance` by path; the
arrow points **inward**, never outward. Nothing in `fuzz/` is reachable from
`src/`, so:

- a consumer of `irradiance` never sees any of these crates;
- `cargo build`, `cargo test`, `cargo clippy --all-targets` and the MSRV job at
  the repo root do not compile them (measured: the root gates are unchanged by
  this addition);
- the claim *"irradiance has no dependencies"* stays true as written.

`cargo fuzz` also needs **nightly**, which is the second reason the fuzz package
must stay out of the root's target set — the MSRV gate pins 1.90.0.

## Licences — measured by the gate, not hand-checked

⚠ **AMENDED 2026-08-20 by SPEC-003's second build cycle (HANDOFF-013), after
SPEC-003's verify cycle raised SB-1.** The table that stood here was wrong on the
one crate this decision exists to sanction, and the paragraph below it asserted a
limit of `cargo deny` that does not exist. Both are corrected in place, and the
correction is recorded rather than quietly swapped, because the failure mode is
the point: this document was the *only* thing standing in for a gate over `fuzz/`,
and it got the licences wrong on its first and only use.

**The enumeration below is `cargo metadata --manifest-path fuzz/Cargo.toml
--all-features`, read whole — not a recollection of what the fuzz build printed.**

| Crate | Version | Declared licence |
|---|---|---|
| `libfuzzer-sys` | 0.4.13 | **`(MIT OR Apache-2.0) AND NCSA`** — see below |
| `arbitrary` | 1.4.2 | `MIT OR Apache-2.0` |
| `cc` | 1.4.3 | `MIT OR Apache-2.0` |
| `cfg-if` | 1.0.4 | `MIT OR Apache-2.0` |
| `find-msvc-tools` | 0.1.11 | `MIT OR Apache-2.0` |
| `getrandom` | 0.4.3 | `MIT OR Apache-2.0` |
| `jobserver` | 0.1.35 | `MIT OR Apache-2.0` |
| `libc` | 0.2.189 | `MIT OR Apache-2.0` |
| `r-efi` | 6.0.0 | **`MIT OR Apache-2.0 OR LGPL-2.1-or-later`** — see below |
| `shlex` | 2.0.1 | `MIT OR Apache-2.0` |
| `irradiance` | 0.1.0 | `MIT OR Apache-2.0` (this library, by path) |
| `irradiance-fuzz` | 0.0.0 | `MIT OR Apache-2.0` — **added here**; it had none |

### `libfuzzer-sys` and NCSA — the `AND` is conjunctive

```
libfuzzer-sys-0.4.13/Cargo.toml:36:  license = "(MIT OR Apache-2.0) AND NCSA"
```

`AND`, not `OR`: the NCSA terms apply **in addition** to the permissive pair, not
as an alternative to them. There is no way to take this crate without NCSA. It is
there because the crate vendors LLVM's libFuzzer runtime in `libfuzzer/` and
compiles it from its build script.

**NCSA is permissive** — OSI-approved and FSF Free/Libre, both reported by
cargo-deny itself; attribution and no-endorsement, no reciprocity. So
`no-copyleft-dependencies` is satisfied on substance. It was never satisfied on
*record*: NCSA is not in `deny.toml`'s `allow` list, and this document previously
claimed *"no exception entry was needed and none was added"*, which was false.
`deny.toml` now carries a **per-crate exception** naming `libfuzzer-sys` and NCSA
— chosen over widening `allow`, because `allow` is a standing graph-wide sanction
and NCSA is here for exactly one fuzz-only reason. A second NCSA crate should fail
loudly and get its own decision. The reasoning is written out at the exception.

⚠ **A provenance wrinkle worth the row, since this repo keeps a ledger for exactly
this class of thing.** The crate's README says *"All files in the `libfuzzer`
directory are licensed NCSA"* — but all **49** vendored `.cpp`/`.h`/`.def` files
carry the post-2019 LLVM header, *"Part of the LLVM Project, under the Apache
License v2.0 with LLVM Exceptions"*, and **none** mentions NCSA or the University
of Illinois (counted, not sampled). So the crate's declared SPDX expression and its
README are both stale against the code it actually ships. Nothing is at risk —
every reading is permissive — and the gate enforces the *declared* expression,
which is the stricter one. This is also why `Apache-2.0 WITH LLVM-exception` stays
in `deny.toml`'s `allow` list even though cargo-deny reports it as an unencountered
allowance: it is what the vendored source really claims, and cargo-deny cannot see
vendored C++ because vendored C++ is not a cargo package. The previous version of
this section had that licence attached to the vendored C++ and concluded the
allowance was therefore *needed*; it was right about the source headers and wrong
about what the gate sees.

### `r-efi` and the LGPL option

```
r-efi 6.0.0    MIT OR Apache-2.0 OR LGPL-2.1-or-later
```

**Disjunctive** — a permissive option is selectable, so nothing is violated, and
cargo-deny accepts it against `MIT`/`Apache-2.0` without an exception. It is called
out because `no-copyleft-dependencies` names LGPL explicitly and says *"including
dev-dependencies"*, and this is the only crate anywhere in either graph whose
licence expression mentions LGPL at all. An unrecorded LGPL mention is precisely
what a provenance ledger exists to surface, disjunctive or not. It reaches the
graph as a `getrandom` backend for the `uefi` target and is never built here.

### The gate reaches `fuzz/`. It always did.

This section previously read: *"`cargo deny` evaluates the graph rooted at the
**library**, so it does not reach `fuzz/`. That is a real limit of the gate."*
**That is wrong.** cargo-deny evaluates the graph rooted at whatever manifest it is
pointed at, and pointing it at `fuzz/Cargo.toml` is one flag:

```bash
cargo deny --manifest-path fuzz/Cargo.toml check licenses
```

Measured 2026-08-20, before any fix: it **runs**, and it **fails**, catching both
of the defects the hand-check missed — `irradiance-fuzz` unlicensed, and NCSA not
allowed. The gate was never absent. It was never *invoked*. A hand-check was
substituted for a gate that was one flag away, and the hand-check then got the
answer wrong, which is the argument against hand-checks in general and not just
this one.

It is now wired as a real gate — `just deny-fuzz`, the CI job
`rust / license policy — fuzz graph (cargo-deny)`, and a line in AGENTS.md §6 —
so it is the tenth gate, not a paragraph. `guidance/constraints.yaml`'s
`no-copyleft-dependencies` `enforcement:` field names both invocations, because
`just deny` alone is a green that checked nothing about `fuzz/`.

**The standing rule for `fuzz/`, replacing "gets the same treatment or it does not
land":** a dependency added to `fuzz/` is checked by `just deny-fuzz` like any
other. If it needs an exception, the exception is named in `deny.toml` with its
reason, and the reason is a licence *read*, not a licence *remembered*.

## Alternatives considered

- **`afl.rs`** — same shape of dependency, less common, and `cargo-fuzz` is
  already installed and already proven on this host (32.9 M execs at design).
- **A hand-rolled loop over the seed corpus.** It is genuinely useful and it is
  *also here*: `ifd_survives_every_truncation_of_a_valid_container` and
  `ifd_survives_single_byte_corruption` in `tests/ifd_reader.rs` run on every
  commit with no nightly and no dependency. But a deterministic sweep explores
  what its author thought of; coverage-guided fuzzing explores what the code
  reacts to. The 26-byte input that caught SPEC-003's planted fault was one of
  our own seeds — the two lanes are complements, not substitutes.
- **No fuzz target.** Refused by AGENTS.md §12 bar 2 and by SPEC-003's
  acceptance criteria 4 and 5.

## Consequences

- The fuzz corpus splits in two: `fuzz/seeds/ifd/` is **committed** (hand-built
  tier-A fixtures, own work, regenerated by `cargo run --example fuzz-seeds`),
  and `fuzz/corpus/ifd/` — what libFuzzer discovers — is gitignored, so a fuzz
  run leaves `git status` clean.
- Running the target needs the PATH fix from `guidance/toolchain-brief.md`'s
  "SECOND `+toolchain` trap"; `just fuzz` encodes it so nobody rediscovers it.
- `fuzz/Cargo.lock` is covered by the existing gitignored `Cargo.lock` pattern.
- **The licence gate is TWO invocations, and both are required.** `just deny`
  covers the library; `just deny-fuzz` covers `fuzz/`. Neither sees the other's
  graph. Running one and reporting "licences green" is the exact mistake this
  decision made on its first pass.
