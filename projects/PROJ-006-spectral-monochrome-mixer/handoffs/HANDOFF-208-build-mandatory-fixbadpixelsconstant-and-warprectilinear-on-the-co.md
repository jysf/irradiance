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
  id: HANDOFF-208
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-27
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-098

project:
  id: PROJ-006
  stage: STAGE-034
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

# HANDOFF-208: Mandatory FixBadPixelsConstant on the colour entries and WarpRectilinear passed through at sensor geometry, the Q2, Q3 and SL3 files

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-098` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. The colour develop, the sensor-geometry
planes and the developed mix apply `OpcodeList1`'s first `FixBadPixelsConstant` on the raw
CFA, filling each bad photosite from its four nearest same-colour photosites (colour from
`CFAPattern`; `BayerPhase` unused). A new `DevelopPath::SensorGeometry` lets the planes
pass a mandatory `WarpRectilinear` through to the consumer. The developed colour still
refuses it, and the developed mix refuses a mandatory one with `N ≠ 1`; both wait for
SPEC-099. The Q2, Q3 and SL3 are promoted to `[[file]]` entries. There is no new `Error`
variant, no version bump and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md` (Design Notes: onyx's two rules)
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-098-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md` (Design §1 to §9, Implementation Context, AC1 to AC11, Failing Tests, the rule-clause map, mutants (a) to (r), the moved assertions and the count sites). The spec is the contract.
- **Decision:** `./decisions/DEC-086-the-colour-entries-fix-bad-pixels-from-the-four-nearest-same-colour-photosites-and-sensor-geometry-passes-the-warp-through.md` (proposed)
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-086`: this spec's decision: who applies the fix, where, the kernel, `BayerPhase`, the new path, the warp rules, the error order.
- `DEC-084` item 3, as amended (dated note): the developed mix's opcodes.
- `DEC-040`: the CFA colour is indexed from the `ActiveArea` origin with `wrapping_sub`.
- `DEC-053`, `DEC-049`: `check_required_processing` is every entry's first question; the paths say what each composition applies.
- `DEC-078` §2: `SensorGeometry` refuses the colour path's three malformed-tag sets.
- `DEC-069` §6: the SDK is read and run locally only; never vendored, built in CI or linked; none of its output is committed.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.

### Constraints that apply

- `no-panics-on-untrusted-input`: the kernel uses checked arithmetic only; the `opcode` fuzz target gains the four-path chain (AC10).
- `oracle-must-be-shown-red`: AC1's literals were computed by the Adobe SDK; mutants (a) to (r) must each turn a named test red.
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: one ledger row for the kernel (Design §9).

### Prior related work

- SPEC-017 (the mono applier), SPEC-054/SPEC-056 (the required-processing check), SPEC-091, SPEC-092, SPEC-093 (the entries), SPEC-097 (the last manifest promotion; its verify found two missed count sites, all listed this time).

### Facts the design probe measured (2026-11-27, release build, this machine)

- **The full table is in the spec's `## Implementation Context`:** opcode facts, bad-photosite counts, the SDK comparisons, the synthetic literals, the digests per entry, and what each entry returns.
- **AC7's digests are pins from the prototype, not an oracle.** Record them as given, and report a mismatch rather than editing it.
- **The prototype passed the whole corpus suite** except the 13 moved assertions the spec lists; `ac6_the_mix_never_parses_opcode_list_1` also moves once the mix parses list 1 (the prototype did not).
- **Probe sources** (not in the repo), in `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b`: `spec-098-design-probe.rs.txt` (every entry on the three files), `spec-098-design-sdk.rs.txt` (stage 1/2 comparisons), `spec-098-design-synth.rs.txt` (the prototype against SDK-fixed mosaics), `spec-098-design-synth-dng.py.txt` (the synthetic DNG writer), `spec-098-design-prototype.diff.txt` (the uncommitted src diff), and `spec-098-design-probe-output.txt`.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. **Never copy any `-PB` file anywhere in the repo.**

## Expected Deliverables

- Code and doc changes implementing SPEC-098's Acceptance Criteria (Design §1 to §9).
- All Failing Tests written first, seen failing against `main`, then passing; every moved assertion moved as listed.
- The mutant table (a) to (r), with the test each mutant failed.
- The `opcode` fuzz smoke with the new seed.
- A PR against `main` from branch `feat/spec-098-colour-fix-bad-pixels`, referencing HANDOFF-208, SPEC-098, STAGE-034, PROJ-006, DEC-086, DEC-084, DEC-040 and DEC-053, with the provenance row and the oracle (the SDK-computed literals) stated.

## Out of Scope

- The warp on `develop_color_*` and the developed mix, the per-plane parser and accessor: SPEC-099.
- A public CFA bad-pixel applier; the demosaic and colour-transform primitives keep refusing list 1.
- Any change to the mono path, `src/warp.rs`, `DEVELOP_VERSION`, `MONO_MIX_VERSION`, the `Error` enum, or any existing output, hash or Tier-B pin.
- Committing, copying or redistributing any `-PB` corpus file.

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
