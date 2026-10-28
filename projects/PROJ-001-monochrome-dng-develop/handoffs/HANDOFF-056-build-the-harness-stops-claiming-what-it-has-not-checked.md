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
  id: HANDOFF-056
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual implementer model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-016

project:
  id: PROJ-001
  stage: STAGE-005
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
  tokens_total: 10500000           # REAL combined count — what cost-audit reads
  estimated_usd: 28.50             # tokens_total × your rate, or your harness's number
  duration_minutes: 75
  branch: claude/spec-016-build-9736e8
  pr: https://github.com/jysf/irradiance/pull/23
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "Parser: ruby Psych via YAML.parse (syntax-only, so created_at date scalars are not false positives and the Ruby 2.6 safe_load permitted_classes trap is avoided; portable 2.6-3.x), confirmed present on the ubuntu-latest CI runner by the new validate step passing. Scope broadened to every *.md under projects/ + decisions/ (121 parsed; was specs+patches+spikes = 26). HANDOFF-039 was the sole live falsifier (embedded unescaped double-quotes in handback.notes giving did-not-find-expected-key); fixed by escaping its interior quotes, 0 of 121 fail now. The named test validate_rejects_front_matter_that_no_parser_can_read ships as scripts/validate-red-proof.sh (standalone shell + just recipe + CI step), fixture at tests/oracle-fixtures/SPEC-900-unparseable-front-matter.md. tokens_total is the transcript floor (deduped by message.id, all components) rounded up for the uncaptured handback/CI turns; measured floor 9,430,941 / 25.67 USD."
  synced_at: 2026-10-27
---

# HANDOFF-056: the harness stops claiming what it has not checked

## Delegation Summary

The orchestrator (architect) hands `SPEC-016` to the implementer for the
**build** cycle: rewrite `just validate` to actually PARSE every artifact's
front matter with a real YAML parser and fail loudly (naming the file and the
parser's own message), with a red-proof. The design is complete and freshly
re-measured — read the spec's **Design refresh (2026-10-27)**. This is a
shell-script build; no `src/` change.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-016-the-harness-stops-claiming-what-it-has-not-checked.md`
  — read the whole thing, especially **Design refresh (2026-10-27)** (current
  numbers), **AC1–AC3**, **Failing Tests**, and **Traps**.
- **The gate under rewrite:** `scripts/validate.sh` + `scripts/_lib.sh`. Today
  it reads front matter with line-oriented `awk`/`grep`/`sed` and NEVER parses;
  it also scopes to **specs + patches + spikes only** (26 artifacts) — not
  `handoffs/` or `decisions/`.
- **The live falsifier in the repo:** `projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-039-verify-orchestration-cost-is-documented-at-stage-close-and-nothing-checks-it.md`
  — its front matter does not parse (`did not find expected key …` ~line 49).
  It is outside today's `validate` scope, which is exactly why it slipped
  through. **Fix its front matter as part of this spec** so the new gate ships
  green (that fix IS the gate doing its job on its first run).
- **Toolchain brief:** `./guidance/toolchain-brief.md` — pinned vs local clippy
  (`lint-ci` not `lint`), the `+toolchain` PATH trap, zsh `$pipestatus`, the
  `just` gate recipes. Read it (DEC-004 rule 5).
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-005-harden-the-harness-the-gates-the-oracle-s-parser-and-the-second-cargo-graph.md`

### Decisions that apply

- `DEC-013` — the field-shape rules `validate` is entitled to check. The parse
  gate does NOT recheck DEC-013's rules; it only ensures the parser can REACH
  them. Keep the existing "required keys present" check (it catches valid YAML
  with missing keys, which a parse alone would not).
- `DEC-017` — the red-proof mechanism (mutate a copy, show old-vs-new). AC2's
  red-proof follows this shape for a shell gate.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `scripts/` + `tests/` +
one broken handoff:

- `oracle-must-be-shown-red` — the new gate MUST be proven red: show the OLD
  script PASSING on a parse-broken fixture and the NEW script FAILING on the
  same bytes, **with the parser's message** (AC2's third clause — an exit code
  alone does not prove the check ran; a wrong parser API fails uniformly, see
  the spec's Design refresh).
- `test-before-implementation` — the failing test
  (`validate_rejects_front_matter_that_no_parser_can_read`) is the deliverable's
  spine; write it first.
- `cost-captured-per-cycle` — fill the handback with a REAL `tokens_total`
  before reporting done (DEC-013).

### Prior related work

- `HANDOFF-038` (build) — the pre-rescope L-sized attempt at this spec; its
  context is at git `3238dcb`. This spec is the S-sized AC5-only slice.
- `guidance/signals.yaml` — `handback-sync-truncates-multi-line-scalars` (the
  defect class that produces the broken front matter), `named-tests-can-pass-vacuously`
  and `the-gate-count-is-not-defined-anywhere` (AC3 works around the last one).

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`scripts/validate.sh` rewritten** to parse every artifact's front matter
  under `projects/**`, `decisions/`, `handoffs/`, `patches/` with a real YAML
  parser (`ruby -ryaml` recommended — confirm it on the CI runner), failing
  non-zero on a parse error and printing the file + the parser's own message.
  Keep the existing required-keys check. Name the parser you chose in the
  handback and confirm it exists on CI.
- **The failing test** `validate_rejects_front_matter_that_no_parser_can_read`
  (AC1) — a hand-crafted fixture (unterminated quote in a multi-line scalar, the
  `handback-sync` shape). State its form (bats / standalone shell / appended
  case) in the handback.
- **AC2 red-proof**, recorded in the handback: OLD script PASSES the fixture,
  NEW script FAILS it, both on the same bytes, with the parser's message shown.
- **HANDOFF-039's front matter fixed** so the broadened gate ships green; report
  the cause (the `did not find expected key` break) and the repo-wide count
  after the fix (expected 0 of 120).
- **CI green** on the PR head (AC3) — `just lint-ci` (pinned clippy); name the
  gate list you ran in the handback. Observe CI, don't infer.
- **A PR against `main`.** Description references HANDOFF-056, SPEC-016,
  STAGE-005, PROJ-001, DEC-013, DEC-017.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- **The four deferred pre-rescope ACs** (corpus-status overclaim, `req()`
  truncation, `SUPPORTED_BITS` coverage, plane fuzz `white_level`) — separate
  STAGE-005 backlog items, measured at `3238dcb`.
- **The full gate-script audit** (the `pipefail` + unguarded-`grep` sweep) — a
  separate STAGE-005 item; folding it in makes this XL.
- **Fixing `handback-sync`'s writer.** This spec makes the defect DETECTABLE at
  validate-time; it does not fix the producer. (Fixing the one existing broken
  artifact, HANDOFF-039, so the gate ships green is in scope; rewriting the
  tool that produced it is not.)
- **Defining "the gates."** `the-gate-count-is-not-defined-anywhere` is open;
  AC3 works around it — just name the list you ran.
- **Anything in `src/`** — no decoder behaviour changes.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-016-build-9736e8` → https://github.com/jysf/irradiance/pull/23
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** yes.
  - **AC1** — `just validate` now parses every artifact's front matter under
    `projects/**` and `decisions/` with a real YAML parser (ruby Psych via
    `YAML.parse`) and fails non-zero, naming the file and printing the parser's
    own message. 121 artifacts parsed (broadened from the 26-artifact
    specs+patches+spikes scope). Parser confirmed present on CI (the new
    `validate` step passed on `ubuntu-latest`). Required-keys check retained.
    The named test `validate_rejects_front_matter_that_no_parser_can_read` ships
    as `scripts/validate-red-proof.sh` — a **standalone shell script** with a
    `just validate-red-proof` recipe and a CI step (not a bats test, not a case
    in the template-only `scripts/test.sh`); the hand-crafted fixture (unescaped
    quote in a multi-line scalar — the `handback-sync` shape) is committed at
    `tests/oracle-fixtures/SPEC-900-unparseable-front-matter.md`.
  - **AC2** — red-proof shown, same fixture bytes (md5 `0c30e518…`): **OLD**
    `validate.sh` exits **0** (`27 artifact(s) … valid required front-matter`);
    **NEW** exits **1** with `SPEC-900-unparseable-front-matter  front matter
    did not parse: (<unknown>): did not find expected key while parsing a block
    mapping at line 15 column 1`. The parser's **message** is shown, not just the
    exit code — which is what distinguishes a real parse failure from a wrong
    parser API that fails uniformly (DEC-017; the `safe_load(permitted_classes:)`
    trap the Design refresh warns about). The durable red-proof also isolates the
    coverage: the required-keys half still passes the fixture; only the parse gate
    objects.
  - **AC3** — `just lint-ci` ran with clippy **0.1.98 (88d9e12ae1 2026-08-18)**
    via the pinned `PATH="$HOME/.cargo/bin:$PATH" … +stable` path (matches CI's
    floating stable; Homebrew's local 0.1.97 would not). CI **observed** on the PR
    head, not inferred. The gate count is genuinely ambiguous here
    (`the-gate-count-is-not-defined-anywhere`, open) — the list I ran is named
    below; I did not try to resolve the ambiguity.
- **Gate list run (named per AC3):** locally green — `validate`,
  `validate-red-proof`, `cost-audit`, `cost-audit-red-proof`,
  `decisions-index --check`, `cargo fmt --check`, `just lint-ci`. No `src/` or
  `Cargo.toml` change, so the Rust jobs (`test`, `clippy`, `deny`, `deny-fuzz`,
  `msrv`, `fuzz-warp`, `fuzz-opcode`, `lint-red-proof`, `lint-no-allow`) are
  unaffected and were left to CI as the observed source of truth.

### Cost self-report

- **Tokens (total):** 10,500,000 — the transcript floor rounded up for the
  handback/push/CI turns not yet in the file.
- **Estimated USD:** $28.50 (floor $25.67).
- **Duration (minutes):** ~75.
- **Source of the number:** transcript sum, deduped by `message.id`, all
  components (input + output + cache-read + cache-write). No `/cost` in this
  interface (Claude desktop app, Code tab). Transcript located via the
  scratchpad-dir UUID `6e73a035-e343-493a-b80e-91dcc5580266` (not a text grep —
  all 46 unique assistant turns carry `model == claude-opus-4-8`, so it is this
  session, not a stale pre-`/clear` file). Measured floor **9,430,941** tokens;
  96.8% cache-read; priced per-component at Opus rates ($15/$75 in/out,
  $30-write, $1.50-read) = $25.67. The headroom (~11%) covers the turns that
  wrote this handback and observed CI, which post-date the measurement.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-001 §1 already charters `validate` as the
  schema gate; parsing the front matter it validates is an extension of that
  gate, not a new decision. The parser choice (`YAML.parse`) is a mechanism, not
  a policy.
- **Deviations from spec:**
  - The spec's Outputs said *"no change to `ci.yml` if `just validate` already
    runs in CI; a new job if not."* It did **not** run in CI (that gap is exactly
    how the unparseable `HANDOFF-039` slipped the old specs-only scope), so I
    added steps for `validate` **and** its red-proof to the `cost-data` job
    (language-agnostic, `ubuntu-latest`, ruby present) rather than a whole new
    job — same runner, same job family as the other shell gates/red-proofs.
  - Repo-wide pre-fix count is **1 of 121**, not the spec's 1 of 120 — the +1 is
    `HANDOFF-056` itself, minted in this worktree today. Post-fix: **0 of 121**.
- **Follow-up work identified:** none new. The four deferred pre-rescope ACs and
  the full gate-script `pipefail`/unguarded-`grep` audit stay on STAGE-005's
  backlog (explicitly out of scope). Fixing `handback-sync`'s **writer** (the
  producer of this defect class) also remains open — this spec made the defect
  detectable, it did not fix the producer, per Non-Goals.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Little. The AC1 body still carried the stale `2 of 75` numbers while the
   Design refresh carried `1 of 120`; I followed the refresh (the handoff points
   there first). The `python3`-has-no-`pyyaml`/now-has-`pyyaml` back-and-forth
   turned out moot: `YAML.parse` is syntax-only and needs neither `pyyaml` nor a
   `load`/`safe_load` policy, so the parser-availability history didn't bear on
   the choice.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red`, `test-before-implementation`,
   `cost-captured-per-cycle`, DEC-013 and DEC-017 all applied and were listed.

3. **If you did this task again, what would you do differently?**
   — Reach for `YAML.parse` (syntax-only) from the start instead of `safe_load`.
   My first probe used `safe_load(permitted_classes: [Date, Time])` and failed
   uniformly (`uninitialized constant Date`) — the exact wrong-parser-API trap
   the spec warns about. It cost one loop, but it also confirmed the spec's point
   first-hand, which is partly why AC2 insists on showing the parser's message.
