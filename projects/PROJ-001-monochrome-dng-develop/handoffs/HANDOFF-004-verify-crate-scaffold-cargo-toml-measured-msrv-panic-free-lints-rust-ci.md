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
  id: HANDOFF-004
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: 8003149            # REAL combined count — what cost-audit reads
  estimated_usd: null              # see notes — no verified list rate for claude-opus-5[1m]; a
                                   # no-cache-discount rate applied to a 95.5%-cache-READ total
                                   # would overstate real spend by 1-2 orders of magnitude (DEC-013).
  duration_minutes: 14
  branch: feat/spec-001-crate-scaffold
  pr: null                         # committed locally; not pushed, not merged, per the return criteria
  completed_at: 2026-08-18         # YYYY-MM-DD
  notes: "Verdict: PUNCH LIST (7 items, 2 of them P1) at c10f8e6 — sent back to build via `just advance-cycle SPEC-001 build --verdict punch-list`. The DEC-007 mechanism is right and the headline attack is genuinely caught (I ran it: BUILD 0 CLIPPY 0 FMT 0 TEST 0 MSRV 0 DENY 0 REDPROOF 1). Both P1s are gaps in its IMPLEMENTATION, each measured to a seven-green-gates false green with a shipped panic: (P1-1) assertion 3 greps the whole clippy log, and rustc renders the `#![deny(...)]` source span in ANY diagnostic pointing at it, so a mis-landed injection satisfies all three assertions — one legal `//` comment in lib.rs's prologue reproduces it; (P1-2) the policy has five lints, the injection exercises three, so `clippy::panic` and `clippy::expect_used` can be deleted and `panic!()`/`.expect()` shipped with everything green. tokens_total is REAL but not from `/cost`: `/cost` is a client-side slash command the assistant cannot execute, so I summed the `usage` objects in this session's own transcript (~/.claude/projects/-Users-...-verify-spec-001/cb3a5e92-....jsonl) — the same data `/cost` derives from. Composition: input 154 + output 79,223 + cache-write 282,861 + cache-read 7,640,911 (95.5% cache-read). It is a FLOOR: written before the session ends. Comparable in KIND to verify-1's 5,242,951 and build-2's 15,379,660 (same method); NOT to build-1's 197,940 (Agent-result subagent_tokens, unknown composition) — fourth data point on the `token-counts-not-comparable` signal. ⚠ build-2's 15,379,660 is still un-synced into the spec: HANDOFF-003 has `synced_at: null` and cost.totals reads 5,440,891, missing the largest cycle."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-004: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` to `claude-opus-5` (reviewer) for a
**second verify** cycle, at `7446edd`.

Round 1 returned ⚠ PUNCH LIST with two P1s. Both are addressed. **No independent
reviewer has seen the fix** — the orchestrator reconciled it, which is a different
job and deliberately not a substitute.

## Context the Receiving Agent Needs

Read `HANDOFF-002`'s handback (the round-1 punch list), `HANDOFF-003` (the fix
brief), and **`DEC-007`** (which supersedes `DEC-006` and settles the design).

### What the orchestrator already did — don't just repeat it

All seven gates re-run: green. The policy-removal attack re-run independently:
`BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 1`. Previously
all seven were 0.

**That is reconciliation, not verification.** You should still run the attack —
§15 check #9 says a red-proof you did not personally observe failing is a
self-report — but spend the cycle on *judgement*, not on re-confirming green.

⚠ **A trap that cost the orchestrator a false negative.** `src/lib.rs` now
contains **two** occurrences of `#![deny(` — the real attribute, and a module-doc
paragraph naming the proof. A naive `index('#![deny(')` hits the doc one and
deletes 14 characters of prose, leaving the policy intact and the attack
invalid — it looks like the fix failed when it didn't. Target the occurrence at
**column 0**. (This is the third doc-comment collision this spec has produced;
consider whether that is worth a lesson signal.)

### What actually deserves scrutiny

1. **The three assertions.** clippy ran, exited non-zero, and named all three
   lints. The builder added two attacks of its own: a stub `cargo` (caught by
   assertion 1) and **deleting two of the five lints — which still exits 101**, so
   exit-code-only would have passed. Are three assertions sufficient, or is there
   a fourth failure mode? Consider: lints present but at `warn`; the injection
   landing somewhere the lints don't reach; clippy running against a stale copy.
2. **The injection heuristic** parses the attribute prologue by tracking bracket
   depth. `DEC-007` records this as the design's main weakness. Try to break it.
3. **A disclosed deviation beyond scope:** `AGENTS.md` §7 said
   `specs/  # (none yet — STAGE-001 is unframed by design)`, false with SPEC-001–005
   on disk. The builder corrected it and flagged it. Was that the right call?
4. **`core::fmt` / `core::error::Error`** — claimed measured on 1.90.0. Verify.
   `DEC-002` is still `proposed`, so this must not have quietly committed us to
   `no_std`.
5. **A follow-up the builder declined:** `guidance/constraints.yaml`'s
   `enforcement:` for `no-panics-on-untrusted-input` still reads
   *"fuzz targets…; clippy; review"* and should now name the red-proof. Out of
   HANDOFF-003's scope, so filed rather than done. **Confirm declining was right**
   — and if it belongs in this spec after all, say so.

### Settled — do not reopen

MSRV 1.90; the fuzz-job deferral to SPEC-003; `[lints]` in `Cargo.toml` (rejected
in DEC-007); the cost entries; `tier_map.build` (corrected on `main` to
`claude-opus-5` after this build ran on Opus while the map said Sonnet).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**.

Work `AGENTS.md` §15 "During verify". The repo-specific checks that bite here:

- **#9** — run `./scripts/lint-red-proof.sh` and the policy-removal attack
  yourself. Watch the proof fail. Mind the doc-comment trap above.
- **#12** — zero dependencies. Confirm `Cargo.toml` and `Cargo.lock` agree.

If **APPROVED**, set `task.cycle: ship` and say so; the orchestrator runs ship
(reflection, `complexity_actual`, `archive-spec`, CHANGELOG).

⚠ **A merge hazard to be aware of, not to fix:** `guidance/signals.yaml` has
diverged — three signals on `main`, one on this branch, both appends to the same
region. Expect a conflict at merge; resolution is "keep all four." Do not
reconcile it here.

## Out of Scope

- Fixing anything. Punch-list it with file:line and send it back.
- Re-litigating settled items (see above).
- Any decoding work — SPEC-002 onward.
- The `signals.yaml` divergence.

## Return Criteria — how to hand back

1. Append a **verify** cost session with a real `tokens_total`. If `/cost` is
   unavailable to you, sum your transcript's usage objects and **say that is what
   you did** — the previous two cycles reported 197,940 and 15,379,660 by
   different methods, and the `token-counts-not-comparable` process-debt signal
   exists because of it. Note your cache-read share if you can.
2. Fill `## Completion` and the `handback:` block; `handoff.status: completed`.
3. State the verdict with the SHA reviewed.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Verdict

## ⚠ PUNCH LIST — at `c10f8e6` (`feat/spec-001-crate-scaffold`; the fix is `7446edd`)

**Seven items, two of them P1.**

`DEC-007`'s design is right and the fix is real. I ran the headline attack myself
rather than reading the report, and it is genuinely caught:

```
### ATTACK: #![deny(...)] deleted at column 0 + panicking pub fn shipped ###
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=1     <- ERROR: the lint policy did NOT reject the injected violations (clippy exited 0)
```

That is the round-1 P1 closed, and it is not a small thing.

**What I found is that the same false green is still reachable — twice — by
smaller edits than the one the fix was built to catch.** Both P1s below are
*measured*, both end in **seven green gates with a panic on untrusted input
shipped in the library**, and both are failures of the three assertions'
*implementation*, not of DEC-007's mechanism. Neither needs a redesign; one added
assertion kills both.

Not a ❌. `AGENTS.md` §15's four extra checks all hold as written: **check 9
passes — I personally observed the red-proof fail**, on the exact attack the
handoff names. Checks 10 and 11 are correctly N/A. Check 12 is zero dependencies.
This is the same call round 1 made on a strictly worse instance of the same
class, and I am making it for the same reason: the question is what the red
*proves*, not whether it happened.

**Spec sent back to build**: `just advance-cycle SPEC-001 build --verdict punch-list`.
I fixed nothing.

---

### The punch list

#### PL-1 — P1 — Assertion 3 is satisfied by rustc *rendering* the `#![deny(...)]` block, not by the lints firing. One legal `//` comment in `src/lib.rs`'s prologue turns the proof into a false green.

`scripts/lint-red-proof.sh:155-163` (the grep) · `scripts/lint-red-proof.sh:63-90`
(the prologue parser) · `.github/workflows/ci.yml:96-99` · `src/lib.rs:19-24`

`EXPECTED_LINTS` is grepped against the **whole clippy log**. But rustc prints the
source span of any attribute a diagnostic points at — and the multi-line
`#![deny(...)]` span *is* the five lint names. So any error that points at that
block puts all three expected names in the log **without a single lint having
fired**.

The prologue parser at `:63-90` skips blank lines, `//!` docs and `#![...]`
blocks. It does **not** skip a plain `//` comment — which is legal Rust anywhere
in the prologue. Add one and the injection point moves to *above* the inner
attributes, where `pub fn` is a syntax error. Measured, adding exactly this line
before `#![forbid(unsafe_code)]`:

```rust
// Lint policy: see DEC-007.
```

```
• injection point: src/lib.rs line 32 (immediately after the attribute prologue)
error: an inner attribute is not permitted in this context
  --> src/lib.rs:49:1
   |
49 | / #![deny(
50 | |     clippy::unwrap_used,
51 | |     clippy::expect_used,
52 | |     clippy::indexing_slicing,       <- the lint names, in the log,
53 | |     clippy::panic,                     rendered by an UNRELATED error
54 | |     clippy::arithmetic_side_effects
55 | | )]
   | |__^
error: could not compile `irradiance` (lib) due to 2 previous errors
✓ lint policy red-proof: src/lib.rs's own #![deny(...)] rejected the injected violations (clippy exit 101; ... all fired).
REDPROOF EXIT=0
```

Assertion 1 ✅ clippy ran · assertion 2 ✅ exit 101 · assertion 3 ✅ all three
names present · **and the policy never ran at all.** The injected functions were
never linted. The success line is false.

**Weaponised — the full false green.** Two edits: the `//` comment above, plus
`#![deny(` → `#![allow(` **at column 0**, plus a panicking public function:

```
### // comment in prologue + col-0 deny->allow + panicking pub fn ###
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=0     <- "✓ ... src/lib.rs's own #![deny(...)] rejected the injected violations"
```

`src/lib.rs` says `#![allow(`. The proof says `#![deny(`. Seven green gates, a
shipped `b[at] + 1` on attacker-influenced input. **This is round 1's PL-1, intact,
reached through the fix.**

**Why it matters beyond the reproduction.** `DEC-007`'s *Consequences* is explicit
that the injection heuristic is the mechanism's known fragility and that the
mitigation is assertion 3: *"if injection lands somewhere the lints don't apply,
the expected lint names will be **absent** and the proof fails loudly rather than
passing."* That is the load-bearing sentence of the decision, and it is **false as
implemented** — the names are present precisely *because* the injection landed
somewhere they don't apply. `ci.yml:97-99` repeats the claim (*"a non-zero exit
alone would also be produced by an unrelated compile error"* — implying the names
discriminate it; they do not), and `src/lib.rs:19-24` repeats it to readers of the
library. Three artifacts assert a property the script does not have.

**The fix that closes the whole class** (build's call — naming, not prescribing):
run the *same* clippy invocation on the **unmutated** copy first and require exit
**0**, before injecting. Then a non-zero exit on the mutated copy is attributable
to the injection by construction. ~6 lines, and it also kills unrelated compile
errors, a broken prologue parse, a stale copy, and a `Cargo.toml` the copy step
under-copies — no grep can do that. A cheaper partial: grep for the clippy help
URL fragment (`index.html#unwrap_used`), which is emitted **only** when the lint
actually fires, instead of the bare name, which appears in any rendered span. I
confirmed no `index.html#` line appears in the false-green log above.

**A related variant, same root cause.** Putting the `//` comment at **line 1**
(an SPDX header — plausible for a dual-licensed crate) yields `INJECT_AT=1`, and
`scripts/lint-red-proof.sh:130` runs `head -n 0`:

```
head: illegal line count -- 0
REDPROOF EXIT=1
```

Fails closed, but with a raw shell error and no `die` message — DEC-007 promised
"a loud missing-lint failure", and a maintainer gets neither that nor any hint
that the heuristic broke. ⚠ **Unverified half:** BSD `head` rejects `-n 0`; GNU
coreutils accepts it and outputs nothing, which would drop this variant into the
PL-1 false-green path instead of erroring. CI is `ubuntu-latest`. I could not
start a Linux container to confirm, so treat the GNU half as reasoned, not
measured — but it should be checked before this is dismissed as a nit.

#### PL-2 — P1 — The policy has five lints; the proof exercises three. Delete `clippy::panic` and `clippy::expect_used` and ship `panic!()` + `.expect()` with all seven gates green.

`scripts/lint-red-proof.sh:110-126` (the injected violations) ·
`scripts/lint-red-proof.sh:155` (`EXPECTED_LINTS`) · `src/lib.rs:33-39` ·
SPEC-001 acceptance criteria 4 and 5

The injection covers `indexing_slicing`, `arithmetic_side_effects` and
`unwrap_used`. Nothing in it triggers `clippy::expect_used` or `clippy::panic`,
and neither name is in `EXPECTED_LINTS`. Those two lints are therefore
**unprotected by the proof that exists to protect them.**

The builder's own attack (d) deleted two of five and was caught — but it deleted
two of the three that *are* checked. Deleting the other two instead:

```
### clippy::panic + clippy::expect_used removed; panic!()/expect() in the LIBRARY ###
BUILD     EXIT=0
CLIPPY    EXIT=0
FMT       EXIT=0
TEST      EXIT=0
MSRV      EXIT=0
DENY      EXIT=0
REDPROOF  EXIT=0     <- "✓ ... rejected the injected violations ... all fired"
```

The library function that ships:

```rust
pub fn tag_len(b: &[u8]) -> usize {
    if b.len() < 4 {
        panic!("truncated tag header");        // clippy::panic — undenied, unproven
    }
    let n: Option<usize> = b.first().map(|v| usize::from(*v));
    n.expect("first byte present")             // clippy::expect_used — same
}
```

**Why it matters.** `clippy::panic` is the most on-the-nose of the five for
`no-panics-on-untrusted-input` — an explicit `panic!()` on a malformed length is
the canonical version of the defect the constraint names. AC 4 requires *"the five
panic-free lints are `deny`-level"*; AC 5 requires *"the lint policy is shown
RED"*. Together they are satisfied for 3/5. `src/lib.rs:15` says "the five lints
below are `deny`-level here" and `:22` says "Delete or weaken that block and the
proof fails" — false for two of the five. Two more functions in the injection
heredoc and two more names in `EXPECTED_LINTS`.

#### PL-3 — P2 — `deny` → `warn` passes the proof, and the proof then prints a `#![deny(...)]` that is not in the file.

`scripts/lint-red-proof.sh:146-169` · `src/lib.rs:15`, `:22-24`, `:33`

Changing `#![deny(` to `#![warn(` at column 0 leaves the red-proof green: CI's
blanket `-D warnings` still promotes them to errors, so clippy exits 101 and all
three names fire.

```
#![warn(          <- the file
REDPROOF EXIT=0   <- "✓ ... src/lib.rs's own #![deny(...)] rejected ..."
```

Lower severity than PL-1/PL-2 because `-D warnings` means the real tree is still
enforced — but the enforcement has silently moved from the policy to a CI flag.
Measured consequence: with `warn`, a plain `cargo clippy` (no `-D warnings`) and
`cargo build` both **exit 0** on a panicking public function. Anyone running
clippy locally, or a consumer, sees nothing. And `src/lib.rs:15`'s
"`deny`-level" claim is false while the proof calls it true. The unmutated-control
fix in PL-1 does not catch this one; a `deny`-level assertion (or accepting `warn`
and correcting the prose) is a separate call.

#### PL-4 — P2 — Three artifacts state a discrimination property the script does not have.

`src/lib.rs:19-24` · `.github/workflows/ci.yml:96-99` ·
`decisions/DEC-007-…:117-119` (the *Consequences* mitigation sentence)

Not a separate defect — the documentation half of PL-1, listed separately because
it must be corrected in the same change and it is easy to fix the script and leave
the prose. Each says, in its own words, that checking the lint names distinguishes
a real policy rejection from an unrelated failure. It does not. `src/lib.rs:22-24`
is the load-bearing one: it is the sentence that licenses `:15`'s "enforced
mechanically" to readers of the library.

#### PL-5 — P3 — `INJECT_AT=1` produces `head: illegal line count -- 0`, not a diagnostic.

`scripts/lint-red-proof.sh:130`

Detailed under PL-1. `[ "$INJECT_AT" -lt 2 ] && die "…"` — one line, and it turns
a platform-dependent shell error into the loud failure DEC-007 promises.

#### PL-6 — P3 — Flag, not a block: build round 2's cost session is not in the spec, so `cost.totals` understates the spec by ~74%.

`projects/…/handoffs/HANDOFF-003-build-…md:62` (`synced_at: null`) ·
`projects/…/specs/SPEC-001-…md` (`cost.sessions`, `cost.totals`)

§15 check 7. `cost.sessions` has two entries (build 197,940; verify 5,242,951) and
`cost.totals.tokens_total: 5440891`. HANDOFF-003's handback carries a real
`tokens_total: 15379660` that has never been transcribed — the largest cycle on the
spec is missing from its own total. `just handback-sync SPEC-001` picks it up
(idempotent via `synced_at`). Orchestrator's, not the builder's.

Still open from round 1, same check: **the design cycle has no `cost.sessions`
entry** — §4 wants one, `null` with a "main-loop, not separately metered" note.

#### PL-7 — P3 — `decisions-audit --changed` sees nothing on a committed branch.

`scripts/decisions-audit.sh` (via `just decisions-audit --changed`)

§15 check 3 names this as the mechanism for confirming a change stayed consistent
with the decisions governing the files it touched. On this branch it reports:

```
• No changed files in scope (your uncommitted changes).
```

It only inspects the **working tree**, so on a clean branch under review — the
exact moment check 3 is performed — it is structurally blind. I did check 3 by
hand instead: `DEC-007`'s `affected_scope` (`src/lib.rs`,
`scripts/lint-red-proof.sh`, `.github/workflows/ci.yml`) covers all three code
files `7446edd` touched, which is round-1 PL-3's shape correctly fixed. Tooling
gap, not a build defect — worth a `process-debt` signal, since it silently
converts a required verify check into an honour system.

---

### The five things the handoff asked me to scrutinise

**1. Are three assertions enough? Is there a fourth failure mode?**

*No, and yes — two of them.* **PL-1** and **PL-2** above; both measured to seven
green gates. Taking the handoff's three hypotheses in turn:

- *lints at `warn`* — real, **PL-3**. Proof stays green; CI still enforces via
  `-D warnings`, local clippy and `cargo build` do not.
- *injection landing out of scope* — real and **worse than predicted**, **PL-1**.
  DEC-007 says this fails loudly via missing names. It does not: the names are in
  the log because rustc renders the deny block's span. This is the one to fix
  first — it is the decision's own stated mitigation, and it does not hold.
- *clippy running against a stale copy* — **I could not break this one.** With
  `CARGO_TARGET_DIR` pointed at the repo's warm `target/`, the proof still went
  red correctly; cargo's metadata hash includes the package root path, so the
  temp copy does not collide with the real crate. There is no
  `rust-toolchain.toml` and no `.cargo/` in the tree, so the copy step is not
  silently dropping a toolchain pin either. Clean.

The fourth failure mode, stated generally: **assertion 3 tests the log, and the
log contains the policy's own source text.** A grep over rendered rustc output
cannot distinguish "the lint fired" from "something printed the line that defines
the lint". The unmutated-control run in PL-1 is the assertion that does not have
this problem, because it is not a grep.

**2. Try to break the injection heuristic.** Broken — `//` comments, three
variants, `scripts/lint-red-proof.sh:63-90`. A plain `//` line anywhere in the
prologue moves the injection above the inner attributes. Two of the three
variants are **silent false greens** (PL-1); the third is an opaque `head` error
(PL-5). What I could *not* break: `#![...]` attributes with `]` inside a string
literal (depth goes negative, harmless — injection still lands correctly, proof
still red), and multi-line attributes generally. The bracket-depth tracking is
fine. It is the comment case that is missing, and DEC-007's *Revisit* trigger —
"if `lib.rs`'s prologue grows a shape the injection heuristic mishandles" — has
already fired: the shape is one comment line.

**3. Was correcting `AGENTS.md` §7's `specs/` line the right call?** **Yes.**
`AGENTS.md:350` said `specs/  # (none yet — STAGE-001 is unframed by design)`
with SPEC-001–005 on disk. §1 requires AGENTS.md to be true, the replacement
(`# SPEC-001 … SPEC-005 (STAGE-001, framed)`) checks out against disk, it is a
one-line comment with no behavioural surface, and it was disclosed. Declining
would have left a false statement standing in the file that governs every agent
that reads this repo — a worse outcome than a two-word scope excursion. Same
class as round 1's `.gitignore` fix, which verify also endorsed. It is right that
it was flagged rather than done silently; that is the part that makes it safe.

**4. `core::fmt` / `core::error::Error` on 1.90.0 — and does it commit us to
`no_std`?** **Verified, and no.** Measured myself on a **fresh** target dir so no
cache could hide it:

```
$ CARGO_TARGET_DIR=<fresh> ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
    Checking irradiance v0.1.0
    Finished `dev` profile ...                                  EXIT=0
```

and directly against `rustc 1.90.0 (1159e78c4 2025-09-14)`, compiling a
`use core::fmt` + `impl fmt::Display` + `impl core::error::Error` stub → exit 0.
`grep -rn 'std::' src/` returns nothing.

On the `no_std` question specifically: there is **no** `#![no_std]`, no
`[features]` table, no `default = ["std"]`, no `cfg_attr`. The crate still links
`std`; only the *paths* changed. `DEC-002` remains `proposed` and unforeclosed,
which is exactly what it asked for — the door is open, nobody walked through it.

**5. Was declining the `constraints.yaml` follow-up right?** **Yes — and it
should be done in the round that closes PL-1 and PL-2, not before.**

`guidance/constraints.yaml:33` still reads
`enforcement: "fuzz targets from STAGE-001 onward; clippy; review"`. Declining was
right on scope (HANDOFF-003 did not list it) and right on substance (the field is
incomplete, not false — clippy and review do both apply). But there is a stronger
reason not to have done it in this round, which I would not have expected the
builder to see: **naming the red-proof there now would document an enforcement
guarantee the red-proof does not yet provide.** As of PL-2 it covers three of the
five lints in that constraint's own rule text, and as of PL-1 it can be defeated
by a comment. Write the sentence when it is true. The round that closes PL-1/PL-2
is the round that earns it — and at that point it is not optional bookkeeping:
the red-proof will be the *only* mechanical enforcement of a `blocking` constraint
(fuzz does not exist until SPEC-003; "clippy" *is* the policy; "review" is not
mechanical), so nothing in the repo currently records that
`scripts/lint-red-proof.sh` and its CI job are load-bearing.

---

### What I verified myself

I re-ran; I did not read the build report as evidence. Every attack ran in a
throwaway `git worktree` off `c10f8e6`; the working tree was never mutated and
`git status` is clean.

⚠ I walked into the doc-comment trap the handoff warned about — my first
`replace('#![deny(', '#![allow(')` hit the `//!` paragraph at `src/lib.rs:22`, and
the "attack" came back with the policy intact. Caught it by printing the file
after editing rather than trusting the edit. The warning was correct and it is
worth keeping: **three doc-comment collisions on one spec is a pattern**, and the
next one will cost someone a wrong conclusion rather than two minutes. A
`lesson`-type signal seems right.

**Seven gates on the honest tree at `c10f8e6`:**

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
```

**All eight `app.just` recipes:** `install` `dev` `build` `test` `lint`
`typecheck` `deny` `lint-red-proof` — all exit 0, tree clean afterwards, no
`irradiance-red-proof.*` temp dir left behind. `shellcheck -x
scripts/lint-red-proof.sh` clean.

**§15 check 9, personally observed:** the policy-removal attack turns `REDPROOF`
red (output at the top of this handback). I also watched the proof pass correctly
on the honest tree, with `the lint level is defined here --> src/lib.rs:34/36/38`
resolving to the library's own block — the thing DEC-007 exists to make true.

| AC | Verdict | Evidence |
|---|---|---|
| 1 | ✅ | `build`/`test`/`clippy -D warnings`/`fmt --check` all exit 0 |
| 2 | ✅ | `rust-version = "1.90"` (`Cargo.toml:10`); 1.90.0 check exit 0 on a **fresh** target dir; CI pins `dtolnay/rust-toolchain@1.90.0` (`ci.yml:81`) |
| 3 | ✅ | `#![forbid(unsafe_code)]` — `src/lib.rs:32` |
| 4 | ⚠ | Five lints are deny-level at `src/lib.rs:33-39`; the only `#[allow]` is scoped to `#[cfg(test)] mod tests` (`:78-87`); `src/bin/irr.rs` carries no lint attributes. But see **PL-2** — two of the five are unproven, and **PL-3** — `deny` is not asserted |
| 5 | ❌→⚠ | The red is real, correctly sourced to `src/lib.rs`, and I watched it. But **PL-1** and **PL-2** each defeat it to a seven-green false green. This is the criterion the punch list is about |
| 6 | ✅ | `cargo deny check licenses` → `licenses ok`, exit 0; CI job present |
| 7 | ✅ | Public API is `pub enum Error` alone; `irr` appears in `src/lib.rs` only in doc comments |
| 8 | ✅ | All eight recipes exit 0; `AGENTS.md` §6's block now matches `app.just` including `cargo fmt --check` and `./scripts/lint-red-proof.sh` — round-1 **PL-5 closed** |
| 9 | ✅ | No badge, no verification claim; `ci.yml` disclaims explicitly |

Round-1 punch list, independently re-checked: **PL-1 closed** (mechanism replaced
per DEC-007 — with the two new holes above) · **PL-2 closed** (stub `cargo`:
`ERROR: cargo clippy --version failed … Refusing to report green`, exit 1) ·
**PL-3 closed** (DEC-007's `affected_scope` covers `src/lib.rs`; DEC-006 correctly
left superseded) · **PL-4 closed** (`core::`, measured) · **PL-5 closed** (§6
matches) · **PL-6 closed** (path-independent via `SCRIPT_DIR`; temp-dir design
removes the gitignore concern — verified no leftovers).

§15 checks: 1 ⚠(punch list) · 2 ✅ · 3 ✅ by hand, see **PL-7** · 4 ⚠ (`no-panics-on-untrusted-input`
is `blocking` and PL-1/PL-2 are holes in its only mechanical enforcement) · 5 ✅
(DEC-007 emitted, scoped, honest) · 6 ✅ · 7 ⚠ **PL-6** · 8 ✅ · **9 ✅ observed
personally** · 10 N/A · 11 N/A · **12 ✅ zero dependencies** (`Cargo.toml:29-31`
both tables empty; `Cargo.lock` holds exactly one package, `irradiance` itself —
they agree).

### Cost self-report

- **Tokens (total):** **8,003,149** — real, but **not from `/cost`.**
- **Estimated USD:** null. No verified list rate for `claude-opus-5[1m]`, and §4's
  "no cache discount" rule applied to a 95.5%-cache-**read** total would overstate
  actual spend by one to two orders of magnitude. DEC-013 forbids inventing one.
- **Duration (minutes):** 14 (first→last transcript timestamp, plus these turns).
- **Source of the number:** the `usage` objects in this session's own transcript
  (`~/.claude/projects/…-verify-spec-001/cb3a5e92-….jsonl`), summed over 77 usage
  objects — the same data `/cost` derives from. `/cost` is a client-side slash
  command I cannot execute as the assistant, so I read its source directly rather
  than reporting null. **Composition: input 154 · output 79,223 · cache-write
  282,861 · cache-read 7,640,911 — cache-read is 95.5% of the total.** It is a
  **floor**: written before the session ends.

⚠ **Comparability.** Same method as verify-1 (5,242,951, 97% cache-read) and
build-2 (15,379,660, 97.5% cache-read) — those three *are* comparable to each
other. Build-1's **197,940** came from an `Agent`-result `subagent_tokens` figure
of unknown cache composition and is **not**. Fourth data point on the
`token-counts-not-comparable` signal; three of four now agree on method, which
suggests the fix is to restate build-1 rather than to keep annotating everything
else.

I did **not** hand-append the cost session to the spec. `just handback-sync
SPEC-001` transcribes it from the `handback:` block above and is idempotent via
`synced_at` (`scripts/handback-sync.sh:65-67`); appending by hand would
double-count. Running it also picks up **build-2's missing 15,379,660** — see
**PL-6**.

### Drift and new artifacts

- **New decisions emitted:** none. Both P1s are gaps in `DEC-007`'s
  implementation, not new choices. The unmutated-control assertion is a
  build-cycle implementation detail; if the builder prefers a different fix, *that*
  may warrant amending DEC-007's *Consequences* — its mitigation sentence is
  currently false and should not survive the fix unchanged either way.
- **Deviations from spec:** one, disclosed and correct — the `AGENTS.md` §7
  `specs/` line (scrutiny item 3). I found no undisclosed deviation; `7446edd`
  touches eight files and every one is accounted for by HANDOFF-003's brief.
- **Follow-up work identified:**
  - `guidance/constraints.yaml:33` — `enforcement:` must name the red-proof, in
    the round that closes PL-1/PL-2 (scrutiny item 5).
  - `decisions-audit --changed` is blind on a committed branch (**PL-7**) —
    `process-debt`. It makes §15 check 3 unrunnable at the moment it is required.
  - Doc-comment collisions on `src/lib.rs`, three on this spec (**lesson**, N=3).
    Every mechanism that pattern-matches the library's source needs to anchor at
    column 0; the next collision costs a wrong conclusion, not two minutes.
  - Restate build-1's 197,940 by the transcript method, or mark it
    non-comparable in `cost.totals` — three of four sessions now share a method.
  - The design cycle's missing cost session (still open from round 1).
  - `scripts/handback-sync.sh:105` hard-codes `interface: other` (round 1's
    finding; the build-1 entry still reads `other`).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. This handoff was the most useful of the four: naming the
   doc-comment trap, listing three specific hypotheses for a fourth failure mode,
   and saying outright "spend the cycle on judgement, not on re-confirming green"
   is what made two P1s findable in fourteen minutes. Two of the three hypotheses
   were live; I would not have thought to test the stale copy unprompted, and
   ruling it out cleanly is worth as much as the two that broke.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Not listed, but load-bearing and unrecorded: **nothing in the repo says
   `scripts/lint-red-proof.sh` is the only mechanical enforcement of a `blocking`
   constraint.** `constraints.yaml:33` names fuzz (doesn't exist yet), clippy (*is*
   the policy) and review (not mechanical). A future spec could delete that CI job
   as cleanup and no gate, audit or decision record would object. That is the same
   shape as the defect this spec has now spent two rounds on, one level further out
   — the mechanism is unprotected the way the policy was.

3. **If you did this task again, what would you do differently?**
   — Attack the *proof's assertions* before attacking the policy. I spent the
   first third re-running the headline policy-removal attack, which the handoff had
   already told me the orchestrator ran twice; it was necessary (check 9) but it
   found nothing new, by construction. Both P1s came from the opposite question —
   not "can I remove the policy and stay green" but "**can I satisfy all three
   assertions without the policy ever running**" — and that question took about
   four minutes to answer once asked. For a red-proof, the productive attack
   surface is the oracle's own success condition, not the thing it watches.
