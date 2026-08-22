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
  id: HANDOFF-019
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5    # a DISPATCH HINT is not a measurement (SPEC-007/FU-6);
                    # whoever runs this cycle sets it to what ACTUALLY ran
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-008

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
  status: completed                     # completed | blocked | rejected
  tokens_total: 25000000               # REAL combined count — what cost-audit reads
  estimated_usd: 10.20              # tokens_total × your rate, or your harness's number
  duration_minutes: 60
  branch: feat/spec-008-pin-structure-class
  pr: null
  completed_at: 2026-08-21               # YYYY-MM-DD
  notes: "tokens_total rounded up from a measured 24,241,777 floor (deduped by message.id); see spec cost.sessions[0].notes and this handoff's Cost self-report for the full transcript-sourced breakdown. Did NOT run handback-sync per the return criteria."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-019: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-008` for the **build** cycle.

`DEC-012`'s Structure class is **stated but not enforced**. Four of its five tags
can be softened to tolerant with **nothing failing**. This spec makes the decision
real.

⚠ `to_agent` is deliberately `null`. `tier_map` was 0-for-2 as a prediction
(`SPEC-007/FU-6`); set it to what actually ran, in the handback.

## Context the Receiving Agent Needs

### The measurement that motivates this spec

Softening each structural tag, running the **full 58-test suite with the corpus
present** — measured by SPEC-007's reviewer, `Compression` reproduced independently
by the orchestrator:

| structural tag → tolerant | full suite |
|---|---|
| `RowsPerStrip` | **RED** |
| `Compression` | all green |
| `StripOffsets` | all green |
| `StripByteCounts` | all green |
| `BitsPerSample` | all green |

**`Compression` is the dangerous one.** Softened it defaults to `1`,
`require_uncompressed()` passes, and **STAGE-002 reads JPEG bytes as raw samples** —
a wrong image from a file that parsed cleanly.

The orchestrator had mutated `RowsPerStrip` alone and reported *"the boundary test
has teeth."* One point on a boundary is not a boundary — that is
`measurement-over-generalised`, now at N=3.

### The pattern already exists; copy it four times

`malformed_structural_tag_is_still_fatal` (`src/ifd.rs:1716`) plants an invalid
field type and asserts `sensor()` errors. It works for `RowsPerStrip` because it is
the only tag it is written for. Accessors measured at design — `BitsPerSample` via
`required_scalar()` (1171), `Compression` via `scalar()?` (1178), `StripOffsets`
and `StripByteCounts` via `values()` (1186/1187) — all reach `uints()` and all
propagate with `?`, so the same shape should reach them. **Verify that; if one does
not error, that is a finding about the code, not a licence to weaken the test.**

⚠ **`SamplesPerPixel` and `Photometric` are equivalent mutants** — re-reads of tags
`is_sensor_ifd` already validated. Unkillable by construction. Do **not**
manufacture a test that appears to cover them; leave a comment saying why.

### FU-4: one global line

`uints()` at **`src/ifd.rs:800`** accepts `TYPE_RATIONAL` in the **global** match,
so every tag reading through it accepts RATIONAL — including `SubIFDs` (330),
which `DEC-012` calls **structural**. `RATIONAL 400/2` now walks a SubIFD where
`main` returned `Err`. Make it **per-tag**, and write the reasoning down.

### FU-1/2/5: the record says something untrue

- **FU-1** — plane in `IFD0`: `Orientation` is costed twice,
  `malformed_tags = [274, 274]`. The Pentax `.PEF` is `sensor_ifd #0` — a **corpus
  shape**, not hypothetical.
- **FU-2** — a *well-formed* `Orientation` on the sensor IFD is recorded as
  malformed anyway.
- **FU-5** — every well-formed RATIONAL fixture uses denominator `1`, so a mutant
  pushing the numerator and ignoring the quotient passes all 58 tests.

## Expected Deliverables

1. A failing-when-softened test for `Compression`, `StripOffsets`,
   `StripByteCounts`, `BitsPerSample`.
2. **Proof each one kills its mutant** — soften the tag, show the suite red,
   restore, show it green. Paste both directions per tag. A test that exists is not
   a test that bites.
3. `uints()`'s RATIONAL acceptance made **per-tag**; `SubIFDs` rejects it again.
4. FU-1, FU-2, FU-5 corrected.
5. Ten gates green.

## Out of Scope

- Redrawing `DEC-012`'s Structure/Interpretation line. Enforce it; do not move it.
- New tolerance anywhere.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates and **both directions for each of the four mutants**.
2. ⚠ Confirm each named test exists (`--list`) and **sum across targets**.
3. ⚠ **Assert every mutation compiled and applied** before concluding from it —
   that has failed five times on this project, twice in one cycle.
4. Fill `## Completion` and `handback:`; set `to_agent` to the model that
   **actually ran**; capture `tokens_total` deduplicated by `message.id`
   **before the session closes**.
5. `handoff.status: completed`; spec `task.cycle: verify`.
6. Branch `feat/spec-008-pin-structure-class` off `main`; commit; do not merge.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `feat/spec-008-pin-structure-class` (local commit; not pushed, no PR opened per return criteria — "commit; do not merge")
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** yes.
  1. Four new hand-built fixtures (`structural_compression_bad_type_is_fatal`, `structural_strip_offsets_bad_type_is_fatal`, `structural_strip_byte_counts_bad_type_is_fatal`, `structural_bits_per_sample_bad_type_is_fatal`) pin `Compression`/`StripOffsets`/`StripByteCounts`/`BitsPerSample` as fatal-when-malformed, alongside the pre-existing `RowsPerStrip` one — same template (`entries.retain` to drop the well-formed tag, push field-type `250`, assert `sensor()` returns `Err(UnexpectedFieldType)`). `SamplesPerPixel`/`Photometric` deliberately left uncovered, with the equivalent-mutant reasoning written as a comment on their construction in `sensor()`, exactly as instructed.
  2. `uints()`'s `TYPE_RATIONAL` acceptance made per-tag: a new free fn `is_structural_tag(tag)` restates `DEC-012`'s amended Structure row as a predicate (the 11 tags: `NewSubfileType`, `ImageWidth`, `ImageLength`, `BitsPerSample`, `Compression`, `Photometric`, `StripOffsets`, `SamplesPerPixel`, `RowsPerStrip`, `StripByteCounts`, `SubIFDs`); `uints()` now rejects RATIONAL for any of them ahead of the general type-gate match, restoring `main`'s behaviour, while interpretation tags keep `SPEC-007`'s widening. New test `subifds_rational_is_rejected` reproduces the exact `RATIONAL 400/2` shape the handoff's Context measured and asserts `Container::parse` itself errors (the walk fails, not merely `sensor()`).
  3. `FU-1`/`FU-2` fixed together, same root cause: `sensor()`'s `Orientation` read rewritten so `IFD0`'s and the sensor IFD's reads are each computed at most once (the sensor-IFD read is skipped entirely — not merely redundant — when `ifd_index == 0`, i.e. the plane IS `IFD0`), and `malformed` is pushed **at most once**, only when the final `orientation` is `None` **and** at least one of the two reads actually errored (not merely absent). New tests `orientation_costed_once_when_plane_is_ifd0` and `wellformed_orientation_is_not_recorded_malformed`.
  4. `FU-5` fixed with a fixture only, no code change (the division was already correct — `checked_div`/`checked_rem`, no raw `/`): `rational_denominator_is_actually_divided` uses `16736/2` (denominator ≠ 1, unlike every prior RATIONAL fixture) and asserts the quotient `8368`, not the raw numerator.
  5. Ten gates green — see Cost self-report below is not the place; see the spec's `cost.sessions[0].notes` for the full gate-by-gate transcript. Summary: `cargo build --release`; `cargo test --all-features` 66 passed (58 prior + 8 new, corpus 7/7 present, summed across five targets); `cargo clippy --all-targets --all-features -D warnings` + `cargo fmt --check`; `cargo check --all-targets --all-features`; `cargo deny check licenses` + `cargo deny --manifest-path fuzz/Cargo.toml check licenses` (both green); `scripts/lint-red-proof.sh` exit 0; `cargo clippy --lib --quiet -F x5` exit 0; `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` (msrv); fuzz 12,971,280 runs in 61s, zero crashes.
  6. **Mutant-kill proof, both directions, all six mutants** (the required four plus two bonus ones on FU-4 and FU-5, since they were cheap and directly on point): `Compression`, `StripOffsets`, `StripByteCounts`, `BitsPerSample` each individually softened (via `.ok().flatten().unwrap_or(default)` or `.unwrap_or_default()`), each turned its own new test RED, each reverted and confirmed GREEN, `git diff` checked clean before trusting each green. `SubIFDs`' `is_structural_tag` guard block deleted entirely (reverting to `SPEC-007`'s exact global-widening code) turned `subifds_rational_is_rejected` RED, restored GREEN. The whole `Orientation` fix swapped back to the literal pre-fix two-`cost_the_field` version turned **both** `orientation_costed_once_when_plane_is_ifd0` (assertion `left: [274, 274]` `right: [274]` — the exact bug the handoff described, reproduced byte-for-byte) and `wellformed_orientation_is_not_recorded_malformed` RED simultaneously; restored, both GREEN. The RATIONAL division's success arm changed from `out.push(value)` to `out.push(numerator)` turned `rational_denominator_is_actually_divided` RED (`left: {width: 16736, ...}` `right: {width: 8368, ...}`), restored GREEN.
- **For `verify`:** N/A — this is the build handback.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 25,000,000 (rounded up from a measured floor) — a transcript sum, **deduplicated by `message.id`**, from this session's own transcript at `~/.claude/projects/<path-slug>/8e88da11-7b71-4569-b4da-609dfd4d432a.jsonl` (session id read off the scratchpad path the harness gave me, the same method `SPEC-007`'s build/verify used). Measured shortly before writing this note: 235 `usage` objects across 124 distinct message ids; deduped total (input + output + cache-read + cache-write) 24,241,777. 98.6% cache-read; all cache-creation on the 1-hour ephemeral tier (0 in the 5-minute tier, read from the nested `cache_creation` object, not assumed). Rounded up to 25,000,000 to cover the remaining turns spent finishing this handback and committing — **this is a floor**, captured before the session closes, per the handoff's explicit instruction.
- **Estimated USD:** 10.20 — computed at published Sonnet rates ($3/M input, $15/M output, $6/M cache-write-1h, $0.30/M cache-read) on the measured 24.24M figure ($9.88), rounded to match the rounded token total. Not a harness-reported figure; flagged so it isn't mistaken for a metered one.
- **Duration (minutes):** ~60 (session start to this handback, by wall clock; not separately instrumented).
- **Source of the number:** none available via `/cost` from inside a turn — **transcript `usage` objects**, read directly, same methodology as `SPEC-004`'s verify and `SPEC-007`'s build/verify.

### Drift and new artifacts

- **New decisions emitted:**
  - None. Every change enforces `DEC-012`'s existing table rather than redrawing it — `is_structural_tag` is a direct restatement of the amended Structure row, and the `Orientation` fix is a control-flow correction of the existing IFD0-then-sensor-IFD fallback the amendment already prescribed, not a new rule.
- **Deviations from spec:**
  - None substantive. The mutant-kill proof covers six mutants instead of the four the return criteria literally names (added `SubIFDs`' RATIONAL gate and the RATIONAL-division success arm) — strictly additional rigor, not a scope change, and cheap given the fixtures already existed.
- **Follow-up work identified:**
  - None new. `RowsPerStrip`'s and now `Compression`/`StripOffsets`/`StripByteCounts`/`BitsPerSample`'s structural classification remain unverified by real data — every corpus file is still single-strip and uncompressed — which is the same accepted gap `DEC-012`'s amendment and `SPEC-007` already flagged, not a new finding.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The Orientation fix required working out, from scratch, exactly which of the four (IFD0-well-formed / IFD0-absent / IFD0-malformed-with-fallback-success / IFD0-malformed-with-fallback-failure) × (plane-is-IFD0 / plane-is-not-IFD0) cases the two existing tests (`orientation_comes_from_ifd0_when_the_plane_is_a_subifd`, `malformed_orientation_on_ifd0_keeps_the_plane`) already pinned, so the new logic wouldn't regress them — the handoff correctly named the two bugs but not the shape of a fix that satisfies both simultaneously without re-reading the same entry twice.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-012` (amended), `guidance/constraints.yaml`'s five blocking constraints, and `guidance/toolchain-brief.md`'s three `+toolchain` traps were exactly what this build needed and all were already listed.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. I'd capture the transcript token count once near the start as well as near the end (as this handoff itself half-suggests), so the floor-vs-final gap is measured rather than inferred from the two SPEC-007 sessions' own notes.
