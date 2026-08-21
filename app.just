# app.just — YOUR project's commands (build, dev, test, deploy, …)
#
# This file is PROJECT-OWNED. The root `justfile` is template-managed and
# imports this one (`import? 'app.just'`), so:
#   - Put every app-specific recipe HERE, not in the root justfile. Template
#     updates then never conflict with your commands.
#   - Everything defined here is a normal `just` recipe: `just build`,
#     `just dev`, `just test`, etc. Run `just --list` to see them alongside
#     the template's workflow commands (`status`, `new-spec`, …).
#   - These should mirror the "Commands (exact)" block in AGENTS.md §6 — that
#     section documents them for humans; these make them runnable.
#
# Filled in by SPEC-001 (crate scaffold). `irradiance` is a Rust library —
# no dev server, no install step beyond the toolchain itself.

# Install dependencies — nothing to install; the toolchain is the dependency
install:
    cargo fetch

# Run the app locally — no dev server; this is a library. Watch-build instead.
dev:
    cargo build

# Build the app / produce the release artifact
build:
    cargo build --release

# Run the app's test suite.
#
# `corpus-status` runs FIRST and prints one line per manifest entry — present,
# or SKIP + the path it looked at. That is the whole point of the ordering:
# measured at SPEC-002 design, an `eprintln!` inside a PASSING test is captured
# (`cargo test` -> 0 SKIP lines; `cargo test -- --nocapture` -> all of them), so
# a skip announced from inside the harness is silent in exactly the case it
# exists for. A silent skip reports green for work it never did — the same
# defect class as an oracle that cannot go red (DEC-003, AGENTS.md §12 bar 4).
#
# Do NOT "fix" this by adding `-- --nocapture` below. That buries the corpus
# signal in full test output instead of surfacing it, and this recipe's
# contract is that `just test 2>&1 | grep SKIP` names any absent file with no
# extra flags.
test:
    cargo run --quiet --all-features --example corpus-status
    cargo test --all-features

# Lint / format check
lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --check

# Type check — Rust has no separate typecheck; `check` is the fast path
typecheck:
    cargo check --all-targets --all-features

# Licences — the permissive-only gate (constraint no-copyleft-dependencies)
deny:
    cargo deny check licenses

# Prove the LIBRARY's panic-free lint policy actually rejects violating code —
# one violation per policy lint, injected into a temp-dir copy of src/lib.rs,
# so src/lib.rs's own #![deny(...)] is what must reject them. The UNMUTATED
# copy is run first as a negative control and must pass; without that, a red
# proves only that clippy failed, not that the policy rejected anything
# (constraint oracle-must-be-shown-red applied to a gate; SPEC-001 acceptance
# criterion 5; DEC-009, superseding DEC-007).
# See scripts/lint-red-proof.sh.
lint-red-proof:
    ./scripts/lint-red-proof.sh

# Close the #[allow] escape hatch on the LIBRARY (SPEC-006).
#
# `just lint-red-proof` pins the crate-root #![deny(...)] policy; it cannot see
# an #[allow] BENEATH that root, and one such attribute on a pub fn shipped two
# panics past seven green gates. `-F` is `--forbid`: a forbidden lint cannot be
# re-allowed in source, and attempting it is compiler error E0453 — so this
# fires on the escape hatch itself, whether or not the code beneath it panics.
# No text matching, which is the point (attribute text lives inside src/lib.rs's
# own module docs).
#
# Scope is `--lib` deliberately: it excludes #[cfg(test)] modules (not compiled
# without cfg(test)) and src/bin/irr.rs (a different target), so the sanctioned
# exceptions need no per-site special-casing. `--all-targets` here exits 101 on
# the honest tree, because the test module legitimately allows.
lint-no-allow:
    cargo clippy --lib --quiet -- \
        -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
        -F clippy::panic -F clippy::arithmetic_side_effects
