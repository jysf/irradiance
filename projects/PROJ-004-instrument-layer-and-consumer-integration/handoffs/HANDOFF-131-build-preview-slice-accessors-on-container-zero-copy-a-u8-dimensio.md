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
  id: HANDOFF-131
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-12
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-062

project:
  id: PROJ-004
  stage: STAGE-018
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

# HANDOFF-131: Preview slice accessors on Container (SPEC-062)

## Delegation Summary

`claude-opus-5` (the architect, in the orchestrator session) hands `SPEC-062` to a cold
`claude-opus-5` session (the implementer) for the **build** cycle. The task is
`Container::previews()`, a zero-copy, allocation-free iterator over a DNG's embedded
preview IFDs, selected per **DEC-057**. The 21 previews in the held corpus are pinned from
exiftool and checked. Nothing else in the crate changes behaviour.

## Context the Receiving Agent Needs

### Primary

- **The spec is the contract:**
  `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-062-preview-slice-accessors-on-container-zero-copy-a-u8-dimensio.md`.
  Read **Design §1** first. The probe already gives:
  - every file's preview list, by IFD index;
  - exiftool's per-IFD tag names (`PreviewImage*`, `JpgFromRaw*`, `Strip*`);
  - the group-to-index mapping;
  - the fact that `scalar`/`uints` allocate.
- **Decision:** `./decisions/DEC-057-previews-are-newsubfiletype-1-or-65537-returned-as-one-contiguous-zero-copy-slice.md`.
  Do not reopen its rejected alternatives: no bit-0 selection, no size rule, no strip
  iterator, no stop-at-first-error, no decoding, no MakerNote previews.
- **Stage:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/stages/STAGE-018-instrument-layer-a-preview-slice-accessors.md`.
- **Why it exists:** `./feedback/2026-11-11-monorail-000-requests.md`, item 5.
- **Toolchain brief:** `./guidance/toolchain-brief.md` (lints, fuzz PATH, msrv,
  `IRRADIANCE_CORPUS_DIR`).
- **Fixtures:**
  - `tests/support/tiff.rs`, the raw IFD builder, for hostile and multi-IFD fixtures;
  - `tests/support/dng_writer.rs`, whose `IfdLayout::PreviewIfd0WithSensorSubIfd`
    gives a real-shaped file with one 1×1 preview in IFD0.
- **Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
  All 11 DNGs are already there and in the manifest. Use them in place; never copy them
  or `git add` them.

### Decisions that apply

- `DEC-057`: the selection values, contiguous strips only, per-IFD `Result` items, and
  no allocation.
- `DEC-012`: unknown is not no. An unreadable `NewSubfileType` is an `Err` item, never a
  silent skip.
- `DEC-051`: `Error::kind()` has no wildcard, so both new variants must be classified.
- `DEC-052`: the new public types are `#[non_exhaustive]`.
- `DEC-003`: corpus bytes are never committed.

### Constraints that apply

- `no-panics-on-untrusted-input`: every hostile layout gives a typed error, and the
  crate-root lints apply.
- `oracle-must-be-shown-red`: the AC8 mutants, honest run first.
- `provenance-recorded-per-algorithm`: a ledger row citing TIFF 6.0 §8, DNG 1.7 and
  DEC-057.
- `no-copyleft-dependencies`: exiftool is a black box only. Do not read its source or
  another reader's.
- `test-before-implementation`: the spec's failing tests come first.
- `one-spec-per-pr`.

### Prior related work

- **SPEC-057** (#199/#200): the fixture writer.
- **SPEC-040 / DEC-051:** `ErrorKind`.
- **SPEC-042 / DEC-052:** the sealing convention.
- **SPEC-039:** the manifest and tier-B skip-loudly pattern this extends.

## Expected Deliverables

- SPEC-062 AC1 to AC12, with every named failing test present and passing.
- **Corpus pins taken from exiftool 13.55, never from irradiance**, using the procedure
  in spec §6. Report that the tier-B tests **ran**, not skipped, and that all 21 previews
  matched.
- **Fuzz:** run the extended `ifd` target for at least 10 minutes with the new seeds.
  Report the run count and that there were no crashes.
- **Gates all green:**
  - `just test` (it already passes `--all-features`; do not add the flag)
  - `just lint-ci`
  - `just lint-no-allow`
  - `just fmt-check`
  - `just validate`
  - `just doc-check`
  - `just deny`
  - `just deny-fuzz`
  - `just msrv`
  - `just cost-audit`
  - `just gate-audit`
- A PR against `main` from a fresh branch such as `claude/spec-062-build`. The PR body
  references HANDOFF-131, SPEC-062, STAGE-018, PROJ-004 and DEC-057/012/051/052.
- **Commit-message bodies are plain prose.** No code tokens such as arrows, `#[...]` or
  `--flags`; the push ruleset rejects them. PR bodies are fine. `Cargo.lock` is
  gitignored; never add it.
- Never merge, tag, publish, enable auto-merge or force-push.

## Out of Scope

- Decoding previews, or checking JPEG markers.
- MakerNote or EXIF-IFD thumbnails.
- Supporting tiled, `JPEGInterchangeFormat` or non-contiguous previews. They are refused.
- The ADU histogram, zones, bad pixels, and any develop change.
- Changing `payload`, `uints` or `scalar`.
- A version bump or release.

If exiftool and irradiance disagree on a corpus file, **stop and report it**. Do not
adjust the pin to match.

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
