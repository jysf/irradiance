---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-007
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-012]                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-004]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "a DNG-legal file must not become unreadable because of one tag"

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
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-007: Unreadable tags in the extraction path, and DEC-012s contradiction

## Context

`SPEC-004` closed `SPEC-003/FU-11` for the **selection** path — `is_sensor_ifd` is now a
`SensorMatch` tri-state, so a malformed identifying tag on one IFD no longer aborts
the scan of the others. Its verify found the **extraction** path still has the same
gap, twice:

- **`SPEC-004/FU-16`** — `sensor()` reads `Orientation` from `IFD0` with a bare `?`
  (`src/ifd.rs:1011`), so a malformed tag on a **non-sensor** IFD discards an
  already-located plane. Reproduced: `sensor_matches [1]`, then discarded.
- **`SPEC-004/FU-17`** — a **DNG-legal `RATIONAL`** `DefaultCropSize`/`DefaultCropOrigin`/
  `BlackLevel` makes the **whole file unreadable**: `uints()` (`src/ifd.rs:788`)
  returns `UnexpectedFieldType` and `sensor()` propagates it. Reproduced. This is
  fatal to the file, not a missing field — a severity the build's framing
  understated.

Neither is a regression; both are identical on `main`.

⚠ **`DEC-012` must be amended first.** Its principle forbids exactly this, and its
*table* sanctions it. A spec designed against it today would inherit a decision
that blesses the behaviour this spec exists to fix. The contradiction is recorded
on the DEC itself.

## Goal

Make the extraction path obey `DEC-012`'s principle: **a DNG-legal file must not
become unreadable because one interpretation tag is malformed or uses a legal type
we have not implemented.**

`DEC-012` was **amended 2026-08-21** and now answers the question this spec was
framed around, so the spec does not have to. Read the amendment first — it is the
operative text.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

1. **The Structure / Interpretation split is implemented as `DEC-012` states.**
   Measured at design, the affected call sites in `src/ifd.rs` are:

   | line | tag | class | today | required |
   |---|---|---|---|---|
   | 1012/1014/1016 | `Orientation` | interpretation | bare `?` | costs the field |
   | 1031 | `BlackLevel` | interpretation | bare `?` | costs the field |
   | 1032 | `WhiteLevel` | interpretation | bare `?` | costs the field |
   | 1038 | `ActiveArea` | interpretation | bare `?` | costs the field |
   | 1024 | `SamplesPerPixel` | **structure** | bare `?` | **stays fatal** |
   | 1027 | `Compression` | **structure** | bare `?` | **stays fatal** |
   | 1028 | `RowsPerStrip` | **structure** | bare `?` | **stays fatal** — see note |

   ⚠ `RowsPerStrip` is structural because it maps strips to rows; without it a
   multi-strip plane cannot be assembled honestly. It is *inferable* on a
   single-strip file (`rows_per_strip == height`), and every corpus file is
   single-strip — so **do not let a green corpus talk you out of the fatal
   classification.** If you disagree, say so in the handback; do not just soften it.

2. **A leaf accessor may still return `Err`.** `scalar()`/`array()`/`values()`
   keep reporting a malformed tag honestly. What changes is that **`sensor()` must
   not inherit that failure for an interpretation tag** — it records the tag in
   `Sensor::malformed_tags` and continues.

3. **`RATIONAL` is handled** (`SPEC-004/FU-17`). `TYPE_RATIONAL` is not even
   defined in `src/ifd.rs` today (only BYTE/SHORT/LONG/UNDEFINED/IFD at :141-145).
   Read it as the two-`u32` pair the TIFF spec defines. A zero denominator, or a
   value that is not integral, is a **malformed shape** — it costs the field, it
   does not fail the file.

4. **`SPEC-004/FU-20`:** `NoSensorIfdCandidatesMalformed` must not name IFDs that
   were never candidates (`src/ifd.rs:916`).

5. **Fixtures pin the boundary in BOTH directions** — an interpretation tag
   malformed → the file still reads and the tag is recorded; a structural tag
   malformed → still fatal. A change that only demonstrates the new tolerance has
   not shown the boundary still exists.

6. Ten gates green; fuzz covers the widened `uints()`.

## Failing Tests

```bash
cargo test --all-features malformed_interpretation_tag_costs_only_the_field
cargo test --all-features malformed_structural_tag_is_still_fatal
cargo test --all-features rational_default_crop_size_reads_or_costs_the_field
cargo test --all-features malformed_orientation_on_ifd0_keeps_the_plane   # SPEC-004/FU-16
cargo test --all-features candidates_malformed_names_only_candidates      # SPEC-004/FU-20
```

⚠ **`cargo test <name>` matching ZERO tests exits 0** — a spec that names its
tests can pass vacuously (`named-tests-can-pass-vacuously`). Confirm each name
exists with `cargo test -- --list`, and **sum across targets**; reading one
target's line has produced a wrong answer twice on this project, in both
directions.

## Non-Goals

- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Widening `uints()` to types no DNG tag we read can carry. `RATIONAL` is in scope
  because the DNG spec permits it for tags we already read; the signed types and
  `ASCII` are not, unless criterion 1 says otherwise.

## Notes for the Implementer

### `DEC-012`'s amendment is the spec. Read it first.

The line it draws: **"what exists" is the plane — its presence, location and
extent.** A tag that determines whether there is a plane and where it is, is
structural and fatal. Every other tag describes how to *interpret* a plane that
already exists, and malformed costs that field alone.

The defect being fixed is subtle and worth understanding rather than pattern-matching:
the old table said a malformed tag was *"fatal to that call only"* — but `sensor()`
**is** a call, so "only" silently included the plane. It conflated the accessor
that **read** the tag with the accessor the caller **invoked**.

### The shape to copy already exists

`SPEC-004` solved the same problem for the *selection* path: `is_sensor_ifd`
returns a `SensorMatch { Yes | No | Unreadable(tag) }` tri-state, so one bad IFD
does not abort the scan — the structural rule applied **per-IFD instead of
per-file**. Do the analogous thing per-**tag** in `sensor()`.

### Do not treat a green corpus as evidence

Every corpus file is single-strip, so the `RowsPerStrip` classification is
untested by real data. No corpus file carries a malformed tag on the paths this
spec changes — that is why `SPEC-004/FU-16` and `FU-17` were latent for two specs.
The hand-built fixtures are the evidence here; the corpus is a regression check.

### Scope

The extraction path and `uints()`. **No levels arithmetic, no cropping, no
orientation transform** — STAGE-002 and `DEC-008`. Extracting is in scope; applying
is not.

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
