#!/usr/bin/env bash
# scripts/lint-red-proof.sh — proves the LIBRARY's panic-free lint policy
# actually bites (constraint `oracle-must-be-shown-red` applied to a gate rather
# than an oracle; SPEC-001 acceptance criterion 5). Mechanism per DEC-007.
#
# A lint policy that has never rejected anything is not a policy. This script is
# a MUTATION TEST of the real crate:
#
#   1. copy the crate (Cargo.toml, Cargo.lock, src/) to a temp dir
#   2. inject two violating functions into the COPIED src/lib.rs, immediately
#      after its attribute prologue — so `src/lib.rs`'s own `#![deny(...)]` is
#      the only thing that can reject them
#   3. run the same `cargo clippy --all-targets --all-features -- -D warnings`
#      invocation CI runs everywhere else, in the copy
#   4. assert ALL THREE of:
#        (a) clippy actually ran at all
#        (b) clippy exited non-zero
#        (c) all three expected lint NAMES appear in its output
#
# The working tree is never mutated, so there is no restore path to get wrong.
#
# ⚠ Why (a): with a `cargo` on PATH that cannot run clippy, an exit-code-only
# proof reports GREEN having proven nothing (verify punch list PL-2).
# ⚠ Why (c): the injection point is found by parsing the attribute prologue. A
# naive `max()` over lines ending `)]` was tried first and landed INSIDE the
# test module's `#[allow(...)]`, silently suppressing two of the three lints.
# Checking the names is what caught it, and it is what makes a mis-landed
# injection fail loudly instead of passing (DEC-007, Consequences).
#
# DEC-007 supersedes DEC-006: the old mechanism compiled a snippet carrying its
# OWN `#![deny(...)]` header, which proved a fact about Rust, not about this
# library — the policy could be deleted from src/lib.rs with every gate staying
# green.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/_lib.sh
source "${SCRIPT_DIR}/_lib.sh"

# _lib.sh derives REPO_ROOT from the CWD. This script must work from anywhere
# (it is run by CI, by `just lint-red-proof`, and by hand), so resolve the root
# from the script's own location instead.
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

require_initialized

LIB_RS="${REPO_ROOT}/src/lib.rs"
[ -f "${REPO_ROOT}/Cargo.toml" ] || die "missing ${REPO_ROOT}/Cargo.toml — nothing to copy"
[ -f "$LIB_RS" ] || die "missing $LIB_RS — the red-proof has nothing to inject into"

# ── Assertion 1: clippy actually ran ────────────────────────────────────────
# Without this, a `cargo` that cannot run clippy exits non-zero and the proof
# below "passes" having compiled nothing (PL-2, measured with a stub cargo).
if ! CLIPPY_VERSION="$(cargo clippy --version 2>&1)"; then
    die "\`cargo clippy --version\` failed — clippy is not available, so this proof can prove NOTHING. Refusing to report green. Output: ${CLIPPY_VERSION}"
fi
info "clippy is present: ${CLIPPY_VERSION}"

# ── Find the injection point in src/lib.rs ──────────────────────────────────
# Skip blank lines, `//!` inner doc comments and `#![...]` inner attributes
# (tracking bracket depth, since `#![deny(` spans several lines); inject before
# the first real item. Landing inside an `#[allow(...)]` scope is the failure
# mode here, and assertion 3 below is what catches it.
find_injection_line() {
    local file="$1" line trimmed lineno=0 depth=0 opens closes
    while IFS= read -r line || [ -n "$line" ]; do
        lineno=$((lineno + 1))
        if [ "$depth" -gt 0 ]; then
            opens="${line//[^\[]/}"
            closes="${line//[^\]]/}"
            depth=$((depth + ${#opens} - ${#closes}))
            continue
        fi
        trimmed="${line#"${line%%[![:space:]]*}"}"   # strip leading whitespace
        case "$trimmed" in
            '') continue ;;
            '//!'*) continue ;;
            '#!['*)
                opens="${line//[^\[]/}"
                closes="${line//[^\]]/}"
                depth=$((${#opens} - ${#closes}))
                continue
                ;;
            *)
                printf '%s\n' "$lineno"
                return 0
                ;;
        esac
    done < "$file"
    return 1
}

INJECT_AT="$(find_injection_line "$LIB_RS")" || die "could not find an injection point in src/lib.rs — its attribute prologue has a shape this script does not understand (DEC-007 names this as the mechanism's known fragility). Fix the parser; do NOT weaken the proof."
info "injection point: src/lib.rs line ${INJECT_AT} (immediately after the attribute prologue)"

# ── Copy the crate and inject ───────────────────────────────────────────────
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/irradiance-red-proof.XXXXXX")"
cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

mkdir -p "${WORK_DIR}/src"
cp "${REPO_ROOT}/Cargo.toml" "${WORK_DIR}/Cargo.toml"
if [ -f "${REPO_ROOT}/Cargo.lock" ]; then
    cp "${REPO_ROOT}/Cargo.lock" "${WORK_DIR}/Cargo.lock"
fi
cp -R "${REPO_ROOT}/src/." "${WORK_DIR}/src/"

VIOLATION_FILE="${WORK_DIR}/.violation.rs"
cat > "$VIOLATION_FILE" <<'RUST_EOF'

// ── RED-PROOF INJECTION — scripts/lint-red-proof.sh, DEC-007 ───────────────
// Injected into a COPY of src/lib.rs in a temp dir. Never part of the crate.
// These two functions are exactly the byte-reading mistakes the policy exists
// to stop: an unchecked index, unchecked arithmetic, and an `unwrap()` — all
// on caller-supplied (i.e. attacker-influenced) input.
pub fn red_proof_indexing_and_arithmetic(v: &[u8], n: u8) -> u8 {
    v[0] + n
}

pub fn red_proof_unwrap(v: &[u8]) -> u8 {
    *v.first().unwrap()
}
// ── END RED-PROOF INJECTION ───────────────────────────────────────────────

RUST_EOF

INJECTED="${WORK_DIR}/src/lib.rs"
{
    head -n "$((INJECT_AT - 1))" "$LIB_RS"
    cat "$VIOLATION_FILE"
    tail -n "+${INJECT_AT}" "$LIB_RS"
} > "$INJECTED"
rm -f "$VIOLATION_FILE"

# ── Run clippy on the mutated copy ──────────────────────────────────────────
info "running clippy on a mutated copy of the crate — this MUST fail:"
CLIPPY_LOG="${WORK_DIR}/clippy.log"
set +e
(cd "$WORK_DIR" && cargo clippy --all-targets --all-features -- -D warnings) >"$CLIPPY_LOG" 2>&1
CLIPPY_EXIT=$?
set -e
cat "$CLIPPY_LOG"

# ── Assertion 2: clippy rejected the mutation ───────────────────────────────
if [ "$CLIPPY_EXIT" -eq 0 ]; then
    die "the lint policy did NOT reject the injected violations (clippy exited 0). src/lib.rs's \`#![deny(...)]\` block is missing, weakened, or not applying — the panic-free lint set is not wired to what it claims to check. This is exactly the manufactured-confidence failure oracle-must-be-shown-red exists to catch."
fi

# ── Assertion 3: the RIGHT lints fired ──────────────────────────────────────
# A non-zero exit alone proves nothing: an unrelated compile error, or an
# injection that landed inside an `#[allow(...)]` scope, would also be non-zero.
# Space-separated, not an array: /usr/bin/env bash is 3.2 on macOS, where
# expanding an empty array under `set -u` is itself an error.
EXPECTED_LINTS="clippy::indexing_slicing clippy::arithmetic_side_effects clippy::unwrap_used"
MISSING=""
for lint in $EXPECTED_LINTS; do
    grep -q -- "$lint" "$CLIPPY_LOG" || MISSING="${MISSING}${lint} "
done

if [ -n "$MISSING" ]; then
    die "clippy exited ${CLIPPY_EXIT}, but for the WRONG reasons: expected lint(s) ${MISSING}never fired. Either the policy no longer denies them, or the injection landed somewhere they do not apply (e.g. inside an \`#[allow(...)]\` scope — see DEC-007). A non-zero exit is not the proof; these lint names are."
fi

# Evidence that it was the LIBRARY's policy that fired, not a header of the
# proof's own making — this is the whole point of DEC-007. Informational.
grep -A1 'the lint level is defined here' "$CLIPPY_LOG" | grep -- '-->' | sort -u | sed 's/^/    /' || true

success "lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected violations (clippy exit ${CLIPPY_EXIT}; ${EXPECTED_LINTS} all fired)."
