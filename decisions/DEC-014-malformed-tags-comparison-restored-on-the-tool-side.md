---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-014
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-03
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - tests/support/tools.rs
  - tests/metadata_oracle.rs

tags:
  - oracle
  - testing
  - dec-012
  - dec-013
---

# DEC-014: The malformed-tags comparison is restored — generically, on the tool side, and load-bearing

## Decision

**`tools::diff()` (`tests/support/tools.rs`, `SPEC-010`) treats an `Unreadable`
tool reading as a mismatch UNLESS `Sensor::malformed_tags` names the same DNG
tag.** This is `DEC-013`'s rejected Option C — "`diff()` reads `malformed_tags`
and skips exempted fields generically … no per-file knowledge in the
comparator at all" — implemented correctly this time, and it is `DEC-013`'s
true successor: the conclusion that decision reached (remove the hardcoded
guard) stands, and this record is what should have replaced it once `FU-1`
was fixed as `FU-8` specified.

## Context

`DEC-013` was rejected at `SPEC-005`'s verify (`SB-1`) on three measured
counts: the guard it shipped was dead code (removing it left 21 tests green),
its stated premise was false (`K3III.DNG` would not fail `AC1` "forever" — the
two sides already agreed by accident), and it recorded choosing the generic
Option C while shipping the hardcoded-by-tag-number Option B. All three stand;
nothing here relitigates them.

`DEC-013`'s own rejection record went on to predict a future: *"fix `FU-1`
and the guard becomes necessary"* — and warned against re-adding a corrected
guard in advance, on the reasoning that doing so would let the real question
(is a `DEC-012`-tolerated tag exempt from this oracle?) get answered by
accident when `FU-1` landed, rather than deliberately. `SPEC-005/FU-8`
(verify round 2) then measured that prediction directly: it built the
tri-state `FU-1` specifies, ran it **without** a `malformed_tags` comparison
(red on `K3III.DNG`) and **with** one (21 green — the alarm never fires).
`SPEC-010` is the deliberate decision `DEC-013` wanted `FU-1`'s fixer to make,
made with the measured evidence already in hand rather than by absorption.

## Alternatives Considered

- **Option A: no guard at all — every optional field's `Unreadable` state is
  always a mismatch.**
  - What it is: `diff()` never consults `malformed_tags`; a garbled tool
    reading is unconditionally red, even when our own reader independently
    dropped the same tag for the same reason.
  - Why rejected: this is what `DEC-013`'s rejection actually shipped
    (`tools::diff()` compares all eleven fields unconditionally), and it
    makes `K3III.DNG` a **permanent, expected red** the moment `FU-1` lands —
    exactly the failure `DEC-013`'s original text (its one sound instinct)
    identified. `AC3` requires `K3III.DNG` stay green, for a stated reason;
    this option cannot satisfy it without excluding the file from `AC1`
    entirely (`DEC-013`'s own Option A, rejected there for throwing away ten
    working checks to avoid one expected one).

- **Option B: a corrected, still-hardcoded guard.**
  - What it is: keep a per-tag exception (`BlackLevelRepeatDim` by name), now
    correctly gated on the tool side reading `Unreadable` rather than on our
    side alone.
  - Why rejected: identical to `DEC-013`'s original defect on the axis that
    actually mattered — it needs updating every time a **different** tag's
    malformed reading surfaces on a **future** corpus file, which is exactly
    what `DEC-013`'s Option C was chosen to avoid, and what its rejection's
    count 3 measured had silently reverted to (Option C recorded, Option B
    shipped).

- **Option C (chosen): the generic guard, on the tool side, exercised by a
  real file on every run.**
  - What it is: `compare_optional` (`tests/support/tools.rs`) takes a DNG tag
    number, not a field name, and checks `malformed_tags.contains(&tag)` —
    one function, called once per optional field, no per-file or per-tag
    branching anywhere in `diff()`.
  - Why selected: it is `DEC-013`'s own chosen-but-unshipped option, it
    generalizes to any tag `DEC-012`'s tolerance covers (not just
    `BlackLevelRepeatDim`), and `SPEC-010`'s red-proof
    (`removing_the_malformed_comparison_turns_k3iii_red` /
    `the_malformed_comparison_control_is_green`) proves it is load-bearing on
    the real corpus — the exact property `DEC-013`'s guard never had.

## Consequences

- **Positive:** `AC3` holds for a *stated* reason, not an accident — the
  tool-side tri-state and our own `malformed_tags` agree because both name
  the same tag, and that agreement is asserted, not assumed
  (`k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason`).
- **Positive:** the guard is proven necessary, not merely present. Removing
  it (`diff_with_malformed(sensor, reading, &[])`) turns `K3III.DNG` red,
  every run — the property `DEC-013`'s dead-code guard never had, and the
  reason `DEC-013` was rejected in the first place.
- **Negative:** the same sharp edge `DEC-013` named survives structurally
  unchanged — a `malformed_tags` entry that is itself wrongly classified
  (`DEC-012`'s job) would now escape `AC1` on that tag too. `DEC-012`'s own
  tests exercise that classification directly; this decision does not
  introduce a new way for that class of bug to hide, it inherits the one
  `DEC-013` already accepted.
- **Neutral:** `tools::diff()`'s doc comment now states this decision inline,
  the way `DEC-013`'s original text asked `diff()`'s doc comment to (its one
  requirement this record keeps).

## Validation

Right if the red-proof stays green in both directions on every future corpus
file that adds a new `malformed_tags` entry — i.e., if `compare_optional`
never needs a per-tag or per-file exception to keep agreeing for a stated
reason. Revisit if a future file's malformed tag agrees with our reader for a
reason `compare_optional`'s three-state rule cannot express (e.g. two
different malformed shapes of the same tag that should be told apart) — that
would mean the states themselves need to grow, not just the guard's inputs.

## References

- Related specs: `SPEC-005` (`DEC-013`'s origin and rejection), `SPEC-010`
  (this decision; `AC1`–`AC3`, `AC6`, `AC7`)
- Related decisions: `DEC-012` (`Sensor::malformed_tags`' contract, read here
  generically); `DEC-013` (`rejected` — this is its successor)
- Code: `tests/support/tools.rs` — `diff()`, `diff_with_malformed()`,
  `compare_optional()`
- Tests: `removing_the_malformed_comparison_turns_k3iii_red`,
  `the_malformed_comparison_control_is_green`,
  `k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason`
  (`tests/metadata_oracle.rs`)
- Constraints: `oracle-must-be-shown-red`
- Raised by: `SPEC-010` build cycle, `AC7`
