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
  id: HANDOFF-200
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-096

project:
  id: PROJ-002
  stage: STAGE-033
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model)
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 8799858            # REAL combined count — what cost-audit reads
  estimated_usd: 6.79              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 60
  branch: chore/spec-096-wasm32-build-gate
  pr: 346
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript bc4383e8-214e-44a7-a105-8d69a9586e17.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 103 + cache-creation 161,883 (1-hour) + cache-read 8,602,932 + output 34,940, priced at 5 / 10 / 0.5 / 25 USD per million. INCLUDES the design cycle: one side session designed and built this spec, so design is not separately metered. Re-sum at ship."
  synced_at: 2026-11-25
---

# HANDOFF-200: Build SPEC-096, a wasm32 build gate and the live demo link

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-096` to `claude-opus-5` (implementer) for the
**build** cycle. In this run the same session designs and builds (a side session started
for this one small spec); the verify is a separate, cold agent (HANDOFF-201).

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-096-a-wasm32-build-gate-and-the-live-demo-link.md`.
  Its design-time probe, Outputs, the two mutations and the pre-registered mutants M1-M4
  are binding.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-033-the-browser-target-stays-buildable.md`
- **Shape to copy:** `scripts/doc-check.sh`, `scripts/doc-check-red-proof.sh` (SPEC-056).
- **Toolchain brief:** `./guidance/toolchain-brief.md`. Locally, Homebrew's cargo/rustc
  come first on PATH and have no wasm32 std; prefix `PATH="$HOME/.cargo/bin:$PATH"`.

### Decisions that apply

- `DEC-002` (proposed) — keeps the browser target open; this spec gates it, does not amend it.
- `DEC-009` — the red-proof mechanism: negative control first, then an attributed red.

### Constraints that apply

- `oracle-must-be-shown-red` — a gate ships with its red-proof and a control.
- `library-not-application` — no app, CLI or wasm-bindgen surface in the crate.
- `one-spec-per-pr`, `cost-captured-per-cycle`.

## Expected Deliverables

- Every file in SPEC-096's `## Outputs`; `just wasm-check` and `just wasm-check-red-proof`
  green.
- **Gates:** `just gate-audit`, `just validate`, `just cost-audit`, `just fmt-check`,
  `just lint-ci` (no src change, run anyway), `just decisions-index` unchanged.
- **AC3 by hand:** run the gate with Homebrew's toolchain first on PATH and paste the named
  failure into the handback.
- **The PR:** against `main` from `chore/spec-096-wasm32-build-gate`, naming HANDOFF-200,
  SPEC-096, STAGE-033, PROJ-002, DEC-002, DEC-009. Commit message bodies plain prose (the
  push ruleset rejects code tokens). Confirm every `gh pr checks` job passes, the two new
  ones included.

## Out of Scope

- A wasm runtime test lane, other wasm targets, any `src/` change, DEC-002's status.

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

- **Branch / PR:** `chore/spec-096-wasm32-build-gate`, https://github.com/jysf/irradiance/pull/346
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** Yes locally (AC4, CI green, is checked on the PR):
  - AC1: `just wasm-check` → `✓ wasm-check: 2 feature surfaces (default, write) built for wasm32-unknown-unknown.`
  - AC2: `just wasm-check-red-proof` → control green, A native-green + E0433 naming `unix`,
    B native-green with `write` + default surface built + E0080 on `write`; 12.6 s wall.
  - AC3: with Homebrew first on PATH (`/opt/homebrew/bin/cargo`, rustc 1.99.1 Homebrew),
    `./scripts/wasm-check.sh` exits 1 with: `✗ wasm-check: the toolchain at
    /opt/homebrew/Cellar/rust/1.99.1 has no wasm32-unknown-unknown std (no
    .../lib/rustlib/wasm32-unknown-unknown/lib). Install it with rustup target add
    wasm32-unknown-unknown, and put rustup's toolchain first on PATH`.
  - AC5: README status block + "Building and testing"; AGENTS.md §6 block and history line.
  - AC6: `just gate-audit` 15 scripts scanned, clean. Note: it scans `wasm-check.sh`;
    `*-red-proof.sh` files are excluded from its scan by design (gate-audit.sh line 79),
    so the AC's "two new scripts" is one in practice.
  - AC7: no `src/` or `Cargo.toml` change.
  - Also green: `just validate`, `just cost-audit`, `just fmt-check`, `just lint-ci`,
    `just name-length-check`, `just decisions-index --check`.

### Cost self-report

- **Tokens (total):** 8,799,858 (floor; includes design)
- **Estimated USD:** 6.79
- **Duration (minutes):** about 60
- **Source of the number:** the session transcript, deduplicated by message.id

### Drift and new artifacts

- **New decisions emitted:** none
- **Deviations from spec:**
  - AC6 as noted (red-proofs are outside gate-audit's scan).
  - The rlib-exists check after each surface reads the same path for both surfaces, so on
    the `write` surface it can be satisfied by the default build's file; cargo's exit code
    is the verdict there. Left as is.
- **Follow-up work identified:**
  - A run-time wasm lane (tests under `wasm32-wasip1` + wasmtime, or a clippy
    `disallowed_methods` list for `Instant::now` / `SystemTime::now` / `std::fs` /
    `std::thread::spawn` in library code). The build gate cannot see those.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec; the requesting brief's example faults (`Instant::now`, `std::fs`)
   compile on wasm32, which the design probe caught before any script was written.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — gate-audit's exclusion of `*-red-proof.sh`, which made AC6 overclaim.

3. **If you did this task again, what would you do differently?**
   — Read gate-audit's exclusion list at design, before writing the AC.
