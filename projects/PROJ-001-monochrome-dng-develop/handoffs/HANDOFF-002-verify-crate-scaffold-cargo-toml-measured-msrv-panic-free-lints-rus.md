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
  id: HANDOFF-002
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
  tokens_total: 5242951 # REAL combined count — what cost-audit reads
  estimated_usd: null              # see notes — no verified list rate for claude-opus-5[1m]; a
                                   # no-cache-discount rate applied to a total that is ~97%
                                   # cache-READ tokens would overstate real spend by 1-2 orders
                                   # of magnitude. Refusing to invent one (DEC-013).
  duration_minutes: 13
  branch: feat/spec-001-crate-scaffold
  pr: null                         # still unpushed — see the verdict note
  completed_at: 2026-08-18
  notes: "Verdict: PUNCH LIST (6 items, 2 of them P1) at 29515ab — sent back to build via `just advance-cycle SPEC-001 build --verdict punch-list`. tokens_total is REAL but not from `/cost`: `/cost` is a client-side slash command I cannot execute as the assistant, so I summed the `usage` objects in this session's own transcript (~/.claude/projects/-Users-...-verify-spec-001/14bd8f1c-....jsonl) — the same data `/cost` derives from. Composition: input 98 + output 48,357 + cache-write 124,577 + cache-read 5,069,919. It is a FLOOR: written before the session ends, so it excludes these final turns. ⚠ NOT comparable to the build's 197,940, which came from an Agent-result `subagent_tokens` figure of unknown cache composition — do not put them in the same rollup without resolving that (process-debt signal)."
  synced_at: 2026-08-18
---

# HANDOFF-002: Crate scaffold: Cargo.toml, measured MSRV, panic-free lints, Rust CI

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` to `claude-opus-5` (reviewer) for the
**verify** cycle.

Review the crate scaffold on `feat/spec-001-crate-scaffold`. **You are a different
session from the builder — that independence is the entire point of this cycle**
and the dogfood's best-evidenced quality lever. Do not read the build session's
report as evidence; re-run things.

## Context the Receiving Agent Needs

Branch `feat/spec-001-crate-scaffold`, 3 commits on top of `e8633b6`.

**The orchestrator has already reconciled the build against git and disk**
(DEC-004 rule 1) and re-ran every gate. All six pass. So your job is **not** to
re-confirm the gates go green — it is to ask whether green means what it claims.

### What the build did that is worth scrutiny

1. **It changed my spec's red-proof design, and I agreed.** SPEC-001's literal
   snippet would have been a **false green**: a `tests/*.rs` file is its own crate
   root and does *not* inherit `src/lib.rs`'s `#![deny(...)]`. The builder caught
   this and made the snippet carry its own `#![deny]`, swapped in by
   `scripts/lint-red-proof.sh`. I verified the claim independently. **Verify the
   fix is sound, not just present** — e.g. does the script restore state on
   failure, and would it still fail if the lints were silently removed from
   `src/lib.rs`?
2. **It edited `AGENTS.md` §5/§6/§7**, which was not in its deliverables list. My
   read: legitimate, because those sections said "no `Cargo.toml` exists yet",
   which the change made false, and §1 requires AGENTS.md to be true. **Confirm
   the edits are accurate and did not overreach.**
3. **It fixed a `.gitignore` bug** — `Cargo.lock  # comment` — gitignore does not
   strip inline comments, so the pattern matched nothing. Real bug, out of scope,
   correctly fixed. Confirm it now actually ignores `Cargo.lock`.
4. **It emitted `DEC-006`** for the red-proof mechanics at confidence 0.85.
5. **It correctly declined to wire a fuzz job**, deferring to SPEC-003 per §12
   bar 2. Confirm that is the right call rather than an omission.

### Cost is already handled — do not re-open it

`tokens_total: 197940` was filled by the orchestrator from the Agent result
metadata, and `handback-sync` transcribed it. The builder left it `null` and said
why; that was correct under `metering_source: subagent_tokens` (DEC-013).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**.

Work the checklist in `AGENTS.md` §15 "During verify" — the 8 standard checks plus
this repo's 4 extra, of which these apply here:

- **#9 Did the oracle go red?** There is no decode oracle in this spec, but the
  **lint policy is a gate that must be shown red**. Run
  `./scripts/lint-red-proof.sh` yourself and watch it fail the compile. A
  red-proof you did not personally observe failing is a self-report.
- **#12 Is any new dependency permissive, and not a RAW decoder?** The answer
  should be *zero dependencies*. Confirm `Cargo.toml` and `Cargo.lock` agree.

Additionally, the questions I most want answered:

- Does `rust-version = "1.90"` actually hold? Run
  `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` yourself.
  ⚠ `cargo +1.90.0` without the shim path FAILS — see the toolchain brief.
- Are the five panic-free lints **deny-level on the library** and **allowed only**
  in `#[cfg(test)]` and `src/bin/irr.rs`? A blanket allow anywhere else is a
  rejection.
- Does anything in CI or the README imply the decoder is verified? Per DEC-003 CI
  **cannot** run tier-B tests, so a green badge must not overclaim.
- Is `irr` genuinely absent from the library's public API?

## Out of Scope

- Re-doing the build. If something is wrong, punch-list it; don't fix it silently.
- Any decoding work — SPEC-002 onward.
- Re-litigating the MSRV number. 1.90 is measured-and-conservative by design; the
  true floor is knowingly unmeasured. Lowering it is a separate change.
- The cost entry (already correct).

## Return Criteria — how to hand back

1. Append a **verify** cost session to the spec's `cost.sessions` with a real
   `tokens_total` from your own interface (`/cost` in Claude Code). If your
   platform genuinely cannot report one, write `null` **and say why** — do not
   invent a number (DEC-013).
2. Fill this file's `## Completion` and `handback:` block; set
   `handoff.status: completed`.
3. State the verdict with the SHA you reviewed.
4. If APPROVED, set the spec's `task.cycle: ship` and say so; the orchestrator
   handles the ship cycle (reflection, `complexity_actual`, archive, CHANGELOG).
5. If PUNCH LIST, list each item with file:line and why it matters. Send it back
   to build rather than fixing it yourself — the independence cuts both ways.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Verdict

## ⚠ PUNCH LIST — at `29515ab` (`feat/spec-001-crate-scaffold`)

**Six items, two of them P1.** The build is honest, in scope, and its gates are
genuinely green — I re-ran them rather than reading the report. What green does
*not* yet mean is what the P1 items are about.

Not a ❌: §15's four extra checks all hold as written. Check 9 passes — I ran
`./scripts/lint-red-proof.sh` and watched the compile fail with three errors and
clippy exit 101. Checks 10 and 11 are correctly N/A. Check 12 is zero
dependencies. The P1 items are about what the red *proves*, not whether it
happened, and DEC-006 already records the gap honestly as its own "Negative"
consequence — so this is unfinished mechanism, not a misrepresentation.

**Spec sent back to build**: `just advance-cycle SPEC-001 build --verdict punch-list`.
I fixed nothing (per this handoff's "Out of Scope" and the independence rule).

---

### The punch list

#### PL-1 — P1 — The red-proof does not test the library's policy. The policy can be deleted and all six gates stay green.

`tests/lint_policy_red.rs.disabled:13-19` (the snippet's own `#![deny(...)]`)
· `scripts/lint-red-proof.sh:41-47` · `src/lib.rs:26-32`

This is the question the handoff asked me to answer — *"would it still fail if
the lints were silently removed from `src/lib.rs`?"* — and the answer is **yes,
it still passes, and so does everything else.**

Measured, not reasoned, in a throwaway `git worktree` off `29515ab`. I deleted
`src/lib.rs:26-32` (the whole `#![deny(...)]` block) and added a public
library function that panics on untrusted input:

```rust
pub fn read_u8(buf: &[u8], at: usize) -> u8 { buf[at] + 1 }   // OOB panic + overflow
```

Then ran every gate:

```
CLIPPY   EXIT=0
FMT      EXIT=0
TEST     EXIT=0
MSRV     EXIT=0
DENY     EXIT=0
REDPROOF EXIT=0     <- still reports "policy proven red"
```

Six green gates, a shipped panic on attacker-influenced input, and the
red-proof cheerfully confirming the policy bites. The red-proof's own output
says why — the lint levels it fires on are the snippet's, never the library's:

```
note: the lint level is defined here
  --> tests/lint_policy_red.rs:14:5      <- the snippet's header
```

**Why it matters.** `no-panics-on-untrusted-input` is *blocking*, and
`src/lib.rs:15` states the mechanism explicitly: "enforced mechanically, not
only by review." Today it is enforced by review — DEC-006's "Negative" says so
in as many words ("a verify-cycle reviewer should diff the two when either
changes"). The one artifact the whole crate's safety claim rests on is the one
artifact nothing checks. That is `oracle-must-be-shown-red`'s own failure mode,
one level up: a red-proof that cannot go red for the *right* reason.

**Cheap ways to close it** (build's call, not mine — I'm naming options, not
prescribing):
- One line of shell: have `scripts/lint-red-proof.sh` extract the `#![deny(...)]`
  block from `src/lib.rs` and from the snippet and `die` if they differ. That
  closes *both* halves — drift *and* deletion — because a deleted block fails
  the comparison.
- Or the thorough version: a second proof whose violating code lives inside the
  library crate, so `src/lib.rs`'s own attribute is what rejects it. DEC-006's
  rejected `[lints]`-table alternative is adjacent to this.

#### PL-2 — P1 — The red-proof reports green on *any* non-zero clippy exit, including ones with nothing to do with the lint policy.

`scripts/lint-red-proof.sh:45-47` · `.github/workflows/ci.yml:89-91`

The assertion is "clippy exited non-zero," not "clippy rejected the snippet for
the expected lints." Two measured demonstrations:

**A. Unrelated compile error.** I made the snippet fully lint-clean (no
violations at all) and appended `this is not rust` to `src/lib.rs`. Result:

```
✓ lint policy red-proof: the violating snippet failed to compile as expected (clippy exit 101).
REDPROOF EXIT=0
```

**B. The CI-realistic one — clippy simply unavailable.** With a `cargo` on PATH
that answers ``error: no such command: `clippy` `` (exit 101):

```
✓ lint policy red-proof: the violating snippet failed to compile as expected (clippy exit 101).
REDPROOF EXIT=0
```

So if the `clippy` component fails to install on the runner
(`.github/workflows/ci.yml:89-91`), the one job whose entire purpose is to
prevent manufactured confidence goes green having proven nothing at all.

**Why it matters.** DEC-006's *Validation* section says the mechanism was
"verified: clippy exits 101 with the three expected errors" — the human
measurement checked the error *identities*; the script only checks the exit
code. The mechanism is weaker than the measurement that justified it. Grepping
the captured output for the three expected lint names before declaring success
closes this.

#### PL-3 — P2 — DEC-006's `affected_scope` omits `src/lib.rs`, so the drift it predicts is invisible to the tool meant to surface it.

`decisions/DEC-006-lint-policy-red-proof-mechanics.md:28-31`

DEC-006's "Negative" names `src/lib.rs` as the file that must stay in sync with
the snippet. Its `affected_scope` lists only the snippet, the script and
`ci.yml`. AGENTS.md §15 check 3 names `decisions-audit --changed` as the
mechanism for exactly this. Run against this very diff — which *does* change
`src/lib.rs`:

```
⚠ DEC-006 — ... your change touches:
      .github/workflows/ci.yml
      scripts/lint-red-proof.sh
      tests/lint_policy_red.rs.disabled      <- src/lib.rs absent
```

A future spec editing the library's lint set gets no warning from the decision
written to warn about it. Add `src/lib.rs` to `affected_scope`.

#### PL-4 — P2 — Gratuitous `std` dependency, against a stated Non-Goal.

`src/lib.rs:34` (`use std::fmt;`) · `src/lib.rs:68` (`impl std::error::Error for Error {}`)

SPEC-001's Non-Goals: *"leave the door open by not depending on `std`
gratuitously."* `core::error::Error` has been stable since Rust 1.81 — below the
declared MSRV of 1.90 — so both lines are avoidable today at zero cost.
Verified in the scratch worktree: swapping both to `core::` gives

```
MSRV(1.90) with core:: EXIT=0
CLIPPY     with core:: EXIT=0
TEST       with core:: EXIT=0
```

Two one-word edits. `DEC-002` is still `proposed`, and this is the door it asked
to be left open.

#### PL-5 — P3 — AGENTS.md §6 claims `app.just` matches its command block; two runnable recipes have no line in it.

`AGENTS.md:250-252` · `app.just:33-35` · `app.just:48-49`

- `just lint` runs `cargo clippy … -D warnings` **and** `cargo fmt --check`.
  §6's block has no `cargo fmt --check` line anywhere.
- `just lint-red-proof` → `./scripts/lint-red-proof.sh` is named in §6's prose
  but absent from the block.

Acceptance criterion 8 is "app.just recipes run; AGENTS.md §6's command block
matches them." The recipes run (all seven, verified below); the block does not
fully match. Small, but it is the same class of untrue-AGENTS.md claim this
build was otherwise good about fixing.

#### PL-6 — P3 — `scripts/lint-red-proof.sh` robustness.

`scripts/lint-red-proof.sh:26-27,37` · `.gitignore`

- The script sources `_lib.sh` (which defines `REPO_ROOT`) but never uses it —
  `DISABLED_SNIPPET`, `LIVE_SNIPPET`, the `cp` and the trap's `rm` are all
  CWD-relative. Run from `scripts/`, it dies on the `[ -f ]` check: loud, not
  dangerous, but inconsistent with its siblings. A `cd "$REPO_ROOT"` fixes it.
- `tests/lint_policy_red.rs` is not gitignored. The `trap … EXIT` handles the
  normal and `die` paths correctly (verified — see below), but a SIGKILLed run
  leaves the file, where a `git add -A` would commit a file that permanently
  reds `cargo test` and `cargo clippy`. One `.gitignore` line is cheap
  belt-and-braces.

---

### Flags (not blocking — §15 check 7 says flag, don't block)

- **The design cycle has no `cost.sessions` entry.** The spec carries only the
  build entry. §4: "Every cycle on a spec appends a session entry", with
  design/ship "null with a 'main-loop, not separately metered' note." Not the
  builder's omission — the orchestrator's.
- **CI has never actually run.** `git ls-remote --heads origin
  feat/spec-001-crate-scaffold` returns nothing; the branch is unpushed, so all
  seven jobs are inspection-only. I reduced the residual risk rather than just
  noting it: both external action refs resolve upstream
  (`dtolnay/rust-toolchain` has a real `1.90.0` branch;
  `EmbarkStudios/cargo-deny-action` has a real `v2` tag), the YAML parses to the
  expected seven jobs, and I ran every job's command locally. What is left
  unproven is workflow-level, not command-level. Watch the first push.

---

### What I verified myself (I re-ran; I did not read the build report as evidence)

Both items the handoff said to do myself:

- **`./scripts/lint-red-proof.sh` — personally observed failing.** Three errors
  (`arithmetic_side_effects` at snippet:24, `indexing_slicing` at snippet:24,
  `unwrap_used` at snippet:29), `could not compile ... due to 3 previous
  errors`, clippy exit 101, script exit 0. Cleanup verified: `tests/` holds only
  the `.disabled` file afterwards and `git status` is clean.
- **`~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` → exit 0**
  on `cargo 1.90.0 (840b83a10 2025-07-30)`. The shim path matters exactly as the
  toolchain brief says.

Per acceptance criterion:

| AC | Verdict | Evidence |
|---|---|---|
| 1 | ✅ | `build` / `test` / `clippy -D warnings` / `fmt --check` all exit 0, and again from a clean `git worktree` checkout |
| 2 | ✅ | `rust-version = "1.90"` (`Cargo.toml:10`); 1.90.0 check exit 0; CI pins `dtolnay/rust-toolchain@1.90.0` (`ci.yml:81`), not `stable` |
| 3 | ✅ | `#![forbid(unsafe_code)]` — `src/lib.rs:25` |
| 4 | ✅ | deny at `src/lib.rs:26-32`; the *only* `#[allow]` in the tree is scoped to `#[cfg(test)] mod tests` (`src/lib.rs:71-80`); `src/bin/irr.rs` carries no lint attributes at all (separate crate root — restriction lints default to allow); no `[lints]` table, no blanket allow anywhere |
| 5 | ⚠ | The red IS real and I watched it — but see **PL-1** and **PL-2** for what it does and does not prove |
| 6 | ✅ | `cargo deny check licenses` → `licenses ok`, exit 0; permissive-only allow-list `deny.toml:21-30`; CI job present |
| 7 | ✅ | The library's entire public API is `pub enum Error` (`src/lib.rs:45`). `irr` appears in `src/lib.rs` only inside doc comments |
| 8 | ✅ recipes / ⚠ docs | All seven recipes exit 0 and leave the tree clean; §6 mismatch is **PL-5** |
| 9 | ✅ | README has no badge and no verification claim; `ci.yml:8-11` disclaims explicitly, matching `decisions/DEC-003…:114-119` ("a green CI badge on this repo does **not** mean the decoder is bit-exact. Say so.") |

§15 checks: 1 ✅(w/ punch list) · 2 ✅ · 3 ✅(w/ **PL-3**) · 4 ✅ · 5 ✅ · 6 ✅ ·
7 ⚠ flagged · 8 ✅ · **9 ✅ observed personally** · 10 N/A, correctly · 11 N/A,
correctly · **12 ✅ zero dependencies**.

On check 12: `cargo tree` shows the root crate alone, `Cargo.toml:29-31` declares
both dependency tables empty, and `Cargo.lock` contains exactly one package —
`irradiance` itself. `cargo deny` agrees, with the telling warning
`license-not-encountered: "Zlib" unmatched` — i.e. it found nothing to check.

### The five things the handoff asked me to scrutinise

1. **Is the red-proof fix sound, not merely present?** *Partly.* The false green
   the builder caught is real and the fix genuinely removes it — I confirmed the
   snippet's own `#![deny]` is what fires. The script's failure handling is
   right: the `trap … EXIT` cleans up on the `die` path (verified by neutering
   the snippet — exit 1, clear message, no leftover), and the
   pre-existing-file refusal correctly does *not* delete the file it refused to
   overwrite (trap is armed after that check — deliberate and correct). But the
   fix does not close the two holes in **PL-1** and **PL-2**, and PL-1 is
   precisely the question you asked.
2. **Are the AGENTS.md §5/§6/§7 edits accurate and not overreach?** *Yes, with
   one small inaccuracy (**PL-5**).* Every claim checks out against disk: §5's
   CI job list matches `ci.yml`'s seven jobs; the fuzz deferral matches §12 bar 2
   verbatim and SPEC-003's title; §7's tree markers match reality
   (`tests/corpus/manifest.toml` exists and is correctly unmarked; `tier-a/`,
   `tier-b/`, `fuzz/` are absent and correctly marked PLANNED). The edits were
   confined to sections the change actually falsified. No overreach.
3. **Does `.gitignore` now actually ignore `Cargo.lock`?** *Yes, verified
   mechanically:* `git check-ignore -v Cargo.lock` → `.gitignore:46:Cargo.lock`,
   and `git ls-files | grep -c Cargo.lock` → `0`. Real bug, correctly fixed.
4. **DEC-006 at 0.85.** Reasonable — keep it. It weighs three alternatives with
   stated reasons and states its own weakness honestly. That honesty is what
   made PL-1 and PL-3 findable; the decision is sound, the *mechanism* it
   describes is unfinished.
5. **Was declining the fuzz job right?** *Yes — an omission would have been the
   error.* §12 bar 2 is explicit that fuzz targets arrive with the first parser
   spec and are never retrofitted ("a retrofitted fuzz target tests the shape the
   code already has"). SPEC-001 adds no parser and no input surface; SPEC-003's
   title already carries "plus its fuzz target."

### Cost self-report

- **Tokens (total):** 5242951 — **real, but not from `/cost`.**
- **Estimated USD:** null. I have no verified published list rate for
  `claude-opus-5[1m]`, and §4's "no cache discount" rule applied to a total that
  is ~97% cache-*read* tokens would overstate actual spend by one to two orders
  of magnitude. Inventing one is what DEC-013 forbids.
- **Duration (minutes):** 13 (first→last transcript timestamp, +
  the final turns).
- **Source of the number:** the `usage` objects in this session's own transcript
  (`~/.claude/projects/…-verify-spec-001/14bd8f1c-….jsonl`) — the same data
  `/cost` derives from. `/cost` is a client-side slash command; I cannot execute
  it as the assistant, so I read its source directly rather than reporting null.
  Composition: input 98 · output 48,357 · cache-write
  124,577 · cache-read 5,069,919. It is a **floor** — written
  before the session ends.

⚠ **Do not put this number in a rollup beside the build's 197,940 without
resolving the composition question.** That figure came from an `Agent`-result
`subagent_tokens` whose cache-read content is unknown to me. If it excludes
cache reads and mine includes them, verify looks ~20× build for work that was
much smaller. See the reflection below — this is a process-debt signal, not a
finding against the build.

### Drift and new artifacts

- **New decisions emitted:** none. Every finding is a gap in an existing
  mechanism (`DEC-006`), not a new choice that needed recording.
- **Deviations from spec:** the build's three self-reported deviations
  (`.gitignore` fix, two extra `app.just` recipes, `Display` + `Error` impls)
  are all accurate, all in scope, and all disclosed. I found no undisclosed one.
- **Follow-up work identified:**
  - The `metering_source` composition question above — worth a line in
    `guidance/signals.yaml` as `type: process-debt`. Two cycles of the same spec
    have now produced token numbers that are not comparable, which quietly
    defeats what `just calibration` is for.
  - The design cycle's missing cost session (flagged above).
  - `scripts/handback-sync.sh:105` hard-codes `interface: other` on every
    session it transcribes, which quietly defeats §4's "reports aggregate cost
    by interface". I corrected my own entry to `claude-code` by hand. Template
    friction — `/feedback/` per DEC-000, if the orchestrator agrees.
  - PL-1's underlying shape is general: **the repo has no mechanism that
    notices when a lint policy is deleted.** Whatever closes PL-1 for SPEC-001
    should be the pattern every later spec inherits, since every one of them
    depends on this policy being live.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear; the handoff was unusually good at pointing me at the
   right five things. The one thing that cost real time was *procedural*: it
   told me to report a real `tokens_total` "from `/cost`", but `/cost` is a
   client-side slash command an assistant cannot run. I got a real number
   anyway by reading the transcript `/cost` itself reads — but a verifier
   following the instruction literally would have written `null` and been
   correct to. Worth fixing in the template's cost snippet.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraint, but a missing *link*: `no-panics-on-untrusted-input`
   says "enforced mechanically, not by review", and DEC-006 quietly hands one
   half of that enforcement back to review. Nothing in the repo connects those
   two statements, which is why PL-1 survived a build that was otherwise
   careful. The constraint's `enforcement:` field naming the red-proof — and the
   red-proof checking the constraint's actual carrier — would close the loop.

3. **If you did this task again, what would you do differently?**
   — Reach for the throwaway `git worktree` sooner. I spent the first half
   reading and reasoning about whether the red-proof was sound; the answer
   arrived in about ninety seconds once I actually deleted the lint block and
   ran the gates. Every P1 here came from breaking something on purpose, and
   none of them came from reading. For a cycle whose entire premise is "green
   may not mean what it claims", the default first move should be to try to
   manufacture a false green, not to look for one.
