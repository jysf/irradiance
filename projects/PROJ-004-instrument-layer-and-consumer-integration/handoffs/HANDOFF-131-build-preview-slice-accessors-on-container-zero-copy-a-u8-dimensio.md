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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 16626847         # REAL combined count — what cost-audit reads
  estimated_usd: 12.64           # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 30
  branch: claude/spec-062-build
  pr: "https://github.com/jysf/irradiance/pull/208"
  completed_at: 2026-11-12         # YYYY-MM-DD
  notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript (1 jsonl, 72 unique message.ids, all claude-opus-5-5, no subagents): input 148 + cache-creation 256,617 (1-hour) + cache-read 16,292,917 + output 77,165 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 15,119,257 / 11.81 before the final commit and PR turns."
  synced_at: 2026-11-12
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

- **Branch / PR:** `claude/spec-062-build` → [PR 208](https://github.com/jysf/irradiance/pull/208)
- **Completed at:** 2026-11-12
- **All acceptance criteria met?** yes. AC1 to AC12 are each covered by a named test
  or gate:
  - **AC1 to AC5:** `tests/previews.rs`, both byte orders.
  - **AC6:** `tests/previews_alloc.rs`. It counts 0 allocations while draining 3 Ok and
    3 Err items. A negative control proves the counter counts.
  - **AC7:** the tier-B test **ran, not skipped**, with
    `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
    **21/21 previews matched the exiftool 13.55 pins** on all eight fields, plus the
    pointer check. The PEF was reported as not pinned, by name.
  - **AC8:** `red_proof_preview_mutations`. The control ran first in a staged copy and
    matched the in-process tree. Then all three mutants went red: selection widened,
    one strip only, and range off by one.
  - **AC9:** `tests/error_kind.rs`, now 61 variants, split 12 / 20 / 29.
  - **AC10:** no existing assertion changed. The only edits to existing tests are
    `error_kind`'s required rows and counts.
  - **AC11:** the extended `ifd` fuzz target ran for 601 s: **28,847,288 runs, no
    crash** (exit 0, no artifact).
  - **AC12:** every listed gate is green (below).
- **Gates:** all green on this branch:
  - `just test` (with the corpus present): 598 passed, 0 failed, 1 ignored (the existing SPEC-017 Q2M measurement); no corpus file skipped
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
- **Pins come from exiftool, never from irradiance.** They were taken before any
  implementation existed. A throwaway script parsed `exiftool -j -a -G1 -s -n` using
  the spec §6 tag list, and the result was transcribed into the manifest. exiftool and
  irradiance never disagreed.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 15119257 (a floor: measured before the final commit and PR turns)
- **Estimated USD:** 11.81
- **Duration (minutes):** 30
- **Source of the number:** this session's own transcript (`basis: transcript`).
  - Deduplicated by `message.id`, taking the max `output_tokens` per message. There
    were no subagents.
  - Priced per component at opus-5-5 rates: $5/Mtok in, $25/Mtok out, cache read at
    0.10×, 1-hour cache write at 2.0×.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **The "length sum that overflows" case (AC5) cannot reach
    `ValueOverflow { tag: 279 }`.**
    - Summed in `u64`, at most 2^20 `u32` lengths cannot overflow. A contiguous run
      whose true end passes `u32::MAX` also cannot be followed by a `u32` offset.
    - So the fixture is the dangerous shape: two strips that look contiguous only in
      wrapping `u32` arithmetic (`0x4000 + 0xFFFF_FFF0` wraps to `0x3FF0`). Their
      wrapped total would be an in-bounds 16-byte lie.
    - It is refused as `NonContiguousStrips`. A single `u32::MAX` length is refused
      as `Truncated` with the true length.
    - The `checked_add`/`ValueOverflow` arms stay as defensive code, as the spec says.
  - **The new fixtures are not in `tiff::all()`.** `tests/ifd_reader.rs` sweeps
    `all()` and lists every shape it expects to parse, so adding them there would
    edit an existing test (AC10). `tiff::preview_seeds()` is written into the same
    `fuzz/seeds/ifd` directory instead.
  - **The red-proofs follow the repo's staged-copy pattern** (`ljpeg_shape_binding`).
    A temp copy of `src/` gets a probe bin that prints each `previews()` item.
    - The control must equal the in-process answer.
    - Each mutant must change the exact line its named test asserts on.
    - The test is not `#[ignore]`d. It runs in `just test` and takes a few seconds,
      since the staged build is incremental.
  - **`uint_at` refuses `RATIONAL` outright.** `uints` admits it for non-structural
    tags, but every tag on the preview path is structural, so the behaviour is
    identical where it is used.
  - **An unreadable optional tag is an `Err` item** (`Compression`, `Photometric`,
    `SamplesPerPixel` or `BitsPerSample` of the wrong type or out of the file). It
    does not fall back to the default (DEC-012). Absent still takes the default.
  - **A `NewSubfileType` entry with count 0 reads as absent (0, not a preview)**,
    matching `scalar`/`is_sensor_ifd`.
- **Follow-up work identified:**
  - None blocking. If a consumer ever needs tiled or non-contiguous previews, DEC-057
    names the one arm to relax (a strip/tile iterator on the non-exhaustive
    `Preview`).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only the AC5 "length sum that overflows" bullet. With `u64` sums and a contiguity
   check it has no reachable `ValueOverflow` path. The real hazard is a wrapped `u32`
   sum, and the fixture now targets that. Everything else, including the probe table,
   group mapping and §4 order, was transcription.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two, both minor:
   - `tests/ifd_reader.rs` asserts on the full `tiff::all()` list. Anyone adding fuzz
     seeds to `all()` would edit an existing test, so the handoff could have said to
     keep new seeds out of `all()`.
   - The staged-copy red-proof precedent (`tests/ljpeg_shape_binding.rs`) is the
     natural AC8 pattern, and could be named outright.

3. **If you did this task again, what would you do differently?**
   — I would take the exiftool pins even earlier, as the first act, before reading any
   `src/`. Here they came before any implementation, which is what mattered. I would
   also start the 10-minute fuzz run as soon as the target compiles, since it is the
   longest pole beside the tier-B oracles in `just test`.
