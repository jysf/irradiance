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
  id: HANDOFF-193
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-24
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-082

project:
  id: PROJ-005
  stage: STAGE-017
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

# HANDOFF-193: Build SPEC-082, the writer's optional caller JPEG preview

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-082` to `claude-opus-5` (implementer) for the
**build** cycle: extend `irradiance::dng::write` (SPEC-061 and SPEC-081, shipped) with an
optional JPEG preview stored as a SubIFD of the raw IFD0, so the spec's failing tests pass,
on a branch cut from `main`.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-082-optional-caller-jpeg-preview-as-a-reduced-resolution-subfile.md`.
  Read Design §1 to §7, the probe table and the rule-clause map. The API names in §1, the
  tag table in §2 and the layout in §3 are binding.
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
- **Project brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`
- **Prior specs:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/done/SPEC-061-*.md` and `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/done/SPEC-081-*.md`, with
  their handoffs HANDOFF-175/190 and HANDOFF-191/192: the writer you extend and its oracles.
- **Reader:** `src/ifd.rs` `Container::previews` (SPEC-062, DEC-057) is the round-trip
  oracle. Do not change it.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.
- **Corpus:** set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
- **Adobe `dng_validate`:** set
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
  for AC8. Local only; never in CI; never commit its outputs.

### Decisions that apply

- `DEC-082` (proposed, this design): SubIFD placement, layout, the preview IFD's tags, caller
  descriptors, the SOI and dimension checks and their place in the order.
- `DEC-081` §4 (accepted): the check order you extend (6e after XMP, before the walk).
- `DEC-071` §5, `DEC-072`, `DEC-075` §1 (accepted): the scope and the layout around it.
- `DEC-057`: what the reader treats as a preview and how it returns the bytes.
- `DEC-051`, `DEC-052`, `DEC-058`, `DEC-016`, `DEC-069` §6.

### Constraints that apply

- `no-panics-on-untrusted-input`: `just lint-no-allow` must pass.
- `oracle-must-be-shown-red`: oracle 5, negative control first; oracle 2's hashes taken on
  `main` before the change.
- `provenance-recorded-per-algorithm`: the ledger row in Design §7.
- `no-copyleft-dependencies`: no new dependency at all.
- `library-not-application`: no I/O in `src/dng`.

## Expected Deliverables

- Every file in SPEC-082's `## Outputs`, and every test in `## Failing Tests` passing.
- `just test` (with the corpus and `IRRADIANCE_DNG_VALIDATE` set), `just lint-ci`,
  `just lint-no-allow`, `just fmt-check`, `just doc-check`, `just deny`, `just deny-fuzz`,
  `just lint-fuzz`, `just lint-fuzz-red-proof`, `just msrv`, `just validate`,
  `just gate-audit`, `just cost-audit`, and 60 s of `just fuzz-dng-write`, all green.
  **Before handing back, confirm every job in `gh pr checks` for your PR passes.**
- A PR against `main` from `claude/spec-082-build`, naming HANDOFF-193, SPEC-082,
  STAGE-017, PROJ-005, DEC-057/071/081/082 and any new DEC. Commit messages plain prose.
- In the handback: the three pinned MD5s and the commit they were taken on, the exiftool and
  `dng_validate` results per output, and the token count.

## Out of Scope

- Preview in IFD0 with the raw plane in a SubIFD; alternate previews; `PreviewColorSpace`.
- Decoding, encoding or resizing JPEG in `src/`; checking more than the SOI marker.
- Any reader change. If a round trip exposes a reader bug, stop and report it.
- The [XS] backlog items (empty opcode lists, the CI fuzz job, private-items doc links).

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
