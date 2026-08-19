---
insight:
  id: DEC-006
  type: decision
  confidence: 0.85
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

created_at: 2026-08-18
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - tests/lint_policy_red.rs.disabled
  - scripts/lint-red-proof.sh
  - .github/workflows/ci.yml

tags:
  - ci
  - lint
  - oracle-must-be-shown-red
  - spec-001
---

# DEC-006: The lint-policy red-proof is a `.disabled` integration-test file with its own `#![deny(...)]`, swapped in by a script

## Decision

`SPEC-001` acceptance criterion 5 requires CI to compile a deliberately
violating snippet and assert the build FAILS (constraint
`oracle-must-be-shown-red`, applied to the panic-free lint gate rather than to
a decode oracle). The mechanism chosen:

1. The violating snippet lives at `tests/lint_policy_red.rs.disabled` — never
   picked up by `cargo test`/`cargo clippy` because it lacks the `.rs`
   extension cargo auto-discovers under `tests/`.
2. **The snippet carries its own `#![deny(clippy::unwrap_used, ...)]` header**,
   duplicating `src/lib.rs`'s five-lint policy verbatim.
3. `scripts/lint-red-proof.sh` copies it to `tests/lint_policy_red.rs`, runs
   `cargo clippy --all-targets --all-features -- -D warnings`, asserts the
   exit code is **non-zero**, and removes the copy on exit (`trap ... EXIT`)
   whether the assertion passes or the script errors out first.
4. CI job `lint-policy-red-proof` in `.github/workflows/ci.yml` runs the
   script; the job is green iff the snippet's compile **failed**.

## Context

Point 2 is the non-obvious part and is the reason this is a decision rather
than a mechanical transcription of the spec's `## Failing Tests` snippet
(which shows the two violating `pub fn`s with no lint attribute of their
own). **Measured empirically during this build, not assumed:** each file
under `tests/` is its own crate root in Cargo's model, so it does **not**
inherit `src/lib.rs`'s crate-level `#![deny(...)]`. A scratch reproduction
confirmed the literal spec snippet, dropped into `tests/lint_policy_red.rs`
verbatim with no attribute of its own, compiles **clean** under
`cargo clippy --all-targets --all-features -- -D warnings` — i.e. the
red-proof would silently be a false green, exactly the manufactured-confidence
failure `oracle-must-be-shown-red` exists to catch. Adding the same
`#![deny(...)]` header used in `src/lib.rs` to the snippet itself reproduces
the failure reliably (verified: clippy exits 101 with the three expected
errors — `arithmetic_side_effects`, `indexing_slicing`, `unwrap_used`).

## Alternatives Considered

- **A `[lints]` table in `Cargo.toml` shared across the whole package.**
  Would make every crate target (including `tests/*.rs` and `src/bin/irr.rs`)
  inherit the deny set automatically, so the snippet would need no attribute
  of its own. Rejected for now: it would also deny the five lints inside
  `src/bin/irr.rs`, which the spec explicitly keeps relaxed
  (`guidance/toolchain-brief.md`: *"`irr` is a dev/oracle binary... `unwrap()`
  is fine there"*) — reproducing the current per-target split with a shared
  `[lints]` table needs per-target lint overrides Cargo does not cleanly
  support pre-2024-edition. Revisit if a future spec wants one policy surface.
- **A `rustc`-only compile via `rustc --crate-type lib` outside Cargo,
  bypassing clippy entirely.** Rejected: it would only prove the `rustc`-level
  guarantees (`#![forbid(unsafe_code)]`), not the clippy restriction lints,
  which are the actual mechanism behind `no-panics-on-untrusted-input`.
- **Committing the violating snippet as a normal, always-failing `tests/*.rs`
  file with `#[ignore]`.** Rejected: `#[ignore]` skips execution, not
  compilation — `cargo test` would still compile (and, per this decision's
  finding, compile *clean*) an ignored test, so it proves nothing about the
  lint policy without the swap-in step anyway. The swap-in script is no more
  complex and avoids a permanently-red file sitting in the normal test tree.

## Consequences

- **Positive:** the red-proof runs the *same* clippy invocation every other CI
  job runs (`--all-targets --all-features -- -D warnings`), so it is testing
  the policy as actually enforced, not a hand-rolled subset. The swap-in
  script is reusable local tooling (`just lint-red-proof`), not CI-only.
- **Negative:** the snippet's `#![deny(...)]` header must be kept in sync with
  `src/lib.rs`'s by hand — if a future spec changes the lint set on the
  library, this file needs the same edit or the red-proof stops testing the
  real policy. No automated check enforces that sync; a verify-cycle reviewer
  should diff the two when either changes.
- **Neutral:** `scripts/lint-red-proof.sh` intentionally `die`s (refuses to
  run) if `tests/lint_policy_red.rs` already exists, rather than overwriting
  it — a defensive check against a prior run's cleanup having failed.

## Validation

The mechanism was run end-to-end during build: the swap-in produced the
expected three clippy errors and non-zero exit, the script itself exited 0
(asserting the expected failure occurred), and cleanup left `tests/` holding
only the `.disabled` file. A normal `cargo test`/`cargo clippy --all-targets`
immediately after was confirmed unaffected. Revisit if a lint-set change to
`src/lib.rs` is ever made without a corresponding edit here — that is the
drift this decision's "Negative" consequence predicts.

## References

- Related specs: SPEC-001
- Related constraints: `oracle-must-be-shown-red`, `no-panics-on-untrusted-input`
- Handoff: `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-001-build-crate-scaffold-cargo-toml-measured-msrv-panic-free-lints-rust-ci.md`
