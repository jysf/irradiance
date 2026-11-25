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
  id: HANDOFF-201
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 1212540            # REAL combined count — what cost-audit reads
  estimated_usd: 1.68              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 4
  branch: chore/spec-096-wasm32-build-gate
  pr: 346
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "Cold verify by a separate subagent; self-measured from its transcript agent-aa7bc8d73c353b98b.jsonl (14 unique message.ids): input 28 + cache-creation 94,265 + cache-read 1,110,937 + output 7,310, at 5 / 10 / 0.5 / 25 USD per million. APPROVED at cb00c38."
  synced_at: 2026-11-25
---

# HANDOFF-201: Verify SPEC-096, a wasm32 build gate and the live demo link

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-096` to a separate `claude-opus-5` agent (verifier)
for the **verify** cycle, cold, on branch `chore/spec-096-wasm32-build-gate` after HANDOFF-200.

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
- **Build handback:** `HANDOFF-200` (its `## Handback`).

## The verify contract

You are **not** the agent that implemented this. Review it cold.

- AC1-AC7 met? Run `just wasm-check` and `just wasm-check-red-proof` yourself (prefix
  `PATH="$HOME/.cargo/bin:$PATH"` locally) and watch the reds happen (AGENTS.md §15
  check 9).
- Run the pre-registered mutants M1-M3 against a temp copy of `scripts/wasm-check.sh` (never
  the working tree) and report which the red-proof kills; read the red-proof for M4.
- Check the spec's claim that `Instant::now()` / `std::fs` compile on wasm32 (the reason the
  red-proof does not use them).
- `just gate-audit`, `just validate`, `just decisions-audit --changed`.
- README link text accurate (it runs the published crate compiled to WebAssembly).
- Output exactly ONE of: ✅ APPROVED (at SHA) / ⚠ PUNCH LIST / ❌ REJECTED, each finding
  labelled `SB-N` or `FU-N`.

## Out of Scope

- A wasm runtime test lane (already filed as a follow-up in the spec).

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
- **Verdict:** ✅ APPROVED at `cb00c38`. AC1-AC7 hold, and no finding blocks the ship.
- **Reds observed by the verifier:** `just wasm-check-red-proof` passed with rc=0. The
  control was green. Mutation A failed with E0433 naming `unix`. Mutation B failed with
  E0080 after the default surface had built. With Homebrew first on PATH, the gate exits 1
  with the named missing-std reason (AC3). A throwaway crate that calls `Instant::now()` and
  `std::fs::read` builds for wasm32 with rc=0, so the spec's limit claim holds.
- **Mutants** (copied gate, run by the copied red-proof): every mutant was killed. M1
  (drop `--target`) and M1b (M1 with the rlib check removed) died. M2 (default surface
  only) and M2b (counter starts at 1) died, M2b on mutation B. M3 (`|| true` on the cargo
  builds) died. M4 was checked by reading the script.
- **Other checks:** gate-audit (15 scripts), validate, fmt-check and decisions-audit
  `--changed` all pass. The CI YAML parses. In CI, `wasm32` passed (13 s) and
  `wasm32-red-proof` passed (31 s). The README claim is confirmed: the demo's
  `demo-core` depends on `irradiance = "0.2.6"`.
- **Process incident (the verifier's own, fixed):** during the first mutant runs, `src`
  was symlinked into the temp dir, so mutation A was appended twice to the real
  `src/lib.rs`. The verifier restored the file with git checkout and reran with real
  copies. The orchestrator confirmed the tree was clean afterwards.

### Cost self-report

- **Tokens (total):** 1,212,540
- **Estimated USD:** 1.68
- **Duration (minutes):** about 4
- **Source of the number:** the subagent's own transcript, deduplicated by message.id

### Drift and new artifacts

- **New decisions emitted:** none
- **Follow-up work identified:**
  - FU-1: AC6's wording is inaccurate. The red-proof is excluded from gate-audit's scan
    by design (gate-audit.sh line 79), so only `wasm-check.sh` is scanned.
  - FU-2: the rlib-exists check is one path for both surfaces. After the default build
    it proves nothing for `write`, which `set -e` and mutation B cover instead.
  - FU-3: the red-proof's `cp -R` keeps a symlinked `src` as a symlink, so a mutation
    can then write into the real tree. Use `cp -RL`.
  - FU-4: the verifier saw the CI pass statuses but not the CI logs.

### Reflection (3 questions, short answers)

1. **What was unclear?** — The mutant setup in the brief allowed a symlinked `src`, and
   that led to the incident.
2. **Missing constraint or decision?** — None.
3. **Differently?** — Copy real files, never symlink, when a script mutates its copy.
