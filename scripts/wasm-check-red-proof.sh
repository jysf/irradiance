#!/usr/bin/env bash
# scripts/wasm-check-red-proof.sh — proves `just wasm-check` actually bites
# (SPEC-096; `oracle-must-be-shown-red` covers gates). Mechanism per DEC-009:
#
#   1. copy the crate (Cargo.toml, Cargo.lock, README.md, src/) to a temp dir;
#   2. CONTROL — run scripts/wasm-check.sh, the exact gate CI runs, over the
#      UNMUTATED copy. It must exit 0 and report 2 surfaces, or a red below
#      proves nothing;
#   3. MUTATION A — append a function using a unix-only std API
#      (std::os::unix::ffi::OsStrExt) to the COPIED src/lib.rs. The NATIVE
#      build must still pass (so every other CI job would stay green), and the
#      gate must fail with E0433 naming `unix`;
#   4. restore src/lib.rs from the pristine copy;
#   5. MUTATION B — append a constant that assumes a 64-bit usize, behind
#      #[cfg(feature = "write")]. The native build WITH `write` must pass, and
#      the gate must fail with E0080 (overflow) AFTER the default surface built.
#      B is the fixture that proves the `write` surface is really built: a gate
#      that only built the default surface would stay green on it.
#
# Why not `Instant::now()` or `std::fs`: on wasm32-unknown-unknown both compile
# (std stubs them, failing at run time), so a build gate cannot go red on them.
# Measured at SPEC-096 design; see the gate's header for the limit.
#
# The working tree is never mutated, so there is no restore path to get wrong.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { printf '✗ wasm-check red-proof: %s\n' "$1" >&2; exit 1; }

GATE="${SCRIPT_DIR}/wasm-check.sh"
[ -x "${GATE}" ] || fail "missing or non-executable ${GATE} — there is no gate to prove"
[ -f "${ROOT}/src/lib.rs" ] || fail "missing ${ROOT}/src/lib.rs — nothing to inject into"

COPY="${TMP}/crate"
mkdir -p "${COPY}"
cp "${ROOT}/Cargo.toml" "${COPY}/"
[ -f "${ROOT}/Cargo.lock" ] && cp "${ROOT}/Cargo.lock" "${COPY}/"
[ -f "${ROOT}/README.md" ] && cp "${ROOT}/README.md" "${COPY}/"
# -L: a symlinked src must be copied as files, or mutating the copy writes
# through to the real tree (SPEC-096/FU-3).
cp -RL "${ROOT}/src" "${COPY}/src"
PRISTINE="${TMP}/lib.rs.pristine"
cp "${COPY}/src/lib.rs" "${PRISTINE}"
# Keep the copy's build out of the caller's target dir (and the caller's out of
# the copy): an inherited CARGO_TARGET_DIR would share artefacts between them.
export CARGO_TARGET_DIR="${TMP}/target"

run_gate() {
    set +e
    out="$("${GATE}" "${COPY}" 2>&1)"
    rc=$?
    set -e
}

# Native build of the mutated copy, with optional extra cargo args. Must pass:
# the fault has to be one that only the wasm32 build can see.
native_must_pass() {
    local label="$1"
    shift
    set +e
    native_out="$(cd "${COPY}" && cargo build --release --lib "$@" 2>&1)"
    native_rc=$?
    set -e
    [ "${native_rc}" -eq 0 ] \
        || fail "mutation ${label}: the NATIVE build failed too, so a red from the gate would not prove a wasm-only fault (out: ${native_out})"
}

count() { # count <pattern> <text> — grep -c that never aborts under set -e
    local n
    n="$(grep -cE "$1" <<<"$2" || true)"
    printf '%s' "${n:-0}"
}

# ── CONTROL (DEC-009) ────────────────────────────────────────────────────────
run_gate
[ "${rc}" -eq 0 ] \
    || fail "control failed — wasm-check is already red on the unmutated copy; an injected red below would prove nothing (out: ${out})"
control_ok="$(count '^✓ wasm-check: 2 feature surfaces' "${out}")"
[ "${control_ok}" -eq 1 ] \
    || fail "control exited 0 but did not report 2 built surfaces (out: ${out})"

# ── MUTATION A: a unix-only std API, unconditional ───────────────────────────
INJECTED_A="__wasm_check_red_proof_unix"
cat >> "${COPY}/src/lib.rs" <<EOF

/// RED-PROOF INJECTION (scripts/wasm-check-red-proof.sh): a unix-only std API.
pub fn ${INJECTED_A}(s: &std::ffi::OsStr) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    s.as_bytes()
}
EOF
landed="$(count "^pub fn ${INJECTED_A}\\(" "$(cat "${COPY}/src/lib.rs")")"
[ "${landed}" -eq 1 ] || fail "mutation A did not land in the copied src/lib.rs — this red-proof has caught NOTHING"

native_must_pass A
run_gate
[ "${rc}" -ne 0 ] \
    || fail "mutation A: the gate did NOT go red on a unix-only std API — it is not building for wasm32 (out: ${out})"
a_code="$(count 'error\[E0433\]' "${out}")"
a_unix="$(count 'error\[E0433\].*unix' "${out}")"
[ "${a_code}" -ge 1 ] && [ "${a_unix}" -ge 1 ] \
    || fail "mutation A: the gate went red, but not with E0433 naming \`unix\` (E0433: ${a_code}, naming unix: ${a_unix}) — red for the wrong reason (out: ${out})"

# ── restore, then MUTATION B: a 64-bit usize constant behind `write` ─────────
cp "${PRISTINE}" "${COPY}/src/lib.rs"
INJECTED_B="__WASM_CHECK_RED_PROOF_USIZE"
cat >> "${COPY}/src/lib.rs" <<EOF

/// RED-PROOF INJECTION (scripts/wasm-check-red-proof.sh): assumes a 64-bit usize.
#[cfg(feature = "write")]
pub const ${INJECTED_B}: usize = 1 << 32;
EOF
landed="$(count "^pub const ${INJECTED_B}: usize" "$(cat "${COPY}/src/lib.rs")")"
[ "${landed}" -eq 1 ] || fail "mutation B did not land in the copied src/lib.rs — this red-proof has caught NOTHING"

native_must_pass B --features write
run_gate
[ "${rc}" -ne 0 ] \
    || fail "mutation B: the gate did NOT go red on a 64-bit usize constant behind \`write\` — the write surface is not built for wasm32 (out: ${out})"
b_code="$(count 'error\[E0080\].*overflow' "${out}")"
[ "${b_code}" -ge 1 ] \
    || fail "mutation B: the gate went red, but not with E0080 naming the overflow — red for the wrong reason (out: ${out})"
b_default="$(count '^wasm-check: surface default built' "${out}")"
b_write="$(count '^wasm-check: surface write built' "${out}")"
[ "${b_default}" -eq 1 ] && [ "${b_write}" -eq 0 ] \
    || fail "mutation B: expected the default surface to build and the write surface to fail (default built: ${b_default}, write built: ${b_write}) (out: ${out})"

printf '✓ wasm-check red-proof: control green (2 surfaces) → mutation A (unix-only std API) native-green, gate red with E0433 naming unix → mutation B (64-bit usize constant behind write) native-green with write, default surface built, gate red with E0080 on the write surface (2 of 2 mutations attributed).\n'
