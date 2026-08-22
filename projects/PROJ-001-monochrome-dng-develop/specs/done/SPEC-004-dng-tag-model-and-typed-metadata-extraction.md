---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-004
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
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
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 5991740
      estimated_usd: 39.55
      duration_minutes: 45
      recorded_at: 2026-08-21
      notes: "Verify cycle for SPEC-004 (HANDOFF-016), reviewing 37204d0 at HEAD 08fe18f on feat/spec-004-tag-model, not merged. VERDICT: APPROVED at 37204d0 - six follow-ups, NO ship-blockers. The build's deviation is correct and the narrowed scope is COMPLETE: I checked the tag list against docs/measured-q2m-dng.md line by line and every tag it records is extracted and pinned, plus StripOffsets, NewSubfileType, OpcodeList2 and BlackLevelRepeatDim beyond it; the three-stage geometry (ActiveArea -> DefaultCrop* -> Orientation) is whole. TEN GATES re-run by me, all green: build, test (52 passed - 31 lib + 0 irr bin + 9 corpus_manifest + 12 ifd_reader + 0 doc, SUMMED across five Running lines), clippy, fmt, check, deny, deny-fuzz, just msrv, lint-red-proof, lint-no-allow, fuzz (15,649,000 runs in 61 s, zero artifacts). Plus validate, cost-audit, decisions-audit --changed (names DEC-008 and DEC-012 on src/ifd.rs - DEC-012's affected_scope deferral fires as designed) and decisions-index --check. fuzz-seeds regenerates all 25 seeds BYTE-IDENTICALLY. main untouched at 00e0472. I HIT THE INVERSE OF THE HANDOFF'S COUNTING TRAP: my first test run was piped through tail -40, which cut the lib target's 31 off the TOP - the orchestrator under-counted by reading the first target, I under-counted by discarding it. THE TWO MALFORMED-TAG FIXTURES RUN BY ME through the shipped irr ifd binary, not only as assertions: same tag (Photometric forced to field type 250) on different IFDs gives sensor_ifd #1 / dimensions 4x2 / exit 0 on the thumbnail case, and exit 1 with 'no IFD matched the sensor-plane rule, and 1 candidate(s) could not be evaluated because an identifying tag was malformed: [(0, 262)]' on the only-candidate case. The error DOES name which IFD (0) and which tag (262), in the error value itself. FU-11 genuinely closed for the class it names; SensorMatch is the right shape because a Result cannot express skip-and-keep-scanning without a side channel. ORACLE RED-PROOFED BY ME: swapped top/left in the ActiveArea mapping and watched tag_model_matches_exiftool FAIL on PENTAX-K3III-MONO/K3III.DNG (26/34 vs 34/26). Worth recording - the fault is INVISIBLE on the Q2 Monochrom, whose ActiveArea is 0 0 5632 8392, so top and left are both 0 and a swap changes nothing; the Pentax's asymmetric 26/34 is the only thing in the corpus that can see a real transposition. A single-body oracle would have gone green. FUZZ RED-PROOFED AT THE NEW CODE, because reaching is not covering: planted a lint-clean split_at(usize::MAX) in scan_sensor firing only when TWO IFDs have an unreadable identifying tag - a shape no committed seed has. Negative control FIRST with the fault live: cargo test 52 passed exit 0, just lint exit 0, just lint-no-allow exit 0, so the fuzzer is the only thing that can see it. libFuzzer synthesised it (deadly signal, EXIT 77, crash-6a0da6a3cd4b48df, 314 bytes, built by mutating the thumbnail seed into a container with a SECOND unreadable IFD). Restored byte-identical (sha256 496c2baadf5814de..., grep DELIBERATE FAULT = 0, git status clean) and re-ran clean: 10,365,858 runs in 46 s, zero artifacts. SIX FOLLOW-UPS. FU-16 (highest value): sensor() STILL loses the plane to a malformed tag on a NON-SENSOR IFD, via Orientation - src/ifd.rs:1011-1017 reads TAG_ORIENTATION from IFD0 with a bare ?, so a fixture with tag 274 malformed on IFD0 gives sensor_matches [1] (the plane WAS located, the tri-state worked) and then '<none: tag 274 has unreadable field type 250>'. FU-11's exact failure shape at the site FU-11 did not name. NOT a regression - git show main:src/ifd.rs is identical, inherited from SPEC-003 - and DEC-012's TABLE sanctions it (interpret-phase, fatal to that call only), but DEC-012's PRINCIPLE ('costs that field alone' for a known-optional field) points the other way and Sensor::orientation is Option<u32>. The table and the principle disagree for optional scalar tags and the code follows the table; that is what needs deciding, not the one line. FU-17: a DNG-LEGAL RATIONAL DefaultCropSize/DefaultCropOrigin/BlackLevel makes the whole file unreadable - uints() at src/ifd.rs:788-800 rejects TYPE_RATIONAL and array()?/scalar()? propagate it out of sensor(). Built a spec-legal fixture (DefaultCropSize as RATIONAL 8368/1, 5584/1): sensor_matches [0], then '<none: tag 50720 has unreadable field type 5>'. The build flagged RATIONAL as a follow-up but framed it as a missing field; the blast radius is FATAL TO THE FILE and the message calls a spec-legal type 'unreadable'. No corpus file exercises it, which is why it is not a blocker. FU-16 and FU-17 ARE ONE SPEC, not two - both are 'a locatable plane discarded because one optional tag on some IFD could not be read'. FU-18: do NOT set .repo-context.yaml cost.metering_source to none. It is a REPO-GLOBAL switch and DEC-013's own Context lists flipping it as bad outcome #2 - 'the cost data is lost permanently, for cycles that DO have a number available' - and SPEC-003 produced four real deduped figures that this would retroactively excuse. cost-audit is silent now only because scripts/cost-audit.sh:51-57 gates SHIPPED specs; SPEC-004 will fail it at ship. And the premise is wrong anyway: I OBTAINED A REAL NUMBER FROM THE SAME TOP-LEVEL INTERACTIVE MODE the build reported as unmeterable, by reading this session's own transcript at ~/.claude/projects/<path-slug>/<session-id>.jsonl - the session id is discoverable from the scratchpad path the harness provides. The build's reasoning was honest and DEC-013-compliant (never invent a number); its premise that no source existed was simply not checked. Real fix is per-session, already half-written in the open signal token-counts-not-comparable (guidance/signals.yaml:108): add 'none' as a fourth basis and have cost-audit accept null only when that session's basis says so. FU-19: cargo test <name> matching ZERO tests exits 0 - reproduced (four targets, all '0 passed', exit 0). Every spec here names its tests and AGENTS.md 15 verify check 2 is 'failing tests from spec now pass', which a zero-match green satisfies vacuously. Make it a CHECK first and a signal second - a signal alone reproduces brag-step-skipped-at-ship exactly, a step nothing surfaces. A just failing-tests SPEC-NNN recipe reading the spec's ## Failing Tests block; it must SUM ACROSS TARGETS (malformed_tag_costs_only_that_tag is a LIB unit test at src/ifd.rs:1508, which is why a first-target read reports zero) and must run WITH the corpus, since tag_model_matches_exiftool and orientation_is_per_frame also pass vacuously on a bare runner via the skip-when-absent harness. FU-20 (minor): NoSensorIfdCandidatesMalformed can name IFDs that were never candidates - is_sensor_ifd (src/ifd.rs:916-933) reads all three identifying tags before combining, so an IFD with a READABLE NewSubfileType == 1 (a preview, definitively disqualified) still returns Unreadable if its Photometric is malformed, which is exactly the shipped thumbnail fixture's shape. Harmless, but it makes the error's own word 'candidate' untrue. FU-21 (minor): cost.totals is 0/0/session_count 0 with sessions recorded - a ship-cycle item, flagged so it is not inherited silently the way SPEC-003's FU-13 was. CHECKS THAT PASSED AND ARE WORTH STATING: no new DEC needed (SensorMatch, the infallible sensor_candidates and the new Error variant were all PRESCRIBED by FU-11's text and DEC-012's rule, not chosen - though note two BREAKING public API changes, fine at 0.1.0 under DEC-007 with no consumers and disclosed); provenance row EXTENDED not duplicated, same class 1, states no implementation was consulted; no new dependency, Cargo.toml not in the diff; design cycle has no cost session, which is correct per scripts/cost-audit.sh:12-14; DEC-012's two deferred pointer comments are both present and both accurate (src/ifd.rs:741 walk-strict, :876 interpret-tolerant), which was the one thing DEC-012 asked SPEC-004's first edit to do; AC3 holds on real files too, with active_area: None pinned on BOTH M Monochrom bodies; references.decisions is now [DEC-008, DEC-012], closing SPEC-003's FU-12. tokens_total is a transcript sum DEDUPED BY message.id and says so: 124 usage objects, 49 distinct ids, raw 15,050,048 vs deduped 5,991,740 = 2.51x, 97.0% cache-read. It is a FLOOR - computed before the session closed. This is the EIGHTH measured factor and a NEW HIGH: the band was 1.61x/1.76x/1.82x/1.86x/1.95x/2.20x/2.25x and is now 1.61x-2.51x, a 1.56x spread over eight observations, which STRENGTHENS the standing rule that no fixed correction may be applied to any raw figure, SPEC-001's 51,979,929 included. DID NOT run handback-sync, per the handoff (finding 15)."
  totals:
    tokens_total: 5991740
    estimated_usd: 39.55
    session_count: 2
shipped_at: 2026-08-21
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

**1. What would I do differently next time?**

**Read the code before writing the spec that changes it.** This spec's Context
claimed SPEC-003 "stops exactly where geometry begins." It was false: `main`
already carried `black_level`, `white_level`, `active_area`, `orientation`,
`opcode_lists` and `black_level_repeat_dim` at `src/ifd.rs:442-460`, already
satisfying AC3 and part of `DEC-012`. I grepped `pub struct Sensor`, read its first
lines, and stopped — in the same handoff that warned the builder to re-measure
anything it was about to assert.

The build read the code instead of trusting my Context, and was right to. That is
now **twice** a build has corrected my prior claims rather than inheriting them,
and both times the correct move cost one command.

There is a matching, smaller version: I checked the five named tests with a `grep`
that read only the first target's line and reported "0 passed" for four of them.
The reviewer hit the inverse — a `tail -40` that cut the lib target off the top.
Both of us mis-measured the same thing in opposite directions on the same day.

**2. Does any template, constraint, or decision need updating?**

**`DEC-012` contradicts itself, and that is the real finding (FU-16).** Its
principle says a malformedness that changes only *what an optional field says*
costs that field alone. Its *table* sanctions `sensor()` propagating a malformed
`Orientation` read from IFD0 — which loses the whole plane to a tag on a
non-sensor IFD. Reproduced: `sensor_matches [1]`, then discarded. Not a regression
— identical on `main` — but the decision and its own table disagree, and that
disagreement is what needs deciding, not the one line at `src/ifd.rs:1011`.

**FU-18 corrected me:** I asked whether `cost.metering_source: none` should be set
for this execution mode. The reviewer's answer is no, and the reasoning is better
than my question — it is **repo-global**, and `DEC-013`'s own Context names it as
the outcome the decision exists to *avoid*. The premise was also wrong: a real
number *was* obtainable from the same top-level mode by reading the session's own
transcript, so the build's `null` was avoidable rather than forced.

**3. Is there a follow-up spec to write now?**

**Yes, one — and FU-16 and FU-17 are the same spec**, as the reviewer says.
Both ask what an *unreadable* tag costs in the **extraction** path, where FU-11
answered it only for the **selection** path:

- **FU-16** — a malformed `Orientation` on IFD0 discards a located plane.
- **FU-17** — a **DNG-legal `RATIONAL`** `DefaultCropSize`/`Origin`/`BlackLevel`
  makes the whole file unreadable (`uints()` at `src/ifd.rs:788` returns
  `UnexpectedFieldType`, and `sensor()` propagates it). The build framed this as a
  missing field type; it is **fatal to the file**, which is a different severity.

`DEC-012` must be amended before that spec is designed, because today it would
sanction the behaviour the spec exists to fix.

**FU-19** deserves a **check, not a signal** — the reviewer is right that a signal
alone repeats `brag-step-skipped-at-ship`. `cargo test <name>` matching zero tests
exits **0**, so any spec naming its tests can pass vacuously. Two traps for whoever
writes it, both measured here: it must **sum across targets**, and it must run
**with the corpus**, since two of the five also pass vacuously on a bare runner.


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
