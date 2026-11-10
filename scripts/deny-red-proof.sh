#!/usr/bin/env bash
# SPEC-055's red-proof for the licence gate (`just deny` + `just deny-fuzz`).
#
# The named test:  deny_rejects_a_gpl_dev_dependency_in_both_graphs
#
# Constraint `no-copyleft-dependencies` covers dev-dependencies. Until SPEC-055
# the gate did not: cargo-deny 0.19's licence check gathered the whole root
# graph (48 crates) and then evaluated the licence of exactly one of them,
# `irradiance` itself, so a GPL dev-dependency passed both `just deny` and CI's
# `licenses` job. The fix is `include-dev = true` under `[licenses]` in
# deny.toml (DEC-048). This script is what proves the fix, and keeps proving it
# (`oracle-must-be-shown-red`, DEC-009 mechanism: control first, then mutate a
# temp copy):
#
#   * CONTROL, both graphs — the honest copied tree PASSES, and the root run
#     actually evaluated the dev-deps (`toml`, `ssimulacra2` are named in the
#     check's own accepted list). A green that evaluated one crate would pass
#     the exit-code half alone; the named-crate half is what catches that.
#   * ROOT mutation — a synthetic GPL-3.0-only crate added as a DEV-dependency
#     of the copied root Cargo.toml → the gate goes RED, naming the crate and
#     the rejected licence.
#   * FUZZ mutation — the same crate as a DEV-dependency of the copied
#     fuzz/Cargo.toml, gated through `--manifest-path fuzz/Cargo.toml` exactly
#     as `just deny-fuzz` runs it → RED, named. The fuzz graph is a second cargo
#     graph (DEC-011); it needs its own proof.
#   * THE KNOB IS THE TEETH — the SAME root mutation with `include-dev` removed
#     from the copied deny.toml PASSES. That pins the red above to this one
#     config line: delete it from deny.toml and this script fails. (If a future
#     cargo-deny starts evaluating dev-deps by default, this step fails too, and
#     the right response is to re-measure and update this step, not delete it.)
#   * ISOLATION — pristine manifests → green again, so every red above is
#     attributable to the injection by construction.
#
# The GPL crate is a local path crate written into the temp dir: no network, and
# nothing is ever added to the real tree (the spec's no-new-dependency rule).
# Everything happens in a temp copy; the working tree is never written to.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { printf '✗ deny red-proof: %s\n' "$1" >&2; exit 1; }

command -v cargo-deny >/dev/null 2>&1 \
    || fail "cargo-deny is not on PATH (install: cargo install cargo-deny --locked). Not running is not passing."
printf 'deny-red-proof: using %s\n' "$(cargo deny --version)"

# ── The copy: exactly what `cargo metadata` needs for both graphs. ───────────
# Every target path a manifest names must exist (src/lib.rs, src/bin/irr.rs,
# fuzz/fuzz_targets/*.rs). Lockfiles are gitignored in this repo, so copy them
# when present (keeps a local run offline) and let cargo resolve when not (CI).
mkdir -p "$TMP/repo/fuzz"
cp "$ROOT/Cargo.toml" "$ROOT/deny.toml" "$TMP/repo/"
cp -R "$ROOT/src" "$TMP/repo/src"
cp "$ROOT/fuzz/Cargo.toml" "$TMP/repo/fuzz/"
cp -R "$ROOT/fuzz/fuzz_targets" "$TMP/repo/fuzz/fuzz_targets"
[ -f "$ROOT/Cargo.lock" ] && cp "$ROOT/Cargo.lock" "$TMP/repo/"
[ -f "$ROOT/fuzz/Cargo.lock" ] && cp "$ROOT/fuzz/Cargo.lock" "$TMP/repo/fuzz/"

# The synthetic copyleft crate. GPL-3.0-only is not in deny.toml's `allow` and
# has no exception, so the policy must reject it wherever it appears.
GPL_CRATE="gpl-red-proof-probe"
mkdir -p "$TMP/$GPL_CRATE/src"
cat > "$TMP/$GPL_CRATE/Cargo.toml" <<EOF
[package]
name = "$GPL_CRATE"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-only"
publish = false
EOF
: > "$TMP/$GPL_CRATE/src/lib.rs"

cd "$TMP/repo"
cp Cargo.toml "$TMP/root-Cargo.toml.pristine"
cp fuzz/Cargo.toml "$TMP/fuzz-Cargo.toml.pristine"
cp deny.toml "$TMP/deny.toml.pristine"

reset_tree() {
    cp "$TMP/root-Cargo.toml.pristine" Cargo.toml
    cp "$TMP/fuzz-Cargo.toml.pristine" fuzz/Cargo.toml
    cp "$TMP/deny.toml.pristine" deny.toml
}

# The two invocations, verbatim from `just deny` / `just deny-fuzz` (plus `-L
# debug`, which only adds the accepted-crate list to the output, and `--color
# never`). Sets $rc and $out; rc captured per §16 rule 3, never read through an
# unguarded pipeline.
#
# `--color never` is load-bearing: cargo-deny reads CARGO_TERM_COLOR, and CI's
# workflow env sets it to `always`, which wraps the `├` tree glyph in ANSI codes
# so every grep below misses (`\e[36m├\e[0m toml`). That made the control fail
# in CI while passing locally (SPEC-055 verify, HANDOFF-116). Colour changes
# only the rendering, not what is evaluated, so the gate under test is the same.
#
# The greps on $out read it from a here-string, never `printf "$out" | grep -q`:
# under `set -o pipefail`, grep -q exits on its first match, printf takes
# SIGPIPE on output larger than the pipe buffer (the GPL run is ~100 KB), and
# the pipeline reports failure even though the pattern matched. That flaked
# red in CI on a correct rejection (SPEC-055 verify).
run_deny() {
    set +e
    case "$1" in
        root) out="$(cargo deny --color never -L debug check licenses 2>&1)"; rc=$? ;;
        fuzz) out="$(cargo deny --color never -L debug --manifest-path fuzz/Cargo.toml check licenses 2>&1)"; rc=$? ;;
        *) fail "run_deny: unknown graph '$1'" ;;
    esac
    set -e
}

# Append a dev-dependency on the GPL crate to a manifest. The fuzz manifest has
# no [dev-dependencies] table, so append a new one; the root already has one
# and it is the LAST table in the file, so appending lands inside it either way.
inject_gpl_dev_dep() { # manifest, relative path to the GPL crate
    local manifest="$1" rel="$2"
    if grep -q '^\[dev-dependencies\]' "$manifest"; then
        [ "$(grep -E '^\[' "$manifest" | tail -n 1)" = "[dev-dependencies]" ] \
            || fail "[dev-dependencies] is not the last table in ${manifest} — an appended line would land in the wrong table; update this injection"
        printf '\n%s = { path = "%s" }\n' "$GPL_CRATE" "$rel" >> "$manifest"
    else
        printf '\n[dev-dependencies]\n%s = { path = "%s" }\n' "$GPL_CRATE" "$rel" >> "$manifest"
    fi
    grep -q "^${GPL_CRATE} = { path" "$manifest" \
        || fail "GPL dev-dep injection did not land in ${manifest} — this red-proof has caught NOTHING"
}

# A red for the RIGHT reason: the rejected crate AND its licence are named. A
# resolution error (bad path, network) also exits non-zero and must not count.
assert_red_names_gpl() { # graph label
    [ "$rc" -ne 0 ] \
        || fail "$1: the gate did NOT go red on a GPL-3.0-only dev-dependency — dev-deps are not evaluated (out: ${out})"
    grep -q 'error\[rejected\]' <<<"$out" \
        || fail "$1: the gate went red but not with error[rejected] — red for the wrong reason (out: ${out})"
    grep -q "${GPL_CRATE} v0.1.0" <<<"$out" \
        || fail "$1: the gate went red but did not name ${GPL_CRATE} — red for the wrong reason (out: ${out})"
    grep -q 'GPL-3.0-only' <<<"$out" \
        || fail "$1: the gate went red but did not name the GPL-3.0-only licence (out: ${out})"
}

# ── 1. CONTROL: both graphs green on the honest tree, dev-deps evaluated. ────
run_deny root
[ "$rc" -eq 0 ] \
    || fail "control failed — 'just deny' is already red on the honest tree; an injected red below would prove nothing (out: ${out})"
for dep in toml ssimulacra2; do
    grep -qE "├ ${dep} v[0-9]" <<<"$out" \
        || fail "control: the root licence check did not evaluate dev-dependency '${dep}' — the gate is green over a graph without its dev-deps (the SPEC-055 defect)"
done
run_deny fuzz
[ "$rc" -eq 0 ] \
    || fail "control failed — 'just deny-fuzz' is already red on the honest tree (out: ${out})"
grep -qE '├ libfuzzer-sys v[0-9]' <<<"$out" \
    || fail "control: the fuzz licence check did not evaluate libfuzzer-sys — it is not looking at the fuzz graph"

# ── 2. ROOT: a GPL dev-dependency of the library → red, named. ──────────────
reset_tree
inject_gpl_dev_dep Cargo.toml "../${GPL_CRATE}"
run_deny root
assert_red_names_gpl "root graph (just deny)"

# ── 3. FUZZ: a GPL dev-dependency of irradiance-fuzz → red, named. ───────────
reset_tree
inject_gpl_dev_dep fuzz/Cargo.toml "../../${GPL_CRATE}"
run_deny fuzz
assert_red_names_gpl "fuzz graph (just deny-fuzz)"

# ── 4. THE KNOB IS THE TEETH: same root mutation, include-dev removed → green.
reset_tree
grep -q '^include-dev = true$' deny.toml \
    || fail "deny.toml has no 'include-dev = true' line — the dev-graph knob (DEC-048) is gone"
grep -v '^include-dev = true$' "$TMP/deny.toml.pristine" > deny.toml
grep -q '^include-dev' deny.toml && fail "knob removal did not land in the copied deny.toml"
inject_gpl_dev_dep Cargo.toml "../${GPL_CRATE}"
run_deny root
[ "$rc" -eq 0 ] \
    || fail "with include-dev removed the GPL dev-dep was STILL rejected — the red in step 2 is not attributable to include-dev; re-measure cargo-deny's default before trusting this proof (out: ${out})"

# ── 5. ISOLATION: pristine manifests → green again. ─────────────────────────
reset_tree
run_deny root
[ "$rc" -eq 0 ] || fail "root still red after restoring pristine manifests — the reds above were not attributable to the injections (out: ${out})"
run_deny fuzz
[ "$rc" -eq 0 ] || fail "fuzz still red after restoring pristine manifests (out: ${out})"

printf '✓ deny red-proof (deny_rejects_a_gpl_dev_dependency_in_both_graphs): control green on both graphs with toml/ssimulacra2 evaluated → a GPL-3.0-only dev-dependency is rejected BY NAME in the root graph and in the fuzz graph, the same mutation passes with include-dev removed (the knob is the teeth), and both graphs are green once the injection is removed.\n'
