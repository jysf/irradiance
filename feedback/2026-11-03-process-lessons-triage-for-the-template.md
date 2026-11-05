---
source: "irradiance (PROJ-001/PROJ-002) — jysf"
captured_at: 2026-11-03
captured_by: claude
status: open
---

# Irradiance's process lessons, triaged: what belongs in the template, a Rust variant, or nowhere

## The issue

Companion to `2026-11-03-upstream-irradiance-gates-and-script-fixes.md` (which covers
*scripts*). This entry covers *process and discipline*: the lessons, golden paths and
process-debt that irradiance's `guidance/signals.yaml` and `AGENTS.md` §15–16 accumulated
over 8 stages and ~35 specs, and which the template (0.6.39) either lacks or has no
mechanism to carry. None of this has been sent upstream.

**Method and limit.** Sorted from each signal's `type`, `status` and `summary`, plus a full
read of AGENTS.md §16's four rules. I did **not** re-read every signal's full evidence
block. Treat the "template?" column as a proposal; the maintainer should read the evidence
before codifying. The template's own bar applies (lessons: N=3 same-outcome or N=2
paired-opposing; golden paths: N=3). Items below the bar are marked and should be **recorded
as candidates, not codified**.

## A. Belongs in the template — language-agnostic, at or past the bar

| Item | Evidence in irradiance | Proposed template change |
|---|---|---|
| **Four rules earned the hard way** (AGENTS.md §16, codified 2026-08-22 / 2026-09-06) | Rule 1 `measurement-over-generalised` N=5; rule 2 `attribute-text-inside-doc-comments` N=5; rule 3 `a-gate-that-fails-mutely…` N=4; rule 4 `unrun-docs-carry-errors` N=6. | Add as §16 rules. Rules 1, 3, 4 are fully generic. Rule 2 ("assert the match count") is generic; its doc-comment examples are Rust-flavoured — keep the rule, make the examples neutral or variant-specific. Rule 3 is backed by the `gate-audit` script in the sibling entry — ship them together. |
| **A gate/oracle must be shown red** ("oracle-must-be-shown-red") | Drives six red-proof scripts (SPEC-003/FU-14, SPEC-016, SPEC-023, PATCH-002, DEC-006/007, DEC-011/027). Several caught real false-greens, e.g. a panic shipped past seven gates. | State as a standing design rule in §15 (design/verify cycles): any new gate or oracle ships with a proof that it goes red when the thing it guards is broken. The verify cycle should *independently* break the real code (irradiance's verify sessions did this each time). |
| **Follow-ups table has no enforcement** — `follow-up-disposition-has-no-surface` (accepted) | §15 requires a `## Follow-ups` table at ship; nothing checks it exists or is complete. | `validate`/`archive-spec` checks the section exists and every row has a disposition. |
| **Spec reflection can ship unfilled** — `spec-reflection-shipped-unfilled` (open) | Front-matter (`complexity_actual`, `verify_verdict`) is stamped; the prose `## Reflection` stays as template placeholders. | `archive-spec` refuses (or warns) if Reflection still contains placeholder text. |
| **No named gate list** — `the-gate-count-is-not-defined-anywhere` (open) | Specs said "ten / eleven / twelve gates" meaning different sets; some acceptance criteria unsatisfiable as written. | One authoritative, machine-readable gate list (e.g. `just gates`) that specs cite by name. |
| **Cost accounting has several unowned seams** — `cost-field-has-two-owners`, `tier-map-predicts-what-it-should-record`, `token-counts-not-comparable`, `flat-rate-overstates-cached-sessions` (2.7× on a 97.5%-cache-read session), `cost-audit-active-scoped-hides-parked-stages` (all open/accepted) | AGENTS.md §15 says the agent writes `cost.sessions`; DEC-013 says `handback-sync` does. A predicted model is written into a field read as measured. Cost figures from different execution modes are averaged. Parking a project drops its shipped work from the cost gate. | Template decision needed on who owns `cost.sessions`, a measured-vs-predicted distinction in the field, and not scoping `cost-audit` to the active project alone. This one wants a DEC in the template, not a patch. |
| **Desktop sessions can't read `/cost`** — `desktop-cost-tool-unavailable-handbacks-null` (open, **N=8**) | Every delegated build/verify handback from the desktop Code tab came back `tokens_total: null`; the orchestrator recovered each from the session's own transcript at ship. | Document the recovery procedure (identify the transcript by its scratchpad UUID, not text grep — see the memory note `identify-own-transcript-for-cost-handback`) and let `handback-sync` accept an explicit `recovered_from_transcript` marker rather than a silent null. Related: `orchestrator-transcript-looks-like-a-prior-attempt`. |
| **`handback-sync` defects** — hash-as-comment in unquoted notes, multi-line scalars truncated to line 1 (breaks front-matter parse while every gate passes), stale `to_agent` across punch-list rounds, superseded handoffs (the last is `done`) | All four are in the template's script. | Fold into the script fixes in the sibling entry; the multi-line-scalar one is the dangerous one because the failure is silent. |
| **Brag/accomplishment step skipped at ship** — `brag-step-skipped-at-ship` (open) | Skipped on every shipped artifact until a human asked. | If the template keeps an accomplishment-logging step in the ship cycle, make it a checked step, not prose. If it is irradiance-specific tooling, drop it from this list. |
| **ID minting** — `spec-ids-collide-across-unmerged-branches`, `next-id-scans-test-fixtures` | Already filed 2026-09-06; `_lib.sh` fix in sibling entry. | No new action — cross-reference only. |

## B. Candidates — below the bar, record but do not codify

| Item | Status / N | Why it is interesting |
|---|---|---|
| `probe-the-hard-assumption-at-frame-time` | lesson, watch, N=1 | Probe the stage's hardest assumption on the real corpus at **frame** time; otherwise a precursor surfaces mid-design and forces a scope split. Suggests a line in the frame prompt. |
| `patent-check-named-algorithms-at-frame` | lesson, watch, N=1 | Clean-room defeats copyright, **not patents**. MHC demosaic (US 7,502,505, active to ~2027-01) was named in the brief; the stage was re-scoped to Hamilton-Adams (DEC-031). Suggests a frame-time checklist item for any named algorithm or third-party dependency (licence **and** patent). One occurrence, but the cost of a miss is a whole stage. |
| `a-fix-inherits-the-precondition-of-the-thing-it-fixes` | lesson, watch (deferred, N=2) | A fix hardens the logic that failed and keeps the assumption that let it fail; attack the *proof*, not the code. |
| `distinct-value-dedup-drops-multiplicity` | lesson, watch | Per-distinct-value checks lose "how many". |
| `pre-register-the-tolerance` | golden-path, watch | State a numeric acceptance threshold **and its falsifier** before measuring. |
| `findings-carry-ids-across-boundaries` | golden-path, watch | Label review findings ship-blocking or follow-up and give follow-ups per-spec ids so they survive a handoff. |

## C. Rust / oracle-domain — a variant or an example, not the base template

| Item | Note |
|---|---|
| `floating-toolchain-plus-deny-warnings` (risk, open) | CI runs `-D warnings` on `@stable`, so every lint a new clippy adds fails CI on untouched code. Belongs in a Rust variant: pin the toolchain or treat clippy upgrades as a scheduled task. |
| `allow-attribute-exits-the-panic-policy` (done) | One `#[allow(clippy::panic, …)]` passed seven gates. Fixed by `lint-no-allow`. Rust variant. |
| `named-tests-can-pass-vacuously` (accepted) | `cargo test <name>` matching zero tests exits 0. The generic form ("assert the test count") is worth a line in the base template; the cargo specifics go in the Rust variant. |
| `saturating-as-cast-masks-explicit-clamp-in-quantize-tests` (watch, N=1) | Rust numerics. Stays in irradiance. |
| `reusable-bit-exact-oracle-harness` (golden-path, watch) | Build type + analytic bit-exact oracle + red-proof **once** as a foundation spec; later kernel swaps become cheap (SPEC-030 → 031 → 032: the third build cost 14.3M tokens vs 25.4M for the first). Strong, but domain-specific and not yet N=3. Candidate for a documented *example*, not a rule. |
| `ci-cannot-prove-bit-exactness` (risk, watch) | CI can't run corpus-dependent tier-B tests, so green ≠ correct. Generalises to "say which tier a green badge covers"; only the instance is irradiance's. |
| `dnglab-source-reading-tempts-behavior-check` (open) | Clean-room constraint specific to this repo. Stays here. |

## Context

Raised after the gate-scripts entry, when the maintainer asked whether irradiance's *process*
innovations had been sent back as well as its scripts. They had not. The practical risk is
the same as for the scripts: `diamond-exif` (already created at 0.6.39) and `crustyimg` will
re-learn these. Rules 1–4 and "oracle must be shown red" are the highest value because each
was paid for by a real false-green.

## Priority (their assessment)

High for section A rows 1–2 (rules and red-proof principle: cheap to write, expensive to
rediscover) and the cost-accounting seams (they silently corrupt a shipped number). Medium for
the rest of A. Section B is record-only. Section C is a decision about whether the template
grows a Rust variant at all — the maintainer's call.

## Suggested prompt for the template maintainer agent

> Read `feedback/2026-11-03-process-lessons-triage-for-the-template.md` in `jysf/irradiance`
> alongside `feedback/2026-11-03-upstream-irradiance-gates-and-script-fixes.md`, and
> irradiance's `AGENTS.md` §15–16 and `guidance/signals.yaml`.
>
> 1. **Verify, don't trust the triage.** It was sorted from signal summaries. For each row in
>    section A, read the signal's full evidence in `guidance/signals.yaml` and confirm it is
>    generic and at the bar before changing the template.
> 2. **Land the four §16 rules and the oracle-must-be-shown-red principle** in the template's
>    AGENTS.md, with Rust-flavoured examples neutralised. Ship with the `gate-audit` scripts
>    from the sibling entry so the rules have teeth.
> 3. **Add the ship-time checks**: Follow-ups table present and dispositioned; Reflection not
>    left as placeholders. Extend `just template-selftest` so both go red when violated.
> 4. **Write a DEC for cost-field ownership** (agent vs `handback-sync`, measured vs
>    predicted, cost-audit scope). Do not patch around it.
> 5. **Record section B as signals** in the template's own ledger at the bar they are at.
>    Do not codify N=1 items.
> 6. **Decide section C**: does the template grow a Rust variant? If yes, scope what moves in;
>    if no, say so in the resolution so irradiance keeps them locally.
> 7. Report which rows you rejected as irradiance-specific and why. Do not push before the
>    maintainer confirms.

## Resolution

Open. Fill in with the template release(s) that land section A, then bring diamond-exif and
crustyimg up to it.
