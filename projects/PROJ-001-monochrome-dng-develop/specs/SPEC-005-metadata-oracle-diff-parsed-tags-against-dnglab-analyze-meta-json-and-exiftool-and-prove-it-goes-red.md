---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-005
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # ⚠ DISPATCH HINT from tier_map.build (SPEC-007/FU-6, 0 for 2).
                                   #   Whoever runs the cycle corrects it to what ACTUALLY ran.
  created_at: 2026-08-21

references:
  decisions: [DEC-003, DEC-004, DEC-012]  # [DEC-NNN, DEC-MMM]
  constraints: [oracle-must-be-shown-red, no-copyleft-dependencies, provenance-recorded-per-algorithm, library-not-application, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-002, SPEC-003, SPEC-004, SPEC-007, SPEC-008]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-004]                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "proves the container half is right rather than merely plausible"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 12000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-08-21
      notes: "main-loop, not separately metered (AGENTS.md §4). Design cycle: probed both oracle tools against all seven real corpus files BEFORE writing the spec, per §15 design rule 4 — dnglab 0.7.2 and exiftool 13.55 confirmed by --version, the full ours-vs-both-tools matrix in ## Implementation Context measured rather than transcribed. Four traps found by running rather than reasoning: dnglab's stderr warning on K3III.DNG corrupts a 2>&1 JSON stream; its cropArea.p is sensor-absolute where ours and exiftool's are DNG-relative (verified on all six DNGs, worked example on K3III.DNG); its K3III.PEF black/white/crop values are in no tag in that file (rawler's camera database) and its PEF bitDepth is output depth 16 not BitsPerSample 14; and exiftool exits 0 on both a truncated file and an absent tag, so its exit code carries no signal. The third finding removed the serde_json question entirely — exiftool -T -n -s3 emits tab-separated values needing no parser — which is why AC7 forbids a new dependency rather than sanctioning one. HANDOFF-021 written for build."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-005: Metadata oracle: diff parsed tags against `dnglab analyze --meta --json` and `exiftool`, and prove it goes red


## Context

`tests/ifd_reader.rs` already carries an `Expected` table of tag values
**transcribed by hand** from `exiftool 13.55` on 2026-08-20. That file's own
doc comment names the debt:

> *"They are pinned as literals here rather than shelled out to, because
> `SPEC-005` is the spec that builds the **live** metadata oracle."*

A transcribed table is a self-report by a past session (DEC-004 rule 1). It
cannot notice that our reader drifted, that a tool changed its answer, or that
the transcription was wrong on the day — and this repo has already shipped four
wrong corpus claims across SPEC-003/004/008, every one a `find`/`exiftool`
away. This spec replaces the frozen copy with a **live diff**, and ships the
red-proof that makes the diff worth trusting.

It is the last spec in `STAGE-001`'s backlog and the stage's stated success
criterion: *"Our parsed tags match `dnglab analyze --meta --json` and `exiftool`
on every tier-B file we hold"* and *"the metadata oracle goes red on a
deliberately corrupted tag — proven, not assumed."*

### ⚠ The design-time probe changed the design — measured 2026-08-21

Everything in `## Implementation Context` below was **run**, on all seven corpus
files, before this spec was written. Three findings moved the design:

1. **The two tools are not two opinions about the same thing.** `exiftool` reads
   **what the file says**, per IFD. `dnglab` reports **what a decoder concluded**,
   through rawler's camera database. On `K3III.PEF` — a vendor container with no
   DNG tags — our reader reports `black_level: None` and dnglab reports `64`.
   Neither is wrong; they answer different questions. Treating them as
   interchangeable would have produced an oracle that demands our reader
   hallucinate values the file does not contain.
2. **`dnglab`'s `cropArea.p` is in sensor coordinates, ours is DNG-relative.**
   Measured on `K3III.DNG`: ours `(28, 24)`, dnglab `(54, 58)`, and
   `54 = 26 + 28`, `58 = 34 + 24` — dnglab adds the `ActiveArea` origin.
   `exiftool` reports `28 24`, agreeing with us. A naive diff would have called
   our correct reader wrong.
3. **No new dependency is needed.** `exiftool -T -n -s3` emits one tab-separated
   line in the order requested, with `-` for an absent tag. That removes the
   `serde_json` question entirely — which matters, because a JSON dev-dependency
   would need its own `DEC` under `no-new-top-level-deps-without-decision`.

## Goal

Replace `tests/ifd_reader.rs`'s hand-transcribed `Expected` table with a live
oracle that runs `exiftool` and `dnglab` as **tools** (never linked) and diffs
their output against `Sensor` field-by-field, naming any field that disagrees —
and prove it goes red, in CI as well as on a corpus machine.

⚠ Scope boundary — do **not** extend this layer to cover levels *correctness*.
`DEC-004` settles that levels are verified analytically, and `SPIKE-001`
measured why no comparison-based check can do it. Reading `BlackLevel` and
comparing it to what the tools read is in scope; asserting that subtracting it
is right is not.

## Inputs

- **Files to read:**
  - `tests/ifd_reader.rs` — the `Expected` table this replaces, and the skip
    idiom to copy
  - `tests/support/corpus.rs` — `Manifest`, `CorpusRoot`, `CorpusFile::require`.
    ⚠ It already exports a `pub struct Oracle` (the manifest's pinned
    `raw_checksum`). Do **not** reuse that name; the new module is `tools.rs`
  - `docs/oracle-contract.md` — the three layers, and why this is the cheapest
  - `decisions/DEC-012-*.md` — what a malformed tag costs; `K3III.DNG` exercises it
  - `guidance/toolchain-brief.md` — the three `+toolchain` traps
- **External tools (run, never linked):** `exiftool 13.55`, `dnglab 0.7.2`
- **Related code paths:** `src/ifd.rs` (`Sensor`, `Container::sensor`), `tests/`

## Outputs

- **Files created:**
  - `tests/support/tools.rs` — shells out to `exiftool`/`dnglab`, parses their
    output, returns a typed `ToolReading`. Dev-only; **not** in the library.
  - `tests/metadata_oracle.rs` — the oracle test and both red-proofs.
  - `tests/oracle-fixtures/` — committed sample tool output (plain text, a few
    hundred bytes) so the comparator's red-proof runs in CI with no tool and no
    corpus.
- **Files modified:**
  - `tests/ifd_reader.rs` — the `Expected` table's *tag-value* columns are
    deleted and its doc comment updated to point at the live oracle. ⚠ Keep the
    columns the oracle does **not** cover: `big_endian`, `ifds`, `sensor_index`,
    `opcode_lists`, `malformed`. Those are our reader's own structure claims and
    no external tool reports them.
  - `docs/oracle-contract.md` — gains the measured Metadata-layer section.
  - `docs/conformance-matrix.md`, `CHANGELOG.md` — the usual rows.
- **New functions:** `tools::exiftool(path, group, tags) -> Result<Vec<Field>, ToolError>`,
  `tools::dnglab_meta(path) -> Result<DnglabMeta, ToolError>`,
  `tools::diff(sensor: &Sensor, reading: &ToolReading) -> Vec<Mismatch>`
- **New flags / options:** none. The oracle is driven by
  `$IRRADIANCE_CORPUS_DIR` (existing, `SPEC-002`) and by tool presence on
  `PATH`; there is no new environment variable and no new `just` flag.
- **New `just` recipe:** `just oracle-meta` — runs only this test file with the
  corpus, so the gate is a word rather than a pasted command
  (`SPEC-003/FU-8`'s lesson: *a gate documented as a raw command is a gate that
  will be run wrong*).

## Acceptance Criteria

- [ ] **AC1 — exiftool is the tag-level oracle, on all seven files.** For each
      manifest entry, `Sensor`'s `dimensions`, `bits_per_sample`, `compression`,
      `photometric`, `black_level`, `white_level`, `black_repeat`, `active_area`,
      `crop_origin`, `crop_size` and `orientation` are compared to `exiftool`'s
      reading of the **same IFD**, and a disagreement fails **naming the file,
      the field, ours and theirs**. Not "mismatch".
- [ ] **AC2 — absence is compared, not skipped.** A tag exiftool reports as `-`
      must read `None` on our side, and vice versa. Measured cases that must be
      exercised: `ActiveArea` absent on both M Monochrom files; `BlackLevel`,
      `WhiteLevel`, `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize` all
      absent on `K3III.PEF`. An oracle that ignores `None` cannot catch a reader
      that invents values.
- [ ] **AC3 — dnglab cross-checks the six scalars whose keys are unique**
      (`rawWidth`, `rawHeight`, `bitDepth`, `whitelevels`, `orientation`,
      `blacklevels.levels`), on the **six DNG files only**. Each extraction
      **asserts its match count is exactly 1** before using the value.
- [ ] **AC4 — the three known divergences are asserted, not ignored.** Each is a
      positive assertion that the divergence still holds, so a dnglab change is
      caught rather than silently redefining ground truth (`DEC-003`'s rule for
      `raw_checksum`, applied here):
      1. `dnglab.cropArea.p == active_area.(left, top) + crop_origin`, on all
         six DNGs (with an absent `ActiveArea` read as `(0, 0)`).
      2. `K3III.PEF` is excluded from the dnglab comparison **by name and with a
         written reason** — its values come from rawler's camera database, not
         the file. Its `bitDepth` divergence (dnglab `16`, file `14`) is asserted
         as the evidence for that.
      3. `K3III.DNG`'s `BlackLevelRepeatDim` is malformed: exiftool reports a
         bare `1`, dnglab warns and substitutes `1x1`, and we report
         `black_repeat: None` with `50713` in `malformed_tags`. The oracle
         asserts **all three**, because our answer being different here is
         `DEC-012` working, not a defect.
- [ ] **AC5 — the oracle goes RED, and the red runs in CI.** Two proofs, both
      with a negative control:
      - **Tier A (CI, no tool, no corpus):** the comparator over committed
        fixture text. Control: an honest `Sensor` → empty diff. Red: one field
        perturbed → exactly one `Mismatch`, naming **that** field.
      - **Tier B (corpus + tools):** one tag's value bytes patched in an
        in-memory copy of a real file; our reader reads the patched bytes, the
        tool reading is of the original, and the oracle must report exactly that
        field. Restore and re-run → clean.
- [ ] **AC6 — missing tool skips LOUDLY, naming the tool**, on the same terms as
      a missing corpus file (`SPEC-002` AC3). A silent skip reports green for
      work it never did.
- [ ] **AC7 — no new dependency.** `Cargo.toml`'s `[dependencies]` and
      `[dev-dependencies]` are byte-identical. `just deny` and `just deny-fuzz`
      both still pass.
- [ ] **AC8 — `tests/ifd_reader.rs`'s transcribed tag columns are gone**, and no
      expected tag value is a hand-typed literal anywhere outside
      `tests/oracle-fixtures/`. The structure columns listed under Outputs stay.
- [ ] **AC9 — ten gates green**, plus `just oracle-meta`, and the fuzz target
      re-run because `tests/` gained a lane (seeds unchanged is a fine result —
      say so).

## Failing Tests

Written during **design**, BEFORE handoff. Build makes these pass.
⚠ `cargo test <name>` matching **zero** tests exits 0
(`named-tests-can-pass-vacuously`). Confirm each name EXISTS via per-target
`-- --list` before trusting any green, and **sum across targets**.

- **`tests/metadata_oracle.rs`**
  - `metadata_matches_exiftool_on_every_corpus_file` — AC1, AC2. Tier B.
  - `dnglab_scalars_agree_on_the_six_dng_files` — AC3. Tier B.
  - `dnglab_crop_origin_is_active_area_plus_default_crop_origin` — AC4.1. Tier B.
  - `pef_is_excluded_from_dnglab_because_its_values_are_not_in_the_file` — AC4.2. Tier B.
  - `malformed_black_level_repeat_dim_reads_three_different_ways` — AC4.3. Tier B.
  - `oracle_is_clean_on_an_unmodified_reading` — AC5 tier-A **control**. CI.
  - `oracle_names_the_one_field_that_was_perturbed` — AC5 tier-A **red**. CI.
  - `oracle_goes_red_on_a_patched_tag_in_a_real_file` — AC5 tier-B red. Tier B.
  - `a_missing_tool_skips_loudly_naming_it` — AC6. CI.

## Non-Goals

- **Levels correctness** — `DEC-004`; see the Goal's scope boundary.
- **The plane or develop layers** — `--raw-checksum` is already pinned in the
  manifest; `--srgb` is `STAGE-003`.
- **Adding `serde_json` or any other crate.** The probe showed it is unnecessary;
  if build believes otherwise, that is a **stop-and-ask**, not a build step.
- **Reading dnglab's or exiftool's source.** Run them. Reading dnglab is a
  `provenance-recorded-per-algorithm` violation.
- **Making this run in CI end-to-end.** It cannot — `DEC-003`. Only the tier-A
  half does, and that limit is stated rather than papered over.

## Implementation Context

> **Every number below was measured on 2026-08-21** by running the command on
> the real corpus at `$IRRADIANCE_CORPUS_DIR`. `dnglab 0.7.2`, `exiftool 13.55`,
> both confirmed by `--version`. Re-measure before trusting on another host.

### The exiftool invocation, and why this form

```bash
exiftool -T -n -s3 -SubIFD:ImageWidth -SubIFD:BlackLevel ... -IFD0:Orientation <file>
```

`-T` tab-separated · `-n` numeric (without it `Orientation` is
`"Horizontal (normal)"` and `Compression` is `"Uncompressed"`) · `-s3` bare
values, no tag names. Output is **one line**, values in the order requested.

Measured on `L1021223.DNG`:

```
8424	5632	14	512	16383	1 1	0 0 5632 8392	12 24	8368 5584	1	34892	1
```

- **Absent tag → a single `-`.** Measured twice: `SubIFD:ActiveArea` on
  `L1000622.DNG`, and `SubIFD:BlackLevel` on `K3III.PEF`.
- **Multi-value tags are space-separated** inside their tab-delimited field:
  `"1 1"`, `"0 0 5632 8392"` (top left bottom right — our `ActiveArea`'s exact
  field order), `"12 24"`, `"8368 5584"`.
- ⚠ **exiftool exits 0 on a truncated file and on an absent tag.** Measured:
  the first 4 KB of `L1021223.DNG` still yields `SubIFD:ImageWidth 8424`,
  exit 0, because the IFD tables live in that first 4 KB. **The exit code
  carries no signal — only the values do.** (`dnglab` exits `2` on the same
  input.)

### ⚠ The IFD group name is per-file and must not be derived

`exiftool -g1` groups by IFD, and **the first SubIFD has no numeric suffix**:
`IFD0`, `SubIFD`, `SubIFD1`, `SubIFD2`. Our walk indexes `#0, #1, #2, #3`. The
mapping is not positional-obvious and is **measured**, not computed:

| file | our `sensor_ifd` | exiftool group |
|---|---|---|
| `L1021223.DNG`, `L1026016.DNG`, `L1026192.DNG` | `#1` | `SubIFD` |
| `L1000622.DNG`, `M2462362.DNG`, `K3III.DNG` | `#1` | `SubIFD` |
| **`K3III.PEF`** | **`#0`** | **`IFD0`** |

`Orientation` lives in **`IFD0` on all seven**, never in the sensor IFD —
which is exactly what our IFD0-first fallback (`SPEC-007`, `SPEC-008`) reads.
`L1026016.DNG` is the per-frame case: `IFD0:Orientation 6` where its two
siblings read `1`. That single file is why `unrun-docs-carry-errors` exists;
keep it in the set.

⚠ **Do not select the sensor IFD by size.** `L1021223.DNG`'s `SubIFD2` is
`8368 × 5584` — the full-resolution JPEG preview, 56 px narrower than the plane.

### The full measured matrix — ours vs both tools

| file | dims | bits | black | white | repeat | ActiveArea (t l b r) | cropOrigin | cropSize | orient | malformed |
|---|---|---|---|---|---|---|---|---|---|---|
| `L1021223.DNG` | 8424×5632 | 14 | 512 | 16383 | [1,1] | 0 0 5632 8392 | 12 24 | 8368 5584 | 1 | — |
| `L1026016.DNG` | 8424×5632 | 14 | 512 | 16383 | [1,1] | 0 0 5632 8392 | 12 24 | 8368 5584 | **6** | — |
| `L1026192.DNG` | 8424×5632 | 14 | 512 | 16383 | [1,1] | 0 0 5632 8392 | 12 24 | 8368 5584 | 1 | — |
| `L1000622.DNG` | 5216×3472 | 16 | 220 | 16383 | [1,1] | **absent** | 2 2 | 5212 3468 | 1 | — |
| `M2462362.DNG` | 5984×4000 | 12 | 0 | 3750 | [1,1] | **absent** | 4 4 | 5976 3992 | 1 | — |
| `K3III.DNG` | 6304×4224 | 14 | 64 | 16378 | **None** | 34 26 4194 6250 | 28 24 | 6192 4128 | 1 | **[50713]** |
| `K3III.PEF` | 6304×4224 | 14 | **None** | **None** | **None** | **None** | **None** | **None** | 1 | — |

exiftool agrees with this table field-for-field on all seven — including
`K3III.DNG`'s `DefaultCropOrigin "28 24"` and `K3III.PEF`'s empty DNG tag set.

### dnglab: the same file, a different question

```bash
dnglab analyze --meta --json <file>   # or --yaml
```

Shape: `data.metadata.rawParams.{rawWidth,rawHeight,bitDepth,cropArea,activeArea,
blacklevels,whitelevels}` and `data.metadata.rawMetadata.exif.orientation`.

- ⚠ **`blacklevels.levels` is an array of rational STRINGS** — `["512/1"]`, not
  `[512]`. Parse `N/D`; do not assume `D == 1` without checking.
- ⚠ **Capture STDOUT ONLY.** On `K3III.DNG`, dnglab writes an ANSI-coloured
  warning to **stderr** — `File has BlackLevelRepeatDim tag but with invalid
  length: 1` — and merging the streams (`2>&1`) makes the JSON unparseable at
  byte 1. Reproduced: with `2>/dev/null` all seven parse; with `2>&1` that one
  file does not.
- ⚠ **`activeArea` is `null`** on both M Monochrom files, matching exiftool's
  absent tag and our `None`.
- ⚠ **`cropArea.p` is sensor-absolute.** `dnglab.cropArea.p ==
  active_area.(left, top) + crop_origin`, verified on all six DNGs. Worked
  example, `K3III.DNG`: `(26, 34) + (28, 24) = (54, 58)`, which is what dnglab
  prints.
- ⚠ **`K3III.PEF`: dnglab reports `black 64`, `white 16378`, an `activeArea` and
  a `cropArea` that are in NO DNG tag in that file.** exiftool finds none of
  them either — its only Pentax reading is `Pentax:WhiteLevel 16378`, a
  MakerNote namespace we do not parse. Those numbers come from rawler's camera
  database. dnglab also reports `bitDepth 16` where the file's `BitsPerSample`
  is `14` — output depth, not the tag. **This file is excluded from the dnglab
  comparison and included in the exiftool one.**

### Extraction without a dependency

`exiftool -T` needs no parser: split the line on `\t`, then on `' '`.
For dnglab, extract only the **six scalars whose keys are unique in the
document** and **assert the match count is exactly 1** before using a value —
`attribute-text-inside-doc-comments`' general form: *`index()` on source text
finds documentation about the code as readily as the code, so assert your match
count rather than taking the first hit.* `x`/`y`/`w`/`h` are **not** unique
(they appear under both `cropArea` and `activeArea`); read the rectangles from
exiftool and derive dnglab's expected `cropArea.p` arithmetically per AC4.1.

### Red-proof design

The tier-A control/red pair is the load-bearing one, because it is the only half
CI can run. Its fixtures are committed text — no corpus, no tool, licence-clean.

For the tier-B red, patch a **value** in an in-memory copy (e.g. `WhiteLevel`'s
four payload bytes), run our reader on the patched bytes, and compare against
the tool reading of the **unpatched** file. Both must be true before any
conclusion is drawn: **assert the patch changed the buffer**, and assert the
oracle names **that** field and no other. This repo has concluded from a
mutation that never applied five separate times.

### Traps carried in from prior specs

- `~/.cargo/bin/cargo +1.90.0 …` for MSRV; `PATH="$HOME/.cargo/bin:$PATH"` for
  fuzz. Three traps, three different fixes — `guidance/toolchain-brief.md`.
- `just deny` **and** `just deny-fuzz`; two graphs, both required.
- Sum test counts across **all five targets**; a zero-match `cargo test <name>`
  exits 0.
- `cost.sessions` gets a **deduped-by-`message.id`** token figure, and say that
  you deduped. The raw-to-deduped factor has ranged 1.61×–2.51× over eight
  observations; no fixed correction is valid.

## Notes for the Implementer

- **`irradiance` is a library** (`library-not-application`). Every line of this
  spec lives under `tests/`. Nothing here may touch `src/` — if you believe a
  `src/` change is needed to make the oracle work, that is a finding to report,
  not a change to make.
- **`dnglab` is LGPL-2.1 and is RUN, never linked.** Do not add `rawler`,
  `rawloader` or any RAW crate, including as a dev-dependency. Do not read
  dnglab's source to resolve a disagreement — re-measure the file's bytes
  instead.
- **Provenance:** no new algorithm, so no new ledger row is expected — extend
  `src/ifd.rs`'s existing row only if you disagree, with the honest class.
- **Where the tools disagree with us, our reader is not automatically wrong.**
  Three measured cases are already written down above. If you find a **fourth**,
  do not "fix" `src/` to match a tool: record it, and hand it back as a finding.
- Follow-ups you raise get `SB-N`/`FU-N` ids numbered for **this** spec, and
  they will be dispositioned at ship into `fixed` / a spec / a signal / an
  explicit close — AGENTS.md §15, *Where an unresolved follow-up goes*.

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
