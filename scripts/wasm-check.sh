#!/usr/bin/env bash
# scripts/wasm-check.sh — the browser-target build gate (SPEC-096).
#
# Builds the LIBRARY for wasm32-unknown-unknown on both feature surfaces that
# matter, in this order:
#
#   cargo build --release --target wasm32-unknown-unknown --lib
#   cargo build --release --target wasm32-unknown-unknown --lib --features write
#
# Why: the "Anatomy of a RAW" demo (jysf/irradiance-demo) runs the published
# crate compiled to WebAssembly, and DEC-002 keeps that target open. Nothing
# else in CI compiles for wasm32, so a unix-only std API or a constant that
# assumes a 64-bit usize would break the browser build without a red anywhere.
#
# LIMIT, measured at design (rustc 1.98.0): this is a BUILD gate. On
# wasm32-unknown-unknown std ships `Instant::now`, `std::fs`, `thread::spawn`
# and friends as stubs that compile and then panic or return Unsupported at
# run time, so this gate cannot see them. A run-time lane is a follow-up.
#
# Usage: wasm-check.sh [crate-dir]   (default: this repo's root)
#
# The optional crate dir is what lets scripts/wasm-check-red-proof.sh run THIS
# exact invocation over a mutated copy, so the red-proof tests the gate CI runs
# and not a paraphrase of it.
#
# PATCH-004: a gate states which tool produced its verdict. This prints the
# cargo and rustc that answered. Locally, Homebrew's cargo/rustc may come first
# on PATH and have no wasm32 std; `just wasm-check` puts ~/.cargo/bin first.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="${1:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
TARGET="wasm32-unknown-unknown"

fail() { printf '✗ wasm-check: %s\n' "$1" >&2; exit 1; }

[ -f "${CRATE_DIR}/Cargo.toml" ] || fail "no Cargo.toml in ${CRATE_DIR} — nothing to build"

if ! CARGO_VERSION="$(cargo --version 2>&1)"; then
    fail "\`cargo --version\` failed — cargo is not available, so this gate can prove NOTHING. Output: ${CARGO_VERSION}"
fi
if ! RUSTC_VERSION="$(rustc --version 2>&1)"; then
    fail "\`rustc --version\` failed — rustc is not available, so this gate can prove NOTHING. Output: ${RUSTC_VERSION}"
fi
CARGO_BIN="$(command -v cargo 2>/dev/null || echo 'NOT ON PATH')"
RUSTC_BIN="$(command -v rustc 2>/dev/null || echo 'NOT ON PATH')"
printf 'wasm-check: using %s (%s), %s (%s)\n' "${CARGO_VERSION}" "${CARGO_BIN}" "${RUSTC_VERSION}" "${RUSTC_BIN}"

# Pre-flight: the toolchain must carry a wasm32 std, or every build below fails
# with E0463 "can't find crate for `std`" — a red that says nothing about the
# library. Name the real cause instead.
if ! SYSROOT="$(rustc --print sysroot 2>&1)"; then
    fail "\`rustc --print sysroot\` failed: ${SYSROOT}"
fi
[ -d "${SYSROOT}/lib/rustlib/${TARGET}/lib" ] \
    || fail "the toolchain at ${SYSROOT} has no ${TARGET} std (no ${SYSROOT}/lib/rustlib/${TARGET}/lib). Install it with \`rustup target add ${TARGET}\`, and put rustup's toolchain first on PATH (Homebrew's rust has no wasm32 std)."

cd "${CRATE_DIR}"
TARGET_DIR="${CARGO_TARGET_DIR:-${CRATE_DIR}/target}"
RLIB="${TARGET_DIR}/${TARGET}/release/libirradiance.rlib"

surfaces=0
for surface in default write; do
    if [ "${surface}" = "default" ]; then
        printf 'wasm-check: cargo build --release --target %s --lib\n' "${TARGET}"
        cargo build --release --target "${TARGET}" --lib
    else
        printf 'wasm-check: cargo build --release --target %s --lib --features write\n' "${TARGET}"
        cargo build --release --target "${TARGET}" --lib --features write
    fi
    [ -f "${RLIB}" ] || fail "cargo exited 0 for the ${surface} surface but ${RLIB} does not exist — nothing was built for ${TARGET}"
    printf 'wasm-check: surface %s built for %s\n' "${surface}" "${TARGET}"
    surfaces=$((surfaces + 1))
done

# Both surfaces must actually have been built; a loop that ran zero times is
# not a pass (AGENTS.md §16 rule 2).
[ "${surfaces}" -eq 2 ] || fail "expected to build 2 feature surfaces, built ${surfaces}"

printf '✓ wasm-check: %s feature surfaces (default, write) built for %s.\n' "${surfaces}" "${TARGET}"
