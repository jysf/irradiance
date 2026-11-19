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
  id: HANDOFF-003
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5             # from tier_map.<cycle> — the executing agent
                                   # ⚠ CORRECTED BY THE EXECUTOR. This was written
                                   # `claude-sonnet-5` from tier_map.build, but the cycle
                                   # actually ran on claude-opus-5[1m] in a Claude Code CLI
                                   # session. handback-sync copies THIS field into the spec's
                                   # cost session `agent:`, so leaving it would have booked
                                   # 15.4M Opus tokens against Sonnet and corrupted
                                   # `just calibration`. See DEC-004 rule 3.
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
  tokens_total: 15379660           # REAL combined count — what cost-audit reads
  estimated_usd: null              # see notes — no verified list rate for claude-opus-5[1m], and a
                                   # no-cache-discount rate applied to a 97.5%-cache-read total would
                                   # overstate real spend by 1-2 orders of magnitude (DEC-013).
  duration_minutes: 13
  branch: feat/spec-001-crate-scaffold
  pr: null                         # committed locally; not pushed, not merged, per the return criteria
  completed_at: 2026-08-18         # YYYY-MM-DD
  notes: "Second build cycle (punch-list round) on SPEC-001. All six punch-list items closed; both red-proof directions demonstrated and pasted in the handback — (a) policy present + injection -> clippy 101 with all three lint names and the level resolving to src/lib.rs:34/36/38; (b) policy REMOVED + injection -> clippy 0, proof exits 1. Two further attacks also pasted: clippy-unavailable (PL-2) and a partial policy weakening (assertion 3's teeth). All seven gates green on the tree as committed. tokens_total is REAL but not from `/cost`: `/cost` is a client-side slash command the assistant cannot execute, so I summed the `usage` objects in this session's own transcript (~/.claude/projects/-Users-...-verify-spec-001/bc36989d-....jsonl) — the same data `/cost` derives from. Composition: input 248 + output 95,602 + cache-write 284,829 + cache-read 14,998,981. It is a FLOOR: written before the session ends. ⚠ NOT comparable to the first build's 197,940 (an Agent-result subagent_tokens figure of unknown cache composition); it IS comparable in kind to verify's 5,242,951. Third data point on the process-debt signal the verifier filed."
  synced_at: 2026-08-19
---

# HANDOFF-003: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` back to `claude-sonnet-5`
(implementer) for a **second build** cycle — the punch-list round.

The first build was honest and in scope. Verify returned ⚠ PUNCH LIST, not ❌,
and the two P1s were **reproduced independently by the orchestrator** before
being accepted. Your job is the fix round, not a rewrite.

## Context the Receiving Agent Needs

Branch `feat/spec-001-crate-scaffold`. Read the verify handback in
`HANDOFF-002-verify-*.md` for the full punch list, and **`DEC-007`**, which
settles the P1-1 design so you do not have to.

### P1-1 — the red-proof proved the wrong thing (DEC-007 supersedes DEC-006)

Delete the `#![deny(...)]` block from `src/lib.rs`, add
`pub fn read_u8(b: &[u8], at: usize) -> u8 { b[at] + 1 }`, and **all seven gates
go green** — a shipped panic on untrusted input. The old proof's own output says
why: `the lint level is defined here --> tests/lint_policy_red.rs:14`, the
snippet's header, never the library's.

**The design is decided and already probed — transcribe it, don't redesign it.**
`DEC-007` Option C: copy the crate to a temp dir, inject violating functions into
the *copied* `src/lib.rs` right after its attribute prologue, run clippy there.
Working tree is never touched, so no `trap`-based restore is needed.

Injection point that works (verified): skip blank lines, `//!` doc comments, and
`#![...]` blocks (tracking bracket depth); insert before the first real item.
⚠ A naive `max()` over lines ending `)]` was tried first and landed **inside the
test module's `#[allow(...)]`**, silently suppressing two of the three lints. It
was caught only because the lint names were checked — which is why assertion 3
below is not optional.

Assert **all three**:
1. clippy actually ran (`cargo clippy --version` succeeds first)
2. clippy exited non-zero
3. all of `arithmetic_side_effects`, `indexing_slicing`, `unwrap_used` appear in
   its output

Proven both directions before the DEC was written: policy present → exit 101,
three lints fire, level resolves to `src/lib.rs:31`; policy **removed** → exit 0,
so the assertion fails and the removal is caught. **Your change must demonstrate
both directions**, not just the first.

Delete `tests/lint_policy_red.rs.disabled` — DEC-007 supersedes that mechanism.

### P1-2 — the proof greened when clippy was unavailable

With a stub `cargo` on `PATH` the script printed *"✓ lint policy red-proof: the
violating snippet failed to compile as expected"* and exited 0. Assertion 1 above
fixes this. DEC-006's own Validation listed the three error identities; the script
just never implemented them.

### P2/P3 — the rest of the punch list

- `DEC-006`'s `affected_scope` omits `src/lib.rs`, so `decisions-audit --changed`
  cannot surface the drift it predicts. `DEC-007` already scopes this correctly;
  no action beyond leaving DEC-006 superseded.
- **`use std::fmt` / `impl std::error::Error` are gratuitous** — `core::fmt`
  compiles clean on the declared MSRV. `DEC-002` (still `proposed`) says do not
  foreclose `no_std`, and the spec's Non-Goals said so explicitly. Switch to
  `core::`. Do **not** add feature machinery — that is still DEC-002's call.
- `AGENTS.md` §6's command block omits `cargo fmt --check` and the red-proof,
  though `just lint` / `just lint-red-proof` run them. Make §6 match reality.
- `scripts/lint-red-proof.sh` is CWD-relative — resolve paths from the script's
  own location. Its temp file was not gitignored; the copy-to-temp-dir design
  should remove that concern, but confirm.
- One small inaccuracy in the `AGENTS.md` edits, per the reviewer — find and fix.

## Expected Deliverables

1. `scripts/lint-red-proof.sh` rewritten per DEC-007, with all three assertions.
2. `tests/lint_policy_red.rs.disabled` deleted.
3. **Evidence, in the handback, of BOTH directions** — the proof rejecting the
   injection with the policy present, and the proof *failing* when the policy is
   removed. Paste both outputs. This is the acceptance criterion.
4. `core::fmt` in place of `std::fmt`; `core::error::Error` or an equivalent that
   holds on MSRV 1.90 — **measure it, do not assume**.
5. `AGENTS.md` §6 corrected; the one inaccuracy fixed.
6. Path-independent script; temp files clean.
7. All seven gates green afterwards, run for real, output pasted.

## Out of Scope

- Re-litigating DEC-007's design. It was probed both directions before adoption.
- The MSRV number, the cost entries, and the fuzz-job omission — all settled.
- `[lints]` in `Cargo.toml` — considered and rejected in DEC-007 (it applies to
  every target, including the ones deliberately allowed to `unwrap`).
- Any decoding work.

## Return Criteria — how to hand back

1. Paste real output for all seven gates **and** both red-proof directions.
2. Fill `## Completion` and the `handback:` block. For `tokens_total`: if you are
   a sub-agent with no `/cost`, leave it `null` **and say so** — the orchestrator
   fills it from the invocation metadata (DEC-013). Never invent a number. If you
   are a CLI session with `/cost`, use the real figure and note how you obtained
   it — see the `token-counts-not-comparable` signal.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

The orchestrator reconciles against git and disk, and **will re-run the
policy-removal attack itself**. It has been run twice already; a fix that does not
survive it is not a fix.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-001-crate-scaffold` — committed locally, **not
  pushed, not merged** (per this handoff's return criteria). No PR.
- **Completed at:** 2026-08-18
- **All acceptance criteria met?** **yes.** Every deliverable in
  `## Expected Deliverables` is done and measured. Both red-proof directions are
  pasted below; that was the stated acceptance criterion.
- **For `verify`:** N/A — this is a build cycle.

---

## THE ACCEPTANCE CRITERION: both directions, measured

### (a) Policy PRESENT + injection → clippy exits 101, all three lints named

`./scripts/lint-red-proof.sh` on the tree as committed:

```
• clippy is present: clippy 0.1.97
• injection point: src/lib.rs line 41 (immediately after the attribute prologue)
• running clippy on a mutated copy of the crate — this MUST fail:
    Checking irradiance v0.1.0 (/private/var/folders/r7/zcf7c3z94n3ghh80xsr4vf2r0000gn/T/irradiance-red-proof.4fpLnk)
error: arithmetic operation that can potentially result in unexpected side-effects
  --> src/lib.rs:48:5
   |
48 |     v[0] + n
   |     ^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#arithmetic_side_effects
note: the lint level is defined here
  --> src/lib.rs:38:5
   |
38 |     clippy::arithmetic_side_effects
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: indexing may panic
  --> src/lib.rs:48:5
   |
48 |     v[0] + n
   |     ^^^^
   |
   = help: consider using `.get(n)` or `.get_mut(n)` instead
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#indexing_slicing
note: the lint level is defined here
  --> src/lib.rs:36:5
   |
36 |     clippy::indexing_slicing,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^

error: used `unwrap()` on an `Option` value
  --> src/lib.rs:52:6
   |
52 |     *v.first().unwrap()
   |      ^^^^^^^^^^^^^^^^^^
   |
   = note: if this value is `None`, it will panic
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#unwrap_used
note: the lint level is defined here
  --> src/lib.rs:34:5
   |
34 |     clippy::unwrap_used,
   |     ^^^^^^^^^^^^^^^^^^^

error: could not compile `irradiance` (lib) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `irradiance` (lib test) due to 3 previous errors
      --> src/lib.rs:34:5
      --> src/lib.rs:36:5
      --> src/lib.rs:38:5
✓ lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected violations (clippy exit 101; clippy::indexing_slicing clippy::arithmetic_side_effects clippy::unwrap_used all fired).
REDPROOF EXIT=0
```

**`the lint level is defined here --> src/lib.rs:34/36/38`.** That is the whole
point of DEC-007: the level now resolves to the **library's** attribute block,
not to a header the proof brought with it. (DEC-007 recorded `src/lib.rs:31`
when it probed this; the block moved down three lines because `src/lib.rs`'s
module doc grew a paragraph naming this proof — same block, same file.)

### (b) Policy REMOVED + injection → clippy exits 0, so the proof FAILS

**This is the headline attack, run exactly as the handoff states it:** delete the
`#![deny(...)]` block from `src/lib.rs`, add
`pub fn read_u8(b: &[u8], at: usize) -> u8 { b[at] + 1 }`. All seven gates, on
the attacked tree:

```
### THE ATTACK: #![deny(...)] deleted from src/lib.rs + read_u8() added ###
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1     <- WAS 0 before this change
```

The red-proof's own output on that tree:

```
• clippy is present: clippy 0.1.97
• injection point: src/lib.rs line 34 (immediately after the attribute prologue)
• running clippy on a mutated copy of the crate — this MUST fail:
    Checking irradiance v0.1.0 (/private/var/folders/.../T/irradiance-red-proof.9D3kLx)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
ERROR: the lint policy did NOT reject the injected violations (clippy exited 0). src/lib.rs's `#![deny(...)]` block is missing, weakened, or not applying — the panic-free lint set is not wired to what it claims to check. This is exactly the manufactured-confidence failure oracle-must-be-shown-red exists to catch.
REDPROOF EXIT=1
```

Six gates still go green — they never were policy gates, and nothing in this
change pretends otherwise. The one gate whose job is the policy now **fails**,
which is the difference between the seven-green shipped panic and today.

Both directions were run against a **copy of the tree at
`$TMPDIR/…/attack2`**, never the working tree; `git status` after every run
shows only the intended edits, and no `irradiance-red-proof.*` temp dir
survives.

### Two more attacks, because "non-zero exit" is not a proof either

**(c) P1-2 — clippy unavailable.** A stub `cargo` on `PATH` that answers
``error: no such command: `clippy` `` (exit 101). Before this change the script
printed *"✓ … failed to compile as expected (clippy exit 101)"* and exited 0.
Now:

```
ERROR: `cargo clippy --version` failed — clippy is not available, so this proof can prove NOTHING. Refusing to report green. Output: error: no such command: `clippy`
REDPROOF EXIT=1
```

**(d) Assertion 3 has teeth — a PARTIAL weakening.** The realistic version of
the DEC-007 injection-point warning: keep the `#![deny(...)]` block but delete
two of the five lints. Clippy still exits 101 (`unwrap_used` fires), so an
exit-code-only proof would go green:

```
ERROR: clippy exited 101, but for the WRONG reasons: expected lint(s) clippy::indexing_slicing clippy::arithmetic_side_effects never fired. Either the policy no longer denies them, or the injection landed somewhere they do not apply (e.g. inside an `#[allow(...)]` scope — see DEC-007). A non-zero exit is not the proof; these lint names are.
REDPROOF EXIT=1
```

This is the same failure signature a mis-landed injection produces — DEC-007's
naive-`max()`-into-the-test-module case — so the mechanism's known fragility
fails loudly, as the decision predicted.

---

## All seven gates, run for real, on the tree as committed

```
──────────────────────────────────────────────────────────────
$ cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
BUILD     EXIT=0
──────────────────────────────────────────────────────────────
$ cargo clippy --all-targets --all-features -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
CLIPPY    EXIT=0
──────────────────────────────────────────────────────────────
$ cargo fmt --check
FMT       EXIT=0
──────────────────────────────────────────────────────────────
$ cargo test --all-features
running 2 tests
test tests::error_display_carries_context ... ok
test tests::error_type_is_public_and_non_exhaustive ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
TEST      EXIT=0
──────────────────────────────────────────────────────────────
$ /Users/jyashinsky/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
    Checking irradiance v0.1.0 (/Users/…/irradiance-verify-spec-001)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
MSRV      EXIT=0
──────────────────────────────────────────────────────────────
$ cargo deny check licenses
27 │     "Zlib",
   │      ━━━━ unmatched license allowance

licenses ok
DENY      EXIT=0
──────────────────────────────────────────────────────────────
$ ./scripts/lint-red-proof.sh
✓ lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected violations (clippy exit 101; clippy::indexing_slicing clippy::arithmetic_side_effects clippy::unwrap_used all fired).
REDPROOF  EXIT=0
```

`cargo 1.90.0 (840b83a10 2025-07-30)` via the shim — `cargo +1.90.0` without it
still fails exactly as the toolchain brief says.

All eight `app.just` recipes, for acceptance criterion 8:

```
just install         EXIT=0     just typecheck       EXIT=0
just dev             EXIT=0     just deny            EXIT=0
just build           EXIT=0     just lint-red-proof  EXIT=0
just test            EXIT=0     just lint            EXIT=0
```

`shellcheck -x scripts/lint-red-proof.sh` → clean.

---

## The rest of the punch list

**P2 — `core::` instead of `std::`, MEASURED not assumed** (`src/lib.rs:34,68`).
`use std::fmt` → `use core::fmt`; `impl std::error::Error` →
`impl core::error::Error`. Measured on the declared MSRV, which is the part the
handoff insisted on:

```
$ ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
MSRV(1.90) with core:: EXIT=0
```

No feature machinery added — that is still DEC-002's call.

**P2 — `tests/lint_policy_red.rs.disabled` deleted** (`git rm`). DEC-007
supersedes the mechanism it belonged to. `tests/` now holds only `corpus/`.

**P3 — `scripts/lint-red-proof.sh` is path-independent.** `_lib.sh` derives
`REPO_ROOT` from `$(pwd)`, so the script now overrides it from its own
`SCRIPT_DIR` after sourcing. Verified:

```
from scripts/          EXIT=0
from /tmp (abs path)   EXIT=0
```

**P3 — temp files.** Nothing is written into the working tree at all now: the
crate is copied to `mktemp -d` and the copy is `rm -rf`'d by a `trap … EXIT`.
Confirmed after every run above — `git status` shows only the intended edits and
no `irradiance-red-proof.*` directory survives. **No `.gitignore` line is needed
or was added**; the concern PL-6 raised is designed out rather than papered over.
Also confirmed the CI-realistic case where `Cargo.lock` is absent (it is
gitignored, so a runner never has one): the copy step skips it and the proof
still exits 0.

**P3 — AGENTS.md §6 now matches `app.just`.** Added the missing
`cargo fmt --check` line (the second half of `just lint`) and a
`lint-red-proof` entry for `./scripts/lint-red-proof.sh`, plus one sentence
making the block↔recipe correspondence an explicit standing obligation rather
than a thing that happened to be true once.

**P3 — the AGENTS.md inaccuracy the reviewer flagged** is that same §6 block
(PL-5 — the reviewer's "one small inaccuracy" in item 2 of their five). Fixed
above. Two further AGENTS.md statements were falsified *by this change* and were
corrected with it, per §1's rule that AGENTS.md must be true:
§5's red-proof pointer (`DEC-006` → `DEC-007`, noting the supersession) and §7's
tree (the deleted `.disabled` file). **One additional line was fixed and is
disclosed as a deviation below.**

**P2 (no action, as instructed) — DEC-006's `affected_scope`.** Left as-is;
`DEC-006` is `status: superseded`, `superseded_by: DEC-007`, and DEC-007 scopes
`src/lib.rs` correctly. Verified both fields on disk rather than assuming.

---

## Files changed

| File | Change |
|---|---|
| `scripts/lint-red-proof.sh` | rewritten per DEC-007 — copy-to-temp mutation test, three assertions, path-independent |
| `tests/lint_policy_red.rs.disabled` | **deleted** (DEC-007 supersedes DEC-006) |
| `src/lib.rs` | `core::fmt` / `core::error::Error`; module doc now names the proof that makes its "enforced mechanically" claim checkable |
| `AGENTS.md` | §5 DEC-007 pointer; §6 command block matches `app.just`; §7 tree updated |
| `.github/workflows/ci.yml` | red-proof job comment describes the DEC-007 mechanism and the three assertions |
| `app.just` | `lint-red-proof` comment describes the DEC-007 mechanism |

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 15,379,660 — **real, but not from `/cost`.**
- **Estimated USD:** null. No verified list rate for `claude-opus-5[1m]`, and
  §4's "no cache discount" rule applied to a total that is **97.5% cache-read**
  would overstate real spend by one to two orders of magnitude. DEC-013 forbids
  inventing one.
- **Duration (minutes):** 13 (first→last transcript timestamp: 04:12:00Z →
  04:24:45Z, plus the final turns).
- **Source of the number:** the `usage` objects in this session's own transcript
  (`~/.claude/projects/…-verify-spec-001/bc36989d-….jsonl`) — the same data
  `/cost` derives from. I am a Claude Code CLI session, but `/cost` is a
  **client-side slash command the assistant cannot execute**, so I read its
  source directly rather than reporting `null`. Composition: input 248 ·
  output 95,602 · cache-write 284,829 · cache-read 14,998,981. It is a
  **floor** — written before the session ends.

⚠ **Same non-comparability warning the verify cycle raised, now with a third
data point.** This 15.4M and the verify cycle's 5.2M are both cache-inclusive
sums from a 1M-context Opus session; the build cycle's 197,940 came from an
`Agent`-result `subagent_tokens` of unknown composition. Three sessions, two
incompatible units. `just calibration` reads these. The verifier already filed
this as a `process-debt` signal; it is now **more** urgent, not less, because
this cycle looks ~78× the first build for a fraction of the work.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-007 already settled the design; this
  cycle transcribed it. No implementer choice here rose to a decision.
- **Deviations from spec:**
  - **⚠ `handoff.to_agent` corrected: `claude-sonnet-5` → `claude-opus-5`.**
    The handoff was addressed to Sonnet per `tier_map.build`; it actually ran on
    `claude-opus-5[1m]` in a Claude Code CLI session. I did not choose that and
    cannot change it mid-run — but `handback-sync.sh:97` copies `to_agent`
    straight into the spec's cost session, so leaving it would have booked
    **15.4M Opus tokens against Sonnet**. That is precisely the ~6× tier
    surprise DEC-004 rule 3 exists to prevent, and `just calibration` is the
    thing it would have corrupted. Corrected in front-matter with a comment
    saying why. **The dispatch mismatch itself is yours to resolve, not mine**
    — either the tier map or the dispatch is wrong.
  - **AGENTS.md §7, one line beyond the punch list.** The tree comment read
    `specs/ # (none yet — STAGE-001 is unframed by design)`, which is false —
    `SPEC-001`…`SPEC-005` are framed and on disk. Corrected to
    `SPEC-001 … SPEC-005 (STAGE-001, framed)`. Not on the punch list; I was
    already editing that block for the deleted `.disabled` file, and leaving a
    known-false line in a file §1 requires to be true seemed worse than a
    one-line disclosed deviation. **Punch-list it if you disagree — it reverts
    cleanly on its own.**
  - **`src/lib.rs` module doc gained a paragraph** naming
    `scripts/lint-red-proof.sh` as what checks the "enforced mechanically" claim.
    Not requested. It closes the missing *link* the verifier named in their
    reflection #2 — the crate asserted mechanical enforcement while nothing
    pointed at the mechanism. Doc-only; no code or policy change.
  - Nothing else. `Cargo.toml`, `deny.toml`, `src/bin/irr.rs`, the CI job list
    and the lint set are untouched.
- **Follow-up work identified:**
  - **`guidance/constraints.yaml:33`** — `no-panics-on-untrusted-input`'s
    `enforcement:` field still reads `"fuzz targets from STAGE-001 onward;
    clippy; review"`. As of this change the red-proof is a real enforcement
    mechanism and belongs in that list. I left it alone because HANDOFF-003
    does not scope it and constraints are the one file where a build cycle
    editing rules feels wrong. **One line, and it closes the loop the verifier
    asked for.**
  - **Promote the `lint level is defined here --> src/lib.rs` note from
    evidence to assertion.** The script prints it; it does not require it.
    DEC-007 specifies exactly three assertions and I transcribed exactly three
    rather than redesigning, but a fourth — "the level resolved to
    `src/lib.rs`" — would make a mis-landed injection impossible to pass even
    if it somehow triggered the right lint names. Cheap; needs a DEC amendment,
    not a build decision.
  - The `metering_source` composition question above.
  - `scripts/handback-sync.sh:105` hard-coding `interface: other` — the
    verifier already flagged it; it will mis-stamp this session too, which is
    `claude-code`. Untouched here (template friction, `/feedback/` per DEC-000).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. This is the cleanest handoff of the three: DEC-007 had already
   done the design *and* the probing, so build really was transcription. The
   one thing that cost a loop was neither spec nor handoff — `/usr/bin/env
   bash` on this machine resolves to **/bin/bash 3.2.57**, not Homebrew's 5.x,
   so `MISSING=()` + `"${MISSING[@]}"` under `set -u` would have died on the
   *success* path. Worth a line in the toolchain brief: this repo's shell floor
   is bash 3.2, not "bash".

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No, but the loop the verifier described is still one line short of closed.
   `no-panics-on-untrusted-input`'s `enforcement:` field does not name the
   red-proof, so the constraint still points at "review" for the half that is
   now mechanical. Flagged as follow-up rather than done, because the handoff
   did not scope `guidance/`.

3. **If you did this task again, what would you do differently?**
   — Write the *attack* before the fix. I built the new script first and then
   constructed the policy-removal tree to test it. Doing it in the other order —
   stand up the attacked tree, watch the old script pass, then fix until it
   fails — would have made the acceptance criterion a red-green loop instead of
   a confirmation, and it is exactly what the verifier's own reflection #3 said
   about reaching for the throwaway tree sooner. The DEC-007 injection-point
   warning was the tell: that whole failure was found by attacking, not reading.
