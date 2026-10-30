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
  id: HANDOFF-064
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  tokens_total: 48543952           # REAL combined count — what cost-audit reads
  estimated_usd: 105.56            # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/spec-024-cfa-identity-24f81f
  pr: null
  completed_at: 2026-10-29         # YYYY-MM-DD
  notes: "all 8 ACs met; +1 file beyond the spec list (src/bin/irr.rs gains one println surfacing color_filter, AGENTS.md §11 reader); tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at per-category Opus list rates (in 15 / out 75 / cache-write 18.75 / cache-read 1.50 per Mtok), cache-read-dominated"
  synced_at: 2026-10-29
---

# HANDOFF-064: CFA sensor identity + `Cfa` mosaic model (accept PhotometricInterpretation 32803; parse CFAPattern/CFARepeatPatternDim/CFAPlaneColor/CFALayout into a new `Sensor` field; mono stays `None`; mono path unchanged)

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-024` to
`claude-opus-4-8` (acting as implementer) for the **build** cycle — open the
sensor gate to Bayer-CFA (`PhotometricInterpretation == 32803`) and expose a
structured `Cfa` mosaic model on `Sensor`, with the monochrome `LinearRaw`
path byte-for-byte unchanged (regression guard).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-cfa-sensor-model-and-bit-exact-mosaic-plane.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-024-cfa-sensor-identity-cfa-mosaic-model-accept-photometricinterpretation-32803-parse-cfapattern-cfarepeatpatterndim-cfaplanecolor-cfalayout-into-a-new-sensor-field-mono-stays-none-mono-path-unchanged-regression-guard.md`
  — read it in full; the Acceptance Criteria and the 7 Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps,
  `just lint-ci` for CI's clippy, the five-lint panic policy, the corpus env var.
  Read it; it exists so a cold agent doesn't rediscover them (DEC-004 rule 5).
- **The code the change sits in:** `src/ifd.rs` — `is_sensor_ifd` (:1035, the
  photometric disjunct), `struct Sensor` (:508, where `color_filter` is added),
  `sensor()` (:1156, where it is built, mirroring the `black_level`/`active_area`
  reads at :1201–1245 exactly), the tag-const block (:91–138), and the `Build`
  test DSL + `sensor(&b)` / `header_reads_both_byte_orders` (:1344 / :1409 / :1430).
  `src/lib.rs:515` — the `NoSensorIfd` Display text.

### Decisions that apply

- `DEC-012` — strict on structure, tolerant on shape. CFA metadata is
  **interpretation** class: a malformed CFA tag costs that field (→ `None`) and
  is recorded in `malformed_tags`; it never fails the file. The bit-exact plane
  (SPEC-025) must read regardless of CFA-metadata health. This is the rule the
  whole `parse_cfa` shape obeys.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply to the paths
this task touches (`src/ifd.rs`, `src/lib.rs`, `src/bin/irr.rs`):

- `no-panics-on-untrusted-input` (blocking) — `src/ifd.rs`/`src/lib.rs` are
  library paths. All CFA reads go through the existing bounds-checked accessors
  (`uints`/`payload`/`array`/`scalar`); no `unwrap`/`expect`/indexing/`panic`/
  overflowing arithmetic. `just lint-no-allow` and the red-proof gate stay green.
- `test-before-implementation` (blocking) — the 7 Failing Tests are added first.
- `one-spec-per-pr` (blocking) — the PR references SPEC-024 only.
- `provenance-recorded-per-algorithm` — **N/A**: reading DNG tag values is not an
  algorithm/decoder (same as `black_level`/`active_area`, which carry no ledger
  row). The demosaic algorithm is SPEC-025 onward.
- `oracle-must-be-shown-red` — **N/A**: SPEC-024 adds no oracle or gate (the
  bit-exact mosaic plane oracle is SPEC-025). The 7 tests are behaviour tests.

### Prior related work

- `HANDOFF-024` / `SPEC-009` — the `malformed_tags` / `cost_the_field` discipline
  this reuses verbatim.
- `PR #36` (origin/main @ 173a426) — the SPEC-024 design + PROJ-002 framing this
  builds on.

## Expected Deliverables

- Code implementing SPEC-024's Acceptance Criteria, in `src/ifd.rs` + `src/lib.rs`:
  - `PHOTOMETRIC_CFA = 32803` and the four CFA tag consts
    (`TAG_CFA_REPEAT_PATTERN_DIM 33421`, `TAG_CFA_PATTERN 33422`,
    `TAG_CFA_PLANE_COLOR 50710`, `TAG_CFA_LAYOUT 50711`).
  - `pub struct Cfa { repeat_dim: [u32; 2], pattern: Vec<u8>, plane_color: Vec<u8>, layout: u32 }`
    (`#[derive(Debug, Clone, PartialEq, Eq)]`) and `Sensor.color_filter: Option<Cfa>`.
  - `is_sensor_ifd` accepts `photometric ∈ { LinearRaw, CFA }`; `SamplesPerPixel == 1`
    and `NewSubfileType == 0` unchanged.
  - `sensor()` builds `color_filter` via a private `parse_cfa(ifd, &mut malformed)`,
    gated on `photometric == PHOTOMETRIC_CFA`, interpretation-class.
  - `NoSensorIfd` Display names CFA beside LinearRaw.
- **§11 reader (cheap surfacing):** one line in `src/bin/irr.rs`'s `ifd` dump so
  `color_filter` is not a dead field. (Minor deviation from the spec's literal
  file list — see Deviations; §11 is blocking, so the field ships with a reader.)
- The 7 new unit tests in `src/ifd.rs mod tests` passing, and the Q2M mono
  tests/oracles staying green (the regression guard).
- All gates green locally — `just test`, `just lint-ci` (CI's clippy, `-D warnings`),
  `just fmt-check`, `just validate` — and CI observed green on the shipping SHA.
- A PR against `main` referencing PROJ-002 / STAGE-006 / SPEC-024 / HANDOFF-064 /
  DEC-012, and this handoff.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff. (Full rationale in the spec's
`## Non-Goals`.)

- Reading the mosaic bytes / the bit-exact plane oracle — **SPEC-025**.
- Parsing colour matrices / white balance / per-channel levels — **SPEC-026**.
- Registering the Bayer DNG in the corpus manifest + fuzz corpus — **SPEC-027**.
- Any demosaic, colour transform, or level application — later stages.
- X-Trans / non-Bayer CFA, native vendor containers, a typed colour enum.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-024-cfa-identity-24f81f` → (PR opened from this branch; backfilled below)
- **Completed at:** 2026-10-29
- **All acceptance criteria met?** Yes — every AC in SPEC-024.
  - **AC1** A CFA IFD (photometric 32803, 1 sample, NewSubfileType 0) is
    `SensorMatch::Yes`; `sensor_candidates().len() == 1`; `sensor()` is `Ok`
    (`a_cfa_ifd_is_selected_as_the_sensor_plane`).
  - **AC2** Synthetic RGGB → `color_filter == Some(Cfa { repeat_dim: [2,2],
    pattern: [0,1,1,2], plane_color: [0,1,2], layout: 1 })`
    (`a_cfa_plane_exposes_its_mosaic_model`); both corpus Bayer DNGs give `Some`
    with `repeat_dim == [2,2]` (manual corpus check, see below).
  - **AC3** Monochrome LinearRaw → `color_filter == None`, no other `Sensor`
    field changes; the Q2M oracles stay green
    (`a_monochrome_plane_has_no_color_filter` + the full suite).
  - **AC4** Malformed `CFAPattern` → `Ok`, `color_filter == None`,
    `TAG_CFA_PATTERN ∈ malformed_tags`
    (`a_malformed_cfa_pattern_costs_the_field_not_the_file`).
  - **AC5** Absent CFA tags on a CFA plane → nothing in `malformed_tags`,
    `color_filter == None` (`absent_cfa_tags_on_a_cfa_plane_are_not_malformed`).
  - **AC6** CFA IFD with `SamplesPerPixel != 1` → still `SensorMatch::No`
    (`a_cfa_plane_with_more_than_one_sample_is_not_selected`).
  - **AC7** Panic-free on a garbage CFA payload — every read goes through the
    bounds-checked accessors and `cost_the_field` swallows the `Err` into a
    costed `None`; the deny-lint gate proves the path carries no panic.
  - **AC8** `color_filter` is byte-order-agnostic (II and MM identical)
    (`a_cfa_mosaic_model_is_byte_order_agnostic`).

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 48,543,952
- **Estimated USD:** 105.56
- **Duration (minutes):** ~40
- **Source of the number:** harness report — summed from this session's own Claude
  Code transcript JSONL (`input + output + cache_creation + cache_read` across all
  234 usage records; this desktop Code interface exposes no `/cost` slash command).
  Matches the all-inclusive convention prior specs used (SPEC-019 build 41.3M).
  `estimated_usd` at per-category Opus list rates — an order-of-magnitude,
  cache-read-dominated figure.

### Drift and new artifacts

- **New decisions emitted:** none. The one-`Sensor`-type-with-`Option<Cfa>`
  choice is additive and was already sanctioned by the spec's Notes as a
  design-cycle call (not promoted to a DEC — it touches nothing the mono path
  depends on). DEC-029 stays reserved for the crustyimg LUT boundary.
- **Deviations from spec:**
  - One extra file beyond the spec's `src/ifd.rs` + `src/lib.rs` list:
    `src/bin/irr.rs` gains ONE `println!` line surfacing `color_filter` in the
    existing `ifd` dump. AGENTS.md §11 ("ship the reader with the field") is
    blocking and "a parsed tag with no consumer is not parsed"; the cheap reader
    is the sanctioned minimal form. No behavioural change to the library.
- **Follow-up work identified:** none ship-blocking. SPEC-025/026/027 are the
  already-framed next specs that consume `color_filter`.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The spec pinned the tag numbers, types, the `Cfa` shape,
   the exact expected values, and the 7 test names, so the build was mechanical.
   The one judgment call the spec left open — what makes `color_filter` `Some`
   vs `None` — resolves cleanly to "`CFAPattern` is the mosaic; gate on it, and
   fall the two colour-less tags back to their DNG defaults (`plane_color`
   `[0,1,2]`, `layout` 1)."

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The §11 reader. The spec's Outputs list only `src/ifd.rs`/`src/lib.rs`, but
   §11 (blocking, and the most-recurring defect) requires a runtime surfacing in
   the same change. The handoff AC should name the reader when a field is added
   (§11 says exactly this about delegation). Noted here rather than filed.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Reading `payload`/`uints`/`array` first paid off — the
   byte-order-agnostic test passed on the first run because the CFA reads reuse
   the same vetted accessors the mono tags do, rather than hand-rolling endianness.
