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
  id: HANDOFF-127
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-11
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-057

project:
  id: PROJ-004
  stage: STAGE-013
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

# HANDOFF-127: Test-only DNG writer: uncompressed mono and Bayer fixtures, strip and tile layouts, arbitrary opcode bytes (SPEC-057)

## Delegation Summary

`claude-opus-5` (architect, orchestrator session) hands `SPEC-057` to a cold
`claude-opus-5` session (implementer) for the **build** cycle. The task is to
build `tests/support/dng_writer.rs`, a declarative, test-only DNG writer, plus
`tests/dng_writer.rs`, and to prove it against `dnglab --raw-checksum` and an
irradiance round-trip. `src/` is not touched.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-057-test-only-dng-writer-uncompressed-mono-and-bayer-fixtures-st.md`.
  Read **Design §1–6** and the **design-time probe table** in Context first. The
  probe already established:
  - dnglab accepts a minimal DNG (sensor IFD + `DNGVersion` + `UniqueCameraModel`);
  - dnglab's checksum equals the MD5 of the samples as LE `u16`, for strips and
    for tiles with padded edge tiles, under II and MM;
  - rows that are not a whole number of bytes are unsettled, so the writer refuses
    them.
- **Stage:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/stages/STAGE-013-test-fixture-dng-writer-clean-room-uncompressed.md`
  (the verification plan and the "Reader fidelity" note).
- **Project brief:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/brief.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md`.
- **Existing builders to reuse, not copy-paste:** `tests/support/tiff.rs`:
  - `Order`;
  - `short`, `inline_bytes` and `two_shorts` (big-endian inline packing);
  - the valid builders `cfa_dng_16bit` and `dng_16bit_with_opcode_lists`, which
    this generalises;
  - the test `ljpeg` encoder `lj92_strip` / `lj92_strip_components`, for AC6.
- **Reader under round-trip:** `src/ifd.rs`:
  - the `TAG_*` constants;
  - `Sensor` at `:679`;
  - `Container::sensor` at `:1642`;
  - `sensor_candidates`, `malformed_tags`.

  `src/plane.rs` covers DEC-008's packing and `unpack_into`.
- **Tool and hash helpers:** `tests/support/tools.rs` (`dnglab_available`,
  `exiftool_available`, `run_tool`, skip-loudly pattern) and `tests/support/md5.rs`.
- **Oracle definition:** `docs/oracle-contract.md`, Sensor-plane section
  (`--raw-checksum` = MD5 of the uncropped `u16` plane, LE).

### Decisions that apply

- `DEC-055`: the writer lives in `tests/support` now and is promoted to `src/`
  behind a `write` feature later (SPEC-061). Write it so promotion is a move, not a
  rewrite: `Result`-returning, checked arithmetic.
- `DEC-008`: sub-byte depths are MSB-first bitstreams; byte-aligned depths are
  container-order integers. The writer applies this in reverse.
- `DEC-009`: run the negative control first in the red-proof.
- `DEC-003`: tier-A fixtures are own work. No corpus bytes go into any fixture.
- `DEC-053`: a malformed opcode-list tag is refused at develop. AC8 uses it to show
  the `extra_tags` override reaches the reader.

### Constraints that apply

- `oracle-must-be-shown-red`: the `WriterFault` red-proof (AC9).
- `provenance-recorded-per-algorithm`: the ledger row. The writer is derived from
  TIFF 6.0, DNG 1.7 and DEC-008 **only**.
- `no-copyleft-dependencies`: no reading or porting any writer source: not
  rawler/dnglab, not the AGPL `dng` crate, not the DNG SDK, not LibTIFF. Tools run
  as black boxes only.
- `test-before-implementation`: write the failing tests in `tests/dng_writer.rs`
  first.
- `one-spec-per-pr`: one PR, SPEC-057 only.

### Prior related work

- SPEC-003 / SPEC-045 / SPEC-046 / SPEC-054: the `tests/support/tiff.rs`
  builders and the test `ljpeg` encoder.
- PR #195: PROJ-004 framing (DEC-055, STAGE-013/015/016/017).

## Expected Deliverables

- `tests/support/dng_writer.rs` and `tests/dng_writer.rs` implementing SPEC-057
  AC1–AC12. Every named failing test is present and passing.
- A `docs/provenance-ledger.md` row, and the short `docs/oracle-contract.md`
  paragraph on the probe finding about rows that aren't a whole number of bytes.
- **`src/` unchanged.** `git diff origin/main -- src/` must be empty.
- Run the tool-gated tests **locally with dnglab and exiftool present**, and say
  in the handback that they ran rather than skipped. dnglab 0.7.2 and exiftool are
  on this machine's PATH. Quote the dnglab checksum for one strip case and one
  tile case.
- Run `just test` (it already passes `--all-features`; don't add the flag),
  `just lint-ci`, `just fmt-check`, `just validate`, `just doc-check` and
  `just cost-audit`. All green.
- A PR against `main` from a fresh branch such as `claude/spec-057-build`. The PR
  body references HANDOFF-127, SPEC-057, STAGE-013, PROJ-004 and DEC-055/008/009.
- **Commit-message bodies must be plain prose.** No code tokens (arrows, `#[...]`,
  `--flags`); the push ruleset rejects them. PR bodies and files are fine.
- Do not merge, tag, publish or force-push.

## Out of Scope

- Any encoder or compression inside the writer. Compressed fixtures enter only
  via `Payload::Encoded`.
- Changing `src/` at all. If the reader looks wrong, stop and report it as a
  finding.
- Settling rows that aren't a whole number of bytes (refuse them; see the spec's
  Non-Goals).
- Migrating `tiff.rs`'s existing builders or `examples/fuzz-seeds.rs` onto the
  writer.
- Tiled reading in irradiance (SPEC-058), GainMap (SPEC-059), the public API
  (SPEC-061).
- Installing or running `dng_validate`.

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
