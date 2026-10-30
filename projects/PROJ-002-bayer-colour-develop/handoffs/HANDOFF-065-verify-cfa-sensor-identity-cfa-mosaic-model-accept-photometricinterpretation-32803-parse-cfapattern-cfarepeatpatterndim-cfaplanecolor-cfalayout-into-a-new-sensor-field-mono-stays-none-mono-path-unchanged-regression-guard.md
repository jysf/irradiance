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
  id: HANDOFF-065
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-29
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-024

project:
  id: PROJ-002
  stage: STAGE-006
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
  tokens_total: 7307919            # REAL combined count — what cost-audit reads
  estimated_usd: 17.25             # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: claude/spec-024-cfa-identity-24f81f
  pr: https://github.com/jysf/irradiance/pull/37
  completed_at: 2026-10-29         # YYYY-MM-DD
  notes: "✅ APPROVED. Reviewed cold against the spec; every AC reproduced independently (full suite + pinned CI clippy + fmt + validate re-run locally, both corpus Bayer DNGs + a Q2M mono file driven through `irr ifd`). One environment gotcha, not a code defect: a stale cross-branch `target/` (dated before the PR) made the first `just test` link an old rlib and fail with phantom `no field color_filter` errors; `cargo clean -p irradiance` + rebuild -> fully green, corroborated by green CI on the head. PR #37 was already MERGED to main before this verify ran (ship happened in the main session) — this verify stands as the retrospective independent review. tokens_total summed from this session's own transcript JSONL (scratchpad-UUID anchored, deduped by message.id, all 46 usage-bearing msgs claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated; estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
  synced_at: 2026-10-29
---

# HANDOFF-065: CFA sensor identity + `Cfa` mosaic model (accept PhotometricInterpretation 32803; parse CFAPattern/CFARepeatPatternDim/CFAPlaneColor/CFALayout into a new `Sensor` field; mono stays `None`; mono path unchanged) — VERIFY

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-024` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — an independent
cold review of the build (PR #37, HANDOFF-064) against the spec's Acceptance
Criteria, re-running every gate rather than trusting the build's report or a CI
badge.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-024-...-regression-guard.md`
  — the 8 Acceptance Criteria and the 7 Failing Tests are the contract.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-064-build-...md`
  — deliverables, the §11-reader deviation, and the build's honest reflection.
- **The code the change sits in:** `src/ifd.rs` (`is_sensor_ifd`, `struct Cfa`,
  `struct Sensor.color_filter`, `parse_cfa`, `cfa_bytes`, the tag consts, the
  `Build` test DSL), `src/lib.rs` (`NoSensorIfd` Display), `src/bin/irr.rs` (reader).

### Decisions that apply

- `DEC-012` — strict on structure, tolerant on shape. CFA metadata is
  interpretation-class: a malformed CFA tag costs that field (→ `None`, recorded
  in `malformed_tags`) and never fails the file. This is the rule `parse_cfa` obeys.

### Constraints that apply

- `no-panics-on-untrusted-input` (blocking) — all CFA reads delegate to the
  bounds-checked accessors (`uints`/`payload`/`array`/`scalar`) + `cost_the_field`.
- `test-before-implementation`, `one-spec-per-pr` (blocking).
- `provenance-recorded-per-algorithm`, `oracle-must-be-shown-red` — N/A (reading
  tag values is not an algorithm/decoder; SPEC-024 adds no oracle; both the plane
  oracle and the fuzz-corpus registration are SPEC-025/SPEC-027).

### Prior related work

- `HANDOFF-064` (build) · `PR #37` · `PR #36` (design + framing) · `SPEC-009`
  (the `malformed_tags`/`cost_the_field` discipline this reuses verbatim).

## Expected Deliverables

- A clear verdict (✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED), backed by gates
  re-run and evidence reproduced independently.
- The verify cost session recorded (handback above + spec `cost.sessions`).
- On approval: the ship prompt written, verify marked `[x]` in the timeline.

## Out of Scope

- Reading the mosaic bytes / bit-exact plane oracle — SPEC-025.
- Colour matrices / white balance / per-channel levels — SPEC-026.
- Corpus-manifest + fuzz-corpus registration of the Bayer DNG — SPEC-027.
- The ship ritual (reflection, cost totals, archive, merge) — the maintainer's
  main-session step (and PR #37 is already merged).

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-024-cfa-identity-24f81f` → https://github.com/jysf/irradiance/pull/37 (**already MERGED** to `main` before this verify ran)
- **Head reviewed:** `2bef1d6ba73e97a858845d7987aeff7c919b6d2f` (branch tip; feat `070ee11` + docs backfill `2bef1d6`)
- **Completed at:** 2026-10-29
- **All acceptance criteria met?** Yes — every AC reproduced independently.
- **Verdict: ✅ APPROVED** at commit `2bef1d6`. No blocking findings.

#### What I reproduced (reconciled against my own run, not the build's word or a CI badge)

- **Gate: `just test`** — after `cargo clean -p irradiance` (see the env note),
  14 test binaries, **0 failed** in every one; the 7 CFA tests all `... ok`; the
  Q2M mono develop/metadata/warp oracles green (the regression guard). Corpus
  7/7 present, no tier-B skip.
- **Gate: pinned CI clippy** (`just lint-ci`, `~/.cargo/bin/cargo +stable clippy
  --all-targets --all-features -- -D warnings`, clippy 0.1.98) → exit 0, clean.
- **Gate: `cargo fmt --check`** → exit 0. **Gate: `just validate`** → exit 0
  (31 artifacts valid front-matter, 139 parseable).
- **AC1/AC2 (corpus, `irr ifd`):** Canon 5D III `5G4A9394.DNG` and Nikon D300S
  `_DSC4615.DNG` → `photometric 32803`, `samples 1`, selected as the one sensor
  plane, `color_filter == Some(Cfa { repeat_dim: [2,2], pattern: [0,1,1,2],
  plane_color: [0,1,2], layout: 1 })`. Byte-identical to the synthetic AC2 value.
- **AC3 (mono regression):** Q2M `L1021223.DNG` → `photometric 34892`,
  `color_filter == None`; mono path is byte-for-byte unchanged (the only Sensor
  delta is the additive `color_filter: None`; `parse_cfa` is gated on
  `photometric == PHOTOMETRIC_CFA`, so a LinearRaw plane reads no CFA tag).
- **Gate `is_sensor_ifd`:** the `SamplesPerPixel == 1` and `NewSubfileType == 0`
  checks are verbatim-unchanged; the only edit is the photometric disjunct
  (`is_sensor_ifd` src/ifd.rs:1142-1145, samples check still at :1151). AC6
  (`samples != 1` → `SensorMatch::No`) holds.
- **Code-review:** the four CFA tag numbers + `PHOTOMETRIC_CFA` are correct
  (TIFF/EP + DNG: 33421 / 33422 / 50710 / 50711 / 32803); `parse_cfa` mirrors the
  `black_level`/`active_area` reads exactly (`cost_the_field` + `array`/`scalar`),
  so malformed CFA costs its field (AC4) and absent CFA is not malformed (AC5);
  the test DSL's `bytes_word`/`two_shorts_word` packing is genuinely byte-order
  agnostic — the physical value-field bytes are identical in II and MM — so AC7/AC8
  are real, not a coincidence of one endianness. The photometric read was refactored
  to a single `required_scalar` reused by the struct field and the CFA gate; this is
  behaviorally inert because the scan that selected the IFD already read that tag
  cleanly (equivalent-mutant, as the build's comment argues).

### Cost self-report

- **Tokens (total):** 7,307,919
- **Estimated USD:** 17.25
- **Duration (minutes):** ~35
- **Source of the number:** harness report — summed from this verify session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read` over
  all 46 usage-bearing records, deduped by `message.id`, scratchpad-UUID anchored;
  all records `claude-opus-4-8`). Cache-read-dominated (7.11M of 7.31M is cache
  read). `estimated_usd` at per-category Opus list rates (in 15 / out 75 /
  cache-write 18.75 / cache-read 1.50 per Mtok). Floor-by-id snapshot, not a budget
  delta; matches the all-inclusive convention prior verify cycles used.

### Drift and new artifacts

- **New decisions emitted:** none. The one-`Sensor`-type-with-`Option<Cfa>` choice
  was sanctioned by the spec's Notes as a design-cycle call; DEC-029 stays reserved
  for the crustyimg LUT boundary. `just decisions-audit` is clean of new drift.
- **Deviations from spec:** one, already disclosed and sound — `src/bin/irr.rs`
  gains one `println!` surfacing `color_filter` (AGENTS.md §11 "ship the reader
  with the field"; no library behaviour change). Within the §11 sanctioned form.
- **Non-blocking observations (not punch-list; recorded for SPEC-025 onward):**
  1. **PR #37 already merged** before verify — the ship happened in the main
     session; this verify is retrospective. Orchestrator's call, not a defect.
  2. **AC7 "panic-free on truncated/garbage CFA payload"** has no CFA-specific
     unit test (the malformed test exercises a wrong field *type*, caught in
     `cfa_bytes` before `payload`). The guarantee rests on the inherited
     bounds-checked `payload`/`uints` + the `--lib` deny-lint gate, and SPEC-027
     extends the fuzz corpus to CFA input. Sound as shipped; the contract was 7
     named tests and all 7 are present.
  3. **`parse_cfa` falls `repeat_dim` back to `[0, 0]`** when
     `CFARepeatPatternDim` is absent/malformed while `CFAPattern` reads. Honest
     sentinel (the spec gives defaults only for `plane_color`/`layout`), and every
     corpus file carries the dim — but SPEC-025's plane reader should treat
     `[0, 0]` as "undescribed", not trust it.
- **Follow-up work identified:** none ship-blocking. SPEC-025/026/027 already framed.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec. The one real cost was environmental: a stale cross-branch
   `target/` (dated the day before the PR) linked an old `irradiance` rlib into the
   downstream bin/test crates, so the first two `just test` runs failed with phantom
   `no field color_filter` errors while the lib itself compiled fine — the tell that
   it was an incremental-cache artifact, not a code defect. A per-package `cargo
   clean` resolved it; green CI on the head confirmed the code was always sound.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-012 is the governing decision and the build obeys it precisely. Worth
   noting for the record: the recurring guidance here (shared/cross-branch `target/`
   staleness in `.claude/worktrees`) bit the verify, not the build — a verifier on a
   fresh worktree should expect to `cargo clean -p` before trusting a first failure.

3. **If you did this task again, what would you do differently?**
   — Pipe gate output straight to the tool (no `; echo "EXIT=$?"` tail, no `| tee`):
   both masked `just test`'s real non-zero exit behind a trailing command's 0 and
   nearly let a (spurious, in this case) failure read as green. Trust the
   `test result:` lines and the recipe's own exit, captured directly.
