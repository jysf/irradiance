---
# A PATCH is a lightweight fix to ALREADY-SHIPPED behavior (a bug or UX
# papercut) that adds NO new feature/command and doesn't warrant a full
# spec + stage. See AGENTS.md "Patch lane" and docs/decisions/DEC-003.
#
# Collapsed cycle: patch -> verify -> ship (design+build fused into one
# test-first pass; the INDEPENDENT verify is KEPT). It uses the same task.*
# schema as a spec, so `just validate`, `just cost-audit`, and `just status`
# treat a patch as first-class.

task:
  id: PATCH-003
  type: patch                      # epic | story | task | bug | chore | patch
  cycle: patch                     # patch | verify | ship  (collapsed from a spec's 5)
  blocked: false
  priority: medium
  complexity: S                    # S | M  (an L fix is probably a spec, not a patch)

project:
  id: PROJ-001
  # No `stage:` — a patch attaches to the PROJECT, not a stage.
repo:
  id: irradiance

agents:
  implementer: claude-opus-5  # the patch pass (tier_map.build; DEC-005)
  verifier: claude-opus-5        # independent verify — KEPT (tier_map.verify; a separate session/agent)
  created_at: 2026-09-06

references:
  decisions: []                    # add a DEC only when there's a real decision

# Cost: patch + verify are the metered cycles — `just cost-audit` requires a
# real tokens_total on both for a shipped patch. ship is main-loop (null-with-note).
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# PATCH-003: close PATCH-002s two ship blockers the body is scanned as front matter and the gate contradicts DEC-013

## Problem

`PATCH-002` shipped a gate and was **merged before its independent verify ran** —
the maintainer's call, taken to close an ID-collision window. The verify then
returned **⚠ PUNCH LIST: 2 ship-blockers, 10 follow-ups**, against code already
on `main`. Both ship-blockers are real; both were reproduced by the orchestrator
before this patch was written.

**`SB-2` — the gate reads the body as front matter, so prose satisfies it.**
The awk toggled a boolean on every bare `---`, so a **third** one (a horizontal
rule in the body) flipped the body back into "front matter", and `in_oc` was
never cleared at the front-matter close. It only bites when `orchestration_cost:`
is the **last front-matter key** — and it is the last key in the template and in
**all five** `STAGE-00N` files, so the vulnerable arrangement is the repo's
default shape. Reproduced: `STAGE-002` with an empty block plus
`- tokens_total: 84200000` in prose → `cost-audit` **rc=0**.

⚠ **That is documentation about the field satisfying a check on the field — the
exact class the patch exists to prevent**, and the third instance of
`attribute-text-inside-doc-comments` in two days, this time inside the function
written to avoid it.

**`SB-1` — the gate reverses a recorded decision and says it didn't.**
`PATCH-002` wrote *"No `DEC-*`: it decides nothing new."* `DEC-013` §5 reads
*"Warn-only, no gate, no view yet: capture first."* The stage template and all
five stage files told the author, at the field, *"Warn-only, never a gate."*
`STAGE-003`/`004`/`005` would have blocked at close on a field their own front
matter called never a gate.

## Fix

- **`SB-2`** — count the front-matter delimiters and `exit` at the second, so the
  body is unreachable rather than merely unlikely (`scripts/_lib.sh`).
- **`SB-1`** — `DEC-022` amends `DEC-013` §5 explicitly, with the measurement
  that expired *"capture first"* (one capture in three weeks; ≈31 % of a stage's
  spend), and the comment at the field in the template **and all five stage
  files** now says the field is gated.
- **`FU-1`** — the `#` guard is unreachable; kept as defence in depth, but the
  comment no longer credits it as the anti-trap. The `^- …[0-9]+` anchor is the
  defence.
- **`FU-2`** — the red-proof's summary claimed the stage is rejected *by name*
  while only ever checking the reason. It now asserts the name; `M4b` is caught.
- **`FU-4`** — `status: "shipped"` silently skipped the check. Quotes stripped.
- **`FU-6`** — `tokens_total: 0` satisfied the gate while the spec-side gate
  treats 0 as absent. Now requires `> 0`.

**And `SB-2` gets its own case in the red-proof**, proven to fail against the old
awk — otherwise this patch repeats `PATCH-002`'s sin of fixing a thing without a
falsifier.

## Failing Tests

- `./scripts/cost-audit-red-proof.sh` — now four cases: the unfilled template
  (comment and all), **prose in the body (`SB-2`)**, rejection **by name**
  (`FU-2`), and the grandfathered stage still exempt.
- Verified against the old awk: the `SB-2` case fails with
  *"the front-matter scan is leaking into the body again."*

## Deferred, with reasons

- **`FU-3` (`cancelled` not audited)** — deliberate. A cancelled stage did not
  ship and has no orchestration to record. Recorded here so it is a decision
  rather than an oversight.
- **`FU-5` (fifth `+toolchain` instance)** — real and not this patch's: `just
  lint` and `lint-red-proof.sh` call a bare `cargo clippy`, which fails when the
  default toolchain is nightly. Belongs with the other four instances, not here.
- **`M4a`** — the unreachable `#` guard **survives deletion by design**; now
  documented in place rather than silently load-bearing-looking.

## Verification (independent — KEPT)

Run in a SEPARATE session/agent from the patch pass. This is the one discipline
the framework retrospective proved catches real defects; it is non-negotiable
for a patch.

- Run the project's full gate suite (tests, lint/format, and any security/
  dependency gates the repo defines).
- Confirm the failing tests now pass and no existing test regressed.
- Output: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Patch Completion

*Filled at the end of the patch pass, before verify.*

- **Branch / PR:**
- **Fix summary:** <one or two lines>
- **New decision emitted:** `DEC-NNN` (only if a real decision was made)
- **Reflection (1 line):** what would make this class of fix faster next time?
- **Defect-catch-stage:** where the bug this patch fixes was caught —
  `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) —
  one word, for the cross-project defect-escape distribution. (A patch usually
  fixes an `escaped` defect; that's the signal a behavioral pre-flight was missed.)

## Ship

- Add a CHANGELOG entry under `[Unreleased] → Fixed`.
- Append cost sessions (patch + verify metered; ship null-with-note), then
  compute `cost.totals`.
- `just advance-cycle PATCH-NNN ship`, then `just archive-patch PATCH-NNN`.
- **No stage bookkeeping** — a patch attaches to the project, not a stage.
