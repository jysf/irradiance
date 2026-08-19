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
  id: HANDOFF-005
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5    # corrected; see .repo-context.yaml tier_map           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-18
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  tokens_total: 12193717 # REAL combined count — what cost-audit reads
  estimated_usd: null              # see notes — no verified list rate for claude-opus-5[1m]; §4's
                                   # no-cache-discount rule applied to a 96.9%-cache-READ total would
                                   # overstate real spend by 1-2 orders of magnitude (DEC-013 forbids
                                   # inventing one).
  duration_minutes: 35
  branch: feat/spec-001-crate-scaffold
  pr: null                         # committed locally; not pushed, not merged, per the return criteria
  completed_at: 2026-08-19         # YYYY-MM-DD
  notes: "DEC-009 transcribed, not redesigned: negative control + five lints + hardened prologue parser + no INJECT_AT=1 crash, plus a severity run that closes PL-3. All four DEC-009 Validation cases and PL-2/PL-3/PL-5 demonstrated with pasted output; before/after on the round-2 `//`-comment bypass measured at the same mutated tree (old script: seven green gates; new script: REDPROOF 1). Seven gates green on the honest tree. ⚠ ONE RESIDUAL STATED, NOT FIXED: the proof pins the policy at the CRATE ROOT only — a future module carrying its own `#![allow(...)]`/`#[allow]` is not covered. Harmless today (crate is lib.rs + bin), live from SPEC-003. Per the handoff I am reporting it rather than inventing a sixth mechanism. tokens_total is REAL but not from `/cost`: `/cost` is a client-side slash command the assistant cannot execute, so I summed this session's own transcript usage objects (~/.claude/projects/-Users-...-verify-spec-001/1851b505-....jsonl) — the same data `/cost` derives from. Composition: input 178 + output 129,380 + cache-write 247,536 + cache-read 11,816,613 (96.9% cache-read). It is a FLOOR: written before the session ends. Same method as build-2 (15,379,660) and verify-1/verify-2 — comparable to those three, NOT to build-1's 197,940 (`token-counts-not-comparable`). ⚠ Running `just handback-sync` exposed a data-corruption bug in `scripts/_lib.sh:301`: it truncates a quoted handback note at the first `#`, which wrote an unterminated YAML scalar and made this spec's front matter unparseable. Fixed (2 lines, one caller) and both affected sessions re-transcribed by the tool; details in the handback."
  synced_at: 2026-08-19
---

# HANDOFF-005: Crate scaffold: Cargo.toml, measured MSRV, panic-free lints, Rust CI

## Delegation Summary

Third build cycle on `SPEC-001` — the round that closes the lint-policy gate for
good, per **`DEC-009`** (which supersedes `DEC-007`, which superseded `DEC-006`).

Verify round 2 returned ⚠ PUNCH LIST with two P1s. **Both were reproduced
independently by the orchestrator**, and both produce *seven green gates with a
panic shipped in the library*.

## Context the Receiving Agent Needs

Read **`DEC-009` first** — it settles the design; do not redesign it.

### The failure, measured

A plain `//` comment is legal in a prologue. `find_injection_line`
(`scripts/lint-red-proof.sh:63-90`) handles blank, `//!` and `#![` — a `//` line
falls through and **becomes the injection point**, above the inner attributes.
`pub fn` there is a syntax error; clippy exits non-zero; and rustc **renders the
attribute's source span in the diagnostic**, so all three expected lint names
appear in the log with no lint firing.

Reproduced: `//` comment before `#![forbid]`, `#![deny(` → `#![allow(` at column 0,
plus `pub fn boom(v: &[u8], n: u8) -> u8 { v[0] + n }` and one using `.expect()`:

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ lint policy red-proof: ... rejected the injected violations (... all fired).
```

Every word of that success message is false.

**And `DEC-007` claimed this exact case was mitigated.** It is now marked
falsified in place. Do not trust a superseded DEC's reasoning.

### The fix — DEC-009, in order of importance

1. **NEGATIVE CONTROL.** Run the *same* clippy invocation on the **unmutated**
   copy first and require **exit 0**. If it is non-zero, fail with a message
   saying the toolchain/copy/crate is broken and nothing downstream is meaningful.
   *This is the one that closes the class* — every assertion so far inspected only
   the mutated run, so none could tell "failed for my reason" from "failed for any
   reason".
2. **All five lints.** `unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
   `arithmetic_side_effects` — each needs its own injected violation **and** an
   `EXPECTED_LINTS` entry. Today `panic` and `expect_used` are in neither, so
   deleting exactly those two from the policy passes everything (PL-2).
3. **Prologue parser** skips plain `//` comments, and refuses any injection point
   that is not strictly after the last inner attribute.
4. **`INJECT_AT=1` crashes** — `head: illegal line count -- 0`. Hit independently
   by both reviewer and orchestrator.

### The rest of the punch list

- Three artifacts assert a discrimination property the script does not yet have
  (the reviewer names them). Correct the **artifacts**, not just the script — a
  doc that overstates a guarantee is the same defect one level up.
- `deny` → `warn` currently passes. It must not.
- **Run `just handback-sync SPEC-001`** — build-2's 15,379,660 is un-synced, so
  `cost.totals` understates by ~74%.
- `guidance/constraints.yaml`'s `enforcement:` for `no-panics-on-untrusted-input`
  still reads *"fuzz targets…; clippy; review"*. **Now** is the round to add the
  red-proof — the reviewer was right that writing it earlier would have documented
  a guarantee that did not exist. It does after this change.

## Expected Deliverables

1. `scripts/lint-red-proof.sh` per `DEC-009`: control run, five lints, hardened
   prologue parser, no `INJECT_AT=1` crash.
2. **All four `DEC-009` Validation cases demonstrated in the handback**, with
   pasted output:
   - policy present → passes, all five lints named
   - policy deleted → fails
   - policy `deny`→`allow` → fails
   - `//` comment in the prologue → fails (or injects correctly); never passes
3. `guidance/constraints.yaml` `enforcement:` names the red-proof.
4. The three over-claiming artifacts corrected.
5. `just handback-sync SPEC-001` run; `cost.totals` correct.
6. All seven gates green, run for real, output pasted.

## Out of Scope

- Redesigning `DEC-009`. If you believe a fifth bypass exists, **say so in the
  handback** rather than inventing a sixth mechanism — DEC-009 names the
  compile-fail-harness alternative and why it needs its own decision.
- Settled: MSRV 1.90 · fuzz deferral to SPEC-003 · `[lints]` in `Cargo.toml` ·
  `core::` (verified) · `AGENTS.md` §7 (verified correct).
- The `signals.yaml` merge divergence — leave it.
- Any decoding work.

## Return Criteria — how to hand back

1. Paste output for all seven gates **and all four Validation cases**. The
   orchestrator will re-run the `//`-comment bypass itself; it has the exact
   reproduction.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum your transcript usage objects and **say that is what you
   did**, with your cache-read share (see the `token-counts-not-comparable`
   signal).
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

**This gate has now been wrong three times, and each author believed it closed.**
If something feels unproven, say so in the handback rather than shipping a fourth
confident mechanism.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-001-crate-scaffold` — committed locally, **not
  pushed, not merged** (return criterion 4). No PR.
- **Completed at:** 2026-08-19
- **All acceptance criteria met?** **Yes**, with one limit stated rather than
  papered over — see *A residual I did not fix* below. AC 4 and AC 5 now hold
  for **5/5** of the policy they describe, not 3/5.
- **For `verify`:** n/a — this is the build cycle.

---

## What changed

`DEC-009` transcribed, not redesigned. `scripts/lint-red-proof.sh` now runs
clippy over a temp-dir copy of the crate **three** times:

| run | invocation | required | closes |
|---|---|---|---|
| **control** | `clippy --all-targets --all-features -- -D warnings`, **unmutated** | **exit 0** | PL-1 (the class) |
| **mutation** | same invocation, **injected** | non-zero + all five lints fired *at the injected lines* | PL-1, PL-2 |
| **severity** | same, **without** `-D warnings` | non-zero + all five lints fired | PL-3 |

Five assertions, each with a distinct job:

0. clippy is available (kept from round 2 — a stub `cargo` otherwise reports green).
1. **CONTROL** — the unmutated copy is clean. *This is the one that closes the
   class.* With it, a red from the mutated run is attributable to the injection
   **by construction**; no grep over rendered output can establish that.
2. the mutated copy is rejected.
3. all **five** policy lints fired — matched on clippy's `index.html#<lint>`
   help line, **not** the bare lint name. The help line is emitted only when a
   lint *fires*; the bare name is the text of the policy itself and lands in the
   log whenever rustc renders that span, which is exactly how round 2's false
   green satisfied three name checks with zero lints firing.
4. the diagnostics are **located inside the injected line range** — the names
   came from the injected code, not from elsewhere in the file.
5. **severity** — still rejected without CI's blanket `-D warnings`, which is
   what makes the block `deny`-level in the library rather than `warn`-level
   promoted by a CI flag.

**Prologue parser.** Rewritten to anchor on the **last inner attribute**
(`find_last_inner_attr_end`) instead of "the first line that isn't prologue".
Injecting at `last_attr_end + 1` is legal Rust by construction and is *strictly
after* the last `#![...]` by construction — so `//`, `//!` and `///` lines
anywhere in the prologue can no longer move the injection, and a shape the
parser does not understand terminates the scan and dies loudly instead of
relocating the injection above the attributes. A structural post-check re-parses
the **mutated** file and refuses to continue if the prologue moved.

**`INJECT_AT=1`** is now structurally unreachable (`last_attr_end ≥ 1` ⇒
`INJECT_AT ≥ 2`) and additionally asserted with a message, so `head -n 0` cannot
be reached. ⚠ I did **not** re-test the reviewer's unverified GNU/BSD `head -n 0`
divergence on Linux — it no longer matters in either direction, because the
input that produced it can no longer occur.

**`EXPECTED_LINTS` is hard-coded on purpose**, with the reason in the script:
deriving it from `src/lib.rs` would make the proof follow the policy instead of
pinning it — deleting a lint would delete its own expectation, which *is* PL-2.

Runtime: **0.53 s** for all three clippy runs (tiny crate; `DEC-009`'s
cost objection does not bite here).

---

## Seven gates — honest tree, run for real

```
BUILD     EXIT=0        cargo build --release
CLIPPY    EXIT=0        cargo clippy --all-targets --all-features -- -D warnings
FMT       EXIT=0        cargo fmt --check
TEST      EXIT=0        cargo test --all-features
MSRV      EXIT=0        ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
DENY      EXIT=0        cargo deny check licenses
REDPROOF  EXIT=0        ./scripts/lint-red-proof.sh
```

`shellcheck -x scripts/lint-red-proof.sh` — clean. No `irradiance-red-proof.*`
temp dir left behind; the working tree is never mutated.

---

## The four `DEC-009` Validation cases

Every case ran in a throwaway `git worktree` off `47579ab` with the new script
and the new `src/lib.rs`; the real working tree was never mutated. The panicking
function shipped in cases 2, 3 and 4a is the reviewer's own:

```rust
/// Shipped in the LIBRARY. Two panics on attacker-influenced input.
pub fn tag_len(b: &[u8]) -> usize {
    if b.len() < 4 {
        panic!("truncated tag header");
    }
    let n: Option<usize> = b.first().map(|v| usize::from(*v));
    n.expect("first byte present")
}
```

### 1. policy present → proof **passes**, all five lints named

```
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=0

• clippy is present: clippy 0.1.97
• injection point: src/lib.rs line 51 (strictly after the last inner attribute, which ends on line 50)
• control run: the UNMUTATED copy, exact CI invocation — this MUST pass:
✓ control: unmutated copy is clean (clippy exit 0). A red below is now attributable to the injection.
• mutation run: the same invocation on the injected copy — this MUST fail:
  lint fired: arithmetic_side_effects        <- index.html#arithmetic_side_effects
  lint fired: indexing_slicing               <- index.html#indexing_slicing
  lint fired: unwrap_used                    <- index.html#unwrap_used
  lint fired: expect_used                    <- index.html#expect_used
  lint fired: panic                          <- index.html#panic
• diagnostics located inside the injected block: 4 distinct lines in src/lib.rs:51-76
• severity run: the injected copy WITHOUT CI's -D warnings — the LIBRARY's own deny must still reject it:
      --> src/lib.rs:34:5      <- `the lint level is defined here`, resolving to
      --> src/lib.rs:35:5         the LIBRARY's own block, not a header of the
      --> src/lib.rs:36:5         proof's making
      --> src/lib.rs:37:5
      --> src/lib.rs:38:5
✓ lint policy red-proof: control clean (exit 0) → injection rejected (exit 101) → all five
  lints fired at the injected code, and still fire without CI's -D warnings (exit 101).
  src/lib.rs's own #![deny(...)] is what rejected them.
```

**All five lints named — `panic` and `expect_used` included.** That is the half
of AC 4/AC 5 that did not exist before this round.

### 2. policy deleted → proof **fails**

Col-0 `#![deny(...)]` block removed (targeting column 0, not the `//!` prose),
`tag_len` shipped.

```
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1
  ERROR: the lint policy did NOT reject the injected violations (clippy exited 0) — and the
  control run above was clean, so this is the policy's fault and nothing else. src/lib.rs's
  `#![deny(...)]` block is missing, weakened, or not applying. This is exactly the
  manufactured-confidence failure oracle-must-be-shown-red exists to catch.
```

### 3. policy `deny` → `allow` → proof **fails**

```
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1
  ERROR: the lint policy did NOT reject the injected violations (clippy exited 0) — and the
  control run above was clean, so this is the policy's fault and nothing else. [...]
```

### 4a. the round-2 bypass **verbatim** → proof **fails**

`// Lint policy: see DEC-009.` inserted before `#![forbid(unsafe_code)]`, plus
col-0 `#![deny(` → `#![allow(`, plus `tag_len`. This is the exact tree that
produced seven green gates in round 2.

```
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1
  ERROR: the lint policy did NOT reject the injected violations (clippy exited 0) — and the
  control run above was clean, so this is the policy's fault and nothing else. [...]
```

Note *which* assertion caught it: the injection now lands **correctly** (after
the attributes), so there is no syntax error, no rendered span, and clippy exits
**0** — the honest signal that the policy is `allow`. The comment stopped
mattering.

### 4b. `//` comment in the prologue, policy **intact** → proof **passes**, injecting correctly

`DEC-009` allows either outcome here ("fails, *or injects correctly*; never
passes [falsely]"). It injects correctly:

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ ... control clean (exit 0) → injection rejected (exit 101) → all five lints fired at the
  injected code, and still fire without CI's -D warnings (exit 101).
```

### Before / after, same mutated tree — the reproduction and the fix

Round-2 `src/lib.rs` and round-2 script from `47579ab`, `//`-comment bypass applied:

```
===== OLD script (HEAD 47579ab) =====
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected violations
  (clippy exit 101; clippy::indexing_slicing clippy::arithmetic_side_effects clippy::unwrap_used all fired).
                                                    ^ every word of this is false

===== NEW script, IDENTICAL tree =====
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 1
  ERROR: the lint policy did NOT reject the injected violations (clippy exited 0) [...]
```

---

## The rest of the round-2 punch list

### PL-3 (P2) — `deny` → `warn` now **fails**

Policy downgraded to `#![warn(` at column 0, tree otherwise untouched. Caught by
the severity run — the assertion the control alone cannot make (the reviewer said
so explicitly, and was right):

```
BUILD     EXIT=0
CLIPPY    EXIT=0     <- still green: CI's -D warnings promotes them back
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1
  ERROR: without CI's `-D warnings`, clippy exited 0 on the injected violations. The five
  lints are present but NOT at `deny` level — they are `warn`, and the only thing making
  them bite is a CI flag. src/lib.rs claims they are `deny`-level; a consumer running plain
  `cargo clippy` or `cargo build` would see nothing. Restore `#![deny(...)]`.
```

### PL-2 (P1) — dropping the two unexercised lints now **fails**

`clippy::panic` and `clippy::expect_used` deleted from the policy, `panic!()` and
`.expect()` shipped in the library — the attack that passed everything in round 2:

```
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1
  ERROR: mutation run: clippy failed, but the expected lint(s) clippy::expect_used
  clippy::panic never fired (no `index.html#<lint>` help line for them). A non-zero exit is
  not the proof, and neither is the lint's NAME appearing in the log — rustc prints the
  policy's own source text. [...]
```

### PL-5 (P3) — SPDX `//` header on line 1 no longer crashes

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ ... control clean (exit 0) → injection rejected (exit 101) → all five lints fired [...]
```

No `head: illegal line count -- 0`. The parser anchors below the attributes, so
line-1 content is irrelevant.

### PL-4 (P2) — the three over-claiming artifacts, corrected

- **`src/lib.rs`** (module doc) — rewritten. It now describes the control, the
  five lints, the severity run, and the four ways to break it; and it states
  what the proof does **not** establish (module-scoped `#[allow]`). It no longer
  claims that checking lint names discriminates a real rejection.
- **`.github/workflows/ci.yml`** (`lint-policy-red-proof` comment) — rewritten,
  naming the negative control as the load-bearing assertion and recording *why*
  (the rendered-span false green), so the next reader cannot re-derive the wrong
  mitigation.
- **`decisions/DEC-007`** — its *Consequences* was already marked falsified in
  place by the design cycle. Its **Revisit** line repeated the same false claim
  ("the symptom will be a loud missing-lint failure, not a silent pass") and was
  not marked; I added the marker there too. Superseded, not rewritten.

Two more artifacts carried the same stale reference and were corrected for
consistency (beyond the reviewer's three — disclosed): **`app.just`**'s
`lint-red-proof` comment and **`AGENTS.md`** §5/§6/§7 (`DEC-007` → `DEC-009`).

### PL-6 (P3) — cost sync

`just handback-sync SPEC-001` run; see *Cost self-report*.

### PL-7 (P3) — `decisions-audit --changed` blind on a committed branch

Confirmed still true; not mine to fix (tooling, and the reviewer filed it as
`process-debt`). I ran it **before** committing, where it does work:

```
⚠ DEC-009 — The red-proof needs a negative control, and must exercise every lint
      re-read this decision before committing; your change touches:
        .github/workflows/ci.yml
        scripts/lint-red-proof.sh
        src/lib.rs
```

`src/lib.rs` only appears there because I added it to `DEC-009`'s
`affected_scope` (see *Deviations*).

---

## The constraint's `enforcement:` field

`guidance/constraints.yaml`, `no-panics-on-untrusted-input`. It read *"fuzz
targets from STAGE-001 onward; clippy; review"* — which named a mechanism that
does not exist yet, a mechanism that *is* the thing being enforced, and one that
is not mechanical. It now names the red-proof **first**, says it is the only
mechanical enforcement today, says it is load-bearing, and says what happens if
it is deleted:

```yaml
enforcement: "scripts/lint-red-proof.sh (CI job `lint-policy-red-proof`) — the ONLY
  mechanical enforcement today, and load-bearing: it pins the five-lint #![deny(...)]
  policy on the crate root by mutation test with a negative control (DEC-009), and fails
  if the block is deleted, weakened to allow, downgraded to warn, or loses a single lint.
  Do not delete that script or its CI job without replacing the guarantee. Plus: clippy
  (which IS that policy); review; fuzz targets from SPEC-003 onward (not yet present —
  AGENTS.md §12 bar 2)."
```

The reviewer's reflection #2 was that nothing in the repo recorded that the
script and its CI job are load-bearing, so a future spec could delete them as
cleanup with no gate objecting. That sentence is what fixes it, and it is now
true — which is why it belongs in this round and not the last one.

---

## ⚠ A defect the instructed command caused — found, fixed, disclosed

`just handback-sync SPEC-001` **corrupted the spec's front matter into invalid
YAML**, and it would have done so silently.

`scripts/_lib.sh:301` (`get_handback_field`) ends with

```awk
sub("[[:space:]]*#.*$", "")
```

which strips a trailing YAML comment. The handback template puts one after most
fields (`status: completed  # completed | blocked | rejected`), so the strip is
needed — but it also fires **inside the quoted `notes:` string**. Any handback
note containing a `#` is truncated at that character, and `handback-sync.sh:112`
then writes

```yaml
      notes: "…rustc renders the `
```

— an **unterminated double-quoted scalar**. Two of the five cost sessions landed
that way: verify-2's (`#![deny(...)]` in its note) and my own (`#![allow(...)]`).
`SPEC-001`'s entire front matter stopped parsing. Caught only because I
YAML-parsed the spec to check `task.cycle` afterwards; nothing in the repo does
that automatically, and `just cost-audit` still reported ✓ against the broken
file.

**Fixed** — strip the trailing comment only from an *unquoted* scalar:

```awk
if ($0 !~ /^"/ && $0 !~ /^\x27/) sub("[[:space:]]*#.*$", "")
```

Then: removed the two truncated sessions, reset those two handbacks'
`synced_at` to `null`, and re-ran `just handback-sync SPEC-001` so both entries
were re-transcribed **by the tool**, not hand-written. Verified after:

```
FRONT MATTER PARSES OK
cost.totals: {'tokens_total': 41017417, 'estimated_usd': 0.0, 'session_count': 5}
  build      197940  notes  819 chars
  verify    5242951  notes  800 chars
  build    15379660  notes 1135 chars   <- build-2, previously missing entirely
  verify    8003149  notes 1634 chars   <- full length, matches HANDOFF-004 exactly
  build    12193717  notes 1287 chars   <- full length, matches HANDOFF-005 exactly
```

**Why I fixed a template-managed script when round 1's precedent was to file
one.** Round 1 filed `handback-sync.sh:105`'s hard-coded `interface: other`
because it writes *wrong metadata*. This one writes *invalid YAML*, on the exact
command the handoff instructed me to run, and it re-breaks on the next handback
that mentions an attribute — which, in a Rust repo whose whole subject is
`#![deny(...)]`, is most of them. Leaving the spec unparseable was not an option,
and hand-authoring cost entries is what the reviewer explicitly warned against.
Two lines, one caller, disclosed here.

⚠ **`get_handoff_field` and `get_spike_field` (`_lib.sh:283`, `:315`) have the
identical bug** and I did **not** touch them: their callers only read short enum
scalars (`cycle`, `to_agent`, `mode`, `outcome`), so no live path corrupts data
today. Filed, not fixed — flag it if you disagree.

## A residual I did not fix — say it, don't patch it

The handoff asks me to **say so** rather than invent a sixth mechanism if I think
a fifth bypass exists. I do, and here it is:

**The proof pins the policy at the CRATE ROOT only.** The injection lands in
`src/lib.rs` after its last inner attribute. A future module — `src/ifd.rs` with
`#![allow(clippy::indexing_slicing)]` at its top, or a `#[allow(clippy::panic)]`
on one function — is **not covered**: the proof still passes, and panicking code
ships from that module.

- **Not live today.** The crate is `src/lib.rs` + `src/bin/irr.rs`, and the only
  `#[allow]` is scoped to `#[cfg(test)] mod tests`, which is legitimate.
- **Live from `SPEC-003`**, which is the first spec that adds modules — the IFD
  reader, on the most hostile input in the project.
- It is **not** a defect in `DEC-009` (which is about the control, and the
  control is right); it is the *next* boundary out, the same way the reviewer's
  reflection found `constraints.yaml` was the boundary out from the script.
- The cheap shape of a fix, for whoever decides: a grep gate that no
  `allow(` of the five lint names appears outside `#[cfg(test)]` and
  `src/bin/`. That is a policy decision about the crate's shape, not an
  implementation detail of this script, so I have **not** built it.

**Two smaller things I want on the record rather than in the script:**

1. **The control can mask the specific diagnosis.** Measured: `deny`→`warn`
   *plus* a shipped panicking function makes the honest tree fail its own
   `-D warnings` gate, so the control fires first and the proof says *"the
   control run failed — fix this first"* rather than *"the policy is `warn`"*.
   It still fails, loudly and correctly, and pointing at the upstream breakage
   first is the right order — but the message is one step upstream of the cause.
   I improved it to name the command to run. Inherent to having a control.
2. **Assertion 3 still greps a log.** It is sound only because assertion 1
   (control) and assertion 4 (span location) bracket it. The script says so, and
   says the one thing that would reintroduce the round-2 defect: **never put a
   literal clippy help URL in `src/lib.rs`'s prose.** If someone does, a rendered
   span could supply the fragment — though it would still have to satisfy the
   in-range span count, which I judge closes it.

---

## Cost self-report

- **Tokens (total):** **12,193,717** — real, but **not from `/cost`**.
- **Estimated USD:** `null`. No verified list rate for `claude-opus-5[1m]`, and
  §4's "no cache discount" rule applied to a 96.9%-cache-**read** total would
  overstate actual spend by one to two orders of magnitude. `DEC-013` forbids
  inventing one.
- **Duration (minutes):** 35 (first→last transcript timestamp, plus these turns).
- **Source of the number:** the `usage` objects in this session's own transcript
  (`~/.claude/projects/…-verify-spec-001/1851b505-….jsonl`), summed over
  94 usage objects — the same data `/cost` derives from. `/cost` is a
  client-side slash command I cannot execute as the assistant, so I read its
  source data directly rather than reporting null.
  **Composition: input 178 · output 129,380 · cache-write 247,536 ·
  cache-read 11,816,613 — cache-read is 96.9% of the total.** It is a **floor**:
  written before the session ends.

⚠ **Comparability.** Same method as build-2 (15,379,660), verify-1 (5,242,951)
and verify-2 (8,003,149) — those four *are* comparable. Build-1's **197,940** came
from an `Agent`-result `subagent_tokens` figure of unknown cache composition and
is **not**. Fifth data point on `token-counts-not-comparable`; four of five now
agree on method, which strengthens the reviewer's suggestion to restate build-1
rather than keep annotating everything else.

**`just handback-sync SPEC-001` run** (PL-6). It transcribed the two handbacks
that were still `synced_at: null` — **build-2's 15,379,660** (the largest cycle
on the spec, missing from its own total) and **verify-2's 8,003,149** — plus this
one. `cost.totals` before: `5,440,891` over 2 sessions. After: **`41,017,417` over 5
sessions** — the spec's recorded cost was understating by 87%, not the ~74% the
reviewer estimated from build-2 alone, because verify-2 was un-synced too. I did not hand-append anything; the script is idempotent via `synced_at`.

Still open and **not** mine to fix: the **design** cycle has no `cost.sessions`
entry (§4 wants `null`-with-note), and `scripts/handback-sync.sh:105` hard-codes
`interface: other`.

---

## Drift and new artifacts

- **New decisions emitted:** **none.** Everything here is `DEC-009`
  transcribed. The severity run (assertion 5) closes PL-3, which `DEC-009` does
  not name in its Decision but which the handoff put in scope; it is one more
  invocation of a command already being run, not a new mechanism, so I judged it
  an implementation detail rather than a decision. If the reviewer disagrees,
  the honest place for it is an amendment to `DEC-009`'s *Decision*, not a
  `DEC-010`.
- **Deviations from spec / handoff scope** — all disclosed:
  1. **`DEC-009`'s `affected_scope` gained `src/lib.rs`.** `DEC-007` listed it
     (round-1 PL-3, which verify round 2 confirmed closed); `DEC-009` dropped
     it, which made `just decisions-audit --changed` silent on `src/lib.rs` —
     the file this decision's prose lives in and the file the proof mutates.
     Metadata correctness per AGENTS.md §15 build step 4, not a redesign.
  2. **Two artifacts beyond the reviewer's three** — `app.just` and `AGENTS.md`
     §5/§6/§7 — updated `DEC-007` → `DEC-009`. Same class as PL-4: a stale
     pointer to a superseded decision whose reasoning the handoff says not to
     trust.
  3. **`DEC-007`'s *Revisit* line** marked falsified as well as its
     *Consequences*. The reviewer cited `:117-119` (Consequences); the same
     false claim is restated four lines further down.
  4. **`scripts/_lib.sh:301` fixed** — see *A defect the instructed command
     caused* above. Template-managed script, not on the handoff's list; I judged
     an unparseable spec worse than a two-line excursion. Reverse it if that was
     the wrong call — the spec then needs its two notes hand-repaired instead.
  5. **`SPEC-001`'s `references.decisions`** went `[DEC-006]` →
     `[DEC-006, DEC-007, DEC-009]`. It named only the first of three decisions
     that govern this spec (§10 cross-reference rules).
- **Follow-up work identified:**
  - **Module-scoped `allow` is uncovered** (above). Decide before `SPEC-003`
    lands the first module. This is the one I would attack next.
  - **Nothing YAML-parses the spec front matter.** `just cost-audit` reported ✓
    against a `SPEC-001` whose front matter was invalid YAML. A one-line parse
    check in `cost-audit` (or `decisions-audit`) would have caught it at the
    moment it happened rather than one careful read later. Same shape as this
    spec's whole subject: a gate that cannot detect the failure it exists to
    detect.
  - `get_handoff_field` / `get_spike_field` carry the same comment-stripping bug
    (`_lib.sh:283`, `:315`); harmless today, live the moment either is used on a
    free-text field.
  - The reviewer's four open items stand: `decisions-audit --changed` blindness
    (**PL-7**, `process-debt`), the design cycle's missing cost session,
    `handback-sync.sh:105`'s hard-coded `interface: other`, and restating
    build-1's 197,940 by the transcript method.
  - **The doc-comment collision is now N=4, not N=3.** It bit me inside this very
    cycle: my attack harness anchored on `#[cfg(test)]`, matched the
    **`//!` prose** occurrence at `src/lib.rs:16` instead of the real module, and
    inserted a function into the middle of the doc block — producing a bogus
    seven-gate result I nearly pasted. Caught by printing the file after editing,
    exactly as the reviewer described doing. The reviewer's proposed rule is
    right and I would make it explicit: **every mechanism that pattern-matches
    this library's source must anchor at column 0.** Worth the `lesson` signal.

---

## Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear; `DEC-009` was specific enough to transcribe. The one
   thing the handoff put in scope that `DEC-009` does *not* cover is PL-3
   (`deny`→`warn`) — the DEC's Validation lists `allow`, not `warn`, and its
   Decision does not mention severity. I built the severity run for it and have
   flagged the gap above rather than quietly widening the DEC. If a fourth
   design ever happens, the lesson is that the DEC's *Validation* list and the
   handoff's *scope* list must be the same list.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The one now written into `constraints.yaml`: that
   `scripts/lint-red-proof.sh` and its CI job **are** the enforcement of a
   `blocking` constraint. It existed nowhere for three rounds. And the next one
   of that shape is already visible — nothing records that the policy is pinned
   at the crate root only, so the first module to carry its own `#[allow]` will
   be outside a guarantee that everything in the repo currently states without
   qualification. I have qualified it in `src/lib.rs`; it deserves a decision.

3. **If you did this task again, what would you do differently?**
   — Write the negative control *first*, before touching the parser or the lint
   list. I did the parser first out of habit (it was the visible bug), and it
   cost me the ordering insight: once the control was in, case 4a stopped being
   about comments at all — the injection lands correctly and clippy honestly
   exits 0. The parser hardening is still right, but it was the *second*-order
   fix, and three rounds of this gate have each fixed the visible bug and left
   the class. The general form: **when a test can pass for the wrong reason,
   fix the attribution before fixing the symptom.**
