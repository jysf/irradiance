---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-004
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
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
  to_agent: claude-opus-5          # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-08-21

references:
  decisions: [DEC-008, DEC-012]                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-003]                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "delivers the typed metadata the develop pipeline reads"

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
  sessions:
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 50
      recorded_at: 2026-08-21
      notes: "Build cycle for SPEC-004 (HANDOFF-015), commit pending on feat/spec-004-tag-model, not merged. tokens_total is null, not by default: this session ran as the top-level interactive Claude Code session rather than a sub-agent an orchestrator metered via subagent_tokens, and there was no tool-level way to run /cost or read raw per-message usage objects from inside a turn to reproduce SPEC-003's dedup-by-message.id methodology (1.61x-2.25x measured range on record). See HANDOFF-015's handback notes for the full reasoning. Ten gates green and pasted in the handoff, including a fresh 13.6M-execution fuzz run with two new FU-11 seed fixtures. Found and corrected a stale-context issue: SPEC-003's build had already shipped most of AC1's tag extraction (contrary to this spec's own Context section); the real remaining work was AC1's typing (bare arrays to named structs), FU-11 itself, and the literally-named Failing Tests commands, none of which existed under those names before this build."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-004: DNG tag model and typed metadata extraction

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-004]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

SPEC-003 shipped the container reader and a `Sensor` struct carrying the tags
needed to *locate and size* the plane: dimensions, bits, samples, photometric,
compression, and the strip table. It stops exactly where geometry begins.

Everything downstream — STAGE-002's levels and crop, STAGE-003's opcodes — reads
tags this spec has not yet extracted. It also inherits two obligations SPEC-003
deliberately deferred rather than make a `src/` edit in a records-only round.

## Goal

Typed extraction of the remaining tags the develop pipeline consumes:
`BlackLevel`, `WhiteLevel`, `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize`,
`Orientation`, and the **presence** of `OpcodeList1`/`OpcodeList3` (presence only —
executing them is STAGE-003).

And close the two inherited obligations, which are really one question: **when a
tag is malformed, what does it cost?**

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

1. The tags above are extracted with types that make illegal states hard to
   build — `ActiveArea` as a rectangle, not a bare `Vec<u32>` the caller must
   remember is `[top, left, bottom, right]`.
2. **`Orientation` is read from the file, every time.** Measured across our
   corpus it varies frame to frame on one body; a hardcoded value passes on one
   frame and fails on the next.
3. **Absent optional tags are absent, not defaulted silently.** `ActiveArea` is
   missing entirely on the M Monochrom, and `NewSubfileType` is missing on
   `K3III.PEF` — where TIFF's absent-means-0 default is what finds the plane at
   all. The type must distinguish "absent" from "present and zero".
4. **`DEC-012` implemented** — a malformedness that changes *what exists* is
   fatal; one that changes only *what a known-optional field says* costs that
   field alone.
5. **FU-11 closed.** `is_sensor_ifd` currently `?`-propagates `scalar()` errors
   and runs over **every** IFD, so a malformed tag on a *thumbnail* fails the whole
   container — which contradicts DEC-012's own rule. ⚠ **The obvious fix is
   wrong:** silently treating a malformed scalar as "not a sensor IFD" would hide
   a real plane behind a bad tag. A malformed candidate must be **skipped and
   recorded**, and if no candidate is then found, the error must say *why* rather
   than a bare `NoSensorIfd`.
6. Extracted values match `exiftool` on all 7 corpus files, pinned as an expected
   table so it runs every commit.
7. The fuzz target covers the new extraction paths; all ten gates green.

## Failing Tests

```bash
cargo test --all-features tag_model_matches_exiftool     # all 7 files
cargo test --all-features orientation_is_per_frame       # rotated + unrotated
cargo test --all-features absent_tag_is_absent_not_zero  # M Monochrom ActiveArea
cargo test --all-features malformed_tag_costs_only_that_tag   # DEC-012
cargo test --all-features malformed_on_thumbnail_does_not_lose_the_plane  # FU-11
```

The last two are the spec. Build them as **hand-constructed TIFFs** via
`tests/support/tiff.rs` (SPEC-003 shipped it) — a malformed tag on a *non-sensor*
IFD, and a malformed tag on the *sensor* IFD, must have different outcomes and
both must be asserted.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- ...

## Notes for the Implementer

### The two inherited obligations are one question

`DEC-012` states the rule; FU-11 is the place the code contradicts it. Read the
DEC first — it was written during SPEC-003's fix round precisely so this spec
would not have to re-derive it.

**Measured at design:** `is_sensor_ifd` (`src/ifd.rs:836`) calls `self.scalar(...)?`
three times, and `sensor_candidates`, `sensor_ifd` and `sensor` each call it over
every IFD. So the failure is latent today only because no corpus file carries a
malformed tag *on that path* — the Pentax's malformed `BlackLevelRepeatDim`
(tag 50713) is not one of the three. Do not conclude from a green corpus that the
path is sound.

**The subtlety worth getting right:** "skip the malformed candidate" is correct for
a thumbnail and wrong for the plane. If the *sensor* IFD's `Photometric` is
malformed, skipping it silently converts a readable file into `NoSensorIfd` with no
explanation. Record what was skipped and why, and surface it — the same discipline
as the corpus reader's loud skip.

### Corpus facts, re-measured 2026-08-20 — use these, not the older numbers

- **6 `II`, 1 `MM`** across 7 files.
- **4 uncompressed, 2 JPEG (code 7), 1 vendor-private (65535)**.
- `K3III.PEF` has **no SubIFD and no `NewSubfileType`**; its plane is in `IFD0` and
  it is the only file with a real IFD *chain*.
- The M Monochrom has **no `ActiveArea`** and **no opcode lists**.

Three earlier claims in this project's specs were wrong on exactly these points.
Re-measure anything you are about to assert.

### Scope

Tags only. **No levels arithmetic, no cropping, no orientation transform** — those
are STAGE-002 and `DEC-008`'s territory. Extracting `BlackLevel` is in scope;
subtracting it is not.

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
