#!/usr/bin/env bash
# scripts/doc-check.sh — the rustdoc gate (SPEC-056).
#
# `cargo doc` with warnings denied, over the library's public docs, on both
# feature surfaces:
#
#   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib --all-features
#   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib --no-default-features
#
# Why: 0.2.0 shipped 19 `rustdoc::private_intra_doc_links` errors — public docs
# linking private items, which docs.rs renders as plain text — because no gate
# ever built the docs (HANDOFF-124). The fix is to reword, never to silence:
# no `#[allow]`, no `--document-private-items`.
#
# Usage: doc-check.sh [crate-dir]   (default: this repo's root)
#
# The optional crate dir is what lets scripts/doc-check-red-proof.sh run THIS
# exact invocation over a mutated copy, so the red-proof tests the gate CI runs
# and not a paraphrase of it.
#
# PATCH-004: a gate states which tool produced its verdict. This prints the
# `rustdoc` that answered and where it resolved from.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="${1:-$(cd "${SCRIPT_DIR}/.." && pwd)}"

fail() { printf '✗ doc-check: %s\n' "$1" >&2; exit 1; }

[ -f "${CRATE_DIR}/Cargo.toml" ] || fail "no Cargo.toml in ${CRATE_DIR} — nothing to document"

if ! RUSTDOC_VERSION="$(rustdoc --version 2>&1)"; then
    fail "\`rustdoc --version\` failed — rustdoc is not available, so this gate can prove NOTHING. Output: ${RUSTDOC_VERSION}"
fi
RUSTDOC_BIN="$(command -v rustdoc 2>/dev/null || echo 'NOT ON PATH')"
printf 'doc-check: using %s (%s)\n' "${RUSTDOC_VERSION}" "${RUSTDOC_BIN}"

cd "${CRATE_DIR}"
surfaces=0
for features in --all-features --no-default-features; do
    printf 'doc-check: cargo doc --no-deps --lib %s (RUSTDOCFLAGS=-D warnings)\n' "${features}"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --lib "${features}"
    surfaces=$((surfaces + 1))
done

# Both surfaces must actually have been documented; a loop that ran zero times
# is not a pass (AGENTS.md §16 rule 2).
[ "${surfaces}" -eq 2 ] || fail "expected to document 2 feature surfaces, documented ${surfaces}"

printf '✓ doc-check: %s feature surfaces documented with -D warnings, 0 rustdoc warnings.\n' "${surfaces}"
