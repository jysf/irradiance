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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model)
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 11150285           # REAL combined count — what cost-audit reads
  estimated_usd: 8.88              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 80
  branch: claude/spec-082-build
  pr: 336
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 017f3954-2929-4417-8fd6-5d796f797cfa.jsonl (61 unique message.ids, all claude-opus-5-5, no subagents): input 124 + cache-creation 207,801 (1-hour) + cache-read 10,888,159 + output 54,201, priced at 5 and 25 USD per million as HANDOFF-191 did. Re-sum at ship. Oracle 2 MD5s pinned on 165a096 before any src edit (commit a1d00cb). exiftool byte-identical PreviewImage (gray fixture and synthetic YCbCr); dng_validate no warnings on K-3 III (own 6192x4128 preview, gray fixture) and Q2M (own 8368x5584 preview), renders equal the no-preview renders."
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

- **Branch / PR:** `claude/spec-082-build`, https://github.com/jysf/irradiance/pull/336
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes. AC1 to AC10 have evidence below. The local gates are green; CI is confirmed in the PR before reporting.
- **Oracle 2's pinned MD5s** were taken on `165a096`, whose `src/` is identical to `386b4a7`'s. They were committed in `a1d00cb`, before any `src/` change, and still pass after it:
  - no metadata: `fb81830e8abeee795eb1c5ecb660209f`
  - every SPEC-081 field: `36160d2e0d02d0b303716dd9a342fff9`
  - FullSensor, masked areas, three opcode lists: `e5081b7db946fdc4461307e78b0ab592`
- **exiftool 13.55 (oracle 6):** `-b -SubIFD:PreviewImage` is byte-identical to the caller's JPEG for the gray fixture (6318 bytes, 64×48, Gray) and for a 9-byte synthetic YCbCr stand-in (odd length, declared 120×80). `SubfileType` 1, `ImageWidth`, `ImageHeight`, `Compression` 7 and `PhotometricInterpretation` (1 or 6) read back as written.
- **dng_validate 1.7.1 (oracle 7, local only):** every SPEC-081 field was set and the framing was StandardCrop. There was no `Warning` or `Error` line in any of these runs:
  - K-3 III with its own 6192×4128 YCbCr preview (6,440,502 bytes)
  - K-3 III with the gray fixture
  - Q2M with its own 8368×5584 YCbCr preview (2,576,384 bytes)

  Each `-16 -tif` render equals the render of the same file without a preview: 6192×4128 for the K-3 III, 8368×5584 for the Q2M, 0 samples differ.
- **Fuzz:** `just fuzz-dng-write` ran 4,009,773 runs in 61 s with no crash. There are 32 seeds: 4 new, and the 28 old ones are each the previous seed plus one inserted 00 byte (the preview flags), checked byte by byte.
- **Gates run locally, all green:**
  - `just test` with `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_DNG_VALIDATE` set
  - `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`
  - `deny`, `deny-fuzz`, `lint-fuzz`, `lint-fuzz-red-proof`
  - `msrv`, `validate`, `gate-audit`, `cost-audit`

### Cost self-report

- **Tokens (total):** 11,150,285 (floor, measured before the handback commit)
- **Estimated USD:** 8.88
- **Duration (minutes):** about 80
- **Source of the number:** the transcript, deduplicated by `message.id` per docs/cost-tracking.md. There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-082 is left proposed.
- **Deviations from spec:**
  - Spec §5 oracle 2 says the hashes come from `main` at `386b4a7`. They were taken on `165a096`, the up-to-date `main`. Its `src/` tree is identical to `386b4a7`'s, as the handoff's brief noted.
  - To share layout and writing between IFD0 and the preview IFD, the IFD0 layout loop and the entry-writing loop moved, unchanged, into `place_ifd` and `write_ifd`. These are generic over the slot count. Oracle 2's hashes prove the no-preview bytes did not change.
  - The fuzz format can set a "tall" flag that adds 65536 to the preview height, so the upper bound is reachable. Its `refuse-preview-dimensions` seed uses it.
  - The SPEC-081 corpus test's metadata options were factored into `source_metadata` and `metadata_options`, so SPEC-082's oracle 7 builds the same file. The SPEC-081 test is otherwise unchanged and still passes.
  - One extra tier-A test, `preview_dimension_and_marker_boundaries_pass`, covers 65535×65535, 1×1 and a two-byte JPEG.
- **Follow-up work identified:**
  - Quick Look was not re-checked on a written file in this build. The design probe already covered it with a Python-built file of the same layout.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. Spec §5 oracle 2 named commit 386b4a7, but the brief explained that 165a096's `src/` is identical. The spec also did not say how the fuzz format should reach height 65536 with a u16, so I added one flag bit.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. One fact would help future writer specs: `tests/support/md5.rs` already exists. With it, a pinned-hash oracle needs no new code.

3. **If you did this task again, what would you do differently?**
   — I would write the raw IFD walker (`raw_ifd`) first. Every layout and tag assertion builds on it, and it keeps the oracle independent of the crate's reader.
