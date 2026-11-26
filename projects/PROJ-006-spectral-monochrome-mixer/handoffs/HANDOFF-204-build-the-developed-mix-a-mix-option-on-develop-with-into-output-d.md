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
  id: HANDOFF-204
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-26
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-093

project:
  id: PROJ-006
  stage: STAGE-031
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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-204: The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-093` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle: one new `DevelopOptions` field,
`mono_mix: Option<[Rational; 3]>`, with which the four mono develop entries develop a
colour Bayer DNG to SPEC-092's mix at the mono develop's geometry; one new
`DevelopPath::MonoMix`; the mixer run in 512-row strips into caller scratch; an
allocation-only refactor of the fused traversal; no new `Error` variant, no version bump,
and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's three reply sections)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md` (Design §1 to §11, AC1 to AC14, Failing Tests, mutants (a) to (z)). The spec is the contract.
- **Decision:** `./decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md` (proposed; the rule, the opcode set, the strips, the scratch, the order, the versions)
- **The shipped siblings:** `specs/done/SPEC-092-...md` and `specs/done/SPEC-091-...md`, and their handoffs HANDOFF-202 / HANDOFF-203 (`## Handback`: the mutant-harness method, the `Entry`-table count changes, the CI legs).
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` traps for `cargo fuzz` and `lint-ci`).

### Decisions that apply

- `DEC-084` — this spec's decision: the developed mix is the mono develop of the mixed plane.
- `DEC-083` — the weights rule, the row and the kernel the mix uses, unchanged.
- `DEC-079` — weights on linear sRGB, folded into the camera matrix.
- `DEC-058` — `DevelopOptions` is `#[non_exhaustive]`, its default is `develop_into`, options never skip validation.
- `DEC-059` — `warp_alpha` and the `FullActiveArea` bounds rule, unchanged under the mix.
- `DEC-060` — one exact-length caller scratch slice, private layout, no allocation in `develop_with_scratch_into`.
- `DEC-043` — the colour develop's crop/orient scatter, reused for the no-warp mix.
- `DEC-044` — the halo'd row bands the strips run on.
- `DEC-049` — required-processing validation; `DevelopPath::MonoMix` extends it.
- `DEC-068`, `DEC-070` — the colour levels the mix uses (so `black_levels` is not read).
- `DEC-016`, `DEC-002` — caller-owned buffers; determinism.

### Constraints that apply

- `no-panics-on-untrusted-input` — every new path is checked arithmetic and `.get`; fuzzed (AC12).
- `oracle-must-be-shown-red` — AC14's hand-applied mutants.
- `test-before-implementation` — write the Failing Tests first and see them fail (SPEC-092's build did not; do).
- `provenance-recorded-per-algorithm` — no new algorithm, no ledger row (say so in the PR).

### Prior related work

- `HANDOFF-202` / PR 354 — the mixer at sensor geometry this develops.
- `HANDOFF-203` — the verify that found SPEC-092/FU-5 and FU-6, which this spec's AC9 and AC8 answer.
- PR 352, PR 353 — SPEC-092's design and onyx's reply.

### Facts the design probe measured (2026-11-26, release build, this machine)

- Opcode lists on the held colour files: Canon and Nikon none in `OpcodeList3`; Pixel 3a and 7 Pro an **optional non-identity** radial `WarpRectilinear` (no tangential terms); Pixel 4a an optional **identity** warp (skipped). Pixel 7 Pro is Orientation 3. The Q2 Monochrom's `FixBadPixelsConstant` and `WarpRectilinear` are both **mandatory** (flags 0).
- AC4's digests and `scratch_len` numbers were computed by the §2 oracle, not by an implementation. Record them as given; a mismatch is reported.
- Strips of 512 rows through the public band entry cost 0.99× to 1.06× of the whole-plane mix (256 rows: up to 1.12×).
- Probe source (not in the repo): `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b/spec-093-design-probe.rs.txt`. It ran as a throwaway `examples/` file with `cargo run --release --all-features --example`.

## Expected Deliverables

- Code changes implementing SPEC-093's Acceptance Criteria (Design §1 to §11).
- All Failing Tests in SPEC-093 written first, seen failing, then passing.
- The mutant table (AC14) with the test each mutant failed; (u), (y), (z) argued.
- A PR against `main` from branch `feat/spec-093-developed-mix`.
- PR description referencing: HANDOFF-204, SPEC-093, STAGE-031, PROJ-006, DEC-084 and the
  other DECs above (this repo's `decisions/` namespace), provenance "none — no new
  algorithm", and the oracle (the §2 mono-develop oracle, the AC4 digests, the AC11
  hashes, and the mutants as its red-proof).

## Out of Scope

- A CFA-aware `FixBadPixelsConstant`, or any change to how SPEC-092's sensor-geometry
  entries treat a mandatory warp (the spec's Open question 1).
- A new `Error` variant, a `DEVELOP_VERSION` or `MONO_MIX_VERSION` bump, a public strip
  size, threaded variants, an `irr` hook, presets.
- Any change to an existing output, hash or Tier-B pin.

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
