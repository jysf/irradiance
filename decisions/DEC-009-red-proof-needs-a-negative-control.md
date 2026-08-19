---
insight:
  id: DEC-009
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-18
supersedes: DEC-007
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - scripts/lint-red-proof.sh
  - .github/workflows/ci.yml

tags:
  - oracle
  - lints
  - ci
  - spec-001
---

# DEC-009: The red-proof needs a negative control, and must exercise every lint

## Decision

The red-proof runs the **same clippy invocation twice**:

1. **Control — the unmutated copy must exit 0.** If it doesn't, the toolchain,
   the copy, or the crate is broken, and *nothing downstream means anything*.
   Fail here with that message.
2. **Mutation — the injected copy must exit non-zero, and every lint in the
   policy must be named in its output.**

Additionally: **all five policy lints get a violation and an `EXPECTED_LINTS`
entry** (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
`arithmetic_side_effects`); the prologue parser **skips plain `//` comments** and
refuses any injection point that is not strictly after the last inner attribute;
and `INJECT_AT=1` must not crash.

`DEC-007`'s core is retained — mutate a *copy* of the real library, never a
snippet, never the working tree. Only its assertion set was wrong.

## Context

`DEC-007` closed the P1 that `DEC-006` left open, and it genuinely closed it: the
proof now fails when the policy is deleted outright. SPEC-001's second verify
cycle then showed the same false green is reachable by **smaller** edits, and
measured both to seven green gates with a panic shipped in the library.

**PL-1 — the assertion I called the mitigation does not mitigate.**
A plain `//` comment is legal in a prologue. The parser's `case` handles only
blank, `//!` and `#![`, so a `//` line becomes the injection point — *above* the
inner attributes. `pub fn` there is a syntax error, clippy exits non-zero, and
**rustc renders the attribute's source span in the diagnostic**, putting all
three expected lint names in the log **without a single lint firing**.

Reproduced independently by the orchestrator. With a `//` comment before
`#![forbid]`, `#![deny(` changed to `#![allow(` at column 0, and two panicking
public functions (`v[0] + n` and `.expect()`):

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected
  violations (clippy exit 101; ... all fired).
```

Every word of that message is false. **`DEC-007`'s Negative states the opposite
as its mitigation** — that a mis-landed injection would be caught by absent lint
names. It is not, because the names arrive from rendered source, not from lints.

**PL-2 — the proof covers three of five lints.** `clippy::panic` and
`clippy::expect_used` appear in neither the injected violations nor
`EXPECTED_LINTS`. Deleting exactly those two from the policy and shipping
`panic!()` and `.expect()` in a public function passes everything. Acceptance
criteria 4 and 5 hold for 3/5 of the policy they describe.

**The general lesson, which is the reason this DEC exists rather than a patch:**
every assertion in `DEC-007` inspected the *mutated* run. None established that
the *unmutated* run was healthy. A test with no negative control cannot
distinguish "failed for the reason I intended" from "failed for any reason at
all" — and a red-proof is exactly a test whose whole value is that distinction.
Three rounds of adding assertions about the failure case never found this;
one control run does.

## Alternatives Considered

- **Option A: keep adding assertions to the mutated run** (check the error
  *codes*, parse clippy's JSON, count diagnostics).
  - Why rejected: this is round three of that strategy. Each iteration closed the
    specific hole it was shown and left the class open. JSON parsing would defeat
    the rendered-source-span trick, but not "clippy failed for an unrelated
    reason" in general. The control run closes the class.

- **Option B: assert the injected file compiles cleanly *without* the policy.**
  - Why rejected: a third clippy run, more moving parts, and it still doesn't
    prove the unmutated crate is healthy — which is the property actually needed.

- **Option C (chosen): a negative control, plus full lint coverage.**
  - Why selected: the control is one extra invocation of a command already being
    run, it needs no parsing, and it fails loudly with a message that names the
    right cause. Full coverage is bookkeeping the acceptance criteria already
    implied.

## Consequences

- **Positive.** The proof can no longer pass because of a syntax error, a broken
  copy, a toolchain fault, or an injection that landed out of scope — all of
  which now fail the control or the coverage check.
- **Positive.** All five lints are exercised, so the acceptance criteria mean what
  they say.
- **Negative.** Two clippy runs per proof, roughly doubling its cost. It is a
  fast crate; this is not a real objection today and would be one on a large one.
- **Negative — stated plainly:** this is the **third** design for one gate.
  `DEC-006` → `DEC-007` → `DEC-009`. Each was found insufficient by an independent
  reviewer, and each time the previous author (including me) believed it closed.
  That track record is itself evidence for keeping the independent verify cycle
  regardless of cost, and against ever letting the builder self-certify this gate.
- **Neutral.** `DEC-007` is superseded, not deleted. Its core mechanism survives
  intact and is restated here.

## Validation

Right if **all four** of these hold, demonstrated in the same change:

1. policy present → proof passes, all five lints named
2. policy deleted → proof fails
3. policy `deny` → `allow` → proof fails
4. a `//` comment inserted in the prologue → proof fails (or injects correctly),
   never passes

Revisit if a fourth bypass is found. If there is one, the honest conclusion may be
that a shell script is the wrong tool and this belongs in a compile-fail test
harness — but that costs a dev-dependency, so it needs its own decision.

## References

- Supersedes: `DEC-007` (whose Negative this falsifies), which superseded `DEC-006`
- SPEC-001 verify round 2, PL-1 and PL-2, reviewed at `c10f8e6`
- Constraints: `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`
