#!/usr/bin/env bash
# PATCH-002's red-proof for the stage orchestration-cost gate.
#
# `constraints.yaml`: a job that exists and has never passed is a deleted job.
# The sibling failure is a gate that exists and has never FAILED — indistinguish-
# able from one that never ran. STAGE-001 shipped with `sessions: []` and nothing
# noticed for fifteen days, so this gate is adopted precisely because its absence
# was invisible. It ships proven red.
#
# Everything happens in a temp copy; the working tree is never written to.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { printf '✗ cost-audit red-proof: %s\n' "$1" >&2; exit 1; }

# `find`, not `ls` (SC2012) — and the same shape `find_all_stages` uses.
SUBJECT="$(cd "$ROOT" && find projects/*/stages -maxdepth 1 -name 'STAGE-002-*.md' -type f | sort | head -1)"
GRANDFATHERED="$(cd "$ROOT" && find projects/*/stages -maxdepth 1 -name 'STAGE-001-*.md' -type f | sort | head -1)"
[ -n "$SUBJECT" ] || fail "no STAGE-002 file found to mutate"

cp -R "$ROOT" "$TMP/repo"
cd "$TMP/repo"

# ⚠ REPRODUCE THE REAL SHIPPED SHAPE, template comment included. An injection
# that writes a bare `sessions: []` also deletes the commented example — and
# then even a naive `grep -q tokens_total` implementation passes this proof,
# because no text remains to false-match. MEASURED 2026-09-06: the naive version
# passed the first draft of this script. AGENTS.md §16's own warning, verbatim:
# "The obvious test exercises the wrong path."
empty_block() {
    python3 - "$1" <<'INNER'
import sys, re
p = sys.argv[1]
s = open(p).read()
m = re.search(r'^orchestration_cost:\n.*?(?=^---$)', s, re.S | re.M)
if not m:
    sys.exit("orchestration_cost block not found in " + p)
s = s[:m.start()] + (
    "orchestration_cost:\n"
    "  sessions: []                      # - tokens_total: N\n"
    "                                    #   estimated_usd: N\n"
    "                                    #   recorded_at: YYYY-MM-DD\n"
    '                                    #   notes: "framing + spec breakdown"\n'
) + s[m.end():]
open(p, 'w').write(s)
INNER
}

# ── 1. CONTROL: the honest tree must PASS, or a red below proves nothing.
./scripts/cost-audit.sh >/dev/null 2>&1 \
    || fail "control failed — cost-audit is already red on the honest tree, so the injection below would prove nothing"

# ── 2. INJECT into a shipped, non-grandfathered stage.
empty_block "$SUBJECT"

# ⚠ The clause that has caught four false red-proofs in three specs: assert the
# mutation CHANGED THE FILE before concluding anything about what was caught.
if diff -q "$SUBJECT" "$ROOT/$SUBJECT" >/dev/null 2>&1; then
    fail "the injection did not change the file — this red-proof has caught NOTHING"
fi
# And assert it reproduced the shape it claims to: the comment must survive, or
# this is the weaker proof the header warns about.
grep -q '# - tokens_total: N' "$SUBJECT" \
    || fail "the injection removed the template comment, so it exercises the wrong path — see this script's header"

# ── 3. The gate must now FAIL, through its OWN die, with a reason.
set +e
out="$(./scripts/cost-audit.sh 2>&1)"; rc=$?
set -e
[ "$rc" -ne 0 ] || fail "gate did NOT go red on a shipped stage with sessions: [] — the gate is decorative"
printf '%s\n' "$out" | grep -q 'missing cost on: orchestration' \
    || fail "gate went red but never named the stage or the field; a proof that dies without a message cannot be told from one that never ran"
printf '%s\n' "$out" | grep -q 'orchestration_cost.sessions' \
    || fail "the failure message does not say what to do about it"

# ── 4. NEGATIVE CONTROL: restore the subject, empty the GRANDFATHERED stage,
#      and confirm the exemption still holds — otherwise STAGE-001 fails today.
cp "$ROOT/$SUBJECT" "$SUBJECT"
empty_block "$GRANDFATHERED"
./scripts/cost-audit.sh >/dev/null 2>&1 \
    || fail "the grandfathered STAGE-001 tripped the gate — the exemption does not work"

printf '✓ cost-audit red-proof: control clean → a shipped stage whose orchestration_cost is the UNFILLED TEMPLATE (comment and all) is REJECTED by name, with a reason → the grandfathered stage is still exempt.\n'
