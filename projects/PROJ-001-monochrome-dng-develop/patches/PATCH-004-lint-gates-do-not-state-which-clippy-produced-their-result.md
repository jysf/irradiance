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
  id: PATCH-004
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

# PATCH-003: lint gates do not state which clippy produced their result

## Problem

`PATCH-003`'s verify raised `FU-5`: *"`just lint` and `lint-red-proof.sh` call a
bare `cargo clippy`; this machine's default toolchain is now nightly, which has
no clippy — both fail."* **Fifth `+toolchain` instance.**

⚠ **It does not reproduce on the orchestrator's machine, and the reason is the
real finding.** Measured 2026-09-06:

```
nightly toolchain has cargo-clippy   NO
default toolchain                    nightly-aarch64-apple-darwin (default)
bare `cargo clippy` resolves to      /opt/homebrew/bin/cargo-clippy → clippy 0.1.97
just lint                            rc=0        just lint-red-proof   rc=0
```

Homebrew's `cargo-clippy` (rust 1.97.1) **shadows the rustup shim**, so both
commands pass — while linting with a compiler nobody selected. One reviewer's
environment failed **loudly**; this one succeeds **silently**.

**The silent success is the worse half.** A failure gets investigated. A green
that does not say what produced it gets believed — and `lint-red-proof.sh` is
the red-proof guarding the **panic-free policy**, a blocking constraint. It was
reporting *"all five lints fired"* without naming the compiler that fired them.

That is a surface reporting a result it has not established: the same defect
`SPEC-016` exists for, and the same one `PATCH-002`/`PATCH-003` just paid for
twice — this time inside the proof that guards a constraint.

## Fix

- **`scripts/lint-red-proof.sh`** — a second assertion beside the existing one:
  `cargo clippy --version` answering is not the same as knowing *which* clippy.
  Resolve `cargo-clippy` on `PATH` and **`die` if it cannot be named**. The
  success line now ends `PROVED BY: clippy 0.1.97 (/opt/homebrew/bin/cargo-clippy)`,
  so a green states its own provenance.
- **`just lint`** — prints the clippy version **and its resolved binary**, and
  says it is unpinned. Deliberately **not** pinned: `lint` is the fast local
  check and `lint-ci` is the pinned one; collapsing them would delete a
  distinction the repo uses. The defect was never that `lint` is unpinned — it
  is that it did not say so.
- **`just lint-ci`** — prints its clippy too. The pin makes the answer
  predictable; it does not make it stated.

**No `DEC-*`.** This changes what the gates *report*, not what they enforce, and
introduces no policy. ⚠ Stated explicitly because `PATCH-002` made exactly this
claim wrongly — checked: no existing decision says these gates should be silent
about their toolchain, and `DEC-013`-style prior art does not apply.

## Failing Tests

Both new failure paths watched red, each via a `PATH` shim, tree untouched:

| mutation | result |
|---|---|
| `cargo clippy --version` fails | `ERROR: … clippy is not available, so this proof can prove NOTHING` (pre-existing assertion, confirmed still load-bearing) |
| version answers but `cargo-clippy` is **not** on `PATH` | `ERROR: … cannot name the binary that produced its result. Refusing to report green.` (**the new assertion**) |

And the positive: `just lint` now prints
`using clippy 0.1.97 (/opt/homebrew/bin/cargo-clippy) — unpinned`, while
`just lint-ci` prints `using clippy 0.1.98 … (pinned: ~/.cargo/bin/cargo +stable)`
— **the two-clippy situation is now visible in one line each**, which is what
made this finding take five instances to pin down.

## ⚠ And the workaround in the finding does not work here

`FU-5` said *"Passes under `RUSTUP_TOOLCHAIN=stable`."* Measured on this machine,
it does not:

```
RUSTUP_TOOLCHAIN unset     clippy 0.1.97 @ /opt/homebrew/bin/cargo-clippy
RUSTUP_TOOLCHAIN=stable    clippy 0.1.97 @ /opt/homebrew/bin/cargo-clippy   ← NO CHANGE
PATH=~/.cargo/bin:$PATH    clippy 0.1.98 @ ~/.cargo/bin/cargo-clippy        ← what works
```

`cargo` resolves a subcommand by finding `cargo-<name>` **on `PATH`**, so
`RUSTUP_TOOLCHAIN` never reaches it. **PATH order selects clippy; the toolchain
variable does not.** That is why `just lint-ci` pins *both*, and either alone is
insufficient here.

Recorded in `guidance/toolchain-brief.md`, which AGENTS.md §17 says is injected
into every build handoff — so the next session does not re-derive it. This is the
sixth `+toolchain` instance and the first where the *workaround* was the wrong
part.

## What this does NOT fix

- **The underlying environment.** Homebrew's clippy still shadows rustup's, and
  nightly still has none. This patch makes that *visible*, not absent. Installing
  clippy into the default toolchain, or removing the Homebrew one, is the
  maintainer's call about their machine — not a repo change.
- **The other four `+toolchain` instances.** They are recorded in
  `guidance/toolchain-brief.md`; this patch adds the fifth's *resolution*, not a
  general fix for the class.

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
