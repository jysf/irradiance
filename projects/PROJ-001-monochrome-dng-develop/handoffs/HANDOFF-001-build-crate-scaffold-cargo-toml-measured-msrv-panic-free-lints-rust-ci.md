---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-001
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-18
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-001

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-001: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` to `claude-sonnet-5` (implementer)
for the **build** cycle.

Create the crate. Nothing else in PROJ-001 can be built or tested until this
lands, and every later spec inherits the guarantees it sets — most importantly
that **a panic on untrusted input cannot compile**.

## Context the Receiving Agent Needs

**Read first:** the spec, then `guidance/toolchain-brief.md` (per DEC-004 rule 5 —
it leads with the trap below), then `guidance/constraints.yaml`.

### The one thing that will cost you a loop if you skip it

`cargo` on `PATH` is **Homebrew's cargo, not a rustup shim**, because
`/opt/homebrew/bin` precedes `~/.cargo/bin`. So:

```
$ cargo +1.90.0 check
error: no such command: `+1.90.0`
```

Use the shim explicitly: `~/.cargo/bin/cargo +1.90.0 check`. Plain `cargo` for
everything else is fine (Homebrew 1.97.1).

### Two facts measured during design — transcribe, do not re-derive

**1. The lint policy rejects the obvious byte-reading idiom.** Verified on clippy
0.1.97 and on 1.90.0. Bounds-check-then-index **fails**:

```rust
let s = buf.get(at..end).ok_or(..)?;
Ok(u16::from_le_bytes([s[0], s[1]]))   // error: indexing may panic  (x2)
```

`try_into` on the slice is clean:

```rust
let s: [u8; 2] = buf.get(at..end).and_then(|s| s.try_into().ok()).ok_or(..)?;
Ok(u16::from_le_bytes(s))
```

**2. MSRV `1.90.0` is measured**, not guessed — it compiles the intended crate
root including `#![no_std]` + `alloc` and the full lint set. The true floor is
lower but **unmeasured** (nothing older is installed). Declare 1.90, test 1.90
in CI, and do not lower it without actually compiling against something older.

### Decisions that bind you

- **`DEC-002` is `proposed`, not accepted** — target surface is OPEN. Do **not**
  add `rayon`, do **not** commit to `no_std` feature machinery, do **not**
  introduce runtime SIMD dispatch. Leave the door open; decide nothing.
- **`DEC-003`** — CI cannot run tier-B tests, so nothing in CI may imply the
  decoder is bit-exact.
- **Constraints:** `no-panics-on-untrusted-input` (this spec is how it becomes
  mechanical), `no-copyleft-dependencies`, `library-not-application`.

### Dependency policy for this spec: zero

SPIKE-001 established the container reader and unpacker need no crates at all.
DEC-004 rule 4's trivial-dev-dep exception is **narrowed in this repo** — it must
still be permissive and must never be a RAW decoder. For this spec the right
number of dependencies is **zero**. If you think you need one, stop and ask.

## Expected Deliverables

1. `Cargo.toml` — `edition = "2021"`, `rust-version = "1.90"`, `[lib]`,
   `[[bin]] irr`, `license = "MIT OR Apache-2.0"`.
2. `src/lib.rs` — `#![forbid(unsafe_code)]`, the five panic-free lints at `deny`,
   a typed `Error` skeleton (`#[non_exhaustive]`), and one real unit test.
3. `src/bin/irr.rs` — minimal dev/oracle binary. Lints relaxed here is fine.
4. `deny.toml` — permissive-only allow-list (MIT / Apache-2.0 / BSD / Zlib /
   0BSD / Unicode-3.0).
5. `.github/workflows/ci.yml` — **add** Rust jobs beside the existing
   `cost-data` / `decisions-index` gates; do not remove them. Jobs: `fmt --check`,
   `clippy -D warnings`, `test`, `deny check licenses`, an MSRV check on 1.90,
   **and the lint red-proof** (compile a violating snippet, assert it FAILS).
6. `app.just` — replace the TODO stubs for `build` / `test` / `lint` / `typecheck`
   so they match AGENTS.md §6.

**Acceptance criterion 5 is the one to not skip:** a lint policy that has never
rejected anything is not a policy. CI must prove it bites.

## Out of Scope

- **Any decoding.** No TIFF walk, no tag model, no unpack. SPEC-003/004.
- **Any dependency.** See above.
- **`no_std` feature machinery, `rayon`, SIMD** — DEC-002 is unresolved.
- **Publishing to crates.io** — STAGE-004 excludes it from PROJ-001.
- **Reading SPIKE-001's code as an implementation.** It is on an unmerged branch
  and is deliberately discarded; `test-before-implementation` is why. The two
  measured facts you need from it are already transcribed above.

## Return Criteria — how to hand back

Before reporting done:

1. Every acceptance criterion in the spec is met, and you **ran** the gates rather
   than reasoning about them. Paste the actual command output.
2. Fill this file's `## Completion` and `handback:` block — **including a real
   `tokens_total`**. The orchestrator has no meter for you; an invented number is
   worse than none because it looks real in every rollup (DEC-013).
3. Set `handoff.status: completed` and the spec's `task.cycle: verify`.
4. Branch `feat/spec-001-crate-scaffold`, one PR, description per AGENTS.md §13
   (project / stage / spec / handoff / decisions / constraints / provenance / oracle).
5. Emit a `DEC-*` for any non-trivial choice you make — in particular if you
   conclude the MSRV should differ from 1.90, that is a decision with a record,
   not a silent edit.

⚠ The orchestrator will **reconcile your report against git and disk** before
accepting it (DEC-004 rule 1): commits actually present, files actually on disk,
gates actually run. This is not distrust of you specifically — it is the rule,
because truncated reports have claimed "done" with the push missing.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
