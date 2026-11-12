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
  id: HANDOFF-129
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-11
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-058

project:
  id: PROJ-004
  stage: STAGE-015
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

# HANDOFF-129: Tiled plane decode, uncompressed and lossless-JPEG tiles with edge tiles, allocation-free (SPEC-058)

## Delegation Summary

`claude-opus-5` (architect, orchestrator session) hands `SPEC-058` to a cold
`claude-opus-5` session (implementer) for the **build** cycle. The task is to
teach `plane::validate` and `plane::unpack_into` to read tiled sensor planes,
both uncompressed and Compression-7, including partial edge tiles. Per
**DEC-056** this is allocation-free, through a clipped, strided reconstruct
window. The three Pixel phone DNGs must unpack bit-exact against dnglab, and
every strip file must stay unchanged.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-058-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Read the design-time probe table and findings first. The probe already
  established three things, so you will not have to rediscover them:
  - Today's decoder decodes single Pixel tiles bit-exact against dnglab: interior,
    right-edge and corner tiles. P=16 and Nf=2 work, and the codec needs no change.
  - Every one of the 192 tiles per file is a full 256×256 frame, edge tiles
    included.
  - Sensor selection already finds the SubIFD.
- **Decision:** `./decisions/DEC-056-tiled-planes-decode-in-place-through-a-clipped-strided-window.md`.
  Do not reopen its rejected alternatives: no scratch buffer, no caller scratch,
  no new entry point.
- **Stage:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/stages/STAGE-015-tiled-dng-plane-reader-uncompressed-and-lossless-jpeg-tiles.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` (lints, fuzz, msrv, the
  corpus env var).
- **The fixture writer (SPEC-057):** `tests/support/dng_writer.rs`.
  - Use `Organisation::Tiles { tile_width, tile_length, edge_fill }`.
  - Uncompressed tiles come from `Payload::Samples`.
  - Compression-7 tiles come from `Payload::Encoded { compression: 7, chunks }`,
    with one `tiff::lj92_strip_components(..)` chunk per tile, each encoding the
    full tile including padding.
  - The writer needs `tiff.rs` mounted as `mod tiff`.
- **The probe's per-tile checksums** (Pixel 4a, as one-strip DNGs) are a
  debugging aid:
  - tile 0: `a340b03173c939060b228de1c1b49b29`
  - tile 15: `a3cf5c3297d02d88076426ac03311423`
  - tile 191: `c9c984f5bfa594dd5f70c676c3a09180`
  - Pixel 7 Pro tile 191: `1dbd6e768c13c3f7179dbbea470de6a5`

### Decisions that apply

- `DEC-056`: in-place window decode, discard right-of-plane differences, stop after
  the last visible row, bind each tile's frame shape to the tile.
- `DEC-016`: `unpack_into` allocates nothing. This must remain true.
- `DEC-008`: per-row packing rules. Rows that aren't a whole number of bytes are
  refused, per the SPEC-057 probe.
- `DEC-047`: frame-shape binding, now applied per tile.
- `DEC-050`: validate-before-allocate parity. `validate` and `unpack_into` share
  one structural pass.
- `DEC-051`: `Error::kind()` has no wildcard. Every new variant must be classified.
- `DEC-012`: tile tags are structural; a malformed one fails the read.
- `DEC-049`: the Pixel files must still be refused at develop (mandatory GainMap).
- `DEC-003`: corpus bytes are never committed.
- `DEC-009`: run the negative control before each red-proof.

### Constraints that apply

- `no-panics-on-untrusted-input`: every hostile tile table is a typed error. The
  crate-root lints apply.
- `oracle-must-be-shown-red`: the window-clip red-proof (AC7).
- `provenance-recorded-per-algorithm`: the ledger row cites TIFF 6.0 §15, DNG 1.7,
  T.81 Annex H and DEC-056.
- `no-copyleft-dependencies`: never read rawler, LibTIFF or the DNG SDK. dnglab is
  a black box only.
- `test-before-implementation`: write the failing tests first.
- `one-spec-per-pr`.

### Prior related work

- **SPEC-057** (#199, #200): the fixture writer.
- **SPEC-046 / DEC-037:** Nf=2 decode.
- **SPEC-039 / DEC-050:** validate-before-allocate.
- **SPEC-053 / DEC-047:** shape binding.
- **SPEC-054 / DEC-049:** required processing.

## Expected Deliverables

- SPEC-058 AC1–AC11, with every named failing test present and passing.
- **Corpus, machine-local:** copy the three Pixel files from
  `~/PSeven/experiments/crustimg_redo_plus/data.lfs/Google/<model>/` into
  `$IRRADIANCE_CORPUS_DIR` (`~/PSeven/experiments/crustimg_redo_plus/images`), under
  `GOOGLE-PIXEL-4A/`, `GOOGLE-PIXEL-3A/` and `GOOGLE-PIXEL-7-PRO/`. Copy them; do
  not symlink. Add the manifest entries with the pinned values from the spec's
  table. Never `git add` a corpus file.
- **Run the tier-B tests with the corpus present.** Report that they ran rather than
  skipped, and give the three Pixel checksums plus confirmation that every
  pre-existing corpus checksum is unchanged.
- **Fuzz:** run each plane-reaching target for at least 10 minutes with the new
  tiled seeds. Report the run counts and that there were no crashes.
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
- A PR against `main` from a fresh branch such as `claude/spec-058-build`. The PR
  body references HANDOFF-129, SPEC-058, STAGE-015, PROJ-004 and DEC-056/016/047.
- **Commit-message bodies are plain prose.** No code tokens such as arrows,
  `#[...]` or `--flags`; the push ruleset rejects them. PR bodies are fine.
  `Cargo.lock` is gitignored; never add it.
- Never merge, tag, publish, enable auto-merge or force-push.

## Out of Scope

- Applying the GainMap or any opcode (SPEC-059/060). The Pixel files must still be
  refused at develop.
- Fractional or malformed `BlackLevel` on the Pixel 7 Pro. Record it as a finding if
  it bites.
- New compression schemes, PlanarConfiguration 2, or parallel tile decode.
- A scratch buffer or a new public entry point. If the window argument seems to
  fail in the code, stop and report it.
- A version bump or release.

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
