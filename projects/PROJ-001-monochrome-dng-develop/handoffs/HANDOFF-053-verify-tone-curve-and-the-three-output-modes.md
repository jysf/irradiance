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
  id: HANDOFF-053
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-26
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-019

project:
  id: PROJ-001
  stage: STAGE-003
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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-053: Tone curve and the three output modes — verify

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect/orchestrator) hands `SPEC-019`
to `claude-opus-5` (acting as **verifier**) for the **verify** cycle — an
independent cold review of **PR #18**, which implements SPEC-019.

⚠ **You did not implement this. Review it cold.** The build session already
reports "all 10 ACs met and CI green" — your job is to *disprove* that, not
confirm it. The one check the orchestrator could not do from the outside is
**actually running the oracle red-proof** — do that yourself.

## Context the Receiving Agent Needs

Your worktree is cut from the PR branch `claude/spec-019-tone-curve-13488a`
(head `46b5f78`), so the code under review and the spec are already present.
Read these before starting:

### Primary

- **The PR under review:** `#18` — https://github.com/jysf/irradiance/pull/18
  (base `main`, head `46b5f78`). 18 files; the substantive code is `src/tone.rs`,
  the generated `src/tone_table.rs`, `examples/gen-tone-table.rs`,
  `tests/tone_oracle.rs`, `src/bin/irr.rs`, `src/lib.rs`.
- **Spec (the contract):** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-019-tone-curve-and-the-three-output-modes.md`
  — the 10 ACs and Failing Tests are what you check against.
- **Build handoff:** `./projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-052-build-tone-curve-and-the-three-output-modes.md`
  — what was asked and its out-of-scope.
- **Stage:** `./projects/PROJ-001-monochrome-dng-develop/stages/STAGE-003-opcodes-and-output-bad-pixels-warp-tone-curve-develop-oracle.md`
  — shipping SPEC-019 closes this stage's backlog.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps;
  in particular `just lint-ci` is the only local command that sees CI's clippy.

### Decisions that apply

- `DEC-018` — develop output stays scene-linear `u16`. Confirm the build did
  **not** change `develop_into` (a decoded-pixel change to a shipped function
  would be a finding).
- `DEC-002` — determinism: table-driven, no runtime `powf` on the encode path.
- `DEC-025` (new, in this PR) — the tone/output model. Check it is present,
  honest, and matches what the code does.
- `DEC-017` / `DEC-021` — the red-proof shapes: the shipped table must carry no
  fault switch; the red-proof operates on a mutated copy with a negative control.

### Constraints that apply

- `oracle-must-be-shown-red` — **the one you must exercise, not take on trust.**
  Actually run `tests/tone_oracle.rs`'s red-proof, and independently perturb one
  entry of `src/tone_table.rs` by a large delta and confirm the analytic oracle
  turns **red** (then revert). A red-proof that passes on its own is not enough —
  prove it can fail.
- `no-panics-on-untrusted-input` — `src/tone.rs` is a library path: no
  `unwrap`/`expect`/out-of-range indexing/`panic!`/overflow; length mismatch
  returns a typed error. Check `just lint-no-allow` stays green.
- `provenance-recorded-per-algorithm` — `docs/provenance-ledger.md` has the sRGB
  OETF row (IEC 61966-2-1, class 1).
- `no-copyleft-dependencies` — no new dependency added; curve is arithmetic.
- `library-not-application` — no runtime I/O (a committed table / `include_bytes!`
  is fine; a runtime file read is not).

### Prior related work

- `HANDOFF-052` (build, this spec) — the contract the build executed.
- `PR #18` — https://github.com/jysf/irradiance/pull/18 (the subject of this review).
- `DEC-024` / `SPEC-021` — why end-to-end `--srgb` perceptual scoring is NOT this
  spec's check (the reference is uncorrected for warp); SPEC-019 validates tone
  **analytically** instead. Do not fault the absence of an end-to-end score.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review PR #18 cold and return
**exactly one verdict**: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED, at a named
commit SHA, with evidence for each claim.

Walk SPEC-019's **AC1–AC10** and confirm each maps to a test that **actually
ran** (a zero-match `cargo test <name>` exits 0 — check the run count, summed
across targets; this repo has been burned by it, `SPEC-015/AC9`, `SPEC-016`).
Specifically:

- **Run the gates yourself**, don't trust the PR's green: `just test --all-features`,
  `just lint-ci` (pinned toolchain — the only local command that sees CI's
  clippy), `just lint-no-allow`, `just deny` + `just deny-fuzz`, `just msrv`.
- **Run the oracle red-proof (the required adversarial step, AC6).** Confirm
  `tests/tone_oracle.rs`'s red-proof passes, THEN independently perturb one entry
  of the committed `src/tone_table.rs` by a large delta and confirm the analytic
  oracle (AC5) turns **red**; revert and confirm byte-identical. A green oracle
  you never forced red is manufactured confidence.
- **AC4 — no runtime `powf`:** confirm the library (`src/`, excluding
  `examples/`) calls no transcendental on the encode path; the curve is the
  committed table + integer lookup; `to_linear_f32` is a single division.
- **AC7 — determinism:** `just gen-tone-table` regenerates `src/tone_table.rs`
  with no diff; the pinned anchors (0→0, 65535→max, the 0.0031308 breakpoint)
  hold.
- **DEC-018 untouched:** `develop_into` still produces scene-linear `u16`; no
  shipped function's decoded output changed.
- **AC8/AC9:** provenance-ledger row present and correct; every output mode has a
  reader (`irr develop --format` + a test).
- Decision drift — `just decisions-audit --changed`; confirm `DEC-025` exists,
  is honest, and matches the code. Non-trivial choices missing a `DEC-*`.
- Behavioral, not just shape: `irr develop --format linear-f32|srgb16|srgb8`
  actually resolves and emits each mode (AGENTS.md §12 — the escape-prone class).

If **PUNCH LIST** or **REJECTED**, enumerate each finding with file:line and the
exact failing check, so a build round can act on it. Do **not** fix anything
yourself — verify returns findings; fixes are a separate build round.

## Out of Scope

Explicit list of what this verify does NOT include.

- **Fixing anything.** Verify returns findings; fixes go to a build round. Do
  not push to PR #18's branch or open a fix.
- **End-to-end SSIMULACRA2 scoring of a real render vs `dnglab --srgb`.** Not
  this spec's check (`DEC-024`/`SPEC-021`: the `--srgb` reference is uncorrected
  for warp). SPEC-019's correctness check is the analytic tone oracle (AC5). Do
  not fault its absence.
- **Re-litigating `DEC-018`** (scene-linear `u16` develop output),
  `ProfileToneCurve` support, colour, or auto-tone — all explicit SPEC-019
  non-goals.
- **Merging the PR.** Verdict only; the human/orchestrator ships.

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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
