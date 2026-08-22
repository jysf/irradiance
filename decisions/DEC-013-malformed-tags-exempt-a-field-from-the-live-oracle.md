---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-013
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-21
supersedes: null
superseded_by: null
status: rejected
deciders: [claude]

affected_scope:
  - tests/support/tools.rs
  - tests/metadata_oracle.rs

tags:
  - oracle
  - testing
  - dec-012
---

# DEC-013: A tag already recorded in `malformed_tags` is exempt from the live metadata oracle's field-by-field diff — **REJECTED**

> ⚠ **REJECTED 2026-08-22, at `SPEC-005`'s verify punch-list round (`SB-1`).**
> This record was written during `SPEC-005`'s build and shipped in `418be15`.
> It is wrong on three counts, all measured independently by the orchestrator
> and the reviewer, and the guard it sanctioned has been removed. The record is
> kept — not deleted — because *why* it was wrong is the useful part, and
> because a decision that quietly vanishes teaches nobody. The original text is
> preserved verbatim below the line.

## What was decided instead

**`tools::diff()` compares all eleven fields unconditionally. There is no
`malformed_tags` exemption.**

## Why the original was rejected — three measured counts

1. **The guard was dead code.** Removing the
   `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM)` condition left
   **all 21 oracle tests green** with the corpus present. Measured twice
   independently — by the orchestrator during reconciliation and by the reviewer
   during verify — each with the mutation asserted applied and the tree restored
   byte-identical. A guard that nothing dies without is a guard nobody knows
   works.

2. **Its stated premise is false.** The record claims `K3III.DNG` "would fail
   `AC1`'s own test on `BlackLevelRepeatDim` forever." It would not. `exiftool`
   reports that malformed tag as a bare `1`; `reading_from_fields`'
   `<[u32; 2]>::try_from(v.as_slice()).ok()` degrades a one-element vector to
   `None`; `DEC-012` independently gives our reader `None`. `None == None` — no
   mismatch. The permanent red this decision existed to prevent **cannot
   currently occur**, so every alternative it weighed was weighed against a
   scenario that does not arise.

3. **It records choosing Option C and shipped Option B's shape.** Option C is
   stated as *"`diff()` reads `malformed_tags` and skips exempted fields
   **generically** … no per-file knowledge in the comparator at all … needs no
   update when a FUTURE file exercises a different malformed tag."* The code was
   `!sensor.malformed_tags.contains(&TAG_BLACK_LEVEL_REPEAT_DIM)` — a single
   hardcoded tag, which is the hardcoded-exception shape Option B was rejected
   for. The reviewer settled it with the decisive test rather than by reading:
   a malformed `BlackLevelRepeatDim` diffs `[]`, while an **identically**
   malformed `ActiveArea` still reds. The property Option C was chosen for does
   not hold.

   This is the fourth instance of `measurement-over-generalised` — a guard one
   point wide, recorded as covering a class. The signal was already at its
   `N=3` bar; it is now `N=4`.

## Why the guard was REMOVED rather than made generic

This is the part worth carrying forward, and it is a deliberate choice between
two defensible options.

The reviewer's sharpest observation is that the agreement in count 2 **is an
accident of `SPEC-005/FU-1`** — the defect where a shape-odd tool value is
reclassified as absence. *Fix `FU-1` and the guard becomes necessary.* So the
original record's **conclusion** may well be right; only its premise and its
implementation were wrong.

Keeping a corrected guard would therefore have been reasonable. It was rejected
because **removing it makes `FU-1`'s fix self-forcing.** With a guard in place,
whoever fixes `FU-1` gets the consequence absorbed silently and the real
question — *is a `DEC-012`-tolerated tag exempt from this oracle?* — is answered
by accident, which is precisely the outcome this decision was written to
prevent. With no guard, fixing `FU-1` turns `K3III.DNG` red immediately, and the
fixer must decide deliberately **and ship a test with the answer**.

A dead guard is not neutral: it is a decision made in advance, on evidence that
does not exist yet, that disarms the alarm which would have demanded it.

## What is NOT re-litigated

`DEC-012`'s tolerance is untouched and correct. `AC4.3`'s dedicated test
(`malformed_black_level_repeat_dim_reads_three_different_ways`) still positively
asserts the three-way divergence and is unaffected by this rejection — the
divergence remains explicitly checked, which was the original record's one sound
instinct.

## References

- `SPEC-005` (`AC1`, `AC4.3`), and its verify round's `SB-1` / `FU-1`
- `DEC-012` — `Sensor::malformed_tags`' contract, unchanged
- Signal `measurement-over-generalised`, now `N=4`

---

# ORIGINAL TEXT, preserved verbatim — rejected, do not act on it

# DEC-013: A tag already recorded in `malformed_tags` is exempt from the live metadata oracle's field-by-field diff

## Decision

**`tools::diff()` (`tests/support/tools.rs`, `SPEC-005`) skips comparing a
field whose tag number appears in `Sensor::malformed_tags`.** Every other
field is compared exactly; a genuine, unexplained disagreement still fails,
naming the file, the field, ours and theirs (AC1).

Concretely: `K3III.DNG`'s `BlackLevelRepeatDim` is malformed (count 1 where
DNG requires 2). `DEC-012` already has our reader drop the value, read
`black_level_repeat_dim: None`, and record `50713` in `malformed_tags`.
`exiftool` reads a bare `1` for the same tag (also shape-odd, differently).
Comparing `None` against `Some([1])` directly would report this as an
unexplained `Mismatch` on every run — a permanent, expected red the oracle
would then need to tolerate some OTHER way, or the whole file would need
excluding from AC1 the way `K3III.PEF` is excluded from the dnglab comparison
(AC4.2). Neither is right: this divergence is not unexplained, and it is not
file-wide.

## Context

`AC1` requires the oracle compare eleven fields "and a disagreement fails
naming the file, the field, ours and theirs. Not 'mismatch'." Read literally
and applied uniformly, `K3III.DNG` would fail `AC1`'s own test
(`metadata_matches_exiftool_on_every_corpus_file`) on `BlackLevelRepeatDim`
forever, since `DEC-012`'s tolerance is a permanent, correct feature of the
reader, not a bug to fix. `AC4.3` separately requires a **positive, three-way
assertion** that this exact divergence holds (exiftool's bare `1`, dnglab's
stderr warning, our `None` + `malformed_tags`) — so the divergence is already
checked, explicitly, elsewhere. Reporting it again as an unexplained `AC1`
mismatch would not add a check; it would just make the suite permanently red
for a reason every reader of the failure would have to re-derive from
`DEC-012` each time.

## Alternatives Considered

- **Option A: compare every field unconditionally, exclude `K3III.DNG` from
  `AC1` entirely.**
  - What it is: treat `K3III.DNG` the way `K3III.PEF` is treated for the
    dnglab comparison (AC4.2) — skip the whole file for this oracle.
  - Why rejected: `K3III.DNG`'s OTHER ten fields agree with `exiftool`
    exactly, and that agreement is real signal AC1 would then silently stop
    checking. Unlike `K3III.PEF` (whose divergence is structural — the file
    carries none of the tags dnglab reports), `K3III.DNG`'s divergence is
    scoped to exactly one tag; excluding the whole file throws away ten
    working checks to avoid one expected one.
- **Option B: hardcode a per-file, per-field exception in the test.**
  - What it is: an `if file.path == "PENTAX-K3III-MONO/K3III.DNG" &&
    field == "BlackLevelRepeatDim" { continue }` in the test itself.
  - Why rejected: it duplicates `DEC-012`'s tolerance as a SECOND, unrelated
    mechanism instead of reading the first one's own output.
    `Sensor::malformed_tags` already names exactly the tags whose value is
    known to be dropped and why; a parallel hardcoded exception list is a
    second place that same fact has to be kept correct, and the two can
    drift.
- **Option C (chosen): `diff()` reads `malformed_tags` and skips exempted
  fields generically.**
  - What it is: the rule in `## Decision` above — no per-file knowledge in the
    comparator at all.
  - Why selected: it needs no update when a FUTURE file exercises a different
    malformed tag — `malformed_tags` is already the single source of truth
    for "this field's value is not what the file's bytes would naively say,
    and that is deliberate." A comparator that trusts it is one mechanism,
    not two, and generalizes to any tag `DEC-012`'s `array()` tolerance
    covers, not just `BlackLevelRepeatDim`.

## Consequences

- **Positive:** `AC1`'s test (`metadata_matches_exiftool_on_every_corpus_file`)
  stays a clean, uniform loop over all seven files with no per-file
  branching — the exemption lives in `diff()`, once.
- **Positive:** the exemption is not silent. `AC4.3`'s dedicated test
  (`malformed_black_level_repeat_dim_reads_three_different_ways`) positively
  asserts the exact divergence `diff()` is choosing not to re-report, so
  nothing about this tag's disagreement goes unchecked — it is checked in a
  more specific, more informative way than a generic `Mismatch` could state.
- **Negative:** `diff()` is not a "pure" field-by-field comparator — reading
  it in isolation, without knowing `Sensor::malformed_tags`' contract, could
  look like it is silently hiding a real defect on `BlackLevelRepeatDim`
  specifically. The doc comment on `diff()` states the reason inline for
  exactly this.
- **Neutral:** if a corpus file arrives whose malformed tag is NOT already
  covered by a dedicated positive assertion the way `AC4.3` covers this one,
  `diff()` will silently stop reporting that field's disagreement too. This is
  the sharp edge: exempting via `malformed_tags` is only as safe as `DEC-012`
  is at classifying malformed tags correctly, since a wrongly-classified
  malformed tag would now escape `AC1` as well. Existing `src/ifd.rs` tests
  already exercise `DEC-012`'s classification directly, so this decision does
  not introduce a NEW way for that class of bug to hide, but it does make
  `AC1` no longer an independent second check on it.

## Validation

Right if no future corpus file's `malformed_tags` entry ever needs `AC1` to
ALSO flag it as an unexplained mismatch — i.e. if every tag that ever lands in
`malformed_tags` gets its own dedicated, positive assertion the way `AC4.3`
does for `K3III.DNG`. Revisit if a future spec finds a malformed tag whose
`AC1` exemption was NOT accompanied by such an assertion — that would mean a
real divergence went unchecked rather than differently-checked, which is
exactly the failure mode Option B's rejection was avoiding.

## References

- Related specs: `SPEC-003` (the reader), `SPEC-004` (the typed tag model),
  `SPEC-005` (this oracle)
- Related decisions: `DEC-012` (`Sensor::malformed_tags`' contract — the fact
  this decision reads rather than duplicates)
- Code: `tests/support/tools.rs` — `diff()`; `tests/metadata_oracle.rs` —
  `malformed_black_level_repeat_dim_reads_three_different_ways` (`AC4.3`)
- Constraints: `oracle-must-be-shown-red`
- Raised by: `SPEC-005` build cycle, while writing `tools::diff()`
