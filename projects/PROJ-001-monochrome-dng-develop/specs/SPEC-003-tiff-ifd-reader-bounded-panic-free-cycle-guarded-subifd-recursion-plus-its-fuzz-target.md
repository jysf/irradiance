---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-003
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  created_at: 2026-08-20

references:
  decisions: [DEC-008, DEC-011]    # [DEC-NNN, DEC-MMM]
  constraints:                     # [constraint-id-1, constraint-id-2]
    - no-panics-on-untrusted-input
    - provenance-recorded-per-algorithm
    - no-copyleft-dependencies
    - test-before-implementation
    - oracle-must-be-shown-red
    - library-not-application
    - no-new-top-level-deps-without-decision
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-001, SPEC-002]                # blocking order, declared at frame

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "delivers the container half of the stage thesis"

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
      interface: other
      tokens_total: 10967269
      estimated_usd: null
      duration_minutes: 75
      recorded_at: 2026-08-20
      notes: "Build cycle for SPEC-003 (HANDOFF-011), commit b79c7ef on feat/spec-003-ifd-reader, not merged. All 7 acceptance criteria met; nine gates green and pasted in the handback. The fuzz target ships in this change and BOTH directions are pasted: a planted unchecked index in Container::payload gave exit 77 plus crash artifact crash-88173bfa in under a second ('range start index 64 out of range for slice of length 26'), and the input libFuzzer reported was our own count-overflow SEED - tag 273 StripOffsets, LONG, count 0xFFFFFFFF - so the hand-built tier-A corpus caught it on the seed pass; removing the fault gave 12,992,033 runs in 61 s with zero artifacts (an earlier clean run did 14,863,561). Tag extraction matches exiftool 13.55 on all 7 corpus files, read through SPEC-002's manifest reader with no hardcoded paths. NO #[allow] of any policy lint was needed anywhere - the panic-free constraint cost nothing and improved two decisions (a single checked u64 choke point for count x sizeof(type), and packed_bits() returning bits rather than bytes so DEC-008's remainder rule stays STAGE-002's). One new decision: DEC-011 (libfuzzer-sys in a separate fuzz crate; [dependencies] still empty). TWO MEASURED CORRECTIONS to this spec's own notes: only ONE corpus file is big-endian, not two (six II, one MM - checked on the raw header bytes and with exiftool -ExifByteOrder); and K3III.PEF has NO SubIFD at all - it is the only file with a real IFD chain (IFD0->IFD1->IFD2), its plane is in IFD0, and it writes no NewSubfileType tag anywhere, which is what makes TIFF's absent-means-0 default load-bearing rather than decorative. tokens_total is a transcript sum DEDUPED BY message.id and says so: 122 usage objects, 64 distinct ids, raw 19,980,303 vs deduped 10,967,269 = 1.82x inflation, 97.0% cache-read. It is a FLOOR - written before the session closed. Consistent with the 1.7x-2.25x SPEC-002 measured; SPEC-001's totals of 51,979,929 are still raw and should be re-summed."
  totals:
    tokens_total: 10967269
    estimated_usd: 0
    session_count: 1
---

# SPEC-003: TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-003]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Every later stage reads tags, so a wrong container walk silently poisons all of
them. RAW is attacker-influenced binary (`no-panics-on-untrusted-input`), and
this is the first spec to touch it.

## Goal

A bounded, panic-free TIFF/IFD reader with SubIFD recursion, plus its fuzz
target **in the same change** (AGENTS.md §12 — a retrofitted fuzz target tests
the shape the code already has). Depth-limited, cycle-guarded on visited
offsets, every offset and length bounds-checked into a typed error.

Sensor-IFD selection keys on `NewSubfileType == 0 && Photometric == 34892 &&
SamplesPerPixel == 1` — **never on largest dimensions**; `SubIFD2` is a
full-resolution JPEG preview only 56 px narrower than the plane.

⚠ SPIKE-001 built a working version and it is **discarded** — do not consult it
as an implementation. Re-derive test-first.

## Inputs

*(Filled at build from what was actually read — 2026-08-20.)*

- **Files to read:** `guidance/toolchain-brief.md` (the two `+toolchain` traps),
  `guidance/constraints.yaml`, `AGENTS.md` §11/§12/§13,
  `docs/measured-q2m-dng.md` (the tag set), `tests/corpus/manifest.toml`,
  `tests/support/corpus.rs` (SPEC-002's reader — the only route to a corpus
  path).
- **External APIs:** none. TIFF 6.0 (1992) §2 and the Adobe DNG Specification
  1.7.1.0 are the sources, as published specifications — provenance class 1.
- **Oracle tools, run and never linked:** `exiftool 13.55` for the tag
  cross-check, `dnglab 0.7.2` for the pinned `raw_checksum` the manifest
  already holds.
- **Related code paths:** `src/lib.rs` (the crate's `Error` type and the
  panic-free lint policy).

## Outputs

*(Filled at build — 2026-08-20.)*

- **Files created:**
  - `src/ifd.rs` — the reader.
  - `tests/ifd_reader.rs` — corpus + hostile-input tests.
  - `tests/support/tiff.rs` — hand-built tier-A byte fixtures, shared by the
    test lane and the fuzz-seed writer.
  - `fuzz/Cargo.toml`, `fuzz/fuzz_targets/ifd.rs`, `fuzz/seeds/ifd/*` — the
    fuzz target and its committed seed corpus.
  - `examples/fuzz-seeds.rs` — regenerates those seeds.
  - `decisions/DEC-011-*.md` — `libfuzzer-sys` in a separate crate.
- **Files modified:** `src/lib.rs` (13 new `Error` variants, `pub mod ifd`),
  `src/bin/irr.rs` (the `ifd` subcommand), `app.just` + `AGENTS.md` §6 (the
  `fuzz` recipes, and a correction — §6 documented the invocation that does not
  work), `docs/provenance-ledger.md` (first real row), `.gitignore` (`*.PEF`),
  `CHANGELOG.md`.
- **New public API:** `ifd::Container::{parse, ifds, ifd0, payload, uints,
  scalar, required_scalar, values, is_sensor_ifd, sensor_candidates,
  sensor_ifd, sensor}`, `ifd::{Ifd, Entry, Sensor, Compression, ByteOrder}`,
  the `TAG_*` constants, and `MAX_IFD_DEPTH` / `MAX_IFDS` / `MAX_TAG_VALUES`.
- **New flags / options:** `irr ifd [--entries] <file>` — `--entries` defaults
  to **off** and adds a per-entry tag/type/count dump.
- **Database changes:** none.

## Acceptance Criteria

1. A TIFF/IFD reader walks IFD0's chain and recurses `SubIFDs` (tag 330), reading
   entry tags, types, counts and payloads.
2. **Every** offset and length read is bounds-checked and returns a **typed
   error** — no `unwrap`, no indexing, no unchecked arithmetic on any parse path
   (constraint `no-panics-on-untrusted-input`; the lint policy makes this
   mechanical, so it should be a compile-time property, not a review one).
3. **Depth-guarded and cycle-guarded.** A SubIFD chain that points at itself, or
   nests arbitrarily, terminates with an error rather than recursing forever.
4. **A fuzz target ships in this change** (§12 bar 2 — not retrofitted), seeded
   from tier-A fixtures including truncated and malformed inputs.
5. **The fuzz target is shown to WORK**, not merely to exist: a deliberately
   unchecked index, planted temporarily, must be found by libFuzzer and produce a
   crash artifact. Paste that. A fuzz target that has never caught anything is
   the "green oracle that cannot fail" in another costume.
6. On the real corpus, the reader reaches the full-resolution SubIFD and reports
   dimensions, bit depth, compression, levels, `ActiveArea`, `DefaultCrop`,
   `Orientation` and opcode-list presence, matching `exiftool` on all 7 files.
7. All nine gates stay green.

## Failing Tests

```bash
# reader reaches the sensor IFD on every corpus file that is present
cargo test --all-features ifd_reaches_sensor_plane

# hostile input: truncated header, cyclic SubIFD, absurd offsets/counts
cargo test --all-features ifd_rejects_hostile_input

# the fuzz target exists, builds, and runs
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd -- -max_total_time=60
```

**The red-proof for criterion 5** — plant an unchecked index, run the target, and
libFuzzer must produce a crash artifact under `fuzz/artifacts/`.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- **Any pixel decode or unpack.** STAGE-002, where `DEC-008`'s two-path
  (`bits % 8`) rule lands. `StripOffsets`/`StripByteCounts` are read here as
  *tags*; reading the strip they point at is not in scope. Held at build:
  `Sensor::packed_bits()` deliberately returns **bits**, not bytes, so the
  remainder question stays STAGE-002's to answer.
- **The typed tag model.** `SPEC-004`. This module widens `BYTE`/`SHORT`/`LONG`
  to `u32` and returns `Error::UnexpectedFieldType` for `RATIONAL` and the
  signed types rather than guessing.
- **A live metadata oracle.** `SPEC-005` diffs parsed tags against
  `dnglab analyze` and `exiftool` at run time. Here the `exiftool` answers are
  *pinned* as an expected table, checked by hand at build.
- **Decoding the DNG opcode streams.** Presence only (`OpcodeList1/2/3`);
  `WarpRectilinear` and `FixBadPixelsConstant` are STAGE-003.
- **Lossless JPEG (SOF-3) or Pentax PEF decompression.** Three corpus files
  need one of these and are rejected cleanly instead; PROJ-003.
- **Widening the lint exceptions.** None was needed — see the handback.

## Notes for the Implementer

### ⚠ `cargo fuzz` DOES NOT WORK with the default PATH — measured at design

This is a hard blocker and it is not obvious. `cargo fuzz` shells out to a bare
`"cargo" "build"`, and that inner `cargo` resolves to **Homebrew's stable cargo**,
which rejects `-Zsanitizer=address`:

```
error: 1 nightly option were parsed
Error: failed to build fuzz script
```

Even `~/.cargo/bin/cargo +nightly fuzz run` fails, because the *inner* invocation
is what breaks. **The fix is to put the rustup shim first on PATH:**

```bash
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run <target>
```

Verified end to end at design: `cargo fuzz init` works; a target then built and ran
**32.9 M executions in 16 s**; and a deliberately unchecked index was **found**,
producing `Error: Fuzz target exited with exit status: 77` and a crash artifact.
So criteria 4 and 5 are both known-achievable — the mechanism is proven before you
start.

### What SPIKE-001 established — as facts, not as code to copy

Its decoder is **discarded on an unmerged branch and must not be consulted as an
implementation** (`test-before-implementation`); re-derive test-first. What it
*measured* is reusable:

- Selection: `NewSubfileType == 0 && Photometric == 34892 (LinearRaw) &&
  SamplesPerPixel == 1` — **never by largest dimensions**; `SubIFD2` is a
  full-resolution JPEG preview only 56 px narrower than the plane.
- The guards needed: depth limit, cycle detection on visited offsets,
  bounds-checked payload ranges.
- ⚠ Its version used **bounds-check-then-index** (`buf.get(..)?` then `s[0]`),
  which the lint policy **rejects**. Use `try_into` on the slice. Its measured
  "229 lines" is therefore an underestimate; do not treat it as a target.

### Corpus facts that shape the tests

Seven files, `tests/corpus/manifest.toml`, read via the SPEC-002 reader — **do not
hardcode paths**. Two are big-endian (`MM`) where five are `II`. Three are
JPEG-compressed and must be **rejected cleanly**, not decoded. One (Pentax) carries
a `BlackLevelRepeatDim` tag that dnglab itself warns is malformed — a natural
regression fixture, and the reader must not panic on it.

### Scope

Container only. **No pixel decode, no unpack** — that is STAGE-002, where
`DEC-008`'s two-path (`bits % 8`) rule lands. Reading `StripOffsets`/
`StripByteCounts` as *tags* is in scope; reading the strip is not.

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
