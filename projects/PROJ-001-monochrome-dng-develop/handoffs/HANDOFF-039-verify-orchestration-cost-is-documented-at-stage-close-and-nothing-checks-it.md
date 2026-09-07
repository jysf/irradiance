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
  id: HANDOFF-039
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5[1m]       # CORRECTED from the tier_map.verify prediction
                                    # (claude-opus-5). This session's system prompt reports
                                    # model id `claude-opus-5[1m]`, and all 71 unique
                                    # assistant turns in this session's transcript carry
                                    # message.model == claude-opus-5. The [1m] suffix is the
                                    # 1M-context variant and is the precedent HANDOFF-030 set.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: PATCH-002

project:
  id: PROJ-001
  stage: STAGE-XXX
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
  tokens_total: 10300000           # REAL combined count — what cost-audit reads
  estimated_usd: 25.02             # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: fix/patch-002-orchestration-cost-has-no-gate
  pr: null
  completed_at: 2026-09-06         # YYYY-MM-DD
  notes: "Verdict PUNCH LIST, not approved: 2 ship-blockers. SB-1 - the patch claims "no DEC, this decides nothing new" but DEC-013 section 5 records "Warn-only, no gate, no view yet: capture first", and that same sentence is still live in projects/_templates/stage.md:37 and in all five STAGE-00N files, so STAGE-003/004/005 will block at close on a field their own front matter tells the author is never a gate. SB-2 - the awk detector toggles its front-matter fence on EVERY bare --- line and never clears in_oc when the front matter closes, so a shipped stage with an empty orchestration_cost passes green if its markdown BODY contains a --- rule followed by a line matching "- tokens_total: <digits>"; reproduced end to end against STAGE-002 (cost-audit rc=0, JSON offenders 0). Latent today - no stage body currently has a bare --- - but it is the same attribute-text-inside-doc-comments class the patch was written to prevent. All three claimed mutations reproduced (file changed AND ran AND output changed), plus the historical false first draft reconstructed and shown green with the naive grep implementation, confirming the comment-preserving injection is load-bearing. Two implementation mutants SURVIVE the red-proof (FU-1, FU-2). Grandfathering is load-bearing (removed it, STAGE-001 fails by name on both surfaces) and its justification holds, now measured rather than asserted: session 7cbf62d2 spans 2026-08-16 to 2026-08-22 (443 turns, 235M) and d43bad0e spans 2026-08-22 to 2026-09-05 (528 turns, 268M), so two sessions each straddle a stage boundary and no per-stage split is observable. Gates: I ran the 12-item UNION of the two cited enumerations plus lint-ci and the new cost-audit-red-proof - the count is signal the-gate-count-is-not-defined-anywhere and not mine to resolve. clippy 0.1.98 (88d9e12ae1 2026-08-18) via the PATH-prefixed +stable invocation, matching CI. CI observed green 9/9 on 705c784, run 34023570708, and the new step was confirmed to EXECUTE via the jobs API step list plus its own success line in the log at 09:03:47.887Z. Cost measured from this session's own transcript abef176a-32e5-4efe-9de4-7cd820a77e73.jsonl, identified by scratchpad-dir uuid, deduped by message.id: floor 8,522,559 tokens over 71 unique assistant turns, 97.6% cache-read, priced per-component at published Opus rates ($15/$75/$30-write/$1.50-read) = $20.85, both figures rounded up 20% to cover the turns writing this handback."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-039: Verify PATCH-002 — the stage orchestration-cost gate, at `705c784`

## Delegation Summary

Verify `PATCH-002` at **`705c784`** on `fix/patch-002-orchestration-cost-has-no-gate`
(pushed, not merged; `main` at `781930f`). CI 9/9 on that SHA, run `34023570708`.

⚠ **The orchestrator wrote this patch.** Build normally goes to a separate
session here; it did not. **Review it as work by someone who was also grading
it** — that is the whole reason this verify is worth its cost.

## What the patch does

`cost-audit` now fails when a stage with `status: shipped` has an empty
`orchestration_cost`. The template has said *"THE ORCHESTRATOR FILLS THIS"* since
2026-08-15 and nothing checked it: `STAGE-001` shipped 2026-08-22 with
`sessions: []` and no gate, report or status line noticed for fifteen days.
`STAGE-002`'s close on 2026-09-06 is the first time the field was ever filled.

Not a rounding error: `STAGE-002` measured **~84.2M** tokens of orchestration
against **187.0M** of delegated spec cost — roughly **31 %** of the stage, and
spend no spec's `cost.sessions` would ever record.

## ⚠ Attack the red-proof first — I already shipped one false version of it

Its first draft was wrong, and how it was wrong is the most useful thing here.
The injection wrote a bare `sessions: []`, which **also deleted the template's
commented example** (`# - tokens_total: N`). With that text gone, even a naive
`grep -q tokens_total` implementation *passed* the proof — nothing left to
false-match. That is AGENTS.md §16's *"the obvious test exercises the wrong
path"*, verbatim. The injection now reproduces the real shipped shape, comment
included, and asserts the comment survived.

**Do not take my word that it is fixed.** Three mutations are claimed caught:

| mutation | claimed |
|---|---|
| `stage_has_orchestration_cost` always returns "filled" | red-proof FAILS |
| the naive `grep -q tokens_total` implementation | red-proof FAILS |
| the reason string emptied | red-proof FAILS |

Reproduce all three, then **find a fourth I did not think of.** Candidates: an
`orchestration_cost` block absent entirely rather than empty; an entry whose
`tokens_total` is `null`; a real YAML entry sitting *outside* the block;
`status: shipped` written with odd spacing or quoting.

## Your own checks

1. **Is the grandfathering honest, or hiding a live failure?** `STAGE-001` is
   exempt via `STAGE_ORCH_COST_GRANDFATHERED`. Confirm it is load-bearing
   (remove it — STAGE-001 must fail) *and* justified: is reconstructing
   STAGE-001's orchestration genuinely impossible, or merely inconvenient? §4
   says *"a null here is honest; a guess is not"*, and I claimed reconstruction
   would be a guess. **Test that claim rather than accepting it.**
2. **Does the gate fire on the states that matter?** It keys on
   `get_stage_status = "shipped"`. What about `cancelled`? A stage file outside
   `projects/*/stages/`? Judge the scope, not just the code's match to it.
3. **Does `find_all_stages` find every stage?** `-maxdepth 1` under
   `projects/*/stages` — confirm that matches where `just new-stage` puts them.
4. **The JSON surface.** `--json` emits `missing_cost: ["orchestration"]`.
   Confirm it is well-formed and that the human line and JSON cannot drift — I
   collapsed them to one source *because* a mutation showed they could.
5. **Does the new CI step actually execute?** I verified by reading the job log
   (step `cost-audit goes red on an unfilled stage orchestration_cost`, printing
   its own success line). Confirm independently — a step that exists in YAML and
   never runs is this patch's own subject.
6. **`shellcheck`.** Clean on the new script, unchanged in count on the two
   edited ones. Re-run it; I am not confident I checked every warning class.

## Context

- **Patch:** `projects/PROJ-001-monochrome-dng-develop/patches/PATCH-002-*.md` —
  its Problem section carries the measurements and the near-miss that shaped the
  detector.
- Changed: `scripts/_lib.sh` (three helpers), `scripts/cost-audit.sh` (a third
  loop), `scripts/cost-audit-red-proof.sh` (new), `.github/workflows/` (one
  step), `app.just` + `AGENTS.md` §6.
- `DEC-013` §5 — the rationale the gate enforces. **No new `DEC-*`**: this
  decides nothing, it enforces something already decided.
- Constraints: `cost-captured-per-cycle`; and `oracle-must-be-shown-red` by
  analogy — this repo's rule is that a gate ships proven red.

## Out of Scope

- **Backfilling `STAGE-001`.** If you judge the grandfathering unjustified, raise
  it as a finding; do not fill it.
- The gate-count ambiguity and `handback-sync`'s truncation — both filed signals.
- Opening or merging the PR; running `handback-sync`.

## Return Criteria

1. **Gates, run by you**, pasted, clippy version asserted, and **say which list
   you ran** — the count is ambiguous and that is a filed signal, not yours to
   resolve.
2. **Observe CI green on the SHA you approve.**
3. **All three claimed mutations reproduced**, plus your attempts at a fourth.
   Each: file changed **and** ran **and** *output changed*.
4. ⚠ **Mutate in an isolated copy.** I lost this very patch to `git checkout --`
   mid-mutation and recovered it from a scratch backup — SPEC-010's exact
   failure. There is a `wip` commit in the history for that reason.
5. Handback with a real `tokens_total`, deduped by `message.id`, per-component,
   **rounded up ~20 %**. ⚠ **`notes:` on ONE PHYSICAL LINE** — `handback-sync`
   truncates multi-line scalars and leaves front matter unparseable while every
   gate reports green.
6. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
7. Findings `SB-N`/`FU-N` from `FU-1` with proposed §15 dispositions.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

---

## Handback

*Filled in by the receiving agent.*

### Execution notes

- **Branch / PR:** `fix/patch-002-orchestration-cost-has-no-gate` @ `705c784`. No PR
  opened, no merge, no `handback-sync` — all out of scope per the handoff.
- **Completed at:** 2026-09-06
- **All acceptance criteria met?** No — see `SB-1` and `SB-2`.
- **Verdict:** ⚠ **PUNCH LIST** at `705c784`.

### Gates — which list I ran, and why

`the-gate-count-is-not-defined-anywhere` is an open signal and the handoff says it
is not mine to resolve, so I did what `SPEC-015`'s verify did: **I ran the UNION of
the two enumerations the signal cites**, plus `lint-ci` and PATCH-002's own new
red-proof. That is **14 invocations**; I am reporting the number I ran, not
adjudicating what "the gate count" means.

⚠ `just lint` and `just lint-red-proof` invoke a bare `cargo clippy`, and this
machine's default rustup toolchain is now **nightly, which has no clippy component**
— so both fail here for an environment reason, not a code one. `lint-ci`'s
`PATH=`-prefixed `+stable` form is unaffected, and re-running the red-proof under
`RUSTUP_TOOLCHAIN=stable` passes. This is a *fifth* instance of the `+toolchain`
trap family and is raised as `FU-5`; `AGENTS.md` §6 still describes the PATH clippy
as "Homebrew's 0.1.97", which is no longer what is there.

```
clippy 0.1.98 (88d9e12ae1 2026-08-18)     ← PATH="$HOME/.cargo/bin:$PATH" cargo +stable
cargo  1.98.0 (797e8a9bc 2026-08-05)        (identical to CI's dtolnay/rust-toolchain@stable)
shellcheck 0.11.0

fmt                    rc=0
clippy / lint-ci       rc=0     PATH-prefixed +stable, --all-targets --all-features -- -D warnings
typecheck              rc=0
build                  rc=0     --release
corpus-status          rc=0     7/7 tier-B present — no tier-B test skipped
test                   rc=0     152 passed, 0 failed, 0 ignored (9 binaries)
msrv                   rc=0     rustup run 1.90.0 cargo check --all-targets --all-features
deny                   rc=0     library graph
deny-fuzz              rc=0     fuzz/Cargo.toml graph
lint-no-allow          rc=0
lint-red-proof         rc=0     under RUSTUP_TOOLCHAIN=stable (see FU-5)
cost-audit             rc=0
cost-audit-red-proof   rc=0     PATCH-002's new gate
decisions-index --check rc=0
validate               rc=0     18 artifacts
decisions-audit --changed  no DEC governs the paths this patch touched
```

`shellcheck` (handoff check 6), re-run at **both** default and `-S style` severity:

```
                                  before(781930f)  after(705c784)
scripts/_lib.sh                          2               2      (pre-existing SC2034 BOLD/DIM, line 19)
scripts/cost-audit.sh                    0               0
scripts/cost-audit-red-proof.sh          —               0      (new)
```

Nothing in the new `_lib.sh` block (lines 1039–1096) emits at style severity either.
Claim 6 confirmed.

### CI, observed

Run `34023570708`, head `705c784e59af2def8f798d064be1bc99dc3d966b`, **success, 9/9
jobs**. Handoff check 5 (*does the new step actually execute?*) confirmed two ways,
independently of reading the orchestrator's log:

1. The **jobs API step list** for `cost-capture audit` shows step 4,
   `cost-audit goes red on an unfilled stage orchestration_cost`, with
   `started_at 09:03:47Z` / `completed_at 09:03:47Z` / `conclusion success` — a
   skipped step carries no timestamps.
2. The job log carries the script's **own terminal success line** at
   `09:03:47.8877266Z`, which is only printed after step 4 of the proof. A step that
   existed but no-opped could not produce it.

### The red-proof, attacked first

**All three claimed mutations reproduced.** Each one: file changed (md5 before/after),
red-proof *ran*, output *changed*. Every mutation was applied in a disposable
`git clone` of this repo at `705c784` under the session scratchpad — the working
tree was never written to (handoff criterion 4).

| # | mutation | file | result |
|---|---|---|---|
| `M1` | `stage_has_orchestration_cost` short-circuits to `return 0` | `_lib.sh:1080` | ✅ caught — `rc=1`, *"gate did NOT go red … the gate is decorative"* |
| `M2` | body replaced with `grep -q tokens_total "$file"` | `_lib.sh:1078-1094` | ✅ caught — `rc=1`, same message |
| `M3` | `reason="orchestration"` → `reason=""` | `cost-audit.sh:105` | ✅ caught — `rc=1`, *"went red but never named the stage or the field"* |

**And the near-miss itself reproduced.** I rebuilt the false first draft — injection
writes a bare `sessions: []`, comment-survival assertion removed — and ran it against
the `M2` naive implementation. It printed **✓ and exited 0**. The account in the
Problem section is true, and keeping the template comment in the injection is
load-bearing, not decoration.

**The fourth, and it is three.** `M4a` and `M4b` are mutants that **survive**;
`M4c` is a defect in the gate itself.

| # | mutation | result |
|---|---|---|
| `M4a` | delete `if (substr(line,1,1) == "#") next` — the line the function's own comment calls the anti-trap | ❌ **SURVIVES** (`rc=0`). Differential over 10 synthetic shapes + all 5 real stage files: **byte-identical verdicts**. The guard is unreachable — after `sub(/^[ \t]+/,"")` a line cannot both start with `#` and match `^- `. The real defense is the `^- tokens_total:…[0-9]+` anchor. → `FU-1` |
| `M4b` | `printf … "$name"` → `printf … "a stage"` | ❌ **SURVIVES** (`rc=0`). The proof greps only `missing cost on: orchestration`; it never asserts the stage name, while its success line claims *"REJECTED **by name**"* and its failure message says *"never named the stage or the field"*. → `FU-2` |
| `M4c` | data: unfilled template **plus** a `---` rule in the markdown **body** followed by `- tokens_total: 84200000` | ❌ **gate reports GREEN**. → `SB-2` |

### Grandfathering — tested, not accepted

**Load-bearing: yes.** Cleared `STAGE_ORCH_COST_GRANDFATHERED` in the isolated copy →
`cost-audit` exits 1, names `STAGE-001-foundations-…` with reason `orchestration` on the
human line, and emits `"violations":[{"artifact":"STAGE-001-…","missing_cost":["orchestration"]}]`
on the JSON surface. Both surfaces, one source.

**Justified: yes — and now measured rather than asserted.** The claim was that
STAGE-001's orchestration "ran across a week of sessions with no per-stage boundary
recorded". I tested it against the transcripts instead of taking it:

- The raw material is **not** missing. Transcripts covering `2026-08-16 → 2026-08-22`
  exist locally in the same format, with the same `usage` fields and `message.id`
  dedup that produced STAGE-002's number.
- But **no stage boundary exists inside them.** Session `7cbf62d2` runs
  `2026-08-16T06:22 → 2026-08-22T04:50` — six days, 443 unique turns, 235.5M tokens,
  one session covering the scaffold, `SPIKE-001`, `DEC-000` and the whole of STAGE-001.
  Session `d43bad0e` runs `2026-08-22T04:53 → 2026-09-05T17:00` — 528 turns, 267.7M
  tokens — and **straddles the STAGE-001 → STAGE-002 boundary**.
- Worse, the era mixes lanes: several large `claude-sonnet-5` sessions sit in the same
  project directory in that window and are delegated build/verify work whose tokens are
  **already** recorded in the specs' `cost.sessions`. Summing the window would
  double-count, and separating it would be a judgment, not a measurement.

So `AGENTS.md` §4's *"a null here is honest; a guess is not"* applies and the exemption
stands. One correction to the stated reason, for the record: the transcripts are
**readable, not lost** — the obstacle is attribution ambiguity, which is exactly what
the code comment says ("no per-stage boundary recorded"), so the comment is accurate as
written. Recording the measurement here so the next reader does not have to re-derive it.

### Scope and surface checks

- **Check 2 — states.** `cancelled` is **not** audited: I set STAGE-002 to
  `status: cancelled` with an empty block and `cost-audit` stayed green. Defensible —
  a cancelled stage never ran a close ritual — but it is a hole worth naming (`FU-3`).
  A second, sharper one: `status: "shipped"` **in quotes** makes `get_stage_status`
  return `"shipped"` *with* the quotes, so the stage is skipped entirely and the gate
  never looks at it (`FU-4`).
- **Check 3 — `find_all_stages`.** Confirmed against the writer, not assumed:
  `scripts/new-stage.sh:26` writes `${PROJECT_DIR}/stages/${STAGE_ID}-${SLUG}.md`, flat.
  There is no `stages/done/` anywhere in the tree and no `archive-stage` script, so
  `-maxdepth 1` matches every place a stage can be. ✅
- **Check 4 — JSON.** `--json` output parses with `json.load` and carries
  `missing_cost: ["orchestration"]`. Drift is genuinely impossible: under `M3` the
  human line loses its reason **and** the JSON degrades to `"missing_cost":[]` in the
  same run. One source, confirmed by mutation. ✅
- **Zero passes.** `- tokens_total: 0` on a shipped stage is accepted as filled. The
  spec-side gate this one mirrors explicitly rejects it (`case ''|null|0`) and warns
  below `COST_IMPLAUSIBLE_FLOOR=1000`. The stage half has neither. (`FU-6`)

### Findings

| id | label | finding | proposed §15 disposition |
|---|---|---|---|
| `SB-1` | ship-blocker | The patch says **"No `DEC-*`. … it decides nothing new."** `DEC-013` §5 records the opposite decision — *"Warn-only, no gate, no view yet: **capture first**"* — and that sentence is still live in `projects/_templates/stage.md:37` and in **all five** `STAGE-00N-*.md` files. `STAGE-003/004/005` will now block at close on a field whose own front-matter comment tells the author it is *"never a gate"*. Turning a recorded "no gate" into a gate is a decision, and the artifact the gate reads still contradicts it. | `fixed` — amend `DEC-013` §5 (or emit a superseding `DEC-*`), and update the one line in the template + five stage files. |
| `SB-2` | ship-blocker | `stage_has_orchestration_cost` toggles its front-matter fence on **every** bare `---` and never clears `in_oc` when the front matter closes, so the markdown **body** is scanned as front matter. A shipped stage with an empty `orchestration_cost` passes green if its body contains a `---` rule followed by a line matching `- tokens_total: <digits>`. Reproduced end-to-end against `STAGE-002`: `cost-audit` `rc=0`, JSON `offenders: 0`. Both conditions are necessary and I isolated them: `in_oc` survives the closing `---` **only when `orchestration_cost:` is the last front-matter key** — and it is the last key in `projects/_templates/stage.md` and in all five `STAGE-00N` files, so the vulnerable arrangement is this repo's default shape, not an unusual one. Latent today (no stage body has a bare `---`), but it is `attribute-text-inside-doc-comments` — prose *about* the field satisfying a gate — which is the exact class this patch exists to prevent, and the Problem section's claim that the gate "cannot make that mistake" is therefore too broad (§16 rule 1). | `fixed` — `/^---$/ { if (++fence >= 2) exit; next }`, or clear `in_oc` when the front matter closes. Add the shape to the red-proof. |
| `FU-1` | follow-up | The `#` comment guard in `stage_has_orchestration_cost` is **unreachable** — no input can reach it — yet the function's comment names it as the defense against the measured trap. Mutant `M4a` survives the red-proof, and a differential over 15 files shows deleting it changes nothing. The comment points at the wrong line. | `fixed` — delete the guard and re-point the comment at the `^- …[0-9]+` anchor, or keep it and say it is belt-and-braces. |
| `FU-2` | follow-up | The red-proof asserts the *reason* but never the *stage name*, while both its success line ("REJECTED **by name**") and its failure message ("never named the stage **or the field**") claim it does. `M4b` survives. | `fixed` — one line: `grep -q "STAGE-002.*missing cost on: orchestration"`. |
| `FU-3` | follow-up | `status: cancelled` is not audited; a stage moved to `cancelled` records no orchestration cost and nothing objects. | `closed` — with one line of why in the patch, **or** widen to cover `cancelled` too. Reviewer's read: defensible to close, but say so. |
| `FU-4` | follow-up | `get_stage_status` returns `$2` verbatim, so `status: "shipped"` yields `"shipped"` and the stage is silently skipped by the new gate. Pre-existing helper, but PATCH-002 is the first gate to make it load-bearing for enforcement. | `spec:` or `fixed` — strip surrounding quotes in `get_stage_status`. |
| `FU-5` | follow-up | `just lint` and `scripts/lint-red-proof.sh` invoke a bare `cargo clippy`; with a nightly default toolchain they fail with *"'cargo-clippy' is not installed"* rather than running. Fifth instance of the `+toolchain` family. `AGENTS.md` §6 also still says the PATH clippy is "Homebrew's 0.1.97". | `signal: <the +toolchain trap family>` — add this instance; it is a class, not a one-file fix. |
| `FU-6` | follow-up | The stage gate accepts `- tokens_total: 0` as filled; the spec gate it mirrors rejects `0` and warns below `COST_IMPLAUSIBLE_FLOOR`. | `fixed` — require `[1-9][0-9]*`, or route stages through the same implausibility warning. |
| `FU-7` | follow-up | `cost-audit-red-proof.sh` does `cp -R "$ROOT"`, which copies `target/` — **105 s** locally against 3.8 s in a clean clone (0.87 s in CI, where `target/` does not exist). | `fixed` — exclude `target/` and `.git`, or copy only what the gate reads. |
| `FU-8` | follow-up | The new `die` message points the reader at `docs/cost-tracking.md`, which is 90 lines and says **nothing** about stage `orchestration_cost`. | `fixed` — a short section there, or drop it from the message. |
| `FU-9` | follow-up | `PATCH-002`'s own bookkeeping was not advanced by the patch pass: `task.cycle` is still `patch` (not `verify`), `## Patch Completion` is the unfilled template stub (no branch, no fix summary, no reflection, no `defect-catch-stage`), and `cost.sessions` is `[]`. §15 check 7 says flag-don't-block for cost; the empty Completion section is the same *documented-step-with-no-surface* shape this patch is about. | `fixed` at the patch pass's punch-list round, before ship. |
| `FU-10` | follow-up | `STAGE_ORCH_COST_GRANDFATHERED=""` does **not** clear the list — `${VAR:-default}` falls back, so `STAGE-001` stays exempt; only a non-empty string such as `" "` clears it. Replicates the existing `COST_AUDIT_GRANDFATHERED` shape, so it is a convention, but an operator "turning it off" gets a silent no-op. | `closed` — document the `" "` form in the comment, or switch to `${VAR-default}`. |

Nothing here says the gate is wrong. `SB-1` is about the record, `SB-2` about one awk
fence; both are small and surgical, and the detector's core — anchoring on a real YAML
list item rather than on `tokens_total` anywhere in the file — is right and is now
proven right by `M2`.

### Cost self-report

- **Tokens (total):** **10,300,000** — measured floor **8,522,559** over **71** unique
  assistant turns, deduped by `message.id`, **97.6 % cache-read**, rounded up 20 %.
- **Estimated USD:** **$25.02** — per-component at published Opus rates
  ($15 in / $75 out / $30 cache-write / $1.50 cache-read) = **$20.85**, same 20 % uplift.
- **Duration (minutes):** ~40
- **Source of the number:** this session's own transcript,
  `abef176a-32e5-4efe-9de4-7cd820a77e73.jsonl`, **identified by my scratchpad-dir uuid**
  — not by grepping for text, and not the orchestrator's live `e078417d-…` session in
  the same directory, which is a different session on a different model.

### Drift and new artifacts

- **New decisions emitted:** none by me. **One is owed** — see `SB-1`.
- **Deviations from spec:** the patch's *"No `DEC-*`"* claim (`SB-1`).
- **Follow-up work identified:** `FU-1` … `FU-10` above. `FU-5` belongs in
  `guidance/signals.yaml` as another `+toolchain` instance, not in a spec.

### Reflection

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing was unclear; the handoff was unusually good. The one thing that cost time
   was legitimate: the gate count. Running the union and saying so was cheap, but only
   because the handoff pre-authorised not resolving it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Yes, and it is `SB-1`. The handoff and patch both cite `DEC-013` §5 as the
   *authority* for the gate. §5 is the authority for **capturing** the field and
   explicitly withholds a gate. Listing it as a reference invited the reader to check
   the reference, which is how this was found — but the patch should have listed it as
   a decision it was **changing**.

3. **If you did this task again, what would you do differently?**
   — I reproduced the three claimed mutations before probing the gate's own input space,
   and the two most valuable findings (`SB-2`, `FU-1`) came from the probe, not the
   mutations. Next time I would build the input corpus first — a dozen adversarial
   stage shapes through the detector takes two minutes and it is what actually finds
   the false green. Mutating the implementation tests the *proof*; feeding the detector
   tests the *gate*, and only the second one found a defect that ships.
