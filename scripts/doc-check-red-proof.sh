#!/usr/bin/env bash
# scripts/doc-check-red-proof.sh — proves `just doc-check` actually bites
# (SPEC-056; `oracle-must-be-shown-red` covers gates). Mechanism per DEC-009:
#
#   1. copy the crate (Cargo.toml, Cargo.lock, README.md, src/) to a temp dir;
#   2. CONTROL — run scripts/doc-check.sh, the exact gate CI runs, over the
#      UNMUTATED copy. It must exit 0, or a red below proves nothing;
#   3. inject ONE public item whose doc comment links a private item (the
#      `ljpeg` module, which is `mod ljpeg;` in src/lib.rs) into the COPIED
#      src/lib.rs;
#   4. MUTATION — run the same gate again. It must exit non-zero, and the
#      failure must be attributable to the injection: rustdoc's
#      private-intra-doc-links error naming the INJECTED item, exactly once,
#      and the lint named in the diagnostic.
#
# The working tree is never mutated, so there is no restore path to get wrong.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { printf '✗ doc-check red-proof: %s\n' "$1" >&2; exit 1; }

GATE="${SCRIPT_DIR}/doc-check.sh"
[ -x "${GATE}" ] || fail "missing or non-executable ${GATE} — there is no gate to prove"
[ -f "${ROOT}/src/lib.rs" ] || fail "missing ${ROOT}/src/lib.rs — nothing to inject into"

COPY="${TMP}/crate"
mkdir -p "${COPY}"
cp "${ROOT}/Cargo.toml" "${COPY}/"
[ -f "${ROOT}/Cargo.lock" ] && cp "${ROOT}/Cargo.lock" "${COPY}/"
[ -f "${ROOT}/README.md" ] && cp "${ROOT}/README.md" "${COPY}/"
cp -R "${ROOT}/src" "${COPY}/src"
# Keep the copy's build out of the caller's target dir (and the caller's out of
# the copy): an inherited CARGO_TARGET_DIR would share artefacts between them.
export CARGO_TARGET_DIR="${TMP}/target"

# The private item the injection links. It must exist, and be private, for the
# injection to mean anything: `mod ljpeg;` (no `pub`) at the start of a line.
private_mods="$(grep -c '^mod ljpeg;$' "${COPY}/src/lib.rs" || true)"
private_mods="${private_mods:-0}"
[ "${private_mods}" -eq 1 ] \
    || fail "expected exactly one private 'mod ljpeg;' in src/lib.rs, found ${private_mods} — the injection target moved; update this red-proof"

run_gate() {
    set +e
    out="$("${GATE}" "${COPY}" 2>&1)"
    rc=$?
    set -e
}

# ── CONTROL (DEC-009) ────────────────────────────────────────────────────────
run_gate
[ "${rc}" -eq 0 ] \
    || fail "control failed — doc-check is already red on the unmutated copy; an injected red below would prove nothing (out: ${out})"
control_errors="$(grep -c 'links to private item' <<<"${out}" || true)"
control_errors="${control_errors:-0}"
[ "${control_errors}" -eq 0 ] \
    || fail "control passed but printed ${control_errors} private-link diagnostic(s) — the gate is not denying warnings (out: ${out})"

# ── MUTATION ─────────────────────────────────────────────────────────────────
INJECTED="__doc_check_red_proof_injection"
cat >> "${COPY}/src/lib.rs" <<EOF

/// RED-PROOF INJECTION (scripts/doc-check-red-proof.sh): a public doc comment
/// linking the private [\`ljpeg\`] module.
pub fn ${INJECTED}() {}
EOF
injected="$(grep -c "^pub fn ${INJECTED}() {}\$" "${COPY}/src/lib.rs" || true)"
injected="${injected:-0}"
[ "${injected}" -eq 1 ] || fail "the injection did not land in the copied src/lib.rs — this red-proof has caught NOTHING"

run_gate
[ "${rc}" -ne 0 ] \
    || fail "the gate did NOT go red on a public doc linking a private item — warnings are not denied (out: ${out})"
attributed="$(grep -c "public documentation for \`${INJECTED}\` links to private item \`ljpeg\`" <<<"${out}" || true)"
attributed="${attributed:-0}"
[ "${attributed}" -eq 1 ] \
    || fail "the gate went red, but ${attributed} (not 1) diagnostic(s) name the injected item — red for the wrong reason (out: ${out})"
all_private="$(grep -c 'links to private item' <<<"${out}" || true)"
all_private="${all_private:-0}"
[ "${all_private}" -eq 1 ] \
    || fail "expected exactly 1 private-link diagnostic (the injection), saw ${all_private} (out: ${out})"
lint_named="$(grep -c 'rustdoc::private.intra.doc.links' <<<"${out}" || true)"
lint_named="${lint_named:-0}"
[ "${lint_named}" -ge 1 ] \
    || fail "the gate went red but the diagnostic does not name rustdoc::private_intra_doc_links (out: ${out})"

printf '✓ doc-check red-proof: control green on the unmutated copy → one injected public doc linking the private `ljpeg` module is rejected by rustdoc::private_intra_doc_links, attributed to the injected item (1 of 1 diagnostics).\n'
