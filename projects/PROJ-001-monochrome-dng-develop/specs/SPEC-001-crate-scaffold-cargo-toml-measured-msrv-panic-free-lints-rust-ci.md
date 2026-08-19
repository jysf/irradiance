---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-001
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-sonnet-5         # HANDOFF-001, cycle: build (DEC-005 tier_map.build)
  created_at: 2026-08-18

references:
  decisions: [DEC-006]             # [DEC-NNN, DEC-MMM] — DEC-006 emitted during build (this repo's namespace)
  constraints:                     # [constraint-id-1, constraint-id-2]
    - no-panics-on-untrusted-input
    - no-copyleft-dependencies
    - library-not-application
    - oracle-must-be-shown-red
    - no-new-top-level-deps-without-decision
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling every other spec in STAGE-001"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: null
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 197940
      estimated_usd: null
      duration_minutes: 60
      recorded_at: 2026-08-18
      notes: "tokens_total genuinely unavailable to me: I ran as a Task/Agent-tool subagent with no /cost interface and no token-usage tool in my toolset. Per metering_source: subagent_tokens (.repo-context.yaml) and DEC-013, the ORCHESTRATOR reads this number directly from this Agent invocation's own result metadata (subagent_tokens) after I report — that is the intended reader for this metering source, not a number I self-report. Please fill tokens_total from that result and run `just handback-sync SPEC-001`. Also: committed to feat/spec-001-crate-scaffold locally but did NOT push or open a PR — my instructions said commit + do not merge and were silent on push/PR, and pushing to the real jysf/irradiance remote felt like it warranted an explicit go-ahead rather than an autonomous call. The branch is ready to push as-is."
  totals:
    tokens_total: 197940
    estimated_usd: 0.00
    session_count: 1
---

# SPEC-001: Crate scaffold: Cargo.toml, measured MSRV, panic-free lints, Rust CI

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-001]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

The crate does not exist. There is no `Cargo.toml` and no `src/`, so nothing
else in STAGE-001 can be built or tested. AGENTS.md §5 records the measured
toolchain this must be pinned against, and §12 the panic-free lint set that has
been verified to compile and to fire as an error on a violating function.

## Goal

A buildable, lintable, CI-gated crate. In one change: `edition = "2021"`; a
`rust-version` **measured from the real dependency set, never guessed**;
`#![forbid(unsafe_code)]`; the panic-free clippy set (`unwrap_used`,
`expect_used`, `indexing_slicing`, `panic`, `arithmetic_side_effects`) allowed
only inside `#[cfg(test)]` and `src/bin/irr.rs`; and CI jobs for `fmt --check`,
`clippy -D warnings`, `test` and `cargo deny check licenses`.

⚠ Per DEC-003 CI cannot run tier-B tests, so this must not leave anyone reading
a green badge as "bit-exact".

## Inputs

- `AGENTS.md` §5 (measured toolchain), §11 (error handling), §12 (testing bars), §13 (git/PR)
- `guidance/toolchain-brief.md` — **inject this into the build prompt** (DEC-004 rule 5)
- `guidance/constraints.yaml` — `no-panics-on-untrusted-input`, `no-copyleft-dependencies`, `library-not-application`
- `DEC-002` (**`proposed`**, 0.72) — target surface is OPEN
- `DEC-003` — CI cannot run tier-B tests
- `.github/workflows/ci.yml` — currently language-agnostic gates only

## Outputs

- `Cargo.toml` — `edition = "2021"`, `rust-version`, `[lib]` + `[[bin]] irr`, `MIT OR Apache-2.0`
- `src/lib.rs` — crate root carrying the lint policy and a typed `Error` skeleton
- `src/bin/irr.rs` — the internal dev/oracle binary, minimal
- `deny.toml` — permissive-only allow-list
- `.github/workflows/ci.yml` — Rust jobs added alongside the existing gates
- `app.just` — `build` / `test` / `lint` / `typecheck` filled in, replacing the TODO stubs

## Acceptance Criteria

1. `cargo build`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
   and `cargo fmt --check` all pass from a clean checkout.
2. `Cargo.toml` declares `edition = "2021"` and `rust-version = "1.90"`, and CI has a job
   that checks against **exactly that toolchain** — the number is only meaningful if it is tested.
3. `#![forbid(unsafe_code)]` on the library.
4. The five panic-free lints are `deny`-level on the library, and **allowed** inside
   `#[cfg(test)]` and in `src/bin/irr.rs`.
5. **The lint policy is shown RED** — a CI step compiles a deliberately violating snippet
   and asserts the build FAILS. A lint policy that has never rejected anything is not a policy.
   (This is `oracle-must-be-shown-red` applied to a gate rather than an oracle.)
6. `cargo deny check licenses` passes with a permissive-only allow-list, and is a CI job.
7. `irr` builds as a bin target and is **absent from the library's public API**.
8. `app.just` recipes run; AGENTS.md §6's command block matches them.
9. CI does **not** claim bit-exactness. Per DEC-003 tier-B is absent on a runner — if the
   README or CI names a badge, it must not imply the decoder is verified.

## Failing Tests

Written at design, red today because no crate exists. Each must go green by build.

**Gate-level (these ARE the tests for a scaffold spec — all currently fail):**
```bash
cargo build                                              # no Cargo.toml -> fails
cargo clippy --all-targets --all-features -- -D warnings # fails
cargo fmt --check                                        # fails
cargo deny check licenses                                # fails
~/.cargo/bin/cargo +1.90.0 check                         # the declared MSRV -> fails
```

**The red-proof for the lint policy** — this must FAIL to compile, and CI asserts that it does:
```rust
// tests/lint_policy_red.rs.disabled — compiled by CI on purpose, expected to FAIL
pub fn violates(v: &[u8], n: u8) -> u8 { v[0] + n }   // indexing_slicing + arithmetic_side_effects
pub fn also(v: &[u8]) -> u8 { *v.first().unwrap() }   // unwrap_used
```

**One real unit test**, so `cargo test` is not vacuously green:
```rust
#[test]
fn error_type_is_public_and_non_exhaustive() {
    // Error is constructible from within the crate and Debug-printable.
    let e = crate::Error::Truncated { at: 0, len: 0 };
    assert!(format!("{e:?}").contains("Truncated"));
}
```

## Non-Goals

- **No decoding of any kind.** No TIFF walk, no tag model, no unpack — those are SPEC-003/004.
- **No dependencies.** SPIKE-001 showed the container reader and unpacker need none.
  Adding one here needs a DEC and is almost certainly premature.
- **No `no_std` commitment.** `DEC-002` proposes it and is still `proposed`, gated on
  measurement. Do not decide it in this spec; leave the door open by not depending on `std`
  gratuitously, but do not add the feature machinery yet.
- **No `rayon`, no SIMD dispatch** — same reason.
- **No crates.io publish** — STAGE-004 puts it out of scope for PROJ-001.

## Notes for the Implementer

### Two things were measured at design. Do not re-derive them.

**1. The lint policy rejects the obvious byte-reading idiom.** Verified on clippy 0.1.97
*and* 1.90.0. This pattern — bounds-check with `.get()`, then index — **fails**:

```rust
let s = buf.get(at..end).ok_or(..)?;
Ok(u16::from_le_bytes([s[0], s[1]]))   // error: indexing may panic  (x2)
```

This one is clean:

```rust
let s: [u8; 2] = buf.get(at..end).and_then(|s| s.try_into().ok()).ok_or(..)?;
Ok(u16::from_le_bytes(s))
```

⚠ **SPIKE-001's decoder used the failing pattern throughout.** Its measured "229 lines,
zero dependencies" is therefore an *underestimate* of the lint-clean version. Expect the
real reader to be somewhat larger, and do not treat the spike's line count as a target.

**2. MSRV — measured, deliberately conservative, and honest about it.**
`1.90.0` (the oldest toolchain installed here) compiles the intended crate root, including
`#![no_std]` + `alloc` and the full lint set. **The true floor is lower but UNMEASURED** —
nothing older is installed, and declaring a number we have not compiled would be exactly
the guess AGENTS.md §12 forbids. So: declare `1.90`, test `1.90` in CI, and treat lowering
it as a later change that requires `rustup toolchain install` and a real measurement.

### The toolchain trap that will cost you a loop

`cargo` on `PATH` is **Homebrew's**, not a rustup shim, so `cargo +1.90.0` fails with
`no such command`. Use the shim explicitly: `~/.cargo/bin/cargo +1.90.0 check`.
Full detail in `guidance/toolchain-brief.md`.

### Scope discipline

If this spec starts wanting to decode anything, stop — that is SPEC-003. The value here is
that every later spec inherits a crate where a panic on untrusted input **cannot compile**.

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
