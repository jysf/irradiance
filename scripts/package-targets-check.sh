#!/usr/bin/env bash
# SPEC-055's package-surface check (review finding #6, DEC-048).
#
# The named test:  default_build_and_package_expose_no_irr_binary
#
# `irr` is an internal dev/oracle tool (constraint `library-not-application`),
# but until SPEC-055 `Cargo.toml` declared it unconditionally, so
# `cargo install irradiance` built and installed it. It is now gated behind the
# empty `irr` feature (`required-features = ["irr"]`). This check is what keeps
# it gated: it BUILDS (behaviour, not a manifest read — AGENTS.md §12) and reads
# cargo's own artifact stream for executables.
#
#   1. DEFAULT BUILD — `cargo build` on the real tree emits NO executable.
#   2. PACKAGE — `cargo package` succeeds (the published manifest is
#      self-consistent), and the packaged crate, unpacked and built with default
#      features — exactly what `cargo install irradiance` does — emits NO
#      executable either.
#   3. POSITIVE CONTROL — `cargo build --features irr` DOES emit `irr`. Without
#      this, a detector that could never see a binary (a renamed JSON field, a
#      broken parse) would make steps 1-2 green while checking nothing.
#   4. RED — in a temp copy with `required-features` stripped from the bin (the
#      pre-SPEC-055 manifest), the default build emits `irr` and this check's
#      detector fires. That is the regression this check exists to catch, shown
#      caught (`oracle-must-be-shown-red`).
#
# Steps 2-4 work in temp dirs; the working tree is only BUILT, never written to
# (beyond cargo's own target/).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { printf '✗ package-targets check: %s\n' "$1" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || fail "python3 is not on PATH — needed to read cargo's JSON artifact stream. Not running is not passing."

# Every executable cargo produced, one name per line, from `--message-format=json`.
# Reads the JSON with a real parser; never greps it.
executables_of() { # cargo build args...
    local json rc
    set +e
    json="$(cargo build --quiet --message-format=json "$@" 2>"$TMP/build.err")"; rc=$?
    set -e
    [ "$rc" -eq 0 ] || fail "cargo build $* failed (rc ${rc}): $(cat "$TMP/build.err")"
    printf '%s\n' "$json" | python3 -c '
import json, os, sys
artifacts = 0
for line in sys.stdin:
    line = line.strip()
    if not line.startswith("{"):
        continue
    msg = json.loads(line)
    if msg.get("reason") != "compiler-artifact":
        continue
    artifacts += 1
    exe = msg.get("executable")
    if exe:
        print(os.path.basename(exe))
# A build that reported no artifacts at all checked nothing.
if artifacts == 0:
    sys.exit("no compiler-artifact messages in cargo output")
'
}

cd "$ROOT"
NAME_VERSION="$(cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys
pkgs = json.load(sys.stdin)["packages"]
p = [p for p in pkgs if p["name"] == "irradiance"][0]
print(p["name"] + "-" + p["version"])
')"

# ── 1. DEFAULT BUILD on the real tree: no executable. ───────────────────────
exes="$(executables_of)"
[ -z "$exes" ] \
    || fail "the default-feature build produced executable(s): ${exes//$'\n'/ } — the library's default surface must ship no binary (library-not-application; DEC-048). Gate it with required-features."

# ── 2. PACKAGE: self-consistent, and the packaged crate builds no executable.
# --allow-dirty so a local run works mid-edit; CI's checkout is clean anyway.
# --no-verify because step 2's own build below IS the verification, run on the
# unpacked .crate with default features (what `cargo install` builds).
set +e
pkg_out="$(cargo package --quiet --allow-dirty --no-verify 2>&1)"; rc=$?
set -e
[ "$rc" -eq 0 ] || fail "cargo package failed — the published manifest is not self-consistent (rc ${rc}): ${pkg_out}"
CRATE="$ROOT/target/package/${NAME_VERSION}.crate"
[ -f "$CRATE" ] || fail "cargo package succeeded but ${CRATE} is missing"
tar -xzf "$CRATE" -C "$TMP"
[ -f "$TMP/${NAME_VERSION}/Cargo.toml" ] || fail "unpacked crate has no Cargo.toml"
exes="$(cd "$TMP/${NAME_VERSION}" && CARGO_TARGET_DIR="$TMP/pkg-target" executables_of)"
[ -z "$exes" ] \
    || fail "the PACKAGED crate's default build produced executable(s): ${exes//$'\n'/ } — 'cargo install irradiance' would install them"

# ── 3. POSITIVE CONTROL: --features irr does build irr. ─────────────────────
exes="$(executables_of --features irr)"
printf '%s\n' "$exes" | grep -qx 'irr' \
    || fail "positive control failed: 'cargo build --features irr' did not produce an 'irr' executable (got: '${exes}') — either the feature gate is broken or this check cannot see binaries at all"

# ── 4. RED: the pre-SPEC-055 manifest (no required-features) is caught. ─────
mkdir -p "$TMP/red"
cp "$ROOT/Cargo.toml" "$TMP/red/"
cp -R "$ROOT/src" "$TMP/red/src"
[ -f "$ROOT/Cargo.lock" ] && cp "$ROOT/Cargo.lock" "$TMP/red/"
grep -q '^required-features = \["irr"\]$' "$TMP/red/Cargo.toml" \
    || fail "Cargo.toml has no 'required-features = [\"irr\"]' line — the irr gate (DEC-048) is gone"
grep -v '^required-features = \["irr"\]$' "$ROOT/Cargo.toml" > "$TMP/red/Cargo.toml"
grep -q '^required-features' "$TMP/red/Cargo.toml" && fail "red mutation did not land"
exes="$(cd "$TMP/red" && CARGO_TARGET_DIR="$TMP/red-target" executables_of)"
printf '%s\n' "$exes" | grep -qx 'irr' \
    || fail "red half failed: with required-features stripped, the default build did not surface 'irr' (got: '${exes}') — this check would not catch the regression it exists for"

printf '✓ package-targets check (default_build_and_package_expose_no_irr_binary): the default build and the packaged crate (%s, as cargo install builds it) produce no executable; --features irr does produce irr (positive control); and the pre-SPEC-055 manifest with required-features stripped is caught surfacing irr (red).\n' "$NAME_VERSION"
