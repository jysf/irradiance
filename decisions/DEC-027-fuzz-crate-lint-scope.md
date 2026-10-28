---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-027
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

created_at: 2026-10-27
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - "*.just"
  - scripts/lint-fuzz-red-proof.sh

tags:
  - linting
  - fuzzing
  - testing
---

# DEC-027: the fuzz crate gets clippy's default `-D warnings`, not the five panic-free lints

## Decision

**`fuzz/` is linted by a SECOND clippy invocation —
`cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings` —
and its scope is clippy's DEFAULT lint set promoted to error by `-D warnings`,
NOT the library's five panic-free lints** (`clippy::unwrap_used`,
`expect_used`, `indexing_slicing`, `panic`, `arithmetic_side_effects`).

Two parts, decided together:

1. **A second invocation, because the fuzz crate is a second graph.** DEC-011
   keeps `irradiance-fuzz` outside the library's cargo graph so the library's
   `[dependencies]` stays empty, and clippy — exactly like cargo-deny — lints
   the graph rooted at whatever manifest it is pointed at, one per run. The
   library's `cargo clippy` at the repo root (`just lint` / `just lint-ci`, the
   `clippy` CI job) therefore compiles not a single fuzz target. This extends
   DEC-011's **"two invocations, both required"** principle from licensing to
   linting: `--manifest-path fuzz/Cargo.toml` is the whole fix, the same one
   `just deny-fuzz` made. See DEC-011.

2. **Default lints only — the panic-free policy stops at the library boundary.**
   The fuzz crate gets what `-D warnings` catches on clippy's defaults. It does
   **not** carry the five-lint `#![deny(...)]` policy that `src/lib.rs` carries.

The gate ships with a red-proof (`scripts/lint-fuzz-red-proof.sh`, DEC-009's
mechanism), because `oracle-must-be-shown-red` was widened to cover gates
(SPEC-003/FU-14). The fuzz crate is already clippy-clean on default lints (0
warnings on CI-pinned 0.1.98 and local 0.1.99, even under the five panic-free
lints), so SPEC-011 ships **no code fixes**.

## Context

SPEC-011 closes the gap where `fuzz/` sat outside every `-D warnings` gate — the
exact shape `no-copyleft-dependencies` had before SPEC-003 added `--manifest-path
fuzz/Cargo.toml` to the licence gate. The structural half (a second invocation)
was never in doubt; it mirrors DEC-011 directly. What needed deciding, and was
left open at framing, is the **scope**: should the fuzz crate carry the same
five panic-free lints as the library?

## Why the scope is default lints, not the five panic-free lints

The panic-free policy exists for one reason, stated in `constraints.yaml`'s
`no-panics-on-untrusted-input`: **a panic on attacker-controlled input is a
denial-of-service vulnerability in the shipped library.** Its scope is `--lib`
on purpose — the constraint and `just lint-no-allow` both exclude `#[cfg(test)]`
modules and `src/bin/irr.rs`, and the toolchain brief is explicit that `irr` is
"a dev/oracle binary, not a product surface — `unwrap()` is fine there."

`fuzz/` is the same class of surface as those exemptions:

- **It is dev-only glue and never shipped.** A consumer of `irradiance` never
  compiles a fuzz target; DEC-011 is built on exactly that (the arrow points
  inward, `path = ".."`, never outward).
- **A panic in a fuzz harness is not the vulnerability class the policy
  guards.** libFuzzer's entire job is to drive the *library* until *it* panics;
  a panic the harness itself introduces is a test bug libFuzzer surfaces as a
  crash, not a DoS reachable by any library consumer. Forbidding `unwrap()` in
  the harness would forbid the harness from doing its job in the plainest way.
- **The precedent is settled.** `irr` and `#[cfg(test)]` are already exempt from
  the five lints for the same "not a library path" reason. The fuzz crate is a
  third instance of that same category, not a new question.

Default `-D warnings` still buys the real thing the gate is for: the second
cargo graph no longer drifts lint-dirty unseen. Every unused import, needless
clone, or `unnecessary_cast` a future fuzz target might accrue now fails a gate,
which is the hole SPEC-011 was written to close.

## The counter-argument, weighed

The five lints are not free of merit here, and the honest reason to record this
rather than assume it: **the targets pass all five today, for free.** They are
written defensively (`let Ok(..) = .. else { return }`, `.get()`, `checked_*`),
so adopting the five would cost nothing *now* and would stop a future target
from reaching for `unwrap()` on a seed it "knows" is well-formed.

It is rejected for scope discipline, not because it is worthless:

- The gate SPEC-011 exists to add is the **missing graph coverage**, not a new
  policy. Bundling a policy expansion into a plumbing spec is how scope creeps.
- Applying `#![deny(...)]` to a crate the policy's own rationale says it does not
  cover would make the `--lib` scope and the `irr` exemption read as accidents
  rather than the deliberate boundary they are.
- It costs nothing to defer: the targets pass the five today, so **a future spec
  can tighten** `fuzz/` to the five lints with no code changes, should a target
  ever be written that needs `unwrap()` and that we would rather it did not. That
  spec would be a policy decision made on its own merits, which is where it
  belongs.

If `fuzz/` is ever tightened to the five lints, this decision is the thing that
spec supersedes or amends — not DEC-011, which is about dependency hygiene and
says nothing about linting.

## Clippy pinning (PATCH-004)

The CI job `rust / clippy — fuzz graph` uses `dtolnay/rust-toolchain@stable`,
which floats — the same floating gate as the library's `clippy` job. The local
recipe `just lint-fuzz` is **pinned** to `~/.cargo/bin/cargo +stable` with the
load-bearing `PATH` prefix, exactly like `just lint-ci` and for the same reason:
PATCH-004's 17 consecutive red runs were a floating-CI-clippy-vs-pinned-local
divergence, and the fix was a recipe that sees what CI sees. An unpinned local
`lint-fuzz` (like `just lint`) would reopen that gap for the fuzz graph, so it is
pinned and prints which clippy answered. No nightly — `fuzz/` is a plain cargo
crate for clippy; only the `cargo-fuzz` smoke jobs need nightly.

## Consequences

- **The lint gate is TWO invocations, and both are required** — the clippy
  analogue of DEC-011's closing line. `just lint` / `just lint-ci` cover the
  library; `just lint-fuzz` covers `fuzz/`. Neither sees the other's graph.
  Running one and reporting "clippy green" is a green that checked nothing about
  `fuzz/`.
- `just lint-fuzz` + `just lint-fuzz-red-proof` are added to `app.just` and
  AGENTS.md §6; the CI jobs `clippy-fuzz` and `lint-fuzz-red-proof` are added to
  `.github/workflows/ci.yml`.
- The red-proof's injected fault is a DEFAULT, warn-level clippy lint
  (`unnecessary_cast`), not a panic-free-only one — the scope this decision
  chose. It proves `-D warnings` is the gate's only teeth, since the fuzz crate
  carries no `#![deny]`.
- **The standing rule for `fuzz/`, the lint half:** the fuzz graph is linted by
  `just lint-fuzz` like any code. Tightening it beyond clippy defaults is a
  policy decision that gets its own spec and supersedes this one — it is not a
  build step.
