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
  id: HANDOFF-007
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-006

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
  status: completed
  tokens_total: 5121192
  estimated_usd: null
  duration_minutes: 15
  branch: feat/spec-006-allow-attribute-gate
  pr: null
  completed_at: 2026-08-20
  notes: "All six acceptance criteria met; both red-proof directions measured and pasted. The headline: with the spec's #[allow] planted on a pub fn in src/lib.rs (before the #[cfg(test)] module), BUILD 0 CLIPPY 0 FMT 0 TEST 0 MSRV 0 DENY 0 REDPROOF 0 and the new NO-ALLOW gate 101 with two E0453s at src/lib.rs:88 -- the hole reproduced, one gate seeing it. Honest tree: all eight 0. Also proved the inner #![allow] form (101) and, on the honest tree, that --all-targets goes 101 on the test module's legitimate allow, which is why the scope is --lib. Mechanism transcribed verbatim from the spec; no text search; scripts/lint-red-proof.sh and src/lib.rs both untouched. constraints.yaml:33 rewritten to name both jobs, state SCOPE: --lib only, and say plainly that neither job proves any code is panic-free -- only that the policy is intact and inescapable on the library; constraints-view.sh output byte-identical before and after. Three deviations, all recorded in the handback: (1) the branch already pointed at 412cb1b (SPEC-002's design commit) rather than main, so it was reset onto main dd4eb42 -- nothing lost, 412cb1b is still the tip of feat/spec-002-corpus-manifest-reader; (2) AGENTS.md §6 gained the command block, because §6 makes recipe<->block correspondence SPEC-001 acceptance criterion 8; (3) CI inlines the cargo invocation rather than calling just, because just is not on ubuntu-latest -- caught by executing the YAML run: block before commit, not by reading it. Ran in an isolated git worktree: another session was moving HEAD in the shared checkout during this cycle. tokens_total is REAL but not from /cost (a client-side slash command the assistant cannot execute): summed 53 deduplicated usage objects in this session's own transcript (~/.claude/projects/-Users-...-irradiance/e8f27d72-....jsonl). Composition: input 106 + output 39,205 + cache-write 106,729 + cache-read 4,975,152 (97.1% cache-read). FLOOR -- written before the session ends. Same method as SPEC-001's verify-1/verify-2 and build-2/build-3/build-4; NOT comparable to build-1's 197,940 (token-counts-not-comparable). Follow-ups, none blocking: src/lib.rs's module doc now reads as if the gap is still open (one sentence would close it, deliberately left to avoid touching the file the gate protects); --all-features was NOT added to the gate on purpose, but DEC-002's std-behind-a-feature proposal will make that a real question; and toolchain-brief.md's +stable = 1.97.0 has drifted to 1.98.0 on this host."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-007: Close the allow-attribute bypass in the panic-free gate

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-006` for the **build** cycle.

Close the one hole SPEC-001 could not: a single `#[allow]` exits the panic-free
policy. The mechanism is **decided and measured** — transcribe it.

## Context the Receiving Agent Needs

### The hole

Verified on the shipped crate: one attribute passes all seven gates while shipping
two panics on the public API, with no module involved.

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 { if v.is_empty() { panic!("e") } *v.first().expect("x") }
```

`DEC-009`'s red-proof **structurally cannot** see this — it mutates the crate root
and asserts the root's `#![deny]` bites; no `#![deny]` mutation test can observe
an `#[allow]` beneath it.

### The mechanism — measured at design, all three properties

```bash
cargo clippy --lib --quiet -- \
  -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
  -F clippy::panic -F clippy::arithmetic_side_effects
```

| run | measured |
|---|---|
| `#[allow]` planted | **101**, `E0453: allow(clippy::panic) incompatible with previous forbid` |
| honest tree | **0** |
| `--all-targets` | 101 — tests legitimately allow, which is why scope is `--lib` |

`-F` is `--forbid`: it cannot be re-allowed in source, and attempting it is a hard
**compiler** error. `--lib` excludes `#[cfg(test)]` modules and `src/bin/irr.rs`,
so the sanctioned exceptions need no special-casing.

### Do NOT reach for a text search

It is the obvious approach and it is a trap here. `src/lib.rs` contains attribute
text **inside its own module documentation**. The
`attribute-text-inside-doc-comments` signal is at **N=5** on SPEC-001 alone, and
every instance produced a wrong *answer* rather than an error — two false
negatives and one false green that shipped a panic past seven gates. The forbid
check does no text matching at all, which is most of why it was chosen.

### `--force-warn` looks right and is not

It *does* override `#[allow]` (measured: 2 warnings at the planted line), but
`-D warnings` **cannot promote a force-warn diagnostic to an error** — exit stays
**0**. Building on it would require parsing output, reintroducing the fragility
this design avoids.

## Expected Deliverables

1. A CI job in `.github/workflows/ci.yml` running the forbid check.
2. A `just` recipe in `app.just` for it, matching AGENTS.md §6.
3. **Red-proof evidence in the handback, both directions:**
   - `#[allow]` planted → non-zero, with the `E0453` lines pasted
   - honest tree → exit 0
4. `guidance/constraints.yaml:33`'s `enforcement:` corrected — it must state what
   is enforced **and its scope** (`--lib`), without overstating. F-4 was raised
   because the previous wording read as a broader guarantee than held.
5. All existing gates still green, output pasted.

## Out of Scope

- **`scripts/lint-red-proof.sh` — do not touch it.** `DEC-009`'s red-proof is
  sound for what it pins; this gate covers what it structurally cannot. They are
  complementary, and a fourth iteration of that script was explicitly rejected.
- Any decoding work, and SPEC-002's corpus reader (separate spec, no shared files).
- Broadening the claim beyond the `--lib` target.

## Return Criteria — how to hand back

1. Paste both red-proof directions and all existing gates.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share
   (see the `token-counts-not-comparable` signal).
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-006-allow-attribute-gate` off `main`; commit; do not merge.

The orchestrator will re-run the planted-`#[allow]` attack itself — it has the
exact reproduction.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-006-allow-attribute-gate` (reset onto `main` =
  `dd4eb42`; see *Deviations*). Commit: the single commit on this branch — its SHA is the branch tip; this handback ships inside that commit, so it cannot name itself. Not pushed, no PR — the
  handoff said commit and do not merge, and was silent on push.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** yes — all six, each measured below.
- **Worktree:** run in an isolated `git worktree`, not the shared checkout —
  another session was moving `HEAD` in the shared tree while this cycle was
  running (AGENTS.md §13, "one git worktree per concurrent session").

### What shipped

| # | Deliverable | Where |
|---|---|---|
| 1 | CI job `lint-policy-no-allow` | `.github/workflows/ci.yml` |
| 2 | `just lint-no-allow` | `app.just` |
| 3 | Red-proof, both directions | this handback, below |
| 4 | Corrected `enforcement:` | `guidance/constraints.yaml:33` |
| 5 | All gates green, pasted | this handback, below |
| — | §6 command block entry (the recipe↔§6 correspondence SPEC-001 made acceptance criterion 8) | `AGENTS.md` §6 |

The gate, transcribed from the spec's `## Failing Tests` verbatim:

```bash
cargo clippy --lib --quiet -- \
  -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
  -F clippy::panic -F clippy::arithmetic_side_effects
```

`src/lib.rs` is **untouched** — `git diff` against `main` shows no change to it.
No text search anywhere in the gate; `scripts/lint-red-proof.sh` was not opened,
edited, or run differently.

CI runs that cargo invocation **inline**, not via `just`: `just` is not on the
`ubuntu-latest` runner, and every other job in this workflow inlines its command
the same way (`lint-policy-red-proof` runs `./scripts/lint-red-proof.sh`, not
`just lint-red-proof`). Both surfaces were exercised — see below.

---

### Red-proof, direction 1 — attack planted → RED

Planted in `src/lib.rs` immediately **before** the `#[cfg(test)]` module
(placement matters; see *Drift* — appending after it trips
`clippy::items_after_test_module` and makes CLIPPY red for the wrong reason,
which would have manufactured the seven-green baseline):

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 {
    if v.is_empty() {
        panic!("e")
    }
    *v.first().expect("x")
}
```

**Every gate, with that planted.** This is the SPEC-006 claim in one table —
the hole reproduced, and exactly one gate seeing it:

```
=== ATTACK PLANTED — #[allow] on a pub fn in src/lib.rs, before the test module ===
BUILD      EXIT=0
CLIPPY     EXIT=0
FMT        EXIT=0
TEST       EXIT=0
MSRV       EXIT=0
DENY       EXIT=0
REDPROOF   EXIT=0
NO-ALLOW   EXIT=101      <-- the new gate
```

The `NO-ALLOW` output, verbatim (this is the exact YAML `run:` block extracted
from `.github/workflows/ci.yml` and executed with `bash`, so what is pasted here
is what CI runs — not a hand-typed approximation):

```
error[E0453]: allow(clippy::panic) incompatible with previous forbid
  --> src/lib.rs:88:9
   |
88 | #[allow(clippy::panic, clippy::expect_used)]
   |         ^^^^^^^^^^^^^ overruled by previous forbid
   |
   = note: `forbid` lint level was set on command line (`-F clippy::panic`)

error[E0453]: allow(clippy::expect_used) incompatible with previous forbid
  --> src/lib.rs:88:24
   |
88 | #[allow(clippy::panic, clippy::expect_used)]
   |                        ^^^^^^^^^^^^^^^^^^^ overruled by previous forbid
   |
   = note: `forbid` lint level was set on command line (`-F clippy::expect_used`)

For more information about this error, try `rustc --explain E0453`.
error: could not compile `irradiance` (lib) due to 2 previous errors
```

Exit **101**, two `E0453`s — bit-for-bit what design measured. `just
lint-no-allow` on the same planted tree also exits **101**, so the recipe and
the CI step agree.

**Variant B — the inner `#![allow]` form**, because acceptance criterion 1 names
both spellings and the outer one alone does not cover it:

```rust
mod sneaky {
    #![allow(clippy::unwrap_used)]

    pub fn take(v: &[u8]) -> u8 {
        *v.first().unwrap()
    }
}
```

```
error[E0453]: allow(clippy::unwrap_used) incompatible with previous forbid
  --> src/lib.rs:89:14
   |
89 |     #![allow(clippy::unwrap_used)]
   |              ^^^^^^^^^^^^^^^^^^^ overruled by previous forbid
   |
   = note: `forbid` lint level was set on command line (`-F clippy::unwrap_used`)

For more information about this error, try `rustc --explain E0453`.
error: could not compile `irradiance` (lib) due to 1 previous error
```

Exit **101**. Criterion 1 holds for `#[allow]` and `#![allow]` alike, at crate
root or inside a module — `-F` fires on the *attribute*, not on the code under
it, so neither planted function ever had to panic for the gate to see it.

### Red-proof, direction 2 — honest tree → GREEN

The negative control, on the tree exactly as committed (`src/lib.rs` restored
byte-identical to `main`; `git diff -- src/lib.rs` is empty):

```
=== HONEST TREE (this branch's working tree, attack removed) ===
BUILD      EXIT=0
CLIPPY     EXIT=0
FMT        EXIT=0
TEST       EXIT=0
MSRV       EXIT=0
DENY       EXIT=0
REDPROOF   EXIT=0
NO-ALLOW   EXIT=0        <-- the new gate, negative control
```

`cargo test --all-features`: `2 passed; 0 failed` (plus the two empty bin/doc
harnesses). `cargo deny check licenses`: `licenses ok`. `REDPROOF`:
`✓ lint policy red-proof: control clean (exit 0) → injection rejected (exit 101)
→ all five lints fired at the injected code, and still fire without CI's
-D warnings`.

Commands as run: `cargo build --release` · `cargo clippy --all-targets
--all-features -- -D warnings` · `cargo fmt --check` · `cargo test
--all-features` · `~/.cargo/bin/cargo +1.90.0 check --all-targets
--all-features` · `cargo deny check licenses` · `./scripts/lint-red-proof.sh` ·
the extracted CI `run:` block. Host `cargo` is Homebrew **1.97.1** with clippy
**0.1.97**; MSRV went through the rustup shim per the `+toolchain` trap.

### Criterion 2 — the sanctioned exceptions, unchanged

`--lib` is doing the work, with **no per-site special-casing**: the honest tree
above exits 0 while `src/lib.rs` still carries its five-lint `#[allow(...)]` on
the `#[cfg(test)] mod tests`. Measured the other way round, to show the scope
choice is load-bearing rather than incidental — the same command with
`--all-targets` on the **honest** tree:

```
error[E0453]: allow(clippy::unwrap_used) incompatible with previous forbid
  --> src/lib.rs:93:5
   |
93 |     clippy::unwrap_used,
   |     ^^^^^^^^^^^^^^^^^^^ overruled by previous forbid
   ... (one per lint, all five)
```

That is the test module's legitimate allow, and it is exactly why the scope is
`--lib`. `src/bin/irr.rs` is a different target and is likewise out of scope
(it carries no `#[allow]` today and, under `--lib`, never needs one).

### Criterion 5 — `guidance/constraints.yaml:33`

Rewritten to state what is enforced **and** the scope, and to stop short of
what neither job proves. It now: names both jobs and says the pair is the
guarantee; says the red-proof is silent about an `#[allow]` beneath the root
and that no `#![deny]` mutation test structurally can see one; names the forbid
gate, `E0453`, and that the escape hatch fails whether or not the code beneath
it panics; states **`SCOPE: the --lib target only`**, why (the two sanctioned
exceptions) and that it is the limit of the claim (no future example, bench or
second bin); and says plainly that neither job proves any code is panic-free —
only that the policy is intact and inescapable on the library, with actual
panic-freedom still resting on clippy, review, and the SPEC-003 fuzz targets
that do not exist yet. `scripts/constraints-view.sh` output is byte-identical
before and after (diffed), so the edit did not disturb the line-based parser.

---

### Cost self-report

- **Tokens (total):** **5,121,192** — real, but **not from `/cost`**.
- **Estimated USD:** null (no rate configured; every SPEC-001 session recorded
  `estimated_usd: null` too).
- **Duration (minutes):** ~15.
- **Source of the number:** summed the `usage` objects in this session's own
  transcript — the same data `/cost` derives from.
  `/cost` is a client-side slash command the assistant cannot execute as a tool.
  Transcript: `~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/e8f27d72-1432-4f9c-adfb-911429ca7728.jsonl`
  (deduplicated by `message.id`).
- **Composition:** input 106 + output 39,205 + cache-write 106,729 + cache-read 4,975,152 over 53 deduplicated assistant turns. **Cache-read share: 97.1%** — i.e. only ~146,040 tokens (2.9%) are fresh input + output.
- ⚠ It is a **FLOOR** — written before the session ends, so it excludes these
  final turns.
- ⚠ `token-counts-not-comparable`: this is the **same method** as SPEC-001's
  verify-1/verify-2 and build-2/build-3/build-4 figures and is comparable to
  those. It is **not** comparable to SPEC-001 build-1's 197,940, an
  Agent-result `subagent_tokens` number of unknown cache composition.

### Drift and new artifacts

- **New decisions emitted:** none. The mechanism was decided and measured at
  design (and in the spec's `## Failing Tests`); this cycle transcribed it. No
  new dependency, no `DEC-*` needed.
- **Deviations from spec:**
  1. **Branch reset onto `main`.** `feat/spec-006-allow-attribute-gate` already
     existed and pointed at `412cb1b`, the **SPEC-002** design commit — not at
     `main` (`dd4eb42`). The handoff says "branch off `main`", and
     `one-spec-per-pr` says one spec per branch, so the branch was re-pointed at
     `main` before any work. Nothing was lost: `412cb1b` is still the tip of
     `feat/spec-002-corpus-manifest-reader`, where it belongs.
  2. **`AGENTS.md` §6 gained the command block.** Not in the deliverables list,
     but the handoff asks for a recipe "matching AGENTS.md §6", and §6 states
     that recipe↔block correspondence is SPEC-001 acceptance criterion 8 — a new
     recipe without its §6 line would have broken an invariant this repo
     asserts. One command block, one clause in the intro sentence.
  3. **CI inlines the cargo command instead of calling `just`.** First draft ran
     `just lint-no-allow`; `just` is not installed on `ubuntu-latest`, so that
     job would have failed for a reason unrelated to the policy. Caught by the
     §12 behavioral pre-flight (extract the `run:` block from the YAML and
     actually execute it) before commit.
  4. Nothing else. `scripts/lint-red-proof.sh` untouched, `src/lib.rs`
     untouched, no decoding work, no SPEC-002 files.
- **Follow-up work identified:**
  - **`src/lib.rs`'s module doc is now incomplete, not wrong.** It says "What
    the proof does **not** establish: that code in a module carrying its own
    `#[allow(...)]` is covered." That remains true *of `lint-red-proof.sh`*,
    but a reader now infers the repo has no answer, when it does. One sentence
    naming `lint-policy-no-allow` would close it. Deliberately **not** done
    here: the handoff scoped this to "one gate, one CI job, one corrected
    sentence", and `src/lib.rs` is the file the gate protects.
  - **`--all-features` was NOT added to the gate**, on purpose — the spec's
    command is the measured artifact and build is not where to widen it. It is a
    no-op today (the crate has zero features), but `DEC-002` proposes putting
    `std` behind a default-on feature; the day a feature gates real code,
    `--lib` alone stops covering all of it. Worth a one-line spec then.
  - **`guidance/toolchain-brief.md` has drifted:** it records
    `~/.cargo/bin/cargo +stable` as **1.97.0**; it is now **1.98.0** on this
    host. The brief says to prune stale facts aggressively, and a build agent
    that trusts a stale version line wastes the loop the brief exists to save.
    Not edited here (out of scope), but it should be re-measured.
  - **Two live worktrees for the same repo were running concurrently** during
    this cycle (`HEAD` in the shared checkout moved between two reads). This
    cycle worked around it correctly with its own worktree; DEC-004 rule 2
    ("one sub-agent at a time; no interleaved tree ops") is being stretched by
    parallel SPEC-002/SPEC-006 delegation. Worth a signal if it recurs.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing about the mechanism; measuring it at design paid for itself, and
   build really was transcription. The one genuine ambiguity was where in
   `src/lib.rs` to plant the attack. Appending it at the end of the file trips
   `clippy::items_after_test_module` and turns the `-D warnings` gate red for a
   reason that has nothing to do with the `#[allow]` — which would have made the
   headline "seven green gates" table read `CLIPPY 101` and quietly understated
   the hole. The spec's snippet is correct; it just doesn't say *where*, and the
   difference is the whole demonstration.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two. (a) SPEC-001's acceptance criterion 8 (every recipe's command appears
   in AGENTS.md §6) is a standing repo invariant that any spec adding a recipe
   inherits, but it lives inside a shipped spec and in §6's prose, not in
   `constraints.yaml` — easy to miss when the deliverables list doesn't mention
   it. (b) The handoff said "a CI job" without noting that this workflow's jobs
   inline their commands because `just` is absent on the runner. Neither cost
   much here, but both are the kind of unstated local convention that costs a
   cold agent a loop.

3. **If you did this task again, what would you do differently?**
   — Run the all-gates-with-attack sweep *first*, before writing a line of CI or
   `just`. It is the measurement that defines the deliverable — "which gates see
   this?" — and it is what caught the `items_after_test_module` placement trap.
   Doing it first would have made the correct planting site obvious from the
   start rather than after a discarded run.
