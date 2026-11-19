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
  id: HANDOFF-018
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-007

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
  tokens_total: 7830000            # REAL combined count — what cost-audit reads
  estimated_usd: 19.35             # tokens_total × your rate, or your harness's number
  duration_minutes: 55
  branch: feat/spec-007-extraction-tolerance
  pr: null
  completed_at: 2026-08-21         # YYYY-MM-DD
  notes: "APPROVED at 0de18d4. 7 follow-ups, 0 ship-blockers. tokens_total is a transcript sum deduplicated by message.id (108 usage objects, 64 distinct ids; raw 13,161,712, deduped 7,736,717 = 1.70x, 97.5% cache-read, all cache-writes on the 1h tier), rounded up to cover the turns after measurement. estimated_usd computed per-component at published Opus rates, NOT harness-reported. handback-sync deliberately not run."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-018: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-007` for the **verify** cycle, at
`0de18d4`. Independent session.

The spec is a direct transcription of `DEC-012`'s 2026-08-21 amendment. Read that
amendment before the spec — it is the operative text.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator

- **Ten gates green**, 58 tests (37 lib + 9 corpus + 12 ifd_reader), `main`
  untouched at `99086fb`, branch one commit ahead, tree clean.
- **All five named tests exist and pass** — confirmed with `--list` and summed
  across targets.
- **The boundary is preserved in code**: structural tags
  (`SamplesPerPixel:1173`, `Compression:1178`, `RowsPerStrip:1184`) are still bare
  `?`. Interpretation tags go through the new `cost_the_field` helper (11 sites).
- **The boundary test has teeth — mutation-tested by me.** Making `RowsPerStrip`
  tolerant (`.ok().flatten()`) turns `malformed_structural_tag_is_still_fatal`
  **red**; restoring turns it green. A change demonstrating only the *new*
  tolerance would not have proven the *old* fatality survived, so this was the
  property worth checking.

### What deserves scrutiny

1. **A disclosed scope extension.** The design table enumerated **7** call sites;
   the build applied the tolerance to **three more** array-tags —
   `DefaultCropOrigin`, `DefaultCropSize`, `BlackLevelRepeatDim` — arguing the
   amendment's own classification names them. **I think that is correct** (the
   amendment's Interpretation row lists all three explicitly, and my table was the
   narrower artefact). Confirm, and check nothing *structural* was swept in.
2. **`cost_the_field` is a new abstraction on a panic-free path.** Does it
   preserve the leaf/composite distinction the amendment requires — leaves still
   returning `Err` honestly, only the composite swallowing? And does it record
   **every** costed tag, or can one be dropped silently?
3. **`TYPE_RATIONAL`.** Zero denominator and non-integral ratios must cost the
   field, not fail the file. Check `checked_div`/`checked_rem` handle the
   denominator-zero case *before* the division, and that a legitimate integral
   RATIONAL is actually **read**, not merely tolerated.
4. **`SPEC-004/FU-20`** — `is_sensor_ifd` now short-circuits per identifying tag.
   Verify a readable *disqualifying* tag returns `No` before a later malformed tag
   can name a non-candidate, and that this did not change which IFD is selected on
   the 7 real files.
5. **The build ran on Sonnet 5, not the `tier_map.build`-predicted Opus 5**, and
   said so rather than letting it pass silently. That is the **second** tier_map
   mismatch, now in the opposite direction from the first. Worth a view: should
   the map stop predicting and record actuals only?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the both-directions fixtures yourself (check #9), and **mutation-test at least
one structural tag** — a tolerance change that does not break the fatality test
would mean the boundary is unguarded.

⚠ Traps, all of which have produced wrong answers on this project: `cargo test
<name>` matching zero tests **exits 0** — confirm names with `--list`; **sum
across targets** — reading one target's line has misled in both directions; and
**assert your mutation compiled and applied** before concluding from it (that has
now failed five times here, including twice in this cycle).

**Label every finding `SB-N` / `FU-N`** per AGENTS.md §15, numbered for this spec.
If APPROVED, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Re-litigating `DEC-012`'s amended line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates, the both-directions fixtures, and your structural
   mutation test.
2. Fill `## Completion` and `handback:`. `tokens_total` deduplicated by
   `message.id`, and **captured before the session closes** — `SPEC-004`'s build
   left it null and had to be grandfathered out of the cost gate.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship`).
4. Commit on `feat/spec-007-extraction-tolerance`; do not merge. Do not run
   `handback-sync`.

## Handback

*Filled in by the receiving agent.*

### Execution notes

- **Branch / PR:** `feat/spec-007-extraction-tolerance` (local; not pushed, no PR — "commit; do not merge")
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** yes — all six, each checked below.
- **For `verify`:** ✅ **APPROVED at `0de18d4`.** 7 follow-ups (`FU-1` … `FU-7`), **0 ship-blockers**.

### The ten gates — run by me, this session, at `a94f5b4` on a clean tree

| # | gate | result |
|---|---|---|
| 1 | `cargo fmt --check` | clean |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| 3 | `cargo check --all-targets --all-features` | clean |
| 4 | `cargo test --all-features` | **58 passed** = 37 lib + 0 `irr` + 9 `corpus_manifest` + 12 `ifd_reader` + 0 doc, **summed across five targets**, corpus present |
| 5 | `cargo clippy --lib -F` ×5 (lint-no-allow) | exit 0 |
| 6 | `cargo deny check licenses` (library graph) | `licenses ok` |
| 7 | `cargo deny --manifest-path fuzz/Cargo.toml check licenses` | `licenses ok` |
| 8 | `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` | exit 0 |
| 9 | `./scripts/lint-red-proof.sh` | control clean → **injection rejected (101)** → all five lints fired, and still fired without `-D warnings`. **I watched it go red** (check 9) |
| 10 | `cargo build --release` | exit 0 |

Plus: `scripts/cost-audit.sh` ✓ · `scripts/decisions-index.sh --check` ✓ · `just validate` (7 artifacts) ✓ · `just decisions-audit --changed` ✓ · `main` still `99086fb`, branch +2, tree clean.

**Fuzz (check 10), re-run by me:** `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60` → **10,768,231 runs in 61 s, `cov: 620 ft: 2048`, `fuzz/artifacts/ifd/` empty.** The widened `uints()` *is* reached: `fuzz/fuzz_targets/ifd.rs:47` calls `container.uints(entry)` on **every** entry regardless of field type, and `:53` calls `container.sensor()`, so both `TYPE_RATIONAL` and `cost_the_field` are on the fuzzed path — the build's claim, checked rather than taken. Seeds regenerate byte-identical (`cargo run --example fuzz-seeds` → 26 seeds, `diff -rq` clean), so the committed set is reproducible.

### The named tests — existence confirmed before their green was trusted

`cargo test <name>` matching zero tests exits 0, so each name was first grepped out of `-- --list` across **all four** test targets. All five are in the **lib** target and nowhere else; the anchored grep is `(^|::)<name>: test$`, because lib tests list as `ifd::tests::<name>`, and a bare `^<name>` pattern returns **0 for every one of them** — that near-miss is worth recording, since it would have read as "the tests do not exist".

| test | exists | runs |
|---|---|---|
| `malformed_interpretation_tag_costs_only_the_field` | 1 | 1 passed |
| `malformed_structural_tag_is_still_fatal` | 1 | 1 passed |
| `rational_default_crop_size_reads_or_costs_the_field` | 1 | 1 passed |
| `malformed_orientation_on_ifd0_keeps_the_plane` | 1 | 1 passed |
| `candidates_malformed_names_only_candidates` | 1 | 1 passed |

---

## The five points put to scrutiny

### 1. The disclosed scope extension — correct, and nothing structural was swept in

**Confirmed, exactly.** `cost_the_field` has **8 call sites** (`src/ifd.rs:1103, 1112, 1120, 1125, 1136, 1141, 1152, 1158`) covering **7 distinct tags**: `Orientation` (twice — IFD0 and the sensor-IFD fallback), `BlackLevel`, `WhiteLevel`, `BlackLevelRepeatDim`, `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize`.

`DEC-012`'s amendment Interpretation row reads: *`BlackLevel`, `WhiteLevel`, `ActiveArea`, `DefaultCropOrigin`/`Size`, `Orientation`, `OpcodeList*`, `BlackLevelRepeatDim`*. That is a **literal 7-for-7 match** — the build did not extend the rule, it applied the rule the design table had under-enumerated. (`OpcodeList*` is the eighth name in the row and is unreachable by this class: `opcode_lists` uses `ifd.has(...)`, presence only, which cannot fail.) The orchestrator's reading was right.

**Nothing structural was swept in.** The 9 remaining bare-`?` sites in `sensor()` are `ImageWidth:1169`, `ImageLength:1170`, `BitsPerSample:1171`, `SamplesPerPixel:1173`, `Photometric:1175`, `Compression:1178`, `RowsPerStrip:1184`, `StripOffsets:1186`, `StripByteCounts:1187` — every one of them Structure-class or plane-extent, every one unchanged from `main`. The three lines the orchestrator cited (1173 / 1178 / 1184) are exactly where it said they were.

One classification worth naming out loud: **`BitsPerSample` appears in neither of the amendment's two rows**, just as `RowsPerStrip` does not. `RowsPerStrip` at least got an explicit ruling in the spec's own table; `BitsPerSample` was classified structural by inheritance, because it was already `?` and nobody had to decide. It is the right answer — bits-per-sample is the plane's *extent* in the amendment's sense — but it is a classification made by silence, and `FU-3` is where that costs something.

### 2. `cost_the_field` — leaves honest, composite swallowing, nothing dropped unrecorded

**Leaves stay honest.** `scalar()`, `values()`, `uints()` and `array()` all still return `Result` and all are unchanged in that respect; `array()`'s pre-existing count-mismatch tolerance (`:929` — push, return `Ok(None)`) is the one `DEC-012` already sanctions. `cost_the_field` is private (`fn`, no `pub`), takes no `&self`, and is called from **`sensor()` and nowhere else** — the composite is the only thing that swallows. ✅

**Can a costed tag be dropped without being recorded?** No — and I did not take the code's word for it. Mutation **M8**, deleting the `malformed.push(tag)` from the `Err` arm, compiled and turned **four** tests red at once (`malformed_interpretation_tag_costs_only_the_field`, `malformed_orientation_on_ifd0_keeps_the_plane`, `rational_default_crop_size_reads_or_costs_the_field`, `a_non_integral_rational_also_costs_the_field`). Nor is there a double-record on the `array()` path: `array()` returns `Err` *without* pushing and pushes *only* on the `Ok(None)` count-mismatch branch, so the two mechanisms cannot both fire for one read.

**But the inverse is unguarded, and it fires on a shape the corpus actually contains** — `FU-1` and `FU-2` below. The question asked was whether a tag could go unrecorded; the defect is that one can be recorded **twice**, and another recorded **while its value was in fact read**.

### 3. `TYPE_RATIONAL` — zero denominator safe, and a legal RATIONAL genuinely *read*

Zero denominator: `numerator.checked_div(denominator)` and `.checked_rem(denominator)` both return `None` for `denominator == 0`, and the match arm requires `(Some(value), Some(0))`, so the zero case exits to `Error::MalformedRationalValue` without any unchecked division ever executing. There is no raw `/` or `%` anywhere in the arm — which is also why the `-F clippy::arithmetic_side_effects` gate stays green. Strictly, the guard *is* the division rather than something before it, but it is the checked form, so the panic-free property holds. `(0, 0)` → `Err`; `(0, 5)` → `0`. ✅

**Actually read, not merely tolerated:** mutation **M12**, replacing the success arm so a well-formed RATIONAL can never be pushed, compiled and turned `rational_default_crop_size_reads_or_costs_the_field` **red**. So the well-formed path really does produce `DefaultCropSize { 8368, 5584 }` rather than quietly costing the field. ✅

The gap is narrower than the question and real: see `FU-5`. Every well-formed RATIONAL fixture in the repo uses **denominator 1**, so mutation **M11** — push `numerator` and ignore the quotient — passes all 58 tests. The reading is right; the *division* is unpinned.

`chunks_exact(8)` cannot silently drop a trailing partial value: `payload()` returns exactly `count × 8` bytes for `TYPE_RATIONAL` (`type_size(5) == 8`, `:161`), so the remainder is always empty.

### 4. `FU-20`'s short-circuit did not change IFD selection on the 7 real files

**Measured, not reasoned.** I built `irr` at `main` (`99086fb`) in a throwaway worktree and at `HEAD`, ran `irr ifd` over all seven manifest files with `IRRADIANCE_CORPUS_DIR` set, and diffed the two transcripts **in full** — every line, not just the selection: `diff` is **empty**. Byte-identical, including `sensor_matches`, `sensor_ifd`, all tag values, `malformed_tags`, the layer-0 closure and `unpackable`.

```
L1021223.DNG  matches [1] → #1     L1000622.DNG  matches [1] → #1
L1026016.DNG  matches [1] → #1     M2462362.DNG  matches [1] → #1
L1026192.DNG  matches [1] → #1     K3III.DNG     matches [1] → #1   malformed [50713]
                                   K3III.PEF     matches [0] → #0
```

The Pentax `.PEF`'s plane really is **IFD0** — which is what makes `FU-1` a shape the corpus holds rather than a hypothetical. And the K-3 III `.DNG`'s real `BlackLevelRepeatDim` defect (50713) still surfaces, unchanged.

Logically this is also airtight: the short-circuit only re-orders the evaluation of a **conjunction**, so the set of IFDs answering `Yes` is identical by construction; only the `Unreadable` set shrinks, and `sensor()` consults `unreadable` solely when `matches` is empty. Mutation **M10** (deleting the `NewSubfileType` short-circuit) compiled and turned `candidates_malformed_names_only_candidates` **and** `a_reduced_resolution_linear_raw_ifd_is_not_the_sensor` red, so the fix is pinned from both sides.

### 5. `tier_map` — it should record, not predict

The map has now been **wrong on both occasions it has been checked, in opposite directions**: it said `sonnet` when `SPEC-001`'s build ran on Opus (corrected 2026-08-18), and it says `claude-opus-5` while `SPEC-007`'s build ran on Sonnet 5. Its measured accuracy is 0 for 2. The 2026-08-18 comment at `.repo-context.yaml:44-49` even states the premise that has now failed — *"build cycles are dispatched to a CLI session here… so the map now says what actually runs"* — which was a statement about one deployment habit, written into config as though it were a rule.

**The defect is not the values, it is the wiring.** `tier_map` is a *prediction*, and the orchestrator stamps it into `handoff.to_agent`, which `handback-sync` copies into `cost.sessions[].agent` — a field consumed as a *measurement*. A prediction feeding a record is wrong by construction, and `DEC-004` rule 3 is written as though the map were the trustworthy half. In both mismatches the truth came from the executing agent noticing and overriding; twice now, the safeguard has been an alert implementer rather than the mechanism.

**Recommendation:** keep `tier_map` as a *dispatch hint* — what the orchestrator should ask for — and stop pre-filling `to_agent` from it. Leave `handoff.to_agent: null` at creation and let the **handback** be its only writer. Then a mismatch is impossible rather than caught, and the map keeps its one honest job (deciding what to dispatch) while losing the one it keeps failing (asserting what ran). Filed as `FU-6`; it is `N=2` with an identified mechanism, which is a `guidance/signals.yaml` `process-debt` entry, not a hunch.

---

## Findings — 0 ship-blockers, 7 follow-ups

Numbered for **SPEC-007**, per AGENTS.md §15. Nothing was fixed; each carries `file:line`.

### `FU-1` — a malformed `Orientation` is recorded **twice** when the plane is IFD0

`src/ifd.rs:1102-1118`. `sensor()` tries `Orientation` on `ifd0()`, costs the field on failure, then falls back to the **sensor** IFD and costs it again. When the sensor IFD *is* IFD0 — the Pentax `.PEF` shape, `sensor_ifd #0`, confirmed above — that is the same entry read twice, so both pushes fire.

Measured with a throwaway fixture (plane in IFD0, `Orientation` field type 250):

```
malformed_tags = [274, 274]
```

`malformed_tags` is a public field and `DEC-012` makes it the *visible* record of what was tolerated; a consumer taking `.len()` or rendering the list sees two defects where the file has one. Not ship-blocking — the tag is still named, no value is wrong, no plane is lost, and the one real IFD0-plane file has a well-formed `Orientation` — but it is a public return value that does not say a true thing. Cheapest honest fix: skip the fallback when `ifd0()` and `ifd` are the same IFD; or dedupe once at construction (`:1200`).

### `FU-2` — a tag is listed in `malformed_tags` even when its value **was** read

`src/ifd.rs:1110-1118`. Same site, different direction. A malformed `Orientation` on IFD0 with a **well-formed** one on the sensor IFD yields:

```
orientation = Some(6)   malformed_tags = [274]
```

The field was successfully read, so `274` in `malformed_tags` overstates: `DEC-012` documents that list as "the value is dropped, the tag is recorded", and here nothing was dropped. Defensible as "something about 274 was malformed", but that is a different claim from the one the field's doc-comment (`:514-521`) makes. Worth one line of doc or one line of code, whichever the next author prefers.

### `FU-3` — the structural half of the boundary is pinned for exactly **one** tag

`src/ifd.rs:1171, 1178, 1186, 1187`. `malformed_structural_tag_is_still_fatal` is real and has teeth — mutation **M1** (`rows_per_strip: … .ok().flatten()`) compiled and turned it, and only it, red. That reproduces the orchestrator's result independently.

But it pins `RowsPerStrip` **alone**. I mutated each remaining structural site to tolerant and ran the **full 58-test suite with the corpus present**:

| mutation | compiled | full suite |
|---|---|---|
| `RowsPerStrip` → `.ok().flatten()` | ✅ | **RED** — `malformed_structural_tag_is_still_fatal` |
| `Compression` → `.ok().flatten()` | ✅ | **all green** |
| `StripOffsets` → `.unwrap_or_default()` | ✅ | **all green** |
| `StripByteCounts` → `.unwrap_or_default()` | ✅ | **all green** |
| `BitsPerSample` → `.unwrap_or(0)` | ✅ | **all green** |

Each mutation was verified present in the file (`git diff`) and verified to compile before its result was read — the trap that has bitten this project five times, twice in this cycle.

`Compression` is the one that matters: it is named in `DEC-012`'s amendment Structure row **and** in the spec's own acceptance table (line 1027), and a future softening would silently default a malformed `Compression` to `1` — so `require_uncompressed()` passes and STAGE-002 reads JPEG bytes as raw samples. That is precisely the confident-wrong-answer failure `DEC-012` exists to prevent, and nothing in the suite would notice.

**Not a defect in the code — the code is correct.** Acceptance criterion 5 asks for the boundary pinned in both directions and one such pair exists, so the criterion is met as written. This is a thin regression net, filed rather than blocking. The fix is four more fixtures of the shape `malformed_structural_tag_is_still_fatal` already has, or one parameterised over the structural tag list.

*Not a gap, for the record:* mutating `SamplesPerPixel:1173` or `Photometric:1175` in `sensor()` also leaves the suite green, and that one is **correct** — those two are re-reads of tags `is_sensor_ifd` already read successfully for the selected IFD, so their `?` can never fire. They are equivalent mutants, unkillable by construction. Worth knowing so the next reader does not chase them.

### `FU-4` — the `RATIONAL` widening loosened the **walk**, which `DEC-012` classifies as structural

`src/ifd.rs:800`. `uints()`'s type gate is global, so adding `TYPE_RATIONAL` to it widened **every** tag, not the interpretation-class ones the spec was scoped to. Measured at `HEAD` against `main`:

| fixture | `main` (`99086fb`) | `HEAD` (`0de18d4`) |
|---|---|---|
| `SubIFDs` (330) as RATIONAL `400/2` | `Err(UnexpectedFieldType { tag: 330, field_type: 5 })` | **accepted** — `ifds=2`, `candidates=[1]`, the SubIFD walked |
| `StripByteCounts` (279) as RATIONAL `28/2` | `Err(UnexpectedFieldType { tag: 279, field_type: 5 })` | **accepted** — `strip_byte_counts = [14]` |

Tag 330 is the case `DEC-012` singled out as *"the one genuinely debatable case, and it is classified as structural"*, rejecting Option A on the grounds that a tolerant 330 yields *"a container that is structurally a lie"*. The walk now accepts a field type neither TIFF 6.0 nor DNG permits for it. The spec's Non-Goals scoped the widening per-tag — *"RATIONAL is in scope because the DNG spec permits it for tags we already read"* — and `uints()` has no per-tag hook, so that scoping could not be expressed. It was not disclosed in the build's handback.

**Follow-up, not ship-blocking, for three measured reasons.** (a) It widens a looseness that already existed rather than creating one: `uints()` accepted `TYPE_UNDEFINED` and `TYPE_BYTE` for tag 330 on `main` too, equally illegal under DNG — the type gate has never been per-tag. (b) No safety property moves: the offsets still pass every bounds, cycle and depth guard, and 10.7 M fuzz runs found nothing. (c) No wrong answer is produced — `400/2` is a *correct* reading of an out-of-spec encoding, and an attacker gains nothing, since anything expressible as RATIONAL was already expressible as LONG. What it costs is strictness, on the one phase where this library's stated posture is loud rather than lossy. If that trade is intended it should be written down; if not, the gate wants a `RATIONAL`-permitted tag set.

### `FU-5` — no fixture pins the RATIONAL **division**

`src/ifd.rs:1770-1785` and `tests/support/tiff.rs:364-386`. Both well-formed RATIONAL fixtures use denominator **1** (`8368/1`, `5584/1` — the Q2 Monochrom's real crop size). Mutation **M11**, `(Some(_), Some(0)) => out.push(numerator)`, compiled and passed all 58 tests: an implementation that reads the numerator and ignores the denominator entirely is indistinguishable from the correct one. One fixture with a non-unit denominator that divides exactly — `16736/2` for the same 8368 — closes it.

### `FU-6` — `tier_map` predicts into a field that is read as a record

`.repo-context.yaml:42-51`. Full argument in scrutiny point 5. `N=2`, both wrong, opposite directions, mechanism identified. Recommendation: `tier_map` becomes a dispatch hint only; `handoff.to_agent` starts `null` and the handback is its sole writer. Deserves a `guidance/signals.yaml` entry, `type: process-debt`, since it is adjacent to the open `cost-field-has-two-owners` and has the same root — two owners for one field, one of them guessing.

### `FU-7` — `HANDOFF-017`'s YAML `handback:` block is entirely null

`projects/PROJ-001-monochrome-dng-develop/handoffs/HANDOFF-017-…md:43-51`. The prose `## Handback` is complete and careful — the token count, its method, the estimate's basis, the deviations — but the machine-readable block that `handback-sync` actually reads is `status: null, tokens_total: null, …` while `handoff.status: completed`. The number was not lost (the build wrote `cost.sessions` directly, so `SPEC-007` carries the real 19,480,728), but it was saved by the build doing the orchestrator's job — which is the open `cost-field-has-two-owners` signal firing again, from the other side. `just handback-sync SPEC-007` would today read nulls. Filed rather than fixed: this handoff forbids running `handback-sync`, and back-filling another cycle's front-matter is not verify's call.

---

## Acceptance criteria

| # | criterion | verdict |
|---|---|---|
| 1 | Structure / Interpretation split implemented per the table | ✅ 7 interpretation tags costed, 9 structural sites still bare `?`; the 3 cited lines confirmed at 1173/1178/1184 |
| 2 | Leaves still return `Err`; only `sensor()` swallows | ✅ mutation-verified (M7, M8) |
| 3 | `RATIONAL` handled; zero-denominator and non-integral cost the field | ✅ read, not merely tolerated (M12); malformed shapes pinned (M9). `FU-5` on the division |
| 4 | `NoSensorIfdCandidatesMalformed` names only real candidates | ✅ `candidates == [(1, 262)]`; mutation-verified (M10) |
| 5 | Fixtures pin the boundary in **both** directions | ✅ as written — M1 and M7 each turn the opposing fixture red. `FU-3` on the other four structural tags |
| 6 | Ten gates green; fuzz covers the widened `uints()` | ✅ all ten run by me; fuzz target confirmed to reach `uints()` on every entry |

§15 checks 1-8: acceptance met and tested; the five named tests pass; no drift from `DEC-012` (its amendment is the operative text and the implementation transcribes it); no constraint violation (`no-panics-on-untrusted-input` holds — no raw `/`, no `unwrap`, `-F` ×5 green, 10.7 M fuzz runs clean); no new `DEC` needed, and the build's reasoning for that is right — `cost_the_field` copies `SensorMatch`'s prescribed shape and `TYPE_RATIONAL` comes from TIFF 6.0 §2; the reflection is answered substantively; `cost.sessions` carries the build cycle with a real, method-stated number. Checks 9-12: oracle watched red by me; fuzz target exists and ran 61 s; the `src/ifd.rs` provenance row is extended in place with an honest class 1 — specification and the RATIONAL definition cited to TIFF 6.0 §2 "Types"; no new dependency.

### Cost self-report

- **Tokens (total):** **7,830,000** — a transcript sum **deduplicated by `message.id`**, from this session's own JSONL (`~/.claude/projects/<slug>/6be0697c-….jsonl`, session id taken off the scratchpad path): **108** `usage` objects across **64** distinct ids; raw 13,161,712, deduped **7,736,717** (1.70x — inside the 1.61x-2.51x band `SPEC-004`'s verify established). 97.5 % cache-read; deduped cache-creation is **143,213, entirely on the 1-hour tier** (5-minute tier: 0), read from the nested `cache_creation` object rather than assumed. The reported figure rounds the measured 7,736,717 **up** to cover the turns spent writing this handback and committing — captured **before** the session closes, per the warning.
- **Estimated USD:** **19.35** — computed, not harness-reported: `128 × $15/M` input + `46,399 × $75/M` output + `143,213 × $30/M` 1h-cache-write + `7,546,977 × $1.50/M` cache-read at published Opus rates = $19.10 on the measured total, scaled to the rounded figure. Flagged as an estimate. *(Worth noting for whoever reads `cost.rate_per_mtok`: the repo's flat 6.60 USD/Mtok would give **$51.06** for this session — 2.7x high, because a 97.5 %-cache-read session is nothing like the blended basis. The build hit the same thing and also went per-component. That divergence is now on record twice.)*
- **Duration (minutes):** ~55, wall clock.
- **Source of the number:** transcript `usage` objects, read directly — `/cost` is not available from inside a turn.

### Drift and new artifacts

- **New decisions emitted:** none. Nothing in this verify required a judgment `DEC-012`'s amendment does not already make.
- **Deviations from spec:** none by the build beyond the disclosed extension in scrutiny point 1, which is not a deviation — it is the amendment applied where the design table under-enumerated.
- **Follow-up work identified:** `FU-1` … `FU-7` above. `FU-3` and `FU-5` are test-coverage work and belong together in one small spec. `FU-4` needs a decision before code — whether `uints()`'s type gate becomes per-tag, or whether the widening is accepted and written down. `FU-6` and `FU-7` are process, and both want a `guidance/signals.yaml` entry.

### Reflection

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking; the handoff was unusually well-aimed. The one genuine friction was the test-name grep: lib tests list as `ifd::tests::<name>`, so an anchored `^<name>: test$` returns **0 for all five** — which is exactly the shape of the `named-tests-can-pass-vacuously` trap the spec warns about, only inverted into a false *negative*. The warning says "confirm with `--list`"; it does not say the listing is namespaced. Worth one clause.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `DEC-012`'s amendment classifies by **tag**, but the code enforces by **accessor**, and `uints()` is shared by both classes. That mismatch is `FU-4`'s root and nothing in the spec or the decision anticipates it — the amendment is written as though every tag has its own read path. A decision that classifies tags should say which mechanism carries the classification.

3. **If you did this task again, what would you do differently?**
   — Mutation-test the *whole* structural row before reading the one fixture that exists, rather than confirming the claimed mutation first and generalising after. Confirming `RowsPerStrip` felt like confirming the boundary; it was confirming one fifth of it, and the other four fifths took a second pass to notice.
