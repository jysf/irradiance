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
  id: HANDOFF-206
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-27
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-097

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

# HANDOFF-206: Accept a lossless-JPEG precision below BitsPerSample, the Leica M10 file

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-097` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle. `check_precision` accepts a
Compression 7 frame when `P ≤ BitsPerSample ≤ 16`, not only when `P = BitsPerSample`. The
samples are stored as decoded, and white still defaults from `BitsPerSample`. The Leica
M10 and M11 colour files are promoted to `[[file]]` manifest entries and pinned. There is
no new `Error` variant, no version bump and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-097-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md` (Design §1 to §7, Implementation Context, AC1 to AC8, Failing Tests, mutants (a) to (i)). The spec is the contract.
- **Decision:** `./decisions/DEC-085-a-lossless-jpeg-precision-below-bitspersample-decodes-unchanged-and-white-still-defaults-from-bitspersample.md` (proposed; the rule, the samples, the white default, what does not change)
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` trap for `cargo fuzz`).

### Decisions that apply

- `DEC-085`: this spec's decision. The rule is `P ≤ BitsPerSample ≤ 16`; samples are unshifted; white comes from `BitsPerSample`.
- `DEC-077` §3, as amended by DEC-085: where the check sits, its error and its paths, all unchanged.
- `DEC-029`: samples above `WhiteLevel` are legal and clip at white. The M10 has 5.2% of its samples there.
- `DEC-047`: the frame's shape binding, which still runs before the precision check.
- `DEC-061` / `DEC-070`: the mono and colour level maps, unchanged.
- `DEC-003`: tier-B files are never committed; tests skip by name when a file is absent.
- `DEC-069` §6: the SDK is read and run locally only. It is never vendored, built in CI or linked, and none of its output is committed.

### Constraints that apply

- `no-panics-on-untrusted-input`: the check is two comparisons; the `plane` fuzz smoke covers the newly reachable decode (AC8).
- `oracle-must-be-shown-red`: the dnglab checksum oracle's existing red-proof covers the two new files; mutants (a) to (i).
- `test-before-implementation`: write the Failing Tests first and see each fail (or refuse) against `main`.
- `provenance-recorded-per-algorithm`: no new algorithm. Add one sentence to the ledger's lossless-JPEG row for DEC-085's class-4 SDK reading (Design §7).

### Prior related work

- SPEC-086 / DEC-077: the check being relaxed, and `tests/ljpeg_precision_binding.rs`.
- SPEC-044: the dnglab `--raw-checksum` plane oracle and its Compression 7 red-proof.
- SPEC-062: the manifest's preview pins and their exiftool procedure.

### Facts the design probe measured (2026-11-27, release build, this machine)

- **The full table is in the spec's `## Implementation Context`:** SOF-3 headers, sample
  facts, dnglab checksums, stage-1 and stage-2 agreement, and the FNV-1a digests per entry.
- **Both files reach every entry with `Ok`** once the check is relaxed. Neither has
  opcode lists.
- **AC7's digests are pins from the prototype, not an oracle.** Record them as given, and
  report a mismatch rather than editing it.
- **The manifest values in Design §5** were measured on disk: sha256, bytes, dnglab
  checksums, and exiftool's preview pins.
- **Probe source** (not in the repo): `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b/spec-097-design-probe.rs.txt`.
  - It ran as a throwaway `examples/` file with `cargo build --release --example` against
    a `check_precision` patched to `>`.
  - It reads `dng_validate` stage-1 and stage-2 TIFFs from a scratch directory. The build
    does not need them.
- **The corpus:** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  The M11 lives at `LEICA-M11-PB/leica_m11_01.dng`. **Never copy it anywhere in the
  repo.**

## Expected Deliverables

- Code and doc changes implementing SPEC-097's Acceptance Criteria (Design §1 to §7).
- All Failing Tests written first, seen failing against `main`, then passing.
- The mutant table (a) to (i), with the test each mutant failed.
- The `plane` fuzz smoke run with the two new seeds.
- A PR against `main` from branch `feat/spec-097-ljpeg-precision-below-bps`.
- The PR description references HANDOFF-206, SPEC-097, STAGE-034, PROJ-006, DEC-085,
  DEC-077, DEC-029 and DEC-047. It also gives:
  - provenance: no new algorithm, plus the ledger sentence;
  - the oracle: dnglab's checksum on both files, and the red-proof that covers them.

## Out of Scope

- Mandatory `FixBadPixelsConstant` / `WarpRectilinear` on the colour entries (the Q2, Q3,
  SL3): STAGE-034's next spec.
- `P > BitsPerSample` or `BitsPerSample > 16`: still refused.
- Any change to the white default, the decoder, `resolve_white` or
  `resolve_colour_levels`.
- A new `Error` variant, a version bump, or any change to an existing output, hash or
  Tier-B pin.
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
