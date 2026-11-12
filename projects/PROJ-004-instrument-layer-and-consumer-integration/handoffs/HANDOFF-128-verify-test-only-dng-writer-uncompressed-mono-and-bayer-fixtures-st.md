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
  id: HANDOFF-128
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-128: VERIFY: test-only DNG writer, strip and tile layouts, arbitrary opcode bytes (SPEC-057)

## Delegation Summary

You are the **cold verifier** for SPEC-057. You did **not** implement it. Review it
against SPEC-057 and DEC-055 and return exactly one verdict.

- **The build:** **PR #198**, branch `build/spec-057-test-only-dng-writer`, cut
  from `origin/main` at `02a95d4`. The head when this handoff was minted is
  `9e897a1`; its last commit is the handback docs only.
- **Your branch:** your worktree is cut from `claude/spec-057-verify-base`. That is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under
  review is the build's code.

The build delivers:
- `tests/support/dng_writer.rs`: a declarative, test-only DNG writer. Its parts are
  `DngDescription`, `write_dng`, `WriteError`, `Payload::{Samples, Encoded}`,
  `Organisation::{Strips, Tiles}`, `IfdLayout`, `extra_tags` overrides, and a
  `WriterFault` red-proof knob.
- `tests/dng_writer.rs`: 12 tests.
- A `docs/provenance-ledger.md` row and a `docs/oracle-contract.md` paragraph.

**`src/` must be unchanged.**

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-057-test-only-dng-writer-uncompressed-mono-and-bayer-fixtures-st.md`
  (all of it, especially the design-time probe table, Design §2–5, and AC1–AC12).
- **Decision:** `./decisions/DEC-055-...` (writer placement). DEC-008 covers
  packing: sub-byte is an MSB-first bitstream; byte-aligned is container order.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-127-build-test-only-dng-writer-uncompressed-mono-and-bayer-fixtures-st.md`.
  Treat its claims as claims and check them yourself. It declares five deviations:
  - three extra `WriteError` variants;
  - a 12-bit edge fill of `0xBEEF & 0xFFF`;
  - `Order` reused from `tiff.rs`;
  - LONG tile tags;
  - inline `extra_tags` keyed on byte length.

  Judge whether each one is sound.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test`
  already passes `--all-features`; add no flag.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC12 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
- Run `just decisions-audit --changed`.
- Confirm `git diff origin/main -- src/` is empty.

### The §12 behavioural surface (this is where defects escape)

**The tool-gated tests actually run.** dnglab 0.7.2 and exiftool are on PATH.
- Run `cargo test --test dng_writer -- --nocapture`.
- Confirm no SKIP line appears and the dnglab assertions execute.
- **Recompute one strip checksum and one tile checksum independently.** Write the
  fixture to disk, run `dnglab analyze --raw-checksum` by hand, and compare it with
  an MD5 of the input samples as LE `u16` that you compute outside the test code,
  for example with a few lines of Python run with `-I`.

**The oracle is not tautological.** Check that the expected MD5 is computed from
the *input samples*, not from bytes the writer produced or irradiance read back. A
test that hashes the writer's own output proves nothing.

**The red-proof is real.** For each `WriterFault`:
- confirm the honest control passes first (DEC-009);
- confirm the fault turns the dnglab checksum red;
- for strips, confirm it also turns the irradiance round-trip red.

Then mutate the writer yourself outside the knob and confirm the tests go red. For
example: flip the sub-byte packing to LSB-first, write tile offsets wrong, or drop
the edge-tile padding.

**Tiles are genuinely tiled.** Confirm the tiled fixture has partial right **and**
bottom tiles, that padding uses the `edge_fill` sentinel, and that both II and MM
are exercised.

**Rows that aren't a whole number of bytes are refused**, with the typed error, for
both strips and tiles. `src/plane.rs` must not have been touched to work around them.

**Placement matches the reader.** With `PreviewIfd0WithSensorSubIfd`:
- `sensor_candidates()` has exactly one entry, the SubIFD;
- colour and identity tags are read from IFD0;
- plane, CFA and level tags are read from the SubIFD.

Check this against `Container::sensor` (`src/ifd.rs`).

**Clean-room provenance.** The ledger row cites TIFF 6.0, DNG 1.7 and DEC-008
only. There must be no sign of ported writer code; spot-check the structure and
comments.

**Promotion readiness (advisory, not a gate).** Note anything that would make
SPEC-061's move into `src/` costly: panicking paths, unchecked arithmetic, or the
`fault` field.

**CI is observed green on the PR head** with `gh pr checks 198` or `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are
  fine.
- `Cargo.lock` is gitignored; never `git add` it.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.
- Open a NEW PR against `main` from `claude/spec-057-verify-<suffix>` carrying the
  handback, and any punch-list fixes. It must superset #198 and reference
  HANDOFF-128, SPEC-057, DEC-055 and PR #198.

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
