---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-053
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-11
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/opcode.rs
  - src/lib.rs
  - src/demosaic.rs
  - src/color.rs
  - src/warp.rs
  - src/develop.rs
  - tests/required_processing.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-056-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md

tags:
  - opcodes
  - dng
  - required-processing
  - api
---

# DEC-053: Refuse an unreadable OpcodeList at develop time, and guard the Sensor-taking stage primitives

## Decision

Two rulings, both extending DEC-049's required-processing contract. Neither changes
the bytes of any develop that still succeeds, so `DEVELOP_VERSION` is unchanged.

### 1. An OpcodeList that is present but unreadable refuses the develop

- **Read stays tolerant (DEC-012 unchanged).** When an `OpcodeList1/2/3` tag entry
  is present but its payload cannot be read (bad offset or length), parsing still
  succeeds: the field is `None` and the tag number is in `Sensor::malformed_tags`.
  `Container`/`Sensor` reads and metadata reporting are unaffected.
- **Develop becomes strict.** The shared required-processing check
  (`check_sensor_required_processing`, `src/opcode.rs`) refuses when any opcode-list
  tag is in `malformed_tags`, with a **new** variant
  `Error::UnreadableOpcodeList { list: u8 }`, classified `ErrorKind::Corrupt`
  (DEC-051). Lists are checked in order 1, 2, 3; for each list, the unreadable
  check comes before that list's opcode scan.
- **Why refuse:** the library cannot show that a list it cannot read holds no
  mandatory opcode. DNG's opcode rules say a reader that cannot apply a required
  opcode must not render as if it had. Not knowing is the same as not applying, so
  the outcome is no image rather than a possibly-wrong one. This is the same
  `parser-accepts-what-pipeline-cannot-honor` class that DEC-046, DEC-047 and DEC-049
  closed.

### 2. The stage primitives that take a `Sensor` run the same check

- `demosaic::demosaic_into`, `demosaic::demosaic_into_threaded` and
  `color::to_srgb_linear_into` call the check first, on the **colour path**. That
  path applies no opcode, so any mandatory opcode, or any unreadable list, refuses,
  exactly as in `develop_color_into`.
- `warp::apply_warp_into` and `develop::apply_fix_bad_pixels_constant` take **no
  `Sensor`**. They *are* the appliers of `OpcodeList3` and `OpcodeList1`, so there is
  nothing for them to check. Their docs state that the caller owns validation.
- **A public check for composers:** expose
  `opcode::check_required_processing(sensor: &Sensor, path: DevelopPath)` and make
  `DevelopPath` public with `#[non_exhaustive]`. Someone composing primitives can then
  ask the same question every `develop_*` asks. The current `pub(crate)` list-level
  helper with that name is renamed internally.

## Context

The 0.2.0 verify of SPEC-054 (HANDOFF-115) raised two follow-ups, and SPEC-042
deferred both to 0.2.1. Both are listed in the 0.2.0 CHANGELOG under "Known
limitations":

1. A malformed `OpcodeList` tag is read as absent, so a file that declares required
   processing in a list the library cannot parse develops without it.
2. The stage functions below `develop_*` skip the required-processing check.

**Measured 2026-11-11** (`irr ifd` over all 8 held tier-B files): no corpus file has
an opcode-list tag in `malformed_tags`. The only malformed tag in the corpus is the
Pentax K-3 III's `50713` (`BlackLevelRepeatDim`, DEC-012's founding case). The Bayer
files carry no opcode lists, and the Q2-Mono files carry lists 1 and 3, all readable.
So both rulings are pure hardening, and every corpus develop stays `Ok` and
byte-identical.

## Alternatives Considered

- **Reuse `UnsupportedRequiredOpcode` with a sentinel id.**
  - Rejected. The error would misstate its cause (there is no opcode id), and it
    would classify as `Unsupported` when the file is corrupt.
- **Keep tolerant, document only.**
  - Rejected. This leaves a silent wrong-image path in a library whose whole pitch
    is safety on hostile input. The CHANGELOG already called it a defect to fix.
- **Fail at parse time (make OpcodeList tags structural).**
  - Rejected. It breaks metadata reads of such files and contradicts DEC-012's
    split: a defect that changes what *exists* is fatal, while a defect that only
    changes what an interpretation field *says* costs that field. Refusing at develop
    keeps the read and refuses the claim.
- **Change `apply_warp_into` to take `&Sensor` so it can check too.**
  - Rejected for 0.2.x. It is a breaking signature change (0.3.0), and the applier
    has nothing of its own to validate.
- **Public check plus docs only, with no behaviour change to the primitives.**
  - Rejected as the only measure. A doc-only guard on a public door is the shape
    DEC-049 already ruled out for the `develop_*` variants. The public check is kept
    as an addition, not as the fix.
- **A "develop anyway" escape hatch (e.g. a `DevelopOptions` flag to skip unapplied
  or unreadable required processing).**
  - **Deferred, not rejected.** The use cases are real: forensic or salvage work on a
    damaged file, or a quick preview where a slightly wrong image beats none. Today
    they are partly served already, because parsing, metadata and the raw plane
    decode all still work. Only the claim of a *correct develop* is refused. An
    opt-in is purely additive and can land in any later minor version without
    breaking anyone. Revisit when a real consumer asks; crustyimg's sensor-forensics
    stage (its STAGE-062) is the likely first one. It must be explicit and opt-in,
    never the default.

## Consequences

- **Positive:**
  - Both 0.2.0 correctness caveats close.
  - Every public door that takes a `Sensor` and produces developed or
    camera-referred pixels now applies one rule.
  - Composers get the check as a public function.
- **Negative:**
  - A file that developed under 0.2.0, with an unreadable opcode list or a mandatory
    opcode passed to a primitive, now returns `Err`. No held file does this.
  - Two small public additions (`DevelopPath`, `check_required_processing`) to keep
    semver-stable.
- **Neutral:**
  - `Error` and `DevelopPath` are both `#[non_exhaustive]`, so the additions are
    non-breaking in 0.2.1.
  - `ErrorKind`'s wildcard-free match forces the new variant to be classified.
- **Follow-ups:**
  - The escape hatch above (deferred).
  - The colour path still does not *apply* opcodes (DEC-049; out of scope).
