#!/usr/bin/env bash
# scripts/lint-red-proof.sh — proves the panic-free lint policy actually
# bites (constraint `oracle-must-be-shown-red` applied to a gate rather than
# an oracle; SPEC-001 acceptance criterion 5).
#
# A lint policy that has never rejected anything is not a policy. This script
# compiles `tests/lint_policy_red.rs.disabled` — a deliberately violating
# snippet (bounds-check-free indexing + `unwrap()`) — as a real integration
# test file and asserts the build FAILS under the same
# `cargo clippy --all-targets --all-features -- -D warnings` invocation CI
# runs everywhere else. If it does not fail, the policy is not wired to what
# it claims to check, and this script exits 1.
#
# The snippet carries its own `#![deny(...)]` header. Integration test files
# are separate crate roots (each `tests/*.rs` is its own crate), so they do
# NOT inherit `src/lib.rs`'s crate-level `#![deny(...)]` — verified empirically
# during SPEC-001's build. Without its own header the snippet would compile
# clean and the red-proof would be a false green.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/_lib.sh
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

DISABLED_SNIPPET="tests/lint_policy_red.rs.disabled"
LIVE_SNIPPET="tests/lint_policy_red.rs"

[ -f "$DISABLED_SNIPPET" ] || die "missing $DISABLED_SNIPPET — the red-proof has nothing to compile"
[ -e "$LIVE_SNIPPET" ] && die "$LIVE_SNIPPET already exists — refusing to overwrite; remove it first"

cleanup() {
    rm -f "$LIVE_SNIPPET"
}
trap cleanup EXIT

cp "$DISABLED_SNIPPET" "$LIVE_SNIPPET"

info "compiling the deliberately-violating snippet — this MUST fail:"
set +e
cargo clippy --all-targets --all-features -- -D warnings
CLIPPY_EXIT=$?
set -e

if [ "$CLIPPY_EXIT" -eq 0 ]; then
    die "the lint policy did NOT reject $LIVE_SNIPPET (clippy exited 0). The panic-free lint set is not wired to what it claims to check — this is exactly the manufactured-confidence failure oracle-must-be-shown-red exists to catch."
fi

success "lint policy red-proof: the violating snippet failed to compile as expected (clippy exit $CLIPPY_EXIT)."
