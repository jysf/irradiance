---
insight:
  id: DEC-007
  type: decision
  confidence: 0.92
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
supersedes: DEC-006
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/lib.rs
  - scripts/lint-red-proof.sh
  - .github/workflows/ci.yml

tags:
  - oracle
  - lints
  - ci
  - spec-001
---

# DEC-007: The lint red-proof injects into a copy of the LIBRARY, not a snippet

## Decision

The red-proof **copies the crate to a temp directory, injects violating functions
into the copied `src/lib.rs` immediately after its attribute prologue, runs
clippy there, and asserts three things**: that clippy actually ran, that it
exited non-zero, and that all three expected lint names appear in its output.

The working tree is never mutated. `tests/lint_policy_red.rs.disabled` and its
self-contained `#![deny(...)]` header are **deleted** — that mechanism is what
DEC-006 built, and it is what this supersedes.

## Context

SPEC-001's verify cycle found the mechanism proved the wrong thing. Reproduced
independently by the orchestrator before acceptance:

**Delete the `#![deny(...)]` block from `src/lib.rs`, add**
`pub fn read_u8(b: &[u8], at: usize) -> u8 { b[at] + 1 }` — a function that both
indexes a slice and does unchecked arithmetic on attacker-influenced input:

```
cargo build 0 · clippy -D warnings 0 · fmt --check 0 · test 0
MSRV 1.90.0 0 · deny licenses 0 · RED-PROOF 0
```

**Seven green gates and a shipped panic on untrusted input.** The red-proof's own
output said why: `note: the lint level is defined here --> tests/lint_policy_red.rs:14`
— the snippet's own header, never `src/lib.rs`. It proved that a `#![deny]` written
two lines above a violation rejects that violation, which is a fact about Rust,
not about this library.

`src/lib.rs` claimed the policy was *"enforced mechanically, not only by review."*
That claim was false: it was enforced by review. DEC-006 named the gap honestly in
its own **Negative** — it was unfinished mechanism, not misrepresentation — but an
unfinished mechanism that reports green is exactly the `oracle-must-be-shown-red`
failure this repo exists to avoid, arriving one level up in the proof itself.

A second defect, same cycle: the script asserted only a **non-zero exit**. With
`cargo clippy` unavailable it printed *"✓ lint policy red-proof: the violating
snippet failed to compile as expected"* and exited 0 — **passing having proven
nothing.** Verified by putting a stub `cargo` on `PATH`.

## Alternatives Considered

- **Option A: move the lints to a `[lints]` table in `Cargo.toml`.**
  - Why rejected: `[lints]` applies to **every target in the package**, including
    tests and `src/bin/irr.rs`, which are deliberately allowed to `unwrap()`.
    Each would then need its own `#![allow(...)]`, so the exception surface grows
    rather than shrinks. It also does not solve the real problem — a `[lints]`
    table can be deleted exactly as an attribute block can. It moves the config,
    it does not make the proof honest.

- **Option B: have the red-proof grep `src/lib.rs` for the five lint names.**
  - Why rejected: that asserts the *text is present*, not that it *bites*. It
    would pass with the lints at `allow` level, or misspelled, or shadowed. It is
    a shape-check standing in for a behaviour-check — precisely the substitution
    AGENTS.md §12's behavioral pre-flight warns against.

- **Option C (chosen): inject a violation into a copy of the real library and
  require the real policy to reject it.**
  - Why selected: it is a **mutation test**, so it fails for the right reason and
    only the right reason. Proven both directions before adoption:
    - policy present + violation injected → clippy exits 101, all three lints
      fire, and `the lint level is defined here` resolves to **`src/lib.rs:31`**
    - policy **removed** + violation injected → clippy exits **0**, so the
      assertion fails and the removal is caught
  - Copying rather than editing in place also removes DEC-006's `trap`-based
    restore entirely: nothing can be left behind, because nothing was touched.

## Consequences

- **Positive.** `no-panics-on-untrusted-input` becomes mechanically enforced for
  real, and `src/lib.rs`'s claim to that effect becomes true. The proof now fails
  if the policy is weakened, removed, or if clippy silently isn't running.
- **Positive.** No working-tree mutation, so no restore path to get wrong.
- **Negative.** The injection point is found by parsing the attribute prologue
  (blank lines, `//!` docs, `#![...]` blocks, tracking bracket depth). That is
  text surgery and it can break if `lib.rs`'s prologue takes an unusual shape.
  Mitigated by the third assertion: if injection lands somewhere the lints don't
  apply, the expected lint names will be **absent** and the proof fails loudly
  rather than passing. A naive `max()` over `)]` lines was tried first and landed
  inside the test module's `#[allow(...)]`, which suppressed two of the three
  lints — caught only because the names were checked.
- **Neutral.** DEC-006 is superseded, not deleted. Its Validation section — which
  listed the three error identities — was right; the script simply did not
  implement it.

## Validation

Right if a deliberate weakening of `src/lib.rs`'s policy turns CI red. The fix is
not complete until **both** directions are demonstrated in the same change:
rejection with the policy, and failure of the proof without it.

Revisit if `lib.rs`'s prologue grows a shape the injection heuristic mishandles —
the symptom will be a loud missing-lint failure, not a silent pass.

## References

- Supersedes: `DEC-006` (snippet-with-its-own-header mechanism)
- Verify punch list P1-1 and P1-2, SPEC-001, reviewed at `29515ab`
- Constraints: `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`
