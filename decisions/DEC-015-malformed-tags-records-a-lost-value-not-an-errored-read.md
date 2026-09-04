---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-015
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-03
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - src/ifd.rs

tags:
  - oracle
  - testing
  - dec-012
  - dec-014
---

# DEC-015: `malformed_tags` records a lost value, not an errored read — `SPEC-008/FU-3` decided as Option B

## Decision

**A tag is recorded in `Sensor::malformed_tags` only when its value was
actually lost — never merely because some read of it errored while another
read of the same tag, from a different source, succeeded.** Concretely: a
well-formed `IFD0` `Orientation` with an erroring sensor-IFD read yields
`Some(v)` and an **empty** `malformed_tags`. No code changes — `sensor()`'s
existing `Orientation` handling (`SPEC-008/FU-1`, `FU-2`) already implements
this; this record pins it, narrows the field's doc comment
(`src/ifd.rs:553-569`) to state it, and closes `SPEC-008/FU-3`.

## Context

`SPEC-008/FU-3` raised this as an open, undecided question: a well-formed
`IFD0` `Orientation` with a malformed sensor-IFD entry gives the right value
and an empty `malformed_tags`, while the field's doc comment said tags
"present but shaped wrong" are recorded — and this tag *was* shaped wrong on
one of its two possible sources. Both answers were defensible and neither was
tested.

`DEC-014` changed the stakes since `FU-3` was raised. `malformed_tags` is no
longer only a report: `tools::diff()` treats a tag named in it as **exempt**
from comparison with the reference tool. So the two options are no longer
symmetric — Option A (record any errored read, even when a value was found)
would widen the oracle's blind spot for a field this reader actually has a
value for and could check.

## Alternatives Considered

- **Option A: record any errored read.** Push `TAG_ORIENTATION` whenever
  either source's read errored, even when the other source yielded a value.
  - Why rejected: it is closer to the field doc comment's literal historical
    text ("present but shaped wrong") but unsound under `DEC-014` — it would
    exempt `Orientation` from the live oracle on files where this reader's
    own value is fully trustworthy, for no reason but a byte pattern on an
    IFD whose value was never used.

- **Option B (chosen): a value found means silence.** `malformed_tags` means
  "this field's value was lost", which is what `sensor()` already computes:
  `orientation` is set from `IFD0` first, falling back to the sensor IFD only
  when `IFD0` yields nothing, and the field is recorded malformed only when
  **neither** source yields a value.
  - Why selected: zero behavior change — it is what the code already does,
    proven now by `orientation_malformed_on_both_ifds_is_costed_once` (both
    sources malformed → recorded once) and
    `a_malformed_sensor_orientation_with_a_good_ifd0_value_is_silently_dropped`
    (one source well-formed → not recorded, even though the other errored).
    It keeps `DEC-014`'s exemption sound: a tag named in `malformed_tags` is
    always a tag this reader genuinely cannot supply a value for.

## Consequences

- **Positive:** `DEC-014`'s oracle exemption stays sound for `Orientation`
  specifically — `AC4`'s test pins the exact case `FU-3` left undecided, and
  `AC3`'s test pins the sibling case (both sources malformed) that no
  existing fixture exercised before `SPEC-009`.
- **Negative:** the field's doc comment, read as it stood, promised something
  slightly broader than the code delivers ("present but shaped wrong" reads
  as *any* malformed read, not *the* value that was actually used). Fixed
  directly by narrowing the comment in the same change as this record.
- **Neutral:** this decision concerns only tags with more than one possible
  read (today, just `Orientation`'s `IFD0`-then-sensor-IFD fallback). Every
  other `malformed_tags` entry (`BlackLevel`, `ActiveArea`,
  `BlackLevelRepeatDim`, the crop tags) has exactly one source, so "value
  found means silence" and "any errored read is recorded" are the same rule
  for them — nothing about their behavior changes.

## Validation

Right if `AC3`'s and `AC4`'s tests both hold, and if `DEC-014`'s
`compare_optional` never needs a per-field exception for `Orientation`
specifically because a "recovered from the other source" case got wrongly
marked malformed. Revisit if a future field grows the same two-source
fallback shape and needs this same call made explicitly, rather than
inherited from `Orientation`'s precedent by analogy.

## References

- Related specs: `SPEC-008` (`FU-3` raised this, undecided), `SPEC-009` (this
  decision; `AC3`, `AC4`)
- Related decisions: `DEC-012` (`malformed_tags`' original contract),
  `DEC-014` (what changed the stakes — the live-oracle exemption)
- Code: `src/ifd.rs` — `Sensor::malformed_tags` doc comment (`:553-569`,
  narrowed here), `Container::sensor()`'s `Orientation` handling
  (`:1148-1181`, unchanged)
- Tests: `orientation_malformed_on_both_ifds_is_costed_once`,
  `a_malformed_sensor_orientation_with_a_good_ifd0_value_is_silently_dropped`
  (`src/ifd.rs`)
- Constraints: `oracle-must-be-shown-red`
- Raised by: `SPEC-008/FU-3`; decided by `SPEC-009` build cycle, `AC4`
