---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-012
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

created_at: 2026-08-20
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - src/ifd.rs

tags:
  - parsing
  - error-handling
  - hostile-input
---

# DEC-012: Strict on structure, tolerant on shape — where a malformed tag costs the tag and where it costs the file

## Decision

**A malformedness that changes *what exists* is fatal; a malformedness that
changes only *what a known-optional field says* costs that field alone.**
Concretely, in `src/ifd.rs`:

| Phase | What is read | Malformed → |
|---|---|---|
| **Walk** — `Container::parse` | the header, each IFD's entry table, the chain's `next` pointer, and `SubIFDs` (tag 330) | **fatal to the whole container.** Nothing is readable. |
| **Interpret** — `sensor()`, `scalar()`, `values()`, `array()` | every other tag | **fatal to that call only.** The container and all other tags stay readable. |
| **Interpret**, narrow case — `array::<N>()` | an *optional fixed-length* DNG tag whose `count != N` | **costs the tag.** The value is dropped, the tag number is pushed to `Sensor::malformed_tags`, the file reads. |

The asymmetry SPEC-003's verify cycle flagged (FU-5) is **kept, deliberately**,
and it is narrower than it looked: `array()` tolerates a wrong **count** and
nothing else. A wrong *field type* or an out-of-bounds payload is an error
everywhere, uniformly, including inside `array()` — it just isn't a *container*
error outside the walk.

**`SubIFDs` (tag 330) is the one genuinely debatable case, and it is classified as
structural.** It is optional, so a reader could treat a broken 330 the way it
treats a broken `BlackLevelRepeatDim` — record it and walk `IFD0` alone. It does
not, and should not: see below.

## Context

`SPEC-003` shipped the TIFF/IFD reader. Verify approved the reader and raised
FU-5: `array()` (`src/ifd.rs:796-820`) survives the Pentax K-3 III Monochrome's
real, shipping, malformed `BlackLevelRepeatDim` — count 1 where DNG requires 2 —
while `sub_ifd_offsets_of_last()` (`:666-675`) routes tag 330 through `uints()`
with a bare `?`, so a `SubIFDs` entry with an unreadable field type or a payload
past EOF aborts `Container::parse` **entirely**. Both are "an optional tag is
present but shaped wrong", the module applied opposite policies, and it stated no
rule for which applies when.

No corpus file trips it and nothing is wrong today. It needs deciding now because
**`SPEC-004` widens the type model directly on top of `uints()`** and would
inherit the boundary silently — and because "a reader that survives one malformed
tag and dies on another" is an unstated rule, which is the thing this repo treats
as the defect rather than the behaviour itself.

Constraint in play: `no-panics-on-untrusted-input` demands a *typed error*, and is
satisfied either way. It says nothing about *scope*, which is the actual question.

## Alternatives Considered

- **Option A: make `SubIFDs` tolerant, to match `array()`.**
  - What it is: a broken tag 330 records `330` in a malformed list and yields an
    `IFD0`-only walk instead of failing `parse`.
  - Why rejected: it produces a container that is **structurally a lie**. Every
    downstream question — `sensor_candidates`, `sensor_ifd`, `sensor` — is
    "which IFD holds the plane", and on four of the seven corpus files the answer
    is *in a SubIFD*. Silently returning a container that is missing the IFD the
    caller is looking for turns a parse error into a `MissingTag` or, worse, into
    selecting a **preview** as the sensor plane. SPIKE-001 already measured how
    close that failure is: a Q2M's `SubIFD2` is a full-resolution JPEG preview
    only 56 px narrower than the real plane. Degrading a structural read into a
    plausible wrong answer is the single failure mode this reader is built to
    avoid.

- **Option B: make `array()` strict, to match `SubIFDs`.**
  - What it is: a present-but-wrong-length `BlackLevelRepeatDim` fails the file.
  - Why rejected: it is measurably wrong on real input. A shipping camera writes
    that tag malformed, `dnglab` tolerates it, and refusing a 37 MB Pentax file
    over a two-element hint that PROJ-001 does not even consume yet is strictly
    worse behaviour for zero safety. `malformed_tags` exists so the tolerance is
    *reported* rather than silent, which is what makes it acceptable.

- **Option C (chosen): state the rule, keep both behaviours.**
  - What it is: the table above, with the boundary drawn at **walk vs interpret**
    rather than at *tag identity*, and tag 330 classified as structural because it
    is read during the walk and determines which IFDs exist.
  - Why selected: it is the rule the code already follows, it is defensible in
    both directions, and it gives `SPEC-004` a line to check its widening against
    instead of a precedent to guess at. A tolerant `SubIFDs` is a *lossy* answer;
    a strict one is a *loud* one, and this library's stated posture on hostile
    input is loud.

## ⚠ This decision contradicts its own table — found 2026-08-21 (SPEC-004/FU-16)

The **principle** above says a malformedness that changes only *what a known-optional
field says* costs **that field alone**. The **table** sanctions `sensor()`
propagating a malformed `Orientation` read from `IFD0` — which discards an
already-located sensor plane because of a tag on a **non-sensor** IFD.

Reproduced at SPEC-004's verify: `sensor_matches [1]`, then discarded
(`src/ifd.rs:1011`, a bare `?`). Not a regression — identical on `main` — and
`SPEC-004` closed the same class in the *selection* path (`is_sensor_ifd`, now a
`SensorMatch` tri-state) while the *extraction* path kept it.

A second instance of the same gap (SPEC-004/FU-17): a **DNG-legal `RATIONAL`**
`DefaultCropSize`/`DefaultCropOrigin`/`BlackLevel` makes the **whole file
unreadable**, because `uints()` returns `UnexpectedFieldType` and `sensor()`
propagates it. That is fatal to the file, not a missing field.

**Consequence: this decision must be amended before any spec is designed against
it**, or that spec inherits a table sanctioning the behaviour it exists to fix.
The open question is not the two lines — it is whether "what exists" means *the
plane* or *every tag the plane's record carries*.

## Consequences

- **Positive:** `SPEC-004` inherits a stated boundary. New tag readers classify
  themselves by one question — *does this change what exists, or only what a
  field says?* — instead of by copying whichever neighbour they landed next to.
- **Positive:** the tolerance stays *visible*. `Sensor::malformed_tags` is a
  public field, so "we ignored something" is in the return value, not in a log.
- **Negative:** a file with an otherwise-fine `IFD0` and one corrupt `SubIFDs`
  entry is unreadable, where a tolerant reader would return partial metadata. That
  is a real cost and it is accepted: partial metadata from a structurally broken
  container is exactly the input that produces a confident wrong answer later.
- **Negative:** the rule lives in this record and not in a doc comment beside the
  code. HANDOFF-013 forbade `src/` changes this round (verify found the reader
  sound and the round is records-and-config). `affected_scope: src/ifd.rs` means
  `just decisions-audit --changed` surfaces this decision the moment anyone edits
  the reader — **and SPEC-004's first edit should be a one-line pointer to
  DEC-012 above `array()` and `sub_ifd_offsets_of_last()`.**
- **Neutral:** `array()` tolerating only a wrong *count* is now written down. It
  was never stated, and reading `array()` alone suggests a broader tolerance than
  it has, because the `?` on its `uints()` call is easy to miss.

## Validation

Right if `SPEC-004`'s widened type model can classify each new case with the one
question above and nothing needs re-litigating. Revisit if a **real corpus file**
turns up with a malformed `SubIFDs` that a tolerant reader would handle usefully —
that would be evidence from the world rather than from taste, and it is the only
evidence that should move this. Note the shape of the risk: no held file exercises
a broken 330 at all, so this decision is reasoned, not measured, which is why its
confidence is 0.85 and not higher.

## References

- Related specs: SPEC-003 (this reader), SPEC-004 (the typed tag model that
  inherits the boundary)
- Related decisions: DEC-008 (sample packing — the other STAGE-002 boundary
  SPEC-003 deliberately declined to cross)
- Code: `src/ifd.rs` — `array()` `:796`, `sub_ifd_offsets_of_last()` `:666`,
  `uints()` `:714`, `Sensor::malformed_tags` `:468`
- Tests: `a_malformed_fixed_length_tag_costs_the_tag_not_the_file`
  (`src/ifd.rs:1374`) — the tier-A synthetic of the Pentax's real defect
- Constraints: `no-panics-on-untrusted-input`
- Raised by: HANDOFF-012's punch list, FU-5; scoped by HANDOFF-013
