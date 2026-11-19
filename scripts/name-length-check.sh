#!/usr/bin/env bash
# scripts/name-length-check.sh — no tracked path component over 150 bytes
# (SPEC-074, DEC-067).
#
# Why: tools that copy the tree write a temporary file BESIDE each file, named
# the original name plus a suffix. `cargo semver-checks --baseline-rev` clones
# the repo that way, and its temporary name is the file name plus about 45
# bytes, so any name over about 210 bytes passed macOS's 255-byte limit and
# killed the clone ("File name too long", SPEC-073). The generators have capped
# slugs at 60 characters since template v0.6.49, and SPEC-074 renamed the older
# names, but nothing caps a HAND-named file (DECs, docs). This gate does.
#
# 150 bytes leaves about 60 bytes of room below the observed crash point and is
# above the longest DEC name (130 bytes). It counts BYTES, not characters, since
# the file-system limit is in bytes.
#
# What it checks: every `git ls-files` path, split on `/`; each part must be at
# most LIMIT bytes. Untracked files are ignored (they are never cloned). It does
# not check the 60-character slug rule; the generators enforce that.
#
# It asserts its own scan count: zero tracked paths is a loud failure, not a
# green pass (AGENTS.md §16 rule 2).
#
# Usage: name-length-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIMIT=150

fail() { printf '✗ name-length-check: %s\n' "$1" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 \
    || fail "python3 is not on PATH; it measures names in bytes. Not running is not passing."

cd "$ROOT"
# `-z` gives raw path bytes (no quoting of non-ASCII names). The list goes to a
# temp file so git's own exit status is checked by set -e, not lost in a pipe.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
git ls-files -z > "$TMP/listing"

# The checker's exit status IS the verdict; it runs directly, never inside a
# command substitution. On a pass it writes its scan counts to a file, which
# the shell then asserts below.
python3 - "$TMP/listing" "$LIMIT" "$TMP/counts" <<'PY'
import sys

listing, limit, counts = sys.argv[1], int(sys.argv[2]), sys.argv[3]
with open(listing, "rb") as f:
    paths = [p for p in f.read().split(b"\0") if p]

if not paths:
    sys.exit("✗ name-length-check: git ls-files listed 0 tracked paths; nothing was checked")

offenders = []
components = 0
for path in paths:
    for part in path.split(b"/"):
        components += 1
        if len(part) > limit:
            offenders.append((len(part), path.decode("utf-8", "replace")))

if offenders:
    for size, path in sorted(offenders, reverse=True):
        print(f"✗ {size} bytes (limit {limit}): {path}", file=sys.stderr)
    sys.exit(
        f"✗ name-length-check: {len(offenders)} tracked path component(s) over "
        f"{limit} bytes (DEC-067). Shorten the name; for a generated artifact, "
        "use the name slugify gives it."
    )

with open(counts, "w") as f:
    f.write(f"{len(paths)} {components}\n")
PY

# Assert the scan count in the shell too: a pass over zero paths is not a pass.
read -r PATHS COMPONENTS < "$TMP/counts"
[ "${PATHS:-0}" -gt 0 ] || fail "scanned 0 tracked paths; nothing was checked"
[ "${COMPONENTS:-0}" -ge "$PATHS" ] || fail "scanned ${COMPONENTS:-0} components for ${PATHS} paths"
printf '✓ name-length-check: %s tracked paths, %s components, none over %s bytes\n' \
    "$PATHS" "$COMPONENTS" "$LIMIT"
