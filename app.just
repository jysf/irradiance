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

# Run the app's test suite (tier-B tests skip loudly when the corpus is absent)
test:
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

# Prove the panic-free lint policy actually rejects a violating snippet
# (constraint oracle-must-be-shown-red applied to a gate; SPEC-001 acceptance
# criterion 5). See scripts/lint-red-proof.sh.
lint-red-proof:
    ./scripts/lint-red-proof.sh
