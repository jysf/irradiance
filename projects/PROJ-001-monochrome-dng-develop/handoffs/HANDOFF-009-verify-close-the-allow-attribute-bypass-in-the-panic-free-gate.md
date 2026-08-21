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
  id: HANDOFF-009
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  tokens_total: 4814757
  estimated_usd: null
  duration_minutes: 30
  branch: feat/spec-006-allow-attribute-gate
  pr: null
  completed_at: 2026-08-20
  notes: "APPROVED at e4a7087 (implementation 618fd6f). All six acceptance criteria met and re-measured independently in a fresh worktree; eight follow-ups, ZERO ship-blocking. Ran check #9 myself: attack planted before the #[cfg(test)] module -> BUILD 0 CLIPPY 0 FMT 0 TEST 0 MSRV 0 DENY 0 REDPROOF 0, NO-ALLOW 101 with both E0453s at src/lib.rs:88, and `just lint-no-allow` 101 on the same tree; honest tree all eight 0. Then tried thirteen more bypasses and every one was caught: inner #![allow] in a module, crate-root #![allow] under the #![deny], #[cfg_attr(all(), allow(...))], #[cfg_attr(not(test), allow(...))], #[expect(...)], #[warn(...)], renamed aliases (clippy::option_unwrap_used / clippy::integer_arithmetic -> E0453 after rename), macro_rules!-generated #[allow], #[allow(clippy::restriction)] (five E0453s), a lib module pulled in via #[path] from tests/, Cargo.toml [lints.clippy] allow, and both group forms (#[allow(clippy::all)] / #[allow(warnings)]) which emit no E0453 but cannot silence the lints either - all still 101, even with the crate-root #![deny] deleted. Two measured results the constraint text does not yet carry: (a) the gate ALONE re-imposes all five lints at forbid level on --lib - with the crate-root #![deny] block deleted and a plain panicking pub fn with no attribute at all, it still exits 101, so for the library it is not dependent on job (1); (b) the gate is scoped by TARGET but not by FEATURE configuration - it runs default features, a no-op at zero features today but live the day DEC-002 puts std behind one. The largest follow-up: the gate's own flags are unpinned. Measured - swap all five -F to -D and the planted attack goes GREEN (0); drop -F clippy::expect_used and plant #[allow(clippy::expect_used)] on a pub fn that expects, and BOTH the no-allow gate and the full clippy gate exit 0, a panic on the public API with all eight gates green again. Nothing in CI notices either edit. That is DEC-009's own thesis one level up - SPEC-001's gate self-tests in CI, this one was proved red once by hand - and it wants its own spec, not another round here (the criteria as written are met and the gate is sound at this SHA). Three disclosed deviations all confirmed accurate and confined: the branch reset (412cb1b is contained in feat/spec-002-corpus-manifest-reader, whose tip has since moved to 112bd80 - nothing lost), the AGENTS.md §6 addition (two hunks, both inside §6, recipe text matches app.just), and CI inlining cargo (single additive hunk at ci.yml:120-165; every other job inlines the same way). constraints-view.sh output re-diffed byte-identical against main. Placement question answered: appending the attack AFTER the test module does trip clippy::items_after_test_module and turns CLIPPY 101 for an unrelated reason - reproduced - so any future automated red-proof for this gate must pin the site. tokens_total is REAL but not from /cost (a client-side slash command the assistant cannot execute): summed 42 deduplicated usage objects in this session's own transcript (~/.claude/projects/-Users-...-irradiance-verify-spec-006/42350191-....jsonl). Composition: input 84 + output 44,391 + cache-write 137,798 + cache-read 4,632,484 (96.2% cache-read). FLOOR - written before the session ends. Same method as SPEC-001's verify-1/2/3 and build-2/3/4; NOT comparable to build-1's 197,940 (token-counts-not-comparable). Cost transcribed by the tool, never by hand: ran `just handback-sync SPEC-006`, which stamped both HANDOFF-007 (build, 5,121,192) and HANDOFF-009 into cost.sessions and set synced_at - hand-appending double-counts (settled on SPEC-001, HANDOFF-004)."
  synced_at: 2026-08-20
---

# HANDOFF-009: Close the allow-attribute bypass in the panic-free gate

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-006` for the **verify** cycle, at
`618fd6f`. Independent session; that independence is the point.

⚠ **ID note:** this is `HANDOFF-009`, renamed by hand. `just new-handoff`
allocated `008`, which `SPEC-002`'s build handoff already holds on its own branch
— the command counts what is visible in the current worktree, so parallel
branches collide. Do not renumber it back.

Context worth having: SPEC-001's equivalent gate took **three** build rounds and
three verifies, each round found insufficient by a reviewer and believed closed by
its author. This spec exists because of what round 3 found.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat

- Honest tree: **all eight gates exit 0**.
- `#[allow(clippy::panic, clippy::expect_used)]` planted on a `pub fn`: seven
  gates green, **NO-ALLOW 101**, `E0453 … overruled by previous forbid` naming
  the exact attribute. It is the only gate that sees it.
- Inner `#![allow]` spelling: also 101.
- `scripts/lint-red-proof.sh` and `src/lib.rs` confirmed untouched
  (`git diff main..HEAD`).

### What deserves scrutiny

1. **Is `--lib` the right scope, and is the claim honest?** The gate covers the
   library target only, and `constraints.yaml:33` was rewritten to say so. SPEC-001's
   F-4 was raised because the previous wording *overstated*. Does it now overstate,
   or **understate**?
2. **Try to bypass it.** `-F` is a compiler-level forbid, so this should be far
   harder than SPEC-001's shell script — but that is an assumption, not a finding.
   Ideas: an `#[allow]` behind a `cfg`; `#[cfg_attr(..., allow(...))]`; an
   `#[expect(...)]` attribute; a lint alias; something in a test or `src/bin/`
   that leaks into `--lib`.
3. **Three disclosed deviations** — CI inlines the cargo invocation instead of
   calling `just` (because `just` isn't on `ubuntu-latest`, found by *executing*
   the extracted YAML rather than reading it); a branch reset; an `AGENTS.md` §6
   addition. Confirm each is accurate and confined.
4. **A gap in my spec the builder found:** it never said *where* to plant the
   attack, and planting after the `#[cfg(test)]` module trips
   `clippy::items_after_test_module`, turning CLIPPY red for an unrelated reason.
   Should the red-proof pin the placement?

### Settled — do not reopen

`DEC-009`'s red-proof (complementary, deliberately untouched) · the `-F`
mechanism (three properties measured at design) · MSRV 1.90 · SPEC-002's work.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per
`AGENTS.md` §15 "During verify".

Run the planted-`#[allow]` attack yourself — check #9. ⚠ Plant it **before** the
`#[cfg(test)]` module (item 4), and mind the `attribute-text-inside-doc-comments`
signal (**N=5**): `src/lib.rs` carries attribute text in its own module docs.

**Label every finding ship-blocking or follow-up.** A defect letting a panic reach
the library is ship-blocking; a sharp edge that fails loudly is a follow-up — file
it and approve. If the gate is sound, say so plainly.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- `scripts/lint-red-proof.sh`; re-litigating `DEC-009`.
- SPEC-002's branch. ⚠ Both branches touch `app.just`; a conflict is expected and
  is the orchestrator's to resolve. Do not pre-reconcile it.

## Return Criteria — how to hand back

1. Verify cost session with a real `tokens_total`; if `/cost` is unavailable, sum
   transcript usage objects and **say so**, with cache-read share.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with SHA, every finding labelled ship-blocking or follow-up.
4. Commit on `feat/spec-006-allow-attribute-gate`; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-006-allow-attribute-gate`, reviewed at **`e4a7087`**
  (implementation commit `618fd6f`). Not pushed, no PR — the handoff said commit
  and do not merge.
- **Completed at:** 2026-08-20
- **All acceptance criteria met?** yes — all six, each re-measured here rather
  than read.
- **Verdict:** ✅ **APPROVED at `e4a7087`**. Eight follow-ups, **zero
  ship-blocking**.
- **Worktree:** reviewed in `irradiance-verify-spec-006`, an isolated
  `git worktree`. Working tree was clean before and after every attack; each run
  restores `src/lib.rs` from a byte-identical backup and `git status --porcelain`
  was re-checked empty between rounds.

---

## The verdict

**✅ APPROVED at `e4a7087`.** The gate is sound. I planted the attack myself and
watched the tree go red on exactly one gate, then spent the rest of the cycle
trying to get past it and failed fourteen times.

### Check #9 — the deliberate fault, run by me

`#[allow(clippy::panic, clippy::expect_used)]` on a `pub fn`, planted **before**
the `#[cfg(test)]` module (per the handoff's ⚠). Every gate run from this
worktree; `NO-ALLOW` is the YAML `run:` block extracted out of
`.github/workflows/ci.yml` by script and executed with `bash`, so what follows is
what CI runs:

```
=== HONEST TREE (e4a7087, working tree clean) ===   === ATTACK PLANTED ===
BUILD      EXIT=0                                    BUILD      EXIT=0
CLIPPY     EXIT=0                                    CLIPPY     EXIT=0
FMT        EXIT=0                                    FMT        EXIT=0
TEST       EXIT=0                                    TEST       EXIT=0
MSRV       EXIT=0                                    MSRV       EXIT=0
DENY       EXIT=0                                    DENY       EXIT=0
REDPROOF   EXIT=0                                    REDPROOF   EXIT=0
NO-ALLOW   EXIT=0                                    NO-ALLOW   EXIT=101  <--
JUST-RECIPE EXIT=0                                   JUST-RECIPE EXIT=101
```

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

Bit-for-bit what the builder pasted, and what design measured. `src/` and
`Cargo.toml` are byte-identical to `main` (`git diff main..HEAD -- src/ scripts/`
is empty), so the honest-tree column *is* the unmodified-`main` column —
criterion 4.

### The bypass attempts — fourteen, all caught

| # | attack | result |
|---|---|---|
| A | `#[allow(clippy::panic, clippy::expect_used)]` on a `pub fn` | **101** — 2×E0453 |
| B | `#![allow(clippy::unwrap_used)]` inside a `mod` | **101** — E0453 |
| C | `#[cfg_attr(all(), allow(...))]` | **101** — E0453 at the inner span |
| D | `#[cfg_attr(not(test), allow(...))]` — true exactly under `--lib` | **101** — E0453 |
| E | `#[expect(clippy::panic, clippy::expect_used)]` | **101** — `expect(...) incompatible with previous forbid` |
| F | `#[warn(clippy::panic, ...)]` (downgrade, not allow) | **101** — `warn(...) incompatible with previous forbid` |
| G | crate-root `#![allow(...)]` placed under the `#![deny(...)]` | **101** — E0453 |
| H | renamed aliases `clippy::option_unwrap_used`, `clippy::integer_arithmetic` | **101** — renamed, *then* E0453 on the new name |
| I | `macro_rules!` expanding to `#[allow(...)] pub fn` | **101** — E0453, with `in this macro invocation` |
| J | `#[allow(clippy::restriction)]` (the group these five live in) | **101** — 5×E0453, one per forbidden lint |
| K | `#[allow(clippy::all)]` | **101** — no E0453 (see F-8), but the lints still fire |
| L | `#[allow(warnings)]` | **101** — same |
| M | lib module pulled in by `#[path = "../tests/…"]` carrying `#![allow]` | **101** — E0453; it is lib code, so it is covered |
| N | `Cargo.toml` `[lints.clippy] panic = "allow"` (+ all five) | **101** — trailing `-F` outranks it |

And two negative controls on the *robustness* of `-F`, both with `src/lib.rs`'s
crate-root `#![deny(...)]` block **deleted**:

| | | |
|---|---|---|
| O | root `#![deny]` gone + plain panicking `pub fn`, **no attribute at all** | **101** |
| P | root `#![deny]` gone + `#[allow(warnings)]` + panic | **101** |

O is the interesting one and it feeds F-3: the five `-F` flags re-impose the
whole policy at forbid level from the command line, so on the `--lib` target this
gate does not depend on anything in the source at all.

**I could not construct a bypass.** `-F` is enforced by rustc before clippy ever
runs a lint pass, and it is enforced on the *attribute*, so cfg-gating it,
renaming it, generating it from a macro, hiding it in an included file or in
`Cargo.toml` all fail the same way. This is a genuinely different quality of gate
from SPEC-001's shell script, and the design's assumption that it would be
harder to beat is now a finding, not an assumption.

---

## Punch list

**Eight items. All follow-up. None ship-blocking.** Nothing here lets a panic
reach the library at this SHA.

### F-1 — the gate's own flags are unpinned (follow-up, highest priority)

`.github/workflows/ci.yml:162-165` · `app.just:70-73`

The five `-F` flags *are* the mechanism, and nothing in CI asserts they are still
five, or still `-F`. Measured, with the attack planted:

```
gate as committed (-F ×5)                          EXIT=101
all five downgraded -F -> -D                       EXIT=0     <-- hole reopened
-F clippy::expect_used dropped, attack allows only it:
    NO-ALLOW                                       EXIT=0
    full CLIPPY (--all-targets --all-features -D warnings)  EXIT=0
```

That second case is SPEC-006's original hole restored in full — a panicking
`pub fn` on the public API with **all eight gates green** — reachable by deleting
one flag from two files. Nothing in the repo notices.

This is DEC-009's own thesis one level up. `lint-policy-red-proof` exists because
"a lint policy that has never rejected anything is not a policy"; the same
sentence now applies to `lint-policy-no-allow`, which was proved red **once, by
hand, in prose**, while the gate it replaced self-tests on every push. The
asymmetry is the finding.

**Why this is not ship-blocking:** the criteria SPEC-006 actually set are met,
the gate is measured sound at `e4a7087`, and the gap is regression-durability —
which is what a spec is for, not another round of this one. SPEC-006 itself was
split out of SPEC-001 on exactly that reasoning. Suggest a spec before STAGE-001
closes: a script on the DEC-009 pattern (control + mutation) that plants the
attack in a temp-dir copy and asserts the *committed* invocation rejects it.

### F-2 — the scope claim covers targets but not feature configurations (follow-up)

`guidance/constraints.yaml:33`

`SCOPE: the --lib target only … no other target (a future example, bench or
second bin) is covered` enumerates **targets**. The gate also runs under
**default features** — no `--all-features`. Confirmed a no-op today
(`cargo metadata --no-deps` → `"features": {}`), and the builder disclosed it as
a follow-up, but criterion 5 asks the field to say what is enforced "no more, no
less", and the feature axis is the half that is missing. It goes live the day
DEC-002's `std`-behind-a-default-feature lands. One clause.

### F-3 — the enforcement text understates job (2) and overstates the pairing (follow-up)

`guidance/constraints.yaml:33`

"Two CI jobs, and the PAIR is the guarantee — neither alone is." For the
repository as a whole that is fair: job (1) is what keeps the policy *in the
source*, where a developer's bare `cargo clippy`, the `--all-targets` run, and
anyone reading `src/lib.rs` all see it. But for the **library target** it is not
accurate — attack O above deletes the entire crate-root `#![deny(...)]` block,
plants a panicking `pub fn` with no attribute anywhere, and the no-allow gate
still exits **101**. Job (2) is not merely an escape-hatch detector; it is a full
command-line re-imposition of the five-lint policy on `--lib`, immune to any
in-source level change.

The error direction is conservative — the text under-claims safety, which is the
right way to be wrong, and is why this is a follow-up. Worth one sentence, both
because it is a real redundancy and because it names what job (1) is actually
still for.

### F-4 — `signals.yaml` is now stale on the hole this spec closed (follow-up, ship step)

`guidance/signals.yaml:156-176` (`allow-attribute-exits-the-panic-policy`,
`status: accepted`, `disposition_at: project-close`)

It still reads as a live hole; it still predicts the mechanism will be "a gate on
`allow(` outside `#[cfg(test)]` and `src/bin/`" — the grep-shaped approach
SPEC-006 deliberately rejected, and the note even warns it would have to heed
`attribute-text-inside-doc-comments`; and it still says "constraints.yaml:33
currently reads as a stronger guarantee than holds (F-4) and should be softened
in the same change", which this branch did. Close it at ship naming SPEC-006 and
`-F`/E0453, and record that the predicted mechanism was rejected — that is the
more useful half of the entry.

Nit, same file: `attribute-text-inside-doc-comments`'s `evidence:` field
(`guidance/signals.yaml:135`) still says **N=3** while its own `notes:`
(`:143`) and HANDOFF-009 both say **N=5**. One-line reconciliation.

### F-5 — `cost.sessions` was empty; transcribed, and the ownership rule contradicts itself (follow-up; verify check 7 says flag, not block)

`projects/…/specs/SPEC-006-…md` `cost.sessions` was `[]` with
`totals.tokens_total: 0`, while HANDOFF-007 carried a real 5,121,192 and
`synced_at: null`.

**Fixed by the tool, not by hand:** `just handback-sync SPEC-006` now transcribes
both cycles (build 5,121,192 + verify 4,814,757 = 9,935,949, `session_count: 2`)
and stamps `synced_at` on both handoffs. I did **not** hand-append my own entry —
SPEC-001 settled that: `handback-sync` is idempotent via `synced_at`, so a
hand-written session double-counts the moment the tool runs (HANDOFF-004's punch
list, and its `:399` "Orchestrator's, not the builder's"). Re-running it is a
no-op. `cost-audit.sh` would have passed regardless, because it only enforces on
*shipped* specs.

What remains is doc debt: `AGENTS.md` §15 "During build" step 3 and "During
verify" ("Append a verify cost session entry before returning the verdict") both
tell the executing agent to write `cost.sessions` directly, which contradicts the
handoff front-matter's DEC-013 contract that the handback is the return path and
the tool does the transcription. Two owners, one field — and the field is the one
`just calibration` reads.

### F-6 — the red-proof should pin the planting site (follow-up) — *answers question 4*

`projects/…/specs/SPEC-006-…md` `## Failing Tests`

Reproduced: appending the same attack **after** the `#[cfg(test)]` module gives

```
CLIPPY     EXIT=101   error: items after a test module (clippy::items_after_test_module)
NO-ALLOW   EXIT=101
```

— CLIPPY red for a reason that has nothing to do with the `#[allow]`. The
headline evidence would have read "six green gates, two red" and *understated*
the hole, which is the same class of manufactured result DEC-009 was written to
kill. The builder found this and planted correctly; the spec's snippet still does
not say where.

**Yes, pin it** — but pin it in whatever F-1 produces, not only in prose: an
automated proof that plants at the end of the file will fail for the wrong reason
and, worse, will still be *red*, so it will look like it is working.

### F-7 — what neither gate can see (follow-up, informational)

Measured on the honest tree, no attribute anywhere:

```rust
pub fn boom_p(v: &[u8], n: usize) -> (&[u8], &[u8]) {
    if n > 9000 { unreachable!("nope") }
    assert!(n < v.len(), "bad n");
    v.split_at(n)
}
```

```
NO-ALLOW gate   EXIT=0
full CLIPPY     EXIT=0
```

Three panics on untrusted input, both gates green. This is the five-lint
**policy's** scope (SPEC-001), not SPEC-006's, and `constraints.yaml:33` already
disclaims it correctly — "neither job proves any code is panic-free". Recording
it because SPEC-003's parser is where it starts to matter: `clippy::unreachable`
/ `clippy::todo` / `clippy::unimplemented` are cheap additions to the policy; the
std slice-method panics (`split_at`, `copy_from_slice`, `chunks(0)`) are not
lintable at all and are exactly what the §12 bar-2 fuzz targets are for.

### F-8 — one clause in the CI comment is true only of specific-lint attributes (nit)

`.github/workflows/ci.yml:140-142`: "this fires on the ESCAPE HATCH itself,
whether or not the code beneath it actually panics."

Exact for anything naming a policy lint or the `restriction` group (attacks
A–J, M, N). Not exact for `#[allow(clippy::all)]` / `#[allow(warnings)]`:
measured, `#[allow(clippy::all)]` on **clean** code exits **0** with no E0453,
because neither group contains these five `restriction` lints. They are also not
escape hatches — attacks K, L and P show the lints still fire through them, even
with the crate-root `#![deny]` gone — so nothing leaks. Half a sentence if anyone
edits that block.

---

## The four questions I was asked

**1. Is `--lib` the right scope, and does `constraints.yaml:33` state exactly
what holds?**

`--lib` is right, and for a better reason than the one recorded. The recorded
reason is that it excludes the two sanctioned exceptions with no per-site
special-casing — confirmed: honest tree exits 0 *with* `src/lib.rs:89`'s
five-lint `#[allow]` on the test module still in place, and `--all-targets`
on the honest tree exits 101 on precisely that attribute. The better reason is
that `--lib` is the shipped surface: `src/bin/irr.rs` is not a library path
(measured: an `#[allow]`-ed `panic!()` added to it leaves both the no-allow gate
and full CLIPPY at 0 — the exception is real and unguarded, by design), and
`#[cfg(test)]` code is not compiled into it.

On the wording: it **understates** in one place (F-3 — the gate alone covers the
library) and **overstates by omission** in one (F-2 — scoped by target, silent on
features). It does *not* repeat F-4's mistake: "neither job proves any code is
panic-free" is measured true (F-7), and every mechanical claim in the field
— E0453, the escape hatch failing whether or not the code panics, the two
exceptions, no other target — checks out. Net: honest, with two clauses missing.

**2. Can it be bypassed?** Not by me, in fourteen attempts across every spelling
of a lint-level attribute rustc accepts, plus `Cargo.toml`, macro expansion and
a `#[path]`-included file. See the table. The one way in is to weaken the gate
itself (F-1), which is a different attack and is the one worth spending a spec on.

**3. The three deviations.** All accurate, all confined.

- *Branch reset* — `412cb1b` is contained in `feat/spec-002-corpus-manifest-reader`
  (`git branch --contains`), nothing lost. The handback says it is that branch's
  *tip*; it was, and has since moved to `112bd80`. Harmless staleness.
- *`AGENTS.md` §6* — two hunks, both inside §6 (`@@ -250,10 +250,11 @@` and
  `@@ -281,6 +282,16 @@`), and the block text matches `app.just:71-73` modulo
  recipe indentation, exactly as `lint` and `lint-red-proof` do. SPEC-001
  acceptance criterion 8 is intact.
- *CI inlines cargo* — accurate and correct. `ci.yml` gained one additive hunk
  (`@@ -116,3 +116,50 @@`); every other job in the file inlines its command the
  same way, including `lint-policy-red-proof`. `just` is not on `ubuntu-latest`.
  Finding it by *executing* the extracted `run:` block rather than reading it is
  the §12 behavioral pre-flight working as intended — I re-extracted and executed
  the same block for every measurement above rather than retyping it.

Also re-verified: `scripts/lint-red-proof.sh` and `src/lib.rs` untouched
(`git diff main..HEAD -- src/ scripts/` empty); `scripts/constraints-view.sh`
output byte-identical between `main`'s and this branch's `constraints.yaml`,
re-diffed here, so the line-based parser is undisturbed.

**4. Should the red-proof pin placement?** Yes — see F-6. And the question worth
asking behind it is F-1: there is no automated red-proof for this gate to pin
placement *in*.

---

### Cost self-report

- **Tokens (total):** **4,814,757** — real, but **not from `/cost`**.
- **Estimated USD:** null (no rate configured; every session on this project has
  recorded `estimated_usd: null`).
- **Duration (minutes):** ~30.
- **Source of the number:** summed the `usage` objects in this session's own
  transcript — the same data `/cost` derives from. `/cost` is a client-side slash
  command the assistant cannot execute as a tool. Transcript:
  `~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance-verify-spec-006/42350191-f9e4-4121-bca2-9b4b876ef5bb.jsonl`,
  deduplicated by `message.id`.
- **Composition:** input 84 + output 44,391 + cache-write 137,798 + cache-read
  4,632,484 over 42 deduplicated assistant turns. **Cache-read share: 96.2%** —
  only ~44,475 tokens (0.9%) are fresh input + output.
- ⚠ It is a **FLOOR** — written before the session ends.
- ⚠ `token-counts-not-comparable`: same method as SPEC-001's verify-1/2/3 and
  build-2/3/4, and comparable to those. **Not** comparable to SPEC-001 build-1's
  197,940 (an Agent-result `subagent_tokens` figure of unknown cache composition).
- **Transcribed by `just handback-sync SPEC-006`**, not by hand — both cycles are
  now in `cost.sessions` (build 5,121,192 + verify 4,814,757 = 9,935,949) with
  `synced_at` stamped on both handoffs. See F-5.

### Drift and new artifacts

- **New decisions emitted:** none. The mechanism was decided and measured at
  design and transcribed at build; verify measured it independently and agrees.
  Nothing here is a new decision.
- **Deviations from the verify brief:** none. I did not touch
  `scripts/lint-red-proof.sh`, did not re-litigate DEC-009 or the `-F` mechanism,
  did not fix anything, and did not go near `app.just`'s conflict with SPEC-002 —
  the only `app.just` change on this branch is the one the builder committed,
  left exactly as found for the orchestrator to resolve.
- **`just decisions-audit --changed main`** flags DEC-000 (`AGENTS.md`,
  `guidance/constraints.yaml`) and DEC-009 (`.github/workflows/ci.yml`). Both
  re-read: no drift. DEC-009 governs `lint-policy-red-proof`'s mechanics and that
  job is byte-identical to `main`; the new job sits beside it, which is what
  DEC-009's own "structurally cannot" paragraph asks for.
- **`just validate`** ✓ 6 artifacts · **`just status`** clean · **`cost-audit`** ✓.
- **Follow-up work identified:** F-1 (a red-proof for the forbid gate — the one
  worth a spec), F-2/F-3 (two clauses in `constraints.yaml:33`), F-4 (close the
  signal at ship, plus the N=3/N=5 nit), F-5 (`handback-sync`, and the §15/DEC-013
  ownership contradiction), F-6 (pin the planting site), F-7 (policy coverage
  before SPEC-003's parser), F-8 (half a sentence in a CI comment). Carried
  forward from the builder and confirmed still open: `src/lib.rs:34-35`'s module
  doc now reads as if the gap is unclosed, and `guidance/toolchain-brief.md`'s
  `+stable = 1.97.0` — this host now measures `cargo 1.97.1` / `clippy 0.1.97` /
  `rustc 1.97.1` on the default path, so that line wants re-measuring rather than
  trusting either recorded value.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing, and that is worth recording: the handoff pre-empted the two things
   that would have cost a loop (plant before the test module; the gate is
   `--lib`, so don't reach for `--all-targets`), and pre-reconciled the results I
   would otherwise have spent the first third of the cycle reproducing. The only
   thing I had to work out for myself was that "try to bypass it" has two
   distinct meanings — get past the gate, or get at the gate — and the second one
   is where the finding was.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `oracle-must-be-shown-red` (`guidance/constraints.yaml`, severity blocking).
   SPEC-001 applied it to a *gate* rather than an oracle and got a self-testing
   script in CI out of it; SPEC-006 built a gate of the same kind, and neither
   the spec nor the handoff asked whether that constraint reaches it. Criterion 3
   says "shown RED … and that proof ships with the change", which a handback
   satisfies — so nothing was violated. But had the constraint been listed in
   `references.constraints` (it is `[]`), F-1 would have been a design question
   instead of a verify finding.

3. **If you did this task again, what would you do differently?**
   — Attack the gate's configuration before attacking the code. I spent most of
   the cycle on fourteen ways to slip an attribute past `-F` — all caught, which
   is a real result but a predictable one once the first three fail the same way.
   The two-minute experiment that changed the verdict's shape was editing the
   flags themselves. For any gate whose whole mechanism is its arguments, that is
   the first experiment, not the last.
