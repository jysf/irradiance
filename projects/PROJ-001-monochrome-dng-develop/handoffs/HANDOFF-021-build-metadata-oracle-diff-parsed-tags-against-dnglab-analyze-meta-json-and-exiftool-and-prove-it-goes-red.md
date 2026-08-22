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
  id: HANDOFF-021
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5        # ⚠ CORRECTED to what ACTUALLY ran, measured from the
                                   #   build session's own transcript (215/215 message.model
                                   #   = claude-sonnet-5). tier_map.build predicted
                                   #   claude-opus-5 and was WRONG — SPEC-007/FU-6 is now
                                   #   0 for 3, not 0 for 2.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-005

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 30114705           # REAL combined count — what cost-audit reads
  estimated_usd: 13.55             # tokens_total × your rate, or your harness's number
  duration_minutes: 34
  branch: feat/spec-005-metadata-oracle
  pr: null
  completed_at: 2026-08-22         # YYYY-MM-DD
  notes: "⚠ FILLED BY THE ORCHESTRATOR, not by the build session — DEC-004 rule 1's mechanical-remainder clause. The build reported done but left this block null, did not branch, and did not commit; it asked the orchestrator to run /cost, which is a client-side command the assistant cannot execute AND which would have measured the WRONG session. The number below is the build's own, recovered from its own transcript (1148ce23-9e13-4f4b-bc12-15b519c8ae76.jsonl) — the method SPEC-004/FU-18 established, and the reason its premise 'no source exists' was wrong then and is wrong now. DEDUPED BY message.id and I say so: 215 usage objects, 106 distinct ids, raw 59,191,677 vs deduped 30,114,705 = 1.97x, 98.2% cache-read. Components: input 212, output 167,728, cache-write 359,304, cache-read 29,587,461. estimated_usd computed PER-COMPONENT at published SONNET rates ($3/$15/$6/$0.30 per M) because the session ran on claude-sonnet-5 — NOT harness-reported. Two comparisons worth recording: at Opus rates (what tier_map.build predicted) the same session computes $67.74, 5.0x high; at the repo's flat rate_per_mtok 6.60 it computes $198.76, 14.7x high. The orchestrator nearly booked the Opus figure before checking message.model — which is precisely what [[tier-map-predicts-what-it-should-record]] exists to catch, now 0 for 3. Committed by the orchestrator at 418be15 on feat/spec-005-metadata-oracle; reports/daily/2026-08-21.md deliberately left untracked (unrelated generated output, one-spec-per-pr). FOUR reconciliation findings carried into HANDOFF-022 as required checks."
  synced_at: 2026-08-22
---

# HANDOFF-021: Metadata oracle: diff parsed tags against `dnglab analyze --meta --json` and `exiftool`, and prove it goes red

## Delegation Summary

Build `SPEC-005` — the last spec in `STAGE-001`'s backlog and the stage's own
success criterion. Replace `tests/ifd_reader.rs`'s **hand-transcribed** table of
expected tag values with a **live** oracle that runs `exiftool` and `dnglab` as
tools and diffs their output against `Sensor` field-by-field, and ship the
red-proof that makes the diff worth trusting.

**Everything is under `tests/`. Nothing in this spec may touch `src/`.** If you
believe a `src/` change is needed, that is a finding to hand back, not a change
to make.

**Read `SPEC-005`'s `## Implementation Context` before writing a line.** It is
not background — it is the design-time probe, run against all seven real corpus
files on 2026-08-21, and it contains four measured traps that will each cost you
a loop if rediscovered:
1. `dnglab` writes an ANSI warning to **stderr** on `K3III.DNG`; `2>&1` makes
   the JSON unparseable at byte 1.
2. `dnglab`'s `cropArea.p` is **sensor-absolute**; ours (and exiftool's) is
   DNG-relative. They differ by the `ActiveArea` origin.
3. `dnglab` reports black/white/crop/active for `K3III.PEF` that exist in **no
   tag in that file** — rawler's camera database, not the file.
4. `exiftool` exits **0** on a truncated file and on an absent tag. The exit
   code carries no signal; only the values do.

## Context the Receiving Agent Needs

**Read, in this order:**
1. `SPEC-005` — all of it, `## Implementation Context` twice.
2. `AGENTS.md` §5 (measured toolchain), §6 (commands), §12 (testing bars), §15
   (*During build*, and *Where an unresolved follow-up goes*).
3. `guidance/constraints.yaml` — the five blocking ones apply here; `oracle-must-be-shown-red`
   is the one this spec exists to satisfy.
4. `guidance/toolchain-brief.md` — **three** separate `+toolchain` traps, each
   with a different fix.
5. `decisions/DEC-003` (why the corpus is never committed and CI cannot run
   tier B), `DEC-004` (levels are verified analytically — the scope boundary),
   `DEC-012` (what a malformed tag costs; `K3III.DNG` exercises it).
6. `docs/oracle-contract.md` — the three layers.
7. `tests/support/corpus.rs` (`Manifest`, `CorpusFile::require` — the skip
   idiom) and `tests/ifd_reader.rs` (the table you are replacing).

**Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does **not** exist on this host. Seven files, all tier B,
none committed. Never hardcode a path; go through `SPEC-002`'s reader.

**Tools:** `exiftool 13.55`, `dnglab 0.7.2`, both already installed and both
confirmed by `--version` during the probe. `dnglab` is **LGPL-2.1 and is RUN,
never linked** — never add `rawler`/`rawloader` or any RAW crate, including as a
dev-dependency, and **do not read dnglab's source** to resolve a disagreement.
That is a `provenance-recorded-per-algorithm` violation. Re-measure the file's
bytes instead.

**Where the tools disagree with us, our reader is not automatically wrong.**
Three cases are already measured and written into the spec as assertions. If you
find a **fourth**, record it and hand it back — do not "fix" `src/` to match a
tool.

## Expected Deliverables

Per `SPEC-005`'s nine acceptance criteria. In short:

- `tests/support/tools.rs` — shell out, parse, return a typed `ToolReading`.
  ⚠ `tests/support/corpus.rs` already exports `pub struct Oracle`; do not reuse
  that name.
- `tests/metadata_oracle.rs` — the nine named failing tests.
- `tests/oracle-fixtures/` — committed sample tool output so the **comparator's
  red-proof runs in CI** with no tool and no corpus. This is the only half of
  the oracle CI can see, which makes it the load-bearing half.
- `tests/ifd_reader.rs` — transcribed **tag-value** columns deleted; the
  structure columns (`big_endian`, `ifds`, `sensor_index`, `opcode_lists`,
  `malformed`) **kept**, because no external tool reports them.
- `just oracle-meta` recipe **plus its line in `AGENTS.md` §6** — that
  correspondence is `SPEC-001` acceptance criterion 8.
- `docs/oracle-contract.md` gains the measured Metadata-layer section;
  `docs/conformance-matrix.md` and `CHANGELOG.md` get their rows.
- **No new dependency.** `Cargo.toml` byte-identical. If you conclude otherwise,
  **stop and ask** — do not add one and write its DEC.

**The red-proof is the deliverable, not a test among tests.** Both directions,
both with a negative control, and **you must watch each one fail yourself**.
Before drawing any conclusion from a mutation, **assert the mutation changed the
file** — this repo has concluded from a mutation that never applied five
separate times.

## Out of Scope

- Any change under `src/`.
- Levels *correctness* (`DEC-004`), the plane layer, the develop layer.
- `serde_json` or any other crate — the probe showed it is unnecessary.
- Making the tier-B half run in CI. It cannot (`DEC-003`). State the limit; do
  not paper over it.
- Reading dnglab's or exiftool's source.

## Return Criteria — how to hand back

1. **Ten gates green, run by you and pasted**: `cargo fmt --check`;
   `cargo clippy --all-targets --all-features -- -D warnings`;
   `cargo test --all-features` (**sum across all five targets** — a zero-match
   `cargo test <name>` exits 0, `named-tests-can-pass-vacuously`);
   `just msrv`; `just deny`; `just deny-fuzz`; `just lint-red-proof`;
   `just lint-no-allow`; `just cost-audit`; `just decisions-index --check`.
   Plus `just validate`, `just decisions-audit --changed`, and `just oracle-meta`.
2. **Confirm each of the nine named tests EXISTS** via per-target `-- --list`
   before trusting any green, and say how you confirmed it.
3. **Fuzz re-run** — `tests/` gained a lane. Seeds unchanged is a fine result;
   say so explicitly. `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd`.
4. **Both red-proof directions pasted**, each with its negative control, each
   with the mutation asserted applied, and the tree restored byte-identical
   afterwards (`git status` clean).
5. Fill the `handback:` block **including a real `tokens_total`** —
   **deduplicated by `message.id`**, and say that you deduped. The raw-to-deduped
   factor has ranged 1.61×–2.51× over eight observations; no fixed correction is
   valid. Compute `estimated_usd` per-component at published rates, not at the
   repo's flat `rate_per_mtok`, which runs ~2.7× high on these cache-heavy
   sessions — and flag it as computed, not harness-reported.
   ⚠ **Correct `handoff.to_agent` to what actually ran** (`SPEC-007/FU-6`).
6. **Do NOT run `just handback-sync`** — the orchestrator runs it.
7. Report deviations explicitly. A disclosed deviation is fine; an undisclosed
   one is the defect.
8. Label every finding `SB-N` / `FU-N`, numbered for **this** spec starting at 1.
   They will be dispositioned at ship into `fixed` / a spec / a signal / an
   explicit close (AGENTS.md §15, *Where an unresolved follow-up goes*), so a
   finding with no owner is not an acceptable outcome — say which of the four
   you think each one wants.

## Handback

*(Filled by the implementer. Mirror the `handback:` front-matter above, and
answer the reflection questions in `AGENTS.md` §15 "When done".)*
