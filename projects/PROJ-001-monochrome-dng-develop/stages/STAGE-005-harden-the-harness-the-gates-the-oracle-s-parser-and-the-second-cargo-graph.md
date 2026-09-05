---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-005                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-30
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Protects the thesis rather than advancing it: every claim STAGE-001 makes rests on gates, and STAGE-001 shipped having measured that one of them had never run and another cannot tell an absent tag from an unreadable one.
  delivers:
    - "A metadata oracle that distinguishes an ABSENT tag from an UNPARSEABLE one"
    - "The fuzz crate inside a -D warnings gate, like the library"
    - "Every gate script audited for the mute-death shape, with the audit itself a test"
  explicitly_does_not:
    - "Any pixel work — that is STAGE-002"
    - "Any new decoder, camera or format"
    - "Settling the floating-vs-pinned CI toolchain question — that is a project-close decision"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-005: Harden the harness — the gates, the oracle's parser, and the second cargo graph

## What This Stage Is

When this stage ships, the machinery that *checks* `irradiance` is as trustworthy
as the code it checks. Three concrete gaps, every one **measured** during
STAGE-001 rather than suspected:

1. The metadata oracle **cannot tell an absent tag from an unparseable one** —
   both collapse to `None`, so **5 of 5 garbled tool readings diff clean**.
2. The **fuzz crate is linted by nothing** — root `cargo clippy --all-targets`
   mentions `irradiance-fuzz` **0 times**, because `DEC-011` deliberately keeps
   it outside the library's cargo graph.
3. **Six gate scripts carry the mute-death shape** that took the panic-free
   red-proof out for 17 consecutive CI runs: a zero-match `grep` under
   `set -o pipefail` aborting before the script's own `die`.

**Estimated effort: 6–10 hours.** Smaller than a feature stage on purpose.

## Why Now — and why it is NOT urgent

Because STAGE-001 measured all three and it would be dishonest to carry them as
notes. `follow-up-disposition-has-no-surface` exists precisely because findings
without an owner evaporate, and "we'll get to it" is the state that produced 34
un-dispositioned follow-ups.

**But nothing here blocks the plane.** This stage is deliberately *parallelisable
with* or *pullable ahead of* STAGE-002, and the maintainer should schedule it on
appetite rather than dependency. The honest argument for doing it early is that
STAGE-002 writes **two new oracles**, and gaps 1 and 3 are both about oracles
lying quietly.

## Success Criteria

- A **garbled** tool reading and an **absent** tag produce different results, and
  a test proves it on the shape that motivated it (`K3III.DNG`'s malformed
  `BlackLevelRepeatDim`)
- `cargo clippy` over the fuzz graph runs in CI with `-D warnings`, **with its own
  red-proof and negative control** (`oracle-must-be-shown-red`, as widened by
  `SPEC-003/FU-14` to cover gates)
- Every `grep` in every gate script under `scripts/` is guarded and its match
  count asserted — **and the audit is mechanical**, not a one-time read
- No gate can exit non-zero without printing its own reason; proven by forcing
  one to
- `just lint-ci` green under the CI toolchain, and CI **observed** green on the
  shipping SHA — not asserted from a laptop

## Scope

### In scope
- `tests/support/tools.rs`'s parse layer: a tri-state, and `opt()`/`req()`'s
  silent head-truncation of multi-valued readings
- A second `cargo clippy` invocation over `fuzz/Cargo.toml`, mirroring
  `deny` / `deny-fuzz`
- An audit — preferably a **test**, not a review — of `scripts/*.sh` for
  unguarded `grep` in gate paths
- Reconciling the tier-A fixture against live tool output where both are present

### Explicitly out of scope
- Pixels, of any kind
- New cameras, formats or decoders
- Deciding floating-vs-pinned CI toolchain (`floating-toolchain-plus-deny-warnings`
  is a **project-close** call with three options already written)
- Whether the fuzz crate should carry the library's five-lint panic-free policy —
  that is a design question `SPEC-011` opens, not one this stage pre-answers

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-010 (shipped on 2026-09-03) [M] Distinguish an unparseable tool reading from an absent tag
- [ ] SPEC-011 (frame) [S] Lint the fuzz crate — the second cargo graph
- [ ] SPEC-016 (frame) [M] The harness stops claiming what it has not checked — carries SPEC-005/FU-2, FU-3 and SPEC-012/FU-1, FU-2
- [ ] (not yet written) [S] Audit every gate script for the mute-death shape

**Count:** 1 shipped / 2 active / 1 pending

## Design Notes

**`SPEC-010`'s fix is already built and measured** — `SPEC-005/FU-8` implemented
a tri-state compared against `malformed_tags` during verify and confirmed all 21
oracle tests stay green, *and* that a tri-state **without** that comparison reds.
Reproduce it; do not re-derive it. ⚠ It also has a consequence for
`tools.rs`'s `diff()` doc comment and for `DEC-013` (`rejected`), whose
conclusion may deserve a **successor decision that is true**.

**`SPEC-011` is the shape `no-copyleft-dependencies` already had.** `cargo deny`
covered only the library graph until `SPEC-003`'s verify found a hand-written
table standing in for the missing invocation — *and it was wrong on the one crate
it existed to sanction*. The rule written into `constraints.yaml` then was **"if a
graph is not covered, add the invocation — do not write down the answer."** The
lint policy is now where the licence policy was.

**The audit spec should produce a TEST, not a report.** The six scripts measured
to carry the shape are `_lib.sh`, `decisions-audit.sh`, `ready.sh`,
`release-notes.sh`, `report_daily.sh` and `test.sh`. A read-through is a
self-report by one session; `a-gate-that-fails-mutely-is-a-gate-that-never-ran`
is codified at `N=4` **because** review kept missing it. ⚠ And note the trap that
caught the first fix: guarding **one** `grep` in a pipeline is not enough and
looks like it is — a zero-match emits nothing, so the next `grep` zero-matches
too.

## Dependencies

**None.** Every spec here is independent of the others and of STAGE-002. That is
the point: this stage exists so the debt has an owner, not so it blocks anything.
