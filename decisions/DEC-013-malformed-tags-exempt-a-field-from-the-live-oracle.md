---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-013
  type: decision
  confidence: 0.85
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
status: accepted
deciders: [claude]

affected_scope:
  - tests/support/tools.rs
  - tests/metadata_oracle.rs

tags:
  - oracle
  - testing
  - dec-012
---

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
