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
  id: HANDOFF-006
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-20
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
  tokens_total: 10962512           # REAL combined count — what cost-audit reads
  estimated_usd: null              # no verified list rate for claude-opus-5[1m]; §4's no-cache-discount
                                   # rule applied to a 97.1%-cache-READ total would overstate real spend
                                   # by 1-2 orders of magnitude (DEC-013 forbids inventing one).
  duration_minutes: 25
  branch: feat/spec-001-crate-scaffold
  pr: null                         # committed locally; not pushed, not merged, per the return criteria
  completed_at: 2026-08-20         # YYYY-MM-DD
  notes: "APPROVED at b88d1ec (implementation 261706e). Ran the policy-removal attack myself (AGENTS.md §15 check 9) plus seven more: policy deleted + panicking public fn → six gates green, REDPROOF 1 with the control-clean attribution; round-2's `//`-comment bypass verbatim → REDPROOF 1; deny→warn → caught by the severity run ALONE (assertions 2/3/4 all pass); crate-root `#![allow]` after the deny → caught by the index.html#<lint> assertion; `/* */` prologue header → fails loudly and accurately. Tried and failed to build a control-passes-but-mutation-meaningless state: strongest construction (policy deleted + innocent decoy fns colliding with all four injected names → E0428 at 4 spans inside the injected range) passes the control, assertion 2 and assertion 4, and dies on assertion 3. Eight follow-ups, ZERO ship-blocking. Most material: (F-1) the severity run accepts a MIXED policy — deny(panic) + warn(other four) gives seven green gates, because one surviving deny carries the non-zero exit and warn-level lints still emit their help lines; CI's -D warnings still blocks any actual violation, so no panic ships. (F-2) the crate-root-only limit is live in src/lib.rs TODAY, not 'from SPEC-003' — one `#[allow(clippy::panic, clippy::expect_used)]` on a public fn, no module involved, gives seven green gates with two panics on the public API. (F-3) the obligation HANDOFF-006 says was attached to SPEC-003 is not in the tree, and the first module lands in SPEC-002. (F-5) the reason recorded for leaving _lib.sh's siblings unfixed is wrong — validate.sh:234 does read free-text spike.question; the conclusion still holds because no caller writes those values back to a file. tokens_total is REAL but not from `/cost` (a client-side slash command the assistant cannot execute): summed 96 usage objects in this session's own transcript (~/.claude/projects/-Users-...-verify-spec-001/e17489a8-....jsonl). Composition: input 192 + output 76,548 + cache-write 245,120 + cache-read 10,640,652 (97.1% cache-read). FLOOR — written before the session ends. Same method as verify-1/verify-2/build-2/build-3; NOT comparable to build-1's 197,940 (`token-counts-not-comparable`)."
  synced_at: 2026-08-20
---

# HANDOFF-006: <Task Title — same as the spec's title>

## Delegation Summary

Third verify cycle on `SPEC-001`, at `00f098b`. Round 2's two P1s are addressed
per **`DEC-009`** (supersedes `DEC-007`, which superseded `DEC-006`).

**Read the last paragraph of "Expected Deliverables" before you start.** This gate
has had three build rounds, and knowing when to approve is part of this job.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat it

- Seven gates re-run: green. Spec front matter parses; `cost.totals` 41,017,417
  across 5 sessions.
- **Round 2's exact PL-1 bypass re-run**: `//` comment before `#![forbid]`,
  `#![deny(`→`#![allow(` at column 0, two panicking public functions. Previously
  seven green with the proof printing ✓. Now **`REDPROOF 1`**, and the message
  correctly attributes cause: *"the control run above was clean, so this is the
  policy's fault and nothing else."*
- The `_lib.sh` YAML fix is correct — strip trailing comments only from unquoted
  scalars. Reproduced the old truncation.

### What deserves scrutiny

1. **Is the negative control actually load-bearing?** It is the claim that closes
   the class. Try to construct a state where the **control passes** but the
   mutation run is still meaningless.
2. **Three new supporting mechanisms** arrived with it: lint matching on clippy's
   `index.html#<lint>` help line, diagnostics asserted to fall inside the injected
   line range, and a third run without `-D warnings` pinning the policy at `deny`.
   Each is new surface. Are they sound, and is the third run doing something the
   other two don't?
3. **A fifth bypass was disclosed and deliberately not fixed** — the proof pins the
   policy at the **crate root only**, so a module with its own `#![allow(...)]` is
   uncovered. The orchestrator agreed this is a crate-shape decision, not a script
   one, and has **attached the obligation to `SPEC-003`** (which creates the first
   module). Confirm that disposition is right, and that nothing else silently
   depends on the uncovered case today.
4. **The `_lib.sh` fix is partial by choice.** `get_handoff_field()` (:283) and
   `get_spike_field()` (:327) carry the same bug, unfixed because no live caller
   passes them free text. Is "no current caller" an adequate reason, or is that
   the next silent corruption?
5. **Disclosed out-of-scope edits:** `app.just` and `AGENTS.md`. Confirm accurate
   and confined.
6. **`constraints.yaml`'s `enforcement:`** now names the red-proof as the only
   mechanical enforcement and calls it load-bearing. Is that claim now *true* —
   given item 3?

### Settled — do not reopen

MSRV 1.90 · fuzz deferral to SPEC-003 · `[lints]` in `Cargo.toml` · `core::` ·
`AGENTS.md` §7 · the `signals.yaml` merge divergence · cost figures (5 sessions,
41,017,417; the methodology mismatch is tracked as its own signal).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, working
`AGENTS.md` §15 "During verify". Run the policy-removal attack yourself (check #9)
— a red-proof you did not personally observe failing is a self-report.

⚠ **Mind the `attribute-text-inside-doc-comments` lesson signal (N=3, at bar).**
`src/lib.rs` contains two occurrences of `#![deny(`; the second is prose. Anchor
at column 0. Both the orchestrator and round 2's reviewer walked into this.

### On knowing when to approve

This is the **third** build round on one gate in a scaffold spec, and each prior
round was correctly found insufficient. That history argues for care — but it
also means the marginal round is getting expensive, and *"I found something"* is
not automatically *"this must not ship."*

So: separate **ship-blocking** from **follow-up**. A defect that lets a panic
reach the library is ship-blocking. A sharp edge in a dev script that fails loudly
is a follow-up — file it as a signal or a spec and approve. If the gate is sound
and the remaining items are follow-ups, **approve and say so plainly**; the
project needs this crate to exist so SPEC-002 onward can start.

If **APPROVED**, set `task.cycle: ship`; the orchestrator runs ship.

## Out of Scope

- Fixing anything. Punch-list with file:line.
- Re-litigating settled items above.
- The `signals.yaml` divergence (three signals on `main`, two here; resolution is
  "keep all five").
- Any decoding work.

## Return Criteria — how to hand back

1. Append a **verify** cost session with a real `tokens_total`; if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with the SHA reviewed, and for each finding an explicit
   **ship-blocking / follow-up** label.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

## Handback

### Execution notes

- **Branch / PR:** `feat/spec-001-crate-scaffold` @ `b88d1ec` — committed locally,
  **not pushed, not merged** (return criterion 4). No PR.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** **Yes.** AC 1/2/3/6/7/8/9 unchanged from round 2's
  ✅. AC 4 and AC 5 now hold for **5/5** of the policy they describe, and I verified
  the "shown red" of AC 5 personally rather than reading it.
- **For `verify`:** **✅ APPROVED at `b88d1ec`** (the tree; the implementation is
  `261706e`, the two commits after it are documentation). Eight findings, **all
  labelled follow-up — zero ship-blocking.**

---

## Verdict: ✅ APPROVED at `b88d1ec`

The gate is sound. Seven gates green on the honest tree, and the red-proof rejects
every attack that reaches the crate-root policy — including two it was never shown.
The three build rounds converged: round 1's gate could pass with clippy absent,
round 2's could pass with the policy never running, and this one could not be made
to report success for work it did not do in any construction I built.

### AGENTS.md §15 check 9 — I ran the oracle red myself

Not read, not transcribed. Every run below is mine, in a throwaway copy of the crate
outside the working tree, on `clippy 0.1.97` / `~/.cargo/bin/cargo +1.90.0`.

**The policy-removal attack** — col-0 `#![deny(...)]` block deleted, the reviewer's
own two-panic public function shipped in the library:

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 1
  ERROR: the lint policy did NOT reject the injected violations (clippy exited 0) — and the
  control run above was clean, so this is the policy's fault and nothing else. [...]
```

Six gates green; the red-proof is the only thing standing between that tree and a
shipped panic. That is the whole claim of this spec, and it holds.

### Seven more attacks, and which assertion caught each

| # | attack | gates | caught by |
|---|---|---|---|
| A | policy deleted + panicking `pub fn` | REDPROOF **1** | assertion 2 (control clean) |
| B | round-2 bypass **verbatim** (`//` before `#![forbid]`, `deny`→`allow`, panics) | REDPROOF **1** | assertion 2 — injection lands correctly, clippy honestly exits 0 |
| C | `#![allow(panic, expect_used)]` **after** the deny block | REDPROOF **1** | assertion 3 — 3 of 5 help lines present |
| D | `deny` → `warn`, tree otherwise honest | REDPROOF **1** | **assertion 5 alone** — 2, 3 and 4 all pass |
| E | `/* */` SPDX header above the prologue | dies at parse | `find_last_inner_attr_end` — loud, accurate message |
| F | policy deleted + innocent decoy fns colliding with all four injected names | REDPROOF **1** | assertion 3 (see below) |
| G | `#[allow(...)]` on a `pub fn` in `src/lib.rs` | **all seven green** | **nothing** — F-2 |
| H | `deny(panic)` + `warn(other four)` | **all seven green** | **nothing** — F-1 |

### The three questions the handoff asked

**1. Is the negative control load-bearing? Can it pass while the mutation is meaningless?**

I could not build that state. The strongest construction: delete the policy *and*
plant four innocent decoy functions in `src/lib.rs` whose names collide with all four
injected ones. The mutation run then fails with `E0428` at four distinct spans
**inside** the injected line range — so the control passes, assertion 2 passes, and
assertion 4 passes. It dies on assertion 3: `E0428` is a resolution error and aborts
before the lint passes run, so not one `index.html#<lint>` help line is emitted.

So the honest answer is more precise than `DEC-009`'s: **the control never caught a
bypass on its own in eight attacks — 2, 3 or 5 always did.** Its real contribution is
*attribution*: once the unmutated copy is clean, every diagnostic in the mutation run
comes from the injected lines, and that is what makes assertion 3 (a grep over a log)
and assertion 4 (a line-range count) legitimate rather than heuristic. `DEC-009`'s
"the control closes the class" is stronger than what is demonstrable — the *set*
closes it, and the control is what licenses the other two. That is a framing nuance,
not a defect, and it costs one clippy run on a 0.53 s crate. **No finding.**

**2. Are the three new mechanisms sound, and is the third run doing something?**

- **`index.html#<lint>` matching** (`:266-275`) — sound and the workhorse. It caught
  two of eight attacks (C and F). The end-of-line anchor is right: `index.html#panic`
  cannot match `#panic_in_result_fn`.
- **Diagnostic line-range assertion** (`:282-291`) — sound, belt-and-braces. I found
  no case where it is the unique catcher. See **F-8**.
- **The third run** — **yes**, and it is the only assertion that catches `deny`→`warn`
  (attack D: 2, 3 and 4 all pass). It is also the weakest of the three: see **F-1**.

**3. Is the SPEC-003 disposition right?**

The disposition is right — `#[allow]` is a language feature, no `#![deny]` mutation
test can close it, and it *is* a crate-shape decision. But the attachment does not
exist in the tree (**F-3**), the hole is live in `src/lib.rs` today rather than from
the first module (**F-2**), and the first module lands in **SPEC-002**, not SPEC-003.

---

## Punch list — 8 findings, all **follow-up**, none ship-blocking

**F-1 · `scripts/lint-red-proof.sh:300-308` — the severity run accepts a MIXED policy.**
Assertion 5 requires only a non-zero exit from the no-`-D warnings` run. One surviving
`deny` lint carries that exit for all five, and warn-level lints still emit their
`index.html#<lint>` help lines, so `assert_lints_fired` passes too. Measured — replacing
the block with `#![deny(clippy::panic)]` + `#![warn(unwrap_used, expect_used,
indexing_slicing, arithmetic_side_effects)]`:

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
```

`src/lib.rs:15` then falsely says all five are `deny`-level, and a consumer or a dev
running plain `cargo clippy` sees warnings, not errors (measured: plain clippy exit 0
on shipped `.unwrap()` + indexing). **Follow-up, not ship-blocking:** CI's
`-D warnings` clippy job still rejects the actual violation (measured: CLIPPY 101), so
no panic reaches the library through CI. This is round 2's PL-3 (rated P2) surviving
in partial form *inside the mechanism built to close it*. Fix shape: require each
expected lint's severity-run diagnostic to be `error:` rather than `warning:`.

**F-2 · `src/lib.rs:34-35` + `HANDOFF-005` "A residual I did not fix" — the crate-root
limit is live TODAY, and wider than recorded.** Measured: a single
`#[allow(clippy::panic, clippy::expect_used)]` on a `pub fn` in **`src/lib.rs` itself**
— no module, no SPEC-003 — gives seven green gates with two panics on the public API.
The handback says *"Not live today … Live from SPEC-003"*; `src/lib.rs:34-35` scopes
the exclusion to *"code in a module carrying its own `#[allow(...)]`"*. Both are
narrower than the measured hole. **Follow-up** — the mechanism is honest about pinning
the crate root and cannot close this class; the *records* understate its reach and its
timing. Fix shape: `HANDOFF-005`'s own cheap grep gate (no `allow(` of the five lint
names outside `#[cfg(test)]` and `src/bin/`), plus two corrected sentences.

**F-3 · The SPEC-003 attachment does not exist.** `HANDOFF-006` states the obligation
was *"attached to `SPEC-003`"*. A repo-wide grep finds it only in `src/lib.rs:34-35`
and in `HANDOFF-005`'s completed handback — a closed artifact, not a work item.
`projects/PROJ-001-monochrome-dng-develop/specs/SPEC-003-*.md` does not mention it and
`guidance/signals.yaml` has no entry. And the first module arrives in **`SPEC-002`**
(the corpus-manifest reader), one spec earlier than the attachment assumes.
**Follow-up, and the one item I would land during ship** — §15's ship cycle already
says to record exactly this in `signals.yaml`. It is the only finding that is
otherwise silently lost.

**F-4 · `guidance/constraints.yaml:33` — literally true, materially incomplete.**
Every clause holds; I tested all four named failure modes plus `#![allow]`-after-deny,
and all five go red. But `no-panics-on-untrusted-input` is a **blocking** constraint
whose rule is *"no unwrap()/expect()/panic!()/indexing … on any parse or decode path"*,
and the field says *"the ONLY mechanical enforcement today, and load-bearing"*. A
reader takes that as "the rule is mechanically enforced". It is not — one `#[allow]`
line exits it (F-2). Same class as round 2's PL-4, one boundary further out. One
sentence fixes it. Group with it: `SPEC-001` §"Scope discipline" still promises *"every
later spec inherits a crate where a panic on untrusted input **cannot compile**."*
**Follow-up.**

**F-5 · `scripts/_lib.sh:283` / `:327` — the sibling bug's stated reason is wrong.**
`get_handoff_field` and `get_spike_field` carry the identical unguarded
`sub("[[:space:]]*#.*$", "")`. `HANDOFF-005` leaves them on the grounds that *"their
callers only read short enum scalars (`cycle`, `to_agent`, `mode`, `outcome`)"*. Not
accurate: **`scripts/validate.sh:234` reads free-text `spike.question`** (today's:
*"Does the monochrome path generalise beyond one Leica body…"*). The **conclusion still
holds**, for a different reason: no caller writes those values back into a file, so the
worst case is a truncated value in a presence check (`validate.sh:235`) or a status
display — not the unterminated YAML scalar that broke the spec. **Follow-up, low**, but
the recorded reason is what the next reader will rely on, so it should be the true one.
The `get_handback_field` fix itself is correct; I re-verified the `\x27` guard on this
machine's awk (BWK 20200816) and it behaves.

**F-6 · `guidance/signals.yaml:132` — `attribute-text-inside-doc-comments` is stale at
N=3, and its evidence is out of date.** Build round 3 disclosed a fourth instance in its
own handback and it never reached the file (`last_touched: 2026-08-18`). I hit a fifth
this session: an unanchored replace on `#[cfg(test)]` matched the **prose** occurrence
at `src/lib.rs:16` and inserted a function into the middle of the doc block — caught by
printing the file, exactly as round 3 described. **N=5.** Separately, evidence item (2)
and `HANDOFF-006`'s own ⚠ both cite *"the second `#![deny(`"* in the module doc: that
occurrence **no longer exists**. Round 3's `src/lib.rs` rewrite removed it — there is
now exactly one `#![deny(`, at `src/lib.rs:44`, column 0. The hazard is real and has
moved to `#[cfg(test)]` at `src/lib.rs:16`. **Follow-up**; disposition already
`stage-close`, and it is well past bar.

**F-7 · `scripts/lint-red-proof.sh:164-177` — copy fidelity is an allowlist that will
drift.** The copy takes `Cargo.toml`, `Cargo.lock`, `src/` and `rust-toolchain*`/`.cargo`.
It does not take `clippy.toml`, `rustfmt.toml`, `build.rs`, `tests/`, `benches/` or
`examples/` — so `--all-targets` on the copy is not CI's target set the moment
`tests/*.rs` exists, which is **SPEC-002**. Harmless today (`tests/` holds only
`corpus/manifest.toml`) and the divergence direction is safe (the copy is stricter, and
a mismatch fails the control loudly). But `:12` and `src/lib.rs:20-21` both say *"the
exact CI invocation"*, and that stops being true next spec. **Follow-up, low.**

**F-8 · `scripts/lint-red-proof.sh:266-291` — assertions 3 and 4 are not joined.**
Assertion 3 checks *which* lints fired anywhere in the log; assertion 4 checks *how
many* distinct spans landed in the injected range. Neither checks that a given lint
fired *in range*. The control makes this sound in practice — nothing outside the
injection produces diagnostics — and I could not construct an exploit. A per-lint
in-range check would remove the reliance on that reasoning. Related: the `4` at `:288`
is a magic number tied to the injected function count; adding a fifth violating
function would silently weaken it while removing one fails loudly. **Follow-up, low.**

---

## §15 verify checklist

| # | check | result |
|---|---|---|
| 1 | acceptance criteria met and tested | ✅ AC 4/5 now 5/5, verified personally |
| 2 | spec's failing tests pass | ✅ seven gates green, run for real |
| 3 | no drift from referenced decisions | ✅ `DEC-009` transcribed; `just decisions-audit` 0 structural errors. `--changed` is blind on a committed branch (PL-7, `process-debt`) so I checked the diff by hand against `DEC-009`'s `affected_scope` |
| 4 | no constraint violations | ✅ — with **F-4** on the enforcement *text*, not the code |
| 5 | non-trivial choices have a `DEC-*` | ✅ none emitted, correctly: the severity run is one more invocation of a command already running. If it needs a home it is an amendment to `DEC-009`'s *Decision*, as the builder said |
| 6 | implementer reflection answered | ✅ substantive — Q3 ("fix the attribution before fixing the symptom") is the actual lesson of three rounds |
| 7 | `cost.sessions` has prior cycles | ✅ 5 sessions, 41,017,417; front matter parses |
| 8 | behavioral surface exercised | ✅ this is the check that mattered — see check 9 |
| 9 | **did the oracle go red?** | ✅ **run personally**, eight attacks |
| 10 | fuzz target | n/a — no parser (`SPEC-003`, §12 bar 2) |
| 11 | provenance row | n/a — no algorithm |
| 12 | new dependency permissive | n/a — zero dependencies; `cargo deny check licenses` green |

### Seven gates, honest tree, run by me

```
BUILD     EXIT=0        cargo build --release
CLIPPY    EXIT=0        cargo clippy --all-targets --all-features -- -D warnings
FMT       EXIT=0        cargo fmt --check
TEST      EXIT=0        cargo test --all-features
MSRV      EXIT=0        ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
DENY      EXIT=0        cargo deny check licenses
REDPROOF  EXIT=0        ./scripts/lint-red-proof.sh          (0.84 s, all three clippy runs)
```

`shellcheck -x scripts/lint-red-proof.sh` clean · `just validate` ✓ 6 artifacts ·
`./scripts/cost-audit.sh` ✓ · no temp dir left behind · working tree never mutated.

### Disclosed out-of-scope edits — accurate and confined

- **`app.just:45-52`** — comment only. The recipe body is unchanged.
- **`AGENTS.md`** — three `DEC-007` → `DEC-009` pointer updates (§5 `:207-211`,
  §6 `:279-281`, §7 `:309-312`). Nothing behavioral. Both accurate.
- **`scripts/_lib.sh:301`** — the right call, and I would have made the same one: an
  unparseable spec is worse than a two-line excursion, and the alternative was
  hand-authoring cost entries. See **F-5** for its unfixed siblings.

---

## Cost self-report

- **Tokens (total):** **10,962,512** — real, but **not from `/cost`**.
- **Estimated USD:** `null`. No verified list rate for `claude-opus-5[1m]`, and §4's
  "no cache discount" rule applied to a 97.1%-cache-**read** total would overstate
  actual spend by one to two orders of magnitude. `DEC-013` forbids inventing one.
- **Duration (minutes):** 25 (first→last transcript timestamp, plus these turns).
- **Source of the number:** the `usage` objects in this session's own transcript
  (`~/.claude/projects/…-verify-spec-001/e17489a8-….jsonl`), summed over **96** usage
  objects — the same data `/cost` derives from. `/cost` is a client-side slash command
  I cannot execute as the assistant, so I read its source data directly rather than
  reporting null.
  **Composition: input 192 · output 76,548 · cache-write 245,120 ·
  cache-read 10,640,652 — cache-read is 97.1% of the total.** It is a **floor**:
  written before the session ends.

⚠ **Comparability.** Same method as build-2 (15,379,660), build-3 (12,193,717),
verify-1 (5,242,951) and verify-2 (8,003,149) — those five *are* comparable. Build-1's
**197,940** came from an `Agent`-result `subagent_tokens` figure of unknown cache
composition and is **not**. Sixth data point on `token-counts-not-comparable`; five of
six now agree on method, which makes restating build-1 by the transcript method the
obviously cheaper fix.

---

## Drift and new artifacts

- **New decisions emitted:** **none.** Nothing here needs one. F-1 and F-2 are
  amendments to an existing mechanism and a crate-shape question respectively; if F-2
  is ever built, *that* is the decision, and `DEC-009` already names the
  compile-fail-harness alternative and why it would need its own.
- **Deviations from spec:** none. This was a review; I changed nothing but this
  handoff, the spec's cycle/verdict and its cost session.
- **Follow-up work identified:**
  - **F-1** — the severity run's mixed-policy hole. The one I would fix first: it is
    ~5 lines in a script that already exists, and it closes PL-3 properly.
  - **F-2 / F-3** — the crate-root `#[allow]` obligation: record it somewhere
    actionable **before `SPEC-002`**, not SPEC-003, and correct the two sentences that
    scope it to modules. `HANDOFF-005` already wrote the fix's shape.
  - **F-4** — one sentence in `constraints.yaml:33` and one in `SPEC-001`'s
    *Scope discipline*.
  - **F-5 / F-7 / F-8** — small, all fail-loud or safe-direction.
  - **F-6** — `attribute-text-inside-doc-comments` to N=5 with the two new instances,
    and drop the stale "second `#![deny(`" evidence.
  - Carried, unchanged, from prior rounds: `decisions-audit --changed` blindness on a
    committed branch (PL-7); the design cycle's missing `cost.sessions` entry;
    `scripts/handback-sync.sh:105`'s hard-coded `interface: other`; restating build-1's
    197,940 by the transcript method; and **nothing YAML-parses the spec front matter**
    — `just cost-audit` reported ✓ against an unparseable `SPEC-001` for a full round.
    That last one is this spec's own subject in miniature and is the cheapest of the
    five to fix.

---

## Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear, but one warning was stale in a way that cost me a detour:
   `HANDOFF-006` says `src/lib.rs` has two `#![deny(`, the second in prose. It has one.
   Round 3's rewrite removed the prose copy — and moved the hazard, because
   `#[cfg(test)]` still appears in the module doc at `:16`, which is what my first
   unanchored attack hit. The instruction *"anchor at column 0"* was right; the
   *example* attached to it was a round out of date, which is a good argument for a
   warning naming the rule and not the instance.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The one F-3 names. `HANDOFF-006` asked me to confirm the crate-root obligation had
   been attached to `SPEC-003`; checking rather than assuming is what found that it had
   not been attached anywhere, and that `SPEC-002` gets the first module. The general
   shape is this repo's recurring one: an obligation stated in prose inside a *completed*
   artifact is not a work item, and nothing in the toolchain distinguishes the two.

3. **If you did this task again, what would you do differently?**
   — Attack the *newest* mechanism first. I spent most of my budget trying to break the
   negative control, because that is what `DEC-009` calls load-bearing — and it held.
   Both green-gate findings (F-1, F-2) came from the assertions *around* it, in the last
   third of the session. Three rounds of this gate have each hardened the thing the
   previous round broke and left the newest addition least tested; a red-proof's weakest
   assertion is always the one that has never been reviewed.
